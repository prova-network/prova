//! Blob Transaction Type (CHAIN-030)
//!
//! First-class transaction type for submitting data availability commitments
//! alongside inference commits. Links inference execution to verifiable data roots.
//!
//! # Design
//!
//! - Blob transactions carry a data reference (not the data itself)
//! - Separate fee market prevents DA costs from affecting execution gas
//! - EIP-1559 style fee adjustment based on blob utilization
//! - Automatic DAS commitment creation on execution
//! - Pruning after configurable retention period

use crate::das::{BlobId, DasEngine, DasStatus, TOTAL_CHUNKS};
use crate::types::{Address, Epoch, Hash};
use sha2::{Digest, Sha256};
use std::collections::HashMap;

// ─── Constants ───────────────────────────────────────────────────────────────

/// Base fee for any blob submission.
pub const BASE_BLOB_FEE: u128 = 100;
/// Additional fee per erasure-coded chunk.
pub const FEE_PER_CHUNK: u128 = 10;
/// Maximum blob transactions per block.
pub const MAX_BLOBS_PER_BLOCK: usize = 8;
/// Target blobs per block (for fee adjustment).
pub const TARGET_BLOBS_PER_BLOCK: usize = 4;
/// Fee adjustment rate (12.5% = 1/8).
pub const FEE_ADJUSTMENT_DENOM: u128 = 8;
/// Maximum original data size: 16 MiB.
pub const MAX_BLOB_SIZE: u64 = 16 * 1024 * 1024;
/// Default blob retention: ~14 days at 12s epochs.
pub const BLOB_RETENTION_EPOCHS: Epoch = 100_800;
/// Minimum blob size: 1 byte.
pub const MIN_BLOB_SIZE: u64 = 1;

// ─── Types ───────────────────────────────────────────────────────────────────

/// A blob transaction submitted to the chain.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BlobTransaction {
    pub sender: Address,
    pub nonce: u64,
    pub blob_id: BlobId,
    pub data_root: Hash,
    pub blob_size: u64,
    pub chunk_count: usize,
    /// Optional commit hash this blob supports.
    pub reference: Option<Hash>,
    /// Maximum fee the sender is willing to pay.
    pub max_fee: u128,
}

/// Result of executing a blob transaction.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BlobTxResult {
    /// Successfully committed.
    Success {
        blob_id: BlobId,
        fee_charged: u128,
    },
    /// Rejected with reason.
    Rejected(String),
}

/// Metadata for a committed blob (tracked for pruning).
#[derive(Debug, Clone)]
pub struct BlobMeta {
    pub blob_id: BlobId,
    pub sender: Address,
    pub data_root: Hash,
    pub blob_size: u64,
    pub chunk_count: usize,
    pub reference: Option<Hash>,
    pub committed_epoch: Epoch,
    pub fee_paid: u128,
}

/// Blob fee market state.
#[derive(Debug, Clone)]
pub struct BlobFeeMarket {
    /// Current fee multiplier (scaled by 1000 for precision).
    pub multiplier: u128,
    /// Blobs in current block.
    pub current_block_blobs: usize,
    /// Exponential moving average of blob utilization.
    pub utilization_ema: u128,
}

/// The blob transaction processor.
#[derive(Debug)]
pub struct BlobTxEngine {
    blobs: HashMap<BlobId, BlobMeta>,
    fee_market: BlobFeeMarket,
    balances: HashMap<Address, u128>,
    nonces: HashMap<Address, u64>,
    current_epoch: Epoch,
    block_blob_count: usize,
    /// Total fees collected (burnt).
    pub total_fees_burnt: u128,
}

// ─── Implementation ──────────────────────────────────────────────────────────

impl BlobFeeMarket {
    pub fn new() -> Self {
        Self {
            multiplier: 1000, // 1.0x
            current_block_blobs: 0,
            utilization_ema: 0,
        }
    }

    /// Calculate the fee for a blob with given chunk count.
    pub fn calculate_fee(&self, chunk_count: usize) -> u128 {
        let base = BASE_BLOB_FEE + chunk_count as u128 * FEE_PER_CHUNK;
        base * self.multiplier / 1000
    }

