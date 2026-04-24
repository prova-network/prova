# SPEC: Security Threat Model — Prova v2

**Status:** Draft v1 (post-pivot)
**Updated:** 2026-04-24

## 1. Scope

Prova v2 is a set of Solidity contracts on Base + off-chain prover nodes
that store pieces and answer PDP challenges. This document enumerates
attacks against that specific system. Base itself (validator set, L1
finality, bridge security, gas pricing) is out of scope — those are
inherited from Ethereum/Base and their threat model is well covered by
the L1/L2 community.

### Threat levels

| Level | Impact | Response |
|-------|--------|----------|
| **Critical** | Fund loss, data loss, contract bricking | Emergency upgrade via UUPS + timelock bypass |
| **High** | Systemic griefing, temporary denial | Priority fix, timelocked upgrade |
| **Medium** | Economic inefficiency, annoyance | Scheduled fix |
| **Low** | Info leak, UX issue | Best-effort |

## 2. Contract Layer

### T-01. Marketplace fund drain (Critical)

- **Vector:** A bug in `StorageMarketplace` lets a prover claim payment without completing the deal, or lets a client reclaim escrow after it's already been released.
- **Mitigation:** Reentrancy guards on all state-changing external calls. Linear streaming release bounded by elapsed time. Deal state machine transitions gated by `onlyProofVerifier` where appropriate. 40+ contract tests covering the happy path and each error branch.
- **Residual:** Audit before mainnet.

### T-02. Slashing bypass (High)

- **Vector:** Prover finds a way to skip challenges without getting slashed — e.g., front-runs `faultDeal` with `completeDeal` somehow, or exits stake before the slashing tx lands.
- **Mitigation:** 14-day unbonding period on `ProverStaking`. Slashing callable only by authorized controllers (currently `StorageMarketplace`). `MAX_PROOF_GAP` + `faultDeal()` callable by anyone permissionlessly after the gap elapses.
- **Residual:** Need to verify the state-machine transitions cannot be reordered to the prover's advantage (audit item).

### T-03. Upgrade-path abuse (High)

- **Vector:** `ProofVerifier` is UUPS-upgradeable. A compromised owner key pushes a malicious implementation that drains pending sybil fees or rewrites data sets.
- **Mitigation:** Contract owner should be a Safe multisig (2-of-N) plus a `Timelock` with sufficient delay for operators to notice. `announcePlannedUpgrade` exists for this; deployment config must wire it. Plain EOA ownership on mainnet is a deployment bug.
- **Residual:** Key management discipline.

### T-04. Sybil-fee griefing (Low)

- **Vector:** Attacker spams `createDataSet` to waste chain storage, paying 0.1 ETH per call.
- **Mitigation:** The fee itself is the deterrent. 0.1 ETH per pointless data set is not worth sustaining. Worst case: operator can raise the fee via a proxy upgrade.

### T-05. CommP collision (Critical, infeasible)

- **Vector:** Attacker finds two different byte strings with the same CommP hash and uses that to get paid for storing a different object than the one the client requested.
- **Mitigation:** CommP is SHA-256-based; pre-image + collision resistance are computationally infeasible. Not a realistic threat.

## 3. Prover Layer

### T-06. Lazy prover — accept deal, never store (Critical)

- **Vector:** Prover accepts the deal, pockets payment, never actually stores the piece.
- **Mitigation:** Periodic PDP challenges. Missing challenges past `MAX_PROOF_GAP` triggers `faultDeal` which slashes stake + refunds client. Only remediation for first offense is slash; repeat offenders lose stake quickly.
- **Detection:** Any single honest watcher can call `faultDeal`.

### T-07. Source URL DoS (Medium)

- **Vector:** Client publishes a Source URL that points at a huge file, gets a prover to waste bandwidth downloading it, deal fails on CommP mismatch.
- **Mitigation:** `Fetcher.MaxBytes` hard limit (32 GiB default). `ValidateSourceURL` rejects private IPs, loopback, userinfo. Client pays for deal up-front; failure burns part of the escrow.
- **Residual:** First-download cost falls on the prover. Provers can cap accept rate or require a deposit beyond the deal fee for large pieces.

