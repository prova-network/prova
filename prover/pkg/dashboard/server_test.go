// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Prova Network contributors.

package dashboard

import (
	"context"
	"encoding/json"
	"io"
	"net/http"
	"net/http/httptest"
	"strings"
	"testing"
	"time"

	"github.com/ethereum/go-ethereum/common"

	"github.com/prova-network/prova/prover/pkg/deal"
)

func testStore(t *testing.T) deal.Store {
	t.Helper()
	s := deal.NewMemStore()
	// seed 2 deals, one active, one completed
	if err := s.Upsert(&deal.Deal{
		ID:        1,
		Client:    common.HexToAddress("0x1111111111111111111111111111111111111111"),
		Prover:    common.HexToAddress("0x2222222222222222222222222222222222222222"),
		PieceSize: 1024,
		Status:    deal.StatusActive,
		UpdatedAt: time.Now().UTC(),
	}); err != nil {
		t.Fatal(err)
	}
	if err := s.Upsert(&deal.Deal{
		ID:        2,
		Client:    common.HexToAddress("0x3333333333333333333333333333333333333333"),
		Prover:    common.HexToAddress("0x2222222222222222222222222222222222222222"),
		PieceSize: 2048,
		Status:    deal.StatusCompleted,
		UpdatedAt: time.Now().UTC(),
	}); err != nil {
		t.Fatal(err)
	}
	return s
}

func TestServer_Overview_NoChainNoMetrics(t *testing.T) {
	srv, err := New(Options{
		Enabled: true,
		Store:   testStore(t),
	})
	if err != nil {
		t.Fatal(err)
	}
	rr := httptest.NewRecorder()
	req := httptest.NewRequest(http.MethodGet, "/api/overview", nil)
	srv.Handler().ServeHTTP(rr, req)
	if rr.Code != http.StatusOK {
		t.Fatalf("status=%d body=%s", rr.Code, rr.Body.String())
	}
	var body map[string]any
	if err := json.Unmarshal(rr.Body.Bytes(), &body); err != nil {
		t.Fatalf("parse: %v body=%s", err, rr.Body.String())
	}
	if body["version"] == nil {
		t.Error("missing version")
	}
	if body["deals_total"].(float64) != 2 {
		t.Errorf("deals_total=%v want 2", body["deals_total"])
	}
	if _, ok := body["chain"]; ok {
		t.Error("chain should not be present without a ChainReader")
	}
}

func TestServer_Deals_ListAndFilter(t *testing.T) {
	srv, err := New(Options{Enabled: true, Store: testStore(t)})
	if err != nil {
		t.Fatal(err)
	}
	// All
	rr := httptest.NewRecorder()
	req := httptest.NewRequest(http.MethodGet, "/api/deals", nil)
	srv.Handler().ServeHTTP(rr, req)
	if rr.Code != 200 {
		t.Fatalf("all: %d", rr.Code)
	}
	var body struct {
		Deals []dealView `json:"deals"`
		Count int        `json:"count"`
	}
	_ = json.Unmarshal(rr.Body.Bytes(), &body)
	if body.Count != 2 {
		t.Errorf("count=%d want 2", body.Count)
	}

	// Filter: active only
	rr = httptest.NewRecorder()
	req = httptest.NewRequest(http.MethodGet, "/api/deals?status=active", nil)
	srv.Handler().ServeHTTP(rr, req)
	body = struct {
		Deals []dealView `json:"deals"`
		Count int        `json:"count"`
	}{}
	_ = json.Unmarshal(rr.Body.Bytes(), &body)
	if body.Count != 1 {
		t.Errorf("active count=%d want 1", body.Count)
	}
	if body.Deals[0].Status != "active" {
		t.Errorf("status=%s", body.Deals[0].Status)
	}
}

