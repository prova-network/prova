//! Reputation system — EMA-based scoring with decay and slashing integration.
//!
//! Each provider has a reputation score per model, computed as an exponential
//! moving average (EMA) of recent performance signals. Scores decay toward a
//! neutral baseline during inactivity. Slashing events apply instant penalties.
//!
//! Score range: 0–10000 (basis points, 10000 = perfect).
//! Neutral baseline: 5000 (new providers start here).

use crate::types::{Address, Epoch, ModelId, StakeAmount};
use std::collections::HashMap;

/// Maximum reputation score (basis points).
pub const MAX_SCORE: u64 = 10000;
/// Neutral starting score.
pub const NEUTRAL_SCORE: u64 = 5000;
/// Minimum score before provider is suspended.
pub const SUSPENSION_THRESHOLD: u64 = 1000;

/// EMA smoothing factor as basis points (α = EMA_ALPHA_BPS / 10000).
/// α = 0.1 → recent observations weighted 10%.
pub const EMA_ALPHA_BPS: u64 = 1000;

/// Decay rate per epoch of inactivity (bps toward neutral per epoch).
/// Score moves 0.5% toward NEUTRAL_SCORE each idle epoch.
pub const DECAY_RATE_BPS: u64 = 50;

/// Maximum epochs of decay to apply in a single update (caps catch-up).
pub const MAX_DECAY_EPOCHS: u64 = 100;

/// A performance observation fed into the reputation system.
#[derive(Debug, Clone)]
pub struct Observation {
    pub epoch: Epoch,
    pub kind: ObservationKind,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ObservationKind {
    /// Job completed successfully within SLA.
    Success,
    /// Job completed but SLA violated (e.g., late delivery).
    SlaViolation,
    /// Job was assigned but provider failed to deliver.
    JobMissed,
    /// Provider lost a QBP dispute (incorrect inference).
    DisputeLost,
    /// Provider won a QBP dispute (correct inference challenged).
    DisputeWon,
}

impl ObservationKind {
    /// Signal value in basis points (0 = worst, 10000 = best).
    pub fn signal_bps(&self) -> u64 {
        match self {
            Self::Success => 10000,
            Self::DisputeWon => 10000,
            Self::SlaViolation => 3000,
            Self::JobMissed => 0,
            Self::DisputeLost => 0,
        }
    }

    /// Instant penalty applied to score (subtracted directly).
    pub fn instant_penalty_bps(&self) -> u64 {
        match self {
            Self::DisputeLost => 2000,
            Self::JobMissed => 500,
            _ => 0,
        }
    }
}

/// Per-provider, per-model reputation state.
#[derive(Debug, Clone)]
pub struct ReputationEntry {
    pub provider: Address,
    pub model_id: ModelId,
    /// EMA score in basis points [0, 10000].
    pub score: u64,
    /// Epoch of last observation or decay.
    pub last_active_epoch: Epoch,
    /// Total observations recorded.
    pub observation_count: u64,
    /// Total successful observations.
    pub success_count: u64,
    /// Whether provider is suspended (score dropped below threshold).
    pub suspended: bool,
}

impl ReputationEntry {
    pub fn new(provider: Address, model_id: ModelId, epoch: Epoch) -> Self {
        Self {
            provider,
            model_id,
            score: NEUTRAL_SCORE,
            last_active_epoch: epoch,
            observation_count: 0,
            success_count: 0,
            suspended: false,
        }
    }

    /// Apply decay for inactivity since last_active_epoch.
    pub fn apply_decay(&mut self, current_epoch: Epoch) {
        if current_epoch <= self.last_active_epoch {
            return;
        }
        let idle_epochs = (current_epoch - self.last_active_epoch).min(MAX_DECAY_EPOCHS);
        for _ in 0..idle_epochs {
            if self.score > NEUTRAL_SCORE {
                let diff = self.score - NEUTRAL_SCORE;
                let decay = (diff * DECAY_RATE_BPS) / 10000;
                self.score -= decay.max(1); // at least 1 bps decay
            } else if self.score < NEUTRAL_SCORE {
                let diff = NEUTRAL_SCORE - self.score;
                let recovery = (diff * DECAY_RATE_BPS) / 10000;
                self.score += recovery.max(1);
            }
        }
        self.last_active_epoch = current_epoch;
    }

