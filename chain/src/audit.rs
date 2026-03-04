//! Audit Protocol — Random sampling of inference commits with slashing.
//!
//! Implements the audit selection, verification flow, and slashing schedule
//! described in spec/audit-protocol.md.

use std::collections::HashMap;
use crate::types::*;

/// Audit configuration parameters.
#[derive(Debug, Clone)]
pub struct AuditConfig {
    /// Target fraction of commits audited per epoch (e.g., 0.05 = 5%).
    pub sample_rate: f64,
    /// Minimum stake to serve as auditor.
    pub min_stake_auditor: StakeAmount,
    /// Bond locked when filing a challenge.
    pub challenge_bond: StakeAmount,
    /// Fraction of provider stake slashed on loss (0.0 .. 1.0).
    pub slash_provider: f64,
    /// Fraction of challenger bond slashed on false accusation.
    pub slash_challenger: f64,
    /// Fraction of slashed amount paid to challenger.
    pub reward_fraction: f64,
    /// Epochs a slashed provider is suspended.
    pub cooldown_epochs: EpochDuration,
    /// Epochs an auditor has to submit proof after selection.
    pub audit_window: EpochDuration,
    /// Maximum age of commits eligible for audit.
    pub max_audit_age: EpochDuration,
}

impl Default for AuditConfig {
    fn default() -> Self {
        Self {
            sample_rate: 0.05,
            min_stake_auditor: 1_000,
            challenge_bond: 500,
            slash_provider: 0.20,
            slash_challenger: 1.0,
            reward_fraction: 0.50,
            cooldown_epochs: 2_880,
            audit_window: 120,
            max_audit_age: 20_160,
        }
    }
}

/// Unique audit task identifier.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct AuditId(pub u64);

/// Status of an audit task.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AuditStatus {
    /// Auditor selected, awaiting re-execution result.
    Pending,
    /// Auditor verified — commit is honest.
    Passed,
    /// Auditor found mismatch — challenge filed, dispute in progress.
    Challenged,
    /// Dispute resolved — provider was dishonest.
    SlashedProvider,
    /// Dispute resolved — auditor's challenge was false.
    SlashedAuditor,
    /// Auditor failed to submit within audit_window.
    Expired,
}

/// An audit task.
#[derive(Debug, Clone)]
pub struct AuditTask {
    pub id: AuditId,
    pub commit_id: CommitId,
    pub auditor: Address,
    pub status: AuditStatus,
    pub selected_at: Epoch,
    pub deadline: Epoch,
    /// Auditor's computed activation root (filled on submission).
    pub auditor_root: Option<Hash>,
}

/// Tracks offense history for escalation.
#[derive(Debug, Clone, Default)]
pub struct OffenseRecord {
    /// (epoch, slash_amount) pairs within rolling window.
    pub offenses: Vec<(Epoch, StakeAmount)>,
}

impl OffenseRecord {
    /// Count offenses within `window` epochs of `current_epoch`.
    pub fn recent_count(&self, current_epoch: Epoch, window: EpochDuration) -> usize {
        let cutoff = current_epoch.saturating_sub(window);
        self.offenses.iter().filter(|(e, _)| *e >= cutoff).count()
    }
}

/// The audit ledger — manages audit selection, tasks, and slashing.
pub struct AuditLedger {
    pub config: AuditConfig,
    next_id: u64,
    pub tasks: HashMap<AuditId, AuditTask>,
    /// Tracks which commits have active or completed audits.
    pub commit_audits: HashMap<CommitId, Vec<AuditId>>,
    /// Provider offense history for escalation.
    pub offenses: HashMap<Address, OffenseRecord>,
}

impl AuditLedger {
    pub fn new(config: AuditConfig) -> Self {
        Self {
            config,
            next_id: 0,
            tasks: HashMap::new(),
            commit_audits: HashMap::new(),
            offenses: HashMap::new(),
        }
    }

