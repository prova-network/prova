//! Inference Commit — providers publish activation roots after inference.
//!
//! Flow:
//! 1. Provider runs inference on a registered model
//! 2. Provider publishes an InferenceCommit with the activation Merkle root
//! 3. A challenge window opens (e.g., 240 epochs ≈ 2 hours)
//! 4. If unchallenged, the commit is finalized
//! 5. If challenged, a bisection dispute begins

use std::collections::HashMap;
use crate::types::*;

/// Status of an inference commit.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CommitStatus {
    /// Within challenge window — can be disputed.
    Open,
    /// Challenge window expired, no disputes — finalized.
    Finalized,
    /// Under active dispute via bisection game.
    Disputed,
    /// Slashed — provider proved wrong.
    Slashed,
    /// Challenge failed — challenger lost stake.
    Defended,
}

/// An inference commit published by a provider.
#[derive(Debug, Clone)]
pub struct InferenceCommit {
    /// Unique commit identifier.
    pub id: CommitId,
    /// Who performed the inference.
    pub provider: Address,
    /// Which model was used.
    pub model_id: ModelId,
    /// Architecture group the inference ran on.
    pub arch_group: ArchGroup,
    /// SHA-256 of the input prompt/tokens.
    pub input_hash: Hash,
    /// Activation Merkle root (covers input + all layer outputs).
    pub activation_root: Hash,
    /// Number of layers (must match model's layer_count + 1 for input).
    pub leaf_count: u32,
    /// Epoch when committed.
    pub committed_at: Epoch,
    /// Current status.
    pub status: CommitStatus,
}

/// Configuration for the commit system.
#[derive(Debug, Clone)]
pub struct CommitConfig {
    /// How many epochs the challenge window stays open.
    pub challenge_window: EpochDuration,
    /// Minimum stake required to commit.
    pub min_provider_stake: StakeAmount,
    /// Minimum stake required to challenge.
    pub min_challenger_stake: StakeAmount,
}

impl Default for CommitConfig {
    fn default() -> Self {
        Self {
            challenge_window: 240, // ~2 hours at 30s epochs
            min_provider_stake: 1_000_000,
            min_challenger_stake: 500_000,
        }
    }
}

/// The on-chain commit store.
#[derive(Debug)]
pub struct CommitStore {
    commits: HashMap<CommitId, InferenceCommit>,
    next_id: u64,
    config: CommitConfig,
}

impl CommitStore {
    pub fn new(config: CommitConfig) -> Self {
        Self {
            commits: HashMap::new(),
            next_id: 1,
            config,
        }
    }

    /// Publish a new inference commit. Returns the commit ID.
    pub fn publish(
        &mut self,
        provider: Address,
        model_id: ModelId,
        arch_group: ArchGroup,
        input_hash: Hash,
        activation_root: Hash,
        leaf_count: u32,
        current_epoch: Epoch,
    ) -> CommitId {
        let id = CommitId(self.next_id);
        self.next_id += 1;

        let commit = InferenceCommit {
            id,
            provider,
            model_id,
            arch_group,
            input_hash,
            activation_root,
            leaf_count,
            committed_at: current_epoch,
            status: CommitStatus::Open,
        };

        self.commits.insert(id, commit);
        id
    }

    /// Get a commit by ID.
    pub fn get(&self, id: &CommitId) -> Option<&InferenceCommit> {
        self.commits.get(id)
    }

    /// Get a mutable commit by ID.
    pub fn get_mut(&mut self, id: &CommitId) -> Option<&mut InferenceCommit> {
        self.commits.get_mut(id)
    }

    /// Check if a commit is still within its challenge window.
    pub fn is_challengeable(&self, id: &CommitId, current_epoch: Epoch) -> bool {
        self.commits
            .get(id)
            .map(|c| {
                c.status == CommitStatus::Open
                    && current_epoch < c.committed_at + self.config.challenge_window
            })
            .unwrap_or(false)
    }

    /// Finalize all commits whose challenge window has expired.
    /// Returns the number of commits finalized.
    pub fn finalize_expired(&mut self, current_epoch: Epoch) -> usize {
        let mut finalized = 0;
        for commit in self.commits.values_mut() {
            if commit.status == CommitStatus::Open
                && current_epoch >= commit.committed_at + self.config.challenge_window
            {
                commit.status = CommitStatus::Finalized;
                finalized += 1;
            }
        }
        finalized
    }

    /// Mark a commit as disputed.
    pub fn mark_disputed(&mut self, id: &CommitId) -> Result<(), CommitError> {
        let commit = self
            .commits
            .get_mut(id)
            .ok_or(CommitError::NotFound(*id))?;

        if commit.status != CommitStatus::Open {
            return Err(CommitError::NotChallengeable(*id, commit.status));
        }

        commit.status = CommitStatus::Disputed;
        Ok(())
    }

