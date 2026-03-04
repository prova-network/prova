//! P2P Network — gossip-based message propagation and peer management.
//!
//! Implements the networking layer from SPEC-007:
//! - Topic-based gossipsub (commits, challenges, bisection, proofs, blocks)
//! - Peer discovery and management
//! - Message deduplication
//! - Pluggable transport (mock for testing, real TCP/QUIC for production)
//!
//! This is a scaffold: no real networking deps. The transport trait allows
//! swapping in libp2p or custom TCP later.

use prova_chain::types::*;
use sha2::{Digest, Sha256};
use std::collections::{HashMap, HashSet, VecDeque};

/// Unique peer identifier (SHA-256 of public key in production).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PeerId(pub [u8; 32]);

impl PeerId {
    /// Create a test peer ID from a single byte.
    pub fn test(id: u8) -> Self {
        let mut bytes = [0u8; 32];
        bytes[0] = id;
        Self(bytes)
    }

    /// Create from raw bytes.
    pub fn from_bytes(bytes: [u8; 32]) -> Self {
        Self(bytes)
    }
}

impl std::fmt::Display for PeerId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "peer-{:02x}{:02x}", self.0[0], self.0[1])
    }
}

/// Gossip topics for Prova network.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Topic {
    /// `/prova/1/commits` — inference commit announcements.
    Commits,
    /// `/prova/1/challenges` — dispute challenges.
    Challenges,
    /// `/prova/1/bisection` — bisection game moves.
    Bisection,
    /// `/prova/1/proofs` — PDP proof submissions.
    Proofs,
    /// `/prova/1/audits` — audit reports.
    Audits,
    /// `/prova/1/blocks` — new blocks.
    Blocks,
}

impl Topic {
    /// Topic string for wire protocol.
    pub fn as_str(&self) -> &'static str {
        match self {
            Topic::Commits => "/prova/1/commits",
            Topic::Challenges => "/prova/1/challenges",
            Topic::Bisection => "/prova/1/bisection",
            Topic::Proofs => "/prova/1/proofs",
            Topic::Audits => "/prova/1/audits",
            Topic::Blocks => "/prova/1/blocks",
        }
    }

    /// All topics.
    pub fn all() -> &'static [Topic] {
        &[
            Topic::Commits,
            Topic::Challenges,
            Topic::Bisection,
            Topic::Proofs,
            Topic::Audits,
            Topic::Blocks,
        ]
    }
}

/// Network message envelope — wraps typed payloads with routing metadata.
#[derive(Debug, Clone)]
pub struct NetworkMessage {
    /// Unique message ID for deduplication.
    pub id: Hash,
    /// Who sent it.
    pub sender: PeerId,
    /// Which topic this belongs to.
    pub topic: Topic,
    /// The payload.
    pub payload: MessagePayload,
    /// Hop count (incremented at each relay).
    pub hops: u8,
    /// Max TTL in hops.
    pub max_hops: u8,
}

impl NetworkMessage {
    /// Create a new message with auto-generated ID.
    pub fn new(sender: PeerId, topic: Topic, payload: MessagePayload) -> Self {
        let id = Self::compute_id(&sender, &payload);
        let max_hops = match topic {
            Topic::Blocks => 10,
            Topic::Challenges | Topic::Bisection => 8,
            _ => 6,
        };
        Self {
            id,
            sender,
            topic,
            payload,
            hops: 0,
            max_hops,
        }
    }

    fn compute_id(sender: &PeerId, payload: &MessagePayload) -> Hash {
        let mut hasher = Sha256::new();
        hasher.update(sender.0);
        hasher.update(payload.tag_byte().to_le_bytes());
        // Include a nonce from payload content
        match payload {
            MessagePayload::InferenceCommit { activation_root, .. } => {
                hasher.update(activation_root);
            }
            MessagePayload::Challenge { commit_id, .. } => {
                hasher.update(commit_id.0.to_le_bytes());
            }
            MessagePayload::BisectionMove { dispute_id, layer, hash, .. } => {
                hasher.update(dispute_id.to_le_bytes());
                hasher.update(layer.to_le_bytes());
                hasher.update(hash);
            }
            MessagePayload::PdpProof { proof_set_id, .. } => {
                hasher.update(proof_set_id.to_le_bytes());
            }
            MessagePayload::AuditReport { target, epoch, .. } => {
                hasher.update(target.0);
                hasher.update(epoch.to_le_bytes());
            }
            MessagePayload::NewBlock { epoch, block_hash, .. } => {
                hasher.update(epoch.to_le_bytes());
                hasher.update(block_hash);
            }
        }
        hasher.finalize().into()
    }

