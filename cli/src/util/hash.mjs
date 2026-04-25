// Browser-compatible "piece-cid" stub matching the upload page's hash.
// SHA-256 -> base32, prefixed with bafy. Real CommP swap-in TBD.

import { createHash } from 'node:crypto';

const ALPHABET = 'abcdefghijklmnopqrstuvwxyz234567';

function base32(bytes) {
  let bits = 0, val = 0, out = '';
  for (const b of bytes) {
    val = (val << 8) | b;
    bits += 8;
    while (bits >= 5) {
      out += ALPHABET[(val >> (bits - 5)) & 31];
      bits -= 5;
    }
  }
  if (bits) out += ALPHABET[(val << (5 - bits)) & 31];
  return out;
}

export async function computeCid(buffer) {
  const hash = createHash('sha256').update(buffer).digest();
  return 'bafy' + base32(new Uint8Array(hash)).slice(0, 52);
}
