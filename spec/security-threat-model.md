# 6.1 Threat model

The set of attacks Prova considers in scope, the mitigations, and the residual risks we knowingly accept.

## 6.1.1 Adversary model

We assume an adversary who can:

- Deploy and operate one or more provers under arbitrary identities (Sybil-capable up to a budget)
- Buy and sell PROVA freely on a public DEX
- Run the full open-source CLI / SDK / `provad` against the public API and contracts
- Submit valid Base transactions
- Observe all on-chain state and all public network traffic

We do NOT assume the adversary has:

- Access to a private key they don't control (i.e., no key extraction from honest provers)
- Control over a majority of Base validators (i.e., the L2 itself is honest)
- The ability to break SHA-256, BLS12-381, or HMAC-SHA-256

## 6.1.2 In-scope attacks

### A1. Prover misbehavior — failure to store

The protocol's central concern. A prover claims to store bytes but doesn't.

**Mitigation**: PDP challenges every 30 seconds. A dishonest prover holding fraction `1 − δ` of the bytes fails challenges with probability `δ`. After `MAX_PROOF_GAP` of consecutive failures, the deal is faulted, the client is refunded from the unspent USDC escrow, and `slashPerFault` PROVA is burned from the prover's stake.

**Residual risk**: a prover with very high stake might absorb a single slashing event and continue. Mitigated by the `slashFraction` cap (governance-tunable, default 10% per event) plus the off-chain reputation effects of public slashing events.

### A2. Self-dealing

A prover stores their own data and earns the protocol's prover-emission for it.

**Mitigation**: `ProverRewards.recordProof` reverts with `SelfDealing` when `prover == client`.

**Residual risk**: a coordinated attacker uses a separate-on-paper EOA to sign as the client. The protocol cannot tell two distinct addresses controlled by the same person apart. Mitigated by the redundancy cap (a single piece earns up to N=4 provers' worth of emission), making it expensive to scale this attack.

### A3. Replication double-claim

Multiple provers all store the same piece and claim the redundancy fee.

**Mitigation**: per-piece per-epoch redundancy cap in `ProverRewards`. Beyond `redundancyCap` (default 4), additional copies don't earn additional emission.

**Residual risk**: at exactly the cap, all copies could be from a single physical replica behind multiple addresses. The economic cost of running independent infrastructure (separate IPs, separate stake, separate identity attestation above 100 TB) is the deterrent. We accept that a sophisticated attacker can collude up to `N` ways for a given piece; this is by design.

### A4. Sybil identity

One operator runs many provers from the same hardware to capture more emission.

**Mitigation**: tier-gated identity attestation. Below 100 TB committed, a prover registers pseudonymously. Above 100 TB, lightweight ENS / EAS attestation is required. Above 5 PB, full KYB. Lower-tier provers are individually limited in capacity, capping the per-Sybil emission share.

**Residual risk**: an attacker willing to register many ENS subdomains and many EAS attestations could climb the ladder. The cost of doing so at scale (per-subdomain registration costs, attestation gas) puts a floor on the attack.

### A5. Wash uploads

A client repeatedly uploads the same piece to inflate metrics or keep emission flowing.

**Mitigation**: per-(piece, prover, epoch) single-counting in `ProverRewards`. Re-uploading the same `pieceCid` to the same prover within an epoch counts only once.

**Residual risk**: a client uploads the same piece across many prover addresses. The redundancy cap (A3) catches this.

### A6. Free-tier exploitation

The sponsored upload path (no auth required, 100 MB free) is used to inflate emission.

**Mitigation**: `recordProof` skips emission accounting when `client == address(0)`. Sponsored deals don't count.

**Residual risk**: low. We accept the sponsored-tier cost as the price of low-friction onboarding.

### A7. Fast-churn

A prover signs up, takes a cohort of deals, fails them, and disappears with the rewards.

**Mitigation**: `ProverRewards` has a 30-day vesting buffer between epoch end and claim eligibility. A prover whose quality drops below the cutoff in the 30-day window has their emission halved or zeroed before they can claim.

**Residual risk**: an attacker willing to commit 30+ days of honest operation before turning malicious can still claim early-window emission. Mitigated by the slashing on the actual misbehavior, which destroys their stake in addition to denying future emission.

### A8. CID poisoning

A client commits a piece-CID that doesn't match the bytes they actually upload.

**Mitigation**: the prover MUST recompute the piece-CID at intake and refuse the deal on mismatch. The stage server (for sponsored uploads) does the same and returns 422 `cid_mismatch`.

**Residual risk**: a buggy prover that skips CID verification accepts the wrong bytes and fails its challenges later. The slashing path catches this; the buggy prover bears the cost, not the protocol.

### A9. Token-secret theft (gateway)

An attacker who steals `PROVA_TOKEN_SECRET` from the gateway can mint arbitrary API tokens.

**Mitigation**: the secret is stored as a Cloudflare Pages secret (encrypted at rest, never exposed to the runtime as plaintext outside the function context). Rotation invalidates all issued tokens at once.

**Residual risk**: a Cloudflare-side breach. We accept this as the platform's risk model.

### A10. Smart-contract bugs

A bug in `StorageMarketplace`, `ProverStaking`, `ProverRewards`, or `FeeRouter` that allows unauthorized fund movement.

**Mitigation**: 81 automated tests covering the happy path and the named error paths. External audit before mainnet (in progress). UUPS upgrade authority is gated by a 7-day timelock and a 5-of-9 multisig pause.

**Residual risk**: unknown bugs. Mitigated by the audit, the bug-bounty (planned post-mainnet), and the multisig pause.

## 6.1.3 Out-of-scope

We do NOT defend against:

- **The L2 going down.** Base liveness is assumed.
- **The USDC peg breaking.** This is a global-financial event; the protocol behaves the same as any USDC-denominated contract.
- **A break of SHA-256 or BLS12-381.** Same.
- **The prover's hosting provider seizing data.** Provers run on commodity infrastructure; nation-state pressure on a single prover takes that prover offline. The deal-redundancy parameter mitigates but does not eliminate.
- **Adversarial RPC providers.** Clients SHOULD verify the data they retrieve against the on-chain CID; the SDK does this by default.

## 6.1.4 Bug bounty

A bug-bounty program will be active from mainnet onward. Tiers and payouts will be published in [`prova-network/prova/SECURITY.md`](https://github.com/prova-network/prova/blob/main/SECURITY.md) (when finalized). Pre-launch, security disclosures should be sent to [security@prova.network](mailto:security@prova.network).

## 6.1.5 Disclosure policy

We follow a 90-day coordinated disclosure window. A reporter who finds a vulnerability sends it to security@prova.network; we acknowledge within 48 hours, fix within 60 days, and disclose publicly at 90 days (or sooner if mutually agreed).
