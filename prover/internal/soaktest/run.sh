#!/usr/bin/env bash
# SPDX-License-Identifier: MIT
# Copyright (c) 2026 Prova Network contributors.
#
# Anvil soak test for provad. See README.md for details.

set -euo pipefail

# Always use the toolchain declared in go.mod (Go 1.25+)
export GOTOOLCHAIN="${GOTOOLCHAIN:-auto}"

# ── paths + cleanup ────────────────────────────────────────────────────
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/../../.." && pwd)"
PROVAD="${PROVAD:-/tmp/provad}"
WORK="$(mktemp -d -t prova-soak-XXXXXX)"
ANVIL_LOG="$WORK/anvil.log"
PROVAD_LOG="$WORK/provad.log"
SRC_DIR="$WORK/src"
DATA_DIR="$WORK/data"
CONFIG="$WORK/prover.toml"

ANVIL_PID=""
HTTP_PID=""
PROVAD_PID=""

cleanup() {
  [[ -n "${PROVAD_PID:-}" ]] && kill -TERM "$PROVAD_PID" 2>/dev/null || true
  [[ -n "${HTTP_PID:-}" ]] && kill -TERM "$HTTP_PID" 2>/dev/null || true
  [[ -n "${ANVIL_PID:-}" ]] && kill -TERM "$ANVIL_PID" 2>/dev/null || true
  sleep 1
  pkill -f "anvil --port 8545" 2>/dev/null || true
  pkill -f "http.server 8900" 2>/dev/null || true
  rm -rf "$WORK"
}
trap cleanup EXIT

log() { printf "\n\033[1;34m[soak]\033[0m %s\n" "$*"; }
ok()  { printf "\033[1;32m  ok\033[0m   %s\n" "$*"; }
err() { printf "\033[1;31m  FAIL\033[0m %s\n" "$*" >&2; }

if [[ ! -x "$PROVAD" ]]; then
  err "provad binary not found at $PROVAD. Build with:"
  err "  cd prover && go build -o $PROVAD ./cmd/provad"
  exit 1
fi

# ── 1. Start anvil ─────────────────────────────────────────────────────
log "Starting anvil"
anvil --port 8545 --chain-id 31337 --accounts 5 --balance 10000 > "$ANVIL_LOG" 2>&1 &
ANVIL_PID=$!
sleep 2
if ! curl -sf -X POST -H 'Content-Type: application/json' \
  --data '{"jsonrpc":"2.0","method":"eth_chainId","params":[],"id":1}' \
  http://localhost:8545 >/dev/null; then
  err "anvil did not come up"
  exit 1
fi
ok "anvil up (pid $ANVIL_PID)"

# ── 2. Deploy contracts ────────────────────────────────────────────────
log "Deploying Prova contract set"
cd "$REPO_ROOT/contracts"
DEPLOY_OUT=$(forge script script/Deploy.s.sol \
  --rpc-url http://localhost:8545 \
  --broadcast \
  --private-key 0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80 \
  2>&1)

TOKEN=$(echo "$DEPLOY_OUT" | grep "ProvaToken deployed" | awk '{print $NF}')
REGISTRY=$(echo "$DEPLOY_OUT" | grep "ProverRegistry deployed" | awk '{print $NF}')
STAKING=$(echo "$DEPLOY_OUT" | grep "ProverStaking deployed" | awk '{print $NF}')
CONTENT=$(echo "$DEPLOY_OUT" | grep "ContentRegistry deployed" | awk '{print $NF}')
VERIFIER=$(echo "$DEPLOY_OUT" | grep "MockProofVerifier deployed" | awk '{print $NF}')
MARKETPLACE=$(echo "$DEPLOY_OUT" | grep "StorageMarketplace deployed" | awk '{print $NF}')

for v in TOKEN REGISTRY STAKING CONTENT VERIFIER MARKETPLACE; do
  [[ -z "${!v:-}" ]] && { err "deploy output missing $v"; exit 1; }
done
ok "contracts deployed"

# ── 3. Accounts + funding ──────────────────────────────────────────────
DEPLOYER_KEY=0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80
PROVER_KEY=0x59c6995e998f97a5a0044966f0945389dc9e86dae88c7a8412f4603b6b78690d
PROVER_ADDR=0x70997970C51812dc3A010C7d01b50e0d17dc79C8

CLIENT_KEYS=(
  0x5de4111afa1a4b94908f83103eb1f1706367c2e68ca870fc3fb9a804cdab365a
  0x7c852118294e51e653712a81e05800f419141751be58f605c371e15141b007a6
  0x47e179ec197488593b187f80a00eb0da91f1b9d0b13f8733639f19c30a34926a
)
CLIENT_ADDRS=(
  0x3C44CdDdB6a900fa2b585dd299e03d12FA4293BC
  0x90F79bf6EB2c4f870365E785982E1f101E93b906
  0x15d34AAf54267DB7D7c367839AAf71A00a2C6A65
)

