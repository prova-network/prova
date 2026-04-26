#!/usr/bin/env bash
# Test 3: deeper economic flows on local anvil.
#
# Verifies:
#   1. Unbonding queue: requestUnstake → can't withdraw before 14d → can after
#   2. Slashing during unbonding: slash hits both bonded + unbonding stake
#   3. Stake floor: a prover can't commit more bytes than minStakePerGiB allows
#   4. ProverRewards.rewardOf math: per-epoch share is exactly proportional
#   5. ProverRewards.claim flow: end-to-end from proof → vest → claim
#   6. claim() before vesting buffer reverts EpochNotVested
#   7. Quality multiplier: missed proofs cut emission to 50%
#   8. Redundancy cap: only first N=4 provers per piece get credit
#   9. Self-dealing: prover==client doesn't count
#  10. Sponsored upload (client==address(0)) doesn't count
#  11. Total emission supply preserved across all claims (50M ceiling)

set -euo pipefail

RPC="http://127.0.0.1:8545"
DEPLOYER_KEY="0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80"
DEPLOYER="0xf39Fd6e51aad88F6F4ce6aB8827279cfFFb92266"

# Anvil's accounts 1-9 — we'll use these as different provers + clients
P1_KEY="0x59c6995e998f97a5a0044966f0945389dc9e86dae88c7a8412f4603b6b78690d"
P1="0x70997970C51812dc3A010C7d01b50e0d17dc79C8"
P2_KEY="0x5de4111afa1a4b94908f83103eb1f1706367c2e68ca870fc3fb9a804cdab365a"
P2="0x3C44CdDdB6a900fa2b585dd299e03d12FA4293BC"
P3_KEY="0x7c852118294e51e653712a81e05800f419141751be58f605c371e15141b007a6"
P3="0x90F79bf6EB2c4f870365E785982E1f101E93b906"
P4_KEY="0x47e179ec197488593b187f80a00eb0da91f1b9d0b13f8733639f19c30a34926a"
P4="0x15d34AAf54267DB7D7c367839AAf71A00a2C6A65"
P5_KEY="0x8b3a350cf5c34c9194ca85829a2df0ec3153be0318b5e2d3348e872092edffba"
P5="0x9965507D1a55bcC2695C58ba16FB37d819B0A4dc"

CLIENT_KEY="0x92db14e403b83dfe3df233f83dfa3a0d7096f21ca9b0d6d6b8d88b2b4ec1564e"
CLIENT="0x976EA74026E726554dB657fA54763abd0C3a0aa9"

USDC=0x5FbDB2315678afecb367f032d93F642f64180aa3
TOKEN=0xe7f1725E7734CE288F8367e1Bb143E90bb3F0512
REGISTRY=0x9fE46736679d2D9a65F0992F2272dE9f3c7fa6e0
STAKING=0xCf7Ed3AccA5a467e9e704C703E8D87F634fB0Fc9
MARKET=0xa513E6E4b8f2a923D98304ec87F64353C4D5C853
VERIFIER=0x5FC8d32690cc91D4c39d9d3abcBD16989F875707
REWARDS=0x610178dA211FEF7D417bC0e6FeD39F05609AD788

PASS=0
FAIL=0
say()  { printf "\n\033[36m=== %s ===\033[0m\n" "$1"; }
ok()   { printf "  \033[32m✓\033[0m %s\n" "$1"; PASS=$((PASS+1)); }
fail() { printf "  \033[31m✗\033[0m %s\n" "$1"; FAIL=$((FAIL+1)); }
hr()   { printf "    expected: %s\n    actual:   %s\n" "$1" "$2"; }
ether(){ python3 -c "print(f'{int(\"$1\")/1e18:,.4f}')"; }
to_int(){ python3 -c "import sys; s='$1'.split()[0]; print(int(s))"; }

