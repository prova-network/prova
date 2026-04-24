// SPDX-License-Identifier: MIT
pragma solidity ^0.8.24;

import "forge-std/Test.sol";
import "../../src/ProvaToken.sol";
import "../../src/legacy/ProvaVesting.sol";

contract ProvaVestingTest is Test {
    ProvaToken token;
    ProvaVesting vesting;
    address owner = makeAddr("owner");
    address alice = makeAddr("alice");
    address bob = makeAddr("bob");

    uint256 constant TOTAL = 1_000_000 ether;
    uint256 constant TGE = 1_000_000; // timestamp

    function setUp() public {
        vm.startPrank(owner);
        token = new ProvaToken(owner);
        vesting = new ProvaVesting(address(token));
        vesting.setTGE(TGE);
        token.approve(address(vesting), type(uint256).max);
        vm.stopPrank();
    }

    function test_createSchedule() public {
        vm.prank(owner);
        vesting.createSchedule(alice, TOTAL, 2500, 365 days, 730 days, false);

        (uint256 total,,uint256 tgeUnlock,,,,) = vesting.schedules(alice);
        assertEq(total, TOTAL);
        assertEq(tgeUnlock, TOTAL * 2500 / 10000); // 25%
    }

    function test_claimAtTGE() public {
        vm.prank(owner);
        vesting.createSchedule(alice, TOTAL, 2500, 365 days, 730 days, false);

        // Warp to TGE
        vm.warp(TGE);

        uint256 claimable = vesting.claimable(alice);
        assertEq(claimable, 250_000 ether); // 25% of 1M

        vm.prank(alice);
        vesting.claim();
        assertEq(token.balanceOf(alice), 250_000 ether);
    }

    function test_nothingDuringCliff() public {
        vm.prank(owner);
        vesting.createSchedule(alice, TOTAL, 0, 365 days, 730 days, false);

        // Warp to middle of cliff (6 months after TGE)
        vm.warp(TGE + 180 days);

        assertEq(vesting.claimable(alice), 0);
    }

    function test_linearVesting() public {
        vm.prank(owner);
        // 0% TGE, 1 year cliff, 2 year vest
        vesting.createSchedule(alice, TOTAL, 0, 365 days, 730 days, false);

        // Warp to halfway through vesting (cliff + 1 year)
        vm.warp(TGE + 365 days + 365 days);

        uint256 claimable = vesting.claimable(alice);
        assertEq(claimable, TOTAL / 2); // 50% vested
    }

    function test_fullVest() public {
        vm.prank(owner);
        vesting.createSchedule(alice, TOTAL, 2500, 180 days, 540 days, false);

        // Warp past vest end
        vm.warp(TGE + 180 days + 540 days + 1);

        assertEq(vesting.claimable(alice), TOTAL);

        vm.prank(alice);
        vesting.claim();
        assertEq(token.balanceOf(alice), TOTAL);
    }

    function test_revoke() public {
        vm.prank(owner);
        vesting.createSchedule(alice, TOTAL, 0, 365 days, 730 days, true);

        // Warp to cliff + 1 year (50% vested)
        vm.warp(TGE + 365 days + 365 days);

        uint256 ownerBefore = token.balanceOf(owner);

        vm.prank(owner);
        vesting.revoke(alice);

        // Owner should get ~50% back (unvested)
        uint256 returned = token.balanceOf(owner) - ownerBefore;
        assertApproxEqRel(returned, TOTAL / 2, 0.01e18); // within 1%
    }

    function test_RevertWhen_revokeNonRevocable() public {
        vm.prank(owner);
        vesting.createSchedule(alice, TOTAL, 0, 365 days, 730 days, false);

        vm.prank(owner);
        vm.expectRevert("Not revocable");
        vesting.revoke(alice);
    }

    function test_RevertWhen_doubleSchedule() public {
        vm.startPrank(owner);
        vesting.createSchedule(alice, TOTAL, 0, 365 days, 730 days, false);
        vm.expectRevert("Schedule exists");
        vesting.createSchedule(alice, TOTAL, 0, 365 days, 730 days, false);
        vm.stopPrank();
    }

    function test_RevertWhen_claimBeforeTGE() public {
        vm.prank(owner);
        vesting.createSchedule(alice, TOTAL, 2500, 0, 180 days, false);

        vm.warp(TGE - 1);
        vm.prank(alice);
        vm.expectRevert("Nothing to claim");
        vesting.claim();
    }
}
