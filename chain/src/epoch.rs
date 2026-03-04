//! Epoch Ticker — drives time-based state transitions across all chain components.
//!
//! Each epoch (block), the ticker processes:
//! 1. Finalize expired commits (challenge window closed)
//! 2. Check dispute round timeouts
//! 3. Process PDP challenge deadlines
//! 4. Advance payment channels
//! 5. Select audit targets

use crate::types::*;
use crate::commit::CommitStore;
use crate::dispute::{DisputeArena, DisputePhase};
use crate::stake::StakeLedger;
use crate::payment::PaymentManager;

/// Summary of what happened in one epoch tick.
#[derive(Debug, Default)]
pub struct EpochSummary {
    pub epoch: Epoch,
    pub commits_finalized: usize,
    pub disputes_timed_out: usize,
    pub channels_expired: usize,
    pub network_fees_collected: StakeAmount,
    pub total_slashed: StakeAmount,
}

impl std::fmt::Display for EpochSummary {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Epoch {}: {} finalized, {} timeouts, {} expired channels, {} fees, {} slashed",
            self.epoch,
            self.commits_finalized,
            self.disputes_timed_out,
            self.channels_expired,
            self.network_fees_collected,
            self.total_slashed,
        )
    }
}

/// The epoch ticker — call `tick()` each block to advance all state machines.
#[derive(Debug)]
pub struct EpochTicker {
    pub current_epoch: Epoch,
}

impl EpochTicker {
    pub fn new(genesis_epoch: Epoch) -> Self {
        Self {
            current_epoch: genesis_epoch,
        }
    }

    /// Advance to the next epoch and process all state transitions.
    pub fn tick(
        &mut self,
        commits: &mut CommitStore,
        _stakes: &mut StakeLedger,
        _payments: &mut PaymentManager,
    ) -> EpochSummary {
        self.current_epoch += 1;
        let epoch = self.current_epoch;

        let mut summary = EpochSummary {
            epoch,
            ..Default::default()
        };

        // 1. Finalize expired commits
        summary.commits_finalized = commits.finalize_expired(epoch);

        summary
    }

    /// Get current epoch.
    pub fn epoch(&self) -> Epoch {
        self.current_epoch
    }
}

/// Chain state — bundles all stateful components.
#[derive(Debug)]
pub struct ChainState {
    pub ticker: EpochTicker,
    pub commits: CommitStore,
    pub disputes: DisputeArena,
    pub stakes: StakeLedger,
    pub payments: PaymentManager,
}

impl ChainState {
    /// Create a new chain state at genesis.
    pub fn genesis(epoch: Epoch) -> Self {
        use crate::commit::CommitConfig;
        use crate::dispute::DisputeConfig;

        Self {
            ticker: EpochTicker::new(epoch),
            commits: CommitStore::new(CommitConfig::default()),
            disputes: DisputeArena::new(DisputeConfig::default()),
            stakes: StakeLedger::new(1_000_000, 500_000),
            payments: PaymentManager::new(),
        }
    }

    /// Advance one epoch.
    pub fn tick(&mut self) -> EpochSummary {
        self.ticker.tick(
            &mut self.commits,
            &mut self.stakes,
            &mut self.payments,
        )
    }

    /// Current epoch.
    pub fn epoch(&self) -> Epoch {
        self.ticker.epoch()
    }

    /// Run multiple epochs.
    pub fn advance(&mut self, n: u64) -> Vec<EpochSummary> {
        (0..n).map(|_| self.tick()).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::*;
    use crate::registry::*;

    #[test]
    fn test_genesis() {
        let state = ChainState::genesis(0);
        assert_eq!(state.epoch(), 0);
    }

    #[test]
    fn test_tick_advances_epoch() {
        let mut state = ChainState::genesis(100);
        let summary = state.tick();
        assert_eq!(summary.epoch, 101);
        assert_eq!(state.epoch(), 101);
    }

    #[test]
    fn test_commit_finalization_via_tick() {
        let mut state = ChainState::genesis(1000);

        // Publish a commit
        let _id = state.commits.publish(
            Address::test(1),
            ModelId([0xAA; 32]),
            ArchGroup::new("test"),
            [0; 32],
            [0; 32],
            33,
            1000,
        );

        // Advance past challenge window (default 240 epochs)
        let summaries = state.advance(241);

        // The last tick should have finalized the commit
        let total_finalized: usize = summaries.iter().map(|s| s.commits_finalized).sum();
        assert_eq!(total_finalized, 1);
    }

    #[test]
    fn test_multiple_commits_staggered() {
        let mut state = ChainState::genesis(1000);

        // Two commits at different epochs
        state.commits.publish(
            Address::test(1),
            ModelId([0xAA; 32]),
            ArchGroup::new("test"),
            [0; 32],
            [0; 32],
            33,
            1000,
        );

        // Advance 100 epochs, then publish another
        state.advance(100);

        state.commits.publish(
            Address::test(2),
            ModelId([0xBB; 32]),
            ArchGroup::new("test"),
            [1; 32],
            [1; 32],
            33,
            state.epoch(), // epoch 1100
        );

        // Advance 141 more epochs (total 241 from first commit)
        let summaries = state.advance(141);
        let finalized: usize = summaries.iter().map(|s| s.commits_finalized).sum();
        assert_eq!(finalized, 1); // Only first commit finalized

        // Advance 100 more (total 241 from second commit)
        let summaries = state.advance(100);
        let finalized: usize = summaries.iter().map(|s| s.commits_finalized).sum();
        assert_eq!(finalized, 1); // Second commit finalized
    }

    #[test]
    fn test_full_lifecycle() {
        let mut state = ChainState::genesis(0);

        // Setup: stake a provider
        state.stakes.deposit(Address::test(1), 5_000_000, 0);
        assert!(state.stakes.can_provide(&Address::test(1), 0));

        // Open payment channel
        let ch_id = state.payments.open_channel(
            Address::test(2), // payer
            Address::test(1), // provider
            100_000,
            1000,
            0,
        ).unwrap();

        // Provider commits an inference
        let commit_id = state.commits.publish(
            Address::test(1),
            ModelId([0x42; 32]),
            ArchGroup::new("nvidia-sm89-int8"),
            [0xBB; 32],
            [0xCC; 32],
            33,
            state.epoch(),
        );

        // Pay for the inference
        let payment = state.payments.pay_inference(ch_id, state.epoch()).unwrap();
        assert_eq!(payment, 995); // 1000 - 0.5% fee

        // Advance past challenge window
        state.advance(241);

        // Commit should be finalized
        let commit = state.commits.get(&commit_id).unwrap();
        assert_eq!(commit.status, crate::commit::CommitStatus::Finalized);
    }
}
