---
description: Revoke an API token by jti.
---

# POST /api/tokens/revoke

Marks a token's `jti` as revoked. The auth middleware refuses revoked jtis on every subsequent request.

```http
POST /api/tokens/revoke HTTP/1.1
Host: prova.network
Authorization: Bearer pk_live_eyJ...
Content-Type: application/json

{ "jti": "f1b32a90-a234-4f2c-8b1e-22cd5c0e5b3a" }
```

## Request body

| Field | Type | Required | Description |
| --- | --- | --- | --- |
| `jti` | string | yes | The `jti` of the token to revoke. Get it from `/api/tokens/list`. |

## Response

```json
{ "jti": "f1b32a90-a234-4f2c-8b1e-22cd5c0e5b3a", "revoked": true }
```

## Constraints

* You can only revoke tokens **you own** (same `userId`).
* You **can** revoke the token you're currently using to call this endpoint. The request will succeed, but every subsequent request with that token will return `401 revoked_token`.
* Revocation is permanent. Once revoked, the same `jti` cannot be re-instated. Mint a new token instead.

## When to use it

* You suspect a token has leaked.
* You're rotating a CI key.
* You've left a job and want to clean up old credentials.
* You want to enforce least-privilege on a forgotten dev token.

## Errors

| Status | `error` | When |
| --- | --- | --- |
| `400` | `invalid_jti` | Body missing or malformed |
| `401` | auth | Auth failed |
| `404` | `not_found` | The jti doesn't belong to this user |
| `503` | `storage_offline` | Revocation KV not bound |

## Notes

The revocation list is stored in KV with a 366-day TTL — equal to the maximum token lifetime. After expiry, the entry is GC'd because the underlying token would also be expired.
