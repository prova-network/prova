# Local Anvil deployment — 2026-04-26 (test 2)

Comprehensive PROVA-flow audit on local anvil. **16/16 tests pass.**

Two contract bugs found and fixed since test 1:

- **Issue 1: slashing burns PROVA** (was: pooled in `slashedPool`, withdrawable by owner).
  The whitepaper, spec, and tokenomics doc all say slashing burns; code didn't.
  Now: `ProverStaking.slash()` calls `ERC20Burnable.burn()` directly, slashed
  amount is permanently removed from total supply. `withdrawSlashed()` is
  retained as an explicit revert so old call sites fail loudly.

- **Issue 2: ProverRewards never received its 50M emission bucket.**
  `Deploy.s.sol` instantiated the contract but never transferred the 50M PROVA
  from treasury into it, meaning every `claim()` would revert with insufficient
  balance. Fix: deploy script now transfers 50,000,000 PROVA at deploy time
  when `treasury == deployer` (testnet/anvil); the deploy log notes when
  governance must do this manually (mainnet via multisig).

## Address table (anvil chain 31337)

```
MockUSDC            0x5FbDB2315678afecb367f032d93F642f64180aa3
ProvaToken          0xe7f1725E7734CE288F8367e1Bb143E90bb3F0512
ProverRegistry      0x9fE46736679d2D9a65F0992F2272dE9f3c7fa6e0
ProverStaking       0xCf7Ed3AccA5a467e9e704C703E8D87F634fB0Fc9
ContentRegistry     0xDc64a140Aa3E981100a9becA4E685f962f0cF6C9
StorageMarketplace  0xa513E6E4b8f2a923D98304ec87F64353C4D5C853
MockProofVerifier   0x5FC8d32690cc91D4c39d9d3abcBD16989F875707
FeeRouter           0x0165878A594ca255338adfa4d48449f69242Eb8F
ProverRewards       0x610178dA211FEF7D417bC0e6FeD39F05609AD788
```

## Test 2 verifies

1. **PROVA genesis state** — 100M total supply; 50M to deployer/treasury; 50M to ProverRewards.
2. **Prover registration + stake** — 50,000 PROVA staked, custody held by ProverStaking.
3. **Healthy deal lifecycle** — 1000 USDC deal, 10-day proof, prover earns 330 USDC, FeeRouter accrues 3.33 USDC (exact 99:1 split), ProverRewards credits 1 MiB to prover for the epoch.
4. **Slashing burns PROVA** — fault-deal triggers, 50 PROVA permanently removed from total supply, prover stake decreases by exactly the slash amount, slashedPool stays 0, withdrawSlashed reverts.
5. **Anti-gaming: self-dealing** — when prover == client, the marketplace's emission hook fires, ProverRewards reverts SelfDealing inside try/catch, the proof itself succeeds (USDC pays), but emission credit is NOT awarded.
6. **Balance sheet integrity** — sum of all PROVA held in protocol contracts + treasury + prover = total supply (after burns).

## Reproduce

```bash
# 1. Start anvil
anvil &
sleep 3

# 2. Deploy MockUSDC
forge create src/MockUSDC.sol:MockUSDC \
  --rpc-url http://127.0.0.1:8545 \
  --private-key 0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80 \
  --broadcast

# 3. Run main deploy (use the MockUSDC address from step 2)
PROVA_USDC=0x5FbDB2315678afecb367f032d93F642f64180aa3 \
  forge script script/Deploy.s.sol \
  --rpc-url http://127.0.0.1:8545 \
  --broadcast \
  --private-key 0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80

# 4. Run the audit
bash scripts/anvil-test2.sh
```
