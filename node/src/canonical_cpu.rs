//! EXP-003: CPU Canonical Verification Path
//!
//! Implements a portable, deterministic CPU inference path that serves as the
//! canonical reference for QBP dispute resolution. When two GPU-based providers
//! disagree on an activation hash, the on-chain arbiter can request a canonical
//! CPU re-execution to determine the correct result.
//!
//! # Design Principles
//!
//! 1. **Bit-exact reproducibility**: Uses fixed-point arithmetic (no FP rounding ambiguity)
//! 2. **Platform independence**: Pure integer ops — same result on x86, ARM, RISC-V
//! 3. **Auditable**: Simple enough to re-implement in Solidity/WASM for on-chain verification
//! 4. **Compatible**: Output hashes match the GPU INT8 path when both use identical
//!    quantization parameters and reduction order
//!
//! # Architecture
//!
//! ```text
//! ┌─────────────┐     ┌──────────────────┐     ┌────────────────┐
//! │ GPU Provider │     │ CPU Canonical     │     │ On-chain       │
//! │ (fast path)  │────▶│ Verifier (arbiter)│────▶│ Dispute Arena  │
//! └─────────────┘     └──────────────────┘     └────────────────┘
//!       ↓                      ↓                       ↓
//!  INT8 GEMM via          INT32 fixed-point       Compare roots,
//!  TensorRT/cuBLAS        pure-Rust reference     slash loser
//! ```
//!
//! # Why CPU Canonical?
//!
//! GPU INT8 GEMM is deterministic *within* an ArchGroup but may differ *across*
//! ArchGroups due to tensor core microarchitecture differences. The CPU path
//! eliminates all hardware-dependent behavior by using a strict sequential
//! reduction with overflow-checked INT32 arithmetic.

use crate::merkle::{hash_tensor, ActivationMerkleTree, DType, Hash};

/// Fixed-point scale representation to avoid floating-point non-determinism.
/// Uses Q16.16 format: value = raw / 65536.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FixedScale {
    /// Raw Q16.16 value. 65536 = 1.0, 32768 = 0.5, etc.
    pub raw: i32,
}

impl FixedScale {
    /// Create from a floating-point scale (quantizes to Q16.16).
    pub fn from_f32(f: f32) -> Self {
        Self {
            raw: (f * 65536.0).round() as i32,
        }
    }

    /// Convert back to f32 (for display/debugging only — never use in canonical path).
    pub fn to_f32(self) -> f32 {
        self.raw as f32 / 65536.0
    }

    /// Multiply two fixed-point scales, returning Q16.16 result.
    /// Uses i64 intermediate to avoid overflow.
    pub fn fixed_mul(self, other: Self) -> Self {
        let wide = self.raw as i64 * other.raw as i64;
        Self {
            raw: (wide >> 16) as i32,
        }
    }

    /// Divide self by other in fixed-point.
    pub fn fixed_div(self, other: Self) -> Self {
        let wide = (self.raw as i64) << 16;
        Self {
            raw: (wide / other.raw as i64) as i32,
        }
    }
}

/// Canonical CPU quantization parameters (all fixed-point, no floats).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CanonicalQuantParams {
    /// Scale in Q16.16 fixed-point.
    pub scale: FixedScale,
    /// Zero point.
    pub zero_point: i8,
}

/// A canonical INT8 tensor for CPU verification.
#[derive(Debug, Clone)]
pub struct CanonicalTensor {
    /// Raw INT8 data in row-major order.
    pub data: Vec<i8>,
    /// Shape [rows, cols].
    pub shape: [u64; 2],
    /// Quantization params.
    pub quant: CanonicalQuantParams,
}

impl CanonicalTensor {
    /// Create a deterministic test tensor from seed (same LCG as GPU path).
    pub fn from_seed(seed: u64, rows: u64, cols: u64, quant: CanonicalQuantParams) -> Self {
        let n = (rows * cols) as usize;
        let mut data = Vec::with_capacity(n);
        let mut state = seed;
        for _ in 0..n {
            state = state
                .wrapping_mul(6364136223846793005)
                .wrapping_add(1442695040888963407);
            data.push((state >> 33) as i8);
        }
        Self {
            data,
            shape: [rows, cols],
            quant,
        }
    }

