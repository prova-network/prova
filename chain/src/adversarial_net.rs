// chain/src/adversarial_net.rs — Adversarial network scenarios
//
// Implements advanced network-layer attacks on top of NetworkSim + Chaos:
// - Eclipse attacks: isolate a victim node by controlling all its peers
// - Message flooding: overwhelm nodes with junk messages
// - Sybil peer injection: attacker controls N fake nodes
// - Selective message suppression: drop only specific message types
// - Timing attacks: delay blocks to specific validators
// - Man-in-the-middle: intercept and modify messages between peers

use crate::chaos::*;
use crate::network_sim::*;
use std::collections::{HashMap, HashSet, VecDeque};

/// Types of adversarial attacks.
#[derive(Debug, Clone, PartialEq)]
pub enum AttackType {
    /// Eclipse: isolate victim from honest peers, only attacker peers visible.
    Eclipse {
        victim: NodeId,
        attacker_nodes: Vec<NodeId>,
        /// Duration in ticks before honest peers can reconnect.
        isolation_ticks: u64,
    },
    /// Flood: send N junk messages per tick from attacker nodes.
    Flood {
        attackers: Vec<NodeId>,
        /// Messages per tick per attacker.
        rate: u64,
        /// Total ticks of flooding.
        duration_ticks: u64,
        /// Target nodes (empty = broadcast to all).
        targets: Vec<NodeId>,
    },
    /// Sybil: spin up N fake nodes that collude.
    Sybil {
        /// Number of sybil nodes to inject.
        count: u64,
        /// Stake per sybil node (if any).
        stake_each: u64,
        /// Sybil behavior.
        behavior: SybilBehavior,
    },
    /// Selective suppression: drop messages matching a filter.
    SelectiveDrop {
        /// Attacker-controlled relay nodes.
        relays: Vec<NodeId>,
        /// Which message types to drop.
        filter: MessageFilter,
        duration_ticks: u64,
    },
    /// Timing manipulation: delay specific messages to specific targets.
    TimingAttack {
        attacker: NodeId,
        target: NodeId,
        /// Additional delay in ms added to block announcements.
        delay_ms: u64,
        duration_ticks: u64,
    },
}

/// Sybil node behavior patterns.
#[derive(Debug, Clone, PartialEq)]
pub enum SybilBehavior {
    /// Passive: listen only, gather information.
    Passive,
    /// Vote manipulation: sybils all vote the same way.
    VoteManipulation { approve: bool },
    /// Equivocation: produce conflicting blocks/votes.
    Equivocate,
    /// Flood: each sybil floods junk.
    FloodJunk { rate: u64 },
}

/// Filter for selective message dropping.
#[derive(Debug, Clone, PartialEq)]
pub enum MessageFilter {
    /// Drop all block announcements.
    BlockAnnouncements,
    /// Drop all checkpoint votes.
    CheckpointVotes,
    /// Drop all challenge messages.
    Challenges,
    /// Drop messages from specific senders.
    FromSenders(Vec<NodeId>),
    /// Drop messages to specific recipients.
    ToRecipients(Vec<NodeId>),
    /// Composite: drop if ANY filter matches.
    Any(Vec<MessageFilter>),
}

impl MessageFilter {
    /// Check if a message matches this filter.
    pub fn matches(&self, msg: &NetMessage, from: NodeId, to: NodeId) -> bool {
        match self {
            MessageFilter::BlockAnnouncements => matches!(msg, NetMessage::BlockAnnounce { .. }),
            MessageFilter::CheckpointVotes => matches!(msg, NetMessage::CheckpointVote { .. }),
            MessageFilter::Challenges => matches!(msg, NetMessage::ChallengeOpen { .. }),
            MessageFilter::FromSenders(senders) => senders.contains(&from),
            MessageFilter::ToRecipients(recipients) => recipients.contains(&to),
            MessageFilter::Any(filters) => filters.iter().any(|f| f.matches(msg, from, to)),
        }
    }
}