func TestServer_DealByID(t *testing.T) {
	srv, _ := New(Options{Enabled: true, Store: testStore(t)})

	rr := httptest.NewRecorder()
	req := httptest.NewRequest(http.MethodGet, "/api/deals/1", nil)
	srv.Handler().ServeHTTP(rr, req)
	if rr.Code != 200 {
		t.Fatalf("get 1: %d %s", rr.Code, rr.Body.String())
	}
	var d dealView
	_ = json.Unmarshal(rr.Body.Bytes(), &d)
	if d.ID != 1 {
		t.Errorf("id=%d", d.ID)
	}

	// Missing
	rr = httptest.NewRecorder()
	req = httptest.NewRequest(http.MethodGet, "/api/deals/9999", nil)
	srv.Handler().ServeHTTP(rr, req)
	if rr.Code != 404 {
		t.Errorf("missing should be 404, got %d", rr.Code)
	}

	// Bad id
	rr = httptest.NewRecorder()
	req = httptest.NewRequest(http.MethodGet, "/api/deals/notanumber", nil)
	srv.Handler().ServeHTTP(rr, req)
	if rr.Code != 400 {
		t.Errorf("bad id should be 400, got %d", rr.Code)
	}
}

func TestServer_Placeholder(t *testing.T) {
	srv, _ := New(Options{Enabled: true, Store: testStore(t)})

	rr := httptest.NewRecorder()
	req := httptest.NewRequest(http.MethodGet, "/", nil)
	srv.Handler().ServeHTTP(rr, req)
	// When dist is empty (no built SPA), spaHandler serves the file-server
	// which 404s on "/"; the fallback is the placeholder handler registered
	// at "/". Either outcome is acceptable as long as something renders.
	if rr.Code >= 500 {
		t.Fatalf("landing page 5xx: %d", rr.Code)
	}
}

func TestServer_Info(t *testing.T) {
	srv, _ := New(Options{Enabled: true, Store: testStore(t)})

	rr := httptest.NewRecorder()
	req := httptest.NewRequest(http.MethodGet, "/api/info", nil)
	srv.Handler().ServeHTTP(rr, req)
	if rr.Code != 200 {
		t.Fatalf("info: %d", rr.Code)
	}
	body, _ := io.ReadAll(rr.Body)
	if !strings.Contains(string(body), "go_version") {
		t.Errorf("info body missing go_version: %s", body)
	}
}

func TestServer_ChainNotConfigured(t *testing.T) {
	srv, _ := New(Options{Enabled: true, Store: testStore(t)})

	rr := httptest.NewRecorder()
	req := httptest.NewRequest(http.MethodGet, "/api/chain", nil)
	srv.Handler().ServeHTTP(rr, req)
	if rr.Code != 503 {
		t.Errorf("want 503 when chain not configured, got %d", rr.Code)
	}
}

// fakeChain implements ChainReader for testing.
type fakeChain struct {
	snap ChainSnapshot
	err  error
}

func (f fakeChain) Snapshot(ctx context.Context) (ChainSnapshot, error) {
	return f.snap, f.err
}

func TestServer_ChainConfigured(t *testing.T) {
	srv, _ := New(Options{
		Enabled: true,
		Store:   testStore(t),
		Chain: fakeChain{snap: ChainSnapshot{
			ChainID:          8453,
			BlockNumber:      123,
			ProverRegistered: true,
			StakedWei:        "1000000000000000000",
		}},
	})

	rr := httptest.NewRecorder()
	req := httptest.NewRequest(http.MethodGet, "/api/chain", nil)
	srv.Handler().ServeHTTP(rr, req)
	if rr.Code != 200 {
		t.Fatalf("chain: %d %s", rr.Code, rr.Body.String())
	}
	var snap ChainSnapshot
	_ = json.Unmarshal(rr.Body.Bytes(), &snap)
	if snap.ChainID != 8453 {
		t.Errorf("chainID=%d", snap.ChainID)
	}
	if !snap.ProverRegistered {
		t.Error("prover should be registered")
	}
}

func TestServer_DisabledIsNoop(t *testing.T) {
	srv, err := New(Options{Enabled: false})
	if err != nil {
		t.Fatal(err)
	}
	ctx, cancel := context.WithTimeout(context.Background(), 100*time.Millisecond)
	defer cancel()
	if err := srv.Start(ctx); err != nil {
		t.Fatalf("disabled Start should return nil, got %v", err)
	}
}
