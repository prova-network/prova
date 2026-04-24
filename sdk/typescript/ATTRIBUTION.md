# Attribution — TypeScript SDK Sources

The Prova TypeScript SDK is derived from [`FilOzone/synapse-sdk`](https://github.com/FilOzone/synapse-sdk) under the Permissive License Stack (Apache-2.0 OR MIT). Prova elects the MIT side.

## Upstream

- **Repository:** https://github.com/FilOzone/synapse-sdk
- **Forked from:** `master` branch as of 2026-04-24
- **Upstream version:** `@filoz/synapse-core@0.4.1`, `@filoz/synapse-sdk@0.40.4`
- **License:** Apache-2.0 OR MIT (Permissive License Stack)
- **Upstream author:** Rod Vagg (`rod@vagg.org`) + FilOzone contributors

## Prova Packages

| Prova Package | Upstream Package | Status |
|---------------|------------------|--------|
| `@prova-network/core` | `@filoz/synapse-core` | Forked, renamed, repointed to Prova contracts (WIP) |
| `@prova-network/sdk` | `@filoz/synapse-sdk` | Forked, renamed, repointed to Prova contracts (WIP) |

## Structural changes

### Kept as-is (Filecoin-agnostic primitives)
- `core/src/abis/` — ABI definitions (will update to Prova ABIs)
- `core/src/erc20/` — ERC-20 helpers (chain-agnostic)
- `core/src/piece/` — CommP piece calculations (Filecoin-inherited concept, kept for Prova)
- `core/src/pdp-verifier/` — PDP verifier bindings (points at our forked contract)
- `core/src/session-key/` — session key management (generic)
- `core/src/typed-data/` — EIP-712 typed data helpers (generic)
- `core/src/utils/` — miscellaneous utilities

### Renamed
- `@filoz/synapse-core` → `@prova-network/core`
- `@filoz/synapse-sdk` → `@prova-network/sdk`
- `Synapse` class → `Prova` class
- `synapse.ts` → `prova.ts`
- `warm-storage` conceptual model → simply `storage` (warm-storage was Filecoin-specific naming)

### Dropped for v1 (may revisit)
- `filbeam` — Filecoin's CDN retrieval service; Prova doesn't use it
- `usdfc` — Filecoin's USDC-equivalent stablecoin; Base has canonical USDC
- `endorsements` — Filecoin-specific SP endorsement system; may revisit if useful

### Rewritten (pointing at Prova contracts instead of Filecoin contracts)
- `warm-storage/service.ts` (was FilOzone WarmStorageService) → `storage/service.ts` pointing at `StorageMarketplace.sol`
- `sp-registry/service.ts` (was FilOzone SPRegistry) → points at our `ProverRegistry.sol`
- `payments/service.ts` (was FilOzone Payments) → simpler, points at our deal escrow in `StorageMarketplace.sol`
- `chains.ts` — Base mainnet + Base Sepolia (was Filecoin chains)

## Attribution in source files

Each derived file carries a SPDX header and upstream pointer. Upstream
copyright is preserved. New Prova-specific work is additively attributed.

```typescript
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright (c) 2024-2026 Protocol Labs, FilOzone contributors (upstream: synapse-sdk).
// Copyright (c) 2026 Prova Network contributors.
//
// This file is adapted from FilOzone/synapse-sdk
// (https://github.com/FilOzone/synapse-sdk). Originally under the
// Permissive License Stack (Apache-2.0 OR MIT).
```

## SPDX policy for this package

Upstream `synapse-sdk` ships its license declaration at the repository
root (`LICENSE.md`) rather than per-file. Prova follows the same
convention for the TypeScript SDK:

- The root `/LICENSE` declares the dual Apache-2.0 OR MIT license for
  derived content and MIT for original Prova work.
- Per-package `package.json` includes `"license": "Apache-2.0 OR MIT"`
  so consumers tooling (npm, Sonatype, SPDX scanners) sees it.
- Entry points that we substantively edited (e.g. `sdk/src/prova.ts`)
  carry explicit SPDX headers + upstream attribution.
- Deeper internal modules inherit the package-level license declaration
  per customary JS/TS convention. Any module materially rewritten for
  Prova should get its own SPDX header.

This matches upstream's licensing practice and keeps attribution
discoverable without imposing 174 boilerplate headers.

## Open items before publish

Before publishing `@prova-network/*` packages to npm:

- [ ] Complete rename pass (imports, types, docs)
- [ ] Replace WarmStorage references with Prova StorageMarketplace
- [ ] Update ABIs to point at deployed Prova contracts
- [ ] Update chain config to Base mainnet + Sepolia
- [ ] Full test pass against deployed contracts on Base Sepolia
- [ ] README rewrite for Prova positioning
- [ ] Publish under scoped org `@prova-network/*` on npm

---

*Last updated: 2026-04-24.*
