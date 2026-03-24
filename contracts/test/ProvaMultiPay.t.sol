// SPDX-License-Identifier: MIT
pragma solidity ^0.8.24;

import "forge-std/Test.sol";
import "../src/ProvaToken.sol";
import "../src/ProvaMultiPay.sol";

contract MockERC20 is ERC20 {
    uint8 private _dec;
    constructor(string memory name, string memory sym, uint8 dec_) ERC20(name, sym) { _dec = dec_; }
    function decimals() public view override returns (uint8) { return _dec; }
    function mint(address to, uint256 amt) external { _mint(to, amt); }
}

contract ProvaMultiPayTest is Test {
    ProvaToken prova;
    ProvaMultiPay sale;
    MockERC20 usdc;
    MockERC20 wbtc;

    address owner = makeAddr("owner");
    address buyer = makeAddr("buyer");

    uint256 constant SALE_SUPPLY = 150_000_000 ether;
    uint256 constant WALLET_MAX = 5_000_000 ether; // 5M PROVA max per wallet

    function setUp() public {
        vm.startPrank(owner);
        prova = new ProvaToken(owner);
        usdc = new MockERC20("USD Coin", "USDC", 6);
        wbtc = new MockERC20("Wrapped BTC", "WBTC", 8);

        sale = new ProvaMultiPay(address(prova), SALE_SUPPLY, WALLET_MAX);
        prova.transfer(address(sale), SALE_SUPPLY);

        // Configure payment tokens
        // ETH: 233,333 PROVA per 1 ETH (at $3500/ETH, $0.015/PROVA)
        sale.setPaymentToken("ETH", address(0), 233_333 ether, 18);
        // USDC: 66.667 PROVA per 1 USDC
        sale.setPaymentToken("USDC", address(usdc), 66_667000000000000000, 6);
        // WBTC: 4,666,666 PROVA per 1 WBTC
        sale.setPaymentToken("WBTC", address(wbtc), 4_666_666 ether, 8);

        sale.startSale(block.timestamp, block.timestamp + 14 days);
        vm.stopPrank();

        // Fund buyer
        vm.deal(buyer, 100 ether);
        vm.prank(owner);
        usdc.mint(buyer, 100_000 * 1e6);
        vm.prank(owner);
        wbtc.mint(buyer, 2 * 1e8); // 2 WBTC
    }

    function test_buyWithETH() public {
        vm.prank(buyer);
        sale.buyWithETH{value: 1 ether}();

        // Should get ~233,333 PROVA
        assertEq(prova.balanceOf(buyer), 233_333 ether);
        assertEq(sale.totalProvasSold(), 233_333 ether);
    }

    function test_buyWithUSDC() public {
        vm.startPrank(buyer);
        usdc.approve(address(sale), type(uint256).max);
        sale.buyWithToken("USDC", 1000 * 1e6); // $1000 USDC
        vm.stopPrank();

        // 1000 USDC * 66.667 = ~66,667 PROVA
        assertEq(prova.balanceOf(buyer), 66_667 ether);
    }

    function test_buyWithWBTC() public {
        vm.startPrank(buyer);
        wbtc.approve(address(sale), type(uint256).max);
        sale.buyWithToken("WBTC", 1e8); // 1 WBTC
        vm.stopPrank();

        assertEq(prova.balanceOf(buyer), 4_666_666 ether);
    }

    function test_walletLimit() public {
        // Buy close to limit
        vm.startPrank(buyer);
        usdc.approve(address(sale), type(uint256).max);
        // 5M PROVA / 66.667 per USDC = ~75,000 USDC
        sale.buyWithToken("USDC", 74_999 * 1e6);

        // Should fail when exceeding
        vm.expectRevert("Exceeds wallet limit");
        sale.buyWithToken("USDC", 10_000 * 1e6);
        vm.stopPrank();
    }

    function test_withdrawETH() public {
        vm.prank(buyer);
        sale.buyWithETH{value: 5 ether}();

        address payable multisig = payable(makeAddr("multisig"));
        vm.prank(owner);
        sale.withdrawETH(multisig);

        assertEq(multisig.balance, 5 ether);
    }

    function test_withdrawToken() public {
        vm.startPrank(buyer);
        usdc.approve(address(sale), type(uint256).max);
        sale.buyWithToken("USDC", 5000 * 1e6);
        vm.stopPrank();

        address multisig = makeAddr("multisig");
        vm.prank(owner);
        sale.withdrawToken(address(usdc), multisig);

        assertEq(usdc.balanceOf(multisig), 5000 * 1e6);
    }

    function test_RevertWhen_saleNotActive() public {
        vm.prank(owner);
        sale.pause();

        vm.prank(buyer);
        vm.expectRevert("Sale not active");
        sale.buyWithETH{value: 1 ether}();
    }

    function test_multiplePayments() public {
        // Buy with ETH first
        vm.prank(buyer);
        sale.buyWithETH{value: 0.5 ether}();

        // Then buy with USDC
        vm.startPrank(buyer);
        usdc.approve(address(sale), type(uint256).max);
        sale.buyWithToken("USDC", 500 * 1e6);
        vm.stopPrank();

        uint256 fromETH = (0.5 ether * 233_333 ether) / 1e18;
        uint256 fromUSDC = (500 * 1e6 * 66_667000000000000000) / 1e6;
        assertEq(prova.balanceOf(buyer), fromETH + fromUSDC);
    }

    function test_disableToken() public {
        vm.prank(owner);
        sale.disablePaymentToken("WBTC");

        vm.startPrank(buyer);
        wbtc.approve(address(sale), type(uint256).max);
        vm.expectRevert("Token not accepted");
        sale.buyWithToken("WBTC", 1e8);
        vm.stopPrank();
    }
}
