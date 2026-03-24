//! Delegation Client SDK (SDK-010)
//!
//! High-level client for managing stake delegation: delegate, undelegate,
//! redelegate, claim rewards, query provider info, and track unbondings.
//! Wraps RPC calls and transaction signing into ergonomic operations.

use prova_chain::delegation::{
    Address, Amount, Delegation, DelegationError, Epoch, ProviderInfo, Redelegation,
    RewardDistribution, UnbondingEntry, MAX_COMMISSION_BPS, MIN_DELEGATION, UNBONDING_PERIOD,
};
use std::collections::HashMap;

// ── Transport Abstraction ────────────────────────────────────

/// RPC response wrapper.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DelegationRpcError {
    ConnectionFailed(String),
    Timeout,
    InvalidResponse(String),
    ChainError(DelegationError),
    InsufficientBalance,
    SigningFailed,
    Nonce(String),
}

impl std::fmt::Display for DelegationRpcError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ConnectionFailed(s) => write!(f, "connection failed: {s}"),
            Self::Timeout => write!(f, "request timed out"),
            Self::InvalidResponse(s) => write!(f, "invalid response: {s}"),
            Self::ChainError(e) => write!(f, "chain error: {e}"),
            Self::InsufficientBalance => write!(f, "insufficient balance for delegation"),
            Self::SigningFailed => write!(f, "transaction signing failed"),
            Self::Nonce(s) => write!(f, "nonce error: {s}"),
        }
    }
}

/// Provider summary for display/queries.
#[derive(Debug, Clone)]
pub struct ProviderSummary {
    pub address: Address,
    pub commission_bps: u16,
    pub total_delegated: Amount,
    pub accepting_delegations: bool,
    pub delegator_count: usize,
    /// Estimated annual yield in basis points (from recent distributions).
    pub estimated_apy_bps: u32,
}

/// Delegation position summary for a delegator.
#[derive(Debug, Clone)]
pub struct DelegationPosition {
    pub provider: Address,
    pub amount: Amount,
    pub pending_rewards: Amount,
    pub auto_compound: bool,
    pub created_epoch: Epoch,
}

/// Unbonding status.
#[derive(Debug, Clone)]
pub struct UnbondingStatus {
    pub provider: Address,
    pub amount: Amount,
    pub completion_epoch: Epoch,
    pub epochs_remaining: Epoch,
}

/// Redelegation status.
#[derive(Debug, Clone)]
pub struct RedelegationStatus {
    pub from_provider: Address,
    pub to_provider: Address,
    pub amount: Amount,
    pub completion_epoch: Epoch,
    pub epochs_remaining: Epoch,
}

/// Reward history entry.
#[derive(Debug, Clone)]
pub struct RewardEntry {
    pub provider: Address,
    pub amount: Amount,
    pub epoch: Epoch,
    pub was_compounded: bool,
}

/// Portfolio overview — all positions for a delegator.
#[derive(Debug, Clone)]
pub struct DelegationPortfolio {
    pub delegator: Address,
    pub positions: Vec<DelegationPosition>,
    pub unbondings: Vec<UnbondingStatus>,
    pub redelegations: Vec<RedelegationStatus>,
    pub total_delegated: Amount,
    pub total_pending_rewards: Amount,
    pub total_unbonding: Amount,
}

// ── Mock RPC Backend (for testing) ───────────────────────────

/// Simulated chain state for testing the SDK without a live node.
#[derive(Debug)]
struct MockChainState {
    providers: HashMap<Address, ProviderInfo>,
    delegations: HashMap<(Address, Address), Delegation>,
    unbondings: Vec<UnbondingEntry>,
    redelegations: Vec<Redelegation>,
    balances: HashMap<Address, Amount>,
    distributions: Vec<RewardDistribution>,
    current_epoch: Epoch,
    nonces: HashMap<Address, u64>,
}

