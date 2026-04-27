// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Prova Network contributors.
//
// Prova brand mark — hexagonal seal with the P glyph. The two
// gradient palettes mirror the canonical brand SVGs:
//
//   prova-mark-light  -> deeper teal for high contrast on white surfaces
//   prova-mark-dark   -> brighter teal for use on dark surfaces
//
// We render both palettes in one SVG and let CSS `prefers-color-scheme`
// pick which one is visible. This keeps the component a single import
// without a runtime theme prop.

const STROKE_HEX = 9
const STROKE_P = 15

export function Logo({ size = 32, ariaLabel = 'Prova logo' }: { size?: number; ariaLabel?: string }) {
  return (
    <svg
      xmlns="http://www.w3.org/2000/svg"
      viewBox="0 0 200 200"
      width={size}
      height={size}
      aria-label={ariaLabel}
      className="prova-mark"
    >
      <defs>
        <linearGradient id="provaMarkLight" x1="20%" y1="0%" x2="80%" y2="100%">
          <stop offset="0%" stopColor="#1F8FA3" />
          <stop offset="55%" stopColor="#0F6E7E" />
          <stop offset="100%" stopColor="#0A3F4D" />
        </linearGradient>
        <linearGradient id="provaMarkDark" x1="20%" y1="0%" x2="80%" y2="100%">
          <stop offset="0%" stopColor="#5DC3E5" />
          <stop offset="55%" stopColor="#2EC4B6" />
          <stop offset="100%" stopColor="#7FE8D8" />
        </linearGradient>
      </defs>

      {/* Hexagonal seal. Stroke is set via CSS so prefers-color-scheme
          can swap the gradient between the light and dark palettes. */}
      <path
        d="M100 16 L173 58 L173 142 L100 184 L27 142 L27 58 Z"
        fill="none"
        strokeWidth={STROKE_HEX}
        strokeLinejoin="round"
        strokeLinecap="round"
      />
      {/* The P */}
      <path
        d="M70 56 L70 144 M70 56 L116 56 Q138 56 138 80 Q138 104 116 104 L70 104"
        fill="none"
        strokeWidth={STROKE_P}
        strokeLinejoin="round"
        strokeLinecap="round"
      />

      <style>{`
        .prova-mark path { stroke: url(#provaMarkLight); }
        @media (prefers-color-scheme: dark) {
          .prova-mark path { stroke: url(#provaMarkDark); }
        }
      `}</style>
    </svg>
  )
}
