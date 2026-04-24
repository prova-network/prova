# Attribution — desktop/

Prova Desktop was forked from [CheckerNetwork/desktop](https://github.com/CheckerNetwork/desktop) (formerly `filecoin-station/desktop`). That project is archived upstream as of 2025; the Filecoin Spark and Checker Network programs continued in a permissioned architecture.

License: Apache-2.0 OR MIT (both upstream and in this fork).

## Per-file transplant status

| File | Status | Notes |
|---|---|---|
| `main/index.js` | modified | Rewrote boot sequence, renamed Sentry telemetry removed, renamed events to `prova:` prefix |
| `main/provad.js` | rewritten | Formerly `main/checker-node.js`. Replaced `fork()` of Node child with `spawn()` of Go binary; replaced Checker event schema with `provad` structured-log parsing |
| `main/wallet.js` | rewritten | Formerly `main/wallet.js` + `main/wallet-backend.js` (645 LOC). Reduced to ~250 LOC. Dropped FIL RPC polling, FIL transaction history, Glif dependencies. Kept BIP-39 + keytar pattern |
| `main/consts.js` | modified | Renamed `STATION_ROOT` → `PROVA_ROOT`, `appIDs` rewritten for Prova branding |
| `main/ipc.js` | rewritten | All channel names re-prefixed `station:` → `prova:`. Removed FIL-specific handlers, added Prova proof/deal/wallet events |
| `main/activities.js` | kept as-is | Generic activity log, no FIL-specific code |
| `main/logs.js` | kept as-is | Generic log buffer |
| `main/tray.js` | kept, minor updates | Branding strings only |
| `main/ui.js` | kept, minor updates | Window setup, no FIL logic |
| `main/updater.js` | kept as-is | Generic electron-updater wrapper |
| `main/app-menu.js` | kept, minor updates | Menu branding only |
| `main/preload.js` | pending | Needs renaming of exposed channels to match new `prova:*` IPC |
| `main/dialog.js` | kept as-is | Generic dialog helper |
| `main/settings.js` | kept as-is | Generic settings |
| `main/utils.js` | kept as-is | URL validation helper |
| `main/station-config.js` | pending rename/rewrite | Still contains `station*` keys; will rename to `prova-config.js` |
| `main/filforwarder-abi.json` | deleted | FIL-specific contract ABI |
| `main/setup-sentry.js` | deleted | No Sentry telemetry in Prova Desktop |
| `main/telemetry.js` | deleted | InfluxDB telemetry endpoint |
| `main/wallet-backend.js` | deleted | FIL RPC wallet backend |
| `main/test/wallet-backend.test.js` | deleted | tests for deleted module |
| `main/test/wallet.test.js` | pending rewrite | covers old wallet API |
| `renderer/` | pending overhaul | Original React UI is FIL-centric; will rebuild against Prova API |
| `shared/typings.ts` | pending rewrite | `FILTransaction` types etc. need Prova equivalents |
| `build/` | kept as-is | Packaging scripts (notarize-macos, before-pack, after-pack) are generic |
| `electron-builder.yml` | rewritten | All identifiers rebranded; `extraResources` pulls `provad` binary |
| `package.json` | rewritten | Renamed, dropped `@checkernetwork/node` + FIL deps, bumped ethers v5 → v6 |

## License headers

Files we've kept but modified retain a dual-license SPDX header:

```
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright (c) Protocol Labs (original), Prova Network contributors (modifications).
// Forked from CheckerNetwork/desktop ...
```

Files we've rewritten from scratch get:

```
// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Prova Network contributors.
```

## What we did not take

- `@checkernetwork/node` — the actual worker. Replaced entirely by our Go `provad`.
- `@filecoin-station/spark-impact-evaluator` — Filecoin-Spark-specific.
- Glif Filecoin stack (`@glif/filecoin-address`, `@glif/filecoin-number`, etc).
- Sentry integration.
- InfluxDB telemetry.
- FIL transaction history UI.
- Station-era onboarding flow (replaced by fresh Prova wallet-pairing flow).
