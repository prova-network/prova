// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Prova Network contributors.

package challenges

import (
	"context"
	"io"
	"log/slog"
	"math/big"
	"testing"

	"github.com/ethereum/go-ethereum/accounts/abi/bind"
	"github.com/ethereum/go-ethereum/common"
	"github.com/ethereum/go-ethereum/core/types"
	"github.com/stretchr/testify/require"
)

type stubChainClient struct {
	seed         [32]byte
	epoch        *big.Int
	totalLeaves  *big.Int
	submittedDS  *big.Int
	submittedNum int
	txHash       common.Hash
	epochErr     error
	submitErr    error
}

func (s *stubChainClient) GetRandomness(_ context.Context, _ *big.Int) ([32]byte, error) {
	return s.seed, nil
}
func (s *stubChainClient) GetChallengeRange(_ context.Context, _ *big.Int) (*big.Int, error) {
	return s.totalLeaves, nil
}
func (s *stubChainClient) GetNextChallengeEpoch(_ context.Context, _ *big.Int) (*big.Int, error) {
	return s.epoch, s.epochErr
}
func (s *stubChainClient) SubmitProof(
	_ context.Context, _ *bind.TransactOpts, dsID *big.Int, proofs []Proof,
) (common.Hash, error) {
	if s.submitErr != nil {
		return common.Hash{}, s.submitErr
	}
	s.submittedDS = dsID
	s.submittedNum = len(proofs)
	return s.txHash, nil
}
func (s *stubChainClient) WaitReceipt(_ context.Context, _ common.Hash) (*types.Receipt, error) {
	return nil, nil
}

func silentLog() *slog.Logger {
	return slog.New(slog.NewTextHandler(io.Discard, &slog.HandlerOptions{Level: slog.LevelError}))
}

func TestRunner_ProveSet_Happy(t *testing.T) {
	client := &stubChainClient{
		seed:        [32]byte{0x42},
		epoch:       big.NewInt(1000),
		totalLeaves: big.NewInt(4096),
		txHash:      common.HexToHash("0xabc123"),
	}

	runner, err := NewRunner(RunnerOptions{
		Client:        client,
		Lookup:        &stubLookup{},
		Builder:       &stubBuilder{},
		Transactor:    &bind.TransactOpts{From: common.Address{0x1}},
		NumChallenges: 4,
		Logger:        silentLog(),
	})
	require.NoError(t, err)

	hash, err := runner.ProveSet(context.Background(), big.NewInt(7))
	require.NoError(t, err)
	require.Equal(t, common.HexToHash("0xabc123"), hash)
	require.Equal(t, int64(7), client.submittedDS.Int64())
	require.Equal(t, 4, client.submittedNum)
}

func TestRunner_ProveSet_NoEpochScheduled(t *testing.T) {
	client := &stubChainClient{
		seed:        [32]byte{},
		epoch:       big.NewInt(0), // no challenge scheduled
		totalLeaves: big.NewInt(1024),
	}
	runner, _ := NewRunner(RunnerOptions{
		Client: client, Lookup: &stubLookup{}, Builder: &stubBuilder{},
		Transactor: &bind.TransactOpts{}, Logger: silentLog(),
	})
	_, err := runner.ProveSet(context.Background(), big.NewInt(1))
	require.ErrorContains(t, err, "no challenge scheduled")
}

func TestRunner_ProveSet_ZeroLeaves(t *testing.T) {
	client := &stubChainClient{
		epoch: big.NewInt(50), totalLeaves: big.NewInt(0),
	}
	runner, _ := NewRunner(RunnerOptions{
		Client: client, Lookup: &stubLookup{}, Builder: &stubBuilder{},
		Transactor: &bind.TransactOpts{}, Logger: silentLog(),
	})
	_, err := runner.ProveSet(context.Background(), big.NewInt(1))
	require.ErrorContains(t, err, "zero leaves")
}

func TestRunner_ProveSet_SubmitError(t *testing.T) {
	client := &stubChainClient{
		seed: [32]byte{0x01}, epoch: big.NewInt(10), totalLeaves: big.NewInt(100),
		submitErr: errTestSubmit,
	}
	runner, _ := NewRunner(RunnerOptions{
		Client: client, Lookup: &stubLookup{}, Builder: &stubBuilder{},
		Transactor: &bind.TransactOpts{}, Logger: silentLog(),
	})
	_, err := runner.ProveSet(context.Background(), big.NewInt(1))
	require.ErrorContains(t, err, "submit")
}

func TestRunner_Requires(t *testing.T) {
	_, err := NewRunner(RunnerOptions{Client: nil})
	require.ErrorContains(t, err, "chain client")

	_, err = NewRunner(RunnerOptions{Client: &stubChainClient{}})
	require.ErrorContains(t, err, "piece lookup")

	_, err = NewRunner(RunnerOptions{Client: &stubChainClient{}, Lookup: &stubLookup{}})
	require.ErrorContains(t, err, "merkle builder")

	_, err = NewRunner(RunnerOptions{
		Client: &stubChainClient{}, Lookup: &stubLookup{}, Builder: &stubBuilder{},
	})
	require.ErrorContains(t, err, "transactor")
}

var errTestSubmit = &testErr{"rpc error"}
