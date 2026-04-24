# Filecoin Source Map — What We Pull Into Prova

**Date:** 2026-04-24
**Status:** Working reference, will guide Phase 3+ of the pivot plan

## License Verification — All Clear ✅

All 7 source repos are dual-licensed **Apache-2.0 OR MIT** under Protocol Labs' "Permissive License Stack." We can take the MIT side cleanly.

| Repo | License | Status |
|------|---------|--------|
| `FilOzone/pdp` | Apache-2.0 OR MIT | ✅ pullable |
| `FilOzone/synapse-sdk` | Apache-2.0 OR MIT | ✅ pullable |
| `FilOzone/filecoin-cloud` | MIT | ✅ pullable |
| `FilOzone/filecoin-pay-explorer` | Apache-2.0 OR MIT | ✅ pullable |
| `FilOzone/dealbot` | Apache-2.0 OR MIT | ✅ pullable |
| `FilOzone/pdp-explorer` | MIT | ✅ pullable |
| `filecoin-project/curio` | Apache-2.0 OR MIT | ✅ pullable |

**Attribution:** Each source file copied or derived gets an SPDX header and a pointer to the origin repo + commit.

## Source Repos Inventory

All cloned to `/Users/reiers/.openclaw/workspace/prova-sources/`.

### 1. `FilOzone/pdp` — Solidity PDP Contracts (🔥 CROWN JEWEL)

**Size:** 860 KB, 6,688 lines Solidity, active (pushed Apr 14).
**License:** Permissive Stack (Apache-2.0 OR MIT).

**⚠️ Filecoin-specific dependencies (minor, easily swapped):**
- `FVMPay.burn(amount)` at line 227 → replace with our ERC-20 burn logic
- `FVMPay.pay(msg.sender, refund)` at line 846 → replace with ERC-20 `transferFrom`
- `FVMRandom.getBeaconRandomness(epoch)` at line 896 → replace with Chainlink VRF, `block.prevrandao`, or a commit-reveal scheme

Only **4 lines total of FVM-specific integration** in the entire 1,116-line contract. Trivial to swap.

**Key files we pull directly or adapt:**
- `src/PDPVerifier.sol` — **1,116 lines**, the core on-chain PDP verifier contract. UUPS upgradeable. This is essentially our `ProofVerifier.sol`.
- `src/Cids.sol` — 150 lines, CID/CommP encoding/decoding on-chain.
- `src/Proofs.sol` — 217 lines, Merkle proof verification library.
- `src/SimplePDPService.sol` — 305 lines, reference PDP service implementation.
- `src/BitOps.sol` — 97 lines, bit manipulation helpers for Merkle tree indexing.
- `src/Fees.sol` — 37 lines, fee calculation helpers.
- `src/ERC1967Proxy.sol` — 12 lines, proxy wrapper.
- `src/IPDPProvingSchedule.sol` — 30 lines, proving schedule interface.
- `src/interfaces/IPDPEvents.sol`, `IPDPTypes.sol`, `IPDPVerifier.sol` — clean interface files.
- `docs/gas-benchmarks/` — we get actual gas measurements for free.
- `test/` — full Foundry test suite we can adapt.

**What this means:**
We don't need to write `ProofVerifier.sol` from scratch. We fork `PDPVerifier.sol` + `Proofs.sol` + `Cids.sol` + interfaces into our `contracts/src/` directly. Change the listener interface so it calls our `ProverStaking` / `StorageMarketplace` / `DisputeManager` instead of Filecoin's `FilecoinPay`.

**Stripping needed:**
- Remove Filecoin-specific types if any (most are chain-agnostic)
- Replace `IFilecoinPay` integration with our own payment router
- Replace proving-schedule coupling if too Filecoin-specific

**Estimated adaptation effort:** 4-8 hours to fork, wire into our contracts, adapt tests. Compare to writing from scratch: 20-40 hours.

### 2. `FilOzone/synapse-sdk` — TypeScript SDK (🔥 CROWN JEWEL)

**Size:** 15 MB, ~60,000 lines TypeScript, active (pushed Apr 22).
**License:** Permissive Stack.
**Built on:** viem (modern, active, preferred over ethers.js for new projects).

