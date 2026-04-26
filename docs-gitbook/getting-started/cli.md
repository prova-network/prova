---
description: prova put ./dist - the one-line CLI.
---

# CLI

Prova ships a single-binary CLI. Zero dependencies, ESM Node, ~10 KB compiled.

## Install

```bash
npm i -g @prova-network/cli
```

Or on Linux / macOS, the one-liner installer:

```bash
curl -fsSL https://get.prova.network | sh
```

That drops `prova` in `~/.prova/bin` and adds it to your PATH.

## First run

```bash
prova auth
```

Asks for your email, mints an API token tied to it, saves to `~/.prova/config.json` (`mode 600`). The token gets a default 1 GiB/day quota and lasts 1 year.

```bash
$ prova auth
Email: nicklas@example.com

✓ Signed in as nicklas@example.com
  user-id : 6a2e8fe9beee83ef
  quota   : 1.00 GB / day
  expires : 2027-04-25 14:06:05
```

## Upload a file

```bash
prova put ./my-site.tar.gz
```

```
$ prova put ./my-site.tar.gz
hashing  my-site.tar.gz (4.2 MB)…       ✓ baga…q4kr
staging  bytes for prover…              ✓ 712 ms

Stored.
  piece-cid : baga…q4kr
  deal-id   : d-0x9c4f
  size      : 4.2 MB
  paid      : free quota
  term      : 30 days
  retrieve  : https://prova.network/p/baga…q4kr

Verify with: curl -O https://prova.network/p/baga…q4kr
```

## Other commands

```bash
prova ls                       # list your files
prova get <cid>                # stream a file to stdout
prova get <cid> -o ./out.bin   # ... or write to a file
prova whoami                   # show identity + quota usage
prova logout                   # forget local creds
prova help                     # full help
```

## Environment variables

| Variable | Purpose |
| --- | --- |
| `PROVA_API` | Override the API base URL (default `https://prova.network`). |
| `PROVA_TOKEN` | Override the token from `~/.prova/config.json`. Useful in CI. |
| `PROVA_DEBUG` | Print full stack traces on error. |

```bash
# CI-friendly invocation
PROVA_TOKEN="$PROVA_TOKEN" prova put ./build/site.tar.gz
```

## Use it from a script

The CLI returns clean JSON when stdout isn't a TTY (coming soon). For now, scrape the output or use the [HTTP API](../api/) directly.

## Source

The CLI is open source: [github.com/prova-network/prova/tree/main/cli](https://github.com/prova-network/prova/tree/main/cli).
Apache-2.0 OR MIT.
