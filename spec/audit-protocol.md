# Audit Protocol — Random Sampling & Slashing

**Status:** Draft v0.1
**Author:** Capri (for Prova project)
**Date:** 2026-03-04

## 1. Overview

The audit protocol ensures inference integrity through **random sampling** of provider commits. Rather than requiring challengers to re-execute every inference, the protocol probabilistically selects commits for verification, achieving high detection probability with low overhead.

Slashing enforces economic consequences: dishonest providers lose stake, honest challengers earn rewards.

## 2. Design Goals

1. **Probabilistic soundness**: A provider cheating on fraction `f` of inferences is detected with probability `1 - (1-f)^k` after `k` audits
2. **Low overhead**: Auditors verify O(1) inferences per epoch, not all
3. **Incentive compatible**: Expected profit from cheating < expected slashing loss
4. **Permissionless**: Any staked participant can serve as auditor
5. **Composable**: Works alongside PDP storage proofs (orthogonal concerns)

## 3. Definitions

```
type AuditId       = u64
type AuditSeed     = [u8; 32]     // VRF output used for selection
type SlashFraction = f64           // 0.0 .. 1.0

struct AuditConfig {
    sample_rate: f64,              // Target fraction of commits audited per epoch (e.g., 0.05)
    min_stake_auditor: TokenAmount,// Minimum stake to serve as auditor
    challenge_bond: TokenAmount,   // Bond locked when filing a challenge
    slash_provider: SlashFraction, // Fraction of provider stake slashed on loss (e.g., 0.20)
    slash_challenger: SlashFraction,// Fraction of challenger bond slashed on false accusation (e.g., 1.0)
    reward_fraction: f64,          // Fraction of slashed amount paid to challenger (e.g., 0.50)
    cooldown_epochs: u64,          // Epochs a slashed provider is suspended (e.g., 2880 ≈ 24h)
    audit_window: u64,             // Epochs an auditor has to submit proof after selection (e.g., 120)
}
```

## 4. Audit Selection

### 4.1 Epoch-Based Random Sampling

Each epoch, the chain derives an `AuditSeed` from the previous epoch's randomness beacon (drand):

```
AuditSeed_e = SHA-256("prova-audit" || epoch_e || drand_round_e-1)
```

### 4.2 Commit Selection

For each finalized-but-unaudited commit `c` in the eligible window:

```
selection_hash = SHA-256(AuditSeed_e || c.id || auditor_address)
selected = (selection_hash[0..8] as u64) < (u64::MAX * sample_rate)
```

This gives each (commit, auditor) pair an independent `sample_rate` probability of selection per epoch. Multiple auditors may be selected for the same commit — first valid challenge wins the reward.

### 4.3 Eligible Commits

A commit is eligible for audit if:
- Status is `Finalized` (survived initial challenge window)
- Age < `max_audit_age` epochs (e.g., 20,160 ≈ 7 days)
- Not already under active audit/dispute

## 5. Audit Flow

```
┌─────────┐    select     ┌──────────┐   re-execute   ┌──────────┐
│  Chain   │──────────────▶│ Auditor  │───────────────▶│  Result  │
│  Seed    │               │ (staked) │                │  Match?  │
└─────────┘               └──────────┘                └────┬─────┘
                                                           │
                                              ┌────────────┴────────────┐
                                              │                         │
                                        Match: NOP               Mismatch: Challenge
                                        (no action)              (file dispute)
```

### Step 1 — Selection
Auditor checks `selection_hash` for each eligible commit. If selected, proceeds to verification.

### Step 2 — Re-execution
Auditor re-runs inference on the same model, input, and architecture group. Computes activation Merkle root.

### Step 3 — Comparison
If `auditor_root == commit.activation_root`: commit is honest. No action needed.

If roots differ: auditor files a challenge, posting `challenge_bond` and their own activation root.

### Step 4 — Bisection Dispute
Standard QBP bisection game begins (see `qbp-protocol.md`). The dispute narrows to a single layer, resolved by on-chain re-execution or referee judgment.

### Step 5 — Resolution
- **Provider loses**: `slash_provider` fraction of stake slashed. `reward_fraction` of slashed amount sent to auditor. Provider enters cooldown.
- **Auditor loses**: `challenge_bond` fully slashed (sent to provider as compensation). Auditor's reputation score decremented.

## 6. Slashing Schedule

