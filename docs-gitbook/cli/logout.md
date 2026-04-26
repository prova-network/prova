---
description: Forget local credentials.
---

# prova logout

Deletes `~/.prova/config.json`. Your tokens stay valid on the server until they expire — `logout` is local-only.

```bash
$ prova logout
✓ Signed out.
```

To revoke a token server-side (so it can never be used again), see [`POST /api/tokens/revoke`](../api/tokens-revoke.md) or use the [dashboard](https://prova.network/app/).

## Usage

```
prova logout
```

No flags.

## See also

* [`POST /api/tokens/revoke`](../api/tokens-revoke.md)
* [`prova auth`](auth.md)
