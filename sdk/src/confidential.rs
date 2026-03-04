//! Confidential Inference Client SDK (SDK-011)
//!
//! High-level client for submitting, disputing, and revealing confidential
//! inference results. Wraps the chain's `ConfidentialStore` with encryption
//! helpers, blinding factor management, and automatic reveal logic.

use prova_chain::confidential::{ConfidentialStore, ConfidentialStatus};
use prova_chain::types::*;
use sha2::{Sha256, Digest};
use std::collections::HashMap;

// ── Encryption helpers (symmetric, simplified for simulation) ────────────

/// Encrypt plaintext root with a symmetric key (XOR-based simulation).
fn encrypt_root(plaintext: &Hash, key: &[u8; 32]) -> Hash {
    let mut out = [0u8; 32];
    for i in 0..32 {
        out[i] = plaintext[i] ^ key[i];
    }
    out
}

/// Decrypt encrypted root with the same symmetric key.
fn decrypt_root(ciphertext: &Hash, key: &[u8; 32]) -> Hash {
    encrypt_root(ciphertext, key) // XOR is its own inverse
}

/// Compute blinding hash: H(plaintext_root || blinding_factor).
fn blinding_hash(plaintext_root: &Hash, blinding_factor: &[u8; 32]) -> Hash {
    let mut hasher = Sha256::new();
    hasher.update(plaintext_root);
    hasher.update(blinding_factor);
    let result = hasher.finalize();
    let mut out = [0u8; 32];
    out.copy_from_slice(&result);
    out
}

/// Generate a deterministic blinding factor from seed (for testing).
fn generate_blinding_factor(seed: &[u8]) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(b"blinding:");
    hasher.update(seed);
    let result = hasher.finalize();
    let mut out = [0u8; 32];
    out.copy_from_slice(&result);
    out
}

// ── Pending commit record ────────────────────────────────────

/// Local state tracked per confidential commit (provider-side).
#[derive(Debug, Clone)]
struct PendingCommit {
    commit_id: CommitId,
    plaintext_root: Hash,
    blinding_factor: [u8; 32],
    encryption_key: [u8; 32],
    model_id: ModelId,
    auto_reveal: bool,
}

// ── Confidential Inference Client ────────────────────────────

/// High-level client for confidential inference operations.
#[derive(Debug)]
pub struct ConfidentialClient {
    store: ConfidentialStore,
    /// Provider-side secrets indexed by commit ID
    pending: HashMap<CommitId, PendingCommit>,
    /// Encryption keys per provider (simplified key management)
    provider_keys: HashMap<Address, [u8; 32]>,
}

/// Result of a confidential submission.
#[derive(Debug, Clone)]
pub struct SubmitResult {
    pub commit_id: CommitId,
    pub encrypted_root: Hash,
    pub blinding_hash: Hash,
}

/// Status summary for client queries.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CommitStatus {
    Pending,
    Disputed,
    Revealed,
    Finalized,
    Defaulted,
    Unknown,
}

/// Dispute outcome after attempting reveal.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RevealOutcome {
    /// Reveal accepted, plaintext root verified
    Accepted { plaintext_root: Hash },
    /// Commit not in disputed state
    NotDisputed,
    /// Commit not found in local pending set
    NotFound,
    /// Reveal failed verification
    VerificationFailed,
}

impl ConfidentialClient {
    pub fn new() -> Self {
        Self {
            store: ConfidentialStore::new(),
            pending: HashMap::new(),
            provider_keys: HashMap::new(),
        }
    }

    /// Register an encryption key for a provider address.
    pub fn register_key(&mut self, provider: Address, key: [u8; 32]) {
        self.provider_keys.insert(provider, key);
    }

    /// Submit a confidential inference result.
    pub fn submit(
        &mut self,
        provider: Address,
        model_id: ModelId,
        plaintext_root: Hash,
        epoch: Epoch,
        auto_reveal: bool,
    ) -> Result<SubmitResult, String> {
        let key = self.provider_keys.get(&provider)
            .ok_or_else(|| "No encryption key registered for provider".to_string())?;

        let encrypted = encrypt_root(&plaintext_root, key);
        let bf = generate_blinding_factor(&[&provider.0[..], &epoch.to_le_bytes()].concat());
        let bh = blinding_hash(&plaintext_root, &bf);

        let commit_id = self.store.commit(provider, model_id, encrypted, bh, epoch);

        self.pending.insert(commit_id, PendingCommit {
            commit_id,
            plaintext_root,
            blinding_factor: bf,
            encryption_key: *key,
            model_id,
            auto_reveal,
        });

        Ok(SubmitResult {
            commit_id,
            encrypted_root: encrypted,
            blinding_hash: bh,
        })
    }

