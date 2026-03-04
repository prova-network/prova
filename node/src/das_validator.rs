//! DAS Validator — automatic data availability sampling over P2P.
//!
//! Implements NODE-028: validators automatically sample commitments,
//! request chunk proofs from providers over P2P, and submit results
//! to the DAS engine on-chain.
//!
//! # Design
//!
//! - Watches for new DAS commitments via event subscription
//! - Schedules sampling rounds based on epoch progression
//! - Sends P2P sample requests and collects chunk proofs
//! - Verifies proofs locally before submitting on-chain responses
//! - Tracks provider reliability for reputation scoring
//! - Configurable concurrency and retry policies

use prova_chain::das::*;
use prova_chain::types::*;
use sha2::{Digest, Sha256};
use std::collections::{HashMap, HashSet, VecDeque};

// ─── Constants ───────────────────────────────────────────────────────────────

/// Maximum concurrent blob validations.
pub const MAX_CONCURRENT_VALIDATIONS: usize = 32;
/// Retry attempts for a single sample request.
pub const SAMPLE_RETRY_LIMIT: u32 = 3;
/// Epochs before deadline to start sampling (leave margin).
pub const SAMPLING_MARGIN: Epoch = 2;
/// Minimum peer responses needed to trust a negative result.
pub const NEGATIVE_QUORUM: usize = 3;

// ─── Types ───────────────────────────────────────────────────────────────────

/// A request to a provider for chunk proofs.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SampleRequest {
    pub blob_id: BlobId,
    pub indices: Vec<usize>,
    pub round: u32,
    pub provider: Address,
}

/// Response from a provider.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SampleResponse {
    /// Provider returned valid chunk proofs.
    Success { blob_id: BlobId, round: u32, proofs: Vec<ChunkProof> },
    /// Provider did not respond in time.
    Timeout { blob_id: BlobId, round: u32 },
    /// Provider returned invalid/partial data.
    Invalid { blob_id: BlobId, round: u32, reason: String },
}

/// Status of a validation task.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ValidationStatus {
    /// Waiting to start sampling.
    Queued,
    /// Actively sampling.
    Sampling { round: u32, retries: u32 },
    /// All rounds passed.
    Confirmed,
    /// Provider failed — evidence submitted.
    Failed,
    /// Gave up after retries exhausted.
    Abandoned,
}

/// Tracks one blob's validation lifecycle.
#[derive(Debug, Clone)]
pub struct ValidationTask {
    pub blob_id: BlobId,
    pub provider: Address,
    pub data_root: Hash,
    pub chunk_count: usize,
    pub status: ValidationStatus,
    pub started_epoch: Epoch,
    pub last_activity: Epoch,
    pub rounds_completed: u32,
    pub rounds_needed: u32,
}

/// Provider reliability stats for reputation scoring.
#[derive(Debug, Clone, Default)]
pub struct ProviderStats {
    pub challenges_sent: u64,
    pub responses_received: u64,
    pub valid_responses: u64,
    pub timeouts: u64,
    pub invalid_responses: u64,
}

impl ProviderStats {
    /// Response rate as a fraction [0.0, 1.0].
    pub fn response_rate(&self) -> f64 {
        if self.challenges_sent == 0 {
            return 1.0;
        }
        self.responses_received as f64 / self.challenges_sent as f64
    }

    /// Validity rate of received responses [0.0, 1.0].
    pub fn validity_rate(&self) -> f64 {
        if self.responses_received == 0 {
            return 1.0;
        }
        self.valid_responses as f64 / self.responses_received as f64
    }
}

/// The DAS validator engine.
#[derive(Debug)]
pub struct DasValidator {
    /// Our validator identity.
    validator_id: Address,
    /// Active validation tasks keyed by blob ID.
    tasks: HashMap<BlobId, ValidationTask>,
    /// Queue of blobs waiting to be validated.
    queue: VecDeque<BlobId>,
    /// Provider reliability tracking.
    provider_stats: HashMap<Address, ProviderStats>,
    /// Outbound sample requests (consumed by P2P layer).
    outbox: VecDeque<SampleRequest>,
    /// Blobs we've finished with (confirmed or failed).
    completed: HashSet<BlobId>,
    /// Current epoch.
    current_epoch: Epoch,
}

