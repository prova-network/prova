// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Prova Network contributors.
//
// Shared types between the Electron main process and the renderer.
// Keep this file lean; per-process types live in `main/typings.ts` and
// `renderer/src/api.ts` for their respective owners.

/// One row in the user-facing activity feed.
///
/// The renderer (renderer/src/api.ts) declares its own `Activity` type
/// for its IPC bridge. Keep the two structurally compatible: any field
/// added here that the renderer should see must also be added there.
export type Activity = {
  id: string
  /// `started` is reserved for one-shot lifecycle events (process boot,
  /// data set created, etc.). `info` is the day-to-day stream. `error`
  /// is anything the supervisor or daemon classified as a failure.
  type: 'info' | 'error' | 'started'
  /// Subsystem that emitted the event. Examples: 'supervisor', 'provad',
  /// 'engine', 'httpserver', 'wallet'. Used by activities.js to decide
  /// whether the daemon is "online" and shown in the tray icon.
  source: string
  message: string
  timestamp: Date | string
}

/// Result of `wallet.setup()`. `isNew` is true on first launch (a fresh
/// mnemonic was generated) and false on every subsequent boot.
export interface WalletSeed {
  address: string
  isNew: boolean
}
