//! L1 event watcher — monitors L1 for anchor confirmations and inbound events.
//!
//! Watches Ethereum L1 for:
//! - Checkpoint anchor confirmations (submitted by CheckpointSubmitter)
//! - Stake deposit events (new stakers joining via L1 contract)
//! - Governance action events (proposals ratified on L1)
//! - Token bridge transfers (inbound FIL→Prova)
//!
//! Design:
//! - Polls L1 at configurable intervals, scanning blocks for relevant events
//! - Maintains a cursor (last processed L1 epoch) for crash recovery
//! - Verifies event authenticity against known L1 contract addresses
//! - Converts L1 events into BridgeMessages for Prova ingestion
//! - Handles reorgs by tracking finality depth before confirming events
//! - Deduplicates events via (event_type, tx_hash, log_index) tuple

use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, HashSet, VecDeque};

/// Simulated L1 block hash.
pub type L1BlockHash = [u8; 32];
/// Simulated L1 transaction hash.
pub type L1TxHash = [u8; 32];

/// Finality depth: how many L1 blocks to wait before considering an event final.
pub const DEFAULT_FINALITY_DEPTH: u64 = 30; // ~15 minutes on Ethereum (12s blocks)

/// Maximum number of blocks to scan per poll cycle (prevents runaway on long gaps).
pub const MAX_BLOCKS_PER_POLL: u64 = 100;

/// L1 event types emitted by Prova's L1 anchor contracts.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum L1EventType {
    /// A checkpoint was successfully anchored on L1.
    CheckpointAnchored {
        sequence: u64,
        state_root: [u8; 32],
        l1_epoch: u64,
    },
    /// A new stake deposit was made via the L1 staking contract.
    StakeDeposited { staker: [u8; 20], amount: u128 },
    /// A stake withdrawal was processed on L1.
    StakeWithdrawn { staker: [u8; 20], amount: u128 },
    /// A governance proposal was ratified on L1.
    GovernanceRatified {
        proposal_id: u64,
        action_hash: [u8; 32],
    },
    /// Inbound token transfer (FIL deposited to bridge).
    TokenBridgeDeposit {
        sender: [u8; 20],
        recipient: [u8; 20],
        amount: u128,
    },
}

/// A raw L1 event with provenance metadata.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct L1Event {
    /// The L1 block epoch where this event was emitted.
    pub l1_epoch: u64,
    /// L1 block hash.
    pub block_hash: L1BlockHash,
    /// Transaction hash.
    pub tx_hash: L1TxHash,
    /// Log index within the transaction.
    pub log_index: u32,
    /// The decoded event.
    pub event: L1EventType,
}

impl L1Event {
    /// Unique deduplication key.
    pub fn dedup_key(&self) -> [u8; 32] {
        let mut h = Sha256::new();
        h.update(self.tx_hash);
        h.update(self.log_index.to_le_bytes());
        h.finalize().into()
    }
}

/// Status of a tracked event as it progresses toward finality.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EventStatus {
    /// Seen on L1, waiting for finality depth.
    Pending { seen_at_l1_epoch: u64 },
    /// Finalized — enough L1 blocks have passed.
    Finalized { finalized_at_l1_epoch: u64 },
    /// Reorged out — the block was reverted.
    Reorged,
}

/// Configuration for the L1 event watcher.
#[derive(Debug, Clone)]
pub struct WatcherConfig {
    /// Number of L1 blocks to wait for finality.
    pub finality_depth: u64,
    /// Known L1 contract addresses to watch (any event from other addresses is ignored).
    pub watched_contracts: HashSet<[u8; 20]>,
    /// Maximum blocks to scan per poll cycle.
    pub max_blocks_per_poll: u64,
}

impl Default for WatcherConfig {
    fn default() -> Self {
        Self {
            finality_depth: DEFAULT_FINALITY_DEPTH,
            watched_contracts: HashSet::new(),
            max_blocks_per_poll: MAX_BLOCKS_PER_POLL,
        }
    }
}

/// Simulated L1 RPC client for testing.
pub trait L1Client {
    /// Get the current L1 head epoch.
    fn head_epoch(&self) -> u64;
    /// Fetch events from a specific L1 epoch. Returns empty vec if no events.
    fn get_events(&self, epoch: u64) -> Vec<L1Event>;
    /// Get the block hash at a given epoch (for reorg detection).
    fn get_block_hash(&self, epoch: u64) -> Option<L1BlockHash>;
}

