// Prova Pages middleware. Routes by hostname.
//
// - get.prova.network               -> install.sh (one-liner CLI installer)
// - everywhere else                  -> falls through to static + functions

interface Env {}

export const onRequest: PagesFunction<Env> = async (ctx) => {
  const url = new URL(ctx.request.url);
  const host = url.hostname;

  if (host === 'get.prova.network') {
    return serveInstaller(ctx.request);
  }

  return ctx.next();
};

function serveInstaller(req: Request): Response {
  const ua = (req.headers.get('user-agent') || '').toLowerCase();
  const accept = req.headers.get('accept') || '';
  const url = new URL(req.url);

  const isCli =
    ua.includes('curl/') ||
    ua.includes('wget/') ||
    ua.includes('fetch/') ||
    ua.includes('libcurl') ||
    !accept.includes('text/html');

  if (req.method === 'HEAD') {
    return new Response(null, {
      status: 200,
      headers: {
        'content-type': 'text/x-shellscript; charset=utf-8',
        'cache-control': 'public, max-age=300',
      },
    });
  }

  if (!isCli) {
    // Browser hit. Until docs are public, send them to the CLI page on the
    // marketing site rather than docs.prova.network.
    return Response.redirect('https://prova.network/#use', 302);
  }

  const wantPrerelease = url.searchParams.get('prerelease') === '1';
  const wantVersion = url.searchParams.get('version') || (wantPrerelease ? 'prerelease' : 'latest');

  return new Response(installerScript(wantVersion), {
    status: 200,
    headers: {
      'content-type': 'text/x-shellscript; charset=utf-8',
      'cache-control': 'public, max-age=300',
      'x-prova-installer-version': wantVersion,
    },
  });
}

function installerScript(version: string): string {
  const escVer = JSON.stringify(version);
  return [
    '#!/bin/sh',
    '# Prova CLI installer (https://get.prova.network)',
    '#',
    '# Usage:',
    '#   curl -fsSL https://get.prova.network | sh',
    '#',
    '# Options (env vars):',
    '#   PROVA_VERSION="latest"|"prerelease"|"0.1.0"',
    '#   PROVA_PREFIX="$HOME/.prova"',
    '#   PROVA_NO_PATH=1   skip PATH modification',
    '',
    'set -eu',
    '',
    `VERSION="\${PROVA_VERSION:-${version}}"`,
    'PREFIX="${PROVA_PREFIX:-$HOME/.prova}"',
    'NO_PATH="${PROVA_NO_PATH:-0}"',
    'CLI_PKG="@prova-network/cli"',
    '',
    'red()   { printf "\\033[31m%s\\033[0m\\n" "$1"; }',
    'green() { printf "\\033[32m%s\\033[0m\\n" "$1"; }',
    'dim()   { printf "\\033[2m%s\\033[0m\\n"  "$1"; }',
    'bold()  { printf "\\033[1m%s\\033[0m\\n"  "$1"; }',
    '',
    'bold "Installing Prova CLI"',
    'dim  "  version : $VERSION"',
    'dim  "  prefix  : $PREFIX"',
    'echo',
    '',
    'if command -v npm >/dev/null 2>&1; then',
    '  if npm install -g "$CLI_PKG@$VERSION" 2>/dev/null; then',
    '    echo',
    '    green "✓ Installed via npm"',
    '    echo',
    '    bold "Next: prova auth"',
    '    exit 0',
    '  else',
    '    dim "npm install failed (probably not yet published). Falling back to direct download..."',
    '  fi',
    'fi',
    '',
    'mkdir -p "$PREFIX/bin"',
    'TMP="$(mktemp -d)"',
    'TARBALL="$TMP/prova-cli.tar.gz"',
    '',
    '# Distribution URL. Will switch to R2 once enabled.',
    'TAG="$VERSION"',
    '[ "$TAG" = "latest" ] && TAG="v0.1.0"',
    '[ "$TAG" = "prerelease" ] && TAG="v0.1.0"',
    'case "$TAG" in v*) ;; *) TAG="v$TAG" ;; esac',
    'URL="https://prova.network/cli/${TAG}/prova-cli.tar.gz"',
    '',
    'echo "Fetching $URL"',
    'if command -v curl >/dev/null 2>&1; then',
    '  curl -fsSL "$URL" -o "$TARBALL"',
    'elif command -v wget >/dev/null 2>&1; then',
    '  wget -qO "$TARBALL" "$URL"',
    'else',
    '  red "Need curl or wget to download. Install one and rerun."',
    '  exit 1',
    'fi',
    '',
    'mkdir -p "$PREFIX/lib/cli"',
    'tar -xzf "$TARBALL" -C "$PREFIX/lib/cli" --strip-components=1',
    'rm -rf "$TMP"',
    '',
    'if [ -f "$PREFIX/lib/cli/bin/prova.mjs" ]; then',
    '  ln -sf "$PREFIX/lib/cli/bin/prova.mjs" "$PREFIX/bin/prova"',
    '  chmod +x "$PREFIX/lib/cli/bin/prova.mjs"',
    'else',
    '  red "Tarball did not contain bin/prova.mjs"',
    '  exit 1',
    'fi',
    '',
    'if [ "$NO_PATH" != "1" ]; then',
    '  case ":$PATH:" in',
    '    *":$PREFIX/bin:"*) ;;',
    '    *)',
    '      dim "Add $PREFIX/bin to your PATH:"',
    '      dim "  echo \'export PATH=\\"\\$HOME/.prova/bin:\\$PATH\\"\' >> ~/.zshrc"',
    '      ;;',
    '  esac',
    'fi',
    '',
    'if ! command -v node >/dev/null 2>&1; then',
    '  echo',
    '  red "Prova CLI needs Node 18 or newer."',
    '  echo "  Install Node:  https://nodejs.org/"',
    '  exit 1',
    'fi',
    '',
    'echo',
    'green "✓ Installed at $PREFIX/bin/prova"',
    'echo',
    'bold "Next: prova auth"',
    '',
    `# Installer build: ${escVer}`,
  ].join('\n');
}
