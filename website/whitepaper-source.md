# Prova: Verifiable Storage Anchored to Ethereum

**v1.0, April 2026**

Authors: Prova contributors. Comments to <hello@prova.network>.

---

## Abstract

We describe Prova, a thin storage primitive that puts the smallest useful unit of "I have your bytes, I can prove it" on a general-purpose smart-contract chain. A client commits a piece of content by its piece-CID. A prover stakes PROVA and accepts the deal. Every challenge interval the prover answers a verifier-issued challenge with a Merkle inclusion proof against the committed root. Failure to answer within the deadline burns a fraction of the prover's PROVA stake. The client retrieves over plain HTTPS, verifying the bytes against the same piece-CID. Settlement of client-prover payments is in USDC on Base.

The design reuses well-studied building blocks. PDP, CommP/Fr32, and the deal-marketplace pattern are mature primitives from the Filecoin ecosystem, available under permissive licenses. Prova ports them, narrows their scope, and pays them out where most stablecoin economic activity already lives.

---

## 1. Motivation

Most of the storage we rely on is unverifiable in any meaningful sense. The provider claims to hold the bytes; the client either accepts that claim or stops using the provider. For ephemeral content, this is fine. For content that needs to outlive a single vendor — research datasets, public archives, regulatory records, model weights, training corpora, evidence — it is not.

Decentralized storage projects have addressed pieces of this gap. Most charge two coordination costs at once: a new settlement layer with its own native gas, and a maximalist bundle of features (sealing, replication economics, provenance, retrieval markets, governance). The result is high learning cost for clients who only want the verifiable-retention primitive, and high integration cost for builders whose users already settle on Ethereum.

Prova narrows scope aggressively. It does one thing — verifiable retention with on-chain slashing — and uses the rails the rest of the world already uses for stablecoin settlement and stake-bonded service economics.

We make four claims:

1. **Provable Data Possession (PDP) is sufficient** for the verifiable-retention guarantee. SNARK-based replication proofs are not required for the use cases above, and the cost gap is large.
2. **Base is the right L2.** USDC issuance is canonical, blockspace is cheap enough for the proof rate this design needs, ENS is integrated, and Ethereum security is inherited.
3. **Stablecoin payments + token stake** is the right economic split. Clients pay in USDC and provers earn USDC; provers stake the protocol's PROVA token and are slashed in PROVA.
4. **A small protocol surface is auditable.** The on-chain logic is seven contracts; the off-chain prover is a single Go binary. The mental model fits in a page.

---

## 2. Primitives

We avoid inventing cryptography. Everything below is either standard or a port of well-reviewed prior art.

### 2.1 Piece-CIDs (CommP)

Each object is identified by its piece-CID, a binary Merkle root computed over the Fr32-padded bytes of the object using SHA-256. Inner-node hashing zeroes the top two bits of the parent before re-hashing, keeping every digest inside the BLS12-381 scalar field. The root is encoded as CIDv1 with the `fil-commitment-unsealed` codec and `sha2-256-trunc254-padded` multihash. The leading multibase byte is `b`; the canonical printable form for fil-commitment-unsealed begins with `baga…`.

Two important properties:

- **Self-describing.** The CID contains the codec and hash algorithm. Verifiers do not need out-of-band agreement.
- **Recomputable.** Anyone holding the bytes can recompute the piece-CID and check that it matches the on-chain commitment.

Piece-CIDs are chain-agnostic. Ethereum contracts can store them as `bytes32` (the digest) or as a length-prefixed `bytes` blob (the full CID). ENS already uses CIDs for content addressing.

### 2.2 Provable Data Possession

The prover holds the Fr32-padded bytes and the full Merkle tree. A verifier issues a challenge — an index `i` into the leaf array, derived from the on-chain block hash. The prover responds with the leaf at index `i` and the `O(log N)` sibling hashes along the inclusion path. The verifier recomputes the root and compares it against the committed piece-CID.

The probability that a prover who has discarded a fraction `δ` of the bytes can answer a uniformly-random challenge is `1 − δ`. The challenge interval is set so that the cost of repeated cheating exceeds the cost of honest storage by orders of magnitude.

We rely on this single primitive. We do not separately commit to a unique encoding of the data per replica; we do not require that the bytes be decodable only by the original prover; we do not require uniqueness or non-malleability beyond what content-addressing already provides. A prover who replicates someone else's piece-CID still does not earn that prover's deal: the deal escrow is bound to a specific prover address.

### 2.3 Deal lifecycle

A deal is a small on-chain record:

```
struct Deal {
    address client;
    address prover;
    bytes   pieceCid;
    uint64  pieceSize;        // bytes
    uint64  termStart;        // unix
    uint64  termEnd;          // unix
    uint128 escrowUSDC;       // total client deposit
    uint128 perDayUSDC;       // streaming rate
    uint128 stakeLockedPROVA; // prover's locked stake
    DealState state;
}
```

State transitions:

```
Proposed → Active → (Settled | Slashed)
              ↓
         (Withdrawn)
```

`Proposed` is created by the client with the full USDC escrow. The prover transitions to `Active` by accepting and posting the first proof. `Settled` is reached when `termEnd` passes and all required proofs were submitted on time. `Slashed` is reached when the verifier fails to receive a valid proof for `slashThreshold` consecutive intervals; a fraction of the prover's PROVA stake is destroyed and the remaining USDC escrow is refunded to the client.

### 2.4 Settlement

Client–prover payments settle in **USDC** on Base. Provers post slashable bonds in **PROVA**. The marketplace contract streams the per-day USDC fee to the prover at each successful proof, takes a 1% protocol fee, and refunds unspent escrow at term end. The protocol fee routes to a permissionless **FeeRouter** contract which swaps USDC → PROVA on a Uniswap V3 PROVA-USDC pool and burns the PROVA. Network revenue therefore translates directly into a deflationary force on PROVA supply.

---

## 3. Architecture

### 3.1 Contracts

Seven contracts on Base:

| Contract | Responsibility |
| --- | --- |
| `ProvaToken` | Fixed-supply ERC-20 (100M PROVA, 18 decimals, Permit + Burnable). |
| `StorageMarketplace` | Deal lifecycle. Holds USDC escrow. Streams USDC payment. Triggers slashing of PROVA stake. |
| `ProofVerifier` | Verifies Merkle inclusion proofs against committed roots. UUPS-upgradeable for cryptographic patches. |
| `ProverRegistry` | Prover identity, capacity advertisement, region. |
| `ProverStaking` | PROVA stake locking, unbonding, slashing math. |
| `ContentRegistry` | Optional client-side metadata layer (filename, content-type, hints, ENS contenthash binding). Pure data, no economic significance. |
| `FeeRouter` | Receives the marketplace's USDC protocol fees. Swaps USDC to PROVA on Uniswap V3 and burns the PROVA. Three modes: HOLD (default before TGE), BURN (full conversion), SPLIT (configurable share burned, rest held for treasury). |

`ProofVerifier` is upgradeable; the others are not. This concentrates upgrade risk in the smallest auditable surface.

### 3.2 Prover

The prover is a single Go binary, `provad`. It:

1. Watches Base for `DealProposed` events targeting its address.
2. Fetches the bytes from the deal's `sourceURL` (HTTPS, HTTP-range supported).
3. Recomputes the piece-CID. **If it disagrees with the committed CID, the prover refuses the deal.** This protects honest provers from clients who would commit to bytes they do not actually intend to upload.
4. Stores the Fr32-padded bytes in a content-addressed local store.
5. Calls `ProofVerifier.createDataSet` to accept the deal and post the first proof.
6. Every challenge interval, reads the on-chain challenge seed, computes the challenged leaf and inclusion path, and posts the proof.
7. Serves retrievals over HTTPS at `/piece/{cid}`.

The prover is disk-bound, not CPU-bound. There is no GPU path, no SNARK proving, no sealing. A 10 TB single-node prover on commodity SSDs is sufficient.

### 3.3 Client

There are three client surfaces:

- **Browser.** A drag-and-drop page at `prova.network/upload/`. The first 100 MB is sponsored by the protocol treasury so a new user can verify the system end-to-end without setting up payment. Authentication is by emailed magic-link.
- **CLI.** `@prova-network/cli` — a Node binary with `auth`, `put`, `get`, `ls`, and `whoami` commands. Pure ESM, zero dependencies, talks to the public HTTP API.
- **SDK.** `@prova-network/sdk` — a TypeScript SDK with a high-level `Prova` client and low-level `core` primitives. Forked from `FilOzone/synapse-sdk` under the Permissive License Stack and adapted for Base + USDC + PROVA.

All three surfaces ultimately call the same on-chain contracts and the same prover HTTP endpoints. The client surfaces are conveniences; nothing about the protocol requires them.

---

## 4. Token economics

### 4.1 PROVA token

PROVA is a standard ERC-20 deployed on Base. Fixed total supply: **100,000,000 PROVA** (100M). 18 decimals. No mint authority post-genesis. Burnable via the standard `ERC20Burnable` extension; the protocol's burn happens via the FeeRouter.

