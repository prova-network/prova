# Attribution — Third-Party Source

The Prova contracts in `contracts/src/` include code derived from the following upstream projects. All are dual-licensed Apache-2.0 OR MIT under Protocol Labs' "Permissive License Stack." Prova elects the MIT side.

## FilOzone/pdp

**Source:** https://github.com/FilOzone/pdp
**License:** Apache-2.0 OR MIT (SPDX: `Apache-2.0 OR MIT`)
**Forked at:** commit on `main` as of 2026-04-24

### Files derived

| Prova file | Upstream file | Adaptation |
|------------|---------------|------------|
| `src/ProofVerifier.sol` | `src/PDPVerifier.sol` | Renamed contract `PDPVerifier` → `ProofVerifier`. Replaced `FVMPay.burn` with native ETH send to `0x...dEaD`. Replaced `FVMPay.pay` with low-level call refund. Replaced `FVMRandom.getBeaconRandomness` with `block.prevrandao` (EIP-4399). All four logical changes total. |
| `src/Cids.sol` | `src/Cids.sol` | Imported unchanged. |
| `src/Proofs.sol` | `src/Proofs.sol` | Imported unchanged (itself MIT-licensed; upstream adapted from OpenZeppelin Contracts `utils/cryptography/MerkleProof.sol` v5.0.0). |
| `src/BitOps.sol` | `src/BitOps.sol` | Imported unchanged. |
| `src/Fees.sol` | `src/Fees.sol` | Imported unchanged (may be adjusted for Base gas dynamics in a future revision). |
| `src/interfaces/IPDPEvents.sol` | Same path | Imported unchanged. |
| `src/interfaces/IPDPTypes.sol` | Same path | Imported unchanged. |
| `src/interfaces/IPDPVerifier.sol` | Same path | Imported unchanged. |

## OpenZeppelin Contracts

**Source:** https://github.com/OpenZeppelin/openzeppelin-contracts (v5.0.0)
**License:** MIT

Included indirectly via `src/Proofs.sol` (FilOzone adapted `MerkleProof.sol`).
Attribution preserved inline in that file.

## Attribution in source files

Each derived Solidity file carries an SPDX header + upstream attribution:

```solidity
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright (c) 2024-2026 Protocol Labs and contributors (upstream: FilOzone/pdp).
// Copyright (c) 2026 Prova Network contributors.
//
// This file is adapted from FilOzone/pdp <path>.sol
// (https://github.com/FilOzone/pdp). Originally under Permissive License Stack
// (Apache-2.0 OR MIT). Attribution preserved per license.
```

## Original Prova work (MIT only)

The following contracts are original Prova Network code under MIT:

- `src/ProvaToken.sol` — ERC-20 token (Prova-authored, uses OpenZeppelin ERC-20 base)
- `src/ProverRegistry.sol` — prover directory + metadata
- `src/ProverStaking.sol` — slashable stake with unbonding period
- `src/ContentRegistry.sol` — CommP → active deal mapping + ENS binding
- `src/StorageMarketplace.sol` — deal lifecycle + PDPListener implementation
- `src/legacy/` — pre-pivot ICO-era contracts, archived

## License

Dual-licensed Apache-2.0 OR MIT to match upstream. See the root `/LICENSE`
for the project-wide license text.

---

*Last updated: 2026-04-24.*