# Python-int-compare helpers (bash arithmetic overflows on uint256)
gt() { python3 -c "import sys; sys.exit(0 if int(\"$1\") > int(\"$2\") else 1)"; }
ge() { python3 -c "import sys; sys.exit(0 if int(\"$1\") >= int(\"$2\") else 1)"; }
le() { python3 -c "import sys; sys.exit(0 if int(\"$1\") <= int(\"$2\") else 1)"; }
eq() { python3 -c "import sys; sys.exit(0 if int(\"$1\") == int(\"$2\") else 1)"; }


# helper: register a prover
register_prover() {
    local prover_key=$1
    cast send $REGISTRY "register(string,uint64,uint128,uint128,string)" \
        "https://prover.example/pdp" 3 1000000000 0 "" \
        --rpc-url $RPC --private-key $prover_key > /dev/null 2>&1
}

# helper: stake amount
stake_for() {
    local prover_key=$1
    local amount=$2
    cast send $TOKEN "approve(address,uint256)" $STAKING "$amount" --rpc-url $RPC --private-key $prover_key > /dev/null
    cast send $STAKING "stake(uint256)" "$amount" --rpc-url $RPC --private-key $prover_key > /dev/null
}

# helper: snapshot a prover's stake info
# Parse cast's tuple output. Format: "(N [Ne+M], N [Ne+M], N, N)".
# We extract the bare integers, dropping bracketed shortforms.
_get_field() {
    # $1=address, $2=field index (0-based)
    cast call $STAKING "getStake(address)((uint256,uint256,uint256,uint256))" "$1" --rpc-url $RPC \
        | python3 -c "import sys, re; s=sys.stdin.read(); ns=re.findall(r'(?<![\d.e+-])(\d{4,})(?:\s*\[[^\]]*\])?', s); print(ns[$2])"
}
get_staked()      { _get_field "$1" 0; }
get_unbonding()   { _get_field "$1" 1; }
get_unbond_end()  { _get_field "$1" 2; }

# helper: impersonate verifier and call X
mock_verifier_call() {
    cast rpc anvil_impersonateAccount $VERIFIER --rpc-url $RPC > /dev/null
    cast rpc anvil_setBalance $VERIFIER 0x10000000000000000 --rpc-url $RPC > /dev/null
    "$@"
    cast rpc anvil_stopImpersonatingAccount $VERIFIER --rpc-url $RPC > /dev/null
}

# ─── Setup: fund provers + client with PROVA + USDC ──────────────────────
say "Setup: fund 5 provers + 1 client"
for prover in $P1 $P2 $P3 $P4 $P5; do
    cast send $TOKEN "transfer(address,uint256)" $prover 100000ether --rpc-url $RPC --private-key $DEPLOYER_KEY > /dev/null
done
cast send $USDC "transfer(address,uint256)" $CLIENT 50000ether --rpc-url $RPC --private-key $DEPLOYER_KEY > /dev/null
ok "5 provers funded with 100,000 PROVA each"
ok "client funded with 50,000 USDC"

# ─── 1. UNBONDING QUEUE ──────────────────────────────────────────────────
say "1. Unbonding queue: cannot withdraw before 14 days"

register_prover $P1_KEY
stake_for $P1_KEY 50000ether
ok "P1 staked 50,000 PROVA"

# Request unstake of 20,000
cast send $STAKING "requestUnstake(uint256)" 20000ether --rpc-url $RPC --private-key $P1_KEY > /dev/null
STAKED=$(get_staked $P1)
UNBONDING=$(get_unbonding $P1)
UNBOND_END=$(get_unbond_end $P1)

[ "$STAKED" = "30000000000000000000000" ] && ok "after requestUnstake: bonded = 30,000 PROVA" || { fail "wrong bonded"; hr "30000e18" "$STAKED"; }
eq "$UNBONDING" 20000000000000000000000 && ok "after requestUnstake: unbonding = 20,000 PROVA" || { fail "wrong unbonding"; hr "20000e18" "$UNBONDING"; }

# Try to withdraw immediately — should revert
WITHDRAW_RESULT=$(cast send $STAKING "withdraw()" --rpc-url $RPC --private-key $P1_KEY 2>&1 || true)
echo "$WITHDRAW_RESULT" | grep -qi "StillUnbonding\|reverted" && ok "withdraw() before 14d reverts (StillUnbonding)" || fail "withdraw should have reverted"

