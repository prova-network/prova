// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Prova Network contributors.

package deal

import (
	"bytes"
	"context"
	"io"
	"log/slog"
	"net/http"
	"net/http/httptest"
	"os"
	"testing"
	"time"

	commp "github.com/filecoin-project/go-fil-commp-hashhash"
	"github.com/ethereum/go-ethereum/common"
	"github.com/stretchr/testify/require"

	"github.com/prova-network/prova/prover/pkg/store"
)

// mockAccepter records invocations without touching a chain.
type mockAccepter struct {
	nextDataSetID uint64
	calls         []DealID
	err           error
}

func (m *mockAccepter) Accept(_ context.Context, id DealID) (uint64, error) {
	m.calls = append(m.calls, id)
	if m.err != nil {
		return 0, m.err
	}
	m.nextDataSetID++
	return m.nextDataSetID, nil
}

// silentLogger suppresses engine log output in tests.
func silentLogger() *slog.Logger {
	return slog.New(slog.NewTextHandler(io.Discard, &slog.HandlerOptions{Level: slog.LevelError}))
}

// testContent generates deterministic bytes of a given length.
func testContent(n int) []byte {
	b := make([]byte, n)
	for i := range b {
		b[i] = byte(i % 251)
	}
	return b
}

// computeExpectedCommP runs the canonical CommP over content and returns
// the 32-byte commitment + padded size.
func computeExpectedCommP(t *testing.T, content []byte) ([32]byte, uint64) {
	t.Helper()
	var c commp.Calc
	_, err := c.Write(content)
	require.NoError(t, err)
	digest, padded, err := c.Digest()
	require.NoError(t, err)
	var out [32]byte
	copy(out[:], digest)
	return out, padded
}

// newTestEngine builds an Engine with memory-backed stores and an HTTP
// server that returns the given content at /piece.
func newTestEngine(t *testing.T, content []byte) (*Engine, *httptest.Server, *mockAccepter, common.Address) {
	t.Helper()
	tmp := t.TempDir()
	ps, err := store.NewDiskStore(tmp)
	require.NoError(t, err)
	t.Cleanup(func() { _ = ps.Close() })

	mux := http.NewServeMux()
	mux.HandleFunc("/piece", func(w http.ResponseWriter, r *http.Request) {
		w.Header().Set("Content-Type", "application/octet-stream")
		_, _ = w.Write(content)
	})
	srv := httptest.NewServer(mux)
	t.Cleanup(srv.Close)

	// httptest.NewServer binds to 127.0.0.1 — that's a loopback address
	// so the fetcher will refuse it unless insecure mode is set.
	t.Setenv("PROVA_PULL_ALLOW_INSECURE", "1")
	pullAllowInsecure = true
	t.Cleanup(func() { pullAllowInsecure = os.Getenv("PROVA_PULL_ALLOW_INSECURE") == "1" })

	mock := &mockAccepter{}
	ourAddr := common.HexToAddress("0x1111111111111111111111111111111111111111")

	eng, err := NewEngine(EngineOptions{
		OurAddress: ourAddr,
		Deals:      NewMemStore(),
		Pieces:     ps,
		Fetcher:    NewFetcher(FetcherOptions{Timeout: 5 * time.Second}),
		Accepter:   mock,
		Logger:     silentLogger(),
	})
	require.NoError(t, err)

	return eng, srv, mock, ourAddr
}

