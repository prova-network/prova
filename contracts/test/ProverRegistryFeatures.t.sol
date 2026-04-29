// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

import {Test} from "forge-std/Test.sol";
import {ProverRegistry} from "../src/ProverRegistry.sol";

/// @notice Defensive tests pinning the feature-bit constants on
///         ProverRegistry. The bitmap layout is consumed by future
///         contracts (e.g. ComputeMarketplace, RFC #7) and by every
///         indexer that decodes ProverRegistered/ProverUpdated events,
///         so accidental reordering would silently break downstream
///         systems. These tests are cheap and exist only to fail loudly
///         if anyone shifts the bit layout in a future PR.
contract ProverRegistryFeaturesTest is Test {
    ProverRegistry registry;

    function setUp() public {
        registry = new ProverRegistry();
    }

    function test_feature_pdp_is_bit_zero() public view {
        assertEq(registry.FEATURE_PDP(), uint64(1) << 0);
    }

    function test_feature_https_serving_is_bit_one() public view {
        assertEq(registry.FEATURE_HTTPS_SERVING(), uint64(1) << 1);
    }

    function test_feature_compute_gpu_is_bit_two() public view {
        // Allocated by RFC prova-network/prova#7. Currently informational
        // only; a future ComputeMarketplace will gate eligibility on this
        // bit. The constant must not move.
        assertEq(registry.FEATURE_COMPUTE_GPU(), uint64(1) << 2);
    }

    function test_feature_bits_are_mutually_distinct() public view {
        uint64 pdp = registry.FEATURE_PDP();
        uint64 https = registry.FEATURE_HTTPS_SERVING();
        uint64 compute = registry.FEATURE_COMPUTE_GPU();

        assertTrue(pdp != https,    "PDP and HTTPS share a bit");
        assertTrue(pdp != compute,  "PDP and COMPUTE_GPU share a bit");
        assertTrue(https != compute,"HTTPS and COMPUTE_GPU share a bit");

        // Also assert that no two of them collide when OR'd.
        assertEq(pdp | https | compute, uint64(7));
    }
}
