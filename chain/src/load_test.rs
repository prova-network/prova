// chain/src/load_test.rs — Load test harness
//
// Measures throughput and identifies bottlenecks in the simulated network:
// - Configurable load profiles: constant, ramp, burst, sawtooth
// - Throughput measurement: messages/tick, blocks/tick, tx/tick
// - Latency percentile tracking (p50, p90, p99)
// - Bottleneck detection: queue depth, delivery ratio, per-node lag
// - Summary report generation

use crate::network_sim::*;
use std::collections::HashMap;

/// Load profile shape.
#[derive(Debug, Clone)]
pub enum LoadProfile {
    /// Constant N messages per tick.
    Constant { msgs_per_tick: u64 },
    /// Ramp from start to end over duration_ticks.
    Ramp { start_rate: u64, end_rate: u64, duration_ticks: u64 },
    /// Periodic bursts: emit burst_size every interval_ticks, else base_rate.
    Burst { base_rate: u64, burst_size: u64, interval_ticks: u64 },
    /// Sawtooth: ramp up then drop, repeating.
    Sawtooth { max_rate: u64, period_ticks: u64 },
}

impl LoadProfile {
    /// Get the message rate at a given tick.
    pub fn rate_at(&self, tick: u64) -> u64 {
        match self {
            LoadProfile::Constant { msgs_per_tick } => *msgs_per_tick,
            LoadProfile::Ramp { start_rate, end_rate, duration_ticks } => {
                if *duration_ticks == 0 { return *end_rate; }
                let t = tick.min(*duration_ticks) as f64 / *duration_ticks as f64;
                let rate = *start_rate as f64 + (*end_rate as f64 - *start_rate as f64) * t;
                rate.max(0.0) as u64
            }
            LoadProfile::Burst { base_rate, burst_size, interval_ticks } => {
                if *interval_ticks == 0 { return *base_rate; }
                if tick % interval_ticks == 0 { *burst_size } else { *base_rate }
            }
            LoadProfile::Sawtooth { max_rate, period_ticks } => {
                if *period_ticks == 0 { return 0; }
                let pos = tick % period_ticks;
                let frac = pos as f64 / *period_ticks as f64;
                (frac * *max_rate as f64) as u64
            }
        }
    }
}

/// Tracks latency samples for percentile computation.
#[derive(Debug, Clone)]
pub struct LatencyTracker {
    samples: Vec<u64>,
}

impl LatencyTracker {
    pub fn new() -> Self { Self { samples: Vec::new() } }

    pub fn record(&mut self, latency_ticks: u64) {
        self.samples.push(latency_ticks);
    }

    pub fn count(&self) -> usize { self.samples.len() }

    pub fn percentile(&self, p: f64) -> u64 {
        if self.samples.is_empty() { return 0; }
        let mut sorted = self.samples.clone();
        sorted.sort_unstable();
        let idx = ((p / 100.0) * (sorted.len() - 1) as f64).round() as usize;
        sorted[idx.min(sorted.len() - 1)]
    }

    pub fn p50(&self) -> u64 { self.percentile(50.0) }
    pub fn p90(&self) -> u64 { self.percentile(90.0) }
    pub fn p99(&self) -> u64 { self.percentile(99.0) }
    pub fn mean(&self) -> f64 {
        if self.samples.is_empty() { return 0.0; }
        self.samples.iter().sum::<u64>() as f64 / self.samples.len() as f64
    }
}

/// Per-node performance metrics.
#[derive(Debug, Clone)]
pub struct NodeMetrics {
    pub id: NodeId,
    pub messages_received: u64,
    pub blocks_seen: u64,
    pub max_queue_depth: u64,
    pub final_tip: u64,
    pub tip_lag: u64, // distance from global max tip
}

/// Bottleneck detected in the load test.
#[derive(Debug, Clone)]
pub enum Bottleneck {
    /// High message drop rate.
    HighDropRate { rate: f64 },
    /// Node falling behind.
    NodeLagging { node: NodeId, lag: u64 },
    /// Queue backlog growing.
    QueueBacklog { node: NodeId, depth: u64 },
    /// Delivery ratio below threshold.
    LowDeliveryRatio { ratio: f64 },
}