    /// Open a dispute against a confidential commit (challenger-side).
    pub fn dispute(
        &mut self,
        commit_id: CommitId,
        challenger: Address,
        current_epoch: Epoch,
    ) -> Result<(), &'static str> {
        self.store.dispute(commit_id, challenger, current_epoch)
    }

    /// Reveal plaintext for a disputed commit (provider-side).
    pub fn reveal(&mut self, commit_id: CommitId, current_epoch: Epoch) -> RevealOutcome {
        let pending = match self.pending.get(&commit_id) {
            Some(p) => p.clone(),
            None => return RevealOutcome::NotFound,
        };

        match self.store.reveal(
            commit_id,
            pending.plaintext_root,
            &pending.blinding_factor,
            current_epoch,
        ) {
            Ok(()) => {
                self.pending.remove(&commit_id);
                RevealOutcome::Accepted { plaintext_root: pending.plaintext_root }
            }
            Err(e) if e.contains("not") && e.contains("disputed") => {
                RevealOutcome::NotDisputed
            }
            Err(_) => RevealOutcome::VerificationFailed,
        }
    }

    /// Auto-reveal all disputed commits that have auto_reveal enabled.
    pub fn auto_reveal_disputed(&mut self, current_epoch: Epoch) -> Vec<(CommitId, RevealOutcome)> {
        let auto_ids: Vec<CommitId> = self.pending.iter()
            .filter(|(_, p)| p.auto_reveal)
            .map(|(id, _)| *id)
            .collect();

        let mut results = Vec::new();
        for id in auto_ids {
            let status = self.query_status(id);
            if status == CommitStatus::Disputed {
                let outcome = self.reveal(id, current_epoch);
                results.push((id, outcome));
            }
        }
        results
    }

    /// Advance epoch — finalize and enforce defaults.
    pub fn tick(&mut self, current_epoch: Epoch) {
        self.store.finalize(current_epoch);
        self.store.enforce_defaults(current_epoch);
    }

    /// Query the status of a confidential commit.
    pub fn query_status(&self, commit_id: CommitId) -> CommitStatus {
        match self.store.get(commit_id) {
            Some(c) => match &c.status {
                ConfidentialStatus::Committed => CommitStatus::Pending,
                ConfidentialStatus::Disputed { .. } => CommitStatus::Disputed,
                ConfidentialStatus::Revealed { .. } => CommitStatus::Revealed,
                ConfidentialStatus::Finalized => CommitStatus::Finalized,
                ConfidentialStatus::Defaulted => CommitStatus::Defaulted,
            },
            None => CommitStatus::Unknown,
        }
    }

    /// Get all pending commit IDs for a provider.
    pub fn pending_commits(&self, provider: &Address) -> Vec<CommitId> {
        self.pending.iter()
            .filter(|(_, p)| {
                self.store.get(p.commit_id)
                    .map(|c| c.provider == *provider)
                    .unwrap_or(false)
            })
            .map(|(id, _)| *id)
            .collect()
    }

    /// Batch submit multiple confidential inferences.
    pub fn batch_submit(
        &mut self,
        provider: Address,
        jobs: Vec<(ModelId, Hash)>,
        epoch: Epoch,
        auto_reveal: bool,
    ) -> Vec<Result<SubmitResult, String>> {
        jobs.into_iter()
            .map(|(model_id, root)| self.submit(provider, model_id, root, epoch, auto_reveal))
            .collect()
    }

    /// Decrypt an encrypted root given the provider's key (verifier/auditor use).
    pub fn decrypt_for_audit(
        &self,
        provider: &Address,
        encrypted_root: &Hash,
    ) -> Result<Hash, String> {
        let key = self.provider_keys.get(provider)
            .ok_or_else(|| "No key for provider".to_string())?;
        Ok(decrypt_root(encrypted_root, key))
    }
}

