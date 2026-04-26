# PROVA Tokenomics — v1.0, April 2026

> Reference document for [`whitepaper.html`](https://prova.network/whitepaper.html) §4 ("Token economics"). The deployed contracts are the source of truth.

---

## TL;DR

**PROVA** is the protocol token of the Prova storage network, deployed on Base. It plays three roles in the protocol:

1. **Prover stake.** Provers post PROVA as their slashable bond. Storage capacity is gated by stake.
2. **Fee burn.** The 1% protocol fee on USDC client payments is auto-routed through a Uniswap V3 PROVA-USDC pool, swapped to PROVA, and burned. Network revenue → permanent supply reduction.
3. **Governance.** PROVA holders vote on a bounded set of protocol parameters with a 2-day timelock.

Clients pay storage fees in **USDC**. Provers receive 99% of payments in **USDC**. Provers also earn **PROVA emission** proportional to bytes-proven-time, on top of their USDC income. The PROVA token's economic role lives entirely in the prover-stake loop, the fee-burn loop, and the prover-emission loop. The day-to-day client experience is boring stablecoin pricing.

- **Total supply:** 100,000,000 PROVA, fixed, non-mintable.
- **Decimals:** 18.
- **Standard:** ERC-20 + ERC-20 Permit + ERC-20 Burnable.
- **Network:** Base (mainnet) and Base Sepolia (testnet).
- **TGE:** target H2 2026, gated on completion of an external audit and a public testnet milestone.

---

## Allocation

The supply splits across three layers: **genesis distribution** (45%), **prover emission over 8 years** (50%), and **ecosystem + community** (5%).

| Layer / Bucket | Share | Tokens (PROVA) | Vesting |
| --- | ---: | ---: | --- |
| **GENESIS DISTRIBUTION** | **45%** | **45,000,000** | (mostly vested) |
| Public sale (TGE / LBP) | 6% | 6,000,000 | Unlocked at TGE |
| Private SAFT round | 12% | 12,000,000 | 12-month cliff, 24-month linear thereafter |
| Team and core engineers | 14% | 14,000,000 | 12-month cliff, 36-month linear |
| Advisors / BD / sales / design | 4% | 4,000,000 | 12-month cliff, 36-month linear |
| Treasury / community | 6% | 6,000,000 | 5-year linear release to multisig |
| Liquidity (DEX seeding) | 3% | 3,000,000 | LP tokens locked 24 months |
| **PROVER EMISSION** | **50%** | **50,000,000** | 8-year declining curve |
| Year 1 emission | 12.5% | 12,500,000 | weekly, distributed by `ProverRewards` |
| Year 2 emission | 11.0% | 11,000,000 | weekly |
| Year 3 emission | 9.0% | 9,000,000 | weekly |
| Year 4 emission | 7.0% | 7,000,000 | weekly |
| Year 5 emission | 5.0% | 5,000,000 | weekly |
| Year 6 emission | 3.0% | 3,000,000 | weekly |
| Year 7 emission | 1.5% | 1,500,000 | weekly |
| Year 8 emission | 1.0% | 1,000,000 | weekly |
| **ECOSYSTEM + COMMUNITY** | **5%** | **5,000,000** | (multi-year) |
| Ecosystem grants | 3% | 3,000,000 | Released as merit-based grants by treasury multisig |
| Community / referral program | 2% | 2,000,000 | Released for client-acquisition referral payouts and early-tester rewards |
| **TOTAL** | **100%** | **100,000,000** | |

**Genesis vested allocation breakdown:**

```
Public sale (LBP)         6,000,000  ████████
Private SAFT             12,000,000  ████████████████
Team & core              14,000,000  ███████████████████
Advisors / BD / sales     4,000,000  █████
Treasury / community      6,000,000  ████████
Liquidity (DEX)           3,000,000  ████
                         ──────────
Genesis subtotal         45,000,000  ████████████████████████████████████████████████████████████ (45%)

Prover emission (8y)     50,000,000  ████████████████████████████████████████████████████████████████ (50%)
Ecosystem + community     5,000,000  ███████ (5%)
                         ──────────
                        100,000,000
```

Insider allocation (SAFT + team + advisors) is **30%** — comfortably below the 35% line that triggers CEX listing concerns. Supply-side allocation (provers) is **50%** — half of the network's tokens go to the people who actually run the storage, paid out as they prove they are storing it.

All vesting is enforced on-chain. Genesis schedules use [`ProvaVesting`](https://github.com/prova-network/contracts/blob/main/src/ProvaVesting.sol). Prover emission is paid by [`ProverRewards`](https://github.com/prova-network/contracts/blob/main/src/ProverRewards.sol).

## Economic role in detail

### 1. Prover stake (PROVA)

A prover registers with [`ProverRegistry`](https://github.com/prova-network/contracts/blob/main/src/ProverRegistry.sol) and posts a stake to [`ProverStaking`](https://github.com/prova-network/contracts/blob/main/src/ProverStaking.sol). The stake denomination is PROVA. Capacity is gated by stake: `minStakePerGiB × committedGiB`.

Stake is slashed (PROVA destroyed) on three triggers:

- **Missed challenges**: prover fails to respond to N consecutive on-chain challenges.
- **Wrong proof**: prover submits a proof that doesn't verify against the committed piece-CID.
- **Withholding**: prover refuses retrieval after the deal has been accepted.

Slashing destroys a fixed `slashFraction` (governance-set, default 10%) of the offending prover's stake. The destroyed PROVA is **permanently removed from supply** — a deflationary force funded by misbehavior.

**Volatility mitigation**: minimum-stake requirements include a USDC-equivalent floor read from a Chainlink PROVA/USD oracle. If PROVA drops sharply, provers have a 7-day grace window to top up stake before they're paused from accepting new deals.

### 2. Fee burn (PROVA)

The marketplace forwards its 1% USDC fee to [`FeeRouter`](https://github.com/prova-network/contracts/blob/main/src/FeeRouter.sol). The router runs in one of three modes set by governance:

| Mode | Behavior |
| --- | --- |
| `HOLD` | Fees accumulate as USDC; nothing is swapped or burned. Default before TGE. |
| `BURN` | All fees auto-swap USDC → PROVA on Uniswap V3 and burn the PROVA. |
| `SPLIT` | A configurable share (default 50%) is burned; the rest is held in the FeeRouter for treasury operations. |

`process(minProvaOut)` is **permissionless** — anyone can call it. PROVA holders get programmatic, transparent buy-pressure proportional to network revenue.

### 3. Prover emission (PROVA)

[`ProverRewards`](https://github.com/prova-network/contracts/blob/main/src/ProverRewards.sol) holds the 50M PROVA emission bucket and pays out per epoch (7 days) based on actual bytes-proven contributions:

```
reward(prover, epoch) = epoch_emission ×
                       (provenBytes(prover) / totalProvenBytes) ×
                       qualityMultiplier
```

**Anti-gaming protections built in:**

| Vector | Mitigation |
| --- | --- |
| Self-dealing (prover stores own data) | `prover != client` check; self-dealing reverts |
| Sponsored / free-tier farming | Sponsored deals (`client == address(0)`) don't generate emission |
| Replication double-claim | Per-piece redundancy cap (default N=4); copies beyond cap don't earn |
| Per-epoch double-counting | `(piece, prover, epoch)` deduped; one credit per piece per prover per epoch |
| Fast-churn (sign up, take a few deals, leave) | Emission has a 30-day vesting buffer after the epoch ends |
| Quality regression | If a prover's missed-proof rate exceeds 5% in trailing 30 days, emission is cut by 50% |
| Sybil identities | Hobby tier capped at 100 TB without identity attestation. Above 100 TB requires lightweight ENS / EAS attestation. Above 5 PB requires KYB. |

Emission decays over 8 years: 25% / 22% / 18% / 14% / 10% / 6% / 3% / 2% of the bucket per year. Year 1 is heaviest to bootstrap the network; year 8 is a tail. After year 8 no further emission is paid; the prover's income is purely from USDC deal payments.

### 4. Governance

PROVA-weighted vote (one-PROVA-one-vote at v1; we'll evaluate quadratic-voting alternatives if it materially reduces whale capture) over:

- Protocol fee tier (currently 1%, hard-capped at 3%)
- Slash fraction (currently 10%, hard-capped at 25%)
- Minimum stake multiplier
- Prover registry admission rules
- `ProofVerifier` UUPS upgrade authority
- `FeeRouter` mode and burn-share
- `ProverRewards` redundancy cap and quality cutoff

Parameter changes go through a 2-day timelock. Contract upgrades go through a 7-day timelock. A 5-of-9 multisig can pause the system in emergencies but cannot redirect funds.

### 5. What clients see

Clients pay storage fees in **USDC**. They don't need to hold or interact with PROVA at any point in the upload flow.

```
Client uploads → 99% USDC streams to prover
                 1% USDC routes to FeeRouter → swap → burn PROVA
```

### 6. What provers see

```
Prover earns:    USDC (99% of fee stream, per deal, per day)
                 + PROVA emission (epoch-proportional, vested 30 days)
Prover stakes:   PROVA (one-time bond, refundable on unbond)
Prover risks:    PROVA slashing (loss of bond on misbehavior)
```

The economic ratchet: **honest provers earn USDC + PROVA emission and reclaim their PROVA bond; dishonest provers lose both incomes and the bond.** Every prover earns the same per-byte rate. The only way to earn more PROVA is to prove more bytes.

---

## What PROVA is **not**

- **Not** required to be a client.
- **Not** the gas token. Base ETH is gas.
- **Not** a payment unit. Storage fees are USDC.
- **Not** a privacy primitive. Bytes are stored as committed; encrypt before upload.
- **Not** an investment. We do not promise, project, or guarantee any future value.
- **Not** classified as a security in our intended structure, but final classification depends on jurisdiction and we will adjust if a regulator says otherwise.

---

## Compliance posture (high level)

- **Counsel:** retained [TBD].
- **Jurisdiction-of-record:** Norway (TSE Reiersen, Org. no. 929 074 912).
- **MiCA white paper:** to be filed before any EU public sale. Pre-MiCA filing, EU-based grants under §4 use the standard private-placement carve-out for service providers.
- **US persons:** the SAFT round will be structured to comply with Reg D 506(c) (accredited investors only), with a backup posture of not targeting US persons.
- **Securities classification**: PROVA's design (in-protocol stake utility, deflationary fee burn, supply-side emission tied to actual storage proven, no rights to underlying revenue per token) is intended to fall outside both Howey-test US security and MiCA "asset-referenced token" categories. Final classification depends on counsel review.

## Implementation status

### Contracts deployed and tested

| Contract | Purpose | Tests |
| --- | --- | --- |
| [`ProvaToken.sol`](https://github.com/prova-network/contracts/blob/main/src/ProvaToken.sol) | Fixed-supply ERC-20 (100M) with Permit and Burnable | 9/9 passing |
| [`ProvaVesting.sol`](https://github.com/prova-network/contracts/blob/main/src/ProvaVesting.sol) | Owner-administered vesting with cliff, linear, revocability, acceleration | 24/24 passing |
| [`FeeRouter.sol`](https://github.com/prova-network/contracts/blob/main/src/FeeRouter.sol) | Routes marketplace fees to PROVA burn / treasury split | 17/17 passing |
| [`ProverRewards.sol`](https://github.com/prova-network/contracts/blob/main/src/ProverRewards.sol) | 50M PROVA emission over 8 years, anti-gaming hooks, epoch claims | 26/26 passing |
| [`StorageMarketplace.sol`](https://github.com/prova-network/contracts/blob/main/src/StorageMarketplace.sol) | Deal lifecycle, USDC escrow, slashing, emission record-proof hook | (covered by Integration suite) |
| [`ProverStaking.sol`](https://github.com/prova-network/contracts/blob/main/src/ProverStaking.sol) | PROVA-denominated prover stake with slashing | (covered by Integration suite) |
| [`ProverRegistry.sol`](https://github.com/prova-network/contracts/blob/main/src/ProverRegistry.sol) | Prover identity, capacity advertisement | (covered by Integration suite) |
| [`ContentRegistry.sol`](https://github.com/prova-network/contracts/blob/main/src/ContentRegistry.sol) | Optional metadata + ENS contenthash binding | (covered by Integration suite) |
| Integration suite | Full deal lifecycle with separate USDC payment + PROVA stake | 5/5 passing |

**81 tests total** across all suites. All pass.

### Deployment plan

| Phase | Network | What happens |
| --- | --- | --- |
| Now | Base Sepolia | Deploy `ProvaToken`, `ProvaVesting`, `ProverRewards`, `FeeRouter`, `StorageMarketplace`, `ProverStaking` for shakedown. Treasury is a multisig; allocations not transferable until TGE. FeeRouter starts in `HOLD` mode. ProverRewards starts emitting on epoch 0 with the testnet's small prover set. |
| Pre-TGE | Base Sepolia | SAFT closes. Vesting agreements signed. Schedules created on-chain. |
| TGE (H2 2026) | Base mainnet | Deploy production contracts. Migrate signed agreements 1:1. Seed DEX liquidity. Public LBP. FeeRouter mode → `BURN`. ProverRewards genesis epoch starts. |
| Post-TGE | Base mainnet | Standard governance kicks in. Quarterly treasury report. |

The Base Sepolia deployment is for shakedown only. **No tokens minted on Base Sepolia have any economic value or any claim on the production tokens.**

## Points program (testnet)

In parallel with testnet deployment, we run a **points program** for prover operators, content uploaders, and contributors. Points are non-transferable, non-tradable, and explicitly not a token. Points convert at TGE to a **published fraction** of the **Ecosystem Grants** allocation (3% / 3M PROVA total). The conversion ratio is announced before testnet ends.

Categories:

- **Prover operator**: points per TB-day proved on testnet (this serves as a dry-run of the production ProverRewards mechanism).
- **Client**: points per GB-month stored, capped to discourage wash-uploads.
- **Contributor**: discretionary points awarded by maintainers for merged PRs, with a public bonus tier table.
