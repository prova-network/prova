// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Prova Network contributors.

package deal

import (
	"context"
	"fmt"
	"log/slog"
	"time"

	"github.com/ethereum/go-ethereum/accounts/abi/bind"
	"github.com/ethereum/go-ethereum/common"

	sm "github.com/prova-network/prova/prover/pkg/contracts/storagemarketplace"
)

// EventPoller reads StorageMarketplace events from the chain in a
// pull-based loop and forwards them to the engine.
//
// Pull-based (vs WebSocket subscription) is deliberately simple: testnets
// flake on long-lived subscriptions, polling is idempotent, and we can
// resume cleanly after a restart by tracking the last-seen block in the
// deal store.
type EventPoller struct {
	engine        *Engine
	marketplace   *sm.StorageMarketplace
	ourAddress    common.Address
	pollEvery     time.Duration
	blockLookback uint64
	resolver      *SourceURLResolver
	logger        *slog.Logger
}

// EventPollerOptions configures a poller.
type EventPollerOptions struct {
	Engine      *Engine
	Marketplace *sm.StorageMarketplace
	OurAddress  common.Address
	PollEvery   time.Duration // default: 12s (6 Base blocks)
	Logger      *slog.Logger

	// BlockLookback is the reorg-safety margin. Pass a non-nil pointer to
	// override the default of 6; pass &zero to disable lookback entirely
	// (useful for local anvil tests where every block is final).
	BlockLookback *uint64

	// SourceURLResolver derives piece URLs from (client, commP) when the
	// on-chain event itself carries no URL. Nil disables derivation; deals
	// without an out-of-band SourceURL will fail at the download step.
	SourceURLResolver *SourceURLResolver
}

// NewEventPoller constructs a poller.
func NewEventPoller(opts EventPollerOptions) (*EventPoller, error) {
	if opts.Engine == nil {
		return nil, fmt.Errorf("engine required")
	}
	if opts.Marketplace == nil {
		return nil, fmt.Errorf("marketplace binding required")
	}
	if opts.PollEvery == 0 {
		opts.PollEvery = 12 * time.Second
	}
	var lookback uint64 = 6
	if opts.BlockLookback != nil {
		lookback = *opts.BlockLookback
	}
	if opts.Logger == nil {
		opts.Logger = slog.Default()
	}
	return &EventPoller{
		engine:        opts.Engine,
		marketplace:   opts.Marketplace,
		ourAddress:    opts.OurAddress,
		pollEvery:     opts.PollEvery,
		blockLookback: lookback,
		resolver:      opts.SourceURLResolver,
		logger:        opts.Logger,
	}, nil
}

// PollOnce fetches events since the last watermark and processes them:
//   - DealProposed: ingest into the engine if it targets this prover
//   - DealAccepted: transition the local deal to Active
//   - DealCompleted: transition to Completed (terminal)
//   - DealCancelled: transition to Cancelled (terminal)
//   - DealSlashed: transition to Slashed (terminal)
//
// Returns the number of newly-ingested deals (other transitions are
// applied silently to existing records).
//
// The caller provides currentBlock (from eth_blockNumber). We filter up to
// (currentBlock - BlockLookback) to leave a reorg safety margin.
func (p *EventPoller) PollOnce(ctx context.Context, currentBlock uint64) (int, error) {
	last, err := p.engine.deals.LastSeenBlock()
	if err != nil {
		return 0, fmt.Errorf("read watermark: %w", err)
	}

	safe := currentBlock
	if safe > p.blockLookback {
		safe -= p.blockLookback
	}
	if safe <= last {
		return 0, nil
	}

	start := last + 1
	end := safe

	opts := &bind.FilterOpts{
		Start:   start,
		End:     &end,
		Context: ctx,
	}

	proposedAddr := []common.Address{p.ourAddress}
	it, err := p.marketplace.FilterDealProposed(opts, nil, nil, proposedAddr)
	if err != nil {
		return 0, fmt.Errorf("filter DealProposed: %w", err)
	}
	defer it.Close()

	count := 0
	for it.Next() {
		evt := it.Event
		if evt == nil {
			continue
		}
		d := &Deal{
			ID:           DealID(evt.DealId.Uint64()),
			Client:       evt.Client,
			Prover:       evt.Prover,
			PieceSize:    evt.PieceSize,
			TotalPayment: evt.TotalPayment.String(),
			Duration:     time.Duration(evt.DurationSeconds) * time.Second,
		}
		copy(d.CommPHash[:], evt.CommpHash[:])

		// Derive SourceURL from configured template if available. If the
		// resolver is nil or returns empty, the engine will mark the
		// deal Failed on the next Tick — deployments without the template
		// must advertise piece URLs out-of-band.
		if p.resolver != nil {
			url, err := p.resolver.Resolve(d.Client, d.CommPHash)
			if err != nil {
				p.logger.Warn("source url resolution failed",
					"dealID", uint64(d.ID),
					"err", err,
				)
			}
			d.SourceURL = url
		}

		if err := p.engine.Ingest(d); err != nil {
			p.logger.Error("ingest deal failed",
				"dealID", uint64(d.ID),
				"err", err,
			)
			continue
		}
		count++
	}
	if err := it.Error(); err != nil {
		return count, fmt.Errorf("iterate events: %w", err)
	}

	// Process chain-driven transitions: Accepted, Completed, Cancelled, Slashed.
	// Watermark is only advanced after all filters succeed, so a partial
	// failure retries the whole window.
	accepted, err := p.processDealAccepted(ctx, start, end)
	if err != nil {
		return count, fmt.Errorf("DealAccepted: %w", err)
	}
	completed, err := p.processDealCompleted(ctx, start, end)
	if err != nil {
		return count, fmt.Errorf("DealCompleted: %w", err)
	}
	cancelled, err := p.processDealCancelled(ctx, start, end)
	if err != nil {
		return count, fmt.Errorf("DealCancelled: %w", err)
	}
	slashed, err := p.processDealSlashed(ctx, start, end)
	if err != nil {
		return count, fmt.Errorf("DealSlashed: %w", err)
	}

	if err := p.engine.deals.SetLastSeenBlock(safe); err != nil {
		return count, fmt.Errorf("update watermark: %w", err)
	}

	if count+accepted+completed+cancelled+slashed > 0 {
		p.logger.Info("poll complete",
			"blocks", fmt.Sprintf("%d..%d", start, end),
			"newDeals", count,
			"accepted", accepted,
			"completed", completed,
			"cancelled", cancelled,
			"slashed", slashed,
		)
	}
	return count, nil
}