/// Result of an adversarial scenario execution.
#[derive(Debug)]
pub struct AdversarialResult {
    pub attack_type: String,
    pub success: bool,
    /// Did the network recover after the attack ended?
    pub recovered: bool,
    /// Ticks until recovery (None if didn't recover).
    pub recovery_ticks: Option<u64>,
    /// Messages dropped by the attack.
    pub messages_dropped: u64,
    /// Messages injected by the attack.
    pub messages_injected: u64,
    /// Nodes that fell out of sync.
    pub desynchronized_nodes: Vec<NodeId>,
    /// Nodes that were permanently damaged (e.g., bad state).
    pub damaged_nodes: Vec<NodeId>,
    /// Security violation detected (e.g., fork, double-spend).
    pub security_violation: Option<String>,
    pub events: Vec<ChaosEvent>,
}

/// Eclipse attack simulator.
pub struct EclipseAttack {
    pub victim: NodeId,
    pub attacker_nodes: Vec<NodeId>,
    pub isolation_ticks: u64,
    /// Peer table: victim's view of peers (replaced by attacker).
    pub original_peers: Vec<NodeId>,
    pub fake_chain_tip: u64,
}

impl EclipseAttack {
    pub fn new(victim: NodeId, attacker_nodes: Vec<NodeId>, isolation_ticks: u64) -> Self {
        Self {
            victim,
            attacker_nodes,
            isolation_ticks,
            original_peers: Vec::new(),
            fake_chain_tip: 0,
        }
    }

    /// Execute eclipse attack on a network sim.
    /// Returns scenario actions that implement the eclipse.
    pub fn to_chaos_actions(&self, honest_nodes: &[NodeId]) -> Vec<ChaosAction> {
        let mut actions = Vec::new();

        // Step 1: Partition victim from all honest nodes.
        let honest: Vec<NodeId> = honest_nodes
            .iter()
            .filter(|n| **n != self.victim && !self.attacker_nodes.contains(n))
            .copied()
            .collect();

        if !honest.is_empty() {
            actions.push(ChaosAction::Partition {
                group_a: vec![self.victim],
                group_b: honest,
                duration_ticks: self.isolation_ticks,
            });
        }

        // Step 2: Attacker feeds victim a fake (shorter) chain.
        for attacker in &self.attacker_nodes {
            actions.push(ChaosAction::BroadcastMessage {
                from: *attacker,
                msg: NetMessage::BlockAnnounce {
                    height: self.fake_chain_tip,
                    producer: *attacker,
                    hash: [0xEE; 32], // fake hash
                },
            });
        }

        // Step 3: Wait for isolation period.
        actions.push(ChaosAction::Wait(self.isolation_ticks));

        // Step 4: Heal and check recovery.
        actions.push(ChaosAction::HealAll);
        actions.push(ChaosAction::Wait(200)); // recovery window

        actions
    }

    /// Verify the victim wasn't permanently eclipsed.
    pub fn verify_recovery(sim: &NetworkSim) -> EclipseVerification {
        let tips: Vec<(NodeId, u64)> = sim
            .nodes
            .iter()
            .filter(|(_, n)| !n.crashed)
            .map(|(id, n)| (*id, n.chain_tip))
            .collect();

        if tips.is_empty() {
            return EclipseVerification {
                recovered: false,
                max_divergence: 0,
                detail: "no alive nodes".into(),
            };
        }

        let max_tip = tips.iter().map(|(_, t)| *t).max().unwrap();
        let min_tip = tips.iter().map(|(_, t)| *t).min().unwrap();
        let divergence = max_tip - min_tip;

        EclipseVerification {
            recovered: divergence <= 1,
            max_divergence: divergence,
            detail: format!(
                "tips range {}..{}, divergence {}",
                min_tip, max_tip, divergence
            ),
        }
    }
}

#[derive(Debug)]
pub struct EclipseVerification {
    pub recovered: bool,
    pub max_divergence: u64,
    pub detail: String,
}

/// Flood attack simulator.
pub struct FloodAttack {
    pub attackers: Vec<NodeId>,
    pub rate: u64,
    pub duration_ticks: u64,
    pub targets: Vec<NodeId>,
}

impl FloodAttack {
    pub fn new(attackers: Vec<NodeId>, rate: u64, duration_ticks: u64) -> Self {
        Self {
            attackers,
            rate,
            duration_ticks,
            targets: Vec::new(),
        }
    }

