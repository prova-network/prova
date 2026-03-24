// chain/src/chaos.rs — Chaos scenario runner
//
// Provides a scripted failure-injection framework on top of NetworkSim:
// - Declarative scenario DSL: sequence of ChaosAction steps
// - Actions: crash, restart, partition, heal, produce block, inject message,
//   set link quality, wait ticks, assert convergence
// - Convergence assertions: all-same-tip, min-alive, max-tip-divergence
// - Scenario runner with step-by-step execution and full event log
// - Pre-built scenarios: split-brain recovery, cascading crashes, rolling restart

use crate::network_sim::*;
use std::collections::HashMap;

/// A single action in a chaos scenario script.
#[derive(Debug, Clone)]
pub enum ChaosAction {
    /// Crash a specific node.
    CrashNode(NodeId),
    /// Restart a specific node.
    RestartNode(NodeId),
    /// Create a network partition.
    Partition {
        group_a: Vec<NodeId>,
        group_b: Vec<NodeId>,
        duration_ticks: u64,
    },
    /// Remove all active partitions.
    HealAll,
    /// Produce a block from a specific node.
    ProduceBlock { producer: NodeId, height: u64 },
    /// Broadcast a message from a node.
    BroadcastMessage { from: NodeId, msg: NetMessage },
    /// Set link quality between two nodes.
    SetLink {
        a: NodeId,
        b: NodeId,
        config: LinkConfig,
    },
    /// Advance simulation by N ticks.
    Wait(u64),
    /// Assert all alive nodes have the same chain tip.
    AssertConverged,
    /// Assert all alive nodes have chain tip >= given height.
    AssertMinTip(u64),
    /// Assert at least N nodes are alive.
    AssertMinAlive(u64),
    /// Assert max tip divergence among alive nodes is <= N.
    AssertMaxDivergence(u64),
    /// Assert a specific node is crashed.
    AssertCrashed(NodeId),
    /// Assert a specific node is alive.
    AssertAlive(NodeId),
}

/// A recorded event from scenario execution.
#[derive(Debug, Clone)]
pub struct ChaosEvent {
    pub tick: u64,
    pub step: usize,
    pub action: String,
    pub detail: String,
}

/// Result of running a chaos scenario.
#[derive(Debug)]
pub struct ChaosResult {
    pub success: bool,
    pub steps_executed: usize,
    pub events: Vec<ChaosEvent>,
    pub failure: Option<String>,
    pub final_stats: SimStats,
}

/// A complete chaos scenario: setup + scripted actions.
#[derive(Debug, Clone)]
pub struct ChaosScenario {
    pub name: String,
    pub nodes: Vec<SimNode>,
    pub default_link: LinkConfig,
    pub actions: Vec<ChaosAction>,
}

