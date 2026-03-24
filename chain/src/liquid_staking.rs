//! Liquid Staking Tokens (CHAIN-032)
//!
//! Staking derivatives that represent delegated stake. When a user delegates
//! to a provider, they receive stPROVA tokens at the current exchange rate.
//! When they undelegate, stPROVA is burned and PROVA is returned after unbonding.
//!
//! Exchange rate: stPROVA/PROVA = total_staked / total_st_supply
//! As rewards accrue, the exchange rate increases (stPROVA appreciates).
//!
//! Key features:
//! - Mint stPROVA on delegate, burn on undelegate
//! - Exchange rate reflects accrued rewards (rebasing)
//! - Per-provider staking derivative (stPROVA-<provider_short>)
//! - Transferable: holders can sell staked position without unbonding
//! - Slash propagation reduces exchange rate (all holders share loss)

use std::collections::HashMap;

pub type Amount = u64;
pub type Address = [u8; 32];
pub type Epoch = u64;

/// Minimum mint amount to prevent dust.
pub const MIN_MINT_AMOUNT: Amount = 1_000; // 0.001 token

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LiquidStakingError {
    PoolNotFound,
    InsufficientBalance,
    BelowMinimum,
    ZeroAmount,
    ExchangeRateZero,
    MintOverflow,
    BurnExceedsSupply,
    TransferToSelf,
    PoolAlreadyExists,
    SlashExceedsPool,
}

impl std::fmt::Display for LiquidStakingError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::PoolNotFound => write!(f, "staking pool not found"),
            Self::InsufficientBalance => write!(f, "insufficient stPROVA balance"),
            Self::BelowMinimum => write!(f, "amount below minimum"),
            Self::ZeroAmount => write!(f, "amount must be non-zero"),
            Self::ExchangeRateZero => write!(f, "exchange rate is zero (pool drained)"),
            Self::MintOverflow => write!(f, "mint would overflow supply"),
            Self::BurnExceedsSupply => write!(f, "burn exceeds total supply"),
            Self::TransferToSelf => write!(f, "cannot transfer to self"),
            Self::PoolAlreadyExists => write!(f, "pool already exists for provider"),
            Self::SlashExceedsPool => write!(f, "slash amount exceeds pool backing"),
        }
    }
}

/// A liquid staking pool for a single provider.
#[derive(Debug, Clone)]
pub struct StakingPool {
    /// Provider this pool delegates to.
    pub provider: Address,
    /// Total underlying PROVA staked (increases with rewards, decreases with slashing).
    pub total_staked: Amount,
    /// Total stPROVA tokens in circulation for this pool.
    pub total_st_supply: Amount,
    /// Per-holder stPROVA balances.
    pub balances: HashMap<Address, Amount>,
    /// Creation epoch.
    pub created_epoch: Epoch,
    /// Cumulative rewards distributed (for accounting).
    pub cumulative_rewards: Amount,
    /// Cumulative slashed amount (for accounting).
    pub cumulative_slashed: Amount,
}

impl StakingPool {
    pub fn new(provider: Address, epoch: Epoch) -> Self {
        Self {
            provider,
            total_staked: 0,
            total_st_supply: 0,
            balances: HashMap::new(),
            created_epoch: epoch,
            cumulative_rewards: 0,
            cumulative_slashed: 0,
        }
    }

    /// Exchange rate as (numerator, denominator) to avoid floats.
    /// Returns PROVA per stPROVA scaled by 1e9.
    pub fn exchange_rate_scaled(&self) -> u128 {
        if self.total_st_supply == 0 {
            1_000_000_000 // 1:1 for empty pool
        } else {
            (self.total_staked as u128) * 1_000_000_000 / (self.total_st_supply as u128)
        }
    }

    /// How many stPROVA tokens to mint for a given PROVA deposit.
    pub fn prova_to_st(&self, prova_amount: Amount) -> Amount {
        if self.total_staked == 0 || self.total_st_supply == 0 {
            prova_amount // 1:1 initial rate
        } else {
            ((prova_amount as u128) * (self.total_st_supply as u128) / (self.total_staked as u128))
                as Amount
        }
    }

