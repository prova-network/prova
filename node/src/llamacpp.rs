//! Real llama.cpp integration for activation capture.
//!
//! This module provides `LlamaCppRunner` which shells out to a llama.cpp
//! binary that supports activation dumping (via `--dump-activations <dir>`).
//!
//! Activation capture flow:
//! 1. Create temp directory for activation dumps
//! 2. Invoke llama-cli with deterministic settings + `--dump-activations`
//! 3. Read activation tensor files from dump dir (one per layer)
//! 4. Hash each tensor using canonical `hash_tensor()` format
//! 5. Build Activation Merkle Tree and return `InferenceResult`
//!
//! The dump format is: `<dir>/layer_<N>.bin` — raw tensor bytes in row-major order,
//! accompanied by `<dir>/layer_<N>.meta` — JSON metadata with dtype, shape, layout.

use crate::merkle::{hash_tensor, ActivationMerkleTree, DType, Hash};
use crate::runner::{InferenceConfig, InferenceResult};
use serde::Deserialize;
use sha2::{Digest, Sha256};
use std::fs;
use std::io;
use std::path::Path;
use std::process::Command;

/// Errors from llama.cpp invocation.
#[derive(Debug)]
pub enum LlamaCppError {
    /// Engine binary not found or not executable.
    EngineNotFound(String),
    /// Engine exited with non-zero status.
    EngineFailed { status: i32, stderr: String },
    /// Failed to read activation dump files.
    DumpReadError(io::Error),
    /// Activation dump metadata parse error.
    MetadataParseError(String),
    /// No activation files found in dump directory.
    NoActivationsFound,
    /// Layer file sequence has gaps.
    LayerGap { expected: u32, found: u32 },
}

impl std::fmt::Display for LlamaCppError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::EngineNotFound(p) => write!(f, "engine not found: {p}"),
            Self::EngineFailed { status, stderr } => {
                write!(f, "engine exited {status}: {stderr}")
            }
            Self::DumpReadError(e) => write!(f, "dump read error: {e}"),
            Self::MetadataParseError(m) => write!(f, "metadata parse error: {m}"),
            Self::NoActivationsFound => write!(f, "no activation files in dump dir"),
            Self::LayerGap { expected, found } => {
                write!(f, "layer gap: expected {expected}, found {found}")
            }
        }
    }
}

impl std::error::Error for LlamaCppError {}

/// Metadata for a dumped activation tensor.
#[derive(Debug, Deserialize)]
struct ActivationMeta {
    dtype: String,
    shape: Vec<u64>,
    #[allow(dead_code)]
    layout: String,
}

impl ActivationMeta {
    fn to_dtype(&self) -> Result<DType, LlamaCppError> {
        match self.dtype.as_str() {
            "INT8" | "int8" | "i8" => Ok(DType::Int8),
            "INT32" | "int32" | "i32" => Ok(DType::Int32),
            "FP16" | "fp16" | "f16" => Ok(DType::Fp16),
            "FP32" | "fp32" | "f32" => Ok(DType::Fp32),
            other => Err(LlamaCppError::MetadataParseError(format!(
                "unknown dtype: {other}"
            ))),
        }
    }
}

/// Runner that invokes a real llama.cpp binary with activation dumping.
pub struct LlamaCppRunner;

