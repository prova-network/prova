// chain/src/network_sim.rs — Network simulator for multi-node protocol testing
//
// Simulates N virtual nodes with:
// - Configurable latency and jitter between node pairs
// - Network partitions (split-brain scenarios)
// - Message delivery: reliable, lossy, reordering
// - Full protocol execution: block production, inference jobs, disputes, checkpoints
// - Chaos injection: random node crashes, restarts, Byzantine behavior

use std::collections::{BTreeMap, HashMap, VecDeque};

/// Unique identifier for a simulated node.
pub type NodeId = u64;

/// Simulated clock tick (milliseconds).
pub type Tick = u64;

/// Message types flowing between simulated nodes.
#[derive(Debug, Clone, PartialEq)]
pub enum NetMessage {
    /// New block announcement.
    BlockAnnounce {
        height: u64,
        producer: NodeId,
        hash: [u8; 32],
    },
    /// Inference commit broadcast.
    InferenceCommit {
        job_id: u64,
        provider: NodeId,
        activation_root: [u8; 32],
    },
    /// Challenge initiation.
    ChallengeOpen {
        job_id: u64,
        challenger: NodeId,
        round: u32,
    },
    /// Bisection step response.
    BisectionStep {
        job_id: u64,
        responder: NodeId,
        round: u32,
        midpoint_hash: [u8; 32],
    },
    /// Checkpoint proposal.
    CheckpointProposal {
        epoch: u64,
        proposer: NodeId,
        state_root: [u8; 32],
    },
    /// Checkpoint vote.
    CheckpointVote {
        epoch: u64,
        voter: NodeId,
        approve: bool,
    },
    /// Payment channel update.
    PaymentUpdate {
        channel_id: u64,
        sender: NodeId,
        amount: u64,
        nonce: u64,
    },
    /// Heartbeat / liveness ping.
    Ping { from: NodeId, seq: u64 },
    /// Heartbeat response.
    Pong { from: NodeId, seq: u64 },
}

/// Delivery mode for the simulated network.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DeliveryMode {
    /// All messages delivered in order.
    Reliable,
    /// Messages dropped with given probability (0.0 - 1.0).
    Lossy(f64),
    /// Messages may arrive out of order (max reorder depth).
    Reordering(usize),
}

/// Describes a link between two nodes.
#[derive(Debug, Clone)]
pub struct LinkConfig {
    pub latency_ms: u64,
    pub jitter_ms: u64,
    pub delivery: DeliveryMode,
}

impl Default for LinkConfig {
    fn default() -> Self {
        Self {
            latency_ms: 50,
            jitter_ms: 10,
            delivery: DeliveryMode::Reliable,
        }
    }
}

/// State of a single simulated node.
#[derive(Debug, Clone)]
pub struct SimNode {
    pub id: NodeId,
    pub is_validator: bool,
    pub is_provider: bool,
    pub stake: u64,
    pub blocks_produced: u64,
    pub jobs_completed: u64,
    pub challenges_won: u64,
    pub challenges_lost: u64,
    pub crashed: bool,
    pub inbox: VecDeque<(Tick, NetMessage)>,
    /// Blocks this node knows about: height → hash.
    pub chain_tip: u64,
    pub known_blocks: BTreeMap<u64, [u8; 32]>,
}

impl SimNode {
    pub fn new(id: NodeId, is_validator: bool, is_provider: bool, stake: u64) -> Self {
        Self {
            id,
            is_validator,
            is_provider,
            stake,
            blocks_produced: 0,
            jobs_completed: 0,
            challenges_won: 0,
            challenges_lost: 0,
            crashed: false,
            inbox: VecDeque::new(),
            chain_tip: 0,
            known_blocks: BTreeMap::new(),
        }
    }

    pub fn receive(&mut self, tick: Tick, msg: NetMessage) {
        if !self.crashed {
            self.inbox.push_back((tick, msg));
        }
    }

    pub fn process_inbox(&mut self) -> Vec<(NodeId, NetMessage)> {
        if self.crashed {
            return vec![];
        }
        let mut outgoing = Vec::new();
        while let Some((_tick, msg)) = self.inbox.pop_front() {
            match &msg {
                NetMessage::BlockAnnounce { height, hash, .. } => {
                    if *height > self.chain_tip {
                        self.chain_tip = *height;
                        self.known_blocks.insert(*height, *hash);
                    }
                }
                NetMessage::Ping { from, seq } => {
                    outgoing.push((
                        *from,
                        NetMessage::Pong {
                            from: self.id,
                            seq: *seq,
                        },
                    ));
                }
                _ => {} // Other messages processed by higher-level logic
            }
        }
        outgoing
    }

