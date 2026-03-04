# SPEC-007: Network Protocol Specification

**Status:** Draft  
**Author:** Capri  
**Created:** 2026-03-04

## 1. Overview

Prova uses a gossip-based P2P network for propagating commits, challenges, proofs, and blocks. The protocol is designed for low-latency dispute handling and efficient proof dissemination.

## 2. Message Types

### 2.1 Gossip Messages

| Message | Size (approx) | Priority | TTL |
|---------|---------------|----------|-----|
| `InferenceCommit` | ~256 bytes | Normal | 60s |
| `Challenge` | ~320 bytes | High | 30s |
| `BisectionResponse` | ~128 bytes | High | 15s |
| `PDPProof` | ~2-8 KB | Normal | 120s |
| `AuditReport` | ~256 bytes | Normal | 60s |
| `Block` | ~100 KB-1 MB | Critical | 300s |

### 2.2 Request/Response

| Request | Response | Use |
|---------|----------|-----|
| `GetActivation(commit_id, layer)` | `Activation(hash, proof)` | Dispute verification |
| `GetModel(model_id)` | `ModelManifest` | Model discovery |
| `GetPeers` | `PeerList` | Peer discovery |
| `GetBlock(height)` | `Block` | Chain sync |

## 3. Peer Discovery

### 3.1 Bootstrap Nodes
Hard-coded seed nodes for initial connection:
```
/dns4/boot1.prova.network/tcp/30333/p2p/<peer-id>
/dns4/boot2.prova.network/tcp/30333/p2p/<peer-id>
```

### 3.2 Kademlia DHT
After bootstrap, peers discover each other via Kademlia:
- Bucket size: 20
- Refresh interval: 300s
- Peer ID: SHA-256 of public key

## 4. Topic Subscriptions

Gossipsub topics:
```
/prova/1/commits       — inference commits
/prova/1/challenges    — dispute challenges
/prova/1/bisection     — bisection game messages
/prova/1/proofs        — PDP proofs
/prova/1/audits        — audit reports
/prova/1/blocks        — new blocks
```

Nodes subscribe to topics based on their role:
- **Provider:** all topics
- **Challenger/Verifier:** commits, challenges, bisection, audits
- **Light client:** blocks only

## 5. Block Propagation

### 5.1 Block Structure
```
Block {
    header: BlockHeader {
        height: u64,
        parent_hash: Hash,
        state_root: Hash,
        timestamp: u64,
        proposer: Address,
    },
    body: BlockBody {
        commits: Vec<InferenceCommit>,
        challenges: Vec<Challenge>,
        bisection_responses: Vec<BisectionResponse>,
        pdp_proofs: Vec<PDPProof>,
        audit_reports: Vec<AuditReport>,
        payments: Vec<PaymentTx>,
    },
}
```

### 5.2 Block Time
Target: 30 seconds (matches Filecoin epoch duration).

### 5.3 Block Size
Soft limit: 1 MB. Hard limit: 5 MB. Priority ordering:
1. Bisection responses (time-sensitive)
2. Challenges (time-sensitive)
3. Commits
4. PDP proofs
5. Payments
6. Audit reports

## 6. Consensus

Prova uses **dual-weighted Proof of Stake**:
- Storage weight: proportional to PDP-verified storage
- Compute weight: proportional to verified inference throughput

```
total_power(node) = α × storage_power + (1-α) × compute_power
```

Where `α` is a governance parameter (initial: 0.5).

Block proposer selection uses VRF (Verifiable Random Function) weighted by total power.

## 7. Security

### 7.1 Eclipse Resistance
- Minimum peer connections: 8
- Maximum peer connections: 50
- Peer rotation: 10% per hour
- Outbound-only connections: at least 4

### 7.2 DoS Protection
- Rate limiting: 100 messages/second per peer
- Message deduplication: bloom filter (10-minute window)
- Ban score: peers accumulate penalty for invalid messages

### 7.3 Sybil Resistance
All meaningful actions (commit, challenge, audit) require on-chain stake. Network-level sybil attacks don't grant voting power.

## 8. Sync Protocol

### 8.1 Fast Sync
New nodes can sync via:
1. Download block headers from genesis
2. Download state snapshot at recent checkpoint
3. Apply blocks from checkpoint to head

### 8.2 Checkpoint Frequency
Every 2880 epochs (~24 hours). Checkpoints include full state root + proof.

## 9. References

- [libp2p Gossipsub](https://docs.libp2p.io/concepts/pubsub/overview/)
- [Kademlia DHT](https://en.wikipedia.org/wiki/Kademlia)
- Filecoin network protocol (for epoch timing reference)
