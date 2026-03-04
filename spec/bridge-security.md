# SPEC-015: Bridge Security Specification

## Status
Draft — v1.0 (2026-03-04)

## Abstract
Defines the security model, threat landscape, and mitigation strategies for the Prova ↔ Filecoin cross-chain bridge. Covers relay integrity, proof verification, censorship resistance, liveness guarantees, and economic security bounds.

## 1. Security Model

### 1.1 Trust Assumptions

| Component | Trust Assumption |
|-----------|-----------------|
| Filecoin L1 | Honest majority (>50% power). Finality after 900 epochs (~7.5 hours). |
| Prova L2 validators | Honest supermajority (≥2/3 weighted stake) for checkpoint signing. |
| Bridge relayers | Untrusted. Any party can relay; correctness enforced by proofs. |
| Merkle proofs | Cryptographic (SHA-256). Soundness: computationally infeasible to forge. |
| Nonce ordering | Per-sender monotonic. Replay impossible without nonce reuse. |

### 1.2 Security Properties

1. **Integrity**: A message accepted on the destination chain was genuinely created on the source chain.
2. **Uniqueness**: Each message is processed exactly once (replay protection via nonces).
3. **Ordering**: Messages from the same sender are processed in nonce order.
4. **Liveness**: If the source chain is live and ≥1 honest relayer exists, messages are eventually delivered.
5. **Expiry safety**: Expired messages (`MAX_MESSAGE_AGE = 2880 epochs`) cannot be relayed.

## 2. Threat Analysis

### T1 — Forged State Proof

**Attack**: Relayer submits a fabricated Merkle proof for a non-existent message.

**Mitigation**: `StateProof::verify()` recomputes the Merkle root from the claimed leaf hash and sibling path. Acceptance requires the computed root to match the checkpoint's `outbox_root`, which is signed by ≥2/3 validator stake and anchored on L1. Forging requires either:
- Breaking SHA-256 preimage resistance, or
- Corrupting ≥2/3 of Prova validator stake

**Residual risk**: Low. Computational infeasibility of SHA-256 collision + economic cost of acquiring 2/3 stake.

### T2 — Checkpoint Forgery

**Attack**: Attacker creates a fake checkpoint with a malicious `outbox_root` and submits it to the L1 anchor contract.

**Mitigation**: The anchor contract verifies that `signed_stake * 3 >= total_stake * 2`. Validators sign the canonical digest (`SHA-256(sequence || epoch_start || epoch_end || state_root || block_hash || validator_set_hash)`). The validator set hash is itself anchored in the previous checkpoint, creating a hash chain.

**Additional defense**: The L1 contract maintains a `validator_set_hash` and rejects checkpoints signed by a stale validator set unless accompanied by a valid validator set transition proof.

**Residual risk**: Requires corrupting ≥2/3 stake. Cost: `total_stake * 2/3` (currently protocol-defined minimum).

### T3 — Replay Attack

**Attack**: Relayer re-submits a previously processed message.

**Mitigation**: `Inbox::deliver()` tracks `(source_chain, sender) → last_processed_nonce`. Messages with `nonce <= last_processed_nonce` are rejected with `BridgeError::AlreadyProcessed`. Nonces are monotonically increasing and never reset.

**Residual risk**: None, assuming inbox state is not corrupted (protected by L2 consensus).

### T4 — Nonce Gap / Ordering Attack

**Attack**: Relayer skips a nonce, delivering message N+1 before N, potentially causing state inconsistency.

**Mitigation**: `Inbox::deliver()` enforces strict sequential ordering: `msg.nonce == expected_nonce`. Out-of-order delivery returns `BridgeError::InvalidNonce`. The relayer must deliver messages in order.

**Residual risk**: Liveness degradation if message N is lost. Mitigated by message expiry (T5) and relayer redundancy.

### T5 — Stale Message Relay

**Attack**: Relayer submits an ancient message long after its context is relevant, causing unexpected state changes.

