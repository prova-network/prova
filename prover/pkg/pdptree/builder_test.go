// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Prova Network contributors.

package pdptree

import (
	"bytes"
	"crypto/sha256"
	"testing"

	commp "github.com/filecoin-project/go-fil-commp-hashhash"
	"github.com/stretchr/testify/require"

	"github.com/prova-network/prova/prover/pkg/store"
)

// TestStoreBackedBuilder_FullFlow stores a piece, asks the builder for a
// proof, and verifies the proof reconstructs the CommP root.
//
// This is the happy-path end-to-end for Phase D + Phase D.2: if it
// passes, a real prover with this piece can satisfy an on-chain
// challenge targeting any leaf of it.
func TestStoreBackedBuilder_FullFlow(t *testing.T) {
	rawSize := 8128
	data := deterministicBytes(777, rawSize)

	// Compute canonical CommP to know what hash our piece is addressed by
	var c commp.Calc
	_, err := c.Write(data)
	require.NoError(t, err)
	commP, paddedSize, err := c.Digest()
	require.NoError(t, err)
	require.Equal(t, uint64(8192), paddedSize)

	var commpHash [32]byte
	copy(commpHash[:], commP)

	// Stash the piece in a DiskStore addressed by its CommP CID.
	tmpDir := t.TempDir()
	ps, err := store.NewDiskStore(tmpDir)
	require.NoError(t, err)
	defer ps.Close()

	pieceCid, err := commpCID(commpHash)
	require.NoError(t, err)

	n, err := ps.Put(pieceCid, bytes.NewReader(data))
	require.NoError(t, err)
	require.Equal(t, uint64(rawSize), n)

	// Ask the builder for a proof for a specific leaf
	builder := NewStoreBackedBuilder(ps)

	// Leaf at position 100 (well within the 256-leaf tree for 8192 padded bytes)
	const leafIndex = 100
	proof, err := builder.BuildProof(commpHash, leafIndex)
	require.NoError(t, err)
	require.NotZero(t, proof.Leaf)
	require.NotEmpty(t, proof.Proof)

	// Verify: walking the proof from leaf must yield the canonical CommP
	// root. This is exactly what on-chain Proofs.sol does.
	current := proof.Leaf
	idx := int64(leafIndex)
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
		h[NodeSize-1] &= 0x3F
		current = h
		idx /= 2
	}

	// current now holds the reconstructed root; must equal CommP
	require.Equal(t, commpHash, current,
		"reconstructed root from proof path != canonical CommP")
}