    pub fn with_targets(mut self, targets: Vec<NodeId>) -> Self {
        self.targets = targets;
        self
    }

    /// Generate junk messages for one tick of flooding.
    pub fn generate_junk(&self, tick: u64) -> Vec<(NodeId, NetMessage)> {
        let mut msgs = Vec::new();
        for attacker in &self.attackers {
            for i in 0..self.rate {
                msgs.push((
                    *attacker,
                    NetMessage::Ping {
                        from: *attacker,
                        seq: tick * 1000 + i,
                    },
                ));
            }
        }
        msgs
    }

    /// Estimate total junk messages over the attack duration.
    pub fn total_junk_messages(&self) -> u64 {
        self.attackers.len() as u64 * self.rate * self.duration_ticks
    }

    /// Convert to chaos actions.
    pub fn to_chaos_actions(&self) -> Vec<ChaosAction> {
        let mut actions = Vec::new();

        for tick in 0..self.duration_ticks {
            for attacker in &self.attackers {
                for i in 0..self.rate {
                    actions.push(ChaosAction::BroadcastMessage {
                        from: *attacker,
                        msg: NetMessage::Ping {
                            from: *attacker,
                            seq: tick * 1000 + i,
                        },
                    });
                }
            }
            // Interleave waits so sim processes messages.
            if tick % 10 == 9 {
                actions.push(ChaosAction::Wait(10));
            }
        }

        actions
    }
}

/// Flood defense: per-node rate limiter for incoming messages.
#[derive(Debug, Clone)]
pub struct PeerRateLimiter {
    /// Max messages per tick per peer.
    pub max_per_tick: u64,
    /// Current tick counters: peer → count.
    counters: HashMap<NodeId, u64>,
    /// Total messages dropped.
    pub dropped: u64,
    /// Current tick.
    current_tick: u64,
}

impl PeerRateLimiter {
    pub fn new(max_per_tick: u64) -> Self {
        Self {
            max_per_tick,
            counters: HashMap::new(),
            dropped: 0,
            current_tick: 0,
        }
    }

    /// Advance to a new tick, resetting counters.
    pub fn advance_tick(&mut self, tick: u64) {
        if tick > self.current_tick {
            self.counters.clear();
            self.current_tick = tick;
        }
    }

    /// Check if a message from a peer should be accepted.
    pub fn allow(&mut self, peer: NodeId) -> bool {
        let count = self.counters.entry(peer).or_insert(0);
        if *count >= self.max_per_tick {
            self.dropped += 1;
            false
        } else {
            *count += 1;
            true
        }
    }

    /// Reset all state.
    pub fn reset(&mut self) {
        self.counters.clear();
        self.dropped = 0;
        self.current_tick = 0;
    }
}

/// Selective message suppression attack.
pub struct SuppressionAttack {
    pub relays: Vec<NodeId>,
    pub filter: MessageFilter,
    pub duration_ticks: u64,
    pub messages_suppressed: u64,
}

impl SuppressionAttack {
    pub fn new(relays: Vec<NodeId>, filter: MessageFilter, duration_ticks: u64) -> Self {
        Self {
            relays,
            filter,
            duration_ticks,
            messages_suppressed: 0,
        }
    }

    /// Check if a message should be suppressed.
    pub fn should_suppress(&self, msg: &NetMessage, from: NodeId, to: NodeId) -> bool {
        self.relays.contains(&from) && self.filter.matches(msg, from, to)
    }

    /// Convert to chaos actions using link degradation.
    pub fn to_chaos_actions(&self) -> Vec<ChaosAction> {
        let mut actions = Vec::new();

        // Model suppression as lossy links from relay nodes.
        for relay in &self.relays {
            // Set all links from relay to 90% loss (simulates selective drop).
            for target in 0..20u64 {
                if target != *relay {
                    actions.push(ChaosAction::SetLink {
                        a: *relay,
                        b: target,
                        config: LinkConfig {
                            latency_ms: 50,
                            jitter_ms: 10,
                            delivery: DeliveryMode::Lossy(0.9),
                        },
                    });
                }
            }
        }

        actions.push(ChaosAction::Wait(self.duration_ticks));

        // Restore links.
        for relay in &self.relays {
            for target in 0..20u64 {
                if target != *relay {
                    actions.push(ChaosAction::SetLink {
                        a: *relay,
                        b: target,
                        config: LinkConfig::default(),
                    });
                }
            }
        }

        actions
    }
}