    /// Whether this message should still be relayed.
    pub fn should_relay(&self) -> bool {
        self.hops < self.max_hops
    }

    /// Create a relayed copy (increment hop count).
    pub fn relay(&self) -> Self {
        let mut relayed = self.clone();
        relayed.hops += 1;
        relayed
    }
}

/// Typed message payloads.
#[derive(Debug, Clone)]
pub enum MessagePayload {
    InferenceCommit {
        provider: Address,
        model_id: ModelId,
        arch_group: ArchGroup,
        input_hash: Hash,
        activation_root: Hash,
        leaf_count: u32,
    },
    Challenge {
        challenger: Address,
        commit_id: CommitId,
        challenger_root: Hash,
    },
    BisectionMove {
        dispute_id: u64,
        mover: Address,
        layer: u32,
        hash: Hash,
    },
    PdpProof {
        provider: Address,
        proof_set_id: u64,
        challenged_roots: Vec<u32>,
    },
    AuditReport {
        auditor: Address,
        target: Address,
        epoch: Epoch,
        passed: bool,
    },
    NewBlock {
        epoch: Epoch,
        block_hash: Hash,
        producer: Address,
        tx_count: u32,
    },
}

impl MessagePayload {
    fn tag_byte(&self) -> u8 {
        match self {
            MessagePayload::InferenceCommit { .. } => 0x01,
            MessagePayload::Challenge { .. } => 0x02,
            MessagePayload::BisectionMove { .. } => 0x03,
            MessagePayload::PdpProof { .. } => 0x04,
            MessagePayload::AuditReport { .. } => 0x05,
            MessagePayload::NewBlock { .. } => 0x06,
        }
    }
}

/// Peer connection state.
#[derive(Debug, Clone)]
pub struct PeerInfo {
    pub id: PeerId,
    /// Topics this peer is subscribed to.
    pub subscriptions: HashSet<Topic>,
    /// Messages received from this peer (for rate limiting / scoring).
    pub messages_received: u64,
    /// Whether this peer is currently connected.
    pub connected: bool,
    /// Peer score (higher = better, used for relay priority).
    pub score: i32,
}

impl PeerInfo {
    pub fn new(id: PeerId) -> Self {
        Self {
            id,
            subscriptions: HashSet::new(),
            messages_received: 0,
            connected: true,
            score: 0,
        }
    }
}

/// The local node's network state.
#[derive(Debug)]
pub struct NetworkNode {
    /// This node's peer ID.
    pub local_id: PeerId,
    /// Connected peers.
    peers: HashMap<PeerId, PeerInfo>,
    /// Topic subscriptions for this node.
    subscriptions: HashSet<Topic>,
    /// Message deduplication cache (message ID → seen).
    seen_messages: HashSet<Hash>,
    /// Inbound message queue (received from peers, waiting for processing).
    inbound: VecDeque<NetworkMessage>,
    /// Outbound message queue (to be sent to peers).
    outbound: VecDeque<(PeerId, NetworkMessage)>,
    /// Maximum peers to connect to.
    max_peers: usize,
    /// Maximum seen messages cache size.
    max_seen_cache: usize,
}

impl NetworkNode {
    pub fn new(local_id: PeerId, max_peers: usize) -> Self {
        Self {
            local_id,
            peers: HashMap::new(),
            subscriptions: HashSet::new(),
            seen_messages: HashSet::new(),
            inbound: VecDeque::new(),
            outbound: VecDeque::new(),
            max_peers,
            max_seen_cache: 10_000,
        }
    }

    /// Subscribe to a topic.
    pub fn subscribe(&mut self, topic: Topic) {
        self.subscriptions.insert(topic);
    }

    /// Subscribe to all topics.
    pub fn subscribe_all(&mut self) {
        for topic in Topic::all() {
            self.subscriptions.insert(*topic);
        }
    }

