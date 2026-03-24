//! Event log system — structured on-chain event emission and indexing.
//!
//! Every state-changing operation emits typed events that are:
//! - Included in block receipts (Merkle root of events per block)
//! - Filterable by topic (event type + indexed fields)
//! - Queryable by block range, address, or topic
//!
//! Design inspired by Ethereum logs but simplified for Prova's domain:
//! - Topic 0: event type hash (SHA-256 of event signature)
//! - Topics 1-3: indexed fields (address, model ID, commit ID, etc.)
//! - Data: ABI-encoded payload (non-indexed fields)

use std::collections::{BTreeMap, HashMap};

use crate::types::{Address, Epoch, Hash};

/// Maximum topics per event (type hash + up to 3 indexed fields).
pub const MAX_TOPICS: usize = 4;

/// Maximum data payload size in bytes.
pub const MAX_DATA_SIZE: usize = 8192;

/// Well-known event type hashes (pre-computed SHA-256 of signature strings).
pub mod event_types {
    use super::*;

    pub fn hash_signature(sig: &str) -> Hash {
        use sha2::{Digest, Sha256};
        let mut hasher = Sha256::new();
        hasher.update(sig.as_bytes());
        let result = hasher.finalize();
        let mut h = [0u8; 32];
        h.copy_from_slice(&result);
        h
    }

    lazy_static_hashes! {
        TRANSFER => "Transfer(address,address,uint128)",
        STAKE_DEPOSITED => "StakeDeposited(address,uint128)",
        STAKE_WITHDRAWN => "StakeWithdrawn(address,uint128)",
        SLASH => "Slash(address,uint128,bytes32)",
        INFERENCE_COMMITTED => "InferenceCommitted(uint64,address,bytes32)",
        CHALLENGE_OPENED => "ChallengeOpened(uint64,address,address)",
        CHALLENGE_RESOLVED => "ChallengeResolved(uint64,bool)",
        MODEL_REGISTERED => "ModelRegistered(bytes32,address)",
        BLOCK_REWARD => "BlockReward(address,uint128)",
        PAYMENT_OPENED => "PaymentOpened(address,address,uint128)",
        PAYMENT_SETTLED => "PaymentSettled(address,address,uint128)",
        GOVERNANCE_PROPOSAL => "GovernanceProposal(uint64,address)",
        GOVERNANCE_VOTE => "GovernanceVote(uint64,address,bool)",
        JOB_SUBMITTED => "JobSubmitted(uint64,address,bytes32)",
        JOB_COMPLETED => "JobCompleted(uint64,address)",
        CHECKPOINT_ANCHORED => "CheckpointAnchored(uint64,bytes32)",
        UPGRADE_SCHEDULED => "UpgradeScheduled(uint64,uint64)",
    }
}

/// Macro to generate lazy-computed event type hashes.
macro_rules! lazy_static_hashes {
    ($($name:ident => $sig:expr),+ $(,)?) => {
        $(
            pub fn $name() -> Hash {
                hash_signature($sig)
            }
        )+
    };
}
use lazy_static_hashes;

/// A single emitted event (log entry).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Event {
    /// Address that emitted the event.
    pub emitter: Address,
    /// Topics (topic[0] = event type hash, rest = indexed fields).
    pub topics: Vec<Hash>,
    /// Non-indexed data payload.
    pub data: Vec<u8>,
    /// Block in which this event was emitted.
    pub block_number: Epoch,
    /// Index within the block's event list.
    pub log_index: u32,
    /// Transaction index within the block (if applicable).
    pub tx_index: u32,
}

/// Filter for querying events.
#[derive(Debug, Clone, Default)]
pub struct EventFilter {
    /// Block range (inclusive).
    pub from_block: Option<Epoch>,
    pub to_block: Option<Epoch>,
    /// Filter by emitter address.
    pub address: Option<Address>,
    /// Topic filters: position → expected value. None = wildcard.
    pub topics: [Option<Hash>; MAX_TOPICS],
    /// Maximum results.
    pub limit: Option<usize>,
}

