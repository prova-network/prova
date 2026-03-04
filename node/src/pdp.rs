//! PDP Proof Engine — generates and verifies Provable Data Possession proofs.
//!
//! Providers store model weight pieces and prove possession via Merkle
//! inclusion proofs over CommP roots. Challenges are derived from drand
//! randomness each proving period.

use std::collections::HashMap;

/// CommP root — SHA-256 truncated to 254 bits (Fr-safe), stored as 32 bytes.
pub type CommP = [u8; 32];

/// Proof set ID.
pub type ProofSetId = u64;

/// Epoch number.
pub type Epoch = u64;

/// A Merkle inclusion proof for one challenged root.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InclusionProof {
    /// Index of the challenged leaf in the proof set.
    pub leaf_index: usize,
    /// The leaf value (CommP root of the piece).
    pub leaf: CommP,
    /// Sibling hashes from leaf to root, bottom-up.
    pub siblings: Vec<[u8; 32]>,
}

/// A complete PDP proof responding to a challenge epoch.
#[derive(Debug, Clone)]
pub struct PdpProof {
    pub proof_set_id: ProofSetId,
    pub challenge_epoch: Epoch,
    pub proofs: Vec<InclusionProof>,
}

/// Configuration for a proof set.
#[derive(Debug, Clone)]
pub struct ProofSetConfig {
    /// Challenge frequency in epochs (e.g. 2880 ≈ 24h at 30s epochs).
    pub challenge_period: u64,
    /// Number of roots challenged per period.
    pub challenge_count: usize,
    /// Response window in epochs.
    pub response_window: u64,
}

impl Default for ProofSetConfig {
    fn default() -> Self {
        Self {
            challenge_period: 2880,
            challenge_count: 5,
            response_window: 60,
        }
    }
}

/// A proof set: an ordered collection of CommP roots for one provider's model data.
#[derive(Debug, Clone)]
pub struct ProofSet {
    pub id: ProofSetId,
    pub roots: Vec<CommP>,
    pub config: ProofSetConfig,
    /// Epoch when created / last successful proof.
    pub last_proven_epoch: Epoch,
    /// Consecutive misses.
    pub consecutive_misses: u32,
}

/// Lightweight SHA-256 hash (for scaffold: truncated std hasher; real impl uses SHA-256).
fn hash_pair(left: &[u8; 32], right: &[u8; 32]) -> [u8; 32] {
    use std::hash::{Hash, Hasher};
    use std::collections::hash_map::DefaultHasher;
    let mut h = DefaultHasher::new();
    left.hash(&mut h);
    right.hash(&mut h);
    let v = h.finish();
    let mut out = [0u8; 32];
    out[..8].copy_from_slice(&v.to_le_bytes());
    // Second pass with salt for more bits
    left.iter().rev().for_each(|b| b.hash(&mut h));
    right.iter().rev().for_each(|b| b.hash(&mut h));
    let v2 = h.finish();
    out[8..16].copy_from_slice(&v2.to_le_bytes());
    out
}

/// Build a Merkle tree over leaves, returning all levels (leaves = level 0).
fn build_tree(leaves: &[[u8; 32]]) -> Vec<Vec<[u8; 32]>> {
    if leaves.is_empty() {
        return vec![vec![]];
    }
    let mut levels: Vec<Vec<[u8; 32]>> = vec![leaves.to_vec()];
    let mut current = leaves.to_vec();
    while current.len() > 1 {
        // Pad odd-length levels by duplicating last
        if current.len() % 2 == 1 {
            current.push(*current.last().unwrap());
        }
        let next: Vec<[u8; 32]> = current
            .chunks(2)
            .map(|pair| hash_pair(&pair[0], &pair[1]))
            .collect();
        levels.push(next.clone());
        current = next;
    }
    levels
}

/// Compute the Merkle root of a set of leaves.
pub fn merkle_root(leaves: &[[u8; 32]]) -> [u8; 32] {
    let tree = build_tree(leaves);
    tree.last()
        .and_then(|level| level.first().copied())
        .unwrap_or([0u8; 32])
}

