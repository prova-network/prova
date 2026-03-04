// chain/src/benchmark.rs — Performance benchmarking harness
//
// BENCH-001: Measures throughput and latency of critical chain subsystems:
// - State trie operations (credit/debit/transfer)
// - Mempool insert + retrieval
// - Scheduler job submission + assignment
// - Fee market gas pricing
// - Reward distribution
// - Genesis state construction
//
// Outputs structured BenchResult records for regression tracking.

use std::time::{Duration, Instant};

/// Single benchmark measurement.
#[derive(Debug, Clone)]
pub struct BenchResult {
    pub name: String,
    pub iterations: u64,
    pub total_duration: Duration,
    pub min_ns: u64,
    pub max_ns: u64,
    pub mean_ns: u64,
    pub p50_ns: u64,
    pub p95_ns: u64,
    pub p99_ns: u64,
    pub throughput_ops_sec: f64,
}

impl BenchResult {
    pub fn display(&self) -> String {
        format!(
            "{}: {} iters in {:.2?} | mean={:.0}ns p50={:.0}ns p95={:.0}ns p99={:.0}ns | {:.0} ops/sec",
            self.name, self.iterations, self.total_duration,
            self.mean_ns, self.p50_ns, self.p95_ns, self.p99_ns,
            self.throughput_ops_sec
        )
    }
}

/// Benchmark runner — collects timing samples and computes statistics.
pub struct BenchRunner {
    name: String,
    warmup_iters: u64,
    measure_iters: u64,
}

impl BenchRunner {
    pub fn new(name: &str, warmup: u64, measure: u64) -> Self {
        Self {
            name: name.to_string(),
            warmup_iters: warmup,
            measure_iters: measure,
        }
    }

    /// Run benchmark: warmup phase, then measured phase. Returns BenchResult.
    pub fn run<F: FnMut()>(&self, mut f: F) -> BenchResult {
        // Warmup
        for _ in 0..self.warmup_iters {
            f();
        }

        // Measure
        let mut samples = Vec::with_capacity(self.measure_iters as usize);
        let start = Instant::now();
        for _ in 0..self.measure_iters {
            let t0 = Instant::now();
            f();
            samples.push(t0.elapsed().as_nanos() as u64);
        }
        let total = start.elapsed();

        samples.sort_unstable();
        let n = samples.len();
        let sum: u64 = samples.iter().sum();
        let mean = sum / n as u64;
        let p50 = samples[n / 2];
        let idx95 = ((n as f64 * 0.95) as usize).min(n - 1);
        let idx99 = ((n as f64 * 0.99) as usize).min(n - 1);
        let p95 = samples[idx95];
        let p99 = samples[idx99];
        let throughput = self.measure_iters as f64 / total.as_secs_f64();

        BenchResult {
            name: self.name.clone(),
            iterations: self.measure_iters,
            total_duration: total,
            min_ns: samples[0],
            max_ns: samples[n - 1],
            mean_ns: mean,
            p50_ns: p50,
            p95_ns: p95,
            p99_ns: p99,
            throughput_ops_sec: throughput,
        }
    }
}

/// Suite of benchmarks — run all and collect results.
pub struct BenchSuite {
    results: Vec<BenchResult>,
}

impl BenchSuite {
    pub fn new() -> Self {
        Self { results: Vec::new() }
    }

    pub fn add(&mut self, result: BenchResult) {
        self.results.push(result);
    }

    pub fn results(&self) -> &[BenchResult] {
        &self.results
    }

    /// Generate markdown report.
    pub fn report(&self) -> String {
        let mut out = String::from("# Prova Benchmark Report\n\n");
        out.push_str("| Benchmark | Iterations | Mean (ns) | P50 (ns) | P95 (ns) | P99 (ns) | Throughput (ops/s) |\n");
        out.push_str("|-----------|-----------|-----------|----------|----------|----------|--------------------|\n");
        for r in &self.results {
            out.push_str(&format!(
                "| {} | {} | {} | {} | {} | {} | {:.0} |\n",
                r.name, r.iterations, r.mean_ns, r.p50_ns, r.p95_ns, r.p99_ns, r.throughput_ops_sec
            ));
        }
        out
    }

    /// Check for regressions against baseline thresholds (max mean ns per benchmark).
    pub fn check_regressions(&self, baselines: &[(&str, u64)]) -> Vec<String> {
        let mut regressions = Vec::new();
        for (name, max_mean) in baselines {
            if let Some(r) = self.results.iter().find(|r| r.name == *name) {
                if r.mean_ns > *max_mean {
                    regressions.push(format!(
                        "REGRESSION: {} mean={}ns exceeds baseline={}ns (+{:.1}%)",
                        name, r.mean_ns, max_mean,
                        (r.mean_ns as f64 - *max_mean as f64) / *max_mean as f64 * 100.0
                    ));
                }
            }
        }
        regressions
    }
}

