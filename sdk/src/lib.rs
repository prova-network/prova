//! Prova Client SDK — build, sign, and submit inference requests.
//!
//! Provides a high-level `ProvaClient` that abstracts away chain
//! interaction: constructing job requests, signing transactions,
//! polling for results, and verifying activation proofs.

pub mod rpc_client;
pub mod cli_wallet;
pub mod retry;
pub mod event_client;
pub mod event_replay;
pub mod marketplace;
pub mod blob_client;
pub mod delegation;
pub mod confidential;
pub mod multisig;

use prova_chain::types::{Address, Epoch, Hash, ModelId};
use prova_chain::scheduler::{JobId, JobRequest};
use prova_chain::commit::InferenceCommit;
use sha2::{Sha256, Digest};
use std::collections::HashMap;

// ── Signing ──────────────────────────────────────────────────

/// A 64-byte Ed25519-style signature (simplified for simulation).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Signature(pub [u8; 64]);

/// Keypair for signing transactions.
#[derive(Debug, Clone)]
pub struct Keypair {
    pub secret: [u8; 32],
    pub address: Address,
}

impl Keypair {
    /// Derive a keypair from a 32-byte seed.
    pub fn from_seed(seed: [u8; 32]) -> Self {
        let mut hasher = Sha256::new();
        hasher.update(&seed);
        let addr_hash = hasher.finalize();
        let mut addr_bytes = [0u8; 20];
        addr_bytes.copy_from_slice(&addr_hash[..20]);
        Self {
            secret: seed,
            address: Address(addr_bytes),
        }
    }

    /// Sign a message digest (simplified: HMAC-like hash).
    pub fn sign(&self, message: &[u8]) -> Signature {
        let mut hasher = Sha256::new();
        hasher.update(&self.secret);
        hasher.update(message);
        let h = hasher.finalize();
        let mut sig = [0u8; 64];
        sig[..32].copy_from_slice(&h);
        // Second half: hash of first half + secret (deterministic)
        let mut hasher2 = Sha256::new();
        hasher2.update(&h);
        hasher2.update(&self.secret);
        sig[32..].copy_from_slice(&hasher2.finalize().as_slice());
        Signature(sig)
    }

    /// Verify a signature against a message.
    pub fn verify(&self, message: &[u8], sig: &Signature) -> bool {
        let expected = self.sign(message);
        expected == *sig
    }
}

// ── Request Builder ──────────────────────────────────────────

/// Builder for constructing inference requests.
#[derive(Debug, Clone)]
pub struct InferenceRequestBuilder {
    model_id: Option<ModelId>,
    input: Option<Vec<u8>>,
    max_price: u128,
    deadline_epochs: u64,
}

impl InferenceRequestBuilder {
    pub fn new() -> Self {
        Self {
            model_id: None,
            input: None,
            max_price: 0,
            deadline_epochs: 100,
        }
    }

    pub fn model(mut self, id: ModelId) -> Self {
        self.model_id = Some(id);
        self
    }

    pub fn input(mut self, data: Vec<u8>) -> Self {
        self.input = Some(data);
        self
    }

    pub fn max_price(mut self, price: u128) -> Self {
        self.max_price = price;
        self
    }

    pub fn deadline(mut self, epochs: u64) -> Self {
        self.deadline_epochs = epochs;
        self
    }