    /// Adjust fee multiplier at end of block based on utilization.
    pub fn end_block(&mut self, blobs_in_block: usize) {
        let target = TARGET_BLOBS_PER_BLOCK as u128;
        let actual = blobs_in_block as u128;

        if actual > target {
            // Increase: multiplier * (1 + 1/8)
            self.multiplier = self.multiplier + self.multiplier / FEE_ADJUSTMENT_DENOM;
        } else if actual < target && self.multiplier > 125 {
            // Decrease: multiplier * (1 - 1/8), floor at 0.125x
            self.multiplier = self.multiplier - self.multiplier / FEE_ADJUSTMENT_DENOM;
        }

        // Update EMA: ema = 0.9 * ema + 0.1 * actual (scaled by 1000)
        self.utilization_ema = (self.utilization_ema * 900 + actual * 100) / 1000;
        self.current_block_blobs = 0;
    }
}

impl BlobTxEngine {
    pub fn new() -> Self {
        Self {
            blobs: HashMap::new(),
            fee_market: BlobFeeMarket::new(),
            balances: HashMap::new(),
            nonces: HashMap::new(),
            current_epoch: 0,
            block_blob_count: 0,
            total_fees_burnt: 0,
        }
    }

    pub fn set_epoch(&mut self, epoch: Epoch) {
        self.current_epoch = epoch;
    }

    pub fn set_balance(&mut self, addr: Address, balance: u128) {
        self.balances.insert(addr, balance);
    }

    pub fn balance(&self, addr: &Address) -> u128 {
        self.balances.get(addr).copied().unwrap_or(0)
    }

    pub fn blob_count(&self) -> usize {
        self.blobs.len()
    }

    pub fn get_blob(&self, blob_id: &BlobId) -> Option<&BlobMeta> {
        self.blobs.get(blob_id)
    }

    pub fn fee_multiplier(&self) -> u128 {
        self.fee_market.multiplier
    }

    /// Validate a blob transaction without executing it.
    pub fn validate(&self, tx: &BlobTransaction) -> Result<u128, String> {
        // Size checks
        if tx.blob_size < MIN_BLOB_SIZE {
            return Err("blob size too small".into());
        }
        if tx.blob_size > MAX_BLOB_SIZE {
            return Err("blob size exceeds maximum".into());
        }

        // Chunk count validation
        let expected_chunks = expected_chunk_count(tx.blob_size);
        if tx.chunk_count != expected_chunks {
            return Err(format!(
                "chunk count mismatch: got {}, expected {}",
                tx.chunk_count, expected_chunks
            ));
        }

        // Duplicate check
        if self.blobs.contains_key(&tx.blob_id) {
            return Err("duplicate blob_id".into());
        }

        // Block capacity
        if self.block_blob_count >= MAX_BLOBS_PER_BLOCK {
            return Err("block blob limit reached".into());
        }

        // Fee calculation and check
        let required_fee = self.fee_market.calculate_fee(tx.chunk_count);
        if tx.max_fee < required_fee {
            return Err(format!(
                "insufficient fee: offered {}, required {}",
                tx.max_fee, required_fee
            ));
        }

        // Balance check
        let balance = self.balance(&tx.sender);
        if balance < required_fee {
            return Err(format!(
                "insufficient balance: have {}, need {}",
                balance, required_fee
            ));
        }

        // Nonce check
        let expected_nonce = self.nonces.get(&tx.sender).copied().unwrap_or(0);
        if tx.nonce != expected_nonce {
            return Err(format!(
                "nonce mismatch: got {}, expected {}",
                tx.nonce, expected_nonce
            ));
        }

        Ok(required_fee)
    }

    /// Execute a blob transaction: validate, charge fee, create DAS commitment.
    pub fn execute(&mut self, tx: &BlobTransaction) -> BlobTxResult {
        let fee = match self.validate(tx) {
            Ok(f) => f,
            Err(reason) => return BlobTxResult::Rejected(reason),
        };

        // Charge fee
        let balance = self.balances.get_mut(&tx.sender).unwrap();
        *balance -= fee;
        self.total_fees_burnt += fee;

        // Increment nonce
        let nonce = self.nonces.entry(tx.sender).or_insert(0);
        *nonce += 1;

        // Store blob metadata
        self.blobs.insert(
            tx.blob_id,
            BlobMeta {
                blob_id: tx.blob_id,
                sender: tx.sender,
                data_root: tx.data_root,
                blob_size: tx.blob_size,
                chunk_count: tx.chunk_count,
                reference: tx.reference,
                committed_epoch: self.current_epoch,
                fee_paid: fee,
            },
        );

        self.block_blob_count += 1;

        BlobTxResult::Success {
            blob_id: tx.blob_id,
            fee_charged: fee,
        }
    }

