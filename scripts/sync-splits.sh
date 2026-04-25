#!/usr/bin/env bash
# Sync local monorepo subdirs back to their split repos.
#
# Usage:
#   bash scripts/sync-splits.sh                # sync all 7
#   bash scripts/sync-splits.sh cli sdk        # sync just listed
#
# How it works:
#   For each component, we use git subtree split to extract the subdir's
#   history into a temporary branch, then push it to the matching remote.
#
# After this script runs, each split repo's main branch is updated with
# whatever's currently committed locally for that subdir.

set -euo pipefail

# Map: monorepo subdir → remote name (must match `git remote` config)
declare -A SPLITS=(
  [cli]=cli
  [sdk/typescript]=sdk
  [contracts]=contracts
  [prover]=prover
  [website]=website
  [desktop]=desktop
  [brand]=brand
  [docs-gitbook]=docs
)

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

# Check we're on a clean main
if [[ -n "$(git status --porcelain)" ]]; then
  echo "✗ Working tree dirty. Commit or stash first."
  exit 1
fi

# Filter to requested components, or all
TARGETS=("$@")
if [[ ${#TARGETS[@]} -eq 0 ]]; then
  TARGETS=(cli sdk contracts prover website desktop brand docs)
fi

for component in "${TARGETS[@]}"; do
  # Find the source subdir
  subdir=""
  for k in "${!SPLITS[@]}"; do
    if [[ "${SPLITS[$k]}" == "$component" ]]; then
      subdir="$k"
      break
    fi
  done

  if [[ -z "$subdir" ]]; then
    echo "✗ Unknown component: $component"
    continue
  fi

  if [[ ! -d "$subdir" ]]; then
    echo "✗ Subdir not found: $subdir"
    continue
  fi

  echo ""
  echo "── $subdir → prova-network/$component ──"

  branch="split-$component-$$"
  git subtree split --prefix="$subdir" -b "$branch" 2>&1 | tail -1

  git push "$component" "$branch:main" --force-with-lease 2>&1 | tail -3

  git branch -D "$branch" 2>/dev/null
done

echo ""
echo "✓ Done."
