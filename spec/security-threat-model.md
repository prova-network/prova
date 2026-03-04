# Security Threat Model — Prova Network

**Status:** Draft v0.1
**Author:** Capri (for Prova project)
**Date:** 2026-03-04

## 1. Overview

This document enumerates known attack vectors against the Prova network, assesses their severity, and specifies mitigations implemented or planned. It covers consensus, inference verification (QBP), economics, networking, and operational concerns.

### Threat Classification

| Level | Impact | Response Time |
|-------|--------|---------------|
| **Critical** | Network halt, total fund loss, consensus break | Immediate hotfix, emergency governance |
| **High** | Partial fund loss, sustained censorship, proof bypass | Priority fix within 1 epoch cycle |
| **Medium** | Economic inefficiency, temporary DoS, griefing | Scheduled fix |
| **Low** | Information leak, UX degradation | Best-effort |

## 2. Consensus & Block Production

### 2.1 Nothing-at-Stake (THREAT-001)

- **Vector:** Validators sign conflicting blocks at the same height to maximize rewards on all forks
- **Severity:** Critical
- **Mitigation:** Equivocation slashing — any two signed blocks at the same height from the same validator triggers immediate stake seizure (`stake.rs` slashing logic). Slash amount = 100% of staked collateral.
- **Residual Risk:** Requires at least one honest observer to submit equivocation proof within the challenge window.

### 2.2 Long-Range Attack (THREAT-002)

- **Vector:** Attacker acquires old validator keys (from unstaked validators) and rewrites history from a past checkpoint
- **Severity:** High
- **Mitigation:** Weak subjectivity checkpoints — new nodes must sync from a recent trusted checkpoint (< unbonding period). Key deletion after unstaking is recommended but not enforceable.
- **Residual Risk:** Nodes that have been offline longer than the unbonding period must obtain a fresh checkpoint out-of-band.

### 2.3 Validator Censorship (THREAT-003)

- **Vector:** A majority coalition of validators refuses to include specific transactions (e.g., dispute initiations, slashing proofs)
- **Severity:** High
- **Mitigation:** (a) Forced inclusion via proposer rotation — censored txs eventually reach an honest proposer. (b) Mempool gossip ensures transactions propagate to all validators. (c) Governance can forcibly rotate validator sets.
- **Residual Risk:** >67% colluding validators can censor indefinitely within a governance response window.

### 2.4 Block Withholding (THREAT-004)

- **Vector:** Block proposer publishes header but withholds body, preventing validation
- **Severity:** Medium
- **Mitigation:** Timeout-based skip — if block body is not available within `BLOCK_BODY_TIMEOUT` (currently 3 epochs), the slot is skipped and the proposer loses block reward. Repeated withholding triggers reputation penalty.

## 3. Inference Verification (QBP)

### 3.1 Lazy Provider — Skip Computation (THREAT-005)

- **Vector:** Provider submits a random or cached activation root without running inference
- **Severity:** Critical
- **Mitigation:** Any challenger can initiate a bisection dispute. Lazy providers will fail to produce a valid single-layer re-execution proof. Slash = committed stake + challenger bounty from slashed funds.
- **Detection Rate:** Any single honest challenger in the network is sufficient.

### 3.2 Challenger Griefing (THREAT-006)

- **Vector:** Attacker opens many frivolous disputes to drain honest providers' gas and time
- **Severity:** Medium
- **Mitigation:** Challengers must post a dispute bond (currently `MIN_DISPUTE_BOND` = 10× gas cost of full bisection). Bond is slashed if the challenge fails (provider proven correct). Bond returned + bounty if challenge succeeds.
- **Residual Risk:** A well-funded attacker can still impose latency on honest providers, though at escalating cost.

### 3.3 Determinism Evasion (THREAT-007)

- **Vector:** Provider uses a subtly different quantization scheme or GPU driver version to produce outputs that differ by <1 ULP, making disputes ambiguous
- **Severity:** High
- **Mitigation:** (a) Canonical compute capability pinning per model registration (`ComputeCapability` in model manifest). (b) INT8 quantization with controlled accumulation order (row-major, deterministic reduce). (c) CPU canonical verification path as ultimate arbiter (`canonical_cpu.rs`). (d) Tolerance = 0 bits — exact match required at INT8 precision.
- **Residual Risk:** New GPU architectures may introduce unexpected non-determinism. Requires ongoing compatibility testing (see `determinism.rs` harness).

### 3.4 Activation Root Collision (THREAT-008)