/// Generate an inclusion proof for `leaf_index` in `leaves`.
pub fn generate_inclusion_proof(leaves: &[[u8; 32]], leaf_index: usize) -> Option<InclusionProof> {
    if leaf_index >= leaves.len() {
        return None;
    }
    let tree = build_tree(leaves);
    let mut siblings = Vec::new();
    let mut idx = leaf_index;

    for level in &tree[..tree.len().saturating_sub(1)] {
        // Pad for sibling lookup
        let sibling_idx = if idx % 2 == 0 {
            if idx + 1 < level.len() { idx + 1 } else { idx }
        } else {
            idx - 1
        };
        siblings.push(level[sibling_idx]);
        idx /= 2;
    }

    Some(InclusionProof {
        leaf_index,
        leaf: leaves[leaf_index],
        siblings,
    })
}

/// Verify an inclusion proof against a known root.
pub fn verify_inclusion_proof(proof: &InclusionProof, root: &[u8; 32]) -> bool {
    let mut current = proof.leaf;
    let mut idx = proof.leaf_index;

    for sibling in &proof.siblings {
        current = if idx % 2 == 0 {
            hash_pair(&current, sibling)
        } else {
            hash_pair(sibling, &current)
        };
        idx /= 2;
    }

    &current == root
}

/// Derive challenge indices from a seed (drand beacon) and proof set size.
pub fn derive_challenges(seed: &[u8; 32], num_roots: usize, count: usize) -> Vec<usize> {
    if num_roots == 0 {
        return vec![];
    }
    let mut challenges = Vec::with_capacity(count);
    let mut state = *seed;
    for _ in 0..count {
        // Simple deterministic PRNG: hash the state
        state = hash_pair(&state, seed);
        let idx_bytes: [u8; 8] = state[..8].try_into().unwrap();
        let idx = (u64::from_le_bytes(idx_bytes) as usize) % num_roots;
        challenges.push(idx);
    }
    challenges
}

/// PDP proof engine — manages proof sets and generates/verifies proofs.
#[derive(Debug)]
pub struct PdpEngine {
    proof_sets: HashMap<ProofSetId, ProofSet>,
    next_id: ProofSetId,
}

impl Default for PdpEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl PdpEngine {
    pub fn new() -> Self {
        Self {
            proof_sets: HashMap::new(),
            next_id: 1,
        }
    }

    /// Register a new proof set with the given CommP roots.
    pub fn register_proof_set(
        &mut self,
        roots: Vec<CommP>,
        config: ProofSetConfig,
        epoch: Epoch,
    ) -> ProofSetId {
        let id = self.next_id;
        self.next_id += 1;
        self.proof_sets.insert(
            id,
            ProofSet {
                id,
                roots,
                config,
                last_proven_epoch: epoch,
                consecutive_misses: 0,
            },
        );
        id
    }

    /// Check if a proof set needs proving at the given epoch.
    pub fn needs_proving(&self, proof_set_id: ProofSetId, epoch: Epoch) -> bool {
        self.proof_sets.get(&proof_set_id).map_or(false, |ps| {
            epoch >= ps.last_proven_epoch + ps.config.challenge_period
        })
    }

    /// Generate a PDP proof for a proof set at a given epoch + drand seed.
    pub fn generate_proof(
        &self,
        proof_set_id: ProofSetId,
        challenge_epoch: Epoch,
        seed: &[u8; 32],
    ) -> Option<PdpProof> {
        let ps = self.proof_sets.get(&proof_set_id)?;
        let challenges = derive_challenges(seed, ps.roots.len(), ps.config.challenge_count);

        let proofs: Vec<InclusionProof> = challenges
            .iter()
            .filter_map(|&idx| generate_inclusion_proof(&ps.roots, idx))
            .collect();

        Some(PdpProof {
            proof_set_id,
            challenge_epoch,
            proofs,
        })
    }

