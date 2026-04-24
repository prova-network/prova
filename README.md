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

## Install

Once Prova reaches public testnet (see status below), the prover
installs with:

```bash
curl -fsSL https://prova.network/install.sh | bash
```

This fetches the right platform binary, verifies its SHA-256 checksum,
installs to `/usr/local/bin/provad`, drops an example config at
`/etc/prova/prover.toml.example`, and (on Linux) installs a hardened
systemd unit.

Override behavior with env vars:

```bash
PROVA_VERSION=v0.1.0    curl -fsSL https://prova.network/install.sh | bash
PROVA_PREFIX=$HOME/.local  curl -fsSL https://prova.network/install.sh | bash
PROVA_NO_SYSTEMD=1         curl -fsSL https://prova.network/install.sh | bash
```

See [`install.sh`](./install.sh) for the full source; it's intentionally
short and auditable. Uninstall with
`curl -fsSL https://prova.network/uninstall.sh | bash`.

## Project Status

**Pivot in progress.** See [`PIVOT.md`](./PIVOT.md).

- v0.2 codebase (Layer 1 chain implementation): archived at `v0.2-l1-snapshot` tag and `archive/` directory
- v2 architecture: designed, see `PROVA-V2-ARCHITECTURE.md`
- v2 implementation: in progress, see `PROVA-V2-PIVOT-PLAN.md`
- v2 contracts: PDP verifier forked from `FilOzone/pdp`, FVM coupling swapped for Base, compiles cleanly
- Public testnet: targeting mid-2026 on Base Sepolia
- Token launch: deferred until usage and revenue justify it (points-to-token conversion)

### Built so far

- [x] Planning: audit, architecture, migration plan, source map, tokenomics v2
- [x] Solidity contracts: 6 contracts (ProvaToken, ProofVerifier, ProverRegistry,
      ProverStaking, ContentRegistry, StorageMarketplace) + MockProofVerifier
      for local integration. All tests pass.
- [x] TypeScript SDK: forked from `FilOzone/synapse-sdk`, renamed, repointed.
      Awaits ABI regeneration against deployed contracts.
- [x] Go prover daemon (`prover/provad`): full lifecycle works end-to-end
      locally.
      See [`prover/ROADMAP.md`](./prover/ROADMAP.md) for phase-by-phase status
      (A–G complete, H blocked on Base Sepolia deploy).
- [x] Website rewritten in quiet-research mode (see `website/`).
- [x] Attribution audit complete (see per-directory `ATTRIBUTION.md`).

### Next up

- [ ] Deploy contracts to Base Sepolia (gated on author's exit from
      existing engagements; target mid-2026)
- [ ] Regenerate TypeScript SDK ABIs against live deployment
- [ ] Public testnet launch
- [ ] Points program goes live

## Repository Layout

```
contracts/            Solidity contracts (Base-compatible)
  src/                ProofVerifier, registry, staking, marketplace, ...
  script/Deploy.s.sol Foundry deploy script
  test/               Foundry tests

prover/               Go prover daemon
  cmd/provad/         daemon + CLI binary
  pkg/                deal lifecycle, challenges, http, metrics, pdptree, ...
  internal/soaktest/  end-to-end integration scenario (anvil)

sdk/typescript/       TypeScript client SDK
  core/               @prova-network/core
  sdk/                @prova-network/sdk

spec/                 Protocol specifications
website/              Public website source
docs/                 Project documentation index
brand/                Brand assets

LICENSE               MIT for original work; per-directory attribution for derived code
archive/              Pre-pivot artifacts kept for reference
```

## Contributing

Closed to external contributors while the project is in private development.
After the first public testnet, contribution guidelines will live in
`CONTRIBUTING.md`.

## License

MIT for original Prova work; portions derived from upstream projects
([FilOzone/pdp](https://github.com/FilOzone/pdp),
[FilOzone/synapse-sdk](https://github.com/FilOzone/synapse-sdk),
[filecoin-project/curio](https://github.com/filecoin-project/curio),
[filecoin-project/lotus](https://github.com/filecoin-project/lotus))
under Apache-2.0 OR MIT, with attribution preserved per file and in
[`LICENSE`](./LICENSE) + per-directory `ATTRIBUTION.md`.

## Credits

Prova stands on years of storage-proof research by the Filecoin community.
PDP, CommP, FR32 padding, SHA-254 merkle trees, piece-commitment CIDs —
all originated there. Prova ports those primitives onto Base as a
coexistent network.

---

*Last updated: 2026-04-24.*
