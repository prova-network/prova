// SPDX-License-Identifier: MIT
pragma solidity ^0.8.24;
import "@openzeppelin/contracts/token/ERC20/ERC20.sol";

/// @notice Mock USDC for local testing. NOT FOR PRODUCTION.
contract MockUSDC is ERC20 {
    constructor() ERC20("USD Coin (Mock)", "USDC") {
        _mint(msg.sender, 1_000_000_000 ether);
    }
    function mint(address to, uint256 amount) external { _mint(to, amount); }
}
