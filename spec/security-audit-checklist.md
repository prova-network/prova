# Security Audit Checklist — Prova Network

**Status:** Draft v0.1
**Author:** Capri (for Prova project)
**Date:** 2026-03-04
**Companion docs:** `security-threat-model.md`, `bridge-security.md`, `checkpoint-anchoring.md`

## 1. Purpose

Pre-audit checklist for external security review. Each item maps to a module, threat vector, and verification method. Auditors should verify every MUST item; SHOULD items are recommended.

## 2. Checklist Categories

### 2.1 Consensus & Block Production

| ID | Check | Module | Severity | Verification |
|----|-------|--------|----------|-------------|
| AUD-001 | Equivocation detection slashes 100% stake on conflicting blocks at same height | `stake.rs`, `block.rs` | Critical | Submit two blocks at height N from same validator; assert full slash |
| AUD-002 | Finality gadget prevents reorgs beyond checkpoint depth | `finality.rs`, `checkpoint.rs` | Critical | Attempt reorg past last anchored checkpoint; assert rejection |
| AUD-003 | Block validation rejects invalid proposer (wrong turn / insufficient stake) | `block.rs`, `stake.rs` | High | Forge block from non-elected proposer; assert rejection |
| AUD-004 | Epoch transitions are atomic (no partial state) | `epoch.rs` | High | Kill node mid-epoch-transition; restart; assert consistent state |
| AUD-005 | Fork choice rule selects heaviest valid chain | `block.rs` | High | Present two forks with different weights; assert correct selection |

### 2.2 Inference Verification (QBP)

| ID | Check | Module | Severity | Verification |
|----|-------|--------|----------|-------------|
| AUD-010 | Bisection game resolves to single divergent step | `dispute.rs`, `participant.rs` | Critical | Run full bisection with known-bad activation at step K; assert isolation |
| AUD-011 | Challenge window enforced — late challenges rejected | `commit.rs` | High | Submit challenge after window expires; assert rejection |
| AUD-012 | Activation Merkle proofs validate against committed root | `merkle.rs` | Critical | Submit proof with tampered leaf; assert verification failure |
| AUD-013 | Canonical CPU path produces identical output to reference | `canonical_cpu.rs` | High | Run same input on canonical and reference; assert bitwise match |
| AUD-014 | Invalid inference commit leads to slashing after successful challenge | `dispute.rs`, `stake.rs` | Critical | Commit wrong result, challenge, complete bisection; assert slash |

### 2.3 Economic Security

| ID | Check | Module | Severity | Verification |
|----|-------|--------|----------|-------------|
| AUD-020 | Total supply is conserved across all operations (mint + burn + transfer = 0) | `rewards.rs`, `gas.rs`, `stake.rs` | Critical | Run 10K random transactions; assert sum of all balance changes = block rewards minted |
| AUD-021 | Staking lock period enforced — early unstake rejected | `stake.rs` | High | Attempt unstake before lock expires; assert rejection |
| AUD-022 | Gas fee burning matches EIP-1559 base fee calculation | `gas.rs` | Medium | Sequence of blocks at varying utilization; assert base fee follows formula |
| AUD-023 | Payment channels cannot be double-settled | `payment.rs` | Critical | Attempt settle with old state after newer settle; assert rejection |
| AUD-024 | Reward distribution sums to exactly the block reward (no rounding leak) | `rewards.rs` | Medium | Distribute across 100 recipients; assert sum = reward |
| AUD-025 | SLA penalties cannot exceed staked amount | `sla.rs` | High | Trigger maximum penalty accumulation; assert capped at stake |

### 2.4 Access Control & Authorization

| ID | Check | Module | Severity | Verification |
|----|-------|--------|----------|-------------|
| AUD-030 | Role capabilities are least-privilege (Observer cannot submit txs) | `access.rs` | High | Attempt tx submission from Observer role; assert denied |
| AUD-031 | Pause halts all non-governance operations | `access.rs` | Critical | Enable pause; attempt commit/challenge/transfer; assert all rejected |
| AUD-032 | Capability expiry enforced — expired grants rejected | `access.rs` | Medium | Grant capability with TTL; advance past TTL; assert denied |
| AUD-033 | Admin role cannot bypass slashing (no self-exemption) | `access.rs`, `stake.rs` | Critical | Equivocate from Admin account; assert slashing proceeds |
| AUD-034 | Rate limiter prevents address from exceeding tx throughput cap | `rate_limiter.rs` | Medium | Send N+1 txs from same address in one epoch; assert last rejected |

### 2.5 Networking & P2P

