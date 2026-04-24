// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Prova Network contributors.

// Package dashboard exposes a read-only local WebUI + JSON API that lets a
// prover operator inspect the daemon's state: current deals, stake, proofs,
// retrieval stats, and recent events.
//
// The server is disabled by default. When enabled it binds to loopback
// (127.0.0.1) unless the operator explicitly overrides ListenAddr. There
// is no auth because the dashboard is read-only and loopback-only by default.
// If you expose it to a non-local interface, put a reverse proxy with auth
// in front of it. The server never returns private keys, passphrases, or
// keystore content under any codepath.
package dashboard

import (
	"context"
	"encoding/json"
	"errors"
	"fmt"
	"io/fs"
	"log/slog"
	"net/http"
	"runtime"
	"runtime/debug"
	"strconv"
	"strings"
	"sync"
	"time"

	"github.com/ethereum/go-ethereum/common"

	"github.com/prova-network/prova/prover/pkg/deal"
)

// BuildVersion is stamped at link time via -ldflags. Defaulted for dev runs.
var BuildVersion = "dev"

// ChainSnapshot is the subset of on-chain state the dashboard surfaces.
// All fields are optional; the dashboard degrades gracefully if the source
// is unavailable (e.g., RPC down).
type ChainSnapshot struct {
	ChainID          uint64 `json:"chain_id"`
	BlockNumber      uint64 `json:"block_number"`
	LastSeenBlock    uint64 `json:"last_seen_block"`
	ProverRegistered bool   `json:"prover_registered"`
	ProverActive     bool   `json:"prover_active"`
	ProverAddress    string `json:"prover_address"`
	StakedWei        string `json:"staked_wei"`
	UnbondingWei     string `json:"unbonding_wei"`
	UnbondingEndsAt  int64  `json:"unbonding_ends_at_unix"`
	CommittedBytes   uint64 `json:"committed_bytes"`
	Reputation       uint32 `json:"reputation"`
}

// ChainReader is the minimal view the dashboard asks of the on-chain client.
// Implementations may return zero-valued fields when a particular piece of
// state is unreachable. Never return a private key / sensitive field through
// this interface; there is no auth in front of the dashboard by default.
type ChainReader interface {
	Snapshot(ctx context.Context) (ChainSnapshot, error)
}

// MetricsReader exposes a lightweight summary of Prometheus metrics for the
// overview page. For full-fidelity metrics the operator should scrape the
// dedicated /metrics endpoint; this is only a human-friendly digest.
type MetricsReader interface {
	Summary() MetricsSummary
}

// MetricsSummary is the compact metric digest rendered on the overview page.
type MetricsSummary struct {
	DealsActive            uint64  `json:"deals_active"`
	DealsTotal             uint64  `json:"deals_total"`
	ProofsSubmittedTotal   uint64  `json:"proofs_submitted_total"`
	ProofsFailedTotal      uint64  `json:"proofs_failed_total"`
	RetrievalBytesTotal    uint64  `json:"retrieval_bytes_total"`
	PieceStoreBytes        uint64  `json:"piece_store_bytes"`
	ChainRPCErrorsTotal    uint64  `json:"chain_rpc_errors_total"`
	ProofSuccessRatePct    float64 `json:"proof_success_rate_pct"`
	UptimeSeconds          int64   `json:"uptime_seconds"`
}

// Options configure the dashboard server.
type Options struct {
	// Enabled turns the dashboard on. Default: false.
	Enabled bool

	// ListenAddr binds the server. Default: "127.0.0.1:8081".
	// If set to a non-loopback address, the operator MUST front this with
	// a reverse proxy that adds authentication.
	ListenAddr string

	// Store is the deal store the dashboard reads from.
	Store deal.Store

	// Chain is the optional on-chain state reader.
	Chain ChainReader

	// Metrics is the optional metrics summarizer.
	Metrics MetricsReader

	// Logger is used for access + error logging.
	Logger *slog.Logger
}

// Server is the dashboard HTTP server.
type Server struct {
	opts      Options
	startedAt time.Time
	httpSrv   *http.Server
	mu        sync.RWMutex
}

