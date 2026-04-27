// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Prova Network contributors.
//
// Prova brand mark — hexagonal seal with the P glyph. Renders the
// canonical strokes and uses `currentColor` so the parent picks the
// final color via CSS (text-teal-deep / text-teal-cyan, etc.). This
// keeps the SVG a single import that drops cleanly into header /
// sidebar / banner contexts without per-call gradient tuning.

export function Logo({
  size = 32,
  ariaLabel = 'Prova logo',
  className = '',
}: {
  size?: number
  ariaLabel?: string
  className?: string
}) {
  return (
    <svg
      xmlns="http://www.w3.org/2000/svg"
      viewBox="0 0 200 200"
      width={size}
      height={size}
      aria-label={ariaLabel}
      className={className}
      role="img"
    >
      {/* Hexagonal seal */}
      <path
        d="M100 16 L173 58 L173 142 L100 184 L27 142 L27 58 Z"
        fill="none"
        stroke="currentColor"
        strokeWidth={9}
        strokeLinejoin="round"
        strokeLinecap="round"
      />
      {/* The P */}
      <path
        d="M70 56 L70 144 M70 56 L116 56 Q138 56 138 80 Q138 104 116 104 L70 104"
        fill="none"
        stroke="currentColor"
        strokeWidth={15}
        strokeLinejoin="round"
        strokeLinecap="round"
      />
    </svg>
  )
}
