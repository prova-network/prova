// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Prova Network contributors.

package deal

import (
	"bytes"
	"context"
	"errors"
	"fmt"
	"io"
	"log/slog"
	"time"

	commp "github.com/filecoin-project/go-fil-commp-hashhash"
	"github.com/ethereum/go-ethereum/common"
	"github.com/ipfs/go-cid"

	"github.com/prova-network/prova/prover/pkg/store"
)

// Engine drives deals through their lifecycle. One instance per prover.
//
// The engine is tick-driven: external code calls Tick() periodically (from
// a timer or after ingesting new events), and the engine advances eligible
// deals one step. This makes the engine easy to test and keeps side
// effects localized.
type Engine struct {
	ourAddress common.Address
	deals      Store
	pieces     store.Store
	fetcher    *Fetcher
	accepter   Accepter
	logger     *slog.Logger
}

// Accepter is the abstract interface the engine uses to submit the
// acceptance transaction. The concrete implementation calls
// ProofVerifier.createDataSet(marketplaceAddr, abi.encode(dealId)); a mock
// is used in tests.
type Accepter interface {
	Accept(ctx context.Context, dealID DealID) (dataSetID uint64, err error)
}

// EngineOptions configures a new Engine.
type EngineOptions struct {
	OurAddress common.Address
	Deals      Store
	Pieces     store.Store
	Fetcher    *Fetcher
	Accepter   Accepter
	Logger     *slog.Logger
}

// NewEngine constructs an Engine.
func NewEngine(opts EngineOptions) (*Engine, error) {
	if opts.Deals == nil {
		return nil, fmt.Errorf("deal store is required")
	}
	if opts.Pieces == nil {
		return nil, fmt.Errorf("piece store is required")
	}
	if opts.Fetcher == nil {
		opts.Fetcher = NewFetcher(FetcherOptions{})
	}
	if opts.Accepter == nil {
		return nil, fmt.Errorf("accepter is required")
	}
	if opts.Logger == nil {
		opts.Logger = slog.Default()
	}
	return &Engine{
		ourAddress: opts.OurAddress,
		deals:      opts.Deals,
		pieces:     opts.Pieces,
		fetcher:    opts.Fetcher,
		accepter:   opts.Accepter,
		logger:     opts.Logger,
	}, nil
}

// Deals exposes the underlying deal Store for read access. Primarily for
// observability: CLI status commands, tests, and smoke scripts.
func (e *Engine) Deals() Store { return e.deals }

// Ingest adds a newly-observed DealProposed into the local store if it
// targets us. Callers pass deals they've filtered from chain logs.
// Idempotent: calling twice with the same ID is a no-op.
func (e *Engine) Ingest(d *Deal) error {
	if d == nil {
		return fmt.Errorf("nil deal")
	}
	if !d.IsOurs(e.ourAddress) {
		// Not addressed to this prover; silently ignore.
		return nil
	}

	existing, err := e.deals.Get(d.ID)
	if err != nil {
		return fmt.Errorf("check existing: %w", err)
	}
	if existing != nil {
		// Already known; leave alone.
		return nil
	}

	d.Status = StatusProposed
	d.UpdatedAt = time.Now().UTC()
	if err := e.deals.Upsert(d); err != nil {
		return fmt.Errorf("upsert: %w", err)
	}
	e.logger.Info("deal ingested",
		"dealID", uint64(d.ID),
		"client", d.Client.Hex(),
		"pieceSize", d.PieceSize,
	)
	return nil
}

