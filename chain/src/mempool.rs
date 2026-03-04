//! Transaction mempool with priority ordering, nonce tracking, and eviction.
//!
//! Manages pending transactions before they are included in blocks.
//! Supports:
//! - Priority ordering by fee (gas_price × gas_limit)
//! - Per-sender nonce sequencing
//! - Configurable size limits with lowest-fee eviction
//! - Transaction replacement (same sender+nonce, higher fee)
//! - Expiry based on age (epochs)

use crate::types::{Address, Epoch, Hash};
use std::collections::{BTreeMap, BTreeSet, HashMap};

/// Transaction type tags.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TxKind {
    /// Submit an inference commit (activation root).
    InferenceCommit,
    /// Open a dispute / challenge.
    Challenge,
    /// Bisection move in a dispute game.
    BisectionMove,
    /// Stake deposit or withdrawal.
    StakeOp,
    /// Register or update a model.
    ModelRegistry,
    /// Payment channel operation (open/close/settle).
    PaymentOp,
    /// PDP proof submission.
    PdpProof,
    /// Generic data transaction.
    Transfer,
}

/// A pending transaction.
#[derive(Debug, Clone)]
pub struct Transaction {
    /// Unique hash of the transaction.
    pub hash: Hash,
    /// Sender address.
    pub sender: Address,
    /// Sender nonce (must be sequential).
    pub nonce: u64,
    /// Gas price (fee per unit).
    pub gas_price: u64,
    /// Gas limit.
    pub gas_limit: u64,
    /// Transaction type.
    pub kind: TxKind,
    /// Epoch when the transaction was submitted.
    pub submitted_at: Epoch,
    /// Serialized payload size in bytes.
    pub size: usize,
}

impl Transaction {
    /// Effective fee = gas_price × gas_limit.
    pub fn fee(&self) -> u128 {
        self.gas_price as u128 * self.gas_limit as u128
    }
}

/// Ordering key: (fee descending, submission time ascending, hash for tiebreak).
#[derive(Debug, Clone, PartialEq, Eq)]
struct PriorityKey {
    /// Negated fee for descending order in BTreeSet.
    neg_fee: i128,
    submitted_at: Epoch,
    hash: Hash,
}

impl PartialOrd for PriorityKey {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for PriorityKey {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.neg_fee
            .cmp(&other.neg_fee)
            .then(self.submitted_at.cmp(&other.submitted_at))
            .then(self.hash.cmp(&other.hash))
    }
}

fn priority_key(tx: &Transaction) -> PriorityKey {
    PriorityKey {
        neg_fee: -(tx.fee() as i128),
        submitted_at: tx.submitted_at,
        hash: tx.hash,
    }
}

/// Mempool configuration.
#[derive(Debug, Clone)]
pub struct MempoolConfig {
    /// Maximum number of transactions.
    pub max_txs: usize,
    /// Maximum total size in bytes.
    pub max_bytes: usize,
    /// Transaction expiry in epochs (0 = no expiry).
    pub expiry_epochs: Epoch,
    /// Minimum fee multiplier for replacement (e.g., 110 = 10% bump required).
    pub replacement_fee_pct: u64,
    /// Maximum transactions per sender.
    pub max_per_sender: usize,
}

impl Default for MempoolConfig {
    fn default() -> Self {
        Self {
            max_txs: 10_000,
            max_bytes: 64 * 1024 * 1024, // 64 MB
            expiry_epochs: 100,
            replacement_fee_pct: 110,
            max_per_sender: 100,
        }
    }
}

/// Result of adding a transaction.
#[derive(Debug, PartialEq, Eq)]
pub enum AddResult {
    /// Successfully added.
    Added,
    /// Replaced an existing transaction (returns old hash).
    Replaced(Hash),
    /// Rejected: duplicate hash.
    DuplicateHash,
    /// Rejected: nonce too low (already confirmed or pending with higher fee).
    NonceTooLow,
    /// Rejected: replacement fee too low.
    ReplacementFeeTooLow,
    /// Rejected: sender queue full.
    SenderQueueFull,
    /// Rejected: pool full and tx has lower fee than all current txs.
    PoolFull,
}

/// Transaction mempool.
pub struct Mempool {
    config: MempoolConfig,
    /// All transactions by hash.
    txs: HashMap<Hash, Transaction>,
    /// Priority-ordered set for block inclusion.
    priority: BTreeSet<PriorityKey>,
    /// Per-sender: nonce → tx hash.
    sender_txs: HashMap<Address, BTreeMap<u64, Hash>>,
    /// Total byte size.
    total_bytes: usize,
    /// Confirmed nonces per sender (for nonce validation).
    confirmed_nonces: HashMap<Address, u64>,
}

