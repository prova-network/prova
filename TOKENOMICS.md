# Prova Token Economics — ICO Plan

## 1. Token Overview

| Parameter | Value |
|---|---|
| Token name | PROVA |
| Symbol | PROVA |
| Standard | ERC-20 (Ethereum mainnet or Base L2) |
| Total supply | 1,000,000,000 (1B) |
| Decimals | 18 |
| Minting | Fixed supply at genesis. No future minting. |

**Why fixed supply:** The ERC-20 token represents future mainnet PROVA. At mainnet launch, ERC-20 holders swap 1:1 for native PROVA. The mainnet has its own emission schedule (block rewards), but the genesis allocation is fixed and fully transparent from day one. This avoids the "hidden mint" trust problem.

---

## 2. Allocation

| Category | % | Tokens | Purpose |
|---|---|---|---|
| **Network mining** | 45% | 450,000,000 | Block rewards for storage + compute providers post-mainnet |
| **Public sale (ICO)** | 15% | 150,000,000 | Primary fundraise. Community ownership from day one. |
| **Team & founders** | 15% | 150,000,000 | Nicklas + Capri + future core contributors |
| **Ecosystem & grants** | 10% | 100,000,000 | Developer grants, integrations, hackathons, bug bounties |
| **Early backers (seed)** | 7% | 70,000,000 | Pre-ICO strategic investors and advisors |
| **Liquidity** | 5% | 50,000,000 | DEX liquidity pools at launch |
| **Reserve** | 3% | 30,000,000 | Exchange listings, market making, unforeseen needs |

### Why this split works

**45% mining (down from 60% in SPEC-010):** We moved 15% from mining to public sale + liquidity. Mining rewards still dominate long-term supply, but the project needs capital to reach mainnet. A protocol with beautiful code but no runway is a dead protocol.

**15% public sale:** Large enough to raise meaningful capital ($1.5-3M at target pricing) and create broad token distribution. Small enough to not dump the market. Comparable projects (Bittensor, io.net, Render at their early stages) allocated 10-20% to public rounds.

**15% team:** Standard for a two-person founding team with 63K lines of working code. This is earned, not speculative. The vesting schedule ensures long-term alignment.

**7% seed:** Small. We're not giving away the project to VCs. This is for 3-5 strategic backers who bring network effects (GPU operators, AI labs, exchange connections), not just money.

---

## 3. Vesting Schedules

| Category | TGE unlock | Cliff | Vest period | Schedule |
|---|---|---|---|---|
| Public sale | 25% | None | 6 months | Linear monthly |
| Team & founders | 0% | 12 months | 36 months after cliff | Linear monthly |
| Ecosystem & grants | 10% | None | 48 months | Quarterly, DAO-governed |
| Seed | 0% | 6 months | 18 months after cliff | Linear monthly |
| Liquidity | 100% | None | None | Locked in DEX pools (LP tokens locked 12 months) |
| Reserve | 0% | 6 months | 24 months | Multisig-governed |
| Mining | N/A | N/A | N/A | Emitted via block rewards post-mainnet |

### Why these vesting terms

**Team gets nothing for 12 months.** This is the strongest signal we can send. We don't get paid until the network is live and working. This alone differentiates us from 90% of token projects.

**Public sale: 25% at TGE.** Buyers need some immediate liquidity or they won't participate. But 75% vests over 6 months, which prevents immediate dump-and-run. This is generous compared to many ICOs that do 10-20% TGE.

**Seed: 6-month cliff, then 18 months.** Longer than public because seed gets a better price. They need to be patient.

---

## 4. Pricing Strategy

### Seed round

| Parameter | Value |
|---|---|
| Price per PROVA | $0.008 |
| Allocation | 70,000,000 PROVA |
| Raise target | $560,000 |
| FDV at seed price | $8,000,000 |
| Minimum commitment | $10,000 |
| Maximum participants | 5-10 |

**Why $8M FDV:** Prova has 63K lines of production Rust, 1,690 tests, 22 specs, a whitepaper, and experimental validation. This is more built than most projects at $50-100M FDV. An $8M FDV for seed is conservative and gives early backers real upside.

### Public sale (ICO)

| Parameter | Value |
|---|---|
| Price per PROVA | $0.015 |
| Allocation | 150,000,000 PROVA |
| Raise target | $2,250,000 |
| FDV at ICO price | $15,000,000 |
| Minimum purchase | $50 |
| Maximum per wallet | $25,000 |

**Why $15M FDV:** This is the "proof premium" over seed. Public buyers get a working protocol, a whitepaper, demonstrated GPU experiments, and a clear path to testnet. For context:
- Bittensor launched at ~$10M FDV (2021, less built)
- Render launched at ~$20M FDV (2017, no mainnet)
- io.net raised at $1B FDV (2024, less code, more hype)

