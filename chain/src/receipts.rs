//! Receipt storage — persist transaction/block receipts with proof-of-inclusion.
//!
//! Receipts tie transaction execution outcomes to the block they appeared in:
//! - Each transaction produces a `TxReceipt` (status, gas used, events emitted)
//! - Block receipts bundle all tx receipts with a Merkle root
//! - Merkle inclusion proofs let light clients verify a specific receipt/event
//!   belonged to a given block without downloading the full receipt set.

use std::collections::BTreeMap;

use sha2::{Digest, Sha256};

use crate::events::Event;
use crate::types::{Address, Epoch, Hash};

// ── Receipt types ──────────────────────────────────────────────────

/// Outcome of a single transaction execution.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TxStatus {
    Success,
    Reverted,
    OutOfGas,
}

/// Receipt for a single transaction.
#[derive(Debug, Clone)]
pub struct TxReceipt {
    /// Transaction hash (unique identifier).
    pub tx_hash: Hash,
    /// Sender address.
    pub from: Address,
    /// Block in which the transaction was included.
    pub block_number: Epoch,
    /// Index within the block.
    pub tx_index: u32,
    /// Execution outcome.
    pub status: TxStatus,
    /// Gas consumed by this transaction.
    pub gas_used: u64,
    /// Cumulative gas used up to and including this tx in the block.
    pub cumulative_gas: u64,
    /// Events emitted during execution.
    pub events: Vec<Event>,
    /// Merkle root of this receipt's events (for per-tx event proofs).
    pub events_root: Hash,
}

impl TxReceipt {
    /// Canonical hash of this receipt (used as Merkle leaf).
    pub fn receipt_hash(&self) -> Hash {
        let mut hasher = Sha256::new();
        hasher.update(&self.tx_hash);
        hasher.update(&self.from.0);
        hasher.update(&self.block_number.to_be_bytes());
        hasher.update(&self.tx_index.to_be_bytes());
        hasher.update(&[match self.status {
            TxStatus::Success => 1,
            TxStatus::Reverted => 2,
            TxStatus::OutOfGas => 3,
        }]);
        hasher.update(&self.gas_used.to_be_bytes());
        hasher.update(&self.cumulative_gas.to_be_bytes());
        hasher.update(&self.events_root);
        let result = hasher.finalize();
        let mut h = [0u8; 32];
        h.copy_from_slice(&result);
        h
    }
}

// ── Merkle proof ───────────────────────────────────────────────────

/// A Merkle inclusion proof: sibling hashes from leaf to root.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MerkleProof {
    /// Leaf index in the original list.
    pub leaf_index: usize,
    /// Total number of leaves.
    pub leaf_count: usize,
    /// Sibling hashes, bottom-up. Each entry: (sibling_hash, is_left).
    /// `is_left` = true means the sibling is on the left.
    pub siblings: Vec<(Hash, bool)>,
}

impl MerkleProof {
    /// Verify that `leaf_hash` at `self.leaf_index` produces `expected_root`.
    pub fn verify(&self, leaf_hash: &Hash, expected_root: &Hash) -> bool {
        let mut current = *leaf_hash;
        for (sibling, is_left) in &self.siblings {
            let mut hasher = Sha256::new();
            if *is_left {
                hasher.update(sibling);
                hasher.update(&current);
            } else {
                hasher.update(&current);
                hasher.update(sibling);
            }
            let result = hasher.finalize();
            current.copy_from_slice(&result);
        }
        &current == expected_root
    }
}

// ── Merkle tree builder ────────────────────────────────────────────

fn hash_pair(a: &Hash, b: &Hash) -> Hash {
    let mut hasher = Sha256::new();
    hasher.update(a);
    hasher.update(b);
    let result = hasher.finalize();
    let mut h = [0u8; 32];
    h.copy_from_slice(&result);
    h
}

