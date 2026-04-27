// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Prova Network contributors.
//
// Prova Helm brand mark — hexagonal seal with a six-spoke ship's
// wheel inside. Anchors the app to the parent Prova brand (same
// hex) while signaling the "Helm" sub-brand (the wheel).
//
// Two geometry variants:
//   - detailed (>32px): outline rim + 6 spokes terminating in
//     handle bulges. Reads as "ship's helm" at medium/large size.
//   - simplified (<=32px): bolder rim, thicker spokes, no handle
//     bulges, no thin detail. Survives at sidebar / tray scale.
//
// Both use `currentColor` so the parent picks the color via CSS.

export function Logo({
  size = 32,
  ariaLabel = 'Prova Helm logo',
  className = '',
}: {
  size?: number
  ariaLabel?: string
  className?: string
}) {
  // <=28px gets the simplified geometry (no thin handle bulges, bolder
  // strokes); >=29 keeps the detailed wheel that reads as a real helm.
  const isSmall = size <= 28
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
      {/* Outer hexagonal seal (Prova family marker) */}
      <path
        d="M100 16 L173 58 L173 142 L100 184 L27 142 L27 58 Z"
        fill="none"
        stroke="currentColor"
        strokeWidth={isSmall ? 12 : 9}
        strokeLinejoin="round"
        strokeLinecap="round"
      />
      {/* Wheel rim */}
      <circle
        cx="100"
        cy="100"
        r={isSmall ? 38 : 42}
        fill="none"
        stroke="currentColor"
        strokeWidth={isSmall ? 11 : 8}
      />
      {/* Three diameter spokes (six visual spokes through the hub) */}
      <g
        stroke="currentColor"
        strokeWidth={isSmall ? 10 : 6}
        strokeLinecap="round"
      >
        {isSmall ? (
          <>
            {/* shorter spokes that stop at the rim — no handle bulges */}
            <line x1="100"    y1="64"  x2="100"    y2="136" />
            <line x1="131.18" y1="82"  x2="68.82"  y2="118" />
            <line x1="131.18" y1="118" x2="68.82"  y2="82" />
          </>
        ) : (
          <>
            <line x1="100"    y1="48"  x2="100"    y2="152" />
            <line x1="145.03" y1="74"  x2="54.97"  y2="126" />
            <line x1="145.03" y1="126" x2="54.97"  y2="74" />
          </>
        )}
      </g>
      {/* Six handle bulges — only at larger sizes where they're legible */}
      {!isSmall && (
        <g fill="currentColor">
          <circle cx="100"    cy="48"  r={5.5} />
          <circle cx="145.03" cy="74"  r={5.5} />
          <circle cx="145.03" cy="126" r={5.5} />
          <circle cx="100"    cy="152" r={5.5} />
          <circle cx="54.97"  cy="126" r={5.5} />
          <circle cx="54.97"  cy="74"  r={5.5} />
        </g>
      )}
      {/* Central hub */}
      <circle cx="100" cy="100" r={isSmall ? 13 : 9} fill="currentColor" />
    </svg>
  )
}
