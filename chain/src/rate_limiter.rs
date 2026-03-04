//! CHAIN-025: Rate limiter — per-address transaction throttling with adaptive limits.
//!
//! Enforces maximum transaction throughput per address to prevent spam and DoS.
//! Supports:
//! - Per-address sliding window rate limiting
//! - Adaptive limits based on stake (staked providers get higher limits)
//! - Burst allowance with token bucket algorithm
//! - Global rate limiting for network-wide protection
//! - Cooldown penalties for addresses that repeatedly hit limits
//! - Exemptions for system-critical transactions (checkpoints, dispute moves)

use crate::types::{Address, Epoch};
use crate::mempool::TxKind;
use std::collections::HashMap;

/// Rate limiter configuration.
#[derive(Debug, Clone)]
pub struct RateLimitConfig {
    /// Base transactions per window for unstaked addresses.
    pub base_rate: u64,
    /// Window size in epochs.
    pub window_size: Epoch,
    /// Burst capacity (token bucket max tokens).
    pub burst_capacity: u64,
    /// Token refill rate per epoch.
    pub refill_rate: u64,
    /// Multiplier for staked addresses (e.g. 5 = 5x base_rate).
    pub stake_multiplier: u64,
    /// Stake threshold to qualify for elevated limits (in smallest units).
    pub stake_threshold: u128,
    /// Global max transactions per epoch across all addresses.
    pub global_max_per_epoch: u64,
    /// Cooldown penalty epochs added when limit is hit.
    pub cooldown_epochs: Epoch,
    /// Max cooldown penalty that can accumulate.
    pub max_cooldown: Epoch,
    /// Transaction kinds exempt from rate limiting.
    pub exempt_kinds: Vec<TxKind>,
}

impl Default for RateLimitConfig {
    fn default() -> Self {
        Self {
            base_rate: 10,
            window_size: 100,
            burst_capacity: 20,
            refill_rate: 2,
            stake_multiplier: 5,
            stake_threshold: 1_000_000,
            global_max_per_epoch: 10_000,
            cooldown_epochs: 10,
            max_cooldown: 100,
            exempt_kinds: vec![TxKind::BisectionMove, TxKind::PdpProof],
        }
    }
}

/// Per-address rate state.
#[derive(Debug, Clone)]
struct AddressState {
    /// Timestamps (epochs) of recent transactions in the window.
    window_txs: Vec<Epoch>,
    /// Token bucket: current tokens available.
    tokens: u64,
    /// Last epoch tokens were refilled.
    last_refill: Epoch,
    /// Accumulated cooldown penalty epochs.
    cooldown_until: Epoch,
    /// Number of times this address hit the rate limit.
    violations: u64,
}

impl AddressState {
    fn new(burst_capacity: u64, epoch: Epoch) -> Self {
        Self {
            window_txs: Vec::new(),
            tokens: burst_capacity,
            last_refill: epoch,
            cooldown_until: 0,
            violations: 0,
        }
    }
}

/// Result of a rate limit check.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RateLimitResult {
    /// Transaction is allowed.
    Allowed,
    /// Denied: window rate exceeded. Contains remaining cooldown epochs.
    WindowExceeded { cooldown_remaining: Epoch },
    /// Denied: burst tokens exhausted. Contains epochs until next token.
    BurstExhausted { refill_in: Epoch },
    /// Denied: in cooldown from previous violations.
    InCooldown { remaining: Epoch },
    /// Denied: global epoch limit reached.
    GlobalLimitReached,
}

/// Rate limiter tracking per-address and global transaction rates.
#[derive(Debug)]
pub struct RateLimiter {
    config: RateLimitConfig,
    states: HashMap<Address, AddressState>,
    /// Staked addresses and their stake amounts.
    stakes: HashMap<Address, u128>,
    /// Global tx count per epoch.
    global_counts: HashMap<Epoch, u64>,
}

impl RateLimiter {
    /// Create a new rate limiter with the given configuration.
    pub fn new(config: RateLimitConfig) -> Self {
        Self {
            config,
            states: HashMap::new(),
            stakes: HashMap::new(),
            global_counts: HashMap::new(),
        }
    }

    /// Register an address's stake for elevated rate limits.
    pub fn set_stake(&mut self, addr: Address, amount: u128) {
        self.stakes.insert(addr, amount);
    }

    /// Get the effective rate limit for an address.
    pub fn effective_rate(&self, addr: &Address) -> u64 {
        let base = self.config.base_rate;
        match self.stakes.get(addr) {
            Some(&stake) if stake >= self.config.stake_threshold => {
                base.saturating_mul(self.config.stake_multiplier)
            }
            _ => base,
        }
    }

