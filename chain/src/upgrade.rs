//! Protocol Upgrade Mechanism — fork scheduling, version negotiation, and activation.
//!
//! Supports:
//! - Named protocol versions with activation epochs
//! - Signaling by validators (miners/providers signal readiness)
//! - Threshold-based activation (e.g., 80% of stake signals → schedule fork)
//! - Hard fork and soft fork semantics
//! - Emergency upgrades (governance override)
//! - Version negotiation for P2P compatibility

use crate::types::*;
use std::collections::{BTreeMap, HashMap, HashSet};

/// Protocol version identifier.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ProtocolVersion {
    pub major: u32,
    pub minor: u32,
    pub patch: u32,
}

impl ProtocolVersion {
    pub fn new(major: u32, minor: u32, patch: u32) -> Self {
        Self {
            major,
            minor,
            patch,
        }
    }

    /// Check if this version is compatible with another (same major).
    pub fn is_compatible(&self, other: &Self) -> bool {
        self.major == other.major
    }

    pub fn to_string(&self) -> String {
        format!("{}.{}.{}", self.major, self.minor, self.patch)
    }
}

impl std::fmt::Display for ProtocolVersion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}.{}.{}", self.major, self.minor, self.patch)
    }
}

/// Type of protocol upgrade.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ForkType {
    /// Breaking change — old nodes cannot validate new blocks.
    Hard,
    /// Backward-compatible — old nodes still validate but don't use new features.
    Soft,
}

/// Lifecycle state of an upgrade proposal.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UpgradeState {
    /// Proposed but not yet signaling.
    Proposed,
    /// Actively collecting signals from validators.
    Signaling,
    /// Threshold met, activation epoch locked in.
    LockedIn,
    /// Activation epoch reached, upgrade is live.
    Active,
    /// Failed to reach threshold before deadline.
    Failed,
    /// Cancelled by governance.
    Cancelled,
}

/// A protocol upgrade proposal.
#[derive(Debug, Clone)]
pub struct UpgradeProposal {
    pub id: u64,
    pub name: String,
    pub version: ProtocolVersion,
    pub fork_type: ForkType,
    pub description: String,
    /// Epoch when signaling starts.
    pub signal_start: Epoch,
    /// Epoch when signaling must complete (deadline).
    pub signal_deadline: Epoch,
    /// Minimum stake fraction required (basis points, e.g., 8000 = 80%).
    pub threshold_bps: u32,
    /// Grace period (epochs) between lock-in and activation.
    pub activation_delay: EpochDuration,
    /// Current state.
    pub state: UpgradeState,
    /// Activation epoch (set when locked in).
    pub activation_epoch: Option<Epoch>,
    /// Who proposed it.
    pub proposer: Address,
    /// Whether this is an emergency upgrade (bypasses normal signaling).
    pub emergency: bool,
}

/// Signal from a validator expressing readiness.
#[derive(Debug, Clone)]
pub struct Signal {
    pub validator: Address,
    pub proposal_id: u64,
    pub epoch: Epoch,
}

/// Result of a version negotiation between peers.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NegotiationResult {
    /// Both peers agree on version.
    Compatible(ProtocolVersion),
    /// Peers are incompatible — connection should be dropped.
    Incompatible {
        local: ProtocolVersion,
        remote: ProtocolVersion,
    },
    /// Remote is ahead — local should upgrade.
    UpgradeRequired {
        local: ProtocolVersion,
        remote: ProtocolVersion,
    },
}

/// Manages protocol upgrades.
pub struct UpgradeManager {
    current_version: ProtocolVersion,
    proposals: BTreeMap<u64, UpgradeProposal>,
    /// proposal_id → set of signaling validators
    signals: HashMap<u64, HashSet<Address>>,
    /// validator → stake weight (for threshold calc)
    stake_weights: HashMap<Address, StakeAmount>,
    /// History of activated upgrades (version → epoch).
    activated: BTreeMap<ProtocolVersion, Epoch>,
    next_id: u64,
}

