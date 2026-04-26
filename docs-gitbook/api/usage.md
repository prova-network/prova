---
description: Quota and consumption for the authenticated user.
---

# GET /api/usage

Returns today's quota usage and a 7-day rolling history.

```http
GET /api/usage HTTP/1.1
Host: prova.network
Authorization: Bearer pk_live_eyJ...
```

## Response

```json
{
  "userId":              "ed7bd210fea27a8e",
  "email":               "you@example.com",
  "quotaMb":             1024,
  "quotaBytes":          1073741824,
  "today": {
    "date":  "2026-04-25",
    "bytes": 215896
  },
  "last7Days": [
    { "date": "2026-04-19", "bytes": 0 },
    { "date": "2026-04-20", "bytes": 1048576 },
    { "date": "2026-04-21", "bytes": 0 },
    { "date": "2026-04-22", "bytes": 524288 },
    { "date": "2026-04-23", "bytes": 0 },
    { "date": "2026-04-24", "bytes": 0 },
    { "date": "2026-04-25", "bytes": 215896 }
  ],
  "last7DaysTotalBytes":  1788760
}
```

* `quotaBytes` is `quotaMb * 1024 * 1024`. Daily limit before `429 quota_exceeded`.
* `today.bytes` is uploaded bytes since 00:00 UTC.
* `last7Days` is in calendar UTC days, oldest first, 7 entries always present (zero-padded).

## Use cases

### Render a quota bar

```ts
const u = await fetch('/api/usage', {
  headers: { authorization: `Bearer ${token}` }
}).then(r => r.json())

const pct = (u.today.bytes / u.quotaBytes) * 100
// render: ████████░░░░░░░░░░░░  43%
```

### Pre-flight before uploading

```ts
const u = await getUsage()
if (u.today.bytes + file.size > u.quotaBytes) {
  alert('This upload would exceed your daily quota.')
  return
}
```

## Errors

| Status | `error` | When |
| --- | --- | --- |
| `401` | `no_token` / `invalid_token` / `revoked_token` | Auth failed |

## Notes

* Usage is tracked per **userId** (a hash of email), not per token. Multiple tokens for the same user share the quota.
* Quota resets at 00:00 UTC. We don't pro-rate.
* Past-day history is best-effort — stored in KV with a 7-day TTL, so older days expire.
