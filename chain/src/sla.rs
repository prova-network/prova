//! Service-level agreements — SLA enforcement with penalty curves.
//!
//! Providers commit to SLA tiers when registering for a model.
//! Each tier specifies latency bounds, availability targets, and
//! throughput minimums. Violations accumulate penalty points on a
//! convex curve (quadratic), triggering stake slashing at thresholds.

use crate::types::{Address, Epoch, ModelId, StakeAmount};
use std::collections::HashMap;

/// SLA tier — defines performance commitments.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SlaTier {
    pub name: String,
    /// Maximum latency in milliseconds for inference delivery.
    pub max_latency_ms: u64,
    /// Minimum availability ratio (0..10000 = 0.00%..100.00%).
    pub availability_bps: u16,
    /// Minimum inferences per epoch the provider must be able to serve.
    pub min_throughput_per_epoch: u32,
    /// Reward multiplier in basis points (10000 = 1.0x, 15000 = 1.5x).
    pub reward_multiplier_bps: u16,
}

/// Predefined SLA tiers.
impl SlaTier {
    pub fn bronze() -> Self {
        Self {
            name: "bronze".into(),
            max_latency_ms: 5000,
            availability_bps: 9500,  // 95%
            min_throughput_per_epoch: 1,
            reward_multiplier_bps: 10000, // 1.0x
        }
    }

    pub fn silver() -> Self {
        Self {
            name: "silver".into(),
            max_latency_ms: 2000,
            availability_bps: 9900,  // 99%
            min_throughput_per_epoch: 5,
            reward_multiplier_bps: 12000, // 1.2x
        }
    }

    pub fn gold() -> Self {
        Self {
            name: "gold".into(),
            max_latency_ms: 500,
            availability_bps: 9990,  // 99.9%
            min_throughput_per_epoch: 20,
            reward_multiplier_bps: 15000, // 1.5x
        }
    }
}

/// A single SLA violation event.
#[derive(Debug, Clone)]
pub struct Violation {
    pub epoch: Epoch,
    pub kind: ViolationKind,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ViolationKind {
    /// Inference delivered past the latency bound.
    LatencyExceeded { actual_ms: u64, limit_ms: u64 },
    /// Provider was unreachable during a liveness check.
    Unavailable,
    /// Throughput fell below the committed minimum.
    ThroughputDeficit { actual: u32, required: u32 },
    /// Provider missed an assigned job entirely.
    JobMissed,
}

/// Per-provider SLA tracking.
#[derive(Debug, Clone)]
pub struct ProviderSla {
    pub provider: Address,
    pub model_id: ModelId,
    pub tier: SlaTier,
    pub registered_at: Epoch,
    pub violations: Vec<Violation>,
    /// Penalty points — convex accumulation.
    pub penalty_points: u64,
    /// Whether this SLA has been terminated (slashed out).
    pub terminated: bool,
}

/// Penalty curve thresholds — penalty_points → action.
const WARN_THRESHOLD: u64 = 50;
const MINOR_SLASH_THRESHOLD: u64 = 200;
const MAJOR_SLASH_THRESHOLD: u64 = 500;
const TERMINATION_THRESHOLD: u64 = 1000;

/// Slash percentages in basis points.
const MINOR_SLASH_BPS: u64 = 100;   // 1%
const MAJOR_SLASH_BPS: u64 = 500;   // 5%
const TERMINATION_SLASH_BPS: u64 = 2000; // 20%

/// Action triggered by a penalty evaluation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PenaltyAction {
    None,
    Warning,
    MinorSlash { amount: StakeAmount },
    MajorSlash { amount: StakeAmount },
    Termination { amount: StakeAmount },
}

/// SLA registry — manages all provider SLAs.
#[derive(Debug, Default)]
pub struct SlaRegistry {
    /// (provider, model_id) → SLA state.
    pub slas: HashMap<(Address, ModelId), ProviderSla>,
}

