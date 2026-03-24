//! Metrics & Telemetry — Prometheus-style counters, gauges, and histograms.
//!
//! Provides lightweight, lock-free metrics collection for node observability.
//! Supports:
//! - Counters (monotonically increasing)
//! - Gauges (arbitrary values, inc/dec/set)
//! - Histograms (value distribution with configurable buckets)
//! - Labels (key-value dimensions on any metric)
//! - Registry (central collection point, Prometheus text exposition format)

use std::collections::{BTreeMap, HashMap};
use std::sync::atomic::{AtomicI64, AtomicU64, Ordering};
use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant};

// ─── Counter ───────────────────────────────────────────────────────────────

/// A monotonically increasing counter (stored as u64 × 1000 for milli-precision).
#[derive(Debug)]
pub struct Counter {
    name: String,
    help: String,
    /// Keyed by sorted label set → atomic milli-value
    values: RwLock<HashMap<Vec<(String, String)>, AtomicU64>>,
}

impl Counter {
    pub fn new(name: &str, help: &str) -> Self {
        Self {
            name: name.to_string(),
            help: help.to_string(),
            values: RwLock::new(HashMap::new()),
        }
    }

    pub fn inc(&self) {
        self.inc_by(1.0, &[]);
    }

    pub fn inc_with(&self, labels: &[(&str, &str)]) {
        self.inc_by(1.0, labels);
    }

    pub fn inc_by(&self, v: f64, labels: &[(&str, &str)]) {
        assert!(v >= 0.0, "counter can only increase");
        let key = sorted_labels(labels);
        let milli = (v * 1000.0) as u64;

        // Fast path: read lock
        {
            let vals = self.values.read().unwrap();
            if let Some(atom) = vals.get(&key) {
                atom.fetch_add(milli, Ordering::Relaxed);
                return;
            }
        }
        // Slow path: write lock to insert
        let mut vals = self.values.write().unwrap();
        vals.entry(key)
            .or_insert_with(|| AtomicU64::new(0))
            .fetch_add(milli, Ordering::Relaxed);
    }

    pub fn get(&self, labels: &[(&str, &str)]) -> f64 {
        let key = sorted_labels(labels);
        let vals = self.values.read().unwrap();
        vals.get(&key)
            .map(|a| a.load(Ordering::Relaxed) as f64 / 1000.0)
            .unwrap_or(0.0)
    }

    fn collect(&self) -> Vec<MetricSample> {
        let vals = self.values.read().unwrap();
        vals.iter()
            .map(|(labels, atom)| MetricSample {
                name: self.name.clone(),
                labels: labels.clone(),
                value: atom.load(Ordering::Relaxed) as f64 / 1000.0,
            })
            .collect()
    }
}

// ─── Gauge ─────────────────────────────────────────────────────────────────

/// A gauge that can go up and down (stored as i64 × 1000).
#[derive(Debug)]
pub struct Gauge {
    name: String,
    help: String,
    values: RwLock<HashMap<Vec<(String, String)>, AtomicI64>>,
}

impl Gauge {
    pub fn new(name: &str, help: &str) -> Self {
        Self {
            name: name.to_string(),
            help: help.to_string(),
            values: RwLock::new(HashMap::new()),
        }
    }

    pub fn set(&self, v: f64) {
        self.set_with(v, &[]);
    }

    pub fn set_with(&self, v: f64, labels: &[(&str, &str)]) {
        let key = sorted_labels(labels);
        let milli = (v * 1000.0) as i64;
        let mut vals = self.values.write().unwrap();
        vals.entry(key)
            .or_insert_with(|| AtomicI64::new(0))
            .store(milli, Ordering::Relaxed);
    }

    pub fn inc(&self) {
        self.add(1.0, &[]);
    }

    pub fn dec(&self) {
        self.add(-1.0, &[]);
    }

    pub fn add(&self, v: f64, labels: &[(&str, &str)]) {
        let key = sorted_labels(labels);
        let milli = (v * 1000.0) as i64;
        {
            let vals = self.values.read().unwrap();
            if let Some(atom) = vals.get(&key) {
                atom.fetch_add(milli, Ordering::Relaxed);
                return;
            }
        }
        let mut vals = self.values.write().unwrap();
        vals.entry(key)
            .or_insert_with(|| AtomicI64::new(0))
            .fetch_add(milli, Ordering::Relaxed);
    }

