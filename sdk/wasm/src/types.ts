/**
 * Core Prova types — mirrors chain/src/types.rs.
 */

/** 20-byte address. */
export type Address = Uint8Array; // length 20

/** 32-byte hash / model ID. */
export type Hash = Uint8Array; // length 32
export type ModelId = Uint8Array; // length 32

/** Epoch number (block height). */
export type Epoch = number;

/** Job identifier. */
export type JobId = number;

/** 64-byte signature. */
export type Signature = Uint8Array; // length 64

/** A keypair (simplified Ed25519-style, matching Rust SDK). */
export interface Keypair {
  secret: Uint8Array; // 32 bytes
  address: Address;   // 20 bytes
}

/** A job request (matches Rust JobRequest). */
export interface JobRequest {
  id: JobId;
  requester: Address;
  modelId: ModelId;
  maxPrice: bigint;
  inputHash: Hash;
  deadline: Epoch;
  submittedAt: Epoch;
}

/** A signed inference request ready for submission. */
export interface SignedRequest {
  request: JobRequest;
  input: Uint8Array;
  signature: Signature;
}

/** Parsed inference result from a completed job. */
export interface InferenceResult {
  jobId: JobId;
  provider: Address;
  activationRoot: Hash;
  outputHash: Hash;
  epochCompleted: Epoch;
}

/** Provider info for discovery. */
export interface ProviderInfo {
  address: Address;
  models: ModelId[];
  price: bigint;
  reputation: number;
  stake: bigint;
}

/** SDK error types. */
export type SdkErrorKind =
  | 'MISSING_FIELD'
  | 'INVALID_SIGNATURE'
  | 'PROVIDER_NOT_FOUND'
  | 'JOB_NOT_FOUND'
  | 'TIMEOUT';

export class SdkError extends Error {
  constructor(public readonly kind: SdkErrorKind, message: string) {
    super(message);
    this.name = 'SdkError';
  }
}
