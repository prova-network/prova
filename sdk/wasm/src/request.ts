/**
 * Inference request builder — mirrors Rust InferenceRequestBuilder.
 */

import { sha256, concat } from './crypto.js';
import { sign, verify } from './keypair.js';
import type { Keypair, ModelId, Epoch, SignedRequest, JobRequest, Signature } from './types.js';
import { SdkError } from './types.js';

export class InferenceRequestBuilder {
  private _modelId?: ModelId;
  private _input?: Uint8Array;
  private _maxPrice: bigint = 0n;
  private _deadlineEpochs: number = 100;

  model(id: ModelId): this { this._modelId = id; return this; }
  input(data: Uint8Array): this { this._input = data; return this; }
  maxPrice(price: bigint): this { this._maxPrice = price; return this; }
  deadline(epochs: number): this { this._deadlineEpochs = epochs; return this; }

  /** Build and sign the request. */
  build(keypair: Keypair, currentEpoch: Epoch): SignedRequest {
    if (!this._modelId) throw new SdkError('MISSING_FIELD', 'model_id required');
    if (!this._input) throw new SdkError('MISSING_FIELD', 'input required');

    const inputHash = sha256(this._input);
    const request: JobRequest = {
      id: 0, // assigned by scheduler
      requester: keypair.address,
      modelId: this._modelId,
      maxPrice: this._maxPrice,
      inputHash,
      deadline: currentEpoch + this._deadlineEpochs,
      submittedAt: currentEpoch,
    };

    const msg = serializeRequestForSigning(request);
    const signature = sign(keypair, msg);

    return { request, input: this._input, signature };
  }
}

/** Serialize request fields for signing (matches Rust byte layout). */
export function serializeRequestForSigning(req: JobRequest): Uint8Array {
  const priceBuf = new Uint8Array(16);
  const dv = new DataView(priceBuf.buffer);
  // Little-endian u128 (write low 8 bytes, high 8 bytes)
  dv.setBigUint64(0, req.maxPrice & 0xFFFFFFFFFFFFFFFFn, true);
  dv.setBigUint64(8, req.maxPrice >> 64n, true);

  const deadlineBuf = new Uint8Array(8);
  const dv2 = new DataView(deadlineBuf.buffer);
  dv2.setBigUint64(0, BigInt(req.deadline), true);

  return concat(req.requester, req.modelId, req.inputHash, priceBuf, deadlineBuf);
}

/** Verify a signed request's signature. */
export function verifySignedRequest(signed: SignedRequest, keypair: Keypair): boolean {
  const msg = serializeRequestForSigning(signed.request);
  return verify(keypair, msg, signed.signature);
}