    pub fn get(&self, labels: &[(&str, &str)]) -> f64 {
        let key = sorted_labels(labels);
        let vals = self.values.read().unwrap();
        vals.get(&key)
            .map(|a| a.load(Ordering::Relaxed) as f64 / 1000.0)
            .unwrap_or(0.0)
    }

    fn collect(&self) -> Vec<MetricSample> {
        let vals = self.values.read().unwrap();
        vals.iter()
            .map(|(labels, atom)| MetricSample {
                name: self.name.clone(),
                labels: labels.clone(),
                value: atom.load(Ordering::Relaxed) as f64 / 1000.0,
            })
            .collect()
    }
}

// ─── Histogram ─────────────────────────────────────────────────────────────

/// Histogram with configurable bucket boundaries.
#[derive(Debug)]
pub struct Histogram {
    name: String,
    help: String,
    buckets: Vec<f64>,
    /// Per label-set: (bucket_counts[len=buckets.len()], sum_milli, count)
    values: RwLock<HashMap<Vec<(String, String)>, HistogramData>>,
}

#[derive(Debug)]
struct HistogramData {
    bucket_counts: Vec<AtomicU64>,
    sum_milli: AtomicI64,
    count: AtomicU64,
}

impl Histogram {
    pub fn new(name: &str, help: &str, buckets: Vec<f64>) -> Self {
        assert!(!buckets.is_empty(), "histogram needs at least one bucket");
        let mut sorted = buckets;
        sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());
        Self {
            name: name.to_string(),
            help: help.to_string(),
            buckets: sorted,
            values: RwLock::new(HashMap::new()),
        }
    }

    /// Default buckets suitable for latency in seconds.
    pub fn with_default_buckets(name: &str, help: &str) -> Self {
        Self::new(
            name,
            help,
            vec![
                0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1.0, 2.5, 5.0, 10.0,
            ],
        )
    }

    pub fn observe(&self, v: f64) {
        self.observe_with(v, &[]);
    }

    pub fn observe_with(&self, v: f64, labels: &[(&str, &str)]) {
        let key = sorted_labels(labels);
        let milli = (v * 1000.0) as i64;

        // Try read lock first
        {
            let vals = self.values.read().unwrap();
            if let Some(data) = vals.get(&key) {
                record_observation(data, &self.buckets, v, milli);
                return;
            }
        }

        // Insert new entry
        let mut vals = self.values.write().unwrap();
        let data = vals.entry(key).or_insert_with(|| HistogramData {
            bucket_counts: (0..self.buckets.len()).map(|_| AtomicU64::new(0)).collect(),
            sum_milli: AtomicI64::new(0),
            count: AtomicU64::new(0),
        });
        record_observation(data, &self.buckets, v, milli);
    }

    pub fn count(&self, labels: &[(&str, &str)]) -> u64 {
        let key = sorted_labels(labels);
        let vals = self.values.read().unwrap();
        vals.get(&key)
            .map(|d| d.count.load(Ordering::Relaxed))
            .unwrap_or(0)
    }

    pub fn sum(&self, labels: &[(&str, &str)]) -> f64 {
        let key = sorted_labels(labels);
        let vals = self.values.read().unwrap();
        vals.get(&key)
            .map(|d| d.sum_milli.load(Ordering::Relaxed) as f64 / 1000.0)
            .unwrap_or(0.0)
    }

    fn collect(&self) -> Vec<MetricSample> {
        let vals = self.values.read().unwrap();
        let mut samples = Vec::new();
        for (labels, data) in vals.iter() {
            let count = data.count.load(Ordering::Relaxed);
            let sum = data.sum_milli.load(Ordering::Relaxed) as f64 / 1000.0;

            // Bucket samples
            let mut cumulative = 0u64;
            for (i, bound) in self.buckets.iter().enumerate() {
                cumulative += data.bucket_counts[i].load(Ordering::Relaxed);
                let mut bucket_labels = labels.clone();
                bucket_labels.push(("le".to_string(), format!("{}", bound)));
                samples.push(MetricSample {
                    name: format!("{}_bucket", self.name),
                    labels: bucket_labels,
                    value: cumulative as f64,
                });
            }
            // +Inf bucket
            let mut inf_labels = labels.clone();
            inf_labels.push(("le".to_string(), "+Inf".to_string()));
            samples.push(MetricSample {
                name: format!("{}_bucket", self.name),
                labels: inf_labels,
                value: count as f64,
            });

            samples.push(MetricSample {
                name: format!("{}_sum", self.name),
                labels: labels.clone(),
                value: sum,
            });
            samples.push(MetricSample {
                name: format!("{}_count", self.name),
                labels: labels.clone(),
                value: count as f64,
            });
        }
        samples
    }
}