    /// Mark a commit as slashed (provider was wrong).
    pub fn mark_slashed(&mut self, id: &CommitId) -> Result<(), CommitError> {
        let commit = self
            .commits
            .get_mut(id)
            .ok_or(CommitError::NotFound(*id))?;

        if commit.status != CommitStatus::Disputed {
            return Err(CommitError::InvalidTransition(*id, commit.status));
        }

        commit.status = CommitStatus::Slashed;
        Ok(())
    }

    /// Mark a commit as defended (challenger was wrong).
    pub fn mark_defended(&mut self, id: &CommitId) -> Result<(), CommitError> {
        let commit = self
            .commits
            .get_mut(id)
            .ok_or(CommitError::NotFound(*id))?;

        if commit.status != CommitStatus::Disputed {
            return Err(CommitError::InvalidTransition(*id, commit.status));
        }

        commit.status = CommitStatus::Defended;
        Ok(())
    }

    /// Get the challenge window config.
    pub fn challenge_window(&self) -> EpochDuration {
        self.config.challenge_window
    }

    /// Total commits.
    pub fn commit_count(&self) -> usize {
        self.commits.len()
    }
}

#[derive(Debug)]
pub enum CommitError {
    NotFound(CommitId),
    NotChallengeable(CommitId, CommitStatus),
    InvalidTransition(CommitId, CommitStatus),
}

impl std::fmt::Display for CommitError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NotFound(id) => write!(f, "commit {id} not found"),
            Self::NotChallengeable(id, status) => {
                write!(f, "commit {id} is not challengeable (status: {status:?})")
            }
            Self::InvalidTransition(id, status) => {
                write!(f, "invalid state transition for {id} (current: {status:?})")
            }
        }
    }
}

impl std::error::Error for CommitError {}

#[cfg(test)]
mod tests {
    use super::*;

    fn setup() -> CommitStore {
        CommitStore::new(CommitConfig {
            challenge_window: 100,
            min_provider_stake: 1000,
            min_challenger_stake: 500,
        })
    }

    #[test]
    fn test_publish_and_get() {
        let mut store = setup();
        let id = store.publish(
            Address::test(1),
            ModelId([0xAA; 32]),
            ArchGroup::new("nvidia-sm89-int8"),
            [0xBB; 32],
            [0xCC; 32],
            33,
            1000,
        );

        let commit = store.get(&id).unwrap();
        assert_eq!(commit.status, CommitStatus::Open);
        assert_eq!(commit.provider, Address::test(1));
        assert_eq!(commit.committed_at, 1000);
    }

    #[test]
    fn test_challenge_window() {
        let mut store = setup();
        let id = store.publish(
            Address::test(1),
            ModelId([0xAA; 32]),
            ArchGroup::new("test"),
            [0; 32],
            [0; 32],
            33,
            1000,
        );

        // Within window
        assert!(store.is_challengeable(&id, 1050));
        assert!(store.is_challengeable(&id, 1099));

        // At/past window
        assert!(!store.is_challengeable(&id, 1100));
        assert!(!store.is_challengeable(&id, 1200));
    }

    #[test]
    fn test_finalize_expired() {
        let mut store = setup();
        let id1 = store.publish(
            Address::test(1),
            ModelId([0xAA; 32]),
            ArchGroup::new("test"),
            [0; 32],
            [0; 32],
            33,
            1000,
        );
        let id2 = store.publish(
            Address::test(2),
            ModelId([0xBB; 32]),
            ArchGroup::new("test"),
            [1; 32],
            [1; 32],
            33,
            1050, // 50 epochs later
        );

        // At epoch 1100: id1 should finalize, id2 still open
        let finalized = store.finalize_expired(1100);
        assert_eq!(finalized, 1);
        assert_eq!(store.get(&id1).unwrap().status, CommitStatus::Finalized);
        assert_eq!(store.get(&id2).unwrap().status, CommitStatus::Open);

        // At epoch 1150: id2 also finalizes
        let finalized = store.finalize_expired(1150);
        assert_eq!(finalized, 1);
        assert_eq!(store.get(&id2).unwrap().status, CommitStatus::Finalized);
    }

    #[test]
    fn test_dispute_lifecycle() {
        let mut store = setup();
        let id = store.publish(
            Address::test(1),
            ModelId([0xAA; 32]),
            ArchGroup::new("test"),
            [0; 32],
            [0; 32],
            33,
            1000,
        );

        // Challenge
        store.mark_disputed(&id).unwrap();
        assert_eq!(store.get(&id).unwrap().status, CommitStatus::Disputed);

        // Can't challenge again
        assert!(store.mark_disputed(&id).is_err());

        // Resolve: provider was wrong
        store.mark_slashed(&id).unwrap();
        assert_eq!(store.get(&id).unwrap().status, CommitStatus::Slashed);
    }

    #[test]
    fn test_defended_path() {
        let mut store = setup();
        let id = store.publish(
            Address::test(1),
            ModelId([0xAA; 32]),
            ArchGroup::new("test"),
            [0; 32],
            [0; 32],
            33,
            1000,
        );

        store.mark_disputed(&id).unwrap();
        store.mark_defended(&id).unwrap();
        assert_eq!(store.get(&id).unwrap().status, CommitStatus::Defended);
    }
}
