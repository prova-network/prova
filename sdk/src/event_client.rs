//! Event subscription client — subscribe, filter, and replay on-chain events.
//!
//! Provides `EventClient` that connects to a Prova node's event subscription
//! engine, supporting real-time filtered subscriptions, historical replay,
//! and local event caching for offline queries.

use std::collections::{HashMap, VecDeque};
use std::time::Instant;

use prova_chain::events::{Event, EventFilter};
use prova_chain::types::{Epoch, Hash};

// ── Types ────────────────────────────────────────────────────

/// Subscription handle returned to caller.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SubscriptionHandle(pub u64);

/// Event delivery mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeliveryMode {
    /// Receive events as they are emitted (real-time).
    Realtime,
    /// Replay historical events from a given epoch, then switch to real-time.
    ReplayThenRealtime { from_epoch: Epoch },
    /// One-shot historical query only (no ongoing subscription).
    HistoricalOnly { from_epoch: Epoch, to_epoch: Epoch },
}

/// Configuration for a subscription request.
#[derive(Debug, Clone)]
pub struct SubscriptionRequest {
    pub filter: EventFilter,
    pub mode: DeliveryMode,
    /// Maximum events to buffer locally before dropping oldest.
    pub buffer_size: usize,
}

impl SubscriptionRequest {
    pub fn realtime(filter: EventFilter) -> Self {
        Self {
            filter,
            mode: DeliveryMode::Realtime,
            buffer_size: 4096,
        }
    }

    pub fn replay_then_realtime(filter: EventFilter, from_epoch: Epoch) -> Self {
        Self {
            filter,
            mode: DeliveryMode::ReplayThenRealtime { from_epoch },
            buffer_size: 8192,
        }
    }

    pub fn historical(filter: EventFilter, from_epoch: Epoch, to_epoch: Epoch) -> Self {
        Self {
            filter,
            mode: DeliveryMode::HistoricalOnly { from_epoch, to_epoch },
            buffer_size: 16384,
        }
    }
}

/// A received event with metadata.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReceivedEvent {
    pub event: Event,
    pub block_epoch: Epoch,
    pub block_hash: Hash,
    pub log_index: u32,
    pub received_at_ms: u64,
}

/// Error types for event client operations.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EventClientError {
    /// Connection to node failed or was lost.
    ConnectionLost,
    /// Subscription was rejected by the node (e.g., limit reached).
    SubscriptionRejected(String),
    /// Replay range is invalid or unavailable (pruned).
    ReplayUnavailable { requested: Epoch, earliest: Epoch },
    /// Buffer overflow — events were dropped.
    BufferOverflow { dropped: usize },
    /// Subscription not found.
    NotFound(SubscriptionHandle),
    /// Client is shut down.
    Shutdown,
}

// ── Event Cache ──────────────────────────────────────────────

/// Local event cache for offline queries and deduplication.
#[derive(Debug)]
pub struct EventCache {
    /// Events stored by epoch for range queries.
    by_epoch: HashMap<Epoch, Vec<ReceivedEvent>>,
    /// Total cached events.
    total: usize,
    /// Maximum cache size.
    max_size: usize,
    /// Earliest epoch in cache.
    earliest_epoch: Option<Epoch>,
    /// Latest epoch in cache.
    latest_epoch: Option<Epoch>,
}

impl EventCache {
    pub fn new(max_size: usize) -> Self {
        Self {
            by_epoch: HashMap::new(),
            total: 0,
            max_size,
            earliest_epoch: None,
            latest_epoch: None,
        }
    }

