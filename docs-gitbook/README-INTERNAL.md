# Prova docs (internal staging)

This repo holds the public-facing Prova documentation, **staged but not yet
live**.

It's currently:
- Private on GitHub (`prova-network/docs`)
- Wired into a GitBook space in the `Carpi` org (`xMgWCAT8I8F7TVmd4NJl`)
- Not connected to GitBook's GitHub App yet (zero-click activation: when ready,
  click sync in GitBook UI)
- Not pointed at any DNS (docs.prova.network is unconfigured)

## Why staged

Nicklas is on contract with Curio / Protocol Labs / FilOzone until he exits.
The Ethereum-native pivot in Prova is confidential until that exit. These
docs describe the pivot in detail, so they cannot go public until he is
clear.

The rest of the Prova client surface (upload, app, API, CLI) is live at
`prova.network` because none of it names the pivot. Only docs do.

## Go-live runbook

When Nicklas confirms exit:

1. Flip the GitHub repo to public:
   ```
   /Users/reiers/.openclaw/workspace/prova/scripts/docs-go-live.sh
   ```
2. The script:
   - Sets `prova-network/docs` to public on GitHub
   - Re-creates the `docs.prova.network` DNS CNAME (proxied -> hosting.gitbook.io)
   - Re-creates the Pages domain entry (defensive — GitBook handles its own
     cert via Cloudflare for SaaS, so the Pages domain is just a placeholder
     to prevent collision)
   - Cuts a new GitHub Release v0.1.0 (no asset, just the version tag)
   - Posts a deploy summary

3. **One required manual step** (GitBook API doesn't expose this):
   - Open https://app.gitbook.com/o/ASR6x2sseFhc2xkDhHoN/s/xMgWCAT8I8F7TVmd4NJl/
   - Sidebar -> Integrations -> GitHub
   - Authorize GitBook GitHub App on `prova-network/docs`
   - Set the custom domain to `docs.prova.network` in site settings

After that one click, every push to `main` here syncs to docs.prova.network
within ~30 seconds.

## Until then

Edit the docs locally. Push to `main`. They stay in this private repo,
versioned. When the gate lifts, the runbook ships them all at once.

```bash
cd ~/.openclaw/workspace/prova/docs-gitbook
# edit markdown
git add -A && git commit -m "docs: …"
git push
```

## Repo structure

- `README.md` — homepage rendered by GitBook
- `SUMMARY.md` — table of contents
- `getting-started/` — quickstart pages
- `concepts/` — architecture, piece-cids, deal-lifecycle, continuous-proof, resilience
- `api/` — full HTTP API reference
- `cli/` — `prova` subcommand reference
- `sdk/` — TypeScript SDK pages
- `provers/` — operator-facing pages
- `reference/` — glossary, errors, changelog
- `cli/prova-cli-latest.tar.gz` — historical artifact (no longer the
  install path; the live tarball lives at `prova/website/cli/v0.1.0/`
  and is served from the Pages site directly)
