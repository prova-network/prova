#!/usr/bin/env bash
# ─── Prova prover installer ───
#   curl -fsSL https://get.prova.network | bash
#
#   Installs the Prova prover daemon (provad) locally: fetches the right
#   platform binary, verifies its SHA-256 checksum, drops an example
#   config, and (on Linux) offers a hardened systemd unit.
#
#   Safe by default: asks before doing anything destructive. Idempotent.
#   Re-running upgrades in place.
#
# ─── Supported systems ───
#   • Linux:    Ubuntu 22.04+, Debian 12+, Fedora 39+, anything w/ systemd 249+
#   • macOS:    13 (Ventura) or newer
#   • Arches:   amd64, arm64
#   • Windows:  not supported; use WSL2 at your own risk
#
# ─── Dependencies (on the host) ───
#   Already on every modern Linux/macOS by default:
#     bash 4+, curl, tar, install, mktemp, awk, grep, sed, uname
#     sha256sum (Linux) OR shasum -a 256 (macOS)
#     sudo (only if installing into system paths like /usr/local/bin)
#     systemctl (Linux, only if opting into the systemd unit)
#
#   The Prova binary itself is statically compiled, no runtime deps:
#     no Go, no libssl, no glibc-version issues.

set -euo pipefail

# ─── Dep check (fail fast with a useful message) ──────────────────────────
check_deps() {
  local missing=()
  for cmd in bash curl tar install mktemp awk grep sed uname; do
    command -v "$cmd" >/dev/null 2>&1 || missing+=("$cmd")
  done
  if ! command -v sha256sum >/dev/null 2>&1 && ! command -v shasum >/dev/null 2>&1; then
    missing+=("sha256sum or shasum")
  fi
  if (( ${#missing[@]} > 0 )); then
    printf 'prova install: missing required tools: %s\n' "${missing[*]}" >&2
    printf 'prova install: please install them via your package manager and re-run.\n' >&2
    exit 1
  fi
}
check_deps

# ─── colors + banners ────────────────────────────────────────────────────
if [[ -t 1 ]] && [[ "${TERM:-}" != "dumb" ]] && [[ -z "${NO_COLOR:-}" ]]; then
  BOLD=$'\033[1m'; DIM=$'\033[2m'; RESET=$'\033[0m'
  GOLD=$'\033[38;5;178m'; CREAM=$'\033[38;5;223m'
  BRASS=$'\033[38;5;136m'; INK=$'\033[38;5;240m'
  GREEN=$'\033[38;5;71m'; RED=$'\033[38;5;203m'
  AMBER=$'\033[38;5;214m'
else
  BOLD=''; DIM=''; RESET=''
  GOLD=''; CREAM=''; BRASS=''; INK=''
  GREEN=''; RED=''; AMBER=''
fi

print_banner() {
  cat <<EOF

${BRASS}        ╭──────────╮${RESET}          ${BOLD}${CREAM}Prova${RESET}
${BRASS}       │  ${GOLD}◢${BRASS}       │${RESET}          ${DIM}Verifiable storage on Base.${RESET}
${BRASS}       │  ${GOLD}◢◢${BRASS}      │${RESET}
${BRASS}       │  ${GOLD}◢◢◢${BRASS}     │${RESET}          ${INK}one-line install${RESET}
${BRASS}       │  ${GOLD}◢◢${BRASS}      │${RESET}
${BRASS}       │  ${GOLD}◢${BRASS}       │${RESET}
${BRASS}        ╰──────────╯${RESET}

EOF
}

# Pick a quirky storage quote for the install finale.
prova_quote() {
  local quotes=(
    "Data that nobody can prove is stored, isn't."
    "The proof is the product. The bytes are the evidence."
    "Hash first, trust later."
    "A piece without a proof is a prayer."
    "Durability isn't a promise, it's a receipt."
    "Every challenge is a chance to be honest."
    "Content-addressing: the only honest way to name a thing."
    "Provers are paid to remember. Challengers are paid to care."
    "PDP: the receipt you can audit yourself."
    "Storage is easy. Proving it is the hard part."
    "Merkle never forgets. Don't test it."
    "Your bytes, committed to Base."
    "Cheap challenges, honest provers."
    "Upload once. Verify forever."
    "Cryptographic proofs, warm like bread."
    "A bit out of place is a bit found."
  )
  local n=${#quotes[@]}
  local i=$(( $(od -An -N2 -tu2 /dev/urandom 2>/dev/null | tr -d ' ' || echo 0) % n ))
  printf '%s' "${quotes[$i]}"
}
PROVA_QUOTE="$(prova_quote)"
export PROVA_QUOTE

step() { printf "${BOLD}${GOLD}▸${RESET} %s\n" "$*"; }
ok()   { printf "  ${GREEN}✓${RESET} ${DIM}%s${RESET}\n" "$*"; }
warn() { printf "  ${AMBER}!${RESET} %s\n" "$*" >&2; }
fail() { printf "  ${RED}✗${RESET} ${BOLD}%s${RESET}\n" "$*" >&2; exit 1; }
info() { printf "  ${DIM}%s${RESET}\n" "$*"; }
ask()  {
  local prompt="$1" default="${2:-y}" answer
  if [[ "${PROVA_YES:-}" == "1" ]]; then return 0; fi
  if [[ ! -r /dev/tty ]]; then
    [[ "$default" == "y" ]] && return 0 || return 1
  fi
  if [[ "$default" == "y" ]]; then
    printf "  ${BRASS}?${RESET} %s ${DIM}[Y/n]${RESET} " "$prompt"
  else
    printf "  ${BRASS}?${RESET} %s ${DIM}[y/N]${RESET} " "$prompt"
  fi
  read -r answer </dev/tty 2>/dev/null || answer=""
  answer="${answer:-$default}"
  [[ "$answer" =~ ^[Yy]$ ]]
}

# ─── Config ──────────────────────────────────────────────────────────────
PROVA_REPO="${PROVA_REPO:-Reiers/prova}"
PROVA_BINARY="provad"
VERSION="${PROVA_VERSION:-latest}"
PREFIX="${PROVA_PREFIX:-/usr/local}"
CONFIG_DIR="${PROVA_CONFIG:-/etc/prova}"
DRY_RUN="${PROVA_DRY_RUN:-0}"
SKIP_SYSTEMD="${PROVA_NO_SYSTEMD:-0}"

run() {
  if [[ "$DRY_RUN" == "1" ]]; then
    printf "  ${DIM}(dry) %s${RESET}\n" "$*"
  else
    "$@"
  fi
}

sudo_run() {
  local target="$1"
  shift
  if [[ -w "$target" ]] 2>/dev/null || [[ "$(id -u)" == "0" ]]; then
    run "$@"
  elif command -v sudo >/dev/null 2>&1; then
    run sudo "$@"
  else
    fail "need write access to $target but 'sudo' is not available; set PROVA_PREFIX to a writable path or run as root"
  fi
}

# ─── Platform detection ──────────────────────────────────────────────────
detect_platform() {
  local os arch
  case "$(uname -s)" in
    Linux)  os="linux" ;;
    Darwin) os="darwin" ;;
    *) fail "unsupported OS: $(uname -s). Prova releases build for linux and darwin only." ;;
  esac
  case "$(uname -m)" in
    x86_64|amd64)  arch="amd64" ;;
    arm64|aarch64) arch="arm64" ;;
    *) fail "unsupported CPU arch: $(uname -m). Prova releases build for amd64 and arm64." ;;
  esac
  printf '%s-%s' "$os" "$arch"
}

