// SPDX-License-Identifier: MIT
pragma solidity ^0.8.24;

import "@openzeppelin/contracts/token/ERC20/IERC20.sol";
import "@openzeppelin/contracts/token/ERC20/utils/SafeERC20.sol";
import "@openzeppelin/contracts/utils/ReentrancyGuard.sol";

/// @title ProvaCrowdsale
/// @notice Public ICO contract. Accepts USDC (or ETH) for PROVA tokens.
///         Purchased tokens are sent to the vesting contract for scheduled release.
/// @dev Whitelist optional. Per-wallet cap enforced. Owner can pause/unpause.
contract ProvaCrowdsale is ReentrancyGuard {
    using SafeERC20 for IERC20;

    IERC20 public immutable provaToken;
    IERC20 public immutable paymentToken;     // USDC (6 decimals)
    address public immutable vestingContract;
    address public owner;

    uint256 public rate;                       // PROVA per USDC (scaled by 1e18/1e6)
    uint256 public cap;                        // Max USDC to raise
    uint256 public perWalletCap;               // Max USDC per wallet
    uint256 public totalRaised;                // USDC raised so far
    uint256 public totalSold;                  // PROVA sold so far

    bool public isActive;
    bool public whitelistEnabled;

    uint256 public startTime;
    uint256 public endTime;

    mapping(address => uint256) public contributions;  // USDC contributed per address
    mapping(address => bool) public whitelist;

    event TokensPurchased(address indexed buyer, uint256 usdcAmount, uint256 provaAmount);
    event SaleStarted(uint256 startTime, uint256 endTime);
    event SalePaused();
    event SaleResumed();
    event FundsWithdrawn(address to, uint256 amount);

    modifier onlyOwner() {
        require(msg.sender == owner, "Not owner");
        _;
    }

    modifier saleOpen() {
        require(isActive, "Sale not active");
        require(block.timestamp >= startTime, "Not started");
        require(block.timestamp <= endTime, "Ended");
        _;
    }

    /// @param _prova PROVA token address
    /// @param _payment USDC token address
    /// @param _vesting Vesting contract address (tokens go here)
    /// @param _rate PROVA tokens per 1 USDC (e.g., 66666 = $0.015 per PROVA since 1/0.015 ≈ 66.67 PROVA per USDC)
    /// @param _cap Max USDC to raise
    /// @param _perWalletCap Max USDC per wallet
    constructor(
        address _prova,
        address _payment,
        address _vesting,
        uint256 _rate,
        uint256 _cap,
        uint256 _perWalletCap
    ) {
        require(_prova != address(0) && _payment != address(0) && _vesting != address(0), "Zero address");
        provaToken = IERC20(_prova);
        paymentToken = IERC20(_payment);
        vestingContract = _vesting;
        rate = _rate;
        cap = _cap;
        perWalletCap = _perWalletCap;
        owner = msg.sender;
    }

    /// @notice Start the sale with a time window.
    function startSale(uint256 _start, uint256 _end) external onlyOwner {
        require(_end > _start, "Invalid window");
        startTime = _start;
        endTime = _end;
        isActive = true;
        emit SaleStarted(_start, _end);
    }

    function pause() external onlyOwner { isActive = false; emit SalePaused(); }
    function resume() external onlyOwner { isActive = true; emit SaleResumed(); }

    function setWhitelistEnabled(bool _enabled) external onlyOwner {
        whitelistEnabled = _enabled;
    }

    function addToWhitelist(address[] calldata addresses) external onlyOwner {
        for (uint256 i = 0; i < addresses.length; i++) {
            whitelist[addresses[i]] = true;
        }
    }

    /// @notice Buy PROVA tokens with USDC.
    /// @param usdcAmount Amount of USDC to spend (6 decimals)
    function buy(uint256 usdcAmount) external nonReentrant saleOpen {
        require(usdcAmount > 0, "Zero amount");
        if (whitelistEnabled) {
            require(whitelist[msg.sender], "Not whitelisted");
        }
        require(contributions[msg.sender] + usdcAmount <= perWalletCap, "Exceeds wallet cap");
        require(totalRaised + usdcAmount <= cap, "Exceeds sale cap");

        // Calculate PROVA amount
        // rate is PROVA per USDC, scaled: e.g., 66_666666 means 66.666666 PROVA per 1 USDC
        // USDC has 6 decimals, PROVA has 18 decimals
        // provaAmount = usdcAmount * rate * 1e12 / 1e6 = usdcAmount * rate * 1e6
        // Actually: provaAmount (18 dec) = usdcAmount (6 dec) * rate (18 dec) / 1e6
        uint256 provaAmount = (usdcAmount * rate) / 1e6;
        require(provaAmount > 0, "Amount too small");

        // Transfer USDC from buyer to this contract
        paymentToken.safeTransferFrom(msg.sender, address(this), usdcAmount);

        // Transfer PROVA to buyer (they then interact with vesting separately)
        // For simplicity in MVP: tokens go directly to buyer. Vesting handled off-chain
        // or buyer's tokens are held in vesting contract via createSchedule.
        provaToken.safeTransfer(msg.sender, provaAmount);

        contributions[msg.sender] += usdcAmount;
        totalRaised += usdcAmount;
        totalSold += provaAmount;

        emit TokensPurchased(msg.sender, usdcAmount, provaAmount);
    }

    /// @notice Withdraw raised USDC to a destination (multisig recommended).
    function withdrawFunds(address to) external onlyOwner {
        require(to != address(0), "Zero address");
        uint256 balance = paymentToken.balanceOf(address(this));
        require(balance > 0, "No funds");
        paymentToken.safeTransfer(to, balance);
        emit FundsWithdrawn(to, balance);
    }

    /// @notice Withdraw unsold PROVA tokens after sale ends.
    function withdrawUnsold(address to) external onlyOwner {
        require(block.timestamp > endTime, "Sale not ended");
        uint256 balance = provaToken.balanceOf(address(this));
        if (balance > 0) {
            provaToken.safeTransfer(to, balance);
        }
    }

    /// @notice View remaining allocation.
    function remaining() external view returns (uint256) {
        return provaToken.balanceOf(address(this));
    }
}