impl Mempool {
    pub fn new(config: MempoolConfig) -> Self {
        Self {
            config,
            txs: HashMap::new(),
            priority: BTreeSet::new(),
            sender_txs: HashMap::new(),
            total_bytes: 0,
            confirmed_nonces: HashMap::new(),
        }
    }

    /// Set the confirmed nonce for a sender (transactions below this nonce are invalid).
    pub fn set_confirmed_nonce(&mut self, sender: Address, nonce: u64) {
        self.confirmed_nonces.insert(sender, nonce);
        // Evict any transactions with nonce < confirmed
        if let Some(sender_map) = self.sender_txs.get(&sender).cloned() {
            let to_remove: Vec<u64> = sender_map.keys().filter(|&&n| n < nonce).copied().collect();
            for n in to_remove {
                if let Some(hash) = self.sender_txs.get_mut(&sender).and_then(|m| m.remove(&n)) {
                    self.remove_by_hash(&hash);
                }
            }
        }
    }

    /// Add a transaction to the pool.
    pub fn add(&mut self, tx: Transaction) -> AddResult {
        // Duplicate check
        if self.txs.contains_key(&tx.hash) {
            return AddResult::DuplicateHash;
        }

        // Nonce check
        let confirmed = self.confirmed_nonces.get(&tx.sender).copied().unwrap_or(0);
        if tx.nonce < confirmed {
            return AddResult::NonceTooLow;
        }

        // Per-sender limit check
        let sender_count = self.sender_txs.get(&tx.sender).map_or(0, |m| m.len());

        // Check for replacement (same sender + nonce)
        if let Some(existing_hash) = self
            .sender_txs
            .get(&tx.sender)
            .and_then(|m| m.get(&tx.nonce))
            .copied()
        {
            let existing = &self.txs[&existing_hash];
            let min_fee = existing.fee() * self.config.replacement_fee_pct as u128 / 100;
            if tx.fee() < min_fee {
                return AddResult::ReplacementFeeTooLow;
            }
            // Remove old tx, add new
            let old_hash = existing_hash;
            self.remove_by_hash(&old_hash);
            self.insert(tx);
            return AddResult::Replaced(old_hash);
        }

        // Sender queue full
        if sender_count >= self.config.max_per_sender {
            return AddResult::SenderQueueFull;
        }

        // Pool capacity check
        if self.txs.len() >= self.config.max_txs || self.total_bytes + tx.size > self.config.max_bytes {
            // Evict lowest priority if new tx is better
            if let Some(worst) = self.priority.iter().next_back().cloned() {
                let pk = priority_key(&tx);
                if pk >= worst {
                    return AddResult::PoolFull;
                }
                // Evict worst
                self.remove_by_hash(&worst.hash);
            } else {
                return AddResult::PoolFull;
            }
        }

        self.insert(tx);
        AddResult::Added
    }

    /// Remove expired transactions.
    pub fn expire(&mut self, current_epoch: Epoch) {
        if self.config.expiry_epochs == 0 {
            return;
        }
        let cutoff = current_epoch.saturating_sub(self.config.expiry_epochs);
        let expired: Vec<Hash> = self
            .txs
            .values()
            .filter(|tx| tx.submitted_at < cutoff)
            .map(|tx| tx.hash)
            .collect();
        for hash in expired {
            self.remove_by_hash(&hash);
        }
    }

    /// Get the top `n` transactions ordered by priority (for block inclusion).
    pub fn top(&self, n: usize) -> Vec<&Transaction> {
        self.priority
            .iter()
            .take(n)
            .filter_map(|pk| self.txs.get(&pk.hash))
            .collect()
    }