    /// Check if subscribed to a topic.
    pub fn is_subscribed(&self, topic: &Topic) -> bool {
        self.subscriptions.contains(topic)
    }

    /// Add a peer connection.
    pub fn connect_peer(&mut self, peer_id: PeerId) -> Result<(), NetworkError> {
        if peer_id == self.local_id {
            return Err(NetworkError::SelfConnection);
        }
        if self.peers.len() >= self.max_peers {
            return Err(NetworkError::MaxPeersReached);
        }
        if self.peers.contains_key(&peer_id) {
            return Err(NetworkError::AlreadyConnected);
        }
        self.peers.insert(peer_id, PeerInfo::new(peer_id));
        Ok(())
    }

    /// Disconnect a peer.
    pub fn disconnect_peer(&mut self, peer_id: &PeerId) -> bool {
        self.peers.remove(peer_id).is_some()
    }

    /// Number of connected peers.
    pub fn peer_count(&self) -> usize {
        self.peers.len()
    }

    /// Get peer info.
    pub fn peer(&self, id: &PeerId) -> Option<&PeerInfo> {
        self.peers.get(id)
    }

    /// Publish a message to a topic (gossip to all subscribed peers).
    pub fn publish(&mut self, topic: Topic, payload: MessagePayload) -> Hash {
        let msg = NetworkMessage::new(self.local_id, topic, payload);
        let msg_id = msg.id;

        // Mark as seen (don't re-process our own messages)
        self.seen_messages.insert(msg_id);
        self.gc_seen_cache();

        // Queue for all connected peers (topic filtering happens at delivery)
        for peer_id in self.peers.keys().copied().collect::<Vec<_>>() {
            self.outbound.push_back((peer_id, msg.clone()));
        }

        msg_id
    }

    /// Receive a message from a peer.
    pub fn receive(&mut self, from: PeerId, msg: NetworkMessage) -> ReceiveResult {
        // Dedup
        if self.seen_messages.contains(&msg.id) {
            return ReceiveResult::Duplicate;
        }

        // Check if we care about this topic
        if !self.subscriptions.contains(&msg.topic) {
            return ReceiveResult::Ignored;
        }

        // Update peer stats
        if let Some(peer) = self.peers.get_mut(&from) {
            peer.messages_received += 1;
        }

        // Mark as seen
        self.seen_messages.insert(msg.id);
        self.gc_seen_cache();

        // Queue for local processing
        self.inbound.push_back(msg.clone());

        // Relay to other peers if TTL allows
        if msg.should_relay() {
            let relayed = msg.relay();
            for peer_id in self.peers.keys().copied().collect::<Vec<_>>() {
                if peer_id != from {
                    self.outbound.push_back((peer_id, relayed.clone()));
                }
            }
            ReceiveResult::AcceptedAndRelayed
        } else {
            ReceiveResult::Accepted
        }
    }

    /// Poll the next inbound message for processing.
    pub fn poll_inbound(&mut self) -> Option<NetworkMessage> {
        self.inbound.pop_front()
    }

    /// Poll the next outbound message for sending.
    pub fn poll_outbound(&mut self) -> Option<(PeerId, NetworkMessage)> {
        self.outbound.pop_front()
    }

    /// Number of pending inbound messages.
    pub fn inbound_len(&self) -> usize {
        self.inbound.len()
    }

    /// Number of pending outbound messages.
    pub fn outbound_len(&self) -> usize {
        self.outbound.len()
    }

    /// Garbage-collect the seen message cache.
    fn gc_seen_cache(&mut self) {
        if self.seen_messages.len() > self.max_seen_cache {
            // Simple strategy: clear half the cache
            // Real impl would use LRU or time-based eviction
            let to_remove: Vec<Hash> = self.seen_messages.iter().take(self.max_seen_cache / 2).copied().collect();
            for hash in to_remove {
                self.seen_messages.remove(&hash);
            }
        }
    }
}

/// Result of receiving a message.
#[derive(Debug, PartialEq, Eq)]
pub enum ReceiveResult {
    /// Message was new, processed, and relayed to peers.
    AcceptedAndRelayed,
    /// Message was new and processed but not relayed (TTL exhausted).
    Accepted,
    /// Message was already seen (dedup hit).
    Duplicate,
    /// Message was for a topic we're not subscribed to.
    Ignored,
}

