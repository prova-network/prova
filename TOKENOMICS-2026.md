# PROVA Tokenomics — v1, April 2026

> This document supersedes Section 4.4 of the v0.9 whitepaper.
> See [the whitepaper amendment](#whitepaper-amendment) at the bottom.

---

## TL;DR

**PROVA** is an ERC-20 governance + fee-share token deployed on Base. It is **not used as gas, not used as payment, and not required to participate in the protocol as a client or as a prover.** Storage deals settle in USDC; provers stake in USDC; slashing burns USDC. PROVA exists to align the team and early contributors who build and operate the protocol.

- **Total supply:** 1,000,000,000 PROVA (1 B), fixed, non-mintable.
- **Decimals:** 18.
- **Standard:** ERC-20 + ERC-20 Permit + ERC-20 Burnable.
- **Network:** Base (mainnet) and Base Sepolia (testnet).
- **TGE:** target H2 2026, gated on completion of an external audit and a public testnet milestone.
- **Liquidity:** modest seeded position on a Base DEX at TGE. No CEX listings until the protocol has real volume.
- **No presale.** No private investor allocation outside the published table below. No airdrop without a published formula.

---

## Allocation

| Bucket | Share | Tokens (PROVA) | Vesting |
| --- | ---: | ---: | --- |
| Team and core engineers | 18% | 180,000,000 | 4-year linear, 1-year cliff |
| Advisors / BD / sales / design | 12% | 120,000,000 | 4-year linear, 1-year cliff (acceleration possible per individual agreement) |
| Early supporters / friends-and-family | 5% | 50,000,000 | 2-year linear, 6-month cliff |
| Ecosystem grants | 10% | 100,000,000 | Released quarterly to grant recipients who shipped |
| Community / treasury / liquidity | 35% | 350,000,000 | 5-year drip from a multisig, public ledger |
| Protocol incentives / staking | 20% | 200,000,000 | Released as the protocol uses them (slashing-insurance, prover bonuses) |
| **Total** | **100%** | **1,000,000,000** | |

All vesting schedules are enforced on-chain by [`ProvaVesting`](https://github.com/prova-network/contracts/blob/main/src/ProvaVesting.sol). Off-chain memoranda (vesting agreements per individual) memorialise the legal grant; the on-chain schedule is the source of truth for what vests when.

## Economic role

PROVA does three things and no more:

1. **Captures fee flow.** The marketplace contract takes a 1% fee on every USDC payment stream. The fee accrues to the treasury. The treasury's stated mandate is to use it for: (a) buy-back-and-burn against PROVA on a public schedule; (b) ecosystem grants paid in PROVA; (c) operational expenses of the org.
2. **Governs protocol parameters.** Token-weighted vote (one-PROVA-one-vote, with a quadratic-voting alternative under consideration) on:
   - the protocol fee tier (currently 1%, hard-capped at 3%)
   - the slash fraction (currently 10% per slashing event, hard-capped at 25%)
   - the minimum stake multiplier
   - the prover-registry admission rules
   - upgrade authority for the `ProofVerifier` UUPS proxy
3. **Earns a fee discount.** Token-holders who lock PROVA into a fee-discount pool receive a proportional reduction on their protocol fee. This is purely a routing benefit; you can use the protocol with or without PROVA at any tier.

## What PROVA is **not**

- **Not** required to be a prover. Provers stake USDC.
- **Not** required to be a client. Clients pay USDC.
- **Not** the gas token. Base ETH is the gas token.
- **Not** an investment. We do not promise, project, or guarantee any future value.
- **Not** a security in our intended classification, but final classification depends on jurisdiction and we will adjust if a regulator says otherwise.

## Why an off-protocol token, when v0.9 said "no token"?

The v0.9 whitepaper Section 4.4 made three arguments against a *protocol* token:

- Governance can be a small multisig.
- Payment should be USDC.
- Speculation does not have to be minted.

All three are still true, and the protocol still does not require a token in any of those roles. This token is **off-protocol**. It is the equity instrument of the org that builds and operates Prova, paid out under a published schedule with on-chain enforcement. It is not woven into deal settlement, prover staking, or slashing. The protocol would function identically if this token did not exist — the token is for the *people who build it*, not for the protocol itself.

This is a deliberate compromise. We are honest about what the token does and does not do. If we ever push token utility into the protocol's hot path (e.g., requiring PROVA stake on top of USDC), that change requires a numbered amendment to the whitepaper and a public consultation period.

## Compliance posture

- **Counsel:** retained [TBD].
- **Jurisdiction-of-record:** Norway (TSE Reiersen, Org. no. 929 074 912).
- **No MiCA white paper today.** Our position: PROVA at issuance is closer to equity than to a stablecoin or a hybrid-instrument and is granted only to people performing services for the org. We do not market PROVA to the general EU public until either (a) an external counsel concludes MiCA does not apply, or (b) we file an MiCA white paper. EU-based recipients of grants under §4.4 receive their PROVA under standard private-placement carve-outs.
- **US persons:** none of this is investment advice, and PROVA is not being offered or sold to US persons through this mechanism.

## Implementation

### Contracts

- [`ProvaToken.sol`](https://github.com/prova-network/contracts/blob/main/src/ProvaToken.sol) — fixed-supply ERC-20.
- [`ProvaVesting.sol`](https://github.com/prova-network/contracts/blob/main/src/ProvaVesting.sol) — owner-administered vesting with cliff, linear vest, revocability, acceleration, and per-beneficiary `claimAll()`.

Both contracts are under the same dual-license as the rest of the protocol (Apache-2.0 OR MIT) and include a 24-test suite that runs in `forge test`.

### Deployment plan

| Phase | Network | What happens |
| --- | --- | --- |
| Now | Base Sepolia | Deploy `ProvaToken` and `ProvaVesting` to testnet for shakedown. Treasury is the founder address; allocations are loaded but **not transferable** until TGE. |
| Pre-TGE | Base Sepolia | Vesting agreements signed off-chain. Schedules created on-chain. Beneficiaries verify their schedule id is correct. |
| TGE (H2 2026) | Base mainnet | Deploy production `ProvaToken` and `ProvaVesting`. Migrate every signed vesting agreement to the production deployment. Seed DEX liquidity from the Community/Treasury allocation. Public TGE announcement. |
| Post-TGE | Base mainnet | Standard governance kicks in. Treasury operates under public ledger. Quarterly reports. |

### Testnet → mainnet migration

The Base Sepolia deployment is for shakedown only. **No tokens minted on Base Sepolia have any economic value or any claim on the production tokens.** At TGE we deploy fresh on Base mainnet and migrate signed agreements 1:1; testnet recipients receive their production allocation at the same address.

This is conservative. We could redeploy without redoing the schedule legwork, but that risks a beneficiary at testnet not getting credit at mainnet through some operational error. The migration step forces us to verify each schedule explicitly before mainnet.

## Points program (testnet)

In parallel with testnet deployment, we run a **points program** for prover operators, content uploaders, and contributors. Points are non-transferable, non-tradable, and explicitly not a token. Points convert at TGE to a **published fraction** of the **Ecosystem Grants** allocation (10% / 100M PROVA total). The conversion ratio is announced before testnet ends.

The points formula is published before testnet starts. Categories:

- **Prover operator:** points per TB-day proved, with a multiplier for early enrolment and continuous uptime.
- **Client:** points per GB-month stored, capped to discourage wash-uploads.
- **Contributor:** discretionary points awarded by maintainers for merged PRs, with a public bonus tier table.

## Whitepaper amendment

The following amends the v0.9 whitepaper. It will appear inline in the next version (v1.0) of the whitepaper.

> **Amendment 1, 2026-04-26.** Section 4.4 of v0.9 ("Why no token") is replaced with the following:
>
> *Prova does not have a protocol token. Storage deals settle in USDC, provers stake in USDC, and slashing burns USDC. The protocol does not need a token in any of those roles, and a token would only add learning cost and speculation surface to a primitive that benefits from being boring.*
>
> *The org that builds and operates Prova does have an equity-style token, **PROVA**, deployed as a standard ERC-20 on Base. PROVA captures the protocol's 1% fee flow into a public treasury, governs a small set of bounded protocol parameters, and offers an optional fee discount to holders who lock. PROVA is not required to use the protocol as a client or as a prover. The protocol would function identically if PROVA did not exist. The full tokenomics are described in [TOKENOMICS-2026.md](./TOKENOMICS-2026.md).*
>
> *We acknowledge the tension between the v0.9 framing and this amendment. Our position is that the v0.9 arguments were aimed at the protocol's economic surface, and they remain true: the protocol does not need a token. What we did not say in v0.9, but should have, was that the **org needs a way to pay the people who build the protocol**. PROVA is that way. The line between "protocol token" and "org token" is real, and we draw it deliberately.*
>
> *We reserve the right to change our minds again. We do not reserve the right to be quiet about it: any change here would land in a numbered amendment to this document.*
