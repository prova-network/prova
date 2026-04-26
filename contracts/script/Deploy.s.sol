// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Prova Network contributors.
pragma solidity ^0.8.20;

import {Script, console2} from "forge-std/Script.sol";
import {IERC20} from "@openzeppelin/contracts/token/ERC20/IERC20.sol";

import {ProvaToken} from "../src/ProvaToken.sol";
import {ProverRegistry} from "../src/ProverRegistry.sol";
import {ProverStaking} from "../src/ProverStaking.sol";
import {ContentRegistry} from "../src/ContentRegistry.sol";
import {StorageMarketplace} from "../src/StorageMarketplace.sol";
import {MockProofVerifier} from "../src/MockProofVerifier.sol";
import {ProofVerifier} from "../src/ProofVerifier.sol";
import {FeeRouter} from "../src/FeeRouter.sol";
import {ProverRewards} from "../src/ProverRewards.sol";
import {ERC1967Proxy} from "@openzeppelin/contracts/proxy/ERC1967/ERC1967Proxy.sol";

/// @title Deploy
/// @notice Deploys the Prova contract set and wires the cross-contract
///         authorizations.
///
///         Economic model (v1 tokenomics):
///           - Clients pay storage fees in USDC (paymentToken)
///           - Provers post slashable PROVA stake (ProverStaking holds PROVA)
///           - 1% protocol fee on USDC routes to FeeRouter
///           - FeeRouter swaps USDC to PROVA on Uniswap V3 and burns
///
///         Required env vars:
///           PROVA_USDC          - USDC token address on the target chain
///                                 Base mainnet: 0x833589fCD6eDb6E08f4c7C32D4f71b54bdA02913
///                                 Base Sepolia: 0x036CbD53842c5426634e7929541eC2318f3dCF7e
///         Optional env vars:
///           PROVA_SWAP_ROUTER   - Uniswap V3 SwapRouter address. If unset,
///                                 FeeRouter is left in HOLD mode (USDC just
///                                 accumulates; we set the router and flip to
///                                 BURN later via governance). Default: address(0).
///           PROVA_TREASURY      - multisig that receives the PROVA supply.
///                                 Defaults to msg.sender (deployer) for testnet.
///
///         Usage:
///           PROVA_USDC=0x036C... forge script script/Deploy.s.sol \
///             --rpc-url $RPC --broadcast --private-key $PK
///
///         This script does NOT deploy the production ProofVerifier (UUPS
///         proxy). It uses MockProofVerifier so the marketplace + staking
///         flow is end-to-end testable. Replace the verifier before
///         mainnet via marketplace.setProofVerifier().
contract DeployScript is Script {
    struct Addresses {
        address token;
        address registry;
        address staking;
        address content;
        address marketplace;
        address verifier;
        address feeRouter;
        address proverRewards;
    }

    function run() external returns (Addresses memory out) {
        address deployer = msg.sender;
        address treasury = vm.envOr("PROVA_TREASURY", deployer);
        address usdc     = vm.envAddress("PROVA_USDC");
        address swapRouter = vm.envOr("PROVA_SWAP_ROUTER", address(0));

        require(usdc != address(0), "PROVA_USDC env var required");

        console2.log("Deployer: ", deployer);
        console2.log("Treasury: ", treasury);
        console2.log("USDC:     ", usdc);
        console2.log("SwapRouter:", swapRouter);

        vm.startBroadcast();

        // 1. PROVA token. 100M minted to treasury.
        ProvaToken token = new ProvaToken(treasury);
        console2.log("ProvaToken deployed at:        ", address(token));

        // 2. Prover registry
        ProverRegistry registry = new ProverRegistry();
        console2.log("ProverRegistry deployed at:    ", address(registry));

        // 3. Prover staking — PROVA-denominated bond.
        //    Default floor: 0.1 PROVA per TiB committed (PROVA-only floor).
        //    Governance MAY also set the USD-equivalent floor + oracle for the
        //    real binding constraint. Without an oracle, only the PROVA floor
        //    applies.
        //    100 TB → ~10 PROVA stake (PROVA-only); with $3/TiB USD floor
        //    at $0.10 PROVA: ~3,000 PROVA stake at 100 TB.
        uint256 minStakePerTiB = 0.1 ether; // 0.1 PROVA per TiB
        ProverStaking staking = new ProverStaking(IERC20(address(token)), minStakePerTiB);
        console2.log("ProverStaking deployed at:     ", address(staking));

        // 4. Content registry
        ContentRegistry content = new ContentRegistry();
        console2.log("ContentRegistry deployed at:   ", address(content));

        // 5. ProofVerifier. Two paths:
        //    - PROVA_USE_MOCK_VERIFIER=1: deploy MockProofVerifier (fast, used
        //      by anvil smoke tests; rubber-stamps proofs).
        //    - default: deploy the real ProofVerifier (forked from FilOzone/pdp)
        //      behind an ERC1967 UUPS proxy, initialized with challengeFinality=150.
        bool useMock = vm.envOr("PROVA_USE_MOCK_VERIFIER", false);
        address verifier;
        if (useMock) {
            MockProofVerifier mock = new MockProofVerifier();
            verifier = address(mock);
            console2.log("MockProofVerifier deployed at: ", verifier);
        } else {
            // Deploy implementation
            ProofVerifier impl = new ProofVerifier(1);
            console2.log("ProofVerifier impl at:         ", address(impl));
            // Deploy UUPS proxy with initialize(challengeFinality)
            uint256 challengeFinality = vm.envOr("PROVA_CHALLENGE_FINALITY", uint256(150));
            bytes memory initCalldata = abi.encodeCall(ProofVerifier.initialize, (challengeFinality));
            ERC1967Proxy proxy = new ERC1967Proxy(address(impl), initCalldata);
            verifier = address(proxy);
            console2.log("ProofVerifier (proxy) at:      ", verifier);
            console2.log("  challengeFinality:           ", challengeFinality);
        }

        // 6. FeeRouter holds the marketplace's USDC fees, swaps to PROVA, burns.
        //    Defaults to HOLD mode until the swap router is configured.
        FeeRouter feeRouter = new FeeRouter(usdc, address(token), swapRouter, treasury);
        console2.log("FeeRouter deployed at:         ", address(feeRouter));

        // 7. StorageMarketplace - clients pay USDC, treasury is the FeeRouter.
        uint256 slashPerFault = 50 ether; // PROVA slashed per fault
        StorageMarketplace marketplace = new StorageMarketplace(
            verifier,
            IERC20(usdc),       // payment token = USDC
            registry,
            staking,
            content,
            address(feeRouter), // treasury = FeeRouter (so fees auto-route)
            slashPerFault
        );
        console2.log("StorageMarketplace deployed at:", address(marketplace));

        // 8. Cross-contract wiring
        staking.setAuthorizedController(address(marketplace), true);
        content.setMarketplace(address(marketplace));

        // 9. ProverRewards: holds the 50M PROVA emission bucket and pays
        //    provers per epoch based on bytes-proven contributions.
        ProverRewards proverRewards = new ProverRewards(
            IERC20(address(token)),
            treasury,                  // owner = treasury multisig
            uint64(block.timestamp)    // genesis = now
        );
        console2.log("ProverRewards deployed at:     ", address(proverRewards));

        // 10. Wire the marketplace to ping ProverRewards on proofs/misses
        marketplace.setProverRewards(address(proverRewards));

        // 11. Authorize the marketplace as the recorder on ProverRewards
        //     (this requires owner = treasury, which is the same address
        //     in this script when PROVA_TREASURY isn't overridden).
        if (treasury == deployer) {
            proverRewards.setMarketplace(address(marketplace));
        } else {
            console2.log("NOTE: ProverRewards.setMarketplace must be called by the treasury multisig");
        }

        // 12. Fund the ProverRewards contract with the 50M PROVA emission
        //     bucket. Source: the treasury (which holds the full 100M genesis
        //     supply at construction). Without this transfer, claim() reverts.
        if (treasury == deployer) {
            token.transfer(address(proverRewards), 50_000_000 ether);
            console2.log("Funded ProverRewards with:    50,000,000 PROVA");
        } else {
            console2.log("NOTE: treasury multisig must transfer 50,000,000 PROVA to ProverRewards before TGE");
        }

        vm.stopBroadcast();

        out = Addresses({
            token: address(token),
            registry: address(registry),
            staking: address(staking),
            content: address(content),
            marketplace: address(marketplace),
            verifier: address(verifier),
            feeRouter: address(feeRouter),
            proverRewards: address(proverRewards)
        });

        console2.log("---");
        console2.log("ProvaToken         =", out.token);
        console2.log("ProverRegistry     =", out.registry);
        console2.log("ProverStaking      =", out.staking);
        console2.log("ContentRegistry    =", out.content);
        console2.log("StorageMarketplace =", out.marketplace);
        console2.log("MockProofVerifier  =", out.verifier);
        console2.log("FeeRouter          =", out.feeRouter);
        console2.log("ProverRewards      =", out.proverRewards);
    }
}
