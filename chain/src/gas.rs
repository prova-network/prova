// chain/src/gas.rs — CHAIN-012: EIP-1559-style fee market
//
// Dynamic base fee with elastic block gas limits. Transactions specify
// max_fee_per_gas and max_priority_fee_per_gas. Base fee adjusts per block
// targeting 50% utilization. Priority fee goes to block producer; base fee
// is burned (deflationary pressure).

use crate::types::Address;

/// Gas metering constants.
pub const TARGET_GAS_PER_BLOCK: u64 = 15_000_000;
pub const MAX_GAS_PER_BLOCK: u64 = 30_000_000;
pub const ELASTICITY_MULTIPLIER: u64 = 2;
pub const BASE_FEE_CHANGE_DENOMINATOR: u64 = 8;
pub const MIN_BASE_FEE: u128 = 1;
pub const INITIAL_BASE_FEE: u128 = 100;

/// Per-transaction gas costs by operation type.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OpGas {
    Transfer,
    Stake,
    Unstake,
    InferenceCommit,
    Challenge,
    BisectionMove,
    ModelRegistry,
    PaymentOp,
    PdpProof,
    ClaimReward,
    GovernanceVote,
    GovernancePropose,
}

impl OpGas {
    /// Intrinsic gas cost for this operation.
    pub fn intrinsic(&self) -> u64 {
        match self {
            Self::Transfer => 21_000,
            Self::Stake | Self::Unstake => 40_000,
            Self::InferenceCommit => 50_000,
            Self::Challenge => 60_000,
            Self::BisectionMove => 45_000,
            Self::ModelRegistry => 60_000,
            Self::PaymentOp => 25_000,
            Self::PdpProof => 80_000,
            Self::ClaimReward => 30_000,
            Self::GovernanceVote => 35_000,
            Self::GovernancePropose => 70_000,
        }
    }

    /// Per-byte cost for calldata.
    pub fn per_byte_cost() -> u64 {
        16
    }

    /// Per-byte cost for zero bytes (cheaper).
    pub fn per_zero_byte_cost() -> u64 {
        4
    }
}

/// Fee parameters for a transaction (EIP-1559 style).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FeeParams {
    /// Maximum total fee per gas the sender is willing to pay.
    pub max_fee_per_gas: u128,
    /// Maximum priority fee (tip) per gas to the producer.
    pub max_priority_fee_per_gas: u128,
    /// Gas limit for this transaction.
    pub gas_limit: u64,
}

impl FeeParams {
    pub fn new(max_fee: u128, max_priority: u128, gas_limit: u64) -> Self {
        Self {
            max_fee_per_gas: max_fee,
            max_priority_fee_per_gas: max_priority,
            gas_limit,
        }
    }

    /// Effective gas price given current base fee.
    /// Returns None if max_fee < base_fee (tx not viable).
    pub fn effective_gas_price(&self, base_fee: u128) -> Option<u128> {
        if self.max_fee_per_gas < base_fee {
            return None;
        }
        let priority = self
            .max_priority_fee_per_gas
            .min(self.max_fee_per_gas - base_fee);
        Some(base_fee + priority)
    }

    /// Effective priority fee given current base fee.
    pub fn effective_priority_fee(&self, base_fee: u128) -> Option<u128> {
        if self.max_fee_per_gas < base_fee {
            return None;
        }
        Some(
            self.max_priority_fee_per_gas
                .min(self.max_fee_per_gas - base_fee),
        )
    }

    /// Maximum cost this tx can incur.
    pub fn max_cost(&self) -> u128 {
        self.max_fee_per_gas * self.gas_limit as u128
    }
}

/// Fee receipt for an executed transaction.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FeeReceipt {
    pub gas_used: u64,
    pub base_fee: u128,
    pub effective_gas_price: u128,
    pub priority_fee_total: u128,
    pub base_fee_total: u128,
    pub total_fee: u128,
    pub refund: u128,
}