- **Vector:** Attacker finds two different activation tensors producing the same Merkle root
- **Severity:** Low (theoretical)
- **Mitigation:** SHA-256 collision resistance (2^128 security). Activation Merkle tree uses domain-separated hashing with layer index prefix.
- **Residual Risk:** Negligible under standard cryptographic assumptions.

### 3.5 Bisection Timeout Manipulation (THREAT-009)

- **Vector:** Dispute participant deliberately delays responses to exhaust opponent's patience or force timeout in their favor
- **Severity:** Medium
- **Mitigation:** Strict per-round timeout (`BISECTION_ROUND_TIMEOUT`). Non-responding party auto-loses the round and forfeits bond. Clock is on-chain (epoch-based), not wall-clock.

## 4. Economic Attacks

### 4.1 Stake Grinding (THREAT-010)

- **Vector:** Attacker stakes minimally across many identities (Sybil) to increase probability of being selected as provider/validator
- **Severity:** High
- **Mitigation:** (a) Provider selection weighted by stake — splitting stake across N identities gives identical aggregate selection probability. (b) Minimum stake threshold (`MIN_PROVIDER_STAKE`) to impose fixed cost per identity. (c) Reputation system requires history, not just stake.
- **Residual Risk:** Reputation bootstrapping phase is vulnerable to Sybil flooding.

### 4.2 Payment Channel Exhaustion (THREAT-011)

- **Vector:** Payer opens payment channel, consumes inference, then attempts to close channel with stale state (before provider claims)
- **Severity:** High
- **Mitigation:** (a) Dispute period on channel closure — provider can submit latest signed state within `CHANNEL_DISPUTE_WINDOW`. (b) Unilateral closure always uses the highest-nonce state seen on-chain. (c) Streaming payment lockup covers worst-case provider exposure.
- **Residual Risk:** Provider must be online during dispute window or delegate to a watchtower.

### 4.3 Gas Price Manipulation (THREAT-012)

- **Vector:** Attacker floods mempool with high-fee transactions to spike the base fee (EIP-1559 style), making disputes economically infeasible for honest challengers
- **Severity:** Medium
- **Mitigation:** (a) Base fee adjustment rate is dampened (12.5% max change per block). (b) Dispute transactions get priority lane — disputes and slashing proofs are exempt from base fee (pay only tip). (c) Challenger bond return includes gas reimbursement on successful challenge.
- **Residual Risk:** Sustained attack can still degrade network throughput for non-dispute transactions.

### 4.4 Reward Gaming via Self-Challenge (THREAT-013)

- **Vector:** Provider challenges their own (correct) inference to claim "challenger bounty" from the system
- **Severity:** Low
- **Mitigation:** Self-challenges are not profitable: the challenger bond is returned (no bounty) when the challenge fails, and the provider already earned the inference fee. If the provider deliberately produces wrong output to win as challenger, they lose their inference stake (net negative).

## 5. Network & P2P

### 5.1 Eclipse Attack (THREAT-014)

- **Vector:** Attacker controls all peer connections of a target node, feeding it a false view of the chain
- **Severity:** High
- **Mitigation:** (a) Minimum peer diversity requirement — nodes must maintain connections to peers in ≥3 distinct /16 IP ranges. (b) Peer scoring and rotation (`network.rs` reputation tracking). (c) Checkpoint verification against known validators.
- **Residual Risk:** Bootstrapping nodes with no prior peer knowledge are most vulnerable. Hardcoded bootstrap nodes provide initial trust anchor.

### 5.2 Transaction Replay (THREAT-015)

- **Vector:** Valid transaction from one context replayed in another (cross-chain, cross-epoch)
- **Severity:** Medium
- **Mitigation:** (a) Chain ID in transaction signature domain. (b) Nonce-based replay protection (`state.rs` nonce tracking). (c) Epoch-bounded validity window for time-sensitive operations (disputes, challenges).

### 5.3 Gossip Amplification DoS (THREAT-016)

- **Vector:** Attacker crafts messages that are cheap to produce but expensive to validate, overwhelming honest nodes
- **Severity:** Medium
- **Mitigation:** (a) Message size limits per topic. (b) Sender rate limiting in gossip protocol. (c) Signature verification before propagation (invalid messages dropped at first hop). (d) Peer banning on repeated invalid messages.

## 6. Storage & PDP

### 6.1 Data Withholding After Proof (THREAT-017)

