---
description: Inspect every token you've issued.
---

# GET /api/tokens/list

Returns every token tied to the authenticated user, with metadata.

```http
GET /api/tokens/list HTTP/1.1
Host: prova.network
Authorization: Bearer pk_live_eyJ...
```

## Response

```json
{
  "userId": "ed7bd210fea27a8e",
  "email":  "you@example.com",
  "tokens": [
    {
      "jti":         "e88e5343-0827-486b-a824-039473cdd710",
      "label":       "laptop",
      "createdAt":   "2026-04-25T14:40:49.000Z",
      "expiresAt":   "2027-04-25T14:40:49.000Z",
      "scopes":      ["put", "get", "list", "pin"],
      "quotaMb":     1024,
      "isCurrent":   true
    },
    {
      "jti":         "f1b32a90-a234-4f2c-8b1e-22cd5c0e5b3a",
      "label":       "ci",
      "createdAt":   "2026-04-20T09:11:22.000Z",
      "expiresAt":   "2027-04-20T09:11:22.000Z",
      "scopes":      ["put"],
      "quotaMb":     1024,
      "isCurrent":   false
    }
  ]
}
```

`isCurrent: true` is the token used in the request that just hit this endpoint. Useful for rendering "this is the device you're on" labels.

The full token (signature + payload) is **never** returned here. Only the metadata. The full token was shown once at signup; if you've lost it, mint a new one.

## Scope

Requires the `list` scope (granted by default).

## Errors

| Status | `error` | When |
| --- | --- | --- |
| `401` | auth | Auth failed |
| `403` | `insufficient_scope` | Token lacks `list` |