/// Block-level gas tracker and base fee calculator.
#[derive(Debug, Clone)]
pub struct FeeMarket {
    /// Current base fee per gas.
    pub base_fee: u128,
    /// Gas used in current block being built.
    pub block_gas_used: u64,
    /// Historical base fees (last N blocks).
    pub history: Vec<BlockGasInfo>,
    /// Total base fees burned (lifetime).
    pub total_burned: u128,
    /// Total priority fees paid to producers (lifetime).
    pub total_priority: u128,
}

/// Gas info for a finalized block.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BlockGasInfo {
    pub block_number: u64,
    pub gas_used: u64,
    pub gas_limit: u64,
    pub base_fee: u128,
}

impl FeeMarket {
    pub fn new() -> Self {
        Self {
            base_fee: INITIAL_BASE_FEE,
            block_gas_used: 0,
            history: Vec::new(),
            total_burned: 0,
            total_priority: 0,
        }
    }

    pub fn with_base_fee(base_fee: u128) -> Self {
        Self {
            base_fee,
            ..Self::new()
        }
    }

    /// Check if a transaction can fit in the current block.
    pub fn can_fit(&self, gas_limit: u64) -> bool {
        self.block_gas_used + gas_limit <= MAX_GAS_PER_BLOCK
    }

    /// Check if a transaction's fee params are viable at current base fee.
    pub fn is_viable(&self, params: &FeeParams) -> bool {
        params.max_fee_per_gas >= self.base_fee
    }

    /// Charge gas for a transaction. Returns fee receipt or error.
    pub fn charge(
        &mut self,
        params: &FeeParams,
        gas_used: u64,
    ) -> Result<FeeReceipt, &'static str> {
        if gas_used > params.gas_limit {
            return Err("gas used exceeds limit");
        }
        if !self.can_fit(gas_used) {
            return Err("block gas limit exceeded");
        }
        let effective = params
            .effective_gas_price(self.base_fee)
            .ok_or("max fee below base fee")?;
        let priority = params
            .effective_priority_fee(self.base_fee)
            .ok_or("max fee below base fee")?;

        let base_fee_total = self.base_fee * gas_used as u128;
        let priority_fee_total = priority * gas_used as u128;
        let total_fee = effective * gas_used as u128;
        let max_payment = params.max_fee_per_gas * gas_used as u128;
        let refund = max_payment - total_fee;

        self.block_gas_used += gas_used;
        self.total_burned += base_fee_total;
        self.total_priority += priority_fee_total;