    /// How many PROVA tokens are redeemable for a given stPROVA amount.
    pub fn st_to_prova(&self, st_amount: Amount) -> Amount {
        if self.total_st_supply == 0 {
            return 0;
        }
        ((st_amount as u128) * (self.total_staked as u128) / (self.total_st_supply as u128))
            as Amount
    }
}

/// Mint/burn event for audit trail.
#[derive(Debug, Clone)]
pub struct StakingEvent {
    pub pool_provider: Address,
    pub holder: Address,
    pub kind: StakingEventKind,
    pub prova_amount: Amount,
    pub st_amount: Amount,
    pub exchange_rate_scaled: u128,
    pub epoch: Epoch,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StakingEventKind {
    Mint,
    Burn,
    Transfer,
    RewardAccrual,
    SlashApplied,
}

/// The liquid staking registry — manages all staking pools.
#[derive(Debug)]
pub struct LiquidStakingRegistry {
    /// provider → StakingPool
    pub pools: HashMap<Address, StakingPool>,
    pub events: Vec<StakingEvent>,
    pub current_epoch: Epoch,
}

impl LiquidStakingRegistry {
    pub fn new() -> Self {
        Self {
            pools: HashMap::new(),
            events: Vec::new(),
            current_epoch: 0,
        }
    }

    /// Create a staking pool for a provider.
    pub fn create_pool(&mut self, provider: Address) -> Result<(), LiquidStakingError> {
        if self.pools.contains_key(&provider) {
            return Err(LiquidStakingError::PoolAlreadyExists);
        }
        self.pools
            .insert(provider, StakingPool::new(provider, self.current_epoch));
        Ok(())
    }

    /// Mint stPROVA in exchange for PROVA delegation.
    /// Returns the number of stPROVA minted.
    pub fn mint(
        &mut self,
        provider: Address,
        delegator: Address,
        prova_amount: Amount,
    ) -> Result<Amount, LiquidStakingError> {
        if prova_amount == 0 {
            return Err(LiquidStakingError::ZeroAmount);
        }

        let pool = self
            .pools
            .get_mut(&provider)
            .ok_or(LiquidStakingError::PoolNotFound)?;

        let st_amount = pool.prova_to_st(prova_amount);
        if st_amount < MIN_MINT_AMOUNT && pool.total_st_supply > 0 {
            return Err(LiquidStakingError::BelowMinimum);
        }

        // Check overflow
        let new_supply = pool
            .total_st_supply
            .checked_add(st_amount)
            .ok_or(LiquidStakingError::MintOverflow)?;

        pool.total_staked += prova_amount;
        pool.total_st_supply = new_supply;
        *pool.balances.entry(delegator).or_insert(0) += st_amount;

        self.events.push(StakingEvent {
            pool_provider: provider,
            holder: delegator,
            kind: StakingEventKind::Mint,
            prova_amount,
            st_amount,
            exchange_rate_scaled: pool.exchange_rate_scaled(),
            epoch: self.current_epoch,
        });

        Ok(st_amount)
    }

    /// Burn stPROVA to initiate undelegation. Returns PROVA amount to be unbonded.
    pub fn burn(
        &mut self,
        provider: Address,
        holder: Address,
        st_amount: Amount,
    ) -> Result<Amount, LiquidStakingError> {
        if st_amount == 0 {
            return Err(LiquidStakingError::ZeroAmount);
        }

        let pool = self
            .pools
            .get_mut(&provider)
            .ok_or(LiquidStakingError::PoolNotFound)?;

        let balance = pool.balances.get(&holder).copied().unwrap_or(0);
        if balance < st_amount {
            return Err(LiquidStakingError::InsufficientBalance);
        }
        if st_amount > pool.total_st_supply {
            return Err(LiquidStakingError::BurnExceedsSupply);
        }

        let prova_amount = pool.st_to_prova(st_amount);

        pool.total_staked = pool.total_staked.saturating_sub(prova_amount);
        pool.total_st_supply -= st_amount;

        let bal = pool.balances.get_mut(&holder).unwrap();
        *bal -= st_amount;
        if *bal == 0 {
            pool.balances.remove(&holder);
        }

        self.events.push(StakingEvent {
            pool_provider: provider,
            holder,
            kind: StakingEventKind::Burn,
            prova_amount,
            st_amount,
            exchange_rate_scaled: pool.exchange_rate_scaled(),
            epoch: self.current_epoch,
        });

        Ok(prova_amount)
    }