/// Benchmark: state trie credit/debit/transfer.
pub fn bench_state_trie(iters: u64) -> BenchResult {
    use crate::state::StateTrie;
    use crate::types::Address;

    let runner = BenchRunner::new("state_trie_ops", 10, iters);
    let mut trie = StateTrie::new();
    // Pre-populate
    for i in 0..1000u64 {
        trie.set_balance(Address::test(i as u8), 1_000_000);
    }

    let mut idx = 0u64;
    runner.run(|| {
        let from = Address::test((idx % 256) as u8);
        let to = Address::test(((idx + 1) % 256) as u8);
        let _ = trie.transfer(from, to, 1);
        idx += 1;
    })
}

/// Benchmark: mempool insert + top retrieval.
pub fn bench_mempool(iters: u64) -> BenchResult {
    use crate::mempool::{Mempool, MempoolConfig, Transaction as MpTx, TxKind};
    use crate::types::Address;

    let runner = BenchRunner::new("mempool_insert", 10, iters);
    let config = MempoolConfig {
        max_txs: 50_000,
        max_bytes: 10_000_000,
        expiry_epochs: 1000,
        replacement_fee_pct: 110,
        max_per_sender: 100,
    };
    let mut pool = Mempool::new(config);
    let mut nonce = 0u64;

    runner.run(|| {
        let tx = MpTx {
            hash: [nonce as u8; 32],
            sender: Address::test((nonce % 100) as u8),
            nonce,
            gas_price: 10 + (nonce % 50),
            gas_limit: 21_000,
            kind: TxKind::Transfer,
            submitted_at: 0,
            size: 256,
        };
        let _ = pool.add(tx);
        nonce += 1;
    })
}

/// Benchmark: scheduler job submission.
pub fn bench_scheduler(iters: u64) -> BenchResult {
    use crate::scheduler::{Scheduler, Provider};
    use crate::types::{Address, ModelId};
    use std::collections::HashSet;

    let runner = BenchRunner::new("scheduler_routing", 10, iters);
    let mut sched = Scheduler::new(100);

    // Register providers
    for i in 0..50u8 {
        let mut models = HashSet::new();
        models.insert(ModelId([i % 5; 32]));
        sched.register_provider(Provider {
            address: Address::test(i),
            models,
            price: 100,
            stake: 10_000,
            reputation: 500,
            capacity: 10,
            active_jobs: 0,
        });
    }

    let mut idx = 0u64;
    runner.run(|| {
        let _result = sched.submit_job(
            Address::test(200),
            ModelId([(idx % 5) as u8; 32]),
            [idx as u8; 32],
            500,
            idx + 200,
        );
        idx += 1;
    })
}

/// Benchmark: fee market gas pricing.
pub fn bench_fee_market(iters: u64) -> BenchResult {
    use crate::gas::FeeMarket;

    let runner = BenchRunner::new("fee_market", 10, iters);

    runner.run(|| {
        let _next = FeeMarket::next_base_fee(15_000_000, 1_000_000_000);
        let mut market = FeeMarket::new();
        let _ = market.finalize_block(0);
    })
}

/// Benchmark: reward distribution.
pub fn bench_rewards(iters: u64) -> BenchResult {
    use crate::rewards::RewardLedger;
    use crate::types::Address;

    let runner = BenchRunner::new("reward_distribution", 10, iters);
    let mut ledger = RewardLedger::new();
    let mut epoch = 0u64;

    runner.run(|| {
        ledger.distribute_block_reward(Address::test((epoch % 50) as u8), epoch);
        epoch += 1;
    })
}

/// Benchmark: genesis state construction.
pub fn bench_genesis(iters: u64) -> BenchResult {
    use crate::genesis::GenesisConfig;

    let runner = BenchRunner::new("genesis_build", 2, iters);
    runner.run(|| {
        let config = GenesisConfig::devnet();
        let _ = config.build_chain_state();
    })
}

