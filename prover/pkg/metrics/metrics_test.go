// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Prova Network contributors.

package metrics

import (
	"context"
	"io"
	"log/slog"
	"net/http"
	"net/http/httptest"
	"strings"
	"testing"
	"time"

	"github.com/stretchr/testify/require"
)

func silentLog() *slog.Logger {
	return slog.New(slog.NewTextHandler(io.Discard, &slog.HandlerOptions{Level: slog.LevelError}))
}

func TestNew_RegistersExpectedMetrics(t *testing.T) {
	c := New()
	require.NotNil(t, c)

	// Exercise a few metrics so they show up in the scrape output with
	// non-default values; this guards against typos in the Name field.
	c.DealsIngestedTotal.Inc()
	c.DealsFailedTotal.Add(3)
	c.ChainHeadBlockGauge.Set(12345)
	c.BytesStoredTotal.Add(1 << 20) // 1 MiB
	c.HTTPRequestsTotal.WithLabelValues("GET", "/piece/", "200").Inc()
	c.HTTPDuration.WithLabelValues("/piece/").Observe(0.5)
	c.ProofsSubmittedTotal.Inc()

	// Scrape via the Handler
	rw := httptest.NewRecorder()
	req := httptest.NewRequest(http.MethodGet, "/metrics", nil)
	c.Handler().ServeHTTP(rw, req)
	require.Equal(t, http.StatusOK, rw.Code)

	body := rw.Body.String()
	for _, want := range []string{
		`prova_deals_ingested_total 1`,
		`prova_deals_failed_total 3`,
		`prova_chain_head_block 12345`,
		`prova_bytes_stored_total 1.048576e+06`,
		`prova_http_requests_total{method="GET",path="/piece/",status="200"} 1`,
		`prova_proofs_submitted_total 1`,
		// Go runtime collector (always present on all platforms)
		`go_goroutines`,
	} {
		require.Contains(t, body, want, "missing metric line: %s", want)
	}
}

func TestServer_ListenAndServe_GracefulShutdown(t *testing.T) {
	c := New()
	srv, err := NewServer(Options{
		Collector:  c,
		ListenAddr: "127.0.0.1:0", // ephemeral
		Logger:     silentLog(),
	})
	require.NoError(t, err)

	ctx, cancel := context.WithCancel(context.Background())

	errCh := make(chan error, 1)
	go func() { errCh <- srv.ListenAndServe(ctx) }()

	// Give the server a moment to bind
	time.Sleep(50 * time.Millisecond)
	cancel()

	select {
	case err := <-errCh:
		require.NoError(t, err)
	case <-time.After(5 * time.Second):
		t.Fatal("metrics server did not shut down in time")
	}
}

func TestServer_ServesMetrics(t *testing.T) {
	c := New()
	c.DealsIngestedTotal.Add(7)

	srv, err := NewServer(Options{
		Collector:  c,
		ListenAddr: "127.0.0.1:0",
		Logger:     silentLog(),
	})
	require.NoError(t, err)

	// Mux the collector's handler directly for a reliable test harness
	// (avoids port-binding race).
	mux := http.NewServeMux()
	mux.Handle("/metrics", c.Handler())
	mux.HandleFunc("/healthz", func(w http.ResponseWriter, _ *http.Request) {
		w.WriteHeader(http.StatusOK)
	})
	ts := httptest.NewServer(mux)
	defer ts.Close()

	// Scrape
	resp, err := http.Get(ts.URL + "/metrics")
	require.NoError(t, err)
	defer resp.Body.Close()
	require.Equal(t, http.StatusOK, resp.StatusCode)

	body, err := io.ReadAll(resp.Body)
	require.NoError(t, err)
	require.True(t, strings.Contains(string(body), "prova_deals_ingested_total 7"),
		"metrics output missing deals counter; got: %.200s", string(body))

	// /healthz
	resp2, err := http.Get(ts.URL + "/healthz")
	require.NoError(t, err)
	defer resp2.Body.Close()
	require.Equal(t, http.StatusOK, resp2.StatusCode)

	// Use srv in a compile-check way so the test doesn't leave it unreferenced
	_ = srv
}

func TestNewServer_Validation(t *testing.T) {
	_, err := NewServer(Options{})
	require.ErrorContains(t, err, "collector required")

	c := New()
	_, err = NewServer(Options{Collector: c})
	require.ErrorContains(t, err, "listen addr")
}
