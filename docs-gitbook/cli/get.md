---
description: Retrieve a stored file by cid.
---

# prova get

Streams a stored piece back to stdout, or to a file with `-o`.

```bash
prova get baga…q4kr -o ./out.bin
```

## Usage

```
prova get <cid> [-o <output-file>]
```

| Flag | Description |
| --- | --- |
| `-o`, `--output` | Write the bytes to a file instead of stdout. |

## Examples

```bash
# Stream to stdout (good for text)
prova get baga…q4kr

# Pipe to less / xxd
prova get baga…q4kr | xxd | head

# Save to disk
prova get baga…q4kr -o ./recovered.tar.gz

# Verify the prover served the right bytes
prova get baga…q4kr -o /tmp/x
shasum -a 256 /tmp/x
# (compare to the expected cid)
```

## Auth

`prova get` does **not** require auth. Retrieval is public for any cid that exists on the network.

## See also

* [`prova put`](put.md)
* [`GET /p/{cid}`](../api/retrieve.md)
