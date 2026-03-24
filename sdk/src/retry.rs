//! Rate limiting & retry logic for Prova SDK.
//!
//! Provides:
//! - `RetryPolicy`: configurable exponential backoff with jitter
//! - `RateLimiter`: token-bucket rate limiter
//! - `ResilientTransport`: wraps any `Transport` with retry + rate limiting

use crate::rpc_client::{RpcClientError, Transport};
use std::sync::Mutex;
use std::time::{Duration, Instant};

// ── Retry Policy ─────────────────────────────────────────────

/// Configuration for retry behavior with exponential backoff.
#[derive(Debug, Clone)]
pub struct RetryPolicy {
    /// Maximum number of attempts (including the first).
    pub max_attempts: u32,
    /// Base delay before first retry.
    pub base_delay: Duration,
    /// Maximum delay cap.
    pub max_delay: Duration,
    /// Backoff multiplier (e.g. 2.0 for doubling).
    pub multiplier: f64,
    /// Which errors are retryable.
    pub retryable: RetryableErrors,
}

#[derive(Debug, Clone, PartialEq)]
pub enum RetryableErrors {
    /// Retry all transport errors.
    All,
    /// Retry only transport (network) errors, not RPC-level errors.
    TransportOnly,
    /// Retry transport errors and specific RPC error codes.
    WithCodes(Vec<i64>),
}

impl Default for RetryPolicy {
    fn default() -> Self {
        Self {
            max_attempts: 3,
            base_delay: Duration::from_millis(200),
            max_delay: Duration::from_secs(10),
            multiplier: 2.0,
            retryable: RetryableErrors::TransportOnly,
        }
    }
}

impl RetryPolicy {
    pub fn with_max_attempts(mut self, n: u32) -> Self {
        self.max_attempts = n;
        self
    }

    pub fn with_base_delay(mut self, d: Duration) -> Self {
        self.base_delay = d;
        self
    }

    pub fn with_max_delay(mut self, d: Duration) -> Self {
        self.max_delay = d;
        self
    }

    pub fn with_multiplier(mut self, m: f64) -> Self {
        self.multiplier = m;
        self
    }

    pub fn with_retryable(mut self, r: RetryableErrors) -> Self {
        self.retryable = r;
        self
    }

    /// No retries — fail immediately.
    pub fn none() -> Self {
        Self {
            max_attempts: 1,
            ..Default::default()
        }
    }

    /// Aggressive retry for critical operations.
    pub fn aggressive() -> Self {
        Self {
            max_attempts: 5,
            base_delay: Duration::from_millis(100),
            max_delay: Duration::from_secs(30),
            multiplier: 2.0,
            retryable: RetryableErrors::All,
        }
    }

    /// Calculate delay for attempt n (0-indexed).
    pub fn delay_for_attempt(&self, attempt: u32) -> Duration {
        if attempt == 0 {
            return Duration::ZERO;
        }
        let delay_ms =
            self.base_delay.as_millis() as f64 * self.multiplier.powi(attempt as i32 - 1);
        let capped = delay_ms.min(self.max_delay.as_millis() as f64);
        Duration::from_millis(capped as u64)
    }

    /// Check if an error is retryable under this policy.
    pub fn is_retryable(&self, err: &RpcClientError) -> bool {
        match &self.retryable {
            RetryableErrors::All => true,
            RetryableErrors::TransportOnly => {
                matches!(err, RpcClientError::Transport(_))
            }
            RetryableErrors::WithCodes(codes) => match err {
                RpcClientError::Transport(_) => true,
                RpcClientError::Rpc(e) => codes.contains(&e.code),
                _ => false,
            },
        }
    }
}

// ── Token Bucket Rate Limiter ────────────────────────────────

/// Simple token-bucket rate limiter.
#[derive(Debug)]
pub struct RateLimiter {
    inner: Mutex<RateLimiterState>,
}

#[derive(Debug)]
struct RateLimiterState {
    /// Max tokens (burst capacity).
    capacity: u32,
    /// Current available tokens.
    tokens: f64,
    /// Tokens added per second.
    refill_rate: f64,
    /// Last refill timestamp.
    last_refill: Instant,
}

impl RateLimiter {
    /// Create a rate limiter with `capacity` burst and `per_second` sustained rate.
    pub fn new(capacity: u32, per_second: f64) -> Self {
        Self {
            inner: Mutex::new(RateLimiterState {
                capacity,
                tokens: capacity as f64,
                refill_rate: per_second,
                last_refill: Instant::now(),
            }),
        }
    }

