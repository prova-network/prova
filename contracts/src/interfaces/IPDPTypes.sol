// SPDX-License-Identifier: MIT
// Copyright (c) 2024-2026 Protocol Labs and contributors (upstream: FilOzone/pdp).
// Copyright (c) 2026 Prova Network contributors.
//
// This file is imported unchanged from FilOzone/pdp src/interfaces/IPDPTypes.sol.
pragma solidity ^0.8.20;

/// @title IPDPTypes
/// @notice Shared types for PDP contracts and consumers
interface IPDPTypes {
    struct Proof {
        bytes32 leaf;
        bytes32[] proof;
    }

    struct PieceIdAndOffset {
        uint256 pieceId;
        uint256 offset;
    }
}
