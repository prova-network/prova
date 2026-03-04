//! Filecoin checkpoint anchoring.
//!
//! Prova periodically commits state root checkpoints to Filecoin L1,
//! enabling light client verification and cross-chain trust anchoring.
//!
//! Design:
//! - Checkpoints are produced every `CHECKPOINT_INTERVAL` Prova epochs
//! - Each checkpoint contains: Prova epoch range, state root, block hash, validator set hash
//! - Checkpoints require 2/3+ validator signatures (weighted by stake)
//! - Anchoring to Filecoin is via a smart contract call (FVM actor)

use crate::types::{Address, Epoch, Hash};
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;

/// Checkpoint interval in Prova epochs.
pub const CHECKPOINT_INTERVAL: Epoch = 120;

/// Minimum stake fraction required to finalize a checkpoint (2/3).
pub const QUORUM_NUMERATOR: u128 = 2;
pub const QUORUM_DENOMINATOR: u128 = 3;

/// A finalized checkpoint ready for L1 anchoring.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Checkpoint {
    /// Sequential checkpoint number.
    pub sequence: u64,
    /// First Prova epoch covered (inclusive).
    pub epoch_start: Epoch,
    /// Last Prova epoch covered (inclusive).
    pub epoch_end: Epoch,
    /// Prova state root at epoch_end.
    pub state_root: Hash,
    /// Block hash at epoch_end.
    pub block_hash: Hash,
    /// Hash of the active validator set.
    pub validator_set_hash: Hash,
    /// Aggregated signatures (validator → signature bytes).
    pub signatures: BTreeMap<Address, Vec<u8>>,
    /// Total stake that signed.
    pub signed_stake: u128,
    /// Total active stake at epoch_end.
    pub total_stake: u128,
}

impl Checkpoint {
    /// Compute the checkpoint digest that validators sign.
    pub fn digest(&self) -> Hash {
        let mut h = Sha256::new();
        h.update(self.sequence.to_le_bytes());
        h.update(self.epoch_start.to_le_bytes());
        h.update(self.epoch_end.to_le_bytes());
        h.update(self.state_root);
        h.update(self.block_hash);
        h.update(self.validator_set_hash);
        h.finalize().into()
    }

    /// Check if quorum is reached.
    pub fn has_quorum(&self) -> bool {
        // signed_stake / total_stake >= 2/3
        // Rearranged to avoid floating point: signed_stake * 3 >= total_stake * 2
        self.total_stake > 0
            && self.signed_stake * QUORUM_DENOMINATOR >= self.total_stake * QUORUM_NUMERATOR
    }
}

/// Pending checkpoint accumulating validator votes.
#[derive(Debug, Clone)]
pub struct PendingCheckpoint {
    pub sequence: u64,
    pub epoch_start: Epoch,
    pub epoch_end: Epoch,
    pub state_root: Hash,
    pub block_hash: Hash,
    pub validator_set_hash: Hash,
    pub votes: BTreeMap<Address, (Vec<u8>, u128)>, // validator → (signature, stake)
    pub total_stake: u128,
}

impl PendingCheckpoint {
    pub fn new(
        sequence: u64,
        epoch_start: Epoch,
        epoch_end: Epoch,
        state_root: Hash,
        block_hash: Hash,
        validator_set_hash: Hash,
        total_stake: u128,
    ) -> Self {
        Self {
            sequence,
            epoch_start,
            epoch_end,
            state_root,
            block_hash,
            validator_set_hash,
            votes: BTreeMap::new(),
            total_stake,
        }
    }

    /// Add a validator vote. Returns error if duplicate or stake is zero.
    pub fn add_vote(
        &mut self,
        validator: Address,
        signature: Vec<u8>,
        stake: u128,
    ) -> Result<(), CheckpointError> {
        if stake == 0 {
            return Err(CheckpointError::ZeroStake);
        }
        if self.votes.contains_key(&validator) {
            return Err(CheckpointError::DuplicateVote);
        }
        self.votes.insert(validator, (signature, stake));
        Ok(())
    }

    /// Current signed stake.
    pub fn signed_stake(&self) -> u128 {
        self.votes.values().map(|(_, s)| s).sum()
    }