impl ChaosScenario {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            nodes: Vec::new(),
            default_link: LinkConfig::default(),
            actions: Vec::new(),
        }
    }

    pub fn with_default_link(mut self, link: LinkConfig) -> Self {
        self.default_link = link;
        self
    }

    pub fn add_node(mut self, node: SimNode) -> Self {
        self.nodes.push(node);
        self
    }

    pub fn add_validators(mut self, count: u64, stake: u64) -> Self {
        let base = self.nodes.len() as u64 + 1;
        for i in 0..count {
            self.nodes.push(SimNode::new(base + i, true, false, stake));
        }
        self
    }

    pub fn then(mut self, action: ChaosAction) -> Self {
        self.actions.push(action);
        self
    }

    /// Run the scenario, returning results.
    pub fn run(self) -> ChaosResult {
        let mut sim = NetworkSim::new(self.default_link);
        for node in self.nodes {
            sim.add_node(node);
        }

        let mut events = Vec::new();
        let mut partition_counter: u64 = 0;

        for (step_idx, action) in self.actions.iter().enumerate() {
            let tick = sim.tick;
            match action {
                ChaosAction::CrashNode(id) => {
                    if let Some(node) = sim.nodes.get_mut(id) {
                        node.crash();
                        events.push(ChaosEvent {
                            tick,
                            step: step_idx,
                            action: "crash".into(),
                            detail: format!("node {} crashed", id),
                        });
                    }
                }
                ChaosAction::RestartNode(id) => {
                    if let Some(node) = sim.nodes.get_mut(id) {
                        node.restart();
                        events.push(ChaosEvent {
                            tick,
                            step: step_idx,
                            action: "restart".into(),
                            detail: format!("node {} restarted", id),
                        });
                    }
                }
                ChaosAction::Partition {
                    group_a,
                    group_b,
                    duration_ticks,
                } => {
                    sim.add_partition(Partition {
                        group_a: group_a.clone(),
                        group_b: group_b.clone(),
                        start_tick: tick,
                        end_tick: tick + duration_ticks,
                    });
                    partition_counter += 1;
                    events.push(ChaosEvent {
                        tick,
                        step: step_idx,
                        action: "partition".into(),
                        detail: format!(
                            "partition #{}: {:?} | {:?} for {} ticks",
                            partition_counter, group_a, group_b, duration_ticks
                        ),
                    });
                }
                ChaosAction::HealAll => {
                    sim.partitions.clear();
                    events.push(ChaosEvent {
                        tick,
                        step: step_idx,
                        action: "heal".into(),
                        detail: "all partitions removed".into(),
                    });
                }
                ChaosAction::ProduceBlock { producer, height } => {
                    sim.produce_block(*producer, *height);
                    events.push(ChaosEvent {
                        tick,
                        step: step_idx,
                        action: "block".into(),
                        detail: format!("node {} produced block {}", producer, height),
                    });
                }
                ChaosAction::BroadcastMessage { from, msg } => {
                    sim.broadcast(*from, msg.clone());
                    events.push(ChaosEvent {
                        tick,
                        step: step_idx,
                        action: "broadcast".into(),
                        detail: format!("node {} broadcast {:?}", from, msg),
                    });
                }
                ChaosAction::SetLink { a, b, config } => {
                    sim.set_link(*a, *b, config.clone());
                    events.push(ChaosEvent {
                        tick,
                        step: step_idx,
                        action: "link".into(),
                        detail: format!("link {}↔{} set to {:?}", a, b, config.delivery),
                    });
                }
                ChaosAction::Wait(ticks) => {
                    sim.run(*ticks);
                    events.push(ChaosEvent {
                        tick,
                        step: step_idx,
                        action: "wait".into(),
                        detail: format!("advanced {} ticks (now at {})", ticks, sim.tick),
                    });
                }
                ChaosAction::AssertConverged => {
                    let tips: Vec<u64> = sim
                        .nodes
                        .values()
                        .filter(|n| !n.crashed)
                        .map(|n| n.chain_tip)
                        .collect();
                    if tips.is_empty() {
                        return ChaosResult {
                            success: false,
                            steps_executed: step_idx,
                            events,
                            failure: Some("convergence assert: no alive nodes".into()),
                            final_stats: sim.run(0),
                        };
                    }
                    let first = tips[0];
                    if !tips.iter().all(|t| *t == first) {
                        let tip_map: HashMap<NodeId, u64> = sim
                            .nodes
                            .iter()
                            .filter(|(_, n)| !n.crashed)
                            .map(|(id, n)| (*id, n.chain_tip))
                            .collect();
                        return ChaosResult {
                            success: false,
                            steps_executed: step_idx,
                            events,
                            failure: Some(format!(
                                "convergence assert failed: tips = {:?}",
                                tip_map
                            )),
                            final_stats: sim.run(0),
                        };
                    }
                    events.push(ChaosEvent {
                        tick,
                        step: step_idx,
                        action: "assert".into(),
                        detail: format!("converged at tip {}", first),
                    });
                }
                ChaosAction::AssertMinTip(min) => {
                    for (id, node) in &sim.nodes {
                        if !node.crashed && node.chain_tip < *min {
                            return ChaosResult {
                                success: false,
                                steps_executed: step_idx,
                                events,
                                failure: Some(format!(
                                    "min-tip assert: node {} at tip {} < {}",
                                    id, node.chain_tip, min
                                )),
                                final_stats: sim.run(0),
                            };
                        }
                    }
                    events.push(ChaosEvent {
                        tick,
                        step: step_idx,
                        action: "assert".into(),
                        detail: format!("all alive nodes at tip >= {}", min),
                    });
                }
                ChaosAction::AssertMinAlive(min) => {
                    let alive = sim.nodes.values().filter(|n| !n.crashed).count() as u64;
                    if alive < *min {
                        return ChaosResult {
                            success: false,
                            steps_executed: step_idx,
                            events,
                            failure: Some(format!("min-alive assert: {} alive < {}", alive, min)),
                            final_stats: sim.run(0),
                        };
                    }
                    events.push(ChaosEvent {
                        tick,
                        step: step_idx,
                        action: "assert".into(),
                        detail: format!("{} nodes alive (>= {})", alive, min),
                    });
                }
                ChaosAction::AssertMaxDivergence(max_div) => {
                    let tips: Vec<u64> = sim
                        .nodes
                        .values()
                        .filter(|n| !n.crashed)
                        .map(|n| n.chain_tip)
                        .collect();
                    if tips.is_empty() {
                        events.push(ChaosEvent {
                            tick,
                            step: step_idx,
                            action: "assert".into(),
                            detail: "no alive nodes, divergence trivially 0".into(),
                        });
                    } else {
                        let min_tip = *tips.iter().min().unwrap();
                        let max_tip = *tips.iter().max().unwrap();
                        let div = max_tip - min_tip;
                        if div > *max_div {
                            return ChaosResult {
                                success: false,
                                steps_executed: step_idx,
                                events,
                                failure: Some(format!(
                                    "max-divergence assert: {} > {} (tips: {:?})",
                                    div, max_div, tips
                                )),
                                final_stats: sim.run(0),
                            };
                        }
                        events.push(ChaosEvent {
                            tick,
                            step: step_idx,
                            action: "assert".into(),
                            detail: format!("divergence {} <= {}", div, max_div),
                        });
                    }
                }
                ChaosAction::AssertCrashed(id) => {
                    let crashed = sim.nodes.get(id).map_or(false, |n| n.crashed);
                    if !crashed {
                        return ChaosResult {
                            success: false,
                            steps_executed: step_idx,
                            events,
                            failure: Some(format!("node {} expected crashed but is alive", id)),
                            final_stats: sim.run(0),
                        };
                    }
                    events.push(ChaosEvent {
                        tick,
                        step: step_idx,
                        action: "assert".into(),
                        detail: format!("node {} is crashed", id),
                    });
                }
                ChaosAction::AssertAlive(id) => {
                    let alive = sim.nodes.get(id).map_or(false, |n| !n.crashed);
                    if !alive {
                        return ChaosResult {
                            success: false,
                            steps_executed: step_idx,
                            events,
                            failure: Some(format!(
                                "node {} expected alive but is crashed/missing",
                                id
                            )),
                            final_stats: sim.run(0),
                        };
                    }
                    events.push(ChaosEvent {
                        tick,
                        step: step_idx,
                        action: "assert".into(),
                        detail: format!("node {} is alive", id),
                    });
                }
            }
        }

        ChaosResult {
            success: true,
            steps_executed: self.actions.len(),
            events,
            failure: None,
            final_stats: sim.run(0),
        }
    }
}

