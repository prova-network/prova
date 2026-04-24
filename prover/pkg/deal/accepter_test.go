// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Prova Network contributors.

package deal

import (
	"context"
	"testing"

	"github.com/ethereum/go-ethereum/common"
	"github.com/stretchr/testify/require"
)

// mockWaiter is a stub ReceiptWaiter for tests.
type mockWaiter struct {
	res TxResult
	err error
}

func (m *mockWaiter) WaitReceiptInfo(_ context.Context, _ common.Hash) (TxResult, error) {
	return m.res, m.err
}

func TestNewOnChainAccepter_Validation(t *testing.T) {
	_, err := NewOnChainAccepter(OnChainAccepterOptions{})
	require.ErrorContains(t, err, "verifier binding required")
}

// TestTxResult_Shape is a trivial shape-check to lock the public types.
// The real Accepter behavior is exercised in the live anvil smoke-test
// since abi-encoding + the CreateDataSet binding need a real chain to be
// meaningful.
func TestTxResult_Shape(t *testing.T) {
	r := TxResult{
		OK: true,
		Logs: []TxLog{
			{Topics: []common.Hash{{0x01}, {0x02}, {0x03}}, Data: []byte{0xab}},
		},
	}
	require.True(t, r.OK)
	require.Len(t, r.Logs, 1)
	require.Equal(t, 3, len(r.Logs[0].Topics))
}
