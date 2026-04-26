---
description: Show your identity, quota, and recent usage.
---

# prova whoami

Prints your signed-in identity, today's quota usage with a progress bar, and a 7-day total.

```bash
$ prova whoami
nicklas@example.com
  user-id : 6a2e8fe9beee83ef

  today   : 215.9 KB / 1.00 GB
            █░░░░░░░░░░░░░░░░░░░░░░░░░░░░░ 0.0%

  last 7d : 1.7 MB
```

## Usage

```
prova whoami
```

## Auth

Requires a signed-in session. Errors with a friendly "run `prova auth`" message if you're not.

## See also

* [`GET /api/usage`](../api/usage.md)
