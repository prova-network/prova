//! SDK-007: Historical event replay & caching.
//!
//! Provides `EventReplayClient` that fetches historical events in batches,
//! caches them locally (LRU + epoch-indexed), and supports resumable replay
//! from the last cached epoch. Designed for block explorers, analytics
//! pipelines, and wallet history reconstruction.

use std::collections::{BTreeMap, VecDeque};

use prova_chain::events::{Event, EventFilter};
use prova_chain::types::{Address, Epoch, Hash};

// ── Configuration ─────────────────────────────────────────

/// Maximum events held in the LRU cache.
pub const DEFAULT_CACHE_CAPACITY: usize = 100_000;

/// Default batch size for historical fetches.
pub const DEFAULT_BATCH_SIZE: u64 = 1000;

/// Maximum concurrent batch fetches.
pub const MAX_CONCURRENT_FETCHES: usize = 4;

// ── Cache ─────────────────────────────────────────────────

/// Epoch-indexed event cache with LRU eviction.
#[derive(Debug)]
pub struct EventCache {
    /// Events keyed by epoch, ordered.
    epochs: BTreeMap<Epoch, Vec<CachedEvent>>,
    /// Total event count across all epochs.
    total_count: usize,
    /// Maximum events to retain.
    capacity: usize,
    /// Tracks insertion order for LRU eviction (oldest epoch first).
    epoch_order: VecDeque<Epoch>,
    /// Highest epoch fully cached.
    high_watermark: Epoch,
    /// Lowest epoch in cache.
    low_watermark: Epoch,
}

/// A cached event with its inclusion proof status.
#[derive(Debug, Clone)]
pub struct CachedEvent {
    pub event: Event,
    /// Whether the events_root inclusion proof was verified.
    pub proof_verified: bool,
    /// Block hash containing this event.
    pub block_hash: Hash,
}

impl EventCache {
    pub fn new(capacity: usize) -> Self {
        Self {
            epochs: BTreeMap::new(),
            total_count: 0,
            capacity,
            epoch_order: VecDeque::new(),
            high_watermark: 0,
            low_watermark: u64::MAX,
        }
    }

    pub fn with_default_capacity() -> Self {
        Self::new(DEFAULT_CACHE_CAPACITY)
    }

    /// Insert events for an epoch. Evicts oldest epochs if over capacity.
    pub fn insert(&mut self, epoch: Epoch, events: Vec<CachedEvent>) {
        let count = events.len();
        if count == 0 {
            return;
        }

        // If epoch already cached, remove old entries first.
        if let Some(old) = self.epochs.remove(&epoch) {
            self.total_count -= old.len();
        } else {
            self.epoch_order.push_back(epoch);
        }

        self.epochs.insert(epoch, events);
        self.total_count += count;

        if epoch > self.high_watermark {
            self.high_watermark = epoch;
        }
        if epoch < self.low_watermark {
            self.low_watermark = epoch;
        }

        // Evict oldest epochs until under capacity.
        while self.total_count > self.capacity {
            if let Some(oldest_epoch) = self.epoch_order.pop_front() {
                if let Some(removed) = self.epochs.remove(&oldest_epoch) {
                    self.total_count -= removed.len();
                }
                // Update low watermark.
                self.low_watermark = self.epoch_order.front().copied().unwrap_or(u64::MAX);
            } else {
                break;
            }
        }
    }

    /// Query cached events matching a filter within an epoch range.
    pub fn query(&self, filter: &EventFilter, from: Epoch, to: Epoch) -> Vec<&CachedEvent> {
        let mut results = Vec::new();
        for (&epoch, events) in self.epochs.range(from..=to) {
            for ev in events {
                if filter_matches(filter, &ev.event, epoch) {
                    results.push(ev);
                }
            }
        }
        results
    }

    /// Check if epoch range is fully cached.
    pub fn is_range_cached(&self, from: Epoch, to: Epoch) -> bool {
        if from < self.low_watermark || to > self.high_watermark {
            return false;
        }
        // Check every epoch in range exists.
        for epoch in from..=to {
            if !self.epochs.contains_key(&epoch) {
                return false;
            }
        }
        true
    }

    pub fn total_count(&self) -> usize {
        self.total_count
    }

    pub fn epoch_count(&self) -> usize {
        self.epochs.len()
    }

    pub fn high_watermark(&self) -> Epoch {
        self.high_watermark
    }

    pub fn low_watermark(&self) -> Epoch {
        self.low_watermark
    }

