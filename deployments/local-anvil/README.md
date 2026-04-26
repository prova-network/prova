# Local Anvil deployment — 2026-04-26

End-to-end smoke test of the full v2 protocol on `anvil` (foundry's local EVM).

## Status: ✅ working

All 8 contracts deployed, end-to-end deal lifecycle verified, both happy-path
(propose → accept → prove → USDC streams → fee burns) and fault-path
(propose → accept → no proofs → faultDeal → 50 PROVA slashed + USDC refund)
work as specified.

## Addresses

```
Chain ID:           31337 (anvil)
RPC:                http://127.0.0.1:8545
Deployer:           0xf39Fd6e51aad88F6F4ce6aB8827279cfFFb92266
Treasury:           same as deployer

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

## Reproduce

1. Start anvil: `anvil &`
2. Deploy MockUSDC: `forge create src/MockUSDC.sol:MockUSDC --rpc-url http://127.0.0.1:8545 --private-key <ANVIL_KEY> --broadcast`
3. Run deploy script: `PROVA_USDC=<usdc-address> forge script script/Deploy.s.sol --rpc-url http://127.0.0.1:8545 --broadcast --private-key <ANVIL_KEY>`
4. Smoke-test: `bash scripts/anvil-e2e.sh`

## Verified behaviour

- Prover registration via `ProverRegistry.register`
- PROVA stake via `ProverStaking.stake` (50,000 PROVA locked)
- Client deal proposal in USDC via `Marketplace.proposeDeal` (1000 USDC, 30-day term)
- Mock verifier triggers `dataSetCreated` → deal Active
- Skip 10 days → `possessionProven` → 330 USDC streams to prover, 3.33 USDC to FeeRouter
- ProverRewards `recordProof` hook fires: 1 MiB credited to prover for the epoch
- Fault path: skip 3+ days, no proofs → `faultDeal` → 50 PROVA slashed, 500 USDC refunded to client, deal status = Slashed (5)

Next: deploy to Base Sepolia (real testnet) once we have a deployer EOA + faucet ETH.
