---
description: Stage bytes for storage on the Prova network.
---

# POST /api/upload

Stages a file on the Prova network.

```http
POST /api/upload?cid=baga…q4kr HTTP/1.1
Host: prova.network
Authorization: Bearer pk_live_eyJ...
Content-Type: application/octet-stream
X-Filename: my-site.tar.gz
Content-Length: 4194304

<raw bytes>
```

## Query parameters

| Param | Type | Required | Description |
| --- | --- | --- | --- |
| `cid` | string | yes | The content-addressed id of the bytes you're about to send. Must match the regex `^[a-z0-9]{8,80}$/i`. Compute it client-side. |

## Headers

| Header | Required | Description |
| --- | --- | --- |
| `Authorization` | optional | `Bearer pk_live_…`. If omitted, falls back to **sponsored** mode (anonymous, IP-rate-limited). |
| `Content-Type` | yes | Mime type of the bytes. Returned verbatim on retrieval. |
| `Content-Length` | yes | Body size in bytes. Used for cap enforcement. |
| `X-Filename` | no | Suggested filename for retrieval (`Content-Disposition`). URL-encoded. |

## Body

Raw bytes. The `Content-Type` header tells the server what they are.

## Response

```json
{
  "cid":          "baga…q4kr",
  "dealId":       "d-0x9c4f",
  "size":         4194304,
  "retrievalUrl": "https://prova.network/p/baga…q4kr",
  "term":         "30 days",
  "sponsored":    false,
  "owner":        "you@example.com"
}
```

* `dealId` — synthetic during pre-mainnet. After mainnet, it's the on-chain deal id from `StorageMarketplace.sol`.
* `sponsored` — `true` if the upload went through without a token (anonymous tier).
* `owner` — `null` for sponsored uploads, your email for authenticated.

## Tiers

| Tier | Auth | File size | Daily cap |
| --- | --- | --- | --- |
| Sponsored | none | 100 MB | 200 MB / IP |
| Free token | `pk_live_…` | 5 GiB | 1 GiB / day |
| Wallet | wallet sig | unlimited\* | quota = USDC balance |

\*Bounded by gas costs and prover capacity.

## Errors

| Status | `error` | When |
| --- | --- | --- |
| `400` | `invalid_cid` | `cid` query param missing or malformed |
| `400` | `no_body` | Request body is empty |
| `400` | `empty_body` | Body present but zero bytes |
| `401` | `invalid_token` | Auth header present but token invalid/expired |
| `403` | `insufficient_scope` | Token lacks the `put` scope |
| `413` | `too_large` | File exceeds the tier's per-file limit |
| `429` | `rate_limited` | Sponsored daily IP cap hit |
| `429` | `quota_exceeded` | Authenticated daily user quota hit |
| `502` | `stage_failed` | Upstream prover rejected the upload |
| `503` | `storage_offline` | No storage backend bound (configuration error) |

## Examples

### CLI / curl (with token)

```bash
TOKEN="pk_live_..."
CID=$(prova hash ./my-site.tar.gz)   # or compute manually
curl -X POST "https://prova.network/api/upload?cid=$CID" \
  -H "authorization: Bearer $TOKEN" \
  -H "content-type: application/octet-stream" \
  -H "x-filename: my-site.tar.gz" \
  --data-binary @./my-site.tar.gz
```

### Sponsored (no auth)

```bash
curl -X POST "https://prova.network/api/upload?cid=$CID" \
  -H "content-type: image/png" \
  -H "x-filename: screenshot.png" \
  --data-binary @./screenshot.png
```

### From a Web app

```js
const buf = await file.arrayBuffer();
const cid = await computeCid(buf);   // see /api/piece-cids docs

const res = await fetch(`/api/upload?cid=${cid}`, {
  method: 'POST',
  headers: {
    'authorization': `Bearer ${token}`,
    'content-type': file.type,
    'x-filename': encodeURIComponent(file.name),
  },
  body: file,
});
const { retrievalUrl } = await res.json();
```

## Streaming

The endpoint accepts streaming bodies. The Content-Length must be set up front (no chunked transfer-encoding for now). For files larger than 100 MB on the authenticated tier, the SDK splits into chunks and uploads each piece independently.
