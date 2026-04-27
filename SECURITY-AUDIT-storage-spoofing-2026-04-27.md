# Storage spoofing review — Prova economic model

**Date:** 2026-04-27
**Reviewer:** Capri (internal)
**Scope:** Can a prover acquire PROVA, stake it, and extract value from the protocol without actually storing client data?

---

## TL;DR

- **No.** Buying + staking PROVA does not earn anything by itself. Money flows from real deals.
- **For each individual deal, fake storage is structurally blocked.** PDP requires producing a Merkle inclusion proof against the deal's specific `commpHash`. Without the data, you cannot construct a valid path. Wrong path → revert → fault → slash.
- **There is, however, a sybil/wash-trade attack on the emission subsidy.** A single operator can spin up sock-puppet client wallets, fund deals with their own prover, store the (also self-generated) data, submit real proofs, and farm `ProverRewards` emission. Self-dealing is detected only at the address level, so different addresses defeat it. This is the same class of issue Filecoin's mainnet replica-encoding (PoRep) was designed to neutralize, and it is currently open in Prova's PDP-only model.

The wash-trade attack does not let the attacker store *less* than they claim — they really do store the data — but it does let them inflate their share of emission with self-supplied "demand." That's still a real economic concern because it lets a well-capitalized attacker capture a disproportionate share of the 50M PROVA emission pool.

---

## Threat model

**Attacker:** A single operator with capital and access to commodity storage. Willing to deploy multiple wallets and pay protocol fees if the subsidy outweighs them.

**Goal:** Extract PROVA emission and/or USDC fees with the smallest possible real cost.

**Out of scope:** Client-side spoofing (clients lying about what they upload), stake-grinding, governance attacks, MEV.

---

## What the protocol actually charges proofs for

To get money out of the marketplace, an actor needs:

1. A registered prover (`ProverRegistry.register`).
2. Sufficient PROVA staked: `ProverStaking.canCommit(prover, pieceSize)` returns true. The minimum is `max(provaFloor, usdFloor)` per `_requiredStake`, where `provaFloor = tibRequired * minStakePerTiB` and `usdFloor` uses the configured price oracle.
3. A real deal proposed by some client wallet via `StorageMarketplace.proposeDeal(prover, commpHash, pieceSize, durationSeconds, totalPayment)`.
4. The prover atomically calls `ProofVerifier.addPieces(0, listener=marketplace, [piece], extraData)` with the actual piece CID matching the deal's `commpHash`. The marketplace's `dataSetCreated` callback flips the deal to `Active` and `commitBytes` is called on the staking contract.
5. Through each proving period, the prover must call `provePossession` with a valid Merkle inclusion proof for the challenged offset. Failure → `faultDeal` → slash.

Every economic flow we ship is gated on at least one of these steps.

---

## Attack vector A — "claim without store"

> Stake PROVA, claim 100 TiB committed, never store anything, collect.

**Verdict: blocked.**

- `commitBytes` only fires when a deal goes Active. Without an actual `proposeDeal` from a client wallet, no commitment ever occurs. Staking by itself is silent.
- Even if the attacker self-proposes a deal and accepts it, every proving period requires a real Merkle path against the deal's specific `commpHash`. The harness in `contracts/test/RealPdpProofHarness.t.sol` exercises this end-to-end. A tampered sibling, a wrong-leaf-for-challenge, or a missing piece all cause `provePossession` to revert. The deal is then slashable via `faultDeal`.

The PDP proof primitive does what it claims: you cannot prove possession of data you do not have.

---

## Attack vector B — "wash-trade emission farming"

> Spin up sock-puppet clients. Self-propose deals. Self-fund USDC. Store the (self-generated) data. Submit real proofs. Farm `ProverRewards` emission.

**Verdict: open.**

### Why this works

1. Attacker controls wallet `P` (prover) and wallets `B1, B2, ..., Bn` (sock-puppet clients).
2. Attacker generates piece data `D1, ..., Dn` and computes their CommP roots `commp_i`.
3. Each `Bi` calls `proposeDeal(P, commp_i, pieceSize, duration, totalPayment_i)`, escrowing USDC.
4. `P` accepts each deal via `addPieces` on `ProofVerifier`. Deal goes Active. `commitBytes` runs.
5. Through each proving period, `P` submits real proofs over the real (self-generated) data. Proofs verify because the data is real. Streaming USDC release flows from each deal to `P`. 1% goes to `FeeRouter` (burn).
6. `ProverRewards.recordProof(P, Bi, commp_i, bytesProven)` is invoked from the marketplace's `possessionProven` listener. Self-dealing check `prover != client` passes because `P != Bi`.
7. Over the deal's lifetime, `P` accrues PROVA emission proportional to the bytes proven.

### Net economics

- **Cost (per deal):**
  - 1% USDC fee → burned via `FeeRouter`. Real loss to the attacker.
  - Gas: `proposeDeal`, `addPieces`, `provePossession` (per epoch), eventual completion. Real loss.
  - PROVA stake locked while bytes are committed. Opportunity cost, but recoverable.
  - Storage hardware + power for the duration. Attacker provides this.
