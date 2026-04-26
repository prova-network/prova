<div class="spec-meta">
  <span class="label">State</span> <span class="badge state-stable">Stable</span>
  <span class="label">Theory audit</span> <span class="badge audit-na">N/A</span>
  <span class="label">Last updated</span> 2026-04-26
</div>

# 6.2 Audit checklist

The pre-deployment review surface for an external auditor. This is the list of things we expect a tier-1 audit firm to check before we sign off on mainnet. Internal review at this same scope is in [`SECURITY-AUDIT-2026-04-25.md`](https://github.com/prova-network/prova/blob/main/SECURITY-AUDIT-2026-04-25.md).

## 6.2.1 Scope

In scope for the external audit:

- [`prova-network/contracts`](https://github.com/prova-network/contracts) — every Solidity file under `src/`
- [`prova-network/prover`](https://github.com/prova-network/prover) — the Go `provad` daemon, focused on:
  - PDP proof construction
  - Stake manager
  - Retrieval HTTP server
- [`prova-network/website/functions`](https://github.com/prova-network/website/tree/main/functions) — the Cloudflare Pages Functions that serve the public API

Out of scope:

- The marketing site (no security boundary)
- The docs site (read-only content)
- The CLI (talks only to the public API; no privilege boundary)

## 6.2.2 Smart-contract review

### Per-contract checks

For each of `ProvaToken`, `ProvaVesting`, `ProverRewards`, `FeeRouter`, `StorageMarketplace`, `ProverStaking`, `ProverRegistry`, `ContentRegistry`, `ProofVerifier`:

- [ ] All external functions have explicit access control or are deliberately permissionless
- [ ] All state-mutating external functions are `nonReentrant` where reentrancy could be a concern
- [ ] Token transfers use `SafeERC20`
- [ ] Numeric operations use `unchecked` blocks only where overflow is provably impossible
- [ ] Events are emitted before external calls (CEI pattern)
- [ ] No unbounded loops on public functions
- [ ] Storage layout is documented and changes are reviewed for upgrade-safety
- [ ] Constructor arguments are validated (zero-address checks, range checks)

### Cross-contract checks

- [ ] Marketplace's authorized-controller relationships are wired correctly (staking, content, rewards)
- [ ] Stake commitment is balanced: every committed-bytes call has a matching release
- [ ] Slashing accounting is balanced: slashed PROVA is destroyed, not transferred to a treasury
- [ ] FeeRouter swap path is protected against MEV via caller-supplied `minProvaOut`
- [ ] ProverRewards anti-gaming gates (self-dealing, redundancy, single-count, quality) all execute before any state write

### Token-economics math

- [ ] ProvaToken total supply matches the published 100M PROVA
- [ ] ProvaVesting cliff and linear math match the published schedule
- [ ] ProverRewards yearly emission sums to 50M (verified by the test `test_yearlyEmission_sumsTo50M`)
- [ ] FeeRouter SPLIT mode share rounds correctly across fees that don't divide evenly

## 6.2.3 Off-chain review

### `provad` Go daemon

- [ ] PDP proof construction matches the spec exactly (Fr32, trunc254, base32 encoding)
- [ ] Three independent piece-CID implementations (browser, CLI, server) produce byte-identical CIDs for canonical test vectors
- [ ] Wallet handling: keystore is encrypted at rest, never logs the private key
- [ ] Retrieval HTTP server respects the security headers from spec §4.1
- [ ] Range requests are correct: `Content-Range`, `206`, byte-offsets

### API gateway

- [ ] Magic-link flow is single-use with the documented rate limits
- [ ] Origin / Referer guard rejects unauthorized origins
- [ ] No `?token=` query-string auth path remains anywhere
- [ ] CSP, COOP, CORP headers match spec §4.2.9
- [ ] Stage server CID verification is on by default; cannot be silently disabled
- [ ] Abuse-report rate limits and ban-list lookups are atomic with respect to the upload path

## 6.2.4 Operational review

- [ ] Multisig configurations match the published `governance.md`
- [ ] Emergency pause role is the 5-of-9 multisig, not an EOA
- [ ] Treasury address is a multisig
- [ ] FeeRouter owner is a multisig
- [ ] All deploy scripts are deterministic and reproducible
- [ ] Verified contract source on Basescan matches the audited commit hash

## 6.2.5 Recommended deliverables

For a tier-1 audit ($150-300K, 4-6 weeks scope):

1. Initial-findings draft within 2 weeks of engagement start.
2. Final report covering:
   - Critical findings (0 expected at this point)
   - High findings (≤ 3 expected)
   - Medium findings
   - Low / informational findings
   - Architectural notes
   - Test coverage assessment
3. Re-test report after fixes land.

Internal review status (as of 2026-04-26):

- 1 Critical (F-01: email signup with no verification) — closed
- 3 High (F-02 CSRF/Origin, F-03 stage CID verify, F-04 XSS) — closed
- 12 Medium / Low — see [internal audit doc](https://github.com/prova-network/prova/blob/main/SECURITY-AUDIT-2026-04-25.md) for status.

## 6.2.6 Re-audit triggers

We re-audit any of:

- A change to the slashing math
- A change to the emission schedule
- An upgrade to `ProofVerifier` (UUPS)
- A new contract added to the trust boundary
- A material change to the off-chain stage server's role

A re-audit MAY be a delta-review against the previous audit (cheaper, faster) when the change is contained. Major architectural changes require a full audit.