/// Build a Merkle tree from leaf hashes. Returns (root, layers).
/// layers[0] = leaves, layers[last] = [root].
fn build_merkle_tree(leaves: &[Hash]) -> (Hash, Vec<Vec<Hash>>) {
    if leaves.is_empty() {
        return ([0u8; 32], vec![vec![]]);
    }
    if leaves.len() == 1 {
        return (leaves[0], vec![leaves.to_vec()]);
    }

    let mut layers = vec![leaves.to_vec()];
    let mut current = leaves.to_vec();

    while current.len() > 1 {
        let mut next = Vec::with_capacity((current.len() + 1) / 2);
        for chunk in current.chunks(2) {
            if chunk.len() == 2 {
                next.push(hash_pair(&chunk[0], &chunk[1]));
            } else {
                next.push(hash_pair(&chunk[0], &chunk[0])); // duplicate odd
            }
        }
        layers.push(next.clone());
        current = next;
    }

    (current[0], layers)
}

/// Generate a Merkle proof for a leaf at `index`.
fn merkle_proof(layers: &[Vec<Hash>], index: usize, leaf_count: usize) -> Option<MerkleProof> {
    if layers.is_empty() || index >= layers[0].len() {
        return None;
    }

    let mut siblings = Vec::new();
    let mut idx = index;

    for layer in &layers[..layers.len().saturating_sub(1)] {
        let sibling_idx = if idx % 2 == 0 {
            if idx + 1 < layer.len() {
                idx + 1
            } else {
                idx
            }
        } else {
            idx - 1
        };
        let is_left = sibling_idx < idx;
        siblings.push((layer[sibling_idx], is_left));
        idx /= 2;
    }

    Some(MerkleProof {
        leaf_index: index,
        leaf_count,
        siblings,
    })
}

// ── Block receipt ──────────────────────────────────────────────────

/// Aggregated receipt for an entire block.
#[derive(Debug, Clone)]
pub struct BlockReceiptRecord {
    pub block_number: Epoch,
    /// Ordered transaction receipts.
    pub tx_receipts: Vec<TxReceipt>,
    /// Merkle root of receipt hashes.
    pub receipts_root: Hash,
    /// Total gas consumed in this block.
    pub total_gas: u64,
    /// Total events emitted in this block.
    pub total_events: usize,
}

// ── Receipt store ──────────────────────────────────────────────────

/// Persistent receipt storage with Merkle proof generation.
#[derive(Debug, Default)]
pub struct ReceiptStore {
    /// Block → receipt record.
    blocks: BTreeMap<Epoch, BlockReceiptRecord>,
    /// tx_hash → (block_number, tx_index) for fast lookup.
    tx_index: BTreeMap<Hash, (Epoch, u32)>,
    /// Block → Merkle tree layers (for proof generation).
    merkle_trees: BTreeMap<Epoch, Vec<Vec<Hash>>>,
}

impl ReceiptStore {
    pub fn new() -> Self {
        Self::default()
    }

    /// Store all receipts for a block. Computes the receipts root and indexes them.
    pub fn store_block_receipts(
        &mut self,
        block_number: Epoch,
        tx_receipts: Vec<TxReceipt>,
    ) -> Result<Hash, ReceiptError> {
        if self.blocks.contains_key(&block_number) {
            return Err(ReceiptError::BlockAlreadyStored(block_number));
        }

        let leaf_hashes: Vec<Hash> = tx_receipts.iter().map(|r| r.receipt_hash()).collect();
        let (root, layers) = build_merkle_tree(&leaf_hashes);

        let total_gas = tx_receipts.last().map(|r| r.cumulative_gas).unwrap_or(0);
        let total_events = tx_receipts.iter().map(|r| r.events.len()).sum();

        // Index tx hashes
        for receipt in &tx_receipts {
            self.tx_index
                .insert(receipt.tx_hash, (block_number, receipt.tx_index));
        }

        self.merkle_trees.insert(block_number, layers);
        self.blocks.insert(
            block_number,
            BlockReceiptRecord {
                block_number,
                tx_receipts,
                receipts_root: root,
                total_gas,
                total_events,
            },
        );

        Ok(root)
    }

    /// Get the receipts root for a block.
    pub fn receipts_root(&self, block_number: Epoch) -> Option<Hash> {
        self.blocks.get(&block_number).map(|b| b.receipts_root)
    }

    /// Look up a receipt by transaction hash.
    pub fn get_receipt(&self, tx_hash: &Hash) -> Option<&TxReceipt> {
        let (block, tx_idx) = self.tx_index.get(tx_hash)?;
        let record = self.blocks.get(block)?;
        record.tx_receipts.get(*tx_idx as usize)
    }