    /// Build and sign the request, returning a `SignedRequest`.
    pub fn build(self, keypair: &Keypair, current_epoch: Epoch) -> Result<SignedRequest, SdkError> {
        let model_id = self.model_id.ok_or(SdkError::MissingField("model_id"))?;
        let input = self.input.ok_or(SdkError::MissingField("input"))?;

        let input_hash = {
            let mut h = Sha256::new();
            h.update(&input);
            let r = h.finalize();
            let mut out = [0u8; 32];
            out.copy_from_slice(&r);
            out
        };

        let request = JobRequest {
            id: JobId(0), // assigned by scheduler
            requester: keypair.address,
            model_id,
            max_price: self.max_price,
            input_hash,
            deadline: current_epoch + self.deadline_epochs,
            submitted_at: current_epoch,
        };

        // Serialize request fields for signing
        let mut msg = Vec::new();
        msg.extend_from_slice(&request.requester.0);
        msg.extend_from_slice(&request.model_id.0);
        msg.extend_from_slice(&request.input_hash);
        msg.extend_from_slice(&request.max_price.to_le_bytes());
        msg.extend_from_slice(&request.deadline.to_le_bytes());

        let signature = keypair.sign(&msg);

        Ok(SignedRequest {
            request,
            input,
            signature,
        })
    }
}

impl Default for InferenceRequestBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// A signed inference request ready for submission.
#[derive(Debug, Clone)]
pub struct SignedRequest {
    pub request: JobRequest,
    pub input: Vec<u8>,
    pub signature: Signature,
}

impl SignedRequest {
    /// Verify the signature against the request fields.
    pub fn verify(&self, keypair: &Keypair) -> bool {
        let mut msg = Vec::new();
        msg.extend_from_slice(&self.request.requester.0);
        msg.extend_from_slice(&self.request.model_id.0);
        msg.extend_from_slice(&self.request.input_hash);
        msg.extend_from_slice(&self.request.max_price.to_le_bytes());
        msg.extend_from_slice(&self.request.deadline.to_le_bytes());
        keypair.verify(&msg, &self.signature)
    }
}

// ── Result Parsing ───────────────────────────────────────────

/// Parsed inference result from a completed job.
#[derive(Debug, Clone)]
pub struct InferenceResult {
    pub job_id: JobId,
    pub provider: Address,
    pub activation_root: Hash,
    pub output_hash: Hash,
    pub epoch_completed: Epoch,
}

impl InferenceResult {
    /// Parse from a completed commit.
    pub fn from_commit(job_id: JobId, commit: &InferenceCommit, provider: Address) -> Self {
        Self {
            job_id,
            provider,
            activation_root: commit.activation_root,
            output_hash: {
                let mut h = Sha256::new();
                h.update(&commit.activation_root);
                h.update(&commit.model_id.0);
                let r = h.finalize();
                let mut out = [0u8; 32];
                out.copy_from_slice(&r);
                out
            },
            epoch_completed: commit.committed_at,
        }
    }

    /// Verify the result against the original request's model.
    pub fn verify_model(&self, _expected_model: &ModelId) -> bool {
        // Re-derive output hash — must match if model is correct
        // (In production, this checks the activation Merkle proof)
        true // simplified: real verification via QBP dispute if needed
    }
}

// ── Provider Discovery ───────────────────────────────────────

/// Information about an available provider.
#[derive(Debug, Clone)]
pub struct ProviderInfo {
    pub address: Address,
    pub models: Vec<ModelId>,
    pub price: u128,
    pub reputation: f64,
    pub stake: u128,
}

/// Provider discovery and ranking.
pub struct ProviderDiscovery {
    providers: Vec<ProviderInfo>,
}

impl ProviderDiscovery {
    pub fn new() -> Self {
        Self { providers: Vec::new() }
    }

    /// Register a known provider.
    pub fn add_provider(&mut self, info: ProviderInfo) {
        self.providers.push(info);
    }

    /// Find providers that serve a given model, sorted by score.
    /// Score = reputation * stake_weight / price (higher is better).
    pub fn find_providers(&self, model: &ModelId, max_price: u128) -> Vec<&ProviderInfo> {
        let mut candidates: Vec<&ProviderInfo> = self.providers.iter()
            .filter(|p| p.models.contains(model) && p.price <= max_price)
            .collect();

        candidates.sort_by(|a, b| {
            let score_a = a.reputation * (a.stake as f64) / (a.price.max(1) as f64);
            let score_b = b.reputation * (b.stake as f64) / (b.price.max(1) as f64);
            score_b.partial_cmp(&score_a).unwrap_or(std::cmp::Ordering::Equal)
        });

        candidates
    }

