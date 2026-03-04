// chain/src/bench_history.rs — Historical benchmark tracking
//
// BENCH-003: Store benchmark results per commit, track trends, detect
// regressions over time. Results stored as JSON lines for easy parsing.

use crate::benchmark::{BenchResult, BenchSuite};
use std::collections::HashMap;
use std::time::Duration;

/// A single benchmark run record, tied to a commit.
#[derive(Debug, Clone)]
pub struct BenchRecord {
    pub commit: String,
    pub timestamp: u64,
    pub results: Vec<BenchResultRecord>,
}

/// Serializable benchmark result (subset of BenchResult).
#[derive(Debug, Clone)]
pub struct BenchResultRecord {
    pub name: String,
    pub iterations: u64,
    pub mean_ns: u64,
    pub p50_ns: u64,
    pub p95_ns: u64,
    pub p99_ns: u64,
    pub min_ns: u64,
    pub max_ns: u64,
    pub throughput_ops_sec: f64,
}

impl BenchResultRecord {
    pub fn from_bench_result(r: &BenchResult) -> Self {
        Self {
            name: r.name.clone(),
            iterations: r.iterations,
            mean_ns: r.mean_ns,
            p50_ns: r.p50_ns,
            p95_ns: r.p95_ns,
            p99_ns: r.p99_ns,
            min_ns: r.min_ns,
            max_ns: r.max_ns,
            throughput_ops_sec: r.throughput_ops_sec,
        }
    }
}

/// Historical benchmark store — append-only record of benchmark runs.
pub struct BenchHistory {
    records: Vec<BenchRecord>,
}

impl BenchHistory {
    pub fn new() -> Self {
        Self { records: Vec::new() }
    }

    /// Add a benchmark run from a suite.
    pub fn record(&mut self, commit: &str, timestamp: u64, suite: &BenchSuite) {
        let results = suite
            .results()
            .iter()
            .map(BenchResultRecord::from_bench_result)
            .collect();
        self.records.push(BenchRecord {
            commit: commit.to_string(),
            timestamp,
            results,
        });
    }

    pub fn records(&self) -> &[BenchRecord] {
        &self.records
    }

    pub fn len(&self) -> usize {
        self.records.len()
    }

    pub fn is_empty(&self) -> bool {
        self.records.is_empty()
    }

    /// Get the last N records.
    pub fn last_n(&self, n: usize) -> &[BenchRecord] {
        let start = self.records.len().saturating_sub(n);
        &self.records[start..]
    }

    /// Get all results for a specific benchmark name across history.
    pub fn trend(&self, bench_name: &str) -> Vec<(String, u64, u64)> {
        // Returns (commit, timestamp, mean_ns)
        let mut points = Vec::new();
        for rec in &self.records {
            for r in &rec.results {
                if r.name == bench_name {
                    points.push((rec.commit.clone(), rec.timestamp, r.mean_ns));
                }
            }
        }
        points
    }

    /// Compute moving average for a benchmark over last N records (excluding the latest).
    pub fn moving_average(&self, bench_name: &str, window: usize) -> Option<f64> {
        let trend = self.trend(bench_name);
        if trend.len() < window + 1 {
            return None;
        }
        // Exclude latest record (that's what we're comparing against)
        let prior = &trend[..trend.len() - 1];
        let last_n: Vec<_> = prior.iter().rev().take(window).collect();
        let sum: f64 = last_n.iter().map(|(_, _, ns)| *ns as f64).sum();
        Some(sum / window as f64)
    }

    /// Detect if latest result is a regression vs moving average.
    /// Returns (bench_name, latest_ns, avg_ns, pct_change) for regressions.
    pub fn detect_regressions(
        &self,
        threshold_pct: f64,
        window: usize,
    ) -> Vec<(String, u64, f64, f64)> {
        let mut regressions = Vec::new();
        if self.records.is_empty() {
            return regressions;
        }
        let latest = self.records.last().unwrap();
        for r in &latest.results {
            if let Some(avg) = self.moving_average(&r.name, window) {
                let pct = (r.mean_ns as f64 - avg) / avg * 100.0;
                if pct > threshold_pct {
                    regressions.push((r.name.clone(), r.mean_ns, avg, pct));
                }
            }
        }
        regressions
    }

