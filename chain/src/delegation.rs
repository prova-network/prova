//! Delegation System (CHAIN-031)
//!
//! Allows token holders to delegate stake to inference providers without
//! running infrastructure. Delegators earn proportional rewards minus
//! the provider's commission. Slashing propagates to delegators.
//!
//! Key features:
//! - Delegate/undelegate with unbonding period
//! - Provider-set commission rates (with change cooldown)
//! - Proportional reward distribution
//! - Slashing propagation to delegators
//! - Redelegation (provider-to-provider without full unbonding)
//! - Auto-compound option

use std::collections::HashMap;

/// Token amount in smallest unit.
pub type Amount = u64;
/// Address identifier.
pub type Address = [u8; 32];
/// Epoch number.
pub type Epoch = u64;

/// Unbonding period in epochs before delegated tokens become liquid.
pub const UNBONDING_PERIOD: Epoch = 14400; // ~10 days at 60s epochs
/// Minimum delegation amount.
pub const MIN_DELEGATION: Amount = 1_000_000; // 1 token (6 decimals)
/// Maximum commission rate (basis points, 10000 = 100%).
pub const MAX_COMMISSION_BPS: u16 = 5000; // 50%
/// Commission change cooldown in epochs.
pub const COMMISSION_CHANGE_COOLDOWN: Epoch = 7200; // ~5 days
/// Maximum commission increase per change (basis points).
pub const MAX_COMMISSION_INCREASE_BPS: u16 = 500; // 5%

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DelegationError {
    ProviderNotRegistered,
    BelowMinimum,
    InsufficientDelegation,
    NoDelegationFound,
    UnbondingInProgress,
    CommissionTooHigh,
    CommissionCooldown,
    CommissionIncreaseTooLarge,
    SelfDelegationNotAllowed,
    RedelegationToSameProvider,
    SlashExceedsDelegation,
    ZeroAmount,
}

impl std::fmt::Display for DelegationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ProviderNotRegistered => write!(f, "provider not registered"),
            Self::BelowMinimum => write!(f, "delegation below minimum"),
            Self::InsufficientDelegation => write!(f, "insufficient delegation balance"),
            Self::NoDelegationFound => write!(f, "no delegation found"),
            Self::UnbondingInProgress => write!(f, "unbonding already in progress"),
            Self::CommissionTooHigh => write!(f, "commission exceeds maximum"),
            Self::CommissionCooldown => write!(f, "commission change in cooldown"),
            Self::CommissionIncreaseTooLarge => write!(f, "commission increase too large"),
            Self::SelfDelegationNotAllowed => write!(f, "cannot delegate to self"),
            Self::RedelegationToSameProvider => write!(f, "cannot redelegate to same provider"),
            Self::SlashExceedsDelegation => write!(f, "slash exceeds total delegation"),
            Self::ZeroAmount => write!(f, "amount must be non-zero"),
        }
    }
}

/// Provider registration for delegation.
#[derive(Debug, Clone)]
pub struct ProviderInfo {
    pub address: Address,
    /// Commission rate in basis points.
    pub commission_bps: u16,
    /// Epoch of last commission change.
    pub commission_last_changed: Epoch,
    /// Total delegated stake from all delegators.
    pub total_delegated: Amount,
    /// Whether provider accepts new delegations.
    pub accepting_delegations: bool,
    /// Accumulated rewards pending distribution.
    pub pending_rewards: Amount,
    /// Auto-compound enabled delegators.
    pub auto_compound_delegators: Vec<Address>,
}

/// Individual delegation record.
#[derive(Debug, Clone)]
pub struct Delegation {
    pub delegator: Address,
    pub provider: Address,
    pub amount: Amount,
    pub created_epoch: Epoch,
    /// Accumulated rewards claimable by this delegator.
    pub pending_rewards: Amount,
    /// Whether to auto-compound rewards.
    pub auto_compound: bool,
}

/// Unbonding entry — tokens in transit back to delegator.
#[derive(Debug, Clone)]
pub struct UnbondingEntry {
    pub delegator: Address,
    pub provider: Address,
    pub amount: Amount,
    /// Epoch when unbonding completes.
    pub completion_epoch: Epoch,
}

/// Redelegation entry — in-flight move from one provider to another.
#[derive(Debug, Clone)]
pub struct Redelegation {
    pub delegator: Address,
    pub from_provider: Address,
    pub to_provider: Address,
    pub amount: Amount,
    /// Epoch when redelegation completes (same as unbonding period).
    pub completion_epoch: Epoch,
}

