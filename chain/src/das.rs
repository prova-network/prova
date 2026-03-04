//! Data Availability Sampling (DAS) for Prova.
//!
//! Ensures inference inputs and outputs are available for dispute verification.
//! Uses erasure coding (Reed-Solomon style) to allow probabilistic verification
//! with O(sqrt(n)) samples proving O(n) data availability.
//!
//! # Design
//!
//! - Providers commit a **data root** (Merkle root over erasure-coded chunks)
//! - Validators sample random chunk indices and request proofs
//! - After `SAMPLE_ROUNDS` successful rounds, data is considered available
//! - If a provider fails to respond within `RESPONSE_WINDOW` epochs, they are penalized

use crate::types::{Address, Epoch, Hash};
use sha2::{Digest, Sha256};
use std::collections::HashMap;

// ─── Constants ───────────────────────────────────────────────────────────────

/// Number of erasure-coded chunks per blob (original + parity).
pub const TOTAL_CHUNKS: usize = 128;
/// Original data chunks before erasure coding (50% redundancy).
pub const ORIGINAL_CHUNKS: usize = 64;
/// Number of random samples per round.
pub const SAMPLES_PER_ROUND: usize = 16;
/// Number of successful rounds to consider data available.
pub const REQUIRED_ROUNDS: u32 = 3;
/// Epochs a provider has to respond to a sample challenge.
pub const RESPONSE_WINDOW: Epoch = 5;
/// Penalty (in stake units) for failing a DAS challenge.
pub const DAS_PENALTY: u128 = 500;

// ─── Types ───────────────────────────────────────────────────────────────────

/// A unique blob identifier (hash of the original data).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct BlobId(pub Hash);

/// A chunk with its index and Merkle proof.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChunkProof {
    pub index: usize,
    pub data: Vec<u8>,
    pub proof: Vec<Hash>,
}

/// Status of a DAS commitment.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DasStatus {
    /// Awaiting sampling rounds.
    Pending,
    /// All rounds passed — data considered available.
    Confirmed,
    /// Provider failed to respond — data unavailable.
    Failed,
}

/// A DAS commitment from a provider.
#[derive(Debug, Clone)]
pub struct DasCommitment {
    pub blob_id: BlobId,
    pub provider: Address,
    pub data_root: Hash,
    pub chunk_count: usize,
    pub submitted_epoch: Epoch,
    pub status: DasStatus,
    pub rounds_completed: u32,
}

/// An active sample challenge.
#[derive(Debug, Clone)]
pub struct SampleChallenge {
    pub blob_id: BlobId,
    pub round: u32,
    pub indices: Vec<usize>,
    pub deadline: Epoch,
    pub responded: bool,
}

/// DAS verification engine.
#[derive(Debug)]
pub struct DasEngine {
    commitments: HashMap<BlobId, DasCommitment>,
    challenges: Vec<SampleChallenge>,
    penalties: HashMap<Address, u128>,
    current_epoch: Epoch,
}

// ─── Merkle helpers ──────────────────────────────────────────────────────────

fn hash_leaf(index: usize, data: &[u8]) -> Hash {
    let mut h = Sha256::new();
    h.update(b"das-leaf:");
    h.update(index.to_le_bytes());
    h.update(data);
    h.finalize().into()
}

fn hash_pair(left: &Hash, right: &Hash) -> Hash {
    let mut h = Sha256::new();
    h.update(b"das-node:");
    h.update(left);
    h.update(right);
    h.finalize().into()
}

/// Build a Merkle root from chunk hashes. Returns (root, tree_layers).
fn build_merkle_tree(leaves: &[Hash]) -> (Hash, Vec<Vec<Hash>>) {
    assert!(!leaves.is_empty());
    let mut layers: Vec<Vec<Hash>> = vec![leaves.to_vec()];
    let mut current = leaves.to_vec();
    while current.len() > 1 {
        // Pad to even
        if current.len() % 2 == 1 {
            current.push(*current.last().unwrap());
        }
        let next: Vec<Hash> = current
            .chunks(2)
            .map(|pair| hash_pair(&pair[0], &pair[1]))
            .collect();
        layers.push(next.clone());
        current = next;
    }
    (current[0], layers)
}

/// Generate a Merkle proof for a given leaf index.
fn generate_proof(layers: &[Vec<Hash>], index: usize) -> Vec<Hash> {
    let mut proof = Vec::new();
    let mut idx = index;
    for layer in &layers[..layers.len() - 1] {
        let sibling = if idx % 2 == 0 {
            if idx + 1 < layer.len() {
                layer[idx + 1]
            } else {
                layer[idx] // padded duplicate
            }
        } else {
            layer[idx - 1]
        };
        proof.push(sibling);
        idx /= 2;
    }
    proof
}

