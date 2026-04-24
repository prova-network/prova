#!/usr/bin/env bash
# Cross-compile provad for all desktop-bundling targets and drop each
# binary into prover/dist/<os>-<arch>/ where electron-builder expects it.
#
# Used by:
#   - desktop/package.json `prepackage` script (local builds before dmg/exe)
#   - .github/workflows/release.yml (appended to its build matrix later)
#
# Targets: linux-amd64, linux-arm64, darwin-amd64, darwin-arm64, windows-amd64.
# No cross-compile toolchain needed because we build CGO_ENABLED=0; the
# prover has no native deps (piece-commp / fr32 / ecdsa are all pure Go).

set -euo pipefail

HERE="$(cd "$(dirname "$0")" && pwd)"
PROVER="$(cd "$HERE/.." && pwd)"
DIST="$PROVER/dist"

VERSION="${PROVA_VERSION:-dev}"
COMMIT="$(git -C "$PROVER" rev-parse --short HEAD 2>/dev/null || echo "unknown")"

# target list: os:arch[:extra flags]
TARGETS=(
  "linux:amd64"
  "linux:arm64"
  "darwin:amd64"
  "darwin:arm64"
  "windows:amd64"
)

echo "Building provad for ${#TARGETS[@]} targets (version=$VERSION, commit=$COMMIT)"

rm -rf "$DIST"
mkdir -p "$DIST"

for target in "${TARGETS[@]}"; do
  IFS=':' read -r GOOS GOARCH <<<"$target"
  outdir="$DIST/${GOOS}-${GOARCH}"
  mkdir -p "$outdir"

  binary_name="provad"
  [[ "$GOOS" == "windows" ]] && binary_name="provad.exe"

  echo "  [$GOOS/$GOARCH] -> $outdir/$binary_name"

  (
    cd "$PROVER"
    env \
      GOOS="$GOOS" \
      GOARCH="$GOARCH" \
      CGO_ENABLED=0 \
      GOTOOLCHAIN=auto \
      go build \
        -trimpath \
        -ldflags "-s -w -X main.version=$VERSION -X main.commit=$COMMIT" \
        -o "$outdir/$binary_name" \
        ./cmd/provad
  )
done

echo ""
echo "All builds complete:"
ls -la "$DIST"/*/provad* 2>/dev/null | awk '{print $NF, "(" $5, "bytes)"}'
