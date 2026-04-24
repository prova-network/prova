/** @type {import('tailwindcss').Config} */
export default {
  content: ['./index.html', './src/**/*.{ts,tsx}'],
  theme: {
    extend: {
      colors: {
        // Prova brand
        gold: {
          DEFAULT: '#C9A84C',
          dark: '#8a7a3a',
          light: '#e5c776',
        },
        ink: {
          DEFAULT: '#1a1817',
          soft: '#3a3632',
        },
        cream: '#FAFAF9',
        base: '#0052FF',
      },
      fontFamily: {
        sans: ['ui-sans-serif', 'system-ui', 'sans-serif'],
        mono: ['ui-monospace', 'Menlo', 'monospace'],
      },
    },
  },
  plugins: [],
}
