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
        console2.log("Allocation table (allocations are managed off-chain until vesting schedules are created):");
        console2.log("  Team and core engineers (18%):     180,000,000 PROVA");
        console2.log("  Advisors / BD / sales / design (12%): 120,000,000 PROVA");
        console2.log("  Early supporters / FF round (5%):    50,000,000 PROVA");
        console2.log("  Ecosystem grants (10%):              100,000,000 PROVA");
        console2.log("  Community / treasury / liquidity (35%): 350,000,000 PROVA");
        console2.log("  Protocol incentives / staking (20%): 200,000,000 PROVA");
        console2.log("  Total:                            1,000,000,000 PROVA");

        vm.stopBroadcast();
    }
}