/// Verify a Merkle proof for a leaf.
fn verify_proof(root: &Hash, leaf_hash: &Hash, index: usize, proof: &[Hash]) -> bool {
    let mut current = *leaf_hash;
    let mut idx = index;
    for sibling in proof {
        current = if idx % 2 == 0 {
            hash_pair(&current, sibling)
        } else {
            hash_pair(sibling, &current)
        };
        idx /= 2;
    }
    current == *root
}

/// Simple erasure coding simulation: XOR pairs of original chunks to produce parity.
fn erasure_encode(original: &[Vec<u8>]) -> Vec<Vec<u8>> {
    let n = original.len();
    let mut coded = original.to_vec();
    for i in 0..n {
        let j = (i + 1) % n;
        let parity: Vec<u8> = original[i]
            .iter()
            .zip(original[j].iter())
            .map(|(a, b)| a ^ b)
            .collect();
        coded.push(parity);
    }
    coded
}

// ─── Engine implementation ───────────────────────────────────────────────────

impl DasEngine {
    pub fn new() -> Self {
        Self {
            commitments: HashMap::new(),
            challenges: Vec::new(),
            penalties: HashMap::new(),
            current_epoch: 0,
        }
    }

    pub fn set_epoch(&mut self, epoch: Epoch) {
        self.current_epoch = epoch;
    }

    /// Submit a DAS commitment (provider publishes erasure-coded data root).
    pub fn submit_commitment(
        &mut self,
        blob_id: BlobId,
        provider: Address,
        data_root: Hash,
        chunk_count: usize,
    ) -> Result<(), &'static str> {
        if self.commitments.contains_key(&blob_id) {
            return Err("blob already committed");
        }
        if chunk_count == 0 || chunk_count > TOTAL_CHUNKS * 4 {
            return Err("invalid chunk count");
        }
        self.commitments.insert(
            blob_id,
            DasCommitment {
                blob_id,
                provider,
                data_root,
                chunk_count,
                submitted_epoch: self.current_epoch,
                status: DasStatus::Pending,
                rounds_completed: 0,
            },
        );
        Ok(())
    }

    /// Generate a sample challenge for a blob using epoch randomness.
    pub fn generate_challenge(
        &mut self,
        blob_id: BlobId,
        randomness: &Hash,
    ) -> Result<SampleChallenge, &'static str> {
        let commitment = self
            .commitments
            .get(&blob_id)
            .ok_or("blob not found")?;
        if commitment.status != DasStatus::Pending {
            return Err("blob not in pending state");
        }
        let round = commitment.rounds_completed;

        // Derive pseudo-random indices from randomness + round
        let mut indices = Vec::with_capacity(SAMPLES_PER_ROUND);
        for i in 0..SAMPLES_PER_ROUND {
            let mut h = Sha256::new();
            h.update(randomness);
            h.update(round.to_le_bytes());
            h.update(i.to_le_bytes());
            let hash: [u8; 32] = h.finalize().into();
            let idx = u64::from_le_bytes(hash[..8].try_into().unwrap()) as usize
                % commitment.chunk_count;
            indices.push(idx);
        }

        let challenge = SampleChallenge {
            blob_id,
            round,
            indices,
            deadline: self.current_epoch + RESPONSE_WINDOW,
            responded: false,
        };
        self.challenges.push(challenge.clone());
        Ok(challenge)
    }

    /// Respond to a sample challenge with chunk proofs.
    pub fn respond_to_challenge(
        &mut self,
        blob_id: BlobId,
        round: u32,
        proofs: &[ChunkProof],
    ) -> Result<(), &'static str> {
        let commitment = self
            .commitments
            .get(&blob_id)
            .ok_or("blob not found")?;
        if commitment.status != DasStatus::Pending {
            return Err("blob not in pending state");
        }
        let data_root = commitment.data_root;

        // Find the challenge
        let challenge = self
            .challenges
            .iter_mut()
            .find(|c| c.blob_id == blob_id && c.round == round && !c.responded)
            .ok_or("challenge not found")?;

        if self.current_epoch > challenge.deadline {
            return Err("challenge deadline passed");
        }

        // Verify each proof
        if proofs.len() != challenge.indices.len() {
            return Err("wrong number of proofs");
        }
        for (proof, &expected_idx) in proofs.iter().zip(challenge.indices.iter()) {
            if proof.index != expected_idx {
                return Err("proof index mismatch");
            }
            let leaf = hash_leaf(proof.index, &proof.data);
            if !verify_proof(&data_root, &leaf, proof.index, &proof.proof) {
                return Err("invalid merkle proof");
            }
        }

        challenge.responded = true;

        // Advance rounds
        let commitment = self.commitments.get_mut(&blob_id).unwrap();
        commitment.rounds_completed += 1;
        if commitment.rounds_completed >= REQUIRED_ROUNDS {
            commitment.status = DasStatus::Confirmed;
        }
        Ok(())
    }

    /// Process expired challenges — penalize non-responders.
    pub fn process_expired(&mut self) {
        let expired: Vec<(BlobId, u32)> = self
            .challenges
            .iter()
            .filter(|c| !c.responded && self.current_epoch > c.deadline)
            .map(|c| (c.blob_id, c.round))
            .collect();

        for (blob_id, _round) in expired {
            if let Some(commitment) = self.commitments.get_mut(&blob_id) {
                if commitment.status == DasStatus::Pending {
                    commitment.status = DasStatus::Failed;
                    let penalty = self.penalties.entry(commitment.provider).or_insert(0);
                    *penalty += DAS_PENALTY;
                }
            }
        }

        // Remove expired challenges
        self.challenges
            .retain(|c| c.responded || self.current_epoch <= c.deadline);
    }

    pub fn get_commitment(&self, blob_id: &BlobId) -> Option<&DasCommitment> {
        self.commitments.get(blob_id)
    }

    pub fn get_penalty(&self, addr: &Address) -> u128 {
        self.penalties.get(addr).copied().unwrap_or(0)
    }

    pub fn commitment_count(&self) -> usize {
        self.commitments.len()
    }

    pub fn pending_challenges(&self) -> usize {
        self.challenges.iter().filter(|c| !c.responded).count()
    }
}