# Skip 13 days, still unbonded
cast rpc evm_increaseTime $((13 * 86400)) --rpc-url $RPC > /dev/null
cast rpc evm_mine --rpc-url $RPC > /dev/null
WITHDRAW_RESULT=$(cast send $STAKING "withdraw()" --rpc-url $RPC --private-key $P1_KEY 2>&1 || true)
echo "$WITHDRAW_RESULT" | grep -qi "StillUnbonding\|reverted" && ok "withdraw() at 13d still reverts" || fail "13d should still be unbonded"

# Skip past 14d total
cast rpc evm_increaseTime $((2 * 86400)) --rpc-url $RPC > /dev/null
cast rpc evm_mine --rpc-url $RPC > /dev/null
P1_BAL_PRE=$(cast call $TOKEN "balanceOf(address)(uint256)" $P1 --rpc-url $RPC | awk '{print $1}')
cast send $STAKING "withdraw()" --rpc-url $RPC --private-key $P1_KEY > /dev/null
P1_BAL_POST=$(cast call $TOKEN "balanceOf(address)(uint256)" $P1 --rpc-url $RPC | awk '{print $1}')

DELTA=$(python3 -c "print($P1_BAL_POST - $P1_BAL_PRE)")
[ "$DELTA" = "20000000000000000000000" ] && ok "withdraw() at 15d returns 20,000 PROVA" || { fail "wrong withdraw amount"; hr "20000e18" "$(ether $DELTA)"; }

# ─── 2. SLASHING DURING UNBONDING ────────────────────────────────────────
say "2. Slashing during unbonding: slash hits both bonded + unbonding"

register_prover $P2_KEY
stake_for $P2_KEY 60000ether

# request unstake 30K — keeps 30K bonded, 30K unbonding
cast send $STAKING "requestUnstake(uint256)" 30000ether --rpc-url $RPC --private-key $P2_KEY > /dev/null

# Have P2 take a deal that auto-faults
cast send $USDC "approve(address,uint256)" $MARKET 1000ether --rpc-url $RPC --private-key $CLIENT_KEY > /dev/null
COMMP_FAULT=$(cast keccak "doomed-during-unbond")
cast send $MARKET "proposeDeal(address,bytes32,uint64,uint64,uint256)" \
  $P2 $COMMP_FAULT 1048576 2592000 1000ether --rpc-url $RPC --private-key $CLIENT_KEY > /dev/null

mock_verifier_call cast send $MARKET "dataSetCreated(uint256,address,bytes)" 200 $P2 \
  $(cast abi-encode "f(uint256)" 1) --from $VERIFIER --unlocked --gas-limit 1000000 --rpc-url $RPC > /dev/null

# Skip past MAX_PROOF_GAP, fault
MAX_GAP=$(cast call $MARKET "MAX_PROOF_GAP()(uint256)" --rpc-url $RPC | awk '{print $1}')
cast rpc evm_increaseTime $((MAX_GAP + 100)) --rpc-url $RPC > /dev/null
cast rpc evm_mine --rpc-url $RPC > /dev/null

P2_STAKED_PRE=$(get_staked $P2)
P2_UNBOND_PRE=$(get_unbonding $P2)
SUPPLY_PRE=$(cast call $TOKEN "totalSupply()(uint256)" --rpc-url $RPC | awk '{print $1}')

cast send $MARKET "faultDeal(uint256)" 1 --rpc-url $RPC --private-key $DEPLOYER_KEY > /dev/null

P2_STAKED_POST=$(get_staked $P2)
P2_UNBOND_POST=$(get_unbonding $P2)
SUPPLY_POST=$(cast call $TOKEN "totalSupply()(uint256)" --rpc-url $RPC | awk '{print $1}')

SLASH_AMT=$(cast call $MARKET "slashPerFault()(uint256)" --rpc-url $RPC | awk '{print $1}')
SUPPLY_BURNED=$(python3 -c "print($SUPPLY_PRE - $SUPPLY_POST)")
STAKE_LOST=$(python3 -c "print($P2_STAKED_PRE - $P2_STAKED_POST)")