/// Run the full benchmark suite.
pub fn run_full_suite() -> BenchSuite {
    let mut suite = BenchSuite::new();
    suite.add(bench_state_trie(500));
    suite.add(bench_mempool(500));
    suite.add(bench_scheduler(200));
    suite.add(bench_fee_market(1000));
    suite.add(bench_rewards(500));
    suite.add(bench_genesis(20));
    suite
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bench_runner_basic() {
        let runner = BenchRunner::new("test_noop", 5, 100);
        let result = runner.run(|| {
            let _ = 1 + 1;
        });
        assert_eq!(result.name, "test_noop");
        assert_eq!(result.iterations, 100);
        assert!(result.mean_ns > 0);
        assert!(result.throughput_ops_sec > 0.0);
    }

    #[test]
    fn test_bench_result_display() {
        let result = BenchResult {
            name: "test".into(),
            iterations: 100,
            total_duration: Duration::from_millis(50),
            min_ns: 100, max_ns: 900, mean_ns: 500,
            p50_ns: 450, p95_ns: 800, p99_ns: 880,
            throughput_ops_sec: 2000.0,
        };
        let s = result.display();
        assert!(s.contains("test"));
        assert!(s.contains("2000 ops/sec"));
    }

    #[test]
    fn test_bench_suite_report() {
        let mut suite = BenchSuite::new();
        suite.add(BenchResult {
            name: "alpha".into(), iterations: 50,
            total_duration: Duration::from_millis(25),
            min_ns: 200, max_ns: 600, mean_ns: 400,
            p50_ns: 380, p95_ns: 550, p99_ns: 590,
            throughput_ops_sec: 2000.0,
        });
        let report = suite.report();
        assert!(report.contains("alpha"));
        assert!(report.contains("Prova Benchmark Report"));
    }

    #[test]
    fn test_regression_check_pass() {
        let mut suite = BenchSuite::new();
        suite.add(BenchResult {
            name: "fast_op".into(), iterations: 100,
            total_duration: Duration::from_millis(10),
            min_ns: 50, max_ns: 200, mean_ns: 100,
            p50_ns: 95, p95_ns: 180, p99_ns: 195,
            throughput_ops_sec: 10_000.0,
        });
        let regressions = suite.check_regressions(&[("fast_op", 500)]);
        assert!(regressions.is_empty());
    }

    #[test]
    fn test_regression_check_fail() {
        let mut suite = BenchSuite::new();
        suite.add(BenchResult {
            name: "slow_op".into(), iterations: 100,
            total_duration: Duration::from_millis(100),
            min_ns: 500, max_ns: 2000, mean_ns: 1000,
            p50_ns: 950, p95_ns: 1800, p99_ns: 1950,
            throughput_ops_sec: 1000.0,
        });
        let regressions = suite.check_regressions(&[("slow_op", 500)]);
        assert_eq!(regressions.len(), 1);
        assert!(regressions[0].contains("REGRESSION"));
    }

    #[test]
    fn test_percentile_ordering() {
        let runner = BenchRunner::new("ordering", 0, 200);
        let result = runner.run(|| {
            let mut sum = 0u64;
            for i in 0..100 { sum += i; }
            let _ = sum;
        });
        assert!(result.min_ns <= result.p50_ns);
        assert!(result.p50_ns <= result.p95_ns);
        assert!(result.p95_ns <= result.p99_ns);
        assert!(result.p99_ns <= result.max_ns);
    }

    #[test]
    fn test_bench_suite_multiple() {
        let mut suite = BenchSuite::new();
        for i in 0..5 {
            suite.add(BenchResult {
                name: format!("bench_{}", i), iterations: 100,
                total_duration: Duration::from_millis(10),
                min_ns: 50, max_ns: 200, mean_ns: 100,
                p50_ns: 95, p95_ns: 180, p99_ns: 195,
                throughput_ops_sec: 10_000.0,
            });
        }
        assert_eq!(suite.results().len(), 5);
        let report = suite.report();
        assert!(report.contains("bench_4"));
    }

    #[test]
    fn test_bench_runner_warmup() {
        let mut count = 0u64;
        let runner = BenchRunner::new("warmup_test", 50, 100);
        let _result = runner.run(|| { count += 1; });
        assert_eq!(count, 150);
    }

    #[test]
    fn test_regression_missing_baseline() {
        let suite = BenchSuite::new();
        let regressions = suite.check_regressions(&[("nonexistent", 100)]);
        assert!(regressions.is_empty());
    }

    #[test]
    fn test_bench_state_trie() {
        let result = bench_state_trie(50);
        assert_eq!(result.name, "state_trie_ops");
        assert!(result.throughput_ops_sec > 0.0);
    }

    #[test]
    fn test_bench_mempool() {
        let result = bench_mempool(50);
        assert_eq!(result.name, "mempool_insert");
        assert!(result.min_ns <= result.max_ns);
    }

    #[test]
    fn test_bench_scheduler() {
        let result = bench_scheduler(30);
        assert_eq!(result.name, "scheduler_routing");
        assert!(result.throughput_ops_sec > 0.0);
    }

    #[test]
    fn test_bench_fee_market() {
        let result = bench_fee_market(100);
        assert_eq!(result.name, "fee_market");
        assert!(result.throughput_ops_sec > 0.0);
    }

    #[test]
    fn test_bench_rewards() {
        let result = bench_rewards(50);
        assert_eq!(result.name, "reward_distribution");
    }

    #[test]
    fn test_bench_genesis() {
        let result = bench_genesis(5);
        assert_eq!(result.name, "genesis_build");
        assert!(result.throughput_ops_sec > 0.0);
    }

    #[test]
    fn test_full_suite() {
        let suite = run_full_suite();
        assert_eq!(suite.results().len(), 6);
        for r in suite.results() {
            assert!(r.throughput_ops_sec > 0.0);
            assert!(r.min_ns <= r.max_ns);
        }
        let report = suite.report();
        assert!(report.lines().count() > 5);
    }
}
