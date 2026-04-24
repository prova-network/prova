// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright (c) Protocol Labs (original), Prova Network contributors (modifications).
// Forked from CheckerNetwork/desktop.
//
// End-to-end smoke test: launches Electron against a tmp PROVA_ROOT,
// boots the renderer, and asserts the hero elements show up. Does NOT
// spawn a real provad (that would require the Go binary + Base RPC).
// Instead we launch with PROVA_DISABLE_DAEMON=1 so provad.run() is a
// no-op, and verify the UI handles the "daemon absent" state gracefully.

'use strict'

const path = require('path')
const tmp = require('tmp')
const { expect, test } = require('@playwright/test')
const { _electron: electron } = require('playwright')

const TIMEOUT_MULTIPLIER = process.env.CI ? 10 : 1

test.describe.serial('Application launch', () => {
  if (process.env.CI === 'true') test.setTimeout(120_000)

  /** @type {import('playwright').ElectronApplication} */
  let electronApp
  /** @type {import('playwright').Page} */
  let mainWindow

  test.beforeAll(async () => {
    const rootDir = tmp.dirSync({
      prefix: 'prova-e2e-',
      unsafeCleanup: true
    }).name

    electronApp = await electron.launch({
      args: [path.join(__dirname, '..', '..', 'main', 'index.js')],
      env: {
        ...process.env,
        NODE_ENV: 'test',
        PROVA_ROOT: rootDir,
        PROVA_DISABLE_DAEMON: '1'
      },
      timeout: 30000 * TIMEOUT_MULTIPLIER
    })

    // Surface main-process logs so Playwright failures are debuggable.
    electronApp.process().stdout?.pipe(process.stdout)
    electronApp.process().stderr?.pipe(process.stderr)

    mainWindow = await electronApp.firstWindow()

    mainWindow.on('console', msg => {
      msg.args().length > 0 &&
        Promise.all(msg.args().map(a => a.jsonValue().catch(() => null)))
          .then(vals => console.log(`[renderer:${msg.type()}]`, ...vals))
    })
    mainWindow.on('pageerror', err => { throw err })

    await mainWindow.waitForLoadState('domcontentloaded')
  })

  test.afterAll(async () => {
    if (electronApp) await electronApp.close()
  })

  test('renders the Prova Desktop hero', async () => {
    // The h1 says "Prova Desktop" with "Desktop" highlighted in gold.
    const heading = mainWindow.locator('h1:has-text("Prova")')
    await expect(heading).toBeVisible({ timeout: 10000 * TIMEOUT_MULTIPLIER })
  })

  test('shows the four status tiles', async () => {
    // Stat tiles have the data-test-equivalent of visible labels.
    for (const label of ['Active deals', 'Proofs submitted', 'Uptime', 'Build']) {
      await expect(
        mainWindow.locator(`text=${label}`).first()
      ).toBeVisible({ timeout: 5000 * TIMEOUT_MULTIPLIER })
    }
  })

  test('shows the wallet section with an address', async () => {
    // The Wallet section renders an Address label + the checksummed address
    // in a .mono span. wallet.setup() completes synchronously during boot
    // so by the time the DOM settles, the address should be present.
    await expect(
      mainWindow.locator('text=Address').first()
    ).toBeVisible({ timeout: 10000 * TIMEOUT_MULTIPLIER })

    // Poll for a .mono element whose text matches an Ethereum address.
    // Several .mono spans exist on the page (header pill, address, CommP);
    // we find the one that matches the full 42-char form.
    await expect.poll(async () => {
      const monos = await mainWindow.locator('.mono').all()
      for (const el of monos) {
        const t = (await el.textContent()) || ''
        if (/^0x[a-fA-F0-9]{40}$/.test(t.trim())) return true
      }
      return false
    }, { timeout: 15000 * TIMEOUT_MULTIPLIER, message: 'no 0x-address found' })
      .toBe(true)
  })

  test('shows the empty-activity message when no events have fired', async () => {
    await expect(
      mainWindow.locator('text=No activity yet').first()
    ).toBeVisible({ timeout: 10000 * TIMEOUT_MULTIPLIER })
  })

  test('IPC bridge is exposed and usable', async () => {
    // From the renderer context we should be able to invoke IPC handlers.
    const hasBridge = await mainWindow.evaluate(() => {
      return typeof window.electron === 'object' &&
        typeof window.electron.getWalletAddress === 'function'
    })
    expect(hasBridge).toBe(true)

    const addr = await mainWindow.evaluate(() =>
      window.electron.getWalletAddress()
    )
    expect(addr).toMatch(/^0x[a-fA-F0-9]{40}$/)
  })
})
