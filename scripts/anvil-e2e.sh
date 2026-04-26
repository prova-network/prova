#!/usr/bin/env bash
# End-to-end smoke test of the v2 deployment on local anvil.
set -euo pipefail

RPC="http://127.0.0.1:8545"
DEPLOYER_KEY="0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80"
DEPLOYER="0xf39Fd6e51aad88F6F4ce6aB8827279cfFFb92266"
# Anvil's account #1 = our prover
PROVER_KEY="0x59c6995e998f97a5a0044966f0945389dc9e86dae88c7a8412f4603b6b78690d"
PROVER="0x70997970C51812dc3A010C7d01b50e0d17dc79C8"
# Anvil's account #2 = our client
CLIENT_KEY="0x5de4111afa1a4b94908f83103eb1f1706367c2e68ca870fc3fb9a804cdab365a"
CLIENT="0x3C44CdDdB6a900fa2b585dd299e03d12FA4293BC"

USDC=0x5FbDB2315678afecb367f032d93F642f64180aa3
TOKEN=0xe7f1725E7734CE288F8367e1Bb143E90bb3F0512
REGISTRY=0x9fE46736679d2D9a65F0992F2272dE9f3c7fa6e0
STAKING=0xCf7Ed3AccA5a467e9e704C703E8D87F634fB0Fc9
MARKET=0xa513E6E4b8f2a923D98304ec87F64353C4D5C853
VERIFIER=0x5FC8d32690cc91D4c39d9d3abcBD16989F875707
REWARDS=0x610178dA211FEF7D417bC0e6FeD39F05609AD788
FEEROUTER=0x0165878A594ca255338adfa4d48449f69242Eb8F

say() { printf "\n\033[36m=== %s ===\033[0m\n" "$1"; }

# ─── Step 0: Mint USDC to client (we deployed MockUSDC; deployer holds 1B USDC) ───
say "Step 0: Fund client with 10,000 mock USDC"
cast send $USDC "transfer(address,uint256)" $CLIENT 10000ether \
    --rpc-url $RPC --private-key $DEPLOYER_KEY --json | python3 -c "import sys,json;d=json.load(sys.stdin);print('  tx:', d['transactionHash'][:18]+'...', 'status:', d['status'])"

# ─── Step 1: Deployer (acting as treasury) sends PROVA stake bond to prover ───
say "Step 1: Treasury sends 100,000 PROVA to prover for staking"
cast send $TOKEN "transfer(address,uint256)" $PROVER 100000ether \
    --rpc-url $RPC --private-key $DEPLOYER_KEY --json | python3 -c "import sys,json;d=json.load(sys.stdin);print('  tx:', d['transactionHash'][:18]+'...', 'status:', d['status'])"

# ─── Step 2: Prover registers ───
say "Step 2: Prover registers in ProverRegistry"
# register(string endpoint, uint64 features, uint128 pricePerGibDay, uint128 pricePerByteServed, string metadata)
cast send $REGISTRY "register(string,uint64,uint128,uint128,string)" \
    "https://prover.example/pdp" 3 1000000000 0 "" \
    --rpc-url $RPC --private-key $PROVER_KEY --json | python3 -c "import sys,json;d=json.load(sys.stdin);print('  tx:', d['transactionHash'][:18]+'...', 'status:', d['status'])"

ACTIVE=$(cast call $REGISTRY "isActive(address)(bool)" $PROVER --rpc-url $RPC)
echo "  prover active: $ACTIVE"

# ─── Step 3: Prover stakes 50,000 PROVA ───
say "Step 3: Prover stakes 50,000 PROVA into ProverStaking"
cast send $TOKEN "approve(address,uint256)" $STAKING 50000ether \
    --rpc-url $RPC --private-key $PROVER_KEY --json > /dev/null
cast send $STAKING "stake(uint256)" 50000ether \
    --rpc-url $RPC --private-key $PROVER_KEY --json | python3 -c "import sys,json;d=json.load(sys.stdin);print('  tx:', d['transactionHash'][:18]+'...', 'status:', d['status'])"

STAKED=$(cast call $STAKING "getStake(address)((uint256,uint256,uint256,uint256))" $PROVER --rpc-url $RPC)
echo "  prover stake (staked, unbonding, unbondingEndsAt, committedBytes):"
echo "    $STAKED"

# ─── Step 4: Client proposes a deal in USDC ───
say "Step 4: Client proposes a deal (1000 USDC, 30 days, 1 MiB piece)"
cast send $USDC "approve(address,uint256)" $MARKET 1000ether \
    --rpc-url $RPC --private-key $CLIENT_KEY --json > /dev/null

# proposeDeal(prover, commp, pieceSize, durationSeconds, totalPayment)
COMMP=$(cast keccak "test-piece-cid")
TX=$(cast send $MARKET "proposeDeal(address,bytes32,uint64,uint64,uint256)" \
    $PROVER $COMMP 1048576 2592000 1000ether \
    --rpc-url $RPC --private-key $CLIENT_KEY --json)
