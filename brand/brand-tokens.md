# Prova brand tokens

## Logo files

| File | Use |
| ---- | --- |
| `prova-mark.svg` | Default logo (teal gradient on dark) |
| `prova-mark-light.svg` | For light backgrounds (deeper teal stops) |
| `prova-mark-mono.svg` | Single color via `currentColor` for nav/footer/inline |
| `prova-mark-small.svg` | Filled hex for ≤24px, favicon, app icons |
| `prova-wordmark.svg` | Mark + "Prova" wordmark (teal gradient) |
| `prova-wordmark-mono.svg` | Same wordmark, single-color |
| `prova-avatar.svg` | Square 400x400 with rounded corners — github org, x.com avatar, social cards |

## Colors

| Token | Hex | Use |
| ----- | --- | --- |
| `--teal-cyan` | `#5DC3E5` | Gradient start (lightest), accent on dark backgrounds |
| `--teal-mint` | `#2EC4B6` | Gradient mid |
| `--teal-deep` | `#0F4C5C` | Gradient end, brand anchor color |
| `--teal-shadow` | `#08222B` | Avatar/dark backgrounds |

## Gradient definition

```css
background: linear-gradient(135deg, #5DC3E5 0%, #2EC4B6 55%, #0F4C5C 100%);
```

```svg
<linearGradient id="provaTeal" x1="20%" y1="0%" x2="80%" y2="100%">
  <stop offset="0%" stop-color="#5DC3E5"/>
  <stop offset="55%" stop-color="#2EC4B6"/>
  <stop offset="100%" stop-color="#0F4C5C"/>
</linearGradient>
```

## Typography

- **Display + headings**: SF Pro Display, weight 500, letter-spacing -0.022em
- **Body**: SF Pro Text, weight 400, system fallback stack
- **Mono**: SF Mono / ui-monospace
- **Italic accent**: New York / Iowan Old Style (used sparingly for emphasis)
