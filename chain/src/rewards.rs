//! Reward Distribution — block rewards and inference fee distribution.
//!
//! Prova's reward model:
//! 1. **Block rewards**: Fixed per-epoch issuance to the block producer, decaying over time.
//!    - Base reward: 100 tokens/epoch, halving every 2,102,400 epochs (~2 years at 30s epochs).
//!    - Producer gets 80%, 20% goes to the network treasury.
//! 2. **Inference fees**: Paid by clients, split between provider and protocol.
//!    - Provider gets 90%, protocol treasury gets 10%.
//! 3. **Challenger bounties**: When a dispute is won, challenger receives a fraction of the
//!    provider's slashed stake (handled in stake.rs), plus a bounty from the treasury.
//! 4. **Storage rewards**: Providers who maintain PDP proof sets earn a per-epoch storage subsidy
//!    proportional to their proven data size relative to total network storage.

use crate::types::*;
use std::collections::HashMap;

/// Halving interval in epochs (~2 years at 30s epochs).
const HALVING_INTERVAL: Epoch = 2_102_400;

/// Base block reward per epoch (in smallest token unit).
const BASE_BLOCK_REWARD: u128 = 100_000_000_000; // 100 tokens (9 decimal places)

/// Producer share of block reward (basis points).
const PRODUCER_SHARE_BPS: u128 = 8_000; // 80%

/// Provider share of inference fees (basis points).
const PROVIDER_FEE_SHARE_BPS: u128 = 9_000; // 90%

/// Total basis points.
const BPS_TOTAL: u128 = 10_000;

/// Storage subsidy pool per epoch (in smallest token unit).
const STORAGE_SUBSIDY_PER_EPOCH: u128 = 50_000_000_000; // 50 tokens

/// Minimum stake to be eligible for storage rewards.
const MIN_STAKE_FOR_STORAGE: u128 = 1_000_000_000; // 1 token

/// Accumulated reward state for the network.
#[derive(Debug, Clone)]
pub struct RewardLedger {
    /// Treasury balance (accumulates protocol share).
    pub treasury: u128,
    /// Per-address accumulated rewards (unclaimed).
    pub pending: HashMap<Address, u128>,
    /// Per-address total claimed rewards (lifetime).
    pub claimed: HashMap<Address, u128>,
    /// Total tokens minted as block rewards.
    pub total_minted: u128,
    /// Total inference fees collected.
    pub total_inference_fees: u128,
    /// Total storage subsidies distributed.
    pub total_storage_subsidies: u128,
    /// Current epoch (for halving calculation).
    pub current_epoch: Epoch,
}

impl RewardLedger {
    pub fn new() -> Self {
        Self {
            treasury: 0,
            pending: HashMap::new(),
            claimed: HashMap::new(),
            total_minted: 0,
            total_inference_fees: 0,
            total_storage_subsidies: 0,
            current_epoch: 0,
        }
    }

    /// Calculate block reward for a given epoch (with halving).
    pub fn block_reward_at(epoch: Epoch) -> u128 {
        let halvings = epoch / HALVING_INTERVAL;
        if halvings >= 64 {
            return 0; // Effectively zero after 64 halvings
        }
        BASE_BLOCK_REWARD >> halvings
    }

    /// Distribute block reward to a producer for the given epoch.
    pub fn distribute_block_reward(
        &mut self,
        producer: Address,
        epoch: Epoch,
    ) -> BlockRewardResult {
        let total = Self::block_reward_at(epoch);
        if total == 0 {
            return BlockRewardResult {
                total: 0,
                to_producer: 0,
                to_treasury: 0,
            };
        }

        let to_producer = total * PRODUCER_SHARE_BPS / BPS_TOTAL;
        let to_treasury = total - to_producer;

        *self.pending.entry(producer).or_insert(0) += to_producer;
        self.treasury += to_treasury;
        self.total_minted += total;
        self.current_epoch = epoch;

        BlockRewardResult {
            total,
            to_producer,
            to_treasury,
        }
    }

    /// Distribute an inference fee between provider and protocol treasury.
    pub fn distribute_inference_fee(&mut self, provider: Address, fee: u128) -> InferenceFeeResult {
        if fee == 0 {
            return InferenceFeeResult {
                to_provider: 0,
                to_treasury: 0,
            };
        }

        let to_provider = fee * PROVIDER_FEE_SHARE_BPS / BPS_TOTAL;
        let to_treasury = fee - to_provider;

        *self.pending.entry(provider).or_insert(0) += to_provider;
        self.treasury += to_treasury;
        self.total_inference_fees += fee;

        InferenceFeeResult {
            to_provider,
            to_treasury,
        }
    }

