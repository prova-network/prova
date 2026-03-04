# SPEC-012: Light Client Specification

## Summary

Light clients verify Prova chain state without downloading full blocks. They track block headers, verify Merkle proofs against state roots, and validate finality signatures — enabling mobile wallets, browser dApps, and resource-constrained verifiers.

## Design Goals

1. **Minimal bandwidth**: Only headers + targeted proofs, not full blocks
2. **Cryptographic verification**: Every claim verified against header state roots
3. **Finality awareness**: Only trust finalized state (2/3 stake-weighted signatures)
4. **Sync efficiency**: Checkpoint-based fast sync for new clients

## Architecture

```
┌─────────────────────────────────────────┐
│              Light Client               │
│                                         │
│  ┌──────────┐  ┌──────────┐  ┌───────┐ │
│  │ Header   │  │ State    │  │Finality│ │
│  │ Chain    │  │ Verifier │  │Tracker │ │
│  └────┬─────┘  └────┬─────┘  └───┬───┘ │
│       │             │             │     │
│  ┌────┴─────────────┴─────────────┴───┐ │
│  │         Peer Network Layer         │ │
│  └────────────────────────────────────┘ │
└─────────────────────────────────────────┘
```

## Header Chain

Light clients maintain a chain of block headers (without transaction bodies):

```
Header {
    parent_hash:  Hash
    state_root:   Hash      // Merkle root of full state trie
    epoch:        u64
    producer:     Address
    tx_root:      Hash      // Not verified by light client
    tx_count:     u32
    timestamp:    u64
}
```

**Validation rules:**
- `hash(header) == expected` (SHA-256 of serialized fields)
- `header.parent_hash == prev_header.hash`
- `header.epoch == prev_header.epoch + 1`
- Producer is in the current validator set (verified via state proof)

## State Proofs

Light clients verify individual state claims via Merkle inclusion proofs against `state_root`:

### Account Balance Proof
```
Request:  GetStateProof(epoch, account_address)
Response: {
    value:  AccountState { balance, nonce, stake }
    proof:  Vec<(Hash, Direction)>  // Merkle path from leaf to root
    root:   Hash                    // Must match header.state_root
}
```

### Proof Verification
```
verify_proof(key, value, proof, expected_root) -> bool:
    leaf = hash(key || serialize(value))
    current = leaf
    for (sibling, direction) in proof:
        if direction == Left:
            current = hash(sibling || current)
        else:
            current = hash(current || sibling)
    return current == expected_root
```

### Supported Proof Types

| Proof Type | Key | Value |
|-----------|-----|-------|
| Account balance | `account/{address}` | `{balance, nonce, stake}` |
| Model registry | `model/{model_id}` | `{hash, name, layer_count, arch}` |
| Inference commit | `commit/{commit_id}` | `{provider, model, root, epoch}` |
| Dispute status | `dispute/{dispute_id}` | `{state, challenger, provider}` |
| Payment channel | `payment/{channel_id}` | `{payer, payee, balance, rate}` |

## Finality Tracking

A header is considered **finalized** when accompanied by aggregate signatures from validators holding ≥2/3 of total stake weight.

```
FinalityProof {
    epoch:       u64
    header_hash: Hash
    signatures:  Vec<(Address, Signature)>
    stake_sum:   u64       // Sum of signing validators' stake
    total_stake: u64       // Total network stake at that epoch
}
```

**Verification:**
1. `stake_sum * 3 > total_stake * 2` (strict 2/3 threshold)
2. Each signature is valid for `header_hash`
3. Each signer's stake is verified against the **previous finalized** state root

Light clients MUST NOT trust unfinalized state for value transfers.

## Sync Modes

### Full Header Sync
Download every header from genesis. Most secure but slow for long chains.

- **Bandwidth:** ~100 bytes/header × chain_length
- **Use case:** High-security applications, first sync of a persistent client

### Checkpoint Sync
Start from a trusted checkpoint (header + finality proof), then sync headers forward.

```
Checkpoint {
    header:         Header
    finality_proof: FinalityProof
    validator_set:  Vec<(Address, StakeAmount)>
}
```

**Trust model:** Client trusts the checkpoint provider OR verifies the finality proof against a known validator set.

- **Bandwidth:** ~1KB checkpoint + headers since checkpoint
- **Use case:** Mobile wallets, browser clients, rapid onboarding

### Skip Sync
For very long chains, verify every Nth header with finality proofs, skipping intermediate headers.

- Skip interval: configurable, default N=100
- Each skipped range must have a finality proof
- **Bandwidth:** ~(chain_length / N) × (header_size + finality_proof_size)
- **Use case:** Archival verification, cross-chain bridges

## Protocol Messages

### Light Client → Full Node

| Message | Description |
|---------|-------------|
| `GetHeaders(from, count)` | Request header range |
| `GetStateProof(epoch, key)` | Request Merkle proof for state key |
| `GetFinalityProof(epoch)` | Request finality proof for epoch |
| `GetCheckpoint(epoch?)` | Request latest or specific checkpoint |
| `GetValidatorSet(epoch)` | Request validator set at epoch |

### Full Node → Light Client

| Message | Description |
|---------|-------------|
| `Headers(Vec<Header>)` | Header batch |
| `StateProof(key, value, proof)` | Merkle inclusion proof |
| `FinalityProof(proof)` | Aggregate finality signatures |
| `Checkpoint(checkpoint)` | Sync checkpoint |
| `ValidatorSet(set)` | Validator addresses + stakes |
| `NewHead(header, finality?)` | Push notification of new finalized head |

## Security Considerations

1. **Eclipse attacks:** Light clients should connect to multiple peers (minimum 3) and cross-validate headers
2. **Long-range attacks:** Checkpoint sync trusts the checkpoint source; clients should verify checkpoints from multiple independent sources
3. **Data withholding:** A full node could serve valid headers but refuse state proofs; clients should have fallback peers
4. **Validator set changes:** When validators change between checkpoints, the client must verify the transition through intermediate finality proofs
5. **Proof freshness:** State proofs are only valid for the epoch they reference; clients must request proofs against recent finalized headers

## Resource Requirements

| Metric | Full Header Sync | Checkpoint Sync | Skip Sync (N=100) |
|--------|-----------------|-----------------|-------------------|
| Storage | ~100B × epochs | ~1KB + recent headers | ~1KB × (epochs/100) |
| Bandwidth/block | ~100B | ~100B | ~100B/100 avg |
| Verification/block | 1 hash | 1 hash | 1 hash + sig verify / 100 |
| Initial sync (10K blocks) | ~1MB | ~1KB | ~10KB |

## Implementation Notes

- State trie must support efficient Merkle proof generation (already in `chain/src/state.rs`)
- Finality proofs require aggregating validator signatures (future: BLS aggregation for compactness)
- Checkpoint distribution can use gossip network or dedicated HTTP endpoints
- Browser clients can use WebSocket transport to receive `NewHead` push notifications
