// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright (c) Protocol Labs (original checker-node.js), Prova Network contributors (modifications).
// Forked from CheckerNetwork/desktop main/checker-node.js.
//
// Spawns the Prova prover daemon (`provad`) as a child process, pipes its
// stdout/stderr into a local log buffer + activity stream, and restarts
// the process on exit. The key differences vs the upstream Checker code:
//
//   - Child is a native Go binary (`provad`), not a Node script. We use
//     `spawn` instead of `fork`; no IPC channel, just stdout/stderr pipes.
//   - Config is passed as CLI flags + environment (PROVA_KEYSTORE_PASSPHRASE)
//     rather than opaque env vars.
//   - Event format is parsed from JSON lines emitted by provad's
//     structured logger (slog with --log-format=json), not a custom
//     Station event schema.
//   - No Sentry. Log-to-disk + user-facing log view is the only telemetry.

'use strict'

const { app, dialog } = require('electron')
const { join } = require('node:path')
const { spawn } = require('node:child_process')
const wallet = require('./wallet')
const assert = require('node:assert')
const fs = require('node:fs/promises')
const consts = require('./consts')
const { randomUUID } = require('node:crypto')
const { Activities } = require('./activities')
const { Logs } = require('./logs')
const split2 = require('split2')
const { once } = require('node:events')
const { format } = require('node:util')

const log = require('electron-log').scope('provad')

/** @typedef {import('./typings').Context} Context */

// `provad` binary is bundled into the packaged app under resources/provad/.
// In dev mode we look for it next to the desktop/ directory at ../prover/provad.
const provadPath = app.isPackaged
  ? join(process.resourcesPath, 'provad', binName())
  : join(__dirname, '..', '..', 'prover', binName())

function binName () {
  return process.platform === 'win32' ? 'provad.exe' : 'provad'
}

log.info(format('provad binary: %s', provadPath))

const logs = new Logs()
const activities = new Activities()
let totalProofsSubmitted = 0
let totalDealsActive = 0

/**
 * @param {Context} ctx
 */
async function setup (ctx) {
  ctx.saveModuleLogsAs = async () => {
    const opts = {
      defaultPath: `prova-logs-${(new Date()).getTime()}.log`
    }
    ctx.showUI()
    const { filePath } = await dialog.showSaveDialog(opts)
    if (filePath) {
      await fs.writeFile(filePath, logs.get())
    }
  }
}

/**
 * Supervisor loop: keep provad running. If it exits (crash, RPC blip,
 * forced restart), relaunch after a short backoff. Single-instance only;
 * we don't run multiple provad processes per desktop.
 *
 * @param {Context} ctx
 */
async function run (ctx) {
  // Test hook: e2e tests boot the app without a real provad binary.
  // We want the UI to come up (wallet, dashboard, settings) without the
  // supervisor trying and failing to spawn. ENOENT would spam the logs.
  if (process.env.PROVA_DISABLE_DAEMON === '1') {
    log.info('PROVA_DISABLE_DAEMON=1 set; daemon supervisor disabled')
    return new Promise(() => {}) // never resolves; caller expects run() to block
  }

  let backoffMs = 1000
  const backoffMax = 30_000

  while (true) {
    try {
      await start(ctx)
      backoffMs = 1000 // reset on clean exit
    } catch (/** @type {unknown} */ err) {
      log.error(format('provad start failed:', err))
      const msg = err instanceof Error ? err.message : String(err)
      logs.pushLine(`[supervisor] ${msg}`)
    }
    // Small delay before restart so we don't hammer on repeated config errors
    await new Promise(r => setTimeout(r, backoffMs))
    backoffMs = Math.min(backoffMax, backoffMs * 2)
  }
}

/**
 * @param {Context} ctx
 */