    /// Verify a submitted PDP proof.
    pub fn verify_proof(&self, proof: &PdpProof, seed: &[u8; 32]) -> bool {
        let ps = match self.proof_sets.get(&proof.proof_set_id) {
            Some(ps) => ps,
            None => return false,
        };

        let expected_challenges =
            derive_challenges(seed, ps.roots.len(), ps.config.challenge_count);
        let root = merkle_root(&ps.roots);

        // Must have correct number of proofs
        if proof.proofs.len() != expected_challenges.len() {
            return false;
        }

        // Each proof must match the expected challenge index and verify against root
        for (inclusion, &expected_idx) in proof.proofs.iter().zip(expected_challenges.iter()) {
            if inclusion.leaf_index != expected_idx {
                return false;
            }
            if inclusion.leaf != ps.roots[expected_idx] {
                return false;
            }
            if !verify_inclusion_proof(inclusion, &root) {
                return false;
            }
        }

        true
    }

    /// Record a successful proof submission.
    pub fn record_success(&mut self, proof_set_id: ProofSetId, epoch: Epoch) -> bool {
        if let Some(ps) = self.proof_sets.get_mut(&proof_set_id) {
            ps.last_proven_epoch = epoch;
            ps.consecutive_misses = 0;
            true
        } else {
            false
        }
    }

    /// Record a missed proof. Returns the new consecutive miss count.
    pub fn record_miss(&mut self, proof_set_id: ProofSetId) -> Option<u32> {
        let ps = self.proof_sets.get_mut(&proof_set_id)?;
        ps.consecutive_misses += 1;
        Some(ps.consecutive_misses)
    }

    /// Get a proof set by ID.
    pub fn get(&self, id: ProofSetId) -> Option<&ProofSet> {
        self.proof_sets.get(&id)
    }

    /// Add roots to an existing proof set (e.g. provider stores additional model pieces).
    pub fn add_roots(&mut self, proof_set_id: ProofSetId, new_roots: Vec<CommP>) -> bool {
        if let Some(ps) = self.proof_sets.get_mut(&proof_set_id) {
            ps.roots.extend(new_roots);
            true
        } else {
            false
        }
    }

    /// Remove a root by index (queued deletion — takes effect after next proof).
    pub fn remove_root(&mut self, proof_set_id: ProofSetId, index: usize) -> bool {
        if let Some(ps) = self.proof_sets.get_mut(&proof_set_id) {
            if index < ps.roots.len() {
                ps.roots.remove(index);
                return true;
            }
        }
        false
    }

