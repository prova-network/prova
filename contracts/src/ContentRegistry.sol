// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Prova Network contributors.
pragma solidity ^0.8.20;

import {Cids} from "./Cids.sol";

/// @title ContentRegistry
/// @notice Maps content commitments (CommP) to deal IDs and ENS bindings.
/// @dev StorageMarketplace writes here when deals are created/completed.
///      Clients read to resolve commP → active deal → serving prover.
///      ENS binding is optional and purely informational; actual ENS
///      contenthash resolution happens in the ENS contracts.
contract ContentRegistry {
    // ───── Types ─────────────────────────────────────────────────────────

    /// @notice Represents one piece of content known to the registry.
    struct Content {
        address owner;          // who created the deal
        uint256 activeDealId;   // current deal serving this content (0 if none)
        uint64 pieceSize;       // size in bytes of the padded piece
        uint64 firstSeen;       // first registration timestamp
        uint64 lastUpdated;     // last write timestamp
        bytes32 ensNode;        // optional ENS node bound to this content
    }

    // ───── State ─────────────────────────────────────────────────────────

    /// @notice Content by CommP bytes (32-byte hash portion of the CID).
    mapping(bytes32 => Content) public contentByHash;

    /// @notice Reverse lookup: ENS node → CommP hash.
    mapping(bytes32 => bytes32) public commpByENS;

    /// @notice Address of the StorageMarketplace that is allowed to write here.
    address public marketplace;

    /// @notice Admin for setting the marketplace reference.
    address public admin;

    // ───── Events ────────────────────────────────────────────────────────

    event ContentRegistered(bytes32 indexed commpHash, address indexed owner, uint256 indexed dealId, uint64 pieceSize);
    event ContentDealUpdated(bytes32 indexed commpHash, uint256 oldDealId, uint256 newDealId);
    event ENSBound(bytes32 indexed commpHash, bytes32 indexed ensNode, address indexed by);
    event ENSUnbound(bytes32 indexed commpHash, bytes32 indexed ensNode);
    event MarketplaceSet(address indexed oldMarketplace, address indexed newMarketplace);

    // ───── Errors ────────────────────────────────────────────────────────

    error OnlyMarketplace();
    error OnlyAdmin();
    error NotContentOwner();
    error ContentNotFound();
    error ENSAlreadyBound();
    error ENSNotBoundHere();

    // ───── Construction ──────────────────────────────────────────────────

    constructor() {
        admin = msg.sender;
    }

    // ───── Admin ─────────────────────────────────────────────────────────

    function setMarketplace(address _marketplace) external {
        if (msg.sender != admin) revert OnlyAdmin();
        emit MarketplaceSet(marketplace, _marketplace);
        marketplace = _marketplace;
    }

    function transferAdmin(address newAdmin) external {
        if (msg.sender != admin) revert OnlyAdmin();
        admin = newAdmin;
    }

    // ───── Marketplace-Only Writes ───────────────────────────────────────

    /// @notice Register new content at the start of a deal.
    /// @dev Called by StorageMarketplace when a deal is accepted by a prover.
    function registerContent(bytes32 commpHash, address owner, uint256 dealId, uint64 pieceSize) external {
        if (msg.sender != marketplace) revert OnlyMarketplace();

        Content storage c = contentByHash[commpHash];
        if (c.firstSeen == 0) {
            // First registration
            c.owner = owner;
            c.firstSeen = uint64(block.timestamp);
            c.pieceSize = pieceSize;
        }

        uint256 previous = c.activeDealId;
        c.activeDealId = dealId;
        c.lastUpdated = uint64(block.timestamp);

        if (previous == 0) {
            emit ContentRegistered(commpHash, owner, dealId, pieceSize);
        } else {
            emit ContentDealUpdated(commpHash, previous, dealId);
        }
    }

    /// @notice Clear the active deal pointer (deal completed or cancelled).
    function clearActiveDeal(bytes32 commpHash, uint256 expectedDealId) external {
        if (msg.sender != marketplace) revert OnlyMarketplace();

        Content storage c = contentByHash[commpHash];
        if (c.activeDealId != expectedDealId) return; // no-op if stale

        c.activeDealId = 0;
        c.lastUpdated = uint64(block.timestamp);
        emit ContentDealUpdated(commpHash, expectedDealId, 0);
    }

    // ───── Owner-Controlled ENS Binding ──────────────────────────────────

    /// @notice Bind an ENS node to this content. Only the content owner can bind.
    /// @dev This is metadata only. Actual ENS contenthash must be set in the
    ///      ENS resolver separately. The binding here is an on-chain breadcrumb
    ///      so clients can discover "does this ENS name point to Prova content?".
    function bindENS(bytes32 commpHash, bytes32 ensNode) external {
        Content storage c = contentByHash[commpHash];
        if (c.firstSeen == 0) revert ContentNotFound();
        if (c.owner != msg.sender) revert NotContentOwner();

        // If the ENS node is already pointing somewhere, clear that binding first.
        bytes32 existing = commpByENS[ensNode];
        if (existing != bytes32(0) && existing != commpHash) {
            revert ENSAlreadyBound();
        }

        c.ensNode = ensNode;
        c.lastUpdated = uint64(block.timestamp);
        commpByENS[ensNode] = commpHash;

        emit ENSBound(commpHash, ensNode, msg.sender);
    }

    function unbindENS(bytes32 commpHash) external {
        Content storage c = contentByHash[commpHash];
        if (c.firstSeen == 0) revert ContentNotFound();
        if (c.owner != msg.sender) revert NotContentOwner();

        bytes32 node = c.ensNode;
        if (node == bytes32(0)) revert ENSNotBoundHere();

        c.ensNode = bytes32(0);
        c.lastUpdated = uint64(block.timestamp);
        delete commpByENS[node];

        emit ENSUnbound(commpHash, node);
    }

    // ───── Views ─────────────────────────────────────────────────────────

    /// @notice Full content record for a CommP hash.
    function getContent(bytes32 commpHash) external view returns (Content memory) {
        return contentByHash[commpHash];
    }

    /// @notice Resolve ENS node → content record. Returns zero-struct if unbound.
    function resolveENS(bytes32 ensNode) external view returns (Content memory) {
        bytes32 hash = commpByENS[ensNode];
        if (hash == bytes32(0)) return Content({
            owner: address(0),
            activeDealId: 0,
            pieceSize: 0,
            firstSeen: 0,
            lastUpdated: 0,
            ensNode: bytes32(0)
        });
        return contentByHash[hash];
    }

    /// @notice Whether content has an active deal (alias for dealId != 0).
    function hasActiveDeal(bytes32 commpHash) external view returns (bool) {
        return contentByHash[commpHash].activeDealId != 0;
    }
}
