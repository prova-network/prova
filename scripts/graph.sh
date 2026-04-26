#!/usr/bin/env bash
# Surgical graphify wrappers for Prova.
#
# Usage:
#   bash scripts/graph.sh contracts    # AST graph of contracts/src/ only
#   bash scripts/graph.sh spec          # AST + semantic graph of spec/
#   bash scripts/graph.sh full          # full repo (long, expensive)
#   bash scripts/graph.sh open          # open the latest graph.html in the browser
#   bash scripts/graph.sh report        # cat the latest GRAPH_REPORT.md
#
# graphify outputs land in graphify-out/ (gitignored except the top-level
# graph.json / graph.html / GRAPH_REPORT.md).

set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

cmd="${1:-help}"

case "$cmd" in
  contracts)
    graphify contracts/src --no-viz
    graphify cluster-only contracts/src
    echo "→ open graphify-out/graph.html"
    ;;
  spec)
    # spec/ is markdown only — AST extracts what it can, semantic adds the
    # cross-section reference graph
    graphify spec
    echo "→ open graphify-out/graph.html"
    ;;
  full)
    graphify .
    ;;
  upstream-pdp)
    # Compare our PDP fork to the upstream — useful before audits
    graphify clone https://github.com/FilOzone/pdp --branch main --out graphify-out/upstream-pdp
    graphify merge-graphs graphify-out/graph.json graphify-out/upstream-pdp/graph.json --out graphify-out/merged.json
    echo "→ Merged graph at graphify-out/merged.json"
    ;;
  open)
    target="graphify-out/graph.html"
    [[ -f "$target" ]] || { echo "No graphify-out/graph.html — run 'bash scripts/graph.sh contracts' or 'spec' first."; exit 1; }
    open "$target"
    ;;
  report)
    target="graphify-out/GRAPH_REPORT.md"
    [[ -f "$target" ]] || { echo "No graphify-out/GRAPH_REPORT.md — run a graph command first."; exit 1; }
    cat "$target"
    ;;
  help|*)
    sed -n '1,16p' "$0"
    ;;
esac