// ─── Helper: build blob data and commitment ──────────────────────────────────

/// Prepare erasure-coded blob from raw data chunks.
/// Returns (BlobId, data_root, all_chunks, merkle_layers) for use in proofs.
pub fn prepare_blob(
    original_chunks: &[Vec<u8>],
) -> (BlobId, Hash, Vec<Vec<u8>>, Vec<Vec<Hash>>) {
    let all_chunks = erasure_encode(original_chunks);
    let leaf_hashes: Vec<Hash> = all_chunks
        .iter()
        .enumerate()
        .map(|(i, data)| hash_leaf(i, data))
        .collect();
    let (root, layers) = build_merkle_tree(&leaf_hashes);

    // BlobId = hash of original data roots
    let mut h = Sha256::new();
    for chunk in original_chunks {
        h.update(chunk);
    }
    let blob_hash: Hash = h.finalize().into();

    (BlobId(blob_hash), root, all_chunks, layers)
}

/// Build chunk proofs for requested indices.
pub fn build_chunk_proofs(
    indices: &[usize],
    all_chunks: &[Vec<u8>],
    layers: &[Vec<Hash>],
) -> Vec<ChunkProof> {
    indices
        .iter()
        .map(|&idx| ChunkProof {
            index: idx,
            data: all_chunks[idx].clone(),
            proof: generate_proof(layers, idx),
        })
        .collect()
}