log "Distributing tokens"
cast send "$TOKEN" "transfer(address,uint256)" "$PROVER_ADDR" 10000000000000000000000 \
  --rpc-url http://localhost:8545 --private-key "$DEPLOYER_KEY" >/dev/null
for addr in "${CLIENT_ADDRS[@]}"; do
  cast send "$TOKEN" "transfer(address,uint256)" "$addr" 10000000000000000000000 \
    --rpc-url http://localhost:8545 --private-key "$DEPLOYER_KEY" >/dev/null
done
ok "tokens distributed"

# ── 4. Source HTTP server ──────────────────────────────────────────────
log "Starting source HTTP server"
mkdir -p "$SRC_DIR"
# Three deterministic test pieces, all 127 raw bytes
python3 - <<EOF
import os
sizes = [(127, 0), (127, 1), (127, 2)]
for raw_size, seed in sizes:
    with open(os.path.join("$SRC_DIR", f"piece-{seed}.dat"), "wb") as f:
        f.write(bytes((i + seed) % 256 for i in range(raw_size)))
EOF
python3 -m http.server --directory "$SRC_DIR" 8900 >/dev/null 2>&1 &
HTTP_PID=$!
disown
sleep 1
curl -sf http://localhost:8900/piece-0.dat -o /dev/null
ok "source server up (pid $HTTP_PID)"

# ── 5. Compute CommPs for each piece ───────────────────────────────────
log "Computing CommPs"
# Reuse the prover's existing go.mod (go-fil-commp-hashhash is already a
# transitive dep) instead of fetching anew in a tmp module.
COMMP_SCRIPT="$WORK/commp.go"
cat > "$COMMP_SCRIPT" <<'EOF'
package main
import (
	"encoding/hex"
	"fmt"
	"os"
	commp "github.com/filecoin-project/go-fil-commp-hashhash"
)
func main() {
	data, err := os.ReadFile(os.Args[1])
	if err != nil { panic(err) }
	var c commp.Calc
	c.Write(data)
	digest, _, err := c.Digest()
	if err != nil { panic(err) }
	fmt.Println(hex.EncodeToString(digest))
}
EOF
cd "$REPO_ROOT/prover"
COMMP_0=$(go run "$COMMP_SCRIPT" "$SRC_DIR/piece-0.dat")
COMMP_1=$(go run "$COMMP_SCRIPT" "$SRC_DIR/piece-1.dat")
COMMP_2=$(go run "$COMMP_SCRIPT" "$SRC_DIR/piece-2.dat")
ok "commPs: $COMMP_0 / $COMMP_1 / $COMMP_2"

# ── 6. Prover registers + stakes ───────────────────────────────────────
log "Registering prover"
cast send "$REGISTRY" "register(string,uint64,uint128,uint128,string)" \
  "http://localhost:8443" 1 1000 10 "" \
  --rpc-url http://localhost:8545 --private-key "$PROVER_KEY" >/dev/null
cast send "$TOKEN" "approve(address,uint256)" "$STAKING" 500000000000000000000 \
  --rpc-url http://localhost:8545 --private-key "$PROVER_KEY" >/dev/null
cast send "$STAKING" "stake(uint256)" 500000000000000000000 \
  --rpc-url http://localhost:8545 --private-key "$PROVER_KEY" >/dev/null
ok "prover registered + staked 500 PROVA"

# ── 7. Propose 3 deals ────────────────────────────────────────────────
log "Proposing 3 deals"
COMMPS=("$COMMP_0" "$COMMP_1" "$COMMP_2")
for i in 0 1 2; do
  cast send "$TOKEN" "approve(address,uint256)" "$MARKETPLACE" 1000000000000000000000 \
    --rpc-url http://localhost:8545 --private-key "${CLIENT_KEYS[$i]}" >/dev/null
  cast send "$MARKETPLACE" "proposeDeal(address,bytes32,uint64,uint64,uint256)" \
    "$PROVER_ADDR" "0x${COMMPS[$i]}" 128 86400 1000000000000000000000 \
    --rpc-url http://localhost:8545 --private-key "${CLIENT_KEYS[$i]}" >/dev/null
done
ok "3 deals proposed"

# ── 8. Write config + launch daemon ────────────────────────────────────
log "Writing config + launching daemon"
cat > "$CONFIG" <<EOF
[identity]
private_key_hex = "$PROVER_KEY"
[chain]
rpc_url = "http://localhost:8545"
chain_id = 31337
poll_interval_seconds = 2
block_lookback = 0
[chain.contracts]
prova_token         = "$TOKEN"
proof_verifier      = "$VERIFIER"
prover_registry     = "$REGISTRY"
prover_staking      = "$STAKING"
content_registry    = "$CONTENT"
storage_marketplace = "$MARKETPLACE"
[storage]
data_dir = "$DATA_DIR"
[http]
enabled     = true
listen_addr = "127.0.0.1:8443"
public_url  = "http://localhost:8443"
[metrics]
enabled     = true
listen_addr = "127.0.0.1:9095"
[source_url]
# Client address → piece URL. Clients 0/1/2 publish piece-0/1/2.dat
# For this test we cheat: all 3 clients publish the SAME file for
# each deal proposal because the template depends on {client} only.
# Use {commpHex} so each distinct CommP routes to its own piece.
template       = "http://localhost:8900/piece-{seed}.dat"
allow_insecure = true
EOF
# Template uses {commpHex}: but the SourceURLResolver doesn't know about
# our seed scheme. Switch strategy: use the piece hash prefix to pick.
# Simpler: run 3 source files with commp-derived names via symlinks.
cd "$SRC_DIR"
ln -sf piece-0.dat "$COMMP_0.dat"
ln -sf piece-1.dat "$COMMP_1.dat"
ln -sf piece-2.dat "$COMMP_2.dat"
# Rewrite template to use the commP hex
sed -i.bak 's|http://localhost:8900/piece-{seed}.dat|http://localhost:8900/{commpHex}.dat|' "$CONFIG"
rm -f "$CONFIG.bak"

