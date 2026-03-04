# SPEC-021: Validator Set Specification

**Status:** Draft  
**Authors:** Capri (autonomous)  
**Created:** 2026-03-04  
**Implements:** SPEC-021  
**Dependencies:** CHAIN-034 (Validator Set Manager), SPEC-020 (Delegation & Staking), SPEC-014 (Checkpoint Anchoring)

## 1. Overview

This specification formalizes Prova's dynamic validator set — the set of nodes authorized to produce blocks, vote on checkpoints, and enforce SLAs within a given epoch. Validators are selected from a candidate pool using a hybrid scoring function combining economic stake (70%) and behavioral reputation (30%), capped at 128 active members per epoch.

The validator set is **epoch-scoped**: membership only changes at epoch boundaries, providing intra-epoch stability for block production and checkpoint quorum calculations.

## 2. Parameters

| Parameter | Value | Rationale |
|-----------|-------|-----------|
| `MIN_VALIDATOR_STAKE` | 100,000 PROVA | Minimum skin-in-the-game for candidacy |
| `MAX_ACTIVE_VALIDATORS` | 128 | Balances decentralization vs. consensus latency |
| `UNBONDING_EPOCHS` | 14 | ~7 hours; aligns with checkpoint anchoring cadence |
| `DOWNTIME_THRESHOLD` | 3 consecutive epochs | Permissive enough for brief outages, strict enough to maintain liveness |
| `REPUTATION_WEIGHT` | 0.3 | Behavioral component of hybrid scoring |
| `STAKE_WEIGHT` | 0.7 | Economic component of hybrid scoring |
| `REJOIN_COOLDOWN` | 28 epochs | Ejected validators must wait before re-registering |

## 3. Validator Lifecycle

### 3.1 Registration

A node becomes a validator candidate by submitting a `RegisterValidator` transaction:

```
RegisterValidator {
    address:   AccountId,      // operator address (signs blocks)
    stake:     Amount,         // must be ≥ MIN_VALIDATOR_STAKE
    capacity:  u64,            // declared inference throughput (ops/s)
}
```

**Validation rules:**
- `stake ≥ MIN_VALIDATOR_STAKE`
- Address not currently registered in any non-`Exited` state
- Stake is locked immediately upon registration

On success the validator enters `Candidate` status and becomes eligible for the **next** epoch transition.

### 3.2 Active Set Selection (Epoch Transition)

At each epoch boundary the protocol computes the next active set:

1. **Collect candidates:** All validators in `Candidate` or `Active` status with `stake ≥ MIN_VALIDATOR_STAKE`.
2. **Score each candidate:**
   ```
   score(v) = STAKE_WEIGHT × (v.stake / max_stake) + REPUTATION_WEIGHT × v.reputation
   ```
   where `max_stake` is the highest stake among all candidates and `reputation ∈ [0.0, 1.0]`.
3. **Rank by score descending.** Ties broken by lower address (lexicographic determinism).
4. **Select top `min(candidates.len(), MAX_ACTIVE_VALIDATORS)`** → new active set.
5. **Demote** validators in previous active set but not in new set back to `Candidate`.
6. **Commit** new active set; validator set hash is included in the epoch's checkpoint.

### 3.3 Block Production Duty

Active validators take turns producing blocks in round-robin order sorted by address. A validator that fails to produce its assigned block increments `consecutive_misses`. Producing a block resets the counter to zero.

### 3.4 Voluntary Exit

A validator may submit `ExitValidator { address }`:
- Status transitions to `Unbonding { exit_epoch: current_epoch }`.
- Validator is removed from the active set at the **next** epoch transition.
- After `UNBONDING_EPOCHS` elapse, status moves to `Exited` and stake is unlocked.

During unbonding the validator is still subject to slashing for misbehavior committed while active.

### 3.5 Forced Ejection

The protocol forcibly ejects a validator under three conditions:

| Trigger | Condition | Ejection Reason |
|---------|-----------|-----------------|
| **Downtime** | `consecutive_misses ≥ DOWNTIME_THRESHOLD` | `Downtime` |
| **Slashing** | External slashing event (equivocation, invalid proof) | `Slashed` |
| **Underfunded** | `stake < MIN_VALIDATOR_STAKE` (e.g., after partial slash) | `InsufficientStake` |

