# Prova

[![License](https://img.shields.io/badge/license-MIT-blue)](./LICENSE)
[![Ethereum](https://img.shields.io/badge/settles%20on-Ethereum-5F72E6)](https://ethereum.org)
[![Status](https://img.shields.io/badge/status-pivot%20in%20progress-orange)](./PIVOT.md)
[![Chain](https://img.shields.io/badge/chain-Base%20L2-0052FF)](https://base.org)

**Verifiable storage for Ethereum.**
Store websites, AI data, and digital archives with cryptographic proofs of retention. Pay in ETH. Verify on Base. No new chain.

*Prova* — Latin: "to prove."

🌐 [Website](https://prova-network.pages.dev) · 📄 [Whitepaper (v2 draft)](./whitepaper-v2-source.md) · 🔀 [Pivot Plan](./PIVOT.md) · 🎨 [Brand](./brand)

---

## What Prova Is

Prova is an Ethereum-native network for verifiable, retrievable, provable storage. It answers three questions that Ethereum alone cannot:

- **Is my data actually stored?** — Periodic Provable Data Possession (PDP) proofs, verified on-chain.
- **Can I get it back?** — Retrievability challenges, with slashing for provers who disappear.
- **Can I build on this?** — Ethereum-native APIs, ENS integration, standard ERC-20 token, no new runtime to learn.

## Why Now

Filecoin's storage proof technology (PDP, CommP, Fulcrum of Consistency) is battle-tested but lives on a separate chain with separate economics, separate tooling, separate attention. In 2026, Ethereum has the payment rails, the identity (ENS), the composability, and the development velocity that makes a dedicated storage chain unnecessary.

Prova ports the best of that storage technology (under MIT) onto Ethereum, without dragging a new chain along with it.

## Architecture (High-Level)

```
┌────────────────────────────────────────────────────────────┐
│                       ETHEREUM L1                           │
│                                                             │
│   ProvaToken   ProverRegistry   StorageMarketplace          │
│   ProofVerifier   ProverStaking   DisputeManager            │
│                                                             │
│   Identity via address / ENS  ·  Payment in ETH / USDC      │
└─────────────────────┬──────────────────────────────────────┘
                      │
                      ▼
┌────────────────────────────────────────────────────────────┐
│                PROVA NETWORK (off-chain)                    │
│                                                             │
│   Prover Node              Aggregator Node                  │
│   ├─ PDP engine             ├─ Batches proofs               │
│   ├─ Blob storage           ├─ Anchors to L1                │
│   ├─ HTTPS serving          └─ Indexes events               │
│   └─ Ethereum wallet                                        │
└────────────────────────────────────────────────────────────┘
```

See [`PROVA-V2-ARCHITECTURE.md`](./PROVA-V2-ARCHITECTURE.md) for details.

## Core Primitives

### Content Commitments (CommP)

Every stored object has a **CommP**, a binary-Merkle-tree commitment over 32-byte leaves using SHA-256 (Fr-safe). CommPs are Filecoin-compatible and can be computed client-side or by the prover. Objects are addressed by CommP; CommPs can optionally bind to ENS names.

### Provable Data Possession (PDP)

Instead of sealed data (expensive, hours to onboard), Prova uses lightweight PDP over raw unsealed data. Random challenges hit random leaves. Provers respond with Merkle inclusion proofs verified on-chain in O(log N) gas. Onboarding is minutes, not hours.

### Interactive Fraud Proofs (Disputes)

Optimistic proofs for the happy path, interactive bisection for the dispute path. Cheap when everyone's honest, expensive only for the cheater. ZK aggregation reserved for v2 once volume demands it.

### Ethereum-Native Economics

Staking, payment, governance, slashing — all live on Ethereum. PROVA is a standard ERC-20 from day one, no bridge, no native gas, no custom chain runtime.

## Use Cases

### 1. Ethereum-Backed Permanent Websites
Upload a site, bind to `.eth`, always retrievable, cryptographically proven.

### 2. AI Dataset Provenance & Retention
Register training sets, evaluation corpora, model artifacts. Prove they existed and stayed retrievable over time.

### 3. Compliance / Evidence / Audit Archives
Machine-verifiable retention attestations for regulated industries. Pay in USDC.

### 4. Developer Storage Primitive
S3-with-proofs for any application that needs verifiable storage.

## Project Status

**Pivot in progress.** See [`PIVOT.md`](./PIVOT.md).

- v0.2 codebase (Layer 1 chain implementation): archived at `v0.2-l1-snapshot` tag and `archive/` directory
- v2 architecture: designed, see `PROVA-V2-ARCHITECTURE.md`
- v2 implementation: in progress, see `PROVA-V2-PIVOT-PLAN.md`
- v2 contracts: PDP verifier forked from `FilOzone/pdp`, FVM coupling swapped for Base, compiles cleanly
- Public testnet: targeting mid-2026 on Base Sepolia
- Token launch: deferred until usage and revenue justify it (points-to-token conversion)

### What's done
- [x] Phase 0: archive branch, tag, pivot branch
- [x] Phase 1: planning artifacts (audit, architecture, plan, source map)
- [x] Phase 2: spec cleanup (6 obsolete specs archived)
- [x] Phase 3 partial: Rust chain code archived (~68K lines)
- [x] Phase 6 partial: core PDP contracts forked, Base-compatible, builds green

### Next up
- [ ] Write Prova-specific contracts: `ProverRegistry`, `ProverStaking`, `StorageMarketplace`, `ContentRegistry`
- [ ] Fork `FilOzone/synapse-sdk` as `sdk/typescript/`
- [ ] Set up `prover/` as a Go module, port from `filecoin-project/curio`
- [ ] Deploy contracts to Base Sepolia
- [ ] First end-to-end: upload file, see proof land on-chain

## Repository Layout (v2, in progress)

```
prover/       Rust prover node (PDP engine, blob storage, HTTPS serving)
aggregator/   Optional aggregator node (proof batching, L1 anchoring)
sdk/
  rust/       Rust SDK for prover operators and power users
  typescript/ Primary TypeScript SDK for applications
contracts/    Solidity contracts for Ethereum L1
spec/         Protocol specifications
docs/         User and developer documentation
brand/        Brand assets
website/      Public website source
archive/      v1 Layer 1 chain code, kept for reference
```

## Contributing

Currently closed to external contributors during the v2 pivot. Will open after the first public testnet (target: mid-2026).

## License

MIT. Much of the storage-proof primitive technology is derived from MIT-licensed Filecoin ecosystem code (Filecoin Project, FoC, PDP research). Proper attribution lives in each source file.

## Credits

Prova stands on the shoulders of the Filecoin community. PDP, CommP, FoC, storage-proof research — all originated there. We port those primitives to Ethereum as a coexistent network, not a competitor.

---

*Last updated: 2026-04-24*
