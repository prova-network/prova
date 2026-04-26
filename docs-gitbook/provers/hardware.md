---
description: What you need to run a profitable prover.
---

# Hardware requirements

A Prova prover doesn't need exotic hardware. The protocol is **disk-bound, not CPU-bound** — no PoRep, no SNARKs, no GPU.

## Minimum

* **CPU:** 4 cores (2.0 GHz+)
* **RAM:** 8 GB
* **Disk:** anything ≥ 1 TB. SATA SSD or NVMe preferred. Spinning rust works for cold storage.
* **Network:** symmetric 100 Mbps. Real upload, not consumer-asymmetric.
* **Connection:** real IP, real DNS. No CGNAT.
* **OS:** Linux (Ubuntu 22.04+, Debian 12+, RHEL 9+ tested). macOS works for dev. Windows untested.

## Recommended (10 TB tier)

* **CPU:** 8 cores
* **RAM:** 32 GB
* **Disk:** 10 TB NVMe or 4× 4 TB SSD ZFS RAID-Z1. ECC where possible.
* **Network:** 1 Gbps symmetric. Multi-homed if you can.
* **Power:** UPS. Single power loss during a proof = missed proof = slash.

## At scale (100 TB+)

* Server-grade hardware. Hetzner SX-line, OVH bare metal, used Dell R7xx with shelves.
* ZFS or btrfs with checksumming and scrubbing.
* Two upstream providers (BGP if you're feeling spicy).
* Cooling. Real cooling.

## What disk shape

| Disk | Good for | Notes |
| --- | --- | --- |
| **NVMe** | Hot tier, fast retrievals | Most expensive per byte |
| **SATA SSD** | Default. Best balance. | Sweet spot for now |
| **SAS HDD (7.2k or 10k)** | Cold tier, cheap bulk | Slower retrievals → may lose deals to faster provers |
| **SMR HDD** | Don't | SMR's write amplification kills proof latency |

You can mix tiers. The prover daemon supports a "hot" tier (recent + popular pieces) and a "cold" tier (rarely-retrieved). Caching strategies live in `prover/pkg/store/`.

## What CPU

PDP proofs are SHA-256 on a few KB at a time. Any modern CPU does this in microseconds. The CPU bottleneck (if any) is **TLS termination + HTTPS retrieval throughput** — i.e., serving clients, not generating proofs. Modern AES-NI CPUs handle gigabit-line-rate HTTPS without breaking 5% utilization.

## What network

The two metrics that matter:

1. **Upload bandwidth.** You're serving every retrieval. A 1 Gbps link means ~125 MB/s peak retrieval throughput. If your provers are popular, more = more revenue.
2. **Latency.** Clients route to the fastest healthy prover. A prover in São Paulo serving Norwegian users will lose to one in Frankfurt. Place your hardware where the demand is.

## Geographic placement

If you're starting with one node, put it where:

* Land and electricity are cheap (Nordic countries, Iceland)
* Latency to ENS / NFT / web3 traffic is good (EU + NA East coast)
* Regulatory environment is friendly (Norway, Switzerland, Singapore)

If you're scaling to 5+ nodes, spread across continents. Prova's marketplace prefers geographically-diverse redundancy, which means cross-continent provers see preferential placement.

## Stake (the hidden cost)

Your **biggest hardware-equivalent expense** is the USDC stake you must lock to commit capacity. The current ratio is ~$50 stake per TiB committed (configurable per the `ProverStaking.sol` parameters; check before registering).

For a 10 TB prover:

* Disk: $1,000-2,500 (one-time)
* Network: $50-100/month
* Stake: ~$500 USDC locked (refundable when you exit cleanly)
* Slash exposure: up to your full stake if you fail badly

Plan accordingly.

## See also

* [Earnings calculator](earnings.md)
* [Become a prover](become-a-prover.md)