    /// Try to acquire one token. Returns Ok(()) if allowed,
    /// Err(wait_duration) if rate limited.
    pub fn try_acquire(&self) -> Result<(), Duration> {
        let mut state = self.inner.lock().unwrap();
        let now = Instant::now();
        let elapsed = now.duration_since(state.last_refill);

        // Refill tokens
        state.tokens += elapsed.as_secs_f64() * state.refill_rate;
        if state.tokens > state.capacity as f64 {
            state.tokens = state.capacity as f64;
        }
        state.last_refill = now;

        if state.tokens >= 1.0 {
            state.tokens -= 1.0;
            Ok(())
        } else {
            // How long until one token is available
            let deficit = 1.0 - state.tokens;
            let wait = Duration::from_secs_f64(deficit / state.refill_rate);
            Err(wait)
        }
    }

    /// Acquire a token, blocking (simulated via spin) if necessary.
    /// In real async code this would be await-based.
    pub fn acquire_blocking(&self) {
        loop {
            match self.try_acquire() {
                Ok(()) => return,
                Err(wait) => std::thread::sleep(wait),
            }
        }
    }

    /// Current available tokens (for monitoring).
    pub fn available(&self) -> f64 {
        let state = self.inner.lock().unwrap();
        state.tokens
    }

    /// Reset to full capacity.
    pub fn reset(&self) {
        let mut state = self.inner.lock().unwrap();
        state.tokens = state.capacity as f64;
        state.last_refill = Instant::now();
    }
}

// ── Resilient Transport ──────────────────────────────────────

/// Wraps any Transport with retry logic and optional rate limiting.
pub struct ResilientTransport<T: Transport> {
    inner: T,
    retry_policy: RetryPolicy,
    rate_limiter: Option<RateLimiter>,
    /// Track attempt counts for observability.
    stats: Mutex<TransportStats>,
}

#[derive(Debug, Default, Clone)]
pub struct TransportStats {
    pub total_requests: u64,
    pub total_retries: u64,
    pub total_rate_limited: u64,
    pub total_failures: u64,
}

impl<T: Transport> ResilientTransport<T> {
    pub fn new(inner: T, retry_policy: RetryPolicy) -> Self {
        Self {
            inner,
            retry_policy,
            rate_limiter: None,
            stats: Mutex::new(TransportStats::default()),
        }
    }

    pub fn with_rate_limiter(mut self, limiter: RateLimiter) -> Self {
        self.rate_limiter = Some(limiter);
        self
    }

    pub fn stats(&self) -> TransportStats {
        self.stats.lock().unwrap().clone()
    }

    pub fn reset_stats(&self) {
        *self.stats.lock().unwrap() = TransportStats::default();
    }
}

impl<T: Transport> Transport for ResilientTransport<T> {
    fn send_raw(&self, request: &str) -> Result<String, RpcClientError> {
        let mut stats = self.stats.lock().unwrap();
        stats.total_requests += 1;
        drop(stats);

        // Rate limiting
        if let Some(ref limiter) = self.rate_limiter {
            match limiter.try_acquire() {
                Ok(()) => {}
                Err(_wait) => {
                    let mut stats = self.stats.lock().unwrap();
                    stats.total_rate_limited += 1;
                    drop(stats);
                    // Block until token available
                    limiter.acquire_blocking();
                }
            }
        }

        let mut last_err = None;
        for attempt in 0..self.retry_policy.max_attempts {
            // Backoff delay (skip for first attempt)
            let delay = self.retry_policy.delay_for_attempt(attempt);
            if !delay.is_zero() {
                std::thread::sleep(delay);
            }

            match self.inner.send_raw(request) {
                Ok(response) => return Ok(response),
                Err(e) => {
                    if attempt + 1 < self.retry_policy.max_attempts
                        && self.retry_policy.is_retryable(&e)
                    {
                        let mut stats = self.stats.lock().unwrap();
                        stats.total_retries += 1;
                        drop(stats);
                        last_err = Some(e);
                        continue;
                    }
                    let mut stats = self.stats.lock().unwrap();
                    stats.total_failures += 1;
                    drop(stats);
                    return Err(e);
                }
            }
        }

        let mut stats = self.stats.lock().unwrap();
        stats.total_failures += 1;
        drop(stats);
        Err(last_err.unwrap_or_else(|| RpcClientError::Transport("max retries exhausted".into())))
    }
}