    /// Transfer stPROVA between holders (sell staked position).
    pub fn transfer(
        &mut self,
        provider: Address,
        from: Address,
        to: Address,
        st_amount: Amount,
    ) -> Result<(), LiquidStakingError> {
        if st_amount == 0 {
            return Err(LiquidStakingError::ZeroAmount);
        }
        if from == to {
            return Err(LiquidStakingError::TransferToSelf);
        }

        let pool = self
            .pools
            .get_mut(&provider)
            .ok_or(LiquidStakingError::PoolNotFound)?;

        let from_bal = pool.balances.get(&from).copied().unwrap_or(0);
        if from_bal < st_amount {
            return Err(LiquidStakingError::InsufficientBalance);
        }

        let f = pool.balances.get_mut(&from).unwrap();
        *f -= st_amount;
        if *f == 0 {
            pool.balances.remove(&from);
        }
        *pool.balances.entry(to).or_insert(0) += st_amount;

        let prova_equiv = pool.st_to_prova(st_amount);
        self.events.push(StakingEvent {
            pool_provider: provider,
            holder: from,
            kind: StakingEventKind::Transfer,
            prova_amount: prova_equiv,
            st_amount,
            exchange_rate_scaled: pool.exchange_rate_scaled(),
            epoch: self.current_epoch,
        });

        Ok(())
    }

    /// Accrue rewards to a pool (increases exchange rate for all holders).
    pub fn accrue_rewards(
        &mut self,
        provider: Address,
        reward_amount: Amount,
    ) -> Result<(), LiquidStakingError> {
        if reward_amount == 0 {
            return Ok(());
        }
        let pool = self
            .pools
            .get_mut(&provider)
            .ok_or(LiquidStakingError::PoolNotFound)?;

        pool.total_staked += reward_amount;
        pool.cumulative_rewards += reward_amount;

        self.events.push(StakingEvent {
            pool_provider: provider,
            holder: [0u8; 32], // system
            kind: StakingEventKind::RewardAccrual,
            prova_amount: reward_amount,
            st_amount: 0,
            exchange_rate_scaled: pool.exchange_rate_scaled(),
            epoch: self.current_epoch,
        });

        Ok(())
    }

    /// Apply slashing — reduces total_staked, lowering exchange rate for all holders.
    pub fn apply_slash(
        &mut self,
        provider: Address,
        slash_amount: Amount,
    ) -> Result<(), LiquidStakingError> {
        if slash_amount == 0 {
            return Ok(());
        }
        let pool = self
            .pools
            .get_mut(&provider)
            .ok_or(LiquidStakingError::PoolNotFound)?;

        if slash_amount > pool.total_staked {
            return Err(LiquidStakingError::SlashExceedsPool);
        }

        pool.total_staked -= slash_amount;
        pool.cumulative_slashed += slash_amount;

        self.events.push(StakingEvent {
            pool_provider: provider,
            holder: [0u8; 32],
            kind: StakingEventKind::SlashApplied,
            prova_amount: slash_amount,
            st_amount: 0,
            exchange_rate_scaled: pool.exchange_rate_scaled(),
            epoch: self.current_epoch,
        });

        Ok(())
    }

    /// Get a holder's stPROVA balance and its current PROVA value.
    pub fn balance_of(
        &self,
        provider: &Address,
        holder: &Address,
    ) -> Result<(Amount, Amount), LiquidStakingError> {
        let pool = self
            .pools
            .get(provider)
            .ok_or(LiquidStakingError::PoolNotFound)?;
        let st_bal = pool.balances.get(holder).copied().unwrap_or(0);
        let prova_val = pool.st_to_prova(st_bal);
        Ok((st_bal, prova_val))
    }

