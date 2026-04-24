// The Prova brand mark: gold triangle/circle on dark ink.
// Using `currentColor` for the glyph means parent can set via text-color,
// but the dark backing disc stays constant so the mark reads cleanly on
// any surface (white, cream, or gold itself).
export function Logo({ size = 32 }: { size?: number }) {
  return (
    <svg
      xmlns="http://www.w3.org/2000/svg"
      viewBox="0 0 200 200"
      width={size}
      height={size}
      aria-label="Prova logo"
    >
      {/* Dark ink backing disc gives the gold mark real contrast. */}
      <circle cx="100" cy="100" r="94" fill="#1a1817" />
      <circle
        cx="100"
        cy="100"
        r="78"
        fill="none"
        stroke="currentColor"
        strokeWidth="7"
      />
      <polygon
        points="100,55 145,135 55,135"
        fill="none"
        stroke="currentColor"
        strokeWidth="7"
        strokeLinejoin="round"
      />
      <circle cx="100" cy="112" r="7.5" fill="currentColor" />
    </svg>
  )
}