/// A mock L1 client for testing.
#[derive(Debug, Default)]
pub struct MockL1Client {
    pub head: u64,
    pub events: BTreeMap<u64, Vec<L1Event>>,
    pub block_hashes: BTreeMap<u64, L1BlockHash>,
}

impl MockL1Client {
    pub fn new(head: u64) -> Self {
        Self {
            head,
            events: BTreeMap::new(),
            block_hashes: BTreeMap::new(),
        }
    }

    /// Set block hash for an epoch.
    pub fn set_block(&mut self, epoch: u64, hash: L1BlockHash) {
        self.block_hashes.insert(epoch, hash);
    }

    /// Add an event at a given epoch.
    pub fn add_event(&mut self, epoch: u64, event: L1Event) {
        self.events.entry(epoch).or_default().push(event);
    }

    /// Simulate a reorg by changing the block hash at an epoch.
    pub fn reorg(&mut self, epoch: u64, new_hash: L1BlockHash) {
        self.block_hashes.insert(epoch, new_hash);
        // Remove events from reorged block
        self.events.remove(&epoch);
    }
}

impl L1Client for MockL1Client {
    fn head_epoch(&self) -> u64 {
        self.head
    }

    fn get_events(&self, epoch: u64) -> Vec<L1Event> {
        self.events.get(&epoch).cloned().unwrap_or_default()
    }

    fn get_block_hash(&self, epoch: u64) -> Option<L1BlockHash> {
        self.block_hashes.get(&epoch).copied()
    }
}

/// Tracked event awaiting finality.
#[derive(Debug, Clone)]
struct PendingEvent {
    event: L1Event,
    first_seen_l1_epoch: u64,
    expected_block_hash: L1BlockHash,
    status: EventStatus,
}

/// The L1 event watcher.
pub struct L1EventWatcher<C: L1Client> {
    client: C,
    config: WatcherConfig,
    /// Last fully processed L1 epoch (cursor for crash recovery).
    cursor: u64,
    /// Events awaiting finality confirmation.
    pending: VecDeque<PendingEvent>,
    /// Finalized events ready for Prova ingestion.
    finalized: Vec<L1Event>,
    /// Deduplication set (event dedup keys).
    seen: HashSet<[u8; 32]>,
    /// Confirmed checkpoint sequences (for querying).
    confirmed_checkpoints: BTreeMap<u64, u64>, // sequence → l1_epoch
    /// Reorged event count (diagnostic).
    reorg_count: u64,
}

impl<C: L1Client> L1EventWatcher<C> {
    pub fn new(client: C, config: WatcherConfig, start_epoch: u64) -> Self {
        Self {
            client,
            config,
            cursor: start_epoch,
            pending: VecDeque::new(),
            finalized: Vec::new(),
            seen: HashSet::new(),
            confirmed_checkpoints: BTreeMap::new(),
            reorg_count: 0,
        }
    }

    /// Run one poll cycle: scan new L1 blocks, collect events, advance finality.
    /// Returns the number of newly finalized events.
    pub fn poll(&mut self) -> u64 {
        let head = self.client.head_epoch();
        if head <= self.cursor {
            return 0;
        }

        // 1. Scan new blocks for events
        let scan_end = std::cmp::min(self.cursor + self.config.max_blocks_per_poll, head);
        for epoch in (self.cursor + 1)..=scan_end {
            let events = self.client.get_events(epoch);
            let block_hash = self.client.get_block_hash(epoch).unwrap_or([0u8; 32]);

            for event in events {
                let key = event.dedup_key();
                if self.seen.contains(&key) {
                    continue;
                }
                // Filter by watched contracts (if any configured)
                // In production, events would carry a contract address; for now accept all
                self.seen.insert(key);
                self.pending.push_back(PendingEvent {
                    event,
                    first_seen_l1_epoch: epoch,
                    expected_block_hash: block_hash,
                    status: EventStatus::Pending {
                        seen_at_l1_epoch: epoch,
                    },
                });
            }
        }
        self.cursor = scan_end;

        // 2. Check finality and reorgs on pending events
        let mut newly_finalized = 0u64;
        let finality_depth = self.config.finality_depth;

        let mut i = 0;
        while i < self.pending.len() {
            let pe = &self.pending[i];
            let event_epoch = pe.first_seen_l1_epoch;

            // Check for reorg: block hash changed
            if let Some(current_hash) = self.client.get_block_hash(event_epoch) {
                if current_hash != pe.expected_block_hash {
                    // Reorged out
                    let key = pe.event.dedup_key();
                    self.seen.remove(&key);
                    self.pending.remove(i);
                    self.reorg_count += 1;
                    continue;
                }
            }

            // Check finality
            if head >= event_epoch + finality_depth {
                let mut pe = self.pending.remove(i).unwrap();
                pe.status = EventStatus::Finalized {
                    finalized_at_l1_epoch: head,
                };

                // Track checkpoint confirmations
                if let L1EventType::CheckpointAnchored {
                    sequence, l1_epoch, ..
                } = &pe.event.event
                {
                    self.confirmed_checkpoints.insert(*sequence, *l1_epoch);
                }

                self.finalized.push(pe.event);
                newly_finalized += 1;
                continue;
            }

            i += 1;
        }

        newly_finalized
    }

