# SPEC-010: Token Economics Specification

**Status:** v1.0 (specification, pre-TGE)
**Authors:** Prova contributors
**Created:** 2026-03-04
**Updated:** 2026-04-26

## 1. Overview

The Prova network uses a native token, **PROVA**, for prover staking, fee burn, and governance. Storage payments between clients and provers settle in **USDC**. This spec defines issuance, distribution, fee structures, and economic security parameters.

## 2. Token Parameters

| Parameter | Value |
|-----------|-------|
| Symbol | PROVA |
| Decimals | 18 |
| Total supply | 100,000,000 (100M) |
| Minting | Fixed at genesis. No inflation. No mint authority. |
| Standard | ERC-20 + ERC-20 Permit + ERC-20 Burnable |
| Network | Base mainnet (and Base Sepolia for testnet) |

## 3. Allocation

Three layers: **genesis** (45%), **prover emission over 8 years** (50%), **ecosystem + community** (5%).

| Layer / Bucket | % | Tokens (PROVA) | Vesting |
|---|---:|---:|---|
| **GENESIS** | **45%** | **45,000,000** | (mostly vested) |
| Public sale (TGE / LBP) | 6% | 6,000,000 | 100% at TGE, no lock |
| Private SAFT round | 12% | 12,000,000 | 12-month cliff, 24-month linear thereafter |
| Team and core engineers | 14% | 14,000,000 | 12-month cliff, 36-month linear |
| Advisors / BD / sales / design | 4% | 4,000,000 | 12-month cliff, 36-month linear |
| Treasury / community | 6% | 6,000,000 | 5-year linear release to multisig |
| Liquidity (DEX seeding) | 3% | 3,000,000 | LP tokens locked 24 months |
| **PROVER EMISSION** | **50%** | **50,000,000** | 8-year declining curve via `ProverRewards` |
| Year 1 | 12.5% | 12,500,000 | weekly per-epoch |
| Year 2 | 11.0% | 11,000,000 | weekly |
| Year 3 | 9.0% | 9,000,000 | weekly |
| Year 4 | 7.0% | 7,000,000 | weekly |
| Year 5 | 5.0% | 5,000,000 | weekly |
| Year 6 | 3.0% | 3,000,000 | weekly |
| Year 7 | 1.5% | 1,500,000 | weekly |
| Year 8 | 1.0% | 1,000,000 | weekly |
| **ECOSYSTEM + COMMUNITY** | **5%** | **5,000,000** | (multi-year) |
| Ecosystem grants | 3% | 3,000,000 | Released as merit-based grants |
| Community / referrals | 2% | 2,000,000 | Released for referral program + early-tester rewards |
| **Total** | **100%** | **100,000,000** | |

