//! Zero-knowledge proof verifier for activation proofs.
//!
//! Verifies ZK-SNARK proofs that attest to correct inference execution
//! without revealing the actual activations. Enables fully private inference
//! where even disputed results never expose plaintext data.
//!
//! Proof system: Groth16-style (BN254 curve, 2 pairings + public inputs).
//! Verification is O(1) regardless of circuit size — ideal for on-chain use.

use crate::types::*;
use std::collections::HashMap;

// ── Proof Structures ──────────────────────────────────────────────────

/// A verification key for a specific circuit (one per model).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VerificationKey {
    /// Identifier linking to a registered model + architecture pair
    pub circuit_id: CircuitId,
    /// Serialized alpha point (G1)
    pub alpha_g1: [u8; 64],
    /// Serialized beta point (G2)
    pub beta_g2: [u8; 128],
    /// Serialized gamma point (G2)
    pub gamma_g2: [u8; 128],
    /// Serialized delta point (G2)
    pub delta_g2: [u8; 128],
    /// IC points (G1) — one per public input + 1
    pub ic_points: Vec<[u8; 64]>,
    /// Who registered this key (must be model owner or governance)
    pub registrar: Address,
    /// Epoch when registered
    pub registered_at: Epoch,
    /// Whether this key is active
    pub active: bool,
}

/// A ZK-SNARK proof (Groth16).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Proof {
    /// A point (G1)
    pub a: [u8; 64],
    /// B point (G2)
    pub b: [u8; 128],
    /// C point (G1)
    pub c: [u8; 64],
}

/// Public inputs to the proof circuit.
#[derive(Debug, Clone)]
pub struct PublicInputs {
    /// Model ID being proven
    pub model_id: ModelId,
    /// Input hash (H(inference_input))
    pub input_hash: Hash,
    /// Output hash (H(inference_output))
    pub output_hash: Hash,
    /// Activation root (Merkle root of intermediate activations)
    pub activation_root: Hash,
    /// Additional scalar inputs (field elements as 32-byte big-endian)
    pub extra: Vec<[u8; 32]>,
}

impl PublicInputs {
    /// Flatten to ordered field elements for verification.
    pub fn to_field_elements(&self) -> Vec<[u8; 32]> {
        let mut elems = Vec::with_capacity(4 + self.extra.len());
        elems.push(self.model_id.0);
        elems.push(self.input_hash);
        elems.push(self.output_hash);
        elems.push(self.activation_root);
        elems.extend_from_slice(&self.extra);
        elems
    }
}

/// Unique circuit identifier (model + architecture → circuit).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct CircuitId(pub Hash);

/// Result of a verification attempt.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VerifyResult {
    /// Proof is valid
    Valid,
    /// Proof is invalid (pairing check failed)
    Invalid,
    /// No verification key registered for this circuit
    UnknownCircuit,
    /// Public input count doesn't match IC points
    InputMismatch { expected: usize, got: usize },
    /// Verification key is deactivated
    KeyDeactivated,
}

/// A verified proof record stored on-chain.
#[derive(Debug, Clone)]
pub struct ProofRecord {
    pub id: ProofRecordId,
    pub circuit_id: CircuitId,
    pub commit_id: CommitId,
    pub prover: Address,
    pub result: VerifyResult,
    pub epoch: Epoch,
    pub gas_used: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ProofRecordId(pub u64);

// ── Gas Constants ─────────────────────────────────────────────────────

/// Base gas for Groth16 verification (2 pairings).
const ZK_VERIFY_BASE_GAS: u64 = 150_000;
/// Additional gas per public input (EC scalar mul).
const ZK_VERIFY_PER_INPUT_GAS: u64 = 5_000;
/// Gas for registering a verification key.
const VK_REGISTER_GAS: u64 = 50_000;

// ── Verifier Engine ───────────────────────────────────────────────────

/// On-chain ZK proof verifier.
#[derive(Debug)]
pub struct ZkVerifier {
    /// Registered verification keys by circuit ID
    keys: HashMap<CircuitId, VerificationKey>,
    /// Proof records
    records: Vec<ProofRecord>,
    /// Next record ID
    next_id: u64,
}

impl ZkVerifier {
    pub fn new() -> Self {
        Self {
            keys: HashMap::new(),
            records: Vec::new(),
            next_id: 1,
        }
    }