impl MockChainState {
    fn new(epoch: Epoch) -> Self {
        Self {
            providers: HashMap::new(),
            delegations: HashMap::new(),
            unbondings: Vec::new(),
            redelegations: Vec::new(),
            balances: HashMap::new(),
            distributions: Vec::new(),
            current_epoch: epoch,
            nonces: HashMap::new(),
        }
    }
}

// ── Delegation Client ────────────────────────────────────────

/// High-level delegation client. In production this wraps JSON-RPC;
/// here we embed a mock chain state for testing.
pub struct DelegationClient {
    /// Delegator address (the "wallet owner").
    delegator: Address,
    /// Signing key seed.
    secret: [u8; 32],
    /// Mock chain state (replaced by RPC in production).
    state: MockChainState,
}

impl DelegationClient {
    /// Create a new delegation client for the given delegator.
    pub fn new(delegator: Address, secret: [u8; 32], epoch: Epoch) -> Self {
        Self {
            delegator,
            secret,
            state: MockChainState::new(epoch),
        }
    }

    /// Set balance for an address (test helper).
    pub fn set_balance(&mut self, addr: Address, amount: Amount) {
        self.state.balances.insert(addr, amount);
    }

    /// Get balance.
    pub fn balance(&self, addr: &Address) -> Amount {
        self.state.balances.get(addr).copied().unwrap_or(0)
    }

    /// Advance epoch (test helper).
    pub fn advance_epoch(&mut self, epochs: Epoch) {
        self.state.current_epoch += epochs;
        // Process completed unbondings.
        let completed: Vec<_> = self
            .state
            .unbondings
            .iter()
            .filter(|u| u.completion_epoch <= self.state.current_epoch)
            .cloned()
            .collect();
        for entry in &completed {
            *self.state.balances.entry(entry.delegator).or_insert(0) += entry.amount;
        }
        self.state
            .unbondings
            .retain(|u| u.completion_epoch > self.state.current_epoch);
        // Process completed redelegations.
        let completed_redel: Vec<_> = self
            .state
            .redelegations
            .iter()
            .filter(|r| r.completion_epoch <= self.state.current_epoch)
            .cloned()
            .collect();
        for entry in &completed_redel {
            let del = self
                .state
                .delegations
                .entry((entry.delegator, entry.to_provider))
                .or_insert(Delegation {
                    delegator: entry.delegator,
                    provider: entry.to_provider,
                    amount: 0,
                    created_epoch: self.state.current_epoch,
                    pending_rewards: 0,
                    auto_compound: false,
                });
            del.amount += entry.amount;
            if let Some(prov) = self.state.providers.get_mut(&entry.to_provider) {
                prov.total_delegated += entry.amount;
            }
        }
        self.state
            .redelegations
            .retain(|r| r.completion_epoch > self.state.current_epoch);
    }

    /// Register a provider (test helper — in prod this is a chain tx).
    pub fn register_provider(
        &mut self,
        address: Address,
        commission_bps: u16,
    ) -> Result<(), DelegationRpcError> {
        if commission_bps > MAX_COMMISSION_BPS {
            return Err(DelegationRpcError::ChainError(
                DelegationError::CommissionTooHigh,
            ));
        }
        self.state.providers.insert(
            address,
            ProviderInfo {
                address,
                commission_bps,
                commission_last_changed: self.state.current_epoch,
                total_delegated: 0,
                accepting_delegations: true,
                pending_rewards: 0,
                auto_compound_delegators: Vec::new(),
            },
        );
        Ok(())
    }

    // ── Core Operations ──────────────────────────────────────