// ─── Pre-built Scenarios ─────────────────────────────────────────

/// Split-brain: 5 nodes partition into 2|3, both sides produce blocks,
/// heal, then assert convergence to the longer chain.
pub fn scenario_split_brain_recovery() -> ChaosScenario {
    ChaosScenario::new("split-brain-recovery")
        .add_validators(5, 1000)
        .with_default_link(LinkConfig {
            latency_ms: 10,
            jitter_ms: 2,
            delivery: DeliveryMode::Reliable,
        })
        // Produce initial blocks all nodes agree on.
        .then(ChaosAction::ProduceBlock {
            producer: 1,
            height: 1,
        })
        .then(ChaosAction::Wait(50))
        .then(ChaosAction::AssertConverged)
        // Partition: [1,2] | [3,4,5]
        .then(ChaosAction::Partition {
            group_a: vec![1, 2],
            group_b: vec![3, 4, 5],
            duration_ticks: 300,
        })
        // Group A produces block 2
        .then(ChaosAction::ProduceBlock {
            producer: 1,
            height: 2,
        })
        .then(ChaosAction::Wait(50))
        // Group B produces blocks 2 and 3 (longer chain)
        .then(ChaosAction::ProduceBlock {
            producer: 3,
            height: 2,
        })
        .then(ChaosAction::Wait(30))
        .then(ChaosAction::ProduceBlock {
            producer: 4,
            height: 3,
        })
        .then(ChaosAction::Wait(50))
        // During partition: divergence exists
        .then(ChaosAction::AssertMaxDivergence(2))
        // Wait for partition to heal
        .then(ChaosAction::Wait(200))
        // After heal, broadcast the longer chain
        .then(ChaosAction::ProduceBlock {
            producer: 3,
            height: 4,
        })
        .then(ChaosAction::Wait(100))
        .then(ChaosAction::AssertConverged)
        .then(ChaosAction::AssertMinTip(4))
}

