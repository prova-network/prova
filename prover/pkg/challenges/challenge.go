// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Prova Network contributors.

// Package challenges computes and submits PDP proofs in response to
// on-chain challenges.
//
// Flow per proving period:
//
//  1. Poller observes NextProvingPeriod event for a data set we own.
//  2. We read the challenge seed via ProofVerifier.getRandomness(epoch).
//  3. We compute N challenge leaf indices using the seed, data-set ID, and
//     proof index, modulo the total leaf count.
//  4. For each challenge index we look up which piece contains it, build
//     a Merkle inclusion proof using the piece tree, and package as an
//     IPDPTypes.Proof{Leaf, []bytes32 Proof}.
//  5. We submit the batch via ProofVerifier.provePossession(setId, proofs).
//
// The challenge index algorithm matches Filecoin PDP exactly
// (keccak256(seed || uint256(dataSetID) || uint64(proofIndex)) mod
// totalLeaves), so provers built against Filecoin PDPVerifier continue to
// work with Prova's forked ProofVerifier.
//
// Adapted from filecoin-project/curio tasks/pdp/task_prove.go, simplified
// for a single-prover, single-binary deployment with no harmonydb.
package challenges

import (
	"encoding/binary"
	"fmt"
	"math/big"

	"golang.org/x/crypto/sha3"
)

// LeafSize is the canonical PDP leaf size in bytes. Every PDP tree uses
// 32-byte leaves regardless of piece size.
const LeafSize = 32

// ChallengeIndex computes the leaf index a single challenge hits.
//
// Matches Filecoin PDPVerifier convention: challenge = keccak256(seed ||
// uint256(dataSetID) || uint64(proofIndex)) mod totalLeaves.
//
// Arguments:
//
//   - seed: 32-byte randomness, usually from ProofVerifier.getRandomness().
//   - dataSetID: the on-chain data set id.
//   - proofIndex: position within the batch of challenges for this period.
//   - totalLeaves: the data set's challenge range (leaf count).
//
// Returns the challenged leaf index. Caller is responsible for mapping
// the leaf back to a piece via ProofVerifier.findPieceIds().
func ChallengeIndex(seed [32]byte, dataSetID *big.Int, proofIndex uint64, totalLeaves uint64) uint64 {
	if totalLeaves == 0 {
		return 0
	}

	// 32 bytes seed + 32 bytes dataSetID + 8 bytes proofIndex = 72 bytes
	buf := make([]byte, 0, 72)
	buf = append(buf, seed[:]...)
	buf = append(buf, pad32Left(dataSetID.Bytes())...)
	var proofIdxBuf [8]byte
	binary.BigEndian.PutUint64(proofIdxBuf[:], proofIndex)
	buf = append(buf, proofIdxBuf[:]...)

	h := sha3.NewLegacyKeccak256()
	h.Write(buf)
	hashed := h.Sum(nil)

	hashInt := new(big.Int).SetBytes(hashed)
	mod := new(big.Int).SetUint64(totalLeaves)
	idx := new(big.Int).Mod(hashInt, mod)
	return idx.Uint64()
}

// ChallengeIndices computes a batch of challenge indices for one proving
// period. Standard N for Filecoin PDP is 3-10; the exact value comes from
// the ProofVerifier config per deployment.
func ChallengeIndices(seed [32]byte, dataSetID *big.Int, n uint64, totalLeaves uint64) []uint64 {
	out := make([]uint64, n)
	for i := uint64(0); i < n; i++ {
		out[i] = ChallengeIndex(seed, dataSetID, i, totalLeaves)
	}
	return out
}

// pad32Left left-pads the input bytes to 32 bytes (big-endian numeric).
func pad32Left(b []byte) []byte {
	if len(b) >= 32 {
		return b[len(b)-32:]
	}
	out := make([]byte, 32)
	copy(out[32-len(b):], b)
	return out
}

// Proof is a single challenge response: a leaf value and the Merkle path
// (as 32-byte hashes) needed to reconstruct the root.
//
// This matches the on-chain IPDPTypes.Proof struct exactly so it serialises
// 1:1 for the provePossession call.
type Proof struct {
	Leaf  [32]byte
	Proof [][32]byte
}

// MerkleBuilder is the interface a piece-storage backend must provide so
// the challenge handler can generate proofs without knowing how pieces
// are laid out.
//
// An implementation typically:
//   - Loads the piece bytes from the local blob store
//   - Pads the piece to the next power-of-two leaf count
//   - Constructs a SHA2-254 truncated-padded binary Merkle tree
//   - Navigates to the requested leaf and emits the inclusion path
//
// The Merkle tree construction itself is intentionally not in this
// package (it's large, specialised, and benefits from being a shared
// library across prover + verification tooling). See the pdp-primitives
// package (planned) for the default implementation.
type MerkleBuilder interface {
	// BuildProof produces an inclusion proof for challengedLeaf in the
	// piece identified by pieceCID. The returned Proof is ready for
	// on-chain submission.
	BuildProof(pieceCIDHash [32]byte, challengedLeaf uint64) (Proof, error)
}

// PieceLookup resolves "which piece holds leaf X in data set D?" against
// the on-chain ProofVerifier. Split as an interface so tests can stub it
// without mocking the full contract binding.
type PieceLookup interface {
	// FindPieceForLeaves returns (pieceCID, offsetWithinPiece) for each
	// challenged leaf. The returned slice is parallel to challengedLeaves.
	FindPieceForLeaves(dataSetID *big.Int, challengedLeaves []uint64) ([]PieceHit, error)
}

// PieceHit pairs a piece id (or CID) with the leaf offset within that piece.
type PieceHit struct {
	PieceID       *big.Int // on-chain piece id within the data set
	PieceCIDHash  [32]byte // 32-byte CommP hash; empty if lookup returns only id
	LeafInPiece   uint64   // offset (in leaves) into the piece
}

// GenerateProofs produces all N proofs for one proving period.
//
// Caller is responsible for submitting them via ProofVerifier.provePossession.
// This function is pure computation + IO via the two interfaces; no chain
// writes happen here.
func GenerateProofs(
	seed [32]byte,
	dataSetID *big.Int,
	numChallenges uint64,
	totalLeaves uint64,
	lookup PieceLookup,
	builder MerkleBuilder,
) ([]Proof, error) {
	if numChallenges == 0 {
		return nil, fmt.Errorf("numChallenges must be > 0")
	}
	if totalLeaves == 0 {
		return nil, fmt.Errorf("totalLeaves must be > 0")
	}
	if lookup == nil || builder == nil {
		return nil, fmt.Errorf("lookup and builder are required")
	}

	indices := ChallengeIndices(seed, dataSetID, numChallenges, totalLeaves)

	hits, err := lookup.FindPieceForLeaves(dataSetID, indices)
	if err != nil {
		return nil, fmt.Errorf("find pieces: %w", err)
	}
	if uint64(len(hits)) != numChallenges {
		return nil, fmt.Errorf("lookup returned %d hits, expected %d", len(hits), numChallenges)
	}

	proofs := make([]Proof, numChallenges)
	for i, hit := range hits {
		p, err := builder.BuildProof(hit.PieceCIDHash, hit.LeafInPiece)
		if err != nil {
			return nil, fmt.Errorf("build proof %d (piece=%d leaf=%d): %w",
				i, hit.PieceID.Int64(), hit.LeafInPiece, err)
		}
		proofs[i] = p
	}

	return proofs, nil
}