    /// Delegate tokens to a provider.
    pub fn delegate(
        &mut self,
        provider: Address,
        amount: Amount,
    ) -> Result<(), DelegationRpcError> {
        if amount == 0 {
            return Err(DelegationRpcError::ChainError(DelegationError::ZeroAmount));
        }
        if amount < MIN_DELEGATION {
            return Err(DelegationRpcError::ChainError(
                DelegationError::BelowMinimum,
            ));
        }
        if provider == self.delegator {
            return Err(DelegationRpcError::ChainError(
                DelegationError::SelfDelegationNotAllowed,
            ));
        }
        let prov = self
            .state
            .providers
            .get(&provider)
            .ok_or(DelegationRpcError::ChainError(
                DelegationError::ProviderNotRegistered,
            ))?;
        if !prov.accepting_delegations {
            return Err(DelegationRpcError::ChainError(
                DelegationError::ProviderNotRegistered,
            ));
        }
        let bal = self
            .state
            .balances
            .get(&self.delegator)
            .copied()
            .unwrap_or(0);
        if bal < amount {
            return Err(DelegationRpcError::InsufficientBalance);
        }

        // Deduct balance.
        *self.state.balances.entry(self.delegator).or_insert(0) -= amount;

        // Update or create delegation.
        let del = self
            .state
            .delegations
            .entry((self.delegator, provider))
            .or_insert(Delegation {
                delegator: self.delegator,
                provider,
                amount: 0,
                created_epoch: self.state.current_epoch,
                pending_rewards: 0,
                auto_compound: false,
            });
        del.amount += amount;

        // Update provider total.
        if let Some(prov) = self.state.providers.get_mut(&provider) {
            prov.total_delegated += amount;
        }

        Ok(())
    }

    /// Undelegate tokens from a provider (starts unbonding).
    pub fn undelegate(
        &mut self,
        provider: Address,
        amount: Amount,
    ) -> Result<UnbondingStatus, DelegationRpcError> {
        if amount == 0 {
            return Err(DelegationRpcError::ChainError(DelegationError::ZeroAmount));
        }
        let del = self
            .state
            .delegations
            .get_mut(&(self.delegator, provider))
            .ok_or(DelegationRpcError::ChainError(
                DelegationError::NoDelegationFound,
            ))?;
        if del.amount < amount {
            return Err(DelegationRpcError::ChainError(
                DelegationError::InsufficientDelegation,
            ));
        }
        del.amount -= amount;

        if let Some(prov) = self.state.providers.get_mut(&provider) {
            prov.total_delegated = prov.total_delegated.saturating_sub(amount);
        }

        let completion = self.state.current_epoch + UNBONDING_PERIOD;
        self.state.unbondings.push(UnbondingEntry {
            delegator: self.delegator,
            provider,
            amount,
            completion_epoch: completion,
        });

        // Remove empty delegations.
        if del.amount == 0 && del.pending_rewards == 0 {
            self.state.delegations.remove(&(self.delegator, provider));
        }

        Ok(UnbondingStatus {
            provider,
            amount,
            completion_epoch: completion,
            epochs_remaining: UNBONDING_PERIOD,
        })
    }

    /// Redelegate from one provider to another without full unbonding.
    pub fn redelegate(
        &mut self,
        from_provider: Address,
        to_provider: Address,
        amount: Amount,
    ) -> Result<RedelegationStatus, DelegationRpcError> {
        if amount == 0 {
            return Err(DelegationRpcError::ChainError(DelegationError::ZeroAmount));
        }
        if from_provider == to_provider {
            return Err(DelegationRpcError::ChainError(
                DelegationError::RedelegationToSameProvider,
            ));
        }
        // Verify destination provider exists.
        self.state
            .providers
            .get(&to_provider)
            .ok_or(DelegationRpcError::ChainError(
                DelegationError::ProviderNotRegistered,
            ))?;

        let del = self
            .state
            .delegations
            .get_mut(&(self.delegator, from_provider))
            .ok_or(DelegationRpcError::ChainError(
                DelegationError::NoDelegationFound,
            ))?;
        if del.amount < amount {
            return Err(DelegationRpcError::ChainError(
                DelegationError::InsufficientDelegation,
            ));
        }
        del.amount -= amount;

        if let Some(prov) = self.state.providers.get_mut(&from_provider) {
            prov.total_delegated = prov.total_delegated.saturating_sub(amount);
        }

        // Remove empty source delegation.
        if del.amount == 0 && del.pending_rewards == 0 {
            self.state
                .delegations
                .remove(&(self.delegator, from_provider));
        }

        let completion = self.state.current_epoch + UNBONDING_PERIOD;
        self.state.redelegations.push(Redelegation {
            delegator: self.delegator,
            from_provider,
            to_provider,
            amount,
            completion_epoch: completion,
        });

        Ok(RedelegationStatus {
            from_provider,
            to_provider,
            amount,
            completion_epoch: completion,
            epochs_remaining: UNBONDING_PERIOD,
        })
    }