func TestEngine_HappyPath_ProposedToActive(t *testing.T) {
	content := testContent(4 << 10) // 4 KiB
	eng, srv, mock, ourAddr := newTestEngine(t, content)

	commpHash, padded := computeExpectedCommP(t, content)

	d := &Deal{
		ID:           DealID(42),
		Client:       common.HexToAddress("0xC1"),
		Prover:       ourAddr,
		CommPHash:    commpHash,
		PieceSize:    padded,
		TotalPayment: "1000000000000000000",
		SourceURL:    srv.URL + "/piece",
	}

	// Ingest -> Proposed
	require.NoError(t, eng.Ingest(d))
	got, _ := eng.deals.Get(d.ID)
	require.NotNil(t, got)
	require.Equal(t, StatusProposed, got.Status)

	// Tick 1: Proposed -> (Downloading -> Verifying, one step produces two transitions)
	ctx := context.Background()
	require.NoError(t, eng.Tick(ctx))
	got, _ = eng.deals.Get(d.ID)
	require.Equal(t, StatusVerifying, got.Status, "should have downloaded + verified in one step")
	require.Equal(t, uint64(len(content)), got.BytesStored)
	require.Equal(t, commpHash, got.ComputedCommP)

	// Tick 2: Verifying -> Accepting
	require.NoError(t, eng.Tick(ctx))
	got, _ = eng.deals.Get(d.ID)
	require.Equal(t, StatusAccepting, got.Status, "should have moved to Accepting after verify pass")

	// Tick 3: Accepting submits tx, still Accepting (chain event moves to Active)
	require.NoError(t, eng.Tick(ctx))
	got, _ = eng.deals.Get(d.ID)
	require.Equal(t, StatusAccepting, got.Status)
	require.Len(t, mock.calls, 1, "Accepter.Accept should have been called once")
	require.Equal(t, DealID(42), mock.calls[0])
	require.Equal(t, uint64(1), got.DataSetID, "mock Accepter assigned dataSetID=1")

	// External event arrives: DealAccepted observed
	require.NoError(t, eng.MarkActive(d.ID, got.DataSetID))
	got, _ = eng.deals.Get(d.ID)
	require.Equal(t, StatusActive, got.Status)

	// Tick on an Active deal is a no-op
	require.NoError(t, eng.Tick(ctx))
	got, _ = eng.deals.Get(d.ID)
	require.Equal(t, StatusActive, got.Status)
	require.Len(t, mock.calls, 1, "no additional Accept calls once Active")
}

func TestEngine_Ingest_IgnoresOtherProver(t *testing.T) {
	content := testContent(1024)
	eng, srv, _, _ := newTestEngine(t, content)
	commpHash, padded := computeExpectedCommP(t, content)

	d := &Deal{
		ID:        DealID(1),
		Client:    common.HexToAddress("0xC1"),
		Prover:    common.HexToAddress("0x2222222222222222222222222222222222222222"), // not ours
		CommPHash: commpHash,
		PieceSize: padded,
		SourceURL: srv.URL + "/piece",
	}
	require.NoError(t, eng.Ingest(d))

	got, _ := eng.deals.Get(d.ID)
	require.Nil(t, got, "deal for another prover must not be stored")
}

func TestEngine_Ingest_Idempotent(t *testing.T) {
	content := testContent(1024)
	eng, srv, _, ourAddr := newTestEngine(t, content)
	commpHash, padded := computeExpectedCommP(t, content)

	d := &Deal{
		ID:        DealID(7),
		Client:    common.HexToAddress("0xC1"),
		Prover:    ourAddr,
		CommPHash: commpHash,
		PieceSize: padded,
		SourceURL: srv.URL + "/piece",
	}
	require.NoError(t, eng.Ingest(d))

	// Advance it so it's no longer Proposed
	require.NoError(t, eng.Tick(context.Background()))
	mid, _ := eng.deals.Get(d.ID)
	require.Equal(t, StatusVerifying, mid.Status)

	// Re-ingest must not reset status to Proposed
	require.NoError(t, eng.Ingest(d))
	after, _ := eng.deals.Get(d.ID)
	require.Equal(t, StatusVerifying, after.Status)
}

func TestEngine_Download_CommPMismatchFails(t *testing.T) {
	content := testContent(1024)
	eng, srv, _, ourAddr := newTestEngine(t, content)

	// Intentionally wrong commP hash
	wrongHash := [32]byte{0xde, 0xad, 0xbe, 0xef}
	d := &Deal{
		ID:        DealID(99),
		Client:    common.HexToAddress("0xC1"),
		Prover:    ourAddr,
		CommPHash: wrongHash,
		PieceSize: 2048, // also wrong, but download will fail on hash first
		SourceURL: srv.URL + "/piece",
	}
	require.NoError(t, eng.Ingest(d))

	require.NoError(t, eng.Tick(context.Background()))
	got, _ := eng.deals.Get(d.ID)
	require.Equal(t, StatusFailed, got.Status)
	require.Contains(t, got.StatusMsg, "mismatch")
}