    /// Generate a trend report (markdown).
    pub fn trend_report(&self, bench_name: &str) -> String {
        let trend = self.trend(bench_name);
        let mut out = format!("# Trend: {}\n\n", bench_name);
        out.push_str("| Commit | Timestamp | Mean (ns) |\n");
        out.push_str("|--------|-----------|----------|\n");
        for (commit, ts, ns) in &trend {
            let short = if commit.len() > 8 { &commit[..8] } else { commit };
            out.push_str(&format!("| {} | {} | {} |\n", short, ts, ns));
        }
        out
    }

    /// Serialize to JSON lines format.
    pub fn to_jsonl(&self) -> String {
        let mut out = String::new();
        for rec in &self.records {
            // Simple JSON serialization without serde_json
            out.push_str(&format!(
                "{{\"commit\":\"{}\",\"timestamp\":{},\"results\":[",
                rec.commit, rec.timestamp
            ));
            for (i, r) in rec.results.iter().enumerate() {
                if i > 0 {
                    out.push(',');
                }
                out.push_str(&format!(
                    "{{\"name\":\"{}\",\"mean_ns\":{},\"p50_ns\":{},\"p95_ns\":{},\"p99_ns\":{},\"throughput\":{:.0}}}",
                    r.name, r.mean_ns, r.p50_ns, r.p95_ns, r.p99_ns, r.throughput_ops_sec
                ));
            }
            out.push_str("]}\n");
        }
        out
    }

    /// Parse from JSON lines (simple parser for our known format).
    pub fn from_jsonl(data: &str) -> Self {
        let mut records = Vec::new();
        for line in data.lines() {
            let line = line.trim();
            if line.is_empty() {
                continue;
            }
            if let Some(rec) = Self::parse_record(line) {
                records.push(rec);
            }
        }
        Self { records }
    }

    fn parse_record(line: &str) -> Option<BenchRecord> {
        // Minimal JSON parser for our known format
        let commit = Self::extract_str(line, "commit")?;
        let timestamp = Self::extract_u64(line, "timestamp")?;
        // Parse results array
        let arr_start = line.find("\"results\":[")?;
        let arr_content = &line[arr_start + 11..];
        let arr_end = arr_content.rfind(']')?;
        let arr_str = &arr_content[..arr_end];

        let mut results = Vec::new();
        // Split on },{ pattern
        if !arr_str.trim().is_empty() {
            let items: Vec<&str> = arr_str.split("},{").collect();
            for (i, item) in items.iter().enumerate() {
                let mut s = item.to_string();
                if !s.starts_with('{') {
                    s = format!("{{{}", s);
                }
                if !s.ends_with('}') {
                    s = format!("{}}}", s);
                }
                if let Some(r) = Self::parse_result_record(&s) {
                    results.push(r);
                }
            }
        }

        Some(BenchRecord {
            commit,
            timestamp,
            results,
        })
    }

    fn parse_result_record(s: &str) -> Option<BenchResultRecord> {
        let name = Self::extract_str(s, "name")?;
        let mean_ns = Self::extract_u64(s, "mean_ns")?;
        let p50_ns = Self::extract_u64(s, "p50_ns")?;
        let p95_ns = Self::extract_u64(s, "p95_ns")?;
        let p99_ns = Self::extract_u64(s, "p99_ns")?;
        let throughput = Self::extract_f64(s, "throughput")?;

        Some(BenchResultRecord {
            name,
            iterations: 0,
            mean_ns,
            p50_ns,
            p95_ns,
            p99_ns,
            min_ns: 0,
            max_ns: 0,
            throughput_ops_sec: throughput,
        })
    }

