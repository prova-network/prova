# SPEC: Security Audit Checklist — Prova v2

**Status:** Draft v1 (post-pivot)
**Updated:** 2026-04-24
**Companion:** [`security-threat-model.md`](./security-threat-model.md)

## 1. Purpose

Pre-audit checklist for external security review. Each item maps to a
module and a verification method. Auditors should verify every MUST
item before mainnet deployment; SHOULD items are recommended.

## 2. Contracts — `contracts/src/`

### 2.1 `StorageMarketplace`

| ID | Check | Module | Severity | Verification |
|----|-------|--------|----------|--------------|
| MKT-01 | Escrow funds cannot be released more than once per deal | `StorageMarketplace.sol` | Critical | Unit test double-release; assert second call reverts |
| MKT-02 | `completeDeal` pays exactly `totalPayment - paidOut` with protocol fee | `StorageMarketplace.sol` | Critical | Full-flow test with streaming release + completion |
| MKT-03 | `faultDeal` slashes prover exactly once and refunds only unreleased escrow | `StorageMarketplace.sol` | Critical | Exercise fault at various `paidOut` levels |
| MKT-04 | Only `proofVerifier` address can call listener hooks | `StorageMarketplace.sol` | High | Call hooks from arbitrary EOA; assert revert |
| MKT-05 | Reentrancy guard on every `external payable` and on `proposeDeal` / `cancelProposedDeal` / `completeDeal` / `faultDeal` | `StorageMarketplace.sol` | Critical | Malicious token `transferFrom` callback; assert no re-entry |
| MKT-06 | Deal state machine cannot regress (Active → Proposed, Completed → Active, etc.) | `StorageMarketplace.sol` | High | Fuzz sequence of transitions |
| MKT-07 | Cancelled-before-accepted returns full escrow to client | `StorageMarketplace.sol` | High | Propose + cancel; assert client balance restored |
| MKT-08 | Protocol fee bps cap is enforced (≤ 10%) | `StorageMarketplace.sol` | Medium | Try `setProtocolFeeBps(1001)`; assert revert |

### 2.2 `ProverStaking`

| ID | Check | Module | Severity | Verification |
|----|-------|--------|----------|--------------|
| STK-01 | Unbonding period enforced; withdraw before `unbondingEndsAt` reverts | `ProverStaking.sol` | Critical | `stake` → `requestUnstake` → warp 13d → `withdraw` reverts; warp 1d more → succeeds |
| STK-02 | Slashing reduces stake but not beyond the staked amount | `ProverStaking.sol` | Critical | Slash attempts exceeding stake; assert amount clamped |
| STK-03 | Only authorized controllers can `commitBytes` / `releaseBytes` / `slash` | `ProverStaking.sol` | Critical | Call from unauthorized EOA; assert revert |
| STK-04 | Slashed amounts accumulate in `slashedPool`; withdrawal only by owner | `ProverStaking.sol` | High | Slash → owner withdraw; non-owner withdraw reverts |
| STK-05 | `requestUnstake` blocks dropping below `minStakeFor(committedBytes)` | `ProverStaking.sol` | High | Commit bytes → attempt partial unstake; assert floor held |

### 2.3 `ProofVerifier` (forked from FilOzone/pdp, adapted)

| ID | Check | Module | Severity | Verification |
|----|-------|--------|----------|--------------|
| PVR-01 | UUPS upgrade path requires timelock + prior announcement | `ProofVerifier.sol` | Critical | Attempt upgrade without `announcePlannedUpgrade`; assert revert |
| PVR-02 | `createDataSet` requires exactly `sybilFee()` in `msg.value`; excess refunded | `ProofVerifier.sol` | High | Test with insufficient + over-paid calls |
| PVR-03 | `provePossession` rejects proofs for non-live data sets | `ProofVerifier.sol` | Critical | Submit proof for deleted set; assert revert |
| PVR-04 | Merkle proof verification matches canonical (cross-check against `prover/pkg/pdptree`) | `Proofs.sol`, `ProofVerifier.sol` | Critical | Reference vectors from `go-fil-commp-hashhash` |
| PVR-05 | `findPieceIds` returns correct `(pieceId, offset)` for arbitrary leaf indices | `ProofVerifier.sol` | High | Property-based test across data set sizes |
| PVR-06 | Sybil-fee burn transfers to 0x...dEaD; balance changes verified | `ProofVerifier.sol` | Medium | Observe burn address delta after a call |

