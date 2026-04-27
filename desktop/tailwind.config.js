/** @type {import('tailwindcss').Config} */
export default {
  content: [
    './renderer/index.html',
    './renderer/src/**/*.{ts,tsx,js,jsx}'
  ],
  theme: {
    extend: {
      colors: {
        // Prova brand palette. Aligned with the canonical brand-tokens.md.
        // Teal is the primary brand color (mark + accents); ink/cream are
        // the surface colors. Legacy `gold` is kept as a secondary accent
        // for now so existing components don't break, but the dashboard
        // should drift toward teal accents over time.
        teal: {
          cyan: '#5DC3E5',  // gradient start, accent on dark surfaces
          mint: '#2EC4B6',  // gradient mid
          deep: '#0F4C5C',  // brand anchor, gradient end
          shadow: '#08222B' // very-dark teal for tray/dark backgrounds
        },
        gold: {
          DEFAULT: '#C9A84C',
          dark: '#8a7a3a',
          light: '#e5c776'
        },
        ink: {
          DEFAULT: '#1a1817',
          soft: '#3a3632'
        },
        cream: '#FAFAF9',
        base: '#0052FF'
      },
      fontFamily: {
        // macOS Tahoe SF Pro stack. Falls back cleanly on Windows / Linux.
        sans: [
          '-apple-system',
          'BlinkMacSystemFont',
          'SF Pro Text',
          'SF Pro Display',
          'Inter var',
          'Inter',
          'ui-sans-serif',
          'system-ui',
          'sans-serif'
        ],
        display: [
          'SF Pro Display',
          '-apple-system',
          'BlinkMacSystemFont',
          'Inter var',
          'Inter',
          'ui-sans-serif',
          'system-ui',
          'sans-serif'
        ],
        mono: ['SF Mono', 'ui-monospace', 'Menlo', 'Consolas', 'monospace']
      },
      letterSpacing: {
        // Tahoe display tracks slightly tighter than default.
        tight: '-0.011em',
        tighter: '-0.022em'
      },
      boxShadow: {
        // Soft glassy panel shadow for Tahoe-style cards on top of the
        // window vibrancy layer.
        glass: '0 1px 0 0 rgb(255 255 255 / 0.6) inset, 0 1px 2px rgb(0 0 0 / 0.04), 0 8px 24px -12px rgb(0 0 0 / 0.18)'
      }
    }
  },
  plugins: []
}
