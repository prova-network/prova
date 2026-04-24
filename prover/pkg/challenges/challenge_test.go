// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Prova Network contributors.

package challenges

import (
	"math/big"
	"testing"

	"github.com/stretchr/testify/require"
)

func TestChallengeIndex_Deterministic(t *testing.T) {
	seed := [32]byte{
		0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08,
		0x09, 0x0a, 0x0b, 0x0c, 0x0d, 0x0e, 0x0f, 0x10,
		0x11, 0x12, 0x13, 0x14, 0x15, 0x16, 0x17, 0x18,
		0x19, 0x1a, 0x1b, 0x1c, 0x1d, 0x1e, 0x1f, 0x20,
	}
	dataSetID := big.NewInt(42)
	totalLeaves := uint64(1024)

	// Same inputs must always produce the same output.
	idx1 := ChallengeIndex(seed, dataSetID, 0, totalLeaves)
	idx2 := ChallengeIndex(seed, dataSetID, 0, totalLeaves)
	require.Equal(t, idx1, idx2, "challenge index must be deterministic")

	// Must be within [0, totalLeaves)
	require.Less(t, idx1, totalLeaves)
}

func TestChallengeIndex_DifferentProofIndicesDiffer(t *testing.T) {
	seed := [32]byte{0xaa}
	dataSetID := big.NewInt(1)
	totalLeaves := uint64(1_000_000)

	idxs := make(map[uint64]struct{})
	for i := uint64(0); i < 10; i++ {
		idxs[ChallengeIndex(seed, dataSetID, i, totalLeaves)] = struct{}{}
	}
	// With 1M leaves and 10 trials the probability of any collision is ~0.
	require.GreaterOrEqual(t, len(idxs), 8, "challenge indices should be well-distributed")
}

func TestChallengeIndex_DifferentDataSetsDiffer(t *testing.T) {
	seed := [32]byte{0xbb}
	totalLeaves := uint64(1_000_000)

	idx1 := ChallengeIndex(seed, big.NewInt(1), 0, totalLeaves)
	idx2 := ChallengeIndex(seed, big.NewInt(2), 0, totalLeaves)
	require.NotEqual(t, idx1, idx2,
		"same seed + different dataset id should produce different challenge indices")
}

func TestChallengeIndex_ZeroLeavesReturnsZero(t *testing.T) {
	seed := [32]byte{}
	idx := ChallengeIndex(seed, big.NewInt(0), 0, 0)
	require.Zero(t, idx)
}

func TestChallengeIndex_BoundedByTotalLeaves(t *testing.T) {
	seed := [32]byte{0xff, 0xff, 0xff, 0xff}
	dataSetID := big.NewInt(9999)
	totalLeaves := uint64(7) // small modulus, force mod to apply

	for i := uint64(0); i < 100; i++ {
		idx := ChallengeIndex(seed, dataSetID, i, totalLeaves)
		require.Less(t, idx, totalLeaves,
			"challenge index %d for proofIndex=%d exceeded totalLeaves %d", idx, i, totalLeaves)
	}
}

func TestChallengeIndices_Count(t *testing.T) {
	seed := [32]byte{0xcc}
	dataSetID := big.NewInt(7)
	totalLeaves := uint64(10_000)

	indices := ChallengeIndices(seed, dataSetID, 5, totalLeaves)
	require.Len(t, indices, 5)
	for _, i := range indices {
		require.Less(t, i, totalLeaves)
	}
}

func TestPad32Left(t *testing.T) {
	// len(b) < 32: left-pads with zeros
	b := []byte{0xde, 0xad}
	out := pad32Left(b)
	require.Len(t, out, 32)
	require.Zero(t, out[0])
	require.Zero(t, out[29])
	require.Equal(t, byte(0xde), out[30])
	require.Equal(t, byte(0xad), out[31])

	// len(b) == 32: unchanged
	in32 := make([]byte, 32)
	for i := range in32 {
		in32[i] = byte(i)
	}
	out32 := pad32Left(in32)
	require.Len(t, out32, 32)
	require.Equal(t, in32, out32)

	// len(b) > 32: takes rightmost 32 (big-endian numeric convention)
	in40 := make([]byte, 40)
	in40[39] = 0xff
	out40 := pad32Left(in40)
	require.Len(t, out40, 32)
	require.Equal(t, byte(0xff), out40[31])
}

