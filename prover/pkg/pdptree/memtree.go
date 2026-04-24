// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright (c) 2024-2026 Filecoin Project contributors (upstream: filecoin-project/curio).
// Copyright (c) 2026 Prova Network contributors.
//
// This file is adapted from filecoin-project/curio lib/proof/merkle_sha254_memtree.go
// and merkle_proof_memtree.go (https://github.com/filecoin-project/curio).
// Originally under the Permissive License Stack (Apache-2.0 OR MIT).
// Attribution preserved per license.
//
// Adaptations for Prova:
//   - Inlined computeTotalNodes helper (was lib/proof/tree_size.go)
//   - Replaced minio/sha256-simd with stdlib crypto/sha256
//   - Replaced libp2p-buffer-pool with stdlib sync.Pool
//   - Stripped Filecoin-specific logging and go-state-types imports

// Package pdptree builds in-memory SHA-254 Merkle trees over fr32-padded
// piece data and extracts inclusion proofs for arbitrary leaves.
//
// "SHA-254" = SHA-256 with the top 2 bits of every node masked to zero, so
// every hash fits in an Fr element of the BLS12-381 scalar field. This is
// the canonical Filecoin piece-commitment tree.
//
// The package is an isolated implementation of the piece Merkle tree used
// by PDP. It does not depend on Lotus or Curio internals.
package pdptree

import (
	"crypto/sha256"
	"errors"
	"fmt"
	"io"
)

// NodeSize is the byte length of a Merkle tree node (32 bytes).
const NodeSize = 32

// MaxMemtreeSize is the largest padded piece size this package will build a
// tree for in memory. Roughly 1 GiB padded = 2 GiB tree. Anything larger
// needs an out-of-core tree builder (deferred).
const MaxMemtreeSize = 1 << 30

// BuildMemtree constructs a SHA-254 Merkle tree from unpadded piece data.
//
// Steps:
//  1. Read rawSize bytes from rawIn into a buffer
//  2. fr32-pad into the leaf region of the tree buffer (size.Padded())
//  3. Hash adjacent leaves pairwise up to a single root
//  4. Return the whole tree as a []byte (caller indexes into it via memtree
//     protocol, see MemtreeProof below)
//
// rawSize MUST be a multiple of 127 (the fr32 minimum chunk). Callers using
// non-aligned pieces must pad with zeros first. The padded piece size (result
// of applying the 127→128 expansion + rounding up to the next power of 2)
// MUST be a power of 2.
//
// On success, returns the memtree buffer. The exact layout is:
//
//	[level 0 (leaves, nLeaves * 32 bytes)] [level 1 (nLeaves/2 * 32)] ... [root (32 bytes)]
//
// Total size = (2*nLeaves - 1) * 32 bytes. MemtreeProof uses this layout.
func BuildMemtree(rawIn io.Reader, rawSize uint64) ([]byte, error) {
	if rawSize == 0 {
		return nil, errors.New("rawSize must be > 0")
	}
	if rawSize%UnpaddedFr32Chunk != 0 {
		return nil, fmt.Errorf("rawSize %d must be a multiple of %d", rawSize, UnpaddedFr32Chunk)
	}

	padded := paddedSize(rawSize)
	if padded > MaxMemtreeSize {
		return nil, fmt.Errorf("piece too large for in-memory tree: padded=%d, max=%d", padded, MaxMemtreeSize)
	}

	nLeaves := int64(padded) / NodeSize
	totalNodes, levelSizes := computeTotalNodes(nLeaves, 2)

	// Single buffer holds the full tree. Leaves come first (level 0), then
	// internal nodes in order of increasing level, then the root.
	memtree := make([]byte, totalNodes*NodeSize)

	// Read the raw data + fr32-pad directly into the leaf region.
	//
	// Upstream Curio uses a second buffer for the raw read then pads into
	// the memtree. We can do it in one buffer if the unpadded data fits in
	// the leaf region (it always will, since padded > raw).
	unpad := make([]byte, rawSize)
	if _, err := io.ReadFull(rawIn, unpad); err != nil {
		return nil, fmt.Errorf("read input: %w", err)
	}

	// fr32Pad expands rawSize bytes → rawSize*128/127 bytes into memtree[:].
	// Any remaining leaf space (memtree[fr32Out : padded]) stays zeroed, which
	// is the canonical behavior for short pieces padded up to the next power
	// of 2.
	fr32Out := (rawSize / UnpaddedFr32Chunk) * PaddedFr32Chunk
	fr32Pad(unpad, memtree[:fr32Out])

	// Hash levels upward.
	levelStarts := make([]int64, len(levelSizes))
	levelStarts[0] = 0
	for i := 1; i < len(levelSizes); i++ {
		levelStarts[i] = levelStarts[i-1] + levelSizes[i-1]*NodeSize
	}

	d := sha256.New()
	for level := 1; level < len(levelSizes); level++ {
		levelNodes := levelSizes[level]
		prevStart := levelStarts[level-1]
		currStart := levelStarts[level]

		for i := int64(0); i < levelNodes; i++ {
			leftOff := prevStart + (2*i)*NodeSize
			d.Reset()
			d.Write(memtree[leftOff : leftOff+NodeSize*2])

			outOff := currStart + i*NodeSize
			// d.Sum(x) appends the hash to x. Giving it a zero-length slice at
			// the target offset writes directly in place.
			d.Sum(memtree[outOff:outOff])

			// SHA-254: mask the top 2 bits of the last byte to keep the hash
			// in Fr (BLS12-381 scalar field).
			memtree[outOff+NodeSize-1] &= 0x3F
		}
	}

	return memtree, nil
}

