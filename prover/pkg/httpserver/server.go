// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Prova Network contributors.

// Package httpserver exposes stored pieces over HTTP(S) so clients can
// retrieve the content they've stored on this prover.
//
// Routes:
//
//	GET /health              — liveness probe, always 200
//	GET /piece/{pieceCid}    — stream the stored piece bytes
//	HEAD /piece/{pieceCid}   — return Content-Length without body
//	GET /.well-known/prova   — small JSON with prover metadata
//
// The server is deliberately minimal: no auth yet, no rate limiting, no
// range requests. Those land in Phase F.2 with the rest of the production
// polish. The point of this phase is "clients can fetch content back".
package httpserver

import (
	"context"
	"encoding/json"
	"errors"
	"fmt"
	"io"
	"log/slog"
	"net"
	"net/http"
	"strconv"
	"strings"
	"time"

	"github.com/ipfs/go-cid"

	"github.com/prova-network/prova/prover/pkg/store"
)

// Server exposes piece retrieval over HTTP(S).
type Server struct {
	pieces   store.Store
	address  string
	publicURL string
	cert     string
	key      string
	logger   *slog.Logger

	httpSrv *http.Server
}

// Options configures the server.
type Options struct {
	// Pieces is the blob store to serve from.
	Pieces store.Store

	// ListenAddr is the TCP bind address, e.g. ":8443" or "127.0.0.1:8080".
	ListenAddr string

	// PublicURL is the URL the server advertises at /.well-known/prova.
	// Used by clients to verify they've reached the right prover.
	PublicURL string

	// CertPath and KeyPath configure TLS. If both empty, the server runs
	// plain HTTP (intended for local testing behind a reverse proxy).
	CertPath string
	KeyPath  string

	// ReadTimeout caps how long a single request is permitted to take.
	// Default: 5 minutes (large pieces take time to stream).
	ReadTimeout time.Duration

	// Logger is optional; defaults to slog.Default().
	Logger *slog.Logger
}

// New constructs a Server.
func New(opts Options) (*Server, error) {
	if opts.Pieces == nil {
		return nil, errors.New("pieces store required")
	}
	if opts.ListenAddr == "" {
		return nil, errors.New("listen addr required")
	}
	if opts.ReadTimeout == 0 {
		opts.ReadTimeout = 5 * time.Minute
	}
	if opts.Logger == nil {
		opts.Logger = slog.Default()
	}
	return &Server{
		pieces:    opts.Pieces,
		address:   opts.ListenAddr,
		publicURL: opts.PublicURL,
		cert:      opts.CertPath,
		key:       opts.KeyPath,
		logger:    opts.Logger,
	}, nil
}

// Handler returns the HTTP handler for testing or embedding.
func (s *Server) Handler() http.Handler {
	mux := http.NewServeMux()
	mux.HandleFunc("/health", s.handleHealth)
	mux.HandleFunc("/.well-known/prova", s.handleWellKnown)
	mux.HandleFunc("/piece/", s.handlePiece)
	return logMiddleware(s.logger, mux)
}

// ListenAndServe blocks until the context is cancelled or the server
// errors. Cancelling ctx gracefully shuts down in-flight requests up to
// a bounded deadline.
func (s *Server) ListenAndServe(ctx context.Context) error {
	s.httpSrv = &http.Server{
		Addr:         s.address,
		Handler:      s.Handler(),
		ReadTimeout:  30 * time.Second,
		WriteTimeout: 5 * time.Minute,
		IdleTimeout:  60 * time.Second,
	}

	s.logger.Info("http server starting",
		"addr", s.address,
		"tls", s.cert != "" && s.key != "",
		"publicURL", s.publicURL,
	)

	errCh := make(chan error, 1)
	go func() {
		if s.cert != "" && s.key != "" {
			errCh <- s.httpSrv.ListenAndServeTLS(s.cert, s.key)
		} else {
			errCh <- s.httpSrv.ListenAndServe()
		}
	}()

	select {
	case <-ctx.Done():
		s.logger.Info("http server shutting down")
		shutdownCtx, cancel := context.WithTimeout(context.Background(), 30*time.Second)
		defer cancel()
		return s.httpSrv.Shutdown(shutdownCtx)
	case err := <-errCh:
		if errors.Is(err, http.ErrServerClosed) {
			return nil
		}
		return fmt.Errorf("http server: %w", err)
	}
}

