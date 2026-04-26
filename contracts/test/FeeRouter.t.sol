// SPDX-License-Identifier: MIT
pragma solidity ^0.8.24;

import "forge-std/Test.sol";
import "../src/FeeRouter.sol";
import "../src/ProvaToken.sol";
import "@openzeppelin/contracts/token/ERC20/ERC20.sol";

/// @dev Mock USDC (6 decimals like real USDC, but we don't care here).
contract MockUSDC is ERC20 {
    constructor() ERC20("USD Coin", "USDC") {
        _mint(msg.sender, 1_000_000 ether);
    }
    function mint(address to, uint256 amount) external { _mint(to, amount); }
}

/// @dev Mock V3 router. exactInputSingle pulls amountIn USDC and gives
///      back amountOut PROVA at a fixed exchange rate (1 USDC = 5 PROVA),
///      simulating a "PROVA at $0.20" market price. Reverts if minOut
///      not met.
contract MockSwapRouter {
    MockUSDC public immutable usdc;
    ProvaToken public immutable prova;
    address public immutable provaSource;
    uint256 public rateNumerator   = 5; // PROVA per USDC
    uint256 public rateDenominator = 1;

    constructor(address _usdc, address _prova, address _provaSource) {
        usdc = MockUSDC(_usdc);
        prova = ProvaToken(_prova);
        provaSource = _provaSource;
    }

    function setRate(uint256 num, uint256 den) external { rateNumerator = num; rateDenominator = den; }

    function exactInputSingle(ISwapRouter.ExactInputSingleParams calldata params) external returns (uint256) {
        require(params.tokenIn == address(usdc), "token in wrong");
        require(params.tokenOut == address(prova), "token out wrong");

        // Pull USDC from the caller
        usdc.transferFrom(msg.sender, address(this), params.amountIn);

        // Compute PROVA out at the configured rate
        uint256 out = (params.amountIn * rateNumerator) / rateDenominator;
        require(out >= params.amountOutMinimum, "slippage");

        // Send PROVA from the test-controlled source
        prova.transferFrom(provaSource, params.recipient, out);
        return out;
    }
}

