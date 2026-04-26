---
description: Mint an API token from an email address.
---

# POST /api/auth/signup

```http
POST /api/auth/signup HTTP/1.1
Host: prova.network
Content-Type: application/json

{ "email": "you@example.com", "label": "my-app" }
```

## Request body

| Field | Type | Required | Description |
| --- | --- | --- | --- |
| `email` | string | yes | Valid email address. Used to derive a stable user id. |
| `label` | string | no | Up to 64 chars. Shown in the dashboard's token list. Defaults to `unlabeled`. |

## Response

```json
{
  "token":     "pk_live_eyJhbGciOiJIUzI1Ni...",
  "userId":    "ed7bd210fea27a8e",
  "email":     "you@example.com",
  "scopes":    ["put", "get", "list", "pin"],
  "quotaMb":   1024,
  "expiresAt": "2027-04-25T14:40:49.000Z"
}
```

The token is **only returned once**. If you lose it, mint a new one (the user record stays the same — same `userId`, same email, just a different `jti`).

## Errors

| Status | `error` | When |
| --- | --- | --- |
| `400` | `invalid_email` | Email failed regex `^[^\s@]+@[^\s@]+\.[^\s@]+$` |
| `400` | `invalid_json` | Body isn't valid JSON |
| `503` | `auth_offline` | Server-side token signing key not configured |

## Scopes granted

A signup token gets the default scope set: `put | get | list | pin`. To mint a narrower scope (e.g. CI keys with `put`-only), use the dashboard or `POST /api/tokens/create` (coming soon).

## Quota

The default daily quota is **1024 MB / day**. Higher quotas require attaching a wallet and paying USDC.

## Notes

* Multiple signups with the same email give you the **same `userId`** but different `jti`s. You can mint as many tokens as you want (one per `label`).
* Email is normalized to lowercase before hashing. `Foo@bar.com` and `foo@bar.com` are the same user.
* `+` aliases (`me+1@gmail.com`) currently produce different user IDs. This may change in v1.
* Until mainnet, this endpoint **does not send a verification email** — the token is issued immediately. This is a known issue tracked in the [security audit](https://github.com/prova-network/prova/blob/main/SECURITY-AUDIT-2026-04-25.md). Magic-link verification ships before public testnet.