    fn extract_str(s: &str, key: &str) -> Option<String> {
        let pattern = format!("\"{}\":\"", key);
        let start = s.find(&pattern)? + pattern.len();
        let end = s[start..].find('"')? + start;
        Some(s[start..end].to_string())
    }

    fn extract_u64(s: &str, key: &str) -> Option<u64> {
        let pattern = format!("\"{}\":", key);
        let start = s.find(&pattern)? + pattern.len();
        let rest = s[start..].trim_start();
        let end = rest.find(|c: char| !c.is_ascii_digit()).unwrap_or(rest.len());
        rest[..end].parse().ok()
    }

    fn extract_f64(s: &str, key: &str) -> Option<f64> {
        let pattern = format!("\"{}\":", key);
        let start = s.find(&pattern)? + pattern.len();
        let rest = s[start..].trim_start();
        let end = rest
            .find(|c: char| !c.is_ascii_digit() && c != '.' && c != '-')
            .unwrap_or(rest.len());
        rest[..end].parse().ok()
    }
}

/// CI regression gate — runs benchmarks, compares against baselines, exits non-zero on regression.
pub struct CIBenchGate {
    /// Maximum allowed mean_ns per benchmark. If exceeded, gate fails.
    baselines: HashMap<String, u64>,
    /// Percentage threshold above moving average to flag regression.
    regression_threshold_pct: f64,
    /// Moving average window size.
    window: usize,
}

impl CIBenchGate {
    pub fn new() -> Self {
        Self {
            baselines: HashMap::new(),
            regression_threshold_pct: 20.0, // 20% above average = regression
            window: 5,
        }
    }

    pub fn with_threshold(mut self, pct: f64) -> Self {
        self.regression_threshold_pct = pct;
        self
    }

    pub fn with_window(mut self, w: usize) -> Self {
        self.window = w;
        self
    }

    /// Set absolute baseline for a benchmark.
    pub fn set_baseline(&mut self, name: &str, max_mean_ns: u64) {
        self.baselines.insert(name.to_string(), max_mean_ns);
    }

