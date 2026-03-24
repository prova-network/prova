//! Confidential inference — commit-reveal scheme with encrypted activations.
//!
//! Providers can submit inference results with encrypted activation roots.
//! The plaintext is only revealed if a dispute is opened, preserving privacy
//! of model inputs/outputs during normal (unchallenged) operation.
//!
//! Flow:
//! 1. Provider commits: `(encrypted_root, blinding_hash, model_id, epoch)`
//! 2. Challenge window passes → result finalized privately (never revealed)
//! 3. If disputed → provider must reveal `(plaintext_root, blinding_factor)`
//!    within `REVEAL_WINDOW` epochs or gets slashed
//! 4. Revealed root feeds into normal QBP bisection game

use crate::types::*;
use std::collections::HashMap;

/// Configuration constants.
const CHALLENGE_WINDOW: Epoch = 10;
const REVEAL_WINDOW: Epoch = 5;

/// A confidential inference commitment.
#[derive(Debug, Clone)]
pub struct ConfidentialCommit {
    pub id: CommitId,
    pub provider: Address,
    pub model_id: ModelId,
    pub encrypted_root: Hash,
    /// H(plaintext_root || blinding_factor) — used to verify reveals
    pub blinding_hash: Hash,
    pub epoch: Epoch,
    pub status: ConfidentialStatus,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConfidentialStatus {
    /// Within challenge window, no dispute
    Committed,
    /// Disputed — provider must reveal within REVEAL_WINDOW
    Disputed {
        challenger: Address,
        dispute_epoch: Epoch,
    },
    /// Provider revealed plaintext, ready for bisection
    Revealed { plaintext_root: Hash },
    /// Challenge window passed, result finalized privately
    Finalized,
    /// Provider failed to reveal in time — slashed
    Defaulted,
}

/// Manages confidential inference commitments.
#[derive(Debug)]
pub struct ConfidentialStore {
    commits: HashMap<CommitId, ConfidentialCommit>,
    next_id: u64,
}

/// Hash helper — SHA-256 of concatenated inputs.
fn hash_concat(a: &[u8], b: &[u8]) -> Hash {
    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    hasher.update(a);
    hasher.update(b);
    let result = hasher.finalize();
    let mut out = [0u8; 32];
    out.copy_from_slice(&result);
    out
}

impl ConfidentialStore {
    pub fn new() -> Self {
        Self {
            commits: HashMap::new(),
            next_id: 1,
        }
    }

    /// Submit a confidential inference commitment.
    pub fn commit(
        &mut self,
        provider: Address,
        model_id: ModelId,
        encrypted_root: Hash,
        blinding_hash: Hash,
        epoch: Epoch,
    ) -> CommitId {
        let id = CommitId(self.next_id);
        self.next_id += 1;
        self.commits.insert(
            id,
            ConfidentialCommit {
                id,
                provider,
                model_id,
                encrypted_root,
                blinding_hash,
                epoch,
                status: ConfidentialStatus::Committed,
            },
        );
        id
    }

    /// Open a dispute on a committed inference (during challenge window).
    pub fn dispute(
        &mut self,
        commit_id: CommitId,
        challenger: Address,
        current_epoch: Epoch,
    ) -> Result<(), &'static str> {
        let c = self.commits.get_mut(&commit_id).ok_or("commit not found")?;
        if c.status != ConfidentialStatus::Committed {
            return Err("can only dispute committed inferences");
        }
        if current_epoch > c.epoch + CHALLENGE_WINDOW {
            return Err("challenge window expired");
        }
        if challenger == c.provider {
            return Err("provider cannot self-dispute");
        }
        c.status = ConfidentialStatus::Disputed {
            challenger,
            dispute_epoch: current_epoch,
        };
        Ok(())
    }

    /// Provider reveals plaintext root + blinding factor after dispute.
    pub fn reveal(
        &mut self,
        commit_id: CommitId,
        plaintext_root: Hash,
        blinding_factor: &[u8],
        current_epoch: Epoch,
    ) -> Result<(), &'static str> {
        let c = self.commits.get_mut(&commit_id).ok_or("commit not found")?;
        let dispute_epoch = match &c.status {
            ConfidentialStatus::Disputed { dispute_epoch, .. } => *dispute_epoch,
            _ => return Err("commit not in disputed state"),
        };
        if current_epoch > dispute_epoch + REVEAL_WINDOW {
            return Err("reveal window expired");
        }

        // Verify: H(plaintext_root || blinding_factor) == blinding_hash
        let computed = hash_concat(&plaintext_root, blinding_factor);
        if computed != c.blinding_hash {
            return Err("blinding hash mismatch — invalid reveal");
        }

