//! Event subscription engine — WebSocket push + filter subscriptions.
//!
//! Allows clients to subscribe to on-chain events with filters and receive
//! real-time notifications. Designed for integration with the JSON-RPC layer.
//!
//! ## Features
//!
//! - Subscription lifecycle (create, notify, cancel, expire)
//! - Filter-based matching (address, topics, block range)
//! - Subscription limits per client (DoS protection)
//! - Backpressure: bounded notification queue per subscriber
//! - Replay: catch-up from a given block on subscribe
//! - Heartbeat / keepalive detection
//! - Batch notification delivery

use std::collections::{HashMap, VecDeque};
use std::time::{Duration, Instant};

use prova_chain::events::{Event, EventFilter};
use prova_chain::types::{Address, Epoch, Hash};

// ---------------------------------------------------------------------------
// Subscription types
// ---------------------------------------------------------------------------

/// Unique subscription identifier.
pub type SubscriptionId = u64;

/// Unique client (connection) identifier.
pub type ClientId = u64;

/// Maximum subscriptions per client.
pub const MAX_SUBS_PER_CLIENT: usize = 32;

/// Maximum pending notifications per subscription before backpressure.
pub const MAX_PENDING_NOTIFICATIONS: usize = 1024;

/// Default subscription TTL (1 hour).
pub const DEFAULT_TTL_SECS: u64 = 3600;

/// Keepalive interval.
pub const KEEPALIVE_INTERVAL_SECS: u64 = 30;

/// A notification pushed to a subscriber.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Notification {
    pub subscription_id: SubscriptionId,
    pub event: Event,
}

/// Subscription state.
#[derive(Debug, Clone)]
pub struct Subscription {
    pub id: SubscriptionId,
    pub client_id: ClientId,
    pub filter: EventFilter,
    pub created_at: Instant,
    pub ttl: Duration,
    pub last_activity: Instant,
    /// Last block number delivered to this subscription (for replay).
    pub cursor: Option<Epoch>,
    pub active: bool,
}

impl Subscription {
    pub fn is_expired(&self, now: Instant) -> bool {
        now.duration_since(self.created_at) > self.ttl
    }

    pub fn needs_keepalive(&self, now: Instant) -> bool {
        now.duration_since(self.last_activity) > Duration::from_secs(KEEPALIVE_INTERVAL_SECS)
    }
}

/// Client connection state.
#[derive(Debug)]
pub struct ClientState {
    pub client_id: ClientId,
    pub subscriptions: Vec<SubscriptionId>,
    pub connected_at: Instant,
    pub last_seen: Instant,
}

// ---------------------------------------------------------------------------
// Subscription engine
// ---------------------------------------------------------------------------

/// The core subscription manager. Handles subscription lifecycle, event
/// fanout, backpressure, and expiry.
#[derive(Debug)]
pub struct SubscriptionEngine {
    next_sub_id: SubscriptionId,
    next_client_id: ClientId,
    subscriptions: HashMap<SubscriptionId, Subscription>,
    clients: HashMap<ClientId, ClientState>,
    /// Per-subscription notification queue (bounded).
    queues: HashMap<SubscriptionId, VecDeque<Notification>>,
    /// Dropped notification count per subscription (backpressure metric).
    dropped: HashMap<SubscriptionId, u64>,
}

impl SubscriptionEngine {
    pub fn new() -> Self {
        Self {
            next_sub_id: 1,
            next_client_id: 1,
            subscriptions: HashMap::new(),
            clients: HashMap::new(),
            queues: HashMap::new(),
            dropped: HashMap::new(),
        }
    }

    /// Register a new client connection. Returns a ClientId.
    pub fn connect(&mut self) -> ClientId {
        let id = self.next_client_id;
        self.next_client_id += 1;
        let now = Instant::now();
        self.clients.insert(
            id,
            ClientState {
                client_id: id,
                subscriptions: Vec::new(),
                connected_at: now,
                last_seen: now,
            },
        );
        id
    }

    /// Disconnect a client, removing all its subscriptions.
    pub fn disconnect(&mut self, client_id: ClientId) -> usize {
        let removed = if let Some(client) = self.clients.remove(&client_id) {
            let count = client.subscriptions.len();
            for sub_id in &client.subscriptions {
                self.subscriptions.remove(sub_id);
                self.queues.remove(sub_id);
                self.dropped.remove(sub_id);
            }
            count
        } else {
            0
        };
        removed
    }