    /// Register a verification key for a circuit.
    /// Returns gas cost.
    pub fn register_key(&mut self, key: VerificationKey) -> Result<u64, ZkError> {
        if key.ic_points.is_empty() {
            return Err(ZkError::InvalidKey("IC points cannot be empty".into()));
        }
        let id = key.circuit_id;
        if self.keys.contains_key(&id) {
            return Err(ZkError::KeyAlreadyRegistered(id));
        }
        self.keys.insert(id, key);
        Ok(VK_REGISTER_GAS)
    }

    /// Deactivate a verification key (governance action).
    pub fn deactivate_key(&mut self, circuit_id: &CircuitId) -> Result<(), ZkError> {
        let key = self
            .keys
            .get_mut(circuit_id)
            .ok_or(ZkError::UnknownCircuit(*circuit_id))?;
        key.active = false;
        Ok(())
    }

    /// Reactivate a previously deactivated key.
    pub fn reactivate_key(&mut self, circuit_id: &CircuitId) -> Result<(), ZkError> {
        let key = self
            .keys
            .get_mut(circuit_id)
            .ok_or(ZkError::UnknownCircuit(*circuit_id))?;
        key.active = true;
        Ok(())
    }

    /// Verify a ZK proof against registered verification key.
    /// Returns (result, gas_used) and stores a proof record.
    pub fn verify(
        &mut self,
        proof: &Proof,
        inputs: &PublicInputs,
        circuit_id: &CircuitId,
        commit_id: CommitId,
        prover: Address,
        epoch: Epoch,
    ) -> (VerifyResult, u64) {
        let gas = self.estimate_gas(inputs);

        // Look up verification key
        let vk = match self.keys.get(circuit_id) {
            Some(vk) => vk,
            None => {
                let result = VerifyResult::UnknownCircuit;
                self.store_record(*circuit_id, commit_id, prover, result.clone(), epoch, gas);
                return (result, gas);
            }
        };

        if !vk.active {
            let result = VerifyResult::KeyDeactivated;
            self.store_record(*circuit_id, commit_id, prover, result.clone(), epoch, gas);
            return (result, gas);
        }

        // Check public input count: IC has n+1 points for n inputs
        let field_elems = inputs.to_field_elements();
        let expected_inputs = vk.ic_points.len().saturating_sub(1);
        if field_elems.len() != expected_inputs {
            let result = VerifyResult::InputMismatch {
                expected: expected_inputs,
                got: field_elems.len(),
            };
            self.store_record(*circuit_id, commit_id, prover, result.clone(), epoch, gas);
            return (result, gas);
        }

        // Simulated Groth16 verification:
        // In production this would do BN254 pairing checks.
        // Here we simulate via a deterministic hash-based check.
        let valid = self.simulated_pairing_check(proof, &field_elems, vk);

        let result = if valid {
            VerifyResult::Valid
        } else {
            VerifyResult::Invalid
        };

        self.store_record(*circuit_id, commit_id, prover, result.clone(), epoch, gas);
        (result, gas)
    }

    /// Estimate gas for verifying a proof with given inputs.
    pub fn estimate_gas(&self, inputs: &PublicInputs) -> u64 {
        let n = inputs.to_field_elements().len() as u64;
        ZK_VERIFY_BASE_GAS + n * ZK_VERIFY_PER_INPUT_GAS
    }

    /// Get a proof record by ID.
    pub fn get_record(&self, id: ProofRecordId) -> Option<&ProofRecord> {
        self.records.iter().find(|r| r.id == id)
    }

    /// Get all proof records for a commit.
    pub fn records_for_commit(&self, commit_id: CommitId) -> Vec<&ProofRecord> {
        self.records
            .iter()
            .filter(|r| r.commit_id == commit_id)
            .collect()
    }

    /// Get verification key for a circuit.
    pub fn get_key(&self, circuit_id: &CircuitId) -> Option<&VerificationKey> {
        self.keys.get(circuit_id)
    }

    /// Total registered circuits.
    pub fn circuit_count(&self) -> usize {
        self.keys.len()
    }