    /// Get ordered transactions for a specific sender.
    pub fn sender_queue(&self, sender: &Address) -> Vec<&Transaction> {
        self.sender_txs
            .get(sender)
            .map(|m| {
                m.values()
                    .filter_map(|h| self.txs.get(h))
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Remove a transaction by hash (e.g., after inclusion in a block).
    pub fn remove(&mut self, hash: &Hash) -> bool {
        self.remove_by_hash(hash)
    }

    /// Remove all transactions included in a block.
    pub fn remove_batch(&mut self, hashes: &[Hash]) {
        for h in hashes {
            self.remove_by_hash(h);
        }
    }

    /// Number of pending transactions.
    pub fn len(&self) -> usize {
        self.txs.len()
    }

    /// Whether the pool is empty.
    pub fn is_empty(&self) -> bool {
        self.txs.is_empty()
    }

    /// Total bytes of pending transactions.
    pub fn total_bytes(&self) -> usize {
        self.total_bytes
    }

    /// Check if a transaction hash is in the pool.
    pub fn contains(&self, hash: &Hash) -> bool {
        self.txs.contains_key(hash)
    }

    /// Get a transaction by hash.
    pub fn get(&self, hash: &Hash) -> Option<&Transaction> {
        self.txs.get(hash)
    }

    // ---- internal helpers ----

    fn insert(&mut self, tx: Transaction) {
        let pk = priority_key(&tx);
        self.total_bytes += tx.size;
        self.sender_txs
            .entry(tx.sender)
            .or_default()
            .insert(tx.nonce, tx.hash);
        self.priority.insert(pk);
        self.txs.insert(tx.hash, tx);
    }

    fn remove_by_hash(&mut self, hash: &Hash) -> bool {
        if let Some(tx) = self.txs.remove(hash) {
            let pk = priority_key(&tx);
            self.priority.remove(&pk);
            self.total_bytes -= tx.size;
            if let Some(sender_map) = self.sender_txs.get_mut(&tx.sender) {
                sender_map.remove(&tx.nonce);
                if sender_map.is_empty() {
                    self.sender_txs.remove(&tx.sender);
                }
            }
            true
        } else {
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_tx(id: u8, sender_id: u8, nonce: u64, gas_price: u64) -> Transaction {
        let mut hash = [0u8; 32];
        hash[0] = id;
        Transaction {
            hash,
            sender: Address::test(sender_id),
            nonce,
            gas_price,
            gas_limit: 21_000,
            kind: TxKind::Transfer,
            submitted_at: 1,
            size: 256,
        }
    }

    #[test]
    fn test_add_and_retrieve() {
        let mut pool = Mempool::new(MempoolConfig::default());
        let tx = make_tx(1, 1, 0, 100);
        assert_eq!(pool.add(tx), AddResult::Added);
        assert_eq!(pool.len(), 1);
        assert!(!pool.is_empty());
    }

    #[test]
    fn test_duplicate_rejected() {
        let mut pool = Mempool::new(MempoolConfig::default());
        let tx = make_tx(1, 1, 0, 100);
        pool.add(tx.clone());
        assert_eq!(pool.add(tx), AddResult::DuplicateHash);
    }

    #[test]
    fn test_priority_ordering() {
        let mut pool = Mempool::new(MempoolConfig::default());
        pool.add(make_tx(1, 1, 0, 50));  // low fee
        pool.add(make_tx(2, 2, 0, 200)); // high fee
        pool.add(make_tx(3, 3, 0, 100)); // mid fee

        let top = pool.top(3);
        assert_eq!(top.len(), 3);
        assert_eq!(top[0].gas_price, 200); // highest first
        assert_eq!(top[1].gas_price, 100);
        assert_eq!(top[2].gas_price, 50);
    }

    #[test]
    fn test_replacement_with_higher_fee() {
        let mut pool = Mempool::new(MempoolConfig::default());
        let tx1 = make_tx(1, 1, 0, 100);
        let tx2 = make_tx(2, 1, 0, 150); // same sender+nonce, higher fee
        pool.add(tx1);
        let result = pool.add(tx2);
        assert!(matches!(result, AddResult::Replaced(_)));
        assert_eq!(pool.len(), 1);
        assert_eq!(pool.top(1)[0].gas_price, 150);
    }

    #[test]
    fn test_replacement_rejected_low_fee() {
        let mut pool = Mempool::new(MempoolConfig::default());
        pool.add(make_tx(1, 1, 0, 100));
        let tx2 = make_tx(2, 1, 0, 105); // only 5% bump, need 10%
        assert_eq!(pool.add(tx2), AddResult::ReplacementFeeTooLow);
    }

    #[test]
    fn test_nonce_too_low() {
        let mut pool = Mempool::new(MempoolConfig::default());
        pool.set_confirmed_nonce(Address::test(1), 5);
        let tx = make_tx(1, 1, 3, 100); // nonce 3 < confirmed 5
        assert_eq!(pool.add(tx), AddResult::NonceTooLow);
    }

    #[test]
    fn test_eviction_on_full_pool() {
        let config = MempoolConfig {
            max_txs: 2,
            ..Default::default()
        };
        let mut pool = Mempool::new(config);
        pool.add(make_tx(1, 1, 0, 100));
        pool.add(make_tx(2, 2, 0, 200));
        // Pool full. New tx with higher fee evicts lowest.
        let result = pool.add(make_tx(3, 3, 0, 150));
        assert_eq!(result, AddResult::Added);
        assert_eq!(pool.len(), 2);
        // Lowest (100) should be evicted
        assert!(!pool.contains(&make_tx(1, 1, 0, 100).hash));
    }

    #[test]
    fn test_pool_full_rejected() {
        let config = MempoolConfig {
            max_txs: 2,
            ..Default::default()
        };
        let mut pool = Mempool::new(config);
        pool.add(make_tx(1, 1, 0, 100));
        pool.add(make_tx(2, 2, 0, 200));
        // New tx with LOWER fee than all → rejected
        assert_eq!(pool.add(make_tx(3, 3, 0, 50)), AddResult::PoolFull);
    }

    #[test]
    fn test_sender_queue_limit() {
        let config = MempoolConfig {
            max_per_sender: 2,
            ..Default::default()
        };
        let mut pool = Mempool::new(config);
        pool.add(make_tx(1, 1, 0, 100));
        pool.add(make_tx(2, 1, 1, 100));
        assert_eq!(pool.add(make_tx(3, 1, 2, 100)), AddResult::SenderQueueFull);
    }

    #[test]
    fn test_expire() {
        let mut pool = Mempool::new(MempoolConfig {
            expiry_epochs: 10,
            ..Default::default()
        });
        let mut tx = make_tx(1, 1, 0, 100);
        tx.submitted_at = 5;
        pool.add(tx);
        pool.expire(14); // not expired yet (14 - 10 = 4, tx at 5)
        assert_eq!(pool.len(), 1);
        pool.expire(16); // expired (16 - 10 = 6 > 5)
        assert_eq!(pool.len(), 0);
    }

    #[test]
    fn test_remove_batch() {
        let mut pool = Mempool::new(MempoolConfig::default());
        let tx1 = make_tx(1, 1, 0, 100);
        let tx2 = make_tx(2, 2, 0, 100);
        let h1 = tx1.hash;
        let h2 = tx2.hash;
        pool.add(tx1);
        pool.add(tx2);
        pool.remove_batch(&[h1, h2]);
        assert!(pool.is_empty());
    }

    #[test]
    fn test_sender_queue_ordered() {
        let mut pool = Mempool::new(MempoolConfig::default());
        pool.add(make_tx(3, 1, 2, 100));
        pool.add(make_tx(1, 1, 0, 100));
        pool.add(make_tx(2, 1, 1, 100));
        let q = pool.sender_queue(&Address::test(1));
        assert_eq!(q.len(), 3);
        assert_eq!(q[0].nonce, 0);
        assert_eq!(q[1].nonce, 1);
        assert_eq!(q[2].nonce, 2);
    }

    #[test]
    fn test_confirmed_nonce_evicts_stale() {
        let mut pool = Mempool::new(MempoolConfig::default());
        pool.add(make_tx(1, 1, 0, 100));
        pool.add(make_tx(2, 1, 1, 100));
        pool.add(make_tx(3, 1, 2, 100));
        pool.set_confirmed_nonce(Address::test(1), 2);
        assert_eq!(pool.len(), 1); // only nonce 2 remains
        assert!(pool.contains(&make_tx(3, 1, 2, 100).hash));
    }

    #[test]
    fn test_total_bytes_tracking() {
        let mut pool = Mempool::new(MempoolConfig::default());
        let tx = make_tx(1, 1, 0, 100); // size = 256
        let h = tx.hash;
        pool.add(tx);
        assert_eq!(pool.total_bytes(), 256);
        pool.remove(&h);
        assert_eq!(pool.total_bytes(), 0);
    }

    #[test]
    fn test_tx_kinds() {
        let mut pool = Mempool::new(MempoolConfig::default());
        let mut tx = make_tx(1, 1, 0, 100);
        tx.kind = TxKind::InferenceCommit;
        pool.add(tx);
        let mut tx2 = make_tx(2, 1, 1, 100);
        tx2.kind = TxKind::PdpProof;
        pool.add(tx2);
        assert_eq!(pool.len(), 2);
    }

    #[test]
    fn test_get_transaction() {
        let mut pool = Mempool::new(MempoolConfig::default());
        let tx = make_tx(1, 1, 0, 100);
        let h = tx.hash;
        pool.add(tx);
        let got = pool.get(&h).unwrap();
        assert_eq!(got.gas_price, 100);
        assert_eq!(got.nonce, 0);
    }
}
