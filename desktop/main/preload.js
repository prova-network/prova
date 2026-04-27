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
/**
 * @param {string} channel
 * @param {(...args: unknown[]) => void} handler
 */
function subscribe (channel, handler) {
  /** @type {(event: Electron.IpcRendererEvent, ...args: unknown[]) => void} */
  const listener = (_event, ...args) => handler(...args)
  ipcRenderer.on(channel, listener)
  return () => ipcRenderer.removeListener(channel, listener)
}

contextBridge.exposeInMainWorld('electron', {
  // Build metadata injected by main/index.js via process.env.
  buildVersion: process.env.PROVA_BUILD_VERSION || 'dev',

  // ─── Wallet ─────────────────────────────────────────────────────────
  getWalletAddress: () => ipcRenderer.invoke('prova:getWalletAddress'),
  /** @param {string} msg */
  signMessage: (/** @type {string} */ msg) => ipcRenderer.invoke('prova:signMessage', msg),
  exportSeedPhrase: () => ipcRenderer.invoke('prova:exportSeedPhrase'),
  importSeedPhrase: (/** @type {string} */ phrase) =>
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
  openExternalURL: (/** @type {string} */ url) => ipcRenderer.invoke('prova:openExternalURL', url),

  // ─── Storage location (folder picker) ──────────────────────────────
  /** @returns {Promise<{current:string,default:string,isCustom:boolean}>} */
  getStorageDir: () => ipcRenderer.invoke('prova:getStorageDir'),
  /** @returns {Promise<string|null>} */
  selectStorageDir: () => ipcRenderer.invoke('prova:selectStorageDir'),
  /** @returns {Promise<string>} */
  resetStorageDir: () => ipcRenderer.invoke('prova:resetStorageDir'),

  // ─── First-run onboarding ───────────────────────────────────────────────────
  /** @returns {Promise<{completed:boolean, firstRunWalletAddress:string}>} */
  getOnboardingState: () => ipcRenderer.invoke('prova:getOnboardingState'),
  /** @returns {Promise<boolean>} */
  completeOnboarding: () => ipcRenderer.invoke('prova:completeOnboarding'),

  // ─── Daemon status ──────────────────────────────────────────────────────
  /** @returns {Promise<{state:string,lastError:string,lastExitCode:number|null,consecutiveFailures:number}>} */
  getDaemonStatus: () => ipcRenderer.invoke('prova:getDaemonStatus'),

  // ─── Prover stats (storage, earnings, staked, deals, proofs) ───────────────────────────
  /** @returns {Promise<{bytesStored:number,piecesStored:number,dealsActive:number,proofsSubmitted:number,earnedUsdc:number|null,stakedProva:number|null,daemonUptimeSeconds:number|null}>} */
  getProverStats: () => ipcRenderer.invoke('prova:getProverStats'),

  // ─── Network preset selection ───────────────────────────────────────
  listNetworks: () => ipcRenderer.invoke('prova:listNetworks'),
  getNetwork: () => ipcRenderer.invoke('prova:getNetwork'),
  setNetwork: (/** @type {string} */ key) => ipcRenderer.invoke('prova:setNetwork', key),

  // ─── Subscriptions ─────────────────────────────────────────────────
  // Each returns an unsubscribe function for useEffect cleanup.
  /** @param {(activity: import('../shared/typings').Activity) => void} cb */
  onActivityLogged: (/** @type {(...args: unknown[]) => void} */ cb) =>
    subscribe('prova:activity-logged', cb),
  /** @param {(count: number) => void} cb */
  onProofStatsUpdated: (/** @type {(...args: unknown[]) => void} */ cb) =>
    subscribe('prova:proof-stats-updated', cb),
  /** @param {(count: number) => void} cb */
  onDealsActiveUpdated: (/** @type {(...args: unknown[]) => void} */ cb) =>
    subscribe('prova:deals-active-updated', cb),
  /** @param {(addr: string) => void} cb */
  onWalletAddressUpdated: (/** @type {(...args: unknown[]) => void} */ cb) =>
    subscribe('prova:wallet-address-updated', cb),
  /** @param {(dir: string) => void} cb */
  onStorageDirChanged: (/** @type {(...args: unknown[]) => void} */ cb) =>
    subscribe('prova:storage-dir-changed', cb),
  /** @param {(cfg: unknown) => void} cb */
  onNetworkChanged: (/** @type {(...args: unknown[]) => void} */ cb) =>
    subscribe('prova:network-changed', cb),
  /** @param {(status: unknown) => void} cb */
  onDaemonStatusChanged: (/** @type {(...args: unknown[]) => void} */ cb) =>
    subscribe('prova:daemon-status-changed', cb),
  // Updater state is carried by three distinct events in main/updater.js:
  //   UPDATE_CHECK_STARTED   -> 'checking'
  //   UPDATE_CHECK_FINISHED  -> 'idle'  (no update available)
  //   READY_TO_UPDATE        -> 'ready'
  // We normalize them into a single callback surface here so the renderer
  // has one subscription to manage.
  /** @param {(status: 'checking' | 'idle' | 'ready') => void} cb */
  onUpdaterStatusChanged: (/** @type {(s: 'checking' | 'idle' | 'ready') => void} */ cb) => {
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