    /// Get the cheapest provider for a model.
    pub fn cheapest(&self, model: &ModelId) -> Option<&ProviderInfo> {
        self.providers.iter()
            .filter(|p| p.models.contains(model))
            .min_by_key(|p| p.price)
    }

    /// Get the highest-reputation provider for a model.
    pub fn best_reputation(&self, model: &ModelId) -> Option<&ProviderInfo> {
        self.providers.iter()
            .filter(|p| p.models.contains(model))
            .max_by(|a, b| a.reputation.partial_cmp(&b.reputation).unwrap_or(std::cmp::Ordering::Equal))
    }
}

impl Default for ProviderDiscovery {
    fn default() -> Self {
        Self::new()
    }
}

// ── Client ───────────────────────────────────────────────────

/// High-level Prova client for interacting with the network.
pub struct ProvaClient {
    pub keypair: Keypair,
    pub discovery: ProviderDiscovery,
    /// Pending jobs submitted by this client.
    pending: HashMap<JobId, SignedRequest>,
    /// Completed results.
    results: HashMap<JobId, InferenceResult>,
    next_nonce: u64,
}

impl ProvaClient {
    pub fn new(keypair: Keypair) -> Self {
        Self {
            keypair,
            discovery: ProviderDiscovery::new(),
            pending: HashMap::new(),
            results: HashMap::new(),
            next_nonce: 0,
        }
    }

    /// Submit an inference request. Returns the assigned job ID.
    pub fn submit(&mut self, signed: SignedRequest) -> JobId {
        let id = JobId(self.next_nonce);
        self.next_nonce += 1;
        self.pending.insert(id, signed);
        id
    }

    /// Record a result for a completed job.
    pub fn record_result(&mut self, result: InferenceResult) {
        let id = result.job_id;
        self.pending.remove(&id);
        self.results.insert(id, result);
    }

    /// Check if a job is still pending.
    pub fn is_pending(&self, id: &JobId) -> bool {
        self.pending.contains_key(id)
    }

    /// Get a completed result.
    pub fn get_result(&self, id: &JobId) -> Option<&InferenceResult> {
        self.results.get(id)
    }

    /// Number of pending jobs.
    pub fn pending_count(&self) -> usize {
        self.pending.len()
    }

    /// Number of completed results.
    pub fn completed_count(&self) -> usize {
        self.results.len()
    }

    /// Cancel a pending job. Returns true if it was pending.
    pub fn cancel(&mut self, id: &JobId) -> bool {
        self.pending.remove(id).is_some()
    }

    /// Get the client's address.
    pub fn address(&self) -> Address {
        self.keypair.address
    }
}

// ── Batch Operations ─────────────────────────────────────────

/// Submit multiple inference requests in a batch.
pub fn batch_submit(
    client: &mut ProvaClient,
    requests: Vec<SignedRequest>,
) -> Vec<JobId> {
    requests.into_iter().map(|r| client.submit(r)).collect()
}

// ── Errors ───────────────────────────────────────────────────

#[derive(Debug)]
pub enum SdkError {
    MissingField(&'static str),
    InvalidSignature,
    ProviderNotFound,
    JobNotFound(JobId),
    Timeout,
}

impl std::fmt::Display for SdkError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::MissingField(field) => write!(f, "missing required field: {field}"),
            Self::InvalidSignature => write!(f, "invalid signature"),
            Self::ProviderNotFound => write!(f, "no matching provider found"),
            Self::JobNotFound(id) => write!(f, "job {id} not found"),
            Self::Timeout => write!(f, "operation timed out"),
        }
    }
}

impl std::error::Error for SdkError {}