    /// Create a subscription for a client. Returns the subscription ID
    /// or an error if the client has too many subscriptions.
    pub fn subscribe(
        &mut self,
        client_id: ClientId,
        filter: EventFilter,
        start_block: Option<Epoch>,
    ) -> Result<SubscriptionId, SubscribeError> {
        let client = self
            .clients
            .get_mut(&client_id)
            .ok_or(SubscribeError::ClientNotFound)?;

        if client.subscriptions.len() >= MAX_SUBS_PER_CLIENT {
            return Err(SubscribeError::TooManySubscriptions);
        }

        let id = self.next_sub_id;
        self.next_sub_id += 1;
        let now = Instant::now();

        let sub = Subscription {
            id,
            client_id,
            filter,
            created_at: now,
            ttl: Duration::from_secs(DEFAULT_TTL_SECS),
            last_activity: now,
            cursor: start_block,
            active: true,
        };

        self.subscriptions.insert(id, sub);
        self.queues.insert(id, VecDeque::new());
        self.dropped.insert(id, 0);
        client.subscriptions.push(id);

        Ok(id)
    }

    /// Cancel a subscription.
    pub fn unsubscribe(&mut self, sub_id: SubscriptionId) -> Result<(), SubscribeError> {
        let sub = self
            .subscriptions
            .remove(&sub_id)
            .ok_or(SubscribeError::NotFound)?;
        self.queues.remove(&sub_id);
        self.dropped.remove(&sub_id);

        if let Some(client) = self.clients.get_mut(&sub.client_id) {
            client.subscriptions.retain(|s| *s != sub_id);
        }
        Ok(())
    }

    /// Fan out an event to all matching subscriptions.
    /// Returns the number of subscriptions that received the event.
    pub fn notify(&mut self, event: &Event) -> usize {
        let mut count = 0;
        let sub_ids: Vec<SubscriptionId> = self.subscriptions.keys().copied().collect();

        for sub_id in sub_ids {
            let matches = {
                let sub = &self.subscriptions[&sub_id];
                if !sub.active {
                    continue;
                }
                // Check cursor — only deliver events after the cursor block
                if let Some(cursor) = sub.cursor {
                    if event.block_number <= cursor {
                        continue;
                    }
                }
                Self::filter_matches(&sub.filter, event)
            };

            if matches {
                let queue = self.queues.get_mut(&sub_id).unwrap();
                if queue.len() >= MAX_PENDING_NOTIFICATIONS {
                    // Backpressure: drop oldest
                    queue.pop_front();
                    *self.dropped.get_mut(&sub_id).unwrap() += 1;
                }
                queue.push_back(Notification {
                    subscription_id: sub_id,
                    event: event.clone(),
                });
                // Update cursor
                if let Some(sub) = self.subscriptions.get_mut(&sub_id) {
                    sub.cursor = Some(event.block_number);
                    sub.last_activity = Instant::now();
                }
                count += 1;
            }
        }
        count
    }

    /// Drain pending notifications for a subscription.
    pub fn drain(&mut self, sub_id: SubscriptionId) -> Vec<Notification> {
        self.queues
            .get_mut(&sub_id)
            .map(|q| q.drain(..).collect())
            .unwrap_or_default()
    }

    /// Drain up to `max` notifications.
    pub fn drain_batch(&mut self, sub_id: SubscriptionId, max: usize) -> Vec<Notification> {
        self.queues
            .get_mut(&sub_id)
            .map(|q| {
                let n = max.min(q.len());
                q.drain(..n).collect()
            })
            .unwrap_or_default()
    }

    /// Get the number of pending notifications for a subscription.
    pub fn pending_count(&self, sub_id: SubscriptionId) -> usize {
        self.queues.get(&sub_id).map(|q| q.len()).unwrap_or(0)
    }

    /// Get the number of dropped notifications for a subscription.
    pub fn dropped_count(&self, sub_id: SubscriptionId) -> u64 {
        self.dropped.get(&sub_id).copied().unwrap_or(0)
    }

