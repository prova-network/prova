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

## Phase D — Challenge handling

- [ ] `pkg/challenges/`
  - [ ] Subscribe to challenge events from `ProofVerifier` for our active deals
  - [ ] For each challenge: load the piece, navigate the Merkle tree to the challenged leaf, build inclusion proof
  - [ ] Submit proof via `ProofVerifier.provePossession(...)`
  - [ ] Handle failures: retry, log, alert
- [ ] Transplant the Merkle tree + proof logic from Curio (`curio/pdp/` uses `go-fil-commcid` + custom tree walkers)

## Phase E — HTTPS retrieval endpoint

- [ ] `pkg/http/`
  - [ ] TLS termination (ACME or static cert)
  - [ ] `GET /piece/{commp}` — stream piece to client
  - [ ] Optional range requests
  - [ ] Bandwidth accounting for payment
  - [ ] Rate limiting
- [ ] Transplant `curio/pdp/handlers.go` upload handler, `handlers_upload.go`

## Phase F — Production polish

- [ ] Metrics (Prometheus): proofs submitted, proofs failed, bytes stored, deals active
- [ ] Graceful shutdown (SIGTERM handling)
- [ ] Health endpoint for orchestrators
- [ ] Structured logging (slog)
- [ ] Systemd unit file + Docker image

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