| Offense | Slash % | Cooldown | Notes |
|---------|---------|----------|-------|
| Lost QBP dispute (inference) | 20% of stake | 2,880 epochs (24h) | Per-commit, not per-layer |
| Missed PDP proof (storage) | 5% of stake | 1,440 epochs (12h) | Aligned with standard PDP penalties |
| False challenge (auditor) | 100% of bond | None | Bond forfeited to provider |
| Repeated offense (3+ in 30d) | 50% of stake | 8,640 epochs (72h) | Escalating penalty |
| Consensus fault (equivocation) | 100% of stake | Permanent ban | Reserved for future consensus |

### 6.1 Escalation

Slashing escalates for repeat offenders within a rolling 30-day window:

```
effective_slash = base_slash * (1 + 0.5 * prior_offenses_30d)
```

Capped at 100% of deposited stake. After 3 offenses in 30 days, provider must re-stake to continue.

## 7. Economic Analysis

### 7.1 Detection Probability

With sample rate `r = 0.05` and `a = 10` active auditors, a commit has per-epoch audit probability:

```
p_audit = 1 - (1 - r)^a = 1 - 0.95^10 ≈ 0.40
```

Over a 7-day eligibility window (20,160 epochs), probability of escaping all audits:

```
p_escape = (1 - p_audit)^20160 ≈ 0  (effectively zero)
```

Even with `r = 0.01` and `a = 3`: `p_audit ≈ 0.03`, `p_escape over 7d ≈ 0`.

### 7.2 Incentive Compatibility

For cheating to be profitable, a provider needs:

```
profit_from_cheating > p_detect * slash_amount
```

With 20% slash on a 10,000 PROVA stake and near-certain detection:
- `slash_amount = 2,000 PROVA` per offense
- Any cheating profit < 2,000 PROVA makes dishonesty irrational

### 7.3 Auditor Economics

Auditors earn `reward_fraction * slash_amount` when they catch cheaters:
- At 50% reward: 1,000 PROVA per successful catch
- Cost: re-execution compute (GPU time for one inference)
- Expected ROI depends on cheating rate — honest network = low auditor income but also low cost (few selections to verify)

## 8. Auditor Selection & Anti-Gaming

### 8.1 VRF-Based Selection (Future)

Current design uses hash-based selection. Future upgrade path:
- Auditors submit VRF proofs of selection to prevent grinding
- `VRF_prove(auditor_sk, AuditSeed || commit_id)` → `(proof, output)`
- Selection based on VRF output, verifiable on-chain

### 8.2 Anti-Collusion

- Auditor cannot know which commits they'll be selected for until `AuditSeed` is revealed (drand dependency)
- Provider cannot know which auditor will check them
- Collusion requires corrupting drand beacon (economically infeasible)

## 9. Integration Points

### 9.1 With QBP (Inference)
- Audit triggers standard QBP bisection on mismatch
- Same `InferenceCommit` structure, same dispute resolution
- Audit just provides the sampling/triggering layer on top

### 9.2 With PDP (Storage)
- PDP proofs are orthogonal: they verify data availability, not compute correctness
- A provider can pass PDP (data is stored) but fail audit (inference was wrong)
- Both proof systems share the stake ledger — slashing is cumulative

### 9.3 With Payments
- Audit results gate payment finality: providers cannot withdraw earnings for commits under active audit
- Payment channels include an `audit_holdback` period aligned with `max_audit_age`

## 10. Parameters (Recommended Defaults)

| Parameter | Value | Rationale |
|-----------|-------|-----------|
| `sample_rate` | 0.05 (5%) | Balances overhead vs detection speed |
| `min_stake_auditor` | 1,000 PROVA | Prevents sybil auditor spam |
| `challenge_bond` | 500 PROVA | High enough to deter frivolous challenges |
| `slash_provider` | 0.20 (20%) | Painful but not catastrophic for single offense |
| `slash_challenger` | 1.00 (100%) | Full bond loss deters false accusations |
| `reward_fraction` | 0.50 (50%) | Incentivizes auditing without over-rewarding |
| `cooldown_epochs` | 2,880 (24h) | Prevents immediate re-offense |
| `audit_window` | 120 (~1h) | Enough time to re-execute inference |
| `max_audit_age` | 20,160 (7d) | Limits storage of audit-eligible commits |

## 11. Open Questions

1. **Auditor rotation**: Should auditors be rotated or can the same auditor repeatedly audit the same provider?
2. **Partial re-execution**: Can auditors verify a random subset of layers instead of full re-execution?
3. **Audit-as-a-service**: Should the protocol support delegated auditing (auditor pools)?
4. **Cross-architecture audits**: How to handle audits when auditor has different GPU architecture than provider?

---

*Spec complete. Implementation: `chain/src/audit.rs`*
