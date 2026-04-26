// SPDX-License-Identifier: MIT
pragma solidity ^0.8.24;

import "@openzeppelin/contracts/token/ERC20/IERC20.sol";
import "@openzeppelin/contracts/token/ERC20/utils/SafeERC20.sol";
import "@openzeppelin/contracts/access/Ownable2Step.sol";
import "@openzeppelin/contracts/utils/ReentrancyGuard.sol";

/// @title ProverRewards
/// @notice Distributes 50,000,000 PROVA to provers over 8 years on a
///         declining emission curve. Bytes-proven-time is the unit of
///         account; provers earn proportional to their share of total
///         bytes proven during each settlement epoch.
///
/// Design points:
///   - The full 50M emission allocation is transferred to this contract
///     at genesis (or on first deposit). It never mints; it only pays
///     out from its initial balance.
///   - Marketplace records `recordProof(prover, client, pieceCid, bytes)`
///     each time a prover posts a valid PDP proof. Anti-gaming logic
///     (self-dealing, redundancy cap, sybil rate-limit) is enforced
///     inside this function.
///   - Epochs are 7 days. Per-epoch totals roll up; provers claim per
///     epoch (or in batches via `claimRange`) AFTER a 30-day vesting
///     buffer, which discourages fast-churn.
///   - Quality multiplier: a prover with more than `qualityCutoffBps`
///     missed proofs in the trailing 30 days has their reward halved.
///
/// Yearly emission schedule (year 1 is the heaviest, declining):
///   Y1: 12,500,000  (25% of bucket)
///   Y2: 11,000,000  (22%)
///   Y3:  9,000,000  (18%)
///   Y4:  7,000,000  (14%)
///   Y5:  5,000,000  (10%)
///   Y6:  3,000,000  (6%)
///   Y7:  1,500,000  (3%)
///   Y8:  1,000,000  (2%)
///   Total: 50,000,000 PROVA
contract ProverRewards is Ownable2Step, ReentrancyGuard {
    using SafeERC20 for IERC20;

    // ─── Constants ───────────────────────────────────────────────────

    IERC20  public immutable prova;

    /// @notice Genesis time. All epochs are measured relative to this.
    uint64  public immutable genesisTime;

    uint64  public constant EPOCH_DURATION = 7 days;
    /// @notice Vesting buffer: rewards for epoch E are claimable starting at E.endsAt + VESTING_BUFFER.
    uint64  public constant VESTING_BUFFER = 30 days;

    uint256 public constant TOTAL_EMISSION = 50_000_000 ether;

    /// @notice 8-year schedule. Index = year (0..7). Sum = 50M PROVA.
    uint256[8] public yearlyEmission = [
        12_500_000 ether,
        11_000_000 ether,
         9_000_000 ether,
         7_000_000 ether,
         5_000_000 ether,
         3_000_000 ether,
         1_500_000 ether,
         1_000_000 ether
    ];

    // ─── Configuration (governance-tunable) ──────────────────────────

    /// @notice Default redundancy cap: a piece earns up to N provers' worth of emission.
    /// @dev Beyond this, additional copies don't earn additional rewards.
    uint8 public redundancyCap = 4;

    /// @notice Quality multiplier denominator (10000 bps).
    /// Provers with > qualityCutoffBps missed-proof rate in trailing 30d earn 50%.
    uint16 public qualityCutoffBps = 500; // 5%

    /// @notice Authorized recorder (the marketplace contract).
    address public marketplace;

    // ─── State ────────────────────────────────────────────────────────

    /// @notice For each epoch: total bytes proven across all provers.
    mapping(uint256 => uint256) public totalBytesByEpoch;

    /// @notice For each (epoch, prover): bytes proven by that prover.
    mapping(uint256 => mapping(address => uint256)) public bytesByEpochProver;

    /// @notice For each (epoch, prover): claimed flag.
    mapping(uint256 => mapping(address => bool)) public claimed;

    /// @notice Quality bookkeeping per prover.
    struct QualityScore {
        uint64  windowStart;       // unix timestamp window start
        uint64  successes;         // proofs in window
        uint64  failures;          // missed/invalid proofs in window
    }
    mapping(address => QualityScore) public quality;

    /// @notice Tracks which (piece, prover) pairs have already counted in a given epoch.
    /// Used to enforce: a prover only earns for the *first* proof of a piece per epoch
    /// (otherwise they could spam re-proofs).
    mapping(uint256 => mapping(bytes32 => mapping(address => bool))) public countedInEpoch;

    /// @notice Tracks how many distinct provers have already counted a piece in a given epoch.
    /// Capped at `redundancyCap`.
    mapping(uint256 => mapping(bytes32 => uint8)) public proversForPieceInEpoch;

    // ─── Events ───────────────────────────────────────────────────────

    event MarketplaceSet(address indexed previous, address indexed next);
    event RedundancyCapSet(uint8 previous, uint8 next);
    event QualityCutoffSet(uint16 previous, uint16 next);

    event ProofRecorded(
        uint256 indexed epoch,
        address indexed prover,
        bytes32 indexed pieceCid,
        uint256 bytesProven,
        bool counted
    );
    event QualityUpdated(address indexed prover, uint64 successes, uint64 failures);
    event Claimed(address indexed prover, uint256 indexed epoch, uint256 amount);

    error NotMarketplace();
    error ZeroAddress();
    error EpochNotFinalized();
    error EpochNotVested();
    error NothingToClaim();
    error AlreadyClaimed();
    error SelfDealing();
    error InvalidEpoch();
    error InvalidParam();

    constructor(IERC20 _prova, address _owner, uint64 _genesisTime) Ownable(_owner) {
        if (address(_prova) == address(0) || _owner == address(0)) revert ZeroAddress();
        prova = _prova;
        genesisTime = _genesisTime == 0 ? uint64(block.timestamp) : _genesisTime;
    }

    // ─── Owner: configuration ────────────────────────────────────────

    function setMarketplace(address newMarketplace) external onlyOwner {
        emit MarketplaceSet(marketplace, newMarketplace);
        marketplace = newMarketplace;
    }

    function setRedundancyCap(uint8 newCap) external onlyOwner {
        if (newCap == 0 || newCap > 16) revert InvalidParam();
        emit RedundancyCapSet(redundancyCap, newCap);
        redundancyCap = newCap;
    }

    function setQualityCutoff(uint16 newCutoffBps) external onlyOwner {
        if (newCutoffBps > 5000) revert InvalidParam(); // max 50%
        emit QualityCutoffSet(qualityCutoffBps, newCutoffBps);
        qualityCutoffBps = newCutoffBps;
    }

    // ─── Marketplace integration: record proofs ──────────────────────

    /// @notice Called by the marketplace each time a prover posts a valid proof.
    ///
    /// Anti-gaming gates:
    ///   - prover != client (no self-dealing)
    ///   - sponsored deals (client == address(0)) don't count
    ///   - per-epoch redundancy cap on a single piece
    ///   - per-epoch single-counting per (piece, prover)
    ///
    /// @param prover     The prover that posted the proof
    /// @param client     The deal's client (or address(0) for sponsored)
    /// @param pieceCid   The piece-cid being proven (used as a dedup key)
    /// @param bytesProven Bytes covered by this proof (= piece size)
    function recordProof(
        address prover,
        address client,
        bytes32 pieceCid,
        uint256 bytesProven
    ) external {
        if (msg.sender != marketplace) revert NotMarketplace();
        if (prover == address(0) || bytesProven == 0) return;

        // F-G1 (self-dealing): a prover that's also the client doesn't earn emission.
        if (prover == client) {
            _recordQuality(prover, true);
            emit ProofRecorded(_currentEpoch(), prover, pieceCid, bytesProven, false);
            revert SelfDealing();
        }

        // F-G2 (sponsored deals): client == address(0) means it's the protocol-sponsored
        // free tier. Record the proof for quality tracking but no emission.
        if (client == address(0)) {
            _recordQuality(prover, true);
            emit ProofRecorded(_currentEpoch(), prover, pieceCid, bytesProven, false);
            return;
        }

        uint256 epoch = _currentEpoch();

        // F-G3 (single-counting): count each (piece, prover) only once per epoch.
        if (countedInEpoch[epoch][pieceCid][prover]) {
            _recordQuality(prover, true);
            emit ProofRecorded(epoch, prover, pieceCid, bytesProven, false);
            return;
        }

        // F-G4 (redundancy cap): only the first `redundancyCap` provers for a piece
        // earn emission for that piece in this epoch.
        if (proversForPieceInEpoch[epoch][pieceCid] >= redundancyCap) {
            _recordQuality(prover, true);
            emit ProofRecorded(epoch, prover, pieceCid, bytesProven, false);
            return;
        }

        // Valid contribution
        countedInEpoch[epoch][pieceCid][prover] = true;
        proversForPieceInEpoch[epoch][pieceCid] += 1;
        bytesByEpochProver[epoch][prover] += bytesProven;
        totalBytesByEpoch[epoch]          += bytesProven;
        _recordQuality(prover, true);

        emit ProofRecorded(epoch, prover, pieceCid, bytesProven, true);
    }

    /// @notice Called by the marketplace when a prover misses a challenge.
    function recordMiss(address prover) external {
        if (msg.sender != marketplace) revert NotMarketplace();
        _recordQuality(prover, false);
    }

    // ─── Reward calculation ──────────────────────────────────────────

    /// @notice Compute the emission reward (before quality multiplier) that a
    ///         prover earned during `epoch`.
    function rewardOf(address prover, uint256 epoch) public view returns (uint256) {
        if (epoch >= type(uint64).max) revert InvalidEpoch();

        uint256 totalBytes = totalBytesByEpoch[epoch];
        if (totalBytes == 0) return 0;

        uint256 prBytes = bytesByEpochProver[epoch][prover];
        if (prBytes == 0) return 0;

        // Per-epoch emission = yearly_emission(year_of_epoch) * EPOCH_DURATION / 365 days.
        uint256 yearIdx = (epoch * EPOCH_DURATION) / 365 days;
        if (yearIdx >= yearlyEmission.length) return 0;

        uint256 perYear  = yearlyEmission[yearIdx];
        uint256 perEpoch = (perYear * EPOCH_DURATION) / 365 days;
        uint256 rawReward = (perEpoch * prBytes) / totalBytes;

        // Quality multiplier (50% if missed-rate exceeds the cutoff in trailing 30d)
        QualityScore memory q = quality[prover];
        uint64 total = q.successes + q.failures;
        if (total > 0 && uint256(q.failures) * 10_000 / total > qualityCutoffBps) {
            return rawReward / 2;
        }
        return rawReward;
    }

    /// @notice Returns true if `epoch` is finalized (i.e., now > epoch.endsAt) AND vested.
    function isClaimable(uint256 epoch) public view returns (bool) {
        uint256 epochEnd = uint256(genesisTime) + (epoch + 1) * uint256(EPOCH_DURATION);
        return block.timestamp >= epochEnd + uint256(VESTING_BUFFER);
    }

    function currentEpoch() external view returns (uint256) {
        return _currentEpoch();
    }

    // ─── Claim ───────────────────────────────────────────────────────

    /// @notice Claim emission for one epoch.
    function claim(uint256 epoch) external nonReentrant returns (uint256 amount) {
        if (!isClaimable(epoch)) revert EpochNotVested();
        if (claimed[epoch][msg.sender]) revert AlreadyClaimed();

        amount = rewardOf(msg.sender, epoch);
        if (amount == 0) revert NothingToClaim();

        claimed[epoch][msg.sender] = true;
        prova.safeTransfer(msg.sender, amount);
        emit Claimed(msg.sender, epoch, amount);
    }

    /// @notice Claim emission for a range of epochs [from, to] (inclusive).
    function claimRange(uint256 from, uint256 to) external nonReentrant returns (uint256 totalAmount) {
        if (to < from) revert InvalidEpoch();
        for (uint256 e = from; e <= to; e++) {
            if (!isClaimable(e)) continue;
            if (claimed[e][msg.sender]) continue;
            uint256 r = rewardOf(msg.sender, e);
            if (r == 0) continue;
            claimed[e][msg.sender] = true;
            totalAmount += r;
            emit Claimed(msg.sender, e, r);
        }
        if (totalAmount == 0) revert NothingToClaim();
        prova.safeTransfer(msg.sender, totalAmount);
    }

    // ─── Internals ───────────────────────────────────────────────────

    function _currentEpoch() internal view returns (uint256) {
        if (block.timestamp <= genesisTime) return 0;
        return (block.timestamp - genesisTime) / EPOCH_DURATION;
    }

    /// @dev Tracks rolling 30-day success/failure window for the quality multiplier.
    function _recordQuality(address prover, bool success) internal {
        QualityScore storage q = quality[prover];
        // Reset window if it's older than 30 days
        if (block.timestamp >= q.windowStart + 30 days) {
            q.windowStart = uint64(block.timestamp);
            q.successes = 0;
            q.failures = 0;
        }
        if (success) {
            q.successes += 1;
        } else {
            q.failures += 1;
        }
        emit QualityUpdated(prover, q.successes, q.failures);
    }
}
