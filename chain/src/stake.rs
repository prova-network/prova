//! Stake Ledger + Slashing — manages provider stake deposits and penalties.
//!
//! Providers must stake tokens to participate in inference and storage.
//! Slashing occurs when:
//! - Provider loses a QBP dispute (incorrect inference)
//! - Provider misses PDP proofs (storage unavailable)
//! - Challenger loses a dispute (false accusation)

use std::collections::HashMap;
use crate::types::*;

/// Stake entry for a participant.
#[derive(Debug, Clone)]
pub struct StakeEntry {
    /// Total deposited stake.
    pub deposited: StakeAmount,
    /// Currently locked in active disputes.
    pub locked: StakeAmount,
    /// Cumulative amount slashed.
    pub slashed: StakeAmount,
    /// Epoch of last stake change.
    pub last_updated: Epoch,
    /// Cooldown: epoch when the participant can operate again after slashing.
    pub cooldown_until: Option<Epoch>,
}

impl StakeEntry {
    /// Available stake (deposited - locked - slashed).
    pub fn available(&self) -> StakeAmount {
        self.deposited.saturating_sub(self.locked).saturating_sub(self.slashed)
    }

    /// Whether the participant is in cooldown.
    pub fn in_cooldown(&self, current_epoch: Epoch) -> bool {
        self.cooldown_until.map_or(false, |until| current_epoch < until)
    }
}

/// Reason for slashing.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SlashReason {
    /// Lost a QBP dispute (provider was wrong).
    DisputeLost,
    /// False challenge (challenger was wrong).
    FalseChallenge,
    /// Missed PDP proof (first offense — warning only).
    PdpMissFirst,
    /// Missed PDP proof (second consecutive).
    PdpMissSecond,
    /// Missed PDP proof (third — permanent).
    PdpMissThird,
    /// Dispute lost AND data unavailable.
    DisputeLostDataUnavailable,
}

impl SlashReason {
    /// Slash percentage (basis points, 10000 = 100%).
    pub fn slash_bps(&self) -> u32 {
        match self {
            Self::DisputeLost => 1000,           // 10%
            Self::FalseChallenge => 500,         // 5%
            Self::PdpMissFirst => 0,             // Warning
            Self::PdpMissSecond => 500,          // 5%
            Self::PdpMissThird => 10000,         // 100%
            Self::DisputeLostDataUnavailable => 5000, // 50%
        }
    }

    /// Cooldown duration in epochs.
    pub fn cooldown_epochs(&self) -> Option<EpochDuration> {
        match self {
            Self::DisputeLost => Some(2880),          // ~24 hours
            Self::FalseChallenge => Some(2880),       // ~24 hours
            Self::PdpMissFirst => None,               // No cooldown
            Self::PdpMissSecond => Some(20160),       // ~7 days
            Self::PdpMissThird => Some(u64::MAX),     // Permanent
            Self::DisputeLostDataUnavailable => Some(86400), // ~30 days
        }
    }
}

/// The on-chain stake ledger.
#[derive(Debug)]
pub struct StakeLedger {
    stakes: HashMap<Address, StakeEntry>,
    /// Minimum stake required to operate as a provider.
    pub min_provider_stake: StakeAmount,
    /// Minimum stake required to challenge.
    pub min_challenger_stake: StakeAmount,
    /// Total amount slashed (goes to burn or reward pool).
    pub total_slashed: StakeAmount,
}

impl StakeLedger {
    pub fn new(min_provider: StakeAmount, min_challenger: StakeAmount) -> Self {
        Self {
            stakes: HashMap::new(),
            min_provider_stake: min_provider,
            min_challenger_stake: min_challenger,
            total_slashed: 0,
        }
    }

    /// Deposit stake for an address.
    pub fn deposit(&mut self, addr: Address, amount: StakeAmount, epoch: Epoch) {
        let entry = self.stakes.entry(addr).or_insert(StakeEntry {
            deposited: 0,
            locked: 0,
            slashed: 0,
            last_updated: epoch,
            cooldown_until: None,
        });
        entry.deposited += amount;
        entry.last_updated = epoch;
    }