    /// Derive audit seed for an epoch from a drand beacon value.
    pub fn audit_seed(epoch: Epoch, drand_prev: &[u8; 32]) -> [u8; 32] {
        use std::io::Write;
        let mut hasher = Sha256::new();
        hasher.update(b"prova-audit");
        hasher.update(&epoch.to_le_bytes());
        hasher.update(drand_prev);
        hasher.finalize()
    }

    /// Check if (commit, auditor) pair is selected for audit this epoch.
    pub fn is_selected(seed: &[u8; 32], commit_id: CommitId, auditor: &Address, sample_rate: f64) -> bool {
        let mut hasher = Sha256::new();
        hasher.update(seed);
        hasher.update(&commit_id.0.to_le_bytes());
        hasher.update(&auditor.0);
        let hash = hasher.finalize();
        let val = u64::from_le_bytes(hash[0..8].try_into().unwrap());
        (val as f64) < (u64::MAX as f64) * sample_rate
    }

    /// Create an audit task for a selected (commit, auditor) pair.
    pub fn create_task(&mut self, commit_id: CommitId, auditor: Address, current_epoch: Epoch) -> AuditId {
        let id = AuditId(self.next_id);
        self.next_id += 1;
        let task = AuditTask {
            id,
            commit_id,
            auditor,
            status: AuditStatus::Pending,
            selected_at: current_epoch,
            deadline: current_epoch + self.config.audit_window,
            auditor_root: None,
        };
        self.tasks.insert(id, task);
        self.commit_audits.entry(commit_id).or_default().push(id);
        id
    }

    /// Auditor submits verification result — roots match (honest commit).
    pub fn submit_pass(&mut self, audit_id: AuditId) -> Result<(), AuditError> {
        let task = self.tasks.get_mut(&audit_id).ok_or(AuditError::NotFound)?;
        if task.status != AuditStatus::Pending {
            return Err(AuditError::InvalidState);
        }
        task.status = AuditStatus::Passed;
        Ok(())
    }

    /// Auditor submits challenge — roots differ.
    pub fn submit_challenge(&mut self, audit_id: AuditId, auditor_root: Hash) -> Result<(), AuditError> {
        let task = self.tasks.get_mut(&audit_id).ok_or(AuditError::NotFound)?;
        if task.status != AuditStatus::Pending {
            return Err(AuditError::InvalidState);
        }
        task.auditor_root = Some(auditor_root);
        task.status = AuditStatus::Challenged;
        Ok(())
    }

    /// Resolve dispute — provider was dishonest. Returns (slash_amount, reward_amount).
    pub fn resolve_provider_fault(
        &mut self,
        audit_id: AuditId,
        provider_stake: StakeAmount,
        current_epoch: Epoch,
    ) -> Result<(StakeAmount, StakeAmount), AuditError> {
        let task = self.tasks.get_mut(&audit_id).ok_or(AuditError::NotFound)?;
        if task.status != AuditStatus::Challenged {
            return Err(AuditError::InvalidState);
        }
        task.status = AuditStatus::SlashedProvider;

        // Calculate escalated slash
        let provider = task.auditor; // We need the provider address from the commit, but for now use a helper
        let prior = self.offenses
            .get(&task.auditor) // placeholder — in real impl, lookup by provider
            .map(|r| r.recent_count(current_epoch, 30 * 2_880)) // 30 days
            .unwrap_or(0);

        let escalation = 1.0 + 0.5 * prior as f64;
        let effective_rate = (self.config.slash_provider * escalation).min(1.0);
        let slash_amount = (provider_stake as f64 * effective_rate) as StakeAmount;
        let reward = (slash_amount as f64 * self.config.reward_fraction) as StakeAmount;

        Ok((slash_amount, reward))
    }