// New constructs a Server but does not start it.
func New(opts Options) (*Server, error) {
	if !opts.Enabled {
		return &Server{opts: opts, startedAt: time.Now().UTC()}, nil
	}
	if opts.Store == nil {
		return nil, errors.New("dashboard: Store is required when Enabled")
	}
	if opts.ListenAddr == "" {
		opts.ListenAddr = "127.0.0.1:8081"
	}
	if opts.Logger == nil {
		opts.Logger = slog.Default()
	}
	return &Server{opts: opts, startedAt: time.Now().UTC()}, nil
}

// Handler returns the configured http.Handler for embedding/testing.
func (s *Server) Handler() http.Handler {
	mux := http.NewServeMux()

	// JSON API.
	mux.HandleFunc("/api/overview", s.handleOverview)
	mux.HandleFunc("/api/deals", s.handleDeals)
	mux.HandleFunc("/api/deals/", s.handleDealByID)
	mux.HandleFunc("/api/chain", s.handleChain)
	mux.HandleFunc("/api/metrics", s.handleMetrics)
	mux.HandleFunc("/api/info", s.handleInfo)

	// Static SPA (embedded).
	staticFS, err := fs.Sub(distFS, "dist")
	if err == nil {
		mux.Handle("/", spaHandler(staticFS))
	} else {
		// Fallback: serve a tiny placeholder HTML so the server still works
		// without a built frontend (useful during `go test` and dev-loop).
		mux.HandleFunc("/", s.handlePlaceholder)
	}

	return s.corsMiddleware(s.logMiddleware(mux))
}

// Start runs the server until ctx is cancelled.
func (s *Server) Start(ctx context.Context) error {
	if !s.opts.Enabled {
		return nil
	}
	s.httpSrv = &http.Server{
		Addr:              s.opts.ListenAddr,
		Handler:           s.Handler(),
		ReadHeaderTimeout: 5 * time.Second,
	}
	errCh := make(chan error, 1)
	go func() {
		s.opts.Logger.Info("dashboard: listening", "addr", s.opts.ListenAddr)
		if err := s.httpSrv.ListenAndServe(); err != nil && !errors.Is(err, http.ErrServerClosed) {
			errCh <- err
		}
		close(errCh)
	}()
	select {
	case <-ctx.Done():
		shutCtx, cancel := context.WithTimeout(context.Background(), 3*time.Second)
		defer cancel()
		_ = s.httpSrv.Shutdown(shutCtx)
		return nil
	case err := <-errCh:
		return err
	}
}

// ───── Handlers ─────────────────────────────────────────────────────────

func (s *Server) handleOverview(w http.ResponseWriter, r *http.Request) {
	resp := map[string]any{
		"version":    BuildVersion,
		"started_at": s.startedAt.Format(time.RFC3339),
		"uptime":     time.Since(s.startedAt).Round(time.Second).String(),
	}
	if s.opts.Chain != nil {
		snap, err := s.opts.Chain.Snapshot(r.Context())
		if err == nil {
			resp["chain"] = snap
		} else {
			resp["chain_error"] = err.Error()
		}
	}
	if s.opts.Metrics != nil {
		resp["metrics"] = s.opts.Metrics.Summary()
	}
	// Counts by status, all-time.
	all, err := s.opts.Store.ListAll()
	if err == nil {
		counts := map[deal.Status]int{}
		for _, d := range all {
			counts[d.Status]++
		}
		resp["deal_counts"] = counts
		resp["deals_total"] = len(all)
	}
	writeJSON(w, http.StatusOK, resp)
}

func (s *Server) handleDeals(w http.ResponseWriter, r *http.Request) {
	q := r.URL.Query()

	var ds []*deal.Deal
	var err error
	if status := q.Get("status"); status != "" {
		ds, err = s.opts.Store.ListByStatus(deal.Status(status))
	} else {
		ds, err = s.opts.Store.ListAll()
	}
	if err != nil {
		writeError(w, http.StatusInternalServerError, err)
		return
	}

	// Optional pagination.
	limit := parseUint(q.Get("limit"), 100)
	if limit > 500 {
		limit = 500
	}
	if len(ds) > int(limit) {
		ds = ds[:limit]
	}

	out := make([]dealView, 0, len(ds))
	for _, d := range ds {
		out = append(out, toDealView(d))
	}
	writeJSON(w, http.StatusOK, map[string]any{
		"deals": out,
		"count": len(out),
	})
}