// Tick advances every non-terminal, prover-driven deal by at most one
// step. It is safe to call concurrently from multiple goroutines, but
// deals are processed serially per call. In production, a single
// background loop should drive it.
//
// One Tick call == at most one state transition per deal, so callers can
// observe intermediate states (useful for tests and operator monitoring).
// Multiple Ticks are needed to drive a deal from Proposed to Accepting.
func (e *Engine) Tick(ctx context.Context) error {
	all, err := e.deals.ListAll()
	if err != nil {
		return fmt.Errorf("list deals: %w", err)
	}

	for _, d := range all {
		if ctx.Err() != nil {
			return ctx.Err()
		}
		switch d.Status {
		case StatusProposed, StatusDownloading, StatusVerifying, StatusAccepting:
			e.advance(ctx, d)
		default:
			// Active / Completed / Cancelled / Slashed / Failed: engine does nothing.
		}
	}
	return nil
}

// advance moves a single deal one step. Errors are recorded on the deal
// itself; the caller decides whether that's fatal.
func (e *Engine) advance(ctx context.Context, d *Deal) {
	var err error
	switch d.Status {
	case StatusProposed:
		err = e.doDownload(ctx, d)
	case StatusDownloading:
		err = e.doVerify(ctx, d)
	case StatusVerifying:
		err = e.doAccept(ctx, d)
	case StatusAccepting:
		// Nothing to do; waiting for DealAccepted event to move us to Active
		return
	default:
		return
	}

	if err != nil {
		e.logger.Error("deal step failed",
			"dealID", uint64(d.ID),
			"status", string(d.Status),
			"err", err,
		)
		d.Status = StatusFailed
		d.StatusMsg = err.Error()
		d.UpdatedAt = time.Now().UTC()
		_ = e.deals.Upsert(d)
	}
}

// doDownload fetches the piece content into local storage.
//
// In v1 we assume the SourceURL is set on the deal record at ingest time
// (typically via an off-chain hint the client publishes alongside the
// on-chain DealProposed event, or a future extraData field). If SourceURL
// is empty we fail the deal early — the client hasn't actually made the
// piece available.
func (e *Engine) doDownload(ctx context.Context, d *Deal) error {
	if d.SourceURL == "" {
		return fmt.Errorf("deal has no source URL; cannot fetch piece")
	}

	// Transition to Downloading up front so a crash mid-download is
	// retried on the next Tick.
	d.Status = StatusDownloading
	d.UpdatedAt = time.Now().UTC()
	if err := e.deals.Upsert(d); err != nil {
		return fmt.Errorf("mark downloading: %w", err)
	}

	// Stream into a temp buffer while computing CommP; once CommP
	// matches, we move bytes into the piece store.
	//
	// TODO(phase-C-polish): stream directly into the piece store via a
	// tee writer rather than buffering. For now the conservative path is
	// simplest: hold the bytes, verify, then commit. Acceptable up to the
	// 32 GiB fetcher limit but we'll want streaming for production.
	var buf bytes.Buffer
	n, err := e.fetcher.Fetch(ctx, d.SourceURL, &buf)
	if err != nil {
		return fmt.Errorf("fetch: %w", err)
	}
	if n == 0 {
		return fmt.Errorf("fetched empty body")
	}

	d.BytesStored = uint64(n)
	d.UpdatedAt = time.Now().UTC()
	// Stash the in-memory blob on the deal record? No — keep it local.
	// We transition to Verifying; the next step recomputes CommP directly
	// from the buffered bytes. We need to hold them somewhere the next
	// step can see. The simplest path: push into piece store under a
	// provisional key (deal-bound hash), then compute CommP, then rename.
	//
	// For v1 simplicity we combine doDownload+doVerify into a single
	// step when the buffer is still available, skipping the provisional
	// write. This is why doVerify below is effectively a no-op transition
	// when the download succeeded: CommP computation happens inline here.
	//
	// Compute CommP while we have the bytes in hand:
	computed, padded, err := computeCommP(bytes.NewReader(buf.Bytes()), uint64(n))
	if err != nil {
		return fmt.Errorf("compute commP: %w", err)
	}
	if padded != d.PieceSize {
		return fmt.Errorf("padded piece size mismatch: expected %d, computed %d",
			d.PieceSize, padded)
	}
	if computed != d.CommPHash {
		return fmt.Errorf("commP mismatch: expected 0x%x, computed 0x%x",
			d.CommPHash, computed)
	}
	d.ComputedCommP = computed

	// Commit the bytes to the piece store, addressed by the computed CommP
	// as a plain CID v1 (raw codec) — we don't invent a new multicodec yet.
	// This makes the piece store agnostic to the specific hash flavor.
	pieceCid, err := commpCID(computed)
	if err != nil {
		return fmt.Errorf("build piece cid: %w", err)
	}

	// Check-then-store: if we already have this piece (another deal with
	// the same content), skip the write. Piece store is content-addressed
	// so duplicates are safe.
	has, err := e.pieces.Has(pieceCid)
	if err != nil {
		return fmt.Errorf("piece store has: %w", err)
	}
	if !has {
		written, err := e.pieces.Put(pieceCid, bytes.NewReader(buf.Bytes()))
		if err != nil {
			return fmt.Errorf("piece store put: %w", err)
		}
		if written != uint64(n) {
			return fmt.Errorf("piece store wrote %d bytes, expected %d", written, n)
		}
	}

	// Download + verify succeeded; move to Verifying so the next tick
	// transitions to Accepting. Splitting the states here is a minor
	// convention: separates "we did the I/O" from "we'll now submit a tx".
	d.Status = StatusVerifying
	d.StatusMsg = "commP verified"
	d.UpdatedAt = time.Now().UTC()
	return e.deals.Upsert(d)
}