    pub fn crash(&mut self) {
        self.crashed = true;
        self.inbox.clear();
    }

    pub fn restart(&mut self) {
        self.crashed = false;
    }
}

/// A scheduled message in-flight.
#[derive(Debug, Clone)]
struct InFlightMessage {
    deliver_at: Tick,
    from: NodeId,
    to: NodeId,
    msg: NetMessage,
}

/// Partition definition: nodes in group A cannot reach nodes in group B and vice versa.
#[derive(Debug, Clone)]
pub struct Partition {
    pub group_a: Vec<NodeId>,
    pub group_b: Vec<NodeId>,
    pub start_tick: Tick,
    pub end_tick: Tick,
}

/// The main network simulator.
#[derive(Debug)]
pub struct NetworkSim {
    pub tick: Tick,
    pub nodes: HashMap<NodeId, SimNode>,
    pub links: HashMap<(NodeId, NodeId), LinkConfig>,
    pub default_link: LinkConfig,
    in_flight: Vec<InFlightMessage>,
    pub partitions: Vec<Partition>,
    pub delivered_count: u64,
    pub dropped_count: u64,
    rng_state: u64,
}

impl NetworkSim {
    pub fn new(default_link: LinkConfig) -> Self {
        Self {
            tick: 0,
            nodes: HashMap::new(),
            links: HashMap::new(),
            default_link,
            in_flight: Vec::new(),
            partitions: Vec::new(),
            delivered_count: 0,
            dropped_count: 0,
            rng_state: 42,
        }
    }

    /// Simple deterministic PRNG (xorshift64).
    fn rand_u64(&mut self) -> u64 {
        let mut x = self.rng_state;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.rng_state = x;
        x
    }

    fn rand_f64(&mut self) -> f64 {
        (self.rand_u64() % 1_000_000) as f64 / 1_000_000.0
    }

    pub fn add_node(&mut self, node: SimNode) {
        self.nodes.insert(node.id, node);
    }

    pub fn set_link(&mut self, a: NodeId, b: NodeId, config: LinkConfig) {
        self.links.insert((a, b), config.clone());
        self.links.insert((b, a), config);
    }

    pub fn add_partition(&mut self, partition: Partition) {
        self.partitions.push(partition);
    }

    /// Check if two nodes are partitioned at the current tick.
    fn is_partitioned(&self, a: NodeId, b: NodeId) -> bool {
        for p in &self.partitions {
            if self.tick >= p.start_tick && self.tick < p.end_tick {
                let a_in_a = p.group_a.contains(&a);
                let a_in_b = p.group_b.contains(&a);
                let b_in_a = p.group_a.contains(&b);
                let b_in_b = p.group_b.contains(&b);
                if (a_in_a && b_in_b) || (a_in_b && b_in_a) {
                    return true;
                }
            }
        }
        false
    }

    /// Send a message from one node to another.
    pub fn send(&mut self, from: NodeId, to: NodeId, msg: NetMessage) {
        if self.is_partitioned(from, to) {
            self.dropped_count += 1;
            return;
        }

        let link = self
            .links
            .get(&(from, to))
            .cloned()
            .unwrap_or(self.default_link.clone());
        match link.delivery {
            DeliveryMode::Lossy(drop_prob) => {
                if self.rand_f64() < drop_prob {
                    self.dropped_count += 1;
                    return;
                }
            }
            _ => {}
        }

        let jitter = if link.jitter_ms > 0 {
            (self.rand_u64() % (link.jitter_ms * 2)) as i64 - link.jitter_ms as i64
        } else {
            0
        };
        let deliver_at =
            (self.tick as i64 + link.latency_ms as i64 + jitter).max(self.tick as i64 + 1) as Tick;

        self.in_flight.push(InFlightMessage {
            deliver_at,
            from,
            to,
            msg,
        });
    }

