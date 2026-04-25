# Prova Security Audit · 2026-04-25

Surface: client onboarding stack shipped today.
- `website/functions/_shared/tokens.ts` (HMAC HS256 token mint/verify)
- `website/functions/api/auth/signup.ts`
- `website/functions/api/tokens/{list,revoke}.ts`
- `website/functions/api/usage.ts`, `files.ts`, `upload.ts`
- `website/functions/p/[cid].ts`
- `website/upload/{index,upload}.{html,css,js}`
- `website/app/{index,app}.{html,css,js}`
- `cli/src/**` (Node ESM client)
- Hetzner stage server `/opt/prova-stage-server.py`

Auditor: Capri.
Severity scale: Critical / High / Medium / Low / Informational.
Status emoji: 🔴 unfixed · 🟡 mitigated · 🟢 fixed · ⚪ accepted.

---

## Scope notes

This audit covers the **off-chain** client surface. The Solidity contracts
(`contracts/src/`) are excluded — they live behind a separate audit that's a
prerequisite for the testnet deploy.

The audit is run from the perspective of:
1. an unauthenticated visitor
2. a holder of a valid `pk_live_…` token who tries to escalate
3. a fully-malicious peer with no token at all
4. a curious researcher reading the source on GitHub

---

## Findings

### F-01 · Critical · 🔴 · Account squat: any email mints a token immediately

**File:** `website/functions/api/auth/signup.ts`

```ts
return j({
  token,
  userId: <hash of email>,
  ...
});
```

`POST /api/auth/signup { email: "vitalik.eth@gmail.com" }` returns a fully-
functional API token tied to that email address. There is no email verify
step. An attacker can:

- Squat on `eng@<famous-co>.com`, `legal@vitalik.eth`, etc.
- Bulk-register `*@<co>.com` to drain quota
- Drop a CID to a target's account so the target sees a hostile file in
  `/api/files` listed under their email

Impact: account/quota theft, brand-claim squat, denial-of-service.

**Fix:** Add a magic-link verify step.
- `POST /api/auth/request` -> sends a one-time link `https://.../api/auth/confirm?t=<jwt>`
- The confirm endpoint mints the long-lived token and stores
  `confirmed_at` on the user record
- Until confirmed, no token is issued; pre-confirm KV holds an opaque
  pending state for ~10 min.
- Email delivery via Resend/Postmark/SES.

**Interim mitigation:** Set `quotaMb` to 0 by default. Tokens require a
bootstrap step (manual email verify) before they unlock quota. Existing
`/api/auth/signup` becomes a request-only endpoint.

---

### F-02 · High · 🔴 · No CSRF protection on token endpoints

**Files:** `tokens/revoke.ts`, `tokens/list.ts`, `usage.ts`, `files.ts`, `upload.ts`

All endpoints are bearer-auth only. The bearer is read from
`Authorization: Bearer …` or `?token=…` query string. The query-string fallback
makes any GET / POST endpoint reachable from a malicious cross-origin `<img>`,
`<form>`, or `fetch` if the user has logged into the app dashboard with their
token persisted in `localStorage`.

`localStorage`-stored tokens are also XSS-readable. If anything inside the app
domain is XSS-vulnerable (and we haven't proven it isn't — see F-04), the
attacker pulls the token out of `localStorage` and uses it from anywhere.

Impact: token theft, file-listing leak, revocation of legitimate tokens.

**Fix (in order of return):**
1. Drop the `?token=` query-string fallback. Bearer header only.
2. Add `Origin` / `Referer` allow-list on the API endpoints. Reject if the
   request isn't from `https://prova-network.pages.dev` (or the user hasn't
   sent an Origin and isn't a CLI user-agent).
3. Move the user session into an HttpOnly+Secure+SameSite=Strict cookie for
   the dashboard. The CLI continues to use the bearer header explicitly.
4. Add a CORS allow-list — currently the worker doesn't set CORS headers at
   all; we should explicitly allow `prova-network.pages.dev` and reject
   everything else.

---

### F-03 · High · 🔴 · Stage server bearer key is shared and reusable

**Files:** `/opt/prova-stage-server.py`, `website/functions/api/upload.ts`

The Hetzner stage server uses a single static bearer key (`PROVA_STAGE_KEY`).
The worker reuses it for every PUT. Anyone with read access to the worker
secrets, or who breaches the Hetzner box, can impersonate the worker and
write arbitrary `cid -> bytes` mappings.

