// SPDX-License-Identifier: MIT
pragma solidity ^0.8.24;

import "forge-std/Test.sol";
import "../src/ProvaToken.sol";
import "../src/ProvaVesting.sol";

contract ProvaVestingTest is Test {
    ProvaToken token;
    ProvaVesting vesting;

    address treasury  = makeAddr("treasury");
    address owner     = makeAddr("owner");
    address alice     = makeAddr("alice");
    address bob       = makeAddr("bob");

    uint128 constant GRANT = 20_000_000 ether; // 20M PROVA = 2% of 1B

    function setUp() public {
        token   = new ProvaToken(treasury);
        vesting = new ProvaVesting(address(token), owner);

        // Treasury approves the vesting contract via the owner flow:
        // owner pulls from treasury, but in our contract the pull is
        // from msg.sender. So owner must hold the tokens when calling
        // createSchedule. We simulate the "treasury hands tokens to
        // owner first" step here:
        vm.prank(treasury);
        token.transfer(owner, 200_000_000 ether);

        vm.prank(owner);
        token.approve(address(vesting), type(uint256).max);
    }

    // ─── Sanity ────────────────────────────────────────────────

    function test_constructor() public view {
        assertEq(address(vesting.token()), address(token));
        assertEq(vesting.owner(), owner);
        assertEq(vesting.nextId(), 1);
    }

    function test_RevertWhen_zeroToken() public {
        vm.expectRevert(bytes("Zero address"));
        new ProvaVesting(address(0), owner);
    }

    function test_RevertWhen_zeroOwner() public {
        // OZ's Ownable reverts on zero owner before our require fires;
        // accept either path.
        vm.expectRevert();
        new ProvaVesting(address(token), address(0));
    }

    // ─── Schedule creation ─────────────────────────────────────

    function test_createSchedule() public {
        vm.prank(owner);
        uint256 id = vesting.createSchedule(
            alice,
            GRANT,
            uint64(block.timestamp), // start = now
            365 days,                 // 1y cliff
            4 * 365 days,             // 4y total
            true
        );

        assertEq(id, 1);
        assertEq(vesting.nextId(), 2);
        assertEq(token.balanceOf(address(vesting)), GRANT);
        assertEq(token.balanceOf(owner), 200_000_000 ether - GRANT);

        ProvaVesting.Schedule memory s = vesting.getSchedule(id);
        assertEq(s.beneficiary, alice);
        assertEq(s.totalAmount, GRANT);
        assertTrue(s.revocable);
        assertFalse(s.revoked);
    }

    function test_RevertWhen_cliffExceedsDuration() public {
        vm.prank(owner);
        vm.expectRevert(ProvaVesting.CliffExceedsDuration.selector);
        vesting.createSchedule(alice, GRANT, uint64(block.timestamp), 4 * 365 days, 365 days, true);
    }

    function test_RevertWhen_zeroBeneficiary() public {
        vm.prank(owner);
        vm.expectRevert(ProvaVesting.InvalidSchedule.selector);
        vesting.createSchedule(address(0), GRANT, 0, 0, 365 days, false);
    }

    function test_RevertWhen_zeroAmount() public {
        vm.prank(owner);
        vm.expectRevert(ProvaVesting.InvalidSchedule.selector);
        vesting.createSchedule(alice, 0, 0, 0, 365 days, false);
    }

    function test_RevertWhen_nonOwnerCreates() public {
        vm.prank(alice);
        vm.expectRevert();
        vesting.createSchedule(alice, GRANT, 0, 0, 365 days, false);
    }

    // ─── Claim / vesting curve ─────────────────────────────────

    function _fourYearOneYearCliff() internal returns (uint256 id) {
        vm.prank(owner);
        id = vesting.createSchedule(
            alice,
            GRANT,
            uint64(block.timestamp),
            365 days,
            4 * 365 days,
            true
        );
    }

    function test_claimable_beforeCliff() public {
        uint256 id = _fourYearOneYearCliff();
        skip(364 days);
        assertEq(vesting.claimable(id), 0, "should be 0 before cliff");
    }

    function test_claimable_atCliff() public {
        uint256 id = _fourYearOneYearCliff();
        skip(365 days);
        // 1 year of 4 = 25% vested at the cliff
        assertEq(vesting.claimable(id), GRANT / 4);
    }

    function test_claimable_halfWay() public {
        uint256 id = _fourYearOneYearCliff();
        skip(2 * 365 days);
        assertEq(vesting.claimable(id), GRANT / 2);
    }

    function test_claimable_fullyVested() public {
        uint256 id = _fourYearOneYearCliff();
        skip(4 * 365 days);
        assertEq(vesting.claimable(id), GRANT);
    }

    function test_claimable_afterFullyVested() public {
        uint256 id = _fourYearOneYearCliff();
        skip(10 * 365 days); // way past
        assertEq(vesting.claimable(id), GRANT);
    }

    function test_claim_atCliff() public {
        uint256 id = _fourYearOneYearCliff();
        skip(365 days);

        vm.prank(alice);
        uint128 claimed = vesting.claim(id);

        assertEq(claimed, GRANT / 4);
        assertEq(token.balanceOf(alice), GRANT / 4);
        assertEq(vesting.claimable(id), 0); // already claimed
    }

    function test_claim_progressive() public {
        uint256 id = _fourYearOneYearCliff();

        // Year 1 cliff
        skip(365 days);
        vm.prank(alice);
        vesting.claim(id);
        assertEq(token.balanceOf(alice), GRANT / 4);

        // Year 2
        skip(365 days);
        vm.prank(alice);
        vesting.claim(id);
        assertEq(token.balanceOf(alice), GRANT / 2);

        // Year 4 (full)
        skip(2 * 365 days);
        vm.prank(alice);
        vesting.claim(id);
        assertEq(token.balanceOf(alice), GRANT);
    }

    function test_RevertWhen_nonBeneficiaryClaims() public {
        uint256 id = _fourYearOneYearCliff();
        skip(365 days);
        vm.prank(bob);
        vm.expectRevert(ProvaVesting.NotBeneficiary.selector);
        vesting.claim(id);
    }

    function test_RevertWhen_claimNothing() public {
        uint256 id = _fourYearOneYearCliff();
        skip(364 days); // before cliff
        vm.prank(alice);
        vm.expectRevert(ProvaVesting.NothingToClaim.selector);
        vesting.claim(id);
    }

    // ─── Revoke ────────────────────────────────────────────────

    function test_revoke_returnsUnvested() public {
        uint256 id = _fourYearOneYearCliff();
        skip(2 * 365 days);

        // 50% should be vested at this point
        uint256 ownerBefore = token.balanceOf(owner);
        vm.prank(owner);
        vesting.revoke(id);

        // Half goes back to owner
        assertEq(token.balanceOf(owner), ownerBefore + GRANT / 2);

        // Alice can still claim her vested half
        assertEq(vesting.claimable(id), GRANT / 2);

        vm.prank(alice);
        vesting.claim(id);
        assertEq(token.balanceOf(alice), GRANT / 2);
    }

    function test_RevertWhen_revokeNonRevocable() public {
        vm.prank(owner);
        uint256 id = vesting.createSchedule(alice, GRANT, 0, 0, 365 days, false);

        vm.prank(owner);
        vm.expectRevert(ProvaVesting.NotRevocable.selector);
        vesting.revoke(id);
    }

    function test_RevertWhen_revokeTwice() public {
        uint256 id = _fourYearOneYearCliff();
        skip(2 * 365 days);
        vm.prank(owner);
        vesting.revoke(id);
        vm.prank(owner);
        vm.expectRevert(ProvaVesting.AlreadyRevoked.selector);
        vesting.revoke(id);
    }

    // ─── Acceleration ──────────────────────────────────────────

    function test_accelerate_advancesVesting() public {
        uint256 id = _fourYearOneYearCliff();

        // At cliff (1y), vested = GRANT/4. Accelerate by 6 months
        // (180 days). Acceleration shortens duration: 4y → 4y - 180d.
        // Cliff: 365d > 180d, so cliff becomes 365d - 180d = 185d (now
        //   already past since elapsed = 365d).
        // Vested at the same block.timestamp:
        //   elapsed / new_duration = 365d / (4*365d - 180d) of GRANT.
        skip(365 days);
        vm.prank(owner);
        vesting.accelerate(id, 180 days);

        uint64 newDuration = uint64(4 * 365 days - 180 days);
        uint128 expected = uint128((uint256(GRANT) * 365 days) / newDuration);
        assertEq(vesting.claimable(id), expected);
    }

    function test_accelerate_overDuration_fullyVests() public {
        uint256 id = _fourYearOneYearCliff();
        skip(180 days); // pre-cliff
        vm.prank(owner);
        vesting.accelerate(id, 10 * 365 days); // way past total

        // Should be fully vested now
        assertEq(vesting.claimable(id), GRANT);
    }

    // ─── Multi-schedule + claimAll ─────────────────────────────

    function test_claimAll() public {
        // Two schedules for alice
        vm.startPrank(owner);
        uint256 id1 = vesting.createSchedule(alice, GRANT, 0, 0, 365 days, false);     // 0 cliff
        uint256 id2 = vesting.createSchedule(alice, GRANT, 0, 0, 2 * 365 days, false); // 0 cliff
        vm.stopPrank();

        skip(365 days); // 1y in
        // id1: fully vested (1y / 1y)
        // id2: half vested (1y / 2y)
        vm.prank(alice);
        uint128 total = vesting.claimAll();
        assertEq(total, GRANT + GRANT / 2);
        assertEq(token.balanceOf(alice), GRANT + GRANT / 2);

        // Subsequent claimAll should revert (nothing more vested yet)
        vm.prank(alice);
        vm.expectRevert(ProvaVesting.NothingToClaim.selector);
        vesting.claimAll();

        // After another year, id2 fully vested
        skip(365 days);
        vm.prank(alice);
        uint128 second = vesting.claimAll();
        assertEq(second, GRANT / 2);
        assertEq(token.balanceOf(alice), 2 * GRANT);

        // Silence "unused" warning
        id1; id2;
    }

    function test_getSchedulesByBeneficiary() public {
        vm.startPrank(owner);
        vesting.createSchedule(alice, GRANT, 0, 0, 365 days, false);
        vesting.createSchedule(alice, GRANT, 0, 0, 365 days, false);
        vesting.createSchedule(bob,   GRANT, 0, 0, 365 days, false);
        vm.stopPrank();

        uint256[] memory aliceIds = vesting.getSchedulesByBeneficiary(alice);
        uint256[] memory bobIds   = vesting.getSchedulesByBeneficiary(bob);

        assertEq(aliceIds.length, 2);
        assertEq(bobIds.length, 1);
        assertEq(aliceIds[0], 1);
        assertEq(aliceIds[1], 2);
        assertEq(bobIds[0],   3);
    }
}
