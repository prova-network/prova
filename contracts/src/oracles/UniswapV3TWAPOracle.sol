// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Prova Network contributors.
//
// Uniswap V3 PROVA/USDC TWAP oracle.
//
// Wraps a Uniswap V3 pool's `observe()` price oracle into the
// `IPriceOracle` interface used by `ProverStaking` for USD-equivalent
// stake-floor pricing. The wrapper:
//
//   - reads time-weighted-average tick over a configurable window
//     (default 30 minutes; bounded to [5 min, 60 min] by governance)
//   - converts the TWAP tick to a sqrt-price ratio via `TickMath`
//   - converts the sqrt-price ratio to a USDC-per-PROVA quote with
//     decimal-aware math via `Math.mulDiv` (no intermediate overflow)
//   - returns the quote scaled to the Chainlink-standard 8-decimal
//     USD answer, so the consumer (ProverStaking) does not need to
//     special-case Uniswap vs Chainlink at all
//
// This is the Phase 2 mainnet oracle path documented in
// prova-network/contracts#3. Phase 1 (testnet) uses MockPriceOracle.
// Phase 3 swaps in ChainlinkPriceOracle once a verified PROVA/USD
// aggregator is published.
pragma solidity ^0.8.20;

import {Math} from "@openzeppelin/contracts/utils/math/Math.sol";
import {Ownable2Step, Ownable} from "@openzeppelin/contracts/access/Ownable2Step.sol";

import {IPriceOracle} from "../interfaces/IPriceOracle.sol";
import {TickMath} from "./TickMath.sol";

/// @notice Minimal subset of the Uniswap V3 pool ABI we need.
interface IUniswapV3Pool {
    function token0() external view returns (address);
    function token1() external view returns (address);

    /// @notice Returns the cumulative tick and liquidity for each `secondsAgo`.
    /// @dev See Uniswap/v3-core IUniswapV3PoolDerivedState.observe.
    function observe(uint32[] calldata secondsAgos)
        external
        view
        returns (int56[] memory tickCumulatives, uint160[] memory secondsPerLiquidityCumulativeX128s);
}