// BuildMemtreeFromSnapshot builds a memtree from pre-computed leaf data
// (typically a snapshot layer from go-fil-commp-hashhash). The data must
// already be fr32-padded and aligned to leaf boundaries.
//
// Used when a snapshot layer has been saved from a previous CommP
// computation; avoids re-reading and re-padding the raw piece.
func BuildMemtreeFromSnapshot(data []byte) ([]byte, error) {
	if len(data) == 0 || len(data)%NodeSize != 0 {
		return nil, fmt.Errorf("snapshot data length %d must be positive and %d-byte aligned", len(data), NodeSize)
	}
	nLeaves := int64(len(data)) / NodeSize
	// Must be a power of 2
	if nLeaves&(nLeaves-1) != 0 {
		return nil, fmt.Errorf("snapshot leaf count %d is not a power of 2", nLeaves)
	}
	if int64(len(data)) > MaxMemtreeSize {
		return nil, fmt.Errorf("snapshot too large: %d bytes", len(data))
	}

	totalNodes, levelSizes := computeTotalNodes(nLeaves, 2)
	memtree := make([]byte, totalNodes*NodeSize)
	copy(memtree[:len(data)], data)

	levelStarts := make([]int64, len(levelSizes))
	levelStarts[0] = 0
	for i := 1; i < len(levelSizes); i++ {
		levelStarts[i] = levelStarts[i-1] + levelSizes[i-1]*NodeSize
	}

	d := sha256.New()
	for level := 1; level < len(levelSizes); level++ {
		levelNodes := levelSizes[level]
		prevStart := levelStarts[level-1]
		currStart := levelStarts[level]
		for i := int64(0); i < levelNodes; i++ {
			leftOff := prevStart + (2*i)*NodeSize
			d.Reset()
			d.Write(memtree[leftOff : leftOff+NodeSize*2])
			outOff := currStart + i*NodeSize
			d.Sum(memtree[outOff:outOff])
			memtree[outOff+NodeSize-1] &= 0x3F
		}
	}

	return memtree, nil
}

// MerkleProof is a Merkle inclusion proof for one leaf.
type MerkleProof struct {
	Leaf  [NodeSize]byte     // the challenged leaf
	Proof [][NodeSize]byte   // sibling hashes along the path to the root
	Root  [NodeSize]byte     // the computed tree root
}

