// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Prova Network contributors.

package challenges

import (
	"context"
	"fmt"
	"log/slog"
	"math/big"

	"github.com/ethereum/go-ethereum/accounts/abi/bind"
	"github.com/ethereum/go-ethereum/common"
)

// Runner drives one-off proving attempts for a single data set.
//
// Orchestration (daemon scheduling, retries, per-deal tracking) lives
// outside this package. Runner is a pure unit of work that can be called
// by a challenge-event handler or a status command.
type Runner struct {
	client     ChainClient
	lookup     PieceLookup
	builder    MerkleBuilder
	transactor *bind.TransactOpts
	numChallenges uint64
	logger     *slog.Logger
}

// RunnerOptions configures a Runner.
type RunnerOptions struct {
	Client        ChainClient
	Lookup        PieceLookup
	Builder       MerkleBuilder
	Transactor    *bind.TransactOpts
	NumChallenges uint64 // default: 5
	Logger        *slog.Logger
}

// NewRunner constructs a proving Runner.
func NewRunner(opts RunnerOptions) (*Runner, error) {
	if opts.Client == nil {
		return nil, fmt.Errorf("chain client required")
	}
	if opts.Lookup == nil {
		return nil, fmt.Errorf("piece lookup required")
	}
	if opts.Builder == nil {
		return nil, fmt.Errorf("merkle builder required")
	}
	if opts.Transactor == nil {
		return nil, fmt.Errorf("transactor required")
	}
	if opts.NumChallenges == 0 {
		opts.NumChallenges = 5
	}
	if opts.Logger == nil {
		opts.Logger = slog.Default()
	}
	return &Runner{
		client:        opts.Client,
		lookup:        opts.Lookup,
		builder:       opts.Builder,
		transactor:    opts.Transactor,
		numChallenges: opts.NumChallenges,
		logger:        opts.Logger,
	}, nil
}

// ProveSet generates and submits proofs for one data set's current
// challenge period. Returns the tx hash of the provePossession call.
//
// Steps:
//   1. Read nextChallengeEpoch, getRandomness(epoch), getChallengeRange.
//   2. Compute N challenge leaf indices.
//   3. Look up piece mapping for each index.
//   4. Build Merkle proofs via the MerkleBuilder.
//   5. Submit provePossession(setId, proofs).
func (r *Runner) ProveSet(ctx context.Context, dataSetID *big.Int) (common.Hash, error) {
	r.logger.Info("generating proofs", "dataSetID", dataSetID.String())

	epoch, err := r.client.GetNextChallengeEpoch(ctx, dataSetID)
	if err != nil {
		return common.Hash{}, fmt.Errorf("epoch: %w", err)
	}
	if epoch == nil || epoch.Sign() == 0 {
		return common.Hash{}, fmt.Errorf("no challenge scheduled for dataset %s", dataSetID.String())
	}

	seed, err := r.client.GetRandomness(ctx, epoch)
	if err != nil {
		return common.Hash{}, fmt.Errorf("randomness: %w", err)
	}

	total, err := r.client.GetChallengeRange(ctx, dataSetID)
	if err != nil {
		return common.Hash{}, fmt.Errorf("challenge range: %w", err)
	}
	if total == nil || total.Sign() == 0 {
		return common.Hash{}, fmt.Errorf("dataset %s has zero leaves", dataSetID.String())
	}
	totalLeaves := total.Uint64()

	proofs, err := GenerateProofs(seed, dataSetID, r.numChallenges, totalLeaves, r.lookup, r.builder)
	if err != nil {
		return common.Hash{}, fmt.Errorf("generate proofs: %w", err)
	}

	r.logger.Info("submitting proofs",
		"dataSetID", dataSetID.String(),
		"numChallenges", r.numChallenges,
		"epoch", epoch.String(),
		"totalLeaves", totalLeaves,
	)

	txHash, err := r.client.SubmitProof(ctx, r.transactor, dataSetID, proofs)
	if err != nil {
		return common.Hash{}, fmt.Errorf("submit: %w", err)
	}

	r.logger.Info("proof submitted", "dataSetID", dataSetID.String(), "tx", txHash.Hex())
	return txHash, nil
}
