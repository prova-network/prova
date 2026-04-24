// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Prova Network contributors.

// Package daemon wires together the prover components (event poller, deal
// engine, periodic runners) into a supervised long-running process.
//
// The daemon enforces a clear concurrency model:
//
//   - Exactly one goroutine mutates the deal store at a time (the engine tick).
//   - Chain event polling runs on its own goroutine and enqueues deals for
//     the engine to process on the next tick.
//   - Signal handling (SIGINT / SIGTERM) cancels the shared context; all
//     loops unwind via context, not via channels-of-channels.
//   - Graceful shutdown waits for in-flight work with a bounded deadline
//     before hard-exiting.
//
// This is deliberately simpler than Curio's harmonytask scheduler: Prova
// provers have far fewer task types and don't need the cluster-wide
// coordination that Curio does.
package daemon

import (
	"context"
	"errors"
	"fmt"
	"log/slog"
	"sync"
	"time"

	"github.com/ethereum/go-ethereum/common"

	"github.com/prova-network/prova/prover/pkg/deal"
	"github.com/prova-network/prova/prover/pkg/ethclient"
)

// Default tick/poll/status/shutdown timings used when the corresponding
// Config field is left zero.
const (
	defaultTickInterval    = 2 * time.Second
	defaultPollInterval    = 12 * time.Second
	defaultStatusInterval  = 60 * time.Second
	defaultShutdownTimeout = 30 * time.Second
)

// Daemon supervises the prover's background loops.
type Daemon struct {
	cfg    Config
	engine *deal.Engine
	poller *deal.EventPoller
	eth    *ethclient.Client
	logger *slog.Logger

	// Runtime state
	startedAt time.Time
}

// Config holds the knobs the daemon needs.
type Config struct {
	// ProverAddress is this prover's on-chain identity.
	ProverAddress common.Address

	// TickInterval is how often the engine advances deals.
	// Default: 2s. Short tick keeps latency low; the engine is cheap.
	TickInterval time.Duration

	// PollInterval is how often to query the chain for new DealProposed events.
	// Default: 12s (6 Base blocks).
	PollInterval time.Duration

	// StatusInterval is how often to log an aggregate status line.
	// Default: 60s.
	StatusInterval time.Duration

	// ShutdownTimeout is how long to wait for loops to drain on SIGTERM.
	// Default: 30s.
	ShutdownTimeout time.Duration
}

// Options packages the constructor dependencies.
type Options struct {
	Config Config
	Engine *deal.Engine
	Poller *deal.EventPoller
	Eth    *ethclient.Client
	Logger *slog.Logger
}

// New builds a Daemon.
func New(opts Options) (*Daemon, error) {
	if opts.Engine == nil {
		return nil, errors.New("engine required")
	}
	if opts.Poller == nil {
		return nil, errors.New("poller required")
	}
	if opts.Eth == nil {
		return nil, errors.New("eth client required")
	}
	if opts.Config.TickInterval == 0 {
		opts.Config.TickInterval = defaultTickInterval
	}
	if opts.Config.PollInterval == 0 {
		opts.Config.PollInterval = defaultPollInterval
	}
	if opts.Config.StatusInterval == 0 {
		opts.Config.StatusInterval = defaultStatusInterval
	}
	if opts.Config.ShutdownTimeout == 0 {
		opts.Config.ShutdownTimeout = defaultShutdownTimeout
	}
	if opts.Logger == nil {
		opts.Logger = slog.Default()
	}
	return &Daemon{
		cfg:    opts.Config,
		engine: opts.Engine,
		poller: opts.Poller,
		eth:    opts.Eth,
		logger: opts.Logger,
	}, nil
}

