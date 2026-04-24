// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright (c) Protocol Labs (original), Prova Network contributors (modifications).
// Forked from CheckerNetwork/desktop.
//
// Preload script: the only surface the renderer process can access from
// outside the browser sandbox. We use `contextBridge.exposeInMainWorld`
// with contextIsolation enabled; no direct ipcRenderer access leaks into
// window scope.
//
// Channel naming mirrors main/ipc.js — keep them in sync.

'use strict'

const { contextBridge, ipcRenderer } = require('electron')

// Helper: wrap a main-process event broadcast into a subscribe-with-unsubscribe
// pattern that's easier to use from React useEffect.
function subscribe (channel, handler) {
  const listener = (_event, ...args) => handler(...args)
  ipcRenderer.on(channel, listener)
  return () => ipcRenderer.removeListener(channel, listener)
}

contextBridge.exposeInMainWorld('electron', {
  // Build metadata injected by main/index.js via process.env.
  buildVersion: process.env.PROVA_BUILD_VERSION || 'dev',

  // ─── Wallet ─────────────────────────────────────────────────────────
  getWalletAddress: () => ipcRenderer.invoke('prova:getWalletAddress'),
  signMessage: (msg) => ipcRenderer.invoke('prova:signMessage', msg),
  exportSeedPhrase: () => ipcRenderer.invoke('prova:exportSeedPhrase'),
  importSeedPhrase: (phrase) =>
    ipcRenderer.invoke('prova:importSeedPhrase', phrase),

  // ─── Prover stats ──────────────────────────────────────────────────
  getTotalProofsSubmitted: () =>
    ipcRenderer.invoke('prova:getTotalProofsSubmitted'),
  getTotalDealsActive: () =>
    ipcRenderer.invoke('prova:getTotalDealsActive'),
  getActivities: () => ipcRenderer.invoke('prova:getActivities'),

  // ─── Lifecycle & updater ───────────────────────────────────────────
  restartToUpdate: () => ipcRenderer.invoke('prova:restartToUpdate'),
  openReleaseNotes: () => ipcRenderer.invoke('prova:openReleaseNotes'),
  getUpdaterStatus: () => ipcRenderer.invoke('prova:getUpdaterStatus'),
  checkForUpdates: () => ipcRenderer.invoke('prova:checkForUpdates'),
  toggleOpenAtLogin: () => ipcRenderer.invoke('prova:toggleOpenAtLogin'),
  isOpenAtLogin: () => ipcRenderer.invoke('prova:isOpenAtLogin'),

  // ─── Logs & external URLs ──────────────────────────────────────────
  saveLogsAs: () => ipcRenderer.invoke('prova:saveLogsAs'),
  openExternalURL: (url) => ipcRenderer.invoke('prova:openExternalURL', url),

  // ─── Subscriptions ─────────────────────────────────────────────────
  // Each returns an unsubscribe function for useEffect cleanup.
  onActivityLogged: (cb) => subscribe('prova:activity-logged', cb),
  onProofStatsUpdated: (cb) => subscribe('prova:proof-stats-updated', cb),
  onDealsActiveUpdated: (cb) => subscribe('prova:deals-active-updated', cb),
  onWalletAddressUpdated: (cb) => subscribe('prova:wallet-address-updated', cb),
  // Updater state is carried by three distinct events in main/updater.js:
  //   UPDATE_CHECK_STARTED   -> 'checking'
  //   UPDATE_CHECK_FINISHED  -> 'idle'  (no update available)
  //   READY_TO_UPDATE        -> 'ready'
  // We normalize them into a single callback surface here so the renderer
  // has one subscription to manage.
  onUpdaterStatusChanged: (cb) => {
    const checkingHandler = () => cb('checking')
    const finishedHandler = () => cb('idle')
    const readyHandler = () => cb('ready')
    ipcRenderer.on('prova:update-check:started', checkingHandler)
    ipcRenderer.on('prova:update-check:finished', finishedHandler)
    ipcRenderer.on('prova:ready-to-update', readyHandler)
    return () => {
      ipcRenderer.removeListener('prova:update-check:started', checkingHandler)
      ipcRenderer.removeListener('prova:update-check:finished', finishedHandler)
      ipcRenderer.removeListener('prova:ready-to-update', readyHandler)
    }
  }
})