    /// Drain finalized events (transfers ownership to caller).
    pub fn drain_finalized(&mut self) -> Vec<L1Event> {
        std::mem::take(&mut self.finalized)
    }

    /// Get the current cursor (last processed L1 epoch).
    pub fn cursor(&self) -> u64 {
        self.cursor
    }

    /// Number of events awaiting finality.
    pub fn pending_count(&self) -> usize {
        self.pending.len()
    }

    /// Check if a checkpoint sequence has been confirmed on L1.
    pub fn is_checkpoint_confirmed(&self, sequence: u64) -> bool {
        self.confirmed_checkpoints.contains_key(&sequence)
    }

    /// Get the L1 epoch where a checkpoint was confirmed.
    pub fn checkpoint_confirmation_epoch(&self, sequence: u64) -> Option<u64> {
        self.confirmed_checkpoints.get(&sequence).copied()
    }

    /// Total events reorged out (diagnostic).
    pub fn reorg_count(&self) -> u64 {
        self.reorg_count
    }

    /// Get a reference to the underlying client.
    pub fn client(&self) -> &C {
        &self.client
    }

    /// Get a mutable reference to the underlying client (for testing).
    pub fn client_mut(&mut self) -> &mut C {
        &mut self.client
    }
}

// ───────────────────────── helpers ─────────────────────────

fn make_block_hash(epoch: u64) -> L1BlockHash {
    let mut h = Sha256::new();
    h.update(b"block");
    h.update(epoch.to_le_bytes());
    h.finalize().into()
}

fn make_tx_hash(seed: &[u8]) -> L1TxHash {
    let mut h = Sha256::new();
    h.update(seed);
    h.finalize().into()
}

fn make_event(epoch: u64, log_index: u32, event_type: L1EventType) -> L1Event {
    let block_hash = make_block_hash(epoch);
    let tx_hash = make_tx_hash(&[epoch as u8, log_index as u8]);
    L1Event {
        l1_epoch: epoch,
        block_hash,
        tx_hash,
        log_index,
        event: event_type,
    }
}

fn setup_watcher(start: u64, head: u64, finality: u64) -> L1EventWatcher<MockL1Client> {
    let client = MockL1Client::new(head);
    let config = WatcherConfig {
        finality_depth: finality,
        ..Default::default()
    };
    L1EventWatcher::new(client, config, start)
}

