---
description: List your stored files.
---

# GET /api/files

Returns every file owned by the authenticated user.

```http
GET /api/files HTTP/1.1
Host: prova.network
Authorization: Bearer pk_live_eyJ...
```

## Response

```json
{
  "userId":  "ed7bd210fea27a8e",
  "email":   "you@example.com",
  "count":   3,
  "files": [
    {
      "cid":         "baga…q4kr",
      "size":        4194304,
      "filename":    "my-site.tar.gz",
      "contentType": "application/gzip",
      "uploadedAt":  "2026-04-25T14:41:00.359Z",
      "term":        "30 days",
      "sponsored":   false
    },
    {
      "cid":         "baga…3xyz",
      "size":        524288,
      "filename":    "screenshot.png",
      "contentType": "image/png",
      "uploadedAt":  "2026-04-24T10:15:22.110Z",
      "term":        "30 days",
      "sponsored":   false
    }
  ]
}
```

Files are sorted by `uploadedAt` descending (newest first).

## Pagination

Currently returns up to 1000 files in one shot. Pagination via `?cursor=` arrives in v1.

## Scope

Requires the `list` scope.

## Errors

| Status | `error` | When |
| --- | --- | --- |
| `401` | `no_token` / `invalid_token` / `revoked_token` | Auth failed |
| `403` | `insufficient_scope` | Token lacks `list` |