async function start (ctx) {
  log.info('Starting provad...')

  // Verify binary exists before attempting spawn. Gives a clearer error
  // than the generic ENOENT from child_process.
  try {
    await fs.access(provadPath, fs.constants.X_OK)
  } catch (err) {
    throw new Error(
      `provad binary not found or not executable at ${provadPath}. ` +
      'In development, build it with `cd prover && go build -o provad ./cmd/provad`.'
    )
  }

  const walletAddress = await wallet.getAddress()
  const passphrase = await wallet.getKeystorePassphrase()

  const childProcess = spawn(
    provadPath,
    [
      '--config', join(consts.STATE_ROOT, 'prover.toml'),
      '--log-format', 'json',
      'start'
    ],
    {
      env: {
        ...process.env,
        PROVA_KEYSTORE_PASSPHRASE: passphrase,
        PROVA_STATE_ROOT: consts.STATE_ROOT,
        PROVA_CACHE_ROOT: consts.CACHE_ROOT,
        PROVA_WALLET_ADDRESS: walletAddress
      },
      stdio: ['ignore', 'pipe', 'pipe']
    }
  )
  log.info(format('provad pid: %d', childProcess.pid))

  assert(childProcess.stdout)
  childProcess.stdout.setEncoding('utf8')
  childProcess.stdout
    .pipe(split2())
    .on('data', line => {
      logs.pushLine(line)
      parseStdoutLine(ctx, line)
    })

  assert(childProcess.stderr)
  childProcess.stderr.setEncoding('utf8')
  childProcess.stderr
    .pipe(split2())
    .on('data', line => logs.pushLine(`[stderr] ${line}`))

  // Ensure we kill the child if the app quits.
  const onBeforeQuit = () => {
    log.info('before-quit: sending SIGTERM to provad')
    childProcess.kill('SIGTERM')
  }
  app.on('before-quit', onBeforeQuit)

  const onceExited = once(childProcess, 'exit')
  const onceClosed = once(childProcess, 'close')

  const [exitCode, exitSignal] = await onceExited
  app.removeListener('before-quit', onBeforeQuit)
  const reason = exitSignal
    ? `via signal ${exitSignal}`
    : `with code ${exitCode}`
  log.info(`provad exited ${reason}`)

  const [closeCode] = await onceClosed
  log.info(`provad closed all stdio with code ${closeCode ?? '<no code>'}`)

  // Exit code semantics for provad (documented in prover/cmd/provad/main.go):
  //   0 = clean shutdown (SIGTERM, graceful drain)
  //   1 = generic error (bad config, RPC unreachable at start)
  //   2 = wallet authentication failed (bad passphrase / missing keystore)
  if (closeCode === 2) {
    throw new Error(
      'provad: wallet authentication failed. Check the passphrase stored in the OS keychain.'
    )
  }
}

/**
 * Parse a single JSON log line from provad and translate it into UI activity
 * or metric updates.
 *
 * provad emits slog JSON like:
 *   {"time":"2026-04-24T18:00:00Z","level":"INFO","msg":"deal accepted","dealID":42,...}
 *
 * We recognise a handful of high-signal messages and route them to the UI;
 * everything else just goes into the log buffer for the "Logs" view.
 *
 * @param {Context} ctx
 * @param {string} line
 */
function parseStdoutLine (ctx, line) {
  let ev
  try {
    ev = JSON.parse(line)
  } catch {
    return // non-JSON lines (boot banner, etc) are fine to ignore here
  }

  const msg = ev.msg || ''

  switch (msg) {
    case 'deal accepted':
    case 'deal active':
      activities.push(ctx, {
        type: 'info',
        source: 'deal-engine',
        message: `Deal #${ev.dealID} active (${formatBytes(ev.size)})`,
        timestamp: new Date(),
        id: randomUUID()
      })
      break

    case 'proof submitted':
      totalProofsSubmitted += 1
      if (typeof ctx.setTotalProofsSubmitted === 'function') {
        ctx.setTotalProofsSubmitted(totalProofsSubmitted)
      }
      activities.push(ctx, {
        type: 'info',
        source: 'pdp',
        message: `Proof submitted for deal #${ev.dealID} (epoch ${ev.epoch})`,
        timestamp: new Date(),
        id: randomUUID()
      })
      break

    case 'proof failed':
      activities.push(ctx, {
        type: 'error',
        source: 'pdp',
        message: `Proof failed for deal #${ev.dealID}: ${ev.error || 'unknown'}`,
        timestamp: new Date(),
        id: randomUUID()
      })
      break

    case 'deals active gauge': {
      const n = Number(ev.count ?? ev.value ?? 0)
      if (Number.isFinite(n)) {
        totalDealsActive = n
        if (typeof ctx.setTotalDealsActive === 'function') {
          ctx.setTotalDealsActive(n)
        }
      }
      break
    }

    // Levels ERROR/WARN go up as activity:error regardless of msg so the
    // user sees anything alarming in the activity feed.
    default:
      if (ev.level === 'ERROR') {
        activities.push(ctx, {
          type: 'error',
          source: 'provad',
          message: msg,
          timestamp: new Date(),
          id: randomUUID()
        })
      }
  }
}

/** @param {number} n */
function formatBytes (n) {
  if (!Number.isFinite(n) || n <= 0) return '0 B'
  const units = ['B', 'KiB', 'MiB', 'GiB', 'TiB', 'PiB']
  let i = 0
  let x = n
  while (x >= 1024 && i < units.length - 1) {
    x /= 1024
    i++
  }
  return `${x.toFixed(x >= 100 ? 0 : 1)} ${units[i]}`
}

module.exports = {
  setup,
  run,
  isOnline: () => activities.isOnline(),
  getActivities: () => activities.get(),
  getTotalProofsSubmitted: () => totalProofsSubmitted,
  getTotalDealsActive: () => totalDealsActive
}