/// Timing attack: selectively delay messages to a target validator.
pub struct TimingAttack {
    pub attacker: NodeId,
    pub target: NodeId,
    pub extra_delay_ms: u64,
    pub duration_ticks: u64,
}

impl TimingAttack {
    pub fn new(attacker: NodeId, target: NodeId, extra_delay_ms: u64, duration_ticks: u64) -> Self {
        Self {
            attacker,
            target,
            extra_delay_ms,
            duration_ticks,
        }
    }

    /// Convert to chaos actions.
    pub fn to_chaos_actions(&self) -> Vec<ChaosAction> {
        vec![
            ChaosAction::SetLink {
                a: self.attacker,
                b: self.target,
                config: LinkConfig {
                    latency_ms: self.extra_delay_ms,
                    jitter_ms: self.extra_delay_ms / 4,
                    delivery: DeliveryMode::Reliable,
                },
            },
            ChaosAction::Wait(self.duration_ticks),
            ChaosAction::SetLink {
                a: self.attacker,
                b: self.target,
                config: LinkConfig::default(),
            },
        ]
    }
}

/// Comprehensive adversarial scenario builder.
pub struct AdversarialScenarioBuilder {
    pub name: String,
    pub node_count: u64,
    pub validators: u64,
    pub attacks: Vec<AttackType>,
    pub recovery_ticks: u64,
}

impl AdversarialScenarioBuilder {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            node_count: 10,
            validators: 4,
            attacks: Vec::new(),
            recovery_ticks: 500,
        }
    }

    pub fn with_nodes(mut self, count: u64, validators: u64) -> Self {
        self.node_count = count;
        self.validators = validators;
        self
    }

    pub fn add_attack(mut self, attack: AttackType) -> Self {
        self.attacks.push(attack);
        self
    }

    pub fn with_recovery(mut self, ticks: u64) -> Self {
        self.recovery_ticks = ticks;
        self
    }

    /// Build a ChaosScenario from the configured attacks.
    pub fn build(&self) -> ChaosScenario {
        let mut nodes = Vec::new();
        for i in 0..self.node_count {
            nodes.push(SimNode {
                id: i,
                is_validator: i < self.validators,
                is_provider: i >= self.validators && i < self.validators + 3,
                stake: if i < self.validators { 1000 } else { 100 },
                blocks_produced: 0,
                jobs_completed: 0,
                challenges_won: 0,
                challenges_lost: 0,
                crashed: false,
                inbox: VecDeque::new(),
                chain_tip: 0,
                known_blocks: std::collections::BTreeMap::new(),
            });
        }

        let mut actions: Vec<ChaosAction> = Vec::new();

        // Initial block production to establish chain.
        for h in 1..=5 {
            let producer = (h - 1) % self.validators;
            actions.push(ChaosAction::ProduceBlock {
                producer,
                height: h,
            });
            actions.push(ChaosAction::Wait(20));
        }
        actions.push(ChaosAction::AssertConverged);

        // Execute each attack.
        for attack in &self.attacks {
            match attack {
                AttackType::Eclipse {
                    victim,
                    attacker_nodes,
                    isolation_ticks,
                } => {
                    let eclipse =
                        EclipseAttack::new(*victim, attacker_nodes.clone(), *isolation_ticks);
                    let honest: Vec<NodeId> = (0..self.node_count).collect();
                    actions.extend(eclipse.to_chaos_actions(&honest));
                }
                AttackType::Flood {
                    attackers,
                    rate,
                    duration_ticks,
                    ..
                } => {
                    let flood = FloodAttack::new(attackers.clone(), *rate, *duration_ticks);
                    actions.extend(flood.to_chaos_actions());
                }
                AttackType::SelectiveDrop {
                    relays,
                    filter,
                    duration_ticks,
                } => {
                    let suppression =
                        SuppressionAttack::new(relays.clone(), filter.clone(), *duration_ticks);
                    actions.extend(suppression.to_chaos_actions());
                }
                AttackType::TimingAttack {
                    attacker,
                    target,
                    delay_ms,
                    duration_ticks,
                } => {
                    let timing = TimingAttack::new(*attacker, *target, *delay_ms, *duration_ticks);
                    actions.extend(timing.to_chaos_actions());
                }
                AttackType::Sybil {
                    count,
                    stake_each,
                    behavior,
                } => {
                    // Sybil nodes are added as crashed-then-restarted nodes at high IDs.
                    let base_id = self.node_count + 100;
                    for i in 0..*count {
                        let sybil_id = base_id + i;
                        match behavior {
                            SybilBehavior::FloodJunk { rate } => {
                                for _ in 0..10 {
                                    for j in 0..*rate {
                                        actions.push(ChaosAction::BroadcastMessage {
                                            from: sybil_id,
                                            msg: NetMessage::Ping {
                                                from: sybil_id,
                                                seq: j,
                                            },
                                        });
                                    }
                                }
                            }
                            SybilBehavior::VoteManipulation { approve } => {
                                actions.push(ChaosAction::BroadcastMessage {
                                    from: sybil_id,
                                    msg: NetMessage::CheckpointVote {
                                        epoch: 1,
                                        voter: sybil_id,
                                        approve: *approve,
                                    },
                                });
                            }
                            SybilBehavior::Equivocate => {
                                // Produce two conflicting blocks at same height.
                                actions.push(ChaosAction::BroadcastMessage {
                                    from: sybil_id,
                                    msg: NetMessage::BlockAnnounce {
                                        height: 10,
                                        producer: sybil_id,
                                        hash: [0xAA; 32],
                                    },
                                });
                                actions.push(ChaosAction::BroadcastMessage {
                                    from: sybil_id,
                                    msg: NetMessage::BlockAnnounce {
                                        height: 10,
                                        producer: sybil_id,
                                        hash: [0xBB; 32],
                                    },
                                });
                            }
                            SybilBehavior::Passive => {
                                // Do nothing — just observe.
                            }
                        }
                    }
                    let _ = stake_each; // stake tracked externally
                }
            }
        }

        // Recovery phase.
        actions.push(ChaosAction::HealAll);
        actions.push(ChaosAction::Wait(self.recovery_ticks));

        // Final block production to test recovery.
        for h in 6..=10 {
            let producer = (h - 1) % self.validators;
            actions.push(ChaosAction::ProduceBlock {
                producer,
                height: h,
            });
            actions.push(ChaosAction::Wait(20));
        }
        actions.push(ChaosAction::AssertConverged);

        ChaosScenario {
            name: self.name.clone(),
            nodes,
            default_link: LinkConfig::default(),
            actions,
        }
    }
}