// ─── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn make_chunks(n: usize, size: usize) -> Vec<Vec<u8>> {
        (0..n).map(|i| vec![(i & 0xff) as u8; size]).collect()
    }

    fn test_randomness(seed: u8) -> Hash {
        let mut h = Sha256::new();
        h.update([seed]);
        h.finalize().into()
    }

    #[test]
    fn test_erasure_encode_doubles_chunks() {
        let original = make_chunks(4, 32);
        let coded = erasure_encode(&original);
        assert_eq!(coded.len(), 8); // 4 original + 4 parity
    }

    #[test]
    fn test_erasure_parity_correctness() {
        let original = make_chunks(4, 32);
        let coded = erasure_encode(&original);
        // Parity[i] = original[i] ^ original[(i+1)%n]
        for i in 0..4 {
            let j = (i + 1) % 4;
            let expected: Vec<u8> = original[i]
                .iter()
                .zip(original[j].iter())
                .map(|(a, b)| a ^ b)
                .collect();
            assert_eq!(coded[4 + i], expected);
        }
    }

    #[test]
    fn test_merkle_tree_single_leaf() {
        let leaf = hash_leaf(0, b"hello");
        let (root, layers) = build_merkle_tree(&[leaf]);
        assert_eq!(root, leaf);
        assert_eq!(layers.len(), 1);
    }

    #[test]
    fn test_merkle_proof_roundtrip() {
        let chunks = make_chunks(8, 16);
        let leaves: Vec<Hash> = chunks
            .iter()
            .enumerate()
            .map(|(i, d)| hash_leaf(i, d))
            .collect();
        let (root, layers) = build_merkle_tree(&leaves);
        for i in 0..8 {
            let proof = generate_proof(&layers, i);
            assert!(verify_proof(&root, &leaves[i], i, &proof));
        }
    }

    #[test]
    fn test_invalid_proof_rejected() {
        let chunks = make_chunks(4, 16);
        let leaves: Vec<Hash> = chunks
            .iter()
            .enumerate()
            .map(|(i, d)| hash_leaf(i, d))
            .collect();
        let (root, layers) = build_merkle_tree(&leaves);
        let proof = generate_proof(&layers, 0);
        // Wrong leaf should fail
        let wrong_leaf = hash_leaf(0, b"wrong data");
        assert!(!verify_proof(&root, &wrong_leaf, 0, &proof));
    }

    #[test]
    fn test_prepare_blob_and_proofs() {
        let original = make_chunks(8, 64);
        let (blob_id, root, all_chunks, layers) = prepare_blob(&original);
        assert_eq!(all_chunks.len(), 16); // 8 + 8 parity
        assert_ne!(blob_id.0, [0u8; 32]);
        assert_ne!(root, [0u8; 32]);

        let proofs = build_chunk_proofs(&[0, 5, 10, 15], &all_chunks, &layers);
        assert_eq!(proofs.len(), 4);
        for p in &proofs {
            let leaf = hash_leaf(p.index, &p.data);
            assert!(verify_proof(&root, &leaf, p.index, &p.proof));
        }
    }

    #[test]
    fn test_submit_commitment() {
        let mut engine = DasEngine::new();
        let original = make_chunks(4, 32);
        let (blob_id, root, all_chunks, _) = prepare_blob(&original);
        engine
            .submit_commitment(blob_id, Address::test(1), root, all_chunks.len())
            .unwrap();
        let c = engine.get_commitment(&blob_id).unwrap();
        assert_eq!(c.status, DasStatus::Pending);
        assert_eq!(c.rounds_completed, 0);
    }

    #[test]
    fn test_duplicate_commitment_rejected() {
        let mut engine = DasEngine::new();
        let original = make_chunks(4, 32);
        let (blob_id, root, all_chunks, _) = prepare_blob(&original);
        engine
            .submit_commitment(blob_id, Address::test(1), root, all_chunks.len())
            .unwrap();
        let err = engine
            .submit_commitment(blob_id, Address::test(2), root, all_chunks.len())
            .unwrap_err();
        assert_eq!(err, "blob already committed");
    }

    #[test]
    fn test_full_das_flow_confirmed() {
        let mut engine = DasEngine::new();
        let original = make_chunks(8, 64);
        let (blob_id, root, all_chunks, layers) = prepare_blob(&original);

        engine
            .submit_commitment(blob_id, Address::test(1), root, all_chunks.len())
            .unwrap();

        // Complete REQUIRED_ROUNDS of sampling
        for round_seed in 0..REQUIRED_ROUNDS as u8 {
            let randomness = test_randomness(round_seed);
            let challenge = engine.generate_challenge(blob_id, &randomness).unwrap();
            let proofs = build_chunk_proofs(&challenge.indices, &all_chunks, &layers);
            engine
                .respond_to_challenge(blob_id, challenge.round, &proofs)
                .unwrap();
        }

        let c = engine.get_commitment(&blob_id).unwrap();
        assert_eq!(c.status, DasStatus::Confirmed);
        assert_eq!(c.rounds_completed, REQUIRED_ROUNDS);
    }

    #[test]
    fn test_expired_challenge_penalizes() {
        let mut engine = DasEngine::new();
        let original = make_chunks(4, 32);
        let (blob_id, root, all_chunks, _) = prepare_blob(&original);

        engine
            .submit_commitment(blob_id, Address::test(1), root, all_chunks.len())
            .unwrap();
        let randomness = test_randomness(0);
        let challenge = engine.generate_challenge(blob_id, &randomness).unwrap();

        // Advance past deadline without responding
        engine.set_epoch(challenge.deadline + 1);
        engine.process_expired();

        let c = engine.get_commitment(&blob_id).unwrap();
        assert_eq!(c.status, DasStatus::Failed);
        assert_eq!(engine.get_penalty(&Address::test(1)), DAS_PENALTY);
    }

    #[test]
    fn test_wrong_proof_count_rejected() {
        let mut engine = DasEngine::new();
        let original = make_chunks(4, 32);
        let (blob_id, root, all_chunks, layers) = prepare_blob(&original);

        engine
            .submit_commitment(blob_id, Address::test(1), root, all_chunks.len())
            .unwrap();
        let randomness = test_randomness(0);
        let challenge = engine.generate_challenge(blob_id, &randomness).unwrap();

        // Only provide half the proofs
        let half = &challenge.indices[..challenge.indices.len() / 2];
        let proofs = build_chunk_proofs(half, &all_chunks, &layers);
        let err = engine
            .respond_to_challenge(blob_id, challenge.round, &proofs)
            .unwrap_err();
        assert_eq!(err, "wrong number of proofs");
    }

    #[test]
    fn test_invalid_chunk_data_rejected() {
        let mut engine = DasEngine::new();
        let original = make_chunks(4, 32);
        let (blob_id, root, all_chunks, layers) = prepare_blob(&original);

        engine
            .submit_commitment(blob_id, Address::test(1), root, all_chunks.len())
            .unwrap();
        let randomness = test_randomness(0);
        let challenge = engine.generate_challenge(blob_id, &randomness).unwrap();

        let mut proofs = build_chunk_proofs(&challenge.indices, &all_chunks, &layers);
        // Corrupt one chunk
        proofs[0].data = vec![0xff; proofs[0].data.len()];
        let err = engine
            .respond_to_challenge(blob_id, challenge.round, &proofs)
            .unwrap_err();
        assert_eq!(err, "invalid merkle proof");
    }

    #[test]
    fn test_challenge_after_deadline_rejected() {
        let mut engine = DasEngine::new();
        let original = make_chunks(4, 32);
        let (blob_id, root, all_chunks, layers) = prepare_blob(&original);

        engine
            .submit_commitment(blob_id, Address::test(1), root, all_chunks.len())
            .unwrap();
        let randomness = test_randomness(0);
        let challenge = engine.generate_challenge(blob_id, &randomness).unwrap();
        let proofs = build_chunk_proofs(&challenge.indices, &all_chunks, &layers);

        // Advance past deadline
        engine.set_epoch(challenge.deadline + 1);
        let err = engine
            .respond_to_challenge(blob_id, challenge.round, &proofs)
            .unwrap_err();
        assert_eq!(err, "challenge deadline passed");
    }

    #[test]
    fn test_engine_stats() {
        let mut engine = DasEngine::new();
        assert_eq!(engine.commitment_count(), 0);
        assert_eq!(engine.pending_challenges(), 0);

        let original = make_chunks(4, 32);
        let (blob_id, root, all_chunks, _) = prepare_blob(&original);
        engine
            .submit_commitment(blob_id, Address::test(1), root, all_chunks.len())
            .unwrap();
        assert_eq!(engine.commitment_count(), 1);

        let randomness = test_randomness(0);
        engine.generate_challenge(blob_id, &randomness).unwrap();
        assert_eq!(engine.pending_challenges(), 1);
    }

    #[test]
    fn test_multiple_blobs_independent() {
        let mut engine = DasEngine::new();
        let orig1 = make_chunks(4, 32);
        let orig2 = make_chunks(4, 64);
        let (id1, root1, chunks1, layers1) = prepare_blob(&orig1);
        let (id2, root2, chunks2, _) = prepare_blob(&orig2);

        engine.submit_commitment(id1, Address::test(1), root1, chunks1.len()).unwrap();
        engine.submit_commitment(id2, Address::test(2), root2, chunks2.len()).unwrap();

        // Confirm blob1, let blob2 expire
        for seed in 0..REQUIRED_ROUNDS as u8 {
            let r = test_randomness(seed);
            let ch = engine.generate_challenge(id1, &r).unwrap();
            let proofs = build_chunk_proofs(&ch.indices, &chunks1, &layers1);
            engine.respond_to_challenge(id1, ch.round, &proofs).unwrap();
        }

        let r = test_randomness(100);
        engine.generate_challenge(id2, &r).unwrap();
        engine.set_epoch(RESPONSE_WINDOW + 2);
        engine.process_expired();

        assert_eq!(engine.get_commitment(&id1).unwrap().status, DasStatus::Confirmed);
        assert_eq!(engine.get_commitment(&id2).unwrap().status, DasStatus::Failed);
        assert_eq!(engine.get_penalty(&Address::test(1)), 0);
        assert_eq!(engine.get_penalty(&Address::test(2)), DAS_PENALTY);
    }
}
