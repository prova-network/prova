// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Prova Network contributors.

package httpserver

import (
	"bytes"
	"context"
	"encoding/json"
	"io"
	"log/slog"
	"net/http"
	"net/http/httptest"
	"testing"
	"time"

	"github.com/ipfs/go-cid"
	"github.com/multiformats/go-multihash"
	"github.com/stretchr/testify/require"

	"github.com/prova-network/prova/prover/pkg/store"
)

func silentLog() *slog.Logger {
	return slog.New(slog.NewTextHandler(io.Discard, &slog.HandlerOptions{Level: slog.LevelError}))
}

// testCID constructs a plausible CID for a given payload so we can
// round-trip through the /piece/{cid} endpoint without needing a real
// CommP computation (which pkg/pdptree handles elsewhere).
func testCID(t *testing.T, content string) cid.Cid {
	t.Helper()
	mh, err := multihash.Sum([]byte(content), multihash.SHA2_256, -1)
	require.NoError(t, err)
	return cid.NewCidV1(cid.Raw, mh)
}

// newTestServer returns a configured Server backed by an in-memory piece
// store (actually DiskStore in tmpdir — simpler than wiring a memstore).
func newTestServer(t *testing.T) (*Server, store.Store) {
	t.Helper()
	tmp := t.TempDir()
	ps, err := store.NewDiskStore(tmp)
	require.NoError(t, err)
	t.Cleanup(func() { _ = ps.Close() })

	s, err := New(Options{
		Pieces:     ps,
		ListenAddr: ":0",
		PublicURL:  "https://prover.example",
		Logger:     silentLog(),
	})
	require.NoError(t, err)
	return s, ps
}

func TestHandleHealth(t *testing.T) {
	s, _ := newTestServer(t)
	req := httptest.NewRequest(http.MethodGet, "/health", nil)
	rw := httptest.NewRecorder()
	s.Handler().ServeHTTP(rw, req)
	require.Equal(t, http.StatusOK, rw.Code)
	require.Contains(t, rw.Body.String(), "ok")
}

func TestHandleWellKnown(t *testing.T) {
	s, _ := newTestServer(t)
	req := httptest.NewRequest(http.MethodGet, "/.well-known/prova", nil)
	rw := httptest.NewRecorder()
	s.Handler().ServeHTTP(rw, req)
	require.Equal(t, http.StatusOK, rw.Code)
	var info map[string]any
	require.NoError(t, json.Unmarshal(rw.Body.Bytes(), &info))
	require.Equal(t, "prova-prover", info["service"])
	require.Equal(t, "https://prover.example", info["publicURL"])
}

func TestHandlePiece_ReturnsStoredBytes(t *testing.T) {
	s, ps := newTestServer(t)
	payload := []byte("hello from the prover")
	c := testCID(t, string(payload))

	_, err := ps.Put(c, bytes.NewReader(payload))
	require.NoError(t, err)

	req := httptest.NewRequest(http.MethodGet, "/piece/"+c.String(), nil)
	rw := httptest.NewRecorder()
	s.Handler().ServeHTTP(rw, req)

	require.Equal(t, http.StatusOK, rw.Code)
	require.Equal(t, "application/octet-stream", rw.Header().Get("Content-Type"))
	require.Equal(t, "21", rw.Header().Get("Content-Length"))
	require.Equal(t, payload, rw.Body.Bytes())
}

func TestHandlePiece_Head(t *testing.T) {
	s, ps := newTestServer(t)
	payload := []byte("small piece")
	c := testCID(t, string(payload))
	_, err := ps.Put(c, bytes.NewReader(payload))
	require.NoError(t, err)

	req := httptest.NewRequest(http.MethodHead, "/piece/"+c.String(), nil)
	rw := httptest.NewRecorder()
	s.Handler().ServeHTTP(rw, req)

	require.Equal(t, http.StatusOK, rw.Code)
	require.Equal(t, "11", rw.Header().Get("Content-Length"))
	require.Empty(t, rw.Body.Bytes(), "HEAD must not return body")
}

func TestHandlePiece_NotFound(t *testing.T) {
	s, _ := newTestServer(t)
	c := testCID(t, "never stored")
	req := httptest.NewRequest(http.MethodGet, "/piece/"+c.String(), nil)
	rw := httptest.NewRecorder()
	s.Handler().ServeHTTP(rw, req)
	require.Equal(t, http.StatusNotFound, rw.Code)
}

func TestHandlePiece_InvalidCid(t *testing.T) {
	s, _ := newTestServer(t)
	req := httptest.NewRequest(http.MethodGet, "/piece/not-a-valid-cid", nil)
	rw := httptest.NewRecorder()
	s.Handler().ServeHTTP(rw, req)
	require.Equal(t, http.StatusBadRequest, rw.Code)
	require.Contains(t, rw.Body.String(), "invalid piece cid")
}

func TestHandlePiece_EmptyPath(t *testing.T) {
	s, _ := newTestServer(t)
	req := httptest.NewRequest(http.MethodGet, "/piece/", nil)
	rw := httptest.NewRecorder()
	s.Handler().ServeHTTP(rw, req)
	require.Equal(t, http.StatusBadRequest, rw.Code)
}

// TestListenAndServe_GracefulShutdown verifies the server stops cleanly
// when its context is cancelled.
func TestListenAndServe_GracefulShutdown(t *testing.T) {
	s, _ := newTestServer(t)
	ctx, cancel := context.WithCancel(context.Background())

	errCh := make(chan error, 1)
	go func() {
		errCh <- s.ListenAndServe(ctx)
	}()

	// Wait a tick then cancel
	time.Sleep(50 * time.Millisecond)
	cancel()

	select {
	case err := <-errCh:
		require.NoError(t, err, "graceful shutdown should not error")
	case <-time.After(5 * time.Second):
		t.Fatal("server did not shut down in time")
	}
}

func TestNew_Validation(t *testing.T) {
	_, err := New(Options{})
	require.ErrorContains(t, err, "pieces store")

	tmp := t.TempDir()
	ps, _ := store.NewDiskStore(tmp)
	_, err = New(Options{Pieces: ps})
	require.ErrorContains(t, err, "listen addr")
}

func TestClientIP(t *testing.T) {
	cases := []struct {
		name         string
		remoteAddr   string
		xff          string
		want         string
	}{
		{"remote-only", "1.2.3.4:9000", "", "1.2.3.4"},
		{"xff-single", "10.0.0.1:80", "203.0.113.5", "203.0.113.5"},
		{"xff-chain", "10.0.0.1:80", "203.0.113.5, 198.51.100.9", "203.0.113.5"},
	}
	for _, tc := range cases {
		t.Run(tc.name, func(t *testing.T) {
			r := httptest.NewRequest(http.MethodGet, "/health", nil)
			r.RemoteAddr = tc.remoteAddr
			if tc.xff != "" {
				r.Header.Set("X-Forwarded-For", tc.xff)
			}
			require.Equal(t, tc.want, clientIP(r))
		})
	}
}