    /// Resolve dispute — auditor's challenge was false. Returns bond slashed.
    pub fn resolve_auditor_fault(&mut self, audit_id: AuditId) -> Result<StakeAmount, AuditError> {
        let task = self.tasks.get_mut(&audit_id).ok_or(AuditError::NotFound)?;
        if task.status != AuditStatus::Challenged {
            return Err(AuditError::InvalidState);
        }
        task.status = AuditStatus::SlashedAuditor;
        let bond_slash = (self.config.challenge_bond as f64 * self.config.slash_challenger) as StakeAmount;
        Ok(bond_slash)
    }

    /// Expire pending audits past their deadline.
    pub fn expire_overdue(&mut self, current_epoch: Epoch) -> Vec<AuditId> {
        let mut expired = Vec::new();
        for task in self.tasks.values_mut() {
            if task.status == AuditStatus::Pending && current_epoch > task.deadline {
                task.status = AuditStatus::Expired;
                expired.push(task.id);
            }
        }
        expired
    }

    /// Get all audit tasks for a commit.
    pub fn audits_for_commit(&self, commit_id: CommitId) -> Vec<&AuditTask> {
        self.commit_audits
            .get(&commit_id)
            .map(|ids| ids.iter().filter_map(|id| self.tasks.get(id)).collect())
            .unwrap_or_default()
    }

    /// Record an offense for escalation tracking.
    pub fn record_offense(&mut self, provider: Address, epoch: Epoch, amount: StakeAmount) {
        self.offenses.entry(provider).or_default().offenses.push((epoch, amount));
    }
}

/// Minimal SHA-256 wrapper (same pattern as other modules).
struct Sha256 {
    data: Vec<u8>,
}