### 2.4 `ProvaToken`

| ID | Check | Module | Severity | Verification |
|----|-------|--------|----------|--------------|
| TOK-01 | Total supply is 1B PROVA, minted once at genesis, never again | `ProvaToken.sol` | Critical | Inspect constructor; assert no `_mint` elsewhere |
| TOK-02 | ERC-20 Permit (EIP-2612) signatures validate correctly | `ProvaToken.sol` | High | Positive + replay + wrong-signer test |
| TOK-03 | Burn reduces total supply and affects only burner's balance | `ProvaToken.sol` | Medium | `ERC20Burnable` conformance test |

### 2.5 `ProverRegistry` + `ContentRegistry`

| ID | Check | Module | Severity | Verification |
|----|-------|--------|----------|--------------|
| REG-01 | Only the content owner can `bindENS` or `unbindENS` | `ContentRegistry.sol` | High | Call as non-owner; assert revert |
| REG-02 | Prover deregistration is soft (historical lookups still work) | `ProverRegistry.sol` | Low | Deregister + read; active=false but record present |
| REG-03 | Feature bitmap must include `FEATURE_PDP` on register | `ProverRegistry.sol` | Medium | Register without PDP bit; assert revert |

## 3. Prover — `prover/pkg/`

| ID | Check | Module | Severity | Verification |
|----|-------|--------|----------|--------------|
| PRV-01 | `ValidateSourceURL` rejects http, private/loopback/link-local IPs, userinfo | `pkg/deal/fetch.go` | Critical | Table-driven test (already present) |
| PRV-02 | `Fetcher.MaxBytes` hard-caps downloads | `pkg/deal/fetch.go` | High | Set small cap; attempt larger fetch; assert error |
| PRV-03 | Engine marks deal `Failed` on CommP mismatch without ever writing the piece to store | `pkg/deal/engine.go` | Critical | Feed wrong content; assert store unchanged + deal failed |
| PRV-04 | `OnChainAccepter` parses the `DataSetCreated` event correctly | `pkg/deal/accepter.go` | High | Mock receipt with various log topologies |
| PRV-05 | `EventPoller` watermark advances only after all 5 filters succeed | `pkg/deal/events.go` | High | Inject filter error; assert watermark unchanged |
| PRV-06 | PDP tree root reconstruction matches canonical CommP on fixture sizes | `pkg/pdptree/memtree.go` | Critical | Existing cross-check against `go-fil-commp-hashhash` |
| PRV-07 | Wallet loader's env precedence is documented and deterministic | `pkg/wallet/wallet.go` | Medium | Unit tests cover all 3 sources |
| PRV-08 | HTTP server access log does not leak secrets (passphrase, private key) | `pkg/httpserver/server.go` | Medium | grep access log over a realistic session |

## 4. Integration

| ID | Check | Module | Severity | Verification |
|----|-------|--------|----------|--------------|
| INT-01 | End-to-end deal flow succeeds on local anvil | `prover/internal/soaktest/` | Critical | `./run.sh` exits 0 with 3 deals Active |
| INT-02 | Graceful shutdown on SIGTERM drains active HTTP connections within `ShutdownTimeout` | `pkg/daemon/daemon.go` | Medium | Start daemon → `kill -TERM` → assert 'stopped cleanly' within 30s |
| INT-03 | Soak test metrics match expected values (pieces stored, bytes stored, deals active) | `prover/internal/soaktest/` | Medium | Assertions in `run.sh` |

## 5. Deployment

| ID | Check | Severity | Notes |
|----|-------|----------|-------|
| DEP-01 | Mainnet `ProofVerifier` proxy owner is a Safe multisig, not an EOA | Critical | Verify on-chain after deploy |
| DEP-02 | `StorageMarketplace.protocolFeeBps` matches governance-approved value | High | Check post-deploy |
| DEP-03 | `MockProofVerifier` is NOT deployed on mainnet | Critical | Use the UUPS-proxied real `ProofVerifier` |
| DEP-04 | Systemd unit uses hardened defaults (`NoNewPrivileges`, `ProtectSystem=strict`, etc.) | Medium | Compare against `prover/deploy/provad.service` |
| DEP-05 | Prover keystore file has mode 0600 and non-empty passphrase | High | Operator verifies before boot |

## 6. Out of scope

PoRep, sealing, TEE attestation, AI inference proofs, cross-chain
bridges. These are not part of Prova v2.