    /// Get all receipts for a block.
    pub fn block_receipts(&self, block_number: Epoch) -> Option<&[TxReceipt]> {
        self.blocks
            .get(&block_number)
            .map(|b| b.tx_receipts.as_slice())
    }

    /// Generate a Merkle inclusion proof for a transaction receipt.
    pub fn prove_receipt(&self, tx_hash: &Hash) -> Option<(TxReceipt, MerkleProof, Hash)> {
        let (block, tx_idx) = self.tx_index.get(tx_hash)?;
        let record = self.blocks.get(block)?;
        let layers = self.merkle_trees.get(block)?;
        let receipt = record.tx_receipts.get(*tx_idx as usize)?.clone();
        let proof = merkle_proof(layers, *tx_idx as usize, record.tx_receipts.len())?;
        Some((receipt, proof, record.receipts_root))
    }

    /// Verify a receipt inclusion proof against a known root.
    pub fn verify_receipt_proof(receipt: &TxReceipt, proof: &MerkleProof, root: &Hash) -> bool {
        let leaf = receipt.receipt_hash();
        proof.verify(&leaf, root)
    }

    /// Total number of stored blocks.
    pub fn block_count(&self) -> usize {
        self.blocks.len()
    }

    /// Total number of stored receipts across all blocks.
    pub fn receipt_count(&self) -> usize {
        self.tx_index.len()
    }

    /// Prune receipts older than `before` epoch. Returns number of blocks pruned.
    pub fn prune_before(&mut self, before: Epoch) -> usize {
        let to_remove: Vec<Epoch> = self.blocks.range(..before).map(|(&k, _)| k).collect();
        let count = to_remove.len();
        for block in to_remove {
            if let Some(record) = self.blocks.remove(&block) {
                for receipt in &record.tx_receipts {
                    self.tx_index.remove(&receipt.tx_hash);
                }
            }
            self.merkle_trees.remove(&block);
        }
        count
    }

    /// Get block receipt record (metadata without cloning all receipts).
    pub fn block_record(&self, block_number: Epoch) -> Option<&BlockReceiptRecord> {
        self.blocks.get(&block_number)
    }
}

/// Errors from receipt storage.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ReceiptError {
    BlockAlreadyStored(Epoch),
}

impl std::fmt::Display for ReceiptError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::BlockAlreadyStored(b) => write!(f, "block {b} receipts already stored"),
        }
    }
}

