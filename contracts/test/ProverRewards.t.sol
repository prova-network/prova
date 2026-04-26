// SPDX-License-Identifier: MIT
pragma solidity ^0.8.24;

import "forge-std/Test.sol";
import "../src/ProverRewards.sol";
import "../src/ProvaToken.sol";

contract ProverRewardsTest is Test {
    ProvaToken     prova;
    ProverRewards  rewards;

    address treasury = makeAddr("treasury");
    address owner    = makeAddr("owner");
    address marketplace = makeAddr("marketplace");
    address alice    = makeAddr("alice");
    address bob      = makeAddr("bob");
    address carol    = makeAddr("carol");
    address client1  = makeAddr("client1");
    address client2  = makeAddr("client2");

    uint64 constant GENESIS = 1_700_000_000;

    function setUp() public {
        prova = new ProvaToken(treasury);
        // Use a fixed genesis so we can warp deterministically
        vm.warp(GENESIS);
        rewards = new ProverRewards(prova, owner, GENESIS);

        // Treasury seeds the rewards contract with the full 50M emission bucket
        vm.prank(treasury);
        prova.transfer(address(rewards), 50_000_000 ether);

        // Owner authorizes the marketplace to record proofs
        vm.prank(owner);
        rewards.setMarketplace(marketplace);
    }

    // ─── Constructor + admin ──────────────────────────────────────

    function test_constructor() public view {
        assertEq(address(rewards.prova()), address(prova));
        assertEq(rewards.owner(), owner);
        assertEq(rewards.genesisTime(), GENESIS);
        assertEq(rewards.marketplace(), marketplace);
        assertEq(prova.balanceOf(address(rewards)), 50_000_000 ether);
    }

    function test_RevertWhen_zeroToken() public {
        vm.expectRevert(ProverRewards.ZeroAddress.selector);
        new ProverRewards(IERC20(address(0)), owner, 0);
    }

    function test_setRedundancyCap_invalid() public {
        vm.prank(owner);
        vm.expectRevert(ProverRewards.InvalidParam.selector);
        rewards.setRedundancyCap(0);
        vm.prank(owner);
        vm.expectRevert(ProverRewards.InvalidParam.selector);
        rewards.setRedundancyCap(17);
    }

    function test_setRedundancyCap_valid() public {
        vm.prank(owner);
        rewards.setRedundancyCap(8);
        assertEq(rewards.redundancyCap(), 8);
    }

    function test_setQualityCutoff_invalid() public {
        vm.prank(owner);
        vm.expectRevert(ProverRewards.InvalidParam.selector);
        rewards.setQualityCutoff(5001);
    }

    // ─── recordProof: anti-gaming ──────────────────────────────────

    function test_RevertWhen_nonMarketplaceRecords() public {
        vm.expectRevert(ProverRewards.NotMarketplace.selector);
        rewards.recordProof(alice, client1, keccak256("p"), 1024);
    }

    function test_recordProof_creditsValidProof() public {
        bytes32 piece = keccak256("piece-1");
        vm.prank(marketplace);
        rewards.recordProof(alice, client1, piece, 1 << 30); // 1 GiB

        uint256 epoch = rewards.currentEpoch();
        assertEq(rewards.bytesByEpochProver(epoch, alice), 1 << 30);
        assertEq(rewards.totalBytesByEpoch(epoch), 1 << 30);
    }

    function test_RevertWhen_selfDealing() public {
        // Prover and client are the same address — anti-gaming
        bytes32 piece = keccak256("self-deal");
        vm.prank(marketplace);
        vm.expectRevert(ProverRewards.SelfDealing.selector);
        rewards.recordProof(alice, alice, piece, 1 << 30);
    }

    function test_recordProof_sponsoredDealsDontCount() public {
        bytes32 piece = keccak256("sponsored");
        vm.prank(marketplace);
        rewards.recordProof(alice, address(0), piece, 1 << 30);

        uint256 epoch = rewards.currentEpoch();
        // No bytes counted, but quality should be tracked
        assertEq(rewards.bytesByEpochProver(epoch, alice), 0);
        assertEq(rewards.totalBytesByEpoch(epoch), 0);
        // quality successes incremented
        (, uint64 successes, uint64 failures) = rewards.quality(alice);
        assertEq(successes, 1);
        assertEq(failures, 0);
    }

    function test_recordProof_doubleProofSameEpoch_doesntDoubleCount() public {
        bytes32 piece = keccak256("p");
        vm.startPrank(marketplace);
        rewards.recordProof(alice, client1, piece, 1 << 30);
        rewards.recordProof(alice, client1, piece, 1 << 30); // same epoch, same piece, same prover
        vm.stopPrank();

        uint256 epoch = rewards.currentEpoch();
        // Only counted once
        assertEq(rewards.bytesByEpochProver(epoch, alice), 1 << 30);
    }

    function test_recordProof_redundancyCapEnforced() public {
        bytes32 piece = keccak256("p");
        vm.startPrank(marketplace);
        // 4 different provers all post proofs for the same piece
        rewards.recordProof(alice, client1, piece, 1 << 30);
        rewards.recordProof(bob,   client1, piece, 1 << 30);
        rewards.recordProof(carol, client1, piece, 1 << 30);
        rewards.recordProof(makeAddr("dave"), client1, piece, 1 << 30);
        // 5th prover hits the cap, doesn't count
        address eve = makeAddr("eve");
        rewards.recordProof(eve, client1, piece, 1 << 30);
        vm.stopPrank();

        uint256 epoch = rewards.currentEpoch();
        assertEq(rewards.totalBytesByEpoch(epoch), 4 * (1 << 30)); // 4 not 5
        assertEq(rewards.bytesByEpochProver(epoch, eve), 0);
    }

    function test_redundancyCap_canBeRaised() public {
        vm.prank(owner);
        rewards.setRedundancyCap(8);

        bytes32 piece = keccak256("p");
        vm.startPrank(marketplace);
        for (uint256 i = 0; i < 6; i++) {
            address p = makeAddr(string(abi.encodePacked("p", vm.toString(i))));
            rewards.recordProof(p, client1, piece, 1 << 30);
        }
        vm.stopPrank();

        uint256 epoch = rewards.currentEpoch();
        assertEq(rewards.totalBytesByEpoch(epoch), 6 * (1 << 30));
    }

    // ─── recordProof: quality tracking ─────────────────────────────

    function test_recordMiss_dropsQuality() public {
        vm.startPrank(marketplace);
        rewards.recordProof(alice, client1, keccak256("p1"), 1);
        rewards.recordMiss(alice);
        rewards.recordMiss(alice);
        vm.stopPrank();

        (, uint64 successes, uint64 failures) = rewards.quality(alice);
        assertEq(successes, 1);
        assertEq(failures, 2);
    }

    function test_qualityWindow_resetsAfter30d() public {
        vm.prank(marketplace);
        rewards.recordMiss(alice);
        (uint64 ws, , uint64 f1) = rewards.quality(alice);
        assertEq(f1, 1);

        // Skip 31 days, the window should reset
        vm.warp(block.timestamp + 31 days);
        vm.prank(marketplace);
        rewards.recordProof(alice, client1, keccak256("p"), 1 << 30);
        (uint64 ws2, uint64 s2, uint64 f2) = rewards.quality(alice);
        assertEq(s2, 1);
        assertEq(f2, 0);
        assertGt(ws2, ws);
    }

    // ─── Reward math ───────────────────────────────────────────────

    function test_rewardOf_proportionalShare() public {
        bytes32 piece = keccak256("p");
        vm.startPrank(marketplace);
        rewards.recordProof(alice, client1, piece, 3 << 30); // 3 GiB
        rewards.recordProof(bob,   client2, piece, 1 << 30); // 1 GiB
        vm.stopPrank();

        uint256 epoch = rewards.currentEpoch();
        uint256 aliceRew = rewards.rewardOf(alice, epoch);
        uint256 bobRew   = rewards.rewardOf(bob, epoch);

        assertGt(aliceRew, 0);
        assertGt(bobRew, 0);
        // alice should earn ~3x bob
        assertApproxEqRel(aliceRew, 3 * bobRew, 0.01e18);

        // Combined ≈ one epoch's emission for year 0 (12.5M / 52.14)
        uint256 expectedPerEpoch = (uint256(12_500_000 ether) * 7 days) / 365 days;
        assertApproxEqRel(aliceRew + bobRew, expectedPerEpoch, 0.01e18);
    }

    function test_rewardOf_year2EmissionLower() public {
        // Drive enough time forward to be in year 2
        vm.warp(GENESIS + 365 days + 7 days);

        bytes32 piece = keccak256("p");
        vm.prank(marketplace);
        rewards.recordProof(alice, client1, piece, 1 << 30);

        uint256 epoch = rewards.currentEpoch();
        uint256 reward = rewards.rewardOf(alice, epoch);

        // year 1 emission per epoch
        uint256 y1 = (uint256(12_500_000 ether) * 7 days) / 365 days;
        // year 2 emission per epoch
        uint256 y2 = (uint256(11_000_000 ether) * 7 days) / 365 days;

        assertLt(reward, y1);
        assertApproxEqRel(reward, y2, 0.01e18);
    }

    function test_rewardOf_year9_returnsZero() public {
        // Past the 8-year emission window
        vm.warp(GENESIS + (8 * 365 days) + 7 days);

        bytes32 piece = keccak256("p");
        vm.prank(marketplace);
        rewards.recordProof(alice, client1, piece, 1 << 30);

        uint256 epoch = rewards.currentEpoch();
        assertEq(rewards.rewardOf(alice, epoch), 0);
    }

    function test_rewardOf_noBytesNoReward() public view {
        assertEq(rewards.rewardOf(alice, 0), 0);
    }

    function test_qualityMultiplier_halvesReward() public {
        // 1 success then 1 failure → 50% miss rate, way over the 5% cutoff
        vm.startPrank(marketplace);
        rewards.recordProof(alice, client1, keccak256("p"), 1 << 30);
        rewards.recordMiss(alice);
        vm.stopPrank();

        uint256 epoch = rewards.currentEpoch();
        uint256 reward = rewards.rewardOf(alice, epoch);

        uint256 perEpoch = (uint256(12_500_000 ether) * 7 days) / 365 days;
        // Should be HALF of the full epoch emission (alice is the only prover)
        assertApproxEqRel(reward, perEpoch / 2, 0.01e18);
    }

    // ─── Claim flow ────────────────────────────────────────────────

    function test_claim_revertsBeforeVested() public {
        bytes32 piece = keccak256("p");
        vm.prank(marketplace);
        rewards.recordProof(alice, client1, piece, 1 << 30);

        uint256 epoch = rewards.currentEpoch();
        vm.prank(alice);
        vm.expectRevert(ProverRewards.EpochNotVested.selector);
        rewards.claim(epoch);
    }

    function test_claim_succeedsAfterVesting() public {
        bytes32 piece = keccak256("p");
        vm.prank(marketplace);
        rewards.recordProof(alice, client1, piece, 1 << 30);

        uint256 epoch = rewards.currentEpoch();
        // Skip past epoch end + vesting buffer
        vm.warp(GENESIS + (epoch + 1) * 7 days + 30 days + 1);

        uint256 expected = rewards.rewardOf(alice, epoch);
        assertGt(expected, 0);

        vm.prank(alice);
        uint256 claimed = rewards.claim(epoch);
        assertEq(claimed, expected);
        assertEq(prova.balanceOf(alice), expected);
        assertTrue(rewards.claimed(epoch, alice));
    }

    function test_claim_RevertWhen_alreadyClaimed() public {
        bytes32 piece = keccak256("p");
        vm.prank(marketplace);
        rewards.recordProof(alice, client1, piece, 1 << 30);

        uint256 epoch = rewards.currentEpoch();
        vm.warp(GENESIS + (epoch + 1) * 7 days + 30 days + 1);

        vm.startPrank(alice);
        rewards.claim(epoch);
        vm.expectRevert(ProverRewards.AlreadyClaimed.selector);
        rewards.claim(epoch);
        vm.stopPrank();
    }

    function test_claim_RevertWhen_nothingToClaim() public {
        // alice never proved anything; epoch has zero bytes
        vm.warp(GENESIS + 7 days + 30 days + 1);
        vm.prank(alice);
        vm.expectRevert(ProverRewards.NothingToClaim.selector);
        rewards.claim(0);
    }

    function test_claimRange_aggregatesMultipleEpochs() public {
        // Prove in epoch 0 and epoch 2 (skip epoch 1 to test sparse claims)
        vm.prank(marketplace);
        rewards.recordProof(alice, client1, keccak256("p0"), 1 << 30);

        // Move to epoch 2
        vm.warp(GENESIS + 2 * 7 days + 1);
        vm.prank(marketplace);
        rewards.recordProof(alice, client1, keccak256("p2"), 2 << 30);

        // Skip to past vesting of epoch 2
        vm.warp(GENESIS + 3 * 7 days + 30 days + 1);

        uint256 r0 = rewards.rewardOf(alice, 0);
        uint256 r2 = rewards.rewardOf(alice, 2);

        vm.prank(alice);
        uint256 total = rewards.claimRange(0, 2);

        assertEq(total, r0 + r2);
        assertEq(prova.balanceOf(alice), r0 + r2);
        assertTrue(rewards.claimed(0, alice));
        assertFalse(rewards.claimed(1, alice)); // empty epoch, didn't get marked
        assertTrue(rewards.claimed(2, alice));
    }

    // ─── Sanity: full 50M is held + emission curve totals ──────────

    function test_yearlyEmission_sumsTo50M() public view {
        uint256 sum;
        for (uint8 i = 0; i < 8; i++) {
            sum += rewards.yearlyEmission(i);
        }
        assertEq(sum, 50_000_000 ether);
    }

    function test_isClaimable_logic() public {
        assertFalse(rewards.isClaimable(0));
        // Skip 7d (epoch ends) + 29d (still inside vesting buffer)
        vm.warp(GENESIS + 7 days + 29 days);
        assertFalse(rewards.isClaimable(0));
        vm.warp(GENESIS + 7 days + 30 days + 1);
        assertTrue(rewards.isClaimable(0));
    }
}
