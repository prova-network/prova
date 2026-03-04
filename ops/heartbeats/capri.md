# Capri Heartbeats — Prova Build Sprint

## HB 2026-03-04T02:38Z
- Built: CHAIN-001 (commit+dispute+simulation), CHAIN-002 (stake), NODE-003 (runner), NODE-004 (participant)
- Commits: `6652712`, `b5c7248`, `731ffa0`, `12fae24`

## HB 2026-03-04T02:56Z
- Built: SPEC-004 (PDP), SPEC-005 (audit), SPEC-006 (payments), CHAIN-004 (payments), CHAIN-005 (epoch), DOCS-001 (quickstart+arch)
- Commits: `1d2ebf8`, `b6641a2`, `418d5dd`

## HB 2026-03-04T03:38Z
- Built: SPEC-007 (network), CI-001 (GitHub Actions), clippy fixes across workspace
- EXP-001 running on Blackwell (TinyLlama 5×50 tokens, waiting)
- Koda unresponsive via HTTP (timeout), Kestrel session stale (last active Feb 21)
- 70 tests passing, 11 commits pushed since sprint start
- Next: Wait for EXP-001 results, then NODE-005 (real llama.cpp integration)
- 2026-03-04 03:45 CET — NODE-002: PDP proof engine scaffold (11 tests). Merkle tree, inclusion proofs, challenge derivation, proof generation/verification, miss tracking. Also synced backlog for SPEC-006, CHAIN-004, CI-001, CHAIN-005, SPEC-007.
## 2026-03-04 03:44 CET — NODE-005
Built LlamaCppRunner: real llama.cpp activation capture integration with dump file parsing, dtype handling, gap detection, Merkle tree integration. 8 new tests (39 total).
2026-03-04 02:52 UTC — EXP-002: Built INT8 cross-arch determinism harness (GEMM sim, 5 archs, Merkle integration). 12 tests, committed 1147674.
## 2026-03-04 03:54 — EXP-003: CPU canonical verification path
Fixed-point Q16.16 GEMM, CanonicalVerifier for dispute adjudication, 18 tests passing. Promoted from P1→P0.
2026-03-04 03:59 CET — NODE-006: P2P networking scaffold (Kademlia DHT + gossipsub router + NetworkNode). 13 tests passing.
## 2026-03-04 04:10 CET — Capri
DOCS-002: Completed architecture overview — added streaming payments flow, audit protocol lifecycle, and module dependency graph. Committed 118290b.

**2026-03-04T04:15Z** — NODE-007: JSON-RPC 2.0 API scaffold (8 methods, 16 tests). All P0/P1/P2 unassigned tasks complete; created new task from gap analysis.
## 2026-03-04 04:20 CET — CHAIN-008: Transaction mempool
Built `chain/src/mempool.rs` — priority ordering, nonce tracking, fee-based eviction, replacement, expiry. 16 tests passing. Phase 2 backlog created.
## 2026-03-04 04:25 CET — NODE-009
Built CLI scaffold with hand-rolled arg parser (run/status/account/tx subcommands), 22 tests, zero new deps.
- 2026-03-04T04:30+01:00 — INT-001: Multi-node integration test harness (12 tests). ProvaNode + MultiNodeHarness with gossip propagation, commit/dispute/payment E2E across N nodes.
2026-03-04T04:35+01:00 — CHAIN-009: State trie with balances, nonces, storage slots, Merkle root, pruning, snapshots. 16 tests passing.
2026-03-04T04:38+01:00 — SPEC-010: Token economics spec (issuance halving, staking params, fee model, security analysis).
- 2026-03-04 04:40 CET — CHAIN-010: Reward distribution engine (block rewards w/ halving, inference fee splits, storage subsidies, challenger bounties, claim system). 15 tests passing.
## 2026-03-04 04:45 CET — CHAIN-011: Transaction execution engine
Built tx executor with 7 tx types (transfer, stake, unstake, register-model, inference-commit, claim-reward, pay-inference-fee), gas metering, nonce enforcement, atomic rollback, batch execution. 17 tests.
## 2026-03-04 04:44 CET — NODE-010: Wallet + key management
Built Ed25519 signing, encrypted keystore, keyring, signed transactions. 16 tests passing.

