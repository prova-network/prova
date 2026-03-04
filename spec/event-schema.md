# SPEC-016: Event Schema Specification

**Status:** Final  
**Author:** Capri  
**Created:** 2026-03-04

## 1. Overview

Prova emits structured, typed events from all state-changing operations. Events are the primary interface for off-chain indexing, wallets, explorers, and SDK consumers. This specification defines the canonical event signatures, encoding format, topic layout, and versioning rules.

## 2. Event Anatomy

Each event consists of:

| Field      | Size        | Description |
|------------|-------------|-------------|
| `topics`   | 1–4 × 32B  | Topic[0] = SHA-256 of event signature. Topics[1–3] = indexed fields (left-padded to 32 bytes). |
| `data`     | 0–8192B     | ABI-encoded non-indexed fields (big-endian, tightly packed per type). |
| `emitter`  | 32B         | Address of the module that emitted the event. |
| `epoch`    | 8B          | Block epoch in which the event was emitted. |

### 2.1 Topic Hashing

Topic[0] is always `SHA-256(signature_string)` where `signature_string` follows Solidity-style canonical form:

```
EventName(type1,type2,...typeN)
```

Only the _indexed_ parameters appear as separate topics. Non-indexed parameters are ABI-encoded into `data`.

## 3. Canonical Type System

Prova uses a minimal type set for event encoding:

| Type       | Size  | Encoding |
|------------|-------|----------|
| `address`  | 32B   | Left-padded Ed25519 public key |
| `uint64`   | 8B    | Big-endian, left-padded to 32B in topics |
| `uint128`  | 16B   | Big-endian, left-padded to 32B in topics |
| `bytes32`  | 32B   | Raw 32 bytes |
| `bool`     | 1B    | `0x00` = false, `0x01` = true, left-padded to 32B in topics |
| `string`   | var   | UTF-8, length-prefixed (4B big-endian length + bytes) in data only |
| `bytes`    | var   | Length-prefixed (4B big-endian length + bytes) in data only |

Variable-length types (`string`, `bytes`) cannot be indexed (cannot appear as topics).

### 3.1 Data Encoding

Non-indexed fields are packed sequentially in `data`:
- Fixed-size types: written at their native size (no padding in data).
- Variable-length types: 4-byte big-endian length prefix, then raw bytes.
- Field order matches declaration order in the signature.

## 4. Canonical Event Signatures

### 4.1 Token & Economics

| Event | Signature | Indexed | Data |
|-------|-----------|---------|------|
| Transfer | `Transfer(address,address,uint128)` | from, to | amount |
| BlockReward | `BlockReward(address,uint128)` | producer | amount |

### 4.2 Staking

| Event | Signature | Indexed | Data |
|-------|-----------|---------|------|
| StakeDeposited | `StakeDeposited(address,uint128)` | staker | amount |
| StakeWithdrawn | `StakeWithdrawn(address,uint128)` | staker | amount |
| Slash | `Slash(address,uint128,bytes32)` | offender | amount, reason_hash |

### 4.3 Inference & Verification

| Event | Signature | Indexed | Data |
|-------|-----------|---------|------|
| InferenceCommitted | `InferenceCommitted(uint64,address,bytes32)` | commit_id, provider | activation_root |
| ChallengeOpened | `ChallengeOpened(uint64,address,address)` | commit_id, challenger, provider | — |
| ChallengeResolved | `ChallengeResolved(uint64,bool)` | commit_id | honest_provider |

### 4.4 Job Scheduling

| Event | Signature | Indexed | Data |
|-------|-----------|---------|------|
| JobSubmitted | `JobSubmitted(uint64,address,bytes32)` | job_id, requester | model_hash |
| JobCompleted | `JobCompleted(uint64,address)` | job_id, provider | — |
| JobCancelled | `JobCancelled(uint64,address)` | job_id, requester | — |
| JobTimedOut | `JobTimedOut(uint64,address)` | job_id, provider | — |

### 4.5 Model Registry

| Event | Signature | Indexed | Data |
|-------|-----------|---------|------|
| ModelRegistered | `ModelRegistered(bytes32,address)` | model_hash, registrant | — |
| ModelDeprecated | `ModelDeprecated(bytes32,address)` | model_hash, actor | — |

### 4.6 Payment Channels

| Event | Signature | Indexed | Data |
|-------|-----------|---------|------|
| PaymentOpened | `PaymentOpened(address,address,uint128)` | payer, payee | deposit |
| PaymentSettled | `PaymentSettled(address,address,uint128)` | payer, payee | amount |
| PaymentDisputed | `PaymentDisputed(address,address,uint64)` | payer, payee | channel_id |

### 4.7 Governance

| Event | Signature | Indexed | Data |
|-------|-----------|---------|------|
| GovernanceProposal | `GovernanceProposal(uint64,address)` | proposal_id, proposer | — |
| GovernanceVote | `GovernanceVote(uint64,address,bool)` | proposal_id, voter | support |
| GovernanceExecuted | `GovernanceExecuted(uint64,bool)` | proposal_id | passed |

### 4.8 Checkpoint & Bridge

| Event | Signature | Indexed | Data |
|-------|-----------|---------|------|
| CheckpointAnchored | `CheckpointAnchored(uint64,bytes32)` | epoch | l1_tx_hash |
| BridgeMessageSent | `BridgeMessageSent(uint64,bytes32,address)` | nonce, dest_hash | sender |
| BridgeMessageReceived | `BridgeMessageReceived(uint64,bytes32)` | nonce | source_hash |

### 4.9 Protocol Upgrades

| Event | Signature | Indexed | Data |
|-------|-----------|---------|------|
| UpgradeScheduled | `UpgradeScheduled(uint64,uint64)` | version, activation_epoch | — |
| UpgradeActivated | `UpgradeActivated(uint64)` | version | — |

## 5. Filtering Rules

Filters match events using:
- **Topic filter:** Match on any topic position (0–3). `None` = wildcard.
- **Address filter:** Match `emitter` field.
- **Epoch range:** Inclusive `[from_epoch, to_epoch]`.

A filter matches if ALL non-None fields match (conjunction). Multiple filters combine as disjunction (OR).

## 6. Receipt Inclusion

Events within a block are ordered by execution sequence. The block receipt contains:
- `events_root`: Merkle root (SHA-256 binary tree) of all event hashes in the block.
- Each event hash: `SHA-256(emitter ++ topic_count ++ topics ++ data_len ++ data)`.

This enables lightweight proof-of-inclusion for events without downloading full block data.

## 7. Versioning

Event signatures are immutable once deployed. To evolve an event:
1. Emit a new event with a `V2` suffix (e.g., `TransferV2(address,address,uint128,bytes32)`).
2. Old event continues to emit for backward compatibility during a transition period.
3. After a protocol upgrade, old event emission may cease.

The `UpgradeScheduled` event itself signals when schema changes activate.

## 8. SDK Encoding/Decoding

The SDK provides:
- `encode_event(signature, indexed_values, data_values) → Event` — canonical encoding.
- `decode_event(event, signature) → (Vec<Value>, Vec<Value>)` — decode indexed + data fields.
- `event_type_hash(signature) → Hash` — compute topic[0].

These are the ONLY sanctioned encoding paths. Implementations MUST NOT hand-encode events.

## 9. Security Considerations

- Event data is **untrusted input** for indexers — validate lengths and types before processing.
- Topic[0] collisions are computationally infeasible (SHA-256 preimage resistance).
- Variable-length data in `data` field MUST be bounds-checked against `MAX_DATA_SIZE` (8192 bytes).
- Indexers SHOULD verify `events_root` against the block receipt before trusting event content.
