---
description: Tokens, scopes, and how to keep them safe.
---

# Authentication

Prova uses **bearer tokens** for everything except sponsored uploads and public retrieval.

## Token shape

```
pk_live_eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.<payload>.<signature>
```

It's an HS256-signed JWT, prefixed with `pk_live_`. The payload contains:

```json
{
  "sub":     "ed7bd210fea27a8e",
  "email":   "you@example.com",
  "scopes":  ["put", "get", "list", "pin"],
  "quotaMb": 1024,
  "iat":     1777128049,
  "exp":     1808664049,
  "jti":     "e88e5343-0827-486b-a824-039473cdd710"
}
```

* `sub` — user id, derived from a hash of your email
* `quotaMb` — daily byte quota in megabytes
* `jti` — unique token id, used for revocation

You can decode the payload yourself (it's just base64) but you can't forge a valid signature without the server's HMAC key.

## Scopes

| Scope | Allows |
| --- | --- |
| `put` | Upload via `/api/upload` |
| `get` | Retrieve via `/p/{cid}` (only meaningful when files become non-public) |
| `list` | List your files / tokens |
| `pin` | Extend a deal, manage redundancy |

A default token gets all four. CI tokens should be `put`-only.

## How to get one

```bash
curl -X POST https://prova.network/api/auth/signup \
  -H "content-type: application/json" \
  -d '{"email":"you@example.com","label":"my-app"}'
```

Returns `{ token, userId, email, scopes, quotaMb, expiresAt }`. Store the `token` somewhere safe — it's only shown to you once. (You can mint additional tokens later from the [dashboard](https://prova.network/app/).)

## How to use one

In a header:

```http
Authorization: Bearer pk_live_eyJ...
```

In a query string (discouraged, deprecated):

```
https://prova.network/api/files?token=pk_live_eyJ...
```

The query-string form will be removed in v1.

## Token rotation

```bash
# Mint a new token alongside the old one
curl -X POST https://prova.network/api/auth/signup \
  -H "content-type: application/json" \
  -d '{"email":"you@example.com","label":"laptop-2026-04"}'

# Revoke the old one
curl -X POST https://prova.network/api/tokens/revoke \
  -H "authorization: Bearer pk_live_NEW..." \
  -H "content-type: application/json" \
  -d '{"jti":"OLD-jti-here"}'
```

Tokens cannot be modified. To change scopes or quota, mint a new one and revoke the old.

## Where to store tokens

* **CLI** — `~/.prova/config.json` (mode 600)
* **CI** — `PROVA_TOKEN` env var, never echoed to logs
* **Browser apps** — `localStorage` is currently used; we'll move to HttpOnly+Secure+SameSite=Strict cookies in v1
* **Mobile apps** — platform keychain (iOS Keychain, Android Keystore)

Never:
* Commit tokens to git
* Embed tokens in client-side JavaScript bundles
* Share tokens via email or chat

## What to do if a token leaks

1. [Revoke it immediately](tokens-revoke.md) from the dashboard or via the API
2. Mint a replacement
3. Audit `/api/usage` for the leaked period to see if it was abused
4. If it was, file an incident report (see [Reference / Errors](../reference/errors.md))

A revoked token's signature still verifies, but the auth middleware checks the revocation list (KV-backed) on every request and returns `401 revoked_token` if it's there.

## Security model

Tokens are signed with a server-side HMAC key (`PROVA_TOKEN_SECRET`). If that key is rotated, all live tokens become invalid. Users have to re-issue.

We rotate the key every 90 days, or immediately on any suspected compromise.

The HMAC scheme is symmetric — only Prova's worker can sign or verify. If we ever open up third-party verification (e.g., a marketplace SDK that wants to validate tokens locally), we'll switch to RS256 with a published JWKS endpoint.
