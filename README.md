<div align="center">
  <picture>
    <source media="(prefers-color-scheme: dark)" srcset="https://prova.network/brand/prova-mark-dark.svg">
    <img src="https://prova.network/brand/prova-mark-light.svg" alt="Prova" width="96" height="96">
  </picture>

  <h1>Prova</h1>

  <p><strong>Verifiable storage anchored to Ethereum.</strong> Upload a file, pay in USDC on Base, get continuous on-chain proofs that your data is actually stored.</p>

  <p>
    <a href="https://prova.network"><img src="https://img.shields.io/badge/site-prova.network-2EC4B6?style=flat-square" alt="site" /></a>
    <a href="https://docs.prova.network"><img src="https://img.shields.io/badge/docs-docs.prova.network-2EC4B6?style=flat-square" alt="docs" /></a>
    <a href="https://spec.prova.network"><img src="https://img.shields.io/badge/spec-spec.prova.network-2EC4B6?style=flat-square" alt="spec" /></a>
    <a href="https://base.org"><img src="https://img.shields.io/badge/chain-Base%20L2-0052FF?style=flat-square" alt="Base L2" /></a>
    <a href="./LICENSE"><img src="https://img.shields.io/badge/license-Apache--2.0%20OR%20MIT-0F4C5C?style=flat-square" alt="license" /></a>
    <a href="./STATUS.md"><img src="https://img.shields.io/badge/status-pre--testnet%20%E2%80%A2%20iterating-5DC3E5?style=flat-square" alt="status" /></a>
  </p>
</div>

---

Prova is a thin storage primitive that puts the smallest useful unit of *"I have your bytes, I can prove it"* on Base. Deals are USDC-denominated. Provers stake PROVA. Slashing burns the stake. The 1% protocol fee on USDC streams auto-burns PROVA from market revenue. Boring economics, sharp guarantees.

> **Upload a file.** A prover picks it up, recomputes the piece-CID, and stores the bytes.<br>
> **Wait 30 seconds.** The deal lands on Base and the prover starts getting challenged on-chain.<br>
> **Retrieve any time.** Pull the bytes back over HTTPS, verified against the piece-CID you computed yourself.

---

## Why Prova

- 📦 **One storage primitive, done well.** Provable Data Possession (PDP), nothing else. No sealing, no PoRep, no SNARKs, no AI, no new chain.
- ⚡ **Cheap on-chain cost.** Merkle inclusion proofs verify in O(log N) gas on Base. About $0.001 per proof.
- 💰 **Stablecoin payments, token-aligned provers.** Clients pay USDC. Provers earn USDC + PROVA emission. Provers stake slashable PROVA.
- 🔥 **Deflationary by design.** Every USDC fee swaps to PROVA on Uniswap V3 and burns. Network revenue → permanent supply reduction.
- 🌐 **Base-native.** USDC is canonical. ENS works natively. No bridge, no custody, no new chain to learn.
- 🧰 **Single Go binary for provers.** `provad` is the whole prover. Disk-bound, not GPU-bound. Boring on purpose.
- 🤝 **Standing on Filecoin's shoulders.** PDP, CommP, Fr32: MIT-licensed, battle-tested. We credit and reuse.

---

## The org

