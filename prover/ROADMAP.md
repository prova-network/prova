# Prover Implementation Roadmap

Tracking work on the Go prover daemon. Each task links to a matching issue or PR when one exists.

## Phase A — Scaffold ✅

- [x] `go.mod` + `go.sum`, Go 1.25 via toolchain
- [x] Directory layout
- [x] `pkg/piece/` transplanted from `curio/pdp/piece_cid.go`, tests pass
- [x] `pkg/config/` with TOML loader + defaults + validation
- [x] `pkg/store/` disk-backed blob store
- [x] `cmd/provad/` entry point with `version`, `start`, `register`, `status`, `help` subcommands (stubbed)
- [x] Example TOML config
- [x] ATTRIBUTION.md + README.md

## Phase B — Ethereum client wiring ✅

- [x] `pkg/ethclient/` — wrapper over `go-ethereum/ethclient` with chain-ID verification at dial, receipt polling, balance/block helpers, known chain-ID constants
- [x] `pkg/contracts/` — `scripts/gen-bindings.sh` generates all 6 contract bindings via abigen, each in its own sub-package to avoid struct-name collisions
- [x] `pkg/wallet/` — `LoadHex` / `LoadKeystore` (+ `$PROVA_KEYSTORE_PASSPHRASE`) / `LoadFromEnv` (`$PROVA_PRIVATE_KEY`). 7 tests pass.
- [x] `cmd/provad/status` — calls `ProverRegistry.getProver()` and `ProverStaking.getStake()`, prints live block + balance + registration + stake
- [x] `contracts/script/Deploy.s.sol` — foundry deploy script for the 5 non-proxy contracts (ProofVerifier UUPS proxy deferred)
- [ ] `cmd/provad/register` — read-only for now; tx submission deliberately deferred so no accidental registration

## Phase C — Deal lifecycle ✅ (memory-backed; SQLite later)

- [x] `pkg/deal/state.go` — Deal struct + Status state machine (Proposed → Downloading → Verifying → Accepting → Active; terminal Completed/Cancelled/Slashed/Failed)
- [x] `pkg/deal/store.go` — Store interface + MemStore impl with defensive copies and last-seen block watermark
- [x] `pkg/deal/fetch.go` — Fetcher with `ValidateSourceURL` (rejects http, loopback, private IPs, userinfo); `$PROVA_PULL_ALLOW_INSECURE=1` relaxes for dev
- [x] `pkg/deal/engine.go` — Tick-based engine advances deals one step per Tick; doDownload fetches + computes CommP + commits to piece store + compares hash; doVerify confirms piece on-disk; doAccept submits via Accepter interface; MarkActive/Cancelled/Completed/Slashed for external chain events
- [x] `pkg/deal/events.go` — `EventPoller` polls `FilterDealProposed` with prover-address filter, idempotent per watermark, pointer-based `BlockLookback` so 0 is a valid value
- [x] **Validated against live anvil:** deployed contracts, proposed deal from client EOA, ran EventPoller, ingested into engine. Deal 1 landed with status=proposed, commP matched.
- [ ] SQLite-backed Store for crash resilience (deferred; MemStore sufficient for Phase D)
- [ ] Transplant curio/pdp/handlers_pull.go pull-from-peer logic (deferred; HTTPS-only is enough for v1)

## Phase D — Challenge handling ✅ (orchestration done; Merkle builder stubbed for Phase D.2)

