# STATUS — current state and scope-of-work

**Last updated:** 2026-04-26
**Read this before contributing or before opening a PR.**

## Posture

The project is **public**. The `prova-network` org and every repo under it is open-source. There is no NDA, no embargo, no "wait until X is done" rule. Earlier drafts of the README and `PIVOT.md` mentioned a confidentiality constraint that no longer applies — those files have been removed or rewritten and the constraint is **lifted**.

If an AI agent or new contributor reads an old README snapshot and panics about confidentiality, point them at this file.

## What's live

| Surface | URL | Status |
| --- | --- | --- |
| Marketing site | [prova.network](https://prova.network) | live |
| Upload UI | [prova.network/upload](https://prova.network/upload/) | live, sponsored 100 MB tier |
| App dashboard | [prova.network/app](https://prova.network/app/) | live |
| Whitepaper | [prova.network/whitepaper](https://prova.network/whitepaper) | v1.0 specification |
| Docs | [docs.prova.network](https://docs.prova.network) | live |
| Spec | [spec.prova.network](https://spec.prova.network) | live, structured per `spec/` |
| GitHub org | [github.com/prova-network](https://github.com/prova-network) | 10 public repos |
| Stage server | `p.prova.network` (Hetzner) | testnet R2 substitute |

Nothing is on Base mainnet yet. Base Sepolia testnet deploy is the next milestone.

## Test status

| Suite | Tests | Status |
| --- | --- | --- |
| `contracts/` (Foundry) | 81 | ✅ all passing |

`forge test` from `contracts/` reproduces the run.

## Active work — areas the maintainer is touching this week

To avoid PR conflicts, please **don't open PRs against** these areas without coordinating first:

| Area | Why | Coordinate via |
| --- | --- | --- |
| `contracts/src/ProverRewards.sol` and tests | Just landed; under iteration | issue first |
| `contracts/src/StorageMarketplace.sol` | Wiring the rewards hook | issue first |
| `website/whitepaper-source.md` | Pinned to v1.0 spec | issue first |
| `TOKENOMICS-2026.md` | Same | issue first |
| `legal/private/` | Internal staging, gitignored | don't touch |

## Where contributors should focus

These are real, scoped, contributor-friendly areas where help is welcome and won't conflict with active maintainer work:

### A. `prover/` — the Go daemon

- Improve test coverage in `prover/pkg/deal/`, especially the engine state machine.
- Wire the `provad` retrieval HTTP server to honor the security headers documented in [spec §4.1.3](https://spec.prova.network/network-protocol#4-1-3-retrieval).
- Implement structured Prometheus metrics for: bytes proven, missed challenges, retrieval throughput.
- Documentation in `prover/README.md`; see `prover/ROADMAP.md` for unchecked items.

### B. `cli/` — the `prova` CLI

- Add `prova hash <file>` (compute and print piece-CID without uploading). The internal helper exists in `cli/src/util/hash.mjs`; just needs a command + tests.
- Add `prova verify <cid> <file>` (recompute piece-CID from a local file, compare to the given CID, exit 0/1).
- Improve error messages when the API returns a non-200 (currently raw JSON dump).

### C. `sdk/` — the TypeScript SDK

- Cherry-pick from upstream `FilOzone/synapse-sdk` cleanly. The current sdk fork has drift; we need a maintainer to do periodic upstream merges.
- Port the `Prova` client from the placeholder in `sdk/typescript/sdk/src/`.

### D. `docs-gitbook/` — the docs site

- Open issues for any page that's wrong, stale, or unclear.
- Submit a PR for the page directly. Each page is a single `.md` file under `docs-gitbook/` that maps 1:1 to the URL on docs.prova.network.

### E. `spec/` and `spec-site/` — the protocol spec

- Spec content lives in `spec/*.md` (markdown source).
- Rendered presentation lives in `spec-site/*.md` (with VitePress metadata blocks).
- For now, edits go into both. Soon we'll automate the sync.
- Sections marked **Draft / WIP** in [the status overview](https://spec.prova.network/status) are open for help: §2.3 Data availability, §4.1 Network protocol, §5.2 Governance.

### F. `desktop/` — the Electron prover wrapper

- Cross-platform install/auto-update tests on Windows + Linux (we mostly test on macOS).
- Improve the local dashboard's USDC earnings + PROVA stake displays.

## How to coordinate

1. **Look at [open issues](https://github.com/prova-network/prova/issues) first.** If the area you want to work on has an issue, comment there.
2. **Open a new issue before a non-trivial PR.** A 1-paragraph "I plan to do X" prevents two people doing the same thing.
3. **For tiny PRs (typo, broken link, comment fix): just send the PR.** No issue needed.
4. **Email `hello@prova.network`** if you'd rather coordinate privately (e.g., security disclosure, enterprise prover onboarding).

## What graphify says about this codebase

We index the repo with [graphify](https://github.com/safishamsi/graphify). The output is in [`graphify-out/`](./graphify-out/):

- [`GRAPH_REPORT.md`](./graphify-out/GRAPH_REPORT.md) — communities, god nodes, surprises
- [`graph.html`](./graphify-out/graph.html) — interactive graph (open locally)
- [`graph.json`](./graphify-out/graph.json) — queryable structure

Latest run: 1661 nodes, 3078 edges, 57 communities. AST-only pass; semantic pass over docs is on the to-do list. Re-run with `bash scripts/graph.sh full` (long) or `bash scripts/graph.sh contracts` (fast).

## License + governance

Code: Apache-2.0 OR MIT. Brand: CC-BY-4.0. Governance: see [`spec/governance.md`](./spec/governance.md). Security disclosures: `security@prova.network`.
