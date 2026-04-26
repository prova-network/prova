---
description: Run a Prova prover. Earn USDC.
---

# Become a prover

A Prova **prover** stores client files and posts cryptographic proofs that the bytes are still there. In return, the prover earns a slice of the deal's payment for every successful proof.

This page is for operators. If you're a client, see [Welcome](../README.md).

## Quick reality check

Before you set up a prover, sanity-check the economics:

* **Capacity:** typical solo prover starts at 1–10 TB. You can scale to PB if you want.
* **Bandwidth:** a real symmetric link. You're fetching client uploads and serving retrievals.
* **Stake:** you must lock USDC equal to a percentage of the bytes you commit. If you fail proofs, this gets slashed.
* **Net margin:** ~$0.40-0.80 per TiB-month after fees, before electricity. See [Earnings](earnings.md) for the calculator.

It's a real business. Treat it like one.

## Install

```bash
# Linux / macOS
curl -fsSL https://get.prova.network/prover | sh
```

This drops `provad` in `/usr/local/bin` and creates `/etc/prova/prover.toml` with a starter config.

(Alternatively, build from source: `git clone github.com/prova-network/prova && cd prover && go build ./cmd/provad`.)

## Configure

Edit `/etc/prova/prover.toml`:

```toml
[identity]
keystore_path = "/etc/prova/keystore.json"
# Or for local testing only: private_key_hex = "0x..."

[chain]
rpc_url = "https://mainnet.base.org"
chain_id = 8453

[chain.contracts]
storage_marketplace = "0x..."
prover_registry     = "0x..."
prover_staking      = "0x..."

[storage]
data_dir = "/var/lib/prova/pieces"
max_bytes = 10_000_000_000_000   # 10 TB

[http]
enabled = true
listen_addr = "0.0.0.0:443"
public_url = "https://prover.example.com"
cert_path = "/etc/letsencrypt/live/prover.example.com/fullchain.pem"
key_path  = "/etc/letsencrypt/live/prover.example.com/privkey.pem"

[dashboard]
enabled = true
listen_addr = "127.0.0.1:8080"
```

## Register on-chain

```bash
provad --config /etc/prova/prover.toml register \
  --capacity 10TB \
  --price-per-tib-month 4.00 \
  --regions "EU"
```

This submits a `registerProver(...)` transaction and stakes USDC equal to your committed capacity. The amount needed depends on the contract's `STAKE_RATIO` parameter.

## Start

```bash
systemctl enable --now provad
```

Or run directly:

```bash
provad --config /etc/prova/prover.toml start
```

The daemon:

1. Watches Base for `DealProposed` events targeting your address
2. Fetches client bytes from each deal's `sourceURL`
3. Verifies the cid matches
4. Posts the first proof, then every 30 seconds
5. Serves retrievals on `/piece/{cid}` over HTTPS

## Status

```bash
provad --config /etc/prova/prover.toml status
```

```
prover     0xYourAddress
state      active
capacity   10.0 TB committed, 4.2 TB stored (42%)
deals      127 active, 8 settled, 0 slashed
proofs     last posted 12s ago, 99.94% success (last 7d)
balance    1,237.40 USDC available, 4,000.00 USDC staked
```

Or use the local dashboard at `http://localhost:8080`.

## Operations

* **Logs:** `journalctl -u provad -f`
* **Scale up:** edit `max_bytes` in config, run `provad register --capacity <new>` to expand commitment + stake
* **Scale down:** can only reduce capacity by un-pinning Settled deals. Active deals are obligations.
* **Move servers:** export the keystore, copy the `/var/lib/prova/pieces` directory, and start `provad` on the new box. The on-chain identity stays the same.

## Source

[github.com/prova-network/prova/tree/main/prover](https://github.com/prova-network/prova/tree/main/prover)

The prover is Go, Apache-2.0 OR MIT. Forked from FilOzone's curio with attribution preserved, adapted for Base + ETH/USDC instead of Filecoin/FIL.
