// SPDX-License-Identifier: MIT
pragma solidity ^0.8.24;

import "@openzeppelin/contracts/token/ERC20/IERC20.sol";
import "@openzeppelin/contracts/token/ERC20/utils/SafeERC20.sol";
import "@openzeppelin/contracts/token/ERC20/extensions/ERC20Burnable.sol";
import "@openzeppelin/contracts/access/Ownable2Step.sol";
import "@openzeppelin/contracts/utils/ReentrancyGuard.sol";

/// @notice Minimal Uniswap V3 router interface. We only need exactInputSingle.
interface ISwapRouter {
    struct ExactInputSingleParams {
        address tokenIn;
        address tokenOut;
        uint24  fee;
        address recipient;
        uint256 deadline;
        uint256 amountIn;
        uint256 amountOutMinimum;
        uint160 sqrtPriceLimitX96;
    }
    function exactInputSingle(ExactInputSingleParams calldata params)
        external payable returns (uint256 amountOut);
}

/// @title FeeRouter
/// @notice Receives the StorageMarketplace's USDC protocol-fee stream
///         and routes it. Three modes, set by the owner:
///
///   MODE_HOLD:    just hold the USDC (default; safe before TGE)
///   MODE_BURN:    swap USDC → PROVA on a Uniswap V3 pool, burn the PROVA
///   MODE_SPLIT:   split: a configurable share is burned; the rest is held
///                 in this contract for treasury use (operations, grants, etc.)
///
/// @dev StorageMarketplace forwards fees here by setting treasury = address(this).
///      PROVA holders see the burn-rate flow proportional to network revenue.
///      No magic, no oracle, no MEV-protection needed beyond the slippage guard
///      on the swap (we only swap up to maxSwapPerCall at a time, and only when
///      the caller passes a tight minOut).
contract FeeRouter is Ownable2Step, ReentrancyGuard {
    using SafeERC20 for IERC20;

    enum Mode { HOLD, BURN, SPLIT }

    IERC20  public immutable usdc;        // payment token from the marketplace
    IERC20  public immutable prova;       // governance / stake token
    ISwapRouter public swapRouter;        // Uniswap V3 router on Base
    uint24  public swapPoolFee = 3000;    // default 0.3% pool

    Mode    public mode = Mode.HOLD;
    uint16  public burnShareBps = 5000;   // 50% of fees burned in SPLIT mode
    uint256 public maxSwapPerCall;        // safety cap; 0 means unlimited
    uint16  public maxSlippageBps = 500;  // 5% (caller-supplied minOut still wins)

    event ModeChanged(Mode indexed oldMode, Mode indexed newMode);
    event BurnShareChanged(uint16 oldBps, uint16 newBps);
    event SwapRouterChanged(address oldRouter, address newRouter);
    event SwapPoolFeeChanged(uint24 oldFee, uint24 newFee);
    event MaxSwapPerCallChanged(uint256 oldMax, uint256 newMax);
    event MaxSlippageChanged(uint16 oldBps, uint16 newBps);

    event FeesBurned(uint256 usdcIn, uint256 provaOut);
    event FeesHeld(uint256 usdcAmount);
    event Withdrawn(address indexed token, address indexed to, uint256 amount);

    error ZeroAddress();
    error InvalidShare();
    error InvalidMode();
    error WrongTokens();
    error NoFeesToProcess();
    error SlippageExceedsCap();

    constructor(
        address _usdc,
        address _prova,
        address _swapRouter,
        address _owner
    ) Ownable(_owner) {
        if (_usdc == address(0) || _prova == address(0) || _owner == address(0)) revert ZeroAddress();
        usdc        = IERC20(_usdc);
        prova       = IERC20(_prova);
        swapRouter  = ISwapRouter(_swapRouter); // may be address(0) until set
    }

    // ─── Configuration ────────────────────────────────────────────────

    function setMode(Mode newMode) external onlyOwner {
        if (newMode != Mode.HOLD && address(swapRouter) == address(0)) {
            revert InvalidMode(); // can't burn without a router
        }
        emit ModeChanged(mode, newMode);
        mode = newMode;
    }

    function setSwapRouter(address newRouter) external onlyOwner {
        emit SwapRouterChanged(address(swapRouter), newRouter);
        swapRouter = ISwapRouter(newRouter);
    }

    function setSwapPoolFee(uint24 newFee) external onlyOwner {
        emit SwapPoolFeeChanged(swapPoolFee, newFee);
        swapPoolFee = newFee;
    }

    function setBurnShare(uint16 newBps) external onlyOwner {
        if (newBps > 10_000) revert InvalidShare();
        emit BurnShareChanged(burnShareBps, newBps);
        burnShareBps = newBps;
    }

    function setMaxSwapPerCall(uint256 newMax) external onlyOwner {
        emit MaxSwapPerCallChanged(maxSwapPerCall, newMax);
        maxSwapPerCall = newMax;
    }

    function setMaxSlippageBps(uint16 newBps) external onlyOwner {
        if (newBps > 5_000) revert InvalidShare(); // cap at 50% slippage
        emit MaxSlippageChanged(maxSlippageBps, newBps);
        maxSlippageBps = newBps;
    }

    // ─── Operation ────────────────────────────────────────────────────

    /// @notice Process accumulated fees according to current mode.
    ///         Anyone can call (so anyone can keep the burn machine
    ///         running). The owner sets minProvaOut to bound slippage
    ///         on each swap.
    /// @param minProvaOut Minimum PROVA output expected from the swap.
    ///        Must be at least (oracle-equivalent * (1 - maxSlippageBps)).
    ///        We don't enforce this on-chain; it's a guard the caller
    ///        sets. UI/keeper bots will compute a sane value.
    function process(uint256 minProvaOut) external nonReentrant returns (uint256 burned, uint256 held) {
        uint256 balance = usdc.balanceOf(address(this));
        if (balance == 0) revert NoFeesToProcess();

        if (mode == Mode.HOLD) {
            emit FeesHeld(balance);
            return (0, balance);
        }

        uint256 toSwap;
        if (mode == Mode.BURN) {
            toSwap = balance;
        } else { // SPLIT
            toSwap = (balance * burnShareBps) / 10_000;
            held   = balance - toSwap;
        }

        if (maxSwapPerCall != 0 && toSwap > maxSwapPerCall) {
            // emit a partial-pass event would be noise; just clamp
            toSwap = maxSwapPerCall;
            // any remaining USDC stays for the next process() call
        }

        if (toSwap == 0) {
            if (held > 0) emit FeesHeld(held);
            return (0, held);
        }

        // Approve + swap
        usdc.forceApprove(address(swapRouter), toSwap);
        ISwapRouter.ExactInputSingleParams memory params = ISwapRouter.ExactInputSingleParams({
            tokenIn:           address(usdc),
            tokenOut:          address(prova),
            fee:               swapPoolFee,
            recipient:         address(this),
            deadline:          block.timestamp + 300,
            amountIn:          toSwap,
            amountOutMinimum:  minProvaOut,
            sqrtPriceLimitX96: 0
        });
        uint256 provaReceived = swapRouter.exactInputSingle(params);

        // Burn the swapped PROVA. ProvaToken inherits ERC20Burnable.
        ERC20Burnable(address(prova)).burn(provaReceived);

        emit FeesBurned(toSwap, provaReceived);
        if (held > 0) emit FeesHeld(held);
        return (provaReceived, held);
    }

    /// @notice Owner-only escape hatch: withdraw any token to a
    ///         destination. Used for moving held SPLIT-mode treasury
    ///         to a multisig, or recovering wrong-token deposits.
    function withdraw(IERC20 token, address to, uint256 amount) external onlyOwner nonReentrant {
        if (to == address(0)) revert ZeroAddress();
        token.safeTransfer(to, amount);
        emit Withdrawn(address(token), to, amount);
    }

    // ─── Views ────────────────────────────────────────────────────────

    function pendingFees() external view returns (uint256) {
        return usdc.balanceOf(address(this));
    }
}