func TestEngine_Download_MissingSourceURLFails(t *testing.T) {
	content := testContent(1024)
	eng, _, _, ourAddr := newTestEngine(t, content)
	commpHash, padded := computeExpectedCommP(t, content)

	d := &Deal{
		ID:        DealID(55),
		Prover:    ourAddr,
		CommPHash: commpHash,
		PieceSize: padded,
		SourceURL: "",
	}
	require.NoError(t, eng.Ingest(d))

	require.NoError(t, eng.Tick(context.Background()))
	got, _ := eng.deals.Get(d.ID)
	require.Equal(t, StatusFailed, got.Status)
	require.Contains(t, got.StatusMsg, "no source URL")
}

func TestEngine_AccepterError_MarksFailed(t *testing.T) {
	content := testContent(1024)
	eng, srv, mock, ourAddr := newTestEngine(t, content)
	commpHash, padded := computeExpectedCommP(t, content)

	mock.err = errTestAccept

	d := &Deal{
		ID:        DealID(11),
		Prover:    ourAddr,
		CommPHash: commpHash,
		PieceSize: padded,
		SourceURL: srv.URL + "/piece",
	}
	require.NoError(t, eng.Ingest(d))

	ctx := context.Background()
	require.NoError(t, eng.Tick(ctx)) // Proposed -> Verifying
	require.NoError(t, eng.Tick(ctx)) // Verifying -> Accepting
	require.NoError(t, eng.Tick(ctx)) // Accepting -> Failed (because Accepter errors)

	got, _ := eng.deals.Get(d.ID)
	require.Equal(t, StatusFailed, got.Status)
	require.Contains(t, got.StatusMsg, "chain unreachable")
}

var errTestAccept = &testError{"chain unreachable"}

type testError struct{ msg string }

func (e *testError) Error() string { return e.msg }

func TestEngine_MarkTransitions(t *testing.T) {
	content := testContent(1024)
	eng, srv, _, ourAddr := newTestEngine(t, content)
	commpHash, padded := computeExpectedCommP(t, content)

	// Ingest + drive to Accepting
	d := &Deal{
		ID:        DealID(2),
		Prover:    ourAddr,
		CommPHash: commpHash,
		PieceSize: padded,
		SourceURL: srv.URL + "/piece",
	}
	require.NoError(t, eng.Ingest(d))
	ctx := context.Background()
	require.NoError(t, eng.Tick(ctx))
	require.NoError(t, eng.Tick(ctx))
	require.NoError(t, eng.Tick(ctx))

	// MarkActive
	require.NoError(t, eng.MarkActive(d.ID, 42))
	got, _ := eng.deals.Get(d.ID)
	require.Equal(t, StatusActive, got.Status)
	require.Equal(t, uint64(42), got.DataSetID)

	// MarkCompleted
	require.NoError(t, eng.MarkCompleted(d.ID))
	got, _ = eng.deals.Get(d.ID)
	require.Equal(t, StatusCompleted, got.Status)
	require.True(t, got.IsTerminal())
}

func TestEngine_MarkCancelledOnMissing(t *testing.T) {
	content := testContent(1024)
	eng, _, _, _ := newTestEngine(t, content)
	err := eng.MarkCancelled(DealID(999))
	require.Error(t, err)
	require.Contains(t, err.Error(), "not found")
}

// Ensure the mock Accepter sees a writable buffer. Sanity for internal helpers.
func TestMockAccepter_Sanity(t *testing.T) {
	m := &mockAccepter{}
	id, err := m.Accept(context.Background(), DealID(5))
	require.NoError(t, err)
	require.Equal(t, uint64(1), id)
	id2, err := m.Accept(context.Background(), DealID(6))
	require.NoError(t, err)
	require.Equal(t, uint64(2), id2)
	require.Len(t, m.calls, 2)
}

// Make sure io.Discard import is used
var _ = bytes.NewReader