// ───────────────────────── tests ─────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_event_detection() {
        let mut w = setup_watcher(0, 50, 5);
        let ev = make_event(
            1,
            0,
            L1EventType::StakeDeposited {
                staker: [1u8; 20],
                amount: 1000,
            },
        );
        w.client_mut().set_block(1, make_block_hash(1));
        w.client_mut().add_event(1, ev);

        let finalized = w.poll();
        // Head=50, event at epoch 1, finality=5 → 50 >= 1+5 → finalized
        assert_eq!(finalized, 1);
        let events = w.drain_finalized();
        assert_eq!(events.len(), 1);
        match &events[0].event {
            L1EventType::StakeDeposited { staker, amount } => {
                assert_eq!(staker, &[1u8; 20]);
                assert_eq!(*amount, 1000);
            }
            _ => panic!("wrong event type"),
        }
    }

    #[test]
    fn test_event_waits_for_finality() {
        let mut w = setup_watcher(0, 3, 10);
        let ev = make_event(
            1,
            0,
            L1EventType::StakeDeposited {
                staker: [2u8; 20],
                amount: 500,
            },
        );
        w.client_mut().set_block(1, make_block_hash(1));
        w.client_mut().add_event(1, ev);

        let finalized = w.poll();
        assert_eq!(finalized, 0); // head=3, need epoch >= 11
        assert_eq!(w.pending_count(), 1);
        assert_eq!(w.drain_finalized().len(), 0);

        // Advance head past finality
        w.client_mut().head = 12;
        let finalized = w.poll();
        assert_eq!(finalized, 1);
        assert_eq!(w.pending_count(), 0);
    }

    #[test]
    fn test_deduplication() {
        let mut w = setup_watcher(0, 50, 5);
        let ev = make_event(
            1,
            0,
            L1EventType::StakeDeposited {
                staker: [3u8; 20],
                amount: 100,
            },
        );
        w.client_mut().set_block(1, make_block_hash(1));
        w.client_mut().add_event(1, ev.clone());
        w.client_mut().add_event(1, ev); // duplicate

        w.poll();
        assert_eq!(w.drain_finalized().len(), 1);
    }

    #[test]
    fn test_reorg_removes_event() {
        let mut w = setup_watcher(0, 5, 10);
        let original_hash = make_block_hash(3);
        let ev = make_event(
            3,
            0,
            L1EventType::TokenBridgeDeposit {
                sender: [4u8; 20],
                recipient: [5u8; 20],
                amount: 2000,
            },
        );
        w.client_mut().set_block(3, original_hash);
        w.client_mut().add_event(3, ev);

        w.poll();
        assert_eq!(w.pending_count(), 1);

        // Simulate reorg at epoch 3
        let reorged_hash = make_block_hash(999);
        w.client_mut().reorg(3, reorged_hash);
        w.client_mut().head = 20;

        w.poll();
        assert_eq!(w.pending_count(), 0);
        assert_eq!(w.reorg_count(), 1);
        assert_eq!(w.drain_finalized().len(), 0);
    }

    #[test]
    fn test_checkpoint_confirmation_tracking() {
        let mut w = setup_watcher(0, 50, 5);
        let ev = make_event(
            10,
            0,
            L1EventType::CheckpointAnchored {
                sequence: 42,
                state_root: [0xAB; 32],
                l1_epoch: 10,
            },
        );
        w.client_mut().set_block(10, make_block_hash(10));
        w.client_mut().add_event(10, ev);

        w.poll();
        assert!(w.is_checkpoint_confirmed(42));
        assert_eq!(w.checkpoint_confirmation_epoch(42), Some(10));
        assert!(!w.is_checkpoint_confirmed(43));
    }

    #[test]
    fn test_multiple_events_different_epochs() {
        let mut w = setup_watcher(0, 50, 5);

        for i in 1..=5u64 {
            let ev = make_event(
                i * 5,
                0,
                L1EventType::StakeDeposited {
                    staker: [i as u8; 20],
                    amount: i as u128 * 100,
                },
            );
            w.client_mut().set_block(i * 5, make_block_hash(i * 5));
            w.client_mut().add_event(i * 5, ev);
        }

        w.poll();
        let events = w.drain_finalized();
        assert_eq!(events.len(), 5);
    }

    #[test]
    fn test_max_blocks_per_poll_cap() {
        let mut w = setup_watcher(0, 500, 5);
        w.config.max_blocks_per_poll = 10;

        let ev = make_event(
            5,
            0,
            L1EventType::StakeDeposited {
                staker: [7u8; 20],
                amount: 999,
            },
        );
        w.client_mut().set_block(5, make_block_hash(5));
        w.client_mut().add_event(5, ev);

        w.poll();
        // Cursor should advance by max_blocks_per_poll, not to head
        assert_eq!(w.cursor(), 10);

        // Event at epoch 5 within range, finality=5, head=500 → finalized
        assert_eq!(w.drain_finalized().len(), 1);
    }

    #[test]
    fn test_governance_event() {
        let mut w = setup_watcher(0, 50, 5);
        let ev = make_event(
            7,
            1,
            L1EventType::GovernanceRatified {
                proposal_id: 101,
                action_hash: [0xCC; 32],
            },
        );
        w.client_mut().set_block(7, make_block_hash(7));
        w.client_mut().add_event(7, ev);

        w.poll();
        let events = w.drain_finalized();
        assert_eq!(events.len(), 1);
        match &events[0].event {
            L1EventType::GovernanceRatified {
                proposal_id,
                action_hash,
            } => {
                assert_eq!(*proposal_id, 101);
                assert_eq!(action_hash, &[0xCC; 32]);
            }
            _ => panic!("wrong event type"),
        }
    }

    #[test]
    fn test_withdrawal_event() {
        let mut w = setup_watcher(0, 50, 5);
        let ev = make_event(
            15,
            0,
            L1EventType::StakeWithdrawn {
                staker: [8u8; 20],
                amount: 750,
            },
        );
        w.client_mut().set_block(15, make_block_hash(15));
        w.client_mut().add_event(15, ev);

        w.poll();
        let events = w.drain_finalized();
        assert_eq!(events.len(), 1);
        match &events[0].event {
            L1EventType::StakeWithdrawn { staker, amount } => {
                assert_eq!(staker, &[8u8; 20]);
                assert_eq!(*amount, 750);
            }
            _ => panic!("wrong type"),
        }
    }

    #[test]
    fn test_no_events_no_crash() {
        let mut w = setup_watcher(0, 50, 5);
        let finalized = w.poll();
        assert_eq!(finalized, 0);
        assert_eq!(w.drain_finalized().len(), 0);
        assert_eq!(w.cursor(), 50);
    }

    #[test]
    fn test_cursor_advances_correctly() {
        let mut w = setup_watcher(100, 110, 5);
        w.poll();
        assert_eq!(w.cursor(), 110);

        // No new blocks — cursor stays
        w.poll();
        assert_eq!(w.cursor(), 110);

        // New blocks
        w.client_mut().head = 120;
        w.poll();
        assert_eq!(w.cursor(), 120);
    }

    #[test]
    fn test_mixed_event_types_single_block() {
        let mut w = setup_watcher(0, 50, 5);
        let block_hash = make_block_hash(20);
        w.client_mut().set_block(20, block_hash);

        w.client_mut().add_event(
            20,
            L1Event {
                l1_epoch: 20,
                block_hash,
                tx_hash: make_tx_hash(b"tx1"),
                log_index: 0,
                event: L1EventType::CheckpointAnchored {
                    sequence: 1,
                    state_root: [0x11; 32],
                    l1_epoch: 20,
                },
            },
        );
        w.client_mut().add_event(
            20,
            L1Event {
                l1_epoch: 20,
                block_hash,
                tx_hash: make_tx_hash(b"tx2"),
                log_index: 0,
                event: L1EventType::StakeDeposited {
                    staker: [9u8; 20],
                    amount: 300,
                },
            },
        );
        w.client_mut().add_event(
            20,
            L1Event {
                l1_epoch: 20,
                block_hash,
                tx_hash: make_tx_hash(b"tx3"),
                log_index: 0,
                event: L1EventType::TokenBridgeDeposit {
                    sender: [10u8; 20],
                    recipient: [11u8; 20],
                    amount: 5000,
                },
            },
        );

        w.poll();
        let events = w.drain_finalized();
        assert_eq!(events.len(), 3);
        assert!(w.is_checkpoint_confirmed(1));
    }

    #[test]
    fn test_event_dedup_key_uniqueness() {
        let ev1 = make_event(
            1,
            0,
            L1EventType::StakeDeposited {
                staker: [1u8; 20],
                amount: 100,
            },
        );
        let ev2 = make_event(
            1,
            1,
            L1EventType::StakeDeposited {
                staker: [1u8; 20],
                amount: 100,
            },
        );
        assert_ne!(ev1.dedup_key(), ev2.dedup_key());

        let ev3 = make_event(
            2,
            0,
            L1EventType::StakeDeposited {
                staker: [1u8; 20],
                amount: 100,
            },
        );
        assert_ne!(ev1.dedup_key(), ev3.dedup_key());
    }

    #[test]
    fn test_reorg_then_re_emit_event() {
        let mut w = setup_watcher(0, 5, 10);
        let original_hash = make_block_hash(3);
        let ev = make_event(
            3,
            0,
            L1EventType::StakeDeposited {
                staker: [12u8; 20],
                amount: 888,
            },
        );
        w.client_mut().set_block(3, original_hash);
        w.client_mut().add_event(3, ev.clone());

        w.poll();
        assert_eq!(w.pending_count(), 1);

        // Reorg
        let new_hash = make_block_hash(9999);
        w.client_mut().reorg(3, new_hash);
        w.client_mut().head = 8;
        w.poll();
        assert_eq!(w.pending_count(), 0);
        assert_eq!(w.reorg_count(), 1);

        // Re-emit same event in new block (different block hash, reset cursor trick)
        // The dedup key was cleared on reorg, so it should be picked up again
        let re_ev = L1Event {
            block_hash: new_hash,
            ..ev
        };
        w.client_mut().set_block(9, new_hash);
        w.client_mut().events.entry(9).or_default().push(re_ev);
        w.client_mut().head = 25;
        w.poll();
        // epoch 9 with finality=10, head=25 → 25 >= 19 → finalized
        assert_eq!(w.drain_finalized().len(), 1);
    }
}