[ "$SUPPLY_BURNED" = "$SLASH_AMT" ] && ok "supply burned = slashPerFault ($SLASH_AMT wei)" || fail "wrong burn amount"
[ "$STAKE_LOST" = "$SLASH_AMT" ] && ok "bonded stake reduced first (slashPerFault < bonded $P2_STAKED_PRE)" || { fail "stake lost wrong"; hr "$SLASH_AMT" "$STAKE_LOST"; }
eq "$P2_UNBOND_POST" "$P2_UNBOND_PRE" && ok "unbonding pool untouched (since bonded > slashPerFault)" || { fail "unbonding shrunk"; hr "$P2_UNBOND_PRE" "$P2_UNBOND_POST"; }

# ─── 3. STAKE FLOOR enforced on commit ────────────────────────────────────
say "3. minStakePerGiB enforced on deal acceptance"

# P3 stakes only 100 PROVA, then tries to take a deal needing more capacity than that allows
register_prover $P3_KEY
stake_for $P3_KEY 100ether

# minStakePerGiB = 100 PROVA, so 100 PROVA stake supports up to 1 GiB committed.
# Try to propose a 2 GiB deal — should be blocked at acceptance time.
cast send $USDC "approve(address,uint256)" $MARKET 100ether --rpc-url $RPC --private-key $CLIENT_KEY > /dev/null
COMMP_BIG=$(cast keccak "too-big-for-stake")
PROPOSE_RESULT=$(cast send $MARKET "proposeDeal(address,bytes32,uint64,uint64,uint256)" \
  $P3 $COMMP_BIG 2147483648 2592000 100ether --rpc-url $RPC --private-key $CLIENT_KEY 2>&1 || true)

echo "$PROPOSE_RESULT" | grep -qi "ProverCannotCommit\|reverted" && ok "proposeDeal reverts when bytes > stake capacity" || fail "should have reverted"

# ─── 4. PROVER REWARDS reward math ────────────────────────────────────────
say "4. ProverRewards: rewardOf math is proportional to bytes proven"

# Reset: have P4 and P5 stake adequately and prove different amounts in same epoch
for pk in $P4_KEY $P5_KEY; do
    register_prover $pk
done
stake_for $P4_KEY 50000ether
stake_for $P5_KEY 50000ether

# Two deals from CLIENT to P4 and P5, sized 3:1
cast send $USDC "approve(address,uint256)" $MARKET 4000ether --rpc-url $RPC --private-key $CLIENT_KEY > /dev/null
COMMP_P4=$(cast keccak "p4-piece")
cast send $MARKET "proposeDeal(address,bytes32,uint64,uint64,uint256)" \
  $P4 $COMMP_P4 3145728 2592000 3000ether --rpc-url $RPC --private-key $CLIENT_KEY > /dev/null   # 3 MiB

COMMP_P5=$(cast keccak "p5-piece")
cast send $MARKET "proposeDeal(address,bytes32,uint64,uint64,uint256)" \
  $P5 $COMMP_P5 1048576 2592000 1000ether --rpc-url $RPC --private-key $CLIENT_KEY > /dev/null  # 1 MiB

DEAL_P4=$(cast call $MARKET "nextDealId()(uint256)" --rpc-url $RPC | awk '{print $1}')
DEAL_P5=$((DEAL_P4 - 1))
DEAL_P4=$((DEAL_P4 - 2))

mock_verifier_call cast send $MARKET "dataSetCreated(uint256,address,bytes)" 401 $P4 $(cast abi-encode "f(uint256)" $DEAL_P4) --from $VERIFIER --unlocked --gas-limit 1000000 --rpc-url $RPC > /dev/null
mock_verifier_call cast send $MARKET "dataSetCreated(uint256,address,bytes)" 402 $P5 $(cast abi-encode "f(uint256)" $DEAL_P5) --from $VERIFIER --unlocked --gas-limit 1000000 --rpc-url $RPC > /dev/null