**Mitigation**: `BridgeMessage::is_expired()` rejects messages older than `MAX_MESSAGE_AGE` (2880 epochs ≈ 24 hours). The inbox checks expiry before processing.

**Edge case**: Clock skew between chains. Prova epoch timestamps are authoritative for Prova-originated messages; Filecoin tipset heights for L1-originated messages.

**Residual risk**: Messages near the expiry boundary may be accepted or rejected depending on relay timing. This is by design — the 24h window provides ample relay time.

### T6 — Relayer Censorship

**Attack**: All relayers collude to censor specific messages (e.g., slashing results, governance actions).

**Mitigation**:
1. **Permissionless relaying**: Anyone can run a relayer. No registration or stake required.
2. **Economic incentive**: Relayers earn fees (proportional to gas cost + tip). Censored messages create arbitrage opportunities for competing relayers.
3. **Self-relay**: The message sender can always relay their own message.
4. **Validator-integrated relay**: Prova validators run relayer modules by default (`submitter.rs`), providing baseline liveness.

**Residual risk**: If all validators and all independent relayers censor, messages are delayed until an honest relayer appears or messages expire. This requires global coordination unlikely in a permissionless system.

### T7 — Bridge Drain via Token Transfer

**Attack**: Attacker exploits the bridge to mint or transfer tokens exceeding the locked collateral.

**Mitigation**:
1. **Lock-and-mint model**: Tokens transferred Filecoin→Prova are locked in the L1 bridge contract. Prova mints representative tokens 1:1. Prova→Filecoin burns representative tokens and unlocks L1 collateral.
2. **Conservation invariant**: `locked_on_L1 >= total_minted_on_L2` enforced by the L1 contract.
3. **Per-epoch rate limit**: The bridge contract enforces a maximum transfer volume per epoch (`MAX_TRANSFER_PER_EPOCH`), limiting damage from a compromised L2.
4. **Withdrawal delay**: Large withdrawals (>1% of bridge TVL) trigger a `WITHDRAWAL_DELAY` (48 epochs ≈ 24 minutes) during which validators can halt the bridge if fraud is detected.

**Residual risk**: If ≥2/3 validators are compromised, they can sign fraudulent checkpoints authorizing excess withdrawals. The rate limit and withdrawal delay provide a time window for detection and emergency response.

### T8 — L1 Reorg Invalidating Checkpoint

**Attack**: A Filecoin chain reorganization reverts an anchored checkpoint, causing the bridge to reference a non-canonical state.

**Mitigation**:
1. **Confirmation depth**: The L1 watcher (`watcher.rs`) waits for `CONFIRMATION_DEPTH` (30 Filecoin epochs ≈ 15 minutes) before treating an anchor as confirmed.
2. **Finality inheritance**: Full finality requires 900 Filecoin epochs (~7.5 hours). Bridge messages referencing unfinalized checkpoints carry explicit "soft-finality" status.
3. **Reorg detection**: The watcher monitors for L1 reorgs and emits `ReorgDetected` events, triggering automatic re-verification of affected checkpoints.

**Residual risk**: Deep reorgs (>30 epochs) could invalidate confirmed checkpoints. Probability is negligible under Filecoin's Expected Consensus with F3 fast finality (FIP-0086).

### T9 — Validator Set Transition Attack

**Attack**: During a validator set rotation, an attacker uses the old validator set to sign a malicious checkpoint before the new set is recognized.

**Mitigation**:
1. **Overlap period**: Validator set transitions span `TRANSITION_WINDOW` (60 epochs). During this window, checkpoints must be signed by ≥2/3 of BOTH old and new validator sets.
2. **Hash chain**: Each checkpoint includes `validator_set_hash`, and the L1 contract tracks the current expected hash. Transitions must be accompanied by a `ValidatorSetUpdate` message proving the new set.
3. **Stake continuity**: Validators cannot instantly unstake. The unbonding period (`UNBONDING_EPOCHS = 5760`, ~48 hours) ensures outgoing validators remain slashable during the transition window.