    /// Run the gate check against suite results and optional history.
    /// Returns Ok(report) or Err(failure_report).
    pub fn check(
        &self,
        suite: &BenchSuite,
        history: Option<&BenchHistory>,
    ) -> Result<String, String> {
        let mut failures = Vec::new();
        let mut report = String::from("## CI Benchmark Gate\n\n");

        // Check absolute baselines
        for r in suite.results() {
            if let Some(max_ns) = self.baselines.get(&r.name) {
                if r.mean_ns > *max_ns {
                    let msg = format!(
                        "❌ {}: mean={}ns exceeds baseline={}ns (+{:.1}%)",
                        r.name,
                        r.mean_ns,
                        max_ns,
                        (r.mean_ns as f64 - *max_ns as f64) / *max_ns as f64 * 100.0
                    );
                    failures.push(msg.clone());
                    report.push_str(&format!("{}\n", msg));
                } else {
                    report.push_str(&format!(
                        "✅ {}: mean={}ns within baseline={}ns\n",
                        r.name, r.mean_ns, max_ns
                    ));
                }
            }
        }

        // Check historical regressions
        if let Some(hist) = history {
            report.push_str("\n### Historical Comparison\n\n");
            for r in suite.results() {
                if let Some(avg) = hist.moving_average(&r.name, self.window) {
                    let pct = (r.mean_ns as f64 - avg) / avg * 100.0;
                    if pct > self.regression_threshold_pct {
                        let msg = format!(
                            "📉 {}: mean={}ns vs avg={:.0}ns (+{:.1}% > {:.0}% threshold)",
                            r.name, r.mean_ns, avg, pct, self.regression_threshold_pct
                        );
                        failures.push(msg.clone());
                        report.push_str(&format!("{}\n", msg));
                    } else {
                        report.push_str(&format!(
                            "✅ {}: mean={}ns vs avg={:.0}ns ({:+.1}%)\n",
                            r.name, r.mean_ns, avg, pct
                        ));
                    }
                }
            }
        }

        if failures.is_empty() {
            report.push_str("\n**All benchmarks passed.** ✅\n");
            Ok(report)
        } else {
            report.push_str(&format!(
                "\n**{} regression(s) detected.** ❌\n",
                failures.len()
            ));
            Err(report)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::benchmark::{BenchResult, BenchSuite};
    use std::time::Duration;

    fn make_result(name: &str, mean_ns: u64) -> BenchResult {
        BenchResult {
            name: name.to_string(),
            iterations: 1000,
            total_duration: Duration::from_nanos(mean_ns * 1000),
            min_ns: mean_ns / 2,
            max_ns: mean_ns * 2,
            mean_ns,
            p50_ns: mean_ns,
            p95_ns: mean_ns + 100,
            p99_ns: mean_ns + 200,
            throughput_ops_sec: 1_000_000_000.0 / mean_ns as f64,
        }
    }

    fn make_suite(results: Vec<(&str, u64)>) -> BenchSuite {
        let mut suite = BenchSuite::new();
        for (name, mean) in results {
            suite.add(make_result(name, mean));
        }
        suite
    }

    #[test]
    fn test_history_record_and_retrieve() {
        let mut hist = BenchHistory::new();
        let suite = make_suite(vec![("bench_a", 500), ("bench_b", 1000)]);
        hist.record("abc123", 1000, &suite);
        assert_eq!(hist.len(), 1);
        assert_eq!(hist.records()[0].commit, "abc123");
        assert_eq!(hist.records()[0].results.len(), 2);
    }

    #[test]
    fn test_history_trend() {
        let mut hist = BenchHistory::new();
        for i in 0..5 {
            let suite = make_suite(vec![("bench_a", 500 + i * 10)]);
            hist.record(&format!("commit_{}", i), 1000 + i, &suite);
        }
        let trend = hist.trend("bench_a");
        assert_eq!(trend.len(), 5);
        assert_eq!(trend[0].2, 500);
        assert_eq!(trend[4].2, 540);
    }

    #[test]
    fn test_moving_average() {
        let mut hist = BenchHistory::new();
        for i in 0..6 {
            let suite = make_suite(vec![("bench_a", 1000)]);
            hist.record(&format!("c{}", i), i, &suite);
        }
        let avg = hist.moving_average("bench_a", 5).unwrap();
        assert_eq!(avg, 1000.0);
    }

    #[test]
    fn test_moving_average_insufficient_data() {
        let mut hist = BenchHistory::new();
        for i in 0..3 {
            let suite = make_suite(vec![("bench_a", 1000)]);
            hist.record(&format!("c{}", i), i, &suite);
        }
        assert!(hist.moving_average("bench_a", 5).is_none());
    }

    #[test]
    fn test_detect_regressions() {
        let mut hist = BenchHistory::new();
        // 5 records at 1000ns
        for i in 0..5 {
            let suite = make_suite(vec![("bench_a", 1000)]);
            hist.record(&format!("c{}", i), i, &suite);
        }
        // 6th record spikes to 1500ns (50% increase)
        let suite = make_suite(vec![("bench_a", 1500)]);
        hist.record("c5", 5, &suite);

        let regs = hist.detect_regressions(20.0, 5);
        assert_eq!(regs.len(), 1);
        assert_eq!(regs[0].0, "bench_a");
        assert!(regs[0].3 > 40.0); // ~50% regression
    }

    #[test]
    fn test_detect_no_regression() {
        let mut hist = BenchHistory::new();
        for i in 0..6 {
            let suite = make_suite(vec![("bench_a", 1000 + i * 5)]);
            hist.record(&format!("c{}", i), i, &suite);
        }
        let regs = hist.detect_regressions(20.0, 5);
        assert!(regs.is_empty());
    }

    #[test]
    fn test_jsonl_roundtrip() {
        let mut hist = BenchHistory::new();
        let suite = make_suite(vec![("bench_a", 500), ("bench_b", 1000)]);
        hist.record("abc123", 42, &suite);
        hist.record("def456", 43, &suite);

        let jsonl = hist.to_jsonl();
        let parsed = BenchHistory::from_jsonl(&jsonl);
        assert_eq!(parsed.len(), 2);
        assert_eq!(parsed.records()[0].commit, "abc123");
        assert_eq!(parsed.records()[1].commit, "def456");
        assert_eq!(parsed.records()[0].results[0].mean_ns, 500);
    }

    #[test]
    fn test_trend_report() {
        let mut hist = BenchHistory::new();
        let suite = make_suite(vec![("bench_a", 500)]);
        hist.record("abc12345", 1000, &suite);
        let report = hist.trend_report("bench_a");
        assert!(report.contains("abc12345"));
        assert!(report.contains("500"));
    }

    #[test]
    fn test_last_n() {
        let mut hist = BenchHistory::new();
        for i in 0..10 {
            let suite = make_suite(vec![("b", i * 100)]);
            hist.record(&format!("c{}", i), i, &suite);
        }
        let last3 = hist.last_n(3);
        assert_eq!(last3.len(), 3);
        assert_eq!(last3[0].commit, "c7");
    }

    #[test]
    fn test_ci_gate_pass() {
        let suite = make_suite(vec![("bench_a", 500)]);
        let mut gate = CIBenchGate::new();
        gate.set_baseline("bench_a", 1000);
        assert!(gate.check(&suite, None).is_ok());
    }

    #[test]
    fn test_ci_gate_fail_absolute() {
        let suite = make_suite(vec![("bench_a", 1500)]);
        let mut gate = CIBenchGate::new();
        gate.set_baseline("bench_a", 1000);
        let result = gate.check(&suite, None);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("exceeds baseline"));
    }

    #[test]
    fn test_ci_gate_fail_historical() {
        let mut hist = BenchHistory::new();
        for i in 0..6 {
            let suite = make_suite(vec![("bench_a", 1000)]);
            hist.record(&format!("c{}", i), i, &suite);
        }
        // Record the "current" run into history so moving_average can exclude it
        let spike_suite = make_suite(vec![("bench_a", 2000)]);
        hist.record("c6", 6, &spike_suite);
        // Check using the spike suite against history
        let gate = CIBenchGate::new().with_threshold(20.0).with_window(5);
        let result = gate.check(&spike_suite, Some(&hist));
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("threshold"));
    }

