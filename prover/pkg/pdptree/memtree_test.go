// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Prova Network contributors.

package pdptree

import (
	"bytes"
	"crypto/sha256"
	"fmt"
	"io"
	"math/rand/v2"
	"testing"

	commp "github.com/filecoin-project/go-fil-commp-hashhash"
	"github.com/stretchr/testify/require"
)

// deterministicBytes produces n bytes from a fixed seed. Used as test input
// so the resulting CommP is deterministic and we can cross-check against
// commp.Calc.
func deterministicBytes(seed uint64, n int) []byte {
	r := rand.New(rand.NewPCG(seed, seed+1))
	out := make([]byte, n)
	for i := range out {
		out[i] = byte(r.UintN(256))
	}
	return out
}

// TestBuildMemtree_RootMatchesCommPCalc is the central correctness check.
// For several representative input sizes, we:
//   1. Run our BuildMemtree to produce a tree
//   2. Extract the root (last 32 bytes of the tree)
//   3. Run go-fil-commp-hashhash on the same input
//   4. Assert the roots are bit-identical
//
// If this passes for 3+ sizes, our fr32 + tree construction agrees with the
// canonical Filecoin implementation. If it fails, we have a bug before any
// on-chain proof can succeed.
func TestBuildMemtree_RootMatchesCommPCalc(t *testing.T) {
	// Pick sizes that are multiples of 127 (fr32 chunk size), span a range
	// of tree heights, and include zero-padding boundary cases.
	//
	// 127  bytes → 128 padded = 4 leaves (minimum)
	// 1016 bytes → 1024 padded = 32 leaves
	// 8128 bytes → 8192 padded = 256 leaves
	// 65024 → 65536 padded = 2048 leaves
	sizes := []int{127, 1016, 8128, 65024}

	for _, rawSize := range sizes {
		t.Run(
			fmt.Sprintf("rawSize=%d", rawSize),
			func(t *testing.T) {
				data := deterministicBytes(uint64(rawSize), rawSize)

				// Compute expected CommP via the canonical library
				var c commp.Calc
				_, err := c.Write(data)
				require.NoError(t, err)
				expected, expectedPadded, err := c.Digest()
				require.NoError(t, err)

				// Compute via our memtree
				tree, err := BuildMemtree(bytes.NewReader(data), uint64(rawSize))
				require.NoError(t, err, "BuildMemtree failed for rawSize=%d", rawSize)

				// Extract root: last 32 bytes of the tree buffer
				require.GreaterOrEqual(t, len(tree), NodeSize, "tree too small")
				ourRoot := tree[len(tree)-NodeSize:]

				require.Equal(t, expected, ourRoot,
					"rawSize=%d (padded=%d): our root %x != expected %x",
					rawSize, expectedPadded, ourRoot, expected)

				// Sanity: our computed padded size matches commp's
				ourPadded := paddedSize(uint64(rawSize))
				require.Equal(t, expectedPadded, ourPadded,
					"padded size mismatch: ours=%d, commp=%d", ourPadded, expectedPadded)
			})
	}
}

// TestMemtreeProof_Reconstruction confirms that MemtreeProof + ComputeBinShaParent
// can reconstruct the tree root from a leaf + its inclusion path. This is
// what a verifier (including the on-chain Proofs.sol) will do.
func TestMemtreeProof_Reconstruction(t *testing.T) {
	rawSize := 8128
	data := deterministicBytes(99, rawSize)

	tree, err := BuildMemtree(bytes.NewReader(data), uint64(rawSize))
	require.NoError(t, err)

	// Pick a few leaf indices to test
	nLeaves := int64(paddedSize(uint64(rawSize))) / NodeSize
	for _, leaf := range []int64{0, 1, 2, nLeaves / 2, nLeaves - 1} {
		proof, err := MemtreeProof(tree, leaf)
		require.NoError(t, err, "leaf=%d", leaf)

		// Reconstruct the root from leaf + path
		current := proof.Leaf
		idx := leaf
		for _, sibling := range proof.Proof {
			var pairInput [NodeSize * 2]byte
			if idx%2 == 0 {
				copy(pairInput[:NodeSize], current[:])
				copy(pairInput[NodeSize:], sibling[:])
			} else {
				copy(pairInput[:NodeSize], sibling[:])
				copy(pairInput[NodeSize:], current[:])
			}
			h := sha256.Sum256(pairInput[:])
			h[NodeSize-1] &= 0x3F // SHA-254 mask
			current = h
			idx /= 2
		}
		require.Equal(t, proof.Root, current,
			"leaf=%d: reconstructed root != proof.Root", leaf)
	}
}