- **Gain (per deal):**
  - `totalPayment - 1% fee` USDC. The "rest" of the USDC flows from `Bi` through the marketplace back to `P`. Net to attacker = +0 minus the 1% fee.
  - PROVA emission credited via `ProverRewards`. **This is the subsidy that makes the attack profitable.**

If `emission_value > usdc_fee + gas + amortized_hardware`, the attack pays. The 50M PROVA emission pool over 8 years works out to ~17K PROVA/day, divided by total `bytesProven` weighted by `qualityMultiplier`. A well-capitalized attacker who captures a meaningful share of `bytesProven` extracts a corresponding share of the daily emission.

### Why existing defenses don't stop B

| Defense | Why it doesn't apply here |
|---|---|
| `selfDealing` check (`prover != client`) | Sock-puppet wallets are different addresses → check passes. |
| Redundancy cap (one piece can't earn for >N copies) | Attacker uses N **different** pieces. Cap doesn't bind. |
| Quality multiplier (recent miss rate) | Attacker submits valid proofs over real data → no misses → multiplier stays at full. |
| Slashing for fault | Attacker doesn't fault. They store the data; they prove it. |
| Stake floor | Stake is locked but recoverable; not a real cost beyond yield-equivalent opportunity cost. |

### Why this matters

This is the same family of attack that Filecoin mainnet spends a lot of complexity defending against (PoRep + PoSt + pledge + GFIL economics). PDP alone cannot tell the difference between "stored client data the attacker also happens to own" and "stored data from an unrelated client." Without replica encoding tying the data to the prover's identity, sybil-multiplied "demand" looks identical to organic demand from the protocol's perspective.

---

## Recommended mitigations

Listed in increasing order of complexity and effectiveness.

### M1 — Emission gate on burned USDC (near-term, low complexity)

Today, 1% of every USDC payment is burned by `FeeRouter`. Make `ProverRewards.recordProof` only credit emission against deals that have actually had USDC burned beyond a minimum threshold per byte. That converts wash-trades into a strict loss: even after sock-puppets, the attacker is donating burn-fee USDC to nobody.

Spec change: `ProverRewards.recordProof` accepts an attestation from `FeeRouter` of `usdcBurned >= minBurnPerByte * bytesProven` for the deal in question. If not met, no emission credit.

This is the cleanest single-contract change and would be my recommendation as the first mitigation to ship.

### M2 — Per-prover emission cap (near-term, low complexity)

Hard ceiling on PROVA emission any single prover can claim per epoch. Limits the maximum extractable value per attacker without forbidding the attack. Trades off some legitimate large-prover earnings for sybil resistance.

Spec change: `ProverRewards` adds `maxEmissionPerProverPerEpoch`, governance-tunable.

### M3 — Verified-client requirement (medium complexity, centralizing)

Maintain an allowlist (or attestation registry) of verified non-sybil client wallets. Only deals where `client ∈ verifiedSet` earn emission. Forces attackers to either obtain attestations (raising the bar to a real-world cost) or skip the subsidy.

Trade-off: centralizes the protocol's emission policy into whoever maintains the verified set. Prova's "thin verifiable-storage primitive" framing prefers to avoid this kind of permissioned layer if a non-permissioned mitigation is sufficient.

### M4 — External-entropy commitment (high complexity)

Force `commpHash` to be derived from external entropy that the prover cannot precompute (e.g., the prover commits to a piece selected from a Merkle tree of recent block hashes). Self-supplied data becomes impossible because the attacker cannot generate piece bytes whose root matches arbitrary external entropy.

This is essentially Filecoin's PoRep insight ported to PDP. Significant cryptographic and economic redesign.

### M5 — Replica encoding (high complexity, full Filecoin-style)

Tie the on-disk encoding of each piece to the prover's identity (replica ID). The attacker's "I'll store this myself anyway" stops working because storing 2 copies for 2 wallets requires encoding 2 replicas — and encoding cost is the dominant compute. This is the heart of Filecoin's anti-sybil model.

Out of scope for current Prova v2. Would require a meaningful rework of the proof system.

---

## Recommendation

For tonight: **document the attack openly in the spec** so external readers understand what PDP-only protects against and what it doesn't. The right place is `spec-site/security-threat-model.md`, with a cross-link from `spec-site/governance.md`.

For Q3 (pre-mainnet): **ship M1 (emission gate on burned USDC) and M2 (per-prover emission cap)**. Together they bound the maximum extractable value and convert wash-trades into a guaranteed loss for any individual deal.

For long-term (post-TGE): **evaluate M3 vs M4 vs M5** based on observed mainnet behavior. M3 is fastest to ship if the community is willing to accept a verified-client list. M4/M5 are real protocol research and shouldn't be rushed.

---

## What this report does not change today

- 110/110 contract tests still pass. The vulnerability is in the **economic design**, not in the implementation of any single contract.
- The PDP proof primitive itself is sound for the question "does the prover have the data?". The wash-trade attack is "different question, also matters."
- This is not a launch blocker. Filecoin mainnet has the same class of issue mitigated by PoRep + economics, and Prova's smaller scale + lower emission rate gives us runway to ship M1+M2 before sybil incentive grows.