    /// Distribute storage subsidies proportional to proven data.
    /// `providers` maps address → bytes of proven data.
    /// Returns per-provider reward amounts.
    pub fn distribute_storage_subsidies(
        &mut self,
        providers: &[(Address, u64)],
        staked: &HashMap<Address, u128>,
    ) -> Vec<(Address, u128)> {
        // Filter to eligible providers (must have minimum stake)
        let eligible: Vec<(Address, u64)> = providers
            .iter()
            .filter(|(addr, _)| staked.get(addr).copied().unwrap_or(0) >= MIN_STAKE_FOR_STORAGE)
            .copied()
            .collect();

        let total_data: u64 = eligible.iter().map(|(_, sz)| sz).sum();
        if total_data == 0 {
            return vec![];
        }

        let mut results = Vec::new();
        let mut distributed: u128 = 0;

        for (i, (addr, data_size)) in eligible.iter().enumerate() {
            let share = if i == eligible.len() - 1 {
                // Last provider gets remainder to avoid rounding loss
                STORAGE_SUBSIDY_PER_EPOCH - distributed
            } else {
                STORAGE_SUBSIDY_PER_EPOCH * (*data_size as u128) / (total_data as u128)
            };

            *self.pending.entry(*addr).or_insert(0) += share;
            distributed += share;
            results.push((*addr, share));
        }

        self.total_storage_subsidies += distributed;
        results
    }

    /// Claim all pending rewards for an address.
    pub fn claim(&mut self, addr: &Address) -> u128 {
        let amount = self.pending.remove(addr).unwrap_or(0);
        if amount > 0 {
            *self.claimed.entry(*addr).or_insert(0) += amount;
        }
        amount
    }

    /// Get pending rewards for an address.
    pub fn pending_for(&self, addr: &Address) -> u128 {
        self.pending.get(addr).copied().unwrap_or(0)
    }

    /// Get total claimed rewards for an address.
    pub fn claimed_for(&self, addr: &Address) -> u128 {
        self.claimed.get(addr).copied().unwrap_or(0)
    }

    /// Fund a challenger bounty from the treasury. Returns actual amount paid
    /// (may be less than requested if treasury is insufficient).
    pub fn pay_challenger_bounty(&mut self, challenger: Address, requested: u128) -> u128 {
        let actual = requested.min(self.treasury);
        if actual > 0 {
            self.treasury -= actual;
            *self.pending.entry(challenger).or_insert(0) += actual;
        }
        actual
    }
}

/// Result of block reward distribution.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BlockRewardResult {
    pub total: u128,
    pub to_producer: u128,
    pub to_treasury: u128,
}