    /// End the current block: adjust fees, reset block counter.
    pub fn end_block(&mut self) {
        self.fee_market.end_block(self.block_blob_count);
        self.block_blob_count = 0;
    }

    /// Prune blobs older than retention period.
    pub fn prune(&mut self, current_epoch: Epoch) -> Vec<BlobId> {
        let mut pruned = Vec::new();
        self.blobs.retain(|blob_id, meta| {
            if current_epoch.saturating_sub(meta.committed_epoch) > BLOB_RETENTION_EPOCHS {
                pruned.push(*blob_id);
                false
            } else {
                true
            }
        });
        pruned
    }

    /// Execute a blob transaction and also create a DAS commitment.
    pub fn execute_with_das(
        &mut self,
        tx: &BlobTransaction,
        das: &mut DasEngine,
    ) -> BlobTxResult {
        let result = self.execute(tx);
        if let BlobTxResult::Success { blob_id, .. } = &result {
            // Create DAS commitment — ignore error if already exists (shouldn't happen)
            let _ = das.submit_commitment(
                *blob_id,
                tx.sender,
                tx.data_root,
                tx.chunk_count,
            );
        }
        result
    }
}

/// Calculate expected erasure-coded chunk count for a given blob size.
/// Uses 256 KiB chunks, doubled for parity.
pub fn expected_chunk_count(blob_size: u64) -> usize {
    let chunk_size: u64 = 256 * 1024; // 256 KiB
    let original = ((blob_size + chunk_size - 1) / chunk_size) as usize;
    let original = original.max(1); // At least 1 chunk
    original * 2 // Double for erasure coding parity
}

/// Compute a blob ID from raw data.
pub fn compute_blob_id(data: &[u8]) -> BlobId {
    let hash: Hash = Sha256::digest(data).into();
    BlobId(hash)
}

