// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Prova Network contributors.
pragma solidity ^0.8.20;

import {IERC20} from "@openzeppelin/contracts/token/ERC20/IERC20.sol";
import {SafeERC20} from "@openzeppelin/contracts/token/ERC20/utils/SafeERC20.sol";
import {Ownable} from "@openzeppelin/contracts/access/Ownable.sol";
import {ReentrancyGuard} from "@openzeppelin/contracts/utils/ReentrancyGuard.sol";

import {Cids} from "./Cids.sol";
import {ProverRegistry} from "./ProverRegistry.sol";
import {ProverStaking} from "./ProverStaking.sol";
import {ContentRegistry} from "./ContentRegistry.sol";

/// @dev Minimal interface of the ProverRewards emission contract.
///      We use only recordProof / recordMiss so the marketplace can
///      ping it on proof events without depending on the full ABI.
interface IProverRewards {
    function recordProof(address prover, address client, bytes32 pieceCid, uint256 bytesProven) external;
    function recordMiss(address prover) external;
}

/// @dev Minimal subset of the ProofVerifier listener interface we implement.
///      Matches the `PDPListener` defined inside ProofVerifier.sol.
interface IPDPListener {
    function dataSetCreated(uint256 dataSetId, address creator, bytes calldata extraData) external;
    function dataSetDeleted(uint256 dataSetId, uint256 deletedLeafCount, bytes calldata extraData) external;
    function piecesAdded(uint256 dataSetId, uint256 firstAdded, Cids.Cid[] memory pieceData, bytes calldata extraData) external;
    function piecesScheduledRemove(uint256 dataSetId, uint256[] memory pieceIds, bytes calldata extraData) external;
    function possessionProven(uint256 dataSetId, uint256 challengedLeafCount, uint256 seed, uint256 challengeCount) external;
    function nextProvingPeriod(uint256 dataSetId, uint256 challengeEpoch, uint256 leafCount, bytes calldata extraData) external;
    function storageProviderChanged(
        uint256 dataSetId,
        address oldStorageProvider,
        address newStorageProvider,
        bytes calldata extraData
    ) external;
}