/// Pre-built adversarial scenarios.
pub fn eclipse_single_validator() -> ChaosScenario {
    AdversarialScenarioBuilder::new("eclipse_single_validator")
        .with_nodes(10, 4)
        .add_attack(AttackType::Eclipse {
            victim: 0,
            attacker_nodes: vec![8, 9],
            isolation_ticks: 300,
        })
        .build()
}

pub fn sustained_flood() -> ChaosScenario {
    AdversarialScenarioBuilder::new("sustained_flood")
        .with_nodes(8, 3)
        .add_attack(AttackType::Flood {
            attackers: vec![6, 7],
            rate: 100,
            duration_ticks: 200,
            targets: vec![],
        })
        .build()
}

pub fn checkpoint_vote_suppression() -> ChaosScenario {
    AdversarialScenarioBuilder::new("checkpoint_vote_suppression")
        .with_nodes(10, 4)
        .add_attack(AttackType::SelectiveDrop {
            relays: vec![5, 6],
            filter: MessageFilter::CheckpointVotes,
            duration_ticks: 400,
        })
        .build()
}

pub fn sybil_vote_manipulation() -> ChaosScenario {
    AdversarialScenarioBuilder::new("sybil_vote_manipulation")
        .with_nodes(10, 4)
        .add_attack(AttackType::Sybil {
            count: 5,
            stake_each: 10,
            behavior: SybilBehavior::VoteManipulation { approve: false },
        })
        .build()
}