    /// Withdraw available stake.
    pub fn withdraw(&mut self, addr: &Address, amount: StakeAmount, epoch: Epoch) -> Result<(), StakeError> {
        let entry = self.stakes.get_mut(addr).ok_or(StakeError::NotFound)?;
        
        if entry.in_cooldown(epoch) {
            return Err(StakeError::InCooldown(entry.cooldown_until.unwrap()));
        }

        if entry.available() < amount {
            return Err(StakeError::InsufficientStake {
                available: entry.available(),
                requested: amount,
            });
        }

        entry.deposited -= amount;
        entry.last_updated = epoch;
        Ok(())
    }

    /// Lock stake for an active dispute.
    pub fn lock(&mut self, addr: &Address, amount: StakeAmount) -> Result<(), StakeError> {
        let entry = self.stakes.get_mut(addr).ok_or(StakeError::NotFound)?;
        
        if entry.available() < amount {
            return Err(StakeError::InsufficientStake {
                available: entry.available(),
                requested: amount,
            });
        }

        entry.locked += amount;
        Ok(())
    }

    /// Unlock stake after dispute resolution.
    pub fn unlock(&mut self, addr: &Address, amount: StakeAmount) -> Result<(), StakeError> {
        let entry = self.stakes.get_mut(addr).ok_or(StakeError::NotFound)?;
        entry.locked = entry.locked.saturating_sub(amount);
        Ok(())
    }

    /// Slash a participant.
    pub fn slash(
        &mut self,
        addr: &Address,
        reason: SlashReason,
        epoch: Epoch,
    ) -> Result<StakeAmount, StakeError> {
        let entry = self.stakes.get_mut(addr).ok_or(StakeError::NotFound)?;

        let slash_amount = (entry.deposited as u128 * reason.slash_bps() as u128 / 10000) as StakeAmount;

        entry.slashed += slash_amount;
        entry.last_updated = epoch;
        self.total_slashed += slash_amount;

        if let Some(cooldown) = reason.cooldown_epochs() {
            entry.cooldown_until = Some(epoch.saturating_add(cooldown));
        }

        Ok(slash_amount)
    }

    /// Check if an address can operate as a provider.
    pub fn can_provide(&self, addr: &Address, epoch: Epoch) -> bool {
        self.stakes.get(addr).map_or(false, |e| {
            e.available() >= self.min_provider_stake && !e.in_cooldown(epoch)
        })
    }

    /// Check if an address can challenge.
    pub fn can_challenge(&self, addr: &Address, epoch: Epoch) -> bool {
        self.stakes.get(addr).map_or(false, |e| {
            e.available() >= self.min_challenger_stake && !e.in_cooldown(epoch)
        })
    }

    /// Get stake entry for an address.
    pub fn get(&self, addr: &Address) -> Option<&StakeEntry> {
        self.stakes.get(addr)
    }

    /// Total number of stakers.
    pub fn staker_count(&self) -> usize {
        self.stakes.len()
    }
}

#[derive(Debug)]
pub enum StakeError {
    NotFound,
    InsufficientStake { available: StakeAmount, requested: StakeAmount },
    InCooldown(Epoch),
}

impl std::fmt::Display for StakeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NotFound => write!(f, "no stake entry found"),
            Self::InsufficientStake { available, requested } => {
                write!(f, "insufficient stake: {available} available, {requested} requested")
            }
            Self::InCooldown(until) => write!(f, "in cooldown until epoch {until}"),
        }
    }
}

impl std::error::Error for StakeError {}

#[cfg(test)]
mod tests {
    use super::*;

    fn setup() -> StakeLedger {
        StakeLedger::new(1_000_000, 500_000)
    }

    #[test]
    fn test_deposit_and_available() {
        let mut ledger = setup();
        let addr = Address::test(1);
        ledger.deposit(addr, 5_000_000, 100);
        
        let entry = ledger.get(&addr).unwrap();
        assert_eq!(entry.deposited, 5_000_000);
        assert_eq!(entry.available(), 5_000_000);
    }

    #[test]
    fn test_lock_and_unlock() {
        let mut ledger = setup();
        let addr = Address::test(1);
        ledger.deposit(addr, 5_000_000, 100);

        ledger.lock(&addr, 2_000_000).unwrap();
        assert_eq!(ledger.get(&addr).unwrap().available(), 3_000_000);

        ledger.unlock(&addr, 2_000_000).unwrap();
        assert_eq!(ledger.get(&addr).unwrap().available(), 5_000_000);
    }

