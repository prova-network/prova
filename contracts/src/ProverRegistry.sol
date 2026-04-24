// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Prova Network contributors.
pragma solidity ^0.8.20;

import {Ownable} from "@openzeppelin/contracts/access/Ownable.sol";

/// @title ProverRegistry
/// @notice On-chain registry of storage provers for the Prova network.
/// @dev Provers self-register with their network endpoint, features, and pricing.
///      Clients query the registry to discover provers. Staking is enforced
///      separately by ProverStaking; this contract is pure metadata.
contract ProverRegistry is Ownable {
    // ───── Types ─────────────────────────────────────────────────────────

    /// @notice Feature flags a prover advertises.
    /// @dev bit 0 = PDP (required), bit 1 = HTTPS serving, bit 2 = TEE, bit 3 = QBP (v2)
    struct Prover {
        address owner;          // Ethereum address controlling this prover entry
        string endpoint;        // HTTPS URL or multiaddr (provider-controlled)
        uint64 features;        // bitmap of capabilities
        uint128 pricePerGibDay; // price per GiB-day, in wei (or token min-unit)
        uint128 pricePerByteServed; // price per byte of HTTPS retrieval traffic
        uint64 registeredAt;    // block timestamp of registration
        uint64 updatedAt;       // last update timestamp
        bool active;            // soft-deletable via deregister()
        bytes32 ensNode;        // optional ENS node for human-readable ID
        string metadata;        // arbitrary JSON (region, contact, etc.)
    }

    // ───── Constants ─────────────────────────────────────────────────────

    uint64 public constant FEATURE_PDP           = 1 << 0;
    uint64 public constant FEATURE_HTTPS_SERVING = 1 << 1;
    uint64 public constant FEATURE_TEE           = 1 << 2;
    uint64 public constant FEATURE_QBP           = 1 << 3;

    uint256 public constant MAX_ENDPOINT_LENGTH = 512;
    uint256 public constant MAX_METADATA_LENGTH = 2048;

    // ───── State ─────────────────────────────────────────────────────────

    /// @notice Registered provers by address.
    mapping(address => Prover) public provers;

    /// @notice List of all prover addresses ever registered, in order.
    /// @dev Used for enumeration; check `active` before treating as live.
    address[] public proverAddresses;

    /// @notice Whether an address has ever been registered (used to avoid double-enqueue).
    mapping(address => bool) public known;

    // ───── Events ────────────────────────────────────────────────────────

    event ProverRegistered(address indexed prover, string endpoint, uint64 features);
    event ProverUpdated(address indexed prover, string endpoint, uint64 features);
    event ProverDeregistered(address indexed prover);
    event PriceChanged(address indexed prover, uint128 pricePerGibDay, uint128 pricePerByteServed);
    event ENSBound(address indexed prover, bytes32 ensNode);

    // ───── Errors ────────────────────────────────────────────────────────

    error EndpointTooLong();
    error MetadataTooLong();
    error NotRegistered();
    error AlreadyRegistered();
    error InvalidFeatures();
    error NotOwner();

    // ───── Construction ──────────────────────────────────────────────────

    constructor() Ownable(msg.sender) {}

    // ───── Registration ──────────────────────────────────────────────────

    /// @notice Register as a prover. Called once per prover; subsequent changes use `update*`.
    /// @param endpoint HTTPS URL or libp2p multiaddr.
    /// @param features Bitmap of supported features. Must include FEATURE_PDP.
    /// @param pricePerGibDay Price per GiB-day in the base payment token's smallest unit.
    /// @param pricePerByteServed Price per byte of retrieval traffic.
    /// @param metadata Arbitrary JSON for additional prover metadata.
    function register(
        string calldata endpoint,
        uint64 features,
        uint128 pricePerGibDay,
        uint128 pricePerByteServed,
        string calldata metadata
    ) external {
        if (provers[msg.sender].active) revert AlreadyRegistered();
        if (bytes(endpoint).length > MAX_ENDPOINT_LENGTH) revert EndpointTooLong();
        if (bytes(metadata).length > MAX_METADATA_LENGTH) revert MetadataTooLong();
        if ((features & FEATURE_PDP) == 0) revert InvalidFeatures();

        provers[msg.sender] = Prover({
            owner: msg.sender,
            endpoint: endpoint,
            features: features,
            pricePerGibDay: pricePerGibDay,
            pricePerByteServed: pricePerByteServed,
            registeredAt: uint64(block.timestamp),
            updatedAt: uint64(block.timestamp),
            active: true,
            ensNode: bytes32(0),
            metadata: metadata
        });

        if (!known[msg.sender]) {
            known[msg.sender] = true;
            proverAddresses.push(msg.sender);
        }

        emit ProverRegistered(msg.sender, endpoint, features);
    }

    /// @notice Update endpoint and features. Pricing handled by `setPrice`.
    function updateEndpoint(string calldata endpoint, uint64 features, string calldata metadata) external {
        Prover storage p = provers[msg.sender];
        if (!p.active) revert NotRegistered();
        if (bytes(endpoint).length > MAX_ENDPOINT_LENGTH) revert EndpointTooLong();
        if (bytes(metadata).length > MAX_METADATA_LENGTH) revert MetadataTooLong();
        if ((features & FEATURE_PDP) == 0) revert InvalidFeatures();

        p.endpoint = endpoint;
        p.features = features;
        p.metadata = metadata;
        p.updatedAt = uint64(block.timestamp);

        emit ProverUpdated(msg.sender, endpoint, features);
    }

    /// @notice Update pricing.
    function setPrice(uint128 pricePerGibDay, uint128 pricePerByteServed) external {
        Prover storage p = provers[msg.sender];
        if (!p.active) revert NotRegistered();

        p.pricePerGibDay = pricePerGibDay;
        p.pricePerByteServed = pricePerByteServed;
        p.updatedAt = uint64(block.timestamp);

        emit PriceChanged(msg.sender, pricePerGibDay, pricePerByteServed);
    }

    /// @notice Deregister a prover. Soft-delete; the entry remains for historical deals.
    function deregister() external {
        Prover storage p = provers[msg.sender];
        if (!p.active) revert NotRegistered();
        p.active = false;
        p.updatedAt = uint64(block.timestamp);

        emit ProverDeregistered(msg.sender);
    }

    /// @notice Bind an ENS node to a prover for human-readable addressing.
    /// @dev Verification that the caller actually controls the ENS node happens off-chain
    ///      or via an extension contract. This is a pure metadata field.
    function bindENS(bytes32 ensNode) external {
        Prover storage p = provers[msg.sender];
        if (!p.active) revert NotRegistered();
        p.ensNode = ensNode;
        p.updatedAt = uint64(block.timestamp);

        emit ENSBound(msg.sender, ensNode);
    }

    // ───── Views ─────────────────────────────────────────────────────────

    /// @notice Get full prover record.
    function getProver(address prover) external view returns (Prover memory) {
        return provers[prover];
    }

    /// @notice Check if an address is a live prover.
    function isActive(address prover) external view returns (bool) {
        return provers[prover].active;
    }

    /// @notice Check if a prover advertises a specific feature.
    function supportsFeature(address prover, uint64 feature) external view returns (bool) {
        Prover storage p = provers[prover];
        return p.active && (p.features & feature) == feature;
    }

    /// @notice Total number of addresses ever registered (including deregistered).
    function totalRegistered() external view returns (uint256) {
        return proverAddresses.length;
    }

    /// @notice Enumerate active provers (paginated).
    /// @dev O(N) on-chain scan. For N > few thousand, use an off-chain indexer + subgraph.
    /// @param offset Start index into proverAddresses.
    /// @param limit Max number of results.
    function listActive(uint256 offset, uint256 limit)
        external
        view
        returns (address[] memory result, uint256 nextOffset)
    {
        uint256 total = proverAddresses.length;
        if (offset >= total) {
            return (new address[](0), total);
        }

        uint256 end = offset + limit;
        if (end > total) end = total;

        // First pass: count active in range
        uint256 activeCount = 0;
        for (uint256 i = offset; i < end; i++) {
            if (provers[proverAddresses[i]].active) activeCount++;
        }

        // Second pass: fill array
        result = new address[](activeCount);
        uint256 j = 0;
        for (uint256 i = offset; i < end; i++) {
            if (provers[proverAddresses[i]].active) {
                result[j++] = proverAddresses[i];
            }
        }

        return (result, end);
    }
}
