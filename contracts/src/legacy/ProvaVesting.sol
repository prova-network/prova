// SPDX-License-Identifier: MIT
pragma solidity ^0.8.24;

import "@openzeppelin/contracts/token/ERC20/IERC20.sol";
import "@openzeppelin/contracts/token/ERC20/utils/SafeERC20.sol";
import "@openzeppelin/contracts/utils/ReentrancyGuard.sol";

/// @title ProvaVesting
/// @notice Token vesting with cliff + linear release. Supports multiple beneficiaries.
/// @dev Owner creates vesting schedules. Beneficiaries claim unlocked tokens.
contract ProvaVesting is ReentrancyGuard {
    using SafeERC20 for IERC20;

    struct Schedule {
        uint256 total;           // Total tokens allocated
        uint256 released;        // Tokens already claimed
        uint256 tgeUnlock;       // Tokens unlocked at TGE (immediate)
        uint256 cliffEnd;        // Timestamp when cliff ends
        uint256 vestEnd;         // Timestamp when vesting fully complete
        bool revocable;          // Can owner revoke unvested tokens?
        bool revoked;            // Has been revoked?
    }

    IERC20 public immutable token;
    address public owner;
    uint256 public tgeTimestamp;

    mapping(address => Schedule) public schedules;
    address[] public beneficiaries;

    event ScheduleCreated(address indexed beneficiary, uint256 total, uint256 tgeUnlock);
    event TokensClaimed(address indexed beneficiary, uint256 amount);
    event ScheduleRevoked(address indexed beneficiary, uint256 returned);
    event TGESet(uint256 timestamp);

    modifier onlyOwner() {
        require(msg.sender == owner, "Not owner");
        _;
    }

    constructor(address _token) {
        require(_token != address(0), "Zero token");
        token = IERC20(_token);
        owner = msg.sender;
    }

    /// @notice Set the TGE timestamp. Can only be set once.
    function setTGE(uint256 _tge) external onlyOwner {
        require(tgeTimestamp == 0, "TGE already set");
        require(_tge > 0, "Invalid timestamp");
        tgeTimestamp = _tge;
        emit TGESet(_tge);
    }

    /// @notice Create a vesting schedule for a beneficiary.
    /// @param beneficiary Address receiving tokens
    /// @param total Total tokens in the schedule
    /// @param tgeUnlockBps Basis points unlocked at TGE (e.g., 2500 = 25%)
    /// @param cliffDuration Seconds after TGE before any linear vesting starts
    /// @param vestDuration Seconds of linear vesting after cliff
    /// @param revocable Whether the owner can revoke unvested tokens
    function createSchedule(
        address beneficiary,
        uint256 total,
        uint256 tgeUnlockBps,
        uint256 cliffDuration,
        uint256 vestDuration,
        bool revocable
    ) external onlyOwner {
        require(beneficiary != address(0), "Zero address");
        require(total > 0, "Zero amount");
        require(tgeUnlockBps <= 10000, "TGE unlock > 100%");
        require(schedules[beneficiary].total == 0, "Schedule exists");
        require(tgeTimestamp > 0, "TGE not set");

        uint256 tgeAmount = (total * tgeUnlockBps) / 10000;
        uint256 cliff = tgeTimestamp + cliffDuration;
        uint256 end = cliff + vestDuration;

        schedules[beneficiary] = Schedule({
            total: total,
            released: 0,
            tgeUnlock: tgeAmount,
            cliffEnd: cliff,
            vestEnd: end,
            revocable: revocable,
            revoked: false
        });

        beneficiaries.push(beneficiary);
        token.safeTransferFrom(msg.sender, address(this), total);

        emit ScheduleCreated(beneficiary, total, tgeAmount);
    }

    /// @notice Claim all currently unlocked tokens.
    function claim() external nonReentrant {
        Schedule storage s = schedules[msg.sender];
        require(s.total > 0, "No schedule");
        require(!s.revoked, "Revoked");

        uint256 unlocked = _vestedAmount(s);
        uint256 claimable = unlocked - s.released;
        require(claimable > 0, "Nothing to claim");

        s.released += claimable;
        token.safeTransfer(msg.sender, claimable);

        emit TokensClaimed(msg.sender, claimable);
    }

    /// @notice View how many tokens a beneficiary can currently claim.
    function claimable(address beneficiary) external view returns (uint256) {
        Schedule storage s = schedules[beneficiary];
        if (s.total == 0 || s.revoked) return 0;
        return _vestedAmount(s) - s.released;
    }

    /// @notice Revoke a schedule and return unvested tokens to owner.
    function revoke(address beneficiary) external onlyOwner {
        Schedule storage s = schedules[beneficiary];
        require(s.total > 0, "No schedule");
        require(s.revocable, "Not revocable");
        require(!s.revoked, "Already revoked");

        uint256 vested = _vestedAmount(s);
        uint256 unvested = s.total - vested;

        s.revoked = true;

        if (unvested > 0) {
            token.safeTransfer(owner, unvested);
        }

        emit ScheduleRevoked(beneficiary, unvested);
    }

    /// @dev Calculate total vested amount for a schedule at current time.
    function _vestedAmount(Schedule storage s) internal view returns (uint256) {
        if (block.timestamp < tgeTimestamp) return 0;

        // TGE unlock is immediate
        uint256 vested = s.tgeUnlock;

        // Linear vesting after cliff
        if (block.timestamp >= s.vestEnd) {
            vested = s.total;
        } else if (block.timestamp >= s.cliffEnd) {
            uint256 vestable = s.total - s.tgeUnlock;
            uint256 elapsed = block.timestamp - s.cliffEnd;
            uint256 vestDuration = s.vestEnd - s.cliffEnd;
            vested += (vestable * elapsed) / vestDuration;
        }

        return vested;
    }

    /// @notice Number of beneficiaries with schedules.
    function beneficiaryCount() external view returns (uint256) {
        return beneficiaries.length;
    }
}
