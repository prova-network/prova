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

**Total: 56 passing tests, 0 external dependencies**

### P0 — In Progress
- [ ] EXP-001: Determinism harness on Blackwell [KODA] — Blackwell confirmed ready, harness not yet run
- [x] SPEC-005: Audit protocol spec + implementation — `spec/audit-protocol.md`, `chain/src/audit.rs` (10 tests)
- [ ] SPEC-006: Streaming payments spec
- [ ] DOCS-001: Developer quickstart guide [KESTREL]
- [ ] CHAIN-004: Payment channel implementation
- [ ] NODE-002: PDP proof engine scaffold

### P1 — Next
- [ ] EXP-002: TensorRT INT8 cross-architecture determinism test
- [ ] EXP-003: CPU canonical verification path test
- [ ] CI-001: CI pipeline (build + test)
- [ ] NODE-005: Real llama.cpp integration (activation capture via hook)
- [ ] CHAIN-005: Epoch ticker + state transitions
- [ ] SPEC-007: Network protocol (P2P gossip, block propagation)

### P2 — Later
- [ ] NODE-006: P2P networking scaffold
- [ ] CHAIN-006: Block production + consensus
- [ ] CHAIN-007: Genesis state
- [ ] OPS-001: Devnet launch script
- [ ] DOCS-002: Architecture overview diagram

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
| DOCS-001 | Kestrel | 🔄 | 03:28 | — |
