# Prova Build Backlog

## Phase 1 — Foundation (Active Sprint)

### ✅ Completed
- [x] REPO-001: Repo scaffolding (/docs, /spec, /proto, /node, /chain, /ops)
- [x] SPEC-001: QBP formal spec — `spec/qbp-protocol.md`
- [x] SPEC-002: Activation Merkle tree format — `spec/activation-merkle-tree.md`
- [x] SPEC-003: Model registry schema — `spec/model-registry.md`
- [x] SPEC-004: PDP integration spec — `spec/pdp-integration.md`
- [x] PROTO-001: QBP protobuf message definitions — `proto/qbp.proto`
- [x] NODE-001: Activation Merkle tree builder — `node/src/merkle.rs` (8 tests)
- [x] NODE-003: Inference runner (mock + faulty) — `node/src/runner.rs` (6 tests)
- [x] NODE-004: Bisection game participant — `node/src/participant.rs` (6 tests)
- [x] CHAIN-001: Commit store + challenge window — `chain/src/commit.rs` (5 tests)
- [x] CHAIN-001: Dispute arena + bisection game — `chain/src/dispute.rs` (5 tests)
- [x] CHAIN-001: E2E simulation — `chain/src/simulation.rs` (1 test)
- [x] CHAIN-002: Stake ledger + slashing — `chain/src/stake.rs` (11 tests)
- [x] CHAIN-003: Model registry on-chain — `chain/src/registry.rs` (4 tests)

**Total: 173 passing tests (cargo test), 2 external deps (sha2, serde). 26 source files.**

### P0 — In Progress
- [ ] EXP-001: Determinism harness on Blackwell [KODA] — Blackwell confirmed ready, harness not yet run
- [x] SPEC-005: Audit protocol spec + implementation — `spec/audit-protocol.md`, `chain/src/audit.rs` (10 tests)
- [x] SPEC-006: Streaming payments spec — `spec/streaming-payments.md`
- [ ] DOCS-001: Developer quickstart guide [KESTREL]
- [x] CHAIN-004: Payment channel implementation — `chain/src/payment.rs` (9 tests)
- [x] NODE-002: PDP proof engine scaffold — `node/src/pdp.rs` (11 tests)

### P1 — Next
- [x] EXP-002: TensorRT INT8 cross-architecture determinism test — `node/src/determinism.rs` (12 tests)
- [x] EXP-003: CPU canonical verification path test — `node/src/canonical_cpu.rs` (18 tests)
- [x] CI-001: CI pipeline (build + test) — `.github/workflows/ci.yml`
- [x] NODE-005: Real llama.cpp integration (activation capture via hook) — `node/src/llamacpp.rs` (8 tests)
- [x] CHAIN-005: Epoch ticker + state transitions — `chain/src/epoch.rs`
- [x] SPEC-007: Network protocol (P2P gossip, block propagation) — `spec/network-protocol.md`

### P2 — Later
- [x] NODE-018: Configuration manager (TOML config, defaults, validation, env overrides, roundtrip serialization) — `node/src/config.rs` (24 tests)
- [x] NODE-007: JSON-RPC 2.0 API scaffold — `node/src/rpc.rs` (16 tests)
- [x] NODE-006: P2P networking scaffold — `node/src/network.rs` (15 tests)
- [x] CHAIN-006: Block production + consensus — `chain/src/block.rs` (37 tests)
- [x] CHAIN-007: Genesis state — `chain/src/genesis.rs` (14 tests)
- [ ] SPEC-009: Consensus specification [KESTREL]
- [x] OPS-001: Devnet simulation — `node/src/devnet.rs` (5 tests)
- [x] DOCS-002: Architecture overview diagram — `docs/architecture.md` (payments, audit, dependency graph added)

## Phase 4 — Scheduling & Orchestration

### P0 — Active
- [x] CHAIN-013: Job scheduler (inference request routing, assignment, cancellation, timeout) — `chain/src/scheduler.rs` (16 tests)
- [x] NODE-013: Job executor (worker loop: poll scheduler, run inference, deliver result) — `node/src/executor.rs` (11 tests)
- [x] CHAIN-014: Service-level agreements (SLA enforcement, penalty curves) — `chain/src/sla.rs` (14 tests)
- [x] NODE-014: Metrics & telemetry (Prometheus-style counters, histograms) — `node/src/metrics.rs` (15 tests)
- [x] INT-003: Adversarial scheduler test (byzantine providers, deadline gaming) — `chain/src/adversarial_test.rs` (19 tests)