    /// Insert an event into the cache. Returns number of evicted events.
    pub fn insert(&mut self, event: ReceivedEvent) -> usize {
        let epoch = event.block_epoch;
        let mut evicted = 0;

        // Evict oldest epoch(s) if at capacity.
        while self.total >= self.max_size {
            if let Some(earliest) = self.earliest_epoch {
                if let Some(removed) = self.by_epoch.remove(&earliest) {
                    evicted += removed.len();
                    self.total -= removed.len();
                }
                // Find next earliest.
                self.earliest_epoch = self.by_epoch.keys().copied().min();
            } else {
                break;
            }
        }

        self.by_epoch.entry(epoch).or_default().push(event);
        self.total += 1;

        // Update bounds.
        self.earliest_epoch = Some(
            self.earliest_epoch.map_or(epoch, |e| e.min(epoch)),
        );
        self.latest_epoch = Some(
            self.latest_epoch.map_or(epoch, |e| e.max(epoch)),
        );

        evicted
    }

    /// Query cached events in an epoch range matching a filter.
    pub fn query(&self, from: Epoch, to: Epoch, filter: &EventFilter) -> Vec<&ReceivedEvent> {
        let mut results = Vec::new();
        for epoch in from..=to {
            if let Some(events) = self.by_epoch.get(&epoch) {
                for ev in events {
                    if filter_matches(&ev.event, filter) {
                        results.push(ev);
                    }
                }
            }
        }
        results
    }

    pub fn total_cached(&self) -> usize {
        self.total
    }

    pub fn epoch_range(&self) -> Option<(Epoch, Epoch)> {
        match (self.earliest_epoch, self.latest_epoch) {
            (Some(e), Some(l)) => Some((e, l)),
            _ => None,
        }
    }

    /// Clear all cached events.
    pub fn clear(&mut self) {
        self.by_epoch.clear();
        self.total = 0;
        self.earliest_epoch = None;
        self.latest_epoch = None;
    }
}

// ── Subscription State ───────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SubState {
    /// Replaying historical events.
    Replaying,
    /// Receiving real-time events.
    Live,
    /// Completed (historical-only mode).
    Completed,
    /// Cancelled by client.
    Cancelled,
    /// Errored.
    Errored,
}

#[derive(Debug)]
struct ActiveSubscription {
    handle: SubscriptionHandle,
    request: SubscriptionRequest,
    state: SubState,
    buffer: VecDeque<ReceivedEvent>,
    events_received: u64,
    events_dropped: u64,
    created_at: Instant,
    last_event_at: Option<Instant>,
    /// For replay: tracks replay progress.
    replay_cursor: Option<Epoch>,
}

// ── Event Client ─────────────────────────────────────────────

/// Client for subscribing to and querying on-chain events.
///
/// In simulation mode, events are injected directly via `inject_event`.
/// In production, this would connect via WebSocket to the node's
/// subscription engine.
#[derive(Debug)]
pub struct EventClient {
    subscriptions: HashMap<SubscriptionHandle, ActiveSubscription>,
    next_handle: u64,
    cache: EventCache,
    connected: bool,
    /// Simulated node-side events for testing.
    simulated_events: Vec<ReceivedEvent>,
    /// Current simulated epoch.
    current_epoch: Epoch,
}

impl EventClient {
    /// Create a new event client (simulation mode).
    pub fn new(cache_size: usize) -> Self {
        Self {
            subscriptions: HashMap::new(),
            next_handle: 1,
            cache: EventCache::new(cache_size),
            connected: true,
            simulated_events: Vec::new(),
            current_epoch: 0,
        }
    }

    /// Subscribe to events matching a request.
    pub fn subscribe(
        &mut self,
        request: SubscriptionRequest,
    ) -> Result<SubscriptionHandle, EventClientError> {
        if !self.connected {
            return Err(EventClientError::ConnectionLost);
        }

        let handle = SubscriptionHandle(self.next_handle);
        self.next_handle += 1;

        let initial_state = match &request.mode {
            DeliveryMode::Realtime => SubState::Live,
            DeliveryMode::ReplayThenRealtime { .. } => SubState::Replaying,
            DeliveryMode::HistoricalOnly { .. } => SubState::Replaying,
        };

        let replay_cursor = match &request.mode {
            DeliveryMode::ReplayThenRealtime { from_epoch } => Some(*from_epoch),
            DeliveryMode::HistoricalOnly { from_epoch, .. } => Some(*from_epoch),
            _ => None,
        };

        let sub = ActiveSubscription {
            handle,
            request,
            state: initial_state,
            buffer: VecDeque::new(),
            events_received: 0,
            events_dropped: 0,
            created_at: Instant::now(),
            last_event_at: None,
            replay_cursor,
        };

        self.subscriptions.insert(handle, sub);
        Ok(handle)
    }