// --- GenerateProofs wiring tests ---

type stubLookup struct {
	hits    []PieceHit
	callErr error
}

func (s *stubLookup) FindPieceForLeaves(dataSetID *big.Int, leaves []uint64) ([]PieceHit, error) {
	if s.callErr != nil {
		return nil, s.callErr
	}
	if len(s.hits) == 0 {
		// produce one hit per leaf with a recognizable fake
		out := make([]PieceHit, len(leaves))
		for i, leaf := range leaves {
			out[i] = PieceHit{
				PieceID:      big.NewInt(int64(i + 1)),
				PieceCIDHash: [32]byte{byte(i + 1)},
				LeafInPiece:  leaf,
			}
		}
		return out, nil
	}
	return s.hits, nil
}

type stubBuilder struct {
	lastPieceCID  [32]byte
	lastLeaf      uint64
	buildErr      error
	callCount     int
}

func (b *stubBuilder) BuildProof(pieceCIDHash [32]byte, leaf uint64) (Proof, error) {
	b.callCount++
	b.lastPieceCID = pieceCIDHash
	b.lastLeaf = leaf
	if b.buildErr != nil {
		return Proof{}, b.buildErr
	}
	// Return a recognizable stub proof
	var p Proof
	p.Leaf = [32]byte{0xab, byte(leaf)}
	p.Proof = [][32]byte{
		{0xcd, byte(leaf)},
	}
	return p, nil
}

func TestGenerateProofs_HappyPath(t *testing.T) {
	seed := [32]byte{0x11}
	dataSetID := big.NewInt(1)
	lookup := &stubLookup{}
	builder := &stubBuilder{}

	proofs, err := GenerateProofs(seed, dataSetID, 3, 1024, lookup, builder)
	require.NoError(t, err)
	require.Len(t, proofs, 3)
	require.Equal(t, 3, builder.callCount)

	// Each proof should be the stubBuilder output
	for _, p := range proofs {
		require.Equal(t, byte(0xab), p.Leaf[0])
		require.Len(t, p.Proof, 1)
	}
}

func TestGenerateProofs_ZeroChallengesFails(t *testing.T) {
	_, err := GenerateProofs([32]byte{}, big.NewInt(1), 0, 100, &stubLookup{}, &stubBuilder{})
	require.ErrorContains(t, err, "numChallenges")
}

func TestGenerateProofs_ZeroLeavesFails(t *testing.T) {
	_, err := GenerateProofs([32]byte{}, big.NewInt(1), 3, 0, &stubLookup{}, &stubBuilder{})
	require.ErrorContains(t, err, "totalLeaves")
}

func TestGenerateProofs_LookupErrorPropagates(t *testing.T) {
	lookup := &stubLookup{callErr: errTestLookup}
	_, err := GenerateProofs([32]byte{}, big.NewInt(1), 3, 100, lookup, &stubBuilder{})
	require.ErrorContains(t, err, "find pieces")
}

func TestGenerateProofs_BuilderErrorPropagates(t *testing.T) {
	builder := &stubBuilder{buildErr: errTestBuilder}
	_, err := GenerateProofs([32]byte{}, big.NewInt(1), 3, 100, &stubLookup{}, builder)
	require.ErrorContains(t, err, "build proof")
}

func TestGenerateProofs_CountMismatchFails(t *testing.T) {
	// Lookup returns fewer hits than challenges requested
	lookup := &stubLookup{hits: []PieceHit{
		{PieceID: big.NewInt(1), LeafInPiece: 0},
	}}
	_, err := GenerateProofs([32]byte{}, big.NewInt(1), 3, 100, lookup, &stubBuilder{})
	require.ErrorContains(t, err, "returned 1 hits, expected 3")
}

var errTestLookup = &testErr{"chain unreachable"}
var errTestBuilder = &testErr{"piece not in local store"}

type testErr struct{ msg string }

func (e *testErr) Error() string { return e.msg }