// doVerify is a thin transition step. In this v1 layout the hard work
// (CommP compute, equality check) already happened inside doDownload.
// This step exists as an explicit state so crash recovery can resume at
// the right point without re-downloading a verified piece.
func (e *Engine) doVerify(_ context.Context, d *Deal) error {
	// Sanity check that we actually have the piece on-disk before
	// moving to accepting.
	pieceCid, err := commpCID(d.ComputedCommP)
	if err != nil {
		return fmt.Errorf("build piece cid: %w", err)
	}
	has, err := e.pieces.Has(pieceCid)
	if err != nil {
		return fmt.Errorf("piece store has: %w", err)
	}
	if !has {
		return fmt.Errorf("piece missing from local store; commP %x", d.ComputedCommP[:4])
	}

	d.Status = StatusAccepting
	d.UpdatedAt = time.Now().UTC()
	return e.deals.Upsert(d)
}

// doAccept submits the on-chain acceptance transaction via the Accepter
// interface. On success the deal moves to Accepting (awaiting the
// DealAccepted event to flip it to Active). The deal only returns to the
// Tick loop when MarkAccepted is called externally, so doAccept is a
// one-shot step.
func (e *Engine) doAccept(ctx context.Context, d *Deal) error {
	dataSetID, err := e.accepter.Accept(ctx, d.ID)
	if err != nil {
		return fmt.Errorf("accept: %w", err)
	}
	d.DataSetID = dataSetID
	d.AcceptedAt = time.Now().UTC()
	d.Status = StatusAccepting
	d.StatusMsg = fmt.Sprintf("accept tx submitted, dataSetID=%d", dataSetID)
	d.UpdatedAt = time.Now().UTC()
	return e.deals.Upsert(d)
}

// MarkActive transitions a deal from Accepting to Active. Called when we
// observe a DealAccepted event for this deal.
func (e *Engine) MarkActive(id DealID, dataSetID uint64) error {
	d, err := e.deals.Get(id)
	if err != nil || d == nil {
		return fmt.Errorf("deal %d not found: %v", id, err)
	}
	if d.Status == StatusActive {
		return nil
	}
	d.Status = StatusActive
	d.DataSetID = dataSetID
	d.StatusMsg = "deal accepted on-chain"
	d.UpdatedAt = time.Now().UTC()
	return e.deals.Upsert(d)
}

