//! Activation Merkle Tree implementation for QBP protocol.
//!
//! Constructs a binary Merkle tree over layer activation hashes,
//! with domain separation (0x00 for leaves, 0x01 for internal nodes).

use sha2::{Digest, Sha256};

/// A 32-byte SHA-256 hash.
pub type Hash = [u8; 32];

/// Domain separation prefixes
const LEAF_PREFIX: u8 = 0x00;
const NODE_PREFIX: u8 = 0x01;

/// Side of a sibling in a Merkle proof.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Side {
    Left,
    Right,
}

/// One sibling entry in a Merkle proof.
#[derive(Debug, Clone)]
pub struct ProofSibling {
    pub hash: Hash,
    pub side: Side,
}

/// A Merkle proof for a specific leaf.
#[derive(Debug, Clone)]
pub struct MerkleProof {
    pub leaf_index: u32,
    pub leaf_hash: Hash,
    pub siblings: Vec<ProofSibling>,
}

/// Activation Merkle Tree.
///
/// Stores all nodes for proof generation.
/// Leaves are activation hashes (already SHA-256 of serialized tensors).
#[derive(Debug)]
pub struct ActivationMerkleTree {
    /// Number of original leaves (before padding)
    pub leaf_count: usize,
    /// All tree nodes in level-order. Level 0 = leaves (padded to power of 2).
    /// Total nodes = 2 * padded_leaf_count - 1.
    nodes: Vec<Hash>,
    /// Number of leaves after padding to power of 2
    padded_count: usize,
}

impl ActivationMerkleTree {
    /// Build a Merkle tree from activation hashes.
    ///
    /// `activation_hashes` should contain L+1 entries:
    /// h[0] = hash of input, h[1..L] = hash of each layer output.
    pub fn build(activation_hashes: &[Hash]) -> Self {
        let leaf_count = activation_hashes.len();
        assert!(leaf_count > 0, "need at least one leaf");

        // Pad to next power of 2
        let padded_count = leaf_count.next_power_of_two();

        // Build leaf level with domain separation
        let mut leaves: Vec<Hash> = Vec::with_capacity(padded_count);
        for h in activation_hashes {
            leaves.push(hash_leaf(h));
        }
        // Pad with hash of zero byte
        let padding_hash = hash_leaf(&[0u8; 32]);
        while leaves.len() < padded_count {
            leaves.push(padding_hash);
        }

        // Total nodes in a complete binary tree
        let total_nodes = 2 * padded_count - 1;
        let internal_count = padded_count - 1;

        // Allocate: internal nodes first, then leaves
        // Layout: [internal_0, internal_1, ..., internal_{n-1}, leaf_0, leaf_1, ..., leaf_{m-1}]
        let mut nodes = vec![[0u8; 32]; total_nodes];

        // Place leaves
        for (i, leaf) in leaves.iter().enumerate() {
            nodes[internal_count + i] = *leaf;
        }

        // Build internal nodes bottom-up
        for i in (0..internal_count).rev() {
            let left = nodes[2 * i + 1];
            let right = nodes[2 * i + 2];
            nodes[i] = hash_node(&left, &right);
        }

        Self {
            leaf_count,
            nodes,
            padded_count,
        }
    }

    /// Get the Merkle root.
    pub fn root(&self) -> Hash {
        self.nodes[0]
    }

    /// Generate a proof for the leaf at `index` (0-based, in original leaves).
    pub fn prove(&self, index: usize) -> MerkleProof {
        assert!(index < self.leaf_count, "leaf index out of range");

        let internal_count = self.padded_count - 1;
        let mut node_idx = internal_count + index;
        let mut siblings = Vec::new();

        while node_idx > 0 {
            let parent_idx = (node_idx - 1) / 2;
            let left_child = 2 * parent_idx + 1;
            let right_child = 2 * parent_idx + 2;

            if node_idx == left_child {
                siblings.push(ProofSibling {
                    hash: self.nodes[right_child],
                    side: Side::Right,
                });
            } else {
                siblings.push(ProofSibling {
                    hash: self.nodes[left_child],
                    side: Side::Left,
                });
            }

            node_idx = parent_idx;
        }

        MerkleProof {
            leaf_index: index as u32,
            leaf_hash: self.activation_hash(index),
            siblings,
        }
    }

