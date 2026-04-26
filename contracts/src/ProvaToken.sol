// SPDX-License-Identifier: MIT
pragma solidity ^0.8.24;

import "@openzeppelin/contracts/token/ERC20/ERC20.sol";
import "@openzeppelin/contracts/token/ERC20/extensions/ERC20Burnable.sol";
import "@openzeppelin/contracts/token/ERC20/extensions/ERC20Permit.sol";

/// @title ProvaToken
/// @notice ERC-20 token for the Prova network. Fixed 100M supply, no
///         minting beyond genesis, no mainnet swap. This is the canonical
///         PROVA token on Base.
///
///         In-protocol roles:
///           - Prover stake (slashable bond denominated in PROVA)
///           - Fee burn target (USDC fees auto-swapped to PROVA and burned)
///           - Governance vote weight
///
///         The 100M supply mirrors the v1 tokenomics document. Decimals
///         are 18 (standard ERC-20). Total minted to the treasury at
///         construction; distribution happens via the ProvaVesting
///         contract and the public sale.
contract ProvaToken is ERC20, ERC20Burnable, ERC20Permit {
    /// @notice Fixed total supply: 100,000,000 PROVA with 18 decimals.
    uint256 public constant TOTAL_SUPPLY = 100_000_000 ether;

    /// @param treasury Address receiving the full supply for distribution.
    ///                 In production this is a multisig (e.g. a 5-of-9 Safe).
    constructor(address treasury) ERC20("Prova", "PROVA") ERC20Permit("Prova") {
        require(treasury != address(0), "Zero address");
        _mint(treasury, TOTAL_SUPPLY);
    }
}