impl EventFilter {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn from_block(mut self, b: Epoch) -> Self {
        self.from_block = Some(b);
        self
    }

    pub fn to_block(mut self, b: Epoch) -> Self {
        self.to_block = Some(b);
        self
    }

    pub fn address(mut self, a: Address) -> Self {
        self.address = Some(a);
        self
    }

    pub fn topic(mut self, index: usize, hash: Hash) -> Self {
        if index < MAX_TOPICS {
            self.topics[index] = Some(hash);
        }
        self
    }

    pub fn limit(mut self, n: usize) -> Self {
        self.limit = Some(n);
        self
    }

    fn matches(&self, event: &Event) -> bool {
        if let Some(addr) = &self.address {
            if &event.emitter != addr {
                return false;
            }
        }
        for (i, topic_filter) in self.topics.iter().enumerate() {
            if let Some(expected) = topic_filter {
                match event.topics.get(i) {
                    Some(actual) if actual == expected => {}
                    _ => return false,
                }
            }
        }
        true
    }
}

/// Block receipt: events + Merkle root for a single block.
#[derive(Debug, Clone)]
pub struct BlockReceipt {
    pub block_number: Epoch,
    pub events: Vec<Event>,
    pub events_root: Hash,
}

impl BlockReceipt {
    /// Compute Merkle root of events in this block.
    pub fn compute_root(events: &[Event]) -> Hash {
        use sha2::{Digest, Sha256};

        if events.is_empty() {
            return [0u8; 32];
        }

        // Leaf hashes
        let mut hashes: Vec<Hash> = events
            .iter()
            .map(|e| {
                let mut hasher = Sha256::new();
                hasher.update(&e.emitter.0);
                for t in &e.topics {
                    hasher.update(t);
                }
                hasher.update(&e.data);
                hasher.update(&e.block_number.to_be_bytes());
                hasher.update(&e.log_index.to_be_bytes());
                let result = hasher.finalize();
                let mut h = [0u8; 32];
                h.copy_from_slice(&result);
                h
            })
            .collect();

        // Binary Merkle tree
        while hashes.len() > 1 {
            let mut next = Vec::with_capacity((hashes.len() + 1) / 2);
            for chunk in hashes.chunks(2) {
                let mut hasher = Sha256::new();
                hasher.update(&chunk[0]);
                if chunk.len() > 1 {
                    hasher.update(&chunk[1]);
                } else {
                    hasher.update(&chunk[0]); // duplicate odd leaf
                }
                let result = hasher.finalize();
                let mut h = [0u8; 32];
                h.copy_from_slice(&result);
                next.push(h);
            }
            hashes = next;
        }

        hashes[0]
    }
}

/// Event store: append-only log indexed by block, address, and topic.
#[derive(Debug, Default)]
pub struct EventStore {
    /// All events, ordered by (block_number, log_index).
    events: Vec<Event>,
    /// Block → range of indices into `events`.
    block_index: BTreeMap<Epoch, (usize, usize)>,
    /// Address → list of event indices.
    address_index: HashMap<Address, Vec<usize>>,
    /// Topic[0] (event type) → list of event indices.
    type_index: HashMap<Hash, Vec<usize>>,
    /// Block receipts (block_number → events_root).
    receipts: BTreeMap<Epoch, Hash>,
    /// Next log index per block.
    next_log_index: HashMap<Epoch, u32>,
}

impl EventStore {
    pub fn new() -> Self {
        Self::default()
    }