    /// Get the effective burst capacity for an address.
    pub fn effective_burst(&self, addr: &Address) -> u64 {
        let base = self.config.burst_capacity;
        match self.stakes.get(addr) {
            Some(&stake) if stake >= self.config.stake_threshold => {
                base.saturating_mul(self.config.stake_multiplier)
            }
            _ => base,
        }
    }

    /// Refill tokens for an address based on elapsed epochs.
    fn refill_tokens(&mut self, addr: &Address, current_epoch: Epoch) {
        let max_burst = self.effective_burst(addr);
        let state = match self.states.get_mut(addr) {
            Some(s) => s,
            None => return,
        };
        let elapsed = current_epoch.saturating_sub(state.last_refill);
        if elapsed > 0 {
            let refill = elapsed.saturating_mul(self.config.refill_rate);
            state.tokens = (state.tokens + refill).min(max_burst);
            state.last_refill = current_epoch;
        }
    }

    /// Check if a transaction is allowed without consuming quota.
    pub fn check(
        &mut self,
        addr: &Address,
        kind: TxKind,
        current_epoch: Epoch,
    ) -> RateLimitResult {
        // Exempt transaction kinds bypass all limits.
        if self.config.exempt_kinds.contains(&kind) {
            return RateLimitResult::Allowed;
        }

        // Check global limit.
        let global_count = self.global_counts.get(&current_epoch).copied().unwrap_or(0);
        if global_count >= self.config.global_max_per_epoch {
            return RateLimitResult::GlobalLimitReached;
        }

        // Ensure state exists.
        if !self.states.contains_key(addr) {
            self.states.insert(
                addr.clone(),
                AddressState::new(self.effective_burst(addr), current_epoch),
            );
        }

        // Refill tokens.
        self.refill_tokens(addr, current_epoch);

        let state = self.states.get(addr).unwrap();

        // Check cooldown.
        if current_epoch < state.cooldown_until {
            return RateLimitResult::InCooldown {
                remaining: state.cooldown_until - current_epoch,
            };
        }

        // Check burst tokens.
        if state.tokens == 0 {
            let refill_in = if self.config.refill_rate > 0 { 1 } else { u64::MAX };
            return RateLimitResult::BurstExhausted { refill_in };
        }

        // Check sliding window.
        let window_start = current_epoch.saturating_sub(self.config.window_size);
        let window_count = state.window_txs.iter().filter(|&&e| e > window_start).count() as u64;
        let rate = self.effective_rate(addr);
        if window_count >= rate {
            return RateLimitResult::WindowExceeded {
                cooldown_remaining: self.config.cooldown_epochs,
            };
        }

        RateLimitResult::Allowed
    }

    /// Record a transaction, consuming quota. Returns the check result.
    /// Only consumes quota if Allowed.
    pub fn record(
        &mut self,
        addr: &Address,
        kind: TxKind,
        current_epoch: Epoch,
    ) -> RateLimitResult {
        let result = self.check(addr, kind, current_epoch);
        match &result {
            RateLimitResult::Allowed => {
                // Exempt kinds don't consume quota.
                if self.config.exempt_kinds.contains(&kind) {
                    return result;
                }

                let state = self.states.get_mut(addr).unwrap();
                state.window_txs.push(current_epoch);
                state.tokens = state.tokens.saturating_sub(1);

                // Increment global count.
                *self.global_counts.entry(current_epoch).or_insert(0) += 1;
            }
            RateLimitResult::WindowExceeded { .. } | RateLimitResult::BurstExhausted { .. } => {
                // Apply cooldown penalty.
                let state = self.states.get_mut(addr).unwrap();
                state.violations += 1;
                let penalty = self
                    .config
                    .cooldown_epochs
                    .min(self.config.max_cooldown.saturating_sub(
                        state.cooldown_until.saturating_sub(current_epoch),
                    ));
                state.cooldown_until = current_epoch + penalty;
            }
            _ => {}
        }
        result
    }

    /// Prune old window data and global counts to free memory.
    pub fn prune(&mut self, current_epoch: Epoch) {
        let window_start = current_epoch.saturating_sub(self.config.window_size);
        for state in self.states.values_mut() {
            state.window_txs.retain(|&e| e > window_start);
        }
        // Remove states with no activity and no cooldown.
        self.states.retain(|_, s| {
            !s.window_txs.is_empty() || s.cooldown_until > current_epoch
        });
        // Remove old global counts.
        self.global_counts.retain(|&epoch, _| epoch >= window_start);
    }

    /// Get violation count for an address.
    pub fn violations(&self, addr: &Address) -> u64 {
        self.states.get(addr).map_or(0, |s| s.violations)
    }

