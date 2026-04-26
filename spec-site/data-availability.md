<div class="spec-meta">
  <span class="label">State</span> <span class="badge state-wip">Draft / WIP</span>
  <span class="label">Theory audit</span> <span class="badge audit-na">N/A</span>
  <span class="label">Last updated</span> 2026-04-26
</div>

# 2.3 Data availability

PDP (§2.1) proves that a prover **has** the bytes. This section addresses a separate question: are the bytes **retrievable by clients**? A prover that holds bytes but refuses to serve them is a different failure mode from a prover that has lost the bytes.

This spec is **draft** because the on-chain enforcement story is still being designed. The retrieval HTTP path (§4.2) is reliable; the *economic* enforcement of retrievability is what's WIP.

## 2.3.1 The problem

A prover passes PDP challenges by holding the bytes locally. The bytes might still be unreachable to clients due to:

- Misconfigured firewall
- Saturated upstream link
- Deliberate refusal to serve specific clients (censorship)
- TLS misconfiguration

PDP doesn't catch any of this. We need a separate retrievability check.

## 2.3.2 Retrievability sampling (proposed)

A network of independent **samplers** periodically requests random pieces from random provers and reports success/failure. Each sample produces:

```
struct RetrievabilitySample {
    address sampler;
    address prover;
    bytes32 pieceCid;
    uint64  attemptedAt;
    bool    success;
    uint16  latencyMs;     // first-byte latency
    bytes32 evidence;      // hash of the response or failure code
}
```

Samplers MAY be:

- Other provers (reciprocal monitoring)
- A protocol-funded set of geographically distributed sampler nodes
- Anonymous public clients (with rate limits)

## 2.3.3 Aggregation and slashing

Samples are aggregated per-prover per-epoch. A prover with retrievability success rate below `retrievabilityThreshold` (proposed: 95% in trailing 30 days) over a sample population of at least `minSampleCount` (proposed: 100) MAY be slashed.

The slashing path requires:

1. Sample evidence aggregated and signed by ≥ N independent samplers.
2. The prover gets a 24-hour window to dispute.
3. If undisputed, the marketplace's `markRetrievabilityFault(prover)` is callable.

## 2.3.4 What we won't promise

- We do not promise SLA-grade retrieval latency. Provers MAY be slow; sampling thresholds use binary success/failure with a generous timeout (default 30 seconds first byte).
- We do not promise censorship-resistance against state-level pressure on individual provers. The redundancy parameter (deal-level N copies) is the mitigation.
- We do not promise CDN-grade throughput. Prova is for archival and verifiable retrieval, not page-load.

## 2.3.5 Open questions

- **Sampler economics**: who pays the samplers? Protocol-funded samplers create a centralization risk; anonymous samplers create a Sybil risk. Hybrid models are under consideration.
- **False-positive risk**: a prover may legitimately be down for maintenance. The 24-hour dispute window is the current mitigation; longer windows reduce false-positive risk but also slow down legitimate slashing.
- **On-chain cost**: aggregating thousands of samples per prover per epoch is expensive. We are evaluating zk-aggregation similar to checkpoint anchoring (§2.2).

This section will be promoted from Draft to Reliable when the sampler protocol is implemented and tested on Base Sepolia.
