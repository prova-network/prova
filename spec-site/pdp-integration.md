<div class="spec-meta">
  <span class="label">State</span> <span class="badge state-reliable">Reliable</span>
  <span class="label">Theory audit</span> <span class="badge audit-wip">In progress</span>
  <span class="label">Last updated</span> 2026-04-26
</div>

# 2.1 PDP integration

Prova's storage proof primitive is **Provable Data Possession** (PDP). This section specifies how PDP integrates with Prova's deal lifecycle and how piece-CIDs are computed and verified.

The cryptographic primitive is identical to Filecoin's PDP construction. Prova reuses the math and ports it to Base. We do not invent cryptography.

## 2.1.1 Piece-CID (CommP) {#piece-cid}

Each object stored on Prova is identified by its **piece-CID**, a binary Merkle root computed over the Fr32-padded bytes of the object using SHA-256 with truncation at every internal node.

### Format

A piece-CID is encoded as **CIDv1** with:

- **Multibase**: `b` (base32, lowercase, no padding)
- **Codec**: `0xf101` (`fil-commitment-unsealed`), varint-encoded as `0x81 0xe2 0x03`
- **Multihash function**: `0x1012` (`sha2-256-trunc254-padded`), varint `0x91 0x20`
- **Digest length**: `0x20` (32 bytes)
- **Digest**: 32-byte CommP root

The printable form for fil-commitment-unsealed CIDs MUST begin with `baga…`.

### Computation

Given an input byte string `data`:

1. **Fr32 pre-pad**: insert two zero bits after every 254 input bits. Practically: every 127 input bytes expand to 128 padded bytes.
2. **Round up**: pad the leaf count to the next power of two by appending zeroed leaves. Minimum 4 leaves (128-byte padded piece).
3. **Merkle hash up**: build a binary tree where each internal node is `SHA-256(left || right)` with the **top two bits of the digest's last byte cleared** (the `trunc254` step).
4. **Encode**: wrap the 32-byte root in CIDv1 framing as above and base32-encode.

### Reference implementations

Three independent implementations MUST produce byte-identical CIDs for the same input:

| Implementation | File |
| --- | --- |
| Browser (TypeScript) | [`website/upload/piece-cid.js`](https://github.com/prova-network/website/blob/main/upload/piece-cid.js) |
| Node CLI | [`cli/src/util/hash.mjs`](https://github.com/prova-network/cli/blob/main/src/util/hash.mjs) |
| Stage server (Python) | [`prova-stage-server.py`](https://github.com/prova-network/prova/blob/main/scripts/prova-stage-server.py) |

Conformance test vectors are in [`spec/test-vectors/piece-cid.json`](https://github.com/prova-network/prova/blob/main/spec/test-vectors/piece-cid.json) (when published).

### Properties

- **Self-describing**: codec + multihash give the algorithm; verifiers do not need out-of-band agreement.
- **Recomputable**: anyone holding the bytes can recompute and check.
- **Field-valid leaves**: the truncation keeps every digest inside the BLS12-381 scalar field.

## 2.1.2 Provable Data Possession

The prover holds the Fr32-padded bytes and the full Merkle tree. A verifier issues a **challenge** — an index `i` into the leaf array, derived from a verifiable on-chain randomness source.

The prover responds with:

- The leaf at index `i` (32 bytes)
- The `O(log N)` sibling hashes along the inclusion path

The verifier recomputes the root from the challenge response and compares it against the committed piece-CID. The proof passes iff they match.

### Soundness

The probability that a prover holding only fraction `(1 − δ)` of the bytes can answer a uniformly-random challenge is `1 − δ`. Repeated challenges over time compound this: the protocol detects sustained data loss with probability approaching 1 within a small number of challenges.

We rely solely on this primitive. We do not separately commit to a unique encoding of the data per replica; we do not require uniqueness or non-malleability beyond what content-addressing already provides.

## 2.1.3 Challenges and randomness

Challenges MUST use verifiable randomness derived from a recent block hash on Base. The prover MUST NOT be able to predict the challenge before it is issued.

### Challenge cadence

The default challenge interval is **30 seconds** at v1. If chain congestion makes this expensive at scale, governance MAY vote to change the cadence to 5 minutes or block-aligned. Cadence is set per-deal at acceptance time and MUST NOT be modified mid-deal.

### Challenge response window

A prover MUST respond to a challenge within `responseWindow` seconds (default: 600). Failure to respond within the window counts as a missed challenge, contributing to slashing thresholds.

## 2.1.4 Prover obligations

A prover that has accepted a deal MUST:

1. Recompute the piece-CID from the bytes received from the client. If it disagrees with the committed CID, the prover MUST refuse the deal before posting `acceptDeal`. This protects honest provers from clients who would commit to bytes they do not actually intend to upload.
2. Store the Fr32-padded bytes locally with sufficient redundancy that a single-disk failure does not cause a missed challenge.
3. Respond to every challenge within the response window.
4. Serve retrievals over HTTPS at `/piece/{cid}` with `x-prova-piece-cid` and `x-prova-verified` response headers.

## 2.1.5 Verifier obligations

The on-chain `ProofVerifier` MUST:

1. Accept proof submissions only from registered provers active in the deal at hand.
2. Recompute the Merkle root from the challenge response and compare to the committed piece-CID.
3. Emit a `ProofVerified(dataSetId, prover, challengeIndex)` event on success or `ProofFailed(dataSetId, prover, reason)` on failure.
4. Forward the result to the listener (the `StorageMarketplace`) via the listener interface.

## 2.1.6 Forks and upgrades

`ProofVerifier` is UUPS-upgradeable. An upgrade requires a 7-day governance timelock. Upgrades MUST NOT change the semantics of existing accepted proofs (i.e., a proof valid under v1 of the verifier MUST remain valid under v2). A breaking change requires a new contract address and a deal-migration plan.

## 2.1.7 Attribution

PDP integration is forked from [`FilOzone/pdp`](https://github.com/FilOzone/pdp) under the Permissive License Stack (Apache-2.0 OR MIT). Filecoin-specific FVM bindings are replaced with Base EVM equivalents; the cryptographic core is unchanged.
