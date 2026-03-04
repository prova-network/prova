# SPEC-014: Checkpoint Anchoring Specification

## Status
Draft — v1.0 (2026-03-04)

## Abstract
Defines the protocol for anchoring Prova L2 state checkpoints to Filecoin L1, enabling cross-chain trust minimization, light client verification, and finality inheritance.

## 1. Overview

Prova produces checkpoints every `CHECKPOINT_INTERVAL` (120) epochs (~1 hour at 30s blocks). Each checkpoint commits the Prova state root, block hash, and validator set hash to Filecoin L1 via a smart contract call, creating an immutable trust anchor.

```
Prova L2                          Filecoin L1
─────────                         ───────────
epoch 120 ──► Checkpoint #0 ──►  AnchorContract.anchor(seq, stateRoot, ...)
epoch 240 ──► Checkpoint #1 ──►  AnchorContract.anchor(seq, stateRoot, ...)
  ...                               ...
```

## 2. Checkpoint Format

| Field              | Type      | Description                              |
|--------------------|-----------|------------------------------------------|
| sequence           | uint64    | Monotonically increasing checkpoint ID   |
| epoch_start        | uint64    | First Prova epoch covered (inclusive)     |
| epoch_end          | uint64    | Last Prova epoch covered (inclusive)      |
| state_root         | bytes32   | Prova state trie root at epoch_end       |
| block_hash         | bytes32   | Block hash at epoch_end                  |
| validator_set_hash | bytes32   | SHA-256 of active validator set          |
| signatures         | map       | validator_address → signature bytes      |
| signed_stake       | uint128   | Total stake weight of signers            |
| total_stake        | uint128   | Total active stake at epoch_end          |

### 2.1 Checkpoint Digest

The canonical digest that validators sign:

```
digest = SHA-256(
    sequence || epoch_start || epoch_end ||
    state_root || block_hash || validator_set_hash
)
```

All integers are little-endian encoded.

## 3. Quorum Rules

- **Threshold:** `signed_stake * 3 >= total_stake * 2` (≥ 2/3 weighted stake)
- **No duplicate votes:** Each validator may vote once per checkpoint
- **Zero-stake votes rejected:** Prevents sybil amplification
- **Single pending checkpoint:** A new checkpoint cannot begin until the current one is finalized or abandoned

## 4. Finalization Flow

```
1. Epoch reaches CHECKPOINT_INTERVAL boundary
2. Block proposer creates PendingCheckpoint with current state
3. Validators submit signed votes (digest + stake)
4. When quorum reached → PendingCheckpoint → Checkpoint (finalized)
5. Submitter encodes and sends L1 transaction
```

### 4.1 Vote Collection

Votes are collected via P2P gossip (see SPEC-007). Each vote message contains:
- Checkpoint sequence number
- Validator address
- Ed25519 signature over checkpoint digest
- Validator's current stake weight

### 4.2 Abandonment

If a pending checkpoint does not reach quorum within `CHECKPOINT_TIMEOUT` (60 epochs), it is abandoned and a new checkpoint attempt begins at the next interval boundary.

## 5. L1 Submission Protocol

### 5.1 Submitter Role

Any node may run the checkpoint submitter. In practice, a designated set of submitter nodes maintains hot wallets funded with FIL for gas.

### 5.2 Transaction Format

The L1 transaction calls `AnchorContract.anchorCheckpoint()`:

```
Calldata:
  selector:           0xACDC0001 (4 bytes)
  sequence:           uint64 LE (8 bytes)
  epoch_start:        uint64 LE (8 bytes)
  epoch_end:          uint64 LE (8 bytes)
  state_root:         bytes32 (32 bytes)
  block_hash:         bytes32 (32 bytes)
  validator_set_hash: bytes32 (32 bytes)
  signed_stake:       uint128 LE (16 bytes)
  total_stake:        uint128 LE (16 bytes)
  signature_count:    uint32 LE (4 bytes)
```

### 5.3 Gas Management

- **Estimation:** Base 500K gas + 1K per signature
- **Safety multiplier:** 1.2× (configurable via `gas_multiplier_bps`)
- **Max gas cap:** Transactions exceeding `max_gas` are rejected locally
- **Gas price:** Fetched from L1 RPC, with floor price to prevent stuck transactions