// ───── Handlers ───────────────────────────────────────────────────────

func (s *Server) handleHealth(w http.ResponseWriter, _ *http.Request) {
	w.Header().Set("Content-Type", "text/plain")
	w.WriteHeader(http.StatusOK)
	_, _ = io.WriteString(w, "ok\n")
}

func (s *Server) handleWellKnown(w http.ResponseWriter, _ *http.Request) {
	info := map[string]any{
		"service":    "prova-prover",
		"version":    "0.1.0-pre",
		"publicURL":  s.publicURL,
		"features": []string{
			"pdp",
			"https-retrieval",
		},
	}
	w.Header().Set("Content-Type", "application/json")
	w.Header().Set("Cache-Control", "no-cache")
	_ = json.NewEncoder(w).Encode(info)
}

// handlePiece dispatches to the piece-retrieval handler. Matches
// /piece/{pieceCid} exactly and any HEAD variant.
func (s *Server) handlePiece(w http.ResponseWriter, r *http.Request) {
	pieceStr := strings.TrimPrefix(r.URL.Path, "/piece/")
	if pieceStr == "" || strings.Contains(pieceStr, "/") {
		http.Error(w, "piece cid required in path", http.StatusBadRequest)
		return
	}

	pieceCid, err := cid.Decode(pieceStr)
	if err != nil {
		http.Error(w, fmt.Sprintf("invalid piece cid: %v", err), http.StatusBadRequest)
		return
	}

	size, err := s.pieces.Size(pieceCid)
	if err != nil {
		if errors.Is(err, store.ErrNotFound) {
			http.Error(w, "piece not found", http.StatusNotFound)
			return
		}
		http.Error(w, "size lookup failed", http.StatusInternalServerError)
		return
	}

	w.Header().Set("Content-Type", "application/octet-stream")
	w.Header().Set("Content-Length", strconv.FormatUint(size, 10))
	w.Header().Set("Content-Disposition", fmt.Sprintf(`attachment; filename="%s"`, pieceStr))
	w.Header().Set("X-Prova-Piece-Size", strconv.FormatUint(size, 10))

	if r.Method == http.MethodHead {
		w.WriteHeader(http.StatusOK)
		return
	}

	rd, err := s.pieces.Get(pieceCid)
	if err != nil {
		if errors.Is(err, store.ErrNotFound) {
			http.Error(w, "piece not found", http.StatusNotFound)
			return
		}
		http.Error(w, "open piece failed", http.StatusInternalServerError)
		return
	}
	defer rd.Close()

	if _, err := io.Copy(w, rd); err != nil {
		// Client disconnect is normal; log at debug only.
		s.logger.Debug("piece stream ended", "piece", pieceStr, "err", err)
	}
}

// ───── Middleware ─────────────────────────────────────────────────────

// logMiddleware emits a structured access log per request.
func logMiddleware(logger *slog.Logger, next http.Handler) http.Handler {
	return http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		start := time.Now()
		rw := &statusCapturingWriter{ResponseWriter: w, status: 200}
		next.ServeHTTP(rw, r)
		logger.Info("http",
			"method", r.Method,
			"path", r.URL.Path,
			"status", rw.status,
			"bytes", rw.bytes,
			"remote", clientIP(r),
			"duration", time.Since(start).Round(time.Millisecond).String(),
		)
	})
}

// statusCapturingWriter records the HTTP status and byte count so we can
// include them in the access log.
type statusCapturingWriter struct {
	http.ResponseWriter
	status int
	bytes  int
}

func (w *statusCapturingWriter) WriteHeader(status int) {
	w.status = status
	w.ResponseWriter.WriteHeader(status)
}

func (w *statusCapturingWriter) Write(b []byte) (int, error) {
	n, err := w.ResponseWriter.Write(b)
	w.bytes += n
	return n, err
}

// clientIP best-effort extracts the caller's IP address.
func clientIP(r *http.Request) string {
	if f := r.Header.Get("X-Forwarded-For"); f != "" {
		if comma := strings.Index(f, ","); comma >= 0 {
			return strings.TrimSpace(f[:comma])
		}
		return strings.TrimSpace(f)
	}
	host, _, err := net.SplitHostPort(r.RemoteAddr)
	if err != nil {
		return r.RemoteAddr
	}
	return host
}