/// Reward distribution event.
#[derive(Debug, Clone)]
pub struct RewardDistribution {
    pub provider: Address,
    pub total_reward: Amount,
    pub provider_commission: Amount,
    pub delegator_shares: Vec<(Address, Amount)>,
    pub epoch: Epoch,
}

/// The delegation ledger — manages all delegation state.
#[derive(Debug)]
pub struct DelegationLedger {
    pub providers: HashMap<Address, ProviderInfo>,
    /// (delegator, provider) → Delegation
    pub delegations: HashMap<(Address, Address), Delegation>,
    pub unbondings: Vec<UnbondingEntry>,
    pub redelegations: Vec<Redelegation>,
    pub current_epoch: Epoch,
    /// History of reward distributions.
    pub distributions: Vec<RewardDistribution>,
}

impl DelegationLedger {
    pub fn new(epoch: Epoch) -> Self {
        Self {
            providers: HashMap::new(),
            delegations: HashMap::new(),
            unbondings: Vec::new(),
            redelegations: Vec::new(),
            current_epoch: epoch,
            distributions: Vec::new(),
        }
    }

    /// Register a provider to accept delegations.
    pub fn register_provider(
        &mut self,
        address: Address,
        commission_bps: u16,
    ) -> Result<(), DelegationError> {
        if commission_bps > MAX_COMMISSION_BPS {
            return Err(DelegationError::CommissionTooHigh);
        }
        self.providers.insert(
            address,
            ProviderInfo {
                address,
                commission_bps,
                commission_last_changed: self.current_epoch,
                total_delegated: 0,
                accepting_delegations: true,
                pending_rewards: 0,
                auto_compound_delegators: Vec::new(),
            },
        );
        Ok(())
    }

    /// Update provider commission rate.
    pub fn update_commission(
        &mut self,
        provider: Address,
        new_commission_bps: u16,
    ) -> Result<(), DelegationError> {
        let info = self
            .providers
            .get_mut(&provider)
            .ok_or(DelegationError::ProviderNotRegistered)?;
        if new_commission_bps > MAX_COMMISSION_BPS {
            return Err(DelegationError::CommissionTooHigh);
        }
        if self.current_epoch < info.commission_last_changed + COMMISSION_CHANGE_COOLDOWN {
            return Err(DelegationError::CommissionCooldown);
        }
        // Can only increase by MAX_COMMISSION_INCREASE_BPS at a time
        if new_commission_bps > info.commission_bps {
            let increase = new_commission_bps - info.commission_bps;
            if increase > MAX_COMMISSION_INCREASE_BPS {
                return Err(DelegationError::CommissionIncreaseTooLarge);
            }
        }
        info.commission_bps = new_commission_bps;
        info.commission_last_changed = self.current_epoch;
        Ok(())
    }

    /// Delegate tokens to a provider.
    pub fn delegate(
        &mut self,
        delegator: Address,
        provider: Address,
        amount: Amount,
        auto_compound: bool,
    ) -> Result<(), DelegationError> {
        if amount == 0 {
            return Err(DelegationError::ZeroAmount);
        }
        if delegator == provider {
            return Err(DelegationError::SelfDelegationNotAllowed);
        }
        let info = self
            .providers
            .get_mut(&provider)
            .ok_or(DelegationError::ProviderNotRegistered)?;
        if !info.accepting_delegations {
            return Err(DelegationError::ProviderNotRegistered);
        }

        let key = (delegator, provider);
        if let Some(existing) = self.delegations.get_mut(&key) {
            existing.amount += amount;
            existing.auto_compound = auto_compound;
        } else {
            if amount < MIN_DELEGATION {
                return Err(DelegationError::BelowMinimum);
            }
            self.delegations.insert(
                key,
                Delegation {
                    delegator,
                    provider,
                    amount,
                    created_epoch: self.current_epoch,
                    pending_rewards: 0,
                    auto_compound,
                },
            );
        }

        info.total_delegated += amount;
        if auto_compound && !info.auto_compound_delegators.contains(&delegator) {
            info.auto_compound_delegators.push(delegator);
        }
        Ok(())
    }

