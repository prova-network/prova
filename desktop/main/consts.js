// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright (c) Protocol Labs (original), Prova Network contributors (modifications).
// Forked from CheckerNetwork/desktop, adapted for Prova.
//
// Path / version constants for the Prova Desktop app.
// PROVA_ROOT (env var) overrides the default per-platform paths, useful for
// running multiple isolated instances or for tests.

'use strict'

const { app } = require('electron')
const os = require('os')
const packageJson = require('../package.json')
const path = require('path')
const assert = require('assert')

const { getBuildVersion } = require('./build-version')

// Reverse-DNS app IDs used for cache/state directory naming.
// darwin uses reverse-DNS, win32 uses the readable product name, linux uses kebab-case.
const appIDs = {
  darwin: 'network.prova.desktop',
  win32: 'Prova Desktop',
  linux: 'prova-desktop'
}

module.exports = Object.freeze({
  CACHE_ROOT: getCacheRoot(),
  STATE_ROOT: getStateRoot(),
  LEGACY_CACHE_HOME: getLegacyCacheHome(),
  IS_MAC: os.platform() === 'darwin',
  IS_WIN: os.platform() === 'win32',
  IS_APPIMAGE: typeof process.env.APPIMAGE !== 'undefined',
  APP_VERSION: packageJson.version,
  BUILD_VERSION: getBuildVersion(packageJson),
  ELECTRON_VERSION: process.versions.electron,

  // Back-compat alias for anywhere in the codebase still reading STATION_VERSION.
  // Remove once all refs are converted.
  STATION_VERSION: packageJson.version
})

function getCacheRoot () {
  if (process.env.PROVA_ROOT) {
    return path.join(process.env.PROVA_ROOT, 'cache')
  }

  const platform = os.platform()
  switch (platform) {
    case 'darwin':
      return path.join(app.getPath('home'), 'Library', 'Caches', appIDs.darwin)
    case 'win32':
      assert(
        process.env.TEMP,
        'Unsupported Windows environment: TEMP must be set.'
      )
      return path.join(process.env.TEMP, appIDs.win32)
    case 'linux':
      return path.join(
        process.env.XDG_CACHE_HOME || path.join(app.getPath('home'), '.cache'),
        appIDs.linux
      )
    default:
      throw new Error(`Unsupported platform: ${platform}`)
  }
}

function getStateRoot () {
  if (process.env.PROVA_ROOT) {
    return path.join(process.env.PROVA_ROOT, 'state')
  }

  const platform = os.platform()
  switch (platform) {
    case 'darwin':
      return path.join(
        app.getPath('home'),
        'Library',
        'Application Support',
        appIDs.darwin
      )
    case 'win32':
      assert(
        process.env.LOCALAPPDATA,
        'Unsupported Windows environment: LOCALAPPDATA must be set.'
      )
      return path.join(process.env.LOCALAPPDATA, appIDs.win32)
    case 'linux':
      return path.join(
        process.env.XDG_STATE_HOME ||
          path.join(app.getPath('home'), '.local', 'state'),
        appIDs.linux
      )
    default:
      throw new Error(`Unsupported platform: ${platform}`)
  }
}

// Used for migrations from very old installs. Prova has no legacy installs
// but we keep the helper around for future upgrade paths.
function getLegacyCacheHome () {
  if (process.env.PROVA_ROOT) {
    return path.join(process.env.PROVA_ROOT, 'cache')
  }

  const platform = os.platform()
  switch (platform) {
    case 'darwin':
      return path.join(app.getPath('home'), 'Library', 'Caches', app.name)
    case 'win32':
      if (!process.env.LOCALAPPDATA) {
        throw new Error(
          'Unsupported Windows environment: LOCALAPPDATA must be set.'
        )
      }
      return path.join(process.env.LOCALAPPDATA, app.name)
    case 'linux':
      return path.join(
        process.env.XDG_CACHE_HOME || path.join(app.getPath('home'), '.cache'),
        app.name
      )
    default:
      throw new Error(`Unsupported platform: ${platform}`)
  }
}
