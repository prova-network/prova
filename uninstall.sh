#!/usr/bin/env bash
# SPDX-License-Identifier: MIT
# Copyright (c) 2026 Prova Network contributors.
#
# Prova prover one-liner uninstaller.
#
# Usage:
#   curl -fsSL https://prova.network/uninstall.sh | bash
#
# Does NOT touch:
#   - /var/lib/prova (your stored pieces — manual removal)
#   - /etc/prova/prover.toml (your config — manual removal)
#   - the 'prova' system user

set -euo pipefail

PROVA_PREFIX="${PROVA_PREFIX:-/usr/local}"
PROVA_BINARY="provad"

info() { printf '\033[1;34m==>\033[0m %s\n' "$*"; }
ok()   { printf '\033[1;32m  ok\033[0m %s\n' "$*"; }

sudo_run() {
  if [ "$(id -u)" = 0 ] || [ -w "$1" ] 2>/dev/null; then
    shift
    "$@"
  else
    shift
    sudo "$@"
  fi
}

if [ -f "/etc/systemd/system/provad.service" ]; then
  info "Stopping + disabling systemd unit"
  sudo_run /etc/systemd sh -c 'systemctl stop provad 2>/dev/null || true'
  sudo_run /etc/systemd sh -c 'systemctl disable provad 2>/dev/null || true'
  sudo_run /etc/systemd/system rm -f /etc/systemd/system/provad.service
  sudo_run /etc/systemd sh -c 'systemctl daemon-reload'
  ok "systemd unit removed"
fi

if [ -x "${PROVA_PREFIX}/bin/${PROVA_BINARY}" ]; then
  info "Removing ${PROVA_PREFIX}/bin/${PROVA_BINARY}"
  sudo_run "${PROVA_PREFIX}/bin" rm -f "${PROVA_PREFIX}/bin/${PROVA_BINARY}"
  ok "binary removed"
fi

echo ""
echo "Prova prover uninstalled. Preserved:"
echo "  /var/lib/prova            (stored pieces — remove manually if desired)"
echo "  /etc/prova                (config — remove manually if desired)"
echo "  'prova' system user       (remove with: sudo userdel prova)"
