#!/usr/bin/env bash
# Generate app icons for macOS (.icns), Windows (.ico), and Linux (.png)
# from the Prova SVG sources.
#
# Requires: rsvg-convert (brew install librsvg), iconutil (macOS built-in),
# python3 (for hand-packed .ico; no Pillow needed).
#
# Two source SVGs:
#   prova-app-icon.svg         - thin-stroke master, used for >=64px
#   prova-app-icon-small.svg   - filled/thicker variant, used for <=48px
#
# Output: desktop/build/icon.{icns,ico,png}

set -euo pipefail

BRAND_DIR="$(cd "$(dirname "$0")" && pwd)"
BUILD_DIR="$(cd "$BRAND_DIR/../desktop/build" && pwd)"

# Prova Helm app icon. The 'big' SVG is used >=64px (Dock, Finder),
# the 'small' SVG is the simplified mark that survives <=48px
# (menu-bar, sidebar). The Prova-only family icons were retired
# when the desktop app rebranded to Prova Helm.
BIG_SVG="$BRAND_DIR/prova-helm-app-icon.svg"
SMALL_SVG="$BRAND_DIR/prova-helm-mark-small.svg"

[[ -f "$BIG_SVG" ]] || { echo "missing $BIG_SVG" >&2; exit 1; }
[[ -f "$SMALL_SVG" ]] || { echo "missing $SMALL_SVG" >&2; exit 1; }

svg_for_size() {
  if (( $1 <= 48 )); then echo "$SMALL_SVG"; else echo "$BIG_SVG"; fi
}

WORK="$(mktemp -d)"
trap 'rm -rf "$WORK"' EXIT

echo "Rendering PNGs into $WORK"
for size in 16 24 32 48 64 128 256 512 1024; do
  src="$(svg_for_size "$size")"
  rsvg-convert "$src" -w "$size" -h "$size" -o "$WORK/icon-${size}.png"
done

# ─── Linux / generic PNG ──────────────────────────────────────────────
cp "$WORK/icon-512.png" "$BUILD_DIR/icon.png"
echo "  build/icon.png <- 512x512"

# ─── macOS .icns ──────────────────────────────────────────────────────
ICONSET="$WORK/prova.iconset"
mkdir -p "$ICONSET"
cp "$WORK/icon-16.png"    "$ICONSET/icon_16x16.png"
cp "$WORK/icon-32.png"    "$ICONSET/icon_16x16@2x.png"
cp "$WORK/icon-32.png"    "$ICONSET/icon_32x32.png"
cp "$WORK/icon-64.png"    "$ICONSET/icon_32x32@2x.png"
cp "$WORK/icon-128.png"   "$ICONSET/icon_128x128.png"
cp "$WORK/icon-256.png"   "$ICONSET/icon_128x128@2x.png"
cp "$WORK/icon-256.png"   "$ICONSET/icon_256x256.png"
cp "$WORK/icon-512.png"   "$ICONSET/icon_256x256@2x.png"
cp "$WORK/icon-512.png"   "$ICONSET/icon_512x512.png"
cp "$WORK/icon-1024.png"  "$ICONSET/icon_512x512@2x.png"

iconutil -c icns "$ICONSET" -o "$BUILD_DIR/icon.icns"
echo "  build/icon.icns <- 10-resolution iconset"

# ─── Windows .ico (hand-packed PNG-in-ICO, Vista+ format) ─────────────
# Pillow's ICO writer doesn't support per-size artwork; it only resizes one
# master. We need different images for different sizes (since <=48px uses
# the simplified SVG variant), so we pack the container directly. The
# Vista+ PNG-in-ICO format is a plain wrapper: ICONDIR header + N
# ICONDIRENTRY records + concatenated PNG payloads.
WORK_DIR="$WORK" BUILD_DIR_VAR="$BUILD_DIR" python3 - <<'PY'
import os, struct

WORK = os.environ["WORK_DIR"]
BUILD = os.environ["BUILD_DIR_VAR"]

# Larger sizes first is conventional but not required.
sizes = [256, 128, 64, 48, 32, 24, 16]
png_blobs = []
for s in sizes:
    with open(os.path.join(WORK, f"icon-{s}.png"), "rb") as f:
        png_blobs.append((s, f.read()))

header = struct.pack("<HHH", 0, 1, len(sizes))          # reserved, type=1 (ICO), count
entries = bytearray()
payload = bytearray()
offset = 6 + 16 * len(sizes)                            # start of first image blob

for s, data in png_blobs:
    w = 0 if s == 256 else s                            # 0 means 256 in ICO spec
    h = 0 if s == 256 else s
    entries += struct.pack(
        "<BBBBHHII",
        w, h,
        0,        # palette colors (0 = no palette)
        0,        # reserved
        1,        # color planes
        32,       # bits per pixel
        len(data),
        offset,
    )
    payload += data
    offset += len(data)

with open(os.path.join(BUILD, "icon.ico"), "wb") as out:
    out.write(header)
    out.write(entries)
    out.write(payload)

print(f"  build/icon.ico <- {len(sizes)}-resolution multi-size ICO "
      f"({sum(len(d) for _, d in png_blobs)} bytes of PNG data)")
PY

# ─── Summary ──────────────────────────────────────────────────────────
echo ""
echo "Generated:"
ls -la "$BUILD_DIR/icon.icns" "$BUILD_DIR/icon.ico" "$BUILD_DIR/icon.png"

# ─── Tray icons ───────────────────────────────────────────────────────
# Tray icons are monochrome (template images on macOS, plain black
# elsewhere). Four states: on, off, update (on), update-off.
TRAY_DIR="$(cd "$BRAND_DIR/../desktop/assets/tray" && pwd)"

# macOS convention: 22x22 @1x, 44x44 @2x, 66x66 @3x, named state-macos.png
# Windows/Linux: 32x32 named state.png
for pair in \
  "on:prova-tray.svg" \
  "off:prova-tray-off.svg" \
  "update:prova-tray-update.svg" \
  "update-off:prova-tray-update-off.svg"
do
  state="${pair%%:*}"
  src="$BRAND_DIR/${pair##*:}"
  rsvg-convert "$src" -w 22 -h 22 -o "$TRAY_DIR/${state}-macos.png"
  rsvg-convert "$src" -w 44 -h 44 -o "$TRAY_DIR/${state}-macos@2x.png"
  rsvg-convert "$src" -w 66 -h 66 -o "$TRAY_DIR/${state}-macos@3x.png"
  rsvg-convert "$src" -w 32 -h 32 -o "$TRAY_DIR/${state}.png"
done
echo "  assets/tray/*.png <- 4 states x 4 resolutions = 16 tray icons"

echo ""
echo "Tray icons:"
ls -1 "$TRAY_DIR" | head -20