**Residual risk**: Low. Requires corrupting 2/3 of both old AND new validator sets simultaneously.

### T10 — Payload Confusion / Type Mismatch

**Attack**: A `RawData` payload is crafted to be misinterpreted as a `TokenTransfer` by the destination chain's handler.

**Mitigation**: Each `MessagePayload` variant is tagged with a unique discriminant byte (0-5) in the hash computation. Handlers dispatch on the enum variant, not on raw bytes. The type system enforces correct deserialization.

**Residual risk**: None in Rust (type-safe deserialization). External consumers (Solidity contracts on L1) must also enforce discriminant checking — this is specified in the L1 contract interface.

## 3. Economic Security Bounds

### 3.1 Cost of Attack

The minimum cost to forge a bridge message equals the cost of controlling ≥2/3 of Prova validator stake:

```
attack_cost = total_stake × 2/3
```

For bridge security to be meaningful:
```
attack_cost >> max_bridge_tvl_at_risk
```

Where `max_bridge_tvl_at_risk` = `MAX_TRANSFER_PER_EPOCH × WITHDRAWAL_DELAY` (the maximum extractable value before emergency halt).

### 3.2 Recommended Parameters

| Parameter | Value | Rationale |
|-----------|-------|-----------|
| `MAX_MESSAGE_AGE` | 2880 epochs (24h) | Ample relay time, limits stale attack surface |
| `CONFIRMATION_DEPTH` | 30 Filecoin epochs (15min) | Exceeds typical Filecoin reorg depth |
| `MAX_TRANSFER_PER_EPOCH` | 1% of bridge TVL | Limits single-epoch drain |
| `WITHDRAWAL_DELAY` | 48 epochs (24min) | Detection window for large withdrawals |
| `TRANSITION_WINDOW` | 60 epochs (30min) | Dual-signing overlap for validator rotation |
| `UNBONDING_EPOCHS` | 5760 epochs (48h) | Ensures slashability during transition |

### 3.3 Emergency Halt

A governance-triggered emergency halt freezes all bridge operations:
- Activated by ≥2/3 validator vote OR a governance proposal
- Freezes: all `Outbox::queue()`, `Inbox::deliver()`, and L1 contract withdrawals
- Resume requires a new governance vote with ≥3/4 supermajority

## 4. Audit Checklist

Bridge implementations MUST satisfy all items before mainnet deployment:

- [ ] `StateProof::verify()` rejects all non-leaf Merkle paths
- [ ] `Inbox::deliver()` rejects expired, replayed, and out-of-order messages
- [ ] L1 anchor contract verifies ≥2/3 stake quorum on every checkpoint
- [ ] L1 contract enforces validator set hash chain continuity
- [ ] Token conservation invariant holds under all code paths
- [ ] Rate limiter and withdrawal delay are not bypassable
- [ ] Emergency halt correctly freezes all bridge state transitions
- [ ] Validator set transition requires dual-set signing
- [ ] No integer overflow in stake arithmetic (u128 sufficient for 10^18 attoFIL units)
- [ ] Fuzzing: 10,000+ random message sequences with invalid proofs all rejected

## 5. Open Questions

1. **Cross-chain MEV**: Should the bridge enforce ordering fairness across relayers? Current design is first-valid-relay-wins.
2. **Multi-hop bridges**: If Prova bridges to chains beyond Filecoin, should the security model compose transitively or require direct anchoring per chain?
3. **Proof aggregation**: Batching multiple message proofs into a single verification could reduce L1 gas. Trade-off: latency vs. cost.

## References

- SPEC-014: Checkpoint Anchoring Specification
- CHAIN-017: `chain/src/bridge.rs` — Bridge message format and relay logic
- CHAIN-016: `chain/src/checkpoint.rs` — Checkpoint anchoring implementation
- NODE-016: `node/src/submitter.rs` — Checkpoint submission logic
- NODE-017: `node/src/watcher.rs` — L1 event watcher
