// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Prova Network contributors.

package metrics

import (
	"context"
	"errors"
	"fmt"
	"log/slog"
	"net/http"
	"time"
)

// Server is a small HTTP listener that exposes /metrics and /health.
// Kept separate from the retrieval HTTP server so operators can bind it
// to localhost or an internal network without exposing it alongside
// piece retrieval.
type Server struct {
	collector *Collector
	address   string
	logger    *slog.Logger
}

// Options configures the metrics server.
type Options struct {
	Collector  *Collector
	ListenAddr string
	Logger     *slog.Logger
}

// NewServer constructs a metrics Server.
func NewServer(opts Options) (*Server, error) {
	if opts.Collector == nil {
		return nil, errors.New("collector required")
	}
	if opts.ListenAddr == "" {
		return nil, errors.New("listen addr required")
	}
	if opts.Logger == nil {
		opts.Logger = slog.Default()
	}
	return &Server{
		collector: opts.Collector,
		address:   opts.ListenAddr,
		logger:    opts.Logger,
	}, nil
}

// ListenAndServe blocks until ctx is cancelled or the server errors.
func (s *Server) ListenAndServe(ctx context.Context) error {
	mux := http.NewServeMux()
	mux.Handle("/metrics", s.collector.Handler())
	mux.HandleFunc("/healthz", func(w http.ResponseWriter, _ *http.Request) {
		w.WriteHeader(http.StatusOK)
		_, _ = w.Write([]byte("ok\n"))
	})

	srv := &http.Server{
		Addr:         s.address,
		Handler:      mux,
		ReadTimeout:  10 * time.Second,
		WriteTimeout: 30 * time.Second,
	}

	s.logger.Info("metrics server starting", "addr", s.address)

	errCh := make(chan error, 1)
	go func() { errCh <- srv.ListenAndServe() }()

	select {
	case <-ctx.Done():
		s.logger.Info("metrics server shutting down")
		shutdownCtx, cancel := context.WithTimeout(context.Background(), 10*time.Second)
		defer cancel()
		return srv.Shutdown(shutdownCtx)
	case err := <-errCh:
		if errors.Is(err, http.ErrServerClosed) {
			return nil
		}
		return fmt.Errorf("metrics server: %w", err)
	}
}
