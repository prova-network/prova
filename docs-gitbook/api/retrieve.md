---
description: Stream stored bytes back from the Prova network.
---

# GET /p/{cid}

Streams the stored bytes for a piece-cid. Public — no authentication required.

```http
GET /p/baga…q4kr HTTP/1.1
Host: prova.network
```

## Path parameters

| Param | Description |
| --- | --- |
| `cid` | The piece-cid you got back from `POST /api/upload`. |

## Response

`200 OK` with the stored bytes. Headers:

```http
Content-Type: image/png
Content-Length: 4194304
Content-Disposition: inline; filename="screenshot.png"
Cache-Control: public, max-age=3600
X-Prova-Piece-Cid: baga…q4kr
X-Prova-Source: r2 | stage
Access-Control-Allow-Origin: *
```

The `X-Prova-Source` header tells you where the bytes came from:
* `r2` — from Cloudflare R2 (production)
* `stage` — from the testnet staging server

## Errors

| Status | When |
| --- | --- |
| `400` | `cid` doesn't match the expected format |
| `404` | No piece stored under that cid |
| `503` | Storage backend offline |

## Examples

### curl

```bash
curl -O https://prova.network/p/baga…q4kr
```

### Browser (inline image)

```html
<img src="https://prova.network/p/baga…q4kr" />
```

### As an ENS contenthash

```bash
ens set-content yoursite.eth ipfs://baga…q4kr
```

(The cid format in `ipfs://` URIs is the same as Prova's piece-cid for files small enough to fit in a single piece.)

## Caching

Responses are cached at Cloudflare's edge for 1 hour. The cid is content-addressed, so cache invalidation is never needed — the same cid always returns the same bytes.

## Range requests

Range requests (`Range: bytes=0-1023`) will be supported in v1 of the API. Currently, retrieval is whole-file only.

## Verify the bytes

Once you have the file, you can verify the prover served the right content:

```bash
curl -O https://prova.network/p/baga…q4kr
# Recompute the cid
shasum -a 256 ./baga…q4kr
# Compare to what you uploaded
```

If the recomputed cid doesn't match, the prover lied (or there's a bug). [File a bug report](https://github.com/prova-network/prova/issues).
