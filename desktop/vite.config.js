// SPDX-License-Identifier: Apache-2.0 OR MIT
// Forked from CheckerNetwork/desktop.

import { defineConfig } from 'vite'
import react from '@vitejs/plugin-react'
import path from 'node:path'

const rendererDir = path.resolve(__dirname, 'renderer')

export default defineConfig({
  root: rendererDir,
  plugins: [react()],
  // Relative base because index.html is served from file:// via electron-serve.
  base: './',
  build: {
    outDir: path.resolve(rendererDir, 'dist'),
    emptyOutDir: true,
    sourcemap: true,
    target: 'chrome120',
    assetsInlineLimit: 0
  }
})