- [x] `pkg/challenges/challenge.go` — `ChallengeIndex(seed, dataSetID, proofIndex, totalLeaves)` matches Filecoin PDP bit-for-bit (cross-validated against Curio's `generateChallengeIndex` on 6 test vectors). `ChallengeIndices` batch variant, `pad32Left`, `Proof` struct matching on-chain `IPDPTypes.Proof`.
- [x] `pkg/challenges/submit.go` — `ChainClient` interface (`GetRandomness`, `GetChallengeRange`, `GetNextChallengeEpoch`, `SubmitProof`) + `OnChainClient` wrapping our `proofverifier.ProofVerifier` bindings.
- [x] `pkg/challenges/runner.go` — `Runner.ProveSet(ctx, dataSetID)` orchestrates read-challenge → compute-indices → lookup-pieces → build-proofs → submit-tx.
- [x] Interfaces: `PieceLookup` (maps leaf → piece+offset via on-chain `findPieceIds`) and `MerkleBuilder` (builds inclusion proof from piece bytes) keep Merkle tree implementation pluggable.
- [x] Tests: 17 in this package. Challenge determinism, distribution, bounds, dataset isolation, error propagation, runner orchestration.
- [ ] **Phase D.2** (deferred): Merkle tree builder (`pkg/pdptree/` planned) — load piece, pad to next power-of-two, construct SHA2-254-trunc254-padded binary tree, navigate to challenged leaf. Can reuse `curio/lib/proof/merkle_sha254_memtree.go` (~400 LOC, MIT).
- [ ] Challenge-event poller (subscribes to `NextProvingPeriod` events, triggers Runner per data set, handles retry/backoff).

## Phase E — HTTPS retrieval endpoint

- [ ] `pkg/http/`
  - [ ] TLS termination (ACME or static cert)
  - [ ] `GET /piece/{commp}` — stream piece to client
  - [ ] Optional range requests
  - [ ] Bandwidth accounting for payment
  - [ ] Rate limiting
- [ ] Transplant `curio/pdp/handlers.go` upload handler, `handlers_upload.go`

## Phase F — Daemon main loop ✅

- [x] `pkg/daemon/` — `Daemon` struct supervising three concurrent loops:
  poll loop (fetches new events), tick loop (advances deals), status loop
  (aggregate logging every 60s). Single-goroutine-mutates-store discipline.
- [x] `cmdStart` wired: load env → bind contracts → build engine + poller → run daemon
- [x] Structured logging (slog) from day one
- [x] Graceful shutdown on SIGINT/SIGTERM with bounded drain timeout
- [x] Validated end-to-end against live anvil:
  * `provad start` boots, logs version + config
  * Status loop emits periodic summary
  * Poller detected real DealProposed event once past lookback window
  * Engine ingested deal, advanced on tick, reported `deal has no source URL`
    (expected — v1 event has no SourceURL; clients supply out-of-band)
  * SIGTERM triggers clean shutdown with uptime logged

### Production polish items (still pending; moved to Phase F.2)

- [ ] Prometheus metrics (proofs submitted/failed, bytes stored, deals active)
- [ ] Health endpoint for orchestrators
- [ ] Systemd unit file + Docker image
- [ ] Configurable `BlockLookback` in TOML (currently hardcoded default 6)
- [ ] SourceURL transport: extraData in DealProposed, or off-chain hint

## Phase G — Transplant audit

Once Phases A-F are done, go back and audit every file transplanted from Curio:
- Does every file carry SPDX attribution?
- Are any Filecoin-specific assumptions still leaking through (FVM, Lotus RPC, harmonydb, harmonytask)?
- Are error messages generic enough that they don't reveal upstream provenance?
- Is ATTRIBUTION.md accurate?

## Phase H — Integration testing

- [ ] `internal/testutil/` — anvil spawner, deployed contract fixtures
- [ ] End-to-end test: spawn anvil, deploy contracts, run prover, create a deal, watch it complete
- [ ] Chaos tests: kill prover mid-deal, restart, verify state recovery
- [ ] Long-running soak test on Base Sepolia (pending Phase 8 gate lift)

---

## Out of scope for v1

- libp2p networking (Curio uses it; we don't need it for v1 HTTPS-only model)
- Multi-miner support (Curio runs multiple miners on one instance; Prova does not)
- FVM/Filecoin chain interaction (Base is our chain)
- TEE-attested fast path (specified but not implemented)
- QBP / AI inference proofs (deferred to v2)
- ZK aggregation (deferred to v2)