    /// Enable auto-compound for a provider delegation.
    pub fn set_auto_compound(
        &mut self,
        provider: Address,
        enabled: bool,
    ) -> Result<(), DelegationRpcError> {
        let del = self
            .state
            .delegations
            .get_mut(&(self.delegator, provider))
            .ok_or(DelegationRpcError::ChainError(
                DelegationError::NoDelegationFound,
            ))?;
        del.auto_compound = enabled;

        if let Some(prov) = self.state.providers.get_mut(&provider) {
            if enabled {
                if !prov.auto_compound_delegators.contains(&self.delegator) {
                    prov.auto_compound_delegators.push(self.delegator);
                }
            } else {
                prov.auto_compound_delegators
                    .retain(|a| a != &self.delegator);
            }
        }
        Ok(())
    }

    /// Claim pending rewards from a specific provider.
    pub fn claim_rewards(&mut self, provider: Address) -> Result<Amount, DelegationRpcError> {
        let del = self
            .state
            .delegations
            .get_mut(&(self.delegator, provider))
            .ok_or(DelegationRpcError::ChainError(
                DelegationError::NoDelegationFound,
            ))?;
        let rewards = del.pending_rewards;
        del.pending_rewards = 0;
        *self.state.balances.entry(self.delegator).or_insert(0) += rewards;
        Ok(rewards)
    }

    /// Claim rewards from ALL providers at once.
    pub fn claim_all_rewards(&mut self) -> Result<Amount, DelegationRpcError> {
        let keys: Vec<_> = self
            .state
            .delegations
            .keys()
            .filter(|(d, _)| *d == self.delegator)
            .cloned()
            .collect();
        let mut total = 0u64;
        for key in keys {
            if let Some(del) = self.state.delegations.get_mut(&key) {
                total += del.pending_rewards;
                del.pending_rewards = 0;
            }
        }
        *self.state.balances.entry(self.delegator).or_insert(0) += total;
        Ok(total)
    }

    // ── Queries ──────────────────────────────────────────────

    /// Get provider summary.
    pub fn provider_info(&self, provider: &Address) -> Option<ProviderSummary> {
        let prov = self.state.providers.get(provider)?;
        let delegator_count = self
            .state
            .delegations
            .iter()
            .filter(|((_, p), d)| p == provider && d.amount > 0)
            .count();
        // Estimate APY from last 100 epochs of distributions.
        let recent_rewards: Amount = self
            .state
            .distributions
            .iter()
            .filter(|d| d.provider == *provider && d.epoch + 100 >= self.state.current_epoch)
            .map(|d| d.total_reward)
            .sum();
        let epochs_window = 100u64.min(self.state.current_epoch);
        let apy_bps = if prov.total_delegated > 0 && epochs_window > 0 {
            // Annualize: (rewards / stake) * (epochs_per_year / window) * 10000
            let epochs_per_year = 525960u64; // ~365.25 days at 60s
            ((recent_rewards as u128 * epochs_per_year as u128 * 10000)
                / (prov.total_delegated as u128 * epochs_window as u128)) as u32
        } else {
            0
        };

        Some(ProviderSummary {
            address: *provider,
            commission_bps: prov.commission_bps,
            total_delegated: prov.total_delegated,
            accepting_delegations: prov.accepting_delegations,
            delegator_count,
            estimated_apy_bps: apy_bps,
        })
    }

