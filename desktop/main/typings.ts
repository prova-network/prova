// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright (c) Protocol Labs (original), Prova Network contributors (modifications).
// Forked from CheckerNetwork/desktop main/typings.ts.
//
// Strongly-typed surface for the Electron main process. Re-exported from
// JSDoc `@typedef`s on the JS files so we get cross-file type inference
// without converting the whole main/ to TypeScript.

import { Activity, WalletSeed } from '../shared/typings'

export type { Activity, WalletSeed }

/// Updater-status snapshot exposed to the renderer. The shape mirrors
/// the events emitted from `main/updater.js`.
export interface UpdaterStatus {
  readyToUpdate: boolean
}

/// Shared state plumbing object passed to every main-process module.
/// Modules attach their handles to it during `setup()` and read what
/// other modules attached when they need to call across boundaries.
///
/// Modules MUST treat unattached fields as "not yet wired" and throw
/// rather than silently no-op. The boot sequence in `main/index.js`
/// initializes every field with a `not-wired` thrower so the failure
/// shows up loud.
export interface Context {
  // ── Activity feed ──────────────────────────────────────────────────
  recordActivity(activity: Activity): void
  getActivities(): Activity[]

  // ── Prover stats (incremented by provad supervisor) ────────────────
  getTotalProofsSubmitted(): number
  setTotalProofsSubmitted(count: number): void
  getTotalDealsActive(): number
  setTotalDealsActive(count: number): void

  // ── Wallet ─────────────────────────────────────────────────────────
  setWalletAddress(addr: string): void
  exportSeedPhrase: () => void | Promise<void>
  /// Set by main/index.js when wallet.setup() reports a brand-new wallet
  /// was created on this launch. The renderer reads this via
  /// `prova:getOnboardingState` to surface a 'back up your seed' banner.
  firstRunWalletAddress?: string

  // ── UI lifecycle ───────────────────────────────────────────────────
  showUI: () => void
  isShowingUI: boolean
  loadWebUIFromDist: import('electron-serve').loadURL

  // ── Updater (electron-updater) ─────────────────────────────────────
  manualCheckForUpdates: () => void
  saveModuleLogsAs: () => Promise<void>
  openReleaseNotes: () => void
  restartToUpdate: () => void
  getUpdaterStatus: () => UpdaterStatus

  // ── External URL handling (whitelist gate) ─────────────────────────
  openExternalURL: (url: string) => void

  // ── OS integration ─────────────────────────────────────────────────
  toggleOpenAtLogin: () => void
  isOpenAtLogin: () => boolean
}
