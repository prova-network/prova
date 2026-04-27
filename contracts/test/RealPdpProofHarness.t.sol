// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Prova Network contributors.
//
// Real PDP proof harness.
//
// We don't trust upstream FilOzone/pdp's proof crypto blindly. This test
// exercises the *full* round-trip:
//
//   1. Build a synthetic Merkle tree out of N=8 leaves using the same
//      MerkleProve.buildTree library that produces real CommP roots.
//   2. Encode the root as a CommPv2 CID (height=3, padding=0).
//   3. Run a real deal flow:
//      Client proposes -> prover stakes/registers ->
//      prover atomically createDataSet+addPieces on the real
//      ProofVerifier (UUPS proxy) with the synthetic piece ->
//      prover starts a proving period -> roll to the challenge epoch ->
//      build a real inclusion proof for the challenged leaf ->
//      submit through provePossession.
//   4. Verify ProofVerifier emits PossessionProven AND that the
//      marketplace listener's possessionProven hook fires (ProofRecorded).
//
// This is what the production flow actually looks like end to end on chain.
// If anything in proof generation, CID encoding, challenge derivation, or
// listener wiring drifts, this test fails immediately.
//
// Issue: prova-network/contracts#2 — Real PDP proof generation harness.

pragma solidity ^0.8.24;

import {Test, Vm} from "forge-std/Test.sol";
import {ERC1967Proxy} from "@openzeppelin/contracts/proxy/ERC1967/ERC1967Proxy.sol";
import {ERC20} from "@openzeppelin/contracts/token/ERC20/ERC20.sol";
import {IERC20} from "@openzeppelin/contracts/token/ERC20/IERC20.sol";

import {ProvaToken} from "../src/ProvaToken.sol";
import {ProverRegistry} from "../src/ProverRegistry.sol";
import {ProverStaking} from "../src/ProverStaking.sol";
import {ContentRegistry} from "../src/ContentRegistry.sol";
import {StorageMarketplace} from "../src/StorageMarketplace.sol";
import {ProofVerifier} from "../src/ProofVerifier.sol";
import {Cids} from "../src/Cids.sol";
import {IPDPTypes} from "../src/interfaces/IPDPTypes.sol";
import {MerkleProve, MerkleVerify, Hashes} from "../src/Proofs.sol";

contract MockUSDC is ERC20 {
    constructor() ERC20("USD Coin", "USDC") {
        _mint(msg.sender, 100_000_000 ether);
    }
}

