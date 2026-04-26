# 5.1 Token economics

The full canonical document is [`prova-network/prova/TOKENOMICS-2026.md`](https://github.com/prova-network/prova/blob/main/TOKENOMICS-2026.md). This section reproduces the protocol-relevant parts in normative form.

## 5.1.1 Token parameters

| Parameter | Value |
| --- | --- |
| Symbol | PROVA |
| Decimals | 18 |
| Total supply | 100,000,000 (100M) |
| Minting | Fixed at genesis. No mint authority. |
| Standard | ERC-20 + ERC-20 Permit + ERC-20 Burnable |
| Network | Base mainnet, Base Sepolia (testnet) |
| Implementation | [`ProvaToken.sol`](https://github.com/prova-network/contracts/blob/main/src/ProvaToken.sol) |

## 5.1.2 Allocation

The 100M supply splits across three layers: **genesis distribution** (45M, 45%), **prover emission over 8 years** (50M, 50%), and **ecosystem + community** (5M, 5%).

| Layer / Bucket | Share | Tokens (PROVA) | Vesting |
| --- | ---: | ---: | --- |
| **Genesis distribution** | **45%** | **45,000,000** | (mostly vested) |
| Public sale (TGE / LBP) | 6% | 6,000,000 | 100% at TGE, no lock |
| Private SAFT round | 12% | 12,000,000 | 12-month cliff, 24-month linear thereafter |
| Team and core engineers | 14% | 14,000,000 | 12-month cliff, 36-month linear |
| Advisors / BD / sales / design | 4% | 4,000,000 | 12-month cliff, 36-month linear |
| Treasury / community | 6% | 6,000,000 | 5-year linear release to multisig |
| Liquidity (DEX seeding) | 3% | 3,000,000 | LP tokens locked 24 months |
| **Prover emission** | **50%** | **50,000,000** | 8-year declining curve via [`ProverRewards.sol`](https://github.com/prova-network/contracts/blob/main/src/ProverRewards.sol) |
| Year 1 | 12.5% | 12,500,000 | weekly per-epoch |
| Year 2 | 11.0% | 11,000,000 | weekly |
| Year 3 | 9.0% | 9,000,000 | weekly |
| Year 4 | 7.0% | 7,000,000 | weekly |
| Year 5 | 5.0% | 5,000,000 | weekly |
| Year 6 | 3.0% | 3,000,000 | weekly |
| Year 7 | 1.5% | 1,500,000 | weekly |
| Year 8 | 1.0% | 1,000,000 | weekly |
| **Ecosystem + community** | **5%** | **5,000,000** | (multi-year) |
| Ecosystem grants | 3% | 3,000,000 | merit-based grants from treasury multisig |
| Community / referrals | 2% | 2,000,000 | referral program + early-tester rewards |
| **Total** | **100%** | **100,000,000** | |

Genesis schedules MUST be enforced on-chain by `ProvaVesting.sol`. Prover emission MUST be paid by `ProverRewards.sol`.

## 5.1.3 Prover stake

A prover MUST register with `ProverRegistry.sol` and post a stake to `ProverStaking.sol` before accepting deals. The stake denomination is PROVA. Capacity is gated by stake:

```
maxCommittedBytes(prover) = staked(prover) / minStakePerGiB
```

`minStakePerGiB` is governance-tunable. Initial value at TGE: **100 PROVA per GiB**.

### USDC-equivalent floor

To protect honest provers from a PROVA price drop, the effective minimum stake includes a USDC floor read from a Chainlink PROVA/USD oracle:

```
minStake_effective(GiB) = max(
    minStakePerGiB × GiB,
    targetUSD(GiB) / oracle_PROVA_USD
)
```

If `minStake_effective` rises above current stake, the prover SHALL have a 7-day grace window to top up before being paused from accepting new deals. Already-active deals are unaffected for the duration of their committed term.

### Slashing

Slash triggers (per `StorageMarketplace.sol`):

- **Missed challenges**: ≥ N consecutive missed proofs over `MAX_PROOF_GAP` window
- **Wrong proof**: proof submission that fails `ProofVerifier.verifyProof()`
- **Withholding**: prover refuses retrieval after deal acceptance (off-chain attestation, on-chain governance vote)

Slash amount per fault: `slashPerFault` PROVA, governance-tunable. Initial value: **50 PROVA**, hard-capped at **25%** of locked stake per single event.

Slashed PROVA MUST be destroyed (transferred to `address(0)` via `ERC20Burnable.burn`).

## 5.1.4 Fee burn

The marketplace contract takes a 1% USDC fee on every deal payment stream and forwards it to `FeeRouter.sol`. Hard-cap on protocol fee: **3%** (governance can raise toward this cap with a 2-day timelock).

`FeeRouter` modes:

| Mode | Behavior |
| --- | --- |
| `HOLD` | USDC accumulates. Default before TGE. |
| `BURN` | All USDC swaps to PROVA on Uniswap V3 (default 0.3% pool), PROVA is burned. |
| `SPLIT` | A `burnShareBps` portion (default 50%) is swapped + burned; the rest is held in the FeeRouter for treasury operations. |

`process(minProvaOut)` MUST be permissionless. Slippage is bounded by the caller-supplied `minProvaOut`. The owner sets `maxSwapPerCall` to bound per-call market impact.

## 5.1.5 Prover emission

[`ProverRewards.sol`](https://github.com/prova-network/contracts/blob/main/src/ProverRewards.sol) holds the 50M PROVA emission bucket and pays out per epoch (7 days) based on bytes-proven contributions.

### Reward formula

```
reward(prover, epoch) = epoch_emission × (provenBytes(prover) / totalProvenBytes) × qualityMultiplier
```

where `epoch_emission = yearlyEmission[year(epoch)] × (EPOCH_DURATION / 365 days)`.

### Anti-gaming protections

Every protection below MUST be enforced by the contract; loose off-chain implementation is not sufficient.

| Vector | Mitigation | Enforcement |
| --- | --- | --- |
| Self-dealing (prover == client) | `recordProof` reverts with `SelfDealing` | on-chain |
| Sponsored / free-tier farming | `client == address(0)` skips emission accounting | on-chain |
| Replication double-claim | Per-piece per-epoch redundancy cap (default `redundancyCap = 4`) | on-chain |
| Per-epoch double-counting | `(piece, prover, epoch)` deduped in `countedInEpoch` | on-chain |
| Fast-churn | 30-day vesting buffer between epoch end and claimability | on-chain |
| Quality regression | If trailing-30d miss-rate > `qualityCutoffBps` (5%), `qualityMultiplier = 0.5` | on-chain |
| Sybil identities | Tier-gated identity attestation (hobby pseudonymous; prosumer ENS/EAS; enterprise KYB) | off-chain (registration gate) |

### Governance-tunable parameters

- `redundancyCap` (default 4, max 16)
- `qualityCutoffBps` (default 500 = 5%, max 5000 = 50%)
- `marketplace` (the authorized recorder address)

All changes go through the standard 2-day timelock.

## 5.1.6 Governance

PROVA-weighted vote (one-PROVA-one-vote at v1) over:

| Parameter | Hard cap | Timelock |
| --- | --- | --- |
| `protocolFeeBps` | 300 (3%) | 2 days |
| `slashFraction` | 2500 (25%) | 2 days |
| `minStakePerGiB` | governance-set | 2 days |
| Prover registry admission rules | n/a | 2 days |
| `ProofVerifier` UUPS upgrade authority | n/a | 7 days |
| `FeeRouter.mode` and `burnShareBps` | n/a | 2 days |
| `ProverRewards.redundancyCap` | 16 | 2 days |
| `ProverRewards.qualityCutoffBps` | 5000 | 2 days |

A 5-of-9 multisig holds emergency pause authority. The pause cannot redirect funds; it only halts new deal acceptances and challenge grading until governance unpauses.

## 5.1.7 Public sale

### SAFT round (private, pre-TGE)

- Target raise: $1.5M – $3M USDC
- Target tokens: 12,000,000 PROVA (12%)
- Vesting: 12-month cliff, 24-month linear thereafter
- Compliance: Reg D 506(c) for US accredited investors; Reg S for non-US; private placement carve-out under MiCA
- Counsel engagement: see [`prova-network/prova/legal/`](https://github.com/prova-network/prova/tree/main/legal) (public templates only)

### Public sale at TGE

- Mechanism: Liquidity Bootstrapping Pool (LBP) on a Base launchpad
- Tokens: 6,000,000 PROVA (6%)
- Pricing: dynamic, weighted-pool decay over 24-72 hours
- CEX listing: Kraken targeted within 3-6 months of TGE; Coinbase / Binance only after demonstrated organic volume

## 5.1.8 Implementation reference

| Contract | File | Tests |
| --- | --- | --- |
| ProvaToken | [`src/ProvaToken.sol`](https://github.com/prova-network/contracts/blob/main/src/ProvaToken.sol) | 9/9 |
| ProvaVesting | [`src/ProvaVesting.sol`](https://github.com/prova-network/contracts/blob/main/src/ProvaVesting.sol) | 24/24 |
| ProverRewards | [`src/ProverRewards.sol`](https://github.com/prova-network/contracts/blob/main/src/ProverRewards.sol) | 26/26 |
| FeeRouter | [`src/FeeRouter.sol`](https://github.com/prova-network/contracts/blob/main/src/FeeRouter.sol) | 17/17 |
| StorageMarketplace | [`src/StorageMarketplace.sol`](https://github.com/prova-network/contracts/blob/main/src/StorageMarketplace.sol) | (Integration) |
| ProverStaking | [`src/ProverStaking.sol`](https://github.com/prova-network/contracts/blob/main/src/ProverStaking.sol) | (Integration) |

**81 tests** passing across all suites.
