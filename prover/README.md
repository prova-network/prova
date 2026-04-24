# Prova Prover (Go)

Reference implementation of a Prova Network prover node.

## Status

See [`ROADMAP.md`](./ROADMAP.md) for the phase-by-phase status. At a glance:

- Phases A through G complete: full prover binary running locally,
  contract stack, deal lifecycle, HTTP retrieval, Prometheus metrics,
  systemd unit, Docker image, transplant audit done.
- Gate for Phase H (Base Sepolia deploy) is the author's exit from
  Curio engagements; target mid-2026.

Portions of the PDP engine are transplanted from
[`filecoin-project/curio`](https://github.com/filecoin-project/curio)
and [`filecoin-project/lotus`](https://github.com/filecoin-project/lotus)
under the Permissive License Stack (Apache-2.0 OR MIT).
See [`ATTRIBUTION.md`](./ATTRIBUTION.md) for the per-file source map.

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

Or with version info baked in:

```sh
go build -ldflags "-X main.version=$(git describe --always) -X main.commit=$(git rev-parse --short HEAD)" ./cmd/provad
```

The Docker recipe (`./Dockerfile`) produces a distroless nonroot image.

## Transplanted from upstream

See [`ATTRIBUTION.md`](./ATTRIBUTION.md) for the authoritative per-file
source map and adaptation notes. In summary:

- `pkg/piece/cid.go` — `curio/pdp/piece_cid.go` (imported ~unchanged).
- `pkg/pdptree/fr32.go` — `lotus/storage/sealer/fr32/fr32.go`
  (bit-for-bit port of `pad()`).
- `pkg/pdptree/memtree.go` — three files from
  `curio/lib/proof/` merged into one self-contained SHA-254 memtree
  implementation.

All other `prover/pkg/*` packages are Prova-original code.

## License

Apache-2.0 OR MIT (dual, following upstream).