/// Network errors.
#[derive(Debug, PartialEq, Eq)]
pub enum NetworkError {
    SelfConnection,
    MaxPeersReached,
    AlreadyConnected,
    PeerNotFound,
}

/// A simulated network of multiple nodes — for testing gossip propagation.
#[derive(Debug)]
pub struct SimulatedNetwork {
    nodes: HashMap<PeerId, NetworkNode>,
}

impl SimulatedNetwork {
    pub fn new() -> Self {
        Self {
            nodes: HashMap::new(),
        }
    }

    /// Add a node to the simulated network.
    pub fn add_node(&mut self, node: NetworkNode) {
        self.nodes.insert(node.local_id, node);
    }

    /// Connect two nodes bidirectionally.
    pub fn connect(&mut self, a: PeerId, b: PeerId) -> Result<(), NetworkError> {
        // Connect a → b
        self.nodes
            .get_mut(&a)
            .ok_or(NetworkError::PeerNotFound)?
            .connect_peer(b)?;
        // Connect b → a
        self.nodes
            .get_mut(&b)
            .ok_or(NetworkError::PeerNotFound)?
            .connect_peer(a)?;
        Ok(())
    }

    /// Run one round of message propagation.
    /// Drains all outbound queues and delivers to connected peers.
    /// Returns the number of messages delivered.
    pub fn propagate(&mut self) -> usize {
        let mut pending: Vec<(PeerId, PeerId, NetworkMessage)> = Vec::new();

        // Collect all outbound messages
        for (_, node) in self.nodes.iter_mut() {
            while let Some((dest, msg)) = node.poll_outbound() {
                pending.push((node.local_id, dest, msg));
            }
        }

        let count = pending.len();

        // Deliver to destinations
        for (from, to, msg) in pending {
            if let Some(dest_node) = self.nodes.get_mut(&to) {
                dest_node.receive(from, msg);
            }
        }

        count
    }

    /// Run propagation until no more messages are in flight.
    /// Returns total messages delivered across all rounds.
    pub fn propagate_until_quiet(&mut self) -> usize {
        let mut total = 0;
        loop {
            let delivered = self.propagate();
            if delivered == 0 {
                break;
            }
            total += delivered;
        }
        total
    }

    /// Get a node.
    pub fn node(&self, id: &PeerId) -> Option<&NetworkNode> {
        self.nodes.get(id)
    }

    /// Get a mutable node.
    pub fn node_mut(&mut self, id: &PeerId) -> Option<&mut NetworkNode> {
        self.nodes.get_mut(id)
    }

    /// Number of nodes.
    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }
}

impl Default for SimulatedNetwork {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_node(id: u8) -> NetworkNode {
        let mut node = NetworkNode::new(PeerId::test(id), 50);
        node.subscribe_all();
        node
    }

    #[test]
    fn test_peer_connection() {
        let mut node = make_node(1);
        assert_eq!(node.peer_count(), 0);

        node.connect_peer(PeerId::test(2)).unwrap();
        assert_eq!(node.peer_count(), 1);

        node.connect_peer(PeerId::test(3)).unwrap();
        assert_eq!(node.peer_count(), 2);
    }

    #[test]
    fn test_self_connection_rejected() {
        let mut node = make_node(1);
        assert_eq!(
            node.connect_peer(PeerId::test(1)),
            Err(NetworkError::SelfConnection)
        );
    }

    #[test]
    fn test_duplicate_connection_rejected() {
        let mut node = make_node(1);
        node.connect_peer(PeerId::test(2)).unwrap();
        assert_eq!(
            node.connect_peer(PeerId::test(2)),
            Err(NetworkError::AlreadyConnected)
        );
    }

    #[test]
    fn test_max_peers() {
        let mut node = NetworkNode::new(PeerId::test(1), 2);
        node.connect_peer(PeerId::test(2)).unwrap();
        node.connect_peer(PeerId::test(3)).unwrap();
        assert_eq!(
            node.connect_peer(PeerId::test(4)),
            Err(NetworkError::MaxPeersReached)
        );
    }

