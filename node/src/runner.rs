//! Inference Runner — executes model inference and captures activations.
//!
//! This module provides the interface between the QBP protocol and actual
//! model inference engines (llama.cpp, TensorRT, etc.).
//!
//! In production, the runner:
//! 1. Loads a registered model
//! 2. Runs inference with deterministic settings (seed, temp=0, arch-specific)
//! 3. Captures intermediate activations after each layer
//! 4. Hashes activations into canonical tensor format
//! 5. Builds the Activation Merkle Tree
//! 6. Publishes the commit with the Merkle root

use crate::merkle::{ActivationMerkleTree, Hash, hash_tensor, DType};
use prova_chain::types::{ArchGroup, ModelId, Address, Epoch};
use std::process::Command;

/// Configuration for an inference run.
#[derive(Debug, Clone)]
pub struct InferenceConfig {
    /// Path to the model file.
    pub model_path: String,
    /// Path to the inference engine binary (e.g., llama-cli).
    pub engine_path: String,
    /// Input prompt.
    pub prompt: String,
    /// Random seed (must be fixed for determinism).
    pub seed: u64,
    /// Temperature (should be 0 for deterministic output).
    pub temperature: f32,
    /// Number of tokens to predict.
    pub n_predict: u32,
    /// Architecture group this run belongs to.
    pub arch_group: ArchGroup,
}

/// Result of an inference run with captured activations.
#[derive(Debug)]
pub struct InferenceResult {
    /// Generated output text.
    pub output: String,
    /// Per-layer activation hashes (index 0 = input, 1..N = layer outputs).
    pub activation_hashes: Vec<Hash>,
    /// The Activation Merkle Tree built from the hashes.
    pub merkle_tree: ActivationMerkleTree,
    /// SHA-256 of the input prompt.
    pub input_hash: Hash,
}

impl InferenceResult {
    /// Get the Merkle root for committing.
    pub fn activation_root(&self) -> Hash {
        self.merkle_tree.root()
    }

    /// Number of layers (excluding input).
    pub fn layer_count(&self) -> u32 {
        (self.activation_hashes.len() - 1) as u32
    }
}

/// Mock inference runner for testing and simulation.
///
/// Generates deterministic fake activations based on model ID and prompt.
pub struct MockRunner;

impl MockRunner {
    /// Run mock inference that produces deterministic activations.
    pub fn run(model_id: &ModelId, prompt: &str, layer_count: u32) -> InferenceResult {
        use sha2::{Digest, Sha256};

        // Hash the input
        let input_hash: Hash = {
            let mut h = Sha256::new();
            h.update(b"prova-input:");
            h.update(prompt.as_bytes());
            h.finalize().into()
        };

        // Generate deterministic activation hashes
        // In production, these come from actual tensor data via hash_tensor()
        let mut activation_hashes = Vec::with_capacity((layer_count + 1) as usize);

        // Input activation (hash of prompt embedding)
        activation_hashes.push({
            let mut h = Sha256::new();
            h.update(b"activation:0:");
            h.update(&model_id.0);
            h.update(&input_hash);
            h.finalize().into()
        });

        // Layer activations
        for i in 1..=layer_count {
            let prev = &activation_hashes[(i - 1) as usize];
            let hash: Hash = {
                let mut h = Sha256::new();
                h.update(b"activation:");
                h.update(i.to_le_bytes());
                h.update(b":");
                h.update(&model_id.0);
                h.update(prev);
                h.finalize().into()
            };
            activation_hashes.push(hash);
        }

        // Build Merkle tree
        let merkle_tree = ActivationMerkleTree::build(&activation_hashes);

        InferenceResult {
            output: format!("[mock output for '{prompt}' on model {:?}]", model_id),
            activation_hashes,
            merkle_tree,
            input_hash,
        }
    }