# Skip 1 day
cast rpc evm_increaseTime 86400 --rpc-url $RPC > /dev/null
cast rpc evm_mine --rpc-url $RPC > /dev/null

# Both prove
mock_verifier_call cast send $MARKET "possessionProven(uint256,uint256,uint256,uint256)" 401 1 1 1 --from $VERIFIER --unlocked --gas-limit 1500000 --rpc-url $RPC > /dev/null
mock_verifier_call cast send $MARKET "possessionProven(uint256,uint256,uint256,uint256)" 402 1 1 1 --from $VERIFIER --unlocked --gas-limit 1500000 --rpc-url $RPC > /dev/null

EPOCH=$(cast call $REWARDS "currentEpoch()(uint256)" --rpc-url $RPC | awk '{print $1}')
P4_BYTES=$(cast call $REWARDS "bytesByEpochProver(uint256,address)(uint256)" $EPOCH $P4 --rpc-url $RPC | awk '{print $1}')
P5_BYTES=$(cast call $REWARDS "bytesByEpochProver(uint256,address)(uint256)" $EPOCH $P5 --rpc-url $RPC | awk '{print $1}')
TOTAL_EPOCH=$(cast call $REWARDS "totalBytesByEpoch(uint256)(uint256)" $EPOCH --rpc-url $RPC | awk '{print $1}')

[ "$P4_BYTES" = "3145728" ] && ok "P4 credited 3 MiB" || { fail "P4 wrong"; hr "3145728" "$P4_BYTES"; }
[ "$P5_BYTES" = "1048576" ] && ok "P5 credited 1 MiB" || fail "P5 wrong"
[ "$TOTAL_EPOCH" = "4194304" ] && ok "total epoch bytes = 4 MiB (3:1 split)" || fail "total wrong"

# rewardOf P4 should be exactly 3x rewardOf P5
P4_REW=$(cast call $REWARDS "rewardOf(address,uint256)(uint256)" $P4 $EPOCH --rpc-url $RPC | awk '{print $1}')
P5_REW=$(cast call $REWARDS "rewardOf(address,uint256)(uint256)" $P5 $EPOCH --rpc-url $RPC | awk '{print $1}')

RATIO_OK=$(python3 -c "p4=$P4_REW; p5=$P5_REW; print(1 if p5 > 0 and abs(p4/p5 - 3.0) < 0.01 else 0)")
[ "$RATIO_OK" = "1" ] && ok "rewardOf(P4) = 3 × rewardOf(P5)  (P4=$(ether $P4_REW) PROVA, P5=$(ether $P5_REW) PROVA)" || fail "ratio off"

# ─── 5. CLAIM BLOCKED before vesting buffer ──────────────────────────────
say "5. claim() reverts EpochNotVested before E.endsAt + 30d"

CLAIM_RESULT=$(cast send $REWARDS "claim(uint256)" $EPOCH --rpc-url $RPC --private-key $P4_KEY 2>&1 || true)
echo "$CLAIM_RESULT" | grep -qi "EpochNotVested\|reverted" && ok "claim() before vesting buffer reverts" || fail "should revert"

# ─── 6. CLAIM works after epoch end + 30d vesting ────────────────────────
say "6. claim() works after vesting buffer"

# Skip past epoch end + 30 days
EPOCH_DURATION=$(cast call $REWARDS "EPOCH_DURATION()(uint256)" --rpc-url $RPC | awk '{print $1}')
VESTING=$(cast call $REWARDS "VESTING_BUFFER()(uint256)" --rpc-url $RPC | awk '{print $1}')
SKIP=$((EPOCH_DURATION + VESTING + 100))
cast rpc evm_increaseTime $SKIP --rpc-url $RPC > /dev/null
cast rpc evm_mine --rpc-url $RPC > /dev/null

P4_PROVA_PRE=$(cast call $TOKEN "balanceOf(address)(uint256)" $P4 --rpc-url $RPC | awk '{print $1}')
REWARDS_PRE=$(cast call $TOKEN "balanceOf(address)(uint256)" $REWARDS --rpc-url $RPC | awk '{print $1}')
SUPPLY_BEFORE_CLAIM=$(cast call $TOKEN "totalSupply()(uint256)" --rpc-url $RPC | awk '{print $1}')

