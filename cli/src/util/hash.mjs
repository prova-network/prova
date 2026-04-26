// Real piece-CID (CommP) computation, mirroring website/upload/piece-cid.js
// and stage server's compute_piece_cid_from_bytes.
//
// Pipeline:
//   1. Fr32-pre-pad: insert two zero bits after every 254 input bits.
//      Practically: every 127 bytes of input → 128 bytes of output.
//   2. Round leaf count up to next power-of-two; pad with zeroed leaves.
//   3. Build SHA-256 binary Merkle tree, with the top 2 bits of every
//      internal-node digest cleared (sha2-256-trunc254-padded).
//   4. The root is the CommP digest.
//   5. Encode as CIDv1 + fil-commitment-unsealed (0xf101) + sha2-256-trunc254-padded (0x1012).
//      Render base32 lower no-pad with multibase prefix 'b' → "baga..." string.
//
// Spec references: filecoin-project/specs and FilOzone/synapse-sdk.

import { createHash } from 'node:crypto';

const ALPHABET = 'abcdefghijklmnopqrstuvwxyz234567';

function base32LowerNoPad(bytes) {
  let bits = 0, val = 0, out = '';
  for (const b of bytes) {
    val = (val << 8) | b;
    bits += 8;
    while (bits >= 5) {
      out += ALPHABET[(val >>> (bits - 5)) & 31];
      bits -= 5;
    }
  }
  if (bits) out += ALPHABET[(val << (5 - bits)) & 31];
  return out;
}

function nextPow2(n) {
  if (n <= 1) return 1;
  let p = 1;
  while (p < n) p <<= 1;
  return p;
}

function trunc254(digest) {
  digest[31] &= 0x3f;
  return digest;
}

function fr32Expand127(input127) {
  if (input127.byteLength !== 127) throw new Error('fr32Expand127 expects 127 bytes');
  const out = new Uint8Array(128);
  const totalInBits = 1016;
  const bitsPerGroup = 254;
  for (let g = 0; g < 4; g++) {
    const inStart = g * bitsPerGroup;
    const outStart = g * (bitsPerGroup + 2);
    for (let bit = 0; bit < bitsPerGroup; bit++) {
      const ib = inStart + bit;
      if (ib >= totalInBits) break;
      const iByte = ib >> 3;
      const iMask = 1 << (ib & 7);
      const v = (input127[iByte] & iMask) ? 1 : 0;
      const ob = outStart + bit;
      const oByte = ob >> 3;
      const oMask = 1 << (ob & 7);
      if (v) out[oByte] |= oMask;
    }
  }
  return out;
}

function encodeFilCommP(digest32) {
  if (digest32.byteLength !== 32) throw new Error('digest must be 32 bytes');
  const out = new Uint8Array(1 + 3 + 2 + 1 + 32);
  let i = 0;
  out[i++] = 0x01;                   // CIDv1
  out[i++] = 0x81; out[i++] = 0xe2; out[i++] = 0x03; // 0xf101 codec
  out[i++] = 0x91; out[i++] = 0x20;  // 0x1012 hash function
  out[i++] = 0x20;                   // 32-byte digest
  out.set(digest32, i);
  return 'b' + base32LowerNoPad(out);
}

/**
 * Compute the piece-CID of a buffer.
 * @param {Uint8Array | Buffer} buffer
 * @returns {Promise<string>} the piece-CID string (baga...)
 */
export async function computeCid(buffer) {
  if (!buffer || buffer.byteLength === 0) {
    throw new Error('cannot piece-CID an empty buffer');
  }
  const u8 = buffer instanceof Uint8Array ? buffer : new Uint8Array(buffer);

  // Step 1+2: Fr32-pre-pad in 127-byte units, emit 32-byte leaves
  const leaves = [];
  let off = 0;
  while (off + 127 <= u8.byteLength) {
    const padded = fr32Expand127(u8.subarray(off, off + 127));
    leaves.push(padded.subarray(0, 32));
    leaves.push(padded.subarray(32, 64));
    leaves.push(padded.subarray(64, 96));
    leaves.push(padded.subarray(96, 128));
    off += 127;
  }
  if (off < u8.byteLength) {
    const last = new Uint8Array(127);
    last.set(u8.subarray(off));
    const padded = fr32Expand127(last);
    leaves.push(padded.subarray(0, 32));
    leaves.push(padded.subarray(32, 64));
    leaves.push(padded.subarray(64, 96));
    leaves.push(padded.subarray(96, 128));
  }

  // Round to next power of two (minimum 4 leaves = 128-byte padded)
  const target = Math.max(nextPow2(leaves.length), 4);
  while (leaves.length < target) leaves.push(new Uint8Array(32));

  // Step 3: Merkle tree with trunc254 at every internal hash
  let level = leaves;
  while (level.length > 1) {
    const next = new Array(level.length / 2);
    for (let i = 0; i < level.length; i += 2) {
      const concat = new Uint8Array(64);
      concat.set(level[i], 0);
      concat.set(level[i + 1], 32);
      const digest = createHash('sha256').update(concat).digest();
      next[i / 2] = trunc254(digest);
    }
    level = next;
  }

  return encodeFilCommP(level[0]);
}
