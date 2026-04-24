// SPDX-License-Identifier: MIT
pragma solidity ^0.8.24;

import "@openzeppelin/contracts/token/ERC20/IERC20.sol";
import "@openzeppelin/contracts/token/ERC20/utils/SafeERC20.sol";
import "@openzeppelin/contracts/utils/ReentrancyGuard.sol";

/// @title ProvaMultiPay
/// @notice ICO contract accepting ETH + multiple ERC-20 tokens.
///         Each payment token has its own rate (PROVA per token unit).
contract ProvaMultiPay is ReentrancyGuard {
    using SafeERC20 for IERC20;

    struct PaymentToken {
        address token;       // address(0) = ETH
        uint256 rate;        // PROVA (18 dec) per 1 full unit of payment token
        uint8 decimals;      // payment token decimals
        bool enabled;
    }

    IERC20 public immutable prova;
    address public owner;

    uint256 public totalProvasSold;
    uint256 public hardCap;          // max PROVA to sell
    bool public isActive;
    uint256 public startTime;
    uint256 public endTime;
    uint256 public perWalletMaxProva; // max PROVA per wallet

    mapping(string => PaymentToken) public paymentTokens;  // symbol => config
    string[] public supportedTokens;
    mapping(address => uint256) public provaPurchased;      // buyer => total PROVA bought

    event Purchase(address indexed buyer, string paymentToken, uint256 paymentAmount, uint256 provaAmount);
    event PaymentTokenAdded(string symbol, address token, uint256 rate, uint8 decimals);
    event FundsWithdrawn(address to, uint256 ethAmount);
    event ERC20Withdrawn(address token, address to, uint256 amount);

    modifier onlyOwner() { require(msg.sender == owner, "Not owner"); _; }
    modifier saleOpen() {
        require(isActive, "Sale not active");
        require(block.timestamp >= startTime && block.timestamp <= endTime, "Outside sale window");
        _;
    }

    constructor(address _prova, uint256 _hardCap, uint256 _perWalletMax) {
        require(_prova != address(0), "Zero prova");
        prova = IERC20(_prova);
        owner = msg.sender;
        hardCap = _hardCap;
        perWalletMaxProva = _perWalletMax;
    }

    /// @notice Add or update a payment token. Use token=address(0) for ETH.
    function setPaymentToken(
        string calldata symbol,
        address token,
        uint256 rate,
        uint8 decimals
    ) external onlyOwner {
        if (!paymentTokens[symbol].enabled) {
            supportedTokens.push(symbol);
        }
        paymentTokens[symbol] = PaymentToken(token, rate, decimals, true);
        emit PaymentTokenAdded(symbol, token, rate, decimals);
    }

    function disablePaymentToken(string calldata symbol) external onlyOwner {
        paymentTokens[symbol].enabled = false;
    }

    function startSale(uint256 _start, uint256 _end) external onlyOwner {
        require(_end > _start, "Invalid window");
        startTime = _start;
        endTime = _end;
        isActive = true;
    }

    function pause() external onlyOwner { isActive = false; }
    function resume() external onlyOwner { isActive = true; }

    /// @notice Buy PROVA with ETH.
    function buyWithETH() external payable nonReentrant saleOpen {
        PaymentToken storage pt = paymentTokens["ETH"];
        require(pt.enabled, "ETH not accepted");
        require(msg.value > 0, "Zero ETH");

        // rate = PROVA per 1 ETH (in 18 dec)
        // provaAmount = msg.value * rate / 1e18
        uint256 provaAmount = (msg.value * pt.rate) / 1e18;
        _executePurchase(msg.sender, provaAmount, "ETH", msg.value);
    }

    /// @notice Buy PROVA with an ERC-20 token.
    /// @param symbol Token symbol (must be registered)
    /// @param amount Amount of payment token (in its native decimals)
    function buyWithToken(string calldata symbol, uint256 amount) external nonReentrant saleOpen {
        PaymentToken storage pt = paymentTokens[symbol];
        require(pt.enabled, "Token not accepted");
        require(pt.token != address(0), "Use buyWithETH for ETH");
        require(amount > 0, "Zero amount");

        // Transfer payment token from buyer
        IERC20(pt.token).safeTransferFrom(msg.sender, address(this), amount);

        // provaAmount = amount * rate / 10^decimals
        uint256 provaAmount = (amount * pt.rate) / (10 ** pt.decimals);
        _executePurchase(msg.sender, provaAmount, symbol, amount);
    }

    function _executePurchase(address buyer, uint256 provaAmount, string memory symbol, uint256 paymentAmount) internal {
        require(provaAmount > 0, "Amount too small");
        require(totalProvasSold + provaAmount <= hardCap, "Exceeds hard cap");
        require(provaPurchased[buyer] + provaAmount <= perWalletMaxProva, "Exceeds wallet limit");

        provaPurchased[buyer] += provaAmount;
        totalProvasSold += provaAmount;

        prova.safeTransfer(buyer, provaAmount);
        emit Purchase(buyer, symbol, paymentAmount, provaAmount);
    }

    /// @notice Withdraw all ETH to destination.
    function withdrawETH(address payable to) external onlyOwner {
        uint256 bal = address(this).balance;
        require(bal > 0, "No ETH");
        (bool ok,) = to.call{value: bal}("");
        require(ok, "ETH transfer failed");
        emit FundsWithdrawn(to, bal);
    }

    /// @notice Withdraw a specific ERC-20 token.
    function withdrawToken(address token, address to) external onlyOwner {
        uint256 bal = IERC20(token).balanceOf(address(this));
        require(bal > 0, "No balance");
        IERC20(token).safeTransfer(to, bal);
        emit ERC20Withdrawn(token, to, bal);
    }

    /// @notice Withdraw unsold PROVA after sale ends.
    function withdrawUnsold(address to) external onlyOwner {
        require(block.timestamp > endTime, "Sale not ended");
        uint256 bal = prova.balanceOf(address(this));
        if (bal > 0) prova.safeTransfer(to, bal);
    }

    /// @notice Get number of supported payment tokens.
    function supportedTokenCount() external view returns (uint256) {
        return supportedTokens.length;
    }

    /// @notice Remaining PROVA for sale.
    function remaining() external view returns (uint256) {
        return prova.balanceOf(address(this));
    }

    receive() external payable {
        // Allow receiving ETH directly (counted as buyWithETH if sale is active)
    }
}
