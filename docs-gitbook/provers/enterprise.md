---
description: Run Prova at data-center scale. Multi-petabyte, SLA-grade uptime, dedicated infrastructure, SAFT-aligned stake.
---

# Enterprise provers

Enterprise tier is for operators running Prova at multi-petabyte scale in data centers, with dedicated infrastructure and serious uptime engineering.

This page is the operations contract for enterprise scale: hardware expectations, stake economics, SLA targets, identity attestation, and how to enrol with us as a launch partner.

---

## Who this is for

- Operators with **5 PB+** of usable disk
- Production data center or colocation, with redundant power and 10+ Gbps network
- Existing experience operating storage at scale (Filecoin SP, S3-compatible cluster operators, traditional storage businesses)
- Willing to commit 24+ months and put up multi-million-USD stake

If you're below 5 PB, the [prosumer tier](./prosumer.md) gives you the same per-byte economics with simpler operational expectations. Enterprise is mostly about scaling the same model with better discipline.

---

## What's different at enterprise scale

| | Hobby | Prosumer | Enterprise |
| --- | --- | --- | --- |
| Min capacity | 1 TB | 100 TB | 5 PB |
| Identity attestation | wallet 30d active | wallet 90d + ENS/EAS | full org KYB + signed master agreement |
| Stake (at $0.20 PROVA price example) | $20 / TB | $20 / TB + USD floor | $20 / TB + USD floor + reserve buffer |
| Monitoring | best-effort | Prometheus / alerting | 24/7 NOC, paged alerts, MTTR < 15 min |
| Settlement cadence | per-deal | per-deal | per-deal **or** monthly aggregated |
| Earnings tier multiplier | 1.0× | 1.0× | 1.0× — same per-byte rate, no special treatment |
| Insurance / underwriting | client risk | client risk | optional protocol-backed slashing insurance pool (governance-allocated) |

The earnings rate per byte is the same across all tiers. Enterprise gets you bigger numbers because you're providing more bytes, not because the rate per byte is higher.

---

## Hardware reference build (10 PB)

This is one workable shape. Adapt to your existing stack.

| Component | Spec | Qty | Notes |
| --- | --- | --- | --- |
| Storage server | 60-bay 4U JBOD chassis, 22 TB SATA HDDs | 10 | ~1 PB raw per chassis, ZFS Z3 |
| Compute head | 1U, 32-core EPYC, 256 GB ECC RAM, 2× 10 GbE | 10 | One head per JBOD |
| NVMe metadata | 7.68 TB enterprise NVMe | 20 | 2× per pool for ZFS special-vdev |
| Network | 100 GbE backbone, 10 GbE to each head, BGP multi-homed | 1 | Redundant uplinks |
| UPS / generator | 2N redundant, ≥ 30 minute UPS + diesel | 1 | |
| Total Capex | $300K–$450K | | |
| Monthly Opex | $4K–$8K | | Power, network, colocation, on-call |

For a 10 PB cluster at 70% utilisation and $2.50/TB-month effective price, the gross USDC monthly is roughly:

```
10 PB × 1024 TB × 0.70 × $2.50 = $17,920 USDC / month
```

Plus PROVA emission proportional to your share of total network committed bytes. In year 1 of the network, an enterprise prover holding 5–10% of total committed bytes can expect emission worth a meaningful multiple of the USDC stream; in steady-state (year 4+) the emission is much smaller and the USDC stream dominates.

---

## Identity attestation (enterprise)

Enterprise registration requires:

- **Org KYB**: legal entity, beneficial owners, jurisdiction
- **Signed master agreement** with the Prova org (covers SLA, dispute resolution, off-chain coordination)
- **Multi-sig wallet** (2-of-3 minimum) for the prover address; no single-key custody at this scale
- **Compliance contact** for abuse and law-enforcement requests (email + phone)
- **Public attestation**: a EAS attestation linking your legal entity to the prover wallet, on-chain

The attestation is light-touch *legally* (we don't review your books). It's heavy *on identity continuity*: we need to know which org we're talking to and how to reach them when something breaks.

## Stake mechanics at enterprise scale

A 10 PB enterprise prover at 100 PROVA/GiB minimum stake:

```
10 PB × 1024 GB/TB × 1024 = 10,485,760 GiB
10,485,760 × 100 = 1,048,576,000 PROVA
```

That's 1.05 **billion** PROVA. **Total supply is 100M**. So the per-GiB minimum is necessarily lower than 100 PROVA at enterprise scale or the floor binds well before mainnet liquidity supports it.

Practical reality at TGE:

- The `minStakePerGiB` is governance-tunable; we will set it dynamically based on PROVA market price
- At enterprise scale we expect a per-PB stake in the range of **5–50 million PROVA**, set by governance vote informed by available float
- The USDC-equivalent floor will be configured to maintain at least **$2/TB stake-to-capacity ratio** at all times

Until governance votes the production parameter, enterprise provers will negotiate stake commitments individually with the protocol team during the launch-partner phase. That's the honest operating posture; we'll tighten it as float deepens.

## SLAs we expect

For enterprise registration, the operating commitment:

- **Proof success rate**: ≥ 99.9% trailing 30-day
- **Retrieval availability**: ≥ 99.5% monthly, p95 < 5s for first byte
- **Network latency**: < 200ms inter-region for cross-region replication
- **MTTR**: median 15 min, p99 60 min

Failing these isn't an automatic protocol-level slash, but it does:
- Reduce your `qualityMultiplier` for emission rewards (down to 0.5×)
- Trigger a peer review by other enterprise provers
- After 3 consecutive months of SLA breach, the master agreement permits the protocol to refund client deals away from you

## SLAs we offer (enterprise client tier)

For enterprise *clients* using enterprise *provers*, the protocol underwrites:

- **Verifiable retention** (the core protocol guarantee)
- **Slashing-funded refund pool** for catastrophic prover failures
- **Quarterly attestation report** signed by the Prova org

The protocol does NOT underwrite: latency, retrieval throughput beyond contractual minimums, or anything outside the chain-verifiable retention story.

## SAFT alignment

Enterprise provers are a natural participant in the [SAFT round](../about.md). If your organisation:

- Plans to commit > 5 PB in the first year
- Is willing to lock PROVA stake equivalent to your SAFT allocation
- Wants reduced fees in years 1–2 (negotiated, not a protocol-level discount)

… we'd like to talk to you. [hello@prova.network](mailto:hello@prova.network) with subject "Enterprise SAFT".

## Onboarding sequence

1. Email `hello@prova.network` with subject "Enterprise prover".
2. Initial call to scope capacity, geography, SLAs.
3. Signed master agreement (template available on request).
4. KYB document collection.
5. Test deployment on Base Sepolia (1–10 TB, 30-day shakedown).
6. Mainnet registration with capacity commitment.
7. Continuous reporting + quarterly review.

## What we won't do

- **No data-center subsidy.** We don't pay your colocation bills or hardware capex. The network does not subsidise infrastructure.
- **No preferred routing.** Enterprise provers earn the same per-byte rate as hobby provers. The deal-routing algorithm doesn't favour you.
- **No off-chain side payments.** All economic flows go through the on-chain contracts. Anything that looks like a side payment from the protocol to a prover is a red flag.

The point: **the only way an enterprise prover makes more than a hobby prover is by providing more storage**. At every tier, the per-byte rate is the same.

## Next steps

- Read [Hardware](./hardware.md) for sizing across all tiers
- Read [Earnings](./earnings.md) for the per-byte math
- Email [hello@prova.network](mailto:hello@prova.network) to start enterprise onboarding