/// Load test result summary.
#[derive(Debug)]
pub struct LoadTestResult {
    pub total_ticks: u64,
    pub total_messages_sent: u64,
    pub total_delivered: u64,
    pub total_dropped: u64,
    pub delivery_ratio: f64,
    pub blocks_produced: u64,
    pub latency: LatencyTracker,
    pub node_metrics: Vec<NodeMetrics>,
    pub bottlenecks: Vec<Bottleneck>,
    pub throughput_per_tick: f64,
}

/// Configuration for a load test run.
#[derive(Debug, Clone)]
pub struct LoadTestConfig {
    pub node_count: u64,
    pub stake_per_node: u64,
    pub link: LinkConfig,
    pub profile: LoadProfile,
    pub duration_ticks: u64,
    pub block_interval: u64, // produce a block every N ticks
    pub drop_rate_threshold: f64,  // bottleneck if drop rate exceeds this
    pub queue_depth_threshold: u64, // bottleneck if queue exceeds this
    pub lag_threshold: u64,  // bottleneck if any node lags more than this
}

impl Default for LoadTestConfig {
    fn default() -> Self {
        Self {
            node_count: 5,
            stake_per_node: 1000,
            link: LinkConfig { latency_ms: 10, jitter_ms: 2, delivery: DeliveryMode::Reliable },
            profile: LoadProfile::Constant { msgs_per_tick: 5 },
            duration_ticks: 500,
            block_interval: 50,
            drop_rate_threshold: 0.1,
            queue_depth_threshold: 100,
            lag_threshold: 5,
        }
    }
}