    /// Cancel a subscription.
    pub fn unsubscribe(&mut self, handle: SubscriptionHandle) -> Result<(), EventClientError> {
        match self.subscriptions.get_mut(&handle) {
            Some(sub) => {
                sub.state = SubState::Cancelled;
                Ok(())
            }
            None => Err(EventClientError::NotFound(handle)),
        }
    }

    /// Poll for the next batch of events on a subscription.
    /// Returns up to `max` events. Empty vec means no new events.
    pub fn poll(
        &mut self,
        handle: SubscriptionHandle,
        max: usize,
    ) -> Result<Vec<ReceivedEvent>, EventClientError> {
        let sub = self.subscriptions.get_mut(&handle)
            .ok_or(EventClientError::NotFound(handle))?;

        if sub.state == SubState::Cancelled {
            return Err(EventClientError::NotFound(handle));
        }

        let count = max.min(sub.buffer.len());
        let events: Vec<_> = sub.buffer.drain(..count).collect();
        Ok(events)
    }

    /// Inject a simulated event (for testing). Dispatches to matching subscriptions.
    pub fn inject_event(&mut self, event: ReceivedEvent) {
        self.current_epoch = self.current_epoch.max(event.block_epoch);

        // Cache it.
        self.cache.insert(event.clone());

        // Dispatch to matching live subscriptions.
        for sub in self.subscriptions.values_mut() {
            if sub.state != SubState::Live {
                continue;
            }
            if !filter_matches(&event.event, &sub.request.filter) {
                continue;
            }

            if sub.buffer.len() >= sub.request.buffer_size {
                sub.buffer.pop_front();
                sub.events_dropped += 1;
            }
            sub.buffer.push_back(event.clone());
            sub.events_received += 1;
            sub.last_event_at = Some(Instant::now());
        }
    }

    /// Process replay for subscriptions in replay mode.
    /// Call after injecting historical events. Advances replay cursors.
    pub fn process_replay(&mut self) {
        let _cached_events: Vec<ReceivedEvent> = self.simulated_events.clone();

        for sub in self.subscriptions.values_mut() {
            if sub.state != SubState::Replaying {
                continue;
            }

            let (from, to) = match &sub.request.mode {
                DeliveryMode::ReplayThenRealtime { from_epoch } => {
                    (*from_epoch, self.current_epoch)
                }
                DeliveryMode::HistoricalOnly { from_epoch, to_epoch } => {
                    (*from_epoch, *to_epoch)
                }
                _ => continue,
            };

            // Replay from cache.
            let matching = self.cache.query(from, to, &sub.request.filter);
            for ev in matching {
                if sub.buffer.len() >= sub.request.buffer_size {
                    sub.buffer.pop_front();
                    sub.events_dropped += 1;
                }
                sub.buffer.push_back(ev.clone());
                sub.events_received += 1;
            }

            // Transition state.
            match &sub.request.mode {
                DeliveryMode::ReplayThenRealtime { .. } => {
                    sub.state = SubState::Live;
                    sub.replay_cursor = None;
                }
                DeliveryMode::HistoricalOnly { .. } => {
                    sub.state = SubState::Completed;
                }
                _ => {}
            }
        }
    }