    /// List all providers.
    pub fn list_providers(&self) -> Vec<ProviderSummary> {
        self.state
            .providers
            .keys()
            .filter_map(|addr| self.provider_info(addr))
            .collect()
    }

    /// Get delegation position for a specific provider.
    pub fn position(&self, provider: &Address) -> Option<DelegationPosition> {
        let del = self.state.delegations.get(&(self.delegator, *provider))?;
        Some(DelegationPosition {
            provider: *provider,
            amount: del.amount,
            pending_rewards: del.pending_rewards,
            auto_compound: del.auto_compound,
            created_epoch: del.created_epoch,
        })
    }

    /// Get full portfolio overview.
    pub fn portfolio(&self) -> DelegationPortfolio {
        let positions: Vec<DelegationPosition> = self
            .state
            .delegations
            .iter()
            .filter(|((d, _), _)| *d == self.delegator)
            .map(|((_, p), del)| DelegationPosition {
                provider: *p,
                amount: del.amount,
                pending_rewards: del.pending_rewards,
                auto_compound: del.auto_compound,
                created_epoch: del.created_epoch,
            })
            .collect();

        let unbondings: Vec<UnbondingStatus> = self
            .state
            .unbondings
            .iter()
            .filter(|u| u.delegator == self.delegator)
            .map(|u| UnbondingStatus {
                provider: u.provider,
                amount: u.amount,
                completion_epoch: u.completion_epoch,
                epochs_remaining: u.completion_epoch.saturating_sub(self.state.current_epoch),
            })
            .collect();

        let redelegations: Vec<RedelegationStatus> = self
            .state
            .redelegations
            .iter()
            .filter(|r| r.delegator == self.delegator)
            .map(|r| RedelegationStatus {
                from_provider: r.from_provider,
                to_provider: r.to_provider,
                amount: r.amount,
                completion_epoch: r.completion_epoch,
                epochs_remaining: r.completion_epoch.saturating_sub(self.state.current_epoch),
            })
            .collect();

        let total_delegated = positions.iter().map(|p| p.amount).sum();
        let total_pending_rewards = positions.iter().map(|p| p.pending_rewards).sum();
        let total_unbonding = unbondings.iter().map(|u| u.amount).sum();

        DelegationPortfolio {
            delegator: self.delegator,
            positions,
            unbondings,
            redelegations,
            total_delegated,
            total_pending_rewards,
            total_unbonding,
        }
    }

    /// Get reward history for this delegator.
    pub fn reward_history(&self) -> Vec<RewardEntry> {
        self.state
            .distributions
            .iter()
            .flat_map(|dist| {
                dist.delegator_shares
                    .iter()
                    .filter(|(addr, _)| *addr == self.delegator)
                    .map(move |(_, amount)| {
                        let auto = self
                            .state
                            .delegations
                            .get(&(self.delegator, dist.provider))
                            .map(|d| d.auto_compound)
                            .unwrap_or(false);
                        RewardEntry {
                            provider: dist.provider,
                            amount: *amount,
                            epoch: dist.epoch,
                            was_compounded: auto,
                        }
                    })
            })
            .collect()
    }

    // ── Reward Distribution (test helper, simulates chain) ───

