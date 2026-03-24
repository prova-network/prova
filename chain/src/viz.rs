// chain/src/viz.rs — Visualization output (JSON trace for timeline rendering)
//
// SIM-004: Produces structured JSON traces from network simulation runs.
// Output format is compatible with Chrome Trace Event Format (catapult)
// and custom Prova timeline renderers.
//
// Captures:
// - Message sends/receives between nodes (flow events)
// - Block production instants
// - Node state changes (crash/restart/partition)
// - Dispute lifecycle (challenge → bisection → resolution)
// - Aggregated statistics per node

use std::collections::BTreeMap;

/// A single trace event in Chrome Trace Event Format style.
#[derive(Debug, Clone, PartialEq)]
pub struct TraceEvent {
    /// Event name (e.g. "BlockAnnounce", "NodeCrash").
    pub name: String,
    /// Category for grouping (e.g. "block", "message", "dispute", "lifecycle").
    pub category: String,
    /// Phase: B=begin, E=end, i=instant, s=flow_start, f=flow_end, X=complete.
    pub phase: char,
    /// Timestamp in microseconds from simulation start.
    pub timestamp_us: u64,
    /// Duration in microseconds (for phase='X' complete events).
    pub duration_us: Option<u64>,
    /// Process ID (maps to node ID in our case).
    pub pid: u64,
    /// Thread ID (0 for single-threaded sim nodes).
    pub tid: u64,
    /// Flow ID for linking send→receive pairs.
    pub flow_id: Option<u64>,
    /// Arbitrary key-value args.
    pub args: BTreeMap<String, TraceValue>,
}

/// Values storable in trace event args.
#[derive(Debug, Clone, PartialEq)]
pub enum TraceValue {
    String(String),
    Number(i64),
    Float(f64),
    Bool(bool),
}

impl TraceValue {
    pub fn to_json(&self) -> String {
        match self {
            TraceValue::String(s) => {
                format!("\"{}\"", s.replace('\\', "\\\\").replace('"', "\\\""))
            }
            TraceValue::Number(n) => n.to_string(),
            TraceValue::Float(f) => format!("{:.6}", f),
            TraceValue::Bool(b) => b.to_string(),
        }
    }
}

impl TraceEvent {
    pub fn instant(name: &str, category: &str, timestamp_us: u64, pid: u64) -> Self {
        Self {
            name: name.to_string(),
            category: category.to_string(),
            phase: 'i',
            timestamp_us,
            duration_us: None,
            pid,
            tid: 0,
            flow_id: None,
            args: BTreeMap::new(),
        }
    }

    pub fn complete(
        name: &str,
        category: &str,
        timestamp_us: u64,
        duration_us: u64,
        pid: u64,
    ) -> Self {
        Self {
            name: name.to_string(),
            category: category.to_string(),
            phase: 'X',
            timestamp_us,
            duration_us: Some(duration_us),
            pid,
            tid: 0,
            flow_id: None,
            args: BTreeMap::new(),
        }
    }

    pub fn flow_start(
        name: &str,
        category: &str,
        timestamp_us: u64,
        pid: u64,
        flow_id: u64,
    ) -> Self {
        Self {
            name: name.to_string(),
            category: category.to_string(),
            phase: 's',
            timestamp_us,
            duration_us: None,
            pid,
            tid: 0,
            flow_id: Some(flow_id),
            args: BTreeMap::new(),
        }
    }

    pub fn flow_end(name: &str, category: &str, timestamp_us: u64, pid: u64, flow_id: u64) -> Self {
        Self {
            name: name.to_string(),
            category: category.to_string(),
            phase: 'f',
            timestamp_us,
            duration_us: None,
            pid,
            tid: 0,
            flow_id: Some(flow_id),
            args: BTreeMap::new(),
        }
    }

    pub fn with_arg(mut self, key: &str, value: TraceValue) -> Self {
        self.args.insert(key.to_string(), value);
        self
    }

