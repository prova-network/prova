---
description: HTTP API for storing and retrieving files on Prova.
---

# API reference

The Prova HTTP API is the canonical surface. The CLI and the SDK are thin wrappers; anything you can do with the CLI, you can do with `curl`.

## Base URL

```
https://prova.network
```

All endpoints accept HTTPS only. HTTP is force-redirected.

## Authentication

Most endpoints require a bearer token:

```http
Authorization: Bearer pk_live_eyJhbGciOiJIUzI1Ni…
```

Tokens are minted via [`POST /api/auth/signup`](auth-signup.md). They're scoped, revocable, and expire 1 year after issue. See [Authentication](authentication.md).

The two endpoints that **don't** require auth:

* [`POST /api/upload`](upload.md) — accepts uploads in sponsored mode (no token, IP-rate-limited)
* [`GET /p/{cid}`](retrieve.md) — public retrieval

## Content types

Requests and responses are JSON unless noted (uploads are raw bytes; retrievals are the original content type).

## Errors

```json
{
  "error": "rate_limited",
  "detail": "Daily quota of 1024 MB reached.",
  "used": 1073741824,
  "limit": 1073741824
}
```

| Status | Meaning |
| --- | --- |
| `400` | Bad request — see `error` field |
| `401` | Missing or invalid token |
| `403` | Token lacks the required scope |
| `404` | CID not found |
| `413` | File too large for tier |
| `429` | Quota / rate limit hit |
| `503` | Storage backend offline |

## Rate limits

* Sponsored uploads: 200 MB / IP / 24 h
* Authenticated: per-user daily quota (default 1 GiB)
* Token issuance: 10 / IP / hour

Headers on every response:

```
X-RateLimit-Limit: 1073741824
X-RateLimit-Remaining: 1024000000
X-RateLimit-Reset: 1777212444
```

## Endpoints

| Method | Path | Auth | Purpose |
| --- | --- | --- | --- |
| `POST` | [`/api/auth/signup`](auth-signup.md) | none | Mint a token from an email |
| `POST` | [`/api/upload?cid=<cid>`](upload.md) | optional | Stage bytes for storage |
| `GET` | [`/p/{cid}`](retrieve.md) | none | Retrieve stored bytes |
| `GET` | [`/api/files`](files.md) | required | List your stored files |
| `GET` | [`/api/usage`](usage.md) | required | Quota usage (today + last 7 days) |
| `GET` | [`/api/tokens/list`](tokens-list.md) | required | List your tokens |
| `POST` | [`/api/tokens/revoke`](tokens-revoke.md) | required | Revoke a token by jti |

## Versioning

The API is currently un-versioned (effectively `v0`). Once we ship the first stable release, all endpoints will move under `/v1/…` and the current paths will alias to `v1` for one year.
