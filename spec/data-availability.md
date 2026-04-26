# 2.3 Data availability

PDP (§2.1) proves that a prover **has** the bytes. This section addresses a separate question: are the bytes **retrievable by clients**? A prover that holds bytes but refuses to serve them is a different failure mode from a prover that has lost the bytes, and PDP alone cannot detect it.

This spec is **draft** because the on-chain enforcement story is still being designed. The retrieval HTTP path (§4.1.3) is reliable; the *economic* enforcement of retrievability is what's WIP.

## 2.3.1 The problem

A prover passes PDP challenges by holding the bytes locally. The bytes might still be unreachable to clients due to:

- Misconfigured firewall
- Saturated upstream link
- TLS misconfiguration or expired certificate
- Deliberate refusal to serve specific clients (censorship)
- Refusal to honor `Range` requests, breaking large-file streaming
- Selective serving (returning bytes for `HEAD` but not `GET`)

PDP doesn't catch any of this. We need a separate retrievability check.

## 2.3.2 Retrievability sampling

A network of independent **samplers** periodically requests random pieces from random provers and reports success/failure. Each sample produces:

```solidity
struct RetrievabilitySample {
    address sampler;
    address prover;
    bytes32 pieceCid;
    uint64  attemptedAt;     // unix seconds at probe start
    bool    success;
    uint16  latencyMs;       // first-byte latency, capped at 65_535
    uint8   probeKind;       // 1 = HEAD, 2 = GET, 3 = GET+RANGE
    bytes32 evidence;        // sha256 of the response or canonical failure code
}
```

A sampler MUST sign a `RetrievabilityReport` containing a list of samples plus a `samplerEpoch` timestamp:

```solidity
struct RetrievabilityReport {
    address sampler;
    uint64  samplerEpoch;        // monotonically increasing per sampler
    RetrievabilitySample[] samples;
    bytes   signature;            // EIP-191 personal_sign over keccak256(abi.encode(rest))
}
```

Reports are submitted to `RetrievabilityRegistry.submitReport(report)`. The registry stores only the digest on-chain; the full sample payload is committed via the digest and persisted off-chain on the same content-addressed pipeline as deal metadata (see §3.2).

### 2.3.2.1 Sampler eligibility

A sampler MUST register via `RetrievabilityRegistry.registerSampler(endpoint, region)` and MUST stake at least `samplerMinStake` PROVA. Three classes of sampler are explicitly recognized:

| Class | How they qualify | Weight in aggregation |
| --- | --- | --- |
| **Provers** | Already staking under `ProverStaking`; reciprocal sampling. | 1.0 |
| **Protocol-funded** | Run by Prova governance, geographically distributed, deterministic schedule. | 1.0 |
| **Public** | Any address, with `samplerMinStake` posted (small, e.g. 100 PROVA). | 0.5 |

The lower weight on public samplers limits Sybil influence: an attacker creating 10,000 public sampler addresses still has bounded impact relative to the protocol-funded set.

### 2.3.2.2 Sampling targets

A sampler picks a target piece-CID using:

```
seed       = keccak256(samplerAddress, samplerEpoch, blockNumber)
pieceIndex = uint(seed) mod activePieceCount
proverPick = uint(seed >> 128) mod activeProversForPiece(pieceCid)
```

The active piece set is derived from `ContentRegistry`. Public samplers SHOULD reseed their `samplerEpoch` at most once every `minSamplerEpochPeriod` (proposed: 60 seconds) to avoid bursting a single prover with simultaneous probes.

### 2.3.2.3 Probe protocol

A standard probe is:

1. Open TLS to the prover's `ProverRegistry.endpoint`.
2. Issue `HEAD /piece/{cid}` (probeKind=1) and verify status, headers, and `content-length`.
3. Issue `GET /piece/{cid}` for pieces ≤ 1 MiB; otherwise issue `GET /piece/{cid}` with `Range: bytes=0-65535` (probeKind=2 or 3).
4. Recompute the partial piece-CID; the GET probe is a **success only if** the bytes match. (For range probes, success is the bytes matching the Merkle subtree root for that range — see §2.1.4.)
5. The full first-byte latency is recorded, regardless of success.

Probe timeout is 30 seconds; any later response is treated as a failure (and may double-count as latency overflow).

## 2.3.3 Aggregation and slashing

Samples are aggregated per `(prover, epoch)`. An epoch is one day, aligned with PDP epochs.

Per-prover, per-epoch aggregate:

```
weightedAttempts(p, e)   = Σ over samples for prover p in epoch e: weight(sampler)
weightedSuccesses(p, e)  = Σ over successful samples: weight(sampler)
successRate(p, e)        = weightedSuccesses / weightedAttempts
```

Slashing is gated on a trailing window:

```
windowAttempts(p)  = Σ weightedAttempts over last retrievabilityWindowDays
windowSuccesses(p) = Σ weightedSuccesses over last retrievabilityWindowDays
windowRate(p)      = windowSuccesses / windowAttempts
```

A prover MAY be slashed by a `markRetrievabilityFault(prover)` call if and only if **all** of:

| Condition | Default value |
| --- | --- |
| `windowAttempts(p) >= minSampleCount` | 100 |
| `windowRate(p) < retrievabilityThreshold` | 0.95 |
| The contributing samples come from at least `minDistinctSamplers` independent samplers | 5 |
| At least `minDistinctRegions` distinct geographic regions are represented | 3 |
| The prover did not produce a successful retrievability proof during the dispute window | n/a |

The `minDistinctRegions` requirement is the structural mitigation against a coordinated regional outage (a transit link blackholing one upstream) being mistaken for prover misbehavior.

### 2.3.3.1 Dispute window

When `markRetrievabilityFault` is called, the registry opens a 24-hour dispute window. During the window, the prover MAY:

1. Submit a `RetrievabilityProof` directly: a signed assertion from `≥ minProofSamplers` (proposed: 7) independent samplers that the piece is currently retrievable. Successful proof closes the dispute and burns the challenger's bond.
2. Submit a `RetrievabilityException`: a signed claim that the affected window includes a maintenance announcement filed via `RetrievabilityRegistry.scheduleMaintenance` at least 24 hours in advance. Approved exceptions exclude the listed time range from the slashing math; abuse is deterred by a per-prover annual cap of 30 maintenance days.
3. Do nothing, in which case slashing executes after 24h.

### 2.3.3.2 Slash size

The slash is `retrievabilitySlashFraction` (proposed: 5% of bonded stake) per fault, capped at one fault per epoch. A prover that consistently misses retrievability will see compounding slashes over weeks; this is intentional. The challenger receives `challengerBountyBps` (proposed: 10% of the slashed amount) as a bounty, with the remainder burned.

## 2.3.4 What we won't promise

- We do not promise SLA-grade retrieval latency. Provers MAY be slow; sampling thresholds use binary success/failure with a generous timeout (default 30 seconds first byte). Latency is recorded but not slashable.
- We do not promise censorship-resistance against state-level pressure on individual provers. The redundancy parameter (deal-level N copies) is the structural mitigation.
- We do not promise CDN-grade throughput. Prova is for archival and verifiable retrieval, not interactive page-load. Provers MAY put a CDN in front (§4.1.8).

## 2.3.5 Open questions

These are explicitly tracked as open and contributors are welcome to propose answers via PR or governance forum.

- **Sampler economics**: who pays the samplers? *Current design* (this draft): protocol-funded samplers cover the baseline; public samplers earn a small per-report fee from the same emission bucket as `ProverRewards`. Reciprocal sampling between provers is uncompensated (each prover benefits from a healthy network). The fee curve is TBD and must avoid creating a positive ROI for spamming.
- **False-positive risk**: a prover may legitimately be down for maintenance. *Current design*: the 24-hour dispute window plus the `RetrievabilityException` mechanism with a 30-day annual cap covers planned downtime. Unplanned outages still slash; the redundancy parameter compensates clients.
- **On-chain cost**: aggregating thousands of samples per prover per epoch is expensive. *Current design*: only the report digest is on-chain; the full payload is content-addressed and persisted via the same pipeline as deal metadata. A `RetrievabilityCommitment` Merkle root commits to the aggregated samples per epoch and is what the slashing path verifies. The committee-aggregator pattern from §2.2 (checkpoint anchoring) applies, with one root per epoch.
- **Sampler collusion**: a coordinated set of samplers could falsely report a healthy prover as failing. *Mitigations*: stake at risk for all sampler classes, the `minDistinctRegions` requirement, and the dispute path that lets a prover prove retrievability directly. This is not bulletproof at low scale and SHOULD be revisited once we have at least 50 independent samplers across at least 5 regions.

This section will be promoted from Draft to Reliable when:

1. `RetrievabilityRegistry` is deployed and verified on Base Sepolia.
2. At least 50 independent samplers across at least 5 regions are operating on testnet.
3. A red-team simulation of sampler collusion has been run and documented.
4. The sampler fee curve is finalized and audited.