// ─── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn test_addr(id: u8) -> Address {
        Address::test(id)
    }

    fn make_blob_tx(sender: Address, nonce: u64, size: u64) -> BlobTransaction {
        let data = vec![0xAB; size as usize];
        let blob_id = compute_blob_id(&data);
        BlobTransaction {
            sender,
            nonce,
            blob_id,
            data_root: [0x42; 32],
            blob_size: size,
            chunk_count: expected_chunk_count(size),
            reference: None,
            max_fee: 100_000,
        }
    }

    #[test]
    fn test_expected_chunk_count() {
        // 1 byte → 1 original chunk → 2 total
        assert_eq!(expected_chunk_count(1), 2);
        // 256 KiB → 1 original → 2 total
        assert_eq!(expected_chunk_count(256 * 1024), 2);
        // 256 KiB + 1 → 2 original → 4 total
        assert_eq!(expected_chunk_count(256 * 1024 + 1), 4);
        // 1 MiB → 4 original → 8 total
        assert_eq!(expected_chunk_count(1024 * 1024), 8);
        // 16 MiB → 64 original → 128 total
        assert_eq!(expected_chunk_count(16 * 1024 * 1024), 128);
    }

    #[test]
    fn test_compute_blob_id() {
        let data = b"hello prova";
        let id1 = compute_blob_id(data);
        let id2 = compute_blob_id(data);
        assert_eq!(id1, id2);
        let id3 = compute_blob_id(b"different");
        assert_ne!(id1, id3);
    }

    #[test]
    fn test_basic_blob_submission() {
        let mut engine = BlobTxEngine::new();
        let alice = test_addr(1);
        engine.set_balance(alice, 1_000_000);

        let tx = make_blob_tx(alice, 0, 512 * 1024); // 512 KiB
        let result = engine.execute(&tx);

        match result {
            BlobTxResult::Success { fee_charged, .. } => {
                assert!(fee_charged > 0);
                assert_eq!(engine.blob_count(), 1);
                assert!(engine.balance(&alice) < 1_000_000);
            }
            BlobTxResult::Rejected(r) => panic!("unexpected rejection: {r}"),
        }
    }

    #[test]
    fn test_duplicate_blob_rejected() {
        let mut engine = BlobTxEngine::new();
        let alice = test_addr(1);
        engine.set_balance(alice, 1_000_000);

        let tx = make_blob_tx(alice, 0, 1024);
        engine.execute(&tx);

        // Same blob_id, next nonce
        let mut tx2 = tx.clone();
        tx2.nonce = 1;
        let result = engine.execute(&tx2);
        assert!(matches!(result, BlobTxResult::Rejected(r) if r.contains("duplicate")));
    }

    #[test]
    fn test_insufficient_balance() {
        let mut engine = BlobTxEngine::new();
        let alice = test_addr(1);
        engine.set_balance(alice, 1); // Too little

        let tx = make_blob_tx(alice, 0, 1024);
        let result = engine.execute(&tx);
        assert!(matches!(result, BlobTxResult::Rejected(r) if r.contains("insufficient balance")));
    }

    #[test]
    fn test_blob_too_large() {
        let mut engine = BlobTxEngine::new();
        let alice = test_addr(1);
        engine.set_balance(alice, 1_000_000_000);

        let tx = BlobTransaction {
            sender: alice,
            nonce: 0,
            blob_id: BlobId([0x01; 32]),
            data_root: [0x42; 32],
            blob_size: MAX_BLOB_SIZE + 1,
            chunk_count: expected_chunk_count(MAX_BLOB_SIZE + 1),
            reference: None,
            max_fee: 1_000_000_000,
        };
        let result = engine.execute(&tx);
        assert!(matches!(result, BlobTxResult::Rejected(r) if r.contains("exceeds maximum")));
    }

    #[test]
    fn test_blob_size_zero() {
        let mut engine = BlobTxEngine::new();
        let alice = test_addr(1);
        engine.set_balance(alice, 1_000_000);

        let tx = BlobTransaction {
            sender: alice,
            nonce: 0,
            blob_id: BlobId([0x02; 32]),
            data_root: [0x42; 32],
            blob_size: 0,
            chunk_count: 0,
            reference: None,
            max_fee: 100_000,
        };
        let result = engine.execute(&tx);
        assert!(matches!(result, BlobTxResult::Rejected(r) if r.contains("too small")));
    }

    #[test]
    fn test_wrong_nonce() {
        let mut engine = BlobTxEngine::new();
        let alice = test_addr(1);
        engine.set_balance(alice, 1_000_000);

        let mut tx = make_blob_tx(alice, 5, 1024); // Wrong nonce (should be 0)
        let result = engine.execute(&tx);
        assert!(matches!(result, BlobTxResult::Rejected(r) if r.contains("nonce")));
    }

    #[test]
    fn test_block_blob_limit() {
        let mut engine = BlobTxEngine::new();
        let alice = test_addr(1);
        engine.set_balance(alice, 100_000_000);

        // Fill block with MAX_BLOBS_PER_BLOCK blobs
        for i in 0..MAX_BLOBS_PER_BLOCK {
            let data = vec![i as u8; 1024];
            let blob_id = compute_blob_id(&data);
            let tx = BlobTransaction {
                sender: alice,
                nonce: i as u64,
                blob_id,
                data_root: [i as u8; 32],
                blob_size: 1024,
                chunk_count: expected_chunk_count(1024),
                reference: None,
                max_fee: 100_000,
            };
            let result = engine.execute(&tx);
            assert!(matches!(result, BlobTxResult::Success { .. }), "blob {i} should succeed");
        }

        // 9th blob should be rejected
        let data = vec![0xFF; 1024];
        let blob_id = compute_blob_id(&data);
        let tx = BlobTransaction {
            sender: alice,
            nonce: MAX_BLOBS_PER_BLOCK as u64,
            blob_id,
            data_root: [0xFF; 32],
            blob_size: 1024,
            chunk_count: expected_chunk_count(1024),
            reference: None,
            max_fee: 100_000,
        };
        let result = engine.execute(&tx);
        assert!(matches!(result, BlobTxResult::Rejected(r) if r.contains("block blob limit")));
    }

    #[test]
    fn test_end_block_resets_counter() {
        let mut engine = BlobTxEngine::new();
        let alice = test_addr(1);
        engine.set_balance(alice, 100_000_000);

        // Submit blobs in block 1
        let tx = make_blob_tx(alice, 0, 1024);
        engine.execute(&tx);
        engine.end_block();

        // Block 2 should accept blobs again
        let data = vec![0xCC; 2048];
        let blob_id = compute_blob_id(&data);
        let tx2 = BlobTransaction {
            sender: alice,
            nonce: 1,
            blob_id,
            data_root: [0xCC; 32],
            blob_size: 2048,
            chunk_count: expected_chunk_count(2048),
            reference: None,
            max_fee: 100_000,
        };
        let result = engine.execute(&tx2);
        assert!(matches!(result, BlobTxResult::Success { .. }));
    }

    #[test]
    fn test_fee_market_adjustment() {
        let mut market = BlobFeeMarket::new();
        let initial = market.multiplier;

        // Above target → fee increases
        market.end_block(6); // 6 > 4 target
        assert!(market.multiplier > initial);

        // Below target → fee decreases
        let high = market.multiplier;
        market.end_block(1);
        assert!(market.multiplier < high);
    }

    #[test]
    fn test_pruning() {
        let mut engine = BlobTxEngine::new();
        let alice = test_addr(1);
        engine.set_balance(alice, 100_000_000);
        engine.set_epoch(100);

        let tx = make_blob_tx(alice, 0, 1024);
        engine.execute(&tx);
        assert_eq!(engine.blob_count(), 1);

        // Prune at epoch well past retention
        let pruned = engine.prune(100 + BLOB_RETENTION_EPOCHS + 1);
        assert_eq!(pruned.len(), 1);
        assert_eq!(engine.blob_count(), 0);
    }

    #[test]
    fn test_pruning_retains_recent() {
        let mut engine = BlobTxEngine::new();
        let alice = test_addr(1);
        engine.set_balance(alice, 100_000_000);
        engine.set_epoch(1000);

        let tx = make_blob_tx(alice, 0, 1024);
        engine.execute(&tx);

        // Prune at epoch still within retention
        let pruned = engine.prune(1000 + BLOB_RETENTION_EPOCHS - 1);
        assert_eq!(pruned.len(), 0);
        assert_eq!(engine.blob_count(), 1);
    }

    #[test]
    fn test_execute_with_das() {
        let mut engine = BlobTxEngine::new();
        let mut das = DasEngine::new();
        let alice = test_addr(1);
        engine.set_balance(alice, 1_000_000);

        let tx = make_blob_tx(alice, 0, 1024);
        let blob_id = tx.blob_id;
        let result = engine.execute_with_das(&tx, &mut das);

        assert!(matches!(result, BlobTxResult::Success { .. }));
        // DAS commitment should exist
        assert!(das.get_commitment(&blob_id).is_some());
    }

    #[test]
    fn test_fee_scales_with_chunks() {
        let market = BlobFeeMarket::new();
        let fee_small = market.calculate_fee(2);   // 1 KiB blob
        let fee_large = market.calculate_fee(128);  // 16 MiB blob
        assert!(fee_large > fee_small);
        assert_eq!(fee_small, BASE_BLOB_FEE + 2 * FEE_PER_CHUNK);
        assert_eq!(fee_large, BASE_BLOB_FEE + 128 * FEE_PER_CHUNK);
    }

    #[test]
    fn test_chunk_count_mismatch_rejected() {
        let mut engine = BlobTxEngine::new();
        let alice = test_addr(1);
        engine.set_balance(alice, 1_000_000);

        let tx = BlobTransaction {
            sender: alice,
            nonce: 0,
            blob_id: BlobId([0x77; 32]),
            data_root: [0x42; 32],
            blob_size: 1024,
            chunk_count: 999, // Wrong
            reference: None,
            max_fee: 100_000,
        };
        let result = engine.execute(&tx);
        assert!(matches!(result, BlobTxResult::Rejected(r) if r.contains("chunk count mismatch")));
    }

    #[test]
    fn test_blob_with_reference() {
        let mut engine = BlobTxEngine::new();
        let alice = test_addr(1);
        engine.set_balance(alice, 1_000_000);

        let commit_hash = [0xDE; 32];
        let data = b"inference activations";
        let blob_id = compute_blob_id(data);
        let tx = BlobTransaction {
            sender: alice,
            nonce: 0,
            blob_id,
            data_root: [0x42; 32],
            blob_size: data.len() as u64,
            chunk_count: expected_chunk_count(data.len() as u64),
            reference: Some(commit_hash),
            max_fee: 100_000,
        };
        let result = engine.execute(&tx);
        assert!(matches!(result, BlobTxResult::Success { .. }));

        let meta = engine.get_blob(&blob_id).unwrap();
        assert_eq!(meta.reference, Some(commit_hash));
    }
}