Genesis schedules enforced on-chain by [`ProvaVesting.sol`](https://github.com/prova-network/contracts/blob/main/src/ProvaVesting.sol). Prover emission paid by [`ProverRewards.sol`](https://github.com/prova-network/contracts/blob/main/src/ProverRewards.sol).

## 4. Economic mechanism

### 4.1 Prover stake (PROVA)

Provers register via `ProverRegistry.sol` and post stake to `ProverStaking.sol`. Capacity is gated by stake:

```
maxCommittedBytes(prover) = staked(prover) / minStakePerGiB
```

`minStakePerGiB` is governance-tunable. Initial value at TGE: **100 PROVA per GiB**.

### 4.2 Volatility floor (USDC-equivalent)

Effective minimum stake includes a USDC floor read from a Chainlink PROVA/USD oracle:

```
minStake_effective(GiB) = max(
    100 PROVA × GiB,
    targetUSD(GiB) / oracle_PROVA_USD
)
```

If `minStake_effective` rises above current stake (PROVA price drop), provers have a 7-day grace window before they are paused from accepting new deals. Already-active deals are unaffected for the duration of their committed term.

### 4.3 Slashing

Slash triggers (per `StorageMarketplace.sol`):

- **Missed challenges**: ≥ N consecutive missed proofs over `MAX_PROOF_GAP` window
- **Wrong proof**: proof submission that fails `ProofVerifier.verifyProof()`
- **Withholding**: prover refuses retrieval after deal acceptance (off-chain attestation, on-chain governance vote)

Slash amount per fault: `slashPerFault` PROVA, governance-tunable. Initial value: **50 PROVA per fault**, hard-capped at 25% of locked stake per single event.

Slashed PROVA is **destroyed** (transferred to `address(0)` via `ERC20Burnable.burn`). Net effect: permanent supply reduction.

### 4.4 Fee burn

Marketplace contract takes 1% USDC fee on every deal payment stream and forwards it to `FeeRouter.sol`. Hard-cap on protocol fee: 3% (governance can raise toward this cap with 2-day timelock).

FeeRouter modes:

| Mode | Behavior |
|---|---|
| `HOLD` | USDC accumulates. Default before TGE. |
| `BURN` | All USDC swaps to PROVA on Uniswap V3 (`0.3%` fee tier by default), PROVA is burned. |
| `SPLIT` | A `burnShareBps` portion (default 50%) is swapped + burned; the rest is held in the FeeRouter for treasury operations. |

`process(minProvaOut)` is **permissionless**. Slippage is bounded by the caller-supplied `minProvaOut`. The owner sets `maxSwapPerCall` to bound per-call market impact.

### 4.5 Prover emission (`ProverRewards.sol`)

The 50M PROVA emission bucket is held by `ProverRewards.sol` and paid out per epoch (7 days) based on actual bytes-proven contributions, on the schedule in §3.

Reward formula:

```
reward(prover, epoch) = epoch_emission × (provenBytes(prover) / totalProvenBytes) × qualityMultiplier
```

where `epoch_emission` = `yearlyEmission[year(epoch)] × (EPOCH_DURATION / 365 days)`.

#### 4.5.1 Anti-gaming protections

| Vector | Mitigation |
|---|---|
| Self-dealing (prover == client) | `recordProof` reverts with `SelfDealing` |
| Sponsored / free-tier farming | `client == address(0)` skips emission accounting |
| Replication double-claim | Per-piece per-epoch redundancy cap (default `redundancyCap = 4`) |
| Per-epoch double-counting | `(piece, prover, epoch)` deduped in `countedInEpoch` |
| Fast-churn | 30-day vesting buffer between epoch end and claimability |
| Quality regression | If trailing-30d miss-rate > `qualityCutoffBps` (5%), `qualityMultiplier = 0.5` |
| Sybil identities | Tier-gated identity attestation (hobby ≤ 100 TB pseudonymous; prosumer ENS/EAS; enterprise KYB) |

#### 4.5.2 Governance-tunable parameters

- `redundancyCap` (default 4, max 16) — max provers earning per piece per epoch
- `qualityCutoffBps` (default 500 = 5%, max 5000 = 50%) — miss-rate cutoff for the quality multiplier
- `marketplace` — the authorized recorder address

All changes go through the standard 2-day timelock.

### 4.6 Governance

PROVA-weighted vote (one-PROVA-one-vote at v1) over:

| Parameter | Hard cap | Timelock |
|---|---|---|
| `protocolFeeBps` | 300 (3%) | 2 days |
| `slashFraction` | 2500 (25%) | 2 days |
| `minStakePerGiB` | governance-set | 2 days |
| Prover registry admission rules | n/a | 2 days |
| `ProofVerifier` UUPS upgrade authority | n/a | 7 days |
| `FeeRouter.mode` and `burnShareBps` | n/a | 2 days |

A 5-of-9 multisig holds emergency pause authority. The pause cannot redirect funds; it only halts new deal acceptances and challenge grading until governance unpauses.

## 5. Public sale

### 5.1 SAFT round (private, pre-TGE)

- Target raise: $1.5M – $3M USDC
- Target tokens: 12,000,000 PROVA (12%)
- Vesting: 12-month cliff, 24-month linear thereafter
- Compliance: Reg D 506(c) for US accredited investors; Reg S for non-US; private placement carve-out under MiCA
- Counsel: outside firm engaged for SAFT template, securities-law opinion, MiCA white paper

### 5.2 Public sale at TGE

- Mechanism: Liquidity Bootstrapping Pool (LBP) on a Base launchpad
- Tokens: 6,000,000 PROVA (6%)
- Pricing: dynamic, weighted-pool decay over 24-72 hours
- Listing: Kraken targeted within 3-6 months of TGE; Coinbase / Binance only after demonstrated organic volume

## 6. Treasury operations

The 20% Treasury / Community bucket releases linearly over 5 years to a multisig. Public ledger; quarterly reports. Mandate:

- Audit costs (tier-1 firm pre-mainnet, ongoing audits annually)
- Engineering hires not covered by SAFT runway
- Ecosystem grants paid in PROVA
- Operational costs of the org

If `FeeRouter` runs in `SPLIT` mode (post-TGE governance vote), the held USDC portion supplements the treasury without diluting PROVA.

## 7. Compliance

- Jurisdiction-of-record: Norway (TSE Reiersen, Org. no. 929 074 912)
- MiCA: white paper to be filed before any EU public sale
- US: SAFT under Reg D 506(c); no offering or sale to retail US persons before legal review
- KYC/AML: required for SAFT investors and any allocation > $10K equivalent

## 8. Implementation reference

Source of truth is the deployed contracts. The on-chain values override anything in this spec if they ever diverge.

| Contract | File | Tests |
|---|---|---|
| ProvaToken | [`src/ProvaToken.sol`](https://github.com/prova-network/contracts/blob/main/src/ProvaToken.sol) | 9/9 |
| ProvaVesting | [`src/ProvaVesting.sol`](https://github.com/prova-network/contracts/blob/main/src/ProvaVesting.sol) | 24/24 |
| FeeRouter | [`src/FeeRouter.sol`](https://github.com/prova-network/contracts/blob/main/src/FeeRouter.sol) | 17/17 |
| StorageMarketplace | [`src/StorageMarketplace.sol`](https://github.com/prova-network/contracts/blob/main/src/StorageMarketplace.sol) | (Integration) |
| ProverStaking | [`src/ProverStaking.sol`](https://github.com/prova-network/contracts/blob/main/src/ProverStaking.sol) | (Integration) |

55 tests passing across all suites at the time of this spec.