    /// Try to finalize into a Checkpoint if quorum is reached.
    pub fn try_finalize(&self) -> Option<Checkpoint> {
        let signed = self.signed_stake();
        if self.total_stake > 0
            && signed * QUORUM_DENOMINATOR >= self.total_stake * QUORUM_NUMERATOR
        {
            Some(Checkpoint {
                sequence: self.sequence,
                epoch_start: self.epoch_start,
                epoch_end: self.epoch_end,
                state_root: self.state_root,
                block_hash: self.block_hash,
                validator_set_hash: self.validator_set_hash,
                signatures: self.votes.iter().map(|(a, (s, _))| (*a, s.clone())).collect(),
                signed_stake: signed,
                total_stake: self.total_stake,
            })
        } else {
            None
        }
    }
}

/// Checkpoint manager — tracks history and produces new checkpoints.
#[derive(Debug)]
pub struct CheckpointManager {
    /// Finalized checkpoints (sequence → checkpoint).
    pub history: BTreeMap<u64, Checkpoint>,
    /// Current pending checkpoint (if any).
    pub pending: Option<PendingCheckpoint>,
    /// Next sequence number.
    pub next_sequence: u64,
    /// Simulated L1 anchor receipts (sequence → anchor_epoch).
    pub anchored: BTreeMap<u64, u64>,
}

/// Simulated L1 anchor receipt.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AnchorReceipt {
    pub checkpoint_sequence: u64,
    pub l1_epoch: u64,
    pub tx_hash: Hash,
}

impl CheckpointManager {
    pub fn new() -> Self {
        Self {
            history: BTreeMap::new(),
            pending: None,
            next_sequence: 0,
            anchored: BTreeMap::new(),
        }
    }

    /// Should a checkpoint be created at this epoch?
    pub fn should_checkpoint(&self, epoch: Epoch) -> bool {
        epoch > 0 && epoch % CHECKPOINT_INTERVAL == 0
    }

    /// Begin a new checkpoint period.
    pub fn begin_checkpoint(
        &mut self,
        epoch_end: Epoch,
        state_root: Hash,
        block_hash: Hash,
        validator_set_hash: Hash,
        total_stake: u128,
    ) -> Result<u64, CheckpointError> {
        if self.pending.is_some() {
            return Err(CheckpointError::PendingExists);
        }
        let seq = self.next_sequence;
        let epoch_start = if seq == 0 {
            1
        } else {
            self.history
                .get(&(seq - 1))
                .map(|c| c.epoch_end + 1)
                .unwrap_or(1)
        };
        self.pending = Some(PendingCheckpoint::new(
            seq,
            epoch_start,
            epoch_end,
            state_root,
            block_hash,
            validator_set_hash,
            total_stake,
        ));
        Ok(seq)
    }

    /// Submit a validator vote for the current pending checkpoint.
    pub fn vote(
        &mut self,
        validator: Address,
        signature: Vec<u8>,
        stake: u128,
    ) -> Result<Option<Checkpoint>, CheckpointError> {
        let pending = self.pending.as_mut().ok_or(CheckpointError::NoPending)?;
        pending.add_vote(validator, signature, stake)?;
        if let Some(cp) = pending.try_finalize() {
            let seq = cp.sequence;
            self.history.insert(seq, cp.clone());
            self.next_sequence = seq + 1;
            self.pending = None;
            Ok(Some(cp))
        } else {
            Ok(None)
        }
    }

    /// Simulate anchoring a finalized checkpoint to Filecoin L1.
    pub fn anchor_to_l1(
        &mut self,
        sequence: u64,
        l1_epoch: u64,
    ) -> Result<AnchorReceipt, CheckpointError> {
        let cp = self
            .history
            .get(&sequence)
            .ok_or(CheckpointError::NotFinalized)?;
        if self.anchored.contains_key(&sequence) {
            return Err(CheckpointError::AlreadyAnchored);
        }
        // Simulate tx hash from checkpoint digest + l1 epoch
        let mut h = Sha256::new();
        h.update(cp.digest());
        h.update(l1_epoch.to_le_bytes());
        let tx_hash: Hash = h.finalize().into();
        self.anchored.insert(sequence, l1_epoch);
        Ok(AnchorReceipt {
            checkpoint_sequence: sequence,
            l1_epoch,
            tx_hash,
        })
    }

