# SPEC-004: PDP Integration

**Status:** Draft v2
**Updated:** 2026-04-24

## 1. Overview

Prova uses **Provable Data Possession (PDP)** as its single storage
verification mechanism. Clients upload content, provers store the raw
bytes, and the chain periodically challenges provers to supply Merkle
inclusion proofs against random leaves.

PDP was chosen because:

- **Lightweight to onboard** (minutes, not hours).
- **Cheap to verify on-chain** (O(log N) inclusion proofs, low gas).
- **Hot/warm friendly** — pieces stay unsealed, retrievable on demand.
- **Mature** — the underlying cryptography has been deployed at scale.

Anything heavier (sealed replicas, SNARK-based proofs, TEE-attested
storage, proof-of-replication) is **out of scope**. Prova is not a
Filecoin replacement; it is a thin verifiable-storage layer on Base
optimized for data that needs to stay retrievable.

## 2. Content Addressing: CommP

Content is identified by **CommP** (Piece Commitment):

- Multihash: `sha2-256-trunc254-padded` (code `0x1012`)
- Codec: `piece-commitment` (code `0xf101`)
- Binary Merkle tree over 32-byte leaves, top 2 bits of each node masked
  to stay inside the BLS12-381 scalar field (Fr).
- Piece sizes: powers of 2, 128 bytes to 64 GiB.

CommP is standard across the multicodec registry and compatible with any
tooling that understands the same codec. Clients can compute CommP
themselves (see `@prova-network/core`) without trusting the prover.

## 3. On-Chain Data Model

### Data sets

Each deal creates one **data set** inside the `ProofVerifier` contract.
A data set is a collection of one or more pieces that the prover is
responsible for. The `StorageMarketplace` is registered as the
`PDPListener` for its data sets, so lifecycle callbacks
(`dataSetCreated`, `possessionProven`, etc.) route through it.

```
Deal proposed (client)
  → Marketplace.proposeDeal(prover, commP, pieceSize, duration, payment)
  → funds locked in escrow

Prover accepts
  → Prover calls ProofVerifier.createDataSet(marketplace, dealId)
  → Marketplace.dataSetCreated hook flips deal to Active
```

### Proof set state

```solidity
mapping(uint256 => uint256) nextChallengeEpoch;   // when next challenge is sampled
mapping(uint256 => uint256) challengeRange;        // total leaf count
mapping(uint256 => mapping(uint256 => uint256)) pieceLeafCounts;
```

## 4. Challenge Protocol

### 4.1 Randomness source

The seed for challenge leaf selection comes from `block.prevrandao`
(EIP-4399). Base supports it. Real deployments may plug in Chainlink VRF
for stronger bias resistance; the current path is acceptable for Base
where the reorg horizon is effectively instant after the L1 batch is
posted.

### 4.2 Challenge index derivation

For the N challenged leaves of a proving period:

```
challengeIndex[i] = keccak256(seed || uint256(dataSetId) || uint64(i)) mod totalLeaves
```

Bit-for-bit compatible with the canonical PDP convention. Verified by
`prover/pkg/challenges` against a reference implementation on multiple
test vectors.

### 4.3 Proof submission

Prover calls:

```solidity
ProofVerifier.provePossession(setId, IPDPTypes.Proof[] proofs)
```

Each `Proof` is `{ bytes32 leaf, bytes32[] path }`. The verifier walks
the path, reconstructs the root, and compares to the on-chain CommP.
Gas cost is **O(log N)** per proof, which is the key reason PDP scales.

### 4.4 Sybil fee

`createDataSet` and new-dataset `addPieces` charge a flat sybil fee of
`0.1 ETH` (burned) to deter wasteful on-chain state growth. Regular
proof submissions carry no protocol fee; only the deal-level economic
flow (protocol fee on released payment) applies.

## 5. Proving Schedule

Default parameters (tunable via `StorageMarketplace` admin):

| Parameter | Value | Notes |
|-----------|-------|-------|
| Challenge frequency | 1 per deal per day | Enforced via `nextChallengeEpoch` |
| Max proof gap | 3 days | Anyone can `faultDeal` after this |
| Unbonding period | 14 days | Prover stake cannot exit faster |
| Slash per fault | `slashPerFault` (configurable) | Current default: 50 PROVA |

A missed challenge does not slash immediately — the **dispute path** is
explicit. Anyone can call `StorageMarketplace.faultDeal(dealId)` once
the `MAX_PROOF_GAP` window elapses; the transaction slashes the prover
and refunds the client's unreleased escrow.

## 6. Piece Retrieval

Provers that advertise `FEATURE_HTTPS_SERVING` expose:

```
GET  https://<prover>/piece/<pieceCid>     — stream the bytes
HEAD https://<prover>/piece/<pieceCid>     — metadata only
GET  https://<prover>/.well-known/prova    — prover metadata
GET  https://<prover>/health               — liveness
```

Retrieval is out-of-band to the on-chain proof flow; it has its own
rate limiting and pricing (in a future phase). For v1 retrieval is open
and unpriced.

## 7. Staking and Slashing

Prover stake (`ProverStaking.stake`) is the only economic guarantee.
`StorageMarketplace` calls `staking.commitBytes(prover, pieceSize)` on
deal acceptance and `staking.releaseBytes` on completion or fault.
Minimum stake is proportional to committed bytes; falling below the
floor blocks new deal acceptance.

## 8. Gas Costs (ballpark on Base)

These are approximate; real deployments will benchmark and tune.

| Operation | Gas | USDC-equivalent on Base |
|-----------|----:|------------------------:|
| `createDataSet` (1 piece) | ~400K | ~$0.001 |
| `createDataSet` (100 pieces) | ~800K | ~$0.002 |
| `provePossession` (5 challenges) | ~300K | ~$0.001 |
| `addPieces` (existing dataset) | ~150K | ~$0.0005 |

On Ethereum L1 these numbers are about 100x higher, which is why Prova
targets Base by default.

## 9. References

- [`contracts/src/ProofVerifier.sol`](../contracts/src/ProofVerifier.sol) — the on-chain verifier
- [`contracts/src/StorageMarketplace.sol`](../contracts/src/StorageMarketplace.sol) — PDPListener implementation
- [`prover/pkg/pdptree/`](../prover/pkg/pdptree/) — fr32 + SHA-254 memtree (piece-side Merkle)
- [`prover/pkg/challenges/`](../prover/pkg/challenges/) — challenge index derivation + proof submission
- Multicodec registry: <https://github.com/multiformats/multicodec>