        c.status = ConfidentialStatus::Revealed { plaintext_root };
        Ok(())
    }

    /// Finalize commits whose challenge window has passed without dispute.
    pub fn finalize(&mut self, current_epoch: Epoch) -> Vec<CommitId> {
        let mut finalized = Vec::new();
        for (id, c) in self.commits.iter_mut() {
            if c.status == ConfidentialStatus::Committed
                && current_epoch > c.epoch + CHALLENGE_WINDOW
            {
                c.status = ConfidentialStatus::Finalized;
                finalized.push(*id);
            }
        }
        finalized
    }

    /// Default (slash) providers who failed to reveal in time.
    pub fn enforce_defaults(&mut self, current_epoch: Epoch) -> Vec<(CommitId, Address)> {
        let mut defaulted = Vec::new();
        for (id, c) in self.commits.iter_mut() {
            if let ConfidentialStatus::Disputed { dispute_epoch, .. } = c.status {
                if current_epoch > dispute_epoch + REVEAL_WINDOW {
                    defaulted.push((*id, c.provider));
                    c.status = ConfidentialStatus::Defaulted;
                }
            }
        }
        defaulted
    }

    pub fn get(&self, id: CommitId) -> Option<&ConfidentialCommit> {
        self.commits.get(&id)
    }

    pub fn count(&self) -> usize {
        self.commits.len()
    }

    /// Get all commits by provider.
    pub fn by_provider(&self, provider: Address) -> Vec<&ConfidentialCommit> {
        self.commits
            .values()
            .filter(|c| c.provider == provider)
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_hash(v: u8) -> Hash {
        let mut h = [0u8; 32];
        h[0] = v;
        h
    }

    fn make_blinding(plaintext: &Hash, factor: &[u8]) -> Hash {
        hash_concat(plaintext, factor)
    }

    #[test]
    fn test_commit_and_finalize() {
        let mut store = ConfidentialStore::new();
        let provider = Address::test(1);
        let model = ModelId(test_hash(10));
        let plaintext = test_hash(20);
        let factor = b"secret-blinding";
        let blinding = make_blinding(&plaintext, factor);

        let id = store.commit(provider, model, test_hash(30), blinding, 100);
        assert_eq!(store.get(id).unwrap().status, ConfidentialStatus::Committed);

        // Not yet finalizable
        assert!(store.finalize(105).is_empty());

        // Past challenge window
        let finalized = store.finalize(111);
        assert_eq!(finalized.len(), 1);
        assert_eq!(store.get(id).unwrap().status, ConfidentialStatus::Finalized);
    }

    #[test]
    fn test_dispute_and_reveal() {
        let mut store = ConfidentialStore::new();
        let provider = Address::test(1);
        let challenger = Address::test(2);
        let model = ModelId(test_hash(10));
        let plaintext = test_hash(20);
        let factor = b"secret-blinding";
        let blinding = make_blinding(&plaintext, factor);

        let id = store.commit(provider, model, test_hash(30), blinding, 100);

        // Dispute within window
        store.dispute(id, challenger, 105).unwrap();
        assert!(matches!(
            store.get(id).unwrap().status,
            ConfidentialStatus::Disputed { .. }
        ));

        // Reveal with correct blinding
        store.reveal(id, plaintext, factor, 108).unwrap();
        assert!(matches!(
            store.get(id).unwrap().status,
            ConfidentialStatus::Revealed { .. }
        ));
    }

    #[test]
    fn test_dispute_after_window_fails() {
        let mut store = ConfidentialStore::new();
        let provider = Address::test(1);
        let model = ModelId(test_hash(10));
        let id = store.commit(provider, model, test_hash(30), test_hash(40), 100);

        let err = store.dispute(id, Address::test(2), 111).unwrap_err();
        assert_eq!(err, "challenge window expired");
    }

    #[test]
    fn test_self_dispute_rejected() {
        let mut store = ConfidentialStore::new();
        let provider = Address::test(1);
        let model = ModelId(test_hash(10));
        let id = store.commit(provider, model, test_hash(30), test_hash(40), 100);

        let err = store.dispute(id, provider, 105).unwrap_err();
        assert_eq!(err, "provider cannot self-dispute");
    }

    #[test]
    fn test_bad_reveal_rejected() {
        let mut store = ConfidentialStore::new();
        let provider = Address::test(1);
        let challenger = Address::test(2);
        let model = ModelId(test_hash(10));
        let plaintext = test_hash(20);
        let blinding = make_blinding(&plaintext, b"real-secret");

        let id = store.commit(provider, model, test_hash(30), blinding, 100);
        store.dispute(id, challenger, 105).unwrap();

        // Wrong blinding factor
        let err = store
            .reveal(id, plaintext, b"wrong-secret", 108)
            .unwrap_err();
        assert_eq!(err, "blinding hash mismatch — invalid reveal");
    }

    #[test]
    fn test_reveal_window_expired() {
        let mut store = ConfidentialStore::new();
        let provider = Address::test(1);
        let challenger = Address::test(2);
        let model = ModelId(test_hash(10));
        let plaintext = test_hash(20);
        let factor = b"secret";
        let blinding = make_blinding(&plaintext, factor);

        let id = store.commit(provider, model, test_hash(30), blinding, 100);
        store.dispute(id, challenger, 105).unwrap();

        let err = store.reveal(id, plaintext, factor, 111).unwrap_err();
        assert_eq!(err, "reveal window expired");
    }

    #[test]
    fn test_default_enforcement() {
        let mut store = ConfidentialStore::new();
        let provider = Address::test(1);
        let challenger = Address::test(2);
        let model = ModelId(test_hash(10));

        let id = store.commit(provider, model, test_hash(30), test_hash(40), 100);
        store.dispute(id, challenger, 105).unwrap();

        // Before reveal window expires
        assert!(store.enforce_defaults(109).is_empty());

        // After reveal window
        let defaulted = store.enforce_defaults(111);
        assert_eq!(defaulted.len(), 1);
        assert_eq!(defaulted[0].1, provider);
        assert_eq!(store.get(id).unwrap().status, ConfidentialStatus::Defaulted);
    }

    #[test]
    fn test_cannot_dispute_finalized() {
        let mut store = ConfidentialStore::new();
        let provider = Address::test(1);
        let model = ModelId(test_hash(10));
        let id = store.commit(provider, model, test_hash(30), test_hash(40), 100);
        store.finalize(111);

        let err = store.dispute(id, Address::test(2), 112).unwrap_err();
        assert_eq!(err, "can only dispute committed inferences");
    }

    #[test]
    fn test_cannot_reveal_without_dispute() {
        let mut store = ConfidentialStore::new();
        let provider = Address::test(1);
        let model = ModelId(test_hash(10));
        let id = store.commit(provider, model, test_hash(30), test_hash(40), 100);

        let err = store.reveal(id, test_hash(20), b"factor", 105).unwrap_err();
        assert_eq!(err, "commit not in disputed state");
    }

    #[test]
    fn test_by_provider() {
        let mut store = ConfidentialStore::new();
        let p1 = Address::test(1);
        let p2 = Address::test(2);
        let model = ModelId(test_hash(10));

        store.commit(p1, model, test_hash(30), test_hash(40), 100);
        store.commit(p1, model, test_hash(31), test_hash(41), 101);
        store.commit(p2, model, test_hash(32), test_hash(42), 102);

        assert_eq!(store.by_provider(p1).len(), 2);
        assert_eq!(store.by_provider(p2).len(), 1);
    }

    #[test]
    fn test_multiple_commits_independent() {
        let mut store = ConfidentialStore::new();
        let provider = Address::test(1);
        let challenger = Address::test(2);
        let model = ModelId(test_hash(10));

        let plaintext1 = test_hash(20);
        let factor1 = b"secret1";
        let blinding1 = make_blinding(&plaintext1, factor1);

        let id1 = store.commit(provider, model, test_hash(30), blinding1, 100);
        let id2 = store.commit(provider, model, test_hash(31), test_hash(41), 100);

        // Dispute only id1
        store.dispute(id1, challenger, 105).unwrap();
        store.reveal(id1, plaintext1, factor1, 108).unwrap();

        // id2 should still be committable/finalizable
        assert_eq!(
            store.get(id2).unwrap().status,
            ConfidentialStatus::Committed
        );
        store.finalize(111);
        assert_eq!(
            store.get(id2).unwrap().status,
            ConfidentialStatus::Finalized
        );
    }

    #[test]
    fn test_count() {
        let mut store = ConfidentialStore::new();
        assert_eq!(store.count(), 0);
        let model = ModelId(test_hash(10));
        store.commit(Address::test(1), model, test_hash(30), test_hash(40), 100);
        store.commit(Address::test(2), model, test_hash(31), test_hash(41), 101);
        assert_eq!(store.count(), 2);
    }

    #[test]
    fn test_commit_id_monotonic() {
        let mut store = ConfidentialStore::new();
        let model = ModelId(test_hash(10));
        let id1 = store.commit(Address::test(1), model, test_hash(30), test_hash(40), 100);
        let id2 = store.commit(Address::test(1), model, test_hash(31), test_hash(41), 101);
        assert!(id2.0 > id1.0);
    }

    #[test]
    fn test_dispute_nonexistent() {
        let mut store = ConfidentialStore::new();
        let err = store
            .dispute(CommitId(999), Address::test(2), 100)
            .unwrap_err();
        assert_eq!(err, "commit not found");
    }

    #[test]
    fn test_reveal_nonexistent() {
        let mut store = ConfidentialStore::new();
        let err = store
            .reveal(CommitId(999), test_hash(1), b"x", 100)
            .unwrap_err();
        assert_eq!(err, "commit not found");
    }
}