    /// Get remaining cooldown for an address.
    pub fn cooldown_remaining(&self, addr: &Address, current_epoch: Epoch) -> Epoch {
        self.states
            .get(addr)
            .map_or(0, |s| s.cooldown_until.saturating_sub(current_epoch))
    }

    /// Get current token count for an address.
    pub fn tokens_available(&mut self, addr: &Address, current_epoch: Epoch) -> u64 {
        self.refill_tokens(addr, current_epoch);
        self.states.get(addr).map_or(self.effective_burst(addr), |s| s.tokens)
    }

    /// Reset an address's rate limit state (admin action).
    pub fn reset(&mut self, addr: &Address) {
        self.states.remove(addr);
    }

    /// Get the current global tx count for an epoch.
    pub fn global_count(&self, epoch: Epoch) -> u64 {
        self.global_counts.get(&epoch).copied().unwrap_or(0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn addr(n: u8) -> Address {
        let mut a = [0u8; 20];
        a[0] = n;
        Address(a)
    }

    fn default_limiter() -> RateLimiter {
        RateLimiter::new(RateLimitConfig::default())
    }

    #[test]
    fn test_basic_allow() {
        let mut rl = default_limiter();
        let a = addr(1);
        let result = rl.record(&a, TxKind::Transfer, 1);
        assert_eq!(result, RateLimitResult::Allowed);
    }

    #[test]
    fn test_window_rate_exceeded() {
        let mut rl = RateLimiter::new(RateLimitConfig {
            base_rate: 3,
            window_size: 10,
            burst_capacity: 100, // high burst so only window triggers
            ..Default::default()
        });
        let a = addr(1);
        for i in 0..3 {
            assert_eq!(rl.record(&a, TxKind::Transfer, i + 1), RateLimitResult::Allowed);
        }
        match rl.record(&a, TxKind::Transfer, 4) {
            RateLimitResult::WindowExceeded { .. } => {}
            other => panic!("Expected WindowExceeded, got {:?}", other),
        }
    }

    #[test]
    fn test_burst_exhaustion() {
        let mut rl = RateLimiter::new(RateLimitConfig {
            base_rate: 100, // high window so only burst triggers
            burst_capacity: 2,
            refill_rate: 0,
            ..Default::default()
        });
        let a = addr(1);
        assert_eq!(rl.record(&a, TxKind::Transfer, 1), RateLimitResult::Allowed);
        assert_eq!(rl.record(&a, TxKind::Transfer, 1), RateLimitResult::Allowed);
        match rl.record(&a, TxKind::Transfer, 1) {
            RateLimitResult::BurstExhausted { .. } => {}
            other => panic!("Expected BurstExhausted, got {:?}", other),
        }
    }

    #[test]
    fn test_token_refill() {
        let mut rl = RateLimiter::new(RateLimitConfig {
            base_rate: 100,
            burst_capacity: 2,
            refill_rate: 1,
            ..Default::default()
        });
        let a = addr(1);
        // Exhaust tokens
        rl.record(&a, TxKind::Transfer, 1);
        rl.record(&a, TxKind::Transfer, 1);
        // After 1 epoch, 1 token refilled
        assert_eq!(rl.tokens_available(&a, 2), 1);
        assert_eq!(rl.record(&a, TxKind::Transfer, 2), RateLimitResult::Allowed);
    }

    #[test]
    fn test_staked_address_higher_rate() {
        let mut rl = RateLimiter::new(RateLimitConfig {
            base_rate: 2,
            stake_multiplier: 3,
            stake_threshold: 100,
            burst_capacity: 100,
            ..Default::default()
        });
        let a = addr(1);
        rl.set_stake(a, 100);
        assert_eq!(rl.effective_rate(&a), 6); // 2 * 3
        // Can do 6 txs in window
        for i in 0..6 {
            assert_eq!(rl.record(&a, TxKind::Transfer, i + 1), RateLimitResult::Allowed);
        }
        match rl.record(&a, TxKind::Transfer, 7) {
            RateLimitResult::WindowExceeded { .. } => {}
            other => panic!("Expected WindowExceeded, got {:?}", other),
        }
    }

    #[test]
    fn test_below_stake_threshold_gets_base() {
        let mut rl = default_limiter();
        let a = addr(1);
        rl.set_stake(a, 999_999); // Below 1M threshold
        assert_eq!(rl.effective_rate(&a), 10); // base
    }

    #[test]
    fn test_exempt_tx_kinds() {
        let mut rl = RateLimiter::new(RateLimitConfig {
            base_rate: 1,
            burst_capacity: 1,
            ..Default::default()
        });
        let a = addr(1);
        // Exhaust normal quota
        rl.record(&a, TxKind::Transfer, 1);
        // BisectionMove is exempt
        assert_eq!(
            rl.record(&a, TxKind::BisectionMove, 1),
            RateLimitResult::Allowed
        );
        // PdpProof is exempt
        assert_eq!(
            rl.record(&a, TxKind::PdpProof, 1),
            RateLimitResult::Allowed
        );
    }

    #[test]
    fn test_global_limit() {
        let mut rl = RateLimiter::new(RateLimitConfig {
            global_max_per_epoch: 2,
            burst_capacity: 100,
            base_rate: 100,
            ..Default::default()
        });
        rl.record(&addr(1), TxKind::Transfer, 1);
        rl.record(&addr(2), TxKind::Transfer, 1);
        assert_eq!(
            rl.check(&addr(3), TxKind::Transfer, 1),
            RateLimitResult::GlobalLimitReached
        );
    }

    #[test]
    fn test_cooldown_applied_on_violation() {
        let mut rl = RateLimiter::new(RateLimitConfig {
            base_rate: 1,
            burst_capacity: 100,
            cooldown_epochs: 5,
            ..Default::default()
        });
        let a = addr(1);
        rl.record(&a, TxKind::Transfer, 1);
        // Trigger violation
        rl.record(&a, TxKind::Transfer, 2);
        // Should be in cooldown
        assert_eq!(rl.violations(&a), 1);
        match rl.check(&a, TxKind::Transfer, 3) {
            RateLimitResult::InCooldown { remaining } => assert!(remaining > 0),
            other => panic!("Expected InCooldown, got {:?}", other),
        }
    }

    #[test]
    fn test_cooldown_expires() {
        let mut rl = RateLimiter::new(RateLimitConfig {
            base_rate: 1,
            window_size: 5,
            burst_capacity: 100,
            cooldown_epochs: 3,
            refill_rate: 10,
            ..Default::default()
        });
        let a = addr(1);
        rl.record(&a, TxKind::Transfer, 1);
        rl.record(&a, TxKind::Transfer, 2); // violation, cooldown until epoch 5
        // After cooldown + window expires
        assert_eq!(rl.record(&a, TxKind::Transfer, 10), RateLimitResult::Allowed);
    }

    #[test]
    fn test_prune_old_data() {
        let mut rl = RateLimiter::new(RateLimitConfig {
            window_size: 5,
            burst_capacity: 100,
            base_rate: 100,
            ..Default::default()
        });
        let a = addr(1);
        rl.record(&a, TxKind::Transfer, 1);
        rl.record(&a, TxKind::Transfer, 2);
        rl.prune(100); // Far future
        assert_eq!(rl.global_count(1), 0); // pruned
    }

    #[test]
    fn test_reset_clears_state() {
        let mut rl = default_limiter();
        let a = addr(1);
        rl.record(&a, TxKind::Transfer, 1);
        rl.reset(&a);
        assert_eq!(rl.violations(&a), 0);
        assert_eq!(rl.cooldown_remaining(&a, 1), 0);
    }

    #[test]
    fn test_sliding_window_expiry() {
        let mut rl = RateLimiter::new(RateLimitConfig {
            base_rate: 2,
            window_size: 5,
            burst_capacity: 100,
            refill_rate: 10,
            cooldown_epochs: 3,
            ..Default::default()
        });
        let a = addr(1);
        rl.record(&a, TxKind::Transfer, 1);
        rl.record(&a, TxKind::Transfer, 2);
        // Window full at epoch 3 (txs at 1,2 within window of 5)
        match rl.record(&a, TxKind::Transfer, 3) {
            RateLimitResult::WindowExceeded { .. } => {}
            other => panic!("Expected WindowExceeded, got {:?}", other),
        }
        // Cooldown until epoch 6. At epoch 10, cooldown expired AND old txs outside window.
        assert_eq!(rl.record(&a, TxKind::Transfer, 10), RateLimitResult::Allowed);
    }

    #[test]
    fn test_multiple_addresses_independent() {
        let mut rl = RateLimiter::new(RateLimitConfig {
            base_rate: 1,
            burst_capacity: 100,
            ..Default::default()
        });
        let a1 = addr(1);
        let a2 = addr(2);
        rl.record(&a1, TxKind::Transfer, 1);
        // a1 exhausted, a2 still fine
        assert_eq!(rl.record(&a2, TxKind::Transfer, 1), RateLimitResult::Allowed);
    }

    #[test]
    fn test_burst_cap_respects_stake() {
        let mut rl = RateLimiter::new(RateLimitConfig {
            burst_capacity: 3,
            stake_multiplier: 2,
            stake_threshold: 50,
            ..Default::default()
        });
        let a = addr(1);
        rl.set_stake(a, 50);
        assert_eq!(rl.effective_burst(&a), 6); // 3 * 2
    }
}
