// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright (c) Protocol Labs (original), Prova Network contributors (modifications).
// Forked from CheckerNetwork/desktop.
//
// System tray (macOS menu bar, Windows taskbar, Linux status area).
// Icon state cycles between:
//   off           - daemon not running / offline
//   on            - daemon running, at least one active deal
//   update-off    - update available, daemon offline
//   update-on     - update available, daemon online
// macOS icons are template images (black on transparent) so the system
// applies light/dark theme colour; Windows/Linux icons are coloured.

'use strict'

const { IS_MAC, APP_VERSION } = require('./consts')
const { Menu, Tray, app, ipcMain, nativeImage } = require('electron')
const { ipcMainEvents } = require('./ipc')
const path = require('path')
const assert = require('node:assert')
const provad = require('./provad')

/** @typedef {import('./typings').Context} Context */

/** @type {Tray | null} */
let tray = null

const icons = {
  on: icon('on'),
  off: icon('off'),
  updateOn: icon('update'),
  updateOff: icon('update-off')
}

/**
 * @param {'on' | 'off' | 'update' | 'update-off'} state
 */
function icon (state) {
  const dir = path.resolve(path.join(__dirname, '../assets/tray'))
  const file = IS_MAC ? `${state}-macos.png` : `${state}.png`
  const image = nativeImage.createFromPath(path.join(dir, file))
  // On macOS the system will recolour based on the menu bar theme.
  image.setTemplateImage(true)
  return image
}

/**
 * @param {boolean} readyToUpdate
 * @param {boolean} isOnline
 */
function getTrayIcon (readyToUpdate, isOnline) {
  return readyToUpdate
    ? isOnline ? icons.updateOn : icons.updateOff
    : isOnline ? icons.on : icons.off
}

/**
 * @param {Context} ctx
 */
function createContextMenu (ctx) {
  return Menu.buildFromTemplate([
    { label: `Prova Desktop v${APP_VERSION}`, enabled: false },
    { label: 'Open Prova', click: () => ctx.showUI() },
    { type: 'separator' },
    {
      label: `Active deals: ${provad.getTotalDealsActive()}`,
      enabled: false
    },
    {
      label: `Proofs submitted: ${provad.getTotalProofsSubmitted().toLocaleString()}`,
      enabled: false
    },
    { type: 'separator' },
    { label: 'Check for updates', click: () => ctx.manualCheckForUpdates() },
    {
      label: 'Quit Prova',
      click: () => app.quit(),
      accelerator: IS_MAC ? 'Command+Q' : undefined
    }
  ])
}

module.exports = async function setupTray (/** @type {Context} */ ctx) {
  tray = new Tray(getTrayIcon(false, provad.isOnline()))
  tray.setToolTip('Prova Desktop')
  tray.setContextMenu(createContextMenu(ctx))

  // Re-render on state changes.
  const updateTray = () => {
    assert(tray)
    const updStatus = (() => {
      try { return ctx.getUpdaterStatus() } catch { return null }
    })()
    const readyToUpdate = updStatus && typeof updStatus === 'object'
      ? Boolean((updStatus).readyToUpdate)
      : updStatus === 'ready'
    tray.setImage(getTrayIcon(readyToUpdate, provad.isOnline()))
    tray.setContextMenu(createContextMenu(ctx))
  }

  ipcMain.on(ipcMainEvents.ACTIVITY_LOGGED, updateTray)
  ipcMain.on(ipcMainEvents.PROOF_STATS_UPDATED, updateTray)
  ipcMain.on(ipcMainEvents.DEALS_ACTIVE_UPDATED, updateTray)
  ipcMain.on(ipcMainEvents.READY_TO_UPDATE, updateTray)
}