// ── Tests ────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU32, Ordering};
    use std::sync::Arc;

    /// Transport that fails N times then succeeds.
    struct FailNTransport {
        fail_count: AtomicU32,
        target_fails: u32,
    }

    impl FailNTransport {
        fn new(target_fails: u32) -> Self {
            Self {
                fail_count: AtomicU32::new(0),
                target_fails,
            }
        }
    }

    impl Transport for FailNTransport {
        fn send_raw(&self, _request: &str) -> Result<String, RpcClientError> {
            let n = self.fail_count.fetch_add(1, Ordering::SeqCst);
            if n < self.target_fails {
                Err(RpcClientError::Transport("connection reset".into()))
            } else {
                Ok(r#"{"jsonrpc":"2.0","result":"ok","id":1}"#.into())
            }
        }
    }

    /// Transport that always fails.
    struct AlwaysFailTransport;

    impl Transport for AlwaysFailTransport {
        fn send_raw(&self, _request: &str) -> Result<String, RpcClientError> {
            Err(RpcClientError::Transport("permanent failure".into()))
        }
    }

    /// Counting transport.
    struct CountingTransport {
        count: AtomicU32,
    }

    impl CountingTransport {
        fn new() -> Self {
            Self {
                count: AtomicU32::new(0),
            }
        }
        fn call_count(&self) -> u32 {
            self.count.load(Ordering::SeqCst)
        }
    }

    impl Transport for CountingTransport {
        fn send_raw(&self, _request: &str) -> Result<String, RpcClientError> {
            self.count.fetch_add(1, Ordering::SeqCst);
            Ok(r#"{"jsonrpc":"2.0","result":"ok","id":1}"#.into())
        }
    }

    // ── RetryPolicy tests ────────────────────────────────────

    #[test]
    fn test_default_policy() {
        let p = RetryPolicy::default();
        assert_eq!(p.max_attempts, 3);
        assert_eq!(p.multiplier, 2.0);
        assert_eq!(p.retryable, RetryableErrors::TransportOnly);
    }

    #[test]
    fn test_delay_calculation() {
        let p = RetryPolicy::default();
        assert_eq!(p.delay_for_attempt(0), Duration::ZERO);
        assert_eq!(p.delay_for_attempt(1), Duration::from_millis(200));
        assert_eq!(p.delay_for_attempt(2), Duration::from_millis(400));
        assert_eq!(p.delay_for_attempt(3), Duration::from_millis(800));
    }

    #[test]
    fn test_delay_capped_at_max() {
        let p = RetryPolicy::default().with_max_delay(Duration::from_millis(300));
        assert_eq!(p.delay_for_attempt(1), Duration::from_millis(200));
        assert_eq!(p.delay_for_attempt(2), Duration::from_millis(300)); // capped
        assert_eq!(p.delay_for_attempt(5), Duration::from_millis(300)); // capped
    }

    #[test]
    fn test_no_retry_policy() {
        let p = RetryPolicy::none();
        assert_eq!(p.max_attempts, 1);
    }

    #[test]
    fn test_aggressive_policy() {
        let p = RetryPolicy::aggressive();
        assert_eq!(p.max_attempts, 5);
        assert_eq!(p.retryable, RetryableErrors::All);
    }

    #[test]
    fn test_retryable_transport_only() {
        let p = RetryPolicy::default();
        assert!(p.is_retryable(&RpcClientError::Transport("net".into())));
        assert!(!p.is_retryable(&RpcClientError::Serialization("bad json".into())));
    }

    #[test]
    fn test_retryable_with_codes() {
        let p =
            RetryPolicy::default().with_retryable(RetryableErrors::WithCodes(vec![-32000, -32603]));
        // Transport errors always retryable
        assert!(p.is_retryable(&RpcClientError::Transport("net".into())));
        // Matching RPC code
        let rpc_err = RpcClientError::Rpc(crate::rpc_client::JsonRpcError {
            code: -32000,
            message: "server busy".into(),
            data: None,
        });
        assert!(p.is_retryable(&rpc_err));
        // Non-matching code
        let rpc_err2 = RpcClientError::Rpc(crate::rpc_client::JsonRpcError {
            code: -32601,
            message: "method not found".into(),
            data: None,
        });
        assert!(!p.is_retryable(&rpc_err2));
    }

    // ── RateLimiter tests ────────────────────────────────────

    #[test]
    fn test_rate_limiter_burst() {
        let limiter = RateLimiter::new(5, 10.0);
        // Should allow 5 immediate acquisitions
        for _ in 0..5 {
            assert!(limiter.try_acquire().is_ok());
        }
        // 6th should be rate limited
        assert!(limiter.try_acquire().is_err());
    }

    #[test]
    fn test_rate_limiter_refill() {
        let limiter = RateLimiter::new(2, 1000.0); // 1000/sec = refills fast
        assert!(limiter.try_acquire().is_ok());
        assert!(limiter.try_acquire().is_ok());
        // Drain
        let _ = limiter.try_acquire();
        // Sleep a tiny bit to refill
        std::thread::sleep(Duration::from_millis(10));
        // Should have refilled some tokens
        assert!(limiter.try_acquire().is_ok());
    }

    #[test]
    fn test_rate_limiter_reset() {
        let limiter = RateLimiter::new(3, 1.0);
        limiter.try_acquire().unwrap();
        limiter.try_acquire().unwrap();
        limiter.try_acquire().unwrap();
        assert!(limiter.try_acquire().is_err());
        limiter.reset();
        assert!(limiter.try_acquire().is_ok());
    }

    // ── ResilientTransport tests ─────────────────────────────

    #[test]
    fn test_retry_succeeds_after_failures() {
        let transport = FailNTransport::new(2);
        let policy = RetryPolicy::default()
            .with_max_attempts(3)
            .with_base_delay(Duration::from_millis(1)); // fast for tests
        let resilient = ResilientTransport::new(transport, policy);
        let result = resilient.send_raw("test");
        assert!(result.is_ok());
        let stats = resilient.stats();
        assert_eq!(stats.total_requests, 1);
        assert_eq!(stats.total_retries, 2);
    }

    #[test]
    fn test_retry_exhausted() {
        let transport = AlwaysFailTransport;
        let policy = RetryPolicy::default()
            .with_max_attempts(3)
            .with_base_delay(Duration::from_millis(1));
        let resilient = ResilientTransport::new(transport, policy);
        let result = resilient.send_raw("test");
        assert!(result.is_err());
        let stats = resilient.stats();
        assert_eq!(stats.total_failures, 1);
    }

    #[test]
    fn test_no_retry_policy_fails_immediately() {
        let transport = FailNTransport::new(1);
        let resilient = ResilientTransport::new(transport, RetryPolicy::none());
        let result = resilient.send_raw("test");
        assert!(result.is_err());
    }

    #[test]
    fn test_success_on_first_try_no_retries() {
        let transport = FailNTransport::new(0);
        let resilient = ResilientTransport::new(
            transport,
            RetryPolicy::default().with_base_delay(Duration::from_millis(1)),
        );
        let result = resilient.send_raw("test");
        assert!(result.is_ok());
        assert_eq!(resilient.stats().total_retries, 0);
    }

    #[test]
    fn test_rate_limited_transport() {
        let transport = CountingTransport::new();
        let policy = RetryPolicy::none();
        let limiter = RateLimiter::new(2, 1000.0);
        let resilient = ResilientTransport::new(transport, policy).with_rate_limiter(limiter);

        // First two should be immediate
        resilient.send_raw("1").unwrap();
        resilient.send_raw("2").unwrap();
        // Third may trigger rate limiting but should still succeed
        resilient.send_raw("3").unwrap();
        assert_eq!(resilient.stats().total_requests, 3);
    }

    #[test]
    fn test_stats_tracking() {
        let transport = FailNTransport::new(1);
        let policy = RetryPolicy::default()
            .with_max_attempts(3)
            .with_base_delay(Duration::from_millis(1));
        let resilient = ResilientTransport::new(transport, policy);

        resilient.send_raw("test").unwrap();
        let stats = resilient.stats();
        assert_eq!(stats.total_requests, 1);
        assert_eq!(stats.total_retries, 1);
        assert_eq!(stats.total_failures, 0);

        resilient.reset_stats();
        let stats = resilient.stats();
        assert_eq!(stats.total_requests, 0);
    }

    #[test]
    fn test_builder_pattern() {
        let p = RetryPolicy::default()
            .with_max_attempts(5)
            .with_base_delay(Duration::from_millis(100))
            .with_max_delay(Duration::from_secs(60))
            .with_multiplier(3.0)
            .with_retryable(RetryableErrors::All);
        assert_eq!(p.max_attempts, 5);
        assert_eq!(p.base_delay, Duration::from_millis(100));
        assert_eq!(p.max_delay, Duration::from_secs(60));
        assert_eq!(p.multiplier, 3.0);
        assert_eq!(p.retryable, RetryableErrors::All);
    }
}
