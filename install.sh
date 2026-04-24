#!/usr/bin/env bash
# SPDX-License-Identifier: MIT
# Copyright (c) 2026 Prova Network contributors.
#
# Prova prover one-liner installer.
#
# Usage:
#   curl -fsSL https://prova.network/install.sh | bash
#
# Options (env vars):
#   PROVA_VERSION=v0.1.0-pre       pin a specific release (default: latest)
#   PROVA_PREFIX=/usr/local         install prefix (default: /usr/local)
#   PROVA_CONFIG=/etc/prova         config prefix (default: /etc/prova)
#   PROVA_NO_SYSTEMD=1              skip systemd unit setup
#   PROVA_DRY_RUN=1                 print what would happen without doing it

set -euo pipefail

# ── Constants ─────────────────────────────────────────────────────────
PROVA_REPO="${PROVA_REPO:-Reiers/prova}"
PROVA_BINARY="provad"
DEFAULT_VERSION="latest"

VERSION="${PROVA_VERSION:-$DEFAULT_VERSION}"
PREFIX="${PROVA_PREFIX:-/usr/local}"
CONFIG_DIR="${PROVA_CONFIG:-/etc/prova}"
DRY_RUN="${PROVA_DRY_RUN:-0}"
SKIP_SYSTEMD="${PROVA_NO_SYSTEMD:-0}"

# ── UI helpers ────────────────────────────────────────────────────────
bold() { printf '\033[1m%s\033[0m\n' "$*"; }
info() { printf '\033[1;34m==>\033[0m %s\n' "$*"; }
ok()   { printf '\033[1;32m  ok\033[0m %s\n' "$*"; }
warn() { printf '\033[1;33m  !\033[0m %s\n' "$*" >&2; }
die()  { printf '\033[1;31merror:\033[0m %s\n' "$*" >&2; exit 1; }

run() {
  if [ "$DRY_RUN" = "1" ]; then
    printf '  (dry) %s\n' "$*"
  else
    "$@"
  fi
}

sudo_run() {
  # If writing to a privileged path, use sudo. Otherwise run directly.
  local target="$1"
  shift
  if [ -w "$target" ] 2>/dev/null; then
    run "$@"
  elif command -v sudo >/dev/null 2>&1; then
    run sudo "$@"
  else
    die "need write access to $target but 'sudo' is not available; set PROVA_PREFIX to a writable path or run as root"
  fi
}

# ── Platform detection ────────────────────────────────────────────────
detect_platform() {
  local os arch
  case "$(uname -s)" in
    Linux)  os="linux" ;;
    Darwin) os="darwin" ;;
    *) die "unsupported OS: $(uname -s). Prova releases build for linux and darwin only." ;;
  esac

  case "$(uname -m)" in
    x86_64|amd64)  arch="amd64" ;;
    arm64|aarch64) arch="arm64" ;;
    *) die "unsupported CPU arch: $(uname -m). Prova releases build for amd64 and arm64." ;;
  esac

  echo "${os}-${arch}"
}

# ── Version resolution ────────────────────────────────────────────────
resolve_version() {
  local ver="$1"
  if [ "$ver" = "latest" ]; then
    info "Resolving latest release for $PROVA_REPO"
    ver=$(curl -fsSL "https://api.github.com/repos/$PROVA_REPO/releases/latest" \
      | grep -E '"tag_name"' \
      | sed -E 's/.*"tag_name": *"([^"]+)".*/\1/' \
      | head -n1) || true
    if [ -z "$ver" ]; then
      die "could not resolve latest release. Pin with PROVA_VERSION=<tag>."
    fi
  fi
  echo "$ver"
}

# ── Checksum helpers ──────────────────────────────────────────────────
sha256_of() {
  local file="$1"
  if command -v sha256sum >/dev/null 2>&1; then
    sha256sum "$file" | awk '{print $1}'
  elif command -v shasum >/dev/null 2>&1; then
    shasum -a 256 "$file" | awk '{print $1}'
  else
    die "no sha256sum or shasum binary available"
  fi
}