impl DasValidator {
    pub fn new(validator_id: Address) -> Self {
        Self {
            validator_id,
            tasks: HashMap::new(),
            queue: VecDeque::new(),
            provider_stats: HashMap::new(),
            outbox: VecDeque::new(),
            completed: HashSet::new(),
            current_epoch: 0,
        }
    }

    pub fn set_epoch(&mut self, epoch: Epoch) {
        self.current_epoch = epoch;
    }

    /// Register a new DAS commitment to validate.
    pub fn on_new_commitment(
        &mut self,
        blob_id: BlobId,
        provider: Address,
        data_root: Hash,
        chunk_count: usize,
    ) -> Result<(), &'static str> {
        if self.tasks.contains_key(&blob_id) || self.completed.contains(&blob_id) {
            return Err("already tracking this blob");
        }
        if self.tasks.len() >= MAX_CONCURRENT_VALIDATIONS {
            // Queue it for later
            self.queue.push_back(blob_id);
            self.tasks.insert(blob_id, ValidationTask {
                blob_id,
                provider,
                data_root,
                chunk_count,
                status: ValidationStatus::Queued,
                started_epoch: self.current_epoch,
                last_activity: self.current_epoch,
                rounds_completed: 0,
                rounds_needed: REQUIRED_ROUNDS,
            });
            return Ok(());
        }
        self.tasks.insert(blob_id, ValidationTask {
            blob_id,
            provider,
            data_root,
            chunk_count,
            status: ValidationStatus::Sampling { round: 0, retries: 0 },
            started_epoch: self.current_epoch,
            last_activity: self.current_epoch,
            rounds_completed: 0,
            rounds_needed: REQUIRED_ROUNDS,
        });
        Ok(())
    }

    /// Generate sample requests for all active tasks at the current epoch.
    /// Uses epoch + validator_id as randomness source.
    pub fn generate_requests(&mut self) -> Vec<SampleRequest> {
        let mut requests = Vec::new();

        // Promote queued tasks if slots available
        while self.active_count() < MAX_CONCURRENT_VALIDATIONS {
            if let Some(blob_id) = self.queue.pop_front() {
                if let Some(task) = self.tasks.get_mut(&blob_id) {
                    if task.status == ValidationStatus::Queued {
                        task.status = ValidationStatus::Sampling { round: 0, retries: 0 };
                    }
                }
            } else {
                break;
            }
        }

        let blob_ids: Vec<BlobId> = self.tasks.keys().cloned().collect();
        for blob_id in blob_ids {
            let task = self.tasks.get(&blob_id).unwrap();
            let (round, _retries) = match task.status {
                ValidationStatus::Sampling { round, retries } => (round, retries),
                _ => continue,
            };

            // Derive randomness from epoch + validator + round
            let randomness = self.derive_randomness(&blob_id, round);
            let indices = self.sample_indices(&randomness, task.chunk_count);

            let req = SampleRequest {
                blob_id,
                indices,
                round,
                provider: task.provider,
            };
            requests.push(req.clone());
            self.outbox.push_back(req);
        }

        requests
    }

    /// Process a response from a provider.
    pub fn on_response(&mut self, response: SampleResponse) -> Result<(), &'static str> {
        match response {
            SampleResponse::Success { blob_id, round, proofs } => {
                // Extract what we need before mutable borrow
                let task = self.tasks.get(&blob_id).ok_or("unknown blob")?;
                let provider = task.provider;
                let data_root = task.data_root;
                let chunk_count = task.chunk_count;
                let status = task.status;
                let blob_id_copy = task.blob_id;

                // Verify proofs
                match status {
                    ValidationStatus::Sampling { round: r, .. } if r == round => {}
                    _ => return Err("unexpected round"),
                }
                if proofs.len() != SAMPLES_PER_ROUND {
                    return Err("wrong proof count");
                }
                let randomness = self.derive_randomness(&blob_id_copy, round);
                let expected_indices = self.sample_indices(&randomness, chunk_count);
                for (proof, &expected_idx) in proofs.iter().zip(expected_indices.iter()) {
                    if proof.index != expected_idx {
                        return Err("proof index mismatch");
                    }
                    let leaf = hash_leaf(proof.index, &proof.data);
                    if !verify_proof(&data_root, &leaf, proof.index, &proof.proof) {
                        return Err("invalid merkle proof");
                    }
                }

                // Now mutate
                let task = self.tasks.get_mut(&blob_id).unwrap();
                task.rounds_completed += 1;
                task.last_activity = self.current_epoch;
                if task.rounds_completed >= task.rounds_needed {
                    task.status = ValidationStatus::Confirmed;
                } else {
                    task.status = ValidationStatus::Sampling { round: round + 1, retries: 0 };
                }

                let stats = self.provider_stats.entry(provider).or_default();
                stats.challenges_sent += 1;
                stats.responses_received += 1;
                stats.valid_responses += 1;
                Ok(())
            }
            SampleResponse::Timeout { blob_id, round } => {
                let task = self.tasks.get(&blob_id).ok_or("unknown blob")?;
                let provider = task.provider;
                let status = task.status;

                let stats = self.provider_stats.entry(provider).or_default();
                stats.challenges_sent += 1;
                stats.timeouts += 1;

                let task = self.tasks.get_mut(&blob_id).unwrap();
                Self::handle_failure_static(task, round, status);
                Ok(())
            }
            SampleResponse::Invalid { blob_id, round, .. } => {
                let task = self.tasks.get(&blob_id).ok_or("unknown blob")?;
                let provider = task.provider;
                let status = task.status;

                let stats = self.provider_stats.entry(provider).or_default();
                stats.challenges_sent += 1;
                stats.responses_received += 1;
                stats.invalid_responses += 1;

                let task = self.tasks.get_mut(&blob_id).unwrap();
                Self::handle_failure_static(task, round, status);
                Ok(())
            }
        }
    }

    /// Drain the outbox of pending sample requests.
    pub fn drain_outbox(&mut self) -> Vec<SampleRequest> {
        self.outbox.drain(..).collect()
    }

    /// Get the status of a specific blob validation.
    pub fn task_status(&self, blob_id: &BlobId) -> Option<ValidationStatus> {
        self.tasks.get(blob_id).map(|t| t.status)
    }

    /// Get provider reliability stats.
    pub fn provider_stats(&self, provider: &Address) -> Option<&ProviderStats> {
        self.provider_stats.get(provider)
    }

    /// Number of actively sampling tasks.
    pub fn active_count(&self) -> usize {
        self.tasks.values().filter(|t| matches!(t.status, ValidationStatus::Sampling { .. })).count()
    }

    /// Number of completed (confirmed + failed) blobs.
    pub fn completed_count(&self) -> usize {
        self.completed.len()
    }

    /// Number of queued tasks.
    pub fn queued_count(&self) -> usize {
        self.queue.len()
    }

    /// Clean up completed tasks and free slots.
    pub fn gc_completed(&mut self) {
        let done: Vec<BlobId> = self.tasks.iter()
            .filter(|(_, t)| matches!(t.status, ValidationStatus::Confirmed | ValidationStatus::Failed | ValidationStatus::Abandoned))
            .map(|(id, _)| *id)
            .collect();
        for id in done {
            self.tasks.remove(&id);
            self.completed.insert(id);
        }
    }

    // ─── Internal ────────────────────────────────────────────────────────

    fn derive_randomness(&self, blob_id: &BlobId, round: u32) -> Hash {
        let mut h = Sha256::new();
        h.update(b"das-val-rand:");
        h.update(self.current_epoch.to_le_bytes());
        h.update(self.validator_id.0);
        h.update(blob_id.0);
        h.update(round.to_le_bytes());
        h.finalize().into()
    }

    fn sample_indices(&self, randomness: &Hash, chunk_count: usize) -> Vec<usize> {
        let mut indices = Vec::with_capacity(SAMPLES_PER_ROUND);
        for i in 0..SAMPLES_PER_ROUND {
            let mut h = Sha256::new();
            h.update(randomness);
            h.update(i.to_le_bytes());
            let hash: [u8; 32] = h.finalize().into();
            let idx = u64::from_le_bytes(hash[..8].try_into().unwrap()) as usize % chunk_count;
            indices.push(idx);
        }
        indices
    }

    fn handle_failure_static(task: &mut ValidationTask, round: u32, status: ValidationStatus) {
        match status {
            ValidationStatus::Sampling { round: r, retries } if r == round => {
                if retries + 1 >= SAMPLE_RETRY_LIMIT {
                    task.status = ValidationStatus::Failed;
                } else {
                    task.status = ValidationStatus::Sampling {
                        round,
                        retries: retries + 1,
                    };
                }
            }
            _ => {} // Ignore stale responses
        }
    }
}

