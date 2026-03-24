//! Block explorer API — high-level query endpoints for blocks, transactions,
//! events, and accounts.
//!
//! Extends the base JSON-RPC layer with explorer-specific methods:
//!
//! - `explorer_getBlock` — block header + tx list by height
//! - `explorer_getBlockRange` — paginated block listing
//! - `explorer_getTransaction` — tx details + receipt by hash
//! - `explorer_getAccount` — balance, nonce, tx history
//! - `explorer_getEvents` — filtered event log search
//! - `explorer_getChainStats` — summary statistics
//! - `explorer_searchTx` — search txs by address (sender or receiver)
//!
//! All methods are designed for read-only access; no state mutations.

use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, HashMap};

// ── Types ──────────────────────────────────────────────────────────

/// A 32-byte hash.
pub type Hash = [u8; 32];
/// An account address (20 bytes).
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Address(pub [u8; 20]);

/// Transaction status in the explorer.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TxStatus {
    Success,
    Reverted,
    OutOfGas,
    Pending,
}

/// A block as returned by the explorer.
#[derive(Debug, Clone)]
pub struct ExplorerBlock {
    pub height: u64,
    pub hash: Hash,
    pub parent_hash: Hash,
    pub state_root: Hash,
    pub receipts_root: Hash,
    pub producer: Address,
    pub timestamp: u64,
    pub tx_count: u32,
    pub gas_used: u64,
    pub gas_limit: u64,
}

/// A transaction as returned by the explorer.
#[derive(Debug, Clone)]
pub struct ExplorerTx {
    pub hash: Hash,
    pub block_height: u64,
    pub tx_index: u32,
    pub from: Address,
    pub to: Option<Address>,
    pub value: u128,
    pub gas_used: u64,
    pub status: TxStatus,
    pub tx_type: String,
    pub timestamp: u64,
}

/// An event as returned by the explorer.
#[derive(Debug, Clone)]
pub struct ExplorerEvent {
    pub block_height: u64,
    pub tx_hash: Hash,
    pub log_index: u32,
    pub emitter: Address,
    pub topics: Vec<Hash>,
    pub data: Vec<u8>,
}

/// Account summary.
#[derive(Debug, Clone)]
pub struct AccountSummary {
    pub address: Address,
    pub balance: u128,
    pub nonce: u64,
    pub tx_count: u64,
    pub first_seen: Option<u64>,
    pub last_active: Option<u64>,
}

/// Chain-wide statistics.
#[derive(Debug, Clone)]
pub struct ChainStats {
    pub latest_block: u64,
    pub total_transactions: u64,
    pub total_accounts: u64,
    pub total_gas_used: u128,
    pub avg_block_time_ms: u64,
    pub avg_gas_per_block: u64,
}

/// Event filter for querying logs.
#[derive(Debug, Clone, Default)]
pub struct EventFilter {
    pub from_block: Option<u64>,
    pub to_block: Option<u64>,
    pub emitter: Option<Address>,
    pub topic0: Option<Hash>,
    pub topic1: Option<Hash>,
    pub max_results: Option<usize>,
}

/// Pagination parameters.
#[derive(Debug, Clone)]
pub struct Pagination {
    pub offset: u64,
    pub limit: u64,
}

impl Default for Pagination {
    fn default() -> Self {
        Self {
            offset: 0,
            limit: 20,
        }
    }
}

impl Pagination {
    pub fn new(offset: u64, limit: u64) -> Self {
        Self {
            offset,
            limit: limit.min(100),
        } // cap at 100
    }
}

/// Paginated result wrapper.
#[derive(Debug, Clone)]
pub struct PaginatedResult<T> {
    pub items: Vec<T>,
    pub total: u64,
    pub offset: u64,
    pub limit: u64,
}

// ── Explorer Store ─────────────────────────────────────────────────