    /// Get the original activation hash (pre-leaf-hashing) at index.
    /// Note: we only store the leaf-hashed version. This returns the leaf node.
    fn activation_hash(&self, index: usize) -> Hash {
        let internal_count = self.padded_count - 1;
        self.nodes[internal_count + index]
    }
}

/// Verify a Merkle proof against a root.
pub fn verify_proof(proof: &MerkleProof, root: &Hash) -> bool {
    let mut current = proof.leaf_hash;

    for sibling in &proof.siblings {
        current = match sibling.side {
            Side::Left => hash_node(&sibling.hash, &current),
            Side::Right => hash_node(&current, &sibling.hash),
        };
    }

    current == *root
}

/// Hash a leaf with domain separation.
fn hash_leaf(data: &[u8; 32]) -> Hash {
    let mut hasher = Sha256::new();
    hasher.update([LEAF_PREFIX]);
    hasher.update(data);
    hasher.finalize().into()
}

/// Hash an internal node with domain separation.
fn hash_node(left: &Hash, right: &Hash) -> Hash {
    let mut hasher = Sha256::new();
    hasher.update([NODE_PREFIX]);
    hasher.update(left);
    hasher.update(right);
    hasher.finalize().into()
}

// ============================================================
// Tensor Serialization (canonical format)
// ============================================================

/// Magic bytes for Prova tensor format
const TENSOR_MAGIC: [u8; 4] = [0x50, 0x52, 0x4F, 0x56]; // "PROV"
const TENSOR_VERSION: u8 = 0x01;

/// Data type identifier
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum DType {
    Int8 = 0x01,
    Int32 = 0x02,
    Fp16 = 0x03,
    Fp32 = 0x04,
}

impl DType {
    pub fn size_bytes(&self) -> usize {
        match self {
            DType::Int8 => 1,
            DType::Int32 => 4,
            DType::Fp16 => 2,
            DType::Fp32 => 4,
        }
    }
}

