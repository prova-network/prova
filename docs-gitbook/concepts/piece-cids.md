---
description: How content addressing works in Prova.
---

# Piece-CIDs

A **piece-cid** is the content-addressed identifier for a file stored on Prova. Two clients uploading the same bytes get the same piece-cid. Identical files always produce identical cids.

## Format

A piece-cid looks like:

```
baga6ea4reaqphi6dxycc2sand64gjaafwijxrekxnrlktw2zrtvpf62tk57l2bi
```

It's a **CIDv1** with:

- **multibase** prefix `b` (base32 lowercase, no padding) → `b…`
- **codec** `0xf101` (`fil-commitment-unsealed`) → printable bytes start with `aga…`
- **multihash** `0x1012` (`sha2-256-trunc254-padded`)
- **digest length** `0x20` (32 bytes)
- **digest** the 32-byte CommP

Concatenated and rendered, that's why every Prova piece-cid begins with `baga…`.

## How it's computed (CommP)

The digest is the root of a binary Merkle tree over the **Fr32-padded** bytes of the file:

1. **Fr32 padding.** Insert two zero bits after every 254 input bits. The expansion ratio is exactly 127:128 — every 127 input bytes become 128 padded bytes. The padding ensures every 32-byte chunk fits inside the BLS12-381 scalar field.
2. **Round up.** Pad the leaf count to the next power of two by appending zero leaves.
3. **Hash up.** Build a binary Merkle tree where every internal node is `SHA-256(left || right)` with the **top two bits of the digest's last byte cleared** (the `trunc254` step).
4. **Encode.** The root is the CommP digest. Wrap it in CIDv1 framing as above and base32-encode → `baga…`.

This is the same primitive used by Filecoin's PDP, [`go-fil-commp-hashhash`](https://github.com/filecoin-project/go-fil-commp-hashhash), and the [Filecoin specs](https://github.com/filecoin-project/specs). Prova reuses the math unchanged so any Filecoin tooling that recognizes a CommP digest also recognizes a Prova piece-cid.

## Why content addressing

Content addressing means the **identifier is derived from the bytes**, not assigned by some central registry. Three properties matter:

1. **Verifiable.** Anyone can recompute the cid from the bytes and check the prover is serving the right file. If the prover lies, you notice immediately.
2. **De-duplicating.** If you and a thousand other people upload the same file, the network stores one copy. You each get your own deal, but the bytes are shared.
3. **Permanent.** The cid never changes. As long as the bytes exist, the address resolves.

## How to compute a piece-cid

### From the CLI

```bash
prova put ./file.bin
# the CLI computes the cid client-side and prints it
```

### From the SDK

```ts
import { computePieceCid } from '@prova-network/sdk'
const cid = await computePieceCid(bytes)
```

### From scratch (Python reference)

```python
import hashlib, base64

def trunc254(d):
    out = bytearray(d)
    out[31] &= 0x3f
    return bytes(out)

def fr32_expand_127(input127: bytes) -> bytes:
    out = bytearray(128)
    for g in range(4):
        in_start = g * 254
        out_start = g * 256
        for bit in range(254):
            ib = in_start + bit
            if ib >= 1016:
                break
            v = input127[ib >> 3] & (1 << (ib & 7))
            if v:
                ob = out_start + bit
                out[ob >> 3] |= 1 << (ob & 7)
    return bytes(out)

def piece_cid(data: bytes) -> str:
    # Fr32-pad in 127-byte units, emit 32-byte leaves
    leaves = []
    off = 0
    while off + 127 <= len(data):
        padded = fr32_expand_127(data[off:off+127])
        leaves += [padded[j:j+32] for j in range(0, 128, 32)]
        off += 127
    if off < len(data):
        last = bytearray(127)
        last[:len(data) - off] = data[off:]
        padded = fr32_expand_127(bytes(last))
        leaves += [padded[j:j+32] for j in range(0, 128, 32)]

    # Round up to next power of two leaves, min 4
    target = max(1, 4)
    while target < len(leaves):
        target <<= 1
    while len(leaves) < target:
        leaves.append(bytes(32))

    # Merkle, with top-2-bits-cleared at every internal hash
    level = leaves
    while len(level) > 1:
        level = [trunc254(hashlib.sha256(level[i] + level[i+1]).digest())
                 for i in range(0, len(level), 2)]
    digest = level[0]

    # Wrap as CIDv1 + fil-commitment-unsealed + sha2-256-trunc254-padded
    cid = bytes([0x01]) + bytes([0x81, 0xe2, 0x03]) + bytes([0x91, 0x20]) + bytes([0x20]) + digest
    return 'b' + base64.b32encode(cid).decode().lower().rstrip('=')

# 64 zero bytes → baga6ea4reaqdomn3tgwgrh3g532zopskstnbrd2n3sxfqbze7rxt7vqn7veigmy
print(piece_cid(b'\x00' * 64))
```

The browser, server, and CLI all run identical implementations of this algorithm. They produce byte-identical CIDs for byte-identical inputs.

## Verify a retrieval

If you fetch a piece and want to confirm the prover served the right bytes:

```bash
curl -O https://prova.network/p/baga6ea4reaq...
prova hash ./baga6ea4reaq...
# should print the same cid
```

The retrieval response also includes an `x-prova-verified: 1` header. The stage server (and on mainnet, the prover) recomputed the piece-cid from the bytes at intake. If the bytes don't hash to the cid, the upload is rejected with HTTP 422 `cid_mismatch`.

## Why de-duplication is good (and slightly weird)

If you upload the same file as someone else, Prova doesn't double-charge the prover for storage. They store one copy. **But** each of you has your own deal — your own retention term, your own retrieval rights, your own escrow. So the prover earns from both deals while only spending the disk cost once. This is the right incentive: more clients on the same piece = more revenue per byte for the prover, encouraging cheaper pricing.

The only weird side effect: a malicious actor can upload **the same cid as you** to a different prover and "front-run" your storage. Doesn't matter — the bytes are the bytes. They didn't see your content, they just happened to know its hash. Two parties with the same hash can both store the same bytes; they end up with two independent deals on identical content. The fact that the cid is content-addressed makes this safe.