PROVA plays four roles in the protocol:

1. **Prover stake.** Provers post slashable PROVA bonds. Capacity is gated by stake: `minStakePerGiB × committedGiB`. Slashing destroys a fixed fraction of the offending prover's PROVA, removing it from supply.
2. **Prover emission.** Half of the total supply (50M PROVA) is paid out to provers over 8 years on a declining curve, proportional to bytes-proven-time, with anti-gaming protections built into [`ProverRewards`](https://github.com/prova-network/contracts/blob/main/src/ProverRewards.sol). The supply-side gets the largest share of the network's tokens, paid out as they prove they are storing.
3. **Fee burn.** The marketplace's 1% USDC fee routes to the FeeRouter, which swaps USDC → PROVA on Uniswap V3 and burns the PROVA. Network revenue → permanent supply reduction.
4. **Governance.** PROVA-weighted vote on a bounded set of protocol parameters (fee tier, slash fraction, minimum stake multiplier, prover-registry rules, upgrade authority on `ProofVerifier`), with a 2-day timelock on parameter changes and 7-day on contract upgrades.

PROVA is **not** required to be a client. Clients pay in USDC. Provers earn in USDC. The day-to-day storage UX never has to touch PROVA. PROVA is the alignment instrument between honest provers, the protocol's economic flow, and PROVA holders.

### 4.2 Volatility mitigation for prover stake

Minimum-stake requirements include a USDC-equivalent floor read from a Chainlink PROVA/USD oracle:

```
minStake(GiB) = max(absoluteFloorPROVA, oracleEquivalent(targetUSD, GiB))
```

If PROVA price drops sharply, provers have a 7-day grace window to top up stake before they are paused from accepting new deals. Already-active deals continue to honor their original stake commitment until completion. This protects honest provers from a flash crash and keeps the slashable economic value above a published floor in USDC terms.

### 4.3 Allocation

The supply splits across three layers: **genesis distribution** (45%), **prover emission over 8 years** (50%), and **ecosystem + community** (5%).

<div class="allocation-pie" data-allocation>
  <!-- Rendered as an inline SVG pie by whitepaper.html. The data table below is the source of truth. -->
</div>

| Layer / Bucket | Share | Tokens (PROVA) | Vesting |
| --- | ---: | ---: | --- |
| **GENESIS DISTRIBUTION** | **45%** | **45,000,000** | (mostly vested) |
| Public sale (TGE / LBP) | 6% | 6,000,000 | Unlocked at TGE |
| Private SAFT round | 12% | 12,000,000 | 12-month cliff, 24-month linear thereafter |
| Team and core engineers | 14% | 14,000,000 | 12-month cliff, 36-month linear |
| Advisors / BD / sales / design | 4% | 4,000,000 | 12-month cliff, 36-month linear |
| Treasury / community | 6% | 6,000,000 | 5-year linear release to multisig |
| Liquidity (DEX seeding) | 3% | 3,000,000 | LP tokens locked 24 months |
| **PROVER EMISSION (8-year curve)** | **50%** | **50,000,000** | Distributed by `ProverRewards` |
| Year 1 | 12.5% | 12,500,000 | weekly per-epoch |
| Year 2 | 11.0% | 11,000,000 | weekly |
| Year 3 | 9.0% | 9,000,000 | weekly |
| Year 4 | 7.0% | 7,000,000 | weekly |
| Year 5 | 5.0% | 5,000,000 | weekly |
| Year 6 | 3.0% | 3,000,000 | weekly |
| Year 7 | 1.5% | 1,500,000 | weekly |
| Year 8 | 1.0% | 1,000,000 | weekly |
| **ECOSYSTEM + COMMUNITY** | **5%** | **5,000,000** | (multi-year) |
| Ecosystem grants | 3% | 3,000,000 | Released as merit-based grants by the treasury multisig |
| Community / referral program | 2% | 2,000,000 | Released for client-acquisition referrals and early-tester rewards |

Insider allocation (SAFT + team + advisors) is **30%** — below the 35% line that triggers CEX listing concerns. Supply-side allocation (provers) is **50%** — half of the network's tokens go to the people who actually run the storage, paid out as they prove they are storing it.

Genesis schedules are enforced on-chain by `ProvaVesting`. Prover emission is paid by `ProverRewards`. The off-chain memoranda (vesting agreements, SAFT contracts) memorialise the legal grant; the on-chain schedules are the source of truth for what vests when.

#### Anti-gaming rules baked into prover emission

The 50M emission bucket only flows to provers who actually prove bytes for real clients. The `ProverRewards` contract enforces:

- **No self-dealing.** A prover that's also the client of the same deal earns no emission for that deal (`prover != client` check).
- **No sponsored-tier farming.** Free-tier sponsored uploads (no client wallet) don't generate emission.
- **Redundancy cap.** A given piece-cid earns emission for at most N (default 4) provers per epoch. Beyond that, additional copies don't earn additional emission.
- **Per-epoch single-counting.** A prover earns at most once per `(piece, epoch)` regardless of how many times they re-post a proof.
- **Vesting buffer.** Emission for epoch E is only claimable 30 days after E ends, so a prover who signs up, takes a few deals, and disappears earns nothing.
- **Quality multiplier.** A prover with > 5% missed-proof rate in trailing 30 days has emission halved. Slashed in last 90 days → zero emission.
- **Identity attestation tiers.** Hobby provers (≤ 100 TB) register pseudonymously. Prosumer (100 TB – 5 PB) requires lightweight ENS / EAS attestation. Enterprise (> 5 PB) requires full KYB + master agreement.

The combined effect: **the only way to earn more PROVA is to honestly store more bytes for longer**. Sybil and wash-trade strategies against this design are economically dominated by simply running an honest prover.

### 4.4 Pricing and protocol fee

Pricing is set by the prover and discoverable on-chain. The marketplace does not enforce a price floor. Initial provers will quote in the range of $1.50–$3.00 per TB-month — competitive with hot object storage, more expensive than the cheapest cold archive tier, but verifiable and slashable in a way neither competitor is.

The protocol fee is 1% of the USDC payment stream. Routed to FeeRouter → Uniswap V3 → burned PROVA. Hard-capped at 3% by contract; further changes require governance vote and 2-day timelock.

---

## 5. Security model

### 5.1 What we promise

- **Verifiable retention.** Anyone, not just the original client, can verify that a prover is currently storing the bytes that hash to a given piece-CID, by checking the on-chain proof history.
- **Bounded loss.** A successful slashing destroys a fixed fraction of the prover's PROVA stake and refunds the unspent USDC escrow. A client's worst case is a refund plus the cost of re-uploading.
- **Liveness via redundancy.** The protocol does not promise that a single prover will not go down. It promises that a deal stored across N independent provers (the redundancy parameter, set per-deal) survives any subset failure.

### 5.2 What we do not promise

- **Confidentiality.** The bytes are stored as committed. Encrypt before upload if the bytes should not be readable by the prover. The piece-CID is over the ciphertext in that case.
- **Permanence.** Storage is term-bound. A deal expires at `termEnd` and the prover is free to delete the bytes. Long-term retention is implemented by chaining renewals or by depositing a long-term escrow.
- **Censorship resistance at the network layer.** Provers run on commodity hosting. Adversarial nation-state pressure on a single prover can take it offline. The redundancy parameter mitigates this, but does not eliminate it.

### 5.3 Trust assumptions

Four external assumptions:

1. **Ethereum + Base liveness.** Settlement and challenge issuance depend on the chain producing blocks. A halt of either chain pauses the protocol gracefully — proofs do not need to be submitted faster than the chain produces blocks — but a multi-day halt would require manual intervention to reset deal timing.
2. **USDC peg.** Price and client payments denominated in USDC inherit Circle's peg risk. A peg break would be a global financial event; the protocol's behavior in that scenario is the same as any other USDC-denominated contract.
3. **PROVA-USDC liquidity.** The fee burn assumes a tradeable PROVA-USDC pool on Uniswap V3 on Base. The FeeRouter has slippage guards and a permissionless trigger; if liquidity is too thin to swap profitably, fees accumulate and burn later.
4. **SHA-256 + BLS12-381.** Standard cryptographic assumptions. A break of either is a global event.

### 5.4 Audits

Internal review of the on-chain contracts and the off-chain stack is captured in `SECURITY-AUDIT-2026-04-25.md`. External audit by an independent firm is a precondition to mainnet. We will publish the report in full.

---

## 6. Differentiation

We compare against three reference systems.

### 6.1 vs. Filecoin

Filecoin is the closest relative and the source of the cryptographic primitives we use. Differences:

- **No sealing or PoRep.** We use PDP only. This trades off the "unique encoding per replica" guarantee for a much lower hardware bar.
- **Stablecoin client payments.** Settlement is USDC, not the protocol's native token. A volatile token isn't forced on every client.
- **No new chain.** Settlement is on Base, which inherits Ethereum security and EVM tooling.
- **No retrieval markets.** Retrieval is plain HTTPS from the prover; the prover serves it as part of the deal obligation.

The trade is: simpler economics and integration, in exchange for giving up the "physically unique replicas" property that PoRep provides. For the use cases in §1, the trade is correct; for use cases that genuinely need PoRep (e.g., adversarial replication-counting), Filecoin is the right tool.

### 6.2 vs. Arweave

Arweave optimizes for permanent storage with a one-time fee. Differences:

- **Term-bound vs. perpetual.** Prova deals expire and renew. This makes deletion and migration first-class operations.
- **Ongoing proofs vs. attested permanence.** Prova publishes a fresh proof on every challenge interval. Arweave's proof model is different and not directly comparable; we offer no opinion on its strength.
- **USDC + PROVA vs. AR.** Stable client payments, stake-aligned governance.

### 6.3 vs. centralized object storage

S3 / R2 / GCS optimize for hot performance and a vendor-managed durability story. Differences:

- **Verifiable.** Prova lets anyone, not just the vendor, verify retention.
- **Multi-prover.** A single Prova deal is replicated across N independent provers by construction.
- **Slashable.** Loss has a defined economic consequence on-chain.
- **Slower.** Prova is not a low-latency hot path. It targets archival, anchoring, and reproducibility, not page-load.

---

## 7. Roadmap

The following is best-effort planning and should not be read as commitment.

| Phase | Target | Status |
| --- | --- | --- |
| Spec freeze | piece-CID, deal struct, proof format, token model | this document |
| Public testnet | Base Sepolia, free tier, 3+ independent provers | Q2 2026 target |
| External audit | one tier-1 firm, six-week scope | post-testnet |
| Mainnet | Base, audited, term limits raised | H2 2026 |
| Token Generation Event | PROVA on Base mainnet, public LBP | with mainnet |
| SDK 1.0 | feature freeze on `@prova-network/sdk` | with mainnet |
| ENS contenthash integration | drag-and-drop static-site anchoring | with mainnet |

---

## 8. Open questions

We are deliberately listing the things we do not yet know, because the cost of being wrong here is real.

1. **Challenge cadence.** A 30-second cadence is comfortable on Base today. If chain congestion makes this expensive at scale, we may move to 5-minute or block-aligned cadence.
2. **Proof aggregation.** A future version may batch multiple deals' proofs into a single transaction with a small ZK aggregator. We have intentionally not designed this in yet.
3. **Cross-chain settlement.** USDC exists on every major L2. We have considered allowing deals to settle on Optimism, Arbitrum, or Polygon as well as Base, but we are wary of the complexity that bridges introduce. The current answer is "Base only at launch, revisit with data."
4. **Prover incentive uniformity.** Without PoRep, two provers can in principle collude to share a single physical copy and double-claim the redundancy. We assume legal-economic constraints (independent operators, separate stake) make this unprofitable at any meaningful scale, but we will measure rather than assume.
5. **Staking-floor oracle dependency.** The Chainlink PROVA/USD floor depends on a deep PROVA-USDC pool. We will publish quarterly liquidity metrics and revisit the staking floor formula if pool depth is insufficient to support the price feed.

---

## 9. Conclusion

The point is not to do something dramatic. It is to do one useful thing well, on the rails most people are already on, with an economic model anyone can sit through in five minutes. If the design fails it should fail for boring reasons we can fix; if it works it should work because everything inside it has worked somewhere else first.

Comments and corrections to <hello@prova.network>.

---

### Appendix A: References

- Ateniese, Burns, Curtmola, Herring, Kissner, Peterson, Song. *Provable Data Possession at Untrusted Stores.* CCS, 2007.
- Filecoin Project. *PDP specification.* `github.com/FilOzone/pdp`.
- Filecoin Project. *Synapse SDK.* `github.com/FilOzone/synapse-sdk`.
- Ben-Sasson, Bentov, Horesh, Riabzev. *Scalable Zero-Knowledge with no Trusted Setup.* CRYPTO, 2019. (Cited only for context: Prova does not use STARKs.)
- Buterin et al. *EIP-4844: Shard Blob Transactions.* (Cited for context: Prova does not use blobs as the primary storage path.)

### Appendix B: Notation

| Symbol | Meaning |
| --- | --- |
| `N` | Number of leaves in the piece's Merkle tree |
| `δ` | Fraction of bytes a dishonest prover has discarded |
| `T` | Term length in seconds |
| `Δ` | Challenge interval in seconds (default 30) |
| `s` | Stake locked, in PROVA |
| `f` | Slash fraction, in [0, 1] |
| `r` | Redundancy, integer ≥ 1 |