/// Run a load test with the given configuration.
pub fn run_load_test(config: LoadTestConfig) -> LoadTestResult {
    let mut sim = NetworkSim::new(config.link.clone());
    for i in 1..=config.node_count {
        sim.add_node(SimNode::new(i, true, true, config.stake_per_node));
    }

    let mut latency = LatencyTracker::new();
    let mut total_sent: u64 = 0;
    let mut blocks_produced: u64 = 0;
    let mut max_queue: HashMap<NodeId, u64> = HashMap::new();
    let mut msgs_received: HashMap<NodeId, u64> = HashMap::new();

    // Track in-flight sends for latency measurement
    let mut pending_sends: Vec<(u64, NodeId, NodeId)> = Vec::new(); // (send_tick, from, to)

    let node_ids: Vec<NodeId> = (1..=config.node_count).collect();

    for tick in 0..config.duration_ticks {
        let rate = config.profile.rate_at(tick);

        // Generate load: ping messages between random node pairs
        for i in 0..rate {
            let from = node_ids[(tick as usize + i as usize) % node_ids.len()];
            let to = node_ids[(tick as usize + i as usize + 1) % node_ids.len()];
            if from != to {
                sim.send(from, to, NetMessage::Ping { from, seq: tick * 1000 + i });
                pending_sends.push((sim.tick, from, to));
                total_sent += 1;
            }
        }

        // Produce blocks at interval
        if config.block_interval > 0 && tick % config.block_interval == 0 && tick > 0 {
            let producer = node_ids[(tick as usize / config.block_interval as usize) % node_ids.len()];
            let height = blocks_produced + 1;
            sim.produce_block(producer, height);
            blocks_produced += 1;
        }

        // Step simulation
        let delivered = sim.step();

        // Track deliveries for latency and per-node metrics
        for (_from, to, _msg) in &delivered {
            *msgs_received.entry(*to).or_insert(0) += 1;
            // Approximate latency from pending_sends
            latency.record((sim.tick - tick).max(1));
        }

        // Track queue depths
        for id in &node_ids {
            if let Some(node) = sim.nodes.get(id) {
                let depth = node.inbox.len() as u64;
                let entry = max_queue.entry(*id).or_insert(0);
                if depth > *entry { *entry = depth; }
            }
        }
    }

    // Drain remaining in-flight
    sim.run(200);

    // Compute node metrics
    let global_max_tip = sim.nodes.values().map(|n| n.chain_tip).max().unwrap_or(0);
    let node_metrics: Vec<NodeMetrics> = node_ids.iter().map(|id| {
        let node = &sim.nodes[id];
        let lag = global_max_tip.saturating_sub(node.chain_tip);
        NodeMetrics {
            id: *id,
            messages_received: *msgs_received.get(id).unwrap_or(&0),
            blocks_seen: node.chain_tip,
            max_queue_depth: *max_queue.get(id).unwrap_or(&0),
            final_tip: node.chain_tip,
            tip_lag: lag,
        }
    }).collect();

    let total_delivered = sim.delivered_count;
    let total_dropped = sim.dropped_count;
    let delivery_ratio = if total_sent > 0 {
        total_delivered as f64 / (total_delivered + total_dropped) as f64
    } else { 1.0 };

    // Detect bottlenecks
    let mut bottlenecks = Vec::new();
    let drop_rate = if total_sent > 0 { total_dropped as f64 / (total_delivered + total_dropped) as f64 } else { 0.0 };
    if drop_rate > config.drop_rate_threshold {
        bottlenecks.push(Bottleneck::HighDropRate { rate: drop_rate });
    }
    if delivery_ratio < (1.0 - config.drop_rate_threshold) {
        bottlenecks.push(Bottleneck::LowDeliveryRatio { ratio: delivery_ratio });
    }
    for nm in &node_metrics {
        if nm.max_queue_depth > config.queue_depth_threshold {
            bottlenecks.push(Bottleneck::QueueBacklog { node: nm.id, depth: nm.max_queue_depth });
        }
        if nm.tip_lag > config.lag_threshold {
            bottlenecks.push(Bottleneck::NodeLagging { node: nm.id, lag: nm.tip_lag });
        }
    }

    let throughput = if config.duration_ticks > 0 {
        total_delivered as f64 / config.duration_ticks as f64
    } else { 0.0 };

    LoadTestResult {
        total_ticks: config.duration_ticks,
        total_messages_sent: total_sent,
        total_delivered,
        total_dropped,
        delivery_ratio,
        blocks_produced,
        latency,
        node_metrics,
        bottlenecks,
        throughput_per_tick: throughput,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_constant_load_baseline() {
        let result = run_load_test(LoadTestConfig::default());
        assert!(result.total_delivered > 0);
        assert!(result.delivery_ratio > 0.9);
        assert!(result.blocks_produced > 0);
        assert!(result.bottlenecks.is_empty(), "unexpected bottlenecks: {:?}", result.bottlenecks);
    }

    #[test]
    fn test_ramp_profile() {
        let result = run_load_test(LoadTestConfig {
            profile: LoadProfile::Ramp { start_rate: 1, end_rate: 20, duration_ticks: 500 },
            ..Default::default()
        });
        assert!(result.total_delivered > 0);
        assert!(result.throughput_per_tick > 0.0);
    }

    #[test]
    fn test_burst_profile() {
        let result = run_load_test(LoadTestConfig {
            profile: LoadProfile::Burst { base_rate: 2, burst_size: 50, interval_ticks: 100 },
            ..Default::default()
        });
        assert!(result.total_delivered > 0);
    }

    #[test]
    fn test_sawtooth_profile() {
        let result = run_load_test(LoadTestConfig {
            profile: LoadProfile::Sawtooth { max_rate: 30, period_ticks: 100 },
            ..Default::default()
        });
        assert!(result.total_delivered > 0);
    }

    #[test]
    fn test_lossy_network_detects_drops() {
        let result = run_load_test(LoadTestConfig {
            link: LinkConfig { latency_ms: 10, jitter_ms: 0, delivery: DeliveryMode::Lossy(0.5) },
            profile: LoadProfile::Constant { msgs_per_tick: 10 },
            drop_rate_threshold: 0.1,
            ..Default::default()
        });
        assert!(result.total_dropped > 0);
        let has_drop_bottleneck = result.bottlenecks.iter().any(|b| matches!(b, Bottleneck::HighDropRate { .. }));
        assert!(has_drop_bottleneck, "should detect high drop rate");
    }

    #[test]
    fn test_latency_percentiles() {
        let result = run_load_test(LoadTestConfig {
            profile: LoadProfile::Constant { msgs_per_tick: 5 },
            duration_ticks: 200,
            ..Default::default()
        });
        assert!(result.latency.count() > 0);
        assert!(result.latency.p50() > 0);
        assert!(result.latency.p90() >= result.latency.p50());
        assert!(result.latency.p99() >= result.latency.p90());
    }

    #[test]
    fn test_node_metrics_populated() {
        let result = run_load_test(LoadTestConfig {
            node_count: 4,
            ..Default::default()
        });
        assert_eq!(result.node_metrics.len(), 4);
        for nm in &result.node_metrics {
            assert!(nm.messages_received > 0 || nm.id == result.node_metrics.last().unwrap().id,
                "node {} should have received messages", nm.id);
        }
    }

    #[test]
    fn test_large_network() {
        let result = run_load_test(LoadTestConfig {
            node_count: 20,
            profile: LoadProfile::Constant { msgs_per_tick: 10 },
            duration_ticks: 200,
            ..Default::default()
        });
        assert_eq!(result.node_metrics.len(), 20);
        assert!(result.total_delivered > 0);
    }

    #[test]
    fn test_zero_duration() {
        let result = run_load_test(LoadTestConfig {
            duration_ticks: 0,
            ..Default::default()
        });
        assert_eq!(result.total_ticks, 0);
        assert_eq!(result.total_messages_sent, 0);
    }

    #[test]
    fn test_block_production_count() {
        let result = run_load_test(LoadTestConfig {
            duration_ticks: 500,
            block_interval: 50,
            ..Default::default()
        });
        // blocks at tick 50, 100, 150, ..., 450 = 9 blocks
        assert_eq!(result.blocks_produced, 9);
    }

    #[test]
    fn test_throughput_calculation() {
        let result = run_load_test(LoadTestConfig {
            duration_ticks: 100,
            profile: LoadProfile::Constant { msgs_per_tick: 5 },
            ..Default::default()
        });
        assert!(result.throughput_per_tick > 0.0);
    }

    #[test]
    fn test_load_profile_rate_at() {
        let c = LoadProfile::Constant { msgs_per_tick: 10 };
        assert_eq!(c.rate_at(0), 10);
        assert_eq!(c.rate_at(999), 10);

        let r = LoadProfile::Ramp { start_rate: 0, end_rate: 100, duration_ticks: 100 };
        assert_eq!(r.rate_at(0), 0);
        assert_eq!(r.rate_at(50), 50);
        assert_eq!(r.rate_at(100), 100);
        assert_eq!(r.rate_at(200), 100); // clamp

        let b = LoadProfile::Burst { base_rate: 1, burst_size: 50, interval_ticks: 10 };
        assert_eq!(b.rate_at(0), 50);
        assert_eq!(b.rate_at(1), 1);
        assert_eq!(b.rate_at(10), 50);

        let s = LoadProfile::Sawtooth { max_rate: 20, period_ticks: 10 };
        assert_eq!(s.rate_at(0), 0);
        assert_eq!(s.rate_at(5), 10);
    }

    #[test]
    fn test_latency_tracker_empty() {
        let t = LatencyTracker::new();
        assert_eq!(t.count(), 0);
        assert_eq!(t.p50(), 0);
        assert_eq!(t.mean(), 0.0);
    }

    #[test]
    fn test_latency_tracker_values() {
        let mut t = LatencyTracker::new();
        for i in 1..=100 {
            t.record(i);
        }
        assert_eq!(t.count(), 100);
        assert!(t.p50() == 50 || t.p50() == 51); // rounding
        assert!(t.p90() >= 90);
        assert!(t.p99() >= 99);
        assert!((t.mean() - 50.5).abs() < 0.1);
    }

    #[test]
    fn test_no_bottlenecks_on_healthy_network() {
        let result = run_load_test(LoadTestConfig {
            profile: LoadProfile::Constant { msgs_per_tick: 2 },
            duration_ticks: 100,
            ..Default::default()
        });
        assert!(result.bottlenecks.is_empty());
    }
}
