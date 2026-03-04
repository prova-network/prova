# SPEC-010: Token Economics Specification

**Status:** Draft  
**Authors:** Capri (autonomous)  
**Created:** 2026-03-04

## 1. Overview

The Prova network uses a native token (**PROVA**) for staking, payment, and dispute resolution incentives. This spec defines issuance, distribution, fee structures, and economic security parameters.

## 2. Token Parameters

| Parameter | Value |
|-----------|-------|
| Symbol | PROVA |
| Decimals | 18 |
| Max supply | 1,000,000,000 (1B) |
| Genesis supply | 100,000,000 (100M) |
| Block reward (initial) | 10 PROVA/epoch |
| Halving interval | 2,102,400 epochs (~2 years at 30s blocks) |
| Minimum halvings | 7 (reward floor: 0.078125 PROVA/epoch) |

## 3. Issuance Schedule

Block rewards follow a halving curve:

```
reward(epoch) = 10 * (0.5 ^ floor(epoch / 2_102_400))
```

Total issuance from mining converges to ~880M PROVA (geometric series: 10 × 2,102,400 × 2 × (1 - 0.5^7) ≈ 41.75M per halving cycle × 2 initial).

**Supply breakdown:**
- 88% — Mining rewards (vested over ~14 years)
- 10% — Genesis allocation (foundation, ecosystem grants)
- 2% — Genesis allocation (initial liquidity / bootstrapping)

## 4. Staking Economics

### 4.1 Provider Stake

Inference providers must stake to participate:

| Parameter | Value |
|-----------|-------|
| Minimum stake | 10,000 PROVA |
| Slash fraction (invalid proof) | 50% of stake |
| Slash fraction (timeout) | 5% of stake |
| Cooldown period | 7,200 epochs (60 hours) |

### 4.2 Challenger Bonds

Challengers post a bond to initiate disputes:

| Parameter | Value |
|-----------|-------|
| Challenge bond | 1,000 PROVA |
| Successful challenge reward | bond + 50% of provider slash |
| Failed challenge penalty | bond forfeited (burnt) |

### 4.3 Staking Rewards

Block producers are selected proportional to stake. Validators earn:
- Block reward (per epoch schedule)
- 50% of transaction fees in the block
- 50% of dispute resolution fees

Remaining 50% of fees are burnt (deflationary pressure post-halving).

## 5. Fee Structure

### 5.1 Transaction Fees

Base fee model with congestion pricing:

```
fee = base_fee * gas_used
base_fee = max(MIN_FEE, target_adjustment(prev_block_utilization))
```

| Parameter | Value |
|-----------|-------|
| MIN_FEE | 0.001 PROVA |
| Target block utilization | 50% |
| Adjustment factor | ±12.5% per block |

### 5.2 Inference Commit Fee

Providers pay a small fee per inference commit to cover state storage:

```
commit_fee = BASE_COMMIT_FEE + (activation_tree_size * STORAGE_RATE)
```

| Parameter | Value |
|-----------|-------|
| BASE_COMMIT_FEE | 0.1 PROVA |
| STORAGE_RATE | 0.001 PROVA per KB |

### 5.3 Model Registration Fee

One-time fee to register a model in the on-chain registry:

| Parameter | Value |
|-----------|-------|
| Registration fee | 100 PROVA (burnt) |

## 6. Payment Channels

Streaming payments for inference services (see SPEC-006):

| Parameter | Value |
|-----------|-------|
| Minimum channel deposit | 1 PROVA |
| Settlement delay | 100 epochs |
| Network fee | 0.5% of settled amount (burnt) |

## 7. Economic Security Analysis

### 7.1 Attack Cost

For a provider to profit from cheating:
- Expected slash: `stake * 0.5 * detection_probability`
- Expected gain from cheating: `inference_fee * savings_ratio`
- Honest equilibrium requires: `slash > gain`, which holds when `stake > 2 * inference_fee * savings_ratio / detection_probability`

With minimum stake of 10,000 PROVA and typical inference fees of 0.1-1 PROVA, the economic security margin is 10,000-100,000×.

### 7.2 Sybil Resistance

Stake-weighted selection prevents sybil attacks. Creating N identities splits stake N ways, providing no advantage over a single identity.

### 7.3 Deflationary Mechanics

Post-halving, fee burns (50% of tx fees + 100% of registration fees + 0.5% payment channel fees + failed challenge bonds) create deflationary pressure, potentially making PROVA net-deflationary at sufficient network utilization.

## 8. Governance Parameters

All economic parameters are governable via on-chain proposals (future SPEC):

| Parameter | Governance delay |
|-----------|-----------------|
| Block reward | 30-day timelock |
| Slash fractions | 14-day timelock |
| Fee parameters | 7-day timelock |
| Minimum stake | 14-day timelock |

## 9. Implementation Notes

- All amounts stored as `u128` in smallest denomination (10^-18 PROVA)
- Halving computed lazily per-block (no state transition needed)
- Fee burns reduce `total_supply` tracked in state trie
- Staking integrates with `chain/src/stake.rs`
- Payment channels integrate with `chain/src/payment.rs`
- Account balances tracked in `chain/src/state.rs`