- **Vector:** Storage provider passes PDP challenge but deletes data afterward (before next challenge)
- **Severity:** Medium
- **Mitigation:** (a) Random challenge timing — provider cannot predict next challenge epoch. (b) Missed proof → fault → progressive slashing. (c) Erasure coding across multiple providers (planned, not yet implemented).
- **Residual Risk:** Single-provider storage has inherent data loss risk between proof windows.

### 6.2 PDP Proof Outsourcing (THREAT-018)

- **Vector:** Provider doesn't store data locally but retrieves it from another source just-in-time for proofs
- **Severity:** Low
- **Mitigation:** Challenge response timeout is calibrated to local-disk latency. Retrieving from remote storage would exceed timeout for large proof sets. This is fundamentally a PDP protocol concern (inherited from Filecoin PDP spec).

## 7. Operational & Implementation

### 7.1 Key Compromise (THREAT-019)

- **Vector:** Validator/provider private key stolen
- **Severity:** Critical
- **Mitigation:** (a) Key rotation support — operators can migrate to new key with old key's signature. (b) Withdrawal delay (unbonding period) gives time to detect and respond. (c) Governance-triggered emergency key freeze.
- **Recommendation:** HSM/enclave key storage for production validators.

### 7.2 State Trie Bloat (THREAT-020)

- **Vector:** Attacker creates millions of minimum-balance accounts to bloat state trie, degrading node performance
- **Severity:** Medium
- **Mitigation:** (a) Account creation fee (gas cost of state expansion). (b) Minimum account balance (`MIN_ACCOUNT_BALANCE`). (c) State rent (planned) — accounts below threshold are pruned after inactivity period.

### 7.3 Dependency Supply Chain (THREAT-021)

- **Vector:** Compromised or malicious crate injected via build dependencies
- **Severity:** High
- **Mitigation:** (a) Minimal dependency surface — only `sha2` and `serde` as external deps. (b) `Cargo.lock` pinning. (c) CI builds with `--locked`. (d) Periodic `cargo audit`.
- **Status:** Current dep count: 2 external crates. Attack surface is minimal.

## 8. Threat Matrix Summary

| ID | Threat | Severity | Mitigated | Module |
|----|--------|----------|-----------|--------|
| 001 | Nothing-at-Stake | Critical | ✅ | stake.rs |
| 002 | Long-Range Attack | High | ✅ | sync.rs, genesis.rs |
| 003 | Validator Censorship | High | ⚠️ Partial | block.rs, mempool.rs |
| 004 | Block Withholding | Medium | ✅ | block.rs |
| 005 | Lazy Provider | Critical | ✅ | dispute.rs, participant.rs |
| 006 | Challenger Griefing | Medium | ✅ | dispute.rs |
| 007 | Determinism Evasion | High | ✅ | determinism.rs, canonical_cpu.rs |
| 008 | Activation Root Collision | Low | ✅ | merkle.rs |
| 009 | Bisection Timeout | Medium | ✅ | dispute.rs |
| 010 | Stake Grinding | High | ✅ | stake.rs, reputation.rs |
| 011 | Payment Channel Exhaustion | High | ✅ | payment.rs |
| 012 | Gas Price Manipulation | Medium | ✅ | gas.rs |
| 013 | Reward Self-Challenge | Low | ✅ | dispute.rs, rewards.rs |
| 014 | Eclipse Attack | High | ⚠️ Partial | network.rs |
| 015 | Transaction Replay | Medium | ✅ | state.rs |
| 016 | Gossip Amplification | Medium | ✅ | network.rs |
| 017 | Data Withholding | Medium | ⚠️ Partial | pdp.rs |
| 018 | PDP Proof Outsourcing | Low | ✅ | pdp.rs |
| 019 | Key Compromise | Critical | ✅ | wallet.rs |
| 020 | State Trie Bloat | Medium | ⚠️ Partial | state.rs, gas.rs |
| 021 | Supply Chain | High | ✅ | CI, Cargo.lock |

**Legend:** ✅ = Fully mitigated in current implementation | ⚠️ = Partially mitigated, improvements planned

## 9. Open Items

1. **Erasure coding for multi-provider redundancy** (mitigates THREAT-017 fully)
2. **State rent implementation** (mitigates THREAT-020 fully)
3. **Forced transaction inclusion protocol** (strengthens THREAT-003 mitigation)
4. **Formal verification of bisection game termination** (strengthens THREAT-005/009)
5. **HSM integration guide** (operational hardening for THREAT-019)

---

*This is a living document. Update as new threats are identified or mitigations are implemented.*
