/**
 * High-level Prova client — mirrors Rust ProvaClient.
 */

import { ProviderDiscovery } from './discovery.js';
import type { Keypair, Address, JobId, SignedRequest, InferenceResult } from './types.js';

export class ProvaClient {
  readonly keypair: Keypair;
  readonly discovery = new ProviderDiscovery();
  private pending = new Map<JobId, SignedRequest>();
  private results = new Map<JobId, InferenceResult>();
  private nextNonce = 0;

  constructor(keypair: Keypair) { this.keypair = keypair; }

  /** Submit a signed request. Returns assigned job ID. */
  submit(signed: SignedRequest): JobId {
    const id = this.nextNonce++;
    this.pending.set(id, signed);
    return id;
  }

  /** Record a completed result. */
  recordResult(result: InferenceResult): void {
    this.pending.delete(result.jobId);
    this.results.set(result.jobId, result);
  }

  isPending(id: JobId): boolean { return this.pending.has(id); }
  getResult(id: JobId): InferenceResult | undefined { return this.results.get(id); }
  get pendingCount(): number { return this.pending.size; }
  get completedCount(): number { return this.results.size; }

  /** Cancel a pending job. Returns true if it was pending. */
  cancel(id: JobId): boolean { return this.pending.delete(id); }

  /** Client's on-chain address. */
  get address(): Address { return this.keypair.address; }
}

/** Batch-submit multiple requests. */
export function batchSubmit(client: ProvaClient, requests: SignedRequest[]): JobId[] {
  return requests.map(r => client.submit(r));
}