    /// Emit an event. Returns the log index.
    pub fn emit(
        &mut self,
        emitter: Address,
        topics: Vec<Hash>,
        data: Vec<u8>,
        block_number: Epoch,
        tx_index: u32,
    ) -> Result<u32, EventError> {
        if topics.len() > MAX_TOPICS {
            return Err(EventError::TooManyTopics(topics.len()));
        }
        if data.len() > MAX_DATA_SIZE {
            return Err(EventError::DataTooLarge(data.len()));
        }

        let log_index = self.next_log_index.entry(block_number).or_insert(0);
        let idx = *log_index;
        *log_index += 1;

        let event = Event {
            emitter,
            topics: topics.clone(),
            data,
            block_number,
            log_index: idx,
            tx_index,
        };

        let store_idx = self.events.len();
        self.events.push(event);

        // Update block index
        let entry = self
            .block_index
            .entry(block_number)
            .or_insert((store_idx, store_idx));
        entry.1 = store_idx + 1;

        // Update address index
        self.address_index
            .entry(emitter)
            .or_default()
            .push(store_idx);

        // Update type index (topic[0])
        if let Some(&type_hash) = topics.first() {
            self.type_index
                .entry(type_hash)
                .or_default()
                .push(store_idx);
        }

        Ok(idx)
    }

    /// Finalize a block: compute and store the events root.
    pub fn finalize_block(&mut self, block_number: Epoch) -> Hash {
        let events: Vec<Event> = match self.block_index.get(&block_number) {
            Some(&(start, end)) => self.events[start..end].to_vec(),
            None => vec![],
        };
        let root = BlockReceipt::compute_root(&events);
        self.receipts.insert(block_number, root);
        root
    }

    /// Get the events root for a finalized block.
    pub fn events_root(&self, block_number: Epoch) -> Option<Hash> {
        self.receipts.get(&block_number).copied()
    }

    /// Query events matching a filter.
    pub fn query(&self, filter: &EventFilter) -> Vec<&Event> {
        let from = filter.from_block.unwrap_or(0);
        let to = filter.to_block.unwrap_or(u64::MAX);
        let limit = filter.limit.unwrap_or(usize::MAX);

        // Use the most selective index
        let candidates: Box<dyn Iterator<Item = usize>> = if let Some(addr) = &filter.address {
            if let Some(indices) = self.address_index.get(addr) {
                Box::new(indices.iter().copied())
            } else {
                return vec![];
            }
        } else if let Some(Some(type_hash)) = filter.topics.first() {
            if let Some(indices) = self.type_index.get(type_hash) {
                Box::new(indices.iter().copied())
            } else {
                return vec![];
            }
        } else {
            // Full scan
            let range_start = self
                .block_index
                .range(from..)
                .next()
                .map(|(_, &(s, _))| s)
                .unwrap_or(self.events.len());
            Box::new(range_start..self.events.len())
        };

        candidates
            .filter_map(|i| {
                let e = &self.events[i];
                if e.block_number >= from && e.block_number <= to && filter.matches(e) {
                    Some(e)
                } else {
                    None
                }
            })
            .take(limit)
            .collect()
    }

    /// Get all events in a specific block.
    pub fn block_events(&self, block_number: Epoch) -> &[Event] {
        match self.block_index.get(&block_number) {
            Some(&(start, end)) => &self.events[start..end],
            None => &[],
        }
    }

    /// Total event count.
    pub fn len(&self) -> usize {
        self.events.len()
    }

    pub fn is_empty(&self) -> bool {
        self.events.is_empty()
    }

    /// Number of finalized blocks with receipts.
    pub fn finalized_blocks(&self) -> usize {
        self.receipts.len()
    }
}

/// Errors from event emission.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EventError {
    TooManyTopics(usize),
    DataTooLarge(usize),
}

