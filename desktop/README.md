# Prova Desktop

_Electron app that wraps the Prova prover daemon (`provad`) in a consumer-friendly shell with a built-in wallet, auto-updater, and dashboard UI._

**Status:** early scaffold, post-transplant. Not yet runnable end-to-end. Confidentiality-gated with the rest of Prova.

---

## Why a desktop app

The main way to run a Prova prover is the headless `provad` daemon with systemd. That's correct for datacenter operators with 24/7 machines.

A lot of people have a workstation, a Mac mini in a closet, or a Hetzner VPS they're bored of. For them, the shortest path to "run a prover and earn" is:

1. Download a signed app
2. Double-click install
3. Let it create a wallet on first launch
4. Let it run in the background

Prova Desktop is that path. It spawns `provad` as a child process, surfaces its activity in a tray-icon app, and updates itself when new releases ship.

---

## What's in this tier

- **Retrieval-only prover** by default (serves cached pieces it's already received; no heavy storage allocation)
- **Optional full prover** for users who want to stake and commit disk
- **Built-in wallet** (BIP-39 seed stored in the OS keychain via `keytar`)
- **Activity feed + proof/deal counters** from `provad`'s structured logs
- **Auto-update** via `electron-updater` against GitHub releases
- **Tray icon** so the app runs unobtrusively in the background

---

## Provenance

This package was forked from [CheckerNetwork/desktop](https://github.com/CheckerNetwork/desktop) (originally `filecoin-station/desktop`). The upstream is archived as of 2025. License: Apache-2.0 OR MIT.

See [`ATTRIBUTION.md`](./ATTRIBUTION.md) for per-file provenance.

---

## Development

From the repo root:

```bash
# 1. Build the provad binary (the desktop app wraps it)
cd prover
go build -o provad ./cmd/provad

# 2. Install desktop deps + run in dev mode
cd ../desktop
npm install
npm start
```

The app will:
- Create a new wallet on first launch (check logs for the address)
- Look for `provad` at `../prover/provad`
- Tail its stdout and route events into the activity feed

`PROVA_ROOT=/tmp/prova-dev npm start` runs an isolated instance so you can blow away state between runs.

---

## Packaging

```bash
npm run package
```

Builds platform-specific installers:

| Platform | Output |
|---|---|
| macOS | `dist/Prova-<ver>-universal.dmg` |
| Windows | `dist/Prova-<ver>-x64.exe` |
| Linux | `dist/Prova-<ver>-x86_64.AppImage` |

The packaged app includes the `provad` binary for the target platform under `resources/provad/`. Cross-platform builds require the Go toolchain + `goreleaser` (see `../prover/` workflows).

---

## Layout

```
desktop/
├── main/             # Electron main process (Node.js)
│   ├── index.js      # Entry point, boot sequence
│   ├── provad.js     # Spawns + supervises the provad child process
│   ├── wallet.js     # BIP-39 seed, key management, signing
│   ├── ipc.js        # IPC channel definitions
│   ├── tray.js       # System tray integration
│   ├── updater.js    # Auto-update (electron-updater)
│   └── ...
├── renderer/         # React SPA (Vite-built)
│   └── src/
├── shared/           # Shared types between main + renderer
├── build/            # Packaging scripts (before-pack, notarize, entitlements)
├── assets/           # App icons
└── electron-builder.yml
```

---

## License

Apache-2.0 OR MIT (dual). See [`LICENSE.md`](./LICENSE.md) and [`ATTRIBUTION.md`](./ATTRIBUTION.md).