# ── Main ──────────────────────────────────────────────────────────────
main() {
  bold "Prova prover installer"
  echo ""

  local platform
  platform=$(detect_platform)
  info "Platform: $platform"

  local ver
  ver=$(resolve_version "$VERSION")
  info "Version:  $ver"

  local asset="${PROVA_BINARY}-${ver}-${platform}.tar.gz"
  local checksum_file="checksums.txt"
  local base_url="https://github.com/${PROVA_REPO}/releases/download/${ver}"

  info "Asset:    $asset"

  # Download archive + checksums to a temp dir
  local tmp
  tmp=$(mktemp -d)
  trap 'rm -rf "$tmp"' EXIT

  info "Downloading $asset"
  if ! curl -fsSL "${base_url}/${asset}" -o "${tmp}/${asset}"; then
    die "failed to download $asset from $base_url"
  fi
  ok "downloaded"

  info "Downloading $checksum_file"
  if ! curl -fsSL "${base_url}/${checksum_file}" -o "${tmp}/${checksum_file}"; then
    warn "no checksum file found; skipping verification. Consider pinning a release that includes checksums."
  else
    local expected actual
    expected=$(grep "  ${asset}\$" "${tmp}/${checksum_file}" | awk '{print $1}' || true)
    if [ -z "$expected" ]; then
      warn "checksum for $asset not in $checksum_file; skipping verification"
    else
      actual=$(sha256_of "${tmp}/${asset}")
      if [ "$expected" != "$actual" ]; then
        die "checksum mismatch:\n  expected $expected\n  got      $actual"
      fi
      ok "checksum verified"
    fi
  fi

  # Unpack
  info "Unpacking"
  run tar -xzf "${tmp}/${asset}" -C "$tmp"
  [ -f "${tmp}/${PROVA_BINARY}" ] || die "extracted archive does not contain ${PROVA_BINARY}"

  # Install binary
  info "Installing to ${PREFIX}/bin/${PROVA_BINARY}"
  sudo_run "${PREFIX}/bin" install -m 0755 "${tmp}/${PROVA_BINARY}" "${PREFIX}/bin/${PROVA_BINARY}"
  ok "binary installed"

  # Create config dir + example
  if [ ! -d "$CONFIG_DIR" ]; then
    info "Creating config dir $CONFIG_DIR"
    sudo_run "$(dirname "$CONFIG_DIR")" mkdir -p "$CONFIG_DIR"
  fi

  if [ ! -f "${CONFIG_DIR}/prover.toml" ]; then
    info "Writing example config to ${CONFIG_DIR}/prover.toml.example"
    local example_url="https://raw.githubusercontent.com/${PROVA_REPO}/main/prover/examples/prover.toml.example"
    if curl -fsSL "$example_url" -o "${tmp}/prover.toml.example"; then
      sudo_run "$CONFIG_DIR" install -m 0644 "${tmp}/prover.toml.example" "${CONFIG_DIR}/prover.toml.example"
      ok "example config at ${CONFIG_DIR}/prover.toml.example"
    else
      warn "could not fetch example config; skipping"
    fi
  else
    ok "config already present at ${CONFIG_DIR}/prover.toml"
  fi

  # systemd (Linux only, non-root supported paths)
  if [ "$SKIP_SYSTEMD" != "1" ] && [ "$(uname -s)" = "Linux" ] && [ -d /etc/systemd/system ]; then
    info "Installing systemd unit"
    local unit_url="https://raw.githubusercontent.com/${PROVA_REPO}/main/prover/deploy/provad.service"
    if curl -fsSL "$unit_url" -o "${tmp}/provad.service"; then
      sudo_run /etc/systemd/system install -m 0644 "${tmp}/provad.service" "/etc/systemd/system/provad.service"
      sudo_run /etc/systemd sh -c 'systemctl daemon-reload'
      ok "systemd unit at /etc/systemd/system/provad.service"
      echo ""
      bold "Next steps (Linux / systemd):"
      echo "  1. Edit ${CONFIG_DIR}/prover.toml (copy from .example)"
      echo "  2. Create the 'prova' user + data dirs:"
      echo "       sudo useradd --system --home /var/lib/prova --shell /usr/sbin/nologin prova"
      echo "       sudo mkdir -p /var/lib/prova/data /var/log/prova"
      echo "       sudo chown -R prova:prova /var/lib/prova /var/log/prova"
      echo "  3. Enable + start:"
      echo "       sudo systemctl enable --now provad"
      echo "  4. Watch logs:"
      echo "       sudo journalctl -u provad -f"
    else
      warn "could not fetch systemd unit; skipping (you can install it manually later)"
    fi
  elif [ "$(uname -s)" = "Darwin" ]; then
    echo ""
    bold "Next steps (macOS):"
    echo "  1. Edit ${CONFIG_DIR}/prover.toml (copy from .example)"
    echo "  2. Run the daemon in a foreground terminal:"
    echo "       ${PROVA_BINARY} --config ${CONFIG_DIR}/prover.toml start"
    echo "  3. (Or use launchd / tmux / screen for backgrounding.)"
  else
    echo ""
    bold "Next steps:"
    echo "  ${PROVA_BINARY} --config ${CONFIG_DIR}/prover.toml start"
  fi

  echo ""
  bold "Verify:"
  echo "  ${PROVA_BINARY} version"
  echo ""
  bold "Installed."
}

main "$@"