/// Cascading crashes: crash nodes one by one, assert remaining alive,
/// then restart all and verify recovery.
pub fn scenario_cascading_crashes() -> ChaosScenario {
    ChaosScenario::new("cascading-crashes")
        .add_validators(5, 1000)
        .with_default_link(LinkConfig {
            latency_ms: 10,
            jitter_ms: 0,
            delivery: DeliveryMode::Reliable,
        })
        .then(ChaosAction::ProduceBlock {
            producer: 1,
            height: 1,
        })
        .then(ChaosAction::Wait(50))
        .then(ChaosAction::AssertConverged)
        // Crash nodes one at a time
        .then(ChaosAction::CrashNode(5))
        .then(ChaosAction::AssertCrashed(5))
        .then(ChaosAction::AssertMinAlive(4))
        .then(ChaosAction::ProduceBlock {
            producer: 1,
            height: 2,
        })
        .then(ChaosAction::Wait(50))
        .then(ChaosAction::CrashNode(4))
        .then(ChaosAction::AssertMinAlive(3))
        .then(ChaosAction::ProduceBlock {
            producer: 1,
            height: 3,
        })
        .then(ChaosAction::Wait(50))
        .then(ChaosAction::CrashNode(3))
        .then(ChaosAction::AssertMinAlive(2))
        // Only nodes 1,2 alive — they should still converge
        .then(ChaosAction::ProduceBlock {
            producer: 2,
            height: 4,
        })
        .then(ChaosAction::Wait(50))
        .then(ChaosAction::AssertConverged)
        // Restart all crashed nodes
        .then(ChaosAction::RestartNode(3))
        .then(ChaosAction::RestartNode(4))
        .then(ChaosAction::RestartNode(5))
        .then(ChaosAction::AssertMinAlive(5))
        // New block should reach everyone
        .then(ChaosAction::ProduceBlock {
            producer: 1,
            height: 5,
        })
        .then(ChaosAction::Wait(100))
        .then(ChaosAction::AssertConverged)
        .then(ChaosAction::AssertMinTip(5))
}

/// Rolling restart: restart nodes one at a time while blocks keep being produced.
pub fn scenario_rolling_restart() -> ChaosScenario {
    ChaosScenario::new("rolling-restart")
        .add_validators(4, 1000)
        .with_default_link(LinkConfig {
            latency_ms: 10,
            jitter_ms: 0,
            delivery: DeliveryMode::Reliable,
        })
        .then(ChaosAction::ProduceBlock {
            producer: 1,
            height: 1,
        })
        .then(ChaosAction::Wait(50))
        // Restart node 1
        .then(ChaosAction::CrashNode(1))
        .then(ChaosAction::ProduceBlock {
            producer: 2,
            height: 2,
        })
        .then(ChaosAction::Wait(30))
        .then(ChaosAction::RestartNode(1))
        .then(ChaosAction::Wait(20))
        // Restart node 2
        .then(ChaosAction::CrashNode(2))
        .then(ChaosAction::ProduceBlock {
            producer: 3,
            height: 3,
        })
        .then(ChaosAction::Wait(30))
        .then(ChaosAction::RestartNode(2))
        .then(ChaosAction::Wait(20))
        // Restart node 3
        .then(ChaosAction::CrashNode(3))
        .then(ChaosAction::ProduceBlock {
            producer: 4,
            height: 4,
        })
        .then(ChaosAction::Wait(30))
        .then(ChaosAction::RestartNode(3))
        .then(ChaosAction::Wait(20))
        // Restart node 4
        .then(ChaosAction::CrashNode(4))
        .then(ChaosAction::ProduceBlock {
            producer: 1,
            height: 5,
        })
        .then(ChaosAction::Wait(30))
        .then(ChaosAction::RestartNode(4))
        // Re-broadcast so restarted node catches up
        .then(ChaosAction::ProduceBlock {
            producer: 1,
            height: 6,
        })
        .then(ChaosAction::Wait(50))
        // Everyone should be alive and converged
        .then(ChaosAction::AssertMinAlive(4))
        .then(ChaosAction::AssertConverged)
        .then(ChaosAction::AssertMinTip(6))
}