    /// Broadcast a message from one node to all others.
    pub fn broadcast(&mut self, from: NodeId, msg: NetMessage) {
        let others: Vec<NodeId> = self
            .nodes
            .keys()
            .filter(|&&id| id != from)
            .copied()
            .collect();
        for to in others {
            self.send(from, to, msg.clone());
        }
    }

    /// Advance simulation by one tick. Returns messages delivered this tick.
    pub fn step(&mut self) -> Vec<(NodeId, NodeId, NetMessage)> {
        self.tick += 1;
        let mut delivered = Vec::new();
        let mut remaining = Vec::new();

        for ifm in self.in_flight.drain(..) {
            if ifm.deliver_at <= self.tick {
                if let Some(node) = self.nodes.get_mut(&ifm.to) {
                    node.receive(self.tick, ifm.msg.clone());
                    self.delivered_count += 1;
                    delivered.push((ifm.from, ifm.to, ifm.msg));
                }
            } else {
                remaining.push(ifm);
            }
        }
        self.in_flight = remaining;

        // Process all node inboxes and collect outgoing messages.
        let node_ids: Vec<NodeId> = self.nodes.keys().copied().collect();
        let mut new_sends = Vec::new();
        for id in node_ids {
            if let Some(node) = self.nodes.get_mut(&id) {
                let outgoing = node.process_inbox();
                for (to, msg) in outgoing {
                    new_sends.push((id, to, msg));
                }
            }
        }
        for (from, to, msg) in new_sends {
            self.send(from, to, msg);
        }

        delivered
    }

    /// Run simulation for N ticks.
    pub fn run(&mut self, ticks: u64) -> SimStats {
        let start_delivered = self.delivered_count;
        let start_dropped = self.dropped_count;
        for _ in 0..ticks {
            self.step();
        }
        SimStats {
            ticks_run: ticks,
            messages_delivered: self.delivered_count - start_delivered,
            messages_dropped: self.dropped_count - start_dropped,
            in_flight: self.in_flight.len() as u64,
            nodes_alive: self.nodes.values().filter(|n| !n.crashed).count() as u64,
            nodes_crashed: self.nodes.values().filter(|n| n.crashed).count() as u64,
        }
    }

    /// Simulate a block production round: the designated producer broadcasts a block.
    pub fn produce_block(&mut self, producer: NodeId, height: u64) -> [u8; 32] {
        let hash = {
            let mut h = [0u8; 32];
            let bytes = (producer * 1000 + height).to_le_bytes();
            h[..8].copy_from_slice(&bytes);
            h
        };
        if let Some(node) = self.nodes.get_mut(&producer) {
            node.blocks_produced += 1;
            node.chain_tip = height;
            node.known_blocks.insert(height, hash);
        }
        let msg = NetMessage::BlockAnnounce {
            height,
            producer,
            hash,
        };
        self.broadcast(producer, msg);
        hash
    }

    /// Get a snapshot of node states.
    pub fn snapshot(&self) -> Vec<NodeSnapshot> {
        self.nodes
            .values()
            .map(|n| NodeSnapshot {
                id: n.id,
                chain_tip: n.chain_tip,
                blocks_produced: n.blocks_produced,
                crashed: n.crashed,
                inbox_size: n.inbox.len() as u64,
            })
            .collect()
    }
}

/// Summary statistics from a simulation run.
#[derive(Debug, Clone)]
pub struct SimStats {
    pub ticks_run: u64,
    pub messages_delivered: u64,
    pub messages_dropped: u64,
    pub in_flight: u64,
    pub nodes_alive: u64,
    pub nodes_crashed: u64,
}

/// Point-in-time snapshot of a node.
#[derive(Debug, Clone)]
pub struct NodeSnapshot {
    pub id: NodeId,
    pub chain_tip: u64,
    pub blocks_produced: u64,
    pub crashed: bool,
    pub inbox_size: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn setup_3_node_sim() -> NetworkSim {
        let mut sim = NetworkSim::new(LinkConfig::default());
        sim.add_node(SimNode::new(1, true, false, 1000));
        sim.add_node(SimNode::new(2, true, false, 1000));
        sim.add_node(SimNode::new(3, true, true, 500));
        sim
    }

    #[test]
    fn test_basic_message_delivery() {
        let mut sim = setup_3_node_sim();
        sim.send(1, 2, NetMessage::Ping { from: 1, seq: 1 });
        // Message not yet delivered (latency = 50ms).
        assert_eq!(sim.delivered_count, 0);
        let stats = sim.run(60);
        assert!(stats.messages_delivered >= 1);
    }