    /// Expire stale subscriptions. Returns list of expired sub IDs.
    pub fn expire_stale(&mut self) -> Vec<SubscriptionId> {
        let now = Instant::now();
        let expired: Vec<SubscriptionId> = self
            .subscriptions
            .iter()
            .filter(|(_, s)| s.is_expired(now))
            .map(|(id, _)| *id)
            .collect();

        for id in &expired {
            let _ = self.unsubscribe(*id);
        }
        expired
    }

    /// List subscriptions needing keepalive.
    pub fn needs_keepalive(&self) -> Vec<SubscriptionId> {
        let now = Instant::now();
        self.subscriptions
            .iter()
            .filter(|(_, s)| s.active && s.needs_keepalive(now))
            .map(|(id, _)| *id)
            .collect()
    }

    /// Touch a subscription (update last_activity for keepalive).
    pub fn touch(&mut self, sub_id: SubscriptionId) {
        if let Some(sub) = self.subscriptions.get_mut(&sub_id) {
            sub.last_activity = Instant::now();
        }
    }

    /// Total active subscriptions.
    pub fn active_count(&self) -> usize {
        self.subscriptions.values().filter(|s| s.active).count()
    }

    /// Total connected clients.
    pub fn client_count(&self) -> usize {
        self.clients.len()
    }

    /// Get subscription info.
    pub fn get_subscription(&self, sub_id: SubscriptionId) -> Option<&Subscription> {
        self.subscriptions.get(&sub_id)
    }

    /// Replay historical events into a subscription's queue.
    /// Caller provides the events (e.g., from EventStore query).
    pub fn replay(&mut self, sub_id: SubscriptionId, events: &[Event]) -> usize {
        let sub = match self.subscriptions.get(&sub_id) {
            Some(s) => s.clone(),
            None => return 0,
        };
        let mut count = 0;
        for event in events {
            if Self::filter_matches(&sub.filter, event) {
                let queue = self.queues.get_mut(&sub_id).unwrap();
                if queue.len() < MAX_PENDING_NOTIFICATIONS {
                    queue.push_back(Notification {
                        subscription_id: sub_id,
                        event: event.clone(),
                    });
                    count += 1;
                }
            }
        }
        // Update cursor to latest replayed
        if let Some(last) = events.last() {
            if let Some(sub) = self.subscriptions.get_mut(&sub_id) {
                sub.cursor = Some(last.block_number);
            }
        }
        count
    }

    /// Fan out a batch of events (e.g., all events in a new block).
    /// Returns total notifications delivered.
    pub fn notify_batch(&mut self, events: &[Event]) -> usize {
        let mut total = 0;
        for event in events {
            total += self.notify(event);
        }
        total
    }

    // Internal: check if filter matches event (static, no borrow issues).
    fn filter_matches(filter: &EventFilter, event: &Event) -> bool {
        if let Some(addr) = &filter.address {
            if &event.emitter != addr {
                return false;
            }
        }
        if let Some(from) = filter.from_block {
            if event.block_number < from {
                return false;
            }
        }
        if let Some(to) = filter.to_block {
            if event.block_number > to {
                return false;
            }
        }
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
}

// ---------------------------------------------------------------------------
// Errors
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SubscribeError {
    ClientNotFound,
    TooManySubscriptions,
    NotFound,
}

impl std::fmt::Display for SubscribeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ClientNotFound => write!(f, "client not found"),
            Self::TooManySubscriptions => write!(f, "too many subscriptions"),
            Self::NotFound => write!(f, "subscription not found"),
        }
    }
}