## 2026-03-04 05:10 CET
SPEC-011 + INT-002: Fixed full-chain integration tests (5 tests, matched real module APIs) + governance spec verified. CHAIN-012: EIP-1559 fee market with dynamic base fee, surge pricing, calldata metering (19 tests). Total: 192 passing.
2026-03-04 05:15 CET — NODE-011: Persistent storage backend (sled). 6 column families, atomic commit_block(), persistence test. 15 tests passing.
2026-03-04 04:16 UTC — NODE-012: Chain sync protocol (header-first sync, peer scoring, body verification, fork choice). 14 tests passing.
2026-03-04 04:17 UTC — SPEC-012: Light client specification (header chain, state proofs, finality tracking, 3 sync modes).
2026-03-04 04:21 UTC — CHAIN-013: Job scheduler (16 tests). Inference request routing with stake×reputation scoring, deadline expiry, timeout eviction. Phase 4 started.
2026-03-04 05:24 — NODE-013: Job executor with worker loop, retry queue, graceful shutdown, InferenceBackend trait. 11 tests, committed 7b22b16.
- 2026-03-04 05:29 — CHAIN-014: SLA enforcement with quadratic penalty curves, 3 tiers, 4 violation types, slashing integration (14 tests)
2026-03-04T05:34 | NODE-014: Metrics & telemetry — Counter/Gauge/Histogram/Timer/Registry + NodeMetrics presets, Prometheus text exposition, 15 tests passing
2026-03-04 04:41 UTC — INT-003: Adversarial scheduler tests (19 tests: byzantine drops, deadline gaming, sybil, griefing, capacity hoarding, SLA cascade, price manipulation)
2026-03-04 04:43 UTC — CHAIN-015: Reputation system with EMA scoring, decay curves, suspension/recovery, slash multiplier (21 tests)
2026-03-04T05:44 CET — NODE-015: Provider auto-pricing (EMA market tracking, 4 strategies, utilization pressure, 17 tests)
## 2026-03-04 05:49 CET
SPEC-013: Security threat model — 21 threats (consensus, QBP, economics, P2P, storage, ops), full severity matrix, mitigation mapping to existing modules
2026-03-04 05:54 — CHAIN-016: Filecoin checkpoint anchoring (16 tests). Quorum voting, L1 anchor simulation, light client state verification.
- **2026-03-04 05:59** — CHAIN-017: Cross-chain bridge message format. Outbox/Inbox with Merkle state proofs, 6 payload types, nonce ordering, TTL expiry, replay protection. 20 tests.
## 2026-03-04 06:04 UTC+1
NODE-016: Checkpoint submitter (15 tests), SPEC-014: Checkpoint anchoring spec, CHAIN-018: Finality gadget (18 tests)
- 2026-03-04 06:14 CET | NODE-017: L1 event watcher — finality tracking, reorg detection, 5 event types, 14 tests
## 2026-03-04 06:19 CET — SPEC-015: Bridge security specification
10 threat vectors (forged proofs, checkpoint forgery, replay, censorship, bridge drain, L1 reorg, validator rotation), economic security bounds, audit checklist. Committed 54aa83d.
2026-03-04 06:24 — SDK-001: Client SDK with request builder, signing, provider discovery, batch ops (19 tests). New Phase 6 started.
## 2026-03-04 06:29 — SDK-002: JSON-RPC client
Built RpcClient<T: Transport> with pluggable transport, typed methods for all prova_* RPCs, polling, batch ops. 15 tests passing.
2026-03-04 06:34 CET — SDK-003: CLI wallet integration (import keys, sign offline, keystore manager) — 22 tests, committed eaeed7e
- **2026-03-04 06:39 CET** — SDK-004: Built @prova/sdk TypeScript package (pure-JS SHA-256, keypair, request builder, provider discovery, client). 27 tests passing, pushed a599588.
2026-03-04T06:44 CET — SDK-005: Rate limiting & retry logic (RetryPolicy, RateLimiter, ResilientTransport) — 17 tests passing
2026-03-04 06:49 CET — DOCS-003: SDK usage guide & examples (docs/sdk-guide.md). Covers all 5 SDK modules: client, RPC, wallet, retry, WASM. All non-assigned tasks complete.
- 2026-03-04 06:54 CET — NODE-018: Configuration manager (TOML parsing, 7 config sections, defaults, validation, env overrides, roundtrip serialization) — 24 tests
## 2026-03-04 06:59 UTC+1 — Capri
CHAIN-019: State snapshot system — chunked export/import with SHA-256 integrity verification, streaming importer, full restore. 20 tests. Phase 7 (Testnet Readiness) created.
2026-03-04 07:04 CET — NODE-019: Snapshot serving over P2P (SnapshotServer, SnapshotDownload, RateLimiter, manifest verification, 19 tests)
2026-03-04 07:09 — CHAIN-020: State pruning module (retain N snapshots, block GC, checkpoint protection, archive mode). 13 tests passing.