**Structure:**
```
packages/synapse-sdk/src/
├── errors/        — typed error hierarchy
├── filbeam/       — FilBeam (retrieval/CDN) integration
├── payments/      — on-chain payment interactions (PaymentsService)
├── sp-registry/   — Storage Provider registry client (SPRegistryService)
├── storage/       — upload/download, deal management (StorageManager)
├── warm-storage/  — hot storage workflows (WarmStorageService)
├── utils/         — ethers helpers, CID helpers, CommP helpers
├── synapse.ts     — top-level client
├── types.ts       — 24,919 lines (!) of TypeScript types
└── index.ts       — public exports
```

Clean modular service architecture. Rename map is obvious:
- `Synapse` class → `Prova` class
- `SPRegistryService` → `ProverRegistryService`
- `WarmStorageService` → `StorageService`
- `PaymentsService` → keep name, swap contract addresses
- `FilBeamService` → optional adapter, may drop for v1

**What this means:**
This is ~80% of what our `sdk/typescript/` needs. The Synapse SDK is essentially a TypeScript client for a PDP-based storage network with Ethereum-compatible payments. We can:
- Fork it
- Rename `Synapse*` → `Prova*`
- Swap contract addresses for our deployed Prova contracts
- Keep or adapt the warm-storage / retrieval logic
- Keep or adapt the SP-registry abstraction (becomes `ProverRegistry`)
- Keep the error hierarchy wholesale

**Stripping needed:**
- Remove FilBeam-specific retrieval network integration (or keep as optional adapter)
- Remove any Filecoin Virtual Machine (FVM) specifics
- Replace FilecoinPay contract interaction with our contract set

**Estimated adaptation effort:** 10-20 hours for a v1 fork. Writing from scratch: 80-120 hours.

### 3. `FilOzone/filecoin-cloud` — Next.js Frontend (🟢 useful)

**Size:** 15 MB, 6,692 lines TypeScript, active (pushed Apr 19).
**License:** MIT.

**What it is:** The Filecoin Onchain Cloud user-facing web app. Upload files, manage deals, view proofs, browse providers.

**Value:**
- UI components for deal creation, upload flows, proof visualization
- Wallet connection patterns
- ENS/address formatting utilities
- Styled with Tailwind, React hooks well-organized

**Use for Prova:** Strip Filecoin branding and repoint at Prova contracts. Much of our website / dashboard comes from here.

**Estimated adaptation effort:** 8-12 hours for a rebranded, repointed version.

### 4. `FilOzone/filecoin-pay-explorer` — Payment Explorer (🟡 moderate value)

**Size:** 3.1 MB, 18,801 lines TypeScript, active (pushed Apr 24).
**License:** Permissive Stack.

**Structure:**
- `apps/explorer` — UI for browsing on-chain payments
- `apps/metrics` — metrics dashboards
- `packages/subgraph` — TheGraph subgraph definitions
- `packages/ui` — shared UI components
- `packages/types` — shared types

**Value for Prova:**
- The **subgraph** is the biggest win. TheGraph indexing for Ethereum-native storage deals. Adapt the subgraph schema to our `StorageMarketplace` / `ProverRegistry` / `ProofVerifier` events.
- UI components for payment flows.

**Use for Prova:** Fork the subgraph, retarget entities to our events. Build a Prova deal explorer.

**Estimated adaptation effort:** 12-20 hours.

### 5. `FilOzone/dealbot` — Testing / Simulation Tool (🟡 moderate value)

**Size:** 3.4 MB, 23,673 lines TypeScript, active (pushed Apr 24).
**License:** Permissive Stack.

**What it is:** An automated client that simulates realistic storage deal workflows against real Storage Providers. Collects performance metrics.

**Value for Prova:**
- Integration-testing framework we can adapt for Prova testnet deals.
- Metrics collection patterns.
- Realistic load generation for stress-testing our prover network.

**Use for Prova:** Keep as dev tool. Fork, repoint at Prova contracts, use to stress-test testnet.

**Estimated adaptation effort:** 8-15 hours.

### 6. `FilOzone/pdp-explorer` — PDP Network Explorer (🟢 useful)

**Size:** 7.9 MB. Go backend (10,270 lines) + TS frontend (17,499 lines) + TheGraph subgraph.
**License:** MIT.

**Structure:**
- `backend/indexer/` — Go indexer reading on-chain PDP events
- `backend/server/` — Go API server
- `client/` — React frontend
- `subgraph/` — TheGraph subgraph

