# Anvil Soak Test

Integration scenario that exercises the full prover loop against a local anvil, with simulated client activity. Run manually; not part of `go test ./...` since it spawns external processes (anvil, a source HTTP server).

## What it does

1. Starts anvil on port 8545 (chain-id 31337)
2. Deploys the full Prova contract set + MockProofVerifier via `forge script`
3. Registers a prover and stakes 500 PROVA
4. Starts a source HTTP server on port 8900 serving deterministic piece data
5. Proposes 3 deals from 3 different client addresses
6. Starts `provad` with HTTP + metrics + on-chain accepter wired
7. Waits for all 3 deals to reach Active status
8. Retrieves pieces back over `provad`'s HTTPS endpoint
9. Scrapes Prometheus metrics and asserts expected values
10. SIGTERMs the daemon, verifies clean shutdown

## Running

```
cd prover/internal/soaktest
./run.sh
```

Requires: `anvil`, `cast`, `forge`, `go`, `python3`, `curl`. Assumes the binary at `/tmp/provad` is up to date (build via `go build -o /tmp/provad ./cmd/provad` first).

## Exit codes

- `0` — all checks passed
- non-zero — a check failed; log tails printed to stderr

## Runtime

Roughly 30-60 seconds. Mostly spent waiting for tick intervals (2s poll + 2s tick).