2026-03-04 07:14 — NODE-020: Fast sync mode (multi-peer parallel chunk download, peer scoring, recovery). 14 tests, 831 lines.
## 2026-03-04 07:19 UTC+1 — OPS-002
Testnet genesis config (genesis.toml, bootnodes.toml) + loader with 16 validation tests. 5 geo-distributed boot nodes, 8 allocations summing to 1T supply, 2 pre-registered models. Committed 199cc6b.
2026-03-04 06:26 UTC — CHAIN-021: Protocol upgrade mechanism (fork scheduling, version negotiation, stake-weighted signaling, emergency activation). 17 tests.
2026-03-04T07:29Z — NODE-021: Graceful shutdown + state persistence. ShutdownCoordinator with priority draining, checkpoint serialize/deserialize, signal handling. 18 tests passing.
- 2026-03-04 07:34 CET — DOCS-004: Testnet operator guide (build/configure/run/monitor/troubleshoot, 295 lines). All backlog tasks complete except Koda (EXP-001) and Kestrel (DOCS-001, SPEC-009).
2026-03-04 07:41 CET — CHAIN-022: Event log system (emit, Merkle receipt roots, multi-index queries). 15 tests. Koda healthy — sentinel-rs + determinism harness work ongoing.
2026-03-04 07:44 CET — NODE-022: Event subscription engine (client connect/disconnect, filter-based fanout, backpressure, replay, batch notify, keepalive, expiry). 16 tests passing.
2026-03-04 07:49 CET — CHAIN-023: Receipt storage with Merkle proof-of-inclusion. TxReceipt, BlockReceiptRecord, ReceiptStore, MerkleProof generation/verification, pruning. 16 tests.
## 2026-03-04 07:54 CET — SDK-006: Event subscription client
Built EventClient with realtime/replay/historical modes, EventCache with LRU eviction, positional topic filtering, backpressure, disconnect handling. 14 tests passing.