    #[test]
    fn test_lock_insufficient() {
        let mut ledger = setup();
        let addr = Address::test(1);
        ledger.deposit(addr, 1_000_000, 100);

        assert!(ledger.lock(&addr, 2_000_000).is_err());
    }

    #[test]
    fn test_slash_dispute_lost() {
        let mut ledger = setup();
        let addr = Address::test(1);
        ledger.deposit(addr, 10_000_000, 100);

        let slashed = ledger.slash(&addr, SlashReason::DisputeLost, 200).unwrap();
        assert_eq!(slashed, 1_000_000); // 10% of 10M
        assert_eq!(ledger.get(&addr).unwrap().available(), 9_000_000);
        assert_eq!(ledger.total_slashed, 1_000_000);
    }

    #[test]
    fn test_slash_cooldown() {
        let mut ledger = setup();
        let addr = Address::test(1);
        ledger.deposit(addr, 10_000_000, 100);

        ledger.slash(&addr, SlashReason::DisputeLost, 200).unwrap();

        // In cooldown
        assert!(ledger.get(&addr).unwrap().in_cooldown(201));
        assert!(!ledger.can_provide(&addr, 201));

        // After cooldown (200 + 2880 = 3080)
        assert!(!ledger.get(&addr).unwrap().in_cooldown(3081));
        assert!(ledger.can_provide(&addr, 3081));
    }

    #[test]
    fn test_pdp_miss_warning() {
        let mut ledger = setup();
        let addr = Address::test(1);
        ledger.deposit(addr, 10_000_000, 100);

        // First miss is a warning — no slash, no cooldown
        let slashed = ledger.slash(&addr, SlashReason::PdpMissFirst, 200).unwrap();
        assert_eq!(slashed, 0);
        assert!(!ledger.get(&addr).unwrap().in_cooldown(201));
    }

    #[test]
    fn test_pdp_miss_permanent() {
        let mut ledger = setup();
        let addr = Address::test(1);
        ledger.deposit(addr, 10_000_000, 100);

        // Third miss: 100% slash, permanent cooldown
        let slashed = ledger.slash(&addr, SlashReason::PdpMissThird, 200).unwrap();
        assert_eq!(slashed, 10_000_000);
        assert!(ledger.get(&addr).unwrap().in_cooldown(u64::MAX - 1));
    }

    #[test]
    fn test_withdraw() {
        let mut ledger = setup();
        let addr = Address::test(1);
        ledger.deposit(addr, 5_000_000, 100);

        ledger.withdraw(&addr, 2_000_000, 200).unwrap();
        assert_eq!(ledger.get(&addr).unwrap().available(), 3_000_000);
    }

    #[test]
    fn test_withdraw_during_cooldown() {
        let mut ledger = setup();
        let addr = Address::test(1);
        ledger.deposit(addr, 10_000_000, 100);
        ledger.slash(&addr, SlashReason::DisputeLost, 200).unwrap();

        // Can't withdraw during cooldown
        assert!(ledger.withdraw(&addr, 1_000_000, 201).is_err());

        // Can withdraw after cooldown
        assert!(ledger.withdraw(&addr, 1_000_000, 3081).is_ok());
    }

    #[test]
    fn test_can_provide_and_challenge() {
        let mut ledger = setup();
        let provider = Address::test(1);
        let challenger = Address::test(2);
        let poor = Address::test(3);

        ledger.deposit(provider, 2_000_000, 100);
        ledger.deposit(challenger, 600_000, 100);
        ledger.deposit(poor, 100_000, 100);

        assert!(ledger.can_provide(&provider, 100));
        assert!(!ledger.can_provide(&challenger, 100)); // Below min provider stake
        assert!(ledger.can_challenge(&challenger, 100));
        assert!(!ledger.can_challenge(&poor, 100)); // Below min challenger stake
    }

    #[test]
    fn test_multiple_slashes_accumulate() {
        let mut ledger = setup();
        let addr = Address::test(1);
        ledger.deposit(addr, 10_000_000, 100);

        // Two false challenges: 5% each
        ledger.slash(&addr, SlashReason::FalseChallenge, 200).unwrap();
        ledger.slash(&addr, SlashReason::FalseChallenge, 300).unwrap();

        let entry = ledger.get(&addr).unwrap();
        assert_eq!(entry.slashed, 1_000_000); // 500K + 500K
        assert_eq!(entry.available(), 9_000_000);
    }
}
