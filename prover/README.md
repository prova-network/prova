# Prova Prover (Go)

Reference implementation of a Prova Network prover node.

## Status

**Scaffolding in progress.** Transplanted in phases from upstream:
- [`filecoin-project/curio`](https://github.com/filecoin-project/curio) — PDP engine, piece CID machinery, blob storage handlers (Apache-2.0 OR MIT)

See [`ATTRIBUTION.md`](./ATTRIBUTION.md).

## Responsibilities

A prover node:

1. Watches the Prova contracts on Base for new deals targeting its address
2. Downloads pieces from clients or other provers
3. Computes and verifies CommP on received pieces
4. Stores pieces locally, indexed by CommP
5. Responds to on-chain challenges with Merkle inclusion proofs
6. Serves content over HTTPS to clients (optional role)
7. Maintains an Ethereum wallet for settlement

## Design principles

- **Stateless where possible.** State lives on-chain or in local blob storage, not in long-lived memory.
- **Graceful failure.** Missing one proof is not the end of the world. Missing ten in a row is.
- **Observable.** Every decision is logged. Every metric is exported.
- **Composable.** Storage backend, payment client, and proof submitter are separate packages.
- **No Filecoin L1 dependencies.** Everything settles on Base; no Lotus RPC, no FVM.

## Package layout (planned)

```
prover/
├── cmd/
│   └── provad/              — prover daemon entrypoint
├── pkg/
│   ├── piece/               — CommP computation + piece CID parsing (transplanted from curio/pdp)
│   ├── store/               — blob storage backend (local disk, S3 future)
│   ├── ethclient/           — Base/Ethereum JSON-RPC wrapper
│   ├── contracts/           — Go bindings for Prova Solidity contracts (abigen)
│   ├── deal/                — deal lifecycle: accept, download, verify, index
│   ├── challenges/          — challenge listener + PDP proof generator
│   ├── http/                — HTTPS serving endpoint (optional retrieval role)
│   ├── wallet/              — key management, tx signing
│   └── config/              — config loading
├── internal/
│   └── testutil/            — test fixtures, anvil helpers
├── go.mod
├── go.sum
└── ATTRIBUTION.md
```

## Building

```sh
go build ./cmd/provad
```

Once scaffolded (not yet). Current state: no runnable binary.

## Transplanted from Curio

These packages have been adapted from `filecoin-project/curio` under the
Apache-2.0 OR MIT dual license. Full upstream attribution preserved at file
level and in `ATTRIBUTION.md`.

| Prover package | Upstream path | Status |
|----------------|---------------|--------|
| `pkg/piece` | `curio/pdp/piece_cid.go` | Pending |
| — | `curio/pdp/handlers.go` (upload + pull) | Pending |
| — | `curio/pdp/auth.go` | Pending |
| — | `curio/pdp/indexing.go` | Pending |

## License

Apache-2.0 OR MIT (dual, following upstream).
