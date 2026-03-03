# Prova Build Backlog

## Phase 1 — Foundation (Active Sprint)

### P0 — In Progress
- [x] REPO-001: Repo scaffolding (/docs, /spec, /proto, /node, /chain, /ops)
- [ ] SPEC-001: QBP formal spec — message types + state machine
- [ ] SPEC-002: Activation Merkle tree format + serialization rules
- [ ] SPEC-003: Model registry schema
- [ ] EXP-001: Determinism harness (fixed prompts, seeds, arch grouping, divergence reporting) [KODA]
- [ ] PROTO-001: QBP protobuf message definitions
- [ ] CHAIN-001: Minimal commit + challenge window flow (mocked chain)
- [ ] NODE-001: Activation Merkle tree builder (reference implementation)

### P1 — Next
- [ ] EXP-002: TensorRT INT8 cross-architecture determinism test
- [ ] EXP-003: CPU canonical verification path test
- [ ] SPEC-004: PDP integration spec (CommP registration, challenge/response)
- [ ] SPEC-005: Audit protocol spec (random sampling, slashing)
- [ ] SPEC-006: Streaming payments spec
- [ ] NODE-002: PDP proof engine scaffold
- [ ] NODE-003: Inference runner (quantized, with activation capture)

### P2 — Later
- [ ] CHAIN-002: Stake ledger + slashing logic
- [ ] CHAIN-003: Model registry on-chain contract
- [ ] CHAIN-004: Payment channel implementation
- [ ] NODE-004: Bisection game participant
- [ ] CI-001: CI pipeline (build + test)
- [ ] DOCS-001: Developer quickstart guide

## Assignment History
| Task | Assignee | Status | Started | Completed |
|------|----------|--------|---------|-----------|
| REPO-001 | Capri | ✅ Done | 2026-03-04 00:05 | 2026-03-04 00:05 |
| SPEC-001 | Capri | 🔄 Active | 2026-03-04 00:05 | — |
| EXP-001 | Koda | 🔄 Active | 2026-03-04 00:05 | — |