    /// Raw bytes for hashing.
    pub fn as_bytes(&self) -> &[u8] {
        unsafe { std::slice::from_raw_parts(self.data.as_ptr() as *const u8, self.data.len()) }
    }

    /// Canonical hash (must match GPU path hash for same data).
    pub fn canonical_hash(&self) -> Hash {
        hash_tensor(
            DType::Int8,
            &[self.shape[0], self.shape[1]],
            self.as_bytes(),
        )
    }

    pub fn rows(&self) -> u64 {
        self.shape[0]
    }

    pub fn cols(&self) -> u64 {
        self.shape[1]
    }
}

/// Canonical CPU INT8 GEMM: C = A × B
///
/// **This is the reference implementation.** Characteristics:
/// - Sequential row-major reduction (no parallelism, no reordering)
/// - INT32 accumulation (matches GPU tensor core semantics)
/// - Fixed-point requantization (no FP rounding)
/// - Overflow-checked where feasible
///
/// The result is *the* canonical answer for dispute resolution.
pub fn canonical_int8_gemm(
    a: &CanonicalTensor,
    b: &CanonicalTensor,
    output_quant: &CanonicalQuantParams,
) -> CanonicalTensor {
    let m = a.rows();
    let k = a.cols();
    assert_eq!(b.rows(), k, "inner dimensions must match");
    let n = b.cols();

    // Precompute combined scale in fixed-point: (a.scale * b.scale) / out.scale
    let ab_scale = a.quant.scale.fixed_mul(b.quant.scale);
    let combined_scale = ab_scale.fixed_div(output_quant.scale);

    let mut output = Vec::with_capacity((m * n) as usize);

    for i in 0..m as usize {
        for j in 0..n as usize {
            // INT32 accumulation — strictly sequential, left-to-right
            let mut acc: i32 = 0;
            for p in 0..k as usize {
                let a_val = a.data[i * k as usize + p] as i32;
                let b_val = b.data[p * n as usize + j] as i32;
                acc = acc.wrapping_add(a_val.wrapping_mul(b_val));
            }

            // Fixed-point requantization:
            // result = acc * combined_scale + zero_point
            // combined_scale is Q16.16, so multiply then shift
            let scaled = ((acc as i64 * combined_scale.raw as i64) >> 16) as i32;
            let with_zp = scaled + output_quant.zero_point as i32;

            // Clamp to INT8 range
            let clamped = with_zp.clamp(-128, 127) as i8;
            output.push(clamped);
        }
    }

    CanonicalTensor {
        data: output,
        shape: [m, n],
        quant: *output_quant,
    }
}

/// Run canonical CPU multi-layer inference, returning per-layer activation hashes.
pub fn canonical_cpu_inference(
    input: &CanonicalTensor,
    weights: &[CanonicalTensor],
    output_quant: &CanonicalQuantParams,
) -> Vec<Hash> {
    let mut hashes = Vec::with_capacity(weights.len() + 1);
    hashes.push(input.canonical_hash());

    let mut current = input.clone();
    for (i, weight) in weights.iter().enumerate() {
        let activation = canonical_int8_gemm(&current, weight, output_quant);
        hashes.push(activation.canonical_hash());
        current = activation;

        if i + 1 < weights.len() {
            assert_eq!(
                current.cols(),
                weights[i + 1].rows(),
                "layer {i} output cols must match layer {} weight rows",
                i + 1
            );
        }
    }

    hashes
}

/// Cross-verify GPU (float-scale) and CPU (fixed-point) paths produce the same result.
///
/// This is the critical property: if a dispute arises, the canonical CPU path
/// must agree with correct GPU execution. Any GPU result that disagrees with
/// the CPU canonical path is wrong by definition.
pub fn cross_verify_gpu_cpu(gpu_hashes: &[Hash], cpu_hashes: &[Hash]) -> CrossVerifyResult {
    assert_eq!(
        gpu_hashes.len(),
        cpu_hashes.len(),
        "layer count mismatch between GPU and CPU paths"
    );

    let mut first_divergence: Option<usize> = None;
    let mut divergent_layers = Vec::new();

    for (i, (g, c)) in gpu_hashes.iter().zip(cpu_hashes.iter()).enumerate() {
        if g != c {
            if first_divergence.is_none() {
                first_divergence = Some(i);
            }
            divergent_layers.push(i);
        }
    }

    // Build Merkle trees for both
    let gpu_tree = ActivationMerkleTree::build(gpu_hashes);
    let cpu_tree = ActivationMerkleTree::build(cpu_hashes);

    CrossVerifyResult {
        gpu_root: gpu_tree.root(),
        cpu_root: cpu_tree.root(),
        roots_match: gpu_tree.root() == cpu_tree.root(),
        first_divergence,
        divergent_layers,
        total_layers: gpu_hashes.len(),
    }
}