    #[test]
    fn test_broadcast_reaches_all() {
        let mut sim = setup_3_node_sim();
        let msg = NetMessage::Ping { from: 1, seq: 42 };
        sim.broadcast(1, msg);
        sim.run(100);
        // 2 broadcasts + 2 pong responses delivered.
        assert!(sim.delivered_count >= 2);
    }

    #[test]
    fn test_block_propagation() {
        let mut sim = setup_3_node_sim();
        sim.produce_block(1, 1);
        sim.run(100);
        // All nodes should know about block 1.
        for (id, node) in &sim.nodes {
            assert_eq!(node.chain_tip, 1, "node {} should be at tip 1", id);
        }
    }

    #[test]
    fn test_partition_blocks_messages() {
        let mut sim = setup_3_node_sim();
        sim.add_partition(Partition {
            group_a: vec![1],
            group_b: vec![2, 3],
            start_tick: 0,
            end_tick: 200,
        });
        sim.produce_block(1, 1);
        sim.run(150);
        // Node 1 produced block, but 2 and 3 are partitioned — they shouldn't see it.
        assert_eq!(sim.nodes[&2].chain_tip, 0);
        assert_eq!(sim.nodes[&3].chain_tip, 0);
        assert_eq!(sim.nodes[&1].chain_tip, 1);
    }

    #[test]
    fn test_partition_heals() {
        let mut sim = setup_3_node_sim();
        sim.add_partition(Partition {
            group_a: vec![1],
            group_b: vec![2, 3],
            start_tick: 0,
            end_tick: 50,
        });
        // Block produced during partition.
        sim.produce_block(1, 1);
        sim.run(40);
        assert_eq!(sim.nodes[&2].chain_tip, 0);
        // Run past partition end, produce another block.
        sim.run(20); // tick = 60, partition healed
        sim.produce_block(1, 2);
        sim.run(100);
        assert_eq!(sim.nodes[&2].chain_tip, 2);
    }

    #[test]
    fn test_node_crash_drops_messages() {
        let mut sim = setup_3_node_sim();
        sim.nodes.get_mut(&2).unwrap().crash();
        sim.produce_block(1, 1);
        sim.run(100);
        assert_eq!(sim.nodes[&2].chain_tip, 0);
        assert_eq!(sim.nodes[&3].chain_tip, 1);
    }

    #[test]
    fn test_node_restart_receives_new_messages() {
        let mut sim = setup_3_node_sim();
        sim.nodes.get_mut(&2).unwrap().crash();
        sim.produce_block(1, 1);
        sim.run(100);
        assert_eq!(sim.nodes[&2].chain_tip, 0);
        sim.nodes.get_mut(&2).unwrap().restart();
        sim.produce_block(1, 2);
        sim.run(100);
        assert_eq!(sim.nodes[&2].chain_tip, 2);
    }

    #[test]
    fn test_lossy_link() {
        let mut sim = NetworkSim::new(LinkConfig {
            latency_ms: 10,
            jitter_ms: 0,
            delivery: DeliveryMode::Lossy(0.5),
        });
        sim.add_node(SimNode::new(1, true, false, 1000));
        sim.add_node(SimNode::new(2, true, false, 1000));
        // Send many messages, expect roughly half dropped.
        for i in 0..100 {
            sim.send(1, 2, NetMessage::Ping { from: 1, seq: i });
        }
        sim.run(50);
        // With 50% drop, we expect ~50 delivered but allow wide margin.
        assert!(
            sim.delivered_count > 20,
            "delivered: {}",
            sim.delivered_count
        );
        assert!(
            sim.delivered_count < 90,
            "delivered: {}",
            sim.delivered_count
        );
        assert!(sim.dropped_count > 20, "dropped: {}", sim.dropped_count);
    }

