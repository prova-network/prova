# 2.2 Checkpoint anchoring

Frequent on-chain proof submission is expensive. Section 2.1 specifies the *cryptographic* requirements; this section specifies the *batching and anchoring* protocol that lets a prover post many proofs cheaply by aggregating them into checkpoints anchored to Base.

## 2.2.1 Goals

- Reduce on-chain gas cost per proof to a fraction of a cent.
- Preserve the soundness of PDP: an honest verifier still detects a dishonest prover within bounded time.
- Allow a prover to recover from an offline period without losing all stake immediately.

## 2.2.2 Checkpoint structure

A **checkpoint** is a tuple:

```
struct Checkpoint {
    address prover;
    uint64  startEpoch;       // first challenge in this checkpoint
    uint64  endEpoch;         // last challenge in this checkpoint
    bytes32 challengeBatchRoot;  // Merkle root over challenge responses
    bytes   aggregatedProof;     // optional: ZK aggregation (v2)
}
```

`challengeBatchRoot` is the Merkle root over the prover's responses to every challenge in `[startEpoch, endEpoch]`. The on-chain verifier MAY spot-check by sampling random leaves of this tree.

## 2.2.3 Anchoring cadence

A prover MUST anchor at least one checkpoint to Base every `maxAnchorGap` epochs (default: 32 epochs ≈ 16 minutes at 30s cadence). Failure to anchor within the gap triggers a missed-checkpoint event, which counts toward slashing.

A prover MAY anchor more frequently; checkpoints are commutative.

## 2.2.4 On-chain anchor format

The marketplace exposes `submitCheckpoint(Checkpoint calldata cp)`:

```solidity
function submitCheckpoint(Checkpoint calldata cp) external {
    require(msg.sender == cp.prover, "only prover");
    require(cp.endEpoch >= cp.startEpoch, "invalid range");
    require(cp.endEpoch < currentEpoch(), "future epoch");
    // Verify aggregated proof if present
    if (cp.aggregatedProof.length > 0) {
        require(verifyAggregated(cp), "bad aggregation");
    }
    // Record the checkpoint
    checkpoints[cp.prover].push(cp);
    emit CheckpointAnchored(cp.prover, cp.startEpoch, cp.endEpoch, cp.challengeBatchRoot);
}
```

## 2.2.5 Spot-check verification

The verifier MAY at any time call `requestSpotCheck(prover, epoch)`. The prover then has `spotCheckWindow` (default 600 seconds) to provide:

- The challenge response for `epoch` (leaf + path)
- The Merkle path from that leaf up to the `challengeBatchRoot` of the checkpoint covering `epoch`

The verifier checks both paths. A failure here is treated as a slash event.

## 2.2.6 Aggregation (v2 future work)

A future amendment MAY introduce ZK aggregation: a single zk-SNARK or zk-STARK that proves "all challenge responses in [startEpoch, endEpoch] were valid." This reduces verifier cost from `O(spot_check_count)` to `O(1)` per checkpoint.

The current spec does NOT require aggregation. Provers MAY submit empty `aggregatedProof` and rely on spot-checks alone.

## 2.2.7 Cost analysis

At 30-second cadence and 32-epoch checkpoints:

- One checkpoint per 16 minutes per prover
- Checkpoint tx cost on Base: ~80,000 gas
- At 0.05 gwei: ≈ $0.001 per checkpoint at $2,500 ETH
- Per prover per day: ~90 checkpoints ≈ $0.09

This is the design budget. Actual numbers will be measured on testnet and published.

## 2.2.8 Open questions

- **Aggregation primitive choice**: SP1 / Risc0 / SnarkJS-compiled circuits — TBD on testnet experience.
- **Spot-check sampling rate**: how often the verifier calls `requestSpotCheck` per prover. Currently set per-deal; may move to a protocol-wide policy.
- **Slashing for missed-checkpoint vs missed-challenge**: balance is currently 1:1 weight. Open whether to weight checkpoint misses higher.