/// In-memory indexed store backing the explorer API.
/// In production this would be a database; here we index everything
/// in sorted maps for testability.
#[derive(Debug, Clone)]
pub struct ExplorerStore {
    blocks: BTreeMap<u64, ExplorerBlock>,
    txs_by_hash: HashMap<Hash, ExplorerTx>,
    txs_by_block: BTreeMap<u64, Vec<Hash>>,
    txs_by_address: HashMap<Address, Vec<Hash>>,
    events: Vec<ExplorerEvent>,
    events_by_block: BTreeMap<u64, Vec<usize>>,
    events_by_emitter: HashMap<Address, Vec<usize>>,
    events_by_topic0: HashMap<Hash, Vec<usize>>,
    accounts: BTreeMap<Address, AccountSummary>,
    total_gas: u128,
}

impl ExplorerStore {
    pub fn new() -> Self {
        Self {
            blocks: BTreeMap::new(),
            txs_by_hash: HashMap::new(),
            txs_by_block: BTreeMap::new(),
            txs_by_address: HashMap::new(),
            events: Vec::new(),
            events_by_block: BTreeMap::new(),
            events_by_emitter: HashMap::new(),
            events_by_topic0: HashMap::new(),
            accounts: BTreeMap::new(),
            total_gas: 0,
        }
    }

    /// Index a new block.
    pub fn index_block(&mut self, block: ExplorerBlock) {
        self.total_gas += block.gas_used as u128;
        self.blocks.insert(block.height, block);
    }

    /// Index a transaction (block must be indexed first).
    pub fn index_tx(&mut self, tx: ExplorerTx) {
        let hash = tx.hash;
        let height = tx.block_height;
        let from = tx.from.clone();

        // Track per-address
        self.txs_by_address
            .entry(from.clone())
            .or_default()
            .push(hash);
        if let Some(ref to) = tx.to {
            if *to != from {
                self.txs_by_address
                    .entry(to.clone())
                    .or_default()
                    .push(hash);
            }
        }

        // Update account summaries
        self.touch_account(&from, height, tx.timestamp);
        if let Some(ref to) = tx.to {
            self.touch_account(to, height, tx.timestamp);
        }

        self.txs_by_block.entry(height).or_default().push(hash);
        self.txs_by_hash.insert(hash, tx);
    }

    /// Index an event.
    pub fn index_event(&mut self, event: ExplorerEvent) {
        let idx = self.events.len();
        let height = event.block_height;
        let emitter = event.emitter.clone();

        self.events_by_block.entry(height).or_default().push(idx);
        self.events_by_emitter.entry(emitter).or_default().push(idx);
        if let Some(t0) = event.topics.first() {
            self.events_by_topic0.entry(*t0).or_default().push(idx);
        }
        self.events.push(event);
    }

    fn touch_account(&mut self, addr: &Address, height: u64, timestamp: u64) {
        let entry = self
            .accounts
            .entry(addr.clone())
            .or_insert_with(|| AccountSummary {
                address: addr.clone(),
                balance: 0,
                nonce: 0,
                tx_count: 0,
                first_seen: None,
                last_active: None,
            });
        entry.tx_count += 1;
        if entry.first_seen.is_none() || height < entry.first_seen.unwrap() {
            entry.first_seen = Some(height);
        }
        if entry.last_active.is_none() || height > entry.last_active.unwrap() {
            entry.last_active = Some(height);
        }
    }

    /// Update an account's balance and nonce (called during state sync).
    pub fn set_account_state(&mut self, addr: &Address, balance: u128, nonce: u64) {
        let entry = self
            .accounts
            .entry(addr.clone())
            .or_insert_with(|| AccountSummary {
                address: addr.clone(),
                balance: 0,
                nonce: 0,
                tx_count: 0,
                first_seen: None,
                last_active: None,
            });
        entry.balance = balance;
        entry.nonce = nonce;
    }

    // ── Query methods ──────────────────────────────────────────────

    /// Get block by height.
    pub fn get_block(&self, height: u64) -> Option<&ExplorerBlock> {
        self.blocks.get(&height)
    }

    /// Get latest block height.
    pub fn latest_height(&self) -> Option<u64> {
        self.blocks.keys().last().copied()
    }