// Run blocks until the context is cancelled (typically by SIGTERM).
// Returns nil on clean shutdown; non-nil if a loop errored fatally.
func (d *Daemon) Run(ctx context.Context) error {
	d.startedAt = time.Now().UTC()
	d.logger.Info("daemon starting",
		"prover", d.cfg.ProverAddress.Hex(),
		"chainID", d.eth.ChainID().Uint64(),
		"tickInterval", d.cfg.TickInterval,
		"pollInterval", d.cfg.PollInterval,
	)

	// errCh collects fatal errors from any goroutine. First error cancels
	// the shared context; subsequent errors are logged.
	errCh := make(chan error, 3)
	var wg sync.WaitGroup

	// Poller loop
	wg.Add(1)
	go func() {
		defer wg.Done()
		d.runPollLoop(ctx, errCh)
	}()

	// Engine tick loop
	wg.Add(1)
	go func() {
		defer wg.Done()
		d.runTickLoop(ctx, errCh)
	}()

	// Status loop
	wg.Add(1)
	go func() {
		defer wg.Done()
		d.runStatusLoop(ctx)
	}()

	// Wait for context cancellation or fatal error
	var fatalErr error
	select {
	case <-ctx.Done():
		d.logger.Info("shutdown signal received")
	case fatalErr = <-errCh:
		d.logger.Error("fatal error from loop", "err", fatalErr)
	}

	// Drain any additional errors (non-blocking) and log them
	for {
		select {
		case err := <-errCh:
			d.logger.Error("subsequent loop error", "err", err)
		default:
			goto drained
		}
	}
drained:

	// Wait for graceful drain or timeout
	done := make(chan struct{})
	go func() {
		wg.Wait()
		close(done)
	}()

	shutdownCtx, cancel := context.WithTimeout(context.Background(), d.cfg.ShutdownTimeout)
	defer cancel()

	select {
	case <-done:
		d.logger.Info("daemon stopped cleanly",
			"uptime", time.Since(d.startedAt).Round(time.Second).String(),
		)
	case <-shutdownCtx.Done():
		d.logger.Warn("shutdown timeout exceeded, exiting with outstanding goroutines",
			"timeout", d.cfg.ShutdownTimeout,
		)
	}

	return fatalErr
}

// runPollLoop polls the chain for DealProposed events and ingests them.
func (d *Daemon) runPollLoop(ctx context.Context, errCh chan<- error) {
	// First poll immediately, then tick
	if err := d.pollOnce(ctx); err != nil {
		d.logger.Error("initial poll failed", "err", err)
		// Don't send to errCh — transient failures on first poll are
		// expected (RPC warming up, etc.) and self-heal on the next tick.
	}

	t := time.NewTicker(d.cfg.PollInterval)
	defer t.Stop()

	for {
		select {
		case <-ctx.Done():
			return
		case <-t.C:
			if err := d.pollOnce(ctx); err != nil {
				d.logger.Error("poll failed", "err", err)
				// A single poll failure is recoverable; we'll try again.
				// Fatal errors (e.g. RPC gone forever) would show up as
				// a dial failure at startup, not here.
			}
		}
	}
}

func (d *Daemon) pollOnce(ctx context.Context) error {
	current, err := d.eth.BlockNumber(ctx)
	if err != nil {
		return fmt.Errorf("block number: %w", err)
	}
	n, err := d.poller.PollOnce(ctx, current)
	if err != nil {
		return fmt.Errorf("poll: %w", err)
	}
	if n > 0 {
		d.logger.Info("new deals ingested", "count", n, "currentBlock", current)
	}
	return nil
}

// runTickLoop advances deals through their lifecycle.
func (d *Daemon) runTickLoop(ctx context.Context, errCh chan<- error) {
	t := time.NewTicker(d.cfg.TickInterval)
	defer t.Stop()

	for {
		select {
		case <-ctx.Done():
			return
		case <-t.C:
			if err := d.engine.Tick(ctx); err != nil {
				if errors.Is(err, context.Canceled) {
					return
				}
				d.logger.Error("tick failed", "err", err)
			}
		}
	}
}

// runStatusLoop periodically logs aggregate state.
func (d *Daemon) runStatusLoop(ctx context.Context) {
	t := time.NewTicker(d.cfg.StatusInterval)
	defer t.Stop()

	d.logStatus(ctx) // first one immediately

	for {
		select {
		case <-ctx.Done():
			return
		case <-t.C:
			d.logStatus(ctx)
		}
	}
}

// logStatus emits a single summary line of current deal counts + uptime.
func (d *Daemon) logStatus(ctx context.Context) {
	all, err := d.engine.Deals().ListAll()
	if err != nil {
		d.logger.Warn("status: list deals failed", "err", err)
		return
	}

	counts := map[deal.Status]int{}
	for _, dl := range all {
		counts[dl.Status]++
	}

	block, _ := d.eth.BlockNumber(ctx)

	d.logger.Info("status",
		"uptime", time.Since(d.startedAt).Round(time.Second).String(),
		"block", block,
		"deals_total", len(all),
		"deals_proposed", counts[deal.StatusProposed],
		"deals_downloading", counts[deal.StatusDownloading],
		"deals_verifying", counts[deal.StatusVerifying],
		"deals_accepting", counts[deal.StatusAccepting],
		"deals_active", counts[deal.StatusActive],
		"deals_completed", counts[deal.StatusCompleted],
		"deals_failed", counts[deal.StatusFailed],
	)
}
