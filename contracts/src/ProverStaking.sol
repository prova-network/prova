// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Prova Network contributors.
pragma solidity ^0.8.20;

import {IERC20} from "@openzeppelin/contracts/token/ERC20/IERC20.sol";
import {ERC20Burnable} from "@openzeppelin/contracts/token/ERC20/extensions/ERC20Burnable.sol";
import {IPriceOracle} from "./interfaces/IPriceOracle.sol";
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

    /// @notice Legacy minStakePerGib (kept as 0 in v2). Always-zero for storage layout safety.
    uint256 public minStakePerGib;

    /// @notice Minimum PROVA stake per TiB committed. Soft floor in token units.
    uint256 public minStakePerTiB;

    /// @notice Minimum stake per TiB in 8-decimal USD (Chainlink convention).
    ///         Binding when the oracle is set; e.g. $3.00 / TiB = 300_000_000.
    uint256 public minStakeUsdPerTiB;

    /// @notice Optional PROVA/USD price oracle. address(0) disables the USD floor.
    IPriceOracle public priceOracle;

    /// @notice Maximum staleness of an oracle answer before it falls back to PROVA-floor.
    uint256 public oracleStalenessSeconds = 1 hours;

    uint256 public constant GIB = 1024 * 1024 * 1024;
    uint256 public constant TIB = 1024 * GIB;

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

    /// @notice [REMOVED] Slashed PROVA is burned at slash time. This
    ///         storage slot is preserved for upgrade-safe layout but is
    ///         never written to. Always reads as 0.
    uint256 public slashedPool;

    // ───── Events ────────────────────────────────────────────────────────

    event Staked(address indexed prover, uint256 amount, uint256 newTotal);
    event UnstakeRequested(address indexed prover, uint256 amount, uint256 endsAt);
    event Withdrawn(address indexed prover, uint256 amount);
    event Slashed(address indexed prover, uint256 amount, address indexed by, bytes32 reason);
    event CommittedBytesChanged(address indexed prover, uint256 newCommittedBytes);
    event AuthorizedControllerSet(address indexed controller, bool authorized);
    event MinStakePerGibChanged(uint256 oldValue, uint256 newValue);
    event MinStakePerTiBChanged(uint256 oldValue, uint256 newValue);
    event MinStakeUsdPerTiBChanged(uint256 oldValue, uint256 newValue);
    event PriceOracleChanged(address indexed oldOracle, address indexed newOracle);
    event OracleStalenessChanged(uint256 oldValue, uint256 newValue);
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

    constructor(IERC20 _token, uint256 _minStakePerTiB) Ownable(msg.sender) {
        token = _token;
        // legacy field always 0 in v2
        minStakePerGib = 0;
        minStakePerTiB = _minStakePerTiB;
    }

    // ───── Admin ─────────────────────────────────────────────────────────

    /// @notice Authorize or revoke an address that can modify committedBytes and slash.
    function setAuthorizedController(address controller, bool authorized) external onlyOwner {
        authorizedControllers[controller] = authorized;
        emit AuthorizedControllerSet(controller, authorized);
    }

    function setMinStakePerGib(uint256 /*newValue*/) external view onlyOwner {
        revert("setMinStakePerGib: deprecated; use setMinStakePerTiB");
    }

    function setMinStakePerTiB(uint256 newValue) external onlyOwner {
        emit MinStakePerTiBChanged(minStakePerTiB, newValue);
        minStakePerTiB = newValue;
    }

    /// @param newValue 8-decimal USD per TiB. $3.00 = 300_000_000.
    function setMinStakeUsdPerTiB(uint256 newValue) external onlyOwner {
        emit MinStakeUsdPerTiBChanged(minStakeUsdPerTiB, newValue);
        minStakeUsdPerTiB = newValue;
    }

    function setPriceOracle(IPriceOracle newOracle) external onlyOwner {
        emit PriceOracleChanged(address(priceOracle), address(newOracle));
        priceOracle = newOracle;
    }

    function setOracleStalenessSeconds(uint256 newValue) external onlyOwner {
        require(newValue >= 60 && newValue <= 1 days, "staleness out of range");
        emit OracleStalenessChanged(oracleStalenessSeconds, newValue);
        oracleStalenessSeconds = newValue;
    }

    /// @notice [REMOVED in v2] Slashed PROVA is now burned at slash time;
    ///         there is no pool to withdraw from. Kept as an explicit revert
    ///         so old call sites fail loudly instead of silently succeeding.
    function withdrawSlashed(address /*to*/, uint256 /*amount*/) external view onlyOwner {
        revert("withdrawSlashed: slashing now burns; nothing to withdraw");
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

    /// @notice Slash `amount` from prover's bonded stake. Slashed PROVA is
    ///         BURNED on-chain — it is permanently removed from supply.
    ///         This is the protocol's deflationary force funded by misbehavior.
    /// @dev    If the prover's bonded stake is insufficient, the difference is
    ///         taken from their unbonding queue (prevents instant-exit after
    ///         misbehavior).
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

        // Burn the slashed tokens. PROVA implements ERC20Burnable; if the
        // configured token does not, this call reverts and the slash transaction
        // unwinds — which is the right behavior (slashing requires a burnable
        // token to satisfy the spec).
        ERC20Burnable(address(token)).burn(amount);

        emit Slashed(prover, amount, msg.sender, reason);
    }

    // ───── Views ─────────────────────────────────────────────────────────

    /// @notice Minimum stake required for a given amount of committed bytes.
    function minStakeFor(uint256 committedBytes) external view returns (uint256) {
        return _requiredStake(committedBytes);
    }

    /// @notice How much additional bytes this prover could commit with current stake.
    /// @dev    Returns an UPPER BOUND based on the PROVA-only floor. If the USD
    ///         floor is binding (price low) the actual capacity is less.
    ///         Callers SHOULD verify with `canCommit(prover, bytesNeeded)`.
    function availableCapacityBytes(address prover) external view returns (uint256) {
        StakeInfo storage s = stakes[prover];
        if (minStakePerTiB == 0) return type(uint256).max;
        uint256 tibCapacity = s.staked / minStakePerTiB;
        uint256 byteCapacity = tibCapacity * TIB;
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
        if (committedBytes == 0) return 0;
        uint256 tibRequired = (committedBytes + TIB - 1) / TIB;

        // PROVA-only floor (always available)
        uint256 provaFloor = tibRequired * minStakePerTiB;

        // Without an oracle (or USD floor disabled), return PROVA-floor.
        if (address(priceOracle) == address(0) || minStakeUsdPerTiB == 0) {
            return provaFloor;
        }

        // Read oracle. If stale or non-positive answer, fall back safely.
        (, int256 answer, , uint256 updatedAt,) = priceOracle.latestRoundData();
        if (answer <= 0 || block.timestamp > updatedAt + oracleStalenessSeconds) {
            return provaFloor;
        }

        // USD-equivalent: required PROVA = (usdPerTiB * tibRequired * 1e18) / pricePerProva
        // minStakeUsdPerTiB has 8 decimals; oracle answer has 8 decimals.
        // (1e8 * 1 * 1e18) / 1e8 = 1e18 PROVA wei per TiB at $1 oracle answer.
        uint256 usdFloor = (minStakeUsdPerTiB * tibRequired * 1e18) / uint256(answer);

        // Higher floor binds.
        return provaFloor >= usdFloor ? provaFloor : usdFloor;
    }
}