    /// Add events to the simulated store (for replay testing).
    pub fn add_historical_events(&mut self, events: Vec<ReceivedEvent>) {
        for ev in events {
            self.current_epoch = self.current_epoch.max(ev.block_epoch);
            self.cache.insert(ev.clone());
            self.simulated_events.push(ev);
        }
    }

    /// Query the local cache directly (no subscription needed).
    pub fn query_cache(
        &self,
        from: Epoch,
        to: Epoch,
        filter: &EventFilter,
    ) -> Vec<&ReceivedEvent> {
        self.cache.query(from, to, filter)
    }

    /// Get subscription statistics.
    pub fn sub_stats(&self, handle: SubscriptionHandle) -> Result<SubStats, EventClientError> {
        let sub = self.subscriptions.get(&handle)
            .ok_or(EventClientError::NotFound(handle))?;
        Ok(SubStats {
            state: sub.state,
            events_received: sub.events_received,
            events_dropped: sub.events_dropped,
            buffer_len: sub.buffer.len(),
            buffer_capacity: sub.request.buffer_size,
        })
    }

    /// Number of active (non-cancelled, non-errored) subscriptions.
    pub fn active_count(&self) -> usize {
        self.subscriptions.values()
            .filter(|s| matches!(s.state, SubState::Live | SubState::Replaying))
            .count()
    }

    /// Simulate disconnection.
    pub fn disconnect(&mut self) {
        self.connected = false;
        for sub in self.subscriptions.values_mut() {
            if sub.state == SubState::Live || sub.state == SubState::Replaying {
                sub.state = SubState::Errored;
            }
        }
    }

    /// Simulate reconnection.
    pub fn reconnect(&mut self) {
        self.connected = true;
    }

    /// Total events in cache.
    pub fn cache_size(&self) -> usize {
        self.cache.total_cached()
    }

    /// Clear all subscriptions and cache.
    pub fn reset(&mut self) {
        self.subscriptions.clear();
        self.cache.clear();
        self.simulated_events.clear();
        self.next_handle = 1;
        self.current_epoch = 0;
    }
}

/// Subscription statistics.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SubStats {
    pub state: SubState,
    pub events_received: u64,
    pub events_dropped: u64,
    pub buffer_len: usize,
    pub buffer_capacity: usize,
}

// ── Filter matching (client-side) ────────────────────────────

fn filter_matches(event: &Event, filter: &EventFilter) -> bool {
    // Address filter.
    if let Some(ref addr) = filter.address {
        if event.emitter != *addr {
            return false;
        }
    }
    // Topic filter: positional matching (None = wildcard).
    for (i, topic_filter) in filter.topics.iter().enumerate() {
        if let Some(expected) = topic_filter {
            match event.topics.get(i) {
                Some(actual) if actual == expected => {}
                _ => return false,
            }
        }
    }
    true
}