    /// Begin unbonding tokens from a provider.
    pub fn undelegate(
        &mut self,
        delegator: Address,
        provider: Address,
        amount: Amount,
    ) -> Result<(), DelegationError> {
        if amount == 0 {
            return Err(DelegationError::ZeroAmount);
        }
        let key = (delegator, provider);
        let delegation = self
            .delegations
            .get_mut(&key)
            .ok_or(DelegationError::NoDelegationFound)?;
        if delegation.amount < amount {
            return Err(DelegationError::InsufficientDelegation);
        }

        // Check remaining meets minimum or is fully withdrawn
        let remaining = delegation.amount - amount;
        if remaining > 0 && remaining < MIN_DELEGATION {
            return Err(DelegationError::BelowMinimum);
        }

        delegation.amount = remaining;
        if remaining == 0 {
            self.delegations.remove(&key);
        }

        let info = self
            .providers
            .get_mut(&provider)
            .ok_or(DelegationError::ProviderNotRegistered)?;
        info.total_delegated = info.total_delegated.saturating_sub(amount);

        self.unbondings.push(UnbondingEntry {
            delegator,
            provider,
            amount,
            completion_epoch: self.current_epoch + UNBONDING_PERIOD,
        });
        Ok(())
    }

    /// Redelegate from one provider to another without full unbonding.
    pub fn redelegate(
        &mut self,
        delegator: Address,
        from_provider: Address,
        to_provider: Address,
        amount: Amount,
    ) -> Result<(), DelegationError> {
        if amount == 0 {
            return Err(DelegationError::ZeroAmount);
        }
        if from_provider == to_provider {
            return Err(DelegationError::RedelegationToSameProvider);
        }
        if !self.providers.contains_key(&to_provider) {
            return Err(DelegationError::ProviderNotRegistered);
        }

        // Remove from source
        let key = (delegator, from_provider);
        let delegation = self
            .delegations
            .get_mut(&key)
            .ok_or(DelegationError::NoDelegationFound)?;
        if delegation.amount < amount {
            return Err(DelegationError::InsufficientDelegation);
        }
        delegation.amount -= amount;
        if delegation.amount == 0 {
            self.delegations.remove(&key);
        }

        if let Some(info) = self.providers.get_mut(&from_provider) {
            info.total_delegated = info.total_delegated.saturating_sub(amount);
        }

        // Add to destination immediately (but track redelegation for slashing)
        let dest_key = (delegator, to_provider);
        if let Some(existing) = self.delegations.get_mut(&dest_key) {
            existing.amount += amount;
        } else {
            self.delegations.insert(
                dest_key,
                Delegation {
                    delegator,
                    provider: to_provider,
                    amount,
                    created_epoch: self.current_epoch,
                    pending_rewards: 0,
                    auto_compound: false,
                },
            );
        }
        if let Some(info) = self.providers.get_mut(&to_provider) {
            info.total_delegated += amount;
        }

        self.redelegations.push(Redelegation {
            delegator,
            from_provider,
            to_provider,
            amount,
            completion_epoch: self.current_epoch + UNBONDING_PERIOD,
        });
        Ok(())
    }

    /// Distribute rewards to a provider and their delegators.
    pub fn distribute_rewards(
        &mut self,
        provider: Address,
        total_reward: Amount,
    ) -> Result<RewardDistribution, DelegationError> {
        let info = self
            .providers
            .get(&provider)
            .ok_or(DelegationError::ProviderNotRegistered)?;
        let commission_bps = info.commission_bps;
        let total_delegated = info.total_delegated;

        let provider_commission = (total_reward as u128 * commission_bps as u128 / 10000) as Amount;
        let delegator_pool = total_reward - provider_commission;

        let mut delegator_shares = Vec::new();

        if total_delegated > 0 {
            // Collect delegator keys for this provider
            let delegator_keys: Vec<(Address, Address)> = self
                .delegations
                .keys()
                .filter(|(_, p)| *p == provider)
                .cloned()
                .collect();

            for key in &delegator_keys {
                let delegation = self.delegations.get(key).unwrap();
                let share = (delegator_pool as u128 * delegation.amount as u128
                    / total_delegated as u128) as Amount;
                if share > 0 {
                    delegator_shares.push((delegation.delegator, share));
                }
            }

            // Apply rewards
            for (delegator_addr, share) in &delegator_shares {
                let key = (*delegator_addr, provider);
                if let Some(delegation) = self.delegations.get_mut(&key) {
                    if delegation.auto_compound {
                        delegation.amount += share;
                        // Also update provider total
                        if let Some(info) = self.providers.get_mut(&provider) {
                            info.total_delegated += share;
                        }
                    } else {
                        delegation.pending_rewards += share;
                    }
                }
            }
        }

        // Provider keeps commission
        if let Some(info) = self.providers.get_mut(&provider) {
            info.pending_rewards += provider_commission;
        }

        let dist = RewardDistribution {
            provider,
            total_reward,
            provider_commission,
            delegator_shares,
            epoch: self.current_epoch,
        };
        self.distributions.push(dist.clone());
        Ok(dist)
    }