contract UniswapV3TWAPOracle is IPriceOracle, Ownable2Step {
    /// @dev IPriceOracle (Chainlink) decimals convention: 8.
    uint8 public constant override decimals = 8;

    /// @dev Minimum TWAP window. Anything shorter than 5 minutes on a real
    ///      pool is too easy to manipulate with a single block of one-sided
    ///      liquidity.
    uint32 public constant MIN_TWAP_WINDOW = 5 minutes;

    /// @dev Maximum TWAP window. Past 60 minutes, the price stops tracking
    ///      reality on a fast-moving market.
    uint32 public constant MAX_TWAP_WINDOW = 60 minutes;

    /// @dev The pool we read the TWAP from. Must contain (PROVA, USDC).
    IUniswapV3Pool public immutable pool;

    /// @dev True if PROVA is token0 in the pool, false otherwise.
    bool public immutable provaIsToken0;

    /// @dev 10^(provaDecimals). For PROVA = 18 → 1e18.
    uint256 public immutable provaUnit;

    /// @dev 10^(usdcDecimals). For USDC = 6 → 1e6.
    uint256 public immutable usdcUnit;

    /// @dev TWAP window in seconds. Settable by owner within bounds.
    uint32 public twapWindow;

    error InvalidTwapWindow();
    error InvalidPool();
    error PoolHasNoHistory();

    event TwapWindowChanged(uint32 oldWindow, uint32 newWindow);

    /// @param _pool         Uniswap V3 pool to read from
    /// @param _prova        PROVA token address
    /// @param _usdc         USDC token address
    /// @param _twapWindow   Initial TWAP window in seconds
    /// @param _provaDecimals  PROVA decimals (18 in production)
    /// @param _usdcDecimals   USDC decimals (6 in production)
    constructor(
        address _pool,
        address _prova,
        address _usdc,
        uint32 _twapWindow,
        uint8 _provaDecimals,
        uint8 _usdcDecimals
    ) Ownable(msg.sender) {
        if (_pool == address(0) || _prova == address(0) || _usdc == address(0)) revert InvalidPool();
        if (_twapWindow < MIN_TWAP_WINDOW || _twapWindow > MAX_TWAP_WINDOW) revert InvalidTwapWindow();

        pool = IUniswapV3Pool(_pool);
        twapWindow = _twapWindow;

        address t0 = pool.token0();
        address t1 = pool.token1();
        if (t0 == _prova && t1 == _usdc) {
            provaIsToken0 = true;
        } else if (t0 == _usdc && t1 == _prova) {
            provaIsToken0 = false;
        } else {
            revert InvalidPool();
        }

        provaUnit = 10 ** _provaDecimals;
        usdcUnit  = 10 ** _usdcDecimals;
    }

    function setTwapWindow(uint32 newWindow) external onlyOwner {
        if (newWindow < MIN_TWAP_WINDOW || newWindow > MAX_TWAP_WINDOW) revert InvalidTwapWindow();
        emit TwapWindowChanged(twapWindow, newWindow);
        twapWindow = newWindow;
    }

    /// @notice Compute the time-weighted-average tick over the configured
    ///         TWAP window. Reverts with `PoolHasNoHistory` if the pool's
    ///         observation cardinality is too low to cover the window.
    function consultTick() public view returns (int24 timeWeightedAverageTick) {
        uint32 window = twapWindow;
        uint32[] memory secondsAgos = new uint32[](2);
        secondsAgos[0] = window;
        secondsAgos[1] = 0;

        try pool.observe(secondsAgos) returns (int56[] memory tickCumulatives, uint160[] memory) {
            int56 delta = tickCumulatives[1] - tickCumulatives[0];
            timeWeightedAverageTick = int24(delta / int56(uint56(window)));

            // Round toward negative infinity (Uniswap's convention) so we
            // never bias the price slightly positive on negative tick truncation.
            if (delta < 0 && (delta % int56(uint56(window)) != 0)) {
                timeWeightedAverageTick -= 1;
            }
        } catch {
            revert PoolHasNoHistory();
        }
    }

    /// @notice Returns USD price per 1 PROVA, scaled to 8 decimals (Chainlink
    ///         convention). For PROVA=$0.10 the answer is 10_000_000.
    function _consultUsdPerProvaScaled8() internal view returns (uint256) {
        int24 tick = consultTick();
        uint160 sqrtRatioX96 = TickMath.getSqrtRatioAtTick(tick);

        // Quote: how many quote-token base units does 1 PROVA (in PROVA base units) yield?
        // We use Math.mulDiv on a 256-bit intermediate to avoid overflow.
        // Mirrors Uniswap's OracleLibrary.getQuoteAtTick logic.
        uint256 baseAmount = provaUnit;
        uint256 quoteAmount;
        if (sqrtRatioX96 <= type(uint128).max) {
            uint256 ratioX192 = uint256(sqrtRatioX96) * uint256(sqrtRatioX96);
            quoteAmount = provaIsToken0
                ? Math.mulDiv(ratioX192, baseAmount, 1 << 192)
                : Math.mulDiv(1 << 192, baseAmount, ratioX192);
        } else {
            uint256 ratioX128 = Math.mulDiv(uint256(sqrtRatioX96), uint256(sqrtRatioX96), 1 << 64);
            quoteAmount = provaIsToken0
                ? Math.mulDiv(ratioX128, baseAmount, 1 << 128)
                : Math.mulDiv(1 << 128, baseAmount, ratioX128);
        }

        // quoteAmount is now USDC base units per 1 PROVA. Convert to
        // 8-decimal USD: usd_8dec = quoteAmount * 1e8 / usdcUnit.
        // For USDC=6dec this is quoteAmount * 100, no division.
        return Math.mulDiv(quoteAmount, 1e8, usdcUnit);
    }

    /// @inheritdoc IPriceOracle
    /// @dev `roundId` and `answeredInRound` use `block.timestamp` for monotonic
    ///      progression. `startedAt` and `updatedAt` mark the trailing edge of
    ///      the TWAP window so `ProverStaking.oracleStalenessSeconds` still
    ///      correctly rejects a stalled pool.
    function latestRoundData() external view override returns (
        uint80 roundId,
        int256 answer,
        uint256 startedAt,
        uint256 updatedAt,
        uint80 answeredInRound
    ) {
        uint256 priceScaled = _consultUsdPerProvaScaled8();
        require(priceScaled <= uint256(type(int256).max), "price overflow");

        // Updated-at marks the most recent observation included in the TWAP,
        // which by construction is `block.timestamp` (the oracle reads the
        // pool live every call). startedAt marks the trailing edge of the
        // window. The clamp to 0 prevents underflow on chains/tests where
        // block.timestamp could be smaller than twapWindow.
        updatedAt = block.timestamp;
        startedAt = block.timestamp > twapWindow ? block.timestamp - twapWindow : 0;
        roundId = uint80(block.timestamp);
        answeredInRound = roundId;
        answer = int256(priceScaled);
    }
}
