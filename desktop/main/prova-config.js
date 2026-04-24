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

const Store = require('electron-store')

const log = require('electron-log').scope('config')

const ConfigKeys = Object.freeze({
  OnboardingCompleted: 'prova.OnboardingCompleted',
  TrayOperationExplained: 'prova.TrayOperationExplained',
  // Prover-side: feature opt-ins that persist across launches
  ProverModeEnabled: 'prova.ProverModeEnabled',
  AutoUpdateEnabled: 'prova.AutoUpdateEnabled'
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

module.exports = {
  getOnboardingCompleted,
  setOnboardingCompleted,
  getTrayOperationExplained,
  setTrayOperationExplained,
  getProverModeEnabled,
  setProverModeEnabled,
  getAutoUpdateEnabled,
  setAutoUpdateEnabled
}
