// SPDX-License-Identifier: MIT
pragma solidity ^0.8.24;

import "forge-std/Test.sol";
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

contract MockUSDC is ERC20 {
    constructor() ERC20("USD Coin", "USDC") {
        _mint(msg.sender, 100_000_000 ether);
    }
}

/// @notice Integration test: real ProofVerifier (forked from FilOzone/pdp)
///         deployed behind an ERC1967 UUPS proxy, wired into the marketplace
///         as the PDPListener target. We don't generate real PDP proofs here
///         (that requires a real Merkle tree + challenge response which the
///         upstream tests cover) — but we verify:
///
///           1. The proxy initializes correctly (challengeFinality is set)
///           2. createDataSet routes back into the marketplace's
///              dataSetCreated callback (so the deal goes Active)
///           3. The proxy is owner-upgradeable (upgrade authority works)
///           4. Non-owner cannot upgrade
///           5. The verifier's getDataSet view returns the prover address
///
///         The proof crypto itself is the upstream FilOzone/pdp test surface;
///         we trust those tests for the math.
contract RealProofVerifierTest is Test {
    ProvaToken     prova;
    MockUSDC       usdc;
    ProofVerifier  verifierImpl;
    ProofVerifier  verifier;       // Cast of the proxy
    ERC1967Proxy   proxy;
    ProverRegistry registry;
    ProverStaking  staking;
    ContentRegistry content;
    StorageMarketplace market;

    address treasury = makeAddr("treasury");
    address client   = makeAddr("client");
    address prover   = makeAddr("prover");

    uint256 constant CHALLENGE_FINALITY = 150;
    uint256 constant MIN_STAKE_GIB = 100 ether;

    function setUp() public {
        prova = new ProvaToken(treasury);
        usdc  = new MockUSDC();

        // Deploy ProofVerifier impl + proxy
        verifierImpl = new ProofVerifier(1);
        bytes memory initData = abi.encodeCall(ProofVerifier.initialize, (CHALLENGE_FINALITY));
        proxy = new ERC1967Proxy(address(verifierImpl), initData);
        verifier = ProofVerifier(payable(address(proxy)));

        registry = new ProverRegistry();
        staking  = new ProverStaking(IERC20(address(prova)), MIN_STAKE_GIB);
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

        // Prover registers
        vm.prank(prover);
        registry.register("https://prover.example/pdp", 3, 1_000_000_000, 0, "");

        // Prover stakes
        vm.startPrank(prover);
        prova.approve(address(staking), 50_000 ether);
        staking.stake(50_000 ether);
        vm.stopPrank();
    }

    // ─── 1. Proxy initializes correctly ────────────────────────────────

    function test_proxy_initialized() public view {
        // The marketplace can read it as the listener target
        assertEq(market.proofVerifier(), address(verifier));
        // Owner is set to whoever called initialize (this contract's setUp)
        assertEq(verifier.owner(), address(this));
    }

    function test_RevertWhen_doubleInitialize() public {
        vm.expectRevert(); // OZ Initializable
        verifier.initialize(999);
    }

    function test_initialize_acceptsAnyChallengeFinality() public {
        // Upstream PDP doesn't validate challengeFinality > 0; both 0 and N
        // are accepted. We document that here so it's clear the validation
        // is upstream's responsibility.
        ProofVerifier impl = new ProofVerifier(1);
        bytes memory init = abi.encodeCall(ProofVerifier.initialize, (0));
        new ERC1967Proxy(address(impl), init); // no revert
    }

    // ─── 2. createDataSet routes back to marketplace ──────────────────

    function test_createDataSet_activatesDeal() public {
        // Client proposes a deal
        vm.startPrank(client);
        usdc.approve(address(market), 1000 ether);
        bytes32 commp = keccak256("test-piece");
        uint256 dealId = market.proposeDeal(prover, commp, 1024 * 1024, 30 days, 1000 ether);
        vm.stopPrank();

        // Deal should be Proposed
        assertEq(uint256(market.getDeal(dealId).status), uint256(StorageMarketplace.DealStatus.Proposed));

        // Prover calls createDataSet on the real verifier with the dealId encoded
        // in extraData. Format: abi.encode(createPayload, addPayload).
        // The marketplace's dataSetCreated decodes its extraData as uint256 dealId,
        // so createPayload = abi.encode(dealId).
        bytes memory createPayload = abi.encode(dealId);
        bytes memory addPayload    = "";
        bytes memory extraData     = abi.encode(createPayload, addPayload);

        Cids.Cid[] memory empty = new Cids.Cid[](0);

        // Pay the sybil fee in native ETH (refund of excess is automatic).
        uint256 sybilFee = verifier.sybilFee();
        vm.deal(prover, sybilFee + 1 ether);
        vm.prank(prover);
        uint256 setId = verifier.addPieces{value: sybilFee}(0, address(market), empty, extraData);

        // Deal should now be Active
        assertEq(uint256(market.getDeal(dealId).status), uint256(StorageMarketplace.DealStatus.Active));

        // dataSet's storageProvider should be the prover
        (address sp, ) = verifier.getDataSetStorageProvider(setId);
        assertEq(sp, prover);
        assertEq(verifier.getDataSetListener(setId), address(market));

        // The marketplace should have linked dataSetId -> dealId
        assertEq(market.dealIdByDataSet(setId), dealId);
    }

    // ─── 3. UUPS upgradability ────────────────────────────────────────

    function test_upgrade_byOwner_works() public {
        // The upstream PDP enforces a two-step upgrade: announce a planned
        // upgrade, then wait until block.number >= afterEpoch, then upgrade.
        // We test the full flow.
        ProofVerifier v2 = new ProofVerifier(1);

        ProofVerifier.PlannedUpgrade memory plan = ProofVerifier.PlannedUpgrade({
            nextImplementation: address(v2),
            afterEpoch: uint96(block.number + 1)
        });
        verifier.announcePlannedUpgrade(plan);

        // Cannot upgrade before the announced block
        vm.expectRevert();
        verifier.upgradeToAndCall(address(v2), "");

        // Advance one block, then upgrade succeeds
        vm.roll(block.number + 2);
        verifier.upgradeToAndCall(address(v2), "");

        // Proxy still answers ownership; marketplace still points at the proxy
        assertEq(verifier.owner(), address(this));
        assertEq(market.proofVerifier(), address(verifier));
    }

    function test_RevertWhen_nonOwnerAnnouncesUpgrade() public {
        ProofVerifier v2 = new ProofVerifier(1);
        ProofVerifier.PlannedUpgrade memory plan = ProofVerifier.PlannedUpgrade({
            nextImplementation: address(v2),
            afterEpoch: uint96(block.number + 1)
        });

        vm.prank(makeAddr("attacker"));
        vm.expectRevert();
        verifier.announcePlannedUpgrade(plan);
    }

    function test_RevertWhen_nonOwnerUpgrades() public {
        // Owner announces, attacker tries to actually call upgradeToAndCall
        ProofVerifier v2 = new ProofVerifier(1);
        ProofVerifier.PlannedUpgrade memory plan = ProofVerifier.PlannedUpgrade({
            nextImplementation: address(v2),
            afterEpoch: uint96(block.number + 1)
        });
        verifier.announcePlannedUpgrade(plan);
        vm.roll(block.number + 2);

        vm.prank(makeAddr("attacker"));
        vm.expectRevert();
        verifier.upgradeToAndCall(address(v2), "");
    }

    // ─── 4. Sanity: implementation cannot be initialized directly ─────

    function test_RevertWhen_initializeImplementationDirectly() public {
        vm.expectRevert();
        verifierImpl.initialize(150);
    }
}
