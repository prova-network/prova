// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Prova Network contributors.
pragma solidity ^0.8.20;

import {Script, console2} from "forge-std/Script.sol";

import {ProvaToken} from "../src/ProvaToken.sol";
import {ProverRegistry} from "../src/ProverRegistry.sol";
import {ProverStaking} from "../src/ProverStaking.sol";
import {ContentRegistry} from "../src/ContentRegistry.sol";
import {StorageMarketplace} from "../src/StorageMarketplace.sol";

/// @title Deploy
/// @notice Deploys the Prova contract set and wires the cross-contract
///         authorizations. Usage:
///
///           forge script script/Deploy.s.sol --rpc-url http://localhost:8545 \
///             --broadcast --unlocked --sender 0xf39F...92266
///
///         Or with a private key:
///
///           forge script script/Deploy.s.sol --rpc-url http://localhost:8545 \
///             --broadcast --private-key $PK
///
///         This script does NOT deploy ProofVerifier (UUPS proxy) for v0 — it's
///         the forked PDPVerifier and needs separate initialization handling.
///         The marketplace is configured with a placeholder ProofVerifier
///         address that must be replaced before the marketplace is actually
///         used.
contract DeployScript is Script {
    struct Addresses {
        address token;
        address registry;
        address staking;
        address content;
        address marketplace;
    }

    function run() external returns (Addresses memory out) {
        address deployer = msg.sender;
        console2.log("Deployer:", deployer);

        vm.startBroadcast();

        // 1. Token
        ProvaToken token = new ProvaToken(deployer);
        console2.log("ProvaToken deployed at:", address(token));

        // 2. Prover registry (no deps)
        ProverRegistry registry = new ProverRegistry();
        console2.log("ProverRegistry deployed at:", address(registry));

        // 3. Prover staking (needs token)
        uint256 minStakePerGib = 100 ether; // 100 PROVA per GiB committed
        ProverStaking staking = new ProverStaking(token, minStakePerGib);
        console2.log("ProverStaking deployed at:", address(staking));

        // 4. Content registry
        ContentRegistry content = new ContentRegistry();
        console2.log("ContentRegistry deployed at:", address(content));

        // 5. Storage marketplace. Wire ProofVerifier as 0x..dEaD for now
        // (integration tests substitute EOA for ProofVerifier); real deployment
        // will pass the initialized proxy address.
        address proofVerifierPlaceholder = address(0xdead);
        uint256 slashPerFault = 50 ether;

        StorageMarketplace marketplace = new StorageMarketplace(
            proofVerifierPlaceholder,
            token,
            registry,
            staking,
            content,
            deployer,       // treasury
            slashPerFault
        );
        console2.log("StorageMarketplace deployed at:", address(marketplace));

        // 6. Cross-contract wiring
        staking.setAuthorizedController(address(marketplace), true);
        content.setMarketplace(address(marketplace));

        vm.stopBroadcast();

        out = Addresses({
            token: address(token),
            registry: address(registry),
            staking: address(staking),
            content: address(content),
            marketplace: address(marketplace)
        });

        console2.log("---");
        console2.log("ProvaToken         =", out.token);
        console2.log("ProverRegistry     =", out.registry);
        console2.log("ProverStaking      =", out.staking);
        console2.log("ContentRegistry    =", out.content);
        console2.log("StorageMarketplace =", out.marketplace);
    }
}
