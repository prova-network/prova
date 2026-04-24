// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Prova Network contributors.
//
// Mock ProofVerifier for local integration testing. Emits DataSetCreated
// and forwards to the PDPListener exactly as the real ProofVerifier
// would, but without UUPS proxy setup, fee handling, or randomness.
//
// NOT FOR PRODUCTION.

pragma solidity ^0.8.20;

import {Cids} from "./Cids.sol";

interface IMockPDPListener {
    function dataSetCreated(uint256 dataSetId, address creator, bytes calldata extraData) external;
    function possessionProven(uint256 dataSetId, uint256 challengedLeafCount, uint256 seed, uint256 challengeCount) external;
}

/// @title MockProofVerifier
/// @notice Minimal stand-in for the real ProofVerifier. Local tests only.
contract MockProofVerifier {
    event DataSetCreated(uint256 indexed setId, address indexed storageProvider);
    event PossessionProven(uint256 indexed setId);

    uint256 public nextDataSetId = 1;

    mapping(uint256 => address) public listener;
    mapping(uint256 => address) public storageProvider;

    /// @notice Create a data set, mirror ProofVerifier.createDataSet signature.
    function createDataSet(address listenerAddr, bytes calldata extraData)
        external
        payable
        returns (uint256 setId)
    {
        setId = nextDataSetId++;
        listener[setId] = listenerAddr;
        storageProvider[setId] = msg.sender;

        emit DataSetCreated(setId, msg.sender);

        if (listenerAddr != address(0)) {
            IMockPDPListener(listenerAddr).dataSetCreated(setId, msg.sender, extraData);
        }
    }

    /// @notice Simulate a proof event so tests can verify the full flow.
    function simulateProof(uint256 setId) external {
        address l = listener[setId];
        require(l != address(0), "no listener");
        IMockPDPListener(l).possessionProven(setId, 1, 0, 1);
        emit PossessionProven(setId);
    }
}
