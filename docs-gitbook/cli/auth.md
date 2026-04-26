---
description: Sign in with your email.
---

# prova auth

Signs you in by minting an API token tied to an email address. The token is saved to `~/.prova/config.json` (mode 600) and used by every subsequent CLI command.

```bash
$ prova auth
Email: nicklas@example.com

✓ Signed in as nicklas@example.com
  user-id : 6a2e8fe9beee83ef
  quota   : 1.00 GB / day
  expires : 2027-04-25 14:06:05

Token saved to ~/.prova/config.json
```

## Usage

```
prova auth
```

No flags. Prompts for your email interactively.

## What it does

1. Asks for your email
2. Calls `POST /api/auth/signup` with `label = cli@<hostname>`
3. Stores the returned token in `~/.prova/config.json` with permissions `0600`
4. Prints your user id, quota, and token expiry

The label `cli@<hostname>` is what appears in the dashboard's token list, so you know which device a token came from.

## Already signed in?

If `~/.prova/config.json` exists, `prova auth` declines and tells you to `prova logout` first. This prevents accidentally overwriting a working session.

## Pre-mainnet

Today's signup is **email-only, no verification**. After mainnet launch, this command sends a magic link to your inbox and you click it to confirm before the token is issued. The CLI flow stays the same — the `prova auth` command waits for the link to be clicked, then writes the token.

## Environment variables

| Variable | Effect |
| --- | --- |
| `PROVA_API` | Override the API base URL (default `https://prova.network`). |

## See also

* [`prova whoami`](whoami.md)
* [`prova logout`](logout.md)
* [`POST /api/auth/signup`](../api/auth-signup.md)