// Re-export for convenience
pub use prova_chain::das::{hash_leaf, verify_proof};

// ─── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use prova_chain::das::{prepare_blob, build_chunk_proofs};

    fn make_chunks(n: usize, size: usize) -> Vec<Vec<u8>> {
        (0..n).map(|i| vec![(i & 0xff) as u8; size]).collect()
    }

    fn setup_validator() -> (DasValidator, BlobId, Hash, Vec<Vec<u8>>, Vec<Vec<Hash>>) {
        let mut v = DasValidator::new(Address::test(99));
        v.set_epoch(10);
        let original = make_chunks(8, 64);
        let (blob_id, root, chunks, layers) = prepare_blob(&original);
        (v, blob_id, root, chunks, layers)
    }

    #[test]
    fn test_new_validator_empty() {
        let v = DasValidator::new(Address::test(1));
        assert_eq!(v.active_count(), 0);
        assert_eq!(v.completed_count(), 0);
        assert_eq!(v.queued_count(), 0);
    }

    #[test]
    fn test_register_commitment() {
        let (mut v, blob_id, root, chunks, _) = setup_validator();
        v.on_new_commitment(blob_id, Address::test(1), root, chunks.len()).unwrap();
        assert_eq!(v.active_count(), 1);
        assert_eq!(v.task_status(&blob_id), Some(ValidationStatus::Sampling { round: 0, retries: 0 }));
    }

    #[test]
    fn test_duplicate_commitment_rejected() {
        let (mut v, blob_id, root, chunks, _) = setup_validator();
        v.on_new_commitment(blob_id, Address::test(1), root, chunks.len()).unwrap();
        let err = v.on_new_commitment(blob_id, Address::test(1), root, chunks.len()).unwrap_err();
        assert_eq!(err, "already tracking this blob");
    }

    #[test]
    fn test_generate_requests() {
        let (mut v, blob_id, root, chunks, _) = setup_validator();
        v.on_new_commitment(blob_id, Address::test(1), root, chunks.len()).unwrap();
        let reqs = v.generate_requests();
        assert_eq!(reqs.len(), 1);
        assert_eq!(reqs[0].blob_id, blob_id);
        assert_eq!(reqs[0].indices.len(), SAMPLES_PER_ROUND);
        assert_eq!(reqs[0].provider, Address::test(1));
    }

    #[test]
    fn test_full_validation_flow() {
        let (mut v, blob_id, root, chunks, layers) = setup_validator();
        v.on_new_commitment(blob_id, Address::test(1), root, chunks.len()).unwrap();

        for round in 0..REQUIRED_ROUNDS {
            let reqs = v.generate_requests();
            assert_eq!(reqs.len(), 1);
            let proofs = build_chunk_proofs(&reqs[0].indices, &chunks, &layers);
            v.on_response(SampleResponse::Success {
                blob_id,
                round,
                proofs,
            }).unwrap();
        }

        assert_eq!(v.task_status(&blob_id), Some(ValidationStatus::Confirmed));
    }

    #[test]
    fn test_timeout_triggers_retry() {
        let (mut v, blob_id, root, chunks, _) = setup_validator();
        v.on_new_commitment(blob_id, Address::test(1), root, chunks.len()).unwrap();
        v.generate_requests();

        v.on_response(SampleResponse::Timeout { blob_id, round: 0 }).unwrap();
        assert_eq!(v.task_status(&blob_id), Some(ValidationStatus::Sampling { round: 0, retries: 1 }));
    }

    #[test]
    fn test_max_retries_marks_failed() {
        let (mut v, blob_id, root, chunks, _) = setup_validator();
        v.on_new_commitment(blob_id, Address::test(1), root, chunks.len()).unwrap();
        v.generate_requests();

        for _ in 0..SAMPLE_RETRY_LIMIT {
            v.on_response(SampleResponse::Timeout { blob_id, round: 0 }).unwrap();
        }

        assert_eq!(v.task_status(&blob_id), Some(ValidationStatus::Failed));
    }

    #[test]
    fn test_invalid_response_counts_retry() {
        let (mut v, blob_id, root, chunks, _) = setup_validator();
        v.on_new_commitment(blob_id, Address::test(1), root, chunks.len()).unwrap();
        v.generate_requests();

        v.on_response(SampleResponse::Invalid {
            blob_id,
            round: 0,
            reason: "bad data".into(),
        }).unwrap();
        assert_eq!(v.task_status(&blob_id), Some(ValidationStatus::Sampling { round: 0, retries: 1 }));
    }

    #[test]
    fn test_provider_stats_tracking() {
        let (mut v, blob_id, root, chunks, layers) = setup_validator();
        let provider = Address::test(1);
        v.on_new_commitment(blob_id, provider, root, chunks.len()).unwrap();

        // One successful round
        let reqs = v.generate_requests();
        let proofs = build_chunk_proofs(&reqs[0].indices, &chunks, &layers);
        v.on_response(SampleResponse::Success { blob_id, round: 0, proofs }).unwrap();

        let stats = v.provider_stats(&provider).unwrap();
        assert_eq!(stats.challenges_sent, 1);
        assert_eq!(stats.valid_responses, 1);
        assert_eq!(stats.timeouts, 0);
        assert!((stats.response_rate() - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_gc_completed_frees_slots() {
        let (mut v, blob_id, root, chunks, layers) = setup_validator();
        v.on_new_commitment(blob_id, Address::test(1), root, chunks.len()).unwrap();

        // Complete all rounds
        for round in 0..REQUIRED_ROUNDS {
            let reqs = v.generate_requests();
            let proofs = build_chunk_proofs(&reqs[0].indices, &chunks, &layers);
            v.on_response(SampleResponse::Success { blob_id, round, proofs }).unwrap();
        }

        assert_eq!(v.active_count(), 0); // Confirmed, not "sampling"
        v.gc_completed();
        assert_eq!(v.completed_count(), 1);
        assert!(v.task_status(&blob_id).is_none()); // Removed from tasks
    }

    #[test]
    fn test_drain_outbox() {
        let (mut v, blob_id, root, chunks, _) = setup_validator();
        v.on_new_commitment(blob_id, Address::test(1), root, chunks.len()).unwrap();
        v.generate_requests();

        let outbox = v.drain_outbox();
        assert_eq!(outbox.len(), 1);
        assert_eq!(v.drain_outbox().len(), 0); // Drained
    }

    #[test]
    fn test_multiple_blobs_concurrent() {
        let mut v = DasValidator::new(Address::test(99));
        v.set_epoch(10);

        let orig1 = make_chunks(4, 32);
        let orig2 = make_chunks(4, 64);
        let (id1, root1, chunks1, _) = prepare_blob(&orig1);
        let (id2, root2, chunks2, _) = prepare_blob(&orig2);

        v.on_new_commitment(id1, Address::test(1), root1, chunks1.len()).unwrap();
        v.on_new_commitment(id2, Address::test(2), root2, chunks2.len()).unwrap();

        let reqs = v.generate_requests();
        assert_eq!(reqs.len(), 2);
        assert_eq!(v.active_count(), 2);
    }

    #[test]
    fn test_wrong_round_rejected() {
        let (mut v, blob_id, root, chunks, layers) = setup_validator();
        v.on_new_commitment(blob_id, Address::test(1), root, chunks.len()).unwrap();
        let reqs = v.generate_requests();
        let proofs = build_chunk_proofs(&reqs[0].indices, &chunks, &layers);

        // Try round 1 when expecting round 0
        let err = v.on_response(SampleResponse::Success { blob_id, round: 1, proofs }).unwrap_err();
        assert_eq!(err, "unexpected round");
    }

    #[test]
    fn test_provider_stats_mixed() {
        let (mut v, blob_id, root, chunks, _) = setup_validator();
        let provider = Address::test(1);
        v.on_new_commitment(blob_id, provider, root, chunks.len()).unwrap();
        v.generate_requests();

        // Timeout then invalid
        v.on_response(SampleResponse::Timeout { blob_id, round: 0 }).unwrap();
        v.on_response(SampleResponse::Invalid { blob_id, round: 0, reason: "bad".into() }).unwrap();

        let stats = v.provider_stats(&provider).unwrap();
        assert_eq!(stats.challenges_sent, 2);
        assert_eq!(stats.timeouts, 1);
        assert_eq!(stats.invalid_responses, 1);
        assert!((stats.response_rate() - 0.5).abs() < f64::EPSILON);
        assert!((stats.validity_rate() - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_unknown_blob_response_rejected() {
        let mut v = DasValidator::new(Address::test(99));
        let fake_id = BlobId([42u8; 32]);
        let err = v.on_response(SampleResponse::Timeout { blob_id: fake_id, round: 0 }).unwrap_err();
        assert_eq!(err, "unknown blob");
    }
}