/// Result of inference fee distribution.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InferenceFeeResult {
    pub to_provider: u128,
    pub to_treasury: u128,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn addr(id: u8) -> Address {
        Address::test(id)
    }

    #[test]
    fn test_block_reward_at_epoch_zero() {
        assert_eq!(RewardLedger::block_reward_at(0), BASE_BLOCK_REWARD);
    }

    #[test]
    fn test_block_reward_halving() {
        let r0 = RewardLedger::block_reward_at(0);
        let r1 = RewardLedger::block_reward_at(HALVING_INTERVAL);
        let r2 = RewardLedger::block_reward_at(HALVING_INTERVAL * 2);
        assert_eq!(r1, r0 / 2);
        assert_eq!(r2, r0 / 4);
    }

    #[test]
    fn test_block_reward_zero_after_many_halvings() {
        assert_eq!(RewardLedger::block_reward_at(HALVING_INTERVAL * 64), 0);
    }

    #[test]
    fn test_distribute_block_reward_split() {
        let mut ledger = RewardLedger::new();
        let result = ledger.distribute_block_reward(addr(1), 0);

        assert_eq!(result.total, BASE_BLOCK_REWARD);
        assert_eq!(result.to_producer, BASE_BLOCK_REWARD * 80 / 100);
        assert_eq!(result.to_treasury, BASE_BLOCK_REWARD * 20 / 100);
        assert_eq!(result.to_producer + result.to_treasury, result.total);

        assert_eq!(ledger.pending_for(&addr(1)), result.to_producer);
        assert_eq!(ledger.treasury, result.to_treasury);
        assert_eq!(ledger.total_minted, BASE_BLOCK_REWARD);
    }

    #[test]
    fn test_inference_fee_distribution() {
        let mut ledger = RewardLedger::new();
        let fee = 1_000_000_000; // 1 token
        let result = ledger.distribute_inference_fee(addr(2), fee);

        assert_eq!(result.to_provider, fee * 90 / 100);
        assert_eq!(result.to_treasury, fee * 10 / 100);
        assert_eq!(result.to_provider + result.to_treasury, fee);
        assert_eq!(ledger.total_inference_fees, fee);
    }

    #[test]
    fn test_inference_fee_zero() {
        let mut ledger = RewardLedger::new();
        let result = ledger.distribute_inference_fee(addr(1), 0);
        assert_eq!(result.to_provider, 0);
        assert_eq!(result.to_treasury, 0);
    }

    #[test]
    fn test_storage_subsidies_proportional() {
        let mut ledger = RewardLedger::new();
        let providers = vec![
            (addr(1), 3_000_000_000u64), // 3 GB
            (addr(2), 7_000_000_000u64), // 7 GB
        ];
        let staked: HashMap<Address, u128> = [
            (addr(1), MIN_STAKE_FOR_STORAGE),
            (addr(2), MIN_STAKE_FOR_STORAGE * 2),
        ]
        .into();

        let rewards = ledger.distribute_storage_subsidies(&providers, &staked);
        assert_eq!(rewards.len(), 2);

        let total_distributed: u128 = rewards.iter().map(|(_, r)| r).sum();
        assert_eq!(total_distributed, STORAGE_SUBSIDY_PER_EPOCH);

        // 30% and 70% (approximately)
        assert_eq!(rewards[0].1, STORAGE_SUBSIDY_PER_EPOCH * 3 / 10);
        assert_eq!(
            rewards[1].1,
            STORAGE_SUBSIDY_PER_EPOCH - STORAGE_SUBSIDY_PER_EPOCH * 3 / 10
        );
    }

    #[test]
    fn test_storage_subsidies_filters_unstaked() {
        let mut ledger = RewardLedger::new();
        let providers = vec![
            (addr(1), 5_000_000_000u64),
            (addr(2), 5_000_000_000u64), // Not staked
        ];
        let staked: HashMap<Address, u128> = [(addr(1), MIN_STAKE_FOR_STORAGE)].into();

        let rewards = ledger.distribute_storage_subsidies(&providers, &staked);
        assert_eq!(rewards.len(), 1);
        assert_eq!(rewards[0].0, addr(1));
        assert_eq!(rewards[0].1, STORAGE_SUBSIDY_PER_EPOCH);
    }

    #[test]
    fn test_storage_subsidies_empty_providers() {
        let mut ledger = RewardLedger::new();
        let rewards = ledger.distribute_storage_subsidies(&[], &HashMap::new());
        assert!(rewards.is_empty());
    }

    #[test]
    fn test_claim_rewards() {
        let mut ledger = RewardLedger::new();
        ledger.distribute_block_reward(addr(1), 0);
        let pending = ledger.pending_for(&addr(1));
        assert!(pending > 0);

        let claimed = ledger.claim(&addr(1));
        assert_eq!(claimed, pending);
        assert_eq!(ledger.pending_for(&addr(1)), 0);
        assert_eq!(ledger.claimed_for(&addr(1)), claimed);
    }

    #[test]
    fn test_claim_nothing() {
        let mut ledger = RewardLedger::new();
        assert_eq!(ledger.claim(&addr(99)), 0);
    }

    #[test]
    fn test_challenger_bounty_from_treasury() {
        let mut ledger = RewardLedger::new();
        // Seed treasury via block reward
        ledger.distribute_block_reward(addr(1), 0);
        let treasury_before = ledger.treasury;
        assert!(treasury_before > 0);

        let bounty = 5_000_000_000u128;
        let paid = ledger.pay_challenger_bounty(addr(5), bounty);
        assert_eq!(paid, bounty);
        assert_eq!(ledger.treasury, treasury_before - bounty);
        assert_eq!(ledger.pending_for(&addr(5)), bounty);
    }

    #[test]
    fn test_challenger_bounty_capped_by_treasury() {
        let mut ledger = RewardLedger::new();
        ledger.treasury = 100;
        let paid = ledger.pay_challenger_bounty(addr(5), 500);
        assert_eq!(paid, 100);
        assert_eq!(ledger.treasury, 0);
    }

    #[test]
    fn test_multiple_epochs_accumulate() {
        let mut ledger = RewardLedger::new();
        ledger.distribute_block_reward(addr(1), 0);
        ledger.distribute_block_reward(addr(1), 1);
        ledger.distribute_block_reward(addr(1), 2);

        let expected = BASE_BLOCK_REWARD * 80 / 100 * 3;
        assert_eq!(ledger.pending_for(&addr(1)), expected);
        assert_eq!(ledger.total_minted, BASE_BLOCK_REWARD * 3);
    }

    #[test]
    fn test_combined_reward_sources() {
        let mut ledger = RewardLedger::new();
        let provider = addr(1);

        // Block reward as producer
        ledger.distribute_block_reward(provider, 0);
        let after_block = ledger.pending_for(&provider);

        // Inference fee
        ledger.distribute_inference_fee(provider, 1_000_000_000);
        let after_fee = ledger.pending_for(&provider);
        assert!(after_fee > after_block);

        // Storage subsidy
        let staked: HashMap<Address, u128> = [(provider, MIN_STAKE_FOR_STORAGE)].into();
        ledger.distribute_storage_subsidies(&[(provider, 1_000_000)], &staked);
        let after_storage = ledger.pending_for(&provider);
        assert!(after_storage > after_fee);

        // Claim all
        let total = ledger.claim(&provider);
        assert_eq!(total, after_storage);
        assert_eq!(ledger.pending_for(&provider), 0);
    }
}
