---
description: Your file survives us. Both of us.
---

# Resilience and redundancy

A single prover can fail. They might run out of disk, lose power, get DDoS'd, refuse to serve, or just decide to leave the network. Prova's resilience model handles all of these.

## How redundancy works

When you upload a file, you choose a **redundancy factor** (default: 4). The piece is pinned to that many independent provers. They each have their own deal and their own stake.

```
              ┌──── Prover A (Reykjavík) ──── alive
              ├──── Prover B (Singapore) ──── alive
   piece-cid ─┤
              ├──── Prover C (São Paulo) ──── alive
              └──── Prover D (Frankfurt) ──── alive
```

If Prover A goes down, retrieval falls through to B/C/D automatically. Prover A's stake gets slashed. The marketplace re-lists the slot, and a healthy prover picks it up — bringing redundancy back to 4.

You can simulate this in the [redundancy lab](https://prova.network/#redundancy) on the homepage.

## Choosing a redundancy factor

Higher redundancy = higher cost, higher durability. Some rules of thumb:

| Use case | Suggested redundancy |
| --- | --- |
| Personal backup | 2 |
| Static website (ENS contenthash) | 3 |
| Research dataset | 4 (default) |
| AI training corpus | 5 |
| Critical archival (cultural memory, legal records) | 6+ |

Each replica is a separate deal, so the cost scales linearly. Storing a 1 GB file at redundancy 4 costs ~4× the per-replica price.

## Geographic distribution

By default, Prova spreads replicas across **regions** (continents). You can override:

```ts
await prova.storage.upload(bytes, {
  redundancy: 4,
  regions: ['EU', 'NA', 'AP', 'SA'],   // explicit regions
})
```

Or constrain to a single region (for latency):

```ts
await prova.storage.upload(bytes, {
  redundancy: 3,
  regions: ['EU'],
})
```

## What happens during a failure

```
t=0    Prover A fails. Stops posting proofs.
t=30s  First missed proof on-chain.
t=60s  Second missed proof. Marketplace flags the deal.
t=90s  Third missed proof. Deal terminated.
       Prover A's stake is slashed by 5% (configurable).
       Refund of remaining escrow is queued.
t=120s Marketplace re-lists the slot.
       The piece is broadcast as "needs replication".
t=180s Healthy prover picks it up, copies from a sibling replica.
       Redundancy is restored to 4.
```

Total downtime for the affected replica: ~3 minutes. Total downtime for the **piece itself** (retrieval availability): zero, because the other replicas are still serving.

## Slashing parameters

| Failure | Slash |
| --- | --- |
| Single missed proof | 0% (warning) |
| 3 consecutive missed proofs | 5% of stake |
| Sustained failure (10+ misses) | 25% |
| Deletion (proof of non-possession) | 100% + ban |

These are configurable per-deal in `StorageMarketplace.sol` but defaults are conservative.

## Probabilistic durability

If each prover has 99% uptime independently, your **piece** durability with `r` replicas is `1 - (0.01)^r`:

| r | Piece durability |
| --- | --- |
| 1 | 99% |
| 2 | 99.99% |
| 3 | 99.9999% |
| 4 | 99.999999% |
| 5 | 99.99999999% |

That's **eight nines** at the default redundancy of 4. Compare to S3's 11 nines (single-region) — Prova catches up by `r = 6`, with the additional benefit of being multi-jurisdictional.

## When redundancy isn't enough

For data that **must not be lost** (cultural archives, legal records, medical research), you can:

1. Set redundancy to 6+
2. Constrain to specific operators you trust
3. Combine Prova with an air-gapped local backup (cold tape, S3 Glacier)
4. Index the on-chain proofs yourself, so you can detect a network-wide failure even if the API is down

Prova is the **active** layer of a defense-in-depth strategy. It's better than S3, but it's not a replacement for owning your own copy of the bytes that matter most.