contract FeeRouterTest is Test {
    MockUSDC      usdc;
    ProvaToken    prova;
    MockSwapRouter router;
    FeeRouter     fees;

    address treasury = makeAddr("treasury");
    address owner    = makeAddr("owner");
    address poolLP   = makeAddr("poolLP"); // simulates the swap pool's PROVA inventory

    function setUp() public {
        usdc  = new MockUSDC();
        prova = new ProvaToken(treasury);

        // Move some PROVA to the "pool LP" account that backs the mock router.
        // We move 1M PROVA (1% of total supply); plenty for swap simulations.
        vm.prank(treasury);
        prova.transfer(poolLP, 1_000_000 ether);

        router = new MockSwapRouter(address(usdc), address(prova), poolLP);

        // poolLP approves the router to send its PROVA
        vm.prank(poolLP);
        prova.approve(address(router), type(uint256).max);

        fees = new FeeRouter(address(usdc), address(prova), address(router), owner);

        // Seed FeeRouter with some USDC fees
        usdc.mint(address(fees), 10_000 ether);
    }

    // ─── Constructor ──────────────────────────────────────────────────

    function test_constructor() public view {
        assertEq(address(fees.usdc()),  address(usdc));
        assertEq(address(fees.prova()), address(prova));
        assertEq(fees.owner(),          owner);
        assertEq(uint256(fees.mode()),  uint256(FeeRouter.Mode.HOLD));
    }

    function test_RevertWhen_zeroAddress() public {
        vm.expectRevert(FeeRouter.ZeroAddress.selector);
        new FeeRouter(address(0), address(prova), address(router), owner);
    }

    // ─── Mode: HOLD ───────────────────────────────────────────────────

    function test_processInHoldMode_keepsUsdc() public {
        (uint256 burned, uint256 held) = fees.process(0);
        assertEq(burned, 0);
        assertEq(held, 10_000 ether);
        assertEq(usdc.balanceOf(address(fees)), 10_000 ether);
        assertEq(prova.totalSupply(), 100_000_000 ether);
    }

    // ─── Mode: BURN ───────────────────────────────────────────────────

    function test_processInBurnMode_swapsAndBurns() public {
        vm.prank(owner);
        fees.setMode(FeeRouter.Mode.BURN);

        uint256 supplyBefore = prova.totalSupply();

        // 10,000 USDC × 5 PROVA/USDC = 50,000 PROVA expected
        uint256 minOut = 49_000 ether;
        (uint256 burned, uint256 held) = fees.process(minOut);

        assertEq(burned, 50_000 ether);
        assertEq(held, 0);
        assertEq(usdc.balanceOf(address(fees)), 0);
        assertEq(prova.totalSupply(), supplyBefore - 50_000 ether);
    }

    function test_RevertWhen_setBurnModeWithoutRouter() public {
        FeeRouter f = new FeeRouter(address(usdc), address(prova), address(0), owner);
        vm.prank(owner);
        vm.expectRevert(FeeRouter.InvalidMode.selector);
        f.setMode(FeeRouter.Mode.BURN);
    }

    function test_swapRespectsSlippage() public {
        vm.prank(owner);
        fees.setMode(FeeRouter.Mode.BURN);
        // We'd expect ~50K PROVA back; demand 100K → must revert
        vm.expectRevert(bytes("slippage"));
        fees.process(100_000 ether);
    }

    function test_maxSwapPerCall_clampsAmount() public {
        vm.startPrank(owner);
        fees.setMode(FeeRouter.Mode.BURN);
        fees.setMaxSwapPerCall(1_000 ether); // only swap up to 1K USDC per call
        vm.stopPrank();

        // 1,000 USDC × 5 = 5,000 PROVA burned this round
        (uint256 burned, ) = fees.process(4_900 ether);
        assertEq(burned, 5_000 ether);
        // Remaining 9,000 USDC stays on the contract
        assertEq(usdc.balanceOf(address(fees)), 9_000 ether);
    }

    // ─── Mode: SPLIT ──────────────────────────────────────────────────

    function test_processInSplitMode_burnsHalfHoldsHalf() public {
        vm.startPrank(owner);
        fees.setMode(FeeRouter.Mode.SPLIT);
        fees.setBurnShare(5000); // 50%
        vm.stopPrank();

        uint256 supplyBefore = prova.totalSupply();
        // 5,000 USDC swapped → 25,000 PROVA burned. 5,000 USDC held.
        (uint256 burned, uint256 held) = fees.process(24_000 ether);
        assertEq(burned, 25_000 ether);
        assertEq(held,   5_000 ether);
        assertEq(usdc.balanceOf(address(fees)), 5_000 ether);
        assertEq(prova.totalSupply(), supplyBefore - 25_000 ether);
    }

    function test_processInSplitMode_at100PercentBurn_isLikeBurnMode() public {
        vm.startPrank(owner);
        fees.setMode(FeeRouter.Mode.SPLIT);
        fees.setBurnShare(10_000);
        vm.stopPrank();

        (uint256 burned, uint256 held) = fees.process(49_000 ether);
        assertEq(burned, 50_000 ether);
        assertEq(held, 0);
    }

    function test_processInSplitMode_at0PercentBurn_isLikeHold() public {
        vm.startPrank(owner);
        fees.setMode(FeeRouter.Mode.SPLIT);
        fees.setBurnShare(0);
        vm.stopPrank();

        (uint256 burned, uint256 held) = fees.process(0);
        assertEq(burned, 0);
        assertEq(held, 10_000 ether);
    }

    // ─── Errors ───────────────────────────────────────────────────────

    function test_RevertWhen_processWithNoFees() public {
        // Drain the contract
        vm.prank(owner);
        fees.withdraw(usdc, owner, 10_000 ether);

        vm.expectRevert(FeeRouter.NoFeesToProcess.selector);
        fees.process(0);
    }

    function test_RevertWhen_setBurnShareTooHigh() public {
        vm.prank(owner);
        vm.expectRevert(FeeRouter.InvalidShare.selector);
        fees.setBurnShare(10_001);
    }

    function test_RevertWhen_setMaxSlippageTooHigh() public {
        vm.prank(owner);
        vm.expectRevert(FeeRouter.InvalidShare.selector);
        fees.setMaxSlippageBps(5_001);
    }

    function test_RevertWhen_nonOwnerSetsMode() public {
        vm.prank(makeAddr("eve"));
        vm.expectRevert();
        fees.setMode(FeeRouter.Mode.BURN);
    }

    // ─── Withdraw ─────────────────────────────────────────────────────

    function test_ownerCanWithdrawHeldUsdc() public {
        vm.prank(owner);
        fees.withdraw(usdc, owner, 5_000 ether);
        assertEq(usdc.balanceOf(owner), 5_000 ether);
        assertEq(usdc.balanceOf(address(fees)), 5_000 ether);
    }

    function test_RevertWhen_withdrawToZero() public {
        vm.prank(owner);
        vm.expectRevert(FeeRouter.ZeroAddress.selector);
        fees.withdraw(usdc, address(0), 1);
    }

    // ─── Permissionless triggering ────────────────────────────────────

    function test_anyoneCanProcess() public {
        vm.prank(owner);
        fees.setMode(FeeRouter.Mode.BURN);

        uint256 supplyBefore = prova.totalSupply();
        vm.prank(makeAddr("randomCaller"));
        fees.process(49_000 ether);
        assertEq(prova.totalSupply(), supplyBefore - 50_000 ether);
    }
}