    /// Slash a provider — propagates proportionally to delegators.
    pub fn slash_provider(
        &mut self,
        provider: Address,
        slash_bps: u16,
    ) -> Result<Amount, DelegationError> {
        let info = self
            .providers
            .get(&provider)
            .ok_or(DelegationError::ProviderNotRegistered)?;
        let total = info.total_delegated;

        let slash_amount = (total as u128 * slash_bps as u128 / 10000) as Amount;

        // Slash each delegator proportionally
        let delegator_keys: Vec<(Address, Address)> = self
            .delegations
            .keys()
            .filter(|(_, p)| *p == provider)
            .cloned()
            .collect();

        for key in delegator_keys {
            if let Some(delegation) = self.delegations.get_mut(&key) {
                let delegator_slash =
                    (delegation.amount as u128 * slash_bps as u128 / 10000) as Amount;
                delegation.amount = delegation.amount.saturating_sub(delegator_slash);
            }
        }

        if let Some(info) = self.providers.get_mut(&provider) {
            info.total_delegated = info.total_delegated.saturating_sub(slash_amount);
        }

        // Also slash unbonding entries for this provider
        for unbonding in &mut self.unbondings {
            if unbonding.provider == provider {
                let ub_slash = (unbonding.amount as u128 * slash_bps as u128 / 10000) as Amount;
                unbonding.amount = unbonding.amount.saturating_sub(ub_slash);
            }
        }

        Ok(slash_amount)
    }

    /// Process completed unbondings. Returns list of (delegator, amount) now liquid.
    pub fn process_unbondings(&mut self) -> Vec<(Address, Amount)> {
        let (completed, remaining): (Vec<_>, Vec<_>) = self
            .unbondings
            .drain(..)
            .partition(|u| u.completion_epoch <= self.current_epoch);
        self.unbondings = remaining;
        completed
            .into_iter()
            .map(|u| (u.delegator, u.amount))
            .collect()
    }

    /// Advance epoch.
    pub fn advance_epoch(&mut self) {
        self.current_epoch += 1;
    }

    /// Get total stake for a provider (own + delegated).
    pub fn total_provider_stake(&self, provider: &Address) -> Amount {
        self.providers
            .get(provider)
            .map_or(0, |p| p.total_delegated)
    }

