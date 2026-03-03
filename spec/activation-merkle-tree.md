# Activation Merkle Tree — Specification

**Status:** Draft v0.1
**Date:** 2026-03-04

## 1. Purpose

The Activation Merkle Tree (AMT) captures the full execution trace of a neural network inference as a compact, verifiable commitment. It enables the QBP bisection protocol to isolate disputed layers without revealing the entire execution trace upfront.

## 2. Construction

### 2.1 Leaf Generation

Given a model M with L layers and input x:

```
h_0 = x                          // Input tensor
h_i = f_i(h_{i-1})  for i = 1..L // Layer outputs (activations)

leaf_i = SHA-256(canonical_serialize(h_i))  for i = 0..L
```

The tree has L+1 leaves (input + L layer outputs).

### 2.2 Canonical Tensor Serialization

All tensors MUST be serialized identically regardless of the runtime's internal memory layout.

```
canonical_serialize(tensor) -> bytes:
    header:
        magic:     4 bytes = 0x50524F56  ("PROV")
        version:   1 byte  = 0x01
        dtype:     1 byte  (0x01=INT8, 0x02=INT32, 0x03=FP16, 0x04=FP32)
        ndim:      1 byte  (number of dimensions, max 8)
        reserved:  1 byte  = 0x00
        shape:     ndim × 8 bytes (u64 little-endian per dimension)

    data:
        Elements in row-major (C) order, little-endian byte order.
        No padding between elements.
        Total data bytes = product(shape) × sizeof(dtype)

    No alignment padding. No trailing bytes.
```

**Example:** A FP32 tensor of shape [1, 2048, 4096]:
- Header: 4 + 1 + 1 + 1 + 1 + 3×8 = 32 bytes
- Data: 1 × 2048 × 4096 × 4 = 33,554,432 bytes
- Total: 33,554,464 bytes
- SHA-256 of this = one leaf

### 2.3 Merkle Tree Structure

Standard binary Merkle tree with SHA-256:

```
If leaves.len() is not a power of 2:
    Pad with SHA-256(0x00) to next power of 2

Internal nodes:
    node(left, right) = SHA-256(0x01 || left || right)

Leaf nodes:
    leaf(data) = SHA-256(0x00 || data)

Domain separation:
    0x00 prefix for leaves
    0x01 prefix for internal nodes
```

This prevents second-preimage attacks (a leaf cannot be confused with an internal node).

### 2.4 Proof Format

A Merkle proof for leaf at index `i`:

```
struct MerkleProof {
    leaf_index: u32,
    leaf_hash: Hash,
    siblings: Vec<(Hash, Side)>,  // Bottom-up, Left or Right
}

enum Side {
    Left,   // Sibling is on the left
    Right,  // Sibling is on the right
}
```

Verification:
```
fn verify(proof: &MerkleProof, root: Hash) -> bool {
    let mut current = SHA-256(0x00 || proof.leaf_hash);
    for (sibling, side) in &proof.siblings {
        current = match side {
            Left  => SHA-256(0x01 || sibling || current),
            Right => SHA-256(0x01 || current || sibling),
        };
    }
    current == root
}
```

## 3. Performance

### 3.1 Hashing Cost

SHA-256 throughput on modern hardware: ~2 GB/s (single core), ~10 GB/s (multi-threaded).

| Model Size | Layers | Avg Activation Size | Total Data | Hash Time (1 core) | Hash Time (4 cores) |
|---|---|---|---|---|---|
| TinyLlama 1B | 22 | ~8 MB | ~176 MB | 88 ms | 22 ms |
| Llama 3 8B | 32 | ~32 MB | ~1 GB | 500 ms | 125 ms |
| Llama 3 70B | 80 | ~64 MB | ~5.1 GB | 2.6 s | 650 ms |
| Llama 3 405B | 126 | ~128 MB | ~16 GB | 8 s | 2 s |

### 3.2 Storage Cost

Only the Merkle root (32 bytes) is committed on-chain. Full tree is stored off-chain by the prover, revealed only during disputes.

Off-chain storage per inference:
- Leaf hashes: (L+1) × 32 bytes
- Internal nodes: ~L × 32 bytes
- **Total: ~2L × 32 bytes** (negligible compared to activation tensors)

If activation tensors themselves must be retained (for bisection reveal):
- **Total: L × avg_activation_size**
- For 70B model: 80 × 64 MB = 5.1 GB per inference
- Retention period: challenge window duration only

### 3.3 Proof Size

Merkle proof for one leaf:
- log₂(L+1) siblings × 32 bytes each
- For 80-layer model: 7 × 32 = 224 bytes
- Negligible for on-chain submission

## 4. Optimizations

### 4.1 Lazy Hashing

During inference, hash each layer's output immediately after it's computed (before the next layer starts). This overlaps hashing with GPU→CPU transfer time and avoids storing all activations simultaneously.

```
Pipeline:
  GPU computes layer i → transfer h_i to CPU → hash h_i → GPU computes layer i+1
                                                ↓
                                         store leaf_i
```

### 4.2 Streaming Merkle Root

The Merkle tree can be built incrementally as leaves arrive:

```
fn streaming_merkle_build():
    stack = []
    for each leaf:
        node = hash_leaf(leaf)
        while stack.last().depth == node.depth:
            sibling = stack.pop()
            node = hash_internal(sibling, node)
        stack.push(node)
    // Pad and finalize
    while stack.len() > 1:
        ...
    return stack[0]  // root
```

Memory: O(log L) rather than O(L).

### 4.3 Chunked Activation Hashing

For very large activations (>100 MB), hash in chunks using SHA-256's streaming interface rather than materializing the entire serialized tensor in memory:

```
fn hash_activation_chunked(tensor, chunk_size=1MB):
    hasher = SHA256::new()
    hasher.update(header)
    for chunk in tensor.iter_chunks(chunk_size):
        hasher.update(to_le_bytes(chunk))
    return hasher.finalize()
```

## 5. Test Vectors

### 5.1 Canonical Serialization

**Input:** FP32 tensor, shape [2, 3], data [[1.0, 2.0, 3.0], [4.0, 5.0, 6.0]]

**Expected serialization (hex):**
```
50524F56           # magic "PROV"
01                 # version 1
04                 # dtype FP32
02                 # ndim 2
00                 # reserved
0200000000000000   # shape[0] = 2
0300000000000000   # shape[1] = 3
0000803F           # 1.0f LE
00000040           # 2.0f LE
00004040           # 3.0f LE
00008040           # 4.0f LE
0000A040           # 5.0f LE
0000C040           # 6.0f LE
```

**SHA-256 of above:** (to be computed by reference implementation)

### 5.2 Merkle Tree (4 leaves)

```
Leaves: [H0, H1, H2, H3]

                    Root
                   /    \
            Node01        Node23
           /     \       /     \
     Leaf(H0) Leaf(H1) Leaf(H2) Leaf(H3)

Leaf(Hi) = SHA-256(0x00 || Hi)
Node01   = SHA-256(0x01 || Leaf(H0) || Leaf(H1))
Node23   = SHA-256(0x01 || Leaf(H2) || Leaf(H3))
Root     = SHA-256(0x01 || Node01 || Node23)
```

Proof for H2: [Leaf(H3), Right], [Node01, Left]
