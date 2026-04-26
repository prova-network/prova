// SPDX-License-Identifier: MIT
pragma solidity ^0.8.24;

import "forge-std/Test.sol";
import {IERC20} from "@openzeppelin/contracts/token/ERC20/IERC20.sol";
import {ProvaToken} from "../src/ProvaToken.sol";
import {ProverStaking} from "../src/ProverStaking.sol";
import {MockPriceOracle} from "../src/MockPriceOracle.sol";
import {IPriceOracle} from "../src/interfaces/IPriceOracle.sol";

/// @notice Verifies the v2 USD-equivalent stake floor with Chainlink-style oracle.
contract StakeFloorOracleTest is Test {
    ProvaToken     prova;
    ProverStaking  staking;
    MockPriceOracle oracle;

    address treasury = makeAddr("treasury");
    address controller = makeAddr("controller"); // simulates the marketplace
    address prover = makeAddr("prover");

    uint256 constant TIB = uint256(1024) * 1024 * 1024 * 1024;
    uint256 constant MIN_PROVA_PER_TIB = 0.1 ether;       // PROVA-only floor
    uint256 constant MIN_USD_PER_TIB   = 300_000_000;     // $3.00 per TiB (8-dec)
    int256  constant PROVA_AT_10_CENTS = 10_000_000;      // $0.10 with 8 decimals
    int256  constant PROVA_AT_1_USD    = 100_000_000;     // $1.00
    int256  constant PROVA_AT_10_DOLLARS = 1_000_000_000; // $10.00

    function setUp() public {
        prova = new ProvaToken(treasury);
        staking = new ProverStaking(IERC20(address(prova)), MIN_PROVA_PER_TIB);
        oracle = new MockPriceOracle(PROVA_AT_10_CENTS);

        // Authorize controller as a marketplace-equivalent
        staking.setAuthorizedController(controller, true);

        // Set USD floor + oracle
        staking.setMinStakeUsdPerTiB(MIN_USD_PER_TIB);
        staking.setPriceOracle(IPriceOracle(address(oracle)));

        // Fund prover with PROVA so they can stake
        vm.prank(treasury);
        prova.transfer(prover, 10_000_000 ether);
    }

    // ─── Constants sanity ─────────────────────────────────────────────

    function test_constants() public view {
        assertEq(staking.minStakePerTiB(), MIN_PROVA_PER_TIB);
        assertEq(staking.minStakeUsdPerTiB(), MIN_USD_PER_TIB);
        assertEq(address(staking.priceOracle()), address(oracle));
        assertEq(staking.oracleStalenessSeconds(), 1 hours);
        assertEq(staking.minStakePerGib(), 0); // legacy field is 0
    }

    function test_RevertWhen_setMinStakePerGibCalled() public {
        vm.expectRevert();
        staking.setMinStakePerGib(123);
    }

    // ─── PROVA-only floor (oracle disabled) ──────────────────────────

    function test_provaOnlyFloor_noOracle() public {
        staking.setPriceOracle(IPriceOracle(address(0)));

        // 1 TiB of committed bytes -> exactly 0.1 PROVA required
        uint256 req = staking.minStakeFor(TIB);
        assertEq(req, MIN_PROVA_PER_TIB);

        // 100 TiB -> 10 PROVA
        assertEq(staking.minStakeFor(100 * TIB), 100 * MIN_PROVA_PER_TIB);
    }

    // ─── USD floor binds when PROVA price is low ─────────────────────

    function test_usdFloor_bindsAtLowPRICE() public view {
        // Default oracle price: $0.10 PROVA. USD floor: $3 / TiB.
        // Required stake at 1 TiB:
        //   PROVA floor: 0.1 PROVA = 0.1e18
        //   USD floor:   $3 / $0.10 = 30 PROVA = 30e18
        // 30e18 > 0.1e18 -> USD floor binds.
        uint256 req = staking.minStakeFor(TIB);
        assertEq(req, 30 ether, "USD floor at $0.10 PROVA should require 30 PROVA per TiB");

        // 100 TiB -> 3000 PROVA
        assertEq(staking.minStakeFor(100 * TIB), 3000 ether);

        // 1 PiB (1024 TiB) -> 30,720 PROVA (= 0.03% of 100M supply)
        assertEq(staking.minStakeFor(1024 * TIB), 30720 ether);
    }

    // ─── PROVA floor binds when PROVA price is very high ─────────────

    function test_provaFloor_bindsAtHighPrice() public {
        // At PROVA = $1000:
        oracle.setPrice(int256(100_000_000_000)); // $1000 with 8 decimals

        // USD floor at 1 TiB: $3 / $1000 = 0.003 PROVA = 3e15 wei
        // PROVA floor:       0.1 PROVA = 1e17 wei
        // 0.1e18 > 0.003e18 -> PROVA floor binds.
        uint256 req = staking.minStakeFor(TIB);
        assertEq(req, MIN_PROVA_PER_TIB, "PROVA floor should bind when oracle price is very high");
    }

    // ─── USD floor scales linearly with capacity ─────────────────────

    function test_usdFloor_scalesLinearly() public view {
        uint256 base = staking.minStakeFor(TIB);
        assertEq(staking.minStakeFor(2 * TIB),  2 * base);
        assertEq(staking.minStakeFor(10 * TIB), 10 * base);
        assertEq(staking.minStakeFor(100 * TIB), 100 * base);
    }

    // ─── Stale oracle falls back to PROVA-only floor ─────────────────

    function test_staleOracle_fallsBackToProvaFloor() public {
        // Skip past staleness window (default 1 hour)
        skip(1 hours + 60);

        // Even though USD floor is set, stale oracle means PROVA-only floor binds.
        uint256 req = staking.minStakeFor(TIB);
        assertEq(req, MIN_PROVA_PER_TIB, "stale oracle should fall back to PROVA floor");
    }

    function test_freshOracleSurvives_oneRoundUpdate() public {
        skip(30 minutes); // still within 1 hour staleness
        uint256 req = staking.minStakeFor(TIB);
        assertEq(req, 30 ether, "30 min into staleness window: USD floor still binds");
    }

    // ─── Negative oracle price falls back to PROVA-floor ─────────────

    function test_negativeOraclePrice_fallsBack() public {
        oracle.setPrice(-1);
        uint256 req = staking.minStakeFor(TIB);
        assertEq(req, MIN_PROVA_PER_TIB, "negative oracle answer should fall back");
    }

    function test_zeroOraclePrice_fallsBack() public {
        oracle.setPrice(0);
        uint256 req = staking.minStakeFor(TIB);
        assertEq(req, MIN_PROVA_PER_TIB, "zero oracle answer should fall back");
    }

    // ─── canCommit honors the binding floor ──────────────────────────

    function test_canCommit_honorsUsdFloor() public {
        // Prover stakes 100 PROVA. At $0.10 PROVA the USD floor is 30 PROVA / TiB.
        // So they can commit ~3 TiB before they hit the floor.
        vm.startPrank(prover);
        prova.approve(address(staking), 100 ether);
        staking.stake(100 ether);
        vm.stopPrank();

        // 1 TiB: should be OK (req = 30 PROVA, have 100)
        assertTrue(staking.canCommit(prover, TIB));

        // 3 TiB: should be OK (req = 90 PROVA, have 100)
        assertTrue(staking.canCommit(prover, 3 * TIB));

        // 4 TiB: should fail (req = 120 PROVA, have 100)
        assertFalse(staking.canCommit(prover, 4 * TIB));
    }

    function test_canCommit_relaxesWhenPriceRises() public {
        vm.startPrank(prover);
        prova.approve(address(staking), 10 ether);
        staking.stake(10 ether);
        vm.stopPrank();

        // 1 TiB at $0.10: req = 30 PROVA, have 10 → fail
        assertFalse(staking.canCommit(prover, TIB));

        // PROVA appreciates 10x to $1.00. Now 1 TiB req = 3 PROVA → OK with 10 staked.
        oracle.setPrice(PROVA_AT_1_USD);
        assertTrue(staking.canCommit(prover, TIB));

        // 4 TiB at $1.00: req = 12 PROVA, have 10 → fail (PROVA floor: 0.4 PROVA, USD floor: 12 PROVA, USD wins)
        assertFalse(staking.canCommit(prover, 4 * TIB));
    }

    // ─── Setters are owner-only ──────────────────────────────────────

    function test_RevertWhen_nonOwnerSetsFloor() public {
        vm.prank(makeAddr("attacker"));
        vm.expectRevert();
        staking.setMinStakePerTiB(1);
    }

    function test_RevertWhen_nonOwnerSetsUsdFloor() public {
        vm.prank(makeAddr("attacker"));
        vm.expectRevert();
        staking.setMinStakeUsdPerTiB(1);
    }

    function test_RevertWhen_nonOwnerSetsOracle() public {
        vm.prank(makeAddr("attacker"));
        vm.expectRevert();
        staking.setPriceOracle(IPriceOracle(address(0)));
    }

    function test_setOracleStaleness_validBounds() public {
        staking.setOracleStalenessSeconds(60);
        assertEq(staking.oracleStalenessSeconds(), 60);
        staking.setOracleStalenessSeconds(1 days);
        assertEq(staking.oracleStalenessSeconds(), 1 days);
    }

    function test_RevertWhen_oracleStalenessOutOfRange() public {
        vm.expectRevert(bytes("staleness out of range"));
        staking.setOracleStalenessSeconds(59);

        vm.expectRevert(bytes("staleness out of range"));
        staking.setOracleStalenessSeconds(1 days + 1);
    }
}
