/**
 * Keypair operations — mirrors sdk/src/lib.rs Keypair.
 */

import { sha256, concat } from './crypto.js';
import type { Keypair, Signature } from './types.js';

/** Derive a keypair from a 32-byte seed (deterministic). */
export function keypairFromSeed(seed: Uint8Array): Keypair {
  if (seed.length !== 32) throw new Error('seed must be 32 bytes');
  const addrHash = sha256(seed);
  const address = addrHash.slice(0, 20);
  return { secret: new Uint8Array(seed), address };
}

/** Sign a message with a keypair (HMAC-like, matching Rust implementation). */
export function sign(keypair: Keypair, message: Uint8Array): Signature {
  const h1 = sha256(concat(keypair.secret, message));
  const h2 = sha256(concat(h1, keypair.secret));
  const sig = new Uint8Array(64);
  sig.set(h1, 0);
  sig.set(h2, 32);
  return sig;
}

/** Verify a signature against a message and keypair. */
export function verify(keypair: Keypair, message: Uint8Array, signature: Signature): boolean {
  const expected = sign(keypair, message);
  if (expected.length !== signature.length) return false;
  // Constant-time comparison
  let diff = 0;
  for (let i = 0; i < expected.length; i++) diff |= expected[i] ^ signature[i];
  return diff === 0;
}

/** Create a test address from a single byte (for testing). */
export function testAddress(id: number): Uint8Array {
  const addr = new Uint8Array(20);
  addr[19] = id & 0xff;
  return addr;
}