// MemtreeProof extracts the inclusion proof for a single leaf from a tree
// produced by BuildMemtree or BuildMemtreeFromSnapshot.
func MemtreeProof(memtree []byte, leafIndex int64) (*MerkleProof, error) {
	if len(memtree) == 0 || len(memtree)%NodeSize != 0 {
		return nil, fmt.Errorf("memtree size %d must be positive and %d-byte aligned", len(memtree), NodeSize)
	}
	totalNodes := int64(len(memtree)) / NodeSize

	// Reconstruct level sizes from the total node count (arity=2)
	nLeaves := (totalNodes + 1) / 2
	levelSizes := []int64{}
	checkTotal := int64(0)
	curr := nLeaves
	for {
		levelSizes = append(levelSizes, curr)
		checkTotal += curr
		if curr == 1 {
			break
		}
		curr = (curr + 1) / 2
	}
	if checkTotal != totalNodes {
		return nil, errors.New("invalid memtree size; level reconstruction does not match")
	}

	levelStarts := make([]int64, len(levelSizes))
	var off int64
	for i, s := range levelSizes {
		levelStarts[i] = off
		off += s * NodeSize
	}

	if leafIndex < 0 || leafIndex >= levelSizes[0] {
		return nil, fmt.Errorf("invalid leaf index %d for %d leaves", leafIndex, levelSizes[0])
	}

	out := &MerkleProof{
		Proof: make([][NodeSize]byte, 0, len(levelSizes)-1),
	}

	leafOff := levelStarts[0] + leafIndex*NodeSize
	copy(out.Leaf[:], memtree[leafOff:leafOff+NodeSize])

	index := leafIndex
	for level := 0; level < len(levelSizes)-1; level++ {
		siblingIndex := index ^ 1
		siblingOff := levelStarts[level] + siblingIndex*NodeSize
		var sibling [NodeSize]byte
		copy(sibling[:], memtree[siblingOff:siblingOff+NodeSize])
		out.Proof = append(out.Proof, sibling)
		index /= 2
	}

	rootOff := levelStarts[len(levelSizes)-1]
	copy(out.Root[:], memtree[rootOff:rootOff+NodeSize])
	return out, nil
}

// computeTotalNodes returns (total node count, per-level node counts) for a
// binary tree with nLeaves leaves. Transplanted from curio/lib/proof/tree_size.go.
func computeTotalNodes(nLeaves, arity int64) (int64, []int64) {
	total := int64(0)
	levels := []int64{}
	curr := nLeaves
	for curr > 0 {
		levels = append(levels, curr)
		total += curr
		if curr == 1 {
			break
		}
		curr = (curr + arity - 1) / arity
	}
	return total, levels
}

// paddedSize returns the fr32-padded piece size for a raw input of rawSize
// bytes, rounded up to the next power of 2. Matches the Filecoin convention
// used by go-fil-commp-hashhash.
//
// Minimum valid piece size is 128 bytes (127 raw, fr32-padded).
func paddedSize(rawSize uint64) uint64 {
	// fr32 expansion: 127 bytes in → 128 bytes out
	paddedExact := (rawSize / UnpaddedFr32Chunk) * PaddedFr32Chunk
	if rawSize%UnpaddedFr32Chunk != 0 {
		paddedExact += PaddedFr32Chunk
	}

	// Round up to next power of 2. Minimum is 128.
	if paddedExact < 128 {
		return 128
	}
	return nextPow2(paddedExact)
}

func nextPow2(n uint64) uint64 {
	if n == 0 {
		return 1
	}
	// Count leading zeros, then round up
	if n&(n-1) == 0 {
		return n
	}
	return 1 << (64 - leadingZeros64(n))
}

func leadingZeros64(x uint64) uint {
	// Inline to avoid pulling in math/bits; small enough to be readable.
	n := uint(64)
	if x >= 1<<32 {
		x >>= 32
		n -= 32
	}
	if x >= 1<<16 {
		x >>= 16
		n -= 16
	}
	if x >= 1<<8 {
		x >>= 8
		n -= 8
	}
	if x >= 1<<4 {
		x >>= 4
		n -= 4
	}
	if x >= 1<<2 {
		x >>= 2
		n -= 2
	}
	if x >= 1<<1 {
		n -= 1
	}
	if x == 0 {
		return 64
	}
	return n - 1
}