### P1 — Next
- [x] CHAIN-015: Reputation system (EMA scoring, decay, slashing integration) — `chain/src/reputation.rs` (21 tests)
- [x] NODE-015: Provider auto-pricing (market-adaptive fee adjustment) — `node/src/pricing.rs` (17 tests)
- [x] SPEC-013: Security threat model specification — `spec/security-threat-model.md` (21 threats, 8 categories)

## Phase 5 — Cross-Chain & Finality

### P0 — Active
- [x] CHAIN-016: Filecoin checkpoint anchoring (quorum voting, L1 anchor, light client verify) — `chain/src/checkpoint.rs` (16 tests)
- [x] CHAIN-017: Cross-chain bridge message format (Prova↔Filecoin state proofs) — `chain/src/bridge.rs` (20 tests)
- [x] NODE-016: Checkpoint submitter (automatic L1 tx submission) — `node/src/submitter.rs` (15 tests)
- [x] SPEC-014: Checkpoint anchoring specification — `spec/checkpoint-anchoring.md`

### P1 — Next (promoted to P0)
- [x] CHAIN-018: Finality gadget (fast finality via checkpoint + slow finality via L1) — `chain/src/finality.rs` (18 tests)
- [x] NODE-017: L1 event watcher (monitor Filecoin for anchor confirmations) — `node/src/watcher.rs` (14 tests)
- [x] SPEC-015: Bridge security specification — `spec/bridge-security.md`

## Phase 2 — Integration & Hardening

### P0 — Active
- [x] CHAIN-008: Transaction mempool (priority, nonce, eviction) — `chain/src/mempool.rs` (16 tests)
- [x] NODE-009: CLI scaffold (subcommands: run, status, account, tx) — `node/src/cli.rs` (22 tests)
- [x] INT-001: Multi-node integration test harness — `node/src/multinode.rs` (12 tests)
- [x] CHAIN-009: State trie (account balances + nonce tracking) — `chain/src/state.rs` (16 tests)
- [x] SPEC-010: Token economics specification — `spec/token-economics.md`

## Phase 3 — Economics & Governance

### P0 — Active
- [x] CHAIN-010: Reward distribution (block rewards, inference fees, storage subsidies, challenger bounties) — `chain/src/rewards.rs` (15 tests)
- [x] CHAIN-011: Transaction execution engine — `chain/src/executor.rs` (17 tests)
- [x] NODE-010: Wallet + key management (Ed25519 signing, keystore) — `node/src/wallet.rs` (16 tests)
- [x] SPEC-011: Governance specification — `spec/governance.md`, `chain/src/governance.rs` (17 tests)
- [x] INT-002: Full-chain integration test (genesis → blocks → rewards → claims) — `chain/src/integration_test.rs` (5 tests)

### P1 — Next (promoted to P0)
- [x] CHAIN-012: Gas metering + fee market (EIP-1559 style) — `chain/src/gas.rs` (19 tests)
- [x] NODE-011: Persistent storage backend (sled) — `node/src/storage.rs` (15 tests)
- [x] NODE-012: Chain sync protocol (block download + verification) — `node/src/sync.rs` (14 tests)
- [x] SPEC-012: Light client specification — `spec/light-client.md`

## Phase 6 — Client SDK & Developer Tools

### P0 — Active
- [x] SDK-001: Client SDK (request builder, signing, provider discovery, batch ops) — `sdk/src/lib.rs` (19 tests)
- [x] SDK-002: JSON-RPC client (connect to node, submit jobs, poll results) — `sdk/src/rpc_client.rs` (15 tests)
- [x] SDK-003: CLI wallet integration (import keys, sign offline) — `sdk/src/cli_wallet.rs` (22 tests)

