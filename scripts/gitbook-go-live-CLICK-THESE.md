# GitBook go-live — final 2 manual clicks

GitBook's API doesn't expose Git Sync setup or custom domain config.
Both have to be done in the UI. ~30 seconds total.

## Step 1 — connect GitHub Sync

Open: <https://app.gitbook.com/o/ASR6x2sseFhc2xkDhHoN/s/xMgWCAT8I8F7TVmd4NJl/site/synchronize>

(Alt path if that doesn't work: open the space app → settings → integrations → GitHub)

- Click **Connect GitHub**
- Authorize GitBook GitHub App (will land on github.com → install + authorize → returns)
- When asked which repos to grant access to, pick **`prova-network/docs`** (or grant org-wide access if you prefer)
- Back in GitBook, pick the repo **`prova-network/docs`**, branch **`main`**, project root **`/`**
- Click **Save**

GitBook will pull the markdown immediately. Within 30 seconds, the space's
table of contents matches `SUMMARY.md`, and `README.md` becomes the homepage.

## Step 2 — set custom domain

Open: <https://app.gitbook.com/o/ASR6x2sseFhc2xkDhHoN/sites/site_HGN3e/settings/domains>

- Click **Add custom domain**
- Enter `docs.prova.network`
- Click **Save**

The DNS CNAME `docs.prova.network -> hosting.gitbook.io` is already in
Cloudflare, so GitBook will validate immediately and provision a TLS cert.

Within ~2 minutes:

```
curl -I https://docs.prova.network/
HTTP/2 200
```

## After that

Every `git push` to `prova-network/docs` syncs to docs.prova.network within
~30 seconds. You can also edit in the GitBook UI; those edits get committed
back to `main` as commits authored by GitBook.

## Status check

```bash
# GitHub:
curl -s https://api.github.com/repos/prova-network/docs | jq '.private, .size'

# GitBook content:
curl -s "https://api.gitbook.com/v1/spaces/xMgWCAT8I8F7TVmd4NJl/content" \
  -H "Authorization: Bearer $(grep GITBOOK_API_KEY ~/.openclaw/workspace/.vault/gitbook.md | cut -d= -f2)" \
  | jq '.pages | length'

# Live site:
curl -I https://docs.prova.network/
```