impl std::fmt::Display for EventError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::TooManyTopics(n) => write!(f, "too many topics: {n} (max {MAX_TOPICS})"),
            Self::DataTooLarge(n) => write!(f, "data too large: {n} bytes (max {MAX_DATA_SIZE})"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::Address;

    fn addr(id: u8) -> Address {
        Address::test(id)
    }

    fn topic(b: u8) -> Hash {
        let mut h = [0u8; 32];
        h[0] = b;
        h
    }

    #[test]
    fn test_emit_and_query_basic() {
        let mut store = EventStore::new();
        let t = event_types::TRANSFER();
        store.emit(addr(1), vec![t], vec![1, 2, 3], 100, 0).unwrap();
        store.emit(addr(2), vec![t], vec![4, 5, 6], 100, 1).unwrap();

        let all = store.query(&EventFilter::new());
        assert_eq!(all.len(), 2);

        let by_addr = store.query(&EventFilter::new().address(addr(1)));
        assert_eq!(by_addr.len(), 1);
        assert_eq!(by_addr[0].data, vec![1, 2, 3]);
    }

    #[test]
    fn test_block_range_filter() {
        let mut store = EventStore::new();
        let t = event_types::TRANSFER();
        store.emit(addr(1), vec![t], vec![], 10, 0).unwrap();
        store.emit(addr(1), vec![t], vec![], 20, 0).unwrap();
        store.emit(addr(1), vec![t], vec![], 30, 0).unwrap();

        let filtered = store.query(&EventFilter::new().from_block(15).to_block(25));
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].block_number, 20);
    }

    #[test]
    fn test_topic_filter() {
        let mut store = EventStore::new();
        let t1 = event_types::TRANSFER();
        let t2 = event_types::SLASH();
        store
            .emit(addr(1), vec![t1, topic(0xAA)], vec![], 1, 0)
            .unwrap();
        store
            .emit(addr(1), vec![t2, topic(0xBB)], vec![], 1, 1)
            .unwrap();
        store
            .emit(addr(1), vec![t1, topic(0xCC)], vec![], 1, 2)
            .unwrap();

        // Filter by event type
        let transfers = store.query(&EventFilter::new().topic(0, t1));
        assert_eq!(transfers.len(), 2);

        // Filter by event type + indexed field
        let specific = store.query(&EventFilter::new().topic(0, t1).topic(1, topic(0xAA)));
        assert_eq!(specific.len(), 1);
    }

    #[test]
    fn test_block_receipt_merkle_root() {
        let mut store = EventStore::new();
        let t = event_types::STAKE_DEPOSITED();
        store
            .emit(addr(1), vec![t], 100u128.to_be_bytes().to_vec(), 1, 0)
            .unwrap();
        store
            .emit(addr(2), vec![t], 200u128.to_be_bytes().to_vec(), 1, 1)
            .unwrap();

        let root = store.finalize_block(1);
        assert_ne!(root, [0u8; 32]);

        // Same events → same root (deterministic)
        let root2 = BlockReceipt::compute_root(store.block_events(1));
        assert_eq!(root, root2);

        // Stored root matches
        assert_eq!(store.events_root(1), Some(root));
    }

    #[test]
    fn test_empty_block_root() {
        let mut store = EventStore::new();
        let root = store.finalize_block(999);
        assert_eq!(root, [0u8; 32]);
    }

    #[test]
    fn test_merkle_root_different_for_different_events() {
        let t = event_types::TRANSFER();
        let e1 = Event {
            emitter: addr(1),
            topics: vec![t],
            data: vec![1],
            block_number: 1,
            log_index: 0,
            tx_index: 0,
        };
        let e2 = Event {
            emitter: addr(2),
            topics: vec![t],
            data: vec![2],
            block_number: 1,
            log_index: 1,
            tx_index: 1,
        };

        let root_a = BlockReceipt::compute_root(&[e1.clone()]);
        let root_b = BlockReceipt::compute_root(&[e2.clone()]);
        let root_ab = BlockReceipt::compute_root(&[e1, e2]);

        assert_ne!(root_a, root_b);
        assert_ne!(root_a, root_ab);
        assert_ne!(root_b, root_ab);
    }

    #[test]
    fn test_error_too_many_topics() {
        let mut store = EventStore::new();
        let topics = vec![[0u8; 32]; 5]; // 5 > MAX_TOPICS (4)
        let result = store.emit(addr(1), topics, vec![], 1, 0);
        assert_eq!(result, Err(EventError::TooManyTopics(5)));
    }

    #[test]
    fn test_error_data_too_large() {
        let mut store = EventStore::new();
        let data = vec![0u8; MAX_DATA_SIZE + 1];
        let result = store.emit(addr(1), vec![topic(1)], data, 1, 0);
        assert_eq!(result, Err(EventError::DataTooLarge(MAX_DATA_SIZE + 1)));
    }

    #[test]
    fn test_limit_filter() {
        let mut store = EventStore::new();
        let t = event_types::TRANSFER();
        for i in 0..10 {
            store.emit(addr(1), vec![t], vec![i], 1, i as u32).unwrap();
        }
        let limited = store.query(&EventFilter::new().limit(3));
        assert_eq!(limited.len(), 3);
    }

    #[test]
    fn test_block_events() {
        let mut store = EventStore::new();
        let t = event_types::TRANSFER();
        store.emit(addr(1), vec![t], vec![], 10, 0).unwrap();
        store.emit(addr(2), vec![t], vec![], 10, 1).unwrap();
        store.emit(addr(3), vec![t], vec![], 20, 0).unwrap();

        assert_eq!(store.block_events(10).len(), 2);
        assert_eq!(store.block_events(20).len(), 1);
        assert_eq!(store.block_events(30).len(), 0);
    }

    #[test]
    fn test_event_type_hashes_unique() {
        let hashes = vec![
            event_types::TRANSFER(),
            event_types::SLASH(),
            event_types::STAKE_DEPOSITED(),
            event_types::INFERENCE_COMMITTED(),
            event_types::CHALLENGE_OPENED(),
            event_types::BLOCK_REWARD(),
            event_types::JOB_SUBMITTED(),
            event_types::CHECKPOINT_ANCHORED(),
        ];
        for i in 0..hashes.len() {
            for j in (i + 1)..hashes.len() {
                assert_ne!(hashes[i], hashes[j], "hash collision at {i} and {j}");
            }
        }
    }

    #[test]
    fn test_multiple_blocks_finalized() {
        let mut store = EventStore::new();
        let t = event_types::TRANSFER();

        for block in 0..5 {
            store
                .emit(addr(1), vec![t], vec![block as u8], block, 0)
                .unwrap();
            store.finalize_block(block);
        }

        assert_eq!(store.finalized_blocks(), 5);
        assert_eq!(store.len(), 5);

        // Each block has a unique root
        let roots: Vec<Hash> = (0..5).map(|b| store.events_root(b).unwrap()).collect();
        for i in 0..roots.len() {
            for j in (i + 1)..roots.len() {
                assert_ne!(roots[i], roots[j]);
            }
        }
    }

    #[test]
    fn test_combined_address_and_topic_filter() {
        let mut store = EventStore::new();
        let t1 = event_types::TRANSFER();
        let t2 = event_types::SLASH();

        store.emit(addr(1), vec![t1], vec![], 1, 0).unwrap();
        store.emit(addr(1), vec![t2], vec![], 1, 1).unwrap();
        store.emit(addr(2), vec![t1], vec![], 1, 2).unwrap();

        let result = store.query(&EventFilter::new().address(addr(1)).topic(0, t1));
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].emitter, addr(1));
        assert_eq!(result[0].topics[0], t1);
    }

    #[test]
    fn test_log_index_sequential_per_block() {
        let mut store = EventStore::new();
        let t = event_types::TRANSFER();

        let idx0 = store.emit(addr(1), vec![t], vec![], 5, 0).unwrap();
        let idx1 = store.emit(addr(2), vec![t], vec![], 5, 1).unwrap();
        let idx2 = store.emit(addr(3), vec![t], vec![], 5, 2).unwrap();
        // New block resets
        let idx3 = store.emit(addr(1), vec![t], vec![], 6, 0).unwrap();

        assert_eq!(idx0, 0);
        assert_eq!(idx1, 1);
        assert_eq!(idx2, 2);
        assert_eq!(idx3, 0);
    }

    #[test]
    fn test_store_len_and_empty() {
        let mut store = EventStore::new();
        assert!(store.is_empty());
        assert_eq!(store.len(), 0);

        store.emit(addr(1), vec![topic(1)], vec![], 1, 0).unwrap();
        assert!(!store.is_empty());
        assert_eq!(store.len(), 1);
    }
}
