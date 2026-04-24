# Prova TypeScript SDK

Two packages, forked from [`FilOzone/synapse-sdk`](https://github.com/FilOzone/synapse-sdk) under the Permissive License Stack (Apache-2.0 OR MIT). See [ATTRIBUTION.md](./ATTRIBUTION.md).

## Packages

| Package | Purpose | Status |
|---------|---------|--------|
| [`@prova-network/core`](./core/) | Low-level primitives: ABIs, ERC-20 helpers, chain defs, PDP verifier bindings, session keys, typed data, piece/CommP helpers | Forked, renamed, **not yet compilable** — needs regenerated ABIs and Base chain config |
| [`@prova-network/sdk`](./sdk/) | High-level client SDK: `Prova` class, storage, payments, prover-registry | Forked, renamed, **not yet compilable** — depends on `core` |

## Status (as of 2026-04-24)

**Phase 7 of [PROVA-V2-PIVOT-PLAN](../../PROVA-V2-PIVOT-PLAN.md) is partially complete:**

### Done ✅
- Both packages forked from FilOzone/synapse-sdk (53,669 LOC TS → 31,233 LOC after trim)
- Package names: `@filoz/synapse-*` → `@prova-network/*`
- Top-level class: `Synapse` → `Prova` (`prova.ts`)
- Directory renames: `warm-storage/` → `marketplace/` (accurate per our architecture)
- Filecoin-specific modules dropped:
  - `filbeam/` (Filecoin's CDN retrieval service)
  - `usdfc.ts` (Filecoin's USDC equivalent)
  - `endorsements/` (Filecoin SP endorsement system)
- Attribution preserved: `ATTRIBUTION.md`, file-level SPDX pending on new files
- 1,045 `prova`/`Prova` references now live where `synapse`/`Synapse` used to be

### Remaining before compile ⚠️

These are the real engineering tasks, not mechanical renames:

1. **ABI regeneration.** `core/src/abis/generated.ts` is the Filecoin contracts bytecode. We need to regenerate using `wagmi generate` against our deployed Base contracts. Depends on **Phase 8** (Base Sepolia deploy).
2. **Chain definitions.** `core/src/chains.ts` contains Filecoin mainnet + calibration. Replace with Base mainnet + Base Sepolia. Cannot do fully without deployed contract addresses.
3. **Deep filbeam references.** 30 references to `filbeam`/`filBeam` in:
   - `marketplace/read-addresses.ts` (contract storage slot reader)
   - `pay/is-fwss-max-approved.ts` (comment about CDN settlement)
   - `utils/piece-url.ts` (URL construction for CDN retrieval)
   - `chains.ts` (CDN retrieval domain per chain)
   - `piece/resolve-piece-url.ts` (pluggable URL resolver)
   - `devnet/index.ts` (devnet config)
   Some need deletion, some need rewriting to point at Prova gateway.
4. **143 `filecoin` references.** Most in doc-comments and variable names inside mocks/ABIs. Some inside function bodies of `marketplace/` that need updating for our contracts.
5. **Mock system.** `core/src/mocks/jsonrpc/` is Filecoin-specific chain mocking. Needs regeneration against our contracts for the test suite to work.
6. **Dependencies.** Both `package.json`s updated but `pnpm-workspace.yaml` / `pnpm install` not run yet. Old synapse-sdk used pnpm catalog entries for some deps (`viem: catalog:`, etc.); replaced with pinned versions here.

### Not in scope for v1
- `filbeam/` module: Filecoin CDN is Filecoin-specific
- `endorsements/`: we use our own ProverRegistry instead
- `usdfc`: Base has canonical USDC
- Multi-chain support: Base only for v1

## Getting started (once compile-ready)

```typescript
import { Prova } from '@prova-network/sdk'
import { base } from '@prova-network/core/chains'

const prova = Prova.create({
  chain: base,
  account: myAccount,
})

const deal = await prova.storage.store(file, {
  durationDays: 365,
})

console.log('CommP:', deal.commP)
console.log('Deal ID:', deal.dealId)
```

## Next phase after this one

After Phase 8 (deploy to Base Sepolia) unblocks ABI regeneration, Phase 9 is a full end-to-end test:
- Upload a file from TS SDK
- Deal lands on Base Sepolia
- Prover (Go, forked from Curio) picks it up
- Proofs start flowing back

## License

Apache-2.0 OR MIT. Dual-licensed, same as upstream.
