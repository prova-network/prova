// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright (c) Protocol Labs (original station-config.js), Prova Network contributors (modifications).
// Forked from CheckerNetwork/desktop main/station-config.js.
//
// Persistent app configuration: UI state (onboarding seen, tray hint
// shown), and any other non-secret key/value pairs that survive app
// restarts. Secrets (seed phrase) live in the OS keychain via keytar
// in wallet.js.
//
// The underlying store is electron-store (JSON file under the app's
// userData directory). Keys are namespaced under `prova.` so they
// can coexist with any future Station migration blob without conflict.

'use strict'

const path = require('node:path')
const Store = require('electron-store')
const { app } = require('electron')

const log = require('electron-log').scope('config')

const ConfigKeys = Object.freeze({
  OnboardingCompleted: 'prova.OnboardingCompleted',
  TrayOperationExplained: 'prova.TrayOperationExplained',
  // Prover-side: feature opt-ins that persist across launches
  ProverModeEnabled: 'prova.ProverModeEnabled',
  AutoUpdateEnabled: 'prova.AutoUpdateEnabled',
  // Where the local piece store writes data. Empty string means "use
  // the default under the app's userData directory". Users can point
  // this at any folder, including an external drive, via the UI.
  StorageDir: 'prova.StorageDir'
})

const configStore = new Store({
  migrations: {
    // Migrate any prior Station-era keys (unlikely for net-new Prova
    // installs but cheap to handle if someone carries over a config
    // between the archived Station app and Prova Desktop).
    '>=0.1.0': store => {
      const legacyMap = [
        ['station.OnboardingCompleted', ConfigKeys.OnboardingCompleted],
        ['station.TrayOperationExplained', ConfigKeys.TrayOperationExplained]
      ]
      for (const [from, to] of legacyMap) {
        if (store.has(from) && !store.has(to)) {
          store.set(to, store.get(from))
        }
      }
    }
  },
  beforeEachMigration: (_, context) => {
    log.info(
      `Migrating prova-config from ${context.fromVersion} → ${context.toVersion}`
    )
  }
})

log.info('Loading Prova configuration from', configStore.path)

let OnboardingCompleted =
  /** @type {boolean} */ (configStore.get(ConfigKeys.OnboardingCompleted, false))
let TrayOperationExplained =
  /** @type {boolean} */ (configStore.get(ConfigKeys.TrayOperationExplained, false))
let ProverModeEnabled =
  /** @type {boolean} */ (configStore.get(ConfigKeys.ProverModeEnabled, false))
let AutoUpdateEnabled =
  /** @type {boolean} */ (configStore.get(ConfigKeys.AutoUpdateEnabled, true))
let StorageDir =
  /** @type {string} */ (configStore.get(ConfigKeys.StorageDir, ''))

/** @returns {boolean} */
function getOnboardingCompleted () { return !!OnboardingCompleted }
function setOnboardingCompleted () {
  OnboardingCompleted = true
  configStore.set(ConfigKeys.OnboardingCompleted, OnboardingCompleted)
}

/** @returns {boolean} */
function getTrayOperationExplained () { return !!TrayOperationExplained }
function setTrayOperationExplained () {
  TrayOperationExplained = true
  configStore.set(ConfigKeys.TrayOperationExplained, TrayOperationExplained)
}

/** @returns {boolean} */
function getProverModeEnabled () { return !!ProverModeEnabled }
/** @param {boolean} v */
function setProverModeEnabled (v) {
  ProverModeEnabled = !!v
  configStore.set(ConfigKeys.ProverModeEnabled, ProverModeEnabled)
}

/** @returns {boolean} */
function getAutoUpdateEnabled () { return !!AutoUpdateEnabled }
/** @param {boolean} v */
function setAutoUpdateEnabled (v) {
  AutoUpdateEnabled = !!v
  configStore.set(ConfigKeys.AutoUpdateEnabled, AutoUpdateEnabled)
}

/**
 * Default piece-store directory. Users see this on first launch; they
 * can change it via the Storage panel and the change persists across
 * restarts. We pick a directory inside `userData` so the OS file system
 * inherits the app's normal lifecycle (Time Machine excluded under
 * `Library/Application Support` per Apple's recommendation).
 *
 * @returns {string}
 */
function getDefaultStorageDir () {
  // app may be undefined during early Electron module load; guard for it.
  if (!app || typeof app.getPath !== 'function') {
    return path.join(process.cwd(), 'pieces')
  }
  return path.join(app.getPath('userData'), 'pieces')
}

/**
 * Effective storage directory: user-selected if non-empty, otherwise
 * the per-platform default.
 *
 * @returns {string}
 */
function getStorageDir () {
  if (StorageDir && StorageDir.length > 0) return StorageDir
  return getDefaultStorageDir()
}

/**
 * Set a user-selected storage directory. Empty string resets to default.
 *
 * @param {string} dir
 */
function setStorageDir (dir) {
  StorageDir = (dir || '').trim()
  configStore.set(ConfigKeys.StorageDir, StorageDir)
}

module.exports = {
  getOnboardingCompleted,
  setOnboardingCompleted,
  getTrayOperationExplained,
  setTrayOperationExplained,
  getProverModeEnabled,
  setProverModeEnabled,
  getAutoUpdateEnabled,
  setAutoUpdateEnabled,
  getStorageDir,
  setStorageDir,
  getDefaultStorageDir
}
