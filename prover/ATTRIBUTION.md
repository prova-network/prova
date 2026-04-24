# Attribution — Prover Go Package Sources

Prova's Go prover is primarily derived from [`filecoin-project/curio`](https://github.com/filecoin-project/curio) under the Permissive License Stack (Apache-2.0 OR MIT). Prova elects the MIT side.

## Upstream

- **Repository:** https://github.com/filecoin-project/curio
- **Forked from:** `main` branch as of 2026-04-24
- **License:** Apache-2.0 OR MIT (`LICENSE-MIT`, `LICENSE-APACHE` at repo root)
- **Upstream maintainers:** Filecoin Project contributors (notably LexLuthr, snadrus, rvagg, and others)

## Files derived / planned

| Prova package | Upstream file | Adaptation |
|---------------|---------------|------------|
| `pkg/piece/cid.go` | `curio/pdp/piece_cid.go` | Imported ~unchanged; pure CommP logic, no Curio-internal deps |
| TBD | `curio/pdp/handlers.go` | Pending. Will split: upload handlers stay in HTTP server, auth becomes its own package, DB interactions replaced with local-only storage |
| TBD | `curio/pdp/handlers_pull.go` | Pending. Pull-from-peer logic; will be adapted to work off HTTPS without harmonydb |
| TBD | `curio/pdp/auth.go` | Pending. JWT-based auth, swap harmonydb dependency for local key store |
| TBD | `curio/pdp/indexing.go` | Pending. Piece-CID → storage-path indexing |

## Attribution in source files

Each derived file carries an SPDX header and upstream pointer:

```go
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright (c) 2024-2026 Filecoin Project contributors (upstream: filecoin-project/curio).
// Copyright (c) 2026 Prova Network contributors.
//
// This file is adapted from filecoin-project/curio pdp/<filename>.go
// (https://github.com/filecoin-project/curio). Originally under the
// Permissive License Stack (Apache-2.0 OR MIT).
```

## Strip list

Most of Curio's PDP code assumes:

- PostgreSQL via `harmonydb` (a Curio-specific task framework)
- Lotus Filecoin RPC for chain state
- FVM-specific contract semantics
- Curio's task scheduler (`harmonytask`)

These are stripped:
- **`harmonydb` usage** → replaced with plain SQLite or local filesystem indices, depending on the data
- **Lotus RPC** → replaced with `go-ethereum` Base RPC client
- **FVM contracts** → replaced with our forked Base-compatible contracts (see `contracts/`)
- **`harmonytask` scheduler** → replaced with a simple Go worker pool; prover has far fewer task types than a Curio Storage Provider

## New code

Packages written from scratch for Prova (no Curio derivation):

- `cmd/provad/` — entrypoint
- `pkg/config/` — config loading for our specific node shape
- `pkg/ethclient/` — go-ethereum wrapper tailored to our contract interfaces
- `pkg/contracts/` — abigen output against Prova's Solidity contracts
- `pkg/deal/` — deal lifecycle state machine (Prova-specific)
- `pkg/challenges/` — reimplemented against our `ProofVerifier` contract

## License

Apache-2.0 OR MIT dual license, matching upstream.

---

*Last updated: 2026-04-24.*
