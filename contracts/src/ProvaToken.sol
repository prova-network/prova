// SPDX-License-Identifier: MIT
pragma solidity ^0.8.24;

import "@openzeppelin/contracts/token/ERC20/ERC20.sol";
import "@openzeppelin/contracts/token/ERC20/extensions/ERC20Burnable.sol";
import "@openzeppelin/contracts/token/ERC20/extensions/ERC20Permit.sol";

/// @title ProvaToken
/// @notice ERC-20 token for the Prova network. Fixed 1B supply, no minting.
/// @dev At mainnet launch, holders swap 1:1 for native PROVA.
contract ProvaToken is ERC20, ERC20Burnable, ERC20Permit {
    uint256 public constant TOTAL_SUPPLY = 1_000_000_000 ether; // 1B with 18 decimals

    /// @param treasury Address receiving the full supply for distribution
    constructor(address treasury) ERC20("Prova", "PROVA") ERC20Permit("Prova") {
        require(treasury != address(0), "Zero address");
        _mint(treasury, TOTAL_SUPPLY);
    }
}