// TestBuildMemtreeFromSnapshot_RoundTrips builds a tree from raw data,
// extracts the leaf layer, then rebuilds from the snapshot. Roots must match.
func TestBuildMemtreeFromSnapshot_RoundTrips(t *testing.T) {
	rawSize := 1016
	data := deterministicBytes(7, rawSize)

	// Build from raw
	tree1, err := BuildMemtree(bytes.NewReader(data), uint64(rawSize))
	require.NoError(t, err)

	padded := paddedSize(uint64(rawSize))
	nLeaves := int64(padded) / NodeSize

	// Extract leaves (level 0 = first nLeaves*32 bytes of the tree buffer)
	leaves := make([]byte, nLeaves*NodeSize)
	copy(leaves, tree1[:nLeaves*NodeSize])

	// Rebuild from snapshot
	tree2, err := BuildMemtreeFromSnapshot(leaves)
	require.NoError(t, err)

	// Roots must match
	root1 := tree1[len(tree1)-NodeSize:]
	root2 := tree2[len(tree2)-NodeSize:]
	require.Equal(t, root1, root2, "snapshot rebuild produced different root")
}

func TestPaddedSize(t *testing.T) {
	cases := []struct {
		raw      uint64
		expected uint64
	}{
		{127, 128},
		{1016, 1024},
		{1017, 2048}, // 1017 not divisible by 127; padded to 1024 + fr32 overhead → 2048
		{8128, 8192},
		{65024, 65536},
	}
	for _, tc := range cases {
		got := paddedSize(tc.raw)
		require.Equal(t, tc.expected, got,
			"paddedSize(%d): got %d, want %d", tc.raw, got, tc.expected)
	}
}

func TestComputeTotalNodes(t *testing.T) {
	// 4 leaves, arity 2: levels = [4, 2, 1], total = 7
	total, levels := computeTotalNodes(4, 2)
	require.Equal(t, int64(7), total)
	require.Equal(t, []int64{4, 2, 1}, levels)

	// 8 leaves: [8, 4, 2, 1], total = 15
	total, levels = computeTotalNodes(8, 2)
	require.Equal(t, int64(15), total)
	require.Equal(t, []int64{8, 4, 2, 1}, levels)

	// 1 leaf: [1], total = 1
	total, levels = computeTotalNodes(1, 2)
	require.Equal(t, int64(1), total)
	require.Equal(t, []int64{1}, levels)
}

func TestBuildMemtree_RejectsUnalignedRawSize(t *testing.T) {
	data := bytes.Repeat([]byte{0x42}, 100)
	_, err := BuildMemtree(bytes.NewReader(data), 100)
	require.ErrorContains(t, err, "must be a multiple of 127")
}

func TestBuildMemtree_RejectsZeroSize(t *testing.T) {
	_, err := BuildMemtree(bytes.NewReader(nil), 0)
	require.ErrorContains(t, err, "> 0")
}

func TestMemtreeProof_RejectsOutOfRange(t *testing.T) {
	data := deterministicBytes(1, 127)
	tree, err := BuildMemtree(bytes.NewReader(data), 127)
	require.NoError(t, err)

	_, err = MemtreeProof(tree, 100)
	require.ErrorContains(t, err, "invalid leaf index")
}

// Sanity helper: io.ReadFull must be available at compile time for our
// BuildMemtree caller path; this keeps the import graph honest.
var _ = io.ReadFull