### T-08. Source URL SSRF / exfil (High)

- **Vector:** Attacker publishes a source URL pointing at internal infrastructure (metadata service, internal APIs) to trick the prover into fetching secrets.
- **Mitigation:** `ValidateSourceURL` rejects loopback, private, link-local. `$PROVA_PULL_ALLOW_INSECURE` / `[source_url].allow_insecure` must be off in production. Default deny.
- **Residual:** DNS rebinding is possible; defense-in-depth would add post-resolution IP re-checks. Not implemented in v1.

### T-09. Key exfiltration (Critical)

- **Vector:** Prover's signing key leaks; attacker uses it to drain staked PROVA.
- **Mitigation:** Wallet package supports keystore-with-passphrase and env-based loading. Operators should use keystore + `$PROVA_KEYSTORE_PASSPHRASE`. systemd unit has hardened defaults. 14-day unbonding on staking means leaked stake isn't instantly drainable.
- **Residual:** Key management discipline. Hardware-signer support is a future enhancement.

### T-10. Prover service denial (Medium)

- **Vector:** Attacker spams HTTP retrieval endpoint to exhaust the prover's bandwidth.
- **Mitigation:** Operators deploy behind a reverse proxy with rate limiting. `Fetcher` already caps inbound downloads. HTTP server is stateless for retrieval.
- **Residual:** Bandwidth accounting + paid retrieval is a future phase.

## 4. Client Layer

### T-11. Client refuses to release (Low)

- **Vector:** Client never calls `completeDeal` after the deal duration elapses.
- **Mitigation:** `completeDeal` is callable by anyone after `endsAt`, not just the client. Prover can call it themselves.

### T-12. Client cancels right before acceptance (Low)

- **Vector:** Client proposes a deal, prover starts downloading the piece, client cancels before acceptance, prover has wasted bandwidth.
- **Mitigation:** Provers should wait until acceptance tx is mined before committing real resources. Small-piece deals absorb the cost; large-piece deals need operator discipline.

## 5. Chain / Operational

### T-13. RPC censorship (Medium)

- **Vector:** Operator's RPC provider refuses to include the prover's `provePossession` tx, causing missed proofs.
- **Mitigation:** Multiple RPC endpoints configurable (future). Prover retries with backoff. If the whole Base sequencer censors, that's a Base-level issue.

### T-14. Randomness bias (Low)

- **Vector:** Block proposer biases `block.prevrandao` to skew challenge leaf selection.
- **Mitigation:** Only affects *which* leaves are challenged. Provers must still hold the entire piece; they can't pre-bias that. Worst-case gives a marginal statistical advantage over the full challenge horizon.
- **Residual:** Switch to Chainlink VRF or a commit-reveal scheme if the bias becomes meaningful at scale.

### T-15. Chain reorg (Low on Base)

- **Vector:** A Base reorg rewrites a recently-emitted event, temporarily confusing the prover's event poller.
- **Mitigation:** `BlockLookback` reorg buffer (default 6 blocks on mainnet, 0 on anvil). Watermark advances only after all filters succeed.
- **Residual:** Base has very short reorg horizons (L2 blocks are final once the L1 batch posts); in practice this is not a concern.

## 6. Out of scope

- **AI inference proofs** — Prova v2 doesn't do compute verification.
- **PoRep / sealing** — not part of v2.
- **TEE attestation** — not part of v2.
- **Cross-chain bridges** — Prova is single-chain (Base).
- **Token launch / distribution** — covered separately in `TOKENOMICS-v2.md`.

## 7. Open items for audit

- Formal verification of `StorageMarketplace` state machine.
- Fuzz testing of `ProofVerifier` proof path.
- Re-entrancy analysis on every `external payable` entry point.
- Gas-limit edge cases on `addPieces` with pathological piece counts.
- Upgrade-path review for `ProofVerifier` UUPS proxy.