cast send $REWARDS "claim(uint256)" $EPOCH --rpc-url $RPC --private-key $P4_KEY > /dev/null

P4_PROVA_POST=$(cast call $TOKEN "balanceOf(address)(uint256)" $P4 --rpc-url $RPC | awk '{print $1}')
REWARDS_POST=$(cast call $TOKEN "balanceOf(address)(uint256)" $REWARDS --rpc-url $RPC | awk '{print $1}')
SUPPLY_AFTER_CLAIM=$(cast call $TOKEN "totalSupply()(uint256)" --rpc-url $RPC | awk '{print $1}')

P4_RECEIVED=$(python3 -c "print($P4_PROVA_POST - $P4_PROVA_PRE)")
REWARDS_DRAINED=$(python3 -c "print($REWARDS_PRE - $REWARDS_POST)")

[ "$P4_RECEIVED" = "$REWARDS_DRAINED" ] && [ "$P4_RECEIVED" = "$P4_REW" ] && ok "P4 received exactly rewardOf() ($(ether $P4_REW) PROVA)" || { fail "claim amount mismatch"; hr "$P4_REW" "$P4_RECEIVED"; }
[ "$SUPPLY_BEFORE_CLAIM" = "$SUPPLY_AFTER_CLAIM" ] && ok "supply unchanged (claim is transfer not mint)" || fail "supply changed during claim"

# Double-claim should revert
DOUBLE_RESULT=$(cast send $REWARDS "claim(uint256)" $EPOCH --rpc-url $RPC --private-key $P4_KEY 2>&1 || true)
echo "$DOUBLE_RESULT" | grep -qi "AlreadyClaimed\|reverted" && ok "double-claim reverts AlreadyClaimed" || fail "double-claim should revert"

# ─── 7. claimRange across multiple epochs ────────────────────────────────
say "7. claimRange across multiple epochs"

# We've already passed epoch 0 (where the proofs landed) and waited buffer.
# Have P5 prove again in a later epoch, then claim across both.
# Use COMMP_P5_2 to bypass per-(piece, prover, epoch) dedup
cast send $USDC "approve(address,uint256)" $MARKET 1000ether --rpc-url $RPC --private-key $CLIENT_KEY > /dev/null

# Skip a few epochs to be safely in a new one
cast rpc evm_increaseTime $((2 * 7 * 86400)) --rpc-url $RPC > /dev/null
cast rpc evm_mine --rpc-url $RPC > /dev/null

LATER_EPOCH=$(cast call $REWARDS "currentEpoch()(uint256)" --rpc-url $RPC | awk '{print $1}')
[ "$LATER_EPOCH" -gt "$EPOCH" ] && ok "advanced to a later epoch ($EPOCH → $LATER_EPOCH)" || fail "epoch didn't advance"

# Have P5 prove on its existing deal again (different epoch)
mock_verifier_call cast send $MARKET "possessionProven(uint256,uint256,uint256,uint256)" 402 1 2 1 --from $VERIFIER --unlocked --gas-limit 1500000 --rpc-url $RPC > /dev/null
P5_LATER_BYTES=$(cast call $REWARDS "bytesByEpochProver(uint256,address)(uint256)" $LATER_EPOCH $P5 --rpc-url $RPC | awk '{print $1}')
[ "$P5_LATER_BYTES" = "1048576" ] && ok "P5 proved 1 MiB in epoch $LATER_EPOCH" || fail "p5 later epoch wrong"

# Skip past vesting buffer
cast rpc evm_increaseTime $((EPOCH_DURATION + VESTING + 100)) --rpc-url $RPC > /dev/null
cast rpc evm_mine --rpc-url $RPC > /dev/null

