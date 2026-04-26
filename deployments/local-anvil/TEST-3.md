# Anvil test 3 — deeper economic flows

**27/27 passed.** Verifies the parts of the protocol that test 2 didn't reach: unbonding queue, slashing-during-unbonding, ProverRewards.claim() lifecycle, claimRange, redundancy cap.

## What's verified beyond test 2

| Behavior | Notes |
| --- | --- |
| Unbonding queue 14-day timer | requestUnstake moves PROVA bonded → unbonding; withdraw blocked until UNBONDING_PERIOD elapses |
| Slash priority order | Slash hits bonded first; unbonding only if slashPerFault > bonded (prevents instant-exit) |
| minStakePerGiB enforced | proposeDeal reverts ProverCannotCommit when bytes > capacity |
| Reward math proportional | rewardOf(P4) ÷ rewardOf(P5) = 3.0 when bytes are 3 MiB ÷ 1 MiB |
| Vesting buffer | claim() before epoch.endsAt + 30d reverts EpochNotVested |
| claim() transfers (not mints) | supply unchanged; ProverRewards balance decreases by exactly the claim amount |
| Double-claim guard | second claim() of same epoch reverts AlreadyClaimed |
| claimRange across epochs | non-contiguous epochs (0 + 10) batched correctly |
| Self-dealing detected | prover == client emission attempt: no credit, USDC payment still flows |
| Redundancy cap | 5 different provers prove same piece-CID; only first 4 (default cap) get emission credit |
| Emission bucket invariant | ProverRewards balance ≤ 50M PROVA at all times |

## Real numbers from this run

| Metric | Value |
| --- | --- |
| P4 stake | 50,000 PROVA |
| P4 proven | 3 MiB in epoch 2 |
| P4 reward (epoch 2) | 179,794.52 PROVA |
| P5 stake | 50,000 PROVA |
| P5 proven | 1 MiB in epoch 2, 1 MiB in epoch 10 |
| P5 total claimed (range) | 299,657.53 PROVA |
| Slashing burn this run | 50 PROVA (one fault event) |
| ProverRewards remaining | 49,520,547 PROVA (~1% emitted across this test) |

## Reproduce

```bash
# 1. Fresh anvil + deploy
anvil &
sleep 3
forge create src/MockUSDC.sol:MockUSDC --rpc-url http://127.0.0.1:8545 \
  --private-key 0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80 --broadcast
PROVA_USDC=0x5FbDB2315678afecb367f032d93F642f64180aa3 \
  forge script script/Deploy.s.sol --rpc-url http://127.0.0.1:8545 \
  --broadcast --private-key 0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80

# 2. Run the test
bash scripts/anvil-test3.sh
```

Test scripts: `scripts/anvil-e2e.sh` (test 1, smoke), `scripts/anvil-test2.sh` (PROVA flow audit), `scripts/anvil-test3.sh` (deep economic flows).
