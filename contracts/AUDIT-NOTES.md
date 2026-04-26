# Audit notes

Living record of static-analysis findings, manual reviews, and the
disposition of each. Read this before opening or reviewing a security PR.

## Slither (suite-wide static analysis)

### Currently zero open findings

`slither . --config-file slither.config.json` returns clean (analyzed 67
contracts with 54 detectors, 0 result(s) found).

### Disposition log

| Finding | First seen | Disposition | Why |
| --- | --- | --- | --- |
| `reentrancy-no-eth` in `StorageMarketplace.possessionProven` | 2026-04-26 | **Fixed** | Reordered to strict CEI: state writes → internal computation → token transfers → optional rewards-hook callback. Even with try/catch, the prior order let a hostile rewards contract observe stale `paidOut`. Now `paidOut` is updated before any external call. |
| `divide-before-multiply` in `ProverRewards.rewardOf` and `ProverStaking._requiredStake` | 2026-04-26 | **Suppressed (intentional)** | The TIB-ceiling math (`tibRequired = (bytes + TIB - 1) / TIB`) is per-TiB by design — we want a per-TiB fee unit, not a per-byte one. Slither flags any `(div) * x` even when the precision loss is the desired behavior. Filtered globally. |
| `incorrect-equality` on `balance == 0`, `status == DealStatus.Active`, etc. | 2026-04-26 | **Suppressed (false positive)** | Strict equality on `uint256 == 0` and on enum values is the standard pattern; slither's heuristic flags all `==` comparisons. |
| `unused-return` on `priceOracle.latestRoundData()` | 2026-04-26 | **Suppressed (intentional)** | Chainlink's `latestRoundData` returns `(roundId, answer, startedAt, updatedAt, answeredInRound)`. We use `answer` and `updatedAt` and ignore the round-tracking fields, which is standard for staleness-only consumers. |
| `incorrect-exp` in `Proofs.sol` (`index ^ 1`) | 2026-04-26 | **Suppressed (upstream)** | `^` is XOR for sibling-pair lookup in the Merkle proof. Slither misreads it as exponentiation. Inherited from `FilOzone/pdp`; we do not modify upstream cryptography. |

### Configuration

`slither.config.json` excludes:

- Files under `lib/` (dependencies), `test/`, `script/`, and the upstream PDP fork (`Proofs.sol`, `ProofVerifier.sol`, `Cids.sol`, `Fees.sol`, `BitOps.sol`). We trust upstream's audit surface for those; running slither over them produces noise, not signal.
- Mock contracts (`MockProofVerifier.sol`, `MockUSDC.sol`, `MockPriceOracle.sol`) — test fixtures only.
- Detectors that fire on intentional patterns: `naming-convention`, `solc-version`, `assembly`, `timestamp`, `incorrect-exp`, `divide-before-multiply`, `incorrect-equality`, `unused-return`.

We will tighten this over time. The current shape is "production code, our additions, severity ≥ medium."

## forge

| Check | Status |
| --- | --- |
| `forge build --sizes` | clean |
| `forge test` | 106/106 passing across 7 suites |
| `forge fmt --check` | advisory (some files prefer different paragraphing); enforce later |
| `forge snapshot` | baseline committed at `.gas-snapshot` |

## Manual review history

| Commit | Reviewer | Scope | Findings |
| --- | --- | --- | --- |
| internal pre-testnet | Capri | full protocol surface | `SECURITY-AUDIT-2026-04-25.md` (16 findings, 4 closed, rest pre-testnet must-fix) |

## Next reviews planned

- **Tier-1 external audit** scoped before mainnet. Funded from SAFT proceeds. Six-week scope. Findings published in full.
- **Re-audit** when any of the following change: slashing math, emission schedule, ProofVerifier UUPS upgrade, or a new contract enters the trust boundary.
