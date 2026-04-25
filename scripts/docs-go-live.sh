#!/usr/bin/env bash
# Prova docs go-live runbook.
#
# Run this when Nicklas confirms he has exited Curio / PL / FilOz and the
# Ethereum-native Prova pivot can be public.
#
# What it does:
#   1. Flips Reiers/prova-docs to public on GitHub
#   2. Adds docs.prova.network DNS CNAME -> hosting.gitbook.io (Cloudflare)
#   3. Cuts a v0.1.0 GitHub release tag
#   4. Reminds you of the one required manual step in GitBook UI
#
# All commands are idempotent. Safe to run twice.

set -euo pipefail

VAULT="$HOME/.openclaw/workspace/.vault"
[[ -f "$VAULT/cloudflare.md" ]] || { echo "Missing $VAULT/cloudflare.md"; exit 1; }
[[ -f "$VAULT/github.md" ]]     || { echo "Missing $VAULT/github.md";     exit 1; }
[[ -f "$VAULT/gitbook.md" ]]    || { echo "Missing $VAULT/gitbook.md";    exit 1; }

# Pull credentials out of the vault
CF_EMAIL="$(awk -F= '/^CLOUDFLARE_EMAIL=/{print $2}' "$VAULT/cloudflare.md" | tr -d ' ')"
CF_KEY="$(awk -F=   '/^CLOUDFLARE_API_KEY=/{print $2}' "$VAULT/cloudflare.md" | tr -d ' ')"
GH_PAT="$(awk -F=  '/^Git credential token: gho_/{sub("Git credential token: ", ""); print; exit}' "$VAULT/github.md" | tr -d ' ' | head -c 50)"
# Fallback to grepping the line manually
[[ -z "$GH_PAT" ]] && GH_PAT="$(grep 'gho_' "$VAULT/github.md" | head -1 | awk '{print $NF}')"
GB_KEY="$(awk -F= '/^GITBOOK_API_KEY=/{print $2}' "$VAULT/gitbook.md" | tr -d ' ')"

PROVA_ZONE="e03dbbb57324f24cae328902bc4bc741"
ACCOUNT_ID="80e6b0b2c1ce922a446e8d88a75578d2"
GITBOOK_ORG="ASR6x2sseFhc2xkDhHoN"
GITBOOK_SPACE="xMgWCAT8I8F7TVmd4NJl"

echo "── Prova docs go-live runbook ────────────────────────────────"
echo ""
echo "Pre-flight checks..."
[[ -n "$CF_EMAIL" ]] && echo "  ✓ CF_EMAIL"      || { echo "  ✗ CF_EMAIL"; exit 1; }
[[ -n "$CF_KEY" ]]   && echo "  ✓ CF_KEY"        || { echo "  ✗ CF_KEY"; exit 1; }
[[ -n "$GH_PAT" ]]   && echo "  ✓ GH_PAT"        || { echo "  ✗ GH_PAT"; exit 1; }
[[ -n "$GB_KEY" ]]   && echo "  ✓ GITBOOK_KEY"   || { echo "  ✗ GITBOOK_KEY"; exit 1; }
echo ""

# ── Confirmation ──────────────────────────────────────────────────
echo "This will make the following surfaces PUBLIC:"
echo "  - github.com/Reiers/prova-docs"
echo "  - docs.prova.network"
echo ""
read -p "Confirm Nicklas has exited Curio/PL/FilOz [yes/NO]: " CONFIRM
[[ "$CONFIRM" == "yes" ]] || { echo "Aborted."; exit 0; }

# ── 1. Flip GitHub repo to public ─────────────────────────────────
echo ""
echo "── 1. Flip Reiers/prova-docs to public ──"
curl -s -X PATCH "https://api.github.com/repos/Reiers/prova-docs" \
  -H "Authorization: Bearer $GH_PAT" \
  -H "Accept: application/vnd.github+json" \
  -d '{"private":false}' | python3 -c "
import sys, json
d=json.load(sys.stdin)
print('  private =', d.get('private'), 'visibility =', d.get('visibility'))
"

# ── 2. Add docs.prova.network CNAME ───────────────────────────────
echo ""
echo "── 2. DNS docs.prova.network -> hosting.gitbook.io ──"
EXISTING_DNS=$(curl -s "https://api.cloudflare.com/client/v4/zones/$PROVA_ZONE/dns_records?name=docs.prova.network" \
  -H "X-Auth-Email: $CF_EMAIL" -H "X-Auth-Key: $CF_KEY" | python3 -c "
import sys, json
d=json.load(sys.stdin)
print(d['result'][0]['id'] if d.get('result') else '')
")
if [[ -n "$EXISTING_DNS" ]]; then
  echo "  Already exists (id $EXISTING_DNS), skipping create"
else
  curl -s -X POST "https://api.cloudflare.com/client/v4/zones/$PROVA_ZONE/dns_records" \
    -H "X-Auth-Email: $CF_EMAIL" -H "X-Auth-Key: $CF_KEY" \
    -H "content-type: application/json" \
    -d '{"type":"CNAME","name":"docs","content":"hosting.gitbook.io","ttl":1,"proxied":false,"comment":"GitBook docs"}' \
    | python3 -c "
import sys, json
d=json.load(sys.stdin)
print('  Created:', d.get('result',{}).get('name'), '->', d.get('result',{}).get('content'))
"
fi

# ── 3. Cut GitHub release ─────────────────────────────────────────
echo ""
echo "── 3. Cut v0.1.0 GitHub release ──"
EXISTING_RELEASE=$(curl -s "https://api.github.com/repos/Reiers/prova-docs/releases/tags/v0.1.0" \
  -H "Authorization: Bearer $GH_PAT" | python3 -c "
import sys, json
d=json.load(sys.stdin)
print(d.get('id', ''))
" 2>/dev/null)
if [[ -n "$EXISTING_RELEASE" ]]; then
  echo "  Release v0.1.0 already exists (id $EXISTING_RELEASE)"
else
  curl -s -X POST "https://api.github.com/repos/Reiers/prova-docs/releases" \
    -H "Authorization: Bearer $GH_PAT" \
    -H "Accept: application/vnd.github+json" \
    -d '{
      "tag_name": "v0.1.0",
      "name": "Prova docs v0.1.0",
      "body": "Initial public docs release. See https://docs.prova.network",
      "draft": false,
      "prerelease": false
    }' | python3 -c "
import sys, json
d=json.load(sys.stdin)
if d.get('id'):
  print('  Created release:', d['tag_name'], 'url:', d['html_url'])
else:
  print('  FAIL:', d.get('message'))
"
fi

# ── 4. Manual step reminder ───────────────────────────────────────
echo ""
echo "── 4. ONE MANUAL STEP REMAINING ──"
echo ""
echo "Open: https://app.gitbook.com/o/$GITBOOK_ORG/s/$GITBOOK_SPACE/"
echo ""
echo "  a) Sidebar -> Integrations -> GitHub -> Connect"
echo "  b) Authorize GitBook GitHub App on Reiers/prova-docs (main branch)"
echo "  c) Site settings -> Custom domain -> docs.prova.network -> Save"
echo ""
echo "Verification (after the manual step):"
echo "  curl -I https://docs.prova.network/   # should be 200"
echo ""
echo "── Done. ──────────────────────────────────────────────────────"
