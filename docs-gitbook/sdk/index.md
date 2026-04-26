---
description: Programmatic access from TypeScript.
---

# TypeScript SDK

`@prova-network/sdk` is the typed surface for Prova. Use it when you want programmatic access — building a marketplace, an indexer, an upload pipeline in your own app, or anything that talks directly to the chain.

## Install

```bash
npm i @prova-network/sdk viem
```

`viem` is a peer dependency.

## Quickstart

```ts
import { Prova } from '@prova-network/sdk'
import { createWalletClient, http } from 'viem'
import { base } from 'viem/chains'
import { privateKeyToAccount } from 'viem/accounts'

const account = privateKeyToAccount(process.env.PRIVATE_KEY as `0x${string}`)
const wallet = createWalletClient({ account, chain: base, transport: http() })

const prova = Prova.create({ account: wallet.account, chain: base })

// Upload
const result = await prova.storage.upload(bytes)
console.log(result.cid, result.dealId)

// Download
const bytes = await prova.storage.download({ pieceCid: result.cid })
```

## What's inside

The SDK splits into several services on the `Prova` instance:

| Service | Module | Purpose |
| --- | --- | --- |
| `prova.storage` | [Storage](storage.md) | Upload, download, list, manage redundancy |
| `prova.payments` | [Payments](payments.md) | Top up, escrow, withdraw |
| `prova.providers` | (coming) | Discover provers, read reputation |
| `prova.deals` | (coming) | Inspect on-chain deals directly |

## When to use the SDK vs. the API

* **SDK** — you're writing TypeScript, want types, want on-chain access, want efficient retries, want streaming uploads.
* **HTTP API** — you're in another language, doing one-off scripts, integrating from a backend without a wallet.
* **CLI** — you're a human at a shell.

The SDK is the only path that gives you direct chain access (no Prova middleware in the way). For programmatic clients building on Prova as a primitive, this is the right surface.

## Source + license

[github.com/prova-network/prova/tree/main/sdk/typescript](https://github.com/prova-network/prova/tree/main/sdk/typescript)

Apache-2.0 OR MIT. Forked from FilOzone/synapse-sdk with attribution preserved.
