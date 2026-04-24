// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Prova Network contributors.

package pdptree

import (
	"fmt"

	"github.com/ipfs/go-cid"
	"github.com/multiformats/go-multihash"

	"github.com/prova-network/prova/prover/pkg/challenges"
	"github.com/prova-network/prova/prover/pkg/store"
)

// StoreBackedBuilder implements the challenges.MerkleBuilder interface by
// loading pieces from a local blob store and building in-memory trees.
//
// For small/medium pieces (<1 GiB padded) this is fine. Larger pieces need
// an out-of-core builder (out of scope for v1).
type StoreBackedBuilder struct {
	pieces store.Store
}

// NewStoreBackedBuilder wraps a piece Store.
func NewStoreBackedBuilder(pieces store.Store) *StoreBackedBuilder {
	return &StoreBackedBuilder{pieces: pieces}
}

// BuildProof produces an inclusion proof for challengedLeaf within the piece
// identified by its CommP hash. The piece bytes are loaded from the store,
// fr32-padded, and hashed into a memtree; the proof is then extracted.
//
// Satisfies the challenges.MerkleBuilder interface.
func (b *StoreBackedBuilder) BuildProof(pieceCIDHash [32]byte, challengedLeaf uint64) (challenges.Proof, error) {
	pieceCid, err := commpCID(pieceCIDHash)
	if err != nil {
		return challenges.Proof{}, fmt.Errorf("build piece CID from hash: %w", err)
	}

	r, err := b.pieces.Get(pieceCid)
	if err != nil {
		return challenges.Proof{}, fmt.Errorf("get piece %s: %w", pieceCid, err)
	}
	defer r.Close()

	size, err := b.pieces.Size(pieceCid)
	if err != nil {
		return challenges.Proof{}, fmt.Errorf("size %s: %w", pieceCid, err)
	}

	memtree, err := BuildMemtree(r, size)
	if err != nil {
		return challenges.Proof{}, fmt.Errorf("build memtree: %w", err)
	}

	mproof, err := MemtreeProof(memtree, int64(challengedLeaf))
	if err != nil {
		return challenges.Proof{}, fmt.Errorf("extract proof for leaf %d: %w", challengedLeaf, err)
	}

	return challenges.Proof{
		Leaf:  mproof.Leaf,
		Proof: mproof.Proof,
	}, nil
}

// commpCID constructs a cid.Cid v1 for a 32-byte CommP hash using the
// canonical piece-commitment multicodec / multihash pair:
//
//   multihash code 0x1012 (sha2-256-trunc254-padded, 32 bytes)
//   codec code     0xf101 (piece-commitment)
//
// These identifiers come from the cross-ecosystem multicodec registry
// (https://github.com/multiformats/multicodec) and are not specific to
// any one chain.
func commpCID(hash [32]byte) (cid.Cid, error) {
	mh, err := multihash.Encode(hash[:], 0x1012)
	if err != nil {
		return cid.Undef, fmt.Errorf("encode multihash: %w", err)
	}
	return cid.NewCidV1(0xf101, mh), nil
}
