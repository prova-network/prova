---
description: Run the numbers before you commit hardware.
---

# Earnings

Calculate your monthly revenue, fees, and net margin from running a Prova prover.

## Try the live calculator

[prova.network/#earn](https://prova.network/#earn) has the interactive version with sliders. The math below is the same model.

## Formula

```
gross monthly = capacity_TiB × utilization × price_per_TiB_month
fees          = gross × 0.01           # protocol fee
drag          = gross × (1 − uptime)   # missed proofs
net monthly   = gross − fees − drag
```

* `capacity_TiB` — your committed capacity in TiB (TB × 0.9094)
* `utilization` — fraction of your capacity that's actually under contract (0.7–0.9 in practice)
* `price_per_TiB_month` — what you advertise; the marketplace clears at ~75% of advertised
* `uptime` — fraction of proofs you successfully posted (target ≥ 99.5%)

## Worked example

A 20 TB prover at 75% utilization, $4/TiB-month, 99.5% uptime:

```
capacity   = 20 × 0.9094 = 18.19 TiB
active     = 18.19 × 0.75 = 13.64 TiB
gross      = 13.64 × $4.00 = $54.56 / month
fee (1%)   = $0.55
drag       = $0.27
net        = $53.74 / month
annualized = $644 / year
```

For a 100 TB prover, multiply by 5:

```
gross      = $272.80 / month
net        = $268.70 / month
annualized = $3,224 / year
```

For a 1 PB cluster:

```
gross      = $2,728 / month
net        = $2,687 / month
annualized = $32,244 / year
```

## Break-even

Hardware capex on a Hetzner AX41 (10 TB SSD) is ~$50/month rent. Net at $4/TiB-month and 75% utilization breaks even at:

```
50 / (4 × 0.9094 × 0.75 × 0.99 × 0.99) = ~18.6 TiB
```

So a 20 TB prover at the default parameters is **just above break-even** at $4/TiB-month. Push price up to $6 or utilization up to 90% and margins improve quickly.

## Pricing strategy

You set the advertised price. The marketplace doesn't enforce a single price — clients pick provers based on price + reputation + region. Three positions:

* **Cold / archive:** $1-2 / TiB-month. High utilization, low margin per byte.
* **Default:** $3-5 / TiB-month. Most clients. Sweet spot.
* **Premium / SLA:** $6-10 / TiB-month. Constrained regions, high reputation, fast retrieval, white-glove ops.

Beyond ~$10, you're competing with S3 and need a reason — usually compliance (data sovereignty, audit trails).

## Slash risk

Worst case: 100% of stake gets slashed. For a 10 TB prover with $500 staked, that's a $500 hit on top of lost revenue. Conservative slash schedule:

| Failure | Slash |
| --- | --- |
| Single missed proof | 0% (warning) |
| 3 consecutive misses | 5% |
| Sustained failure | 25% |
| Provable deletion | 100% + ban |

Most operators see 0% slashing if they:
* Run on a UPS
* Monitor proof success rate (alert below 99%)
* Use ZFS / btrfs with monthly scrubs
* Plan maintenance windows during low-deal-count periods

## Tax treatment

Storage revenue is ordinary income in most jurisdictions. Slashed stake might be treated as a capital loss or a cost-of-doing-business expense — talk to a real accountant. The on-chain ledger gives you a complete audit trail.

## See also

* [Hardware](hardware.md)
* [Become a prover](become-a-prover.md)