// ── Tests ────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use prova_chain::events::{Event, EventFilter};
    use prova_chain::types::Address;

    fn test_addr(b: u8) -> Address {
        Address([b; 20])
    }

    fn test_event(emitter: Address, topic: Hash, epoch: Epoch) -> ReceivedEvent {
        ReceivedEvent {
            event: Event {
                emitter,
                topics: vec![topic],
                data: vec![],
                block_number: epoch,
                log_index: 0,
                tx_index: 0,
            },
            block_epoch: epoch,
            block_hash: [0u8; 32],
            log_index: 0,
            received_at_ms: 0,
        }
    }

    fn transfer_topic() -> Hash {
        [1u8; 32]
    }

    fn stake_topic() -> Hash {
        [2u8; 32]
    }

    #[test]
    fn test_subscribe_and_poll_empty() {
        let mut client = EventClient::new(1000);
        let handle = client.subscribe(
            SubscriptionRequest::realtime(EventFilter::default())
        ).unwrap();
        let events = client.poll(handle, 10).unwrap();
        assert!(events.is_empty());
    }

    #[test]
    fn test_realtime_event_dispatch() {
        let mut client = EventClient::new(1000);
        let addr = test_addr(1);
        let filter = EventFilter { address: Some(addr), ..Default::default() };
        let handle = client.subscribe(SubscriptionRequest::realtime(filter)).unwrap();

        // Inject matching event.
        client.inject_event(test_event(addr, transfer_topic(), 100));
        // Inject non-matching event.
        client.inject_event(test_event(test_addr(2), transfer_topic(), 100));

        let events = client.poll(handle, 10).unwrap();
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].event.emitter, addr);
    }

    #[test]
    fn test_topic_filtering() {
        let mut client = EventClient::new(1000);
        let filter = EventFilter {
            topics: [Some(transfer_topic()), None, None, None],
            ..Default::default()
        };
        let handle = client.subscribe(SubscriptionRequest::realtime(filter)).unwrap();

        client.inject_event(test_event(test_addr(1), transfer_topic(), 10));
        client.inject_event(test_event(test_addr(1), stake_topic(), 10));

        let events = client.poll(handle, 10).unwrap();
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].event.topics[0], transfer_topic());
    }

    #[test]
    fn test_multiple_subscriptions() {
        let mut client = EventClient::new(1000);
        let h1 = client.subscribe(
            SubscriptionRequest::realtime(EventFilter { address: Some(test_addr(1)), ..Default::default() })
        ).unwrap();
        let h2 = client.subscribe(
            SubscriptionRequest::realtime(EventFilter { address: Some(test_addr(2)), ..Default::default() })
        ).unwrap();

        client.inject_event(test_event(test_addr(1), transfer_topic(), 1));
        client.inject_event(test_event(test_addr(2), transfer_topic(), 1));

        assert_eq!(client.poll(h1, 10).unwrap().len(), 1);
        assert_eq!(client.poll(h2, 10).unwrap().len(), 1);
    }

    #[test]
    fn test_unsubscribe() {
        let mut client = EventClient::new(1000);
        let handle = client.subscribe(
            SubscriptionRequest::realtime(EventFilter::default())
        ).unwrap();
        assert_eq!(client.active_count(), 1);

        client.unsubscribe(handle).unwrap();
        assert_eq!(client.active_count(), 0);
        assert!(client.poll(handle, 10).is_err());
    }

    #[test]
    fn test_buffer_overflow_drops_oldest() {
        let mut client = EventClient::new(1000);
        let mut req = SubscriptionRequest::realtime(EventFilter::default());
        req.buffer_size = 2;
        let handle = client.subscribe(req).unwrap();

        client.inject_event(test_event(test_addr(1), transfer_topic(), 1));
        client.inject_event(test_event(test_addr(1), transfer_topic(), 2));
        client.inject_event(test_event(test_addr(1), transfer_topic(), 3));

        let stats = client.sub_stats(handle).unwrap();
        assert_eq!(stats.events_dropped, 1);

        let events = client.poll(handle, 10).unwrap();
        assert_eq!(events.len(), 2);
        assert_eq!(events[0].block_epoch, 2); // oldest dropped
    }

    #[test]
    fn test_historical_replay() {
        let mut client = EventClient::new(1000);

        // Add historical events.
        client.add_historical_events(vec![
            test_event(test_addr(1), transfer_topic(), 10),
            test_event(test_addr(1), transfer_topic(), 20),
            test_event(test_addr(1), stake_topic(), 15),
        ]);

        let filter = EventFilter {
            topics: [Some(transfer_topic()), None, None, None],
            ..Default::default()
        };
        let handle = client.subscribe(
            SubscriptionRequest::historical(filter, 5, 25)
        ).unwrap();

        client.process_replay();

        let events = client.poll(handle, 10).unwrap();
        assert_eq!(events.len(), 2); // only transfer events
        let stats = client.sub_stats(handle).unwrap();
        assert_eq!(stats.state, SubState::Completed);
    }

    #[test]
    fn test_replay_then_realtime() {
        let mut client = EventClient::new(1000);

        // Historical events.
        client.add_historical_events(vec![
            test_event(test_addr(1), transfer_topic(), 10),
        ]);

        let handle = client.subscribe(
            SubscriptionRequest::replay_then_realtime(EventFilter::default(), 1)
        ).unwrap();

        assert_eq!(client.sub_stats(handle).unwrap().state, SubState::Replaying);
        client.process_replay();
        assert_eq!(client.sub_stats(handle).unwrap().state, SubState::Live);

        // Replayed events should be in buffer.
        let replayed = client.poll(handle, 100).unwrap();
        assert_eq!(replayed.len(), 1);
        assert_eq!(replayed[0].block_epoch, 10);

        // Now inject a live event — should arrive since state is Live.
        client.inject_event(test_event(test_addr(1), transfer_topic(), 50));
        let live = client.poll(handle, 100).unwrap();
        assert_eq!(live.len(), 1);
        assert_eq!(live[0].block_epoch, 50);
    }

    #[test]
    fn test_cache_query() {
        let mut client = EventClient::new(1000);
        client.inject_event(test_event(test_addr(1), transfer_topic(), 10));
        client.inject_event(test_event(test_addr(2), stake_topic(), 20));

        let filter = EventFilter { address: Some(test_addr(1)), ..Default::default() };
        let results = client.query_cache(1, 30, &filter);
        assert_eq!(results.len(), 1);
    }

    #[test]
    fn test_cache_eviction() {
        let mut cache = EventCache::new(2);
        cache.insert(test_event(test_addr(1), transfer_topic(), 1));
        cache.insert(test_event(test_addr(1), transfer_topic(), 2));
        let evicted = cache.insert(test_event(test_addr(1), transfer_topic(), 3));
        assert!(evicted > 0);
        assert!(cache.total_cached() <= 2);
    }

    #[test]
    fn test_disconnect_errors_subscriptions() {
        let mut client = EventClient::new(1000);
        let handle = client.subscribe(
            SubscriptionRequest::realtime(EventFilter::default())
        ).unwrap();

        client.disconnect();
        assert_eq!(client.sub_stats(handle).unwrap().state, SubState::Errored);
        assert!(client.subscribe(SubscriptionRequest::realtime(EventFilter::default())).is_err());

        client.reconnect();
        assert!(client.subscribe(SubscriptionRequest::realtime(EventFilter::default())).is_ok());
    }

    #[test]
    fn test_not_found_errors() {
        let mut client = EventClient::new(1000);
        let bad_handle = SubscriptionHandle(999);
        assert_eq!(
            client.poll(bad_handle, 10),
            Err(EventClientError::NotFound(bad_handle))
        );
        assert_eq!(
            client.unsubscribe(bad_handle),
            Err(EventClientError::NotFound(bad_handle))
        );
    }

    #[test]
    fn test_reset_clears_everything() {
        let mut client = EventClient::new(1000);
        client.subscribe(SubscriptionRequest::realtime(EventFilter::default())).unwrap();
        client.inject_event(test_event(test_addr(1), transfer_topic(), 1));
        assert!(client.active_count() > 0);
        assert!(client.cache_size() > 0);

        client.reset();
        assert_eq!(client.active_count(), 0);
        assert_eq!(client.cache_size(), 0);
    }

    #[test]
    fn test_sub_stats() {
        let mut client = EventClient::new(1000);
        let handle = client.subscribe(
            SubscriptionRequest::realtime(EventFilter::default())
        ).unwrap();

        client.inject_event(test_event(test_addr(1), transfer_topic(), 1));
        client.inject_event(test_event(test_addr(1), transfer_topic(), 2));

        let stats = client.sub_stats(handle).unwrap();
        assert_eq!(stats.events_received, 2);
        assert_eq!(stats.buffer_len, 2);
        assert_eq!(stats.state, SubState::Live);
    }
}
