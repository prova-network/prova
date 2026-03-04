# SPEC-005: Audit Protocol Specification

**Status:** Draft  
**Author:** Capri  
**Created:** 2026-03-04

## 1. Overview

The audit protocol provides probabilistic verification of inference correctness without requiring every inference to be fully re-executed. A random subset of commits are selected for audit each epoch, creating an economic deterrent against cheating.

## 2. Audit Selection

### 2.1 Random Sampling

Each epoch, the chain selects commits for audit using drand randomness:

```
seed = SHA256(drand_beacon(epoch) || "prova-audit")
audit_set = select_weighted_random(open_commits, seed, rate=0.05)
```

**Audit rate:** 5% of all commits per epoch (configurable via governance).

### 2.2 Weighted Selection

Selection probability is weighted by:
- **Stake ratio:** Lower-staked providers are audited more frequently
- **History:** Providers with recent disputes are audited more
- **Value:** Higher-value inferences have higher audit probability

```
weight(provider) = base_rate 
    × (median_stake / provider_stake)     # stake inverse
    × (1 + recent_disputes × 0.2)         # dispute history
    × log(inference_value + 1)             # value scaling
```

## 3. Audit Execution

### 3.1 Designated Verifiers

Audits are performed by **designated verifiers** — nodes that:
1. Have the model weights (verified via PDP)
2. Run the same architecture group as the commit
3. Are staked as verifiers (separate from provider stake)

### 3.2 Audit Flow

```
1. Chain selects commit C for audit
2. Chain assigns verifier V from eligible pool
3. V downloads input from commit metadata
4. V re-executes full inference on the model
5. V computes own activation Merkle root
6. V submits AuditReport(commit_id, own_root, match: bool)
```

### 3.3 Audit Outcomes

| Outcome | Action |
|---------|--------|
| Roots match | Commit confirmed, verifier rewarded |
| Roots differ | Automatic dispute opened (bisection) |
| Verifier timeout | Verifier slashed, new verifier assigned |
| Multiple verifier disagreement | Escalated to full committee |

## 4. Verifier Economics

### 4.1 Rewards
- Audit confirmation reward: `0.1 × inference_fee`
- Successful dispute detection: `50% of slashed provider stake`

### 4.2 Costs
- Must maintain model weights (storage cost via PDP)
- Must run inference for each audit (compute cost)
- Stake requirement: `min_verifier_stake`

### 4.3 Verifier Selection
```
eligible = verifiers WHERE:
    stake >= min_verifier_stake
    AND has_model(commit.model_id)
    AND arch_group == commit.arch_group
    AND NOT in_cooldown
    AND address != commit.provider  # can't audit yourself

selected = weighted_random(eligible, seed, count=1)
```

## 5. Anti-Gaming

### 5.1 Lazy Verification Prevention
Verifiers must submit their own root — not just "match/no-match". The chain stores both roots. If a later dispute proves the verifier rubber-stamped, the verifier is slashed.

### 5.2 Collusion Resistance
- Verifier assignment is random (unpredictable before drand reveal)
- Provider doesn't know which commits will be audited
- Verifier doesn't know assignment until the epoch begins

### 5.3 Eclipse Resistance
If a provider controls >50% of eligible verifiers for their arch group, the audit rate automatically increases to compensate.

## 6. Audit Committee (Escalation)

When single-verifier audits produce conflicting results:

1. Committee of 3 verifiers is assembled (different operators)
2. Each re-executes independently
3. 2-of-3 majority determines outcome
4. Minority verifier is slashed (assumed faulty or malicious)

Committee audits are expensive and rare — expected <0.1% of all audits.

## 7. Parameters

| Parameter | Value | Governance |
|-----------|-------|------------|
| Base audit rate | 5% | Yes |
| Verifier reward | 10% of inference fee | Yes |
| Dispute detection reward | 50% of slash | Yes |
| Verifier response window | 120 epochs (~1 hour) | Yes |
| Committee size | 3 | Yes |
| Committee threshold | 2-of-3 | No |
| Max audits per verifier per epoch | 10 | Yes |

## 8. Security Analysis

**Expected cheating cost:**
- With 5% audit rate, a cheating provider is caught within ~20 inferences on average
- Expected loss per cheat attempt: `0.05 × slash_amount - (1 - 0.05) × inference_fee`
- For this to be profitable: `inference_fee > 0.053 × slash_amount`
- With 10% slash on 1M stake: profitable only if inference fee > 5,263 tokens — far above market rate

**Conclusion:** Cheating is economically irrational for any reasonable stake/fee ratio.

## 9. References

- [SPEC-001: QBP Protocol](./qbp-protocol.md)
- [SPEC-004: PDP Integration](./pdp-integration.md)
- Prova Whitepaper §5 (Verification Economics)