Ejected validators:
- Immediately removed from the active set.
- Cannot re-register until `REJOIN_COOLDOWN` epochs have passed.
- For `Slashed` ejections, a configurable percentage of stake is burnt.

## 4. Hybrid Scoring Function

The scoring function deliberately weights stake heavily (70%) to maintain Sybil resistance while incorporating reputation (30%) to reward reliable operators. Reputation is an externally maintained EMA value (see CHAIN-015) that decays toward 0.5 during inactivity and adjusts based on:

- Block production rate
- Inference job completion (SLA compliance)
- Checkpoint vote participation
- Challenge participation (honest dispute resolution)

**Normalization:** Stake is normalized against the current maximum to prevent absolute stake dominance and keep scores in a consistent `[0.0, 1.0]` range.

## 5. Epoch Transition Protocol

```
EpochTransition(current_epoch):
  1. Increment epoch counter
  2. Process downtime: for each active validator with consecutive_misses ≥ DOWNTIME_THRESHOLD → eject
  3. Process unbonding: for each Unbonding validator where current_epoch ≥ exit_epoch + UNBONDING_EPOCHS → Exited, unlock stake
  4. Compute new active set (§3.2)
  5. Reset per-epoch counters (consecutive_misses stays until block produced)
  6. Emit ValidatorSetRotated event with new set hash
```

**Determinism:** The transition function is fully deterministic — all nodes processing the same state at the same epoch boundary produce an identical active set. This is critical for consensus: the validator set hash is committed to the checkpoint anchor (SPEC-014, §2).

## 6. Interaction with Delegation

Delegated stake counts toward a validator's total stake for scoring purposes (SPEC-020):

```
effective_stake(v) = v.self_stake + sum(delegations_to_v)
```

When a validator is slashed, delegators bear proportional losses (SPEC-020, §5). When a validator is ejected for downtime, delegators may redelegate without cooldown penalty (grace period of 1 epoch after ejection event).

## 7. Interaction with Checkpoints

Each checkpoint (SPEC-014) includes a `validator_set_hash` — the SHA-256 of the sorted active validator addresses and stakes. This enables:

- **Light client verification:** A light client can verify that a checkpoint was signed by a quorum of the validator set without knowing the full state.
- **Cross-epoch auditability:** Anyone can verify the active set at any historical checkpoint.
- **Quorum threshold:** Checkpoints require signatures from validators representing ≥ 67% of total active stake.

## 8. Security Considerations

### 8.1 Stake Grinding
An attacker could split stake across many identities to game the scoring function. Mitigation: `MIN_VALIDATOR_STAKE` makes Sybil attacks expensive, and the scoring function normalizes against max stake (splitting reduces each identity's normalized score).

### 8.2 Reputation Manipulation
Selectively completing easy jobs to inflate reputation. Mitigation: reputation is an EMA with slow growth, and job assignment is randomized (CHAIN-013). A validator cannot choose which jobs it receives.

### 8.3 Long-Range Attacks
An attacker accumulates historical validator keys from exited validators. Mitigation: Filecoin L1 checkpoint anchoring (SPEC-014) provides an external finality root. Clients that sync from a checkpoint less than `UNBONDING_EPOCHS` old are safe.

### 8.4 Validator Set Capture
A wealthy actor acquires enough stake to control >67% of the active set. Mitigation: `MAX_ACTIVE_VALIDATORS = 128` means the attacker must outbid 85+ honest validators. The reputation component (30%) further penalizes newcomers without operational history.

## 9. Wire Format

Validator set commitments use a canonical encoding for deterministic hashing:

```
ValidatorSetCommitment:
  epoch:       uint64 (big-endian)
  count:       uint32 (big-endian)
  entries[]:   sorted by address ascending
    address:   32 bytes (zero-padded)
    stake:     uint64 (big-endian)
```

`validator_set_hash = SHA-256(ValidatorSetCommitment)`

## 10. Future Work

- **Validator rotation smoothing:** Limit the number of validators that can enter/exit per epoch to reduce set churn.
- **Geographic diversity scoring:** Incentivize geographic distribution to improve censorship resistance.
- **Validator key rotation:** Allow operators to rotate signing keys without re-registering.
- **Dynamic `MAX_ACTIVE_VALIDATORS`:** Governance-adjustable cap based on network scale.