func (s *Server) handleDealByID(w http.ResponseWriter, r *http.Request) {
	idStr := strings.TrimPrefix(r.URL.Path, "/api/deals/")
	if idStr == "" {
		writeError(w, http.StatusBadRequest, errors.New("missing deal id"))
		return
	}
	id, err := strconv.ParseUint(idStr, 10, 64)
	if err != nil {
		writeError(w, http.StatusBadRequest, fmt.Errorf("invalid deal id: %w", err))
		return
	}
	d, err := s.opts.Store.Get(deal.DealID(id))
	if err != nil {
		writeError(w, http.StatusNotFound, err)
		return
	}
	if d == nil {
		writeError(w, http.StatusNotFound, fmt.Errorf("deal %d not found", id))
		return
	}
	writeJSON(w, http.StatusOK, toDealView(d))
}

func (s *Server) handleChain(w http.ResponseWriter, r *http.Request) {
	if s.opts.Chain == nil {
		writeError(w, http.StatusServiceUnavailable, errors.New("chain reader not configured"))
		return
	}
	snap, err := s.opts.Chain.Snapshot(r.Context())
	if err != nil {
		writeError(w, http.StatusBadGateway, err)
		return
	}
	writeJSON(w, http.StatusOK, snap)
}

func (s *Server) handleMetrics(w http.ResponseWriter, r *http.Request) {
	if s.opts.Metrics == nil {
		writeError(w, http.StatusServiceUnavailable, errors.New("metrics reader not configured"))
		return
	}
	writeJSON(w, http.StatusOK, s.opts.Metrics.Summary())
}

func (s *Server) handleInfo(w http.ResponseWriter, r *http.Request) {
	bi, _ := debug.ReadBuildInfo()
	resp := map[string]any{
		"version":    BuildVersion,
		"started_at": s.startedAt.Format(time.RFC3339),
		"go_version": runtime.Version(),
		"goos":       runtime.GOOS,
		"goarch":     runtime.GOARCH,
	}
	if bi != nil {
		resp["go_module"] = bi.Main.Path
	}
	writeJSON(w, http.StatusOK, resp)
}

// Placeholder HTML when the built SPA isn't embedded (e.g., during tests
// or when building without the dashboard-ui target). Gives operators a
// useful landing page that points them at the JSON API and metrics.
func (s *Server) handlePlaceholder(w http.ResponseWriter, r *http.Request) {
	if r.URL.Path != "/" && r.URL.Path != "" {
		http.NotFound(w, r)
		return
	}
	w.Header().Set("Content-Type", "text/html; charset=utf-8")
	fmt.Fprintf(w, `<!doctype html>
<html lang="en">
<head>
  <meta charset="utf-8">
  <title>Prova Prover Dashboard</title>
  <style>
    body { font: 16px/1.5 ui-sans-serif, system-ui, sans-serif; max-width: 720px; margin: 40px auto; padding: 0 16px; color: #1a1817; background: #FAFAF9; }
    h1 { border-bottom: 2px solid #C9A84C; padding-bottom: 8px; }
    code { background: #eee; padding: 2px 5px; border-radius: 3px; font-size: 0.9em; }
    .ok { color: #2c5c2e; } .warn { color: #b84918; }
    a { color: #0052FF; }
  </style>
</head>
<body>
  <h1>Prova Prover <span style="color:#C9A84C">●</span></h1>
  <p>The prover is running but the WebUI SPA is not embedded in this build.</p>
  <p>Raw JSON API is live:</p>
  <ul>
    <li><a href="/api/overview"><code>/api/overview</code></a>, high-level snapshot</li>
    <li><a href="/api/deals"><code>/api/deals</code></a>, all deals</li>
    <li><a href="/api/chain"><code>/api/chain</code></a>, on-chain state</li>
    <li><a href="/api/metrics"><code>/api/metrics</code></a>, metrics summary</li>
    <li><a href="/api/info"><code>/api/info</code></a>, build info</li>
  </ul>
  <p>To build the full dashboard UI: <code>cd prover/webui && npm ci && npm run build</code> then rebuild the daemon.</p>
  <p><small>provad %s, %s</small></p>
</body>
</html>
`, BuildVersion, time.Now().UTC().Format(time.RFC3339))
}

