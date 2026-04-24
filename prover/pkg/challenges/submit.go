// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Prova Network contributors.

package challenges

import (
	"context"
	"fmt"
	"math/big"

	"github.com/ethereum/go-ethereum/accounts/abi/bind"
	"github.com/ethereum/go-ethereum/common"
	"github.com/ethereum/go-ethereum/core/types"

	pv "github.com/prova-network/prova/prover/pkg/contracts/proofverifier"
)

// ChainClient is the minimal subset of on-chain operations the challenge
// handler needs. Split as an interface for testability.
type ChainClient interface {
	// GetRandomness returns the PDP challenge seed for a given epoch.
	// Implementations call ProofVerifier.getRandomness(epoch).
	GetRandomness(ctx context.Context, epoch *big.Int) ([32]byte, error)

	// GetChallengeRange returns the total leaf count for a data set.
	GetChallengeRange(ctx context.Context, dataSetID *big.Int) (*big.Int, error)

	// GetNextChallengeEpoch returns the block number at which the next
	// challenge for this data set is sampled.
	GetNextChallengeEpoch(ctx context.Context, dataSetID *big.Int) (*big.Int, error)

	// SubmitProof calls ProofVerifier.provePossession(setId, proofs) and
	// returns the transaction hash.
	SubmitProof(ctx context.Context, opts *bind.TransactOpts, dataSetID *big.Int, proofs []Proof) (common.Hash, error)

	// WaitReceipt blocks until the given tx has a mined receipt.
	WaitReceipt(ctx context.Context, txHash common.Hash) (*types.Receipt, error)
}

// OnChainClient adapts the generated proofverifier.ProofVerifier bindings
// to the ChainClient interface used by this package.
type OnChainClient struct {
	verifier *pv.ProofVerifier
}

// NewOnChainClient wraps a contract binding.
func NewOnChainClient(verifier *pv.ProofVerifier) *OnChainClient {
	return &OnChainClient{verifier: verifier}
}

// GetRandomness proxies to ProofVerifier.getRandomness.
func (c *OnChainClient) GetRandomness(ctx context.Context, epoch *big.Int) ([32]byte, error) {
	val, err := c.verifier.GetRandomness(&bind.CallOpts{Context: ctx}, epoch)
	if err != nil {
		return [32]byte{}, fmt.Errorf("getRandomness(%s): %w", epoch, err)
	}
	// The contract returns uint256; encode big-endian into 32 bytes.
	var out [32]byte
	val.FillBytes(out[:])
	return out, nil
}

// GetChallengeRange proxies to ProofVerifier.getChallengeRange.
func (c *OnChainClient) GetChallengeRange(ctx context.Context, dataSetID *big.Int) (*big.Int, error) {
	r, err := c.verifier.GetChallengeRange(&bind.CallOpts{Context: ctx}, dataSetID)
	if err != nil {
		return nil, fmt.Errorf("getChallengeRange(%s): %w", dataSetID, err)
	}
	return r, nil
}

// GetNextChallengeEpoch proxies to ProofVerifier.getNextChallengeEpoch.
func (c *OnChainClient) GetNextChallengeEpoch(ctx context.Context, dataSetID *big.Int) (*big.Int, error) {
	e, err := c.verifier.GetNextChallengeEpoch(&bind.CallOpts{Context: ctx}, dataSetID)
	if err != nil {
		return nil, fmt.Errorf("getNextChallengeEpoch(%s): %w", dataSetID, err)
	}
	return e, nil
}

// SubmitProof packs proofs into the generated struct type and calls
// ProofVerifier.provePossession.
func (c *OnChainClient) SubmitProof(
	ctx context.Context,
	opts *bind.TransactOpts,
	dataSetID *big.Int,
	proofs []Proof,
) (common.Hash, error) {
	if opts == nil {
		return common.Hash{}, fmt.Errorf("transact opts required")
	}
	opts.Context = ctx

	// Convert our local Proof type to the generated on-chain struct.
	packed := make([]pv.IPDPTypesProof, len(proofs))
	for i, p := range proofs {
		packed[i] = pv.IPDPTypesProof{
			Leaf:  p.Leaf,
			Proof: p.Proof,
		}
	}

	tx, err := c.verifier.ProvePossession(opts, dataSetID, packed)
	if err != nil {
		return common.Hash{}, fmt.Errorf("provePossession: %w", err)
	}
	return tx.Hash(), nil
}

// WaitReceipt is intentionally not implemented at this layer; callers
// that need tx confirmation should use the ethclient.Client wrapper
// directly. This keeps our challenge package dependency-light.
//
// We keep the method on ChainClient for completeness and to avoid
// sprinkling ethclient imports across the prover — Phase D callers will
// typically pass a wrapper that composes OnChainClient with the
// ethclient's WaitReceipt.
func (c *OnChainClient) WaitReceipt(_ context.Context, _ common.Hash) (*types.Receipt, error) {
	return nil, fmt.Errorf("WaitReceipt is not implemented on OnChainClient; compose with ethclient")
}