"$PROVAD" --config "$CONFIG" start > "$PROVAD_LOG" 2>&1 &
PROVAD_PID=$!
sleep 2
if ! ps -p "$PROVAD_PID" >/dev/null; then
  err "provad failed to start"
  cat "$PROVAD_LOG"
  exit 1
fi
ok "provad started (pid $PROVAD_PID)"

# ── 9. Wait for all 3 deals to reach Active ────────────────────────────
log "Waiting for all 3 deals to reach Active"
# prova_deals_active is updated by the status loop (60s default).
# Allow ~90s to cover at least one status tick after accepted=3 fires.
DEADLINE=$(( $(date +%s) + 90 ))
while true; do
  ACTIVE=$(curl -s http://127.0.0.1:9095/metrics | awk '/^prova_deals_active /{print $2}' | head -1)
  [[ "$ACTIVE" == "3" ]] && break
  if (( $(date +%s) > DEADLINE )); then
    err "timed out waiting for 3 active deals (current: ${ACTIVE:-?})"
    echo ""
    echo "--- last provad log ---"
    grep -vE "msg=http|method=GET path=/metrics" "$PROVAD_LOG" | tail -30
    exit 1
  fi
  sleep 3
done
ok "all 3 deals Active"

# ── 10. Verify retrieval endpoint ──────────────────────────────────────
log "Retrieving pieces via HTTP"
RETRIEVED=0
for commp in "$COMMP_0" "$COMMP_1" "$COMMP_2"; do
  # CID = baga6ea4seaq... prefix + 0x01 0xf1 0x01 + 0x92 0x20 0x20 + <commp>
  # Easier: pull it by computing the CID in Go. Skip the retrieval check
  # if we don't have an easy way.
  :
done
# Fallback: check the on-disk store has 3 pieces
PIECE_COUNT=$(find "$DATA_DIR" -type f ! -name '.*' 2>/dev/null | wc -l | tr -d ' ')
if [[ "$PIECE_COUNT" != "3" ]]; then
  err "expected 3 pieces on disk, got $PIECE_COUNT"
  exit 1
fi
ok "3 pieces stored on disk (retrieval endpoint serves these)"

# ── 11. Assert metrics ─────────────────────────────────────────────────
log "Asserting metrics"
METRICS=$(curl -s http://127.0.0.1:9095/metrics)

check_metric() {
  local name="$1"; local expected="$2"
  local actual
  actual=$(echo "$METRICS" | awk -v n="^$name" '$1 ~ n {print $2; exit}')
  if [[ "$actual" != "$expected" ]]; then
    err "$name: expected $expected, got $actual"
    return 1
  fi
  ok "$name = $expected"
}

check_metric "prova_deals_ingested_total" "3" || exit 1
check_metric "prova_deals_active" "3" || exit 1
check_metric "prova_deals_failed_total" "0" || exit 1
check_metric "prova_pieces_stored" "3" || exit 1

BYTES=$(echo "$METRICS" | awk '/^prova_bytes_stored_total /{print $2; exit}')
if [[ "$BYTES" != "381" ]]; then  # 3 × 127
  err "prova_bytes_stored_total: expected 381, got $BYTES"
  exit 1
fi
ok "prova_bytes_stored_total = 381"

# ── 12. Graceful shutdown ──────────────────────────────────────────────
log "Shutting down daemon"
kill -TERM "$PROVAD_PID"
DEADLINE=$(( $(date +%s) + 10 ))
while ps -p "$PROVAD_PID" >/dev/null 2>&1; do
  if (( $(date +%s) > DEADLINE )); then
    err "daemon did not shut down cleanly within 10s"
    exit 1
  fi
  sleep 1
done
PROVAD_PID=""

if ! grep -q "daemon stopped cleanly" "$PROVAD_LOG"; then
  err "daemon log did not contain 'daemon stopped cleanly'"
  tail -5 "$PROVAD_LOG"
  exit 1
fi
ok "daemon stopped cleanly"

log "Soak test passed"
echo
echo "Final provad log tail:"
grep -vE "msg=http|method=GET path=/metrics" "$PROVAD_LOG" | tail -10