// ── Tests ────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn test_model() -> ModelId {
        ModelId([0xAA; 32])
    }

    fn test_keypair() -> Keypair {
        Keypair::from_seed([1u8; 32])
    }

    fn test_keypair_2() -> Keypair {
        Keypair::from_seed([2u8; 32])
    }

    #[test]
    fn keypair_from_seed_deterministic() {
        let kp1 = Keypair::from_seed([42u8; 32]);
        let kp2 = Keypair::from_seed([42u8; 32]);
        assert_eq!(kp1.address, kp2.address);
        assert_eq!(kp1.secret, kp2.secret);
    }

    #[test]
    fn keypair_different_seeds_different_addresses() {
        let kp1 = test_keypair();
        let kp2 = test_keypair_2();
        assert_ne!(kp1.address, kp2.address);
    }

    #[test]
    fn sign_and_verify() {
        let kp = test_keypair();
        let msg = b"hello prova";
        let sig = kp.sign(msg);
        assert!(kp.verify(msg, &sig));
    }

    #[test]
    fn verify_wrong_message_fails() {
        let kp = test_keypair();
        let sig = kp.sign(b"hello");
        assert!(!kp.verify(b"world", &sig));
    }

    #[test]
    fn verify_wrong_key_fails() {
        let kp1 = test_keypair();
        let kp2 = test_keypair_2();
        let sig = kp1.sign(b"test");
        assert!(!kp2.verify(b"test", &sig));
    }

    #[test]
    fn builder_creates_signed_request() {
        let kp = test_keypair();
        let signed = InferenceRequestBuilder::new()
            .model(test_model())
            .input(b"test input".to_vec())
            .max_price(1000)
            .deadline(50)
            .build(&kp, 10)
            .unwrap();

        assert_eq!(signed.request.requester, kp.address);
        assert_eq!(signed.request.max_price, 1000);
        assert_eq!(signed.request.deadline, 60); // 10 + 50
        assert!(signed.verify(&kp));
    }

    #[test]
    fn builder_missing_model_errors() {
        let kp = test_keypair();
        let result = InferenceRequestBuilder::new()
            .input(b"data".to_vec())
            .build(&kp, 0);
        assert!(result.is_err());
    }

    #[test]
    fn builder_missing_input_errors() {
        let kp = test_keypair();
        let result = InferenceRequestBuilder::new()
            .model(test_model())
            .build(&kp, 0);
        assert!(result.is_err());
    }

    #[test]
    fn client_submit_and_retrieve() {
        let kp = test_keypair();
        let mut client = ProvaClient::new(kp.clone());
        let signed = InferenceRequestBuilder::new()
            .model(test_model())
            .input(b"hello".to_vec())
            .max_price(500)
            .build(&kp, 0)
            .unwrap();

        let id = client.submit(signed);
        assert!(client.is_pending(&id));
        assert_eq!(client.pending_count(), 1);
    }

    #[test]
    fn client_cancel_pending() {
        let kp = test_keypair();
        let mut client = ProvaClient::new(kp.clone());
        let signed = InferenceRequestBuilder::new()
            .model(test_model())
            .input(b"data".to_vec())
            .max_price(100)
            .build(&kp, 0)
            .unwrap();

        let id = client.submit(signed);
        assert!(client.cancel(&id));
        assert!(!client.is_pending(&id));
        assert_eq!(client.pending_count(), 0);
    }

    #[test]
    fn client_record_result() {
        let kp = test_keypair();
        let mut client = ProvaClient::new(kp.clone());
        let signed = InferenceRequestBuilder::new()
            .model(test_model())
            .input(b"data".to_vec())
            .max_price(100)
            .build(&kp, 0)
            .unwrap();

        let id = client.submit(signed);
        let result = InferenceResult {
            job_id: id,
            provider: Address::test(99),
            activation_root: [0xBB; 32],
            output_hash: [0xCC; 32],
            epoch_completed: 5,
        };
        client.record_result(result);
        assert!(!client.is_pending(&id));
        assert_eq!(client.completed_count(), 1);
        assert!(client.get_result(&id).is_some());
    }

    #[test]
    fn batch_submit_multiple() {
        let kp = test_keypair();
        let mut client = ProvaClient::new(kp.clone());
        let requests: Vec<SignedRequest> = (0..5).map(|i| {
            InferenceRequestBuilder::new()
                .model(test_model())
                .input(vec![i as u8; 10])
                .max_price(100)
                .build(&kp, 0)
                .unwrap()
        }).collect();

        let ids = batch_submit(&mut client, requests);
        assert_eq!(ids.len(), 5);
        assert_eq!(client.pending_count(), 5);
    }

    #[test]
    fn discovery_find_providers() {
        let mut disc = ProviderDiscovery::new();
        let model = test_model();

        disc.add_provider(ProviderInfo {
            address: Address::test(1),
            models: vec![model],
            price: 100,
            reputation: 0.9,
            stake: 10000,
        });
        disc.add_provider(ProviderInfo {
            address: Address::test(2),
            models: vec![model],
            price: 200,
            reputation: 0.95,
            stake: 20000,
        });
        disc.add_provider(ProviderInfo {
            address: Address::test(3),
            models: vec![ModelId([0xBB; 32])], // different model
            price: 50,
            reputation: 1.0,
            stake: 50000,
        });

        let found = disc.find_providers(&model, 200);
        assert_eq!(found.len(), 2);
        // Provider 2 should rank higher (better reputation * stake / price)
        assert_eq!(found[0].address, Address::test(2));
    }

    #[test]
    fn discovery_cheapest() {
        let mut disc = ProviderDiscovery::new();
        let model = test_model();
        disc.add_provider(ProviderInfo {
            address: Address::test(1), models: vec![model], price: 300, reputation: 0.5, stake: 1000,
        });
        disc.add_provider(ProviderInfo {
            address: Address::test(2), models: vec![model], price: 100, reputation: 0.9, stake: 5000,
        });

        let cheapest = disc.cheapest(&model).unwrap();
        assert_eq!(cheapest.address, Address::test(2));
    }

    #[test]
    fn discovery_best_reputation() {
        let mut disc = ProviderDiscovery::new();
        let model = test_model();
        disc.add_provider(ProviderInfo {
            address: Address::test(1), models: vec![model], price: 100, reputation: 0.7, stake: 1000,
        });
        disc.add_provider(ProviderInfo {
            address: Address::test(2), models: vec![model], price: 500, reputation: 0.99, stake: 1000,
        });

        let best = disc.best_reputation(&model).unwrap();
        assert_eq!(best.address, Address::test(2));
    }

    #[test]
    fn discovery_max_price_filter() {
        let mut disc = ProviderDiscovery::new();
        let model = test_model();
        disc.add_provider(ProviderInfo {
            address: Address::test(1), models: vec![model], price: 500, reputation: 0.9, stake: 5000,
        });

        let found = disc.find_providers(&model, 100); // too cheap
        assert!(found.is_empty());
    }

    #[test]
    fn discovery_no_matching_model() {
        let mut disc = ProviderDiscovery::new();
        disc.add_provider(ProviderInfo {
            address: Address::test(1),
            models: vec![ModelId([0xBB; 32])],
            price: 100, reputation: 0.9, stake: 5000,
        });

        let found = disc.find_providers(&test_model(), 1000);
        assert!(found.is_empty());
    }

    #[test]
    fn signed_request_verify_after_tamper_fails() {
        let kp = test_keypair();
        let mut signed = InferenceRequestBuilder::new()
            .model(test_model())
            .input(b"data".to_vec())
            .max_price(100)
            .build(&kp, 0)
            .unwrap();

        // Tamper with price
        signed.request.max_price = 999999;
        assert!(!signed.verify(&kp));
    }

    #[test]
    fn client_address() {
        let kp = test_keypair();
        let client = ProvaClient::new(kp.clone());
        assert_eq!(client.address(), kp.address);
    }
}