fn record_observation(data: &HistogramData, buckets: &[f64], v: f64, milli: i64) {
    // Find the first bucket where v <= bound and increment only that one.
    // collect() will compute cumulative sums.
    for (i, bound) in buckets.iter().enumerate() {
        if v <= *bound {
            data.bucket_counts[i].fetch_add(1, Ordering::Relaxed);
            break;
        }
    }
    // If v > all buckets, no bucket_count is incremented (only +Inf in collect)
    data.sum_milli.fetch_add(milli, Ordering::Relaxed);
    data.count.fetch_add(1, Ordering::Relaxed);
}

// ─── Timer ─────────────────────────────────────────────────────────────────

/// RAII timer that observes duration on drop.
pub struct Timer<'a> {
    histogram: &'a Histogram,
    labels: Vec<(&'a str, &'a str)>,
    start: Instant,
}

impl<'a> Timer<'a> {
    pub fn new(histogram: &'a Histogram, labels: &[(&'a str, &'a str)]) -> Self {
        Self {
            histogram,
            labels: labels.to_vec(),
            start: Instant::now(),
        }
    }

    /// Stop and observe manually (also consumed by drop).
    pub fn observe(self) -> Duration {
        let elapsed = self.start.elapsed();
        self.histogram
            .observe_with(elapsed.as_secs_f64(), &self.labels);
        std::mem::forget(self); // prevent double-observe in drop
        elapsed
    }
}

impl<'a> Drop for Timer<'a> {
    fn drop(&mut self) {
        let elapsed = self.start.elapsed();
        self.histogram
            .observe_with(elapsed.as_secs_f64(), &self.labels);
    }
}

// ─── Registry ──────────────────────────────────────────────────────────────

#[derive(Debug)]
struct MetricSample {
    name: String,
    labels: Vec<(String, String)>,
    value: f64,
}

/// Central metric registry — collects all metrics and exposes them.
#[derive(Debug, Default)]
pub struct Registry {
    counters: RwLock<Vec<Arc<Counter>>>,
    gauges: RwLock<Vec<Arc<Gauge>>>,
    histograms: RwLock<Vec<Arc<Histogram>>>,
}

impl Registry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register_counter(&self, c: Arc<Counter>) {
        self.counters.write().unwrap().push(c);
    }

    pub fn register_gauge(&self, g: Arc<Gauge>) {
        self.gauges.write().unwrap().push(g);
    }

    pub fn register_histogram(&self, h: Arc<Histogram>) {
        self.histograms.write().unwrap().push(h);
    }

    /// Render all metrics in Prometheus text exposition format.
    pub fn render(&self) -> String {
        let mut out = String::new();
        let mut seen_help: std::collections::HashSet<String> = std::collections::HashSet::new();

        // Counters
        for c in self.counters.read().unwrap().iter() {
            if seen_help.insert(c.name.clone()) {
                out.push_str(&format!("# HELP {} {}\n", c.name, c.help));
                out.push_str(&format!("# TYPE {} counter\n", c.name));
            }
            for s in c.collect() {
                write_sample(&mut out, &s);
            }
        }

        // Gauges
        for g in self.gauges.read().unwrap().iter() {
            if seen_help.insert(g.name.clone()) {
                out.push_str(&format!("# HELP {} {}\n", g.name, g.help));
                out.push_str(&format!("# TYPE {} gauge\n", g.name));
            }
            for s in g.collect() {
                write_sample(&mut out, &s);
            }
        }

        // Histograms
        for h in self.histograms.read().unwrap().iter() {
            if seen_help.insert(h.name.clone()) {
                out.push_str(&format!("# HELP {} {}\n", h.name, h.help));
                out.push_str(&format!("# TYPE {} histogram\n", h.name));
            }
            for s in h.collect() {
                write_sample(&mut out, &s);
            }
        }

        out
    }
}