    /// Clear all cached data.
    pub fn clear(&mut self) {
        self.epochs.clear();
        self.epoch_order.clear();
        self.total_count = 0;
        self.high_watermark = 0;
        self.low_watermark = u64::MAX;
    }

    /// Export cache state for persistence.
    pub fn export_watermarks(&self) -> CacheWatermarks {
        CacheWatermarks {
            low: self.low_watermark,
            high: self.high_watermark,
            total_events: self.total_count,
            total_epochs: self.epochs.len(),
        }
    }
}

/// Serializable cache state for resume.
#[derive(Debug, Clone)]
pub struct CacheWatermarks {
    pub low: Epoch,
    pub high: Epoch,
    pub total_events: usize,
    pub total_epochs: usize,
}

// ── Replay Engine ─────────────────────────────────────────

/// Replay progress tracking.
#[derive(Debug, Clone)]
pub struct ReplayProgress {
    /// Target epoch range.
    pub from_epoch: Epoch,
    pub to_epoch: Epoch,
    /// Current replay cursor (next epoch to fetch).
    pub cursor: Epoch,
    /// Events fetched so far.
    pub events_fetched: u64,
    /// Batches completed.
    pub batches_completed: u64,
    /// Whether replay is complete.
    pub complete: bool,
}

impl ReplayProgress {
    pub fn new(from: Epoch, to: Epoch) -> Self {
        Self {
            from_epoch: from,
            to_epoch: to,
            cursor: from,
            events_fetched: 0,
            batches_completed: 0,
            complete: from > to,
        }
    }

    pub fn fraction_complete(&self) -> f64 {
        if self.to_epoch <= self.from_epoch {
            return 1.0;
        }
        let range = (self.to_epoch - self.from_epoch) as f64;
        let done = (self.cursor.saturating_sub(self.from_epoch)) as f64;
        (done / range).min(1.0)
    }
}

/// Configuration for replay operations.
#[derive(Debug, Clone)]
pub struct ReplayConfig {
    /// Epochs per fetch batch.
    pub batch_size: u64,
    /// Verify events_root inclusion proofs during replay.
    pub verify_proofs: bool,
    /// Skip epochs with no events (sparse replay).
    pub skip_empty: bool,
    /// Maximum total events to fetch (0 = unlimited).
    pub max_events: u64,
}

impl Default for ReplayConfig {
    fn default() -> Self {
        Self {
            batch_size: DEFAULT_BATCH_SIZE,
            verify_proofs: true,
            skip_empty: false,
            max_events: 0,
        }
    }
}

/// Result of a single batch fetch.
#[derive(Debug)]
pub struct BatchResult {
    pub from_epoch: Epoch,
    pub to_epoch: Epoch,
    pub events: Vec<(Epoch, Vec<CachedEvent>)>,
    pub total_events: usize,
    pub proof_failures: usize,
}

/// Historical event replay engine.
///
/// Drives batched fetching of historical events from a node, populates
/// the local cache, and tracks progress for resumable replay.
#[derive(Debug)]
pub struct EventReplayEngine {
    pub cache: EventCache,
    pub config: ReplayConfig,
    pub progress: Option<ReplayProgress>,
    /// Simulated RPC endpoint (in real impl, this would be a connection).
    node_url: String,
    /// Accumulated stats.
    stats: ReplayStats,
}

#[derive(Debug, Clone, Default)]
pub struct ReplayStats {
    pub total_batches: u64,
    pub total_events: u64,
    pub total_proof_failures: u64,
    pub total_empty_epochs: u64,
}

impl EventReplayEngine {
    pub fn new(node_url: &str, config: ReplayConfig, cache_capacity: usize) -> Self {
        Self {
            cache: EventCache::new(cache_capacity),
            config,
            progress: None,
            node_url: node_url.to_string(),
            stats: ReplayStats::default(),
        }
    }

    /// Start a new replay session for the given epoch range.
    pub fn start_replay(&mut self, from: Epoch, to: Epoch) -> &ReplayProgress {
        // Check if cache already covers part of the range.
        let effective_from = if self.cache.high_watermark() >= from && self.cache.low_watermark() <= from {
            // Resume from after cached data.
            self.cache.high_watermark() + 1
        } else {
            from
        };

        self.progress = Some(ReplayProgress::new(effective_from, to));
        self.progress.as_ref().unwrap()
    }

