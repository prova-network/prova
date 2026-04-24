// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright (c) 2024-2026 Protocol Labs and contributors (upstream: FilOzone/pdp).
// Copyright (c) 2026 Prova Network contributors.
//
// This file is adapted from FilOzone/pdp src/Fees.sol
// (https://github.com/FilOzone/pdp). Originally under Permissive License Stack
// (Apache-2.0 OR MIT). Attribution preserved per license.
//
// Adaptations for Prova:
//   - Constants and docs use ETH/wei/USDC (Base-native assets) rather
//     than the upstream chain-specific naming. Numeric values are
//     unchanged: both conventions use 18 decimals.
pragma solidity ^0.8.20;

/// @title PDPFees
/// @notice Fee calculations for the Prova ProofVerifier. Denominated in wei.
library PDPFees {
    /// @notice 1 wei, the smallest unit of ETH. Named explicitly for clarity.
    uint256 constant WEI = 1;

    /// @notice 1 ETH in wei. Base and Ethereum both use 18-decimal ETH.
    uint256 constant ETH_TO_WEI = 1e18 * WEI;

    /// @notice Sybil fee charged on createDataSet and new-dataset addPieces.
    ///         0.1 ETH — a modest operational deterrent, not a revenue item.
    uint256 constant SYBIL_FEE = ETH_TO_WEI / 10;

    /// @notice Default proof fee per TiB of challenged data, in wei.
    ///         230_000 gwei = 2.3e14 wei = 0.00023 ETH.
    ///         Picked to approximate ~0.00067 USDC per TiB at an ETH/USDC
    ///         reference of ~2.88 USDC/ETH. Operators tune this per-deployment
    ///         via the governance-adjusted feePerTiB on ProofVerifier.
    uint96 constant DEFAULT_FEE_PER_TIB = 230000 gwei;

    /// @notice 1 TiB in bytes (2^40).
    uint256 constant TIB_IN_BYTES = 2 ** 40;

    /// @notice Calculates the proof fee based on the challenged raw-byte
    ///         size and a per-TiB rate.
    /// @param rawSize The raw size of the proof challenge in bytes.
    /// @param feePerTiB The fee rate per TiB in wei (source of truth lives in ProofVerifier).
    /// @return proof fee in wei.
    /// @dev The proof fee is calculated as: feePerTiB * rawSize / 2^40.
    function calculateProofFee(uint256 rawSize, uint96 feePerTiB) internal pure returns (uint256) {
        require(rawSize > 0, "failed to validate: raw size must be greater than 0");
        return (feePerTiB * rawSize) >> 40;
    }

    /// @notice Sybil fee: adds cost to growing on-chain state on the
    ///         ProofVerifier to prevent wasteful expansion. 0.1 ETH.
    function sybilFee() internal pure returns (uint256) {
        return SYBIL_FEE;
    }
}