/// Lossy network: all links have 30% packet loss, blocks should still propagate.
pub fn scenario_lossy_network_convergence() -> ChaosScenario {
    ChaosScenario::new("lossy-network-convergence")
        .add_validators(5, 1000)
        .with_default_link(LinkConfig {
            latency_ms: 20,
            jitter_ms: 10,
            delivery: DeliveryMode::Lossy(0.3),
        })
        // Produce several blocks with redundant broadcasts
        .then(ChaosAction::ProduceBlock {
            producer: 1,
            height: 1,
        })
        .then(ChaosAction::Wait(100))
        .then(ChaosAction::ProduceBlock {
            producer: 1,
            height: 1,
        }) // re-broadcast
        .then(ChaosAction::Wait(100))
        .then(ChaosAction::ProduceBlock {
            producer: 2,
            height: 2,
        })
        .then(ChaosAction::Wait(100))
        .then(ChaosAction::ProduceBlock {
            producer: 2,
            height: 2,
        }) // re-broadcast
        .then(ChaosAction::Wait(100))
        .then(ChaosAction::ProduceBlock {
            producer: 3,
            height: 3,
        })
        .then(ChaosAction::Wait(100))
        .then(ChaosAction::ProduceBlock {
            producer: 3,
            height: 3,
        }) // re-broadcast
        .then(ChaosAction::Wait(200))
        // With re-broadcasts and 30% loss, should converge
        .then(ChaosAction::AssertMinTip(3))
}