pub fn combined_eclipse_and_flood() -> ChaosScenario {
    AdversarialScenarioBuilder::new("combined_eclipse_and_flood")
        .with_nodes(12, 4)
        .add_attack(AttackType::Eclipse {
            victim: 0,
            attacker_nodes: vec![10, 11],
            isolation_ticks: 200,
        })
        .add_attack(AttackType::Flood {
            attackers: vec![10, 11],
            rate: 50,
            duration_ticks: 200,
            targets: vec![1, 2, 3],
        })
        .build()
}

pub fn timing_attack_on_block_producer() -> ChaosScenario {
    AdversarialScenarioBuilder::new("timing_attack_on_block_producer")
        .with_nodes(8, 4)
        .add_attack(AttackType::TimingAttack {
            attacker: 7,
            target: 0,
            delay_ms: 5000,
            duration_ticks: 300,
        })
        .build()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_message_filter_block_announcements() {
        let filter = MessageFilter::BlockAnnouncements;
        let block = NetMessage::BlockAnnounce {
            height: 1,
            producer: 0,
            hash: [0; 32],
        };
        let ping = NetMessage::Ping { from: 0, seq: 1 };
        assert!(filter.matches(&block, 0, 1));
        assert!(!filter.matches(&ping, 0, 1));
    }

    #[test]
    fn test_message_filter_checkpoint_votes() {
        let filter = MessageFilter::CheckpointVotes;
        let vote = NetMessage::CheckpointVote {
            epoch: 1,
            voter: 0,
            approve: true,
        };
        assert!(filter.matches(&vote, 0, 1));
    }

    #[test]
    fn test_message_filter_from_senders() {
        let filter = MessageFilter::FromSenders(vec![3, 5]);
        let msg = NetMessage::Ping { from: 3, seq: 1 };
        assert!(filter.matches(&msg, 3, 0));
        assert!(!filter.matches(&msg, 4, 0));
    }

    #[test]
    fn test_message_filter_composite() {
        let filter = MessageFilter::Any(vec![
            MessageFilter::BlockAnnouncements,
            MessageFilter::CheckpointVotes,
        ]);
        let block = NetMessage::BlockAnnounce {
            height: 1,
            producer: 0,
            hash: [0; 32],
        };
        let vote = NetMessage::CheckpointVote {
            epoch: 1,
            voter: 0,
            approve: true,
        };
        let ping = NetMessage::Ping { from: 0, seq: 1 };
        assert!(filter.matches(&block, 0, 1));
        assert!(filter.matches(&vote, 0, 1));
        assert!(!filter.matches(&ping, 0, 1));
    }

    #[test]
    fn test_peer_rate_limiter_basic() {
        let mut limiter = PeerRateLimiter::new(3);
        assert!(limiter.allow(1));
        assert!(limiter.allow(1));
        assert!(limiter.allow(1));
        assert!(!limiter.allow(1)); // 4th blocked
        assert_eq!(limiter.dropped, 1);
    }

    #[test]
    fn test_peer_rate_limiter_advance_tick() {
        let mut limiter = PeerRateLimiter::new(2);
        assert!(limiter.allow(1));
        assert!(limiter.allow(1));
        assert!(!limiter.allow(1));
        limiter.advance_tick(1);
        assert!(limiter.allow(1)); // reset
    }

    #[test]
    fn test_peer_rate_limiter_multiple_peers() {
        let mut limiter = PeerRateLimiter::new(2);
        assert!(limiter.allow(1));
        assert!(limiter.allow(2));
        assert!(limiter.allow(1));
        assert!(limiter.allow(2));
        assert!(!limiter.allow(1));
        assert!(!limiter.allow(2));
        assert_eq!(limiter.dropped, 2);
    }

    #[test]
    fn test_eclipse_attack_actions() {
        let eclipse = EclipseAttack::new(0, vec![8, 9], 300);
        let honest: Vec<NodeId> = (0..10).collect();
        let actions = eclipse.to_chaos_actions(&honest);
        assert!(!actions.is_empty());
        // Should have partition, broadcasts, wait, heal, wait
        assert!(actions.len() >= 5);
    }

    #[test]
    fn test_flood_attack_junk_generation() {
        let flood = FloodAttack::new(vec![5, 6], 10, 100);
        let junk = flood.generate_junk(0);
        assert_eq!(junk.len(), 20); // 2 attackers × 10 rate
    }

    #[test]
    fn test_flood_total_messages() {
        let flood = FloodAttack::new(vec![5, 6, 7], 50, 200);
        assert_eq!(flood.total_junk_messages(), 3 * 50 * 200);
    }

    #[test]
    fn test_timing_attack_actions() {
        let timing = TimingAttack::new(7, 0, 5000, 300);
        let actions = timing.to_chaos_actions();
        assert_eq!(actions.len(), 3); // set link, wait, restore link
    }

    #[test]
    fn test_suppression_attack_should_suppress() {
        let attack = SuppressionAttack::new(vec![5], MessageFilter::BlockAnnouncements, 100);
        let block = NetMessage::BlockAnnounce {
            height: 1,
            producer: 0,
            hash: [0; 32],
        };
        let ping = NetMessage::Ping { from: 5, seq: 1 };
        assert!(attack.should_suppress(&block, 5, 0));
        assert!(!attack.should_suppress(&ping, 5, 0));
        assert!(!attack.should_suppress(&block, 3, 0)); // not a relay
    }

    #[test]
    fn test_builder_eclipse_scenario() {
        let scenario = eclipse_single_validator();
        assert_eq!(scenario.name, "eclipse_single_validator");
        assert_eq!(scenario.nodes.len(), 10);
        assert!(!scenario.actions.is_empty());
    }

    #[test]
    fn test_builder_flood_scenario() {
        let scenario = sustained_flood();
        assert_eq!(scenario.name, "sustained_flood");
        assert_eq!(scenario.nodes.len(), 8);
    }

    #[test]
    fn test_builder_combined_scenario() {
        let scenario = combined_eclipse_and_flood();
        assert_eq!(scenario.name, "combined_eclipse_and_flood");
        assert_eq!(scenario.nodes.len(), 12);
        // Should have more actions than either attack alone.
        assert!(scenario.actions.len() > 20);
    }

    #[test]
    fn test_builder_sybil_scenario() {
        let scenario = sybil_vote_manipulation();
        assert_eq!(scenario.name, "sybil_vote_manipulation");
        assert_eq!(scenario.nodes.len(), 10);
    }

    #[test]
    fn test_builder_timing_scenario() {
        let scenario = timing_attack_on_block_producer();
        assert_eq!(scenario.name, "timing_attack_on_block_producer");
        assert_eq!(scenario.nodes.len(), 8);
    }

    #[test]
    fn test_builder_checkpoint_suppression_scenario() {
        let scenario = checkpoint_vote_suppression();
        assert_eq!(scenario.name, "checkpoint_vote_suppression");
    }

    #[test]
    fn test_adversarial_builder_custom() {
        let scenario = AdversarialScenarioBuilder::new("custom_attack")
            .with_nodes(20, 7)
            .with_recovery(1000)
            .add_attack(AttackType::Eclipse {
                victim: 2,
                attacker_nodes: vec![18, 19],
                isolation_ticks: 500,
            })
            .add_attack(AttackType::Flood {
                attackers: vec![15, 16, 17],
                rate: 200,
                duration_ticks: 100,
                targets: vec![0, 1],
            })
            .build();

        assert_eq!(scenario.name, "custom_attack");
        assert_eq!(scenario.nodes.len(), 20);
        assert!(scenario.nodes[0].is_validator);
        assert!(scenario.nodes[6].is_validator);
        assert!(!scenario.nodes[7].is_validator);
    }

    #[test]
    fn test_eclipse_verification_structure() {
        // Create a minimal sim to test verification.
        let mut sim = NetworkSim::new(LinkConfig::default());
        for i in 0..3 {
            sim.add_node(SimNode::new(i, i < 2, false, 100));
        }
        // All at tip 0 — should be converged.
        let v = EclipseAttack::verify_recovery(&sim);
        assert!(v.recovered);
        assert_eq!(v.max_divergence, 0);
    }

    #[test]
    fn test_rate_limiter_reset() {
        let mut limiter = PeerRateLimiter::new(1);
        assert!(limiter.allow(0));
        assert!(!limiter.allow(0));
        limiter.reset();
        assert_eq!(limiter.dropped, 0);
        assert!(limiter.allow(0));
    }
}
