---
description: Verifiable storage for humanity's most important digital memory.
---

# Welcome to Prova

Prova is verifiable storage anchored to Ethereum.

You give Prova a file. Prova chunks it, replicates it across independent **provers**, and pins it down with an on-chain deal. Every day, every prover proves the file is still there with a cryptographic proof of data possession. You retrieve it over HTTPS, IPFS, or libp2p. You pay in USDC on Base. If a prover misses a proof, they get slashed and your file is re-pinned to a healthy prover.

That's the whole loop.

## Three things people store on Prova today

* **Static websites.** Drop a build folder in, point your ENS contenthash at the returned `piece-cid`. Your site lives across the network, served over HTTPS, proven daily.
* **Datasets.** Upload a Parquet shard, a CSV, a research archive. Prova handles chunking, redundancy, and proof. Pay per TiB-month.
* **AI corpora.** Anchor model weights and training data so the corpus is auditable. Anyone can verify the file you trained on is the file you said.

## Three ways to use it

* [**Web upload**](getting-started/web-upload.md) — drag and drop in your browser, no install, no wallet. The fastest way to try Prova.
* [**CLI**](getting-started/cli.md) — `prova put ./dist`, get a `piece-cid` back. One line.
* [**HTTP API**](api/) — same endpoints from any language, any runtime.

For programmatic / on-chain workflows, the [TypeScript SDK](sdk/) gives you a typed surface.

## What's different about Prova

Prova is intentionally minimal. We are **not** building:

* a new chain
* sealed PoRep / TEE / fancy crypto theater
* a custodial backup service
* a payment-token UX (clients pay USDC, not PROVA)

Prova is **PDP on Base**. Clients pay in USDC. Provers stake PROVA and earn USDC. Slashing burns PROVA. The 1% protocol fee on USDC auto-buys and burns PROVA from market revenue. That's the whole loop.

If you want the full architecture, start at [How Prova works](concepts/architecture.md). If you want to use it, [start storing](getting-started/web-upload.md).