echo "$TX" | python3 -c "import sys,json;d=json.load(sys.stdin);print('  tx:', d['transactionHash'][:18]+'...', 'status:', d['status'], 'logs:', len(d.get('logs',[])))"

DEAL_ID=1  # First deal
say "Step 5: Verify deal status = Proposed (0)"
DEAL_RAW=$(cast call $MARKET "deals(uint256)(address,address,bytes32,uint64,uint64,uint64,uint128,uint128,uint64,uint64,uint256,uint8)" $DEAL_ID --rpc-url $RPC)
echo "  deal raw: $(echo "$DEAL_RAW" | head -c 200)..."

# ─── Step 6: Mock verifier triggers acceptance ───
say "Step 6: Mock ProofVerifier triggers acceptance via dataSetCreated"
EXTRA_DATA=$(cast abi-encode "f(uint256)" $DEAL_ID)
# The MockProofVerifier in our deploy script doesn't have a public way to call this — the marketplace's
# dataSetCreated takes (uint256, address, bytes) and is callable only by proofVerifier.
# Anvil's impersonation lets us pretend to be the verifier:
cast rpc anvil_impersonateAccount $VERIFIER --rpc-url $RPC > /dev/null
# Send some ETH to verifier so it can pay gas
cast send $VERIFIER --value 1ether --rpc-url $RPC --private-key $DEPLOYER_KEY > /dev/null
# Now call as the verifier
cast send $MARKET "dataSetCreated(uint256,address,bytes)" 42 $PROVER $EXTRA_DATA \
    --rpc-url $RPC --from $VERIFIER --unlocked --json | python3 -c "import sys,json;d=json.load(sys.stdin);print('  tx:', d['transactionHash'][:18]+'...', 'status:', d['status'])"
cast rpc anvil_stopImpersonatingAccount $VERIFIER --rpc-url $RPC > /dev/null

DEAL_RAW2=$(cast call $MARKET "deals(uint256)(address,address,bytes32,uint64,uint64,uint64,uint128,uint128,uint64,uint64,uint256,uint8)" $DEAL_ID --rpc-url $RPC)
DEAL_STATUS=$(echo "$DEAL_RAW2" | tail -1)
echo "  deal status after acceptance (should be 1=Active): $DEAL_STATUS"

# ─── Step 7: Skip 10 days, post a proof, verify USDC streams ───
say "Step 7: Skip 10 days, post a proof"
cast rpc evm_increaseTime 864000 --rpc-url $RPC > /dev/null
cast rpc evm_mine --rpc-url $RPC > /dev/null

PROVER_USDC_BEFORE=$(cast call $USDC "balanceOf(address)(uint256)" $PROVER --rpc-url $RPC)
TREASURY_USDC_BEFORE=$(cast call $USDC "balanceOf(address)(uint256)" $FEEROUTER --rpc-url $RPC)
echo "  prover USDC before: $PROVER_USDC_BEFORE"
echo "  feeRouter USDC before: $TREASURY_USDC_BEFORE"

cast rpc anvil_impersonateAccount $VERIFIER --rpc-url $RPC > /dev/null
cast send $MARKET "possessionProven(uint256,uint256,uint256,uint256)" 42 1 123 1 \
    --rpc-url $RPC --from $VERIFIER --unlocked --json | python3 -c "import sys,json;d=json.load(sys.stdin);print('  tx:', d['transactionHash'][:18]+'...', 'status:', d['status'])"
cast rpc anvil_stopImpersonatingAccount $VERIFIER --rpc-url $RPC > /dev/null

PROVER_USDC_AFTER=$(cast call $USDC "balanceOf(address)(uint256)" $PROVER --rpc-url $RPC)
TREASURY_USDC_AFTER=$(cast call $USDC "balanceOf(address)(uint256)" $FEEROUTER --rpc-url $RPC)
echo "  prover USDC after: $PROVER_USDC_AFTER"
echo "  feeRouter USDC after: $TREASURY_USDC_AFTER"
echo "  → prover earned $(python3 -c "print(($PROVER_USDC_AFTER - $PROVER_USDC_BEFORE) / 1e18)") USDC"
echo "  → feeRouter earned $(python3 -c "print(($TREASURY_USDC_AFTER - $TREASURY_USDC_BEFORE) / 1e18)") USDC (1% protocol fee)"

# ─── Step 8: Show ProverRewards epoch state ───
say "Step 8: ProverRewards saw the proof (anti-gaming + emission accounting)"
EPOCH=$(cast call $REWARDS "currentEpoch()(uint256)" --rpc-url $RPC)
echo "  current epoch: $EPOCH"
PROVEN=$(cast call $REWARDS "bytesByEpochProver(uint256,address)(uint256)" $EPOCH $PROVER --rpc-url $RPC)
echo "  bytesByEpochProver for epoch $EPOCH: $PROVEN"
TOTAL=$(cast call $REWARDS "totalBytesByEpoch(uint256)(uint256)" $EPOCH --rpc-url $RPC)
echo "  totalBytesByEpoch for epoch $EPOCH: $TOTAL"

say "DONE — full deal lifecycle worked on anvil"