/// Serialize a tensor in canonical Prova format and return its SHA-256 hash.
///
/// Format:
///   magic(4) | version(1) | dtype(1) | ndim(1) | reserved(1) | shape(ndim×8) | data(row-major LE)
pub fn hash_tensor(dtype: DType, shape: &[u64], data: &[u8]) -> Hash {
    let expected_elements: u64 = shape.iter().product();
    let expected_bytes = expected_elements as usize * dtype.size_bytes();
    assert_eq!(
        data.len(),
        expected_bytes,
        "data length mismatch: expected {expected_bytes}, got {}",
        data.len()
    );
    assert!(shape.len() <= 8, "max 8 dimensions");

    let mut hasher = Sha256::new();

    // Header
    hasher.update(TENSOR_MAGIC);
    hasher.update([TENSOR_VERSION]);
    hasher.update([dtype as u8]);
    hasher.update([shape.len() as u8]);
    hasher.update([0x00]); // reserved

    // Shape
    for &dim in shape {
        hasher.update(dim.to_le_bytes());
    }

    // Data (must already be in row-major, little-endian order)
    hasher.update(data);

    hasher.finalize().into()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_single_leaf_tree() {
        let h = [0xAA; 32];
        let tree = ActivationMerkleTree::build(&[h]);
        assert_eq!(tree.leaf_count, 1);
        // Root should be hash_leaf of the single leaf (padded to 2, then combined)
        let root = tree.root();
        assert_ne!(root, [0; 32]);
    }

    #[test]
    fn test_four_leaves() {
        let leaves: Vec<Hash> = (0..4u8)
            .map(|i| {
                let mut h = [0u8; 32];
                h[0] = i;
                h
            })
            .collect();

        let tree = ActivationMerkleTree::build(&leaves);
        assert_eq!(tree.leaf_count, 4);

        // Verify proofs for each leaf
        let root = tree.root();
        for i in 0..4 {
            let proof = tree.prove(i);
            assert!(verify_proof(&proof, &root), "proof failed for leaf {i}");
        }
    }

    #[test]
    fn test_non_power_of_two_leaves() {
        // 5 leaves → padded to 8
        let leaves: Vec<Hash> = (0..5u8)
            .map(|i| {
                let mut h = [0u8; 32];
                h[0] = i;
                h
            })
            .collect();

        let tree = ActivationMerkleTree::build(&leaves);
        assert_eq!(tree.leaf_count, 5);

        let root = tree.root();
        for i in 0..5 {
            let proof = tree.prove(i);
            assert!(verify_proof(&proof, &root), "proof failed for leaf {i}");
        }
    }

    #[test]
    fn test_proof_rejects_wrong_root() {
        let leaves: Vec<Hash> = (0..4u8)
            .map(|i| {
                let mut h = [0u8; 32];
                h[0] = i;
                h
            })
            .collect();

        let tree = ActivationMerkleTree::build(&leaves);
        let proof = tree.prove(0);
        let wrong_root = [0xFF; 32];
        assert!(!verify_proof(&proof, &wrong_root));
    }

    #[test]
    fn test_different_trees_different_roots() {
        let leaves_a: Vec<Hash> = (0..4u8)
            .map(|i| {
                let mut h = [0u8; 32];
                h[0] = i;
                h
            })
            .collect();

        let leaves_b: Vec<Hash> = (10..14u8)
            .map(|i| {
                let mut h = [0u8; 32];
                h[0] = i;
                h
            })
            .collect();

        let tree_a = ActivationMerkleTree::build(&leaves_a);
        let tree_b = ActivationMerkleTree::build(&leaves_b);

        assert_ne!(tree_a.root(), tree_b.root());
    }

    #[test]
    fn test_tensor_hash_deterministic() {
        let shape = [2u64, 3];
        // 6 FP32 values: 1.0, 2.0, 3.0, 4.0, 5.0, 6.0
        let data: Vec<u8> = [1.0f32, 2.0, 3.0, 4.0, 5.0, 6.0]
            .iter()
            .flat_map(|f| f.to_le_bytes())
            .collect();

        let h1 = hash_tensor(DType::Fp32, &shape, &data);
        let h2 = hash_tensor(DType::Fp32, &shape, &data);
        assert_eq!(h1, h2, "tensor hashing must be deterministic");
    }

    #[test]
    fn test_tensor_hash_different_data() {
        let shape = [2u64, 2];
        let data_a: Vec<u8> = [1.0f32, 2.0, 3.0, 4.0]
            .iter()
            .flat_map(|f| f.to_le_bytes())
            .collect();
        let data_b: Vec<u8> = [1.0f32, 2.0, 3.0, 5.0]
            .iter()
            .flat_map(|f| f.to_le_bytes())
            .collect();

        let h_a = hash_tensor(DType::Fp32, &shape, &data_a);
        let h_b = hash_tensor(DType::Fp32, &shape, &data_b);
        assert_ne!(h_a, h_b, "different data must produce different hashes");
    }

    #[test]
    fn test_80_layer_model() {
        // Simulate an 80-layer model (81 leaves: input + 80 layers)
        let leaves: Vec<Hash> = (0..81u8)
            .map(|i| {
                let mut h = [0u8; 32];
                h[0] = i;
                h[31] = 255 - i;
                h
            })
            .collect();

        let tree = ActivationMerkleTree::build(&leaves);
        assert_eq!(tree.leaf_count, 81);

        let root = tree.root();

        // Verify all 81 proofs
        for i in 0..81 {
            let proof = tree.prove(i);
            assert!(verify_proof(&proof, &root), "proof failed for leaf {i}");
            // 81 padded to 128 → log2(128) = 7 siblings
            assert_eq!(
                proof.siblings.len(),
                7,
                "expected 7 siblings for 128-leaf tree"
            );
        }
    }
}
