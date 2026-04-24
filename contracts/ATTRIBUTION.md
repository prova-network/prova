# Attribution — Third-Party Source

The Prova contracts in `contracts/src/` include code derived from the following upstream projects. All are dual-licensed Apache-2.0 OR MIT under Protocol Labs' "Permissive License Stack." Prova uses the MIT side.

## FilOzone/pdp

**Source:** https://github.com/FilOzone/pdp
**License:** Apache-2.0 OR MIT (SPDX: `Apache-2.0 OR MIT`)
**Adapted at:** commit `main` as of 2026-04-24

### Files derived

| Prova file | Upstream file | Adaptation |
|------------|---------------|------------|
| `src/ProofVerifier.sol` | `src/PDPVerifier.sol` | Renamed contract, replaced `FVMPay` burn/pay with ERC-20 logic, replaced `FVMRandom.getBeaconRandomness` with on-chain randomness (prevrandao / commit-reveal). |
| `src/Cids.sol` | `src/Cids.sol` | Imported unchanged. |
| `src/Proofs.sol` | `src/Proofs.sol` | Imported unchanged. |
| `src/BitOps.sol` | `src/BitOps.sol` | Imported unchanged. |
| `src/Fees.sol` | `src/Fees.sol` | Imported unchanged (may be adjusted for Base gas dynamics). |
| `src/interfaces/IPDPEvents.sol` | Same path | Imported unchanged. |
| `src/interfaces/IPDPTypes.sol` | Same path | Imported unchanged. |
| `src/interfaces/IPDPVerifier.sol` | Same path | Imported unchanged. |

## Upstream notice template

Each derived file carries an SPDX header and a comment pointer to the upstream origin:

```solidity
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright (c) 2024-2026 Protocol Labs and contributors.
// Copyright (c) 2026 Prova Network contributors.
//
// This file is adapted from FilOzone/pdp (github.com/FilOzone/pdp).
// Originally under the Permissive License Stack (Apache-2.0 OR MIT).
// Attribution preserved per license.
```

## New Prova work

The following contracts are original Prova Network code under MIT:

- `src/ProvaToken.sol` (ERC-20 token, written before pivot)
- `src/ProverRegistry.sol` (TBD)
- `src/ProverStaking.sol` (TBD)
- `src/StorageMarketplace.sol` (TBD)
- `src/ContentRegistry.sol` (TBD)
- `src/DisputeManager.sol` (TBD, v2)
- `src/SettlementVault.sol` (TBD, v2)
- `src/legacy/` (ICO-era contracts, deprecated)

## Also planned for fork

- `FilOzone/synapse-sdk` — TypeScript client SDK (Permissive Stack, MIT side)
- `FilOzone/filecoin-cloud` — Next.js web UI (MIT)
- `FilOzone/filecoin-pay-explorer` — TheGraph subgraph (Permissive Stack, MIT side)
- `FilOzone/dealbot` — testing / load tool (Permissive Stack, MIT side)
- `FilOzone/pdp-explorer` — alternative indexer pattern (MIT)
- `filecoin-project/curio` — Go prover code, PDP/market modules (Permissive Stack, MIT side)

Each incorporation will be documented in this file.

---

*Last updated: 2026-04-24.*