$15M for Prova's tech is honest pricing. Not cheap enough to look like a scam, not expensive enough to limit upside.

### Listing price target

| Parameter | Value |
|---|---|
| Target DEX listing | $0.02-0.03 |
| Initial market cap | ~$2-4M (circulating at TGE) |
| Liquidity depth | $500K-1M (from liquidity allocation) |

---

## 5. Use of Funds

| Category | % of raise | Amount (at full raise) |
|---|---|---|
| Engineering | 40% | ~$1,120,000 |
| Security audits | 15% | ~$420,000 |
| Infrastructure (testnet, nodes) | 10% | ~$280,000 |
| Legal & compliance | 10% | ~$280,000 |
| Marketing & community | 15% | ~$420,000 |
| Operations & reserve | 10% | ~$280,000 |

**Total raise (seed + public): ~$2,810,000**

This funds 18-24 months of development through mainnet launch (Q1 2027 per roadmap).

---

## 6. Token Utility

PROVA is not a governance-only token. It has direct protocol utility:

1. **Staking:** Providers stake PROVA to offer storage and compute services (min 10,000 PROVA)
2. **Payment:** Clients pay providers in PROVA for storage and inference services
3. **Challenges:** Challengers bond PROVA to initiate dispute resolution (1,000 PROVA)
4. **Governance:** PROVA holders vote on protocol parameters
5. **Model registration:** Registering AI models costs PROVA (100 PROVA, burnt)
6. **Fee burns:** 50% of transaction fees + registration fees are burnt (deflationary post-halving)

---

## 7. Circulating Supply at TGE

| Source | Tokens at TGE |
|---|---|
| Public sale (25% unlock) | 37,500,000 |
| Liquidity pools | 50,000,000 |
| **Total circulating** | **87,500,000** |
| % of total supply | 8.75% |

Market cap at listing ($0.02): **$1,750,000**
Market cap at listing ($0.03): **$2,625,000**

This is a healthy float. Not so much that there's sell pressure. Not so little that it's illiquid. The team, seed, ecosystem, and reserve are all locked at TGE.

---

## 8. Deflationary Mechanics (Post-Mainnet)

Once mainnet launches:
- 50% of all transaction fees burned
- 100% of model registration fees burned (100 PROVA each)
- 0.5% of payment channel settlements burned
- Failed challenge bonds burned (1,000 PROVA each)

At sufficient network utilization, PROVA becomes net-deflationary after the first few halving cycles.

---

## 9. ICO Mechanics

### Smart contract approach

**ERC-20 token + vesting contracts on Ethereum (or Base for lower gas):**

1. **ProvaToken.sol** — Standard ERC-20, fixed 1B supply minted to deployer
2. **ProvaVesting.sol** — Token vesting with cliff + linear release, revocable for team (in case of departure)
3. **ProvaCrowdsale.sol** — Whitelist-based sale contract with per-wallet caps, USDC/ETH accepted, automatic vesting enrollment
4. **LiquidityLock.sol** — LP token lock for 12 months (use existing Unicrypt or Team Finance for trust)

### Sale flow

1. Website goes live with whitepaper, allocation chart, roadmap
2. Seed round opens (private, 5-10 participants, $10K+ each)
3. Seed closes after $560K raised or 2 weeks
4. Public ICO opens (whitelist or FCFS with per-wallet cap)
5. Public closes after $2.25M raised or 2 weeks
6. TGE: token deployed, 25% public unlock, liquidity pools created
7. Vesting begins for all locked allocations

### KYC/Compliance

For legal safety (especially from Norway):
- Seed: KYC required (passport/ID verification)
- Public: Terms of service with geo-blocking for US/sanctioned jurisdictions
- Token explicitly labeled as "utility token" with protocol function
- No promises of returns, profit sharing, or dividends

---

## 10. Comparable Projects

| Project | Token | FDV at launch | Raise | Tech at launch |
|---|---|---|---|---|
| Bittensor | TAO | ~$10M | Small community | Concept + subnet |
| Render | RNDR | ~$20M | $5M | Prototype |
| Akash | AKT | ~$15M | $2M | Working testnet |
| io.net | IO | ~$1B | $30M (private) | Dashboard + GPU cluster |
| **Prova** | PROVA | **$15M** | **$2.8M** | **63K lines, 1,690 tests, whitepaper, GPU experiments** |

Prova has more working code than any of these had at their fundraise stage. The FDV is honest.

---

## 11. Timeline

| Date | Milestone |
|---|---|
| April 2026 | Website + whitepaper live, X/Twitter active |
| April 2026 | Seed round (2 weeks) |
| May 2026 | Public ICO (2 weeks) |
| May 2026 | TGE + DEX listing |
| Q3 2026 | Public testnet |
| Q4 2026 | Security audit |
| Q1 2027 | Mainnet + 1:1 swap |
