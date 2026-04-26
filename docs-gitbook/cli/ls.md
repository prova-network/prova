---
description: List your stored files.
---

# prova ls

Lists every file you've stored under your account, newest first.

```bash
$ prova ls
3 file(s):

  baga…q4kr     4.2 MB  4/25/2026, 16:41:00   my-site.tar.gz
  baga…3xyz   512.0 KB  4/24/2026, 10:15:22   screenshot.png
  baga…1abc    24.7 MB  4/20/2026, 09:11:22   genome.parquet
```

## Usage

```
prova ls
```

No flags. Prints all your files. Pagination ships in v1.

## Auth

Requires a signed-in session. Run `prova auth` first.

## Source

This is a thin wrapper around [`GET /api/files`](../api/files.md). For pretty output you can also use:

```bash
curl https://prova.network/api/files \
  -H "authorization: Bearer $PROVA_TOKEN" \
  | jq -r '.files[] | [.cid, .size, .filename] | @tsv'
```

## See also

* [`prova whoami`](whoami.md) — quota usage
* [`GET /api/files`](../api/files.md)