    /// Get the latest finalized checkpoint.
    pub fn latest(&self) -> Option<&Checkpoint> {
        self.history.values().last()
    }

    /// Verify a state root against a checkpoint (for light clients).
    pub fn verify_state_at(&self, sequence: u64, expected_root: Hash) -> bool {
        self.history
            .get(&sequence)
            .map(|cp| cp.state_root == expected_root)
            .unwrap_or(false)
    }

    /// Get total finalized checkpoints.
    pub fn finalized_count(&self) -> usize {
        self.history.len()
    }

    /// Get total anchored checkpoints.
    pub fn anchored_count(&self) -> usize {
        self.anchored.len()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CheckpointError {
    PendingExists,
    NoPending,
    DuplicateVote,
    ZeroStake,
    NotFinalized,
    AlreadyAnchored,
}

impl std::fmt::Display for CheckpointError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::PendingExists => write!(f, "checkpoint already pending"),
            Self::NoPending => write!(f, "no pending checkpoint"),
            Self::DuplicateVote => write!(f, "duplicate vote"),
            Self::ZeroStake => write!(f, "zero stake vote"),
            Self::NotFinalized => write!(f, "checkpoint not finalized"),
            Self::AlreadyAnchored => write!(f, "checkpoint already anchored"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_hash(val: u8) -> Hash {
        let mut h = [0u8; 32];
        h[0] = val;
        h
    }

    fn make_manager_with_checkpoint() -> (CheckpointManager, Checkpoint) {
        let mut mgr = CheckpointManager::new();
        let seq = mgr
            .begin_checkpoint(120, test_hash(1), test_hash(2), test_hash(3), 300)
            .unwrap();
        assert_eq!(seq, 0);

        // 3 validators: 100 + 100 + 100 = 300 total stake
        // 200/300 = 66.7% >= 66.7% — quorum reached on second vote
        let result1 = mgr.vote(Address::test(1), vec![1, 2, 3], 100).unwrap();
        assert!(result1.is_none()); // 100/300 not enough
        let result2 = mgr.vote(Address::test(2), vec![4, 5, 6], 100).unwrap();
        let cp = result2.expect("should finalize with 2/3 quorum (200/300)");
        (mgr, cp)
    }

    #[test]
    fn test_should_checkpoint() {
        let mgr = CheckpointManager::new();
        assert!(!mgr.should_checkpoint(0));
        assert!(!mgr.should_checkpoint(1));
        assert!(!mgr.should_checkpoint(119));
        assert!(mgr.should_checkpoint(120));
        assert!(mgr.should_checkpoint(240));
        assert!(!mgr.should_checkpoint(121));
    }

    #[test]
    fn test_begin_and_finalize() {
        let (mgr, cp) = make_manager_with_checkpoint();
        assert_eq!(cp.sequence, 0);
        assert_eq!(cp.epoch_start, 1);
        assert_eq!(cp.epoch_end, 120);
        assert!(cp.has_quorum());
        assert_eq!(mgr.finalized_count(), 1);
        assert_eq!(mgr.next_sequence, 1);
    }

    #[test]
    fn test_quorum_exact_two_thirds() {
        let mut mgr = CheckpointManager::new();
        mgr.begin_checkpoint(120, test_hash(1), test_hash(2), test_hash(3), 300)
            .unwrap();
        // 200/300 = exactly 2/3 — should finalize
        mgr.vote(Address::test(1), vec![1], 100).unwrap();
        let result = mgr.vote(Address::test(2), vec![2], 100).unwrap();
        assert!(result.is_some());
    }

    #[test]
    fn test_quorum_not_reached() {
        let mut mgr = CheckpointManager::new();
        mgr.begin_checkpoint(120, test_hash(1), test_hash(2), test_hash(3), 300)
            .unwrap();
        // 100/300 = 33% < 67%
        let result = mgr.vote(Address::test(1), vec![1], 100).unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn test_duplicate_vote_rejected() {
        let mut mgr = CheckpointManager::new();
        mgr.begin_checkpoint(120, test_hash(1), test_hash(2), test_hash(3), 300)
            .unwrap();
        mgr.vote(Address::test(1), vec![1], 100).unwrap();
        let err = mgr.vote(Address::test(1), vec![2], 100).unwrap_err();
        assert_eq!(err, CheckpointError::DuplicateVote);
    }

    #[test]
    fn test_zero_stake_rejected() {
        let mut mgr = CheckpointManager::new();
        mgr.begin_checkpoint(120, test_hash(1), test_hash(2), test_hash(3), 300)
            .unwrap();
        let err = mgr.vote(Address::test(1), vec![1], 0).unwrap_err();
        assert_eq!(err, CheckpointError::ZeroStake);
    }

    #[test]
    fn test_no_pending_vote_rejected() {
        let mut mgr = CheckpointManager::new();
        let err = mgr.vote(Address::test(1), vec![1], 100).unwrap_err();
        assert_eq!(err, CheckpointError::NoPending);
    }

    #[test]
    fn test_double_begin_rejected() {
        let mut mgr = CheckpointManager::new();
        mgr.begin_checkpoint(120, test_hash(1), test_hash(2), test_hash(3), 300)
            .unwrap();
        let err = mgr
            .begin_checkpoint(240, test_hash(4), test_hash(5), test_hash(6), 300)
            .unwrap_err();
        assert_eq!(err, CheckpointError::PendingExists);
    }

    #[test]
    fn test_anchor_to_l1() {
        let (mut mgr, _) = make_manager_with_checkpoint();
        let receipt = mgr.anchor_to_l1(0, 1000).unwrap();
        assert_eq!(receipt.checkpoint_sequence, 0);
        assert_eq!(receipt.l1_epoch, 1000);
        assert_ne!(receipt.tx_hash, [0u8; 32]);
        assert_eq!(mgr.anchored_count(), 1);
    }

    #[test]
    fn test_double_anchor_rejected() {
        let (mut mgr, _) = make_manager_with_checkpoint();
        mgr.anchor_to_l1(0, 1000).unwrap();
        let err = mgr.anchor_to_l1(0, 2000).unwrap_err();
        assert_eq!(err, CheckpointError::AlreadyAnchored);
    }

    #[test]
    fn test_anchor_unfinalized_rejected() {
        let mgr = &mut CheckpointManager::new();
        let err = mgr.anchor_to_l1(0, 1000).unwrap_err();
        assert_eq!(err, CheckpointError::NotFinalized);
    }

    #[test]
    fn test_verify_state() {
        let (mgr, _) = make_manager_with_checkpoint();
        assert!(mgr.verify_state_at(0, test_hash(1)));
        assert!(!mgr.verify_state_at(0, test_hash(99)));
        assert!(!mgr.verify_state_at(1, test_hash(1)));
    }

    #[test]
    fn test_checkpoint_digest_deterministic() {
        let (_, cp) = make_manager_with_checkpoint();
        let d1 = cp.digest();
        let d2 = cp.digest();
        assert_eq!(d1, d2);
        assert_ne!(d1, [0u8; 32]);
    }

    #[test]
    fn test_sequential_checkpoints() {
        let (mut mgr, _) = make_manager_with_checkpoint();
        // Second checkpoint
        let seq = mgr
            .begin_checkpoint(240, test_hash(10), test_hash(11), test_hash(12), 200)
            .unwrap();
        assert_eq!(seq, 1);
        mgr.vote(Address::test(1), vec![1], 100).unwrap();
        let result = mgr.vote(Address::test(2), vec![2], 100).unwrap();
        let cp2 = result.unwrap();
        assert_eq!(cp2.epoch_start, 121);
        assert_eq!(cp2.epoch_end, 240);
        assert_eq!(mgr.finalized_count(), 2);
    }

    #[test]
    fn test_latest() {
        let mut mgr = CheckpointManager::new();
        assert!(mgr.latest().is_none());
        let (mgr, _) = make_manager_with_checkpoint();
        assert_eq!(mgr.latest().unwrap().sequence, 0);
    }

    #[test]
    fn test_pending_signed_stake() {
        let mut pending = PendingCheckpoint::new(0, 1, 120, test_hash(1), test_hash(2), test_hash(3), 300);
        assert_eq!(pending.signed_stake(), 0);
        pending.add_vote(Address::test(1), vec![1], 100).unwrap();
        assert_eq!(pending.signed_stake(), 100);
        pending.add_vote(Address::test(2), vec![2], 150).unwrap();
        assert_eq!(pending.signed_stake(), 250);
    }
}
