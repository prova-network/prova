<div class="spec-meta">
  <span class="label">State</span> <span class="badge state-reliable">Reliable</span>
  <span class="label">Theory audit</span> <span class="badge audit-na">N/A</span>
  <span class="label">Last updated</span> 2026-04-26
</div>

# 4.2 API gateway

The public REST surface served at `prova.network/api/*`. The gateway is a Cloudflare Pages Functions deployment that fronts the marketplace, the staging backend, and the user-facing token system.

The full reference (every endpoint with examples) is on the docs site at [docs.prova.network/api](https://docs.prova.network/api). This section is the spec — what each endpoint commits to.

## 4.2.1 Base URL

```
https://prova.network/api
```

All endpoints accept and return `application/json` unless explicitly noted (uploads accept any `content-type`; piece retrieval through `/p/{cid}` returns the original bytes).

## 4.2.2 Authentication

Bearer-token only. The gateway MUST NOT accept `?token=` query-string authentication on any endpoint (closed; F-02 in the security audit).

```
Authorization: Bearer <pk_live_…>
```

Tokens are JWTs signed with HMAC-SHA-256 over the gateway secret. Token lifetime: 1 year, with a published revocation list. Token issuance is described in §4.2.4.

## 4.2.3 Origin / CSRF guard

Mutating endpoints (`POST`, `PUT`, `DELETE`) MUST verify the `Origin` or `Referer` header against a same-site allowlist:

- `prova.network`
- `www.prova.network`
- `*.prova-network.pages.dev` (preview deploys)

CLI / SDK callers without an Origin header MAY be allowed only on endpoints that accept Bearer authentication.

## 4.2.4 Magic-link sign-in

Two-step flow:

```
POST /api/auth/start  { email, label?, returnUrl? }
  → 200 { sent: true, email, expiresIn: 900, challenge }

POST /api/auth/verify { challenge } | { email, code }
  → 200 { token, userId, email, scopes, quotaMb, expiresAt, returnUrl }
```

`/start` is rate-limited to 5 per IP and 5 per email in any 15-minute window. The challenge is single-use and burned on verify (regardless of success). The 6-digit code has at most 5 attempts before the challenge is invalidated.

## 4.2.5 Upload

```
POST /api/upload?cid={piece-cid}
Authorization: Bearer <pk_live_…>   (optional; without auth, sponsored tier)
Content-Type: <mime>
X-Filename: <filename>

<binary body>
```

Constraints:

- The `cid` query parameter MUST be a Filecoin piece-CID (matches `^baga[a-z0-9]{4,76}$`). Older `bafy…` SHA-256 stubs are rejected with 400 `invalid_cid`.
- File size limits:
  - Sponsored (no auth): 100 MB / file, 200 MB / IP / 24h
  - Authenticated: 5 GiB / file, daily quota per token
- The gateway MUST reject the upload with 422 `cid_mismatch` if the bytes received don't hash to the claimed CID.
- The gateway MUST consult an IP ban list before processing.
- Per-IP rate limit: 60 upload calls per minute.

Successful response:

```json
{
  "cid": "baga6ea4r…",
  "dealId": "d-0x…",
  "size": 12345,
  "retrievalUrl": "https://prova.network/p/baga6ea4r…",
  "term": "30 days",
  "sponsored": true,
  "owner": null
}
```

## 4.2.6 Retrieval

```
GET /p/{cid}
HEAD /p/{cid}
```

Streams the piece bytes from the staging backend. Headers MUST include:

- `x-prova-piece-cid: <cid>`
- `x-prova-verified: 1` if the staging server recomputed the CID at intake
- `x-prova-source: stage` (or `r2` once R2 is enabled)
- `content-disposition: attachment; filename=…` for non-media types
- `content-security-policy: default-src 'none'; sandbox` for served bytes
- `x-content-type-options: nosniff`

The retrieval is publicly accessible; no authentication required.

## 4.2.7 User endpoints

| Endpoint | Method | Purpose |
| --- | --- | --- |
| `/api/usage` | GET | Quota usage for the bearer token |
| `/api/files` | GET | List files owned by the bearer token's user |
| `/api/tokens/list` | GET | List active tokens for the user |
| `/api/tokens/revoke` | POST | Revoke a token by its `jti` |

All require Bearer auth.

## 4.2.8 Abuse reporting

```
POST /api/abuse/report  { cid?, url?, reason, contact? }
  → 200 { received: true, reportId, detail }
```

Public, unauthenticated. Rate-limited to 10 reports per IP per hour. Reason MUST be ≥ 20 characters. Either `cid` or `url` MUST be provided.

The report is logged to KV with a 1-year TTL and forwarded to `hello@prova.network` via Resend if configured.

## 4.2.9 CSP and security headers

Every response from the gateway MUST include the global security headers set by `_middleware.ts`:

- `content-security-policy` (allowlists self + jsdelivr + unpkg for known third-party scripts)
- `x-content-type-options: nosniff`
- `x-frame-options: DENY`
- `referrer-policy: strict-origin-when-cross-origin`
- `permissions-policy: camera=(), microphone=(), geolocation=(), payment=(), usb=()`
- `strict-transport-security: max-age=63072000; includeSubDomains; preload`
- `cross-origin-opener-policy: same-origin`
- `cross-origin-resource-policy: same-site`

## 4.2.10 Source

[`website/functions/api`](https://github.com/prova-network/website/tree/main/functions/api). Every endpoint above maps to a `*.ts` file in that directory.