    /// Serialize to JSON object string.
    pub fn to_json(&self) -> String {
        let mut parts = Vec::new();
        parts.push(format!("\"name\":\"{}\"", self.name));
        parts.push(format!("\"cat\":\"{}\"", self.category));
        parts.push(format!("\"ph\":\"{}\"", self.phase));
        parts.push(format!("\"ts\":{}", self.timestamp_us));
        if let Some(dur) = self.duration_us {
            parts.push(format!("\"dur\":{}", dur));
        }
        parts.push(format!("\"pid\":{}", self.pid));
        parts.push(format!("\"tid\":{}", self.tid));
        if let Some(fid) = self.flow_id {
            parts.push(format!("\"id\":{}", fid));
        }
        if !self.args.is_empty() {
            let args_str: Vec<String> = self
                .args
                .iter()
                .map(|(k, v)| format!("\"{}\":{}", k, v.to_json()))
                .collect();
            parts.push(format!("\"args\":{{{}}}", args_str.join(",")));
        }
        format!("{{{}}}", parts.join(","))
    }
}

/// Recorder that accumulates trace events during simulation.
#[derive(Debug, Clone)]
pub struct TraceRecorder {
    events: Vec<TraceEvent>,
    next_flow_id: u64,
    /// Metadata: node ID → display name.
    node_names: BTreeMap<u64, String>,
}

impl TraceRecorder {
    pub fn new() -> Self {
        Self {
            events: Vec::new(),
            next_flow_id: 1,
            node_names: BTreeMap::new(),
        }
    }

    pub fn set_node_name(&mut self, node_id: u64, name: &str) {
        self.node_names.insert(node_id, name.to_string());
    }

    /// Record a raw event.
    pub fn record(&mut self, event: TraceEvent) {
        self.events.push(event);
    }

    /// Record a message send→receive pair. Returns flow_id used.
    pub fn record_message(
        &mut self,
        name: &str,
        category: &str,
        send_time_us: u64,
        receive_time_us: u64,
        from_node: u64,
        to_node: u64,
    ) -> u64 {
        let fid = self.next_flow_id;
        self.next_flow_id += 1;

        self.events.push(
            TraceEvent::flow_start(name, category, send_time_us, from_node, fid)
                .with_arg("to", TraceValue::Number(to_node as i64)),
        );
        self.events.push(
            TraceEvent::flow_end(name, category, receive_time_us, to_node, fid)
                .with_arg("from", TraceValue::Number(from_node as i64)),
        );
        fid
    }

    /// Record block production.
    pub fn record_block(&mut self, timestamp_us: u64, producer: u64, height: u64, hash: &[u8; 32]) {
        self.events.push(
            TraceEvent::instant("BlockProduced", "block", timestamp_us, producer)
                .with_arg("height", TraceValue::Number(height as i64))
                .with_arg("hash", TraceValue::String(hex_short(hash))),
        );
    }

    /// Record a node crash.
    pub fn record_crash(&mut self, timestamp_us: u64, node_id: u64) {
        self.events.push(
            TraceEvent::instant("NodeCrash", "lifecycle", timestamp_us, node_id)
                .with_arg("state", TraceValue::String("crashed".into())),
        );
    }

    /// Record a node restart.
    pub fn record_restart(&mut self, timestamp_us: u64, node_id: u64) {
        self.events.push(
            TraceEvent::instant("NodeRestart", "lifecycle", timestamp_us, node_id)
                .with_arg("state", TraceValue::String("running".into())),
        );
    }

    /// Record a network partition start.
    pub fn record_partition(&mut self, timestamp_us: u64, group_a: &[u64], group_b: &[u64]) {
        // Record on a synthetic "network" pid=0
        self.events.push(
            TraceEvent::instant("PartitionStart", "partition", timestamp_us, 0)
                .with_arg("group_a", TraceValue::String(format!("{:?}", group_a)))
                .with_arg("group_b", TraceValue::String(format!("{:?}", group_b))),
        );
    }