    /// Total roots across all proof sets.
    pub fn total_roots(&self) -> usize {
        self.proof_sets.values().map(|ps| ps.roots.len()).sum()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_commp(seed: u8) -> CommP {
        let mut c = [0u8; 32];
        c[0] = seed;
        c[31] = seed.wrapping_mul(7);
        c
    }

    fn test_seed() -> [u8; 32] {
        let mut s = [0u8; 32];
        s[0] = 0xDE;
        s[1] = 0xAD;
        s
    }

    #[test]
    fn test_merkle_root_single() {
        let leaves = vec![test_commp(1)];
        let root = merkle_root(&leaves);
        assert_eq!(root, leaves[0]); // Single leaf IS the root
    }

    #[test]
    fn test_merkle_root_deterministic() {
        let leaves: Vec<CommP> = (0..8).map(test_commp).collect();
        let r1 = merkle_root(&leaves);
        let r2 = merkle_root(&leaves);
        assert_eq!(r1, r2);
    }

    #[test]
    fn test_inclusion_proof_roundtrip() {
        let leaves: Vec<CommP> = (0..8).map(test_commp).collect();
        let root = merkle_root(&leaves);

        for i in 0..8 {
            let proof = generate_inclusion_proof(&leaves, i).unwrap();
            assert!(verify_inclusion_proof(&proof, &root), "failed for leaf {i}");
        }
    }

    #[test]
    fn test_inclusion_proof_wrong_root_fails() {
        let leaves: Vec<CommP> = (0..4).map(test_commp).collect();
        let proof = generate_inclusion_proof(&leaves, 0).unwrap();
        let bad_root = [0xFFu8; 32];
        assert!(!verify_inclusion_proof(&proof, &bad_root));
    }

    #[test]
    fn test_derive_challenges_deterministic() {
        let seed = test_seed();
        let c1 = derive_challenges(&seed, 100, 5);
        let c2 = derive_challenges(&seed, 100, 5);
        assert_eq!(c1, c2);
        assert_eq!(c1.len(), 5);
        assert!(c1.iter().all(|&i| i < 100));
    }

    #[test]
    fn test_engine_register_and_prove() {
        let mut engine = PdpEngine::new();
        let roots: Vec<CommP> = (0..10).map(test_commp).collect();
        let id = engine.register_proof_set(roots, ProofSetConfig::default(), 100);

        assert!(!engine.needs_proving(id, 100));
        assert!(engine.needs_proving(id, 100 + 2880));

        let seed = test_seed();
        let proof = engine.generate_proof(id, 2980, &seed).unwrap();
        assert_eq!(proof.proofs.len(), 5);
        assert!(engine.verify_proof(&proof, &seed));
    }

    #[test]
    fn test_engine_tampered_proof_fails() {
        let mut engine = PdpEngine::new();
        let roots: Vec<CommP> = (0..10).map(test_commp).collect();
        let id = engine.register_proof_set(roots, ProofSetConfig::default(), 100);

        let seed = test_seed();
        let mut proof = engine.generate_proof(id, 2980, &seed).unwrap();
        // Tamper with one proof
        proof.proofs[0].leaf[0] ^= 0xFF;
        assert!(!engine.verify_proof(&proof, &seed));
    }

    #[test]
    fn test_engine_miss_tracking() {
        let mut engine = PdpEngine::new();
        let roots: Vec<CommP> = (0..4).map(test_commp).collect();
        let id = engine.register_proof_set(roots, ProofSetConfig::default(), 100);

        assert_eq!(engine.record_miss(id), Some(1));
        assert_eq!(engine.record_miss(id), Some(2));
        assert_eq!(engine.get(id).unwrap().consecutive_misses, 2);

        engine.record_success(id, 5000);
        assert_eq!(engine.get(id).unwrap().consecutive_misses, 0);
    }

    #[test]
    fn test_engine_add_remove_roots() {
        let mut engine = PdpEngine::new();
        let roots: Vec<CommP> = (0..4).map(test_commp).collect();
        let id = engine.register_proof_set(roots, ProofSetConfig::default(), 100);

        assert_eq!(engine.get(id).unwrap().roots.len(), 4);

        engine.add_roots(id, vec![test_commp(10), test_commp(11)]);
        assert_eq!(engine.get(id).unwrap().roots.len(), 6);

        engine.remove_root(id, 0);
        assert_eq!(engine.get(id).unwrap().roots.len(), 5);
    }

    #[test]
    fn test_engine_total_roots() {
        let mut engine = PdpEngine::new();
        engine.register_proof_set(vec![test_commp(0); 5], ProofSetConfig::default(), 0);
        engine.register_proof_set(vec![test_commp(1); 3], ProofSetConfig::default(), 0);
        assert_eq!(engine.total_roots(), 8);
    }

    #[test]
    fn test_large_proof_set() {
        let mut engine = PdpEngine::new();
        let roots: Vec<CommP> = (0..1000u16).map(|i| {
            let mut c = [0u8; 32];
            c[..2].copy_from_slice(&i.to_le_bytes());
            c
        }).collect();
        let id = engine.register_proof_set(roots, ProofSetConfig::default(), 0);

        let seed = test_seed();
        let proof = engine.generate_proof(id, 2880, &seed).unwrap();
        assert!(engine.verify_proof(&proof, &seed));
    }
}