impl SlaRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    /// Register a provider's SLA commitment for a model.
    pub fn register(
        &mut self,
        provider: Address,
        model_id: ModelId,
        tier: SlaTier,
        epoch: Epoch,
    ) -> Result<(), SlaError> {
        let key = (provider, model_id);
        if self.slas.contains_key(&key) {
            return Err(SlaError::AlreadyRegistered);
        }
        self.slas.insert(key, ProviderSla {
            provider,
            model_id,
            tier,
            registered_at: epoch,
            violations: Vec::new(),
            penalty_points: 0,
            terminated: false,
        });
        Ok(())
    }

    /// Record a violation and compute penalty points (quadratic curve).
    /// Returns the penalty action triggered.
    pub fn record_violation(
        &mut self,
        provider: Address,
        model_id: ModelId,
        violation: Violation,
        provider_stake: StakeAmount,
    ) -> Result<PenaltyAction, SlaError> {
        let key = (provider, model_id);
        let sla = self.slas.get_mut(&key).ok_or(SlaError::NotRegistered)?;
        if sla.terminated {
            return Err(SlaError::Terminated);
        }

        // Quadratic penalty: points = (violation_count)^2
        // This makes repeated violations increasingly expensive.
        let count = sla.violations.len() as u64 + 1;
        let new_points = count * count;
        let old_points = sla.penalty_points;

        sla.violations.push(violation);
        sla.penalty_points = new_points;

        // Determine action based on threshold crossing.
        let action = if new_points >= TERMINATION_THRESHOLD {
            sla.terminated = true;
            let slash = provider_stake * TERMINATION_SLASH_BPS as u128 / 10000;
            PenaltyAction::Termination { amount: slash }
        } else if new_points >= MAJOR_SLASH_THRESHOLD && old_points < MAJOR_SLASH_THRESHOLD {
            let slash = provider_stake * MAJOR_SLASH_BPS as u128 / 10000;
            PenaltyAction::MajorSlash { amount: slash }
        } else if new_points >= MINOR_SLASH_THRESHOLD && old_points < MINOR_SLASH_THRESHOLD {
            let slash = provider_stake * MINOR_SLASH_BPS as u128 / 10000;
            PenaltyAction::MinorSlash { amount: slash }
        } else if new_points >= WARN_THRESHOLD && old_points < WARN_THRESHOLD {
            PenaltyAction::Warning
        } else {
            PenaltyAction::None
        };

        Ok(action)
    }

    /// Get the current penalty points for a provider's SLA.
    pub fn penalty_points(&self, provider: Address, model_id: ModelId) -> Option<u64> {
        self.slas.get(&(provider, model_id)).map(|s| s.penalty_points)
    }

    /// Check if a provider's SLA is active (registered and not terminated).
    pub fn is_active(&self, provider: Address, model_id: ModelId) -> bool {
        self.slas.get(&(provider, model_id))
            .map(|s| !s.terminated)
            .unwrap_or(false)
    }

    /// Get violation count for a provider.
    pub fn violation_count(&self, provider: Address, model_id: ModelId) -> usize {
        self.slas.get(&(provider, model_id))
            .map(|s| s.violations.len())
            .unwrap_or(0)
    }

    /// Compute the reward multiplier for a provider (returns bps).
    pub fn reward_multiplier_bps(&self, provider: Address, model_id: ModelId) -> u16 {
        self.slas.get(&(provider, model_id))
            .filter(|s| !s.terminated)
            .map(|s| s.tier.reward_multiplier_bps)
            .unwrap_or(10000) // default 1.0x
    }

    /// Reset penalty points (e.g., after a grace period with clean record).
    /// Only allowed if below minor slash threshold.
    pub fn reset_penalties(
        &mut self,
        provider: Address,
        model_id: ModelId,
    ) -> Result<(), SlaError> {
        let sla = self.slas.get_mut(&(provider, model_id))
            .ok_or(SlaError::NotRegistered)?;
        if sla.terminated {
            return Err(SlaError::Terminated);
        }
        if sla.penalty_points >= MINOR_SLASH_THRESHOLD {
            return Err(SlaError::PenaltyTooHigh);
        }
        sla.penalty_points = 0;
        sla.violations.clear();
        Ok(())
    }

    /// Upgrade or downgrade a provider's SLA tier.
    pub fn change_tier(
        &mut self,
        provider: Address,
        model_id: ModelId,
        new_tier: SlaTier,
    ) -> Result<(), SlaError> {
        let sla = self.slas.get_mut(&(provider, model_id))
            .ok_or(SlaError::NotRegistered)?;
        if sla.terminated {
            return Err(SlaError::Terminated);
        }
        sla.tier = new_tier;
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SlaError {
    AlreadyRegistered,
    NotRegistered,
    Terminated,
    PenaltyTooHigh,
}

impl std::fmt::Display for SlaError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::AlreadyRegistered => write!(f, "SLA already registered"),
            Self::NotRegistered => write!(f, "SLA not registered"),
            Self::Terminated => write!(f, "SLA terminated"),
            Self::PenaltyTooHigh => write!(f, "penalty too high to reset"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_model() -> ModelId {
        ModelId([0xAA; 32])
    }

    fn provider(id: u8) -> Address {
        Address::test(id)
    }

    #[test]
    fn test_register_and_query() {
        let mut reg = SlaRegistry::new();
        let p = provider(1);
        let m = test_model();
        reg.register(p, m, SlaTier::silver(), 100).unwrap();
        assert!(reg.is_active(p, m));
        assert_eq!(reg.penalty_points(p, m), Some(0));
        assert_eq!(reg.reward_multiplier_bps(p, m), 12000);
    }

    #[test]
    fn test_duplicate_registration_rejected() {
        let mut reg = SlaRegistry::new();
        let p = provider(1);
        let m = test_model();
        reg.register(p, m, SlaTier::bronze(), 100).unwrap();
        assert_eq!(reg.register(p, m, SlaTier::gold(), 200), Err(SlaError::AlreadyRegistered));
    }

    #[test]
    fn test_violation_quadratic_penalty() {
        let mut reg = SlaRegistry::new();
        let p = provider(1);
        let m = test_model();
        reg.register(p, m, SlaTier::gold(), 100).unwrap();

        // 1st violation: points = 1
        let v = Violation { epoch: 101, kind: ViolationKind::Unavailable };
        reg.record_violation(p, m, v, 1_000_000).unwrap();
        assert_eq!(reg.penalty_points(p, m), Some(1));

        // 5th violation: points = 25
        for i in 2..=5 {
            let v = Violation { epoch: 100 + i, kind: ViolationKind::Unavailable };
            reg.record_violation(p, m, v, 1_000_000).unwrap();
        }
        assert_eq!(reg.penalty_points(p, m), Some(25));

        // 10th violation: points = 100
        for i in 6..=10 {
            let v = Violation { epoch: 100 + i, kind: ViolationKind::Unavailable };
            reg.record_violation(p, m, v, 1_000_000).unwrap();
        }
        assert_eq!(reg.penalty_points(p, m), Some(100));
    }

    #[test]
    fn test_warning_threshold() {
        let mut reg = SlaRegistry::new();
        let p = provider(1);
        let m = test_model();
        reg.register(p, m, SlaTier::bronze(), 100).unwrap();

        // Need ceil(sqrt(50)) = 8 violations to reach 50+ points (8^2 = 64)
        for i in 1..=7 {
            let v = Violation { epoch: 100 + i, kind: ViolationKind::Unavailable };
            let action = reg.record_violation(p, m, v, 1_000_000).unwrap();
            assert_eq!(action, PenaltyAction::None);
        }
        // 8th: 64 >= 50
        let v = Violation { epoch: 108, kind: ViolationKind::Unavailable };
        let action = reg.record_violation(p, m, v, 1_000_000).unwrap();
        assert_eq!(action, PenaltyAction::Warning);
    }

    #[test]
    fn test_minor_slash_threshold() {
        let mut reg = SlaRegistry::new();
        let p = provider(1);
        let m = test_model();
        reg.register(p, m, SlaTier::bronze(), 100).unwrap();
        let stake: StakeAmount = 1_000_000;

        // Need ceil(sqrt(200)) = 15 violations (15^2 = 225)
        for i in 1..=14 {
            let v = Violation { epoch: 100 + i, kind: ViolationKind::LatencyExceeded { actual_ms: 6000, limit_ms: 5000 } };
            reg.record_violation(p, m, v, stake).unwrap();
        }
        let v = Violation { epoch: 115, kind: ViolationKind::LatencyExceeded { actual_ms: 6000, limit_ms: 5000 } };
        let action = reg.record_violation(p, m, v, stake).unwrap();
        assert_eq!(action, PenaltyAction::MinorSlash { amount: stake * 100 / 10000 }); // 1%
    }

    #[test]
    fn test_termination_at_threshold() {
        let mut reg = SlaRegistry::new();
        let p = provider(1);
        let m = test_model();
        reg.register(p, m, SlaTier::gold(), 100).unwrap();
        let stake: StakeAmount = 10_000_000;

        // Need ceil(sqrt(1000)) = 32 violations (32^2 = 1024)
        for i in 1..=31 {
            let v = Violation { epoch: 100 + i, kind: ViolationKind::JobMissed };
            reg.record_violation(p, m, v, stake).unwrap();
        }
        let v = Violation { epoch: 132, kind: ViolationKind::JobMissed };
        let action = reg.record_violation(p, m, v, stake).unwrap();
        assert_eq!(action, PenaltyAction::Termination { amount: stake * 2000 / 10000 }); // 20%
        assert!(!reg.is_active(p, m));
    }

    #[test]
    fn test_violation_on_terminated_sla_fails() {
        let mut reg = SlaRegistry::new();
        let p = provider(1);
        let m = test_model();
        reg.register(p, m, SlaTier::gold(), 100).unwrap();

        // Force termination
        for i in 1..=32 {
            let v = Violation { epoch: 100 + i, kind: ViolationKind::Unavailable };
            reg.record_violation(p, m, v, 10_000_000).unwrap();
        }
        let v = Violation { epoch: 200, kind: ViolationKind::Unavailable };
        assert_eq!(reg.record_violation(p, m, v, 10_000_000), Err(SlaError::Terminated));
    }

    #[test]
    fn test_reset_penalties_below_threshold() {
        let mut reg = SlaRegistry::new();
        let p = provider(1);
        let m = test_model();
        reg.register(p, m, SlaTier::silver(), 100).unwrap();

        // Add a few violations (below minor slash)
        for i in 1..=5 {
            let v = Violation { epoch: 100 + i, kind: ViolationKind::Unavailable };
            reg.record_violation(p, m, v, 1_000_000).unwrap();
        }
        assert_eq!(reg.penalty_points(p, m), Some(25));
        reg.reset_penalties(p, m).unwrap();
        assert_eq!(reg.penalty_points(p, m), Some(0));
        assert_eq!(reg.violation_count(p, m), 0);
    }

    #[test]
    fn test_reset_rejected_above_threshold() {
        let mut reg = SlaRegistry::new();
        let p = provider(1);
        let m = test_model();
        reg.register(p, m, SlaTier::bronze(), 100).unwrap();

        // Push past minor slash threshold (15 violations = 225 points)
        for i in 1..=15 {
            let v = Violation { epoch: 100 + i, kind: ViolationKind::Unavailable };
            reg.record_violation(p, m, v, 1_000_000).unwrap();
        }
        assert_eq!(reg.reset_penalties(p, m), Err(SlaError::PenaltyTooHigh));
    }

    #[test]
    fn test_change_tier() {
        let mut reg = SlaRegistry::new();
        let p = provider(1);
        let m = test_model();
        reg.register(p, m, SlaTier::bronze(), 100).unwrap();
        assert_eq!(reg.reward_multiplier_bps(p, m), 10000);
        reg.change_tier(p, m, SlaTier::gold()).unwrap();
        assert_eq!(reg.reward_multiplier_bps(p, m), 15000);
    }

    #[test]
    fn test_latency_violation_records_details() {
        let mut reg = SlaRegistry::new();
        let p = provider(1);
        let m = test_model();
        reg.register(p, m, SlaTier::silver(), 100).unwrap();

        let v = Violation {
            epoch: 101,
            kind: ViolationKind::LatencyExceeded { actual_ms: 3500, limit_ms: 2000 },
        };
        reg.record_violation(p, m, v, 1_000_000).unwrap();
        let sla = reg.slas.get(&(p, m)).unwrap();
        assert_eq!(sla.violations.len(), 1);
        assert_eq!(sla.violations[0].kind, ViolationKind::LatencyExceeded { actual_ms: 3500, limit_ms: 2000 });
    }

    #[test]
    fn test_throughput_deficit_violation() {
        let mut reg = SlaRegistry::new();
        let p = provider(1);
        let m = test_model();
        reg.register(p, m, SlaTier::gold(), 100).unwrap();

        let v = Violation {
            epoch: 101,
            kind: ViolationKind::ThroughputDeficit { actual: 5, required: 20 },
        };
        let action = reg.record_violation(p, m, v, 1_000_000).unwrap();
        assert_eq!(action, PenaltyAction::None); // first violation = 1 point
        assert_eq!(reg.violation_count(p, m), 1);
    }

    #[test]
    fn test_unregistered_provider_errors() {
        let mut reg = SlaRegistry::new();
        let p = provider(99);
        let m = test_model();
        assert_eq!(reg.penalty_points(p, m), None);
        assert!(!reg.is_active(p, m));
        let v = Violation { epoch: 100, kind: ViolationKind::Unavailable };
        assert_eq!(reg.record_violation(p, m, v, 1_000_000), Err(SlaError::NotRegistered));
    }

    #[test]
    fn test_gold_tier_properties() {
        let gold = SlaTier::gold();
        assert_eq!(gold.max_latency_ms, 500);
        assert_eq!(gold.availability_bps, 9990);
        assert_eq!(gold.min_throughput_per_epoch, 20);
        assert_eq!(gold.reward_multiplier_bps, 15000);
    }
}
