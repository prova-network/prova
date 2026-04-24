// SPDX-License-Identifier: MIT
// Minimal Playwright config for the Electron e2e smoke test.

module.exports = {
  testDir: './test/e2e',
  testMatch: /.*\.e2e\.test\.js$/,
  timeout: 120_000,
  expect: {
    timeout: 15_000
  },
  fullyParallel: false, // Electron apps are singletons
  forbidOnly: !!process.env.CI,
  retries: process.env.CI ? 1 : 0,
  workers: 1,
  reporter: [['list'], ['html', { open: 'never' }]],
  use: {
    trace: 'retain-on-failure',
    video: 'retain-on-failure'
  }
}
