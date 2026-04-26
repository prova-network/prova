// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

import {Test} from "forge-std/Test.sol";
import {ERC20} from "@openzeppelin/contracts/token/ERC20/ERC20.sol";
import {IERC20} from "@openzeppelin/contracts/token/ERC20/IERC20.sol";

import {ProvaToken} from "../src/ProvaToken.sol";
import {ProverRegistry} from "../src/ProverRegistry.sol";
import {ProverStaking} from "../src/ProverStaking.sol";
import {ContentRegistry} from "../src/ContentRegistry.sol";
import {StorageMarketplace} from "../src/StorageMarketplace.sol";

/// @notice Mock USDC. 6 decimals like real USDC, but for these tests we
///         use 18 to keep the math identical to the prior fixture; we
///         only care about the dual-token semantics here.
contract MockUSDC is ERC20 {
    constructor() ERC20("USD Coin", "USDC") {
        _mint(msg.sender, 100_000_000 ether);
    }
}

/// @notice End-to-end sanity test for the v2 (dual-token) economic model:
///         - Clients pay USDC. Provers earn USDC.
///         - Provers stake PROVA. Slashing burns PROVA stake.
///
///         The ProofVerifier is mocked via FAKE_VERIFIER; we drive the
///         listener hooks directly. Full UUPS-proxied PDPVerifier flow is
///         covered by separate Foundry fork tests against Base Sepolia.
contract IntegrationTest is Test {
    ProvaToken          prova;
    MockUSDC            usdc;
    ProverRegistry      registry;
    ProverStaking       staking;
    ContentRegistry     content;
    StorageMarketplace  market;

    address constant TREASURY = address(0xBEEF);
    address constant CLIENT   = address(0xC1);
    address constant PROVER   = address(0x51);

    address constant FAKE_VERIFIER = address(0xFFFF);

    uint256 constant ONE_TOKEN     = 1e18;
    uint256 constant CLIENT_USDC   = 100_000 * 1e18;
    uint256 constant PROVER_PROVA  = 1_000_000 * 1e18; // 1M PROVA = 1% of supply
    uint256 constant MIN_STAKE_GIB = 100 * 1e18;       // 100 PROVA per GiB

    function setUp() public {
        prova = new ProvaToken(address(this));
        usdc  = new MockUSDC();

        registry = new ProverRegistry();
        staking  = new ProverStaking(IERC20(address(prova)), MIN_STAKE_GIB);
        content  = new ContentRegistry();

        market = new StorageMarketplace(
            FAKE_VERIFIER,
            IERC20(address(usdc)),
            registry,
            staking,
            content,
            TREASURY,
            50 * 1e18 // slash per fault, in PROVA
        );

        staking.setAuthorizedController(address(market), true);
        content.setMarketplace(address(market));

        // Fund client (USDC) and prover (PROVA stake + a little USDC for gas)
        usdc.transfer(CLIENT, CLIENT_USDC);
        prova.transfer(PROVER, PROVER_PROVA);
    }

    function test_ProverRegistration() public {
        uint64 FEATURE_PDP = registry.FEATURE_PDP();
        uint64 FEATURE_HTTPS = registry.FEATURE_HTTPS_SERVING();
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
        uint256 stakeAmount = 200 * 1e18; // 200 PROVA

        vm.startPrank(PROVER);
        prova.approve(address(staking), stakeAmount);
        staking.stake(stakeAmount);
        vm.stopPrank();

        assertEq(staking.getStake(PROVER).staked, stakeAmount);

        vm.prank(address(market));
        staking.commitBytes(PROVER, 1 gwei);
        assertTrue(staking.getStake(PROVER).committedBytes > 0);

        vm.prank(address(market));
        staking.releaseBytes(PROVER, 1 gwei);
        assertEq(staking.getStake(PROVER).committedBytes, 0);
    }

    function test_FullDealFlow_ProposeAcceptPayFaultCompletes() public {
        uint64 FEATURE_PDP = registry.FEATURE_PDP();

        // 1. Prover registers + stakes 500 PROVA
        vm.startPrank(PROVER);
        registry.register("https://prover.example/pdp", FEATURE_PDP, 1_000, 10, "");
        prova.approve(address(staking), 500 * 1e18);
        staking.stake(500 * 1e18);
        vm.stopPrank();

        // 2. Client proposes a deal, paying 1000 USDC
        bytes32 commp = keccak256("test-content-commp");
        uint64 pieceSize = uint64(1024 * 1024);
        uint64 duration  = 30 days;
        uint256 totalPayment = 1_000 * 1e18;

        vm.startPrank(CLIENT);
        usdc.approve(address(market), totalPayment);
        uint256 dealId = market.proposeDeal(PROVER, commp, pieceSize, duration, totalPayment);
        vm.stopPrank();

        assertEq(uint256(market.getDeal(dealId).status), uint256(StorageMarketplace.DealStatus.Proposed));

        // 3. ProofVerifier (mocked) activates the deal
        uint256 fakeDataSetId = 42;
        bytes memory extraData = abi.encode(dealId);
        vm.prank(FAKE_VERIFIER);
        market.dataSetCreated(fakeDataSetId, PROVER, extraData);

        StorageMarketplace.Deal memory d = market.getDeal(dealId);
        assertEq(uint256(d.status), uint256(StorageMarketplace.DealStatus.Active));
        assertEq(d.dataSetId, fakeDataSetId);
        assertGt(d.endsAt, d.startedAt);

        assertTrue(content.hasActiveDeal(commp));
        assertEq(content.getContent(commp).activeDealId, dealId);
        assertEq(staking.getStake(PROVER).committedBytes, pieceSize);

        // 4. Skip 10 days, record a proof. USDC streams to prover + treasury.
        vm.warp(block.timestamp + 10 days);
        uint256 proverUsdcBefore   = usdc.balanceOf(PROVER);
        uint256 treasuryUsdcBefore = usdc.balanceOf(TREASURY);

        vm.prank(FAKE_VERIFIER);
        market.possessionProven(fakeDataSetId, 1, 123, 1);

        uint256 proverPaid   = usdc.balanceOf(PROVER) - proverUsdcBefore;
        uint256 treasuryPaid = usdc.balanceOf(TREASURY) - treasuryUsdcBefore;
        assertGt(proverPaid, 0);
        assertGt(treasuryPaid, 0);
        // 10/30 days released → ~333 USDC. 1% protocol fee on that.
        assertApproxEqRel(proverPaid + treasuryPaid, (totalPayment * 10) / 30, 0.01e18);

        // 5. Skip to deal end and complete
        vm.warp(d.endsAt + 1);
        uint256 clientUsdcBefore = usdc.balanceOf(CLIENT);

        market.completeDeal(dealId);

        assertEq(uint256(market.getDeal(dealId).status), uint256(StorageMarketplace.DealStatus.Completed));
        // Successful deal → no client refund
        assertEq(usdc.balanceOf(CLIENT), clientUsdcBefore);
        assertFalse(content.hasActiveDeal(commp));
        assertEq(staking.getStake(PROVER).committedBytes, 0);
    }

    function test_CancelProposedDealRefundsClient() public {
        uint64 FEATURE_PDP = registry.FEATURE_PDP();

        vm.startPrank(PROVER);
        registry.register("https://p.example", FEATURE_PDP, 0, 0, "");
        prova.approve(address(staking), 500 * 1e18);
        staking.stake(500 * 1e18);
        vm.stopPrank();

        vm.startPrank(CLIENT);
        usdc.approve(address(market), 100 * 1e18);
        uint256 dealId = market.proposeDeal(PROVER, keccak256("c"), 1024, 7 days, 100 * 1e18);

        uint256 usdcBefore = usdc.balanceOf(CLIENT);
        market.cancelProposedDeal(dealId);
        uint256 usdcAfter = usdc.balanceOf(CLIENT);
        vm.stopPrank();

        assertEq(usdcAfter - usdcBefore, 100 * 1e18);
        assertEq(uint256(market.getDeal(dealId).status), uint256(StorageMarketplace.DealStatus.Cancelled));
    }

    function test_FaultDealSlashesProverAndRefundsClient() public {
        uint64 FEATURE_PDP = registry.FEATURE_PDP();

        vm.startPrank(PROVER);
        registry.register("https://p.example", FEATURE_PDP, 0, 0, "");
        prova.approve(address(staking), 500 * 1e18);
        staking.stake(500 * 1e18);
        vm.stopPrank();

        vm.startPrank(CLIENT);
        usdc.approve(address(market), 1000 * 1e18);
        uint256 dealId = market.proposeDeal(PROVER, keccak256("d"), 1024, 30 days, 1000 * 1e18);
        vm.stopPrank();

        vm.prank(FAKE_VERIFIER);
        market.dataSetCreated(100, PROVER, abi.encode(dealId));

        // Never record any proofs; warp past MAX_PROOF_GAP
        vm.warp(block.timestamp + market.MAX_PROOF_GAP() + 1);

        uint256 clientUsdcBefore   = usdc.balanceOf(CLIENT);
        uint256 proverStakedBefore = staking.getStake(PROVER).staked;

        market.faultDeal(dealId);

        assertEq(uint256(market.getDeal(dealId).status), uint256(StorageMarketplace.DealStatus.Slashed));

        // Client got the full USDC payment refunded (no proofs were recorded)
        assertEq(usdc.balanceOf(CLIENT) - clientUsdcBefore, 1000 * 1e18);

        // Prover was slashed in PROVA
        uint256 proverStakedAfter = staking.getStake(PROVER).staked;
        assertEq(proverStakedBefore - proverStakedAfter, market.slashPerFault());

        assertFalse(content.hasActiveDeal(keccak256("d")));
    }

    function test_ContentRegistryENSBinding() public {
        uint64 FEATURE_PDP = registry.FEATURE_PDP();

        vm.startPrank(PROVER);
        registry.register("https://p.example", FEATURE_PDP, 0, 0, "");
        prova.approve(address(staking), 500 * 1e18);
        staking.stake(500 * 1e18);
        vm.stopPrank();

        bytes32 commp = keccak256("ens-test");
        vm.startPrank(CLIENT);
        usdc.approve(address(market), 100 * 1e18);
        uint256 dealId = market.proposeDeal(PROVER, commp, 1024, 7 days, 100 * 1e18);
        vm.stopPrank();

        vm.prank(FAKE_VERIFIER);
        market.dataSetCreated(1, PROVER, abi.encode(dealId));

        // Client binds ENS to their deal's content
        bytes32 ensNode = keccak256("example.eth");
        vm.prank(CLIENT);
        content.bindENS(commp, ensNode);

        assertEq(content.resolveENS(ensNode).activeDealId, dealId);
        assertEq(content.getContent(commp).ensNode, ensNode);
    }
}
