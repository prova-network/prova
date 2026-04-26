---
description: Run Prova on the hardware you already have. Earn from spare disk on a laptop, Mac mini, or NAS.
---

# Hobby provers

A hobby prover runs Prova on existing hardware: a laptop, a Mac mini, a Synology NAS, an old desktop in the closet. Spare TB you weren't using anyway, monetised. No data center, no UPS, no rack.

This is the smallest tier we support. **Earnings scale with bytes proven, nothing else.** A 4 TB NAS will earn proportionally less than a 100 TB rack, full stop. The only way to earn more is to provide more storage.

---

## Who this is for

- Someone with a Mac mini, Synology / QNAP NAS, or unused desktop with at least 1 TB free
- A residential connection with **decent upload bandwidth** (≥ 50 Mbps symmetric is comfortable; 25 Mbps works but limits throughput)
- Someone OK with running a process 24/7 and getting woken up by uptime alerts a few times a year

This is **not** for people who:
- Have a laptop they close at night (provers must stay online)
- Have an asymmetric connection with < 10 Mbps upload (you'll miss proofs)
- Want to monetise consumer hardware they don't intend to keep running

## What you get

| | Hobby tier |
| --- | --- |
| Min usable disk | 1 TB free |
| Min stake | `100 PROVA × committedGiB` (with USDC-equivalent floor) |
| Hardware preset | Self-hosted |
| Setup time | 30 minutes |
| Realistic capacity | 1–20 TB |
| Earnings | USDC per deal × bytes proven, plus protocol PROVA emission proportional to bytes proven |

## Setup (Linux, macOS, Synology DSM)

### 1. Install `provad`

```bash
curl -fsSL https://get.prova.network/prover | sh
```

Drops the binary in `/usr/local/bin/` and writes a starter config to `/etc/prova/prover.toml`. On macOS the launchd plist is offered; on Linux the systemd unit is offered.

On Synology, follow [the Synology DSM install guide](./synology.md) (uses the Container Manager package).

### 2. Configure your share

`provad` will ask:

- **Storage path**: where bytes get stored. On a NAS this is usually `/volume1/prova/` (Synology) or `/mnt/storage/prova/` (Linux).
- **Capacity**: how many GB you want to commit. Start small (e.g. 500 GB). You can grow later.
- **Wallet**: your Base address. PROVA stake goes here and earnings settle here.

### 3. Stake PROVA

```bash
prova-prover stake 50000   # 50,000 PROVA → enough for ~500 GB
```

The stake locks until you unbond. Unbonding has a 7-day cooldown.

### 4. Register

```bash
prova-prover register --capacity 500GB --region eu-north
```

After registration, the marketplace can route deals to you.

---

## Hobby-tier limits

- **Hard cap**: 100 TB committed. Above this, you must register as a [prosumer](./prosumer.md) tier (different identity attestation requirements).
- **No KYC**: under 100 TB you can register pseudonymously with just a wallet address that's been active > 30 days.
- **Single identity**: one prover per wallet. The protocol detects and slashes obvious sybil patterns (multiple provers on the same `cf-connecting-ip` or with correlated proof timings).

---

## Realistic earnings

Earnings come from two places:

### 1. USDC from clients

For every deal you accept, you earn 99% of the per-day USDC stream as long as you keep posting valid proofs. At launch pricing ($1.50–$3.00 per TB-month), a 5 TB hobby prover with 80% utilisation earns roughly:

```
5 TB × 0.8 utilisation × $2.00/TB-mo × 99% = $7.92 USDC / month
```

Modest. Hobby tier is not a get-rich path. The math gets meaningful at 20 TB+, or once you stack the protocol PROVA emission on top.

### 2. PROVA emission

In addition to USDC, the protocol mints **PROVA emission** to provers proportional to bytes proven. Emission is heavy in the early years to bootstrap the network (declining schedule, full details in the [tokenomics doc](https://github.com/prova-network/prova/blob/main/TOKENOMICS-2026.md)).

Critical: **emission only flows to provers actually proving bytes**, gated by:

- The `prover != client` check (you can't farm emission by storing your own data)
- The redundancy cap (a piece-cid only earns up to N=4 provers' worth of emission)
- A 30-day vesting on emission rewards (fast-churn provers earn nothing)
- A quality multiplier (if your proof success rate drops below 95% in trailing 30 days, your emission is halved)

The whole point: **the only way to earn more PROVA is to honestly store more bytes for longer**.

---

## Operational discipline

- **Uptime matters.** Missed challenges → slashing. Hobby provers should aim for ≥ 99% proof success.
- **Disk health.** SMART failures cause missed proofs. Replace drives at the first warning.
- **Restarts.** Plan reboots between proof epochs. The dashboard shows the current epoch boundary.
- **Backups.** You don't need to back up the bytes (the network handles redundancy at the protocol level). But back up your `prover.toml` and the wallet's keystore.

---

## Next steps

- Read [Hardware](./hardware.md) for storage tier guidance
- Read [Earnings](./earnings.md) for the full math
- Move to [Prosumer](./prosumer.md) when you outgrow 100 TB
- Or jump to [Enterprise](./enterprise.md) if you're starting at multi-PB scale
