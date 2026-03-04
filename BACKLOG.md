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

**Total: 220 passing tests, 2 external deps (sha2, serde). 25 source files, ~10,069 lines of Rust.**

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
- [x] NODE-007: JSON-RPC 2.0 API scaffold — `node/src/rpc.rs` (16 tests)
- [x] NODE-006: P2P networking scaffold — `node/src/network.rs` (15 tests)
- [x] CHAIN-006: Block production + consensus — `chain/src/block.rs` (37 tests)
- [x] CHAIN-007: Genesis state — `chain/src/genesis.rs` (14 tests)
- [ ] SPEC-009: Consensus specification [KESTREL]
- [x] OPS-001: Devnet simulation — `node/src/devnet.rs` (5 tests)
- [x] DOCS-002: Architecture overview diagram — `docs/architecture.md` (payments, audit, dependency graph added)

## Phase 2 — Integration & Hardening

### P0 — Active
- [x] CHAIN-008: Transaction mempool (priority, nonce, eviction) — `chain/src/mempool.rs` (16 tests)
- [x] NODE-009: CLI scaffold (subcommands: run, status, account, tx) — `node/src/cli.rs` (22 tests)
- [ ] INT-001: Multi-node integration test harness
- [ ] CHAIN-009: State trie (account balances + nonce tracking)
- [ ] SPEC-010: Token economics specification

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