impl UpgradeManager {
    pub fn new(genesis_version: ProtocolVersion) -> Self {
        let mut activated = BTreeMap::new();
        activated.insert(genesis_version.clone(), 0);
        Self {
            current_version: genesis_version,
            proposals: BTreeMap::new(),
            signals: HashMap::new(),
            stake_weights: HashMap::new(),
            activated,
            next_id: 1,
        }
    }

    pub fn current_version(&self) -> &ProtocolVersion {
        &self.current_version
    }

    /// Register or update a validator's stake weight.
    pub fn set_stake(&mut self, validator: Address, stake: StakeAmount) {
        self.stake_weights.insert(validator, stake);
    }

    /// Total stake across all validators.
    pub fn total_stake(&self) -> StakeAmount {
        self.stake_weights.values().sum()
    }

    /// Propose a new protocol upgrade.
    pub fn propose(
        &mut self,
        name: &str,
        version: ProtocolVersion,
        fork_type: ForkType,
        description: &str,
        signal_start: Epoch,
        signal_deadline: Epoch,
        threshold_bps: u32,
        activation_delay: EpochDuration,
        proposer: Address,
        emergency: bool,
    ) -> Result<u64, UpgradeError> {
        if signal_deadline <= signal_start {
            return Err(UpgradeError::InvalidWindow);
        }
        if threshold_bps > 10000 {
            return Err(UpgradeError::InvalidThreshold);
        }
        // Check version isn't already proposed or active.
        for p in self.proposals.values() {
            if p.version == version
                && p.state != UpgradeState::Failed
                && p.state != UpgradeState::Cancelled
            {
                return Err(UpgradeError::DuplicateVersion);
            }
        }

        let id = self.next_id;
        self.next_id += 1;

        let state = if emergency {
            UpgradeState::Signaling
        } else {
            UpgradeState::Proposed
        };

        self.proposals.insert(
            id,
            UpgradeProposal {
                id,
                name: name.to_string(),
                version,
                fork_type,
                description: description.to_string(),
                signal_start,
                signal_deadline,
                threshold_bps,
                activation_delay,
                state,
                activation_epoch: None,
                proposer,
                emergency,
            },
        );

        self.signals.insert(id, HashSet::new());
        Ok(id)
    }

    /// Validator signals readiness for an upgrade.
    pub fn signal(
        &mut self,
        validator: Address,
        proposal_id: u64,
        epoch: Epoch,
    ) -> Result<(), UpgradeError> {
        let proposal = self
            .proposals
            .get(&proposal_id)
            .ok_or(UpgradeError::NotFound)?;

        if proposal.state != UpgradeState::Signaling {
            return Err(UpgradeError::NotSignaling);
        }
        if epoch > proposal.signal_deadline {
            return Err(UpgradeError::DeadlinePassed);
        }
        if !self.stake_weights.contains_key(&validator) {
            return Err(UpgradeError::NotValidator);
        }

        self.signals
            .get_mut(&proposal_id)
            .unwrap()
            .insert(validator);
        Ok(())
    }

    /// Get signal progress for a proposal (basis points).
    pub fn signal_progress(&self, proposal_id: u64) -> Result<u32, UpgradeError> {
        let _proposal = self
            .proposals
            .get(&proposal_id)
            .ok_or(UpgradeError::NotFound)?;

        let signalers = self.signals.get(&proposal_id).unwrap();
        let total = self.total_stake();
        if total == 0 {
            return Ok(0);
        }

        let signaled_stake: StakeAmount = signalers
            .iter()
            .filter_map(|v| self.stake_weights.get(v))
            .sum();

        Ok(((signaled_stake * 10000) / total) as u32)
    }