This is the umbrella repo. The implementation is split across the [prova-network](https://github.com/prova-network) org:

| Repo | Purpose |
| --- | --- |
| [`prova-network/contracts`](https://github.com/prova-network/contracts) | Solidity contracts: `ProvaToken`, `ProvaVesting`, `ProverRewards`, `FeeRouter`, `StorageMarketplace`, `ProverStaking`, `ProverRegistry`, `ContentRegistry`, `ProofVerifier` |
| [`prova-network/prover`](https://github.com/prova-network/prover) | Go `provad` daemon — the prover-side software |
| [`prova-network/cli`](https://github.com/prova-network/cli) | `@prova-network/cli` — the `prova` command-line tool |
| [`prova-network/sdk`](https://github.com/prova-network/sdk) | `@prova-network/sdk` + `@prova-network/core` — TypeScript SDK |
| [`prova-network/website`](https://github.com/prova-network/website) | The [prova.network](https://prova.network) site (Pages + Functions) |
| [`prova-network/docs`](https://github.com/prova-network/docs) | Public documentation rendered at [docs.prova.network](https://docs.prova.network) |
| [`prova-network/desktop`](https://github.com/prova-network/desktop) | Electron desktop wrapper for `provad` |
| [`prova-network/brand`](https://github.com/prova-network/brand) | Logos, diagrams, brand assets |

This umbrella repo holds the cross-cutting docs:

- [`STATUS.md`](./STATUS.md) — current state + scope-of-work for collaborators
- [`TOKENOMICS-2026.md`](./TOKENOMICS-2026.md) — full token economics reference
- [`spec/`](./spec/) — protocol specifications (rendered at [spec.prova.network](https://spec.prova.network))
- [`SECURITY-AUDIT-2026-04-25.md`](./SECURITY-AUDIT-2026-04-25.md) — internal pre-deployment audit log
- [`FILECOIN-SOURCE-MAP.md`](./FILECOIN-SOURCE-MAP.md) — upstream attribution
- [`graphify-out/`](./graphify-out/) — knowledge-graph snapshot of the codebase
- [`scripts/`](./scripts/) — operational scripts (deploy helpers, sync-splits, graph)

---

## How the token works (one paragraph)

There is **one token, PROVA**, an ERC-20 on Base with a fixed total supply of **100,000,000**. PROVA plays three roles inside the protocol: provers post slashable PROVA stake to gate their committed-byte capacity (`minStakePerGiB × committedGiB`); the marketplace's 1% USDC fee routes to a permissionless `FeeRouter` that swaps USDC → PROVA on Uniswap V3 and burns the PROVA; and PROVA-weighted governance votes set bounded parameters (fee tier, slash fraction, redundancy cap) with a 2-day timelock. **50%** of total supply (50M) is reserved for **prover emission** over 8 years on a declining curve, paid out weekly per byte proven, gated by anti-gaming protections (no self-dealing, per-piece redundancy cap, 30-day vesting buffer, quality multiplier). Clients never need to hold PROVA. Storage payments are entirely in USDC.

Full economic specification: [`TOKENOMICS-2026.md`](./TOKENOMICS-2026.md) and [spec.prova.network/token-economics](https://spec.prova.network/token-economics).

---

## Three ways to use Prova

### 1. Browser, no install

[**prova.network/upload**](https://prova.network/upload/) — drag a file in, get a piece-cid. First 100 MB free, sponsored.

### 2. CLI

```bash
curl -fsSL https://get.prova.network | sh
prova auth
prova put ./dist.tar.gz
```

Three commands, one piece-cid back. See the [CLI docs](https://docs.prova.network/cli/auth).

### 3. SDK

```ts
import { Prova } from '@prova-network/sdk'
const prova = Prova.create({ account, chain: base })
const { cid, dealId } = await prova.storage.upload(bytes)
```

For programmatic / on-chain workflows. See the [SDK docs](https://docs.prova.network/sdk/).

---

## Status

| Component | State |
| --- | --- |
| Solidity contracts | 8 contracts, **81 unit tests passing**, full deal lifecycle (happy + fault) verified on local anvil |
| Go prover (`provad`) | 13 packages, end-to-end validated against local anvil |
| TypeScript SDK | forked from FilOzone/synapse-sdk, renamed to Prova, building |
| CLI | `@prova-network/cli` — single Node binary, zero deps |
| Web (drag-drop + dashboard + API) | live at prova.network |
| Docs site | live at docs.prova.network |
| Spec site | live at spec.prova.network (per-section state + audit metadata) |
| Stage server (testnet R2 substitute) | live at p.prova.network (Hetzner) |
| Base Sepolia deploy | in progress |
| External audit | post-Sepolia, pre-mainnet |
| Mainnet | post-audit, target H2 2026 |

For a current scope-of-work breakdown (what the maintainer is actively touching, where contributors should focus), see [`STATUS.md`](./STATUS.md).

---

## Credits

Prova reuses the cryptographic primitives and patterns the Filecoin community has refined over years of production use: PDP, CommP, Fr32 padding, sha2-256-trunc254-padded multihashing, piece-commitment CIDs. Prova ports them onto Base as a separate, coexistent network with stablecoin-denominated client payments and PROVA-denominated prover stake.

Key upstreams (all permissively licensed):

- [`filecoin-project/specs`](https://github.com/filecoin-project/specs) — foundational specs
- [`FilOzone/pdp`](https://github.com/FilOzone/pdp) — Solidity PDP verifier
- [`FilOzone/synapse-sdk`](https://github.com/FilOzone/synapse-sdk) — TypeScript SDK ancestry
- [`filecoin-project/curio`](https://github.com/filecoin-project/curio) — Go prover patterns

Full source map at [`FILECOIN-SOURCE-MAP.md`](./FILECOIN-SOURCE-MAP.md).

## License

Dual-licensed: [MIT](./LICENSE), or Apache-2.0 by way of the [Permissive License Stack](https://protocol.ai/blog/announcing-the-permissive-license-stack/). Forked components retain upstream attribution.