func (p *EventPoller) processDealAccepted(ctx context.Context, start, end uint64) (int, error) {
	opts := &bind.FilterOpts{Start: start, End: &end, Context: ctx}
	it, err := p.marketplace.FilterDealAccepted(opts, nil, []common.Address{p.ourAddress})
	if err != nil {
		return 0, err
	}
	defer it.Close()
	n := 0
	for it.Next() {
		evt := it.Event
		if evt == nil {
			continue
		}
		if err := p.engine.MarkActive(DealID(evt.DealId.Uint64()), evt.DataSetId.Uint64()); err != nil {
			p.logger.Warn("mark active failed",
				"dealID", evt.DealId.Uint64(),
				"err", err,
			)
			continue
		}
		n++
	}
	return n, it.Error()
}

func (p *EventPoller) processDealCompleted(ctx context.Context, start, end uint64) (int, error) {
	opts := &bind.FilterOpts{Start: start, End: &end, Context: ctx}
	it, err := p.marketplace.FilterDealCompleted(opts, nil)
	if err != nil {
		return 0, err
	}
	defer it.Close()
	n := 0
	for it.Next() {
		evt := it.Event
		if evt == nil {
			continue
		}
		id := DealID(evt.DealId.Uint64())
		if existing, err := p.engine.deals.Get(id); err == nil && existing != nil {
			if err := p.engine.MarkCompleted(id); err == nil {
				n++
			}
		}
	}
	return n, it.Error()
}

func (p *EventPoller) processDealCancelled(ctx context.Context, start, end uint64) (int, error) {
	opts := &bind.FilterOpts{Start: start, End: &end, Context: ctx}
	it, err := p.marketplace.FilterDealCancelled(opts, nil)
	if err != nil {
		return 0, err
	}
	defer it.Close()
	n := 0
	for it.Next() {
		evt := it.Event
		if evt == nil {
			continue
		}
		id := DealID(evt.DealId.Uint64())
		if existing, err := p.engine.deals.Get(id); err == nil && existing != nil {
			if err := p.engine.MarkCancelled(id); err == nil {
				n++
			}
		}
	}
	return n, it.Error()
}

func (p *EventPoller) processDealSlashed(ctx context.Context, start, end uint64) (int, error) {
	opts := &bind.FilterOpts{Start: start, End: &end, Context: ctx}
	it, err := p.marketplace.FilterDealSlashed(opts, nil, nil)
	if err != nil {
		return 0, err
	}
	defer it.Close()
	n := 0
	for it.Next() {
		evt := it.Event
		if evt == nil {
			continue
		}
		id := DealID(evt.DealId.Uint64())
		if existing, err := p.engine.deals.Get(id); err == nil && existing != nil {
			if err := p.engine.MarkSlashed(id, "slashed on-chain"); err == nil {
				n++
			}
		}
	}
	return n, it.Error()
}
