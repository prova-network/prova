# Attribution — Prover Go Package Sources

Portions of the Prova prover are derived from upstream Filecoin-ecosystem projects under the Permissive License Stack (Apache-2.0 OR MIT). Prova elects the MIT side. Original Prova-authored code is MIT.

## Upstream projects

| Upstream | Repository | License |
|----------|------------|---------|
| `curio` | https://github.com/filecoin-project/curio | Apache-2.0 OR MIT (dual: `LICENSE-MIT`, `LICENSE-APACHE`) |
| `lotus` | https://github.com/filecoin-project/lotus | Apache-2.0 OR MIT (dual) |

Both forked at `main` branches as of 2026-04-24.

## Files derived (transplanted)

| Prova file | Upstream file | Adaptation |
|------------|---------------|------------|
| `pkg/piece/cid.go` | `curio/pdp/piece_cid.go` | Imported ~unchanged; package name `pdp` → `piece`. Pure CommP logic, no Curio-internal deps. |
| `pkg/piece/cid_test.go` | `curio/pdp/piece_cid_test.go` | Package rename only. |
| `pkg/pdptree/fr32.go` | `lotus/storage/sealer/fr32/fr32.go` | Bit-for-bit port of `pad()`; stripped multi-threading, `abi.*PieceSize` types, and `Unpad` helpers. |
| `pkg/pdptree/memtree.go` | `curio/lib/proof/merkle_sha254_memtree.go` + `merkle_proof_memtree.go` + `tree_size.go` | Three files merged; `minio/sha256-simd` → stdlib `crypto/sha256`; `libp2p-buffer-pool` → plain slices; logging stripped; `abi.PieceSize` types removed. |

## Files planned (not yet transplanted)

| Prova file | Upstream file | Notes |
|------------|---------------|-------|
| _future_ | `curio/pdp/handlers.go` | Upload / piece-post handlers; would replace DB interactions with local storage. |
| _future_ | `curio/pdp/handlers_pull.go` | Pull-from-peer flow; needs harmonydb stripped. |
| _future_ | `curio/pdp/auth.go` | JWT auth; would swap harmonydb for local key store. |
| _future_ | `curio/pdp/indexing.go` | Piece-CID → on-disk-path index. |

## Attribution in source files

Each derived file carries a SPDX header + upstream pointer in the form:

```go
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright (c) 2024-2026 Filecoin Project contributors (upstream: filecoin-project/curio).
// Copyright (c) 2026 Prova Network contributors.
//
// This file is adapted from filecoin-project/curio <path>.go
// (https://github.com/filecoin-project/curio). Originally under the
// Permissive License Stack (Apache-2.0 OR MIT). Attribution preserved
// per license.
//
// Adaptations for Prova: <concrete list>
```

Strip list typical for Curio transplants:

- **`harmonydb` usage** → replaced with plain SQLite or in-memory maps
- **Lotus RPC** → replaced with `go-ethereum` Base RPC client
- **FVM contracts** → replaced with Prova contracts (see `contracts/`)
- **`harmonytask` scheduler** → replaced with a simple tick loop

## Original Prova code (MIT only)

Packages authored from scratch for Prova (no upstream derivation):

- `cmd/provad/` — daemon entrypoint
- `pkg/config/` — TOML config loader
- `pkg/wallet/` — key loading (keystore / hex / env)
- `pkg/ethclient/` — go-ethereum wrapper with chain-ID verification
- `pkg/contracts/` — abigen output against Prova Solidity contracts
- `pkg/deal/` — deal lifecycle engine, store, fetcher, source-URL resolver, event poller
- `pkg/challenges/` — challenge index generation + submit path
- `pkg/pdptree/builder.go` — adapter connecting our piece store to the transplanted tree/fr32 code
- `pkg/httpserver/` — HTTP retrieval endpoint
- `pkg/daemon/` — goroutine supervisor
- `pkg/metrics/` — Prometheus metrics
- `pkg/store/` — disk-backed blob store (content-addressed)

## License

Dual-licensed Apache-2.0 OR MIT to match upstream. The root `/LICENSE`
declares the project-wide terms. See individual source file SPDX headers
for per-file classification.

---

*Last updated: 2026-04-24.*
