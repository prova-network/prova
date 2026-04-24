// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Prova Network contributors.
pragma solidity ^0.8.20;

import {IERC20} from "@openzeppelin/contracts/token/ERC20/IERC20.sol";
import {SafeERC20} from "@openzeppelin/contracts/token/ERC20/utils/SafeERC20.sol";
import {Ownable} from "@openzeppelin/contracts/access/Ownable.sol";
import {ReentrancyGuard} from "@openzeppelin/contracts/utils/ReentrancyGuard.sol";

/// @title ProverStaking
/// @notice Holds PROVA stake on behalf of provers. Stake is slashable by the
///         DisputeManager when a prover fails challenges. Unstaking requires
///         an unbonding period to prevent fast-exit after misbehavior.
/// @dev Minimum stake is enforced per byte committed via `minStakeFor(bytes)`.
///      StorageMarketplace consults this contract before accepting new deals.
contract ProverStaking is Ownable, ReentrancyGuard {
    using SafeERC20 for IERC20;

    // ───── Types ─────────────────────────────────────────────────────────

    struct StakeInfo {
        uint256 staked;       // currently-bonded stake
        uint256 unbonding;    // stake in unbonding queue (not slashable after unbondingEndsAt)
        uint256 unbondingEndsAt; // timestamp when unbonding completes
        uint256 committedBytes;  // bytes of storage this prover has outstanding deals for
    }

    // ───── Constants ─────────────────────────────────────────────────────

    /// @notice Unbonding period (slashable window after unstake request).
    uint256 public constant UNBONDING_PERIOD = 14 days;

    /// @notice Minimum stake required per GiB committed.
    /// @dev Default: 1 token (in 1e18 units). Adjustable by owner via setMinStakePerGib.
    uint256 public minStakePerGib;

    uint256 public constant GIB = 1024 * 1024 * 1024;

    // ───── State ─────────────────────────────────────────────────────────

    /// @notice The PROVA ERC-20 token used for staking.
    IERC20 public immutable token;

    /// @notice Stake state per prover.
    mapping(address => StakeInfo) public stakes;

    /// @notice Addresses authorized to commit/release bytes on a prover's behalf
    ///         (typically the StorageMarketplace and DisputeManager contracts).
    mapping(address => bool) public authorizedControllers;

    /// @notice Total staked across all provers (accounting).
    uint256 public totalStaked;

    /// @notice Total slashed tokens held by this contract pending treasury claim.
    uint256 public slashedPool;

    // ───── Events ────────────────────────────────────────────────────────

    event Staked(address indexed prover, uint256 amount, uint256 newTotal);
    event UnstakeRequested(address indexed prover, uint256 amount, uint256 endsAt);
    event Withdrawn(address indexed prover, uint256 amount);
    event Slashed(address indexed prover, uint256 amount, address indexed by, bytes32 reason);
    event CommittedBytesChanged(address indexed prover, uint256 newCommittedBytes);
    event AuthorizedControllerSet(address indexed controller, bool authorized);
    event MinStakePerGibChanged(uint256 oldValue, uint256 newValue);
    event SlashedPoolWithdrawn(address indexed to, uint256 amount);

    // ───── Errors ────────────────────────────────────────────────────────

    error InsufficientStake();
    error InsufficientBonded();
    error StillUnbonding();
    error NothingToWithdraw();
    error NotAuthorized();
    error WouldDropBelowMinimum();
    error ZeroAmount();

    // ───── Construction ──────────────────────────────────────────────────

    constructor(IERC20 _token, uint256 _minStakePerGib) Ownable(msg.sender) {
        token = _token;
        minStakePerGib = _minStakePerGib;
    }

    // ───── Admin ─────────────────────────────────────────────────────────

    /// @notice Authorize or revoke an address that can modify committedBytes and slash.
    function setAuthorizedController(address controller, bool authorized) external onlyOwner {
        authorizedControllers[controller] = authorized;
        emit AuthorizedControllerSet(controller, authorized);
    }

    function setMinStakePerGib(uint256 newValue) external onlyOwner {
        emit MinStakePerGibChanged(minStakePerGib, newValue);
        minStakePerGib = newValue;
    }

    /// @notice Withdraw accumulated slashed tokens to the treasury.
    function withdrawSlashed(address to, uint256 amount) external onlyOwner {
        require(amount <= slashedPool, "exceeds slashed pool");
        slashedPool -= amount;
        token.safeTransfer(to, amount);
        emit SlashedPoolWithdrawn(to, amount);
    }

    // ───── Staking ───────────────────────────────────────────────────────

    /// @notice Stake PROVA. Caller must have approved at least `amount` tokens.
    function stake(uint256 amount) external nonReentrant {
        if (amount == 0) revert ZeroAmount();

        token.safeTransferFrom(msg.sender, address(this), amount);
        stakes[msg.sender].staked += amount;
        totalStaked += amount;

        emit Staked(msg.sender, amount, stakes[msg.sender].staked);
    }

    /// @notice Request unstake. Moves `amount` from staked -> unbonding.
    ///         After UNBONDING_PERIOD elapses, caller can `withdraw`.
    /// @dev Cannot reduce staked below what's required for current committedBytes.
    function requestUnstake(uint256 amount) external nonReentrant {
        if (amount == 0) revert ZeroAmount();

        StakeInfo storage s = stakes[msg.sender];
        if (amount > s.staked) revert InsufficientBonded();

        uint256 requiredAfter = _requiredStake(s.committedBytes);
        if (s.staked - amount < requiredAfter) revert WouldDropBelowMinimum();

        s.staked -= amount;
        totalStaked -= amount;
        s.unbonding += amount;
        s.unbondingEndsAt = block.timestamp + UNBONDING_PERIOD;

        emit UnstakeRequested(msg.sender, amount, s.unbondingEndsAt);
    }

    /// @notice Withdraw fully-unbonded stake to the caller.
    function withdraw() external nonReentrant {
        StakeInfo storage s = stakes[msg.sender];
        uint256 amount = s.unbonding;
        if (amount == 0) revert NothingToWithdraw();
        if (block.timestamp < s.unbondingEndsAt) revert StillUnbonding();

        s.unbonding = 0;
        s.unbondingEndsAt = 0;
        token.safeTransfer(msg.sender, amount);

        emit Withdrawn(msg.sender, amount);
    }

    // ───── Controller Hooks (StorageMarketplace, DisputeManager) ─────────

    /// @notice Mark additional bytes as committed by `prover` (called when a deal starts).
    function commitBytes(address prover, uint256 newBytes) external {
        if (!authorizedControllers[msg.sender]) revert NotAuthorized();
        StakeInfo storage s = stakes[prover];
        s.committedBytes += newBytes;

        // Check stake is still sufficient
        if (s.staked < _requiredStake(s.committedBytes)) revert InsufficientStake();

        emit CommittedBytesChanged(prover, s.committedBytes);
    }

    /// @notice Mark bytes as no longer committed (deal completed or canceled).
    function releaseBytes(address prover, uint256 freedBytes) external {
        if (!authorizedControllers[msg.sender]) revert NotAuthorized();
        StakeInfo storage s = stakes[prover];

        if (freedBytes > s.committedBytes) {
            s.committedBytes = 0;
        } else {
            s.committedBytes -= freedBytes;
        }

        emit CommittedBytesChanged(prover, s.committedBytes);
    }

    /// @notice Slash `amount` from prover's bonded stake. Also slashes from unbonding
    ///         if bonded is insufficient (prevents instant-exit after misbehavior).
    function slash(address prover, uint256 amount, bytes32 reason) external {
        if (!authorizedControllers[msg.sender]) revert NotAuthorized();
        if (amount == 0) revert ZeroAmount();

        StakeInfo storage s = stakes[prover];
        uint256 total = s.staked + s.unbonding;
        if (amount > total) amount = total; // cap at what they actually have

        // Prefer to slash bonded first
        if (amount <= s.staked) {
            s.staked -= amount;
            totalStaked -= amount;
        } else {
            uint256 fromUnbonding = amount - s.staked;
            totalStaked -= s.staked;
            s.staked = 0;
            s.unbonding -= fromUnbonding;
        }

        slashedPool += amount;
        emit Slashed(prover, amount, msg.sender, reason);
    }

    // ───── Views ─────────────────────────────────────────────────────────

    /// @notice Minimum stake required for a given amount of committed bytes.
    function minStakeFor(uint256 committedBytes) external view returns (uint256) {
        return _requiredStake(committedBytes);
    }

    /// @notice How much additional bytes this prover could commit with current stake.
    function availableCapacityBytes(address prover) external view returns (uint256) {
        StakeInfo storage s = stakes[prover];
        if (minStakePerGib == 0) return type(uint256).max;
        uint256 gibCapacity = s.staked / minStakePerGib;
        uint256 byteCapacity = gibCapacity * GIB;
        if (byteCapacity <= s.committedBytes) return 0;
        return byteCapacity - s.committedBytes;
    }

    /// @notice Is prover eligible to take on `bytesNeeded` more bytes?
    function canCommit(address prover, uint256 bytesNeeded) external view returns (bool) {
        StakeInfo storage s = stakes[prover];
        uint256 required = _requiredStake(s.committedBytes + bytesNeeded);
        return s.staked >= required;
    }

    /// @notice Full stake state for a prover.
    function getStake(address prover) external view returns (StakeInfo memory) {
        return stakes[prover];
    }

    // ───── Internal ──────────────────────────────────────────────────────

    function _requiredStake(uint256 committedBytes) internal view returns (uint256) {
        if (minStakePerGib == 0 || committedBytes == 0) return 0;
        // Ceiling division: any fraction of a GiB requires a full GiB of stake
        uint256 gibRequired = (committedBytes + GIB - 1) / GIB;
        return gibRequired * minStakePerGib;
    }
}