    #[test]
    fn test_publish_creates_outbound() {
        let mut node = make_node(1);
        node.connect_peer(PeerId::test(2)).unwrap();
        node.connect_peer(PeerId::test(3)).unwrap();

        node.publish(
            Topic::Commits,
            MessagePayload::InferenceCommit {
                provider: Address::test(1),
                model_id: ModelId([0x42; 32]),
                arch_group: ArchGroup::new("nvidia-sm89"),
                input_hash: [0xAA; 32],
                activation_root: [0xBB; 32],
                leaf_count: 33,
            },
        );

        // Should have 2 outbound messages (one per peer)
        assert_eq!(node.outbound_len(), 2);
    }

    #[test]
    fn test_receive_and_dedup() {
        let mut node = make_node(1);
        node.connect_peer(PeerId::test(2)).unwrap();

        let msg = NetworkMessage::new(
            PeerId::test(2),
            Topic::Blocks,
            MessagePayload::NewBlock {
                epoch: 100,
                block_hash: [0xFF; 32],
                producer: Address::test(2),
                tx_count: 5,
            },
        );

        // First receive: accepted
        let result = node.receive(PeerId::test(2), msg.clone());
        assert_eq!(result, ReceiveResult::AcceptedAndRelayed);
        assert_eq!(node.inbound_len(), 1);

        // Second receive: duplicate
        let result = node.receive(PeerId::test(2), msg);
        assert_eq!(result, ReceiveResult::Duplicate);
        assert_eq!(node.inbound_len(), 1); // still 1
    }

    #[test]
    fn test_unsubscribed_topic_ignored() {
        let mut node = NetworkNode::new(PeerId::test(1), 50);
        // Only subscribe to blocks
        node.subscribe(Topic::Blocks);
        node.connect_peer(PeerId::test(2)).unwrap();

        let msg = NetworkMessage::new(
            PeerId::test(2),
            Topic::Commits, // not subscribed
            MessagePayload::InferenceCommit {
                provider: Address::test(2),
                model_id: ModelId([0; 32]),
                arch_group: ArchGroup::new("test"),
                input_hash: [0; 32],
                activation_root: [0; 32],
                leaf_count: 1,
            },
        );

        assert_eq!(node.receive(PeerId::test(2), msg), ReceiveResult::Ignored);
        assert_eq!(node.inbound_len(), 0);
    }

    #[test]
    fn test_message_relay_increments_hops() {
        let msg = NetworkMessage::new(
            PeerId::test(1),
            Topic::Challenges,
            MessagePayload::Challenge {
                challenger: Address::test(1),
                commit_id: CommitId(42),
                challenger_root: [0xCC; 32],
            },
        );

        assert_eq!(msg.hops, 0);
        let relayed = msg.relay();
        assert_eq!(relayed.hops, 1);
        assert_eq!(relayed.id, msg.id); // ID preserved
    }

    #[test]
    fn test_ttl_exhaustion() {
        let mut msg = NetworkMessage::new(
            PeerId::test(1),
            Topic::Proofs,
            MessagePayload::PdpProof {
                provider: Address::test(1),
                proof_set_id: 1,
                challenged_roots: vec![0, 5, 10],
            },
        );

        assert!(msg.should_relay());

        // Exhaust the TTL
        msg.hops = msg.max_hops;
        assert!(!msg.should_relay());
    }

    // --- SimulatedNetwork tests ---

    #[test]
    fn test_simulated_two_nodes() {
        let mut net = SimulatedNetwork::new();
        net.add_node(make_node(1));
        net.add_node(make_node(2));
        net.connect(PeerId::test(1), PeerId::test(2)).unwrap();

        // Node 1 publishes a block
        net.node_mut(&PeerId::test(1)).unwrap().publish(
            Topic::Blocks,
            MessagePayload::NewBlock {
                epoch: 1,
                block_hash: [0x11; 32],
                producer: Address::test(1),
                tx_count: 0,
            },
        );

        let delivered = net.propagate();
        assert_eq!(delivered, 1);

        // Node 2 should have received it
        let node2 = net.node(&PeerId::test(2)).unwrap();
        assert_eq!(node2.inbound_len(), 1);
    }