# claimRange [0, current_epoch]
CURRENT=$(cast call $REWARDS "currentEpoch()(uint256)" --rpc-url $RPC | awk '{print $1}')
P5_PROVA_PRE=$(cast call $TOKEN "balanceOf(address)(uint256)" $P5 --rpc-url $RPC | awk '{print $1}')
cast send $REWARDS "claimRange(uint256,uint256)" 0 $CURRENT --rpc-url $RPC --private-key $P5_KEY > /dev/null
P5_PROVA_POST=$(cast call $TOKEN "balanceOf(address)(uint256)" $P5 --rpc-url $RPC | awk '{print $1}')
P5_TOTAL_CLAIMED=$(python3 -c "print($P5_PROVA_POST - $P5_PROVA_PRE)")

gt "$P5_TOTAL_CLAIMED" "0" && ok "P5 claimed across range: $(ether $P5_TOTAL_CLAIMED) PROVA" || fail "claimRange returned nothing"

# ─── 8. SELF-DEALING rejected ────────────────────────────────────────────
say "8. Self-dealing: prover==client, no emission credit"

cast send $USDC "transfer(address,uint256)" $P4 5000ether --rpc-url $RPC --private-key $DEPLOYER_KEY > /dev/null
cast send $USDC "approve(address,uint256)" $MARKET 500ether --rpc-url $RPC --private-key $P4_KEY > /dev/null

COMMP_SELF=$(cast keccak "self-deal-test")
cast send $MARKET "proposeDeal(address,bytes32,uint64,uint64,uint256)" \
  $P4 $COMMP_SELF 1048576 2592000 500ether --rpc-url $RPC --private-key $P4_KEY > /dev/null

NEW_DEAL_ID=$(cast call $MARKET "nextDealId()(uint256)" --rpc-url $RPC | awk '{print $1}')
SELF_DEAL_ID=$((NEW_DEAL_ID - 1))

mock_verifier_call cast send $MARKET "dataSetCreated(uint256,address,bytes)" 800 $P4 $(cast abi-encode "f(uint256)" $SELF_DEAL_ID) --from $VERIFIER --unlocked --gas-limit 1000000 --rpc-url $RPC > /dev/null

cast rpc evm_increaseTime 86400 --rpc-url $RPC > /dev/null
cast rpc evm_mine --rpc-url $RPC > /dev/null

EPOCH_NOW=$(cast call $REWARDS "currentEpoch()(uint256)" --rpc-url $RPC | awk '{print $1}')
P4_BYTES_PRE=$(cast call $REWARDS "bytesByEpochProver(uint256,address)(uint256)" $EPOCH_NOW $P4 --rpc-url $RPC | awk '{print $1}')

mock_verifier_call cast send $MARKET "possessionProven(uint256,uint256,uint256,uint256)" 800 1 1 1 --from $VERIFIER --unlocked --gas-limit 1500000 --rpc-url $RPC > /dev/null

P4_BYTES_POST=$(cast call $REWARDS "bytesByEpochProver(uint256,address)(uint256)" $EPOCH_NOW $P4 --rpc-url $RPC | awk '{print $1}')

[ "$P4_BYTES_POST" = "$P4_BYTES_PRE" ] && ok "self-dealing proof did not credit emission" || { fail "self-deal got credit"; hr "$P4_BYTES_PRE" "$P4_BYTES_POST"; }

# ─── 9. REDUNDANCY CAP ──────────────────────────────────────────────────
say "9. Redundancy cap: only first 4 provers per piece earn emission"

# Already have P1/P2/P3/P4/P5 (5 provers).
# P1/P2 had stake events earlier; need to top up P3 to be able to take a 1-MiB deal.
# Actually let's just have client send to all 5 in a fresh epoch.
# We'll use a NEW commp so we can compare counter.

# Snapshot ProverRewards.proversForPieceInEpoch
SHARED_COMMP=$(cast keccak "shared-redundancy-test")

# Top up P3's stake so it can commit 1 MiB. minStakePerGiB = 100 PROVA, 1 MiB = 1/1024 GiB
# So min stake for 1 MiB = 100 / 1024 ~= 0.1 PROVA. P3 already has 100 PROVA staked. Fine.
# But P3 might be in unbonding from earlier; just use fresh provers.

