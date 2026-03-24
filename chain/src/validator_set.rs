// chain/src/validator_set.rs — Dynamic validator set management
//
// Manages the active validator set with epoch-based rotation:
// - Validator registration (bond stake, declare capacity)
// - Voluntary exit (unbonding period)
// - Forced ejection (slashing, downtime)
// - Epoch transitions (compute next set from candidates)
// - Weight calculation (stake + reputation hybrid)

use std::collections::{BTreeMap, HashMap, HashSet};

/// Minimum stake to register as validator candidate
pub const MIN_VALIDATOR_STAKE: u64 = 100_000;
/// Maximum active validators per epoch
pub const MAX_ACTIVE_VALIDATORS: usize = 128;
/// Unbonding period in epochs after voluntary exit
pub const UNBONDING_EPOCHS: u64 = 14;
/// Downtime threshold — miss this many consecutive epochs → ejected
pub const DOWNTIME_THRESHOLD: u64 = 3;
/// Reputation weight in validator scoring (0.0–1.0)
pub const REPUTATION_WEIGHT: f64 = 0.3;
/// Stake weight in validator scoring
pub const STAKE_WEIGHT: f64 = 0.7;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ValidatorStatus {
    /// Registered candidate, not yet in active set
    Candidate,
    /// Currently in the active set
    Active,
    /// Voluntarily exiting, in unbonding period
    Unbonding { exit_epoch: u64 },
    /// Forcibly ejected (slashed or downtime)
    Ejected { reason: EjectionReason, epoch: u64 },
    /// Fully exited, stake returned
    Exited,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EjectionReason {
    Slashed,
    Downtime,
    InsufficientStake,
}

#[derive(Debug, Clone)]
pub struct Validator {
    pub address: String,
    pub stake: u64,
    pub reputation: f64, // 0.0–1.0
    pub status: ValidatorStatus,
    pub registered_epoch: u64,
    pub consecutive_misses: u64,
    pub blocks_produced: u64,
    pub capacity: u64, // declared inference capacity (ops/s)
}

impl Validator {
    /// Hybrid score: weighted combination of normalized stake and reputation
    pub fn score(&self, max_stake: u64) -> f64 {
        let stake_norm = if max_stake > 0 {
            self.stake as f64 / max_stake as f64
        } else {
            0.0
        };
        STAKE_WEIGHT * stake_norm + REPUTATION_WEIGHT * self.reputation
    }
}

#[derive(Debug, Clone)]
pub struct EpochRecord {
    pub epoch: u64,
    pub active_set: Vec<String>, // ordered by score
    pub total_stake: u64,
}

#[derive(Debug)]
pub struct ValidatorSet {
    validators: HashMap<String, Validator>,
    current_epoch: u64,
    epoch_history: Vec<EpochRecord>,
}

#[derive(Debug, PartialEq)]
pub enum ValidatorError {
    AlreadyRegistered,
    InsufficientStake,
    NotFound,
    NotActive,
    NotCandidate,
    AlreadyUnbonding,
    StillUnbonding,
    AlreadyExited,
    InvalidReputation,
}

impl ValidatorSet {
    pub fn new() -> Self {
        Self {
            validators: HashMap::new(),
            current_epoch: 0,
            epoch_history: Vec::new(),
        }
    }

    /// Register a new validator candidate
    pub fn register(
        &mut self,
        address: &str,
        stake: u64,
        capacity: u64,
    ) -> Result<(), ValidatorError> {
        if self.validators.contains_key(address) {
            return Err(ValidatorError::AlreadyRegistered);
        }
        if stake < MIN_VALIDATOR_STAKE {
            return Err(ValidatorError::InsufficientStake);
        }
        self.validators.insert(
            address.to_string(),
            Validator {
                address: address.to_string(),
                stake,
                reputation: 0.5, // start neutral
                capacity,
                status: ValidatorStatus::Candidate,
                registered_epoch: self.current_epoch,
                consecutive_misses: 0,
                blocks_produced: 0,
            },
        );
        Ok(())
    }

    /// Add stake to an existing validator
    pub fn add_stake(&mut self, address: &str, amount: u64) -> Result<u64, ValidatorError> {
        let v = self
            .validators
            .get_mut(address)
            .ok_or(ValidatorError::NotFound)?;
        match v.status {
            ValidatorStatus::Exited | ValidatorStatus::Ejected { .. } => {
                return Err(ValidatorError::AlreadyExited);
            }
            _ => {}
        }
        v.stake += amount;
        Ok(v.stake)
    }

    /// Initiate voluntary exit
    pub fn begin_exit(&mut self, address: &str) -> Result<u64, ValidatorError> {
        let v = self
            .validators
            .get_mut(address)
            .ok_or(ValidatorError::NotFound)?;
        match v.status {
            ValidatorStatus::Unbonding { .. } => return Err(ValidatorError::AlreadyUnbonding),
            ValidatorStatus::Exited | ValidatorStatus::Ejected { .. } => {
                return Err(ValidatorError::AlreadyExited)
            }
            _ => {}
        }
        let exit_epoch = self.current_epoch;
        v.status = ValidatorStatus::Unbonding { exit_epoch };
        Ok(exit_epoch + UNBONDING_EPOCHS)
    }

    /// Complete exit after unbonding period — returns stake
    pub fn complete_exit(&mut self, address: &str) -> Result<u64, ValidatorError> {
        let v = self
            .validators
            .get_mut(address)
            .ok_or(ValidatorError::NotFound)?;
        match v.status {
            ValidatorStatus::Unbonding { exit_epoch } => {
                if self.current_epoch < exit_epoch + UNBONDING_EPOCHS {
                    return Err(ValidatorError::StillUnbonding);
                }
                let stake = v.stake;
                v.stake = 0;
                v.status = ValidatorStatus::Exited;
                Ok(stake)
            }
            ValidatorStatus::Ejected { .. } => {
                let stake = v.stake;
                v.stake = 0;
                v.status = ValidatorStatus::Exited;
                Ok(stake)
            }
            _ => Err(ValidatorError::NotActive),
        }
    }

    /// Record a block produced by a validator
    pub fn record_block(&mut self, address: &str) -> Result<(), ValidatorError> {
        let v = self
            .validators
            .get_mut(address)
            .ok_or(ValidatorError::NotFound)?;
        v.blocks_produced += 1;
        v.consecutive_misses = 0;
        Ok(())
    }

    /// Record a missed slot
    pub fn record_miss(&mut self, address: &str) -> Result<bool, ValidatorError> {
        let v = self
            .validators
            .get_mut(address)
            .ok_or(ValidatorError::NotFound)?;
        v.consecutive_misses += 1;
        if v.consecutive_misses >= DOWNTIME_THRESHOLD {
            v.status = ValidatorStatus::Ejected {
                reason: EjectionReason::Downtime,
                epoch: self.current_epoch,
            };
            return Ok(true); // ejected
        }
        Ok(false)
    }

    /// Slash a validator (reduce stake, eject)
    pub fn slash(&mut self, address: &str, penalty: u64) -> Result<u64, ValidatorError> {
        let v = self
            .validators
            .get_mut(address)
            .ok_or(ValidatorError::NotFound)?;
        let actual = penalty.min(v.stake);
        v.stake -= actual;
        v.status = ValidatorStatus::Ejected {
            reason: EjectionReason::Slashed,
            epoch: self.current_epoch,
        };
        Ok(actual)
    }

    /// Update reputation (clamped to 0.0–1.0)
    pub fn update_reputation(&mut self, address: &str, rep: f64) -> Result<(), ValidatorError> {
        if rep < 0.0 || rep > 1.0 {
            return Err(ValidatorError::InvalidReputation);
        }
        let v = self
            .validators
            .get_mut(address)
            .ok_or(ValidatorError::NotFound)?;
        v.reputation = rep;
        Ok(())
    }

    /// Transition to next epoch: select top candidates by score
    pub fn advance_epoch(&mut self) -> EpochRecord {
        self.current_epoch += 1;

        // Eject active validators with insufficient stake
        let addresses: Vec<String> = self.validators.keys().cloned().collect();
        for addr in &addresses {
            let v = self.validators.get_mut(addr).unwrap();
            if matches!(v.status, ValidatorStatus::Active) && v.stake < MIN_VALIDATOR_STAKE {
                v.status = ValidatorStatus::Ejected {
                    reason: EjectionReason::InsufficientStake,
                    epoch: self.current_epoch,
                };
            }
        }

        // Collect eligible candidates (Candidate or Active with sufficient stake)
        let max_stake = self
            .validators
            .values()
            .filter(|v| {
                matches!(
                    v.status,
                    ValidatorStatus::Candidate | ValidatorStatus::Active
                ) && v.stake >= MIN_VALIDATOR_STAKE
            })
            .map(|v| v.stake)
            .max()
            .unwrap_or(0);

        let mut eligible: Vec<(String, f64)> = self
            .validators
            .iter()
            .filter(|(_, v)| {
                matches!(
                    v.status,
                    ValidatorStatus::Candidate | ValidatorStatus::Active
                ) && v.stake >= MIN_VALIDATOR_STAKE
            })
            .map(|(addr, v)| (addr.clone(), v.score(max_stake)))
            .collect();

        // Sort descending by score, tie-break by address
        eligible.sort_by(|a, b| {
            b.1.partial_cmp(&a.1)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then_with(|| a.0.cmp(&b.0))
        });

        let active_set: Vec<String> = eligible
            .iter()
            .take(MAX_ACTIVE_VALIDATORS)
            .map(|(addr, _)| addr.clone())
            .collect();

        let active_addrs: HashSet<&str> = active_set.iter().map(|s| s.as_str()).collect();

        // Update statuses
        for (addr, v) in self.validators.iter_mut() {
            match v.status {
                ValidatorStatus::Candidate | ValidatorStatus::Active => {
                    if active_addrs.contains(addr.as_str()) {
                        v.status = ValidatorStatus::Active;
                    } else if v.stake >= MIN_VALIDATOR_STAKE {
                        v.status = ValidatorStatus::Candidate;
                    }
                }
                _ => {}
            }
        }

        let total_stake: u64 = active_set
            .iter()
            .filter_map(|a| self.validators.get(a))
            .map(|v| v.stake)
            .sum();

        let record = EpochRecord {
            epoch: self.current_epoch,
            active_set: active_set.clone(),
            total_stake,
        };
        self.epoch_history.push(record.clone());
        record
    }

    pub fn get(&self, address: &str) -> Option<&Validator> {
        self.validators.get(address)
    }

    pub fn current_epoch(&self) -> u64 {
        self.current_epoch
    }

    pub fn active_count(&self) -> usize {
        self.validators
            .values()
            .filter(|v| matches!(v.status, ValidatorStatus::Active))
            .count()
    }

    pub fn epoch_history(&self) -> &[EpochRecord] {
        &self.epoch_history
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn addr(n: u32) -> String {
        format!("validator_{n}")
    }

    #[test]
    fn test_register_and_advance() {
        let mut vs = ValidatorSet::new();
        vs.register(&addr(1), 200_000, 100).unwrap();
        vs.register(&addr(2), 150_000, 80).unwrap();
        let rec = vs.advance_epoch();
        assert_eq!(rec.active_set.len(), 2);
        assert_eq!(rec.total_stake, 350_000);
        assert_eq!(vs.get(&addr(1)).unwrap().status, ValidatorStatus::Active);
    }

    #[test]
    fn test_insufficient_stake_rejected() {
        let mut vs = ValidatorSet::new();
        let err = vs.register(&addr(1), 50_000, 100).unwrap_err();
        assert_eq!(err, ValidatorError::InsufficientStake);
    }

    #[test]
    fn test_duplicate_registration() {
        let mut vs = ValidatorSet::new();
        vs.register(&addr(1), 200_000, 100).unwrap();
        let err = vs.register(&addr(1), 200_000, 100).unwrap_err();
        assert_eq!(err, ValidatorError::AlreadyRegistered);
    }

    #[test]
    fn test_voluntary_exit_and_unbonding() {
        let mut vs = ValidatorSet::new();
        vs.register(&addr(1), 200_000, 100).unwrap();
        vs.advance_epoch();
        let ready_epoch = vs.begin_exit(&addr(1)).unwrap();
        assert_eq!(ready_epoch, 1 + UNBONDING_EPOCHS);

        // Can't complete yet
        let err = vs.complete_exit(&addr(1)).unwrap_err();
        assert_eq!(err, ValidatorError::StillUnbonding);

        // Advance past unbonding
        for _ in 0..UNBONDING_EPOCHS {
            vs.advance_epoch();
        }
        let stake = vs.complete_exit(&addr(1)).unwrap();
        assert_eq!(stake, 200_000);
        assert_eq!(vs.get(&addr(1)).unwrap().status, ValidatorStatus::Exited);
    }

    #[test]
    fn test_downtime_ejection() {
        let mut vs = ValidatorSet::new();
        vs.register(&addr(1), 200_000, 100).unwrap();
        vs.advance_epoch();

        for i in 0..DOWNTIME_THRESHOLD - 1 {
            let ejected = vs.record_miss(&addr(1)).unwrap();
            assert!(!ejected, "should not eject on miss {i}");
        }
        let ejected = vs.record_miss(&addr(1)).unwrap();
        assert!(ejected);
        assert!(matches!(
            vs.get(&addr(1)).unwrap().status,
            ValidatorStatus::Ejected {
                reason: EjectionReason::Downtime,
                ..
            }
        ));
    }

    #[test]
    fn test_slash() {
        let mut vs = ValidatorSet::new();
        vs.register(&addr(1), 200_000, 100).unwrap();
        vs.advance_epoch();
        let slashed = vs.slash(&addr(1), 50_000).unwrap();
        assert_eq!(slashed, 50_000);
        assert_eq!(vs.get(&addr(1)).unwrap().stake, 150_000);
        assert!(matches!(
            vs.get(&addr(1)).unwrap().status,
            ValidatorStatus::Ejected {
                reason: EjectionReason::Slashed,
                ..
            }
        ));
    }

    #[test]
    fn test_slash_capped_at_stake() {
        let mut vs = ValidatorSet::new();
        vs.register(&addr(1), 200_000, 100).unwrap();
        let slashed = vs.slash(&addr(1), 999_999).unwrap();
        assert_eq!(slashed, 200_000);
        assert_eq!(vs.get(&addr(1)).unwrap().stake, 0);
    }

    #[test]
    fn test_add_stake() {
        let mut vs = ValidatorSet::new();
        vs.register(&addr(1), 200_000, 100).unwrap();
        let total = vs.add_stake(&addr(1), 50_000).unwrap();
        assert_eq!(total, 250_000);
    }

    #[test]
    fn test_max_active_validators() {
        let mut vs = ValidatorSet::new();
        for i in 0..150 {
            vs.register(&addr(i), MIN_VALIDATOR_STAKE + i as u64 * 1000, 100)
                .unwrap();
        }
        let rec = vs.advance_epoch();
        assert_eq!(rec.active_set.len(), MAX_ACTIVE_VALIDATORS);
        // Highest-staked should be in
        assert!(rec.active_set.contains(&addr(149)));
        // Lowest-staked should be out
        assert!(!rec.active_set.contains(&addr(0)));
    }

    #[test]
    fn test_score_hybrid() {
        let v = Validator {
            address: "test".into(),
            stake: 500_000,
            reputation: 0.9,
            status: ValidatorStatus::Active,
            registered_epoch: 0,
            consecutive_misses: 0,
            blocks_produced: 10,
            capacity: 100,
        };
        let score = v.score(1_000_000);
        // 0.7 * 0.5 + 0.3 * 0.9 = 0.35 + 0.27 = 0.62
        assert!((score - 0.62).abs() < 0.001);
    }

    #[test]
    fn test_reputation_update() {
        let mut vs = ValidatorSet::new();
        vs.register(&addr(1), 200_000, 100).unwrap();
        vs.update_reputation(&addr(1), 0.95).unwrap();
        assert!((vs.get(&addr(1)).unwrap().reputation - 0.95).abs() < 0.001);

        let err = vs.update_reputation(&addr(1), 1.5).unwrap_err();
        assert_eq!(err, ValidatorError::InvalidReputation);
    }

    #[test]
    fn test_record_block_resets_misses() {
        let mut vs = ValidatorSet::new();
        vs.register(&addr(1), 200_000, 100).unwrap();
        vs.advance_epoch();
        vs.record_miss(&addr(1)).unwrap();
        vs.record_miss(&addr(1)).unwrap();
        assert_eq!(vs.get(&addr(1)).unwrap().consecutive_misses, 2);
        vs.record_block(&addr(1)).unwrap();
        assert_eq!(vs.get(&addr(1)).unwrap().consecutive_misses, 0);
        assert_eq!(vs.get(&addr(1)).unwrap().blocks_produced, 1);
    }

    #[test]
    fn test_ejected_not_in_next_epoch() {
        let mut vs = ValidatorSet::new();
        vs.register(&addr(1), 200_000, 100).unwrap();
        vs.register(&addr(2), 200_000, 100).unwrap();
        vs.advance_epoch();
        vs.slash(&addr(1), 50_000).unwrap();
        let rec = vs.advance_epoch();
        assert!(!rec.active_set.contains(&addr(1)));
        assert!(rec.active_set.contains(&addr(2)));
    }

    #[test]
    fn test_insufficient_stake_ejection_on_epoch() {
        let mut vs = ValidatorSet::new();
        vs.register(&addr(1), MIN_VALIDATOR_STAKE, 100).unwrap();
        vs.advance_epoch();
        assert_eq!(vs.get(&addr(1)).unwrap().status, ValidatorStatus::Active);
        // Slash below minimum but don't fully eject via slash (partial)
        // Manually reduce stake by slashing just enough
        vs.slash(&addr(1), MIN_VALIDATOR_STAKE - 1000).unwrap();
        // Re-register to test the epoch-based ejection path
        // Actually slash already ejected. Let's test differently:
        let mut vs2 = ValidatorSet::new();
        vs2.register(&addr(1), MIN_VALIDATOR_STAKE + 10_000, 100)
            .unwrap();
        vs2.advance_epoch();
        // Simulate external stake reduction (e.g., delegation withdrawal)
        vs2.validators.get_mut(&addr(1)).unwrap().stake = MIN_VALIDATOR_STAKE - 1;
        let rec = vs2.advance_epoch();
        assert!(!rec.active_set.contains(&addr(1)));
        assert!(matches!(
            vs2.get(&addr(1)).unwrap().status,
            ValidatorStatus::Ejected {
                reason: EjectionReason::InsufficientStake,
                ..
            }
        ));
    }

    #[test]
    fn test_epoch_history() {
        let mut vs = ValidatorSet::new();
        vs.register(&addr(1), 200_000, 100).unwrap();
        vs.advance_epoch();
        vs.advance_epoch();
        vs.advance_epoch();
        assert_eq!(vs.epoch_history().len(), 3);
        assert_eq!(vs.current_epoch(), 3);
    }

    #[test]
    fn test_complete_exit_after_ejection() {
        let mut vs = ValidatorSet::new();
        vs.register(&addr(1), 200_000, 100).unwrap();
        vs.advance_epoch();
        vs.slash(&addr(1), 50_000).unwrap();
        let returned = vs.complete_exit(&addr(1)).unwrap();
        assert_eq!(returned, 150_000);
        assert_eq!(vs.get(&addr(1)).unwrap().status, ValidatorStatus::Exited);
    }
}
