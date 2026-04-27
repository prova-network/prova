// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright (c) Protocol Labs (original), Prova Network contributors (modifications).
//
// IPC channels between the Electron main process and the renderer.
//
// Naming: all channels are prefixed `prova:` to make grep-across-the-codebase
// straightforward and to avoid collisions with any embedded webview.

'use strict'

const { ipcMain, dialog } = require('electron')
const fs = require('node:fs/promises')

const wallet = require('./wallet')
const provaConfig = require('./prova-config')
const provad = require('./provad')

/** @typedef {import('./typings').Context} Context */

// Event names emitted from the main process via ipcMain.emit. The renderer
// receives them through the preload script's typed wrapper.
const ipcMainEvents = Object.freeze({
  ACTIVITY_LOGGED: 'prova:activity-logged',
  PROOF_STATS_UPDATED: 'prova:proof-stats-updated',
  DEALS_ACTIVE_UPDATED: 'prova:deals-active-updated',
  WALLET_ADDRESS_UPDATED: 'prova:wallet-address-updated',
  STORAGE_DIR_CHANGED: 'prova:storage-dir-changed',
  NETWORK_CHANGED: 'prova:network-changed',
  DAEMON_STATUS_CHANGED: 'prova:daemon-status-changed',

  UPDATE_CHECK_STARTED: 'prova:update-check:started',
  UPDATE_CHECK_FINISHED: 'prova:update-check:finished',
  READY_TO_UPDATE: 'prova:ready-to-update'
})

function setupIpcMain (/** @type {Context} */ ctx) {
  // Wallet
  ipcMain.handle('prova:getWalletAddress', () => wallet.getAddress())
  ipcMain.handle('prova:signMessage', (_e, msg) => wallet.signMessage(msg))
  ipcMain.handle('prova:exportSeedPhrase', () => ctx.exportSeedPhrase())
  ipcMain.handle(
    'prova:importSeedPhrase',
    (_e, phrase) => wallet.importMnemonic(phrase)
  )

  // Activity feed & stats
  ipcMain.handle('prova:getActivities', () => ctx.getActivities())
  ipcMain.handle(
    'prova:getTotalProofsSubmitted',
    () => ctx.getTotalProofsSubmitted()
  )
  ipcMain.handle(
    'prova:getTotalDealsActive',
    () => ctx.getTotalDealsActive()
  )

  // Lifecycle & updater
  ipcMain.handle('prova:restartToUpdate', () => ctx.restartToUpdate())
  ipcMain.handle('prova:openReleaseNotes', () => ctx.openReleaseNotes())
  ipcMain.handle('prova:getUpdaterStatus', () => ctx.getUpdaterStatus())
  ipcMain.handle('prova:checkForUpdates', () => ctx.manualCheckForUpdates())
  ipcMain.handle('prova:toggleOpenAtLogin', () => ctx.toggleOpenAtLogin())
  ipcMain.handle('prova:isOpenAtLogin', () => ctx.isOpenAtLogin())

  // Logs & external URLs
  ipcMain.handle('prova:saveLogsAs', () => ctx.saveModuleLogsAs())
  ipcMain.handle('prova:openExternalURL', (_e, url) => ctx.openExternalURL(url))

  // ── Storage location (folder/drive picker) ─────────────────────────
  ipcMain.handle('prova:getStorageDir', () => ({
    current: provaConfig.getStorageDir(),
    default: provaConfig.getDefaultStorageDir(),
    isCustom: !!provaConfig.getStorageDir() && provaConfig.getStorageDir() !== provaConfig.getDefaultStorageDir()
  }))
  ipcMain.handle('prova:selectStorageDir', async () => {
    // Open a native folder picker. The current value is the starting
    // point so the dialog opens where the user expects.
    const current = provaConfig.getStorageDir()
    const res = await dialog.showOpenDialog({
      title: 'Choose Prova storage location',
      defaultPath: current,
      buttonLabel: 'Use this folder',
      message: 'Pieces will be stored here. An external drive works fine.',
      properties: ['openDirectory', 'createDirectory', 'dontAddToRecent']
    })
    if (res.canceled || res.filePaths.length === 0) return null
    const chosen = res.filePaths[0]
    // Make sure the directory is writable before persisting.
    try {
      await fs.mkdir(chosen, { recursive: true })
      const probe = require('node:path').join(chosen, '.prova-write-probe')
      await fs.writeFile(probe, '')
      await fs.unlink(probe)
    } catch (/** @type {unknown} */ err) {
      const msg = err instanceof Error ? err.message : String(err)
      throw new Error(`Selected folder is not writable: ${msg}`)
    }
    provaConfig.setStorageDir(chosen)
    ipcMain.emit(ipcMainEvents.STORAGE_DIR_CHANGED, chosen)
    return chosen
  })
  ipcMain.handle('prova:resetStorageDir', () => {
    provaConfig.setStorageDir('')
    const reset = provaConfig.getStorageDir()
    ipcMain.emit(ipcMainEvents.STORAGE_DIR_CHANGED, reset)
    return reset
  })

  // ── First-run onboarding ──────────────────────────────────────────────────────────────
  // The renderer asks for this on boot. If `firstRunWalletAddress` is
  // set on the context (assigned by main/index.js when wallet.setup()
  // reported a new wallet), and the user hasn't completed onboarding
  // yet, the renderer surfaces a 'back up your seed' banner.
  ipcMain.handle('prova:getOnboardingState', () => ({
    completed: provaConfig.getOnboardingCompleted(),
    firstRunWalletAddress: ctx.firstRunWalletAddress || ''
  }))
  ipcMain.handle('prova:completeOnboarding', () => {
    provaConfig.setOnboardingCompleted()
    return true
  })

  // ── Network preset selection (anvil / base-sepolia / base-mainnet) ────────
  ipcMain.handle('prova:listNetworks', () => provaConfig.listNetworkPresets())
  ipcMain.handle('prova:getNetwork', () => provaConfig.getNetworkConfig())
  ipcMain.handle('prova:setNetwork', (_e, key) => {
    provaConfig.setNetwork(typeof key === 'string' ? key : 'anvil')
    const cfg = provaConfig.getNetworkConfig()
    ipcMain.emit(ipcMainEvents.NETWORK_CHANGED, cfg)
    return cfg
  })

  // ── Daemon status (running / starting / failing) ───────────────────────────────
  ipcMain.handle('prova:getDaemonStatus', () => provad.getDaemonStatus())

  // ── Aggregated prover stats (storage / earnings / staked / deals / proofs) ───────
  ipcMain.handle('prova:getProverStats', () => provad.getProverStats())
}

module.exports = {
  setupIpcMain,
  ipcMainEvents
}