    /// Record an observation and update EMA score.
    pub fn record(&mut self, obs: &Observation) -> ReputationUpdate {
        let old_score = self.score;

        // Apply decay first
        self.apply_decay(obs.epoch);

        // Apply instant penalty
        let penalty = obs.kind.instant_penalty_bps();
        self.score = self.score.saturating_sub(penalty);

        // EMA update: score = α * signal + (1-α) * score
        let signal = obs.kind.signal_bps();
        let alpha = EMA_ALPHA_BPS;
        self.score = (alpha * signal + (10000 - alpha) * self.score) / 10000;

        // Clamp
        self.score = self.score.min(MAX_SCORE);

        self.observation_count += 1;
        if obs.kind == ObservationKind::Success || obs.kind == ObservationKind::DisputeWon {
            self.success_count += 1;
        }
        self.last_active_epoch = obs.epoch;

        // Check suspension
        let was_suspended = self.suspended;
        self.suspended = self.score < SUSPENSION_THRESHOLD;

        ReputationUpdate {
            old_score,
            new_score: self.score,
            newly_suspended: !was_suspended && self.suspended,
            newly_restored: was_suspended && !self.suspended,
        }
    }

    /// Success rate as basis points.
    pub fn success_rate_bps(&self) -> u64 {
        if self.observation_count == 0 {
            return 10000; // no data = assume good
        }
        self.success_count * 10000 / self.observation_count
    }
}

/// Result of a reputation update.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReputationUpdate {
    pub old_score: u64,
    pub new_score: u64,
    pub newly_suspended: bool,
    pub newly_restored: bool,
}

/// Global reputation registry.
#[derive(Debug, Default)]
pub struct ReputationRegistry {
    entries: HashMap<(Address, ModelId), ReputationEntry>,
}

impl ReputationRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    /// Get or create a reputation entry.
    pub fn get_or_create(&mut self, provider: Address, model_id: ModelId, epoch: Epoch) -> &mut ReputationEntry {
        self.entries
            .entry((provider, model_id))
            .or_insert_with(|| ReputationEntry::new(provider, model_id, epoch))
    }

    /// Record an observation for a provider.
    pub fn record(
        &mut self,
        provider: Address,
        model_id: ModelId,
        obs: Observation,
    ) -> ReputationUpdate {
        let entry = self.get_or_create(provider, model_id, obs.epoch);
        entry.record(&obs)
    }

    /// Get current score (with decay applied to current epoch).
    pub fn score(&mut self, provider: Address, model_id: ModelId, current_epoch: Epoch) -> u64 {
        let entry = self.get_or_create(provider, model_id, current_epoch);
        entry.apply_decay(current_epoch);
        entry.score
    }

    /// Check if provider is suspended.
    pub fn is_suspended(&self, provider: Address, model_id: ModelId) -> bool {
        self.entries
            .get(&(provider, model_id))
            .map(|e| e.suspended)
            .unwrap_or(false)
    }

    /// Get immutable entry reference.
    pub fn get(&self, provider: Address, model_id: ModelId) -> Option<&ReputationEntry> {
        self.entries.get(&(provider, model_id))
    }

    /// Compute a slash amount based on reputation score.
    /// Lower reputation → higher slash multiplier.
    pub fn slash_multiplier_bps(&self, provider: Address, model_id: ModelId) -> u64 {
        let score = self.entries
            .get(&(provider, model_id))
            .map(|e| e.score)
            .unwrap_or(NEUTRAL_SCORE);
        // Inverse relationship: low score → high multiplier
        // At score 0: 2x slash (20000 bps)
        // At score 5000: 1x slash (10000 bps)
        // At score 10000: 0.5x slash (5000 bps)
        let multiplier = 20000u64.saturating_sub(score * 15000 / MAX_SCORE);
        multiplier.max(5000)
    }

    /// Number of tracked provider-model pairs.
    pub fn entry_count(&self) -> usize {
        self.entries.len()
    }
}