    /// Record partition heal.
    pub fn record_partition_heal(&mut self, timestamp_us: u64) {
        self.events.push(TraceEvent::instant(
            "PartitionHeal",
            "partition",
            timestamp_us,
            0,
        ));
    }

    /// Record a dispute lifecycle span.
    pub fn record_dispute(
        &mut self,
        start_us: u64,
        end_us: u64,
        challenger: u64,
        job_id: u64,
        rounds: u32,
        outcome: &str,
    ) {
        self.events.push(
            TraceEvent::complete(
                "Dispute",
                "dispute",
                start_us,
                end_us - start_us,
                challenger,
            )
            .with_arg("job_id", TraceValue::Number(job_id as i64))
            .with_arg("rounds", TraceValue::Number(rounds as i64))
            .with_arg("outcome", TraceValue::String(outcome.into())),
        );
    }

    /// Total event count.
    pub fn event_count(&self) -> usize {
        self.events.len()
    }

    /// Get events filtered by category.
    pub fn events_by_category(&self, category: &str) -> Vec<&TraceEvent> {
        self.events
            .iter()
            .filter(|e| e.category == category)
            .collect()
    }

    /// Get events filtered by node (pid).
    pub fn events_by_node(&self, node_id: u64) -> Vec<&TraceEvent> {
        self.events.iter().filter(|e| e.pid == node_id).collect()
    }

    /// Compute per-node summary statistics.
    pub fn node_stats(&self) -> BTreeMap<u64, NodeStats> {
        let mut stats: BTreeMap<u64, NodeStats> = BTreeMap::new();
        for event in &self.events {
            let s = stats.entry(event.pid).or_default();
            s.event_count += 1;
            match event.name.as_str() {
                "BlockProduced" => s.blocks_produced += 1,
                "NodeCrash" => s.crashes += 1,
                "NodeRestart" => s.restarts += 1,
                "Dispute" => {
                    s.disputes += 1;
                    if let Some(TraceValue::String(o)) = event.args.get("outcome") {
                        if o == "challenger_wins" {
                            s.disputes_won += 1;
                        }
                    }
                }
                _ => {}
            }
            if event.phase == 's' {
                s.messages_sent += 1;
            }
            if event.phase == 'f' {
                s.messages_received += 1;
            }
        }
        stats
    }

    /// Serialize entire trace to Chrome Trace Event Format JSON.
    pub fn to_json(&self) -> String {
        let mut out = String::from("{\"traceEvents\":[");
        // Add metadata events for node names
        let mut first = true;
        for (nid, name) in &self.node_names {
            if !first {
                out.push(',');
            }
            first = false;
            out.push_str(&format!(
                "{{\"name\":\"process_name\",\"ph\":\"M\",\"pid\":{},\"args\":{{\"name\":\"{}\"}}}}",
                nid, name
            ));
        }
        for event in &self.events {
            if !first {
                out.push(',');
            }
            first = false;
            out.push_str(&event.to_json());
        }
        out.push_str("]}");
        out
    }

    /// Serialize to newline-delimited JSON (one event per line) for streaming.
    pub fn to_ndjson(&self) -> String {
        self.events
            .iter()
            .map(|e| e.to_json())
            .collect::<Vec<_>>()
            .join("\n")
    }

    /// Timeline summary: min/max timestamp, total events, duration.
    pub fn timeline_summary(&self) -> TimelineSummary {
        let min_ts = self
            .events
            .iter()
            .map(|e| e.timestamp_us)
            .min()
            .unwrap_or(0);
        let max_ts = self
            .events
            .iter()
            .map(|e| e.timestamp_us + e.duration_us.unwrap_or(0))
            .max()
            .unwrap_or(0);
        TimelineSummary {
            start_us: min_ts,
            end_us: max_ts,
            duration_us: max_ts.saturating_sub(min_ts),
            total_events: self.events.len(),
            categories: {
                let mut cats: BTreeMap<String, usize> = BTreeMap::new();
                for e in &self.events {
                    *cats.entry(e.category.clone()).or_default() += 1;
                }
                cats
            },
        }
    }
}

