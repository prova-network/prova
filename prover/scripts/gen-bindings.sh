#!/usr/bin/env bash
# SPDX-License-Identifier: MIT
# Copyright (c) 2026 Prova Network contributors.
#
# Generate Go contract bindings from Foundry artifacts.
#
# Each contract gets its own sub-package under pkg/contracts/ to avoid
# struct-name collisions when multiple contracts reference the same type
# (e.g., Cids.Cid is referenced by both ProofVerifier and StorageMarketplace).
#
# Requires:
#   - abigen (go install github.com/ethereum/go-ethereum/cmd/abigen@latest)
#   - forge build already run in contracts/
#   - python3 for artifact parsing
#
# Run from prover/ (or any parent):
#   ./scripts/gen-bindings.sh

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROVER_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"
REPO_ROOT="$(cd "$PROVER_DIR/.." && pwd)"
CONTRACTS_OUT="$REPO_ROOT/contracts/out"
BINDINGS_OUT="$PROVER_DIR/pkg/contracts"

if ! command -v abigen >/dev/null 2>&1; then
  if [[ -x "$HOME/go/bin/abigen" ]]; then
    export PATH="$HOME/go/bin:$PATH"
  else
    echo "error: abigen not found. Install with:" >&2
    echo "  go install github.com/ethereum/go-ethereum/cmd/abigen@latest" >&2
    exit 1
  fi
fi

if [[ ! -d "$CONTRACTS_OUT" ]]; then
  echo "error: $CONTRACTS_OUT not found. Run 'forge build' in contracts/ first." >&2
  exit 1
fi

mkdir -p "$BINDINGS_OUT"

# <Foundry artifact basename>:<sub-package name>:<Go type name>
CONTRACTS=(
  "ProvaToken:provatoken:ProvaToken"
  "ProofVerifier:proofverifier:ProofVerifier"
  "ProverRegistry:proverregistry:ProverRegistry"
  "ProverStaking:proverstaking:ProverStaking"
  "ContentRegistry:contentregistry:ContentRegistry"
  "StorageMarketplace:storagemarketplace:StorageMarketplace"
)

# Clean prior generation (only bindings subdir contents, not README.md etc.)
for entry in "${CONTRACTS[@]}"; do
  IFS=':' read -r _ PKG _ <<< "$entry"
  if [[ -d "$BINDINGS_OUT/$PKG" ]]; then
    rm -rf "$BINDINGS_OUT/$PKG"
  fi
done

for entry in "${CONTRACTS[@]}"; do
  IFS=':' read -r NAME PKG TYPE <<< "$entry"
  ARTIFACT="$CONTRACTS_OUT/$NAME.sol/$NAME.json"
  if [[ ! -f "$ARTIFACT" ]]; then
    echo "warn: $ARTIFACT missing, skipping $NAME" >&2
    continue
  fi

  TMP_ABI="$(mktemp)"
  TMP_BIN="$(mktemp)"

  python3 -c "
import json
with open('$ARTIFACT') as f:
    d = json.load(f)
with open('$TMP_ABI', 'w') as f:
    json.dump(d['abi'], f)
with open('$TMP_BIN', 'w') as f:
    f.write(d['bytecode']['object'])
"

  PKG_DIR="$BINDINGS_OUT/$PKG"
  mkdir -p "$PKG_DIR"
  OUT_FILE="$PKG_DIR/$PKG.go"
  echo "generating $OUT_FILE"

  abigen \
    --abi="$TMP_ABI" \
    --bin="$TMP_BIN" \
    --pkg="$PKG" \
    --type="$TYPE" \
    --out="$OUT_FILE"

  # Tack on an SPDX + generation note at the top
  {
    echo "// SPDX-License-Identifier: MIT"
    echo "// Generated from contracts/out/${NAME}.sol/${NAME}.json via abigen."
    echo "// Do not edit by hand; run ./scripts/gen-bindings.sh instead."
    echo ""
    cat "$OUT_FILE"
  } > "$OUT_FILE.tmp" && mv "$OUT_FILE.tmp" "$OUT_FILE"

  rm -f "$TMP_ABI" "$TMP_BIN"
done

echo ""
echo "✅ Bindings generated"
for entry in "${CONTRACTS[@]}"; do
  IFS=':' read -r NAME PKG _ <<< "$entry"
  [[ -f "$BINDINGS_OUT/$PKG/$PKG.go" ]] && printf "  %-40s %6s bytes\n" "pkg/contracts/$PKG/$PKG.go" "$(wc -c < "$BINDINGS_OUT/$PKG/$PKG.go")"
done
