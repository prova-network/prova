// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Prova Network contributors.

package deal

import (
	"testing"

	"github.com/ethereum/go-ethereum/common"
	"github.com/stretchr/testify/require"
)

func TestMemStore_UpsertGetList(t *testing.T) {
	s := NewMemStore()

	d1 := &Deal{ID: DealID(1), Status: StatusProposed, Prover: common.HexToAddress("0x1")}
	d2 := &Deal{ID: DealID(2), Status: StatusActive, Prover: common.HexToAddress("0x1")}
	d3 := &Deal{ID: DealID(3), Status: StatusProposed, Prover: common.HexToAddress("0x1")}

	require.NoError(t, s.Upsert(d1))
	require.NoError(t, s.Upsert(d2))
	require.NoError(t, s.Upsert(d3))

	got, err := s.Get(DealID(1))
	require.NoError(t, err)
	require.NotNil(t, got)
	require.Equal(t, StatusProposed, got.Status)

	missing, err := s.Get(DealID(999))
	require.NoError(t, err)
	require.Nil(t, missing)

	proposed, err := s.ListByStatus(StatusProposed)
	require.NoError(t, err)
	require.Len(t, proposed, 2)

	active, err := s.ListByStatus(StatusActive)
	require.NoError(t, err)
	require.Len(t, active, 1)

	all, err := s.ListAll()
	require.NoError(t, err)
	require.Len(t, all, 3)
}

func TestMemStore_UpsertReplaces(t *testing.T) {
	s := NewMemStore()
	d := &Deal{ID: DealID(1), Status: StatusProposed}
	require.NoError(t, s.Upsert(d))

	// Modify locally; upsert should overwrite the stored copy
	d.Status = StatusActive
	require.NoError(t, s.Upsert(d))

	got, err := s.Get(DealID(1))
	require.NoError(t, err)
	require.Equal(t, StatusActive, got.Status)
}

func TestMemStore_DefensiveCopy(t *testing.T) {
	// Mutating a Deal retrieved via Get must not mutate the store's copy.
	s := NewMemStore()
	d := &Deal{ID: DealID(1), Status: StatusProposed, StatusMsg: "first"}
	require.NoError(t, s.Upsert(d))

	got, _ := s.Get(DealID(1))
	got.StatusMsg = "mutated externally"

	again, _ := s.Get(DealID(1))
	require.Equal(t, "first", again.StatusMsg, "external mutation must not leak into store")
}

func TestMemStore_LastSeenBlock(t *testing.T) {
	s := NewMemStore()
	blk, err := s.LastSeenBlock()
	require.NoError(t, err)
	require.Zero(t, blk)

	require.NoError(t, s.SetLastSeenBlock(12345))
	blk, err = s.LastSeenBlock()
	require.NoError(t, err)
	require.Equal(t, uint64(12345), blk)
}

func TestDeal_IsTerminal(t *testing.T) {
	cases := []struct {
		s    Status
		term bool
	}{
		{StatusProposed, false},
		{StatusDownloading, false},
		{StatusVerifying, false},
		{StatusAccepting, false},
		{StatusActive, false},
		{StatusCompleted, true},
		{StatusCancelled, true},
		{StatusSlashed, true},
		{StatusFailed, true},
	}
	for _, tc := range cases {
		d := &Deal{Status: tc.s}
		require.Equal(t, tc.term, d.IsTerminal(), tc.s)
	}
}