### P1 — Next
- [x] SDK-004: TypeScript/WASM bindings for browser clients — `sdk/wasm/` (27 tests)
- [x] SDK-005: Rate limiting & retry logic — `sdk/src/retry.rs` (17 tests)
- [x] DOCS-003: SDK usage guide & examples — `docs/sdk-guide.md`

## Phase 7 — Testnet Readiness

### P0 — Active
- [x] CHAIN-019: State snapshot system (chunked export/import, integrity verification, streaming importer) — `chain/src/snapshot.rs` (20 tests)
- [x] NODE-019: Snapshot serving over P2P — `node/src/snapshot_serve.rs` (19 tests)
- [x] CHAIN-020: State pruning (retain last N snapshots, garbage collect old blocks) — `chain/src/pruning.rs` (13 tests)
- [x] NODE-020: Fast sync mode (download snapshot instead of replaying blocks) — `node/src/fast_sync.rs` (14 tests)
- [x] OPS-002: Testnet genesis config + boot nodes — `ops/testnet/` (genesis.toml, bootnodes.toml, testnet.rs — 16 tests)

### P1 — Next
- [x] CHAIN-021: Protocol upgrade mechanism (fork scheduling, version negotiation) — `chain/src/upgrade.rs` (17 tests)
- [x] NODE-021: Graceful shutdown + state persistence — `node/src/shutdown.rs` (18 tests)
- [x] DOCS-004: Testnet operator guide — `docs/testnet-operator-guide.md`

## Phase 8 — Observability & Indexing

### P0 — Active
- [x] CHAIN-022: Event log system (structured emission, Merkle receipts, indexed queries) — `chain/src/events.rs` (15 tests)
- [x] NODE-022: Event subscription engine (WebSocket push, filter subscriptions) — `node/src/subscriptions.rs` (16 tests)
- [x] CHAIN-023: Receipt storage (persist receipts alongside blocks, proof-of-inclusion) — `chain/src/receipts.rs` (16 tests)
- [x] SDK-006: Event subscription client (subscribe, filter, replay) — `sdk/src/event_client.rs` (14 tests)

### P1 — Next
- [x] NODE-023: Block explorer API (blocks, txs, events, accounts) — `node/src/explorer.rs` (19 tests)
- [x] SPEC-016: Event schema specification — `spec/event-schema.md`
- [x] SDK-007: Historical event replay & caching — `sdk/src/event_replay.rs` (16 tests)

## Phase 9 — Security Hardening & Audit Prep

### P0 — Active
- [x] CHAIN-024: Access control layer (role-based capabilities, pause, expiry, overrides) — `chain/src/access.rs` (17 tests)
- [x] CHAIN-025: Rate limiter (per-address tx throttling, adaptive limits) — `chain/src/rate_limiter.rs` (15 tests)
- [x] NODE-024: TLS transport layer (encrypted P2P, certificate pinning) — `node/src/tls.rs` (16 tests)
- [x] SPEC-017: Security audit checklist specification — `spec/security-audit-checklist.md` (73 checks)

### P1 — Next
- [x] CHAIN-026: Formal invariant checker (balance conservation, stake consistency, nonce monotonicity, reward conservation, job uniqueness) — `chain/src/invariants.rs` (16 tests)
- [x] NODE-025: Fuzz testing harness (property-based testing for chain state) — `node/src/fuzz.rs` (12 tests)
- [x] DOCS-005: Security model documentation — `docs/security-model.md`

## Phase 10 — Model Marketplace & Discovery

### P0 — Active
- [x] CHAIN-027: Model marketplace (listing, bidding, provider discovery) — `chain/src/marketplace.rs` (17 tests)
- [x] NODE-026: Marketplace CLI commands (list, bid, discover) — `node/src/marketplace_cli.rs` (37 tests)
- [x] SDK-008: Marketplace client SDK (provider search, bid placement) — `sdk/src/marketplace.rs` (17 tests)
- [x] SPEC-018: Marketplace specification — `spec/marketplace.md`

### P1 — Next
- [x] CHAIN-028: Auction mechanism (Dutch auction for premium model slots) — `chain/src/auction.rs` (18 tests)
- [x] NODE-027: Marketplace event indexer (listing/bid/match events) — `node/src/marketplace_indexer.rs` (19 tests)
- [x] DOCS-006: Marketplace integration guide — `docs/marketplace-integration-guide.md`

