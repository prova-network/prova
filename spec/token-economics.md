# SPEC-010: Token Economics Specification

**Status:** Draft v2
**Authors:** Nicklas Reiers, Capri
**Created:** 2026-03-04
**Updated:** 2026-03-24

## 1. Overview

The Prova network uses a native token (**PROVA**) for staking, payment, dispute resolution, and governance. This spec defines issuance, distribution, fee structures, and economic security parameters.

## 2. Token Parameters

| Parameter | Value |
|-----------|-------|
| Symbol | PROVA |
| Decimals | 18 |
| Total supply | 1,000,000,000 (1B) |
| Minting | Fixed at genesis (no inflation beyond block rewards) |

## 3. Allocation

| Category | % | Tokens | Vesting |
|---|---|---|---|
| Network mining | 45% | 450,000,000 | Emitted via block rewards (~14 years) |
| Public sale | 15% | 150,000,000 | 25% at TGE, 75% over 6 months |
| Team & founders | 15% | 150,000,000 | 12-month cliff, 36-month linear |
| Ecosystem & grants | 10% | 100,000,000 | 10% at TGE, quarterly over 48 months |
| Early backers (seed) | 7% | 70,000,000 | 6-month cliff, 18-month linear |
| Liquidity | 5% | 50,000,000 | 100% at TGE, LP locked 12 months |
| Reserve | 3% | 30,000,000 | 6-month cliff, 24-month multisig |

## 4. Block Reward Schedule

Block rewards follow a halving curve:

```
reward(epoch) = BASE_REWARD * (0.5 ^ floor(epoch / HALVING_INTERVAL))
```

| Parameter | Value |
|-----------|-------|
| BASE_REWARD | 10 PROVA/epoch |
| HALVING_INTERVAL | 2,102,400 epochs (~2 years at 30s blocks) |
| Minimum halvings | 7 (reward floor: 0.078125 PROVA/epoch) |

Total mining emission converges to ~450M PROVA over ~14 years.

## 5. Staking Economics

### 5.1 Provider Stake

| Parameter | Value |
|-----------|-------|
| Minimum stake | 10,000 PROVA |
| Slash fraction (invalid proof) | 50% of stake |
| Slash fraction (timeout) | 5% of stake |
| Cooldown period | 7,200 epochs (60 hours) |

### 5.2 Challenger Bonds

| Parameter | Value |
|-----------|-------|
| Challenge bond | 1,000 PROVA |
| Successful challenge reward | bond + 50% of provider slash |
| Failed challenge penalty | bond forfeited (burnt) |

### 5.3 Staking Rewards

Block producers selected proportional to stake. Validators earn:
- Block reward (per epoch schedule)
- 50% of transaction fees in the block
- 50% of dispute resolution fees

Remaining 50% of fees are burnt (deflationary pressure post-halving).

## 6. Fee Structure

### 6.1 Transaction Fees

EIP-1559 style congestion pricing:

| Parameter | Value |
|-----------|-------|
| MIN_FEE | 0.001 PROVA |
| Target block utilization | 50% |
| Adjustment factor | ±12.5% per block |

### 6.2 Inference Commit Fee

| Parameter | Value |
|-----------|-------|
| BASE_COMMIT_FEE | 0.1 PROVA |
| STORAGE_RATE | 0.001 PROVA per KB |

### 6.3 Model Registration Fee

| Parameter | Value |
|-----------|-------|
| Registration fee | 100 PROVA (burnt) |

## 7. Payment Channels

| Parameter | Value |
|-----------|-------|
| Minimum channel deposit | 1 PROVA |
| Settlement delay | 100 epochs |
| Network fee | 0.5% of settled amount (burnt) |

## 8. Deflationary Mechanics

Fee burns create deflationary pressure post-halving:
- 50% of transaction fees burnt
- 100% of model registration fees burnt
- 0.5% of payment channel settlements burnt
- 100% of failed challenge bonds burnt

At sufficient utilization, PROVA becomes net-deflationary after early halving cycles.

## 9. Economic Security Analysis

### Attack Cost

For cheating to be profitable:
```
S > ((1 - r) / r) × s
```
Where S = stake, r = audit rate (5%), s = single job reward.

With 10,000 PROVA minimum stake and typical 0.1-1 PROVA inference fees, the security margin is 10,000-100,000x.

### Detection Probability

| Jobs cheated | P(detection) |
|---|---|
| 1 | 5% |
| 10 | 40% |
| 50 | 92% |
| 100 | 99.4% |

### Sybil Resistance

Stake-weighted selection. Splitting stake across N identities provides no advantage.

## 10. Governance

All economic parameters are governable:

| Parameter | Governance delay |
|-----------|-----------------|
| Block reward | 30-day timelock |
| Slash fractions | 14-day timelock |
| Fee parameters | 7-day timelock |
| Minimum stake | 14-day timelock |

## 11. Implementation Notes

- All amounts stored as `u128` in smallest denomination (10^-18 PROVA)
- Halving computed lazily per-block
- Fee burns reduce `total_supply` in state trie
- Staking: `chain/src/stake.rs`
- Payments: `chain/src/payment.rs`
- Balances: `chain/src/state.rs`