    /// Run mock inference with a single layer producing a different result.
    /// Used to simulate a faulty provider.
    pub fn run_faulty(
        model_id: &ModelId,
        prompt: &str,
        layer_count: u32,
        fault_at_layer: u32,
    ) -> InferenceResult {
        use sha2::{Digest, Sha256};

        let correct = Self::run(model_id, prompt, layer_count);
        let mut activation_hashes = correct.activation_hashes;

        // Corrupt the activation at the faulty layer
        activation_hashes[fault_at_layer as usize] = {
            let mut h = Sha256::new();
            h.update(b"CORRUPTED:");
            h.update(fault_at_layer.to_le_bytes());
            h.finalize().into()
        };

        // Recompute downstream activations (they'd be different due to corruption)
        for i in (fault_at_layer + 1)..=layer_count {
            let prev = &activation_hashes[(i - 1) as usize];
            activation_hashes[i as usize] = {
                let mut h = Sha256::new();
                h.update(b"corrupted-chain:");
                h.update(i.to_le_bytes());
                h.update(prev);
                h.finalize().into()
            };
        }

        let merkle_tree = ActivationMerkleTree::build(&activation_hashes);

        InferenceResult {
            output: format!("[faulty output, diverged at layer {fault_at_layer}]"),
            activation_hashes,
            merkle_tree,
            input_hash: correct.input_hash,
        }
    }
}

/// Check if an inference engine is available at the given path.
pub fn check_engine(engine_path: &str) -> bool {
    Command::new(engine_path)
        .arg("--version")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::merkle::verify_proof;

    fn test_model_id() -> ModelId {
        ModelId([0x42; 32])
    }

    #[test]
    fn test_mock_runner_deterministic() {
        let r1 = MockRunner::run(&test_model_id(), "test prompt", 32);
        let r2 = MockRunner::run(&test_model_id(), "test prompt", 32);

        assert_eq!(r1.activation_root(), r2.activation_root());
        assert_eq!(r1.activation_hashes, r2.activation_hashes);
        assert_eq!(r1.layer_count(), 32);
    }

    #[test]
    fn test_mock_runner_different_prompts() {
        let r1 = MockRunner::run(&test_model_id(), "prompt A", 32);
        let r2 = MockRunner::run(&test_model_id(), "prompt B", 32);

        assert_ne!(r1.activation_root(), r2.activation_root());
    }

    #[test]
    fn test_mock_runner_proofs_valid() {
        let result = MockRunner::run(&test_model_id(), "test", 32);
        let root = result.activation_root();

        for i in 0..33 {
            let proof = result.merkle_tree.prove(i);
            assert!(verify_proof(&proof, &root), "proof failed for layer {i}");
        }
    }

    #[test]
    fn test_faulty_runner_diverges() {
        let correct = MockRunner::run(&test_model_id(), "test", 32);
        let faulty = MockRunner::run_faulty(&test_model_id(), "test", 32, 15);

        // Roots should differ
        assert_ne!(correct.activation_root(), faulty.activation_root());

        // Activations should match before fault, differ from fault onwards
        for i in 0..15 {
            assert_eq!(
                correct.activation_hashes[i], faulty.activation_hashes[i],
                "layer {i} should match before fault"
            );
        }
        assert_ne!(
            correct.activation_hashes[15], faulty.activation_hashes[15],
            "layer 15 (fault point) should differ"
        );
    }

    #[test]
    fn test_faulty_runner_cascades() {
        let correct = MockRunner::run(&test_model_id(), "test", 32);
        let faulty = MockRunner::run_faulty(&test_model_id(), "test", 32, 10);

        // Everything from layer 10 onward should differ
        for i in 10..33 {
            assert_ne!(
                correct.activation_hashes[i], faulty.activation_hashes[i],
                "layer {i} should differ after fault"
            );
        }
    }

    #[test]
    fn test_end_to_end_dispute_detection() {
        // Simulate: provider runs correctly, challenger claims different result
        let model_id = test_model_id();
        let prompt = "The meaning of life is";
        let layer_count = 32;

        let provider_result = MockRunner::run(&model_id, prompt, layer_count);
        let challenger_result = MockRunner::run_faulty(&model_id, prompt, layer_count, 21);

        // Roots differ → dispute is valid
        assert_ne!(provider_result.activation_root(), challenger_result.activation_root());

        // Find the first layer where they disagree
        let first_disagreement = provider_result
            .activation_hashes
            .iter()
            .zip(challenger_result.activation_hashes.iter())
            .position(|(a, b)| a != b)
            .expect("should find disagreement");

        assert_eq!(first_disagreement, 21, "should find fault at layer 21");
    }
}