    /// Total proof records.
    pub fn record_count(&self) -> usize {
        self.records.len()
    }

    // ── Internal ──────────────────────────────────────────────────────

    fn store_record(
        &mut self,
        circuit_id: CircuitId,
        commit_id: CommitId,
        prover: Address,
        result: VerifyResult,
        epoch: Epoch,
        gas_used: u64,
    ) {
        let id = ProofRecordId(self.next_id);
        self.next_id += 1;
        self.records.push(ProofRecord {
            id,
            circuit_id,
            commit_id,
            prover,
            result,
            epoch,
            gas_used,
        });
    }

    /// Simulated pairing check. Uses SHA-256 of (proof || inputs || vk.alpha)
    /// to deterministically decide validity. The "valid" condition is that
    /// the first byte of the hash < 0x80 AND the proof A point is non-zero.
    /// This gives us testable valid/invalid paths.
    fn simulated_pairing_check(
        &self,
        proof: &Proof,
        field_elements: &[[u8; 32]],
        vk: &VerificationKey,
    ) -> bool {
        use sha2::{Digest, Sha256};
        let mut hasher = Sha256::new();
        hasher.update(&proof.a);
        hasher.update(&proof.b);
        hasher.update(&proof.c);
        for elem in field_elements {
            hasher.update(elem);
        }
        hasher.update(&vk.alpha_g1);
        let digest: [u8; 32] = hasher.finalize().into();

        // Non-zero proof A point required
        let a_nonzero = proof.a.iter().any(|&b| b != 0);
        a_nonzero && digest[0] < 0x80
    }
}

// ── Errors ────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ZkError {
    UnknownCircuit(CircuitId),
    KeyAlreadyRegistered(CircuitId),
    InvalidKey(String),
}

impl std::fmt::Display for ZkError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ZkError::UnknownCircuit(id) => write!(f, "unknown circuit {:?}", id),
            ZkError::KeyAlreadyRegistered(id) => write!(f, "key already registered for {:?}", id),
            ZkError::InvalidKey(msg) => write!(f, "invalid key: {msg}"),
        }
    }
}

// ── Test Helpers ──────────────────────────────────────────────────────

/// Create a test circuit ID from a byte.
pub fn test_circuit_id(b: u8) -> CircuitId {
    let mut h = [0u8; 32];
    h[0] = b;
    CircuitId(h)
}

/// Create a test verification key with n public inputs.
pub fn test_vk(
    circuit_id: CircuitId,
    registrar: Address,
    epoch: Epoch,
    n_inputs: usize,
) -> VerificationKey {
    VerificationKey {
        circuit_id,
        alpha_g1: [1u8; 64],
        beta_g2: [2u8; 128],
        gamma_g2: [3u8; 128],
        delta_g2: [4u8; 128],
        ic_points: (0..=n_inputs)
            .map(|i| {
                let mut p = [0u8; 64];
                p[0] = i as u8;
                p[1] = 1; // non-zero
                p
            })
            .collect(),
        registrar,
        registered_at: epoch,
        active: true,
    }
}

/// Create a test proof that will pass simulated verification (first hash byte < 0x80).
/// We brute-force the C point's first byte to find a passing proof.
pub fn test_valid_proof(inputs: &PublicInputs, vk: &VerificationKey) -> Proof {
    use sha2::{Digest, Sha256};
    let a = [1u8; 64]; // non-zero
    let b = [2u8; 128];
    let field_elems = inputs.to_field_elements();

    for nonce in 0u8..=255 {
        let mut c = [0u8; 64];
        c[0] = nonce;
        let mut hasher = Sha256::new();
        hasher.update(&a);
        hasher.update(&b);
        hasher.update(&c);
        for elem in &field_elems {
            hasher.update(elem);
        }
        hasher.update(&vk.alpha_g1);
        let digest: [u8; 32] = hasher.finalize().into();
        if digest[0] < 0x80 {
            return Proof { a, b, c };
        }
    }
    // Fallback — statistically shouldn't happen (p ≈ 1 - 0.5^256)
    Proof { a, b, c: [0u8; 64] }
}