# ─── Version resolution ──────────────────────────────────────────────────
resolve_version() {
  local ver="$1"
  if [[ "$ver" == "latest" ]]; then
    info "resolving latest release for $PROVA_REPO"
    ver=$(curl -fsSL "https://api.github.com/repos/$PROVA_REPO/releases/latest" 2>/dev/null \
      | grep -E '"tag_name"' \
      | sed -E 's/.*"tag_name": *"([^"]+)".*/\1/' \
      | head -n1) || true
    if [[ -z "$ver" ]]; then
      fail "could not resolve latest release. Pin with PROVA_VERSION=<tag>."
    fi
  fi
  printf '%s' "$ver"
}

sha256_of() {
  local file="$1"
  if command -v sha256sum >/dev/null 2>&1; then
    sha256sum "$file" | awk '{print $1}'
  elif command -v shasum >/dev/null 2>&1; then
    shasum -a 256 "$file" | awk '{print $1}'
  else
    fail "no sha256sum or shasum binary available"
  fi
}

# ─── Main ────────────────────────────────────────────────────────────────
main() {
  print_banner

  local platform ver
  platform=$(detect_platform)
  ver=$(resolve_version "$VERSION")

  step "Platform: ${BOLD}${CREAM}$platform${RESET}"
  step "Version:  ${BOLD}${CREAM}$ver${RESET}"
  step "Prefix:   ${BOLD}${CREAM}$PREFIX${RESET}"
  echo ""

  local asset="${PROVA_BINARY}-${ver}-${platform}.tar.gz"
  local checksum_file="checksums.txt"
  local base_url="https://github.com/${PROVA_REPO}/releases/download/${ver}"

  local tmp
  tmp=$(mktemp -d)
  trap 'rm -rf "$tmp"' EXIT

  step "Downloading $asset"
  if ! curl -fsSL "${base_url}/${asset}" -o "${tmp}/${asset}" 2>/dev/null; then
    fail "failed to download $asset from $base_url (does the tag exist and is the release published?)"
  fi
  ok "downloaded $(du -h "${tmp}/${asset}" | awk '{print $1}')"

  step "Verifying checksum"
  if curl -fsSL "${base_url}/${checksum_file}" -o "${tmp}/${checksum_file}" 2>/dev/null; then
    local expected actual
    expected=$(grep "  ${asset}\$" "${tmp}/${checksum_file}" | awk '{print $1}' || true)
    if [[ -z "$expected" ]]; then
      warn "checksum for $asset not in $checksum_file; skipping verification"
    else
      actual=$(sha256_of "${tmp}/${asset}")
      if [[ "$expected" != "$actual" ]]; then
        fail "checksum mismatch: expected $expected, got $actual"
      fi
      ok "sha256 matches published checksum"
    fi
  else
    warn "no checksum file found; skipping verification"
  fi

  step "Unpacking"
  run tar -xzf "${tmp}/${asset}" -C "$tmp"
  [[ -f "${tmp}/${PROVA_BINARY}" ]] || fail "extracted archive does not contain ${PROVA_BINARY}"
  ok "binary extracted"

  step "Installing to ${PREFIX}/bin/${PROVA_BINARY}"
  sudo_run "${PREFIX}/bin" install -m 0755 "${tmp}/${PROVA_BINARY}" "${PREFIX}/bin/${PROVA_BINARY}"
  ok "binary installed"

  if [[ ! -d "$CONFIG_DIR" ]]; then
    step "Creating config dir $CONFIG_DIR"
    sudo_run "$(dirname "$CONFIG_DIR")" mkdir -p "$CONFIG_DIR"
    ok "created $CONFIG_DIR"
  fi

  if [[ ! -f "${CONFIG_DIR}/prover.toml" ]]; then
    step "Writing example config to ${CONFIG_DIR}/prover.toml.example"
    local example_url="https://raw.githubusercontent.com/${PROVA_REPO}/main/prover/examples/prover.toml.example"
    if curl -fsSL "$example_url" -o "${tmp}/prover.toml.example" 2>/dev/null; then
      sudo_run "$CONFIG_DIR" install -m 0644 "${tmp}/prover.toml.example" "${CONFIG_DIR}/prover.toml.example"
      ok "example config at ${CONFIG_DIR}/prover.toml.example"
      info "copy to prover.toml and edit before starting"
    else
      warn "could not fetch example config; see prover/examples/prover.toml.example on GitHub"
    fi
  else
    ok "config already present at ${CONFIG_DIR}/prover.toml"
  fi

  if [[ "$SKIP_SYSTEMD" != "1" ]] && [[ "$(uname -s)" == "Linux" ]] && [[ -d /etc/systemd/system ]]; then
    if ask "Install systemd unit at /etc/systemd/system/provad.service?"; then
      step "Installing systemd unit"
      local unit_url="https://raw.githubusercontent.com/${PROVA_REPO}/main/prover/deploy/provad.service"
      if curl -fsSL "$unit_url" -o "${tmp}/provad.service" 2>/dev/null; then
        sudo_run /etc/systemd/system install -m 0644 "${tmp}/provad.service" "/etc/systemd/system/provad.service"
        sudo_run /etc/systemd sh -c 'systemctl daemon-reload'
        ok "systemd unit installed"
      else
        warn "could not fetch systemd unit; skipping"
      fi
    else
      info "skipping systemd setup"
    fi
  fi

  # Verify
  step "Verifying install"
  if "${PREFIX}/bin/${PROVA_BINARY}" version >/dev/null 2>&1; then
    local actual_ver
    actual_ver=$("${PREFIX}/bin/${PROVA_BINARY}" version 2>/dev/null | head -1)
    ok "$actual_ver"
  else
    warn "'$PROVA_BINARY version' did not succeed; you may need to restart your shell or add ${PREFIX}/bin to PATH"
  fi

  echo ""
  printf "${BOLD}${GOLD}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${RESET}\n"
  printf "${BOLD}${CREAM}  Installed.${RESET}\n"
  echo ""
  if [[ "$(uname -s)" == "Linux" ]] && [[ -f /etc/systemd/system/provad.service ]]; then
    printf "  ${BOLD}Next:${RESET}\n"
    printf "    ${INK}1.${RESET} edit ${CONFIG_DIR}/prover.toml (copy from .example)\n"
    printf "    ${INK}2.${RESET} create the prova user + data dirs:\n"
    printf "       ${DIM}sudo useradd --system --home /var/lib/prova --shell /usr/sbin/nologin prova${RESET}\n"
    printf "       ${DIM}sudo mkdir -p /var/lib/prova/data /var/log/prova${RESET}\n"
    printf "       ${DIM}sudo chown -R prova:prova /var/lib/prova /var/log/prova${RESET}\n"
    printf "    ${INK}3.${RESET} start it:\n"
    printf "       ${DIM}sudo systemctl enable --now provad${RESET}\n"
    printf "    ${INK}4.${RESET} watch logs:\n"
    printf "       ${DIM}sudo journalctl -u provad -f${RESET}\n"
  else
    printf "  ${BOLD}Next:${RESET}\n"
    printf "    ${INK}1.${RESET} edit ${CONFIG_DIR}/prover.toml (copy from .example)\n"
    printf "    ${INK}2.${RESET} run the daemon:\n"
    printf "       ${DIM}${PROVA_BINARY} --config ${CONFIG_DIR}/prover.toml start${RESET}\n"
  fi
  echo ""
  printf "${DIM}  ${PROVA_QUOTE}${RESET}\n"
  printf "${BOLD}${GOLD}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${RESET}\n"
  echo ""
}

main "$@"