// ───── Helpers ──────────────────────────────────────────────────────────

type dealView struct {
	ID            uint64    `json:"id"`
	Status        string    `json:"status"`
	StatusMessage string    `json:"status_message,omitempty"`
	Client        string    `json:"client"`
	PieceSize     uint64    `json:"piece_size"`
	CommPHex      string    `json:"commp_hex"`
	SourceURL     string    `json:"source_url,omitempty"`
	TotalPayment  string    `json:"total_payment_wei,omitempty"`
	Duration      string    `json:"duration,omitempty"`
	DataSetID     uint64    `json:"data_set_id,omitempty"`
	BytesStored   uint64    `json:"bytes_stored,omitempty"`
	AcceptedAt    time.Time `json:"accepted_at,omitempty"`
	UpdatedAt     time.Time `json:"updated_at"`
}

func toDealView(d *deal.Deal) dealView {
	v := dealView{
		ID:            uint64(d.ID),
		Status:        string(d.Status),
		StatusMessage: d.StatusMsg,
		Client:        common.Address(d.Client).Hex(),
		PieceSize:     d.PieceSize,
		CommPHex:      fmt.Sprintf("0x%x", d.CommPHash),
		SourceURL:     d.SourceURL,
		TotalPayment:  d.TotalPayment,
		DataSetID:     d.DataSetID,
		BytesStored:   d.BytesStored,
		UpdatedAt:     d.UpdatedAt,
	}
	if d.Duration > 0 {
		v.Duration = d.Duration.String()
	}
	if !d.AcceptedAt.IsZero() {
		v.AcceptedAt = d.AcceptedAt
	}
	return v
}

func writeJSON(w http.ResponseWriter, code int, payload any) {
	w.Header().Set("Content-Type", "application/json; charset=utf-8")
	w.Header().Set("Cache-Control", "no-store")
	w.WriteHeader(code)
	_ = json.NewEncoder(w).Encode(payload)
}

func writeError(w http.ResponseWriter, code int, err error) {
	writeJSON(w, code, map[string]string{"error": err.Error()})
}

func parseUint(s string, dflt uint64) uint64 {
	if s == "" {
		return dflt
	}
	n, err := strconv.ParseUint(s, 10, 64)
	if err != nil {
		return dflt
	}
	return n
}

func (s *Server) logMiddleware(next http.Handler) http.Handler {
	return http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		start := time.Now()
		next.ServeHTTP(w, r)
		s.opts.Logger.Debug("dashboard",
			"method", r.Method,
			"path", r.URL.Path,
			"remote", r.RemoteAddr,
			"dur_ms", time.Since(start).Milliseconds(),
		)
	})
}

// corsMiddleware allows the dashboard to be consumed by a dev-server
// (e.g., `vite dev` on :5173) during development. Production builds embed
// the SPA and don't need CORS, but leaving it permissive on loopback is
// safe because loopback is the default bind.
func (s *Server) corsMiddleware(next http.Handler) http.Handler {
	return http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		origin := r.Header.Get("Origin")
		if origin != "" && (strings.HasPrefix(origin, "http://localhost:") ||
			strings.HasPrefix(origin, "http://127.0.0.1:")) {
			w.Header().Set("Access-Control-Allow-Origin", origin)
			w.Header().Set("Access-Control-Allow-Methods", "GET, OPTIONS")
			w.Header().Set("Access-Control-Allow-Headers", "Content-Type")
		}
		if r.Method == http.MethodOptions {
			w.WriteHeader(http.StatusNoContent)
			return
		}
		next.ServeHTTP(w, r)
	})
}

// spaHandler serves the built SPA. Unknown paths fall back to /index.html
// so React Router (or similar) can own client-side routing.
func spaHandler(root fs.FS) http.Handler {
	fileServer := http.FileServer(http.FS(root))
	return http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		// Try the file first. If it's missing, fall back to index.html.
		if r.URL.Path != "/" {
			trimmed := strings.TrimPrefix(r.URL.Path, "/")
			if _, err := fs.Stat(root, trimmed); err != nil {
				r.URL.Path = "/"
			}
		}
		fileServer.ServeHTTP(w, r)
	})
}