**Value for Prova:**
- **Indexer pattern in Go** — if we want a high-performance Go indexer, this is the template.
- TheGraph subgraph for PDP events.
- UI patterns for showing proof history, provider reliability, etc.

**Use for Prova:** Fork the subgraph (complements #4). Go indexer is optional; we may just use TheGraph.

**Estimated adaptation effort:** 10-15 hours.

### 7. `filecoin-project/curio` — Storage Provider Software (🔥 CROWN JEWEL for prover binary)

We already have this cloned at `/Users/reiers/.openclaw/workspace/curio`.

**PDP/market/FoC-related code:**
- `pdp/` — **27 Go files, 20,338 lines**. Storage provider PDP integration.
  - `handlers.go`, `handlers_add.go`, `handlers_create.go`, `handlers_pull.go`, `handlers_upload.go`
  - `auth.go` — auth token handling
  - `piece_cid.go` — CommP computation
  - `indexing.go` — IPNI / content indexing
  - `contract/` — PDP contract bindings (Go)
- `pdpv0/` — Legacy PDPv0 task (separate)
- `market/` — **52 Go files, 22,720 lines**.
  - `mk12/` — legacy market protocol
  - `mk20/` — new market protocol (Mk20 update, PR #1087)
  - `http/` — HTTP transport for deals
  - `retrieval/` — retrieval flows
  - `libp2p/` — libp2p transport
  - `indexstore/` — deal indexing
  - `ipni/` — IPNI integration
  - `storageingest/` — data ingest
  - `denylist/` — client denylisting
- `tasks/storage-market/` — **7 Go files, 3,848 lines**. Task runner integrations.
- `tasks/pdp/` — PDP task runner (v1). This is where PR #1167 savecache work landed.
- `tasks/pdpv0/` — PDPv0 task runner.
- `tasks/pay/settle_task.go` — FilecoinPay settlement task.
- `lib/filecoinpayment/` — FilecoinPay integration.
- `lib/market/` — shared market types.
- `cmd/pdptool/` — PDP CLI tool.
- `web/static/pages/{mk20,pdp,market,...}/` — web UI for operators.

**Value for Prova:**

**TAKE:**
- PDP engine (piece CID computation, Merkle tree proofs, challenge response) — already battle-tested
- HTTP handlers for piece upload/download
- CommP computation with savecache/snapshot optimization (the work from PR #1167)
- Deal lifecycle machinery (accept, download, verify, index)
- Indexstore patterns
- Retrieval flows (HTTP-based)

**STRIP:**
- All Filecoin consensus / chain-head watching
- Lotus-specific RPC integration
- libp2p (optional, we probably drop for v1)
- FVM-specific code (FilecoinPay can become a pattern but we replace with our contracts)
- mk12 legacy code (only take if needed for backwards compat)

**TRANSPLANT PATH:**
- Curio is Go. Our prover is currently Rust.
- **Decision point:** write Prova prover in Go (reuse Curio wholesale) or Rust (reuse current Prova PDP code, adapt from Curio manually).
- **Recommendation:** Switch Prova prover language to Go. Reasons:
  - Curio's PDP + market code is ~40K lines of battle-tested production Go
  - Our existing Rust prover (`node/src/pdp.rs`, `merkle.rs`, etc.) is maybe 10K lines of unproven code
  - Go has equivalent Ethereum tooling (go-ethereum, geth bindings via `abigen`)
  - Go prover runs the same GC / TLS / HTTPS patterns Curio SPs already run
  - Filecoin storage providers (our initial prover audience) already run Go stacks

**This is a big call.** Flag for Nicklas decision.

**Estimated adaptation effort if we go Go:** 30-60 hours to extract, strip, and adapt. Much less than writing equivalents from scratch.

## The Transplant Strategy

### Phase A: Solidity Contracts (first)
1. Fork `FilOzone/pdp/src/PDPVerifier.sol` → `prova/contracts/src/ProofVerifier.sol`.
2. Fork `Cids.sol`, `Proofs.sol`, `BitOps.sol`, `Fees.sol` as-is.
3. Fork `SimplePDPService.sol` as reference, evolve into our `StorageMarketplace.sol`.
4. Keep `ProvaToken.sol` as-is.
5. Write fresh `ProverRegistry.sol`, `ProverStaking.sol`, `ContentRegistry.sol` (inspired by SimplePDPService patterns).
6. Write `DisputeManager.sol` (v2, can defer).
7. Port Foundry test suite.

**Outcome:** A complete contract set on Ethereum, using battle-tested PDP verification math.

### Phase B: TypeScript SDK (next)
1. Fork `FilOzone/synapse-sdk/packages/synapse-sdk/` → `prova/sdk/typescript/`.
2. Rename `Synapse*` types to `Prova*`.
3. Swap contract addresses for our deployed Prova contracts.
4. Keep error hierarchy, warm-storage patterns, SP-registry abstraction.
5. Publish as `@prova-network/sdk`.

**Outcome:** A complete TypeScript SDK for Prova, day 1.

### Phase C: Prover Binary (biggest decision)
**If Go (recommended):**
1. Create `/Users/reiers/.openclaw/workspace/prova/prover/` as a Go module.
2. Extract `curio/pdp/` + relevant `curio/market/` + `curio/tasks/pdp/` code.
3. Strip Lotus / consensus / libp2p / FVM dependencies.
4. Wire up to our Ethereum contracts via go-ethereum.
5. Simplify: one binary, no harmonytask scheduler, just prover + HTTPS server.
6. Archive the Rust prover code to `archive/rust-prover/`.

**If Rust (keep current):**
1. Keep `prover/` in Rust.
2. Manually port key Curio logic to Rust.
3. More work, less leverage.

### Phase D: Indexer + Explorer
1. Fork `FilOzone/filecoin-pay-explorer/packages/subgraph/` → `prova/subgraph/`.
2. Adapt entity types to our events.
3. Deploy to TheGraph.
4. Fork `filecoin-cloud` → `prova/app/` as web UI.
5. Rebrand, repoint.

### Phase E: Testing Tool
1. Fork `FilOzone/dealbot` → `prova/dealbot/` or `prova/testtool/`.
2. Adapt for Prova testnet stress testing.

## Revised Scope Estimate

| Phase | Old estimate (scratch) | New estimate (fork) | Savings |
|-------|------------------------|---------------------|---------|
| Contracts | 20-40 hours | 4-8 hours | ~80% |
| TypeScript SDK | 80-120 hours | 10-20 hours | ~85% |
| Prover (if Go, reusing Curio) | 100+ hours | 30-60 hours | ~50% |
| Prover (if Rust) | 60-90 hours | 40-70 hours | ~20% |
| Indexer + UI | 40-60 hours | 10-20 hours | ~70% |
| Testing tool | 20-30 hours | 8-15 hours | ~55% |
| **Total (Go prover path)** | **~260-400 hours** | **~65-125 hours** | **~70%** |
| **Total (Rust prover path)** | **~220-370 hours** | **~75-140 hours** | **~65%** |

**This is a major acceleration.** Time to internal demo drops from 4-6 weeks to 1.5-3 weeks realistic.

## Open Questions

1. **Go prover or Rust prover?**
   Recommendation: Go. Reuses Curio wholesale, SPs already run Go.

2. **Which contracts to fork vs write fresh?**
   - Fork: `PDPVerifier.sol`, `Cids.sol`, `Proofs.sol`, `BitOps.sol`, `Fees.sol`.
   - Write fresh (using their patterns): `ProverRegistry`, `ProverStaking`, `StorageMarketplace`, `ContentRegistry`, `DisputeManager`.

3. **Use synapse-sdk directly or adapt selectively?**
   Recommendation: Fork wholesale, rename, repoint. Much faster than cherry-picking.

4. **Keep our Rust code?**
   If we go Go prover, the Rust prover code archives. If we keep Rust, we need a call on whether to port from Curio (tedious) or keep our own clean-room implementation.

5. **How aggressively do we pull?**
   Forking entire repos, renaming, repointing is faster than tip-toeing. Attribution stays in place. License allows it.

## Attribution Template

Every file copied or derived gets this header:

```solidity
// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2024-2026 Protocol Labs and contributors.
// Copyright (c) 2026 Prova Network contributors.
//
// This file is adapted from FilOzone/pdp (github.com/FilOzone/pdp)
// commit <sha>, originally under the Permissive License Stack.
// Attribution retained as required.
```

And a root `ATTRIBUTION.md` listing every upstream source repo.

---

*End of source map v1. Next: update PIVOT-PLAN to incorporate the fork strategy.*