## 2026-03-04 07:59 CET
NODE-023: Block explorer API — indexed store, paginated block/tx/event/account queries, event filtering, chain stats. 19 tests.
## 2026-03-04 08:04 — SPEC-016 + SDK-007
Built event schema spec (27 canonical signatures, ABI encoding, versioning) + historical event replay engine with LRU cache, batched fetch, resumable replay (16 tests).
- 2026-03-04 08:09 CET — CHAIN-024: Built access control layer (role-based capabilities, pause/unpause, expiry, overrides). 17 tests. Phase 9 opened.
2026-03-04 08:14 CET — CHAIN-025: Rate limiter with token bucket + sliding window, stake-adaptive limits, cooldown penalties, exempt kinds. 15 tests passing.
2026-03-04 07:21 UTC — NODE-024: TLS transport layer (self-signed certs, mTLS handshake, pinning, revocation, rotation) — 16 tests passing
## 2026-03-04 08:24 CET
SPEC-017: Security audit checklist — 73 checks across 8 categories, coverage targets, fuzz roadmap, engagement plan.
2026-03-04 08:29 — CHAIN-026: Formal invariant checker (5 invariants, 16 tests). Balance conservation, stake consistency, nonce monotonicity, reward conservation, job uniqueness.
## 2026-03-04 08:34 CET
NODE-025: Fuzz testing harness — 12 property-based tests (no-panic, nonce monotonicity, replay protection, deterministic roots, stress). Commit dc8d3a8.
2026-03-04 08:39 CET — DOCS-005: Security model documentation (docs/security-model.md). Unified 10KB reference: trust model, consensus, QBP, economics, bridge, RBAC, audit readiness. All P0/P1 tasks complete except Koda (EXP-001) and Kestrel (DOCS-001, SPEC-009).
## 2026-03-04 08:50 — CHAIN-027: Model marketplace
Built listing/bidding/discovery system with stake requirements, bid matching, fee collection, capacity tracking, and filtered provider discovery. 17 tests passing.
2026-03-04T08:49 CET | NODE-026: Marketplace CLI — 7 subcommands (list/bid/discover/show/create/deactivate/my-listings), parser + formatters, 37 tests
## 2026-03-04 08:54 CET
SDK-008: Marketplace client SDK (17 tests) + SPEC-018: Marketplace specification. Phase 10 P0 complete.
- 2026-03-04 08:59 CET — CHAIN-028: Dutch auction for premium model slots (18 tests). Linear price decay, anti-snipe guard, revenue tracking.
## 2026-03-04 09:04 — NODE-027: Marketplace event indexer
Promoted from P1, built materialized views for listing/bid/match/auction events with 19 tests. Cursor pagination, price history, multi-model isolation.
2026-03-04 09:09 — DOCS-006: Marketplace integration guide (client/provider quickstart, auction, events, indexer, security)
## 2026-03-04 09:14 CET — CHAIN-029: DAS engine
Built data availability sampling module with erasure coding, Merkle proofs, multi-round challenge/response, and penalty enforcement. 15 tests, Phase 11 opened.
2026-03-04 09:19 — NODE-028: DAS validator — automatic sampling, P2P proof requests, provider reliability stats, retry/failure handling. 15 tests.
2026-03-04 09:24 — SPEC-019 + CHAIN-030: DA spec (erasure coding, DAS protocol, blob fee market, security) + blob transaction engine with EIP-1559 fees, pruning, DAS integration (17 tests)
2026-03-04 08:31 UTC — NODE-029: Blob storage backend (chunked storage, GC, disk quotas, LRU eviction, integrity checks, pin/unpin). 17 tests.
2026-03-04 09:34 CET — SDK-009: Blob upload client with erasure encoding, batch uploads, progress tracking, fee estimation. 19 tests passing.
Promoted from P1→P0, all Phase 11 P1 tasks except INT-004 now complete.
2026-03-04 09:39 CET — INT-004: DAS adversarial test suite (19 tests). Withholding, corruption, replay, multi-provider penalty stacking.

2026-03-04 09:44 CET — CHAIN-031: Delegation system (18 tests). Delegate/undelegate/redelegate, commission, slash propagation, auto-compound. Fixed blob_client doctest. Total: 1165 tests, 0 failures.
2026-03-04 08:51 UTC — CHAIN-032: Liquid staking tokens (stPROVA mint/burn/transfer/rewards/slash, exchange rate appreciation) — 15 tests, committed e76f912
2026-03-04 09:54 CET — NODE-030: Built delegation CLI (delegate/undelegate/redelegate/rewards/list/providers + stPROVA integration). 31 tests, committed dd3aaac.
- 2026-03-04 09:59 CET — SDK-010: Delegation client SDK (delegate/undelegate/redelegate/rewards/portfolio/auto-compound). 16 tests, committed a0b4ddc.
2026-03-04 10:04 CET — SPEC-020 + CHAIN-033: Delegation & staking spec (9.5KB, full lifecycle/security/gas docs) + delegation governance voting with delegator override (16 tests)