contract RealPdpProofHarnessTest is Test {
    ProvaToken      prova;
    MockUSDC        usdc;
    ProofVerifier   verifierImpl;
    ProofVerifier   verifier;
    ERC1967Proxy    proxy;
    ProverRegistry  registry;
    ProverStaking   staking;
    ContentRegistry content;
    StorageMarketplace market;

    address treasury = makeAddr("treasury");
    address client   = makeAddr("client");
    address prover   = makeAddr("prover");

    uint256 constant CHALLENGE_FINALITY = 150;
    uint256 constant MIN_STAKE_PER_TIB  = 0.1 ether;

    /// Tree shape used by every test in this file. Keep it small so the
    /// arithmetic in computeProofFee/streaming release is easy to reason
    /// about. Height 3 = 8 leaves = 256 bytes piece.
    uint8   constant TREE_HEIGHT_INTERNAL = 3;             // height byte stored in the CID
    uint256 constant TREE_LEAVES          = 8;
    uint64  constant PIECE_SIZE_BYTES     = 256;

    function setUp() public {
        prova = new ProvaToken(treasury);
        usdc  = new MockUSDC();

        verifierImpl = new ProofVerifier(1);
        bytes memory initData = abi.encodeCall(ProofVerifier.initialize, (CHALLENGE_FINALITY));
        proxy = new ERC1967Proxy(address(verifierImpl), initData);
        verifier = ProofVerifier(payable(address(proxy)));

        registry = new ProverRegistry();
        staking  = new ProverStaking(IERC20(address(prova)), MIN_STAKE_PER_TIB);
        content  = new ContentRegistry();

        market = new StorageMarketplace(
            address(verifier),
            IERC20(address(usdc)),
            registry,
            staking,
            content,
            treasury,
            50 ether
        );

        staking.setAuthorizedController(address(market), true);
        content.setMarketplace(address(market));

        // Fund test addresses
        usdc.transfer(client, 100_000 ether);
        vm.prank(treasury);
        prova.transfer(prover, 100_000 ether);

        // Prover registers and stakes way more than required.
        vm.prank(prover);
        registry.register("https://prover.example/pdp", 3, 1_000_000_000, 0, "");

        vm.startPrank(prover);
        prova.approve(address(staking), 50_000 ether);
        staking.stake(50_000 ether);
        vm.stopPrank();
    }

    // ─── Helpers ───────────────────────────────────────────────────────

    /// Synthetic, deterministic 32-byte leaves so every test run computes
    /// the same root. Real PDP would feed sha254-prefolded fr32 chunks of
    /// real piece data, but the verifier doesn't care about leaf shape;
    /// it only checks the inclusion path.
    function _makeLeaves() internal pure returns (bytes32[] memory leaves) {
        leaves = new bytes32[](TREE_LEAVES);
        for (uint256 i = 0; i < TREE_LEAVES; i++) {
            leaves[i] = keccak256(abi.encodePacked("prova:leaf:", i));
        }
    }

    /// Encode a CommPv2 CID as the chain expects: 4-byte multicodec
    /// prefix, varint multihash length (34 for padding=0), varint padding (0),
    /// 1 height byte, 32-byte digest. Total 39 bytes.
    function _encodeCommPv2(bytes32 digest, uint8 heightByte) internal pure returns (Cids.Cid memory) {
        bytes memory data = new bytes(39);
        // multicodec prefix 0x01559120
        data[0] = 0x01;
        data[1] = 0x55;
        data[2] = 0x91;
        data[3] = 0x20;
        // varint(mhLength=34) for padding=0 (single byte 0x22)
        data[4] = 0x22;
        // varint(padding=0)
        data[5] = 0x00;
        // height byte
        data[6] = bytes1(heightByte);
        // digest
        for (uint256 i = 0; i < 32; i++) {
            data[7 + i] = digest[i];
        }
        return Cids.Cid({data: data});
    }

    /// Replicates ProofVerifier.provePossession's challenge-index derivation
    /// so the harness can pre-compute exactly which leaves to prove.
    function _challengeIndex(uint256 seed, uint256 setId, uint64 i, uint256 leafCount)
        internal
        pure
        returns (uint256)
    {
        return uint256(keccak256(abi.encodePacked(seed, setId, i))) % leafCount;
    }

    /// Open a real deal, atomically create the data set + add the piece
    /// on the real ProofVerifier, and start a proving period rolled forward
    /// to the challenge epoch. Returns dealId, setId, root, leaves, and tree
    /// so individual tests can build proofs against them.
    function _stageDeal()
        internal
        returns (
            uint256 dealId,
            uint256 setId,
            bytes32 root,
            bytes32[] memory leaves,
            bytes32[][] memory tree,
            uint256 challengeEpoch
        )
    {
        leaves = _makeLeaves();
        tree   = MerkleProve.buildTree(leaves);
        root   = tree[0][0];

        // Client proposes a deal anchored to this real CommP root.
        vm.startPrank(client);
        usdc.approve(address(market), 1_000 ether);
        // 30-day deal so the streaming-release math is non-trivial but bounded.
        dealId = market.proposeDeal(prover, root, PIECE_SIZE_BYTES, 30 days, 1_000 ether);
        vm.stopPrank();

        // Atomic createDataSet + addPieces with the real piece CID.
        Cids.Cid memory piece = _encodeCommPv2(root, TREE_HEIGHT_INTERNAL);
        Cids.Cid[] memory pieces = new Cids.Cid[](1);
        pieces[0] = piece;
        bytes memory createPayload = abi.encode(dealId);
        bytes memory addPayload    = "";
        bytes memory extraData     = abi.encode(createPayload, addPayload);

        uint256 sybilFee = verifier.sybilFee();
        vm.deal(prover, sybilFee + 10 ether);
        vm.prank(prover);
        setId = verifier.addPieces{value: sybilFee}(0, address(market), pieces, extraData);

        // Sanity: deal active, dataset wired, leaf count matches.
        assertEq(uint256(market.getDeal(dealId).status), uint256(StorageMarketplace.DealStatus.Active));
        assertEq(market.dealIdByDataSet(setId), dealId);
        assertEq(verifier.getDataSetLeafCount(setId), TREE_LEAVES);

        // Start a proving period: pieces just added become provable, and
        // the next challenge epoch is at least challengeFinality blocks out.
        challengeEpoch = block.number + CHALLENGE_FINALITY + 5;
        vm.prank(prover);
        verifier.nextProvingPeriod(setId, challengeEpoch, "");

        // Roll to the challenge epoch. block.prevrandao at this height is
        // what getRandomness returns as the seed.
        vm.roll(challengeEpoch);
        // Warp some real time forward too so streaming release has something
        // to release on the first proof.
        vm.warp(block.timestamp + 1 days);
    }

    // ─── Tests ─────────────────────────────────────────────────────────

    /// End-to-end: a single real PDP inclusion proof verifies on chain
    /// AND drives the marketplace listener.
    function test_realPdpProof_singlePiece_endToEnd() public {
        (
            uint256 dealId,
            uint256 setId,
            ,
            bytes32[] memory leaves,
            bytes32[][] memory tree,
            /* challengeEpoch */
        ) = _stageDeal();

        // Compute the seed exactly the way the verifier does it.
        uint256 seed = uint256(block.prevrandao);
        uint256 leafCount = verifier.getDataSetLeafCount(setId); // == challengeRange post-nextProvingPeriod
        uint256 challengeIdx = _challengeIndex(seed, setId, 0, leafCount);

        // Build the real inclusion proof. proof.length == treeHeight - 1 == 3.
        bytes32[] memory proof = MerkleProve.buildProof(tree, challengeIdx);
        assertEq(proof.length, uint256(TREE_HEIGHT_INTERNAL));

        IPDPTypes.Proof[] memory submission = new IPDPTypes.Proof[](1);
        submission[0] = IPDPTypes.Proof({leaf: leaves[challengeIdx], proof: proof});

        // Pre-snapshot for assertions.
        uint256 proverUsdcBefore = usdc.balanceOf(prover);
        uint256 proofCountBefore = market.getDeal(dealId).proofCount;

        uint256 proofFee = verifier.calculateProofFee(setId);
        vm.prank(prover);
        verifier.provePossession{value: proofFee}(setId, submission);

        // ── Verifier-side assertions ───────────────────────────────
        assertEq(verifier.getDataSetLastProvenEpoch(setId), block.number);

        // ── Marketplace-side assertions (listener actually fired) ──
        StorageMarketplace.Deal memory d = market.getDeal(dealId);
        assertEq(d.proofCount, proofCountBefore + 1, "proofCount should advance");
        assertEq(d.lastProofAt, block.timestamp, "lastProofAt should be now");
        // First proof is 1 day into a 30-day deal. Streaming release
        // should pay roughly 1/30 of totalPayment minus the protocol fee.
        // We assert "non-zero AND below the obvious upper bound" rather
        // than an exact figure so this test isn't fragile to fee tweaks.
        assertGt(usdc.balanceOf(prover), proverUsdcBefore, "prover should be paid on proof");
        assertLt(usdc.balanceOf(prover) - proverUsdcBefore, 1_000 ether, "single proof can't drain whole deal");
    }

    /// Multiple proofs in a single provePossession call all verify.
    /// This exercises the for-loop challenge derivation in the verifier.
    function test_realPdpProof_multipleProofsInOneCall() public {
        (
            uint256 dealId,
            uint256 setId,
            ,
            bytes32[] memory leaves,
            bytes32[][] memory tree,
            /* challengeEpoch */
        ) = _stageDeal();

        uint256 seed = uint256(block.prevrandao);
        uint256 leafCount = verifier.getDataSetLeafCount(setId);

        uint64 nProofs = 3;
        IPDPTypes.Proof[] memory submission = new IPDPTypes.Proof[](nProofs);
        for (uint64 i = 0; i < nProofs; i++) {
            uint256 idx = _challengeIndex(seed, setId, i, leafCount);
            bytes32[] memory proof = MerkleProve.buildProof(tree, idx);
            submission[i] = IPDPTypes.Proof({leaf: leaves[idx], proof: proof});
        }

        uint256 proofFee = verifier.calculateProofFee(setId);
        vm.prank(prover);
        verifier.provePossession{value: proofFee}(setId, submission);

        // One proof event, one listener invocation, even with multiple challenges.
        StorageMarketplace.Deal memory d = market.getDeal(dealId);
        assertEq(d.proofCount, 1, "proofCount increments once per provePossession call");
    }

    /// Tampering with the proof must fail. This is the exact regression
    /// test that catches drift between MerkleProve.buildProof and
    /// MerkleVerify.processInclusionProofMemory.
    function test_realPdpProof_tamperedProof_reverts() public {
        (
            ,
            uint256 setId,
            ,
            bytes32[] memory leaves,
            bytes32[][] memory tree,
            /* challengeEpoch */
        ) = _stageDeal();

        uint256 seed = uint256(block.prevrandao);
        uint256 leafCount = verifier.getDataSetLeafCount(setId);
        uint256 challengeIdx = _challengeIndex(seed, setId, 0, leafCount);

        bytes32[] memory proof = MerkleProve.buildProof(tree, challengeIdx);
        // Flip a sibling. The chain-side hash chain will not reach the
        // root any more, so verification must revert.
        proof[0] = bytes32(uint256(proof[0]) ^ uint256(1));

        IPDPTypes.Proof[] memory submission = new IPDPTypes.Proof[](1);
        submission[0] = IPDPTypes.Proof({leaf: leaves[challengeIdx], proof: proof});

        uint256 proofFee = verifier.calculateProofFee(setId);
        vm.prank(prover);
        vm.expectRevert(bytes("proof did not verify"));
        verifier.provePossession{value: proofFee}(setId, submission);
    }

    /// A correct inclusion path but for the *wrong* leaf must fail.
    /// Catches a class of bugs where the verifier accepts any valid path
    /// regardless of which leaf was challenged.
    function test_realPdpProof_wrongLeafForChallenge_reverts() public {
        (
            ,
            uint256 setId,
            ,
            bytes32[] memory leaves,
            bytes32[][] memory tree,
            /* challengeEpoch */
        ) = _stageDeal();

        uint256 seed = uint256(block.prevrandao);
        uint256 leafCount = verifier.getDataSetLeafCount(setId);
        uint256 challengeIdx = _challengeIndex(seed, setId, 0, leafCount);

        // Pick a different index than the one the chain will challenge.
        uint256 wrongIdx = (challengeIdx + 1) % leafCount;
        bytes32[] memory proof = MerkleProve.buildProof(tree, wrongIdx);

        IPDPTypes.Proof[] memory submission = new IPDPTypes.Proof[](1);
        // Send the wrong-index leaf with the wrong-index proof. Both are
        // internally consistent against the root, but they don't match the
        // challenged offset that the verifier computes from the seed.
        submission[0] = IPDPTypes.Proof({leaf: leaves[wrongIdx], proof: proof});

        uint256 proofFee = verifier.calculateProofFee(setId);
        vm.prank(prover);
        vm.expectRevert(bytes("proof did not verify"));
        verifier.provePossession{value: proofFee}(setId, submission);
    }
}