| ID | Check | Module | Severity | Verification |
|----|-------|--------|----------|-------------|
| AUD-040 | TLS mutual authentication rejects invalid certificates | `tls.rs` | High | Connect with self-signed cert not in trust store; assert rejection |
| AUD-041 | Gossip protocol rejects messages from non-staked peers | `network.rs` | Medium | Send gossip from address with 0 stake; assert dropped |
| AUD-042 | Block sync validates all blocks in downloaded range | `sync.rs` | High | Inject invalid block in sync range; assert detection and disconnect |
| AUD-043 | Eclipse attack resistance — minimum diverse peer connections | `network.rs` | High | Attempt to fill all peer slots from single subnet; assert diversity requirement |

### 2.6 Cross-Chain & Bridge

| ID | Check | Module | Severity | Verification |
|----|-------|--------|----------|-------------|
| AUD-050 | Checkpoint quorum requires >2/3 stake weight | `checkpoint.rs` | Critical | Submit checkpoint with 60% quorum; assert rejection |
| AUD-051 | Bridge message replay rejected (nonce tracking) | `bridge.rs` | Critical | Replay previously processed bridge message; assert rejection |
| AUD-052 | L1 anchor verification checks Filecoin tipset validity | `watcher.rs`, `checkpoint.rs` | High | Submit anchor referencing non-existent tipset; assert rejection |
| AUD-053 | State proof verification matches committed state root | `bridge.rs` | Critical | Submit proof against wrong state root; assert verification failure |

### 2.7 State & Storage

| ID | Check | Module | Severity | Verification |
|----|-------|--------|----------|-------------|
| AUD-060 | State trie produces deterministic root for identical state | `state.rs` | Critical | Apply same txs in same order on two nodes; assert identical roots |
| AUD-061 | Snapshot integrity verified on import (Merkle root check) | `snapshot.rs` | High | Corrupt one chunk of snapshot; import; assert rejection |
| AUD-062 | Pruning preserves all state needed for active dispute windows | `pruning.rs` | High | Prune aggressively; attempt to resolve open dispute; assert state available |
| AUD-063 | Mempool eviction respects priority ordering | `mempool.rs` | Low | Fill mempool; submit higher-priority tx; assert lowest evicted |

### 2.8 Governance

| ID | Check | Module | Severity | Verification |
|----|-------|--------|----------|-------------|
| AUD-070 | Governance proposals require minimum stake to submit | `governance.rs` | Medium | Submit proposal from account below threshold; assert rejection |
| AUD-071 | Vote weight proportional to stake at snapshot height | `governance.rs` | High | Change stake after snapshot; vote; assert weight uses snapshot |
| AUD-072 | Emergency pause requires supermajority (>80%) | `governance.rs` | Critical | Attempt emergency pause with 75% vote; assert rejection |

## 3. Testing Requirements

### 3.1 Coverage Targets

| Category | Minimum Line Coverage | Minimum Branch Coverage |
|----------|----------------------|------------------------|
| Consensus (`block.rs`, `epoch.rs`) | 90% | 85% |
| Economics (`stake.rs`, `rewards.rs`, `gas.rs`, `payment.rs`) | 95% | 90% |
| Dispute (`dispute.rs`, `participant.rs`, `merkle.rs`) | 95% | 90% |
| Access control (`access.rs`, `rate_limiter.rs`) | 90% | 85% |
| Networking (`tls.rs`, `network.rs`, `sync.rs`) | 80% | 75% |
| Bridge (`checkpoint.rs`, `bridge.rs`) | 90% | 85% |

### 3.2 Fuzz Testing Targets (Future — CHAIN-026, NODE-025)

- State trie: random tx sequences must never produce inconsistent roots
- Merkle tree: arbitrary leaf insertions/deletions must maintain valid proofs
- Mempool: random insert/evict patterns must maintain ordering invariants
- Payment channels: random open/update/settle/dispute sequences must conserve funds

## 4. Audit Engagement Recommendations

1. **Scope:** All `chain/src/` and `node/src/` modules (≈30 files, ~15K lines)
2. **Duration:** 2–3 week engagement for two senior auditors
3. **Methodology:** Manual review + automated (Clippy strict, Miri for UB, cargo-fuzz)
4. **Priority order:** Economics → Consensus → Bridge → Dispute → Access Control → Networking
5. **Deliverable:** Finding-by-finding report with severity, PoC, and remediation guidance
6. **Re-audit trigger:** Any change to slashing, finality, or bridge logic

## 5. Known Accepted Risks

| Risk | Rationale | Monitoring |
|------|-----------|------------|
| Single checkpoint submitter can delay (not forge) anchors | Timeout fallback promotes next submitter | `submitter.rs` watchdog |
| Canonical CPU path is slower than GPU for verification | Correctness > speed for dispute resolution | Performance metrics |
| Observer role can read all state (no privacy) | Public chain by design | N/A |

---

*This checklist should be updated whenever new modules are added or threat model changes.*
