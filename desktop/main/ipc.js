// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright (c) Protocol Labs (original), Prova Network contributors (modifications).
//
// IPC channels between the Electron main process and the renderer.
//
// Naming: all channels are prefixed `prova:` to make grep-across-the-codebase
// straightforward and to avoid collisions with any embedded webview.

'use strict'

const { ipcMain } = require('electron')

const wallet = require('./wallet')

/** @typedef {import('./typings').Context} Context */

// Event names emitted from the main process via ipcMain.emit. The renderer
// receives them through the preload script's typed wrapper.
const ipcMainEvents = Object.freeze({
  ACTIVITY_LOGGED: 'prova:activity-logged',
  PROOF_STATS_UPDATED: 'prova:proof-stats-updated',
  DEALS_ACTIVE_UPDATED: 'prova:deals-active-updated',
  WALLET_ADDRESS_UPDATED: 'prova:wallet-address-updated',

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
}

module.exports = {
  setupIpcMain,
  ipcMainEvents
}
