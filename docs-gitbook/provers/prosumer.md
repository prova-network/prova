---
description: Run Prova on a home rack or small server room. Higher capacity, harder uptime requirements, better economics.
---

# Prosumer provers

The prosumer tier sits between hobby (laptop/NAS, < 100 TB) and enterprise (data center, multi-PB). Typical setup: a home rack or a small server in a colocation closet, 100 TB to 5 PB of storage, ~1 Gbps symmetric.

This is the sweet spot for someone who wants to make storage their actual side income or a small business. Earnings scale linearly with bytes proven.

---

## Who this is for

- An operator with 100 TB to 5 PB of usable disk
- Symmetric 1 Gbps (residential fibre or business connection)
- Comfortable with Linux, systemd, ZFS or similar redundancy
- Willing to commit 12+ months of operation

## What you get

| | Prosumer tier |
| --- | --- |
| Min usable disk | 100 TB |
| Min stake | `100 PROVA × committedGiB` (with USDC-equivalent floor) |
| Recommended hardware | Bare metal server, ECC RAM, ZFS or RAID-Z2, separate redundant network |
| Setup time | 1–2 days (including hardware burn-in) |
| Realistic capacity | 100 TB – 5 PB |
| Identity attestation | wallet active 90+ days, optionally SBT or ENS subdomain (not full KYC) |

## Setup overview

The CLI install is the same as hobby tier:

```bash
curl -fsSL https://get.prova.network/prover | sh
```

Differences from hobby tier:

1. **Multiple disks via ZFS pool.** `provad` can map multiple physical disks into one logical pool. Configure in `prover.toml` under `[storage]`.
2. **Dedicated network interface.** Recommend a separate NIC for proof traffic so retrievals don't impact proof submissions during peak.
3. **Rate-monitor your stake.** At 1 PB committed, your stake is in the millions of PROVA. Set up alerts when the USDC-equivalent floor moves close to your minimum (the [stake-floor oracle](../concepts/economics.md#stake-floor-oracle)).
4. **Multi-region announce.** If you have presence in multiple regions, register multiple provers (one per region) under separate wallets. Each will earn proportionally.

## Hardware notes

| Component | Recommended | Notes |
| --- | --- | --- |
| CPU | 8+ cores, modern x86 or ARM | Storage is disk-bound, not CPU-bound. Older 8-core Xeons work fine. |
| RAM | 64 GB ECC | ZFS likes RAM. ECC catches silent corruption. |
| Disk | NVMe metadata + spinning rust for bulk | NVMe for the ZFS metadata special-vdev; HDDs for the bulk. |
| Network | 1 Gbps symmetric, real bandwidth | Asymmetric "1 Gbps down / 50 Mbps up" residential is **not** enough. |
| Power | UPS, 60+ minute runtime | Sustained outages cause missed proofs and slashing. |

Cost estimate for a 1 PB prosumer setup (mid-2026 prices):

- HDDs: $9,000–14,000 (1 PB usable from raw with ZFS Z2)
- NVMe (metadata + L2ARC): $400
- Server (used Dell R730 or similar): $1,500
- Network: $0 (existing fibre) – $200/mo (business circuit)
- Power: ~$50–80/mo
- **One-time**: $11,000–16,000
- **Monthly**: $50–280

## Earnings model

Same as hobby tier — you earn USDC per deal proven, plus PROVA emission proportional to bytes proven. The emission curve is published; the [Earnings page](./earnings.md) has worked examples for prosumer scale.

A 1 PB prosumer at 70% utilisation, $2.50/TB-month effective price, 99.5% proof success:

```
USDC monthly:  1 PB × 1024 TB × 0.70 × $2.50 = $1,792 USDC
PROVA monthly: ~ 1 PB / total network committed × monthly_emission
```

The PROVA emission share scales with your fraction of total committed bytes on the network. Early provers earn a larger absolute share because the network is small; this is intentional.

## Slashing exposure

Prosumer tier has substantially more capital at risk than hobby. A 1 PB prover with the default `slashFraction = 10%` and `slashPerFault = 50 PROVA` is hit for ~50 PROVA per individual missed proof, and up to 10% of total locked stake per fault event.

Practical implication: at 1 PB you have ~10M PROVA staked. A serious cluster failure (e.g. ZFS pool degraded for 24 h producing missed proofs) could cost up to 1M PROVA. **Set up monitoring before you scale above 100 TB.**

The protocol's [stake-floor oracle](../concepts/economics.md#stake-floor-oracle) gives you a 7-day grace window to top up stake if PROVA's USD price drops; use this window, don't ignore the alert.

## Identity attestation

Above 100 TB committed, the protocol requires **lightweight identity attestation** (not full KYC). Options, any one suffices:

- An ENS name with verified contenthash and an ENS-resolver TXT record claiming this prover wallet
- An EAS attestation (Ethereum Attestation Service) signed by a published attester
- A 30-day continuous on-chain history showing legitimate activity

This is to discourage Sybil — the same person spinning up 50 prosumer provers from the same hardware to capture a larger emission share.

## Operational discipline

- **24/7 monitoring.** Prometheus + Grafana setup recommended. The official [`provad` Grafana dashboard](https://github.com/prova-network/prover/tree/main/observability) is included.
- **Alerting.** Set up PagerDuty / OpsGenie / opsgenie / your phone for: missed proofs, disk errors, network partition, stake-floor breach.
- **Quarterly capacity review.** Plan disk growth. Adding capacity requires re-registering with the new committed bytes and topping up stake.
- **Annual disk replacement budget.** HDDs fail. Replace SMART-warning drives within 7 days.

## Next steps

- [Hardware](./hardware.md) for the full per-tier table
- [Enterprise](./enterprise.md) if you cross 5 PB
- [Earnings](./earnings.md) for the per-tier earnings math
