---
description: Upload, download, and manage stored pieces.
---

# Storage

The `prova.storage` service handles every operation that involves bytes.

```ts
const prova = Prova.create({ account, chain: base })
```

## Upload

```ts
const result = await prova.storage.upload(bytes, {
  redundancy: 4,
  termDays: 30,
  callbacks: {
    onProviderSelected: (p) => console.log('chose', p.address),
    onProgress: (loaded, total) => console.log(`${loaded}/${total}`),
    onStored: (cid) => console.log('stored', cid),
  },
})

// result: { cid, dealId, size, retrievalUrl, pieces }
```

Options:

| Option | Type | Default | Description |
| --- | --- | --- | --- |
| `redundancy` | number | 2 | How many independent provers to pin to. |
| `termDays` | number | 30 | How long the deal stays Active. |
| `regions` | string[] | auto | Constrain to specific regions (`'EU' | 'NA' | 'AP' | ...`). |
| `providerIds` | bigint[] | auto | Pin to specific providers by id. |
| `pieceCid` | string | computed | Skip CommP computation, trust this cid. |
| `signal` | AbortSignal | none | Cancel mid-upload. |
| `callbacks` | object | none | Lifecycle callbacks. |

## Download

```ts
const bytes = await prova.storage.download({ pieceCid: 'baga…q4kr' })
```

Returns a `Uint8Array`. The SDK retries against multiple replicas if the first prover fails.

```ts
// Download via a specific provider
const bytes = await prova.storage.download({
  pieceCid: 'baga…',
  providerId: 7n,
})

// Stream instead of buffering
const stream = await prova.storage.downloadStream({ pieceCid: 'baga…' })
for await (const chunk of stream) {
  // ...
}
```

## List

```ts
const files = await prova.storage.list({ owner: '0xYourAddress' })
// → [{ cid, size, dealIds, providerIds, term, ... }, ...]
```

## Pin / unpin

Add or remove a replica from an existing piece:

```ts
// Add another replica
await prova.storage.pin({ pieceCid: 'baga…', additionalReplicas: 2 })

// Drop a specific provider's replica
await prova.storage.unpin({ pieceCid: 'baga…', providerId: 5n })
```

## Compute a piece-cid without uploading

```ts
import { computePieceCid } from '@prova-network/sdk'
const cid = await computePieceCid(bytes)
```

Useful for pre-flighting deal proposals or de-duplication checks.

## Costs

```ts
const quote = await prova.storage.getUploadCosts({
  bytes,
  redundancy: 4,
  termDays: 365,
})
// → { totalUSDC, perReplicaUSDC, gasUSDC, breakdown }
```

Always quote before uploading, especially for large files or long terms.

## See also

* [`POST /api/upload`](../api/upload.md) — same operation via HTTP
* [Architecture](../concepts/architecture.md) — what these operations map to on-chain