    pub fn advance_epoch(&mut self) {
        self.current_epoch += 1;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn addr(b: u8) -> Address {
        let mut a = [0u8; 32];
        a[0] = b;
        a
    }

    #[test]
    fn test_create_pool() {
        let mut reg = LiquidStakingRegistry::new();
        assert!(reg.create_pool(addr(1)).is_ok());
        assert_eq!(
            reg.create_pool(addr(1)),
            Err(LiquidStakingError::PoolAlreadyExists)
        );
    }

    #[test]
    fn test_mint_initial_1_to_1() {
        let mut reg = LiquidStakingRegistry::new();
        reg.create_pool(addr(1)).unwrap();
        let st = reg.mint(addr(1), addr(10), 1_000_000).unwrap();
        assert_eq!(st, 1_000_000); // 1:1 initial
        let (st_bal, prova_val) = reg.balance_of(&addr(1), &addr(10)).unwrap();
        assert_eq!(st_bal, 1_000_000);
        assert_eq!(prova_val, 1_000_000);
    }

    #[test]
    fn test_mint_zero_rejected() {
        let mut reg = LiquidStakingRegistry::new();
        reg.create_pool(addr(1)).unwrap();
        assert_eq!(
            reg.mint(addr(1), addr(10), 0),
            Err(LiquidStakingError::ZeroAmount)
        );
    }

    #[test]
    fn test_mint_nonexistent_pool() {
        let mut reg = LiquidStakingRegistry::new();
        assert_eq!(
            reg.mint(addr(99), addr(10), 1000),
            Err(LiquidStakingError::PoolNotFound)
        );
    }

    #[test]
    fn test_rewards_increase_exchange_rate() {
        let mut reg = LiquidStakingRegistry::new();
        reg.create_pool(addr(1)).unwrap();
        reg.mint(addr(1), addr(10), 1_000_000).unwrap();

        // Accrue 100k rewards
        reg.accrue_rewards(addr(1), 100_000).unwrap();

        // stPROVA balance unchanged, but PROVA value increased
        let (st_bal, prova_val) = reg.balance_of(&addr(1), &addr(10)).unwrap();
        assert_eq!(st_bal, 1_000_000);
        assert_eq!(prova_val, 1_100_000);
    }

    #[test]
    fn test_second_minter_gets_fewer_st_tokens() {
        let mut reg = LiquidStakingRegistry::new();
        reg.create_pool(addr(1)).unwrap();
        reg.mint(addr(1), addr(10), 1_000_000).unwrap();
        reg.accrue_rewards(addr(1), 100_000).unwrap();

        // Second delegator deposits 1.1M PROVA — should get ~1M stPROVA
        let st = reg.mint(addr(1), addr(20), 1_100_000).unwrap();
        assert_eq!(st, 1_000_000);

        // Both have 1M stPROVA but pool now has 2.2M PROVA total
        let pool = reg.pools.get(&addr(1)).unwrap();
        assert_eq!(pool.total_staked, 2_200_000);
        assert_eq!(pool.total_st_supply, 2_000_000);
    }

    #[test]
    fn test_burn_returns_correct_prova() {
        let mut reg = LiquidStakingRegistry::new();
        reg.create_pool(addr(1)).unwrap();
        reg.mint(addr(1), addr(10), 1_000_000).unwrap();
        reg.accrue_rewards(addr(1), 200_000).unwrap();

        let prova = reg.burn(addr(1), addr(10), 500_000).unwrap();
        assert_eq!(prova, 600_000); // half of 1.2M

        let (st_bal, prova_val) = reg.balance_of(&addr(1), &addr(10)).unwrap();
        assert_eq!(st_bal, 500_000);
        assert_eq!(prova_val, 600_000);
    }

    #[test]
    fn test_burn_insufficient() {
        let mut reg = LiquidStakingRegistry::new();
        reg.create_pool(addr(1)).unwrap();
        reg.mint(addr(1), addr(10), 1_000_000).unwrap();
        assert_eq!(
            reg.burn(addr(1), addr(10), 2_000_000),
            Err(LiquidStakingError::InsufficientBalance)
        );
    }

    #[test]
    fn test_transfer() {
        let mut reg = LiquidStakingRegistry::new();
        reg.create_pool(addr(1)).unwrap();
        reg.mint(addr(1), addr(10), 1_000_000).unwrap();

        reg.transfer(addr(1), addr(10), addr(20), 400_000).unwrap();

        let (st10, _) = reg.balance_of(&addr(1), &addr(10)).unwrap();
        let (st20, _) = reg.balance_of(&addr(1), &addr(20)).unwrap();
        assert_eq!(st10, 600_000);
        assert_eq!(st20, 400_000);
    }

    #[test]
    fn test_transfer_to_self_rejected() {
        let mut reg = LiquidStakingRegistry::new();
        reg.create_pool(addr(1)).unwrap();
        reg.mint(addr(1), addr(10), 1_000_000).unwrap();
        assert_eq!(
            reg.transfer(addr(1), addr(10), addr(10), 100),
            Err(LiquidStakingError::TransferToSelf)
        );
    }

    #[test]
    fn test_slash_reduces_exchange_rate() {
        let mut reg = LiquidStakingRegistry::new();
        reg.create_pool(addr(1)).unwrap();
        reg.mint(addr(1), addr(10), 1_000_000).unwrap();
        reg.mint(addr(1), addr(20), 1_000_000).unwrap();

        // Slash 500k from 2M pool
        reg.apply_slash(addr(1), 500_000).unwrap();

        // Each holder's PROVA value drops proportionally
        let (_, prova10) = reg.balance_of(&addr(1), &addr(10)).unwrap();
        let (_, prova20) = reg.balance_of(&addr(1), &addr(20)).unwrap();
        assert_eq!(prova10, 750_000); // 1M * (1.5M/2M)
        assert_eq!(prova20, 750_000);
    }

    #[test]
    fn test_slash_exceeds_pool() {
        let mut reg = LiquidStakingRegistry::new();
        reg.create_pool(addr(1)).unwrap();
        reg.mint(addr(1), addr(10), 1_000_000).unwrap();
        assert_eq!(
            reg.apply_slash(addr(1), 2_000_000),
            Err(LiquidStakingError::SlashExceedsPool)
        );
    }

    #[test]
    fn test_full_lifecycle() {
        let mut reg = LiquidStakingRegistry::new();
        reg.create_pool(addr(1)).unwrap();

        // Alice delegates 10M
        let st_alice = reg.mint(addr(1), addr(10), 10_000_000).unwrap();
        assert_eq!(st_alice, 10_000_000);

        // Rewards accrue
        reg.advance_epoch();
        reg.accrue_rewards(addr(1), 1_000_000).unwrap();

        // Bob delegates 11M (should get 10M stPROVA at 1.1 rate)
        let st_bob = reg.mint(addr(1), addr(20), 11_000_000).unwrap();
        assert_eq!(st_bob, 10_000_000);

        // More rewards
        reg.accrue_rewards(addr(1), 2_000_000).unwrap();
        // Pool: 24M PROVA, 20M stPROVA, rate = 1.2

        // Alice sells half her position to Charlie
        reg.transfer(addr(1), addr(10), addr(30), 5_000_000)
            .unwrap();

        // Charlie redeems
        let prova_charlie = reg.burn(addr(1), addr(30), 5_000_000).unwrap();
        assert_eq!(prova_charlie, 6_000_000); // 5M * 1.2

        // Slash event
        reg.apply_slash(addr(1), 3_000_000).unwrap();
        // Pool: 15M PROVA, 15M stPROVA, rate = 1.0

        let (st_a, prova_a) = reg.balance_of(&addr(1), &addr(10)).unwrap();
        assert_eq!(st_a, 5_000_000);
        assert_eq!(prova_a, 5_000_000); // rate back to 1:1

        // Events recorded
        assert!(reg.events.len() >= 6);
    }

    #[test]
    fn test_burn_full_removes_from_map() {
        let mut reg = LiquidStakingRegistry::new();
        reg.create_pool(addr(1)).unwrap();
        reg.mint(addr(1), addr(10), 1_000_000).unwrap();
        reg.burn(addr(1), addr(10), 1_000_000).unwrap();

        let pool = reg.pools.get(&addr(1)).unwrap();
        assert!(!pool.balances.contains_key(&addr(10)));
        assert_eq!(pool.total_st_supply, 0);
    }

    #[test]
    fn test_event_trail() {
        let mut reg = LiquidStakingRegistry::new();
        reg.create_pool(addr(1)).unwrap();
        reg.mint(addr(1), addr(10), 1_000_000).unwrap();
        reg.accrue_rewards(addr(1), 50_000).unwrap();
        reg.burn(addr(1), addr(10), 500_000).unwrap();

        assert_eq!(reg.events.len(), 3);
        assert_eq!(reg.events[0].kind, StakingEventKind::Mint);
        assert_eq!(reg.events[1].kind, StakingEventKind::RewardAccrual);
        assert_eq!(reg.events[2].kind, StakingEventKind::Burn);
    }
}