Worse, since CIDs are content-addressed, anyone who can upload can plant
poison content under any CID they care to compute. Any prover or client that
later resolves that CID gets the planted content.

Impact: content substitution (the whole point of "verifiable storage" is
broken), quota bypass.

**Fix:**
1. Per-upload signed URLs. Worker mints a short-lived (60s) HMAC over
   `cid|size|owner|exp` and the stage server verifies before accepting.
2. Stage server enforces "first-write-wins": once a CID exists, refuse
   overwrites. Currently the code does `tmp.rename(path)` which overwrites.
3. CID verification at write time: compute SHA-256 of the body, base32
   it the same way the client does, refuse if it doesn't match. This
   alone closes the substitution attack regardless of how the request
   was authed.

The CID-verify step is the strongest mitigation. Even if a bearer key
leaks, the attacker can't substitute under an existing CID because the
hash check fails.

---

### F-04 · High · 🔴 · No HTML escaping on files panel ⇒ stored XSS vector

**File:** `website/app/app.js`

```js
row.innerHTML = `
  <span class="file-name" title="${escapeHtml(f.filename)}">${escapeHtml(f.filename)}</span>
  ...
`;
```

We *do* call `escapeHtml(f.filename)`. Good. But let me look at the field
sources:

- `f.filename` originates from the `X-Filename` request header, decoded with
  `decodeURIComponent`, then sanitized with `replace(/[^\w.\-]+/g, '_')`.
  So an attacker can only get `[a-zA-Z0-9._-]` through. Safe.

- `f.cid` is regex-validated `/^[a-z0-9]{8,80}$/i`. Safe.

- `t.label` in tokens is **not** sanitized. Label flows through:
  `app.js` -> `prompt('Label …')` -> `POST /api/auth/signup { label }` ->
  stored in KV -> `GET /api/tokens/list` returns it ->
  `app.js` `escapeHtml(t.label || 'unnamed')`. ✅ Escaped at render.

  Backend stores up to 64 chars, no validation. Fine because it's escaped on
  read. But: if a future surface renders it without escaping (`prova whoami`,
  email notifications, admin panel), we risk XSS / log injection. **Add
  server-side validation: strip control chars + length cap at signup.**

**Status:** Currently safe. **Fix:** harden server-side validation as defense
in depth, since labels are user-controlled and might be rendered in places we
haven't built yet.

---

### F-05 · Medium · 🔴 · `prova` CLI stores token in world-readable config on shared systems

**File:** `cli/src/util/config.mjs`

```js
await mkdir(CONFIG_DIR, { recursive: true, mode: 0o700 });
await writeFile(CONFIG_PATH, JSON.stringify(cfg, null, 2), { mode: 0o600 });
```

Mode 0700/0600 is correct. But on Windows the mode is ignored. And if the
user's home dir is shared (CI runners, Docker volumes, multi-user boxes),
mode 0600 alone isn't enough — the token sits in plaintext.

**Fix:**
1. Document `PROVA_TOKEN` env var as the preferred CI / container path.
2. Offer `prova auth --no-store` that prints the token but doesn't save.
3. Optional follow-up: encrypt-at-rest using the OS keychain
   (`security` on macOS, `secret-tool` on Linux, `wincred` on Windows).
   Probably overkill for v1.

---

### F-06 · Medium · 🔴 · Token JTI predictable enough to enumerate

**File:** `website/functions/_shared/tokens.ts` & `signup.ts`

Tokens have a `jti = crypto.randomUUID()`. RFC 4122 v4 UUIDs are 122 random
bits. That's fine. But the **revocation key** in KV is `tokens:<jti>`. If an
attacker knows / guesses a JTI they can ping the revoke endpoint.

But revoke requires a bearer token + ownership check (F-04 scoped: 
`PROVA_USERS.get(u:<sub>:t:<jti>)`), so cross-user revoke is blocked.
However, the auth-token JTI is **inside the JWT**, which is sent over the
wire to the worker every request. The JWT is HS256 — if `PROVA_TOKEN_SECRET`
ever leaks (CI logs, env dump, git accident), every JTI in the wild becomes
forgeable retroactively.

