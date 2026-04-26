---
description: Upload a file.
---

# prova put

Uploads a file to Prova. Hashes locally, streams to the network, prints back a `piece-cid` and retrieval URL.

```bash
$ prova put ./my-site.tar.gz
hashing  my-site.tar.gz (4.2 MB)…       ✓ baga…q4kr
staging  bytes for prover…              ✓ 712 ms

Stored.
  piece-cid : baga…q4kr
  deal-id   : d-0x9c4f
  size      : 4.2 MB
  paid      : free quota
  term      : 30 days
  retrieve  : https://prova.network/p/baga…q4kr

Verify with: curl -O https://prova.network/p/baga…q4kr
```

## Usage

```
prova put <file>
```

Pass a path to a regular file. Directories aren't supported yet — tar them first:

```bash
tar -czf site.tar.gz ./dist
prova put ./site.tar.gz
```

## Tiers

The CLI auto-detects whether you're signed in:

| State | Tier | File limit | Daily quota |
| --- | --- | --- | --- |
| `~/.prova/config.json` exists with valid token | Authenticated | 5 GiB | 1 GiB / day (default) |
| No token | Sponsored | 100 MB | 200 MB / IP / 24h |

If a file exceeds the limit, the CLI prints a clear error and suggests `prova auth`.

## Pipeline

```
hashing   → SHA-256 the bytes locally, print the resulting cid
staging   → POST /api/upload?cid=<cid>, stream the body
```

The `staging` step's wall-clock time depends on your upload bandwidth, not Prova's prover speed. The prover does the actual chain interaction asynchronously.

## Exit codes

| Code | Meaning |
| --- | --- |
| `0` | Success |
| `1` | File not found / not a file / network error |
| `2` | Bad arguments |

## Environment variables

| Variable | Effect |
| --- | --- |
| `PROVA_TOKEN` | Override token from `~/.prova/config.json`. Useful in CI. |
| `PROVA_API` | Override API base URL. |
| `PROVA_DEBUG` | Print full stack traces on error. |

## See also

* [`prova get`](get.md)
* [`prova ls`](ls.md)
* [`POST /api/upload`](../api/upload.md)