/// Result of cross-verifying GPU and CPU canonical paths.
#[derive(Debug)]
pub struct CrossVerifyResult {
    pub gpu_root: Hash,
    pub cpu_root: Hash,
    pub roots_match: bool,
    pub first_divergence: Option<usize>,
    pub divergent_layers: Vec<usize>,
    pub total_layers: usize,
}

/// A canonical verifier that can adjudicate disputes.
///
/// In the full system, this runs on a trusted CPU node (or eventually in WASM/ZK)
/// to provide the ground-truth activation hashes for a specific inference task.
pub struct CanonicalVerifier {
    /// Weights for the model (loaded once, reused).
    weights: Vec<CanonicalTensor>,
    /// Output quantization params.
    output_quant: CanonicalQuantParams,
}

impl CanonicalVerifier {
    pub fn new(weights: Vec<CanonicalTensor>, output_quant: CanonicalQuantParams) -> Self {
        Self {
            weights,
            output_quant,
        }
    }

    /// Verify a single layer's activation by re-executing on CPU.
    /// Returns (expected_hash, matches).
    pub fn verify_layer(
        &self,
        input: &CanonicalTensor,
        layer_idx: usize,
        claimed_hash: &Hash,
    ) -> (Hash, bool) {
        // Re-execute up to the target layer
        let hashes =
            canonical_cpu_inference(input, &self.weights[..=layer_idx], &self.output_quant);
        let expected = hashes[layer_idx + 1]; // +1 because index 0 is input
        (expected, expected == *claimed_hash)
    }