/// Create a test proof that will fail simulated verification.
pub fn test_invalid_proof() -> Proof {
    // Zero A point always fails (a_nonzero check)
    Proof {
        a: [0u8; 64],
        b: [0u8; 128],
        c: [0u8; 64],
    }
}

fn test_inputs(model_id: ModelId) -> PublicInputs {
    PublicInputs {
        model_id,
        input_hash: [10u8; 32],
        output_hash: [11u8; 32],
        activation_root: [12u8; 32],
        extra: vec![],
    }
}

// ── Tests ─────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn setup() -> (ZkVerifier, CircuitId, Address, ModelId) {
        let mut verifier = ZkVerifier::new();
        let cid = test_circuit_id(1);
        let registrar = Address::test(1);
        let model_id = ModelId([42u8; 32]);
        let vk = test_vk(cid, registrar, 100, 4); // 4 public inputs
        verifier.register_key(vk).unwrap();
        (verifier, cid, registrar, model_id)
    }

    #[test]
    fn register_and_lookup() {
        let (verifier, cid, registrar, _) = setup();
        let vk = verifier.get_key(&cid).unwrap();
        assert_eq!(vk.registrar, registrar);
        assert!(vk.active);
        assert_eq!(vk.ic_points.len(), 5); // 4 inputs + 1
        assert_eq!(verifier.circuit_count(), 1);
    }

    #[test]
    fn duplicate_key_rejected() {
        let (mut verifier, cid, _, _) = setup();
        let vk2 = test_vk(cid, Address::test(2), 200, 4);
        assert_eq!(
            verifier.register_key(vk2),
            Err(ZkError::KeyAlreadyRegistered(cid))
        );
    }

    #[test]
    fn empty_ic_rejected() {
        let mut verifier = ZkVerifier::new();
        let cid = test_circuit_id(99);
        let mut vk = test_vk(cid, Address::test(1), 100, 0);
        vk.ic_points.clear();
        assert!(matches!(
            verifier.register_key(vk),
            Err(ZkError::InvalidKey(_))
        ));
    }

    #[test]
    fn valid_proof_accepted() {
        let (mut verifier, cid, _, model_id) = setup();
        let inputs = test_inputs(model_id);
        let vk = verifier.get_key(&cid).unwrap().clone();
        let proof = test_valid_proof(&inputs, &vk);
        let commit = CommitId(1);
        let prover = Address::test(5);

        let (result, gas) = verifier.verify(&proof, &inputs, &cid, commit, prover, 150);
        assert_eq!(result, VerifyResult::Valid);
        assert_eq!(gas, ZK_VERIFY_BASE_GAS + 4 * ZK_VERIFY_PER_INPUT_GAS);
        assert_eq!(verifier.record_count(), 1);
    }

    #[test]
    fn invalid_proof_rejected() {
        let (mut verifier, cid, _, model_id) = setup();
        let inputs = test_inputs(model_id);
        let proof = test_invalid_proof();
        let commit = CommitId(2);

        let (result, _) = verifier.verify(&proof, &inputs, &cid, commit, Address::test(5), 150);
        assert_eq!(result, VerifyResult::Invalid);
    }

    #[test]
    fn unknown_circuit_error() {
        let (mut verifier, _, _, model_id) = setup();
        let bad_cid = test_circuit_id(99);
        let inputs = test_inputs(model_id);
        let proof = test_invalid_proof();

        let (result, _) = verifier.verify(
            &proof,
            &inputs,
            &bad_cid,
            CommitId(3),
            Address::test(5),
            150,
        );
        assert_eq!(result, VerifyResult::UnknownCircuit);
    }

    #[test]
    fn input_mismatch_error() {
        let (mut verifier, cid, _, model_id) = setup();
        // VK expects 4 inputs, give 5
        let mut inputs = test_inputs(model_id);
        inputs.extra.push([99u8; 32]);
        let proof = test_invalid_proof();

        let (result, _) =
            verifier.verify(&proof, &inputs, &cid, CommitId(4), Address::test(5), 150);
        assert_eq!(
            result,
            VerifyResult::InputMismatch {
                expected: 4,
                got: 5
            }
        );
    }

    #[test]
    fn deactivated_key_rejected() {
        let (mut verifier, cid, _, model_id) = setup();
        verifier.deactivate_key(&cid).unwrap();

        let inputs = test_inputs(model_id);
        let proof = test_invalid_proof();
        let (result, _) =
            verifier.verify(&proof, &inputs, &cid, CommitId(5), Address::test(5), 150);
        assert_eq!(result, VerifyResult::KeyDeactivated);
    }

    #[test]
    fn reactivate_key() {
        let (mut verifier, cid, _, model_id) = setup();
        verifier.deactivate_key(&cid).unwrap();
        verifier.reactivate_key(&cid).unwrap();

        let inputs = test_inputs(model_id);
        let vk = verifier.get_key(&cid).unwrap().clone();
        let proof = test_valid_proof(&inputs, &vk);
        let (result, _) =
            verifier.verify(&proof, &inputs, &cid, CommitId(6), Address::test(5), 150);
        assert_eq!(result, VerifyResult::Valid);
    }

    #[test]
    fn gas_estimation() {
        let verifier = ZkVerifier::new();
        let inputs = test_inputs(ModelId([0u8; 32]));
        assert_eq!(
            verifier.estimate_gas(&inputs),
            ZK_VERIFY_BASE_GAS + 4 * ZK_VERIFY_PER_INPUT_GAS
        );

        let mut inputs_extra = inputs.clone();
        inputs_extra.extra.push([0u8; 32]);
        assert_eq!(
            verifier.estimate_gas(&inputs_extra),
            ZK_VERIFY_BASE_GAS + 5 * ZK_VERIFY_PER_INPUT_GAS
        );
    }

    #[test]
    fn records_for_commit() {
        let (mut verifier, cid, _, model_id) = setup();
        let inputs = test_inputs(model_id);
        let vk = verifier.get_key(&cid).unwrap().clone();
        let proof = test_valid_proof(&inputs, &vk);
        let commit = CommitId(10);
        let prover = Address::test(5);

        verifier.verify(&proof, &inputs, &cid, commit, prover, 200);
        verifier.verify(&proof, &inputs, &cid, commit, prover, 201);
        verifier.verify(&proof, &inputs, &cid, CommitId(11), prover, 202);

        let records = verifier.records_for_commit(commit);
        assert_eq!(records.len(), 2);
        assert_eq!(records[0].epoch, 200);
        assert_eq!(records[1].epoch, 201);
    }

    #[test]
    fn proof_record_stored_correctly() {
        let (mut verifier, cid, _, model_id) = setup();
        let inputs = test_inputs(model_id);
        let vk = verifier.get_key(&cid).unwrap().clone();
        let proof = test_valid_proof(&inputs, &vk);
        let prover = Address::test(7);

        verifier.verify(&proof, &inputs, &cid, CommitId(20), prover, 300);
        let record = verifier.get_record(ProofRecordId(1)).unwrap();
        assert_eq!(record.circuit_id, cid);
        assert_eq!(record.commit_id, CommitId(20));
        assert_eq!(record.prover, prover);
        assert_eq!(record.result, VerifyResult::Valid);
        assert_eq!(record.epoch, 300);
    }

    #[test]
    fn deactivate_unknown_circuit_error() {
        let mut verifier = ZkVerifier::new();
        let cid = test_circuit_id(42);
        assert_eq!(
            verifier.deactivate_key(&cid),
            Err(ZkError::UnknownCircuit(cid))
        );
    }

    #[test]
    fn multiple_circuits() {
        let mut verifier = ZkVerifier::new();
        let cid1 = test_circuit_id(1);
        let cid2 = test_circuit_id(2);
        let reg = Address::test(1);

        verifier.register_key(test_vk(cid1, reg, 100, 4)).unwrap();
        verifier.register_key(test_vk(cid2, reg, 100, 2)).unwrap();
        assert_eq!(verifier.circuit_count(), 2);

        // cid2 expects 2 inputs — using 4 should fail
        let inputs = test_inputs(ModelId([0u8; 32]));
        let proof = test_invalid_proof();
        let (result, _) = verifier.verify(&proof, &inputs, &cid2, CommitId(1), reg, 100);
        assert_eq!(
            result,
            VerifyResult::InputMismatch {
                expected: 2,
                got: 4
            }
        );
    }
}