/// @title StorageMarketplace
/// @notice Orchestrates storage deals between clients and provers.
/// @dev Implements the PDPListener interface and is registered as the listener
///      when ProofVerifier.createDataSet is called. Deals are the unit of
///      payment and proof accountability.
///
///      Lifecycle:
///          Proposed -> Active -> Completed | Cancelled | Slashed
///
///      Proposed:  client has created a deal, escrow is locked, waiting for
///                 prover acceptance.
///      Active:    prover has accepted and is storing + proving. Streaming
///                 payment releases to prover on each successful proof.
///      Completed: deal duration elapsed, all funds released.
///      Cancelled: client cancelled before prover acceptance.
///      Slashed:   prover failed sufficient proofs; remaining escrow returned
///                 to client, prover slashed via ProverStaking.
contract StorageMarketplace is IPDPListener, Ownable, ReentrancyGuard {
    using SafeERC20 for IERC20;

    // ───── Types ─────────────────────────────────────────────────────────

    enum DealStatus {
        None,
        Proposed,
        Active,
        Completed,
        Cancelled,
        Slashed
    }

    struct Deal {
        address client;        // who created the deal
        address prover;        // selected prover (target of deal)
        bytes32 commpHash;     // content commitment (32-byte portion of CommP CID)
        uint64 pieceSize;      // padded piece size in bytes
        uint64 startedAt;      // block.timestamp when prover accepted
        uint64 endsAt;         // block.timestamp when deal naturally completes
        uint256 dataSetId;     // ProofVerifier data set id (set on acceptance)
        uint256 totalPayment;  // total payment locked at creation (in payment token units)
        uint256 paidOut;       // cumulative payment released to prover
        uint256 lastProofAt;   // timestamp of most recent successful proof
        uint256 proofCount;    // number of successful proofs
        DealStatus status;
    }

    // ───── Constants ─────────────────────────────────────────────────────

    /// @notice Maximum proving gap before a deal can be faulted by anyone.
    /// @dev If prover misses proofs for this long, anyone can call faultDeal()
    ///      to trigger slashing. Conservative default; tune with real data.
    uint256 public constant MAX_PROOF_GAP = 3 days;

    /// @notice Minimum deal duration.
    uint256 public constant MIN_DEAL_DURATION = 1 days;

    /// @notice Maximum deal duration (10 years).
    uint256 public constant MAX_DEAL_DURATION = 10 * 365 days;

    /// @notice Protocol fee in basis points (deducted from payment to prover).
    uint256 public protocolFeeBps = 100; // 1%
    uint256 public constant BPS_DENOMINATOR = 10_000;

    /// @notice Slash amount per faulted deal (absolute tokens).
    uint256 public slashPerFault;

    // ───── State ─────────────────────────────────────────────────────────

    /// @notice Next deal id to assign.
    uint256 public nextDealId = 1;

    /// @notice Deals by id.
    mapping(uint256 => Deal) public deals;

    /// @notice Reverse lookup: ProofVerifier dataSetId → dealId.
    mapping(uint256 => uint256) public dealIdByDataSet;

    /// @notice Treasury address receiving protocol fees.
    address public treasury;

    /// @notice The PDP verifier contract (on Base).
    address public immutable proofVerifier;

    /// @notice The payment token (PROVA or USDC, depending on deal). For v1
    ///         we use a single token; v2 can be multi-token.
    IERC20 public immutable paymentToken;

    /// @notice The prover registry.
    ProverRegistry public immutable proverRegistry;

    /// @notice The prover staking contract.
    ProverStaking public immutable proverStaking;

    /// @notice The content registry.
    ContentRegistry public immutable contentRegistry;

    /// @notice The prover-rewards (PROVA emission) contract. Optional;
    ///         when set, the marketplace pings it on every successful
    ///         proof so emission can accrue. Settable by owner so existing
    ///         deployments can opt in without redeploying the marketplace.
    address public proverRewards;

    // ───── Events ────────────────────────────────────────────────────────

    event DealProposed(
        uint256 indexed dealId,
        address indexed client,
        address indexed prover,
        bytes32 commpHash,
        uint64 pieceSize,
        uint64 durationSeconds,
        uint256 totalPayment
    );
    event DealAccepted(uint256 indexed dealId, address indexed prover, uint256 dataSetId, uint64 endsAt);
    event DealCompleted(uint256 indexed dealId, uint256 finalPaidOut);
    event DealCancelled(uint256 indexed dealId, uint256 refund);
    event DealSlashed(uint256 indexed dealId, address indexed prover, uint256 slashedAmount, uint256 refunded);
    event ProofRecorded(uint256 indexed dealId, uint256 proofCount, uint256 paymentReleased);
    event ProtocolFeeChanged(uint256 oldBps, uint256 newBps);
    event TreasuryChanged(address indexed oldTreasury, address indexed newTreasury);
    event SlashPerFaultChanged(uint256 oldValue, uint256 newValue);
    event ProverRewardsSet(address indexed previous, address indexed next);

    // ───── Errors ────────────────────────────────────────────────────────

    error OnlyProofVerifier();
    error OnlyProver();
    error OnlyClient();
    error DealNotProposed();
    error DealNotActive();
    error InvalidDuration();
    error InvalidPayment();
    error ProverNotActive();
    error ProverCannotCommit();
    error WrongDataSetOwner();
    error ProverMismatch();
    error ProofGapTooSmall();

    // ───── Modifiers ─────────────────────────────────────────────────────

    modifier onlyProofVerifier() {
        if (msg.sender != proofVerifier) revert OnlyProofVerifier();
        _;
    }

    // ───── Construction ──────────────────────────────────────────────────

    constructor(
        address _proofVerifier,
        IERC20 _paymentToken,
        ProverRegistry _proverRegistry,
        ProverStaking _proverStaking,
        ContentRegistry _contentRegistry,
        address _treasury,
        uint256 _slashPerFault
    ) Ownable(msg.sender) {
        proofVerifier = _proofVerifier;
        paymentToken = _paymentToken;
        proverRegistry = _proverRegistry;
        proverStaking = _proverStaking;
        contentRegistry = _contentRegistry;
        treasury = _treasury;
        slashPerFault = _slashPerFault;
    }

    // ───── Admin ─────────────────────────────────────────────────────────

    function setProtocolFeeBps(uint256 newBps) external onlyOwner {
        require(newBps <= 1000, "fee too high"); // cap at 10%
        emit ProtocolFeeChanged(protocolFeeBps, newBps);
        protocolFeeBps = newBps;
    }

    function setProverRewards(address newProverRewards) external onlyOwner {
        emit ProverRewardsSet(proverRewards, newProverRewards);
        proverRewards = newProverRewards;
    }

    function setTreasury(address newTreasury) external onlyOwner {
        emit TreasuryChanged(treasury, newTreasury);
        treasury = newTreasury;
    }

    function setSlashPerFault(uint256 newValue) external onlyOwner {
        emit SlashPerFaultChanged(slashPerFault, newValue);
        slashPerFault = newValue;
    }

    // ───── Deal Creation ─────────────────────────────────────────────────

    /// @notice Propose a deal with a specific prover.
    /// @dev Caller must have approved `totalPayment` of paymentToken to this
    ///      contract. Funds are pulled in and held in escrow.
    /// @param prover Target prover (must be active in ProverRegistry).
    /// @param commpHash 32-byte content commitment hash.
    /// @param pieceSize Padded piece size in bytes.
    /// @param durationSeconds How long the deal runs.
    /// @param totalPayment Total payment locked for this deal.
    function proposeDeal(
        address prover,
        bytes32 commpHash,
        uint64 pieceSize,
        uint64 durationSeconds,
        uint256 totalPayment
    ) external nonReentrant returns (uint256 dealId) {
        if (durationSeconds < MIN_DEAL_DURATION || durationSeconds > MAX_DEAL_DURATION) {
            revert InvalidDuration();
        }
        if (totalPayment == 0) revert InvalidPayment();
        if (pieceSize == 0) revert InvalidPayment();
        if (!proverRegistry.isActive(prover)) revert ProverNotActive();
        if (!proverStaking.canCommit(prover, pieceSize)) revert ProverCannotCommit();

        dealId = nextDealId++;

        deals[dealId] = Deal({
            client: msg.sender,
            prover: prover,
            commpHash: commpHash,
            pieceSize: pieceSize,
            startedAt: 0,
            endsAt: 0,
            dataSetId: 0,
            totalPayment: totalPayment,
            paidOut: 0,
            lastProofAt: 0,
            proofCount: 0,
            status: DealStatus.Proposed
        });

        // Pull escrow from client
        paymentToken.safeTransferFrom(msg.sender, address(this), totalPayment);

        emit DealProposed(dealId, msg.sender, prover, commpHash, pieceSize, durationSeconds, totalPayment);

        // Store duration in endsAt slot as "relative duration" until acceptance.
        // On acceptance we resolve to absolute timestamp. Small trick to avoid
        // a separate durationSeconds field in the struct.
        deals[dealId].endsAt = durationSeconds;
    }

    /// @notice Client cancels a proposed deal before prover acceptance.
    ///         Returns full escrow to client.
    function cancelProposedDeal(uint256 dealId) external nonReentrant {
        Deal storage d = deals[dealId];
        if (d.status != DealStatus.Proposed) revert DealNotProposed();
        if (msg.sender != d.client) revert OnlyClient();

        d.status = DealStatus.Cancelled;

        paymentToken.safeTransfer(d.client, d.totalPayment);
        emit DealCancelled(dealId, d.totalPayment);
    }

    /// @notice Prover accepts a deal by calling ProofVerifier.createDataSet
    ///         with this contract as the listener. ProofVerifier will call
    ///         dataSetCreated() on us, at which point we activate the deal.
    /// @dev Prover passes the dealId as extraData when calling createDataSet.
    ///      No direct entrypoint here; acceptance happens via the listener hook.

    // ───── PDPListener Hooks ─────────────────────────────────────────────

    /// @notice Called by ProofVerifier when a new data set is created.
    ///         `creator` is the prover; `extraData` must be abi-encoded dealId.
    function dataSetCreated(uint256 dataSetId, address creator, bytes calldata extraData)
        external
        override
        onlyProofVerifier
    {
        uint256 dealId = abi.decode(extraData, (uint256));
        Deal storage d = deals[dealId];

        if (d.status != DealStatus.Proposed) revert DealNotProposed();
        if (creator != d.prover) revert ProverMismatch();

        // Activate the deal
        uint64 durationSeconds = d.endsAt; // still holds the relative duration
        d.startedAt = uint64(block.timestamp);
        d.endsAt = uint64(block.timestamp + durationSeconds);
        d.dataSetId = dataSetId;
        d.status = DealStatus.Active;
        d.lastProofAt = block.timestamp; // grace period starts now

        dealIdByDataSet[dataSetId] = dealId;

        // Notify staking + content registry
        proverStaking.commitBytes(d.prover, d.pieceSize);
        contentRegistry.registerContent(d.commpHash, d.client, dealId, d.pieceSize);

        emit DealAccepted(dealId, d.prover, dataSetId, d.endsAt);
    }

    /// @notice Called by ProofVerifier when a data set is deleted.
    function dataSetDeleted(uint256 dataSetId, uint256 /*deletedLeafCount*/, bytes calldata /*extraData*/)
        external
        override
        onlyProofVerifier
    {
        uint256 dealId = dealIdByDataSet[dataSetId];
        Deal storage d = deals[dealId];
        if (d.status != DealStatus.Active) return; // already terminal

        // Treat as prover walking away: refund remaining escrow to client.
        _terminateAsSlashed(dealId, d);
    }

    /// @notice Called when pieces are added to an existing data set.
    ///         For v1 we only support one piece per data set, so this is a no-op.
    function piecesAdded(
        uint256 /*dataSetId*/,
        uint256 /*firstAdded*/,
        Cids.Cid[] memory /*pieceData*/,
        bytes calldata /*extraData*/
    ) external override onlyProofVerifier {
        // no-op in v1
    }

    function piecesScheduledRemove(
        uint256 /*dataSetId*/,
        uint256[] memory /*pieceIds*/,
        bytes calldata /*extraData*/
    ) external override onlyProofVerifier {
        // no-op in v1
    }

    /// @notice Called by ProofVerifier after a successful possession proof.
    ///         This is where we release the streaming payment to the prover.
    function possessionProven(
        uint256 dataSetId,
        uint256 /*challengedLeafCount*/,
        uint256 /*seed*/,
        uint256 /*challengeCount*/
    ) external override onlyProofVerifier {
        uint256 dealId = dealIdByDataSet[dataSetId];
        Deal storage d = deals[dealId];
        if (d.status != DealStatus.Active) return;

        // ── CEI order: state, internal effects, external interactions ──
        // 1. State: bump proof counters
        d.proofCount += 1;
        d.lastProofAt = block.timestamp;

        // 2. State: compute and accrue the streaming release before any
        //    external calls. This closes a slither-flagged reentrancy where
        //    the rewards-contract callback could re-enter the marketplace
        //    while paidOut was still stale.
        uint256 released = _computeStreamingRelease(d);
        uint256 fee = 0;
        uint256 proverShare = 0;
        if (released > 0) {
            d.paidOut += released;
            fee = (released * protocolFeeBps) / BPS_DENOMINATOR;
            proverShare = released - fee;
        }

        // 3. Interactions: USDC transfers and rewards hook. State is fully
        //    settled by this point; reentrancy can read but not exploit
        //    inconsistent intermediate state.
        if (released > 0) {
            if (fee > 0 && treasury != address(0)) {
                paymentToken.safeTransfer(treasury, fee);
            }
            paymentToken.safeTransfer(d.prover, proverShare);
        }

        // PROVA emission hook (optional). Wrapped in try/catch so a
        // misconfigured or hostile rewards contract can never block a
        // payment. The marketplace's job is to pay USDC; emission is a
        // bonus that runs after all financial state has settled.
        if (proverRewards != address(0)) {
            try IProverRewards(proverRewards).recordProof(
                d.prover,
                d.client,
                d.commpHash,
                d.pieceSize
            ) {} catch {}
        }

        emit ProofRecorded(dealId, d.proofCount, released);
    }

    function nextProvingPeriod(
        uint256 /*dataSetId*/,
        uint256 /*challengeEpoch*/,
        uint256 /*leafCount*/,
        bytes calldata /*extraData*/
    ) external override onlyProofVerifier {
        // no-op in v1; could be used for telemetry
    }

    function storageProviderChanged(
        uint256 dataSetId,
        address /*oldStorageProvider*/,
        address newStorageProvider,
        bytes calldata /*extraData*/
    ) external override onlyProofVerifier {
        // For v1 we don't support mid-deal prover change. Refuse by terminating.
        uint256 dealId = dealIdByDataSet[dataSetId];
        Deal storage d = deals[dealId];
        if (d.status != DealStatus.Active) return;

        // Hostile takeover attempt or legitimate migration? Either way, v1 rule:
        // the deal is done, refund client, slash original prover for walking.
        _terminateAsSlashed(dealId, d);

        // Silence unused-variable warning
        newStorageProvider;
    }

    // ───── Public Lifecycle Actions ──────────────────────────────────────

    /// @notice Anyone can call this to complete a deal that has reached endsAt.
    ///         Remaining escrow is released to prover, and committed bytes are freed.
    function completeDeal(uint256 dealId) external nonReentrant {
        Deal storage d = deals[dealId];
        if (d.status != DealStatus.Active) revert DealNotActive();
        require(block.timestamp >= d.endsAt, "deal not ended");

        d.status = DealStatus.Completed;

        // Release any remaining unpaid amount
        uint256 remaining = d.totalPayment - d.paidOut;
        if (remaining > 0) {
            d.paidOut = d.totalPayment;

            uint256 fee = (remaining * protocolFeeBps) / BPS_DENOMINATOR;
            uint256 proverShare = remaining - fee;

            if (fee > 0 && treasury != address(0)) {
                paymentToken.safeTransfer(treasury, fee);
            }
            paymentToken.safeTransfer(d.prover, proverShare);
        }

        proverStaking.releaseBytes(d.prover, d.pieceSize);
        contentRegistry.clearActiveDeal(d.commpHash, dealId);

        emit DealCompleted(dealId, d.paidOut);
    }

    /// @notice Anyone can fault a deal where the prover has gone silent for
    ///         longer than MAX_PROOF_GAP. Slashes the prover and refunds
    ///         the client's remaining escrow.
    function faultDeal(uint256 dealId) external nonReentrant {
        Deal storage d = deals[dealId];
        if (d.status != DealStatus.Active) revert DealNotActive();
        if (block.timestamp < d.lastProofAt + MAX_PROOF_GAP) revert ProofGapTooSmall();

        _terminateAsSlashed(dealId, d);
    }

    // ───── Internal ──────────────────────────────────────────────────────

    function _terminateAsSlashed(uint256 dealId, Deal storage d) internal {
        d.status = DealStatus.Slashed;

        uint256 refund = d.totalPayment - d.paidOut;
        d.paidOut = d.totalPayment; // mark fully consumed

        if (refund > 0) {
            paymentToken.safeTransfer(d.client, refund);
        }

        // Slash prover's stake
        proverStaking.slash(d.prover, slashPerFault, bytes32(uint256(dealId)));
        proverStaking.releaseBytes(d.prover, d.pieceSize);
        contentRegistry.clearActiveDeal(d.commpHash, dealId);

        // PROVA emission hook: record the miss so the prover's quality
        // multiplier reflects the slashing event.
        if (proverRewards != address(0)) {
            try IProverRewards(proverRewards).recordMiss(d.prover) {} catch {}
        }

        emit DealSlashed(dealId, d.prover, slashPerFault, refund);
    }

    /// @notice Compute how much payment should release on this proof event.
    /// @dev Linear release: totalPayment * (timeSinceStart / duration), minus
    ///      what's already been paid out. Bounded by totalPayment.
    function _computeStreamingRelease(Deal storage d) internal view returns (uint256) {
        uint256 totalDuration = d.endsAt - d.startedAt;
        if (totalDuration == 0) return 0;

        uint256 elapsed = block.timestamp - d.startedAt;
        if (elapsed > totalDuration) elapsed = totalDuration;

        uint256 owed = (d.totalPayment * elapsed) / totalDuration;
        if (owed <= d.paidOut) return 0;
        return owed - d.paidOut;
    }

    // ───── Views ─────────────────────────────────────────────────────────

    function getDeal(uint256 dealId) external view returns (Deal memory) {
        return deals[dealId];
    }

    function isActive(uint256 dealId) external view returns (bool) {
        return deals[dealId].status == DealStatus.Active;
    }

    /// @notice How much of a deal's payment could be released right now.
    function pendingRelease(uint256 dealId) external view returns (uint256) {
        Deal storage d = deals[dealId];
        if (d.status != DealStatus.Active) return 0;
        return _computeStreamingRelease(d);
    }
}
