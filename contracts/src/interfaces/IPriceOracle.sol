// SPDX-License-Identifier: MIT
pragma solidity ^0.8.24;

/// @title IPriceOracle
/// @notice Minimal price oracle interface for PROVA-USD lookups.
///         Compatible with Chainlink AggregatorV3Interface (the standard the
///         Prova production deployment uses), but kept as a Prova-specific
///         interface so we can swap implementations during testnet without
///         pulling in the full Chainlink dependency surface.
///
///         Returns price in 8-decimal USD (Chainlink convention) for
///         compatibility with mainnet PROVA/USD feeds when they exist.
///         For testnet we wrap a constant or a stub feed in MockPriceOracle.
interface IPriceOracle {
    /// @notice Returns the most recent PROVA/USD price.
    /// @return roundId        Round identifier
    /// @return answer         Price in USD with 8 decimals (e.g. $0.10 -> 10_000_000)
    /// @return startedAt      When the round started
    /// @return updatedAt      When the round was last updated
    /// @return answeredInRound Round id of the answer
    function latestRoundData() external view returns (
        uint80 roundId,
        int256 answer,
        uint256 startedAt,
        uint256 updatedAt,
        uint80 answeredInRound
    );

    /// @notice Decimals of the price answer. Always 8 for Chainlink USD pairs.
    function decimals() external view returns (uint8);
}
