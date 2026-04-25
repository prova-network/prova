#!/usr/bin/env bash
# Sync local monorepo subdirs back to their split repos.
#
# Usage:
#   bash scripts/sync-splits.sh                 # sync all components
#   bash scripts/sync-splits.sh cli sdk         # sync just listed
#
# Each component is git-subtree-split into a temp branch and force-pushed
# to its split repo's main. Subtree split preserves per-subdir history.

set -e

# Pairs of "subdir|remote" - keeps macOS bash 3.2 happy
SPLITS=(
  "cli|cli"
  "sdk/typescript|sdk"
  "contracts|contracts"
  "prover|prover"
  "website|website"
  "desktop|desktop"
  "brand|brand"
  "docs-gitbook|docs"
)

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

if [[ -n "$(git status --porcelain)" ]]; then
  echo "✗ Working tree dirty. Commit or stash first."
  exit 1
fi

# Resolve targets
TARGETS=("$@")
if [[ ${#TARGETS[@]} -eq 0 ]]; then
  for pair in "${SPLITS[@]}"; do
    TARGETS+=("${pair#*|}")
  done
fi

for target in "${TARGETS[@]}"; do
  subdir=""
  for pair in "${SPLITS[@]}"; do
    if [[ "${pair#*|}" == "$target" ]]; then
      subdir="${pair%|*}"
      break
    fi
  done

  if [[ -z "$subdir" ]]; then
    echo "✗ Unknown component: $target"
    continue
  fi
  if [[ ! -d "$subdir" ]]; then
    echo "✗ Subdir not found: $subdir"
    continue
  fi

  echo ""
  echo "── $subdir → prova-network/$target ──"

  branch="split-$target-$$"
  git subtree split --prefix="$subdir" -b "$branch" 2>&1 | tail -1
  git push "$target" "$branch:main" --force 2>&1 | tail -3
  git branch -D "$branch" >/dev/null 2>&1
done

echo ""
echo "✓ Done."