    #[test]
    fn test_simulated_three_node_gossip() {
        let mut net = SimulatedNetwork::new();
        net.add_node(make_node(1));
        net.add_node(make_node(2));
        net.add_node(make_node(3));

        // Linear topology: 1 -- 2 -- 3
        net.connect(PeerId::test(1), PeerId::test(2)).unwrap();
        net.connect(PeerId::test(2), PeerId::test(3)).unwrap();

        // Node 1 publishes
        net.node_mut(&PeerId::test(1)).unwrap().publish(
            Topic::Commits,
            MessagePayload::InferenceCommit {
                provider: Address::test(1),
                model_id: ModelId([0x42; 32]),
                arch_group: ArchGroup::new("nvidia-sm89"),
                input_hash: [0xAA; 32],
                activation_root: [0xBB; 32],
                leaf_count: 33,
            },
        );

        // Propagate until quiet
        let total = net.propagate_until_quiet();
        assert!(total >= 2, "message should reach node 3 via node 2");

        // Node 3 should have received it (relayed through node 2)
        let node3 = net.node(&PeerId::test(3)).unwrap();
        assert_eq!(node3.inbound_len(), 1);
    }

    #[test]
    fn test_simulated_star_topology() {
        let mut net = SimulatedNetwork::new();

        // Node 0 is hub, nodes 1-5 are spokes
        for i in 0..6u8 {
            net.add_node(make_node(i));
        }
        for i in 1..6u8 {
            net.connect(PeerId::test(0), PeerId::test(i)).unwrap();
        }

        // Node 1 publishes an audit report
        net.node_mut(&PeerId::test(1)).unwrap().publish(
            Topic::Audits,
            MessagePayload::AuditReport {
                auditor: Address::test(1),
                target: Address::test(2),
                epoch: 500,
                passed: true,
            },
        );

        net.propagate_until_quiet();

        // All nodes except 1 should have received it
        for i in 0..6u8 {
            if i == 1 { continue; } // sender
            let node = net.node(&PeerId::test(i)).unwrap();
            assert_eq!(
                node.inbound_len(), 1,
                "node {i} should have received the audit report"
            );
        }
    }

    #[test]
    fn test_simulated_dedup_prevents_loops() {
        let mut net = SimulatedNetwork::new();

        // Triangle: 1 -- 2 -- 3 -- 1
        for i in 1..=3u8 {
            net.add_node(make_node(i));
        }
        net.connect(PeerId::test(1), PeerId::test(2)).unwrap();
        net.connect(PeerId::test(2), PeerId::test(3)).unwrap();
        net.connect(PeerId::test(3), PeerId::test(1)).unwrap();

        // Node 1 publishes
        net.node_mut(&PeerId::test(1)).unwrap().publish(
            Topic::Blocks,
            MessagePayload::NewBlock {
                epoch: 42,
                block_hash: [0x99; 32],
                producer: Address::test(1),
                tx_count: 3,
            },
        );

        // Should converge (no infinite loops)
        let total = net.propagate_until_quiet();
        assert!(total < 20, "dedup should prevent message explosion, got {total}");

        // Both other nodes received exactly 1 copy
        for i in 2..=3u8 {
            let node = net.node(&PeerId::test(i)).unwrap();
            assert_eq!(node.inbound_len(), 1);
        }
    }

    #[test]
    fn test_disconnect_peer() {
        let mut node = make_node(1);
        node.connect_peer(PeerId::test(2)).unwrap();
        assert_eq!(node.peer_count(), 1);

        assert!(node.disconnect_peer(&PeerId::test(2)));
        assert_eq!(node.peer_count(), 0);

        // Disconnect non-existent peer
        assert!(!node.disconnect_peer(&PeerId::test(99)));
    }

    #[test]
    fn test_topic_subscription() {
        let mut node = NetworkNode::new(PeerId::test(1), 50);
        assert!(!node.is_subscribed(&Topic::Blocks));

        node.subscribe(Topic::Blocks);
        assert!(node.is_subscribed(&Topic::Blocks));
        assert!(!node.is_subscribed(&Topic::Commits));

        node.subscribe_all();
        for topic in Topic::all() {
            assert!(node.is_subscribed(topic));
        }
    }
}
