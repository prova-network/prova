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
  StorageDir: 'prova.StorageDir',
  // Which chain the prover talks to. One of: 'anvil' | 'base-sepolia'
  // | 'base-mainnet'. Default is 'anvil' so a freshly-installed copy
  // boots toward a local devnet instead of trying (and failing) to
  // hit a real chain unconfigured.
  Network: 'prova.Network'
})

// Chain presets the desktop ships with. Contract addresses are blank
// until each chain has its Prova v2 contract suite deployed; the
// desktop surfaces "Configure contracts" guidance in that case.
const NetworkPresets = Object.freeze({
  anvil: {
    label: 'Local anvil (dev)',
    rpcUrl: 'http://127.0.0.1:8545',
    chainId: 31337,
    // These are the deterministic addresses produced by
    // contracts/script/Deploy.s.sol when run against a fresh anvil
    // instance with the standard pre-funded account 0 as deployer
    // (anvil account 0 = 0xf39Fd6e51aad88F6F4ce6aB8827279cfFFb92266,
    // private key ac0974...). Re-running the deploy script on the
    // same anvil run produces the same addresses; restarting anvil
    // resets nonces, in which case re-running the script reproduces
    // them again.
    contracts: {
      provaToken:         '0xe7f1725E7734CE288F8367e1Bb143E90bb3F0512',
      proofVerifier:      '0x5FC8d32690cc91D4c39d9d3abcBD16989F875707',
      proverRegistry:     '0x9fE46736679d2D9a65F0992F2272dE9f3c7fa6e0',
      proverStaking:      '0xCf7Ed3AccA5a467e9e704C703E8D87F634fB0Fc9',
      contentRegistry:    '0xDc64a140Aa3E981100a9becA4E685f962f0cF6C9',
      storageMarketplace: '0xa513E6E4b8f2a923D98304ec87F64353C4D5C853'
    }
  },
  'base-sepolia': {
    label: 'Base Sepolia (testnet)',
    rpcUrl: 'https://sepolia.base.org',
    chainId: 84532,
    contracts: {
      provaToken: '',
      proofVerifier: '',
      proverRegistry: '',
      proverStaking: '',
      contentRegistry: '',
      storageMarketplace: ''
    }
  },
  'base-mainnet': {
    label: 'Base mainnet',
    rpcUrl: 'https://mainnet.base.org',
    chainId: 8453,
    contracts: {
      provaToken: '',
      proofVerifier: '',
      proverRegistry: '',
      proverStaking: '',
      contentRegistry: '',
      storageMarketplace: ''
    }
  }
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
let Network =
  /** @type {string} */ (configStore.get(ConfigKeys.Network, 'anvil'))

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

/**
 * Currently selected network preset key.
 * @returns {'anvil' | 'base-sepolia' | 'base-mainnet'}
 */
function getNetwork () {
  const presets = /** @type {Record<string, unknown>} */ (NetworkPresets)
  if (Network && presets[Network]) {
    return /** @type {'anvil' | 'base-sepolia' | 'base-mainnet'} */ (Network)
  }
  return 'anvil'
}

/**
 * Set the active chain preset. Unknown values fall back to 'anvil'.
 *
 * @param {string} v
 */
function setNetwork (v) {
  const presets = /** @type {Record<string, unknown>} */ (NetworkPresets)
  if (!presets[v]) v = 'anvil'
  Network = v
  configStore.set(ConfigKeys.Network, Network)
}

/**
 * Resolved chain configuration for the active network: rpcUrl, chainId,
 * contract addresses, and a friendly label for the UI.
 */
function getNetworkConfig () {
  const key = getNetwork()
  const presets = /** @type {Record<string, typeof NetworkPresets['anvil']>} */ (NetworkPresets)
  const preset = presets[key]
  return { key, ...preset }
}

/**
 * List every chain preset the desktop knows about, for UI selectors.
 */
function listNetworkPresets () {
  return Object.entries(NetworkPresets).map(([key, preset]) => ({
    key,
    label: preset.label,
    rpcUrl: preset.rpcUrl,
    chainId: preset.chainId,
    /** True if the preset has all six contract addresses set. */
    isConfigured: Object.values(preset.contracts).every(addr => addr && addr.length > 0)
  }))
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
  getDefaultStorageDir,
  getNetwork,
  setNetwork,
  getNetworkConfig,
  listNetworkPresets
}