## Phase 11 — Data Availability & Blob Storage

### P0 — Active
- [x] CHAIN-029: Data availability sampling (DAS) engine (erasure coding, Merkle proofs, challenge/response, penalties) — `chain/src/das.rs` (15 tests)
- [x] NODE-028: DAS validator (automatic sampling, proof requests over P2P) — `node/src/das_validator.rs` (15 tests)
- [ ] SPEC-019: Data availability specification — `spec/data-availability.md`
- [ ] CHAIN-030: Blob transaction type (submit blob data, reference from inference commits) — `chain/src/blob_tx.rs`

### P1 — Next
- [ ] NODE-029: Blob storage backend (chunked storage, GC, disk quotas) — `node/src/blob_store.rs`
- [ ] SDK-009: Blob upload client (erasure encode + submit) — `sdk/src/blob_client.rs`
- [ ] INT-004: DAS adversarial test (withholding attacks, partial responses) — `chain/src/das_adversarial_test.rs`

## Assignment History
| Task | Assignee | Status | Started | Completed |
|------|----------|--------|---------|-----------|
| REPO-001 | Capri | ✅ | 2026-03-04 00:05 | 00:05 |
| SPEC-001 | Capri | ✅ | 2026-03-04 00:05 | 00:30 |
| SPEC-002 | Capri | ✅ | 00:30 | 00:45 |
| SPEC-003 | Capri | ✅ | 00:45 | 01:00 |
| SPEC-004 | Capri | ✅ | 02:50 | 02:55 |
| PROTO-001 | Capri | ✅ | 01:00 | 01:15 |
| NODE-001 | Capri | ✅ | 01:15 | 01:40 |
| CHAIN-001 | Capri | ✅ | 02:42 | 02:48 |
| CHAIN-002 | Capri | ✅ | 02:50 | 02:56 |
| CHAIN-003 | Capri | ✅ | 02:42 | 02:48 |
| NODE-003 | Capri | ✅ | 02:48 | 02:50 |
| NODE-004 | Capri | ✅ | 02:50 | 02:56 |
| SPEC-005 | Capri | ✅ | 03:34 | 03:40 |
| EXP-001 | Koda | 🔄 | 00:05 | — |
| SPEC-006 | Capri | ✅ | 02:42 | 02:48 |
| CHAIN-004 | Capri | ✅ | 02:42 | 02:48 |
| NODE-002 | Capri | ✅ | 03:39 | 03:45 |
| DOCS-001 | Kestrel | 🔄 | 03:28 | — |
| EXP-002 | Capri | ✅ | 03:49 | 03:55 |
| DOCS-002 | Capri | ✅ | 04:04 | 04:10 |
| NODE-007 | Capri | ✅ | 04:09 | 04:15 |
| CHAIN-008 | Capri | ✅ | 04:14 | 04:20 |
| NODE-009 | Capri | ✅ | 04:19 | 04:25 |
| INT-001 | Capri | ✅ | 04:24 | 04:30 |
| CHAIN-009 | Capri | ✅ | 04:29 | 04:35 |
| SPEC-010 | Capri | ✅ | 04:35 | 04:38 |
| CHAIN-010 | Capri | ✅ | 04:34 | 04:40 |
| CHAIN-011 | Capri | ✅ | 04:39 | 04:45 |
| SPEC-011 | Capri | ✅ | 04:59 | 05:05 |
| INT-002 | Capri | ✅ | 04:59 | 05:05 |
| CHAIN-012 | Capri | ✅ | 05:05 | 05:10 |
| NODE-011 | Capri | ✅ | 05:09 | 05:15 |
| NODE-012 | Capri | ✅ | 05:09 | 05:15 |
| SPEC-012 | Capri | ✅ | 05:09 | 05:15 |
| CHAIN-013 | Capri | ✅ | 05:19 | 05:25 |
| SPEC-013 | Capri | ✅ | 05:49 | 05:55 |
| CHAIN-019 | Capri | ✅ | 06:59 | 07:05 |
| CHAIN-027 | Capri | ✅ | 08:44 | 08:50 |