        Ok(FeeReceipt {
            gas_used,
            base_fee: self.base_fee,
            effective_gas_price: effective,
            priority_fee_total,
            base_fee_total,
            total_fee,
            refund,
        })
    }

    /// Calculate next base fee from parent block gas usage.
    /// EIP-1559: if gas_used > target, base fee increases; if below, decreases.
    pub fn next_base_fee(parent_gas_used: u64, parent_base_fee: u128) -> u128 {
        if parent_gas_used == TARGET_GAS_PER_BLOCK {
            return parent_base_fee;
        }

        if parent_gas_used > TARGET_GAS_PER_BLOCK {
            let gas_delta = parent_gas_used - TARGET_GAS_PER_BLOCK;
            let fee_delta = (parent_base_fee * gas_delta as u128)
                / (TARGET_GAS_PER_BLOCK as u128 * BASE_FEE_CHANGE_DENOMINATOR as u128);
            let fee_delta = fee_delta.max(1); // always increase by at least 1
            parent_base_fee + fee_delta
        } else {
            let gas_delta = TARGET_GAS_PER_BLOCK - parent_gas_used;
            let fee_delta = (parent_base_fee * gas_delta as u128)
                / (TARGET_GAS_PER_BLOCK as u128 * BASE_FEE_CHANGE_DENOMINATOR as u128);
            (parent_base_fee - fee_delta).max(MIN_BASE_FEE)
        }
    }

    /// Finalize the current block and update base fee for next block.
    pub fn finalize_block(&mut self, block_number: u64) -> BlockGasInfo {
        let info = BlockGasInfo {
            block_number,
            gas_used: self.block_gas_used,
            gas_limit: MAX_GAS_PER_BLOCK,
            base_fee: self.base_fee,
        };

        self.base_fee = Self::next_base_fee(self.block_gas_used, self.base_fee);
        self.block_gas_used = 0;
        self.history.push(info.clone());
        info
    }

    /// Estimate gas cost for a calldata payload.
    pub fn calldata_gas(data: &[u8]) -> u64 {
        data.iter().fold(0u64, |acc, &b| {
            if b == 0 {
                acc + OpGas::per_zero_byte_cost()
            } else {
                acc + OpGas::per_byte_cost()
            }
        })
    }

    /// Total gas for an operation with calldata.
    pub fn total_gas(op: OpGas, data: &[u8]) -> u64 {
        op.intrinsic() + Self::calldata_gas(data)
    }

    /// Get utilization ratio of last finalized block (0.0 to 1.0 as basis points).
    pub fn last_utilization_bps(&self) -> Option<u64> {
        self.history
            .last()
            .map(|info| (info.gas_used as u128 * 10_000 / info.gas_limit as u128) as u64)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_initial_base_fee() {
        let market = FeeMarket::new();
        assert_eq!(market.base_fee, INITIAL_BASE_FEE);
        assert_eq!(market.block_gas_used, 0);
    }

    #[test]
    fn test_effective_gas_price() {
        let params = FeeParams::new(200, 10, 21_000);
        // base_fee = 100, priority = min(10, 200-100) = 10
        assert_eq!(params.effective_gas_price(100), Some(110));
        // base_fee = 195, priority = min(10, 200-195) = 5
        assert_eq!(params.effective_gas_price(195), Some(200));
        // base_fee = 201 > max_fee
        assert_eq!(params.effective_gas_price(201), None);
    }

    #[test]
    fn test_charge_basic() {
        let mut market = FeeMarket::new(); // base_fee = 100
        let params = FeeParams::new(200, 10, 21_000);
        let receipt = market.charge(&params, 21_000).unwrap();
        assert_eq!(receipt.gas_used, 21_000);
        assert_eq!(receipt.base_fee, 100);
        assert_eq!(receipt.effective_gas_price, 110);
        assert_eq!(receipt.base_fee_total, 100 * 21_000);
        assert_eq!(receipt.priority_fee_total, 10 * 21_000);
        assert_eq!(receipt.total_fee, 110 * 21_000);
        assert_eq!(receipt.refund, (200 - 110) * 21_000);
        assert_eq!(market.block_gas_used, 21_000);
    }

    #[test]
    fn test_charge_rejects_low_fee() {
        let mut market = FeeMarket::with_base_fee(500);
        let params = FeeParams::new(100, 10, 21_000);
        assert!(market.charge(&params, 21_000).is_err());
    }

    #[test]
    fn test_charge_rejects_exceeding_block_limit() {
        let mut market = FeeMarket::new();
        let params = FeeParams::new(200, 10, MAX_GAS_PER_BLOCK + 1);
        assert!(market.charge(&params, MAX_GAS_PER_BLOCK + 1).is_err());
    }

    #[test]
    fn test_base_fee_increases_above_target() {
        // If parent used 20M gas (above 15M target), fee should increase
        let next = FeeMarket::next_base_fee(20_000_000, 100);
        assert!(next > 100);
        // delta = 100 * 5M / (15M * 8) = 100 * 5/120 = 4.16 -> 4
        assert_eq!(next, 104);
    }

    #[test]
    fn test_base_fee_decreases_below_target() {
        let next = FeeMarket::next_base_fee(10_000_000, 100);
        assert!(next < 100);
        // delta = 100 * 5M / (15M * 8) = 4
        assert_eq!(next, 96);
    }

    #[test]
    fn test_base_fee_stable_at_target() {
        let next = FeeMarket::next_base_fee(TARGET_GAS_PER_BLOCK, 100);
        assert_eq!(next, 100);
    }

    #[test]
    fn test_base_fee_floor() {
        let next = FeeMarket::next_base_fee(0, 1);
        assert_eq!(next, MIN_BASE_FEE);
    }

    #[test]
    fn test_finalize_block_updates_state() {
        let mut market = FeeMarket::new();
        let params = FeeParams::new(200, 10, 21_000);
        market.charge(&params, 21_000).unwrap();
        let info = market.finalize_block(1);
        assert_eq!(info.gas_used, 21_000);
        assert_eq!(info.block_number, 1);
        assert_eq!(market.block_gas_used, 0);
        // Low usage → base fee should decrease
        assert!(market.base_fee < INITIAL_BASE_FEE);
        assert_eq!(market.history.len(), 1);
    }

    #[test]
    fn test_multi_block_convergence() {
        let mut market = FeeMarket::new();
        // Simulate 20 blocks at exactly target utilization
        for i in 0..20 {
            let params = FeeParams::new(10_000, 10, TARGET_GAS_PER_BLOCK);
            market.charge(&params, TARGET_GAS_PER_BLOCK).unwrap();
            market.finalize_block(i);
        }
        // Base fee should remain stable at initial value
        assert_eq!(market.base_fee, INITIAL_BASE_FEE);
    }

    #[test]
    fn test_calldata_gas() {
        let data = vec![0, 1, 2, 0, 0, 255];
        // 3 zero bytes × 4 + 3 non-zero × 16 = 12 + 48 = 60
        assert_eq!(FeeMarket::calldata_gas(&data), 60);
    }

    #[test]
    fn test_total_gas_with_calldata() {
        let data = vec![1; 100]; // 100 non-zero bytes
        let total = FeeMarket::total_gas(OpGas::Transfer, &data);
        assert_eq!(total, 21_000 + 100 * 16);
    }

    #[test]
    fn test_op_gas_intrinsic_values() {
        assert_eq!(OpGas::Transfer.intrinsic(), 21_000);
        assert_eq!(OpGas::PdpProof.intrinsic(), 80_000);
        assert_eq!(OpGas::GovernancePropose.intrinsic(), 70_000);
    }

    #[test]
    fn test_fee_params_max_cost() {
        let params = FeeParams::new(200, 10, 21_000);
        assert_eq!(params.max_cost(), 200 * 21_000);
    }

    #[test]
    fn test_utilization_tracking() {
        let mut market = FeeMarket::new();
        let params = FeeParams::new(200, 10, MAX_GAS_PER_BLOCK / 2);
        market.charge(&params, MAX_GAS_PER_BLOCK / 2).unwrap();
        market.finalize_block(0);
        let util = market.last_utilization_bps().unwrap();
        assert_eq!(util, 5000); // 50%
    }

    #[test]
    fn test_surge_pricing() {
        // Simulate sustained high demand → base fee escalation
        let mut market = FeeMarket::new();
        let initial = market.base_fee;
        for i in 0..10 {
            // Fill blocks to 90% capacity
            let gas = (MAX_GAS_PER_BLOCK as f64 * 0.9) as u64;
            let params = FeeParams::new(100_000, 50, gas);
            market.charge(&params, gas).unwrap();
            market.finalize_block(i);
        }
        // Base fee should have increased significantly
        assert!(market.base_fee > initial * 2);
    }

    #[test]
    fn test_fee_market_burned_and_priority_totals() {
        let mut market = FeeMarket::new();
        let params = FeeParams::new(200, 10, 50_000);
        let r1 = market.charge(&params, 50_000).unwrap();
        let params2 = FeeParams::new(300, 20, 30_000);
        let r2 = market.charge(&params2, 30_000).unwrap();
        assert_eq!(market.total_burned, r1.base_fee_total + r2.base_fee_total);
        assert_eq!(
            market.total_priority,
            r1.priority_fee_total + r2.priority_fee_total
        );
    }

    #[test]
    fn test_can_fit() {
        let mut market = FeeMarket::new();
        assert!(market.can_fit(MAX_GAS_PER_BLOCK));
        market.block_gas_used = MAX_GAS_PER_BLOCK - 100;
        assert!(!market.can_fit(200));
        assert!(market.can_fit(100));
    }
}