    /// Process epoch — advance proposal states.
    pub fn tick(&mut self, epoch: Epoch) -> Vec<UpgradeEvent> {
        let mut events = Vec::new();
        let proposal_ids: Vec<u64> = self.proposals.keys().cloned().collect();

        for id in proposal_ids {
            let proposal = self.proposals.get_mut(&id).unwrap();

            match proposal.state {
                UpgradeState::Proposed if epoch >= proposal.signal_start => {
                    proposal.state = UpgradeState::Signaling;
                    events.push(UpgradeEvent::SignalingStarted { proposal_id: id });
                }
                UpgradeState::Signaling => {
                    // Check threshold.
                    let progress = {
                        let signalers = self.signals.get(&id).unwrap();
                        let total: StakeAmount = self.stake_weights.values().sum();
                        if total == 0 {
                            0
                        } else {
                            ((signalers
                                .iter()
                                .filter_map(|v| self.stake_weights.get(v))
                                .sum::<StakeAmount>()
                                * 10000)
                                / total) as u32
                        }
                    };

                    if progress >= proposal.threshold_bps {
                        let activation = epoch + proposal.activation_delay;
                        proposal.state = UpgradeState::LockedIn;
                        proposal.activation_epoch = Some(activation);
                        events.push(UpgradeEvent::LockedIn {
                            proposal_id: id,
                            activation_epoch: activation,
                        });
                    } else if epoch >= proposal.signal_deadline {
                        proposal.state = UpgradeState::Failed;
                        events.push(UpgradeEvent::Failed { proposal_id: id });
                    }
                }
                UpgradeState::LockedIn => {
                    if let Some(act_epoch) = proposal.activation_epoch {
                        if epoch >= act_epoch {
                            proposal.state = UpgradeState::Active;
                            self.current_version = proposal.version.clone();
                            self.activated.insert(proposal.version.clone(), epoch);
                            events.push(UpgradeEvent::Activated {
                                proposal_id: id,
                                version: proposal.version.clone(),
                            });
                        }
                    }
                }
                _ => {}
            }
        }
        events
    }

    /// Emergency activation by governance — skips signaling.
    pub fn emergency_activate(
        &mut self,
        proposal_id: u64,
        epoch: Epoch,
    ) -> Result<UpgradeEvent, UpgradeError> {
        let proposal = self
            .proposals
            .get_mut(&proposal_id)
            .ok_or(UpgradeError::NotFound)?;

        if !proposal.emergency {
            return Err(UpgradeError::NotEmergency);
        }

        proposal.state = UpgradeState::Active;
        proposal.activation_epoch = Some(epoch);
        self.current_version = proposal.version.clone();
        self.activated.insert(proposal.version.clone(), epoch);

        Ok(UpgradeEvent::Activated {
            proposal_id,
            version: proposal.version.clone(),
        })
    }

    /// Cancel a proposal (governance action).
    pub fn cancel(&mut self, proposal_id: u64) -> Result<(), UpgradeError> {
        let proposal = self
            .proposals
            .get_mut(&proposal_id)
            .ok_or(UpgradeError::NotFound)?;

        match proposal.state {
            UpgradeState::Active => Err(UpgradeError::AlreadyActive),
            UpgradeState::Failed | UpgradeState::Cancelled => Err(UpgradeError::AlreadyTerminal),
            _ => {
                proposal.state = UpgradeState::Cancelled;
                Ok(())
            }
        }
    }

    /// Negotiate version with a remote peer.
    pub fn negotiate(&self, remote_version: &ProtocolVersion) -> NegotiationResult {
        if self.current_version == *remote_version {
            NegotiationResult::Compatible(self.current_version.clone())
        } else if !self.current_version.is_compatible(remote_version) {
            NegotiationResult::Incompatible {
                local: self.current_version.clone(),
                remote: remote_version.clone(),
            }
        } else if remote_version > &self.current_version {
            NegotiationResult::UpgradeRequired {
                local: self.current_version.clone(),
                remote: remote_version.clone(),
            }
        } else {
            // Remote is behind but same major — still compatible.
            NegotiationResult::Compatible(self.current_version.clone())
        }
    }

    /// Get a proposal by ID.
    pub fn get_proposal(&self, id: u64) -> Option<&UpgradeProposal> {
        self.proposals.get(&id)
    }

    /// List all proposals.
    pub fn proposals(&self) -> Vec<&UpgradeProposal> {
        self.proposals.values().collect()
    }

    /// Get activation history.
    pub fn activation_history(&self) -> &BTreeMap<ProtocolVersion, Epoch> {
        &self.activated
    }
}