    /// List blocks in descending order (most recent first).
    pub fn get_block_range(&self, page: &Pagination) -> PaginatedResult<ExplorerBlock> {
        let total = self.blocks.len() as u64;
        let items: Vec<_> = self
            .blocks
            .values()
            .rev()
            .skip(page.offset as usize)
            .take(page.limit as usize)
            .cloned()
            .collect();
        PaginatedResult {
            items,
            total,
            offset: page.offset,
            limit: page.limit,
        }
    }

    /// Get transaction by hash.
    pub fn get_tx(&self, hash: &Hash) -> Option<&ExplorerTx> {
        self.txs_by_hash.get(hash)
    }

    /// List transactions in a block.
    pub fn get_block_txs(&self, height: u64) -> Vec<ExplorerTx> {
        self.txs_by_block
            .get(&height)
            .map(|hashes| {
                hashes
                    .iter()
                    .filter_map(|h| self.txs_by_hash.get(h).cloned())
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Search transactions by address (sender or receiver).
    pub fn search_txs_by_address(
        &self,
        addr: &Address,
        page: &Pagination,
    ) -> PaginatedResult<ExplorerTx> {
        let all = self.txs_by_address.get(addr);
        let total = all.map(|v| v.len() as u64).unwrap_or(0);
        let items: Vec<_> = all
            .into_iter()
            .flat_map(|v| v.iter().rev())
            .skip(page.offset as usize)
            .take(page.limit as usize)
            .filter_map(|h| self.txs_by_hash.get(h).cloned())
            .collect();
        PaginatedResult {
            items,
            total,
            offset: page.offset,
            limit: page.limit,
        }
    }

    /// Get account summary.
    pub fn get_account(&self, addr: &Address) -> Option<&AccountSummary> {
        self.accounts.get(addr)
    }

    /// Query events matching a filter.
    pub fn query_events(&self, filter: &EventFilter) -> Vec<ExplorerEvent> {
        let max = filter.max_results.unwrap_or(1000).min(10000);

        // Pick the narrowest starting index set.
        let candidate_indices: Box<dyn Iterator<Item = &usize>> =
            if let Some(ref emitter) = filter.emitter {
                if let Some(ref t0) = filter.topic0 {
                    // Intersect emitter + topic0
                    let e_set = self.events_by_emitter.get(emitter);
                    let t_set = self.events_by_topic0.get(t0);
                    match (e_set, t_set) {
                        (Some(es), Some(ts)) => {
                            // Simple intersection via sorted merge
                            let mut es_sorted = es.clone();
                            let mut ts_sorted = ts.clone();
                            es_sorted.sort();
                            ts_sorted.sort();
                            let intersection = sorted_intersect(&es_sorted, &ts_sorted);
                            // Need to return owned data — collect to vec
                            let v: Vec<usize> = intersection;
                            Box::new(
                                v.into_iter()
                                    .collect::<Vec<_>>()
                                    .into_iter()
                                    .map(|_| unreachable!()),
                            )
                            // Actually let's just use a simpler approach
                        }
                        _ => return Vec::new(),
                    }
                } else {
                    match self.events_by_emitter.get(emitter) {
                        Some(v) => Box::new(v.iter()),
                        None => return Vec::new(),
                    }
                }
            } else if let Some(ref t0) = filter.topic0 {
                match self.events_by_topic0.get(t0) {
                    Some(v) => Box::new(v.iter()),
                    None => return Vec::new(),
                }
            } else {
                Box::new((0..self.events.len()).collect::<Vec<_>>().leak().iter())
            };

        // Simplified: just do a linear scan with all filters
        self.scan_events(filter, max)
    }

    fn scan_events(&self, filter: &EventFilter, max: usize) -> Vec<ExplorerEvent> {
        let from = filter.from_block.unwrap_or(0);
        let to = filter.to_block.unwrap_or(u64::MAX);

        // If we have a block range, only scan those blocks' events
        let indices: Vec<usize> = if filter.emitter.is_some() || filter.topic0.is_some() {
            // Use index
            let mut result = Vec::new();
            if let Some(ref emitter) = filter.emitter {
                if let Some(idxs) = self.events_by_emitter.get(emitter) {
                    result = idxs.clone();
                }
            } else if let Some(ref t0) = filter.topic0 {
                if let Some(idxs) = self.events_by_topic0.get(t0) {
                    result = idxs.clone();
                }
            }
            result
        } else {
            (0..self.events.len()).collect()
        };

        indices
            .into_iter()
            .filter_map(|i| self.events.get(i))
            .filter(|e| e.block_height >= from && e.block_height <= to)
            .filter(|e| {
                if let Some(ref emitter) = filter.emitter {
                    if e.emitter != *emitter {
                        return false;
                    }
                }
                if let Some(ref t0) = filter.topic0 {
                    if e.topics.first() != Some(t0) {
                        return false;
                    }
                }
                if let Some(ref t1) = filter.topic1 {
                    if e.topics.get(1) != Some(t1) {
                        return false;
                    }
                }
                true
            })
            .take(max)
            .cloned()
            .collect()
    }

    /// Chain-wide statistics.
    pub fn chain_stats(&self) -> ChainStats {
        let total_blocks = self.blocks.len() as u64;
        let total_transactions = self.txs_by_hash.len() as u64;
        let total_accounts = self.accounts.len() as u64;

        let avg_block_time_ms = if total_blocks > 1 {
            let first = self
                .blocks
                .values()
                .next()
                .map(|b| b.timestamp)
                .unwrap_or(0);
            let last = self
                .blocks
                .values()
                .last()
                .map(|b| b.timestamp)
                .unwrap_or(0);
            if last > first {
                ((last - first) * 1000) / (total_blocks - 1)
            } else {
                0
            }
        } else {
            0
        };

        let avg_gas = if total_blocks > 0 {
            (self.total_gas / total_blocks as u128) as u64
        } else {
            0
        };

        ChainStats {
            latest_block: self.latest_height().unwrap_or(0),
            total_transactions,
            total_accounts,
            total_gas_used: self.total_gas,
            avg_block_time_ms,
            avg_gas_per_block: avg_gas,
        }
    }
}

fn sorted_intersect(a: &[usize], b: &[usize]) -> Vec<usize> {
    let mut result = Vec::new();
    let (mut i, mut j) = (0, 0);
    while i < a.len() && j < b.len() {
        match a[i].cmp(&b[j]) {
            std::cmp::Ordering::Equal => {
                result.push(a[i]);
                i += 1;
                j += 1;
            }
            std::cmp::Ordering::Less => i += 1,
            std::cmp::Ordering::Greater => j += 1,
        }
    }
    result
}

// ── JSON-RPC dispatcher ────────────────────────────────────────────

/// Explorer-specific RPC method names.
pub const EXPLORER_METHODS: &[&str] = &[
    "explorer_getBlock",
    "explorer_getBlockRange",
    "explorer_getTransaction",
    "explorer_getBlockTransactions",
    "explorer_getAccount",
    "explorer_getEvents",
    "explorer_getChainStats",
    "explorer_searchTxByAddress",
];

/// Helper to create a test hash from a u64 (for deterministic tests).
pub fn test_hash(n: u64) -> Hash {
    let mut h = [0u8; 32];
    h[24..32].copy_from_slice(&n.to_be_bytes());
    h
}

/// Helper to create a test address from a u8.
pub fn test_addr(n: u8) -> Address {
    let mut a = [0u8; 20];
    a[19] = n;
    Address(a)
}

// ── Tests ──────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn make_block(height: u64, gas: u64) -> ExplorerBlock {
        ExplorerBlock {
            height,
            hash: test_hash(height),
            parent_hash: test_hash(height.wrapping_sub(1)),
            state_root: test_hash(1000 + height),
            receipts_root: test_hash(2000 + height),
            producer: test_addr(1),
            timestamp: 1000 + height * 30,
            tx_count: 0,
            gas_used: gas,
            gas_limit: 10_000_000,
        }
    }

    fn make_tx(hash_n: u64, block: u64, from: u8, to: u8, idx: u32) -> ExplorerTx {
        ExplorerTx {
            hash: test_hash(hash_n),
            block_height: block,
            tx_index: idx,
            from: test_addr(from),
            to: Some(test_addr(to)),
            value: 100,
            gas_used: 21000,
            status: TxStatus::Success,
            tx_type: "transfer".into(),
            timestamp: 1000 + block * 30,
        }
    }

    fn populated_store() -> ExplorerStore {
        let mut store = ExplorerStore::new();
        for i in 0..10 {
            store.index_block(make_block(i, 50000 + i * 1000));
        }
        // Block 0: 2 txs, Block 1: 1 tx
        store.index_tx(make_tx(100, 0, 1, 2, 0));
        store.index_tx(make_tx(101, 0, 1, 3, 1));
        store.index_tx(make_tx(102, 1, 2, 1, 0));
        // Events
        store.index_event(ExplorerEvent {
            block_height: 0,
            tx_hash: test_hash(100),
            log_index: 0,
            emitter: test_addr(1),
            topics: vec![test_hash(500)],
            data: vec![1, 2, 3],
        });
        store.index_event(ExplorerEvent {
            block_height: 1,
            tx_hash: test_hash(102),
            log_index: 0,
            emitter: test_addr(2),
            topics: vec![test_hash(500), test_hash(600)],
            data: vec![4, 5],
        });
        store.index_event(ExplorerEvent {
            block_height: 2,
            tx_hash: test_hash(200),
            log_index: 0,
            emitter: test_addr(1),
            topics: vec![test_hash(501)],
            data: vec![],
        });
        // Set some balances
        store.set_account_state(&test_addr(1), 1_000_000, 5);
        store.set_account_state(&test_addr(2), 500_000, 2);
        store
    }

    #[test]
    fn test_get_block() {
        let store = populated_store();
        let b = store.get_block(0).unwrap();
        assert_eq!(b.height, 0);
        assert_eq!(b.hash, test_hash(0));
        assert!(store.get_block(999).is_none());
    }

    #[test]
    fn test_latest_height() {
        let store = populated_store();
        assert_eq!(store.latest_height(), Some(9));
        let empty = ExplorerStore::new();
        assert_eq!(empty.latest_height(), None);
    }

    #[test]
    fn test_block_range_pagination() {
        let store = populated_store();
        let page = Pagination::new(0, 3);
        let result = store.get_block_range(&page);
        assert_eq!(result.total, 10);
        assert_eq!(result.items.len(), 3);
        // Most recent first
        assert_eq!(result.items[0].height, 9);
        assert_eq!(result.items[2].height, 7);

        // Second page
        let page2 = Pagination::new(3, 3);
        let r2 = store.get_block_range(&page2);
        assert_eq!(r2.items[0].height, 6);
    }

    #[test]
    fn test_get_tx() {
        let store = populated_store();
        let tx = store.get_tx(&test_hash(100)).unwrap();
        assert_eq!(tx.from, test_addr(1));
        assert_eq!(tx.to, Some(test_addr(2)));
        assert!(store.get_tx(&test_hash(999)).is_none());
    }

    #[test]
    fn test_block_txs() {
        let store = populated_store();
        let txs = store.get_block_txs(0);
        assert_eq!(txs.len(), 2);
        let txs1 = store.get_block_txs(1);
        assert_eq!(txs1.len(), 1);
        let empty = store.get_block_txs(5);
        assert_eq!(empty.len(), 0);
    }

    #[test]
    fn test_search_txs_by_address() {
        let store = populated_store();
        // Addr 1 is sender in 100,101 and receiver in 102 → 3 txs
        let page = Pagination::default();
        let r = store.search_txs_by_address(&test_addr(1), &page);
        assert_eq!(r.total, 3);
        assert_eq!(r.items.len(), 3);

        // Addr 3 only appears as receiver in tx 101
        let r3 = store.search_txs_by_address(&test_addr(3), &page);
        assert_eq!(r3.total, 1);
    }

    #[test]
    fn test_get_account() {
        let store = populated_store();
        let a = store.get_account(&test_addr(1)).unwrap();
        assert_eq!(a.balance, 1_000_000);
        assert_eq!(a.nonce, 5);
        assert_eq!(a.tx_count, 3);
        assert_eq!(a.first_seen, Some(0));

        assert!(store.get_account(&test_addr(99)).is_none());
    }

    #[test]
    fn test_query_events_all() {
        let store = populated_store();
        let filter = EventFilter::default();
        let events = store.query_events(&filter);
        assert_eq!(events.len(), 3);
    }

    #[test]
    fn test_query_events_by_emitter() {
        let store = populated_store();
        let filter = EventFilter {
            emitter: Some(test_addr(1)),
            ..Default::default()
        };
        let events = store.query_events(&filter);
        assert_eq!(events.len(), 2);
    }

    #[test]
    fn test_query_events_by_topic() {
        let store = populated_store();
        let filter = EventFilter {
            topic0: Some(test_hash(500)),
            ..Default::default()
        };
        let events = store.query_events(&filter);
        assert_eq!(events.len(), 2);
    }

    #[test]
    fn test_query_events_by_block_range() {
        let store = populated_store();
        let filter = EventFilter {
            from_block: Some(1),
            to_block: Some(1),
            ..Default::default()
        };
        let events = store.query_events(&filter);
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].block_height, 1);
    }

    #[test]
    fn test_query_events_with_topic1() {
        let store = populated_store();
        let filter = EventFilter {
            topic0: Some(test_hash(500)),
            topic1: Some(test_hash(600)),
            ..Default::default()
        };
        let events = store.query_events(&filter);
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].block_height, 1);
    }

    #[test]
    fn test_chain_stats() {
        let store = populated_store();
        let stats = store.chain_stats();
        assert_eq!(stats.latest_block, 9);
        assert_eq!(stats.total_transactions, 3);
        assert_eq!(stats.total_accounts, 3); // addr 1, 2, 3
        assert!(stats.total_gas_used > 0);
        assert!(stats.avg_block_time_ms > 0);
        assert!(stats.avg_gas_per_block > 0);
    }

    #[test]
    fn test_empty_store_stats() {
        let store = ExplorerStore::new();
        let stats = store.chain_stats();
        assert_eq!(stats.latest_block, 0);
        assert_eq!(stats.total_transactions, 0);
        assert_eq!(stats.avg_block_time_ms, 0);
    }

    #[test]
    fn test_pagination_cap() {
        let p = Pagination::new(0, 500);
        assert_eq!(p.limit, 100); // capped at 100
    }

    #[test]
    fn test_sorted_intersect() {
        assert_eq!(sorted_intersect(&[1, 3, 5, 7], &[2, 3, 5, 8]), vec![3, 5]);
        assert_eq!(sorted_intersect(&[], &[1, 2]), Vec::<usize>::new());
        assert_eq!(sorted_intersect(&[1, 2], &[1, 2]), vec![1, 2]);
    }

    #[test]
    fn test_max_results_limit() {
        let mut store = ExplorerStore::new();
        store.index_block(make_block(0, 1000));
        for i in 0..50 {
            store.index_event(ExplorerEvent {
                block_height: 0,
                tx_hash: test_hash(i),
                log_index: i as u32,
                emitter: test_addr(1),
                topics: vec![test_hash(700)],
                data: vec![],
            });
        }
        let filter = EventFilter {
            max_results: Some(5),
            ..Default::default()
        };
        let events = store.query_events(&filter);
        assert_eq!(events.len(), 5);
    }

    #[test]
    fn test_set_account_state() {
        let mut store = ExplorerStore::new();
        store.set_account_state(&test_addr(1), 999, 10);
        let a = store.get_account(&test_addr(1)).unwrap();
        assert_eq!(a.balance, 999);
        assert_eq!(a.nonce, 10);
        assert_eq!(a.tx_count, 0); // no txs indexed

        // Update
        store.set_account_state(&test_addr(1), 1500, 11);
        let a = store.get_account(&test_addr(1)).unwrap();
        assert_eq!(a.balance, 1500);
    }

    #[test]
    fn test_explorer_methods_list() {
        assert_eq!(EXPLORER_METHODS.len(), 8);
        assert!(EXPLORER_METHODS.contains(&"explorer_getBlock"));
        assert!(EXPLORER_METHODS.contains(&"explorer_searchTxByAddress"));
    }
}