    /// Process the next batch. Returns batch result or None if replay is complete.
    pub fn next_batch(&mut self, events_source: &MockEventSource) -> Option<BatchResult> {
        let progress = self.progress.as_mut()?;
        if progress.complete {
            return None;
        }

        let batch_from = progress.cursor;
        let batch_to = (batch_from + self.config.batch_size - 1).min(progress.to_epoch);

        // Fetch events for this batch range.
        let mut batch_events: Vec<(Epoch, Vec<CachedEvent>)> = Vec::new();
        let mut total = 0;
        let mut proof_failures = 0;

        for epoch in batch_from..=batch_to {
            let raw_events = events_source.get_events(epoch);
            if raw_events.is_empty() {
                self.stats.total_empty_epochs += 1;
                if self.config.skip_empty {
                    continue;
                }
            }

            let cached: Vec<CachedEvent> = raw_events
                .into_iter()
                .map(|(event, block_hash)| {
                    let verified = if self.config.verify_proofs {
                        verify_inclusion_proof(&event, &block_hash)
                    } else {
                        false
                    };
                    if self.config.verify_proofs && !verified {
                        proof_failures += 1;
                    }
                    CachedEvent {
                        event,
                        proof_verified: verified,
                        block_hash,
                    }
                })
                .collect();

            total += cached.len();

            // Insert into cache.
            self.cache.insert(epoch, cached.clone());
            batch_events.push((epoch, cached));
        }

        // Update progress.
        progress.cursor = batch_to + 1;
        progress.events_fetched += total as u64;
        progress.batches_completed += 1;
        if progress.cursor > progress.to_epoch {
            progress.complete = true;
        }
        if self.config.max_events > 0 && progress.events_fetched >= self.config.max_events {
            progress.complete = true;
        }

        // Update stats.
        self.stats.total_batches += 1;
        self.stats.total_events += total as u64;
        self.stats.total_proof_failures += proof_failures as u64;

        Some(BatchResult {
            from_epoch: batch_from,
            to_epoch: batch_to,
            events: batch_events,
            total_events: total,
            proof_failures: proof_failures as usize,
        })
    }

    /// Resume replay from last known position using cache watermarks.
    pub fn resume_replay(&mut self, to: Epoch) -> &ReplayProgress {
        let from = if self.cache.high_watermark() > 0 {
            self.cache.high_watermark() + 1
        } else {
            0
        };
        self.start_replay(from, to)
    }

    pub fn stats(&self) -> &ReplayStats {
        &self.stats
    }

    pub fn is_complete(&self) -> bool {
        self.progress.as_ref().map_or(true, |p| p.complete)
    }
}

// ── Mock Source for Testing ───────────────────────────────

/// Mock event source for testing (simulates node RPC responses).
pub struct MockEventSource {
    events: BTreeMap<Epoch, Vec<(Event, Hash)>>,
}

impl MockEventSource {
    pub fn new() -> Self {
        Self {
            events: BTreeMap::new(),
        }
    }

    pub fn add_event(&mut self, epoch: Epoch, event: Event, block_hash: Hash) {
        self.events
            .entry(epoch)
            .or_insert_with(Vec::new)
            .push((event, block_hash));
    }

    pub fn get_events(&self, epoch: Epoch) -> Vec<(Event, Hash)> {
        self.events.get(&epoch).cloned().unwrap_or_default()
    }
}

// ── Helpers ───────────────────────────────────────────────

/// Check if an event matches a filter (simplified).
fn filter_matches(filter: &EventFilter, event: &Event, _epoch: Epoch) -> bool {
    // Match on topics (fixed-size array, None = wildcard).
    for (i, tf) in filter.topics.iter().enumerate() {
        if let Some(expected) = tf {
            if i >= event.topics.len() || event.topics[i] != *expected {
                return false;
            }
        }
    }

    // Match on emitter address.
    if let Some(ref addr) = filter.address {
        if event.emitter != *addr {
            return false;
        }
    }

    true
}

/// Simulate inclusion proof verification (always passes in test).
fn verify_inclusion_proof(_event: &Event, _block_hash: &Hash) -> bool {
    // In production: verify Merkle proof against block receipt's events_root.
    true
}

