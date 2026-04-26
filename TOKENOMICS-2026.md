# PROVA Tokenomics — v2, April 2026

> This document supersedes Section 4.4 of the v0.9 whitepaper.
> See [the whitepaper amendment](#whitepaper-amendment) at the bottom.

---

## TL;DR

**PROVA** is the protocol token of the Prova storage network, deployed on Base. It plays three roles in the protocol:

1. **Prover stake.** Provers post PROVA as their slashable bond. Storage capacity is gated by stake.
2. **Fee burn.** The 1% protocol fee on USDC client payments is auto-routed through a Uniswap V3 PROVA-USDC pool, swapped to PROVA, and burned. Network revenue → permanent supply reduction.
3. **Governance.** PROVA holders vote on a bounded set of protocol parameters with a 2-day timelock.

Clients still pay storage fees in **USDC**. Provers still receive 99% of payments in **USDC**. The PROVA → USDC interaction sits in two narrow places: prover stake (one-time, refundable on unbond) and fee burn (a swap done by a public keeper). The day-to-day client experience is boring stable-coin pricing.

- **Total supply:** 100,000,000 PROVA, fixed, non-mintable.
- **Decimals:** 18.
- **Standard:** ERC-20 + ERC-20 Permit + ERC-20 Burnable.
- **Network:** Base (mainnet) and Base Sepolia (testnet).
- **Burn schedule:** dynamic, equal to network revenue × 1% × (PROVA/USDC market price).
- **TGE:** target H2 2026, gated on completion of an external audit and a public testnet milestone.

---

## Allocation

| Bucket | Share | Tokens (PROVA) | Vesting |
| --- | ---: | ---: | --- |
| Public sale (TGE / LBP) | 8% | 8,000,000 | Unlocked at TGE |
| Private SAFT round | 17% | 17,000,000 | 12-month cliff, 24-month linear thereafter (3y total) |
| Team and core engineers | 18% | 18,000,000 | 12-month cliff, 36-month linear (4y total) |
| Advisors / BD / sales / design | 7% | 7,000,000 | 12-month cliff, 36-month linear (4y total) |
| Ecosystem grants | 10% | 10,000,000 | 5-year drip, multisig-administered |
| Liquidity (DEX seeding) | 5% | 5,000,000 | LP tokens locked 24 months |
| Treasury / community | 20% | 20,000,000 | 5-year linear release to multisig |
| Protocol incentives (provers / users) | 15% | 15,000,000 | Released as the protocol uses them — no time vesting |
| **Total** | **100%** | **100,000,000** | |

Insider allocations (SAFT + team + advisors = 42%) is the upper bound of "still acceptable to credible CEXes." Public-leaning allocations (public + ecosystem + liquidity + protocol incentives = 38%) keep float at TGE healthy without the kind of insider-heavy structure that taints a launch.

All vesting schedules are enforced on-chain by [`ProvaVesting`](https://github.com/prova-network/contracts/blob/main/src/ProvaVesting.sol). Off-chain memoranda (vesting agreements per individual / SAFT contracts per investor) memorialise the legal grant; the on-chain schedule is the source of truth for what vests when.

## Economic role in detail

### 1. Prover stake (PROVA)

A prover registers with [`ProverRegistry.sol`](https://github.com/prova-network/contracts/blob/main/src/ProverRegistry.sol) and posts a stake to [`ProverStaking.sol`](https://github.com/prova-network/contracts/blob/main/src/ProverStaking.sol). The stake denomination is PROVA. Capacity is gated by stake: `minStakePerGiB × committedGiB`.

Stake is slashed (PROVA burned) on three triggers:

- **Missed challenges**: prover fails to respond to N consecutive on-chain challenges.
- **Wrong proof**: prover submits a proof that doesn't verify against the committed piece-CID.
- **Withholding**: prover refuses retrieval after the deal has been accepted.

Slashing destroys a fixed `slashFraction` (governance-set, default 10%) of the offending prover's stake. The destroyed PROVA is **permanently removed from supply** — the same effect as a buy-and-burn, but funded by misbehavior rather than network revenue.

**Volatility mitigation**: minimum-stake requirements include a USDC-equivalent floor read from a Chainlink PROVA/USD oracle (`max(absoluteFloor, oracleEquivalent)`). If PROVA drops sharply, provers have a 7-day grace window to top up stake before they're paused from new deals. This protects honest provers from a flash crash.

### 2. Fee burn (PROVA)

Implemented by [`FeeRouter.sol`](https://github.com/prova-network/contracts/blob/main/src/FeeRouter.sol). The marketplace forwards its 1% USDC fee to the FeeRouter. The router runs in one of three modes set by governance:

| Mode | Behavior |
| --- | --- |
| `HOLD` | Fees accumulate as USDC; nothing is swapped or burned. Default before TGE. |
| `BURN` | All fees auto-swap USDC → PROVA on Uniswap V3 and burn the PROVA. |
| `SPLIT` | A configurable share (default 50%) is burned; the rest is held in the FeeRouter for treasury operations (grants, audits, BD costs). |

`process(minProvaOut)` is **permissionless** — anyone can call it. Slippage is bounded by the caller-supplied `minProvaOut`. The owner sets a `maxSwapPerCall` cap to bound the per-call market impact.

This means: **PROVA holders get programmatic, transparent buy-pressure proportional to network revenue**. Not magic, not hand-wavy — just a permissionless function that anyone can run when fees pile up. The total burn rate scales with the size of the storage market.

### 3. Governance

PROVA-weighted vote (one-PROVA-one-vote at v1; we'll evaluate quadratic-voting alternatives if it materially reduces whale capture) over:

- Protocol fee tier (currently 1%, hard-capped at 3% in code)
- Slash fraction (currently 10%, hard-capped at 25%)
- Minimum stake multiplier
- Prover registry admission rules
- `ProofVerifier` UUPS upgrade authority
- `FeeRouter` mode and burn-share

All parameter changes go through a 2-day timelock. Contract upgrades go through a 7-day timelock. A 5-of-9 multisig can pause the system in emergencies (e.g. discovered exploit) but cannot redirect funds.

### 4. What clients see

Clients pay storage fees in **USDC**. They don't need to hold or interact with PROVA at any point in the upload flow.

```
Client uploads → 99% USDC streams to prover
                 1% USDC routes to FeeRouter → swap → burn PROVA
```

The same boring USDC UX as any centralized object-storage service. PROVA's economic role happens in the background; clients see piece-CIDs and stable-coin invoices.

### 5. What provers see

Provers earn **USDC** for the storage service they provide. They stake **PROVA** as a refundable bond against their honest behavior. Stake is slashable. Stake is denominated in PROVA but with a USDC-equivalent floor.

```
Prover earns:    USDC (99% of fee stream, per deal, per day)
Prover stakes:   PROVA (one-time bond, refundable on unbond)
Prover risks:    PROVA slashing (loss of bond on misbehavior)
```

The economic ratchet is: **honest provers earn USDC and reclaim their PROVA; dishonest provers lose both the income and the bond.** That ratchet is the entire reason the protocol works.

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
- **MiCA white paper:** to be filed before any EU public sale. Pre-MiCA filing, EU-based grants under §4.4 use the standard private-placement carve-out for service providers.
- **US persons:** the SAFT round will be structured to comply with Reg D 506(c) (accredited investors only), with a backup posture of not targeting US persons.
- **Securities classification**: PROVA's design (in-protocol stake utility, burn-from-fees deflationary mechanism, no rights to underlying revenue per token) is intended to fall outside both Howey-test US security and MiCA "asset-referenced token" categories. Final classification depends on counsel review.

Detailed compliance, sale mechanics, and investor outreach plan: [`TOKEN-MODEL-V2-2026-04-26.md`](./TOKEN-MODEL-V2-2026-04-26.md) (internal).

## Implementation status

### Contracts deployed and tested

| Contract | Purpose | Tests |
| --- | --- | --- |
| [`ProvaToken.sol`](https://github.com/prova-network/contracts/blob/main/src/ProvaToken.sol) | Fixed-supply ERC-20 with Permit and Burnable | 9/9 passing |
| [`ProvaVesting.sol`](https://github.com/prova-network/contracts/blob/main/src/ProvaVesting.sol) | Owner-administered vesting with cliff, linear, revocability, acceleration | 24/24 passing |
| [`FeeRouter.sol`](https://github.com/prova-network/contracts/blob/main/src/FeeRouter.sol) | Routes marketplace fees to PROVA burn / treasury split | 17/17 passing |
| [`ProverStaking.sol`](https://github.com/prova-network/contracts/blob/main/src/ProverStaking.sol) | PROVA-denominated prover stake with slashing | 6/6 passing |

55 tests total across the protocol. All pass.

### Deployment plan

| Phase | Network | What happens |
| --- | --- | --- |
| Now | Base Sepolia | Deploy `ProvaToken`, `ProvaVesting`, `FeeRouter`, `StorageMarketplace`, `ProverStaking` for shakedown. Treasury is the founder address; allocations not transferable until TGE. FeeRouter starts in `HOLD` mode. |
| Pre-TGE | Base Sepolia | SAFT closes. Vesting agreements signed. Schedules created on-chain. |
| TGE (H2 2026) | Base mainnet | Deploy production contracts. Migrate signed agreements 1:1. Seed DEX liquidity. Public LBP. FeeRouter mode → `BURN`. |
| Post-TGE | Base mainnet | Standard governance kicks in. Quarterly treasury report. |

The Base Sepolia deployment is for shakedown only. **No tokens minted on Base Sepolia have any economic value or any claim on the production tokens.**

## Points program (testnet)

In parallel with testnet deployment, we run a **points program** for prover operators, content uploaders, and contributors. Points are non-transferable, non-tradable, and explicitly not a token. Points convert at TGE to a **published fraction** of the **Ecosystem Grants** allocation (10% / 10M PROVA total). The conversion ratio is announced before testnet ends.

The points formula is published before testnet starts. Categories:

- **Prover operator**: points per TB-day proved, multiplier for early enrolment + continuous uptime.
- **Client**: points per GB-month stored, capped to discourage wash-uploads.
- **Contributor**: discretionary points awarded by maintainers for merged PRs, with a public bonus tier table.

## Whitepaper amendment

The following amends the v0.9 whitepaper. It will appear inline in the next published version (v1.1) of the whitepaper.

> **Amendment 2, 2026-04-26.** Section 4.4 of v0.9 ("Why no token") is replaced with the following:
>
> *Prova is not a tokenless protocol. PROVA is the protocol's stake and governance token, used for prover bonds, deflationary fee burn, and parameter governance. Clients pay in USDC and provers earn USDC; PROVA's role is constrained to the prover-stake economic loop and the fee-burn mechanism that ties protocol revenue to PROVA supply. Full tokenomics in [TOKENOMICS-2026.md](./TOKENOMICS-2026.md).*
>
> *The original v0.9 framing argued against a token because the protocol's economic surface (settlement, payment, speculation) doesn't require one. We have changed our position. The protocol still doesn't require PROVA for client-facing settlement (USDC remains the unit of account), but using PROVA as the prover-stake instrument and the fee-burn target means the token has a constant, transparent, programmatic role in the protocol's economic loop. This makes PROVA more than a governance vehicle — it is the alignment instrument between provers, clients, and the operating org.*
>
> *We acknowledge that this is a meaningful shift from v0.9. The shift is forced by two operating realities: (a) we need runway, and a token sale produces useful runway in a way a small SAFE round does not; (b) without a slashable stake, prover behavior reduces to "honor system + USDC bond," which is weaker than "honor system + slashable PROVA bond + reputational consequence."*
>
> *We reserve the right to change our minds again. Any change here would land in a further numbered amendment to this document.*
