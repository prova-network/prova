// SPDX-License-Identifier: MIT
pragma solidity ^0.8.24;

import "forge-std/Test.sol";
import "../src/ProvaToken.sol";

contract ProvaTokenTest is Test {
    ProvaToken token;
    address treasury = makeAddr("treasury");

    function setUp() public {
        token = new ProvaToken(treasury);
    }

    function test_name() public view {
        assertEq(token.name(), "Prova");
    }

    function test_symbol() public view {
        assertEq(token.symbol(), "PROVA");
    }

    function test_totalSupply() public view {
        assertEq(token.totalSupply(), 1_000_000_000 ether);
    }

    function test_treasuryBalance() public view {
        assertEq(token.balanceOf(treasury), 1_000_000_000 ether);
    }

    function test_decimals() public view {
        assertEq(token.decimals(), 18);
    }

    function test_burn() public {
        vm.prank(treasury);
        token.burn(1000 ether);
        assertEq(token.totalSupply(), 1_000_000_000 ether - 1000 ether);
    }

    function test_transfer() public {
        address alice = makeAddr("alice");
        vm.prank(treasury);
        token.transfer(alice, 500 ether);
        assertEq(token.balanceOf(alice), 500 ether);
    }

    function test_RevertWhen_zeroTreasury() public {
        vm.expectRevert("Zero address");
        new ProvaToken(address(0));
    }
}