    /// Get delegation info.
    pub fn get_delegation(&self, delegator: &Address, provider: &Address) -> Option<&Delegation> {
        self.delegations.get(&(*delegator, *provider))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn addr(n: u8) -> Address {
        let mut a = [0u8; 32];
        a[0] = n;
        a
    }

    #[test]
    fn test_register_provider() {
        let mut ledger = DelegationLedger::new(0);
        assert!(ledger.register_provider(addr(1), 1000).is_ok());
        assert!(ledger.providers.contains_key(&addr(1)));
    }

    #[test]
    fn test_commission_too_high() {
        let mut ledger = DelegationLedger::new(0);
        assert_eq!(
            ledger.register_provider(addr(1), 6000),
            Err(DelegationError::CommissionTooHigh)
        );
    }

    #[test]
    fn test_delegate_and_query() {
        let mut ledger = DelegationLedger::new(0);
        ledger.register_provider(addr(1), 1000).unwrap();
        ledger.delegate(addr(2), addr(1), 5_000_000, false).unwrap();

        let d = ledger.get_delegation(&addr(2), &addr(1)).unwrap();
        assert_eq!(d.amount, 5_000_000);
        assert_eq!(ledger.total_provider_stake(&addr(1)), 5_000_000);
    }

    #[test]
    fn test_delegate_below_minimum() {
        let mut ledger = DelegationLedger::new(0);
        ledger.register_provider(addr(1), 1000).unwrap();
        assert_eq!(
            ledger.delegate(addr(2), addr(1), 100, false),
            Err(DelegationError::BelowMinimum)
        );
    }

    #[test]
    fn test_self_delegation_blocked() {
        let mut ledger = DelegationLedger::new(0);
        ledger.register_provider(addr(1), 1000).unwrap();
        assert_eq!(
            ledger.delegate(addr(1), addr(1), 5_000_000, false),
            Err(DelegationError::SelfDelegationNotAllowed)
        );
    }

    #[test]
    fn test_undelegate_with_unbonding() {
        let mut ledger = DelegationLedger::new(100);
        ledger.register_provider(addr(1), 1000).unwrap();
        ledger.delegate(addr(2), addr(1), 5_000_000, false).unwrap();
        ledger.undelegate(addr(2), addr(1), 5_000_000).unwrap();

        assert!(ledger.get_delegation(&addr(2), &addr(1)).is_none());
        assert_eq!(ledger.unbondings.len(), 1);
        assert_eq!(
            ledger.unbondings[0].completion_epoch,
            100 + UNBONDING_PERIOD
        );

        // Not yet completed
        assert!(ledger.process_unbondings().is_empty());

        // Advance past unbonding
        ledger.current_epoch = 100 + UNBONDING_PERIOD;
        let completed = ledger.process_unbondings();
        assert_eq!(completed.len(), 1);
        assert_eq!(completed[0], (addr(2), 5_000_000));
    }

    #[test]
    fn test_reward_distribution_with_commission() {
        let mut ledger = DelegationLedger::new(0);
        ledger.register_provider(addr(1), 2000).unwrap(); // 20% commission
        ledger.delegate(addr(2), addr(1), 3_000_000, false).unwrap();
        ledger.delegate(addr(3), addr(1), 7_000_000, false).unwrap();

        let dist = ledger.distribute_rewards(addr(1), 1_000_000).unwrap();
        // 20% commission = 200,000 to provider
        assert_eq!(dist.provider_commission, 200_000);
        // 800,000 split 30/70
        let shares: HashMap<Address, Amount> = dist.delegator_shares.into_iter().collect();
        assert_eq!(*shares.get(&addr(2)).unwrap(), 240_000); // 30% of 800k
        assert_eq!(*shares.get(&addr(3)).unwrap(), 560_000); // 70% of 800k
    }

    #[test]
    fn test_auto_compound() {
        let mut ledger = DelegationLedger::new(0);
        ledger.register_provider(addr(1), 0).unwrap(); // 0% commission
        ledger.delegate(addr(2), addr(1), 10_000_000, true).unwrap();

        ledger.distribute_rewards(addr(1), 1_000_000).unwrap();

        let d = ledger.get_delegation(&addr(2), &addr(1)).unwrap();
        // Auto-compounded: original 10M + 1M reward
        assert_eq!(d.amount, 11_000_000);
        assert_eq!(d.pending_rewards, 0);
    }

    #[test]
    fn test_slash_propagation() {
        let mut ledger = DelegationLedger::new(0);
        ledger.register_provider(addr(1), 1000).unwrap();
        ledger.delegate(addr(2), addr(1), 6_000_000, false).unwrap();
        ledger.delegate(addr(3), addr(1), 4_000_000, false).unwrap();

        // Slash 10%
        let slashed = ledger.slash_provider(addr(1), 1000).unwrap();
        assert_eq!(slashed, 1_000_000);

        let d2 = ledger.get_delegation(&addr(2), &addr(1)).unwrap();
        assert_eq!(d2.amount, 5_400_000); // 6M - 10%
        let d3 = ledger.get_delegation(&addr(3), &addr(1)).unwrap();
        assert_eq!(d3.amount, 3_600_000); // 4M - 10%
    }

    #[test]
    fn test_slash_unbonding_entries() {
        let mut ledger = DelegationLedger::new(0);
        ledger.register_provider(addr(1), 1000).unwrap();
        ledger
            .delegate(addr(2), addr(1), 10_000_000, false)
            .unwrap();
        ledger.undelegate(addr(2), addr(1), 5_000_000).unwrap();

        // Slash 20% — affects both active and unbonding
        ledger.slash_provider(addr(1), 2000).unwrap();

        let d = ledger.get_delegation(&addr(2), &addr(1)).unwrap();
        assert_eq!(d.amount, 4_000_000); // 5M - 20%
        assert_eq!(ledger.unbondings[0].amount, 4_000_000); // 5M - 20%
    }

    #[test]
    fn test_redelegate() {
        let mut ledger = DelegationLedger::new(0);
        ledger.register_provider(addr(1), 1000).unwrap();
        ledger.register_provider(addr(3), 500).unwrap();
        ledger
            .delegate(addr(2), addr(1), 10_000_000, false)
            .unwrap();

        ledger
            .redelegate(addr(2), addr(1), addr(3), 4_000_000)
            .unwrap();

        let d1 = ledger.get_delegation(&addr(2), &addr(1)).unwrap();
        assert_eq!(d1.amount, 6_000_000);
        let d3 = ledger.get_delegation(&addr(2), &addr(3)).unwrap();
        assert_eq!(d3.amount, 4_000_000);
        assert_eq!(ledger.redelegations.len(), 1);
    }

    #[test]
    fn test_redelegate_to_same_provider_blocked() {
        let mut ledger = DelegationLedger::new(0);
        ledger.register_provider(addr(1), 1000).unwrap();
        ledger
            .delegate(addr(2), addr(1), 10_000_000, false)
            .unwrap();
        assert_eq!(
            ledger.redelegate(addr(2), addr(1), addr(1), 5_000_000),
            Err(DelegationError::RedelegationToSameProvider)
        );
    }

    #[test]
    fn test_commission_change_cooldown() {
        let mut ledger = DelegationLedger::new(0);
        ledger.register_provider(addr(1), 1000).unwrap();

        // Immediate change should fail (cooldown)
        assert_eq!(
            ledger.update_commission(addr(1), 1200),
            Err(DelegationError::CommissionCooldown)
        );

        // After cooldown
        ledger.current_epoch = COMMISSION_CHANGE_COOLDOWN;
        assert!(ledger.update_commission(addr(1), 1500).is_ok());
    }

    #[test]
    fn test_commission_increase_cap() {
        let mut ledger = DelegationLedger::new(0);
        ledger.register_provider(addr(1), 1000).unwrap();
        ledger.current_epoch = COMMISSION_CHANGE_COOLDOWN;

        // Try to increase by 600 bps (over 500 cap)
        assert_eq!(
            ledger.update_commission(addr(1), 1600),
            Err(DelegationError::CommissionIncreaseTooLarge)
        );

        // Decrease is unlimited
        assert!(ledger.update_commission(addr(1), 100).is_ok());
    }

    #[test]
    fn test_zero_amount_errors() {
        let mut ledger = DelegationLedger::new(0);
        ledger.register_provider(addr(1), 1000).unwrap();
        assert_eq!(
            ledger.delegate(addr(2), addr(1), 0, false),
            Err(DelegationError::ZeroAmount)
        );
        assert_eq!(
            ledger.undelegate(addr(2), addr(1), 0),
            Err(DelegationError::ZeroAmount)
        );
        assert_eq!(
            ledger.redelegate(addr(2), addr(1), addr(3), 0),
            Err(DelegationError::ZeroAmount)
        );
    }

    #[test]
    fn test_incremental_delegation() {
        let mut ledger = DelegationLedger::new(0);
        ledger.register_provider(addr(1), 1000).unwrap();
        ledger.delegate(addr(2), addr(1), 5_000_000, false).unwrap();
        // Add more (below minimum is ok for existing delegation)
        ledger.delegate(addr(2), addr(1), 500, false).unwrap();
        let d = ledger.get_delegation(&addr(2), &addr(1)).unwrap();
        assert_eq!(d.amount, 5_000_500);
    }

    #[test]
    fn test_partial_undelegate_below_minimum() {
        let mut ledger = DelegationLedger::new(0);
        ledger.register_provider(addr(1), 1000).unwrap();
        ledger.delegate(addr(2), addr(1), 2_000_000, false).unwrap();
        // Undelegating 1.5M would leave 500K < MIN_DELEGATION
        assert_eq!(
            ledger.undelegate(addr(2), addr(1), 1_500_000),
            Err(DelegationError::BelowMinimum)
        );
        // Full withdrawal is fine
        assert!(ledger.undelegate(addr(2), addr(1), 2_000_000).is_ok());
    }

    #[test]
    fn test_error_display() {
        assert_eq!(
            format!("{}", DelegationError::ZeroAmount),
            "amount must be non-zero"
        );
        assert_eq!(
            format!("{}", DelegationError::CommissionCooldown),
            "commission change in cooldown"
        );
    }
}