impl LlamaCppRunner {
    /// Run inference and capture activations.
    ///
    /// Requires a llama.cpp build with `--dump-activations` support.
    /// The engine must write `layer_0.bin`/`layer_0.meta` through `layer_N.bin`/`layer_N.meta`
    /// into the specified dump directory.
    pub fn run(config: &InferenceConfig) -> Result<InferenceResult, LlamaCppError> {
        let dump_dir = tempfile::tempdir().map_err(LlamaCppError::DumpReadError)?;
        let dump_path = dump_dir.path();

        // Invoke llama-cli with deterministic flags
        let output = Command::new(&config.engine_path)
            .args([
                "-m",
                &config.model_path,
                "-p",
                &config.prompt,
                "--seed",
                &config.seed.to_string(),
                "--temp",
                &config.temperature.to_string(),
                "-n",
                &config.n_predict.to_string(),
                "--dump-activations",
                dump_path.to_str().unwrap(),
                // Determinism flags
                "--threads",
                "1",
                "--no-mmap",
            ])
            .output()
            .map_err(|_| LlamaCppError::EngineNotFound(config.engine_path.clone()))?;

        if !output.status.success() {
            return Err(LlamaCppError::EngineFailed {
                status: output.status.code().unwrap_or(-1),
                stderr: String::from_utf8_lossy(&output.stderr).to_string(),
            });
        }

        let generated_text = String::from_utf8_lossy(&output.stdout).to_string();

        // Read activation dumps
        let activation_hashes = Self::read_activations(dump_path)?;
        if activation_hashes.is_empty() {
            return Err(LlamaCppError::NoActivationsFound);
        }

        // Compute input hash
        let input_hash: Hash = {
            let mut h = Sha256::new();
            h.update(b"prova-input:");
            h.update(config.prompt.as_bytes());
            h.finalize().into()
        };

        let merkle_tree = ActivationMerkleTree::build(&activation_hashes);

        Ok(InferenceResult {
            output: generated_text,
            activation_hashes,
            merkle_tree,
            input_hash,
        })
    }

    /// Read activation tensor files from a dump directory.
    ///
    /// Expects pairs of `layer_N.bin` and `layer_N.meta` files numbered
    /// sequentially from 0.
    pub fn read_activations(dump_dir: &Path) -> Result<Vec<Hash>, LlamaCppError> {
        let mut layer_num = 0u32;
        let mut hashes = Vec::new();

        loop {
            let bin_path = dump_dir.join(format!("layer_{layer_num}.bin"));
            let meta_path = dump_dir.join(format!("layer_{layer_num}.meta"));

            if !bin_path.exists() {
                // Check if there's a gap (higher-numbered layers exist)
                let next_bin = dump_dir.join(format!("layer_{}.bin", layer_num + 1));
                if next_bin.exists() {
                    return Err(LlamaCppError::LayerGap {
                        expected: layer_num,
                        found: layer_num + 1,
                    });
                }
                break;
            }

            // Read metadata
            let meta_json = fs::read_to_string(&meta_path).map_err(LlamaCppError::DumpReadError)?;
            let meta: ActivationMeta = serde_json::from_str(&meta_json)
                .map_err(|e| LlamaCppError::MetadataParseError(format!("{meta_path:?}: {e}")))?;

            // Read raw tensor bytes
            let tensor_bytes = fs::read(&bin_path).map_err(LlamaCppError::DumpReadError)?;

            // Hash using canonical format
            let dtype = meta.to_dtype()?;
            let hash = hash_tensor(dtype, &meta.shape, &tensor_bytes);

            hashes.push(hash);
            layer_num += 1;
        }

        Ok(hashes)
    }

    /// Verify that a llama.cpp binary supports activation dumping.
    pub fn check_dump_support(engine_path: &str) -> bool {
        Command::new(engine_path)
            .args(["--help"])
            .output()
            .map(|o| {
                let help = String::from_utf8_lossy(&o.stdout);
                help.contains("--dump-activations")
            })
            .unwrap_or(false)
    }
}