    /// Distribute rewards to a provider's delegators (simulates epoch rewards).
    pub fn distribute_rewards(
        &mut self,
        provider: Address,
        total_reward: Amount,
    ) -> Result<(), DelegationRpcError> {
        let prov = self
            .state
            .providers
            .get(&provider)
            .ok_or(DelegationRpcError::ChainError(
                DelegationError::ProviderNotRegistered,
            ))?
            .clone();

        if prov.total_delegated == 0 {
            return Ok(());
        }

        let commission = (total_reward as u128 * prov.commission_bps as u128 / 10000) as Amount;
        let delegator_pool = total_reward - commission;

        // Distribute proportionally.
        let delegator_keys: Vec<_> = self
            .state
            .delegations
            .iter()
            .filter(|((_, p), d)| *p == provider && d.amount > 0)
            .map(|(k, _)| *k)
            .collect();

        let mut shares = Vec::new();
        for key in &delegator_keys {
            let del = self.state.delegations.get(key).unwrap();
            let share = (delegator_pool as u128 * del.amount as u128 / prov.total_delegated as u128)
                as Amount;
            shares.push((key.0, share));
        }

        // Apply rewards.
        for (delegator_addr, share) in &shares {
            if let Some(del) = self.state.delegations.get_mut(&(*delegator_addr, provider)) {
                if del.auto_compound {
                    del.amount += share;
                    if let Some(prov) = self.state.providers.get_mut(&provider) {
                        prov.total_delegated += share;
                    }
                } else {
                    del.pending_rewards += share;
                }
            }
        }

        // Give commission to provider balance.
        *self.state.balances.entry(provider).or_insert(0) += commission;

        self.state.distributions.push(RewardDistribution {
            provider,
            total_reward,
            provider_commission: commission,
            delegator_shares: shares,
            epoch: self.state.current_epoch,
        });

        Ok(())
    }
}