// ===========================================================================
// Tests
// ===========================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use prova_chain::events::event_types;

    fn make_event(emitter: Address, block: Epoch, topic0: Hash) -> Event {
        Event {
            emitter,
            topics: vec![topic0],
            data: vec![],
            block_number: block,
            log_index: 0,
            tx_index: 0,
        }
    }

    fn addr(n: u8) -> Address {
        let mut a = [0u8; 20];
        a[19] = n;
        Address(a)
    }

    #[test]
    fn test_connect_disconnect() {
        let mut engine = SubscriptionEngine::new();
        let c1 = engine.connect();
        let c2 = engine.connect();
        assert_eq!(engine.client_count(), 2);
        engine.disconnect(c1);
        assert_eq!(engine.client_count(), 1);
        engine.disconnect(c2);
        assert_eq!(engine.client_count(), 0);
    }

    #[test]
    fn test_subscribe_and_notify() {
        let mut engine = SubscriptionEngine::new();
        let client = engine.connect();
        let filter = EventFilter::new().address(addr(1));
        let sub = engine.subscribe(client, filter, None).unwrap();

        let ev = make_event(addr(1), 10, event_types::TRANSFER());
        assert_eq!(engine.notify(&ev), 1);
        assert_eq!(engine.pending_count(sub), 1);

        let notifs = engine.drain(sub);
        assert_eq!(notifs.len(), 1);
        assert_eq!(notifs[0].event.block_number, 10);
    }

    #[test]
    fn test_filter_no_match() {
        let mut engine = SubscriptionEngine::new();
        let client = engine.connect();
        let filter = EventFilter::new().address(addr(1));
        let sub = engine.subscribe(client, filter, None).unwrap();

        // Event from different address
        let ev = make_event(addr(2), 10, event_types::TRANSFER());
        assert_eq!(engine.notify(&ev), 0);
        assert_eq!(engine.pending_count(sub), 0);
    }

    #[test]
    fn test_topic_filter() {
        let mut engine = SubscriptionEngine::new();
        let client = engine.connect();
        let transfer_hash = event_types::TRANSFER();
        let filter = EventFilter::new().topic(0, transfer_hash);
        let sub = engine.subscribe(client, filter, None).unwrap();

        let ev_match = make_event(addr(1), 10, transfer_hash);
        let ev_no = make_event(addr(1), 11, event_types::SLASH());

        engine.notify(&ev_match);
        engine.notify(&ev_no);
        assert_eq!(engine.pending_count(sub), 1);
    }

    #[test]
    fn test_cursor_skip() {
        let mut engine = SubscriptionEngine::new();
        let client = engine.connect();
        let filter = EventFilter::new();
        // Start from block 5 — should skip blocks <= 5
        let sub = engine.subscribe(client, filter, Some(5)).unwrap();

        let ev_old = make_event(addr(1), 3, event_types::TRANSFER());
        let ev_new = make_event(addr(1), 6, event_types::TRANSFER());

        engine.notify(&ev_old);
        engine.notify(&ev_new);
        assert_eq!(engine.pending_count(sub), 1);
    }

    #[test]
    fn test_unsubscribe() {
        let mut engine = SubscriptionEngine::new();
        let client = engine.connect();
        let sub = engine.subscribe(client, EventFilter::new(), None).unwrap();
        assert_eq!(engine.active_count(), 1);
        engine.unsubscribe(sub).unwrap();
        assert_eq!(engine.active_count(), 0);
    }

    #[test]
    fn test_unsubscribe_not_found() {
        let mut engine = SubscriptionEngine::new();
        assert_eq!(engine.unsubscribe(999), Err(SubscribeError::NotFound));
    }

    #[test]
    fn test_max_subs_per_client() {
        let mut engine = SubscriptionEngine::new();
        let client = engine.connect();
        for _ in 0..MAX_SUBS_PER_CLIENT {
            engine.subscribe(client, EventFilter::new(), None).unwrap();
        }
        assert_eq!(
            engine.subscribe(client, EventFilter::new(), None),
            Err(SubscribeError::TooManySubscriptions)
        );
    }

    #[test]
    fn test_backpressure_drops_oldest() {
        let mut engine = SubscriptionEngine::new();
        let client = engine.connect();
        let sub = engine.subscribe(client, EventFilter::new(), None).unwrap();

        // Fill to max
        for i in 0..MAX_PENDING_NOTIFICATIONS {
            let ev = make_event(addr(1), i as Epoch + 1, event_types::TRANSFER());
            engine.notify(&ev);
        }
        assert_eq!(engine.pending_count(sub), MAX_PENDING_NOTIFICATIONS);
        assert_eq!(engine.dropped_count(sub), 0);

        // One more should drop oldest
        let ev = make_event(
            addr(1),
            MAX_PENDING_NOTIFICATIONS as Epoch + 1,
            event_types::TRANSFER(),
        );
        engine.notify(&ev);
        assert_eq!(engine.pending_count(sub), MAX_PENDING_NOTIFICATIONS);
        assert_eq!(engine.dropped_count(sub), 1);

        // First notification should be block 2 (block 1 was dropped)
        let notifs = engine.drain_batch(sub, 1);
        assert_eq!(notifs[0].event.block_number, 2);
    }

    #[test]
    fn test_drain_batch() {
        let mut engine = SubscriptionEngine::new();
        let client = engine.connect();
        let sub = engine.subscribe(client, EventFilter::new(), None).unwrap();

        for i in 1..=10 {
            let ev = make_event(addr(1), i, event_types::TRANSFER());
            engine.notify(&ev);
        }

        let batch = engine.drain_batch(sub, 3);
        assert_eq!(batch.len(), 3);
        assert_eq!(engine.pending_count(sub), 7);
    }

    #[test]
    fn test_disconnect_cleans_subs() {
        let mut engine = SubscriptionEngine::new();
        let client = engine.connect();
        engine.subscribe(client, EventFilter::new(), None).unwrap();
        engine.subscribe(client, EventFilter::new(), None).unwrap();
        assert_eq!(engine.active_count(), 2);

        let removed = engine.disconnect(client);
        assert_eq!(removed, 2);
        assert_eq!(engine.active_count(), 0);
    }

    #[test]
    fn test_replay_events() {
        let mut engine = SubscriptionEngine::new();
        let client = engine.connect();
        let filter = EventFilter::new().address(addr(1));
        let sub = engine.subscribe(client, filter, None).unwrap();

        let historical = vec![
            make_event(addr(1), 1, event_types::TRANSFER()),
            make_event(addr(2), 2, event_types::TRANSFER()), // different addr
            make_event(addr(1), 3, event_types::SLASH()),
        ];

        let replayed = engine.replay(sub, &historical);
        assert_eq!(replayed, 2); // only addr(1) events
        assert_eq!(engine.pending_count(sub), 2);
    }

    #[test]
    fn test_notify_batch() {
        let mut engine = SubscriptionEngine::new();
        let client = engine.connect();
        let sub = engine.subscribe(client, EventFilter::new(), None).unwrap();

        let events = vec![
            make_event(addr(1), 1, event_types::TRANSFER()),
            make_event(addr(2), 2, event_types::SLASH()),
            make_event(addr(3), 3, event_types::BLOCK_REWARD()),
        ];

        let delivered = engine.notify_batch(&events);
        assert_eq!(delivered, 3);
        assert_eq!(engine.pending_count(sub), 3);
    }

    #[test]
    fn test_multi_subscriber_fanout() {
        let mut engine = SubscriptionEngine::new();
        let c1 = engine.connect();
        let c2 = engine.connect();

        let s1 = engine
            .subscribe(c1, EventFilter::new().address(addr(1)), None)
            .unwrap();
        let s2 = engine.subscribe(c2, EventFilter::new(), None).unwrap(); // wildcard

        let ev = make_event(addr(1), 10, event_types::TRANSFER());
        let delivered = engine.notify(&ev);
        assert_eq!(delivered, 2);
        assert_eq!(engine.pending_count(s1), 1);
        assert_eq!(engine.pending_count(s2), 1);
    }

    #[test]
    fn test_block_range_filter() {
        let mut engine = SubscriptionEngine::new();
        let client = engine.connect();
        let filter = EventFilter::new().from_block(5).to_block(10);
        let sub = engine.subscribe(client, filter, None).unwrap();

        let ev_before = make_event(addr(1), 3, event_types::TRANSFER());
        let ev_in = make_event(addr(1), 7, event_types::TRANSFER());
        let ev_after = make_event(addr(1), 15, event_types::TRANSFER());

        engine.notify(&ev_before);
        engine.notify(&ev_in);
        engine.notify(&ev_after);

        assert_eq!(engine.pending_count(sub), 1);
        let notifs = engine.drain(sub);
        assert_eq!(notifs[0].event.block_number, 7);
    }

    #[test]
    fn test_subscribe_client_not_found() {
        let mut engine = SubscriptionEngine::new();
        assert_eq!(
            engine.subscribe(999, EventFilter::new(), None),
            Err(SubscribeError::ClientNotFound)
        );
    }
}
