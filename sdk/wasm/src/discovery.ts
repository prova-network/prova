/**
 * Provider discovery and ranking — mirrors Rust ProviderDiscovery.
 */

import { toHex } from './crypto.js';
import type { ModelId, ProviderInfo } from './types.js';

function modelKey(m: ModelId): string { return toHex(m); }

export class ProviderDiscovery {
  private providers: ProviderInfo[] = [];

  addProvider(info: ProviderInfo): void {
    this.providers.push(info);
  }

  /** Find providers serving a model within max price, sorted by score (desc). */
  findProviders(model: ModelId, maxPrice: bigint): ProviderInfo[] {
    const key = modelKey(model);
    return this.providers
      .filter(p => p.models.some(m => modelKey(m) === key) && p.price <= maxPrice)
      .sort((a, b) => {
        const scoreA = a.reputation * Number(a.stake) / Number(a.price || 1n);
        const scoreB = b.reputation * Number(b.stake) / Number(b.price || 1n);
        return scoreB - scoreA;
      });
  }

  /** Cheapest provider for a model. */
  cheapest(model: ModelId): ProviderInfo | undefined {
    const key = modelKey(model);
    return this.providers
      .filter(p => p.models.some(m => modelKey(m) === key))
      .sort((a, b) => Number(a.price - b.price))[0];
  }

  /** Highest-reputation provider for a model. */
  bestReputation(model: ModelId): ProviderInfo | undefined {
    const key = modelKey(model);
    return this.providers
      .filter(p => p.models.some(m => modelKey(m) === key))
      .sort((a, b) => b.reputation - a.reputation)[0];
  }

  /** Total registered providers. */
  get count(): number { return this.providers.length; }
}