// ─── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn addr(id: u8) -> Address { Address::test(id) }
    fn model(id: u8) -> ModelId {
        let mut h = [0u8; 32]; h[0] = id; ModelId(h)
    }

    #[test]
    fn test_new_entry_starts_at_neutral() {
        let entry = ReputationEntry::new(addr(1), model(1), 0);
        assert_eq!(entry.score, NEUTRAL_SCORE);
        assert!(!entry.suspended);
        assert_eq!(entry.observation_count, 0);
    }

    #[test]
    fn test_success_increases_score() {
        let mut entry = ReputationEntry::new(addr(1), model(1), 0);
        let obs = Observation { epoch: 1, kind: ObservationKind::Success };
        let update = entry.record(&obs);
        assert!(update.new_score > NEUTRAL_SCORE);
    }

    #[test]
    fn test_job_missed_decreases_score() {
        let mut entry = ReputationEntry::new(addr(1), model(1), 0);
        let obs = Observation { epoch: 1, kind: ObservationKind::JobMissed };
        let update = entry.record(&obs);
        // Instant penalty (500) + EMA toward 0
        assert!(update.new_score < NEUTRAL_SCORE);
    }

    #[test]
    fn test_dispute_lost_severe_penalty() {
        let mut entry = ReputationEntry::new(addr(1), model(1), 0);
        let obs = Observation { epoch: 1, kind: ObservationKind::DisputeLost };
        let update = entry.record(&obs);
        // 2000 bps instant penalty + EMA toward 0
        assert!(update.new_score < 3500);
    }

    #[test]
    fn test_ema_converges_with_repeated_success() {
        let mut entry = ReputationEntry::new(addr(1), model(1), 0);
        for i in 1..=50 {
            let obs = Observation { epoch: i, kind: ObservationKind::Success };
            entry.record(&obs);
        }
        // After 50 successes, score should be close to MAX_SCORE
        assert!(entry.score > 9000, "score {} should be >9000", entry.score);
    }

    #[test]
    fn test_ema_converges_with_repeated_failures() {
        let mut entry = ReputationEntry::new(addr(1), model(1), 0);
        for i in 1..=30 {
            let obs = Observation { epoch: i, kind: ObservationKind::JobMissed };
            entry.record(&obs);
        }
        assert!(entry.score < 500, "score {} should be <500", entry.score);
        assert!(entry.suspended);
    }

    #[test]
    fn test_suspension_on_low_score() {
        let mut entry = ReputationEntry::new(addr(1), model(1), 0);
        // Repeated disputes drive score below threshold
        for i in 1..=5 {
            let obs = Observation { epoch: i, kind: ObservationKind::DisputeLost };
            let update = entry.record(&obs);
            if update.newly_suspended {
                assert!(entry.score < SUSPENSION_THRESHOLD);
                return;
            }
        }
        assert!(entry.suspended, "should be suspended after 5 dispute losses");
    }

    #[test]
    fn test_suspension_recovery() {
        let mut entry = ReputationEntry::new(addr(1), model(1), 0);
        // Drive below threshold
        for i in 1..=5 {
            entry.record(&Observation { epoch: i, kind: ObservationKind::DisputeLost });
        }
        assert!(entry.suspended);

        // Recover with many successes
        for i in 6..=60 {
            let update = entry.record(&Observation { epoch: i, kind: ObservationKind::Success });
            if update.newly_restored {
                assert!(!entry.suspended);
                return;
            }
        }
        // Should have recovered by now
        assert!(!entry.suspended, "score {} should have recovered", entry.score);
    }

    #[test]
    fn test_decay_toward_neutral_from_above() {
        let mut entry = ReputationEntry::new(addr(1), model(1), 0);
        entry.score = 9000;
        entry.apply_decay(50);
        assert!(entry.score < 9000);
        assert!(entry.score > NEUTRAL_SCORE); // shouldn't overshoot
    }

    #[test]
    fn test_decay_toward_neutral_from_below() {
        let mut entry = ReputationEntry::new(addr(1), model(1), 0);
        entry.score = 2000;
        entry.apply_decay(50);
        assert!(entry.score > 2000);
        assert!(entry.score < NEUTRAL_SCORE);
    }

    #[test]
    fn test_decay_capped_at_max_epochs() {
        let mut entry = ReputationEntry::new(addr(1), model(1), 0);
        entry.score = 9000;
        let mut e1 = entry.clone();
        e1.apply_decay(MAX_DECAY_EPOCHS);
        let mut e2 = entry.clone();
        e2.apply_decay(MAX_DECAY_EPOCHS * 10); // way more, but capped
        assert_eq!(e1.score, e2.score);
    }

    #[test]
    fn test_no_decay_at_neutral() {
        let mut entry = ReputationEntry::new(addr(1), model(1), 0);
        assert_eq!(entry.score, NEUTRAL_SCORE);
        entry.apply_decay(100);
        assert_eq!(entry.score, NEUTRAL_SCORE);
    }

    #[test]
    fn test_registry_record_and_query() {
        let mut reg = ReputationRegistry::new();
        let p = addr(1);
        let m = model(1);

        reg.record(p, m, Observation { epoch: 1, kind: ObservationKind::Success });
        let score = reg.score(p, m, 1);
        assert!(score > NEUTRAL_SCORE);
        assert_eq!(reg.entry_count(), 1);
    }

    #[test]
    fn test_registry_separate_models() {
        let mut reg = ReputationRegistry::new();
        let p = addr(1);
        let m1 = model(1);
        let m2 = model(2);

        reg.record(p, m1, Observation { epoch: 1, kind: ObservationKind::Success });
        reg.record(p, m2, Observation { epoch: 1, kind: ObservationKind::DisputeLost });

        assert!(reg.score(p, m1, 1) > reg.score(p, m2, 1));
        assert_eq!(reg.entry_count(), 2);
    }

    #[test]
    fn test_slash_multiplier_inversely_proportional() {
        let mut reg = ReputationRegistry::new();
        let p1 = addr(1);
        let p2 = addr(2);
        let m = model(1);

        // p1: high rep
        for i in 1..=20 {
            reg.record(p1, m, Observation { epoch: i, kind: ObservationKind::Success });
        }
        // p2: low rep
        for i in 1..=10 {
            reg.record(p2, m, Observation { epoch: i, kind: ObservationKind::DisputeLost });
        }

        let m1 = reg.slash_multiplier_bps(p1, m);
        let m2 = reg.slash_multiplier_bps(p2, m);
        assert!(m2 > m1, "low-rep provider should have higher slash multiplier: {} vs {}", m2, m1);
    }

    #[test]
    fn test_success_rate_tracking() {
        let mut reg = ReputationRegistry::new();
        let p = addr(1);
        let m = model(1);

        reg.record(p, m, Observation { epoch: 1, kind: ObservationKind::Success });
        reg.record(p, m, Observation { epoch: 2, kind: ObservationKind::Success });
        reg.record(p, m, Observation { epoch: 3, kind: ObservationKind::JobMissed });
        reg.record(p, m, Observation { epoch: 4, kind: ObservationKind::Success });

        let entry = reg.get(p, m).unwrap();
        assert_eq!(entry.observation_count, 4);
        assert_eq!(entry.success_count, 3);
        assert_eq!(entry.success_rate_bps(), 7500); // 75%
    }

    #[test]
    fn test_dispute_won_counts_as_success() {
        let mut reg = ReputationRegistry::new();
        let p = addr(1);
        let m = model(1);

        let update = reg.record(p, m, Observation { epoch: 1, kind: ObservationKind::DisputeWon });
        assert!(update.new_score > NEUTRAL_SCORE);
        assert_eq!(reg.get(p, m).unwrap().success_count, 1);
    }

    #[test]
    fn test_sla_violation_moderate_penalty() {
        let mut entry = ReputationEntry::new(addr(1), model(1), 0);
        let update = entry.record(&Observation { epoch: 1, kind: ObservationKind::SlaViolation });
        // SLA violation: no instant penalty, signal=3000 → EMA pulls down slightly
        assert!(update.new_score < NEUTRAL_SCORE);
        assert!(update.new_score > 4000); // not as harsh as JobMissed
    }

    #[test]
    fn test_score_clamped_at_max() {
        let mut entry = ReputationEntry::new(addr(1), model(1), 0);
        entry.score = MAX_SCORE;
        let obs = Observation { epoch: 1, kind: ObservationKind::Success };
        entry.record(&obs);
        assert!(entry.score <= MAX_SCORE);
    }
}
