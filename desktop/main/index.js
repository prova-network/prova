// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright (c) Protocol Labs (original), Prova Network contributors (modifications).
// Forked from CheckerNetwork/desktop.
//
// Prova Desktop main-process entry point. Boots the Electron app, wires
// the context object that every module uses, then kicks off the provad
// supervisor loop which runs for the lifetime of the app.

'use strict'

const { app, dialog, shell, ipcMain } = require('electron')
const electronLog = require('electron-log')
const path = require('node:path')
const os = require('os')
const { format } = require('node:util')

console.log('Log file:', electronLog.transports.file.getFile().path)
const log = electronLog.scope('main')

// Honour PROVA_ROOT for isolated test/dev instances. Must be applied before
// any module reads from electron's userData path (settings, stores, etc).
if (process.env.PROVA_ROOT) {
  app.setPath('userData', path.join(process.env.PROVA_ROOT, 'user-data'))
}

const { ipcMainEvents, setupIpcMain } = require('./ipc')
const { BUILD_VERSION } = require('./consts')
const provad = require('./provad')
const wallet = require('./wallet')
const settings = require('./settings')
const serve = require('electron-serve')
const { setupAppMenu } = require('./app-menu')
const setupTray = require('./tray')
const setupUI = require('./ui')
const setupUpdater = require('./updater')
const { validateExternalURL } = require('./utils')

const inTest = (process.env.NODE_ENV === 'test')
const isDev = !app.isPackaged && !inTest

log.info(format(
  'Prova Desktop build: %s %s-%s%s%s',
  BUILD_VERSION,
  os.platform(),
  os.arch(),
  isDev ? ' [DEV]' : '',
  inTest ? ' [TEST]' : ''
))
log.info(format('Machine: %s version %s', os.type(), os.release()))

if (app.runningUnderARM64Translation) {
  log.warn(
    'Running under ARM64 translation (macOS Rosetta or Windows WOW). ' +
    'A native-arm64 provad would be faster; download a matching Prova Desktop build.'
  )
}

// Exposed to the preload script so the renderer can show the build in the UI.
process.env.PROVA_BUILD_VERSION = BUILD_VERSION

function handleError (/** @type {any} */ err) {
  log.error(err)
  try {
    dialog.showErrorBox('Prova error', err.stack ?? err.message ?? String(err))
  } catch (_) {
    // dialog may not be available during very early startup
  }
}

process.on('uncaughtException', handleError)
process.on('unhandledRejection', handleError)

// Windows notification APIs require this AppUserModelID to be set.
if (process.platform === 'win32') {
  app.setAppUserModelId('network.prova.desktop')
}

// Enforce single instance so we don't end up with two provad processes.
if (!inTest && !app.requestSingleInstanceLock()) {
  app.quit()
}

// If the user re-launches while the app is already running, surface the UI.
app.on('second-instance', () => {
  ctx.showUI()
})

/** @type {import('./typings').Context} */
const ctx = {
  // Activity feed (populated by provad.js via parseStdoutLine).
  getActivities: () => provad.getActivities(),
  recordActivity: activity => {
    ipcMain.emit(ipcMainEvents.ACTIVITY_LOGGED, activity)
  },

  // Prover stats. These are incremented by the provad module as the daemon
  // emits its structured log lines; the UI consumes them via IPC.
  getTotalProofsSubmitted: () => provad.getTotalProofsSubmitted(),
  setTotalProofsSubmitted: (count) => {
    ipcMain.emit(ipcMainEvents.PROOF_STATS_UPDATED, count)
  },
  getTotalDealsActive: () => provad.getTotalDealsActive(),
  setTotalDealsActive: (count) => {
    ipcMain.emit(ipcMainEvents.DEALS_ACTIVE_UPDATED, count)
  },

  // Wallet address is pushed to the UI once setup() resolves.
  setWalletAddress: (addr) => {
    ipcMain.emit(ipcMainEvents.WALLET_ADDRESS_UPDATED, addr)
  },

  // Lifecycle handles populated by the modules that own them.
  manualCheckForUpdates: () => { throw new Error('not-wired') },
  saveModuleLogsAs: () => { throw new Error('not-wired') },
  toggleOpenAtLogin: () => { throw new Error('not-wired') },
  isOpenAtLogin: () => { throw new Error('not-wired') },
  exportSeedPhrase: () => wallet.exportMnemonic(),
  showUI: () => { throw new Error('not-wired') },
  isShowingUI: false,
  loadWebUIFromDist: serve({
    directory: path.resolve(__dirname, '../renderer/dist')
  }),
  restartToUpdate: () => { throw new Error('not-wired') },
  openReleaseNotes: () => { throw new Error('not-wired') },
  getUpdaterStatus: () => { throw new Error('not-wired') },
  openExternalURL: (/** @type {string} */ url) => {
    validateExternalURL(url)
    shell.openExternal(url)
  }
}

// Boot sequence. Order matters: setup() calls before run() because run()
// blocks forever (provad supervisor loop).
async function run () {
  try {
    await app.whenReady()
  } catch (e) {
    handleError(e)
    app.exit(1)
  }

  try {
    setupTray(ctx)
    if (process.platform === 'darwin') {
      await setupAppMenu(ctx)
    }
    await setupUI(ctx)
    await setupUpdater(ctx)
    await setupIpcMain(ctx)

    await wallet.setup(ctx)
    await provad.setup(ctx)
    await settings.setup(ctx)

    // Blocks here until app quit.
    await provad.run(ctx)
  } catch (e) {
    handleError(e)
  }
}

run()