    #[test]
    fn test_ci_gate_pass_with_history() {
        let mut hist = BenchHistory::new();
        for i in 0..6 {
            let suite = make_suite(vec![("bench_a", 1000)]);
            hist.record(&format!("c{}", i), i, &suite);
        }
        // Current run is within threshold
        let suite = make_suite(vec![("bench_a", 1100)]);
        hist.record("c6", 6, &suite);
        let gate = CIBenchGate::new().with_threshold(20.0).with_window(5);
        assert!(gate.check(&suite, Some(&hist)).is_ok());
    }

    #[test]
    fn test_ci_gate_custom_threshold() {
        let mut hist = BenchHistory::new();
        for i in 0..6 {
            let suite = make_suite(vec![("bench_a", 1000)]);
            hist.record(&format!("c{}", i), i, &suite);
        }
        let suite = make_suite(vec![("bench_a", 1060)]);
        hist.record("c6", 6, &suite);
        // 5% threshold — 6% increase should fail
        let gate = CIBenchGate::new().with_threshold(5.0).with_window(5);
        assert!(gate.check(&suite, Some(&hist)).is_err());
    }

    #[test]
    fn test_empty_history() {
        let hist = BenchHistory::new();
        assert!(hist.is_empty());
        assert_eq!(hist.len(), 0);
        assert!(hist.trend("anything").is_empty());
        assert!(hist.moving_average("anything", 5).is_none());
        assert!(hist.detect_regressions(20.0, 5).is_empty());
    }

    #[test]
    fn test_from_empty_jsonl() {
        let hist = BenchHistory::from_jsonl("");
        assert!(hist.is_empty());
    }
}