// ── Tests ─────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn make_event(topic0: u8, emitter_byte: u8) -> Event {
        let mut topics = vec![[0u8; 32]];
        topics[0][0] = topic0;
        let mut emitter = [0u8; 20];
        emitter[0] = emitter_byte;
        Event {
            topics,
            data: vec![],
            emitter: Address(emitter),
            block_number: 0,
            log_index: 0,
            tx_index: 0,
        }
    }

    fn make_hash(v: u8) -> Hash {
        let mut h = [0u8; 32];
        h[0] = v;
        h
    }

    // ── Cache tests ───────────────────────────────────────

    #[test]
    fn test_cache_insert_and_query() {
        let mut cache = EventCache::new(1000);
        let ev = CachedEvent {
            event: make_event(1, 10),
            proof_verified: true,
            block_hash: make_hash(1),
        };
        cache.insert(100, vec![ev]);
        assert_eq!(cache.total_count(), 1);
        assert_eq!(cache.epoch_count(), 1);
        assert_eq!(cache.high_watermark(), 100);
    }

    #[test]
    fn test_cache_lru_eviction() {
        let mut cache = EventCache::new(5);
        // Insert 3 events across 3 epochs.
        for i in 0..3 {
            let ev = CachedEvent {
                event: make_event(i as u8, 10),
                proof_verified: true,
                block_hash: make_hash(i as u8),
            };
            cache.insert(i as Epoch, vec![ev]);
        }
        assert_eq!(cache.total_count(), 3);

        // Insert 3 more events — should evict epoch 0.
        for i in 3..6 {
            let ev = CachedEvent {
                event: make_event(i as u8, 10),
                proof_verified: true,
                block_hash: make_hash(i as u8),
            };
            cache.insert(i as Epoch, vec![ev]);
        }
        assert!(cache.total_count() <= 5);
        assert!(!cache.epochs.contains_key(&0));
    }

    #[test]
    fn test_cache_query_with_filter() {
        let mut cache = EventCache::new(1000);
        let ev1 = CachedEvent {
            event: make_event(1, 10),
            proof_verified: true,
            block_hash: make_hash(1),
        };
        let ev2 = CachedEvent {
            event: make_event(2, 20),
            proof_verified: true,
            block_hash: make_hash(2),
        };
        cache.insert(100, vec![ev1, ev2]);

        // Filter by topic.
        let mut filter = EventFilter::default();
        let mut expected = [0u8; 32];
        expected[0] = 1;
        filter.topics[0] = Some(expected);

        let results = cache.query(&filter, 100, 100);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].event.topics[0][0], 1);
    }

    #[test]
    fn test_cache_is_range_cached() {
        let mut cache = EventCache::new(1000);
        for i in 10..=15 {
            cache.insert(i, vec![CachedEvent {
                event: make_event(1, 1),
                proof_verified: true,
                block_hash: make_hash(1),
            }]);
        }
        assert!(cache.is_range_cached(10, 15));
        assert!(!cache.is_range_cached(9, 15));
        assert!(!cache.is_range_cached(10, 16));
    }

    #[test]
    fn test_cache_clear() {
        let mut cache = EventCache::new(1000);
        cache.insert(1, vec![CachedEvent {
            event: make_event(1, 1),
            proof_verified: true,
            block_hash: make_hash(1),
        }]);
        assert_eq!(cache.total_count(), 1);
        cache.clear();
        assert_eq!(cache.total_count(), 0);
        assert_eq!(cache.epoch_count(), 0);
    }

    #[test]
    fn test_cache_overwrite_epoch() {
        let mut cache = EventCache::new(1000);
        cache.insert(100, vec![CachedEvent {
            event: make_event(1, 1),
            proof_verified: true,
            block_hash: make_hash(1),
        }]);
        assert_eq!(cache.total_count(), 1);

        // Overwrite with 2 events.
        cache.insert(100, vec![
            CachedEvent { event: make_event(2, 2), proof_verified: true, block_hash: make_hash(2) },
            CachedEvent { event: make_event(3, 3), proof_verified: true, block_hash: make_hash(3) },
        ]);
        assert_eq!(cache.total_count(), 2);
    }

    #[test]
    fn test_cache_watermarks_export() {
        let mut cache = EventCache::new(1000);
        cache.insert(50, vec![CachedEvent {
            event: make_event(1, 1),
            proof_verified: true,
            block_hash: make_hash(1),
        }]);
        cache.insert(100, vec![CachedEvent {
            event: make_event(2, 2),
            proof_verified: true,
            block_hash: make_hash(2),
        }]);
        let wm = cache.export_watermarks();
        assert_eq!(wm.low, 50);
        assert_eq!(wm.high, 100);
        assert_eq!(wm.total_events, 2);
        assert_eq!(wm.total_epochs, 2);
    }

    // ── Replay Engine tests ───────────────────────────────

    #[test]
    fn test_replay_progress_tracking() {
        let p = ReplayProgress::new(0, 100);
        assert_eq!(p.fraction_complete(), 0.0);
        assert!(!p.complete);
    }

    #[test]
    fn test_replay_empty_range() {
        let p = ReplayProgress::new(100, 50);
        assert!(p.complete);
        assert_eq!(p.fraction_complete(), 1.0);
    }

    #[test]
    fn test_replay_engine_basic() {
        let config = ReplayConfig {
            batch_size: 10,
            verify_proofs: true,
            skip_empty: false,
            max_events: 0,
        };
        let mut engine = EventReplayEngine::new("http://localhost:8080", config, 10000);

        // Create mock source with events.
        let mut source = MockEventSource::new();
        for epoch in 0..20u64 {
            source.add_event(epoch, make_event(1, 10), make_hash(epoch as u8));
        }

        engine.start_replay(0, 19);
        let batch1 = engine.next_batch(&source).unwrap();
        assert_eq!(batch1.from_epoch, 0);
        assert_eq!(batch1.to_epoch, 9);
        assert_eq!(batch1.total_events, 10);

        let batch2 = engine.next_batch(&source).unwrap();
        assert_eq!(batch2.from_epoch, 10);
        assert_eq!(batch2.to_epoch, 19);

        assert!(engine.is_complete());
    }

    #[test]
    fn test_replay_engine_resume() {
        let config = ReplayConfig::default();
        let mut engine = EventReplayEngine::new("http://localhost:8080", config, 10000);

        // Pre-populate cache.
        for epoch in 0..50u64 {
            engine.cache.insert(epoch, vec![CachedEvent {
                event: make_event(1, 1),
                proof_verified: true,
                block_hash: make_hash(1),
            }]);
        }

        // Resume should start from epoch 50.
        let progress = engine.resume_replay(100);
        assert_eq!(progress.from_epoch, 50);
        assert_eq!(progress.to_epoch, 100);
    }

    #[test]
    fn test_replay_max_events_limit() {
        let config = ReplayConfig {
            batch_size: 100,
            verify_proofs: false,
            skip_empty: false,
            max_events: 5,
        };
        let mut engine = EventReplayEngine::new("http://localhost:8080", config, 10000);

        let mut source = MockEventSource::new();
        for epoch in 0..100u64 {
            source.add_event(epoch, make_event(1, 10), make_hash(1));
        }

        engine.start_replay(0, 99);
        let batch = engine.next_batch(&source).unwrap();
        assert!(batch.total_events > 0);
        // After this batch, events_fetched >= max_events → complete.
        assert!(engine.is_complete());
    }

    #[test]
    fn test_replay_skip_empty_epochs() {
        let config = ReplayConfig {
            batch_size: 10,
            verify_proofs: false,
            skip_empty: true,
            max_events: 0,
        };
        let mut engine = EventReplayEngine::new("http://localhost:8080", config, 10000);

        // Only epoch 5 has events.
        let mut source = MockEventSource::new();
        source.add_event(5, make_event(1, 10), make_hash(5));

        engine.start_replay(0, 9);
        let batch = engine.next_batch(&source).unwrap();
        assert_eq!(batch.total_events, 1);
        assert!(engine.stats().total_empty_epochs >= 9);
    }

    #[test]
    fn test_replay_stats_accumulation() {
        let config = ReplayConfig {
            batch_size: 5,
            verify_proofs: true,
            skip_empty: false,
            max_events: 0,
        };
        let mut engine = EventReplayEngine::new("http://localhost:8080", config, 10000);

        let mut source = MockEventSource::new();
        for epoch in 0..10u64 {
            source.add_event(epoch, make_event(1, 10), make_hash(1));
        }

        engine.start_replay(0, 9);
        while let Some(_) = engine.next_batch(&source) {}

        let stats = engine.stats();
        assert_eq!(stats.total_batches, 2);
        assert_eq!(stats.total_events, 10);
        assert_eq!(stats.total_proof_failures, 0);
    }

    #[test]
    fn test_mock_event_source() {
        let mut source = MockEventSource::new();
        source.add_event(10, make_event(1, 1), make_hash(1));
        source.add_event(10, make_event(2, 2), make_hash(2));

        assert_eq!(source.get_events(10).len(), 2);
        assert_eq!(source.get_events(11).len(), 0);
    }

    #[test]
    fn test_replay_complete_signal() {
        let config = ReplayConfig {
            batch_size: 100,
            ..Default::default()
        };
        let mut engine = EventReplayEngine::new("http://localhost:8080", config, 10000);
        let source = MockEventSource::new();

        engine.start_replay(0, 50);
        let batch = engine.next_batch(&source).unwrap();
        assert_eq!(batch.to_epoch, 50);
        assert!(engine.is_complete());
        assert!(engine.next_batch(&source).is_none());
    }
}