// ── Tests ──────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::Address;

    fn addr(id: u8) -> Address {
        Address::test(id)
    }

    fn tx_hash(id: u8) -> Hash {
        let mut h = [0u8; 32];
        h[0] = id;
        h[31] = id;
        h
    }

    fn make_receipt(id: u8, block: Epoch, index: u32, gas: u64, cumulative: u64) -> TxReceipt {
        TxReceipt {
            tx_hash: tx_hash(id),
            from: addr(id),
            block_number: block,
            tx_index: index,
            status: TxStatus::Success,
            gas_used: gas,
            cumulative_gas: cumulative,
            events: vec![],
            events_root: [0u8; 32],
        }
    }

    #[test]
    fn test_store_and_retrieve_receipts() {
        let mut store = ReceiptStore::new();
        let receipts = vec![
            make_receipt(1, 100, 0, 21000, 21000),
            make_receipt(2, 100, 1, 50000, 71000),
        ];
        let root = store.store_block_receipts(100, receipts).unwrap();
        assert_ne!(root, [0u8; 32]);
        assert_eq!(store.block_count(), 1);
        assert_eq!(store.receipt_count(), 2);
    }

    #[test]
    fn test_lookup_by_tx_hash() {
        let mut store = ReceiptStore::new();
        let receipts = vec![
            make_receipt(1, 10, 0, 21000, 21000),
            make_receipt(2, 10, 1, 50000, 71000),
        ];
        store.store_block_receipts(10, receipts).unwrap();

        let r = store.get_receipt(&tx_hash(2)).unwrap();
        assert_eq!(r.gas_used, 50000);
        assert_eq!(r.tx_index, 1);

        assert!(store.get_receipt(&tx_hash(99)).is_none());
    }

    #[test]
    fn test_merkle_proof_single_receipt() {
        let mut store = ReceiptStore::new();
        let receipts = vec![make_receipt(1, 5, 0, 21000, 21000)];
        store.store_block_receipts(5, receipts).unwrap();

        let (receipt, proof, root) = store.prove_receipt(&tx_hash(1)).unwrap();
        assert!(ReceiptStore::verify_receipt_proof(&receipt, &proof, &root));
    }

    #[test]
    fn test_merkle_proof_multiple_receipts() {
        let mut store = ReceiptStore::new();
        let receipts = vec![
            make_receipt(1, 20, 0, 10000, 10000),
            make_receipt(2, 20, 1, 20000, 30000),
            make_receipt(3, 20, 2, 30000, 60000),
            make_receipt(4, 20, 3, 15000, 75000),
        ];
        let root = store.store_block_receipts(20, receipts).unwrap();

        // Prove each receipt
        for id in 1..=4u8 {
            let (receipt, proof, proven_root) = store.prove_receipt(&tx_hash(id)).unwrap();
            assert_eq!(proven_root, root);
            assert!(
                ReceiptStore::verify_receipt_proof(&receipt, &proof, &root),
                "proof failed for tx {id}"
            );
        }
    }

    #[test]
    fn test_merkle_proof_odd_count() {
        let mut store = ReceiptStore::new();
        let receipts = vec![
            make_receipt(1, 30, 0, 10000, 10000),
            make_receipt(2, 30, 1, 20000, 30000),
            make_receipt(3, 30, 2, 30000, 60000),
        ];
        store.store_block_receipts(30, receipts).unwrap();

        for id in 1..=3u8 {
            let (receipt, proof, root) = store.prove_receipt(&tx_hash(id)).unwrap();
            assert!(ReceiptStore::verify_receipt_proof(&receipt, &proof, &root));
        }
    }

    #[test]
    fn test_proof_fails_with_wrong_root() {
        let mut store = ReceiptStore::new();
        let receipts = vec![
            make_receipt(1, 40, 0, 10000, 10000),
            make_receipt(2, 40, 1, 20000, 30000),
        ];
        store.store_block_receipts(40, receipts).unwrap();

        let (receipt, proof, _root) = store.prove_receipt(&tx_hash(1)).unwrap();
        let fake_root = [0xFFu8; 32];
        assert!(!ReceiptStore::verify_receipt_proof(
            &receipt, &proof, &fake_root
        ));
    }

    #[test]
    fn test_proof_fails_with_tampered_receipt() {
        let mut store = ReceiptStore::new();
        let receipts = vec![
            make_receipt(1, 50, 0, 10000, 10000),
            make_receipt(2, 50, 1, 20000, 30000),
        ];
        store.store_block_receipts(50, receipts).unwrap();

        let (mut receipt, proof, root) = store.prove_receipt(&tx_hash(1)).unwrap();
        receipt.gas_used = 99999; // tamper
        assert!(!ReceiptStore::verify_receipt_proof(&receipt, &proof, &root));
    }

    #[test]
    fn test_duplicate_block_error() {
        let mut store = ReceiptStore::new();
        let receipts = vec![make_receipt(1, 60, 0, 21000, 21000)];
        store.store_block_receipts(60, receipts.clone()).unwrap();
        assert_eq!(
            store.store_block_receipts(60, receipts),
            Err(ReceiptError::BlockAlreadyStored(60))
        );
    }

    #[test]
    fn test_receipts_root_deterministic() {
        let mut store1 = ReceiptStore::new();
        let mut store2 = ReceiptStore::new();
        let receipts = vec![
            make_receipt(1, 70, 0, 10000, 10000),
            make_receipt(2, 70, 1, 20000, 30000),
        ];
        let root1 = store1.store_block_receipts(70, receipts.clone()).unwrap();
        let root2 = store2.store_block_receipts(70, receipts).unwrap();
        assert_eq!(root1, root2);
    }

    #[test]
    fn test_block_receipts_query() {
        let mut store = ReceiptStore::new();
        let receipts = vec![
            make_receipt(1, 80, 0, 10000, 10000),
            make_receipt(2, 80, 1, 20000, 30000),
        ];
        store.store_block_receipts(80, receipts).unwrap();

        let block_rx = store.block_receipts(80).unwrap();
        assert_eq!(block_rx.len(), 2);
        assert!(store.block_receipts(81).is_none());
    }

    #[test]
    fn test_prune_before() {
        let mut store = ReceiptStore::new();
        for b in 0..5u64 {
            store
                .store_block_receipts(b, vec![make_receipt(b as u8, b, 0, 10000, 10000)])
                .unwrap();
        }
        assert_eq!(store.block_count(), 5);
        assert_eq!(store.receipt_count(), 5);

        let pruned = store.prune_before(3);
        assert_eq!(pruned, 3);
        assert_eq!(store.block_count(), 2);
        assert_eq!(store.receipt_count(), 2);
        assert!(store.block_receipts(0).is_none());
        assert!(store.block_receipts(2).is_none());
        assert!(store.block_receipts(3).is_some());
        assert!(store.block_receipts(4).is_some());
    }

    #[test]
    fn test_receipt_with_events() {
        let mut store = ReceiptStore::new();
        let event = Event {
            emitter: addr(1),
            topics: vec![[0xAA; 32]],
            data: vec![1, 2, 3],
            block_number: 90,
            log_index: 0,
            tx_index: 0,
        };
        let events_root = {
            use crate::events::BlockReceipt;
            BlockReceipt::compute_root(&[event.clone()])
        };
        let receipt = TxReceipt {
            tx_hash: tx_hash(1),
            from: addr(1),
            block_number: 90,
            tx_index: 0,
            status: TxStatus::Success,
            gas_used: 50000,
            cumulative_gas: 50000,
            events: vec![event],
            events_root,
        };
        let root = store.store_block_receipts(90, vec![receipt]).unwrap();
        let (r, proof, _) = store.prove_receipt(&tx_hash(1)).unwrap();
        assert_eq!(r.events.len(), 1);
        assert!(ReceiptStore::verify_receipt_proof(&r, &proof, &root));
    }

    #[test]
    fn test_block_record_metadata() {
        let mut store = ReceiptStore::new();
        let receipts = vec![
            make_receipt(1, 95, 0, 10000, 10000),
            make_receipt(2, 95, 1, 20000, 30000),
        ];
        store.store_block_receipts(95, receipts).unwrap();

        let record = store.block_record(95).unwrap();
        assert_eq!(record.total_gas, 30000);
        assert_eq!(record.total_events, 0);
        assert_eq!(record.tx_receipts.len(), 2);
    }

    #[test]
    fn test_reverted_and_oog_status() {
        let mut store = ReceiptStore::new();
        let mut r1 = make_receipt(1, 99, 0, 21000, 21000);
        r1.status = TxStatus::Reverted;
        let mut r2 = make_receipt(2, 99, 1, 100000, 121000);
        r2.status = TxStatus::OutOfGas;

        store.store_block_receipts(99, vec![r1, r2]).unwrap();

        let rx1 = store.get_receipt(&tx_hash(1)).unwrap();
        assert_eq!(rx1.status, TxStatus::Reverted);
        let rx2 = store.get_receipt(&tx_hash(2)).unwrap();
        assert_eq!(rx2.status, TxStatus::OutOfGas);
    }

    #[test]
    fn test_empty_block_receipts() {
        let mut store = ReceiptStore::new();
        let root = store.store_block_receipts(200, vec![]).unwrap();
        assert_eq!(root, [0u8; 32]);
        assert_eq!(store.block_count(), 1);
        assert_eq!(store.receipt_count(), 0);
    }

    #[test]
    fn test_large_block_proofs() {
        let mut store = ReceiptStore::new();
        let receipts: Vec<TxReceipt> = (0..16u8)
            .map(|i| {
                let mut r = make_receipt(i, 300, i as u32, 10000, (i as u64 + 1) * 10000);
                r.tx_hash = {
                    let mut h = [0u8; 32];
                    h[0] = i;
                    h[1] = 0xFF;
                    h
                };
                r
            })
            .collect();
        let tx_hashes: Vec<Hash> = receipts.iter().map(|r| r.tx_hash).collect();
        let root = store.store_block_receipts(300, receipts).unwrap();

        for hash in &tx_hashes {
            let (receipt, proof, proven_root) = store.prove_receipt(hash).unwrap();
            assert_eq!(proven_root, root);
            assert!(ReceiptStore::verify_receipt_proof(&receipt, &proof, &root));
        }
    }
}