/// Events emitted during upgrade processing.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UpgradeEvent {
    SignalingStarted {
        proposal_id: u64,
    },
    LockedIn {
        proposal_id: u64,
        activation_epoch: Epoch,
    },
    Failed {
        proposal_id: u64,
    },
    Activated {
        proposal_id: u64,
        version: ProtocolVersion,
    },
}

/// Errors from upgrade operations.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UpgradeError {
    InvalidWindow,
    InvalidThreshold,
    DuplicateVersion,
    NotFound,
    NotSignaling,
    DeadlinePassed,
    NotValidator,
    NotEmergency,
    AlreadyActive,
    AlreadyTerminal,
}

impl std::fmt::Display for UpgradeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidWindow => write!(f, "signal deadline must be after start"),
            Self::InvalidThreshold => write!(f, "threshold must be <= 10000 bps"),
            Self::DuplicateVersion => write!(f, "version already proposed"),
            Self::NotFound => write!(f, "proposal not found"),
            Self::NotSignaling => write!(f, "proposal not in signaling state"),
            Self::DeadlinePassed => write!(f, "signaling deadline has passed"),
            Self::NotValidator => write!(f, "address is not a registered validator"),
            Self::NotEmergency => write!(f, "proposal is not an emergency upgrade"),
            Self::AlreadyActive => write!(f, "upgrade already active"),
            Self::AlreadyTerminal => write!(f, "proposal already in terminal state"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn setup() -> UpgradeManager {
        let mut mgr = UpgradeManager::new(ProtocolVersion::new(1, 0, 0));
        // 3 validators with equal stake.
        mgr.set_stake(Address::test(1), 1000);
        mgr.set_stake(Address::test(2), 1000);
        mgr.set_stake(Address::test(3), 1000);
        mgr
    }

    #[test]
    fn test_propose_and_list() {
        let mut mgr = setup();
        let id = mgr
            .propose(
                "v1.1.0",
                ProtocolVersion::new(1, 1, 0),
                ForkType::Soft,
                "Minor improvements",
                10,
                100,
                8000,
                50,
                Address::test(1),
                false,
            )
            .unwrap();
        assert_eq!(id, 1);
        assert_eq!(mgr.proposals().len(), 1);
        assert_eq!(mgr.get_proposal(id).unwrap().state, UpgradeState::Proposed);
    }

    #[test]
    fn test_invalid_window() {
        let mut mgr = setup();
        let err = mgr
            .propose(
                "bad",
                ProtocolVersion::new(2, 0, 0),
                ForkType::Hard,
                "Bad window",
                100,
                50,
                8000,
                10,
                Address::test(1),
                false,
            )
            .unwrap_err();
        assert_eq!(err, UpgradeError::InvalidWindow);
    }

    #[test]
    fn test_signaling_lifecycle() {
        let mut mgr = setup();
        let id = mgr
            .propose(
                "v1.1.0",
                ProtocolVersion::new(1, 1, 0),
                ForkType::Soft,
                "Test",
                10,
                100,
                6000,
                20,
                Address::test(1),
                false,
            )
            .unwrap();

        // Before signal_start — still proposed.
        let events = mgr.tick(5);
        assert!(events.is_empty());

        // At signal_start — transitions to signaling.
        let events = mgr.tick(10);
        assert_eq!(
            events,
            vec![UpgradeEvent::SignalingStarted { proposal_id: id }]
        );

        // Signal from 2/3 validators (66.6% > 60% threshold).
        mgr.signal(Address::test(1), id, 15).unwrap();
        mgr.signal(Address::test(2), id, 15).unwrap();

        let progress = mgr.signal_progress(id).unwrap();
        assert_eq!(progress, 6666); // 66.66% in bps

        // Tick should lock in.
        let events = mgr.tick(16);
        assert_eq!(events.len(), 1);
        match &events[0] {
            UpgradeEvent::LockedIn {
                proposal_id,
                activation_epoch,
            } => {
                assert_eq!(*proposal_id, id);
                assert_eq!(*activation_epoch, 36); // 16 + 20
            }
            _ => panic!("expected LockedIn"),
        }

        // Tick at activation epoch.
        let events = mgr.tick(36);
        assert_eq!(
            events,
            vec![UpgradeEvent::Activated {
                proposal_id: id,
                version: ProtocolVersion::new(1, 1, 0),
            }]
        );
        assert_eq!(*mgr.current_version(), ProtocolVersion::new(1, 1, 0));
    }

    #[test]
    fn test_signaling_failure() {
        let mut mgr = setup();
        let id = mgr
            .propose(
                "v2.0.0",
                ProtocolVersion::new(2, 0, 0),
                ForkType::Hard,
                "Hard fork",
                10,
                50,
                8000,
                20,
                Address::test(1),
                false,
            )
            .unwrap();

        mgr.tick(10); // start signaling
                      // Only 1/3 signals (33% < 80%).
        mgr.signal(Address::test(1), id, 15).unwrap();

        // At deadline — fails.
        let events = mgr.tick(50);
        assert_eq!(events, vec![UpgradeEvent::Failed { proposal_id: id }]);
        assert_eq!(mgr.get_proposal(id).unwrap().state, UpgradeState::Failed);
    }

    #[test]
    fn test_signal_not_validator() {
        let mut mgr = setup();
        let id = mgr
            .propose(
                "v1.1.0",
                ProtocolVersion::new(1, 1, 0),
                ForkType::Soft,
                "Test",
                0,
                100,
                8000,
                20,
                Address::test(1),
                false,
            )
            .unwrap();
        mgr.tick(0);
        let err = mgr.signal(Address::test(99), id, 5).unwrap_err();
        assert_eq!(err, UpgradeError::NotValidator);
    }

    #[test]
    fn test_signal_after_deadline() {
        let mut mgr = setup();
        let id = mgr
            .propose(
                "v1.1.0",
                ProtocolVersion::new(1, 1, 0),
                ForkType::Soft,
                "Test",
                0,
                50,
                8000,
                20,
                Address::test(1),
                false,
            )
            .unwrap();
        mgr.tick(0); // start signaling
        let err = mgr.signal(Address::test(1), id, 51).unwrap_err();
        assert_eq!(err, UpgradeError::DeadlinePassed);
    }

    #[test]
    fn test_emergency_activation() {
        let mut mgr = setup();
        let id = mgr
            .propose(
                "emergency-fix",
                ProtocolVersion::new(1, 0, 1),
                ForkType::Hard,
                "Critical fix",
                0,
                100,
                8000,
                20,
                Address::test(1),
                true,
            )
            .unwrap();

        let event = mgr.emergency_activate(id, 5).unwrap();
        assert_eq!(
            event,
            UpgradeEvent::Activated {
                proposal_id: id,
                version: ProtocolVersion::new(1, 0, 1),
            }
        );
        assert_eq!(*mgr.current_version(), ProtocolVersion::new(1, 0, 1));
    }

    #[test]
    fn test_emergency_on_non_emergency() {
        let mut mgr = setup();
        let id = mgr
            .propose(
                "normal",
                ProtocolVersion::new(1, 1, 0),
                ForkType::Soft,
                "Normal",
                10,
                100,
                8000,
                20,
                Address::test(1),
                false,
            )
            .unwrap();
        let err = mgr.emergency_activate(id, 5).unwrap_err();
        assert_eq!(err, UpgradeError::NotEmergency);
    }

    #[test]
    fn test_cancel_proposal() {
        let mut mgr = setup();
        let id = mgr
            .propose(
                "v1.1.0",
                ProtocolVersion::new(1, 1, 0),
                ForkType::Soft,
                "Test",
                10,
                100,
                8000,
                20,
                Address::test(1),
                false,
            )
            .unwrap();
        mgr.cancel(id).unwrap();
        assert_eq!(mgr.get_proposal(id).unwrap().state, UpgradeState::Cancelled);
    }

    #[test]
    fn test_cancel_active_fails() {
        let mut mgr = setup();
        let id = mgr
            .propose(
                "e",
                ProtocolVersion::new(1, 0, 1),
                ForkType::Hard,
                "Fix",
                0,
                100,
                8000,
                20,
                Address::test(1),
                true,
            )
            .unwrap();
        mgr.emergency_activate(id, 0).unwrap();
        assert_eq!(mgr.cancel(id).unwrap_err(), UpgradeError::AlreadyActive);
    }

    #[test]
    fn test_duplicate_version_rejected() {
        let mut mgr = setup();
        mgr.propose(
            "v1.1.0",
            ProtocolVersion::new(1, 1, 0),
            ForkType::Soft,
            "First",
            10,
            100,
            8000,
            20,
            Address::test(1),
            false,
        )
        .unwrap();
        let err = mgr
            .propose(
                "v1.1.0-dup",
                ProtocolVersion::new(1, 1, 0),
                ForkType::Hard,
                "Dup",
                10,
                100,
                8000,
                20,
                Address::test(2),
                false,
            )
            .unwrap_err();
        assert_eq!(err, UpgradeError::DuplicateVersion);
    }

    #[test]
    fn test_version_negotiation_compatible() {
        let mgr = UpgradeManager::new(ProtocolVersion::new(1, 2, 0));
        assert_eq!(
            mgr.negotiate(&ProtocolVersion::new(1, 2, 0)),
            NegotiationResult::Compatible(ProtocolVersion::new(1, 2, 0))
        );
    }

    #[test]
    fn test_version_negotiation_incompatible() {
        let mgr = UpgradeManager::new(ProtocolVersion::new(1, 0, 0));
        assert!(matches!(
            mgr.negotiate(&ProtocolVersion::new(2, 0, 0)),
            NegotiationResult::Incompatible { .. }
        ));
    }

    #[test]
    fn test_version_negotiation_upgrade_required() {
        let mgr = UpgradeManager::new(ProtocolVersion::new(1, 0, 0));
        assert!(matches!(
            mgr.negotiate(&ProtocolVersion::new(1, 2, 0)),
            NegotiationResult::UpgradeRequired { .. }
        ));
    }

    #[test]
    fn test_version_negotiation_remote_behind() {
        let mgr = UpgradeManager::new(ProtocolVersion::new(1, 2, 0));
        // Remote is behind but same major — compatible.
        assert_eq!(
            mgr.negotiate(&ProtocolVersion::new(1, 0, 0)),
            NegotiationResult::Compatible(ProtocolVersion::new(1, 2, 0))
        );
    }

    #[test]
    fn test_activation_history() {
        let mut mgr = setup();
        let id = mgr
            .propose(
                "e",
                ProtocolVersion::new(1, 0, 1),
                ForkType::Hard,
                "Fix",
                0,
                100,
                8000,
                20,
                Address::test(1),
                true,
            )
            .unwrap();
        mgr.emergency_activate(id, 42).unwrap();
        let history = mgr.activation_history();
        assert_eq!(history.len(), 2); // genesis + emergency
        assert_eq!(history[&ProtocolVersion::new(1, 0, 0)], 0);
        assert_eq!(history[&ProtocolVersion::new(1, 0, 1)], 42);
    }

    #[test]
    fn test_stake_weighted_signaling() {
        let mut mgr = UpgradeManager::new(ProtocolVersion::new(1, 0, 0));
        // Whale validator.
        mgr.set_stake(Address::test(1), 9000);
        mgr.set_stake(Address::test(2), 500);
        mgr.set_stake(Address::test(3), 500);

        let id = mgr
            .propose(
                "v1.1.0",
                ProtocolVersion::new(1, 1, 0),
                ForkType::Soft,
                "Whale test",
                0,
                100,
                8000,
                10,
                Address::test(1),
                false,
            )
            .unwrap();
        mgr.tick(0); // start signaling

        // Just the whale signals — 90% > 80%.
        mgr.signal(Address::test(1), id, 1).unwrap();
        let progress = mgr.signal_progress(id).unwrap();
        assert_eq!(progress, 9000); // 90%

        let events = mgr.tick(2);
        assert!(matches!(events[0], UpgradeEvent::LockedIn { .. }));
    }
}