/// Rapid partition flip-flop: partitions change every 50 ticks.
pub fn scenario_partition_flapping() -> ChaosScenario {
    ChaosScenario::new("partition-flapping")
        .add_validators(4, 1000)
        .with_default_link(LinkConfig {
            latency_ms: 5,
            jitter_ms: 0,
            delivery: DeliveryMode::Reliable,
        })
        .then(ChaosAction::ProduceBlock {
            producer: 1,
            height: 1,
        })
        .then(ChaosAction::Wait(20))
        // Partition A
        .then(ChaosAction::Partition {
            group_a: vec![1, 2],
            group_b: vec![3, 4],
            duration_ticks: 50,
        })
        .then(ChaosAction::ProduceBlock {
            producer: 1,
            height: 2,
        })
        .then(ChaosAction::Wait(60)) // partition heals
        // Partition B (reversed)
        .then(ChaosAction::Partition {
            group_a: vec![1, 3],
            group_b: vec![2, 4],
            duration_ticks: 50,
        })
        .then(ChaosAction::ProduceBlock {
            producer: 2,
            height: 3,
        })
        .then(ChaosAction::Wait(60)) // heals
        // Partition C
        .then(ChaosAction::Partition {
            group_a: vec![1, 4],
            group_b: vec![2, 3],
            duration_ticks: 50,
        })
        .then(ChaosAction::ProduceBlock {
            producer: 3,
            height: 4,
        })
        .then(ChaosAction::Wait(60)) // heals
        // Final convergence
        .then(ChaosAction::ProduceBlock {
            producer: 4,
            height: 5,
        })
        .then(ChaosAction::Wait(100))
        .then(ChaosAction::AssertConverged)
        .then(ChaosAction::AssertMinTip(5))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_split_brain_recovery() {
        let result = scenario_split_brain_recovery().run();
        assert!(result.success, "failed: {:?}", result.failure);
        assert!(result.events.len() > 5);
    }

    #[test]
    fn test_cascading_crashes() {
        let result = scenario_cascading_crashes().run();
        assert!(result.success, "failed: {:?}", result.failure);
    }

    #[test]
    fn test_rolling_restart() {
        let result = scenario_rolling_restart().run();
        assert!(result.success, "failed: {:?}", result.failure);
    }

    #[test]
    fn test_lossy_network() {
        let result = scenario_lossy_network_convergence().run();
        assert!(result.success, "failed: {:?}", result.failure);
    }

    #[test]
    fn test_partition_flapping() {
        let result = scenario_partition_flapping().run();
        assert!(result.success, "failed: {:?}", result.failure);
    }

    #[test]
    fn test_empty_scenario() {
        let result = ChaosScenario::new("empty").add_validators(3, 1000).run();
        assert!(result.success);
        assert_eq!(result.steps_executed, 0);
    }

    #[test]
    fn test_single_crash_assert() {
        let result = ChaosScenario::new("single-crash")
            .add_validators(3, 1000)
            .then(ChaosAction::CrashNode(1))
            .then(ChaosAction::AssertCrashed(1))
            .then(ChaosAction::AssertAlive(2))
            .then(ChaosAction::AssertMinAlive(2))
            .run();
        assert!(result.success, "failed: {:?}", result.failure);
    }

    #[test]
    fn test_assert_converged_fails_on_divergence() {
        let result = ChaosScenario::new("divergence-fail")
            .add_validators(3, 1000)
            .with_default_link(LinkConfig {
                latency_ms: 1000,
                jitter_ms: 0,
                delivery: DeliveryMode::Reliable,
            })
            .then(ChaosAction::ProduceBlock {
                producer: 1,
                height: 5,
            })
            // Don't wait long enough for propagation
            .then(ChaosAction::Wait(1))
            .then(ChaosAction::AssertConverged)
            .run();
        assert!(!result.success);
        assert!(result
            .failure
            .unwrap()
            .contains("convergence assert failed"));
    }

    #[test]
    fn test_assert_min_alive_fails() {
        let result = ChaosScenario::new("min-alive-fail")
            .add_validators(3, 1000)
            .then(ChaosAction::CrashNode(1))
            .then(ChaosAction::CrashNode(2))
            .then(ChaosAction::AssertMinAlive(3))
            .run();
        assert!(!result.success);
        assert!(result.failure.unwrap().contains("min-alive"));
    }

    #[test]
    fn test_assert_max_divergence_fails() {
        let result = ChaosScenario::new("divergence-limit-fail")
            .add_validators(2, 1000)
            .with_default_link(LinkConfig {
                latency_ms: 5000,
                jitter_ms: 0,
                delivery: DeliveryMode::Reliable,
            })
            .then(ChaosAction::ProduceBlock {
                producer: 1,
                height: 10,
            })
            .then(ChaosAction::Wait(1))
            .then(ChaosAction::AssertMaxDivergence(0))
            .run();
        assert!(!result.success);
        assert!(result.failure.unwrap().contains("max-divergence"));
    }

    #[test]
    fn test_heal_all_removes_partitions() {
        let result = ChaosScenario::new("heal-all")
            .add_validators(4, 1000)
            .with_default_link(LinkConfig {
                latency_ms: 5,
                jitter_ms: 0,
                delivery: DeliveryMode::Reliable,
            })
            .then(ChaosAction::Partition {
                group_a: vec![1, 2],
                group_b: vec![3, 4],
                duration_ticks: 100000, // very long
            })
            .then(ChaosAction::ProduceBlock {
                producer: 1,
                height: 1,
            })
            .then(ChaosAction::Wait(50))
            // Node 3 shouldn't see block yet
            .then(ChaosAction::HealAll)
            .then(ChaosAction::ProduceBlock {
                producer: 1,
                height: 2,
            })
            .then(ChaosAction::Wait(50))
            .then(ChaosAction::AssertConverged)
            .run();
        assert!(result.success, "failed: {:?}", result.failure);
    }

    #[test]
    fn test_set_link_quality() {
        let result = ChaosScenario::new("link-quality")
            .add_validators(2, 1000)
            .then(ChaosAction::SetLink {
                a: 1,
                b: 2,
                config: LinkConfig {
                    latency_ms: 5,
                    jitter_ms: 0,
                    delivery: DeliveryMode::Reliable,
                },
            })
            .then(ChaosAction::ProduceBlock {
                producer: 1,
                height: 1,
            })
            .then(ChaosAction::Wait(20))
            .then(ChaosAction::AssertConverged)
            .run();
        assert!(result.success, "failed: {:?}", result.failure);
    }

    #[test]
    fn test_broadcast_message_action() {
        let result = ChaosScenario::new("broadcast-msg")
            .add_validators(3, 1000)
            .then(ChaosAction::BroadcastMessage {
                from: 1,
                msg: NetMessage::CheckpointProposal {
                    epoch: 1,
                    proposer: 1,
                    state_root: [0xAA; 32],
                },
            })
            .then(ChaosAction::Wait(100))
            .then(ChaosAction::AssertMinAlive(3))
            .run();
        assert!(result.success, "failed: {:?}", result.failure);
    }

    #[test]
    fn test_event_log_populated() {
        let result = ChaosScenario::new("event-log")
            .add_validators(2, 1000)
            .then(ChaosAction::CrashNode(1))
            .then(ChaosAction::RestartNode(1))
            .then(ChaosAction::ProduceBlock {
                producer: 1,
                height: 1,
            })
            .then(ChaosAction::Wait(50))
            .run();
        assert!(result.success);
        assert_eq!(result.events.len(), 4);
        assert_eq!(result.events[0].action, "crash");
        assert_eq!(result.events[1].action, "restart");
        assert_eq!(result.events[2].action, "block");
        assert_eq!(result.events[3].action, "wait");
    }

    #[test]
    fn test_scenario_builder_chaining() {
        let scenario = ChaosScenario::new("chain-test")
            .add_validators(3, 500)
            .with_default_link(LinkConfig {
                latency_ms: 1,
                jitter_ms: 0,
                delivery: DeliveryMode::Reliable,
            })
            .then(ChaosAction::ProduceBlock {
                producer: 1,
                height: 1,
            })
            .then(ChaosAction::Wait(10))
            .then(ChaosAction::AssertConverged);
        assert_eq!(scenario.name, "chain-test");
        assert_eq!(scenario.nodes.len(), 3);
        assert_eq!(scenario.actions.len(), 3);
        let result = scenario.run();
        assert!(result.success);
    }

    #[test]
    fn test_assert_min_tip_fails() {
        let result = ChaosScenario::new("min-tip-fail")
            .add_validators(2, 1000)
            .then(ChaosAction::AssertMinTip(5))
            .run();
        assert!(!result.success);
        assert!(result.failure.unwrap().contains("min-tip"));
    }

    #[test]
    fn test_combined_crash_partition_recovery() {
        let result = ChaosScenario::new("combined-chaos")
            .add_validators(6, 1000)
            .with_default_link(LinkConfig {
                latency_ms: 5,
                jitter_ms: 0,
                delivery: DeliveryMode::Reliable,
            })
            .then(ChaosAction::ProduceBlock {
                producer: 1,
                height: 1,
            })
            .then(ChaosAction::Wait(30))
            .then(ChaosAction::AssertConverged)
            // Crash 2 nodes AND partition the rest
            .then(ChaosAction::CrashNode(5))
            .then(ChaosAction::CrashNode(6))
            .then(ChaosAction::Partition {
                group_a: vec![1, 2],
                group_b: vec![3, 4],
                duration_ticks: 100,
            })
            .then(ChaosAction::ProduceBlock {
                producer: 1,
                height: 2,
            })
            .then(ChaosAction::Wait(50))
            .then(ChaosAction::AssertMinAlive(4))
            // Heal and restart
            .then(ChaosAction::Wait(60)) // partition heals at tick ~140
            .then(ChaosAction::RestartNode(5))
            .then(ChaosAction::RestartNode(6))
            .then(ChaosAction::ProduceBlock {
                producer: 3,
                height: 3,
            })
            .then(ChaosAction::Wait(100))
            .then(ChaosAction::AssertConverged)
            .then(ChaosAction::AssertMinAlive(6))
            .then(ChaosAction::AssertMinTip(3))
            .run();
        assert!(result.success, "failed: {:?}", result.failure);
    }
}