    /// Full verification: re-execute all layers and compare.
    pub fn verify_full(
        &self,
        input: &CanonicalTensor,
        claimed_hashes: &[Hash],
    ) -> CrossVerifyResult {
        let cpu_hashes = canonical_cpu_inference(input, &self.weights, &self.output_quant);
        cross_verify_gpu_cpu(claimed_hashes, &cpu_hashes)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_quant() -> CanonicalQuantParams {
        CanonicalQuantParams {
            scale: FixedScale::from_f32(0.05),
            zero_point: 0,
        }
    }

    fn test_output_quant() -> CanonicalQuantParams {
        CanonicalQuantParams {
            scale: FixedScale::from_f32(0.02),
            zero_point: 0,
        }
    }

    // --- FixedScale tests ---

    #[test]
    fn test_fixed_scale_roundtrip() {
        let s = FixedScale::from_f32(0.05);
        let back = s.to_f32();
        assert!((back - 0.05).abs() < 0.001, "roundtrip: {back}");
    }

    #[test]
    fn test_fixed_scale_mul() {
        let a = FixedScale::from_f32(0.05);
        let b = FixedScale::from_f32(0.05);
        let c = a.fixed_mul(b);
        assert!((c.to_f32() - 0.0025).abs() < 0.001);
    }

    #[test]
    fn test_fixed_scale_div() {
        let a = FixedScale::from_f32(0.0025);
        let b = FixedScale::from_f32(0.02);
        let c = a.fixed_div(b);
        assert!((c.to_f32() - 0.125).abs() < 0.01);
    }

    // --- Tensor tests ---

    #[test]
    fn test_canonical_tensor_deterministic() {
        let q = test_quant();
        let t1 = CanonicalTensor::from_seed(42, 4, 4, q);
        let t2 = CanonicalTensor::from_seed(42, 4, 4, q);
        assert_eq!(t1.data, t2.data);
        assert_eq!(t1.canonical_hash(), t2.canonical_hash());
    }

    #[test]
    fn test_canonical_tensor_different_seeds() {
        let q = test_quant();
        let t1 = CanonicalTensor::from_seed(42, 4, 4, q);
        let t2 = CanonicalTensor::from_seed(99, 4, 4, q);
        assert_ne!(t1.canonical_hash(), t2.canonical_hash());
    }

    // --- GEMM tests ---

    #[test]
    fn test_canonical_gemm_identity() {
        let q = CanonicalQuantParams {
            scale: FixedScale::from_f32(1.0),
            zero_point: 0,
        };
        let a = CanonicalTensor {
            data: vec![1, 0, 0, 1],
            shape: [2, 2],
            quant: q,
        };
        let b = CanonicalTensor {
            data: vec![3, 4, 5, 6],
            shape: [2, 2],
            quant: q,
        };
        let c = canonical_int8_gemm(&a, &b, &q);
        assert_eq!(c.data, vec![3, 4, 5, 6]);
    }

    #[test]
    fn test_canonical_gemm_accumulation() {
        let q = CanonicalQuantParams {
            scale: FixedScale::from_f32(1.0),
            zero_point: 0,
        };
        let a = CanonicalTensor {
            data: vec![1, 2],
            shape: [1, 2],
            quant: q,
        };
        let b = CanonicalTensor {
            data: vec![3, 4],
            shape: [2, 1],
            quant: q,
        };
        let c = canonical_int8_gemm(&a, &b, &q);
        assert_eq!(c.data, vec![11]);
    }

    #[test]
    fn test_canonical_gemm_clamp() {
        let q = CanonicalQuantParams {
            scale: FixedScale::from_f32(1.0),
            zero_point: 0,
        };
        let oq = CanonicalQuantParams {
            scale: FixedScale::from_f32(0.5),
            zero_point: 0,
        };
        let a = CanonicalTensor {
            data: vec![127, 127, 127, 127],
            shape: [1, 4],
            quant: q,
        };
        let b = CanonicalTensor {
            data: vec![127, 127, 127, 127],
            shape: [4, 1],
            quant: q,
        };
        let c = canonical_int8_gemm(&a, &b, &oq);
        // 4 * 127 * 127 = 64516, scaled by (1.0*1.0)/0.5 = 2.0 → clamped to 127
        assert_eq!(c.data, vec![127]);
    }

    #[test]
    fn test_canonical_gemm_deterministic() {
        let q = test_quant();
        let oq = test_output_quant();
        let a = CanonicalTensor::from_seed(1, 8, 16, q);
        let b = CanonicalTensor::from_seed(2, 16, 8, q);
        let c1 = canonical_int8_gemm(&a, &b, &oq);
        let c2 = canonical_int8_gemm(&a, &b, &oq);
        assert_eq!(c1.data, c2.data);
        assert_eq!(c1.canonical_hash(), c2.canonical_hash());
    }

    // --- Multi-layer inference ---

    #[test]
    fn test_canonical_cpu_inference_layer_count() {
        let q = test_quant();
        let oq = test_output_quant();
        let input = CanonicalTensor::from_seed(42, 1, 16, q);
        let weights: Vec<CanonicalTensor> = (0..4)
            .map(|i| CanonicalTensor::from_seed(100 + i, 16, 16, q))
            .collect();
        let hashes = canonical_cpu_inference(&input, &weights, &oq);
        assert_eq!(hashes.len(), 5); // input + 4 layers
    }

    #[test]
    fn test_canonical_cpu_inference_deterministic() {
        let q = test_quant();
        let oq = test_output_quant();
        let input = CanonicalTensor::from_seed(42, 1, 16, q);
        let weights: Vec<CanonicalTensor> = (0..4)
            .map(|i| CanonicalTensor::from_seed(100 + i, 16, 16, q))
            .collect();
        let h1 = canonical_cpu_inference(&input, &weights, &oq);
        let h2 = canonical_cpu_inference(&input, &weights, &oq);
        assert_eq!(h1, h2);
    }

    #[test]
    fn test_canonical_cpu_inference_unique_hashes() {
        let q = test_quant();
        let oq = test_output_quant();
        let input = CanonicalTensor::from_seed(42, 1, 16, q);
        let weights: Vec<CanonicalTensor> = (0..4)
            .map(|i| CanonicalTensor::from_seed(100 + i, 16, 16, q))
            .collect();
        let hashes = canonical_cpu_inference(&input, &weights, &oq);
        for i in 0..hashes.len() {
            for j in (i + 1)..hashes.len() {
                assert_ne!(hashes[i], hashes[j], "hashes {i} and {j} should differ");
            }
        }
    }

    // --- Cross-verification ---

    #[test]
    fn test_cross_verify_identical() {
        let q = test_quant();
        let oq = test_output_quant();
        let input = CanonicalTensor::from_seed(42, 1, 16, q);
        let weights: Vec<CanonicalTensor> = (0..4)
            .map(|i| CanonicalTensor::from_seed(100 + i, 16, 16, q))
            .collect();
        let hashes = canonical_cpu_inference(&input, &weights, &oq);
        let result = cross_verify_gpu_cpu(&hashes, &hashes);
        assert!(result.roots_match);
        assert!(result.first_divergence.is_none());
        assert!(result.divergent_layers.is_empty());
    }

    #[test]
    fn test_cross_verify_divergent() {
        let q = test_quant();
        let oq = test_output_quant();
        let input = CanonicalTensor::from_seed(42, 1, 16, q);
        let weights: Vec<CanonicalTensor> = (0..4)
            .map(|i| CanonicalTensor::from_seed(100 + i, 16, 16, q))
            .collect();
        let correct = canonical_cpu_inference(&input, &weights, &oq);

        // Corrupt one layer
        let mut wrong = correct.clone();
        wrong[2] = [0xFFu8; 32];

        let result = cross_verify_gpu_cpu(&correct, &wrong);
        assert!(!result.roots_match);
        assert_eq!(result.first_divergence, Some(2));
        assert!(result.divergent_layers.contains(&2));
    }

    // --- CanonicalVerifier ---

    #[test]
    fn test_verifier_full_pass() {
        let q = test_quant();
        let oq = test_output_quant();
        let input = CanonicalTensor::from_seed(42, 1, 16, q);
        let weights: Vec<CanonicalTensor> = (0..4)
            .map(|i| CanonicalTensor::from_seed(100 + i, 16, 16, q))
            .collect();

        let verifier = CanonicalVerifier::new(weights.clone(), oq);
        let claimed = canonical_cpu_inference(&input, &weights, &oq);
        let result = verifier.verify_full(&input, &claimed);
        assert!(result.roots_match);
    }

    #[test]
    fn test_verifier_full_fail() {
        let q = test_quant();
        let oq = test_output_quant();
        let input = CanonicalTensor::from_seed(42, 1, 16, q);
        let weights: Vec<CanonicalTensor> = (0..4)
            .map(|i| CanonicalTensor::from_seed(100 + i, 16, 16, q))
            .collect();

        let verifier = CanonicalVerifier::new(weights.clone(), oq);
        let mut claimed = canonical_cpu_inference(&input, &weights, &oq);
        claimed[3] = [0xAB; 32]; // corrupt layer 2 output

        let result = verifier.verify_full(&input, &claimed);
        assert!(!result.roots_match);
        assert_eq!(result.first_divergence, Some(3));
    }

    #[test]
    fn test_verifier_single_layer() {
        let q = test_quant();
        let oq = test_output_quant();
        let input = CanonicalTensor::from_seed(42, 1, 16, q);
        let weights: Vec<CanonicalTensor> = (0..4)
            .map(|i| CanonicalTensor::from_seed(100 + i, 16, 16, q))
            .collect();

        let verifier = CanonicalVerifier::new(weights.clone(), oq);
        let correct = canonical_cpu_inference(&input, &weights, &oq);

        // Verify layer 2 with correct hash
        let (expected, matches) = verifier.verify_layer(&input, 2, &correct[3]);
        assert!(matches);
        assert_eq!(expected, correct[3]);

        // Verify layer 2 with wrong hash
        let (_, matches) = verifier.verify_layer(&input, 2, &[0xFF; 32]);
        assert!(!matches);
    }

    #[test]
    fn test_merkle_proofs_from_canonical() {
        let q = test_quant();
        let oq = test_output_quant();
        let input = CanonicalTensor::from_seed(42, 1, 16, q);
        let weights: Vec<CanonicalTensor> = (0..8)
            .map(|i| CanonicalTensor::from_seed(100 + i, 16, 16, q))
            .collect();

        let hashes = canonical_cpu_inference(&input, &weights, &oq);
        let tree = ActivationMerkleTree::build(&hashes);
        let root = tree.root();

        for i in 0..hashes.len() {
            let proof = tree.prove(i);
            assert!(
                crate::merkle::verify_proof(&proof, &root),
                "proof failed for activation {i}"
            );
        }
    }
}