**Fix:**
1. Move `PROVA_TOKEN_SECRET` to a Cloudflare-wrangler-managed binding (already
   done — it's a Pages secret) but also document the rotation runbook:
   > Rotating `PROVA_TOKEN_SECRET` invalidates all live tokens. Users
   > re-issue via /api/auth/signup (post-magic-link).
2. Rotate every 90 days as policy.
3. Long-term: switch to RS256 with a published JWKS so verifiers can't forge,
   only sign-ers can. Probably overkill until we have third-party verifiers.

---

### F-07 · Medium · 🔴 · Quota check race

**File:** `website/functions/api/upload.ts`

```ts
const used = parseInt((await env.PROVA_RATE.get(rateKey)) || '0', 10);
if (used + sizeBytes > limit) return 429;
// … upload happens …
await env.PROVA_RATE.put(rateKey, String(used + sizeBytes), { … });
```

Read-then-write isn't atomic in KV. Two concurrent uploads both read the
same `used`, both pass the check, both write. The user can exceed quota by
a factor proportional to concurrency.

**Fix:**
1. Use Durable Objects for per-user counters (atomic increments).
2. Or: accept the race for the free tier (cheap to abuse, low blast radius)
   and only tighten when quotas are paid.
3. Soft limit that reconciles every minute via a ledger table — for free
   tier, this is fine.

**Verdict:** accept for free tier. Note in roadmap to switch to DO when
metered.

---

### F-08 · Medium · 🔴 · Stage server has no body-size enforcement during stream

**File:** `/opt/prova-stage-server.py`

```python
clen = int(self.headers.get('content-length', '0') or '0')
if clen > MAX_BODY: return 413
…
remaining = clen
while remaining > 0:
    chunk = self.rfile.read(min(remaining, 64*1024))
    f.write(chunk)
    remaining -= len(chunk)
```

We trust `content-length`. A client lying about content-length (claiming
50 KB but streaming forever) will eventually exhaust disk because the
stage server only reads `clen` bytes — actually, **OK, this part is safe**:
we read exactly `clen` bytes and stop. The DOS vector is different:
a slow-loris that opens many connections, writes 1 byte/sec, holds for
minutes. The Python `ThreadingHTTPServer` spawns a thread per connection,
no read timeout, no max concurrent connections.

**Fix:**
1. Set `BaseHTTPRequestHandler.timeout = 30` or use a real WSGI server
   (waitress, gunicorn).
2. Add `iptables` rate limit by source IP on Hetzner (since the stage server
   is behind Cloudflare Worker for production traffic, the only direct
   callers are us).
3. Better: switch to nginx + a small Python WSGI app, lean on nginx's
   `limit_req_zone` + connection limits. (Already-installed on the Hetzner
   box.)

---

### F-09 · Medium · 🔴 · `escapeHtml` in app.js trusts source structure too eagerly

**File:** `website/app/app.js`

`f.uploadedAt`, `f.size`, `t.createdAt`, etc. flow into template literals
without escaping. Currently they come from JSON we wrote, but if the KV
gets corrupted, or if a future endpoint forwards user input here, we have
a sleeping XSS.

**Fix:** Use `textContent =` / `appendChild` for everything user-influenced.
The current `innerHTML` template-string pattern is fragile. Either:
- Migrate to a tiny render helper (`el('span', {class: '…'}, text)`)
- Or: escape **every** dynamic value, not just the obvious ones.

**Status:** Low practical risk today. Fix as defense in depth before audit.

---

### F-10 · Medium · 🔴 · CLI hash isn't real CommP, claims to be `bafy…`

**File:** `cli/src/util/hash.mjs`, `website/upload/upload.js`

```js
return 'bafy' + base32(new Uint8Array(hash)).slice(0, 52);
```

This is **SHA-256 dressed up as a `bafy…` CID**. It's not a real CIDv1, not
a real CommP, doesn't validate against any ipld codec. We're passing it
around as if it were the canonical content address.

Two problems:
1. Anyone who knows the algo (it's open source) can compute `bafy<X>` for
   any content. That's actually fine — that's how content-addressing works.
2. **But:** when we wire to the real PDP contract on Base, the contract
   expects a real `bytes32 commpHash` derived from the proper CommP
   (sha256-trunc254-padded, multihash 0x1012). Our fake hash is not that.
   So either:
   - The bridge from "browser hash" -> "on-chain commp" has to do a server-side
     re-hash. That means re-uploading or re-computing.
   - Or we just admit the browser hash is a *receipt*, not a real CID, and
     name it differently (`receipt-id`?).

**Fix (before testnet deploy):**
- Either ship real CommP in the browser (WASM) and hash deterministically.
- Or rename `cid` to `receipt-id` everywhere and only call it `piece-cid`
  after the prover has computed and signed off.

**Status:** This is a correctness bug, not a security bug per se, but it
will turn into a fraud surface (claim-arbitrage between fake-cid receipts and
real piece-cids) if we ship as-is.

---

### F-11 · Low · 🔴 · Email field accepts `+` aliases that resolve to same userId

**File:** `signup.ts`

`userId = sha256("prova:" + email).slice(0,16)` — so `me@x.com` and
`me+evil@x.com` produce different userIds. That's the standard behavior of
most SaaS, but it allows a single human to claim multiple quotas
(`me+1@`, `me+2@`, …).

**Fix:** Normalize the email before hashing. `gmail.com` style: drop `+suffix`
and `.` from the local part. Document the policy.

**Status:** Accept for v1; document the policy in TOS.

---

### F-12 · Low · 🔴 · No rate limit on `/api/auth/signup`

**File:** `signup.ts`

An attacker can spam `POST /api/auth/signup` with random emails to mint
tokens. Each token gets 1 GiB/day. With 1000 tokens that's 1 TiB/day of
free quota.

**Fix:**
1. Per-IP rate limit on `/api/auth/signup` (e.g. 10/hour).
2. Reuse the same KV-based per-IP counter we have for sponsored uploads.
3. After magic-link is added (F-01), this collapses to "spam emails to
   real addresses" which is rate-limited by the email provider.

---

### F-13 · Low · 🔴 · CORS not set anywhere

**Files:** all `functions/`

The worker returns no `Access-Control-Allow-Origin` header. Browsers will
block cross-origin XHRs to `/api/*` because the response is opaque.

Currently this isn't broken because the dashboard, upload page, and CLI all
talk to `https://prova-network.pages.dev` from the same origin (or no Origin,
in CLI's case).

**Fix:**
1. Add explicit CORS to all `/api/*`:
   - `Access-Control-Allow-Origin: https://prova-network.pages.dev`
   - `Vary: Origin`
2. Handle OPTIONS preflight on `/api/upload` (which uses `authorization` +
   `x-filename` custom headers).
3. Reject other origins.

---

### F-14 · Informational · 🔴 · No /robots.txt or /security.txt

**Fix:** Add `/security.txt` with a contact (`security@prova.network`),
preferred-languages, encryption (PGP key fingerprint optional), policy URL.
RFC 9116. Quick win, signals seriousness.

---

### F-15 · Informational · 🔴 · No Content-Security-Policy header

**Files:** Cloudflare Pages serves static files with no CSP.

**Fix:** Add `_headers` file at site root:

```
/*
  Content-Security-Policy: default-src 'self'; script-src 'self' https://unpkg.com; style-src 'self' https://fonts.googleapis.com; font-src https://fonts.gstatic.com; img-src 'self' data: https:; connect-src 'self' https://prova-network.pages.dev; frame-ancestors 'none'
  Strict-Transport-Security: max-age=63072000; includeSubDomains; preload
  X-Frame-Options: DENY
  X-Content-Type-Options: nosniff
  Referrer-Policy: strict-origin-when-cross-origin
  Permissions-Policy: geolocation=(), camera=(), microphone=()
```

---

### F-16 · Informational · 🔴 · No abuse-reporting hook

If someone uploads CSAM / illegal content, we have no take-down mechanism.

**Fix:**
1. `/abuse` page with form -> emails an abuse mailbox.
2. Background scan of new uploads for known-bad SHA-256 hashes (NCMEC
   PhotoDNA-style or open hash list).
3. Worker supports admin-issued `cid -> blocked` flag in KV; both `/p/<cid>`
   and `/api/upload` check it.

This is a precondition for opening to public traffic, not just security.

---

## Summary

| Sev | Count |
| --- | --- |
| Critical | 1 |
| High     | 3 |
| Medium   | 6 |
| Low      | 3 |
| Info     | 3 |
| **Total** | **16** |

**Pre-testnet checklist (must fix before public testnet):**
- F-01 (magic-link)
- F-02 (Origin allow-list, drop `?token=` query)
- F-03 (CID verify on stage server, first-write-wins)
- F-10 (rename or implement real CommP)
- F-12 (signup rate limit)
- F-13 (CORS)
- F-15 (CSP headers)
- F-16 (abuse + CSAM hook)

**Pre-mainnet:**
- All Mediums
- F-04 polish, F-09 polish

**Out of scope (separate audit):**
- Solidity contracts in `contracts/src/`
- Prover daemon `prover/` (Go)
- SDK `sdk/typescript/`

---

_Audit closes with: do not let public traffic at this surface until F-01,
F-02, F-03, F-10, F-12, F-13, F-15, F-16 are fixed. Internal smoke testing
with one or two known emails is fine._