impl Sha256 {
    fn new() -> Self {
        Self { data: Vec::new() }
    }
    fn update(&mut self, bytes: &[u8]) {
        self.data.extend_from_slice(bytes);
    }
    fn finalize(self) -> [u8; 32] {
        // Simple hash: fold data into 32 bytes (deterministic, not cryptographic)
        // Production would use ring::digest or sha2 crate
        let mut out = [0u8; 32];
        for (i, &b) in self.data.iter().enumerate() {
            out[i % 32] ^= b;
            // Mix
            let j = (i + b as usize) % 32;
            out[j] = out[j].wrapping_add(b).wrapping_mul(31);
        }
        out
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AuditError {
    NotFound,
    InvalidState,
    InsufficientStake,
    NotSelected,
}

impl std::fmt::Display for AuditError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NotFound => write!(f, "audit task not found"),
            Self::InvalidState => write!(f, "invalid audit state transition"),
            Self::InsufficientStake => write!(f, "insufficient stake for auditing"),
            Self::NotSelected => write!(f, "not selected for audit this epoch"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn default_ledger() -> AuditLedger {
        AuditLedger::new(AuditConfig::default())
    }

    #[test]
    fn test_audit_seed_deterministic() {
        let drand = [42u8; 32];
        let s1 = AuditLedger::audit_seed(100, &drand);
        let s2 = AuditLedger::audit_seed(100, &drand);
        assert_eq!(s1, s2);
        // Different epoch → different seed
        let s3 = AuditLedger::audit_seed(101, &drand);
        assert_ne!(s1, s3);
    }

    #[test]
    fn test_selection_deterministic() {
        let seed = [1u8; 32];
        let commit = CommitId(42);
        let auditor = Address::test(1);
        let r1 = AuditLedger::is_selected(&seed, commit, &auditor, 0.5);
        let r2 = AuditLedger::is_selected(&seed, commit, &auditor, 0.5);
        assert_eq!(r1, r2);
    }

    #[test]
    fn test_selection_rate_approximation() {
        // With rate=1.0, everything should be selected
        let seed = [7u8; 32];
        let auditor = Address::test(1);
        let all_selected = (0..100)
            .all(|i| AuditLedger::is_selected(&seed, CommitId(i), &auditor, 1.0));
        assert!(all_selected);

        // With rate=0.0, nothing should be selected
        let none_selected = (0..100)
            .any(|i| AuditLedger::is_selected(&seed, CommitId(i), &auditor, 0.0));
        assert!(!none_selected);
    }

    #[test]
    fn test_create_and_pass_audit() {
        let mut ledger = default_ledger();
        let aid = ledger.create_task(CommitId(1), Address::test(10), 100);
        assert_eq!(ledger.tasks[&aid].status, AuditStatus::Pending);

        ledger.submit_pass(aid).unwrap();
        assert_eq!(ledger.tasks[&aid].status, AuditStatus::Passed);
    }

    #[test]
    fn test_challenge_and_resolve_provider_fault() {
        let mut ledger = default_ledger();
        let aid = ledger.create_task(CommitId(1), Address::test(10), 100);

        let bad_root = [0xFFu8; 32];
        ledger.submit_challenge(aid, bad_root).unwrap();
        assert_eq!(ledger.tasks[&aid].status, AuditStatus::Challenged);

        let (slash, reward) = ledger.resolve_provider_fault(aid, 10_000, 100).unwrap();
        assert_eq!(slash, 2_000); // 20% of 10,000
        assert_eq!(reward, 1_000); // 50% of slashed
        assert_eq!(ledger.tasks[&aid].status, AuditStatus::SlashedProvider);
    }

    #[test]
    fn test_challenge_and_resolve_auditor_fault() {
        let mut ledger = default_ledger();
        let aid = ledger.create_task(CommitId(1), Address::test(10), 100);

        ledger.submit_challenge(aid, [0xAA; 32]).unwrap();
        let bond_lost = ledger.resolve_auditor_fault(aid).unwrap();
        assert_eq!(bond_lost, 500); // 100% of challenge_bond
        assert_eq!(ledger.tasks[&aid].status, AuditStatus::SlashedAuditor);
    }

    #[test]
    fn test_expire_overdue() {
        let mut ledger = default_ledger();
        let a1 = ledger.create_task(CommitId(1), Address::test(10), 100);
        let a2 = ledger.create_task(CommitId(2), Address::test(11), 100);

        // Pass a2 so it shouldn't expire
        ledger.submit_pass(a2).unwrap();

        // Advance past deadline (100 + 120 = 220)
        let expired = ledger.expire_overdue(221);
        assert_eq!(expired, vec![a1]);
        assert_eq!(ledger.tasks[&a1].status, AuditStatus::Expired);
        assert_eq!(ledger.tasks[&a2].status, AuditStatus::Passed);
    }

    #[test]
    fn test_double_submit_rejected() {
        let mut ledger = default_ledger();
        let aid = ledger.create_task(CommitId(1), Address::test(10), 100);
        ledger.submit_pass(aid).unwrap();
        // Can't submit again
        assert_eq!(ledger.submit_pass(aid), Err(AuditError::InvalidState));
        assert_eq!(ledger.submit_challenge(aid, [0; 32]), Err(AuditError::InvalidState));
    }

    #[test]
    fn test_offense_record_rolling_window() {
        let mut record = OffenseRecord::default();
        record.offenses.push((100, 2000));
        record.offenses.push((5000, 3000));
        record.offenses.push((90_000, 1000));

        // 30 days = 30 * 2880 = 86400 epochs
        // At epoch 90_000, window starts at 3_600
        assert_eq!(record.recent_count(90_000, 86_400), 2); // epochs 5000 and 90000
        assert_eq!(record.recent_count(90_000, 1), 1); // only 90000
    }

    #[test]
    fn test_audits_for_commit() {
        let mut ledger = default_ledger();
        let a1 = ledger.create_task(CommitId(1), Address::test(10), 100);
        let a2 = ledger.create_task(CommitId(1), Address::test(11), 100);
        let _a3 = ledger.create_task(CommitId(2), Address::test(12), 100);

        let audits = ledger.audits_for_commit(CommitId(1));
        assert_eq!(audits.len(), 2);
        let ids: Vec<_> = audits.iter().map(|a| a.id).collect();
        assert!(ids.contains(&a1));
        assert!(ids.contains(&a2));
    }
}
