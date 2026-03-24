// SPDX-License-Identifier: MIT
pragma solidity ^0.8.24;

import "forge-std/Test.sol";
import "../src/ProvaToken.sol";
import "../src/ProvaCrowdsale.sol";

/// @dev Mock USDC with 6 decimals
contract MockUSDC is ERC20 {
    constructor() ERC20("USD Coin", "USDC") {
        _mint(msg.sender, 100_000_000 * 1e6); // 100M USDC
    }

    function decimals() public pure override returns (uint8) { return 6; }

    function mint(address to, uint256 amount) external {
        _mint(to, amount);
    }
}

contract ProvaCrowdsaleTest is Test {
    ProvaToken prova;
    MockUSDC usdc;
    ProvaCrowdsale sale;

    address owner = makeAddr("owner");
    address buyer1 = makeAddr("buyer1");
    address buyer2 = makeAddr("buyer2");
    address vesting = makeAddr("vesting"); // placeholder for test

    // Rate: 66.666666 PROVA per 1 USDC (= $0.015 per PROVA)
    // In 18 decimals: 66_666666_000000_000000 = 66.666666e18
    uint256 constant RATE = 66_666666 * 1e12; // 66.666666e18

    uint256 constant CAP = 2_250_000 * 1e6;       // $2.25M USDC
    uint256 constant WALLET_CAP = 25_000 * 1e6;    // $25K per wallet

    function setUp() public {
        vm.startPrank(owner);
        prova = new ProvaToken(owner);
        usdc = new MockUSDC();

        sale = new ProvaCrowdsale(
            address(prova),
            address(usdc),
            vesting,
            RATE,
            CAP,
            WALLET_CAP
        );

        // Fund the sale with ICO allocation (150M PROVA)
        prova.transfer(address(sale), 150_000_000 ether);

        // Give buyers USDC
        usdc.mint(buyer1, 50_000 * 1e6);
        usdc.mint(buyer2, 50_000 * 1e6);

        // Start sale
        sale.startSale(block.timestamp, block.timestamp + 14 days);
        vm.stopPrank();

        // Approve USDC spending
        vm.prank(buyer1);
        usdc.approve(address(sale), type(uint256).max);
        vm.prank(buyer2);
        usdc.approve(address(sale), type(uint256).max);
    }

    function test_buy() public {
        uint256 usdcAmount = 1000 * 1e6; // $1000

        vm.prank(buyer1);
        sale.buy(usdcAmount);

        // Should receive ~66,666.666 PROVA
        uint256 expected = (usdcAmount * RATE) / 1e6;
        assertEq(prova.balanceOf(buyer1), expected);
        assertEq(sale.totalRaised(), usdcAmount);
        assertEq(sale.contributions(buyer1), usdcAmount);
    }

    function test_walletCap() public {
        vm.startPrank(buyer1);

        // Buy up to cap
        sale.buy(WALLET_CAP);
        assertEq(sale.contributions(buyer1), WALLET_CAP);

        // Try to exceed
        vm.expectRevert("Exceeds wallet cap");
        sale.buy(1e6);

        vm.stopPrank();
    }

    function test_saleCap() public {
        // Give buyer1 enough USDC to exceed sale cap
        vm.prank(owner);
        usdc.mint(buyer1, CAP);

        vm.prank(buyer1);
        usdc.approve(address(sale), type(uint256).max);

        // Buy wallet cap
        vm.prank(buyer1);
        sale.buy(WALLET_CAP);

        // Second buyer
        vm.prank(owner);
        usdc.mint(buyer2, CAP);
        vm.prank(buyer2);
        usdc.approve(address(sale), type(uint256).max);
        vm.prank(buyer2);
        sale.buy(WALLET_CAP);

        assertEq(sale.totalRaised(), WALLET_CAP * 2);
    }

    function test_withdrawFunds() public {
        vm.prank(buyer1);
        sale.buy(10_000 * 1e6);

        address multisig = makeAddr("multisig");
        vm.prank(owner);
        sale.withdrawFunds(multisig);

        assertEq(usdc.balanceOf(multisig), 10_000 * 1e6);
    }

    function test_withdrawUnsold() public {
        uint256 saleBefore = prova.balanceOf(address(sale));

        // Buy a small amount
        vm.prank(buyer1);
        sale.buy(100 * 1e6);

        // Warp past end
        vm.warp(block.timestamp + 15 days);

        vm.prank(owner);
        sale.withdrawUnsold(owner);

        uint256 sold = (100 * 1e6 * RATE) / 1e6;
        assertApproxEqAbs(prova.balanceOf(owner), prova.totalSupply() - saleBefore + (saleBefore - sold), 1e12);
    }

    function test_RevertWhen_buyWhenPaused() public {
        vm.prank(owner);
        sale.pause();

        vm.prank(buyer1);
        vm.expectRevert("Sale not active");
        sale.buy(100 * 1e6);
    }

    function test_RevertWhen_buyBeforeStart() public {
        // Create new sale starting in the future
        vm.startPrank(owner);
        ProvaCrowdsale futureSale = new ProvaCrowdsale(
            address(prova), address(usdc), vesting, RATE, CAP, WALLET_CAP
        );
        prova.transfer(address(futureSale), 1000 ether);
        futureSale.startSale(block.timestamp + 1 days, block.timestamp + 15 days);
        vm.stopPrank();

        vm.prank(buyer1);
        vm.expectRevert("Not started");
        futureSale.buy(100 * 1e6);
    }

    function test_whitelist() public {
        vm.startPrank(owner);
        sale.setWhitelistEnabled(true);

        address[] memory wl = new address[](1);
        wl[0] = buyer1;
        sale.addToWhitelist(wl);
        vm.stopPrank();

        // buyer1 can buy
        vm.prank(buyer1);
        sale.buy(100 * 1e6);

        // buyer2 cannot
        vm.prank(buyer2);
        vm.expectRevert("Not whitelisted");
        sale.buy(100 * 1e6);
    }
}
