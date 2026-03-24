# Data Availability Specification

**Status:** Draft  
**Authors:** Capri (AI), Nicklas Reiers  
**Created:** 2026-03-04  
**Implements:** SPEC-019

## 1. Overview

Prova's data availability (DA) layer ensures that inference inputs, outputs, and model activations committed on-chain are retrievable by any validator or challenger. Without DA guarantees, the dispute system cannot function — a malicious provider could commit results but withhold the underlying data, making challenges impossible.

Prova uses **Data Availability Sampling (DAS)** with erasure coding to provide probabilistic DA guarantees with O(√n) sample overhead for O(n) data, combined with **blob transactions** that embed data references directly in the chain.

## 2. Threat Model

| Threat | Impact | Mitigation |
|--------|--------|------------|
| Data withholding | Dispute system unusable | DAS with multi-round sampling |
| Selective withholding | Only some chunks missing | Erasure coding (50% redundancy) |
| Collusion among validators | False confirmation | Minimum validator quorum + independent sampling |
| Late availability | Data published after challenge window | Response deadline with penalty |
| Blob spam | Chain bloat, storage exhaustion | Blob-specific fee market, size limits, pruning |

## 3. Erasure Coding

### 3.1 Scheme

Data is split into `ORIGINAL_CHUNKS` (64) chunks, then extended to `TOTAL_CHUNKS` (128) via parity chunks (XOR-based Reed-Solomon simulation). This provides **2× extension factor** — any 64 of 128 chunks can reconstruct the original data.

### 3.2 Chunk Structure

```
Original data (arbitrary bytes)
  → Split into 64 equal-sized chunks (zero-padded if needed)
  → Erasure-encode to 128 chunks
  → Each chunk hashed: H("das-leaf:" || index_le || data)
  → Merkle tree built: H("das-node:" || left || right)
  → Root = data_root committed on-chain
```

### 3.3 Parameters

| Parameter | Value | Rationale |
|-----------|-------|-----------|
| `TOTAL_CHUNKS` | 128 | Balances proof size vs reconstruction threshold |
| `ORIGINAL_CHUNKS` | 64 | 50% redundancy — tolerates up to 50% missing |
| Max chunk size | 256 KiB | Keeps P2P messages manageable |
| Max blob size | 16 MiB (64 × 256 KiB) | Sufficient for model activations |

## 4. DAS Protocol

### 4.1 Commitment Phase

1. Provider erasure-encodes data into 128 chunks
2. Builds Merkle tree over chunk hashes → `data_root`
3. Submits `DasCommitment { blob_id, provider, data_root, chunk_count }` on-chain
4. Status: `Pending`

### 4.2 Sampling Phase

For each pending commitment, validators execute `REQUIRED_ROUNDS` (3) sampling rounds:

1. **Challenge generation:** Using epoch randomness + round number, derive `SAMPLES_PER_ROUND` (16) pseudo-random chunk indices:
   ```
   For i in 0..16:
     h = SHA256(randomness || round_le || i_le)
     index = u64_le(h[0..8]) % chunk_count
   ```

2. **Request:** Validators send P2P `SampleRequest` to the provider for the selected indices

3. **Response:** Provider must return `ChunkProof { index, data, merkle_proof }` for each index within `RESPONSE_WINDOW` (5 epochs)

4. **Verification:** Validator checks each proof:
   - Compute `leaf_hash = H("das-leaf:" || index_le || data)`
   - Verify Merkle inclusion against `data_root`
   - All proofs valid → round passes

5. After `REQUIRED_ROUNDS` successful rounds → status transitions to `Confirmed`

### 4.3 Failure Handling

- **Timeout:** If provider fails to respond within `RESPONSE_WINDOW`, any validator can call `check_expired_challenges()` → status transitions to `Failed`, provider penalized `DAS_PENALTY` (500 stake units)
- **Invalid proof:** Merkle verification failure → round fails, provider must be re-challenged
- **Negative quorum:** `NEGATIVE_QUORUM` (3) independent validators must report timeout before penalizing (prevents single malicious validator from triggering false penalties)

### 4.4 Probabilistic Guarantees

With 16 samples per round and 3 rounds (48 total samples from 128 chunks):

- Probability of missing a withholding attack where ≥50% of chunks are unavailable: `(0.5)^48 ≈ 3.5 × 10⁻¹⁵`
- Even with 25% withholding: `(0.75)^48 ≈ 6.6 × 10⁻⁷`

This provides overwhelming confidence that confirmed blobs are fully reconstructable.

## 5. Blob Transactions

### 5.1 Purpose

Blob transactions are a first-class transaction type that allows submitting data references alongside inference commits. They link inference execution to verifiable data roots.

### 5.2 Transaction Format

```rust
BlobTransaction {
    sender: Address,
    nonce: u64,
    blob_id: BlobId,           // SHA-256 of original data
    data_root: Hash,           // Merkle root of erasure-coded chunks
    blob_size: u64,            // Original data size in bytes
    chunk_count: usize,        // Number of erasure-coded chunks
    reference: Option<Hash>,   // Optional: commit hash this blob supports
    max_fee: u128,             // Maximum blob fee willing to pay
}
```

### 5.3 Blob Fee Market

Blob fees are separate from execution gas to prevent DA costs from affecting regular transactions:

| Parameter | Value | Rationale |
|-----------|-------|-----------|
| `BASE_BLOB_FEE` | 100 | Minimum fee per blob submission |
| `FEE_PER_CHUNK` | 10 | Scales with data size |
| `MAX_BLOBS_PER_BLOCK` | 8 | Prevents block bloat |
| `BLOB_FEE_ADJUSTMENT` | 12.5% | EIP-1559 style adjustment per block |
| `TARGET_BLOBS_PER_BLOCK` | 4 | Target utilization for fee stability |

Total blob fee = `BASE_BLOB_FEE + chunk_count × FEE_PER_CHUNK × multiplier`

Where `multiplier` adjusts based on recent blob utilization (exponential moving average).

### 5.4 Lifecycle

1. Sender submits `BlobTransaction` → enters mempool (sorted by fee)
2. Block producer includes up to `MAX_BLOBS_PER_BLOCK` blob txs
3. On execution: creates `DasCommitment`, charges fee, emits `BlobSubmitted` event
4. DAS sampling begins automatically
5. Once `Confirmed`: blob data can be referenced by disputes, inference verifications
6. Blob data pruned after `BLOB_RETENTION_EPOCHS` (configurable, default 100,800 = ~14 days)

### 5.5 Validation Rules

A blob transaction is invalid if:
- `blob_size` is 0 or exceeds 16 MiB
- `chunk_count` doesn't match expected `ceil(blob_size / chunk_size) × 2`
- `max_fee < BASE_BLOB_FEE + chunk_count × FEE_PER_CHUNK × current_multiplier`
- Sender balance insufficient for fee
- Duplicate `blob_id` already committed
- Block already contains `MAX_BLOBS_PER_BLOCK` blobs

## 6. Provider Responsibilities

Providers MUST:
1. Store all erasure-coded chunks for committed blobs
2. Respond to DAS sample requests within `RESPONSE_WINDOW`
3. Serve chunk data to any requesting validator over P2P
4. Maintain chunks until `BLOB_RETENTION_EPOCHS` after confirmation

Providers SHOULD:
- Pre-distribute chunks to multiple peers for redundancy
- Prioritize DAS responses over regular P2P traffic

## 7. Validator Responsibilities

Validators MUST:
- Monitor new `DasCommitment` events
- Schedule and execute sampling rounds
- Report timeout/invalid responses on-chain
- Participate in negative quorum voting before penalization

The `DasValidator` component (NODE-028) automates this:
- `MAX_CONCURRENT_VALIDATIONS`: 32 simultaneous blob validations
- `SAMPLE_RETRY_LIMIT`: 3 retries per sample request
- `SAMPLING_MARGIN`: Start sampling 2 epochs before deadline

## 8. Integration Points

### 8.1 Dispute System

When a dispute is opened, the challenger references a `blob_id`. The dispute system verifies:
- Blob status is `Confirmed` (data is available)
- Challenger can reconstruct data from available chunks
- Bisection game operates over data referenced by confirmed blobs

### 8.2 Inference Commits

Inference commit transactions can include a `blob_reference` field linking to a blob transaction. This creates an on-chain proof that the inference data was available at commit time.

### 8.3 PDP Integration

For long-term storage (beyond `BLOB_RETENTION_EPOCHS`), blob data can be migrated to PDP proof sets on Ethereum L1, providing persistent storage guarantees.

## 9. Security Analysis

### 9.1 Adaptive Adversary

An adversary who sees sample indices before responding could selectively withhold. Mitigation: indices derived from future-epoch drand randomness (unpredictable at commitment time).

### 9.2 Network-Level Attacks

Eclipse attacks could prevent validators from receiving sample responses. Mitigation: negative quorum requirement — multiple independent validators must confirm unavailability.

### 9.3 Long-Range Data Attacks

After pruning window, data is no longer available on the DA layer. Mitigation: critical data migrated to PDP/L1 for permanent storage; dispute windows close before pruning begins.

## 10. Constants Summary

| Constant | Value | Module |
|----------|-------|--------|
| `TOTAL_CHUNKS` | 128 | `chain/das` |
| `ORIGINAL_CHUNKS` | 64 | `chain/das` |
| `SAMPLES_PER_ROUND` | 16 | `chain/das` |
| `REQUIRED_ROUNDS` | 3 | `chain/das` |
| `RESPONSE_WINDOW` | 5 epochs | `chain/das` |
| `DAS_PENALTY` | 500 | `chain/das` |
| `MAX_CONCURRENT_VALIDATIONS` | 32 | `node/das_validator` |
| `SAMPLE_RETRY_LIMIT` | 3 | `node/das_validator` |
| `SAMPLING_MARGIN` | 2 epochs | `node/das_validator` |
| `NEGATIVE_QUORUM` | 3 | `node/das_validator` |
| `BASE_BLOB_FEE` | 100 | `chain/blob_tx` |
| `FEE_PER_CHUNK` | 10 | `chain/blob_tx` |
| `MAX_BLOBS_PER_BLOCK` | 8 | `chain/blob_tx` |
| `TARGET_BLOBS_PER_BLOCK` | 4 | `chain/blob_tx` |
| `BLOB_RETENTION_EPOCHS` | 100,800 | `chain/blob_tx` |