fn write_sample(out: &mut String, s: &MetricSample) {
    if s.labels.is_empty() {
        out.push_str(&format!("{} {}\n", s.name, format_value(s.value)));
    } else {
        let label_str: Vec<String> = s
            .labels
            .iter()
            .map(|(k, v)| format!("{}=\"{}\"", k, v))
            .collect();
        out.push_str(&format!(
            "{}{{{}}} {}\n",
            s.name,
            label_str.join(","),
            format_value(s.value)
        ));
    }
}

fn format_value(v: f64) -> String {
    if v == v.floor() && v.abs() < 1e15 {
        format!("{}", v as i64)
    } else {
        format!("{:.3}", v)
    }
}

// ─── Node Metrics (pre-defined) ────────────────────────────────────────────

/// Pre-defined metrics for the Prova node.
pub struct NodeMetrics {
    pub jobs_completed: Arc<Counter>,
    pub jobs_failed: Arc<Counter>,
    pub inference_duration: Arc<Histogram>,
    pub active_jobs: Arc<Gauge>,
    pub blocks_processed: Arc<Counter>,
    pub peers_connected: Arc<Gauge>,
    pub chain_height: Arc<Gauge>,
    pub mempool_size: Arc<Gauge>,
    pub disputes_opened: Arc<Counter>,
    pub disputes_won: Arc<Counter>,
    pub pdp_proofs_submitted: Arc<Counter>,
    pub registry: Arc<Registry>,
}

impl NodeMetrics {
    pub fn new() -> Self {
        let registry = Arc::new(Registry::new());

        let jobs_completed = Arc::new(Counter::new(
            "prova_jobs_completed_total",
            "Total inference jobs completed successfully",
        ));
        let jobs_failed = Arc::new(Counter::new(
            "prova_jobs_failed_total",
            "Total inference jobs failed",
        ));
        let inference_duration = Arc::new(Histogram::new(
            "prova_inference_duration_seconds",
            "Inference job duration in seconds",
            vec![0.1, 0.5, 1.0, 2.5, 5.0, 10.0, 30.0, 60.0, 120.0],
        ));
        let active_jobs = Arc::new(Gauge::new(
            "prova_active_jobs",
            "Currently running inference jobs",
        ));
        let blocks_processed = Arc::new(Counter::new(
            "prova_blocks_processed_total",
            "Total blocks processed",
        ));
        let peers_connected = Arc::new(Gauge::new(
            "prova_peers_connected",
            "Number of connected peers",
        ));
        let chain_height = Arc::new(Gauge::new("prova_chain_height", "Current chain height"));
        let mempool_size = Arc::new(Gauge::new("prova_mempool_size", "Transactions in mempool"));
        let disputes_opened = Arc::new(Counter::new(
            "prova_disputes_opened_total",
            "Total disputes opened",
        ));
        let disputes_won = Arc::new(Counter::new(
            "prova_disputes_won_total",
            "Total disputes won",
        ));
        let pdp_proofs_submitted = Arc::new(Counter::new(
            "prova_pdp_proofs_submitted_total",
            "Total PDP proofs submitted",
        ));

        // Register all
        registry.register_counter(Arc::clone(&jobs_completed));
        registry.register_counter(Arc::clone(&jobs_failed));
        registry.register_histogram(Arc::clone(&inference_duration));
        registry.register_gauge(Arc::clone(&active_jobs));
        registry.register_counter(Arc::clone(&blocks_processed));
        registry.register_gauge(Arc::clone(&peers_connected));
        registry.register_gauge(Arc::clone(&chain_height));
        registry.register_gauge(Arc::clone(&mempool_size));
        registry.register_counter(Arc::clone(&disputes_opened));
        registry.register_counter(Arc::clone(&disputes_won));
        registry.register_counter(Arc::clone(&pdp_proofs_submitted));

        Self {
            jobs_completed,
            jobs_failed,
            inference_duration,
            active_jobs,
            blocks_processed,
            peers_connected,
            chain_height,
            mempool_size,
            disputes_opened,
            disputes_won,
            pdp_proofs_submitted,
            registry,
        }
    }