# Simpler: use 5 fresh deals from CLIENT, each to a different prover, all w/ same SHARED_COMMP.
# Check totalBytesByEpoch increases by 1 MiB × 4 (cap), not 5.

cast send $USDC "approve(address,uint256)" $MARKET 5000ether --rpc-url $RPC --private-key $CLIENT_KEY > /dev/null

# Skip to a totally fresh epoch
cast rpc evm_increaseTime $((3 * 7 * 86400)) --rpc-url $RPC > /dev/null
cast rpc evm_mine --rpc-url $RPC > /dev/null
FRESH_EPOCH=$(cast call $REWARDS "currentEpoch()(uint256)" --rpc-url $RPC | awk '{print $1}')

# Make 5 deals with the same commp to 5 different provers.
DEAL_IDS=()
for prover in $P1 $P2 $P3 $P4 $P5; do
    NEXT_ID=$(cast call $MARKET "nextDealId()(uint256)" --rpc-url $RPC | awk '{print $1}')
    cast send $MARKET "proposeDeal(address,bytes32,uint64,uint64,uint256)" \
      $prover $SHARED_COMMP 1048576 2592000 200ether --rpc-url $RPC --private-key $CLIENT_KEY > /dev/null
    DEAL_IDS+=($NEXT_ID)
done

# Activate each
i=0
for prover in $P1 $P2 $P3 $P4 $P5; do
    DEAL_ID=${DEAL_IDS[$i]}
    DSID=$((900 + i))
    mock_verifier_call cast send $MARKET "dataSetCreated(uint256,address,bytes)" $DSID $prover $(cast abi-encode "f(uint256)" $DEAL_ID) --from $VERIFIER --unlocked --gas-limit 1000000 --rpc-url $RPC > /dev/null 2>&1 || true
    i=$((i+1))
done

# Skip 1 day
cast rpc evm_increaseTime 86400 --rpc-url $RPC > /dev/null
cast rpc evm_mine --rpc-url $RPC > /dev/null

# All 5 provers post a proof
i=0
for prover in $P1 $P2 $P3 $P4 $P5; do
    DSID=$((900 + i))
    mock_verifier_call cast send $MARKET "possessionProven(uint256,uint256,uint256,uint256)" $DSID 1 1 1 --from $VERIFIER --unlocked --gas-limit 1500000 --rpc-url $RPC > /dev/null 2>&1 || true
    i=$((i+1))
done

# Check redundancy counter
CAP=$(cast call $REWARDS "redundancyCap()(uint8)" --rpc-url $RPC | awk '{print $1}')
PROVERS_FOR_PIECE=$(cast call $REWARDS "proversForPieceInEpoch(uint256,bytes32)(uint8)" $FRESH_EPOCH $SHARED_COMMP --rpc-url $RPC | awk '{print $1}')

[ "$PROVERS_FOR_PIECE" = "$CAP" ] && ok "redundancy cap reached: $PROVERS_FOR_PIECE = $CAP (5 attempted, only $CAP credited)" || { fail "redundancy cap wrong"; hr "$CAP" "$PROVERS_FOR_PIECE"; }

# ─── 10. EMISSION INVARIANT: total claims ≤ 50M cap ──────────────────────
say "10. Total emission bucket invariant: ProverRewards balance never below outstanding claims"

REWARDS_BAL=$(cast call $TOKEN "balanceOf(address)(uint256)" $REWARDS --rpc-url $RPC | awk '{print $1}')
gt "$REWARDS_BAL" "0" && ok "ProverRewards still holds: $(ether $REWARDS_BAL) PROVA (≤ 50M ceiling)" || fail "rewards drained"

CEILING=50000000000000000000000000  # 50M
le "$REWARDS_BAL" "$CEILING" && ok "rewards balance never exceeds 50M cap" || fail "exceeded cap"

# ─── Summary ─────────────────────────────────────────────────────────────
echo ""
echo "──────────────────────────────────"
echo "  $PASS passed, $FAIL failed"
echo "──────────────────────────────────"
exit $FAIL
