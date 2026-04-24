// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

import {Test} from "forge-std/Test.sol";

import {ProvaToken} from "../src/ProvaToken.sol";
import {ProverRegistry} from "../src/ProverRegistry.sol";
import {ProverStaking} from "../src/ProverStaking.sol";
import {ContentRegistry} from "../src/ContentRegistry.sol";
import {StorageMarketplace} from "../src/StorageMarketplace.sol";

/// @notice End-to-end sanity test exercising the 4 Prova-specific contracts.
///         Deliberately lightweight: full PDP flow needs the ProofVerifier
///         to be deployed as UUPS + initialized, which is out of scope for
///         a first smoke test. We instead mock the ProofVerifier caller to
///         drive the listener hooks directly.
contract IntegrationTest is Test {
    ProvaToken token;
    ProverRegistry registry;
    ProverStaking staking;
    ContentRegistry content;
    StorageMarketplace market;

    address constant TREASURY = address(0xBEEF);
    address constant CLIENT   = address(0xC1);
    address constant PROVER   = address(0x51);

    // We stand in as the ProofVerifier so we can call the listener hooks.
    address constant FAKE_VERIFIER = address(0xFFFF);

    uint256 constant ONE_TOKEN     = 1e18;
    uint256 constant CLIENT_FUND   = 10_000 * 1e18;
    uint256 constant PROVER_FUND   = 1_000_000 * 1e18;
    uint256 constant MIN_STAKE_GIB = 100 * 1e18; // 100 PROVA per GiB

    function setUp() public {
        token = new ProvaToken(address(this));

        registry = new ProverRegistry();
        staking  = new ProverStaking(token, MIN_STAKE_GIB);
        content  = new ContentRegistry();

        market = new StorageMarketplace(
            FAKE_VERIFIER,
            token,
            registry,
            staking,
            content,
            TREASURY,
            50 * 1e18 // slash per fault
        );

        // Wire up: marketplace is authorized to control staking + content
        staking.setAuthorizedController(address(market), true);
        content.setMarketplace(address(market));

        // Fund client + prover
        token.transfer(CLIENT, CLIENT_FUND);
        token.transfer(PROVER, PROVER_FUND);
    }

    function test_ProverRegistration() public {
        // Cache feature constants first so the vm.prank isn't consumed by getters
        uint64 FEATURE_PDP = registry.FEATURE_PDP();
        uint64 FEATURE_HTTPS = registry.FEATURE_HTTPS_SERVING();
        // Unsupported feature bit (anything outside PDP + HTTPS_SERVING)
        uint64 UNKNOWN_FEATURE = 1 << 10;

        vm.prank(PROVER);
        registry.register(
            "https://prover.example/pdp",
            FEATURE_PDP | FEATURE_HTTPS,
            1_000,
            10,
            ""
        );

        assertTrue(registry.isActive(PROVER));
        assertTrue(registry.supportsFeature(PROVER, FEATURE_PDP));
        assertTrue(registry.supportsFeature(PROVER, FEATURE_HTTPS));
        assertFalse(registry.supportsFeature(PROVER, UNKNOWN_FEATURE));
    }

    function test_Staking_CommitAndRelease() public {
        uint256 stakeAmount = 200 * 1e18;

        vm.startPrank(PROVER);
        token.approve(address(staking), stakeAmount);
        staking.stake(stakeAmount);
        vm.stopPrank();

        assertEq(staking.getStake(PROVER).staked, stakeAmount);

        // Simulate marketplace committing 1 GiB (requires 100 PROVA)
        vm.prank(address(market));
        staking.commitBytes(PROVER, 1 gwei); // just use some bytes
        assertTrue(staking.getStake(PROVER).committedBytes > 0);

        vm.prank(address(market));
        staking.releaseBytes(PROVER, 1 gwei);
        assertEq(staking.getStake(PROVER).committedBytes, 0);
    }

    function test_FullDealFlow_ProposeAcceptPayFaultCompletes() public {
        uint64 FEATURE_PDP = registry.FEATURE_PDP();

        // 1. Prover registers + stakes
        vm.startPrank(PROVER);
        registry.register(
            "https://prover.example/pdp",
            FEATURE_PDP,
            1_000,
            10,
            ""
        );
        token.approve(address(staking), 500 * 1e18);
        staking.stake(500 * 1e18);
        vm.stopPrank();

        // 2. Client proposes a deal
        bytes32 commp = keccak256("test-content-commp");
        uint64 pieceSize = uint64(1024 * 1024); // 1 MiB
        uint64 duration = 30 days;
        uint256 totalPayment = 1_000 * 1e18;

        vm.startPrank(CLIENT);
        token.approve(address(market), totalPayment);
        uint256 dealId = market.proposeDeal(PROVER, commp, pieceSize, duration, totalPayment);
        vm.stopPrank();

        // Deal should be Proposed
        assertEq(uint256(market.getDeal(dealId).status), uint256(StorageMarketplace.DealStatus.Proposed));

        // 3. ProofVerifier (mocked) calls dataSetCreated to activate
        uint256 fakeDataSetId = 42;
        bytes memory extraData = abi.encode(dealId);
        vm.prank(FAKE_VERIFIER);
        market.dataSetCreated(fakeDataSetId, PROVER, extraData);

        // Deal should be Active
        StorageMarketplace.Deal memory d = market.getDeal(dealId);
        assertEq(uint256(d.status), uint256(StorageMarketplace.DealStatus.Active));
        assertEq(d.dataSetId, fakeDataSetId);
        assertGt(d.endsAt, d.startedAt);

        // Content registry should know about the content
        assertTrue(content.hasActiveDeal(commp));
        assertEq(content.getContent(commp).activeDealId, dealId);

        // Prover's committedBytes reflects the deal
        assertEq(staking.getStake(PROVER).committedBytes, pieceSize);

        // 4. Skip forward 10 days and record a proof
        vm.warp(block.timestamp + 10 days);
        uint256 proverBalanceBefore = token.balanceOf(PROVER);
        uint256 treasuryBalanceBefore = token.balanceOf(TREASURY);

        vm.prank(FAKE_VERIFIER);
        market.possessionProven(fakeDataSetId, 1, 123, 1);

        // Prover and treasury should have been paid some fraction
        uint256 proverPaid = token.balanceOf(PROVER) - proverBalanceBefore;
        uint256 treasuryPaid = token.balanceOf(TREASURY) - treasuryBalanceBefore;
        assertGt(proverPaid, 0);
        assertGt(treasuryPaid, 0);
        // 10/30 days elapsed → ~333 PROVA released. Protocol fee 1% → treasury ~3.33.
        // Check rough magnitude
        assertApproxEqRel(proverPaid + treasuryPaid, (totalPayment * 10) / 30, 0.01e18);

        // 5. Skip to end of deal + a day, then complete
        vm.warp(d.endsAt + 1);
        uint256 clientBalanceBefore = token.balanceOf(CLIENT);

        market.completeDeal(dealId);

        assertEq(uint256(market.getDeal(dealId).status), uint256(StorageMarketplace.DealStatus.Completed));
        // Prover received the rest, client got no refund (successful deal)
        assertEq(token.balanceOf(CLIENT), clientBalanceBefore);
        // Content registry cleared
        assertFalse(content.hasActiveDeal(commp));
        // Bytes freed from prover
        assertEq(staking.getStake(PROVER).committedBytes, 0);
    }

    function test_CancelProposedDealRefundsClient() public {
        uint64 FEATURE_PDP = registry.FEATURE_PDP();

        vm.startPrank(PROVER);
        registry.register("https://p.example", FEATURE_PDP, 0, 0, "");
        token.approve(address(staking), 500 * 1e18);
        staking.stake(500 * 1e18);
        vm.stopPrank();

        vm.startPrank(CLIENT);
        token.approve(address(market), 100 * 1e18);
        uint256 dealId = market.proposeDeal(PROVER, keccak256("c"), 1024, 7 days, 100 * 1e18);

        uint256 balBefore = token.balanceOf(CLIENT);
        market.cancelProposedDeal(dealId);
        uint256 balAfter = token.balanceOf(CLIENT);
        vm.stopPrank();

        assertEq(balAfter - balBefore, 100 * 1e18);
        assertEq(uint256(market.getDeal(dealId).status), uint256(StorageMarketplace.DealStatus.Cancelled));
    }

    function test_FaultDealSlashesProverAndRefundsClient() public {
        uint64 FEATURE_PDP = registry.FEATURE_PDP();

        vm.startPrank(PROVER);
        registry.register("https://p.example", FEATURE_PDP, 0, 0, "");
        token.approve(address(staking), 500 * 1e18);
        staking.stake(500 * 1e18);
        vm.stopPrank();

        vm.startPrank(CLIENT);
        token.approve(address(market), 1000 * 1e18);
        uint256 dealId = market.proposeDeal(PROVER, keccak256("d"), 1024, 30 days, 1000 * 1e18);
        vm.stopPrank();

        vm.prank(FAKE_VERIFIER);
        market.dataSetCreated(100, PROVER, abi.encode(dealId));

        // Never record any proofs; warp past MAX_PROOF_GAP
        vm.warp(block.timestamp + market.MAX_PROOF_GAP() + 1);

        uint256 clientBalBefore = token.balanceOf(CLIENT);
        uint256 proverStakedBefore = staking.getStake(PROVER).staked;

        // Anyone can fault
        market.faultDeal(dealId);

        assertEq(uint256(market.getDeal(dealId).status), uint256(StorageMarketplace.DealStatus.Slashed));

        // Client got full refund (no proofs were recorded yet)
        assertEq(token.balanceOf(CLIENT) - clientBalBefore, 1000 * 1e18);

        // Prover was slashed
        uint256 proverStakedAfter = staking.getStake(PROVER).staked;
        assertEq(proverStakedBefore - proverStakedAfter, market.slashPerFault());

        // Content cleared
        assertFalse(content.hasActiveDeal(keccak256("d")));
    }

    function test_ContentRegistryENSBinding() public {
        uint64 FEATURE_PDP = registry.FEATURE_PDP();

        // Trigger content registration by setting up a deal and activating
        vm.startPrank(PROVER);
        registry.register("https://p.example", FEATURE_PDP, 0, 0, "");
        token.approve(address(staking), 500 * 1e18);
        staking.stake(500 * 1e18);
        vm.stopPrank();

        bytes32 commp = keccak256("ens-test");
        vm.startPrank(CLIENT);
        token.approve(address(market), 100 * 1e18);
        uint256 dealId = market.proposeDeal(PROVER, commp, 1024, 7 days, 100 * 1e18);
        vm.stopPrank();

        vm.prank(FAKE_VERIFIER);
        market.dataSetCreated(1, PROVER, abi.encode(dealId));

        // Now client binds ENS
        bytes32 ensNode = keccak256("nicklas.eth");
        vm.prank(CLIENT);
        content.bindENS(commp, ensNode);

        // Reverse lookup works
        assertEq(content.resolveENS(ensNode).activeDealId, dealId);
        assertEq(content.getContent(commp).ensNode, ensNode);
    }
}
