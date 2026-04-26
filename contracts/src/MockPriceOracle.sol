// SPDX-License-Identifier: MIT
pragma solidity ^0.8.24;

import {IPriceOracle} from "./interfaces/IPriceOracle.sol";

/// @title MockPriceOracle
/// @notice Test-only price feed that returns whatever the owner sets.
///         Used by:
///           - Foundry tests that need to swing PROVA/USD price to verify
///             the USD-equivalent stake floor binds correctly
///           - Base Sepolia deployments before a real Chainlink feed exists
///             (we set price to a reasonable testnet anchor like $0.10)
///
///         NOT FOR MAINNET. Replace with a real Chainlink AggregatorV3
///         feed before production.
contract MockPriceOracle is IPriceOracle {
    int256  public price;        // 8-decimal USD price
    uint8   public override decimals = 8;
    uint80  public roundId = 1;
    uint256 public lastUpdated;  // captured at construction / setPrice time

    address public owner;

    error NotOwner();

    constructor(int256 initialPrice) {
        owner = msg.sender;
        price = initialPrice;
        lastUpdated = block.timestamp;
    }

    function setPrice(int256 newPrice) external {
        if (msg.sender != owner) revert NotOwner();
        price = newPrice;
        roundId += 1;
        lastUpdated = block.timestamp;
    }

    function setDecimals(uint8 newDecimals) external {
        if (msg.sender != owner) revert NotOwner();
        decimals = newDecimals;
    }

    /// @notice Manually backdate the oracle (for testing staleness paths).
    function setLastUpdated(uint256 ts) external {
        if (msg.sender != owner) revert NotOwner();
        lastUpdated = ts;
    }

    function latestRoundData() external view override returns (
        uint80, int256, uint256, uint256, uint80
    ) {
        return (roundId, price, lastUpdated, lastUpdated, roundId);
    }
}
