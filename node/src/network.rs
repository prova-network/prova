// NODE-006: P2P Networking Scaffold
// Implements gossipsub topics, peer discovery, and message routing per SPEC-007.

use std::collections::{HashMap, HashSet, VecDeque};
use std::time::{Duration, Instant};

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

pub type PeerId = [u8; 32];
pub type Hash = [u8; 32];

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Topic {
    Commits,
    Challenges,
    Bisection,
    Proofs,
    Audits,
    Blocks,
}

impl Topic {
    pub fn path(&self) -> &'static str {
        match self {
            Topic::Commits => "/prova/1/commits",
            Topic::Challenges => "/prova/1/challenges",
            Topic::Bisection => "/prova/1/bisection",
            Topic::Proofs => "/prova/1/proofs",
            Topic::Audits => "/prova/1/audits",
            Topic::Blocks => "/prova/1/blocks",
        }
    }

    pub fn all() -> Vec<Topic> {
        vec![
            Topic::Commits,
            Topic::Challenges,
            Topic::Bisection,
            Topic::Proofs,
            Topic::Audits,
            Topic::Blocks,
        ]
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NodeRole {
    Provider,
    Challenger,
    LightClient,
}

impl NodeRole {
    pub fn subscriptions(&self) -> Vec<Topic> {
        match self {
            NodeRole::Provider => Topic::all(),
            NodeRole::Challenger => vec![
                Topic::Commits,
                Topic::Challenges,
                Topic::Bisection,
                Topic::Audits,
            ],
            NodeRole::LightClient => vec![Topic::Blocks],
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Priority {
    Critical,
    High,
    Normal,
}

#[derive(Debug, Clone)]
pub struct GossipMessage {
    pub id: Hash,
    pub topic: Topic,
    pub payload: Vec<u8>,
    pub priority: Priority,
    pub ttl: Duration,
    pub origin: PeerId,
    pub created_at: Instant,
}

impl GossipMessage {
    pub fn new(topic: Topic, payload: Vec<u8>, origin: PeerId) -> Self {
        let (priority, ttl) = match &topic {
            Topic::Blocks => (Priority::Critical, Duration::from_secs(300)),
            Topic::Challenges | Topic::Bisection => (Priority::High, Duration::from_secs(30)),
            _ => (Priority::Normal, Duration::from_secs(60)),
        };
        let mut id = [0u8; 32];
        // Simple hash: XOR origin with payload length for deterministic IDs
        for (i, b) in origin.iter().enumerate() {
            id[i] = b ^ (payload.len() as u8).wrapping_add(i as u8);
        }
        Self {
            id,
            topic,
            payload,
            priority,
            ttl,
            origin,
            created_at: Instant::now(),
        }
    }

    pub fn is_expired(&self) -> bool {
        self.created_at.elapsed() > self.ttl
    }
}

// ---------------------------------------------------------------------------
// Kademlia DHT (simplified)
// ---------------------------------------------------------------------------

const K_BUCKET_SIZE: usize = 20;
const NUM_BUCKETS: usize = 256;

#[derive(Debug, Clone)]
pub struct PeerInfo {
    pub id: PeerId,
    pub addr: String,
    pub last_seen: Instant,
}

pub struct KademliaTable {
    local_id: PeerId,
    buckets: Vec<VecDeque<PeerInfo>>,
}

impl KademliaTable {
    pub fn new(local_id: PeerId) -> Self {
        Self {
            local_id,
            buckets: (0..NUM_BUCKETS).map(|_| VecDeque::new()).collect(),
        }
    }

    fn bucket_index(a: &PeerId, b: &PeerId) -> usize {
        let mut xor = [0u8; 32];
        for i in 0..32 {
            xor[i] = a[i] ^ b[i];
        }
        // Find highest bit
        for i in 0..256 {
            let byte_idx = i / 8;
            let bit_idx = 7 - (i % 8);
            if (xor[byte_idx] >> bit_idx) & 1 == 1 {
                return 255 - i;
            }
        }
        0
    }

    pub fn add_peer(&mut self, peer: PeerInfo) {
        if peer.id == self.local_id {
            return;
        }
        let idx = Self::bucket_index(&self.local_id, &peer.id);
        let bucket = &mut self.buckets[idx];

        // Update existing
        if let Some(pos) = bucket.iter().position(|p| p.id == peer.id) {
            bucket.remove(pos);
            bucket.push_back(peer);
            return;
        }

        if bucket.len() < K_BUCKET_SIZE {
            bucket.push_back(peer);
        }
        // If full, drop (simplified — real impl would ping head)
    }

    pub fn closest_peers(&self, target: &PeerId, count: usize) -> Vec<&PeerInfo> {
        let mut all_peers: Vec<&PeerInfo> = self.buckets.iter().flat_map(|b| b.iter()).collect();
        all_peers.sort_by_key(|p| {
            let mut dist = [0u8; 32];
            for i in 0..32 {
                dist[i] = p.id[i] ^ target[i];
            }
            dist
        });
        all_peers.truncate(count);
        all_peers
    }

    pub fn peer_count(&self) -> usize {
        self.buckets.iter().map(|b| b.len()).sum()
    }

    pub fn remove_peer(&mut self, peer_id: &PeerId) -> bool {
        for bucket in &mut self.buckets {
            if let Some(pos) = bucket.iter().position(|p| &p.id == peer_id) {
                bucket.remove(pos);
                return true;
            }
        }
        false
    }
}

// ---------------------------------------------------------------------------
// Gossipsub Router
// ---------------------------------------------------------------------------

pub struct GossipRouter {
    pub local_id: PeerId,
    pub role: NodeRole,
    subscriptions: HashSet<Topic>,
    seen_messages: HashSet<Hash>,
    outbox: VecDeque<(PeerId, GossipMessage)>,
    peers: HashMap<PeerId, HashSet<Topic>>,
    message_log: Vec<GossipMessage>,
}

impl GossipRouter {
    pub fn new(local_id: PeerId, role: NodeRole) -> Self {
        let subscriptions: HashSet<Topic> = role.subscriptions().into_iter().collect();
        Self {
            local_id,
            role,
            subscriptions,
            seen_messages: HashSet::new(),
            outbox: VecDeque::new(),
            peers: HashMap::new(),
            message_log: Vec::new(),
        }
    }

    pub fn is_subscribed(&self, topic: &Topic) -> bool {
        self.subscriptions.contains(topic)
    }

    pub fn add_peer(&mut self, peer_id: PeerId, topics: Vec<Topic>) {
        self.peers.insert(peer_id, topics.into_iter().collect());
    }

    pub fn remove_peer(&mut self, peer_id: &PeerId) {
        self.peers.remove(peer_id);
    }

    pub fn publish(&mut self, msg: GossipMessage) -> Result<usize, &'static str> {
        if self.seen_messages.contains(&msg.id) {
            return Err("duplicate message");
        }
        self.seen_messages.insert(msg.id);
        self.message_log.push(msg.clone());

        let mut forwarded = 0;
        let topic = msg.topic.clone();
        for (peer_id, peer_topics) in &self.peers {
            if peer_topics.contains(&topic) && *peer_id != msg.origin {
                self.outbox.push_back((*peer_id, msg.clone()));
                forwarded += 1;
            }
        }
        Ok(forwarded)
    }

    pub fn receive(&mut self, msg: GossipMessage) -> bool {
        if msg.is_expired() {
            return false;
        }
        if self.seen_messages.contains(&msg.id) {
            return false;
        }
        if !self.is_subscribed(&msg.topic) {
            return false;
        }
        // Accept and re-publish
        let _ = self.publish(msg);
        true
    }

    pub fn drain_outbox(&mut self) -> Vec<(PeerId, GossipMessage)> {
        self.outbox.drain(..).collect()
    }

    pub fn connected_peers(&self) -> usize {
        self.peers.len()
    }

    pub fn received_messages(&self) -> &[GossipMessage] {
        &self.message_log
    }
}

// ---------------------------------------------------------------------------
// Network Node (ties DHT + Router together)
// ---------------------------------------------------------------------------

pub struct NetworkNode {
    pub dht: KademliaTable,
    pub router: GossipRouter,
    pub bootstrap_addrs: Vec<String>,
}

impl NetworkNode {
    pub fn new(id: PeerId, role: NodeRole) -> Self {
        Self {
            dht: KademliaTable::new(id),
            router: GossipRouter::new(id, role),
            bootstrap_addrs: vec![
                "/dns4/boot1.prova.network/tcp/30333".into(),
                "/dns4/boot2.prova.network/tcp/30333".into(),
            ],
        }
    }

    pub fn connect_peer(&mut self, info: PeerInfo, topics: Vec<Topic>) {
        let id = info.id;
        self.dht.add_peer(info);
        self.router.add_peer(id, topics);
    }

    pub fn disconnect_peer(&mut self, peer_id: &PeerId) {
        self.dht.remove_peer(peer_id);
        self.router.remove_peer(peer_id);
    }

    pub fn broadcast(&mut self, topic: Topic, payload: Vec<u8>) -> Result<usize, &'static str> {
        let msg = GossipMessage::new(topic, payload, self.router.local_id);
        self.router.publish(msg)
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn make_peer_id(seed: u8) -> PeerId {
        let mut id = [0u8; 32];
        id[0] = seed;
        id[31] = seed.wrapping_mul(7);
        id
    }

    fn make_peer_info(seed: u8) -> PeerInfo {
        PeerInfo {
            id: make_peer_id(seed),
            addr: format!("/ip4/10.0.0.{}/tcp/30333", seed),
            last_seen: Instant::now(),
        }
    }

    #[test]
    fn test_topic_paths() {
        assert_eq!(Topic::Commits.path(), "/prova/1/commits");
        assert_eq!(Topic::Blocks.path(), "/prova/1/blocks");
        assert_eq!(Topic::all().len(), 6);
    }

    #[test]
    fn test_role_subscriptions() {
        assert_eq!(NodeRole::Provider.subscriptions().len(), 6);
        assert_eq!(NodeRole::Challenger.subscriptions().len(), 4);
        assert_eq!(NodeRole::LightClient.subscriptions().len(), 1);
        assert!(NodeRole::LightClient.subscriptions().contains(&Topic::Blocks));
        assert!(!NodeRole::LightClient.subscriptions().contains(&Topic::Commits));
    }

    #[test]
    fn test_message_priority() {
        let origin = make_peer_id(1);
        let block_msg = GossipMessage::new(Topic::Blocks, vec![1], origin);
        let challenge_msg = GossipMessage::new(Topic::Challenges, vec![2], origin);
        let commit_msg = GossipMessage::new(Topic::Commits, vec![3], origin);

        assert_eq!(block_msg.priority, Priority::Critical);
        assert_eq!(challenge_msg.priority, Priority::High);
        assert_eq!(commit_msg.priority, Priority::Normal);
        assert_eq!(block_msg.ttl, Duration::from_secs(300));
        assert_eq!(challenge_msg.ttl, Duration::from_secs(30));
    }

    #[test]
    fn test_kademlia_add_and_count() {
        let local = make_peer_id(0);
        let mut dht = KademliaTable::new(local);
        assert_eq!(dht.peer_count(), 0);

        dht.add_peer(make_peer_info(1));
        dht.add_peer(make_peer_info(2));
        dht.add_peer(make_peer_info(3));
        assert_eq!(dht.peer_count(), 3);

        // Adding self is a no-op
        dht.add_peer(PeerInfo {
            id: local,
            addr: "self".into(),
            last_seen: Instant::now(),
        });
        assert_eq!(dht.peer_count(), 3);
    }

    #[test]
    fn test_kademlia_remove() {
        let local = make_peer_id(0);
        let mut dht = KademliaTable::new(local);
        let peer_id = make_peer_id(5);
        dht.add_peer(make_peer_info(5));
        assert_eq!(dht.peer_count(), 1);
        assert!(dht.remove_peer(&peer_id));
        assert_eq!(dht.peer_count(), 0);
        assert!(!dht.remove_peer(&peer_id));
    }

    #[test]
    fn test_kademlia_closest_peers() {
        let local = make_peer_id(0);
        let mut dht = KademliaTable::new(local);
        for i in 1..=10 {
            dht.add_peer(make_peer_info(i));
        }
        let target = make_peer_id(3);
        let closest = dht.closest_peers(&target, 3);
        assert_eq!(closest.len(), 3);
        // First result should be the exact match
        assert_eq!(closest[0].id, make_peer_id(3));
    }

    #[test]
    fn test_kademlia_bucket_overflow() {
        let local = make_peer_id(0);
        let mut dht = KademliaTable::new(local);
        // Force many peers into same bucket by using adjacent IDs
        for i in 1..=30 {
            let mut id = [0u8; 32];
            id[0] = 128; // same high bit → same bucket
            id[1] = i;
            dht.add_peer(PeerInfo {
                id,
                addr: format!("addr-{}", i),
                last_seen: Instant::now(),
            });
        }
        // Bucket size capped at K_BUCKET_SIZE=20
        assert!(dht.peer_count() <= K_BUCKET_SIZE);
    }

    #[test]
    fn test_router_publish_and_forward() {
        let id_a = make_peer_id(1);
        let id_b = make_peer_id(2);
        let id_c = make_peer_id(3);

        let mut router = GossipRouter::new(id_a, NodeRole::Provider);
        router.add_peer(id_b, vec![Topic::Commits, Topic::Blocks]);
        router.add_peer(id_c, vec![Topic::Blocks]);

        let msg = GossipMessage::new(Topic::Commits, vec![42], id_a);
        let forwarded = router.publish(msg).unwrap();
        // Only id_b subscribes to Commits
        assert_eq!(forwarded, 1);

        let outbox = router.drain_outbox();
        assert_eq!(outbox.len(), 1);
        assert_eq!(outbox[0].0, id_b);
    }

    #[test]
    fn test_router_dedup() {
        let id_a = make_peer_id(1);
        let mut router = GossipRouter::new(id_a, NodeRole::Provider);

        let msg = GossipMessage::new(Topic::Commits, vec![1], id_a);
        assert!(router.publish(msg.clone()).is_ok());
        assert_eq!(router.publish(msg), Err("duplicate message"));
    }

    #[test]
    fn test_router_receive_filters_unsubscribed() {
        let id_a = make_peer_id(1);
        let id_b = make_peer_id(2);

        // Light client only subscribes to Blocks
        let mut router = GossipRouter::new(id_a, NodeRole::LightClient);
        router.add_peer(id_b, vec![Topic::Blocks]);

        let commit_msg = GossipMessage::new(Topic::Commits, vec![1], id_b);
        assert!(!router.receive(commit_msg)); // rejected — not subscribed

        let block_msg = GossipMessage::new(Topic::Blocks, vec![2], id_b);
        assert!(router.receive(block_msg)); // accepted
    }

    #[test]
    fn test_network_node_connect_disconnect() {
        let id = make_peer_id(0);
        let mut node = NetworkNode::new(id, NodeRole::Provider);
        assert_eq!(node.dht.peer_count(), 0);
        assert_eq!(node.router.connected_peers(), 0);

        let peer = make_peer_info(1);
        node.connect_peer(peer, vec![Topic::Commits]);
        assert_eq!(node.dht.peer_count(), 1);
        assert_eq!(node.router.connected_peers(), 1);

        node.disconnect_peer(&make_peer_id(1));
        assert_eq!(node.dht.peer_count(), 0);
        assert_eq!(node.router.connected_peers(), 0);
    }

    #[test]
    fn test_network_node_broadcast() {
        let id = make_peer_id(0);
        let mut node = NetworkNode::new(id, NodeRole::Provider);

        node.connect_peer(make_peer_info(1), vec![Topic::Blocks]);
        node.connect_peer(make_peer_info(2), vec![Topic::Blocks]);
        node.connect_peer(make_peer_info(3), vec![Topic::Commits]);

        let forwarded = node.broadcast(Topic::Blocks, vec![99]).unwrap();
        assert_eq!(forwarded, 2);
    }

    #[test]
    fn test_bootstrap_addrs() {
        let node = NetworkNode::new(make_peer_id(0), NodeRole::Provider);
        assert_eq!(node.bootstrap_addrs.len(), 2);
        assert!(node.bootstrap_addrs[0].contains("boot1.prova.network"));
    }
}
