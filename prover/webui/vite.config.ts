import { defineConfig } from 'vite'
import react from '@vitejs/plugin-react'

// Build target: prover/pkg/dashboard/dist (go:embed picks up from there).
// Dev target: vite dev server at :5173 proxies /api to :8081.
export default defineConfig({
  plugins: [react()],
  server: {
    port: 5173,
    proxy: {
      '/api': {
        target: 'http://127.0.0.1:8081',
        changeOrigin: true,
      },
    },
  },
  build: {
    // Written by the "build" script in package.json; see --outDir flag.
    emptyOutDir: true,
    assetsInlineLimit: 0,
  },
})