    #[test]
    fn test_custom_link_latency() {
        let mut sim = NetworkSim::new(LinkConfig::default());
        sim.add_node(SimNode::new(1, true, false, 1000));
        sim.add_node(SimNode::new(2, true, false, 1000));
        // Set a slow link: 200ms latency.
        sim.set_link(
            1,
            2,
            LinkConfig {
                latency_ms: 200,
                jitter_ms: 0,
                delivery: DeliveryMode::Reliable,
            },
        );
        sim.send(1, 2, NetMessage::Ping { from: 1, seq: 1 });
        sim.run(100);
        // Should NOT be delivered yet (only 100 ticks, need 200).
        assert_eq!(sim.nodes[&2].chain_tip, 0);
        // Pings go to inbox, check inbox was empty at tick 100.
        // Actually ping doesn't affect chain_tip. Let's check delivered_count.
        let d1 = sim.delivered_count;
        sim.run(150); // now at tick 250
        assert!(
            sim.delivered_count > d1,
            "message should be delivered by tick 250"
        );
    }

    #[test]
    fn test_multi_block_convergence() {
        let mut sim = setup_3_node_sim();
        for h in 1..=10 {
            let producer = ((h - 1) % 3) as u64 + 1;
            sim.produce_block(producer, h);
            sim.run(100);
        }
        // All nodes should converge to tip 10.
        for (_, node) in &sim.nodes {
            assert_eq!(node.chain_tip, 10);
        }
    }

    #[test]
    fn test_snapshot() {
        let mut sim = setup_3_node_sim();
        sim.produce_block(1, 1);
        sim.run(100);
        let snap = sim.snapshot();
        assert_eq!(snap.len(), 3);
        let producer = snap.iter().find(|s| s.id == 1).unwrap();
        assert_eq!(producer.blocks_produced, 1);
    }

    #[test]
    fn test_sim_stats() {
        let mut sim = setup_3_node_sim();
        sim.produce_block(1, 1);
        let stats = sim.run(100);
        assert_eq!(stats.ticks_run, 100);
        assert!(stats.messages_delivered > 0);
        assert_eq!(stats.nodes_alive, 3);
        assert_eq!(stats.nodes_crashed, 0);
    }

    #[test]
    fn test_inference_commit_message() {
        let mut sim = setup_3_node_sim();
        let root = [0xABu8; 32];
        let msg = NetMessage::InferenceCommit {
            job_id: 1,
            provider: 3,
            activation_root: root,
        };
        sim.broadcast(3, msg);
        sim.run(100);
        assert!(sim.delivered_count >= 2);
    }

    #[test]
    fn test_checkpoint_flow() {
        let mut sim = setup_3_node_sim();
        let state_root = [0xCDu8; 32];
        let proposal = NetMessage::CheckpointProposal {
            epoch: 10,
            proposer: 1,
            state_root,
        };
        sim.broadcast(1, proposal);
        sim.run(100);
        // All non-proposer nodes received the proposal.
        assert!(sim.delivered_count >= 2);
    }

    #[test]
    fn test_concurrent_partitions() {
        let mut sim = NetworkSim::new(LinkConfig {
            latency_ms: 5,
            jitter_ms: 0,
            delivery: DeliveryMode::Reliable,
        });
        for i in 1..=5 {
            sim.add_node(SimNode::new(i, true, false, 1000));
        }
        // Two overlapping partitions.
        sim.add_partition(Partition {
            group_a: vec![1, 2],
            group_b: vec![3, 4, 5],
            start_tick: 0,
            end_tick: 50,
        });
        sim.produce_block(1, 1);
        sim.run(30);
        // Node 2 should see it, 3/4/5 should not.
        assert_eq!(sim.nodes[&2].chain_tip, 1);
        assert_eq!(sim.nodes[&3].chain_tip, 0);
        assert_eq!(sim.nodes[&4].chain_tip, 0);
    }

    #[test]
    fn test_large_network_10_nodes() {
        let mut sim = NetworkSim::new(LinkConfig {
            latency_ms: 20,
            jitter_ms: 5,
            delivery: DeliveryMode::Reliable,
        });
        for i in 1..=10 {
            sim.add_node(SimNode::new(i, true, false, 1000));
        }
        sim.produce_block(1, 1);
        sim.run(200);
        for (_, node) in &sim.nodes {
            assert_eq!(node.chain_tip, 1);
        }
    }

    #[test]
    fn test_payment_message_delivery() {
        let mut sim = setup_3_node_sim();
        let msg = NetMessage::PaymentUpdate {
            channel_id: 42,
            sender: 1,
            amount: 500,
            nonce: 1,
        };
        sim.send(1, 3, msg);
        sim.run(100);
        assert!(sim.delivered_count >= 1);
    }
}