// ── Tests ────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn addr(n: u8) -> Address {
        let mut a = [0u8; 32];
        a[0] = n;
        a
    }
    fn secret() -> [u8; 32] {
        [42u8; 32]
    }

    #[test]
    fn test_delegate_basic() {
        let mut client = DelegationClient::new(addr(1), secret(), 100);
        client.set_balance(addr(1), 100_000_000);
        client.register_provider(addr(2), 1000).unwrap();

        client.delegate(addr(2), 50_000_000).unwrap();
        assert_eq!(client.balance(&addr(1)), 50_000_000);

        let pos = client.position(&addr(2)).unwrap();
        assert_eq!(pos.amount, 50_000_000);
    }

    #[test]
    fn test_delegate_insufficient_balance() {
        let mut client = DelegationClient::new(addr(1), secret(), 100);
        client.set_balance(addr(1), 500_000);
        client.register_provider(addr(2), 1000).unwrap();

        let err = client.delegate(addr(2), 50_000_000).unwrap_err();
        assert_eq!(err, DelegationRpcError::InsufficientBalance);
    }

    #[test]
    fn test_delegate_below_minimum() {
        let mut client = DelegationClient::new(addr(1), secret(), 100);
        client.set_balance(addr(1), 100_000_000);
        client.register_provider(addr(2), 1000).unwrap();

        let err = client.delegate(addr(2), 100).unwrap_err();
        assert_eq!(
            err,
            DelegationRpcError::ChainError(DelegationError::BelowMinimum)
        );
    }

    #[test]
    fn test_delegate_self_not_allowed() {
        let mut client = DelegationClient::new(addr(1), secret(), 100);
        client.set_balance(addr(1), 100_000_000);
        client.register_provider(addr(1), 1000).unwrap();

        let err = client.delegate(addr(1), 5_000_000).unwrap_err();
        assert_eq!(
            err,
            DelegationRpcError::ChainError(DelegationError::SelfDelegationNotAllowed)
        );
    }

    #[test]
    fn test_undelegate_and_unbonding() {
        let mut client = DelegationClient::new(addr(1), secret(), 100);
        client.set_balance(addr(1), 100_000_000);
        client.register_provider(addr(2), 1000).unwrap();
        client.delegate(addr(2), 50_000_000).unwrap();

        let status = client.undelegate(addr(2), 20_000_000).unwrap();
        assert_eq!(status.amount, 20_000_000);
        assert_eq!(status.epochs_remaining, UNBONDING_PERIOD);

        // Still unbonding — balance unchanged.
        assert_eq!(client.balance(&addr(1)), 50_000_000);

        // Advance past unbonding.
        client.advance_epoch(UNBONDING_PERIOD + 1);
        assert_eq!(client.balance(&addr(1)), 70_000_000);
    }

    #[test]
    fn test_redelegate() {
        let mut client = DelegationClient::new(addr(1), secret(), 100);
        client.set_balance(addr(1), 100_000_000);
        client.register_provider(addr(2), 1000).unwrap();
        client.register_provider(addr(3), 500).unwrap();
        client.delegate(addr(2), 50_000_000).unwrap();

        let status = client.redelegate(addr(2), addr(3), 30_000_000).unwrap();
        assert_eq!(status.from_provider, addr(2));
        assert_eq!(status.to_provider, addr(3));

        // Source reduced immediately.
        let pos2 = client.position(&addr(2)).unwrap();
        assert_eq!(pos2.amount, 20_000_000);

        // Destination not yet active (in transit).
        assert!(client.position(&addr(3)).is_none());

        // Complete redelegation.
        client.advance_epoch(UNBONDING_PERIOD + 1);
        let pos3 = client.position(&addr(3)).unwrap();
        assert_eq!(pos3.amount, 30_000_000);
    }

    #[test]
    fn test_redelegate_same_provider_fails() {
        let mut client = DelegationClient::new(addr(1), secret(), 100);
        client.set_balance(addr(1), 100_000_000);
        client.register_provider(addr(2), 1000).unwrap();
        client.delegate(addr(2), 50_000_000).unwrap();

        let err = client.redelegate(addr(2), addr(2), 10_000_000).unwrap_err();
        assert_eq!(
            err,
            DelegationRpcError::ChainError(DelegationError::RedelegationToSameProvider)
        );
    }

    #[test]
    fn test_claim_rewards() {
        let mut client = DelegationClient::new(addr(1), secret(), 100);
        client.set_balance(addr(1), 100_000_000);
        client.register_provider(addr(2), 1000).unwrap(); // 10% commission
        client.delegate(addr(2), 50_000_000).unwrap();

        // Distribute 1M reward.
        client.distribute_rewards(addr(2), 1_000_000).unwrap();

        // 10% commission = 100k to provider, 900k to delegator.
        let pos = client.position(&addr(2)).unwrap();
        assert_eq!(pos.pending_rewards, 900_000);

        let claimed = client.claim_rewards(addr(2)).unwrap();
        assert_eq!(claimed, 900_000);
        assert_eq!(client.balance(&addr(1)), 50_900_000);
    }

    #[test]
    fn test_auto_compound() {
        let mut client = DelegationClient::new(addr(1), secret(), 100);
        client.set_balance(addr(1), 100_000_000);
        client.register_provider(addr(2), 1000).unwrap();
        client.delegate(addr(2), 50_000_000).unwrap();
        client.set_auto_compound(addr(2), true).unwrap();

        client.distribute_rewards(addr(2), 1_000_000).unwrap();

        // Rewards auto-compounded into stake, not pending.
        let pos = client.position(&addr(2)).unwrap();
        assert_eq!(pos.pending_rewards, 0);
        assert_eq!(pos.amount, 50_900_000); // 50M + 900k (90% of 1M)
    }

    #[test]
    fn test_claim_all_rewards() {
        let mut client = DelegationClient::new(addr(1), secret(), 100);
        client.set_balance(addr(1), 200_000_000);
        client.register_provider(addr(2), 1000).unwrap();
        client.register_provider(addr(3), 500).unwrap();
        client.delegate(addr(2), 50_000_000).unwrap();
        client.delegate(addr(3), 50_000_000).unwrap();

        client.distribute_rewards(addr(2), 1_000_000).unwrap();
        client.distribute_rewards(addr(3), 2_000_000).unwrap();

        // Provider 2: 900k to delegator. Provider 3: 5% = 100k commission, 1.9M to delegator.
        let total = client.claim_all_rewards().unwrap();
        assert_eq!(total, 900_000 + 1_900_000);
    }

    #[test]
    fn test_portfolio_overview() {
        let mut client = DelegationClient::new(addr(1), secret(), 100);
        client.set_balance(addr(1), 200_000_000);
        client.register_provider(addr(2), 1000).unwrap();
        client.register_provider(addr(3), 500).unwrap();
        client.delegate(addr(2), 50_000_000).unwrap();
        client.delegate(addr(3), 30_000_000).unwrap();
        client.undelegate(addr(2), 10_000_000).unwrap();

        let portfolio = client.portfolio();
        assert_eq!(portfolio.total_delegated, 40_000_000 + 30_000_000);
        assert_eq!(portfolio.total_unbonding, 10_000_000);
        assert_eq!(portfolio.positions.len(), 2);
        assert_eq!(portfolio.unbondings.len(), 1);
    }

    #[test]
    fn test_provider_info_and_list() {
        let mut client = DelegationClient::new(addr(1), secret(), 100);
        client.set_balance(addr(1), 200_000_000);
        client.register_provider(addr(2), 1000).unwrap();
        client.register_provider(addr(3), 2000).unwrap();
        client.delegate(addr(2), 50_000_000).unwrap();

        let info = client.provider_info(&addr(2)).unwrap();
        assert_eq!(info.commission_bps, 1000);
        assert_eq!(info.total_delegated, 50_000_000);
        assert_eq!(info.delegator_count, 1);

        let providers = client.list_providers();
        assert_eq!(providers.len(), 2);
    }

    #[test]
    fn test_reward_history() {
        let mut client = DelegationClient::new(addr(1), secret(), 100);
        client.set_balance(addr(1), 100_000_000);
        client.register_provider(addr(2), 1000).unwrap();
        client.delegate(addr(2), 50_000_000).unwrap();

        client.distribute_rewards(addr(2), 500_000).unwrap();
        client.advance_epoch(10);
        client.distribute_rewards(addr(2), 700_000).unwrap();

        let history = client.reward_history();
        assert_eq!(history.len(), 2);
        assert_eq!(history[0].amount, 450_000);
        assert_eq!(history[1].amount, 630_000);
    }

    #[test]
    fn test_undelegate_more_than_delegated() {
        let mut client = DelegationClient::new(addr(1), secret(), 100);
        client.set_balance(addr(1), 100_000_000);
        client.register_provider(addr(2), 1000).unwrap();
        client.delegate(addr(2), 5_000_000).unwrap();

        let err = client.undelegate(addr(2), 10_000_000).unwrap_err();
        assert_eq!(
            err,
            DelegationRpcError::ChainError(DelegationError::InsufficientDelegation)
        );
    }

    #[test]
    fn test_multiple_delegations_same_provider() {
        let mut client = DelegationClient::new(addr(1), secret(), 100);
        client.set_balance(addr(1), 100_000_000);
        client.register_provider(addr(2), 1000).unwrap();

        client.delegate(addr(2), 10_000_000).unwrap();
        client.delegate(addr(2), 20_000_000).unwrap();

        let pos = client.position(&addr(2)).unwrap();
        assert_eq!(pos.amount, 30_000_000);
        assert_eq!(client.balance(&addr(1)), 70_000_000);
    }

    #[test]
    fn test_unregistered_provider() {
        let mut client = DelegationClient::new(addr(1), secret(), 100);
        client.set_balance(addr(1), 100_000_000);

        let err = client.delegate(addr(99), 5_000_000).unwrap_err();
        assert_eq!(
            err,
            DelegationRpcError::ChainError(DelegationError::ProviderNotRegistered)
        );
    }
}