### 5.4 Retry Policy

| Condition         | Action                                      |
|-------------------|---------------------------------------------|
| RPC error         | Retry with exponential backoff              |
| Nonce conflict    | Re-fetch nonce, retry                       |
| Revert            | Retry up to `max_retries` (default: 5)      |
| Gas exceeded      | Mark as failed, alert operator              |
| Max retries       | Mark as failed, require manual intervention |

Backoff: `delay = base_backoff_ticks * 2^attempt`

### 5.5 Nonce Management

The submitter maintains a local nonce counter, incremented on each accepted submission. On nonce conflicts (from concurrent submitters), it re-fetches the L1 nonce and retries.

## 6. L1 Anchor Contract

### 6.1 State

```solidity
mapping(uint64 => AnchoredCheckpoint) public checkpoints;
uint64 public latestSequence;

struct AnchoredCheckpoint {
    bytes32 stateRoot;
    bytes32 blockHash;
    bytes32 validatorSetHash;
    uint64 epochStart;
    uint64 epochEnd;
    uint128 signedStake;
    uint128 totalStake;
    uint256 anchoredAt;  // L1 block number
}
```

### 6.2 Verification

The contract verifies:
1. `sequence == latestSequence + 1` (sequential ordering)
2. `signedStake * 3 >= totalStake * 2` (quorum met)
3. No duplicate anchoring (sequence not already stored)

Signature verification is deferred to a future upgrade (initially trusted submitter set).

## 7. Light Client Verification

A Prova light client can verify any state claim by:

1. Fetching the anchored checkpoint from L1 (trusted by Filecoin consensus)
2. Verifying the state root matches
3. Verifying Merkle proofs against the state root for specific account/storage queries

```
Client                    L1 Contract              Prova Node
──────                    ───────────              ──────────
  │── getCheckpoint(seq) ──►│                          │
  │◄── (stateRoot, ...) ───│                          │
  │                         │                          │
  │── getStateProof(key) ──────────────────────────►  │
  │◄── (value, merkleProof) ───────────────────────── │
  │                                                    │
  │ verify(stateRoot, key, value, proof) ✓             │
```

## 8. Security Considerations

### 8.1 Validator Collusion
A 2/3+ coalition could anchor false state roots. Mitigation: fraud proofs allow honest validators to challenge within a dispute window.

### 8.2 Submitter Censorship
If all submitters refuse to anchor, checkpoints stall. Mitigation: any funded address can submit; permissionless fallback.

### 8.3 L1 Reorgs
If Filecoin reorgs past an anchor transaction, the checkpoint must be resubmitted. The contract's sequential ordering prevents stale anchors.

### 8.4 Eclipse Attacks
An attacker isolating a light client from L1 could serve stale checkpoints. Mitigation: clients should verify L1 finality depth before trusting anchors.

## 9. Metrics

| Metric                          | Description                              |
|---------------------------------|------------------------------------------|
| `submitter_pending_count`       | Checkpoints awaiting submission          |
| `submitter_confirmed_count`     | Successfully anchored checkpoints        |
| `submitter_failed_count`        | Failed submissions                       |
| `submitter_total_gas`           | Cumulative gas spent on anchoring        |
| `submitter_nonce`               | Current L1 nonce                         |
| `checkpoint_finalize_latency`   | Time from epoch boundary to finalization |
| `anchor_latency`                | Time from finalization to L1 confirmation|

## 10. Configuration

```toml
[checkpoint.submitter]
enabled = true
max_retries = 5
base_backoff_ticks = 2
gas_multiplier_bps = 12000
max_gas = 10_000_000
l1_rpc = "https://api.node.glif.io/rpc/v1"
submitter_key = "/path/to/keystore/submitter.key"
```

## References

- SPEC-007: Network Protocol (vote gossip)
- `chain/src/checkpoint.rs`: Checkpoint manager implementation
- `chain/src/bridge.rs`: Cross-chain bridge message format
- `node/src/submitter.rs`: Checkpoint submitter implementation