/// Write synthetic activation dump files for testing.
/// Creates `layer_N.bin` + `layer_N.meta` pairs.
#[cfg(test)]
fn write_test_activations(dir: &Path, count: u32, dtype: &str, shape: &[u64]) {
    for i in 0..count {
        // Deterministic fake tensor data
        let elem_size: usize = match dtype {
            "INT8" | "int8" => 1,
            "FP16" | "fp16" => 2,
            "INT32" | "int32" | "FP32" | "fp32" => 4,
            _ => 4,
        };
        let total_elems: usize = shape.iter().map(|&d| d as usize).product();
        let mut tensor_bytes = vec![0u8; total_elems * elem_size];
        // Fill with deterministic pattern: layer index + position
        for (j, byte) in tensor_bytes.iter_mut().enumerate() {
            *byte = ((i as usize * 31 + j * 7) % 256) as u8;
        }

        fs::write(dir.join(format!("layer_{i}.bin")), &tensor_bytes).unwrap();

        let meta = serde_json::json!({
            "dtype": dtype,
            "shape": shape,
            "layout": "RowMajor"
        });
        fs::write(dir.join(format!("layer_{i}.meta")), meta.to_string()).unwrap();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::merkle::verify_proof;
    use std::fs;

    #[test]
    fn test_read_activations_basic() {
        let dir = tempfile::tempdir().unwrap();
        write_test_activations(dir.path(), 4, "INT8", &[1, 128]);

        let hashes = LlamaCppRunner::read_activations(dir.path()).unwrap();
        assert_eq!(hashes.len(), 4);

        // All hashes should be different (different layer patterns)
        for i in 0..4 {
            for j in (i + 1)..4 {
                assert_ne!(hashes[i], hashes[j], "layers {i} and {j} should differ");
            }
        }
    }

    #[test]
    fn test_read_activations_deterministic() {
        let dir1 = tempfile::tempdir().unwrap();
        let dir2 = tempfile::tempdir().unwrap();

        write_test_activations(dir1.path(), 8, "FP32", &[4, 32]);
        write_test_activations(dir2.path(), 8, "FP32", &[4, 32]);

        let h1 = LlamaCppRunner::read_activations(dir1.path()).unwrap();
        let h2 = LlamaCppRunner::read_activations(dir2.path()).unwrap();

        assert_eq!(h1, h2, "same inputs should produce same hashes");
    }

    #[test]
    fn test_read_activations_empty_dir() {
        let dir = tempfile::tempdir().unwrap();
        let hashes = LlamaCppRunner::read_activations(dir.path()).unwrap();
        assert!(hashes.is_empty());
    }

    #[test]
    fn test_read_activations_gap_detection() {
        let dir = tempfile::tempdir().unwrap();
        write_test_activations(dir.path(), 3, "INT8", &[1, 64]);

        // Remove layer_1 to create a gap
        fs::remove_file(dir.path().join("layer_1.bin")).unwrap();
        fs::remove_file(dir.path().join("layer_1.meta")).unwrap();

        let result = LlamaCppRunner::read_activations(dir.path());
        assert!(matches!(result, Err(LlamaCppError::LayerGap { .. })));
    }

    #[test]
    fn test_read_activations_merkle_tree_integration() {
        let dir = tempfile::tempdir().unwrap();
        write_test_activations(dir.path(), 32, "INT8", &[1, 4096]);

        let hashes = LlamaCppRunner::read_activations(dir.path()).unwrap();
        assert_eq!(hashes.len(), 32);

        // Build Merkle tree and verify all proofs
        let tree = ActivationMerkleTree::build(&hashes);
        let root = tree.root();

        for i in 0..32 {
            let proof = tree.prove(i);
            assert!(verify_proof(&proof, &root), "proof failed for layer {i}");
        }
    }

    #[test]
    fn test_read_activations_fp16_dtype() {
        let dir = tempfile::tempdir().unwrap();
        write_test_activations(dir.path(), 4, "fp16", &[2, 64]);

        let hashes = LlamaCppRunner::read_activations(dir.path()).unwrap();
        assert_eq!(hashes.len(), 4);
    }

    #[test]
    fn test_dtype_parsing() {
        let meta = ActivationMeta {
            dtype: "INT8".to_string(),
            shape: vec![1, 128],
            layout: "RowMajor".to_string(),
        };
        assert!(matches!(meta.to_dtype(), Ok(DType::Int8)));

        let meta_bad = ActivationMeta {
            dtype: "BFLOAT16".to_string(),
            shape: vec![1],
            layout: "RowMajor".to_string(),
        };
        assert!(meta_bad.to_dtype().is_err());
    }

    #[test]
    fn test_activation_hash_changes_with_data() {
        let dir1 = tempfile::tempdir().unwrap();
        let dir2 = tempfile::tempdir().unwrap();

        // Same layer count, different shapes → different data sizes → different hashes
        write_test_activations(dir1.path(), 1, "INT8", &[1, 128]);
        write_test_activations(dir2.path(), 1, "INT8", &[1, 256]);

        let h1 = LlamaCppRunner::read_activations(dir1.path()).unwrap();
        let h2 = LlamaCppRunner::read_activations(dir2.path()).unwrap();

        assert_ne!(
            h1[0], h2[0],
            "different tensor shapes should produce different hashes"
        );
    }
}
