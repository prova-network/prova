// SPDX-License-Identifier: MIT
pragma solidity ^0.8.24;

import "@openzeppelin/contracts/token/ERC20/IERC20.sol";
import "@openzeppelin/contracts/token/ERC20/utils/SafeERC20.sol";
import "@openzeppelin/contracts/access/Ownable2Step.sol";
import "@openzeppelin/contracts/utils/ReentrancyGuard.sol";

/// @title ProvaVesting
/// @notice Linear vesting with cliff for PROVA token allocations.
///         Designed for team, advisor, and BD grants. Owner (treasury
///         multisig) creates schedules; beneficiaries claim vested
///         tokens at any time after the cliff.
///
///         A grant is permanent once created — the owner can only
///         revoke if `revocable=true` was set at grant time, and only
///         non-vested tokens are clawed back. Already-vested tokens
///         remain claimable by the beneficiary.
///
///         Notation matches the whitepaper amendment §4.4.A:
///           cliff_seconds      — no tokens vest before this
///           duration_seconds   — total vest length (cliff + linear)
///           start_seconds      — vesting clock starts at this unix time
///                                 (TGE for most grants)
contract ProvaVesting is Ownable2Step, ReentrancyGuard {
    using SafeERC20 for IERC20;

    IERC20 public immutable token;

    struct Schedule {
        address beneficiary;
        uint64  start;        // unix
        uint64  cliff;        // seconds AFTER start
        uint64  duration;     // seconds AFTER start (must be >= cliff)
        uint128 totalAmount;
        uint128 claimedAmount;
        bool    revocable;
        bool    revoked;
    }

    /// @dev id starts at 1 so 0 means "not set"
    uint256 public nextId = 1;
    mapping(uint256 => Schedule) public schedules;
    mapping(address => uint256[]) public idsByBeneficiary;

    event ScheduleCreated(
        uint256 indexed id,
        address indexed beneficiary,
        uint128 amount,
        uint64  start,
        uint64  cliff,
        uint64  duration,
        bool    revocable
    );
    event Claimed(uint256 indexed id, address indexed beneficiary, uint128 amount);
    event Revoked(uint256 indexed id, uint128 returnedToTreasury);
    event AcceleratedBy(uint256 indexed id, uint64 secondsAccelerated);

    error CliffExceedsDuration();
    error NotBeneficiary();
    error AlreadyRevoked();
    error NotRevocable();
    error NothingToClaim();
    error InvalidSchedule();
    error TransferFailed();

    constructor(address _token, address _owner) Ownable(_owner) {
        require(_token != address(0) && _owner != address(0), "Zero address");
        token = IERC20(_token);
    }

    // ─── Owner: grant + manage ────────────────────────────────────────

    /// @notice Create a vesting schedule. The owner must already have
    ///         transferred (or pre-approved this contract for) `amount`
    ///         of PROVA — we pull on creation so funds are locked.
    /// @return id The schedule id.
    function createSchedule(
        address beneficiary,
        uint128 amount,
        uint64  start,
        uint64  cliffSeconds,
        uint64  durationSeconds,
        bool    revocable
    ) external onlyOwner returns (uint256 id) {
        if (beneficiary == address(0) || amount == 0) revert InvalidSchedule();
        if (cliffSeconds > durationSeconds) revert CliffExceedsDuration();
        if (start == 0) start = uint64(block.timestamp);

        id = nextId++;
        schedules[id] = Schedule({
            beneficiary:   beneficiary,
            start:         start,
            cliff:         cliffSeconds,
            duration:      durationSeconds,
            totalAmount:   amount,
            claimedAmount: 0,
            revocable:     revocable,
            revoked:       false
        });
        idsByBeneficiary[beneficiary].push(id);

        token.safeTransferFrom(msg.sender, address(this), amount);

        emit ScheduleCreated(id, beneficiary, amount, start, cliffSeconds, durationSeconds, revocable);
    }

    /// @notice Revoke a schedule (only if it was marked revocable).
    ///         Vested-but-unclaimed tokens stay claimable by the
    ///         beneficiary. Unvested tokens go back to the owner.
    function revoke(uint256 id) external onlyOwner nonReentrant {
        Schedule storage s = schedules[id];
        if (s.beneficiary == address(0)) revert InvalidSchedule();
        if (!s.revocable) revert NotRevocable();
        if (s.revoked) revert AlreadyRevoked();

        uint128 vested  = uint128(_vestedAt(s, uint64(block.timestamp)));
        uint128 toReturn = s.totalAmount - vested;

        s.revoked     = true;
        s.totalAmount = vested; // future _vestedAt() returns vested cap
        s.duration    = uint64(block.timestamp) - s.start; // freeze the curve

        if (toReturn > 0) {
            token.safeTransfer(owner(), toReturn);
        }
        emit Revoked(id, toReturn);
    }

    /// @notice Accelerate a schedule by `secondsToAccelerate`.
    ///         Equivalent to shortening the duration by that many
    ///         seconds (so the same fractional curve completes sooner).
    ///         Cliff is also reduced; if the cliff would go negative
    ///         it's clamped to zero. Cannot accelerate beyond fully
    ///         vested.
    function accelerate(uint256 id, uint64 secondsToAccelerate) external onlyOwner {
        Schedule storage s = schedules[id];
        if (s.beneficiary == address(0)) revert InvalidSchedule();
        if (s.revoked) revert AlreadyRevoked();

        // Shorten the duration. Capped at zero → fully vested.
        if (secondsToAccelerate >= s.duration) {
            s.duration = 0;
            s.cliff = 0;
        } else {
            s.duration -= secondsToAccelerate;
            if (secondsToAccelerate >= s.cliff) {
                s.cliff = 0;
            } else {
                s.cliff -= secondsToAccelerate;
            }
        }
        emit AcceleratedBy(id, secondsToAccelerate);
    }

    // ─── Beneficiary: claim ───────────────────────────────────────────

    function claimable(uint256 id) public view returns (uint128) {
        Schedule memory s = schedules[id];
        if (s.beneficiary == address(0)) return 0;
        uint128 vested = uint128(_vestedAt(s, uint64(block.timestamp)));
        if (vested <= s.claimedAmount) return 0;
        return vested - s.claimedAmount;
    }

    /// @notice Claim all currently-vested tokens for one schedule.
    function claim(uint256 id) external nonReentrant returns (uint128 amount) {
        Schedule storage s = schedules[id];
        if (msg.sender != s.beneficiary) revert NotBeneficiary();

        amount = claimable(id);
        if (amount == 0) revert NothingToClaim();

        s.claimedAmount += amount;
        token.safeTransfer(s.beneficiary, amount);
        emit Claimed(id, s.beneficiary, amount);
    }

    /// @notice Convenience: claim across all of caller's schedules.
    function claimAll() external nonReentrant returns (uint128 totalClaimed) {
        uint256[] memory ids = idsByBeneficiary[msg.sender];
        for (uint256 i = 0; i < ids.length; i++) {
            uint256 id = ids[i];
            uint128 amount = claimable(id);
            if (amount > 0) {
                schedules[id].claimedAmount += amount;
                totalClaimed += amount;
                emit Claimed(id, msg.sender, amount);
            }
        }
        if (totalClaimed == 0) revert NothingToClaim();
        token.safeTransfer(msg.sender, totalClaimed);
    }

    // ─── View ─────────────────────────────────────────────────────────

    function getSchedule(uint256 id) external view returns (Schedule memory) {
        return schedules[id];
    }

    function getSchedulesByBeneficiary(address beneficiary) external view returns (uint256[] memory) {
        return idsByBeneficiary[beneficiary];
    }

    /// @notice Total amount vested at `at` for schedule `id`.
    function vestedAmount(uint256 id, uint64 at) external view returns (uint128) {
        return uint128(_vestedAt(schedules[id], at));
    }

    // ─── Internals ────────────────────────────────────────────────────

    function _vestedAt(Schedule memory s, uint64 at) internal pure returns (uint256) {
        if (s.beneficiary == address(0)) return 0;

        // Pre-start: zero
        if (at < s.start) return 0;

        // Duration zero (e.g. fully accelerated) → full amount immediately.
        if (s.duration == 0) return s.totalAmount;

        // Pre-cliff: zero
        if (at < s.start + s.cliff) return 0;

        // Post-end: full
        uint64 elapsed = at - s.start;
        if (elapsed >= s.duration) return s.totalAmount;

        // Linear between cliff and duration.
        // Note: at the cliff exactly, fraction = cliff/duration vests
        //       all at once (so the cliff is a real cliff, not a delay).
        return (uint256(s.totalAmount) * elapsed) / s.duration;
    }
}
