---
description: Store your first file in 60 seconds.
---

# Quickstart in 60 seconds

Three paths. Pick one.

## Path A — Browser (no install)

1. Open [prova.network/upload](https://prova.network/upload/).
2. Drag a file in.
3. Done. Copy the `piece-cid` and the retrieval URL.

That's the whole quickstart. Read [Web upload](web-upload.md) for the limits.

## Path B — CLI

```bash
# Install
npm i -g @prova-network/cli
# or: curl -fsSL https://get.prova.network | sh

# Sign in (email only, no password)
prova auth

# Store
prova put ./dist.tar.gz

# Retrieve
prova get baga…q4kr -o ./dist.tar.gz
```

## Path C — HTTP API

```bash
# 1. Mint a token
TOKEN=$(curl -s -X POST https://prova.network/api/auth/signup \
  -H "content-type: application/json" \
  -d '{"email":"you@example.com","label":"my-app"}' \
  | jq -r .token)

# 2. Hash your file (sha-256 → base32)
HASH=$(shasum -a 256 ./file.bin | awk '{print $1}')
CID="baga<base32-of-$HASH-truncated-to-52-chars>"

# 3. Upload
curl -X POST "https://prova.network/api/upload?cid=$CID" \
  -H "authorization: Bearer $TOKEN" \
  -H "content-type: application/octet-stream" \
  -H "x-filename: file.bin" \
  --data-binary @./file.bin

# 4. Retrieve
curl -O https://prova.network/p/$CID
```

(In practice, use the [SDK](../sdk/) so you don't have to do the cid math yourself.)

## What you've just done

* Created a Prova account tied to your email.
* Stored a file under a content-addressed `piece-cid`.
* Got a permanent retrieval URL (for the term of the deal).
* Locked a Prova prover into a daily proof obligation. They'll prove your file is still there every 30 seconds for the next 30 days.

## Where to go next

* [How Prova works](../concepts/architecture.md) — the three roles, the deal lifecycle.
* [Authentication](../api/authentication.md) — token scopes, rotation, revocation.
* [Building on Prova](../sdk/) — the TypeScript SDK.