/// Per-node aggregate statistics.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct NodeStats {
    pub event_count: usize,
    pub blocks_produced: usize,
    pub messages_sent: usize,
    pub messages_received: usize,
    pub crashes: usize,
    pub restarts: usize,
    pub disputes: usize,
    pub disputes_won: usize,
}

/// Summary of the entire trace timeline.
#[derive(Debug, Clone, PartialEq)]
pub struct TimelineSummary {
    pub start_us: u64,
    pub end_us: u64,
    pub duration_us: u64,
    pub total_events: usize,
    pub categories: BTreeMap<String, usize>,
}

fn hex_short(bytes: &[u8; 32]) -> String {
    format!(
        "{:02x}{:02x}..{:02x}{:02x}",
        bytes[0], bytes[1], bytes[30], bytes[31]
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_trace_event_instant_json() {
        let e = TraceEvent::instant("BlockProduced", "block", 1000, 1);
        let json = e.to_json();
        assert!(json.contains("\"name\":\"BlockProduced\""));
        assert!(json.contains("\"ph\":\"i\""));
        assert!(json.contains("\"ts\":1000"));
        assert!(json.contains("\"pid\":1"));
    }

    #[test]
    fn test_trace_event_complete_json() {
        let e = TraceEvent::complete("Dispute", "dispute", 5000, 3000, 2);
        let json = e.to_json();
        assert!(json.contains("\"dur\":3000"));
        assert!(json.contains("\"ph\":\"X\""));
    }

    #[test]
    fn test_trace_event_with_args() {
        let e = TraceEvent::instant("Test", "test", 0, 0)
            .with_arg("height", TraceValue::Number(42))
            .with_arg("valid", TraceValue::Bool(true));
        let json = e.to_json();
        assert!(json.contains("\"height\":42"));
        assert!(json.contains("\"valid\":true"));
    }

    #[test]
    fn test_flow_events_linked() {
        let s = TraceEvent::flow_start("Msg", "message", 100, 1, 7);
        let e = TraceEvent::flow_end("Msg", "message", 200, 2, 7);
        assert_eq!(s.flow_id, Some(7));
        assert_eq!(e.flow_id, Some(7));
        assert_eq!(s.phase, 's');
        assert_eq!(e.phase, 'f');
    }

    #[test]
    fn test_recorder_record_message() {
        let mut rec = TraceRecorder::new();
        let fid = rec.record_message("BlockAnnounce", "message", 100_000, 150_000, 1, 2);
        assert_eq!(fid, 1);
        assert_eq!(rec.event_count(), 2);
        let fid2 = rec.record_message("Ping", "message", 200_000, 250_000, 2, 3);
        assert_eq!(fid2, 2);
        assert_eq!(rec.event_count(), 4);
    }

    #[test]
    fn test_recorder_block_and_crash() {
        let mut rec = TraceRecorder::new();
        rec.record_block(1000, 1, 1, &[0u8; 32]);
        rec.record_crash(2000, 1);
        rec.record_restart(3000, 1);
        assert_eq!(rec.event_count(), 3);
        let lifecycle = rec.events_by_category("lifecycle");
        assert_eq!(lifecycle.len(), 2);
    }

    #[test]
    fn test_recorder_partition_events() {
        let mut rec = TraceRecorder::new();
        rec.record_partition(1000, &[1, 2], &[3, 4]);
        rec.record_partition_heal(5000);
        let parts = rec.events_by_category("partition");
        assert_eq!(parts.len(), 2);
        assert_eq!(parts[0].name, "PartitionStart");
        assert_eq!(parts[1].name, "PartitionHeal");
    }

    #[test]
    fn test_recorder_dispute() {
        let mut rec = TraceRecorder::new();
        rec.record_dispute(1000, 5000, 2, 42, 4, "challenger_wins");
        let disputes = rec.events_by_category("dispute");
        assert_eq!(disputes.len(), 1);
        assert_eq!(disputes[0].duration_us, Some(4000));
    }

    #[test]
    fn test_events_by_node() {
        let mut rec = TraceRecorder::new();
        rec.record_block(100, 1, 1, &[0u8; 32]);
        rec.record_block(200, 2, 2, &[1u8; 32]);
        rec.record_crash(300, 1);
        assert_eq!(rec.events_by_node(1).len(), 2);
        assert_eq!(rec.events_by_node(2).len(), 1);
    }

    #[test]
    fn test_node_stats() {
        let mut rec = TraceRecorder::new();
        rec.record_block(100, 1, 1, &[0u8; 32]);
        rec.record_block(200, 1, 2, &[1u8; 32]);
        rec.record_crash(300, 1);
        rec.record_restart(400, 1);
        rec.record_message("Msg", "message", 500, 600, 1, 2);
        rec.record_dispute(700, 900, 1, 10, 3, "challenger_wins");

        let stats = rec.node_stats();
        let s1 = &stats[&1];
        assert_eq!(s1.blocks_produced, 2);
        assert_eq!(s1.crashes, 1);
        assert_eq!(s1.restarts, 1);
        assert_eq!(s1.messages_sent, 1);
        assert_eq!(s1.disputes, 1);
        assert_eq!(s1.disputes_won, 1);
    }

    #[test]
    fn test_to_json_format() {
        let mut rec = TraceRecorder::new();
        rec.set_node_name(1, "Validator-1");
        rec.record_block(1000, 1, 1, &[0xABu8; 32]);
        let json = rec.to_json();
        assert!(json.starts_with("{\"traceEvents\":["));
        assert!(json.ends_with("]}"));
        assert!(json.contains("process_name"));
        assert!(json.contains("Validator-1"));
        assert!(json.contains("BlockProduced"));
    }

    #[test]
    fn test_to_ndjson_format() {
        let mut rec = TraceRecorder::new();
        rec.record_block(100, 1, 1, &[0u8; 32]);
        rec.record_crash(200, 2);
        let ndjson = rec.to_ndjson();
        let lines: Vec<&str> = ndjson.lines().collect();
        assert_eq!(lines.len(), 2);
    }

    #[test]
    fn test_timeline_summary() {
        let mut rec = TraceRecorder::new();
        rec.record_block(1000, 1, 1, &[0u8; 32]);
        rec.record_message("Msg", "message", 2000, 3000, 1, 2);
        rec.record_dispute(4000, 8000, 2, 1, 3, "provider_wins");
        let summary = rec.timeline_summary();
        assert_eq!(summary.start_us, 1000);
        assert_eq!(summary.end_us, 8000);
        assert_eq!(summary.duration_us, 7000);
        assert_eq!(summary.total_events, 4); // 1 block + 2 flow + 1 dispute
        assert_eq!(summary.categories["block"], 1);
        assert_eq!(summary.categories["message"], 2);
        assert_eq!(summary.categories["dispute"], 1);
    }

    #[test]
    fn test_trace_value_json_escaping() {
        let v = TraceValue::String("hello \"world\"".into());
        assert_eq!(v.to_json(), "\"hello \\\"world\\\"\"");
        let v2 = TraceValue::Float(3.14159);
        assert!(v2.to_json().starts_with("3.14"));
    }

    #[test]
    fn test_empty_recorder() {
        let rec = TraceRecorder::new();
        assert_eq!(rec.event_count(), 0);
        let json = rec.to_json();
        assert_eq!(json, "{\"traceEvents\":[]}");
        let summary = rec.timeline_summary();
        assert_eq!(summary.total_events, 0);
        assert_eq!(summary.duration_us, 0);
    }

    #[test]
    fn test_hex_short() {
        let mut h = [0u8; 32];
        h[0] = 0xDE;
        h[1] = 0xAD;
        h[30] = 0xBE;
        h[31] = 0xEF;
        assert_eq!(hex_short(&h), "dead..beef");
    }
}