// ── Tests ────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn addr(n: u8) -> Address { let mut a = [0u8; 20]; a[0] = n; Address(a) }
    fn hash_val(n: u8) -> Hash { let mut h = [0u8; 32]; h[0] = n; h }
    fn model(n: u8) -> ModelId { let mut h = [0u8; 32]; h[0] = n; ModelId(h) }
    fn key(n: u8) -> [u8; 32] { let mut k = [0u8; 32]; k[0] = n; k }

    fn setup() -> ConfidentialClient {
        let mut c = ConfidentialClient::new();
        c.register_key(addr(1), key(42));
        c
    }

    #[test]
    fn test_submit_and_query() {
        let mut c = setup();
        let res = c.submit(addr(1), model(1), hash_val(1), 0, false).unwrap();
        assert_eq!(res.commit_id, CommitId(1));
        assert_eq!(c.query_status(CommitId(1)), CommitStatus::Pending);
    }

    #[test]
    fn test_submit_no_key() {
        let mut c = ConfidentialClient::new();
        let res = c.submit(addr(99), model(1), hash_val(1), 0, false);
        assert!(res.is_err());
    }

    #[test]
    fn test_encryption_roundtrip() {
        let c = setup();
        let plaintext = hash_val(77);
        let encrypted = encrypt_root(&plaintext, &key(42));
        let decrypted = c.decrypt_for_audit(&addr(1), &encrypted).unwrap();
        assert_eq!(decrypted, plaintext);
    }

    #[test]
    fn test_dispute_and_reveal() {
        let mut c = setup();
        let res = c.submit(addr(1), model(1), hash_val(1), 0, false).unwrap();
        c.dispute(res.commit_id, addr(2), 1).unwrap();
        assert_eq!(c.query_status(res.commit_id), CommitStatus::Disputed);

        let outcome = c.reveal(res.commit_id, 2);
        assert!(matches!(outcome, RevealOutcome::Accepted { .. }));
        assert_eq!(c.query_status(res.commit_id), CommitStatus::Revealed);
    }

    #[test]
    fn test_reveal_not_disputed() {
        let mut c = setup();
        let res = c.submit(addr(1), model(1), hash_val(1), 0, false).unwrap();
        let outcome = c.reveal(res.commit_id, 1);
        assert_eq!(outcome, RevealOutcome::NotDisputed);
    }

    #[test]
    fn test_reveal_not_found() {
        let mut c = setup();
        let outcome = c.reveal(CommitId(999), 0);
        assert_eq!(outcome, RevealOutcome::NotFound);
    }

    #[test]
    fn test_finalization() {
        let mut c = setup();
        c.submit(addr(1), model(1), hash_val(1), 0, false).unwrap();
        c.tick(11);
        assert_eq!(c.query_status(CommitId(1)), CommitStatus::Finalized);
    }

    #[test]
    fn test_default_on_no_reveal() {
        let mut c = setup();
        let res = c.submit(addr(1), model(1), hash_val(1), 0, false).unwrap();
        c.dispute(res.commit_id, addr(2), 1).unwrap();
        c.pending.remove(&res.commit_id);
        c.tick(7);
        assert_eq!(c.query_status(res.commit_id), CommitStatus::Defaulted);
    }

    #[test]
    fn test_auto_reveal() {
        let mut c = setup();
        let r1 = c.submit(addr(1), model(1), hash_val(1), 0, true).unwrap();
        let r2 = c.submit(addr(1), model(2), hash_val(2), 0, false).unwrap();
        c.dispute(r1.commit_id, addr(2), 1).unwrap();
        c.dispute(r2.commit_id, addr(2), 1).unwrap();

        let results = c.auto_reveal_disputed(2);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].0, r1.commit_id);
        assert!(matches!(results[0].1, RevealOutcome::Accepted { .. }));
        assert_eq!(c.query_status(r2.commit_id), CommitStatus::Disputed);
    }

    #[test]
    fn test_batch_submit() {
        let mut c = setup();
        let jobs = vec![(model(1), hash_val(1)), (model(2), hash_val(2)), (model(3), hash_val(3))];
        let results = c.batch_submit(addr(1), jobs, 0, false);
        assert_eq!(results.len(), 3);
        assert!(results.iter().all(|r| r.is_ok()));
    }

    #[test]
    fn test_pending_commits() {
        let mut c = setup();
        c.register_key(addr(3), key(43));
        c.submit(addr(1), model(1), hash_val(1), 0, false).unwrap();
        c.submit(addr(1), model(2), hash_val(2), 0, false).unwrap();
        c.submit(addr(3), model(3), hash_val(3), 0, false).unwrap();

        let p1 = c.pending_commits(&addr(1));
        assert_eq!(p1.len(), 2);
        let p3 = c.pending_commits(&addr(3));
        assert_eq!(p3.len(), 1);
    }

    #[test]
    fn test_blinding_hash_deterministic() {
        let root = hash_val(42);
        let bf = generate_blinding_factor(b"test-seed");
        let h1 = blinding_hash(&root, &bf);
        let h2 = blinding_hash(&root, &bf);
        assert_eq!(h1, h2);
        let bf2 = generate_blinding_factor(b"other-seed");
        let h3 = blinding_hash(&root, &bf2);
        assert_ne!(h1, h3);
    }

    #[test]
    fn test_status_unknown() {
        let c = setup();
        assert_eq!(c.query_status(CommitId(999)), CommitStatus::Unknown);
    }

    #[test]
    fn test_full_lifecycle() {
        let mut c = setup();
        let res = c.submit(addr(1), model(1), hash_val(42), 0, true).unwrap();
        assert_eq!(c.query_status(res.commit_id), CommitStatus::Pending);

        c.dispute(res.commit_id, addr(2), 3).unwrap();
        assert_eq!(c.query_status(res.commit_id), CommitStatus::Disputed);

        let reveals = c.auto_reveal_disputed(4);
        assert_eq!(reveals.len(), 1);
        assert_eq!(c.query_status(res.commit_id), CommitStatus::Revealed);

        let commit = c.store.get(res.commit_id).unwrap();
        if let ConfidentialStatus::Revealed { plaintext_root } = &commit.status {
            assert_eq!(*plaintext_root, hash_val(42));
        } else {
            panic!("Expected Revealed status");
        }
    }
}