    /// Time an inference job — returns a Timer that auto-observes on drop.
    pub fn time_inference<'a>(&'a self, model: &'a str) -> Timer<'a> {
        Timer::new(&self.inference_duration, &[("model", model)])
    }
}

impl Default for NodeMetrics {
    fn default() -> Self {
        Self::new()
    }
}

// ─── Helpers ───────────────────────────────────────────────────────────────

fn sorted_labels(labels: &[(&str, &str)]) -> Vec<(String, String)> {
    let mut v: Vec<(String, String)> = labels
        .iter()
        .map(|(k, v)| (k.to_string(), v.to_string()))
        .collect();
    v.sort();
    v
}

// ─── Tests ─────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_counter_basic() {
        let c = Counter::new("test_total", "test");
        c.inc();
        c.inc();
        c.inc_by(2.5, &[]);
        assert_eq!(c.get(&[]), 4.5);
    }

    #[test]
    fn test_counter_with_labels() {
        let c = Counter::new("req_total", "requests");
        c.inc_with(&[("method", "GET")]);
        c.inc_with(&[("method", "GET")]);
        c.inc_with(&[("method", "POST")]);
        assert_eq!(c.get(&[("method", "GET")]), 2.0);
        assert_eq!(c.get(&[("method", "POST")]), 1.0);
        assert_eq!(c.get(&[("method", "DELETE")]), 0.0);
    }

    #[test]
    fn test_gauge_set_inc_dec() {
        let g = Gauge::new("temperature", "temp");
        g.set(20.0);
        assert_eq!(g.get(&[]), 20.0);
        g.inc();
        assert_eq!(g.get(&[]), 21.0);
        g.dec();
        assert_eq!(g.get(&[]), 20.0);
        g.add(-5.0, &[]);
        assert_eq!(g.get(&[]), 15.0);
    }

    #[test]
    fn test_gauge_with_labels() {
        let g = Gauge::new("pool_size", "pool");
        g.set_with(10.0, &[("pool", "worker")]);
        g.set_with(3.0, &[("pool", "io")]);
        assert_eq!(g.get(&[("pool", "worker")]), 10.0);
        assert_eq!(g.get(&[("pool", "io")]), 3.0);
    }

    #[test]
    fn test_histogram_observe() {
        let h = Histogram::new("duration", "dur", vec![0.1, 0.5, 1.0, 5.0]);
        h.observe(0.05);
        h.observe(0.3);
        h.observe(0.8);
        h.observe(3.0);
        h.observe(10.0);

        assert_eq!(h.count(&[]), 5);
        // sum = 0.05 + 0.3 + 0.8 + 3.0 + 10.0 = 14.15
        let sum = h.sum(&[]);
        assert!((sum - 14.15).abs() < 0.01, "sum was {}", sum);
    }

    #[test]
    fn test_histogram_bucket_distribution() {
        let h = Histogram::new("lat", "latency", vec![1.0, 5.0, 10.0]);
        h.observe(0.5); // bucket 1.0
        h.observe(3.0); // bucket 5.0
        h.observe(7.0); // bucket 10.0
        h.observe(15.0); // +Inf only

        let samples = h.collect();
        // Find bucket samples
        let bucket_samples: Vec<_> = samples.iter().filter(|s| s.name == "lat_bucket").collect();

        // le=1.0 → cumulative 1 (0.5)
        let le1 = bucket_samples
            .iter()
            .find(|s| s.labels.iter().any(|(k, v)| k == "le" && v == "1"))
            .unwrap();
        assert_eq!(le1.value, 1.0);

        // le=5.0 → cumulative 2 (0.5 + 3.0)
        let le5 = bucket_samples
            .iter()
            .find(|s| s.labels.iter().any(|(k, v)| k == "le" && v == "5"))
            .unwrap();
        assert_eq!(le5.value, 2.0);

        // le=+Inf → 4 (all)
        let le_inf = bucket_samples
            .iter()
            .find(|s| s.labels.iter().any(|(k, v)| k == "le" && v == "+Inf"))
            .unwrap();
        assert_eq!(le_inf.value, 4.0);
    }

    #[test]
    fn test_histogram_with_labels() {
        let h = Histogram::new("req_dur", "request duration", vec![0.1, 1.0]);
        h.observe_with(0.05, &[("endpoint", "/health")]);
        h.observe_with(0.5, &[("endpoint", "/infer")]);
        assert_eq!(h.count(&[("endpoint", "/health")]), 1);
        assert_eq!(h.count(&[("endpoint", "/infer")]), 1);
        assert_eq!(h.count(&[("endpoint", "/other")]), 0);
    }

    #[test]
    fn test_timer_observes_on_drop() {
        let h = Histogram::new("timer_test", "test", vec![0.001, 0.01, 0.1, 1.0]);
        {
            let _t = Timer::new(&h, &[]);
            std::thread::sleep(Duration::from_millis(5));
        }
        assert_eq!(h.count(&[]), 1);
        assert!(h.sum(&[]) > 0.0);
    }

    #[test]
    fn test_timer_manual_observe() {
        let h = Histogram::new("timer_manual", "test", vec![0.001, 0.01, 0.1]);
        let t = Timer::new(&h, &[("op", "test")]);
        std::thread::sleep(Duration::from_millis(2));
        let dur = t.observe();
        assert!(dur.as_millis() >= 1);
        assert_eq!(h.count(&[("op", "test")]), 1);
    }

    #[test]
    fn test_registry_render_prometheus() {
        let reg = Registry::new();
        let c = Arc::new(Counter::new("http_requests_total", "Total HTTP requests"));
        c.inc_with(&[("method", "GET"), ("status", "200")]);
        c.inc_with(&[("method", "GET"), ("status", "200")]);
        c.inc_with(&[("method", "POST"), ("status", "500")]);
        reg.register_counter(c);

        let g = Arc::new(Gauge::new("temperature_celsius", "Current temperature"));
        g.set(23.5);
        reg.register_gauge(g);

        let output = reg.render();
        assert!(output.contains("# HELP http_requests_total"));
        assert!(output.contains("# TYPE http_requests_total counter"));
        assert!(output.contains("http_requests_total{"));
        assert!(output.contains("# TYPE temperature_celsius gauge"));
        assert!(output.contains("temperature_celsius 23"));
    }

    #[test]
    fn test_node_metrics_integration() {
        let m = NodeMetrics::new();

        // Simulate some activity
        m.jobs_completed.inc_with(&[("model", "llama-7b")]);
        m.jobs_completed.inc_with(&[("model", "llama-7b")]);
        m.jobs_failed.inc_with(&[("model", "llama-7b")]);
        m.active_jobs.set(3.0);
        m.blocks_processed.inc();
        m.chain_height.set(1000.0);
        m.peers_connected.set(12.0);
        m.mempool_size.set(45.0);
        m.disputes_opened.inc();
        m.disputes_won.inc();
        m.pdp_proofs_submitted.inc();

        // Time an inference
        {
            let _timer = m.time_inference("llama-7b");
            std::thread::sleep(Duration::from_millis(2));
        }

        let output = m.registry.render();
        assert!(output.contains("prova_jobs_completed_total"));
        assert!(output.contains("prova_inference_duration_seconds"));
        assert!(output.contains("prova_chain_height"));
        assert!(output.contains("prova_peers_connected"));

        assert_eq!(m.jobs_completed.get(&[("model", "llama-7b")]), 2.0);
        assert_eq!(m.active_jobs.get(&[]), 3.0);
        assert_eq!(m.chain_height.get(&[]), 1000.0);
    }

    #[test]
    fn test_default_histogram_buckets() {
        let h = Histogram::with_default_buckets("default_hist", "test");
        h.observe(0.001);
        h.observe(0.5);
        h.observe(5.0);
        assert_eq!(h.count(&[]), 3);
    }

    #[test]
    fn test_counter_cannot_decrease() {
        let c = Counter::new("test", "test");
        let result = std::panic::catch_unwind(|| {
            c.inc_by(-1.0, &[]);
        });
        assert!(result.is_err());
    }

    #[test]
    fn test_empty_registry_render() {
        let reg = Registry::new();
        assert_eq!(reg.render(), "");
    }

    #[test]
    fn test_label_sorting() {
        let c = Counter::new("test", "test");
        // Same labels in different order should be the same metric
        c.inc_with(&[("b", "2"), ("a", "1")]);
        c.inc_with(&[("a", "1"), ("b", "2")]);
        assert_eq!(c.get(&[("a", "1"), ("b", "2")]), 2.0);
    }
}
