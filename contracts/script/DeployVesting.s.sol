// SPDX-License-Identifier: MIT
pragma solidity ^0.8.24;

import {Script, console2} from "forge-std/Script.sol";
import {ProvaToken}  from "../src/ProvaToken.sol";
import {ProvaVesting} from "../src/ProvaVesting.sol";

/// @title DeployVesting
/// @notice Deploys ProvaVesting and (optionally) ProvaToken if not yet
///         deployed. Outputs the addresses for the deployment manifest.
///
/// Usage:
///   PROVA_TREASURY=0x...   # multisig that holds the supply
///   PROVA_OWNER=0x...      # multisig that creates schedules (often same as treasury)
///   PROVA_TOKEN=0x...      # optional: skip token deploy if already done
///   forge script script/DeployVesting.s.sol \
///     --rpc-url $RPC --broadcast --private-key $PK --verify
contract DeployVesting is Script {
    function run() external {
        address treasury = vm.envAddress("PROVA_TREASURY");
        address owner    = vm.envOr("PROVA_OWNER", treasury);
        address existingToken = vm.envOr("PROVA_TOKEN", address(0));

        vm.startBroadcast();

        ProvaToken token;
        if (existingToken == address(0)) {
            token = new ProvaToken(treasury);
            console2.log("ProvaToken deployed at:", address(token));
        } else {
            token = ProvaToken(existingToken);
            console2.log("Reusing existing ProvaToken at:", address(token));
        }

        ProvaVesting vesting = new ProvaVesting(address(token), owner);
        console2.log("ProvaVesting deployed at:", address(vesting));

        console2.log("--");
        console2.log("Allocation table (v2, supply-side heavy):");
        console2.log("  GENESIS DISTRIBUTION (45M, 45%, vested):");
        console2.log("    Public sale at TGE / LBP (6%):          6,000,000 PROVA");
        console2.log("    Private SAFT round (12%):              12,000,000 PROVA");
        console2.log("    Team and core engineers (14%):         14,000,000 PROVA");
        console2.log("    Advisors / BD / sales / design (4%):    4,000,000 PROVA");
        console2.log("    Treasury / community (6%):              6,000,000 PROVA");
        console2.log("    Liquidity (DEX seeding) (3%):           3,000,000 PROVA");
        console2.log("  PROVER EMISSION (50M, 50%, 8-year curve):");
        console2.log("    Distributed by ProverRewards.sol");
        console2.log("  ECOSYSTEM + COMMUNITY (5M, 5%):");
        console2.log("    Ecosystem grants (3%):                  3,000,000 PROVA");
        console2.log("    Community / referrals (2%):             2,000,000 PROVA");
        console2.log("  TOTAL:                                  100,000,000 PROVA");

        vm.stopBroadcast();
    }
}