// MarkCancelled transitions a deal to Cancelled. Called when the client
// cancels a Proposed deal on-chain.
func (e *Engine) MarkCancelled(id DealID) error {
	d, err := e.deals.Get(id)
	if err != nil || d == nil {
		return fmt.Errorf("deal %d not found: %v", id, err)
	}
	d.Status = StatusCancelled
	d.StatusMsg = "cancelled by client"
	d.UpdatedAt = time.Now().UTC()
	return e.deals.Upsert(d)
}

// MarkCompleted transitions a deal to Completed. Called on DealCompleted.
func (e *Engine) MarkCompleted(id DealID) error {
	d, err := e.deals.Get(id)
	if err != nil || d == nil {
		return fmt.Errorf("deal %d not found: %v", id, err)
	}
	d.Status = StatusCompleted
	d.StatusMsg = "deal completed"
	d.UpdatedAt = time.Now().UTC()
	return e.deals.Upsert(d)
}

// MarkSlashed transitions a deal to Slashed. Called on DealSlashed.
func (e *Engine) MarkSlashed(id DealID, reason string) error {
	d, err := e.deals.Get(id)
	if err != nil || d == nil {
		return fmt.Errorf("deal %d not found: %v", id, err)
	}
	d.Status = StatusSlashed
	d.StatusMsg = "slashed: " + reason
	d.UpdatedAt = time.Now().UTC()
	return e.deals.Upsert(d)
}

// computeCommP runs the CommP calculation over the given reader, returning
// the 32-byte commitment and the padded piece size in bytes.
func computeCommP(r io.Reader, rawSize uint64) ([32]byte, uint64, error) {
	_ = rawSize // not needed by Calc; kept in signature for future streaming variants
	var c commp.Calc
	if _, err := io.Copy(&c, r); err != nil {
		return [32]byte{}, 0, err
	}
	digest, padded, err := c.Digest()
	if err != nil {
		return [32]byte{}, 0, err
	}
	if len(digest) != 32 {
		return [32]byte{}, 0, fmt.Errorf("unexpected digest length %d", len(digest))
	}
	var out [32]byte
	copy(out[:], digest)
	return out, padded, nil
}

// commpCID wraps a 32-byte CommP hash into a CID v1 using the standard
// Filecoin piece-commitment multicodec.
func commpCID(hash [32]byte) (cid.Cid, error) {
	// Match the "raw commP" CID encoding used by curio/pdp: sha2-256-trunc254-padded multihash, fil-commitment-unsealed codec.
	// The multihash bytes are: <0x91, 0x20, 0x20, hash...>
	mh := append([]byte{0x91, 0x20, 0x20}, hash[:]...)
	_ = mh
	return cid.Parse(mhToCIDBytes(hash))
}

// mhToCIDBytes constructs the binary CID v1 bytes for a CommP.
// Prefix layout:
//   version(1) || codec(0xf101, varint) || multihash(0x91,0x20 + 0x20 + 32 bytes hash)
// codec 0xf101 = fil-commitment-unsealed
func mhToCIDBytes(hash [32]byte) []byte {
	// version 1, raw = 0x01. fil-commitment-unsealed codec = 0xf101 (varint: 0x81 0xE2 0x03).
	prefix := []byte{0x01, 0x81, 0xe2, 0x03}
	// multihash: sha2-256-trunc254-padded = 0x1012 (varint: 0x92, 0x20). Length 32 = 0x20.
	// Note: Curio uses multicodec Sha2_256Trunc254Padded = 0x1012.
	mh := []byte{0x92, 0x20, 0x20}
	out := make([]byte, 0, len(prefix)+len(mh)+32)
	out = append(out, prefix...)
	out = append(out, mh...)
	out = append(out, hash[:]...)
	return out
}

// Errors for introspection by callers.
var (
	ErrDealNotFound = errors.New("deal not found")
)
