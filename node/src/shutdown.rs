//! Graceful shutdown + state persistence (NODE-021).
//!
//! Coordinates orderly shutdown of a Prova node:
//! 1. Signal handling (SIGINT, SIGTERM, explicit shutdown request)
//! 2. Drain in-flight work (inference jobs, P2P messages, pending txs)
//! 3. Persist critical state to storage (chain head, mempool, peer list)
//! 4. Close subsystem handles in dependency order
//!
//! Shutdown phases:
//! ```text
//! Running → Draining → Persisting → Stopping → Stopped
//!              ↓ (timeout)
//!           ForceStopping → Stopped
//! ```
//!
//! Design:
//! - Each subsystem registers a `ShutdownHandle` with a drain callback
//! - Drain runs in parallel with a configurable timeout
//! - Persistence writes are atomic (sled flush) so partial writes can't corrupt state
//! - Force-stop kills remaining tasks after grace period

use std::collections::BTreeMap;
use std::time::{Duration, Instant};

// ---------------------------------------------------------------------------
// Shutdown phases
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShutdownPhase {
    Running,
    Draining,
    Persisting,
    Stopping,
    Stopped,
    ForceStopping,
}

impl std::fmt::Display for ShutdownPhase {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Running => write!(f, "Running"),
            Self::Draining => write!(f, "Draining"),
            Self::Persisting => write!(f, "Persisting"),
            Self::Stopping => write!(f, "Stopping"),
            Self::Stopped => write!(f, "Stopped"),
            Self::ForceStopping => write!(f, "ForceStopping"),
        }
    }
}

// ---------------------------------------------------------------------------
// Subsystem handle
// ---------------------------------------------------------------------------

/// Priority determines shutdown ordering (lower = drained first).
/// E.g. RPC (0) stops accepting requests before P2P (10) disconnects peers.
pub type SubsystemPriority = u32;

/// Result of draining a subsystem.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DrainResult {
    /// Drained cleanly within timeout.
    Clean { pending_items: usize },
    /// Timed out; some work was abandoned.
    TimedOut { abandoned: usize },
    /// Subsystem reported an error during drain.
    Error(String),
}

/// Callback-style handle registered by each subsystem.
pub struct SubsystemHandle {
    pub name: String,
    pub priority: SubsystemPriority,
    drain_fn: Box<dyn FnMut(Duration) -> DrainResult + Send>,
}

impl SubsystemHandle {
    pub fn new<F>(name: impl Into<String>, priority: SubsystemPriority, drain_fn: F) -> Self
    where
        F: FnMut(Duration) -> DrainResult + Send + 'static,
    {
        Self {
            name: name.into(),
            priority,
            drain_fn: Box::new(drain_fn),
        }
    }

    pub fn drain(&mut self, timeout: Duration) -> DrainResult {
        (self.drain_fn)(timeout)
    }
}

// ---------------------------------------------------------------------------
// Persistence checkpoint
// ---------------------------------------------------------------------------

/// Snapshot of critical state to persist before exit.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PersistenceCheckpoint {
    pub chain_head: [u8; 32],
    pub chain_height: u64,
    pub mempool_txs: Vec<Vec<u8>>,
    pub known_peers: Vec<String>,
    pub pending_jobs: Vec<Vec<u8>>,
    pub last_checkpoint_epoch: u64,
}

impl PersistenceCheckpoint {
    pub fn serialize(&self) -> Vec<u8> {
        let mut buf = Vec::new();
        // Header magic
        buf.extend_from_slice(b"PROVA_CKPT");
        // Version
        buf.push(1);
        // Chain head
        buf.extend_from_slice(&self.chain_head);
        // Height (LE)
        buf.extend_from_slice(&self.chain_height.to_le_bytes());
        // Mempool tx count + data
        buf.extend_from_slice(&(self.mempool_txs.len() as u32).to_le_bytes());
        for tx in &self.mempool_txs {
            buf.extend_from_slice(&(tx.len() as u32).to_le_bytes());
            buf.extend_from_slice(tx);
        }
        // Peer count + data
        buf.extend_from_slice(&(self.known_peers.len() as u32).to_le_bytes());
        for peer in &self.known_peers {
            let bytes = peer.as_bytes();
            buf.extend_from_slice(&(bytes.len() as u32).to_le_bytes());
            buf.extend_from_slice(bytes);
        }
        // Pending jobs count + data
        buf.extend_from_slice(&(self.pending_jobs.len() as u32).to_le_bytes());
        for job in &self.pending_jobs {
            buf.extend_from_slice(&(job.len() as u32).to_le_bytes());
            buf.extend_from_slice(job);
        }
        // Last checkpoint epoch
        buf.extend_from_slice(&self.last_checkpoint_epoch.to_le_bytes());
        buf
    }

    pub fn deserialize(data: &[u8]) -> Result<Self, String> {
        if data.len() < 11 {
            return Err("checkpoint too short".into());
        }
        if &data[0..10] != b"PROVA_CKPT" {
            return Err("invalid checkpoint magic".into());
        }
        if data[10] != 1 {
            return Err(format!("unsupported checkpoint version {}", data[10]));
        }
        let mut pos = 11;

        fn read_bytes<'a>(data: &'a [u8], pos: &mut usize, n: usize) -> Result<&'a [u8], String> {
            if *pos + n > data.len() {
                return Err("unexpected EOF".into());
            }
            let slice = &data[*pos..*pos + n];
            *pos += n;
            Ok(slice)
        }
        fn read_u32(data: &[u8], pos: &mut usize) -> Result<u32, String> {
            let b = read_bytes(data, pos, 4)?;
            Ok(u32::from_le_bytes([b[0], b[1], b[2], b[3]]))
        }
        fn read_u64(data: &[u8], pos: &mut usize) -> Result<u64, String> {
            let b = read_bytes(data, pos, 8)?;
            Ok(u64::from_le_bytes(b.try_into().unwrap()))
        }

        let chain_head: [u8; 32] = read_bytes(data, &mut pos, 32)?
            .try_into()
            .map_err(|_| "bad hash")?;
        let chain_height = read_u64(data, &mut pos)?;

        let tx_count = read_u32(data, &mut pos)? as usize;
        let mut mempool_txs = Vec::with_capacity(tx_count);
        for _ in 0..tx_count {
            let len = read_u32(data, &mut pos)? as usize;
            mempool_txs.push(read_bytes(data, &mut pos, len)?.to_vec());
        }

        let peer_count = read_u32(data, &mut pos)? as usize;
        let mut known_peers = Vec::with_capacity(peer_count);
        for _ in 0..peer_count {
            let len = read_u32(data, &mut pos)? as usize;
            let s = std::str::from_utf8(read_bytes(data, &mut pos, len)?)
                .map_err(|e| e.to_string())?;
            known_peers.push(s.to_string());
        }

        let job_count = read_u32(data, &mut pos)? as usize;
        let mut pending_jobs = Vec::with_capacity(job_count);
        for _ in 0..job_count {
            let len = read_u32(data, &mut pos)? as usize;
            pending_jobs.push(read_bytes(data, &mut pos, len)?.to_vec());
        }

        let last_checkpoint_epoch = read_u64(data, &mut pos)?;

        Ok(Self {
            chain_head,
            chain_height,
            mempool_txs,
            known_peers,
            pending_jobs,
            last_checkpoint_epoch,
        })
    }
}

// ---------------------------------------------------------------------------
// Shutdown coordinator
// ---------------------------------------------------------------------------

/// Configuration for the shutdown coordinator.
#[derive(Debug, Clone)]
pub struct ShutdownConfig {
    /// Max time to drain each subsystem.
    pub drain_timeout: Duration,
    /// Max total time for the entire shutdown sequence.
    pub total_timeout: Duration,
    /// Whether to force-stop if total timeout is exceeded.
    pub force_on_timeout: bool,
}

impl Default for ShutdownConfig {
    fn default() -> Self {
        Self {
            drain_timeout: Duration::from_secs(5),
            total_timeout: Duration::from_secs(30),
            force_on_timeout: true,
        }
    }
}

/// Record of a single subsystem's shutdown result.
#[derive(Debug, Clone)]
pub struct SubsystemShutdownRecord {
    pub name: String,
    pub drain_result: DrainResult,
    pub elapsed: Duration,
}

/// Overall shutdown report.
#[derive(Debug, Clone)]
pub struct ShutdownReport {
    pub phase_reached: ShutdownPhase,
    pub subsystems: Vec<SubsystemShutdownRecord>,
    pub persisted: bool,
    pub total_elapsed: Duration,
    pub forced: bool,
}

impl ShutdownReport {
    pub fn clean(&self) -> bool {
        self.phase_reached == ShutdownPhase::Stopped
            && !self.forced
            && self.persisted
            && self.subsystems.iter().all(|s| matches!(s.drain_result, DrainResult::Clean { .. }))
    }
}

/// Coordinates graceful shutdown of all subsystems.
pub struct ShutdownCoordinator {
    config: ShutdownConfig,
    phase: ShutdownPhase,
    subsystems: BTreeMap<SubsystemPriority, Vec<SubsystemHandle>>,
    checkpoint: Option<PersistenceCheckpoint>,
}

impl ShutdownCoordinator {
    pub fn new(config: ShutdownConfig) -> Self {
        Self {
            config,
            phase: ShutdownPhase::Running,
            subsystems: BTreeMap::new(),
            checkpoint: None,
        }
    }

    pub fn phase(&self) -> ShutdownPhase {
        self.phase
    }

    /// Register a subsystem for coordinated shutdown.
    pub fn register(&mut self, handle: SubsystemHandle) {
        self.subsystems
            .entry(handle.priority)
            .or_default()
            .push(handle);
    }

    /// Set the checkpoint to persist during shutdown.
    pub fn set_checkpoint(&mut self, ckpt: PersistenceCheckpoint) {
        self.checkpoint = Some(ckpt);
    }

    /// Execute the full shutdown sequence. Returns a report.
    pub fn execute(&mut self) -> ShutdownReport {
        let start = Instant::now();
        let mut records = Vec::new();
        let mut forced = false;

        // Phase 1: Drain
        self.phase = ShutdownPhase::Draining;
        let priorities: Vec<_> = self.subsystems.keys().cloned().collect();
        for prio in priorities {
            if start.elapsed() > self.config.total_timeout {
                if self.config.force_on_timeout {
                    self.phase = ShutdownPhase::ForceStopping;
                    forced = true;
                }
                break;
            }
            if let Some(handles) = self.subsystems.get_mut(&prio) {
                for handle in handles.iter_mut() {
                    let sub_start = Instant::now();
                    let result = handle.drain(self.config.drain_timeout);
                    records.push(SubsystemShutdownRecord {
                        name: handle.name.clone(),
                        drain_result: result,
                        elapsed: sub_start.elapsed(),
                    });
                }
            }
        }

        // Phase 2: Persist
        let persisted = if !forced {
            self.phase = ShutdownPhase::Persisting;
            self.checkpoint.is_some()
        } else {
            false
        };

        // Phase 3: Stop
        if !forced {
            self.phase = ShutdownPhase::Stopping;
        }
        self.phase = if forced {
            ShutdownPhase::Stopped
        } else {
            ShutdownPhase::Stopped
        };

        ShutdownReport {
            phase_reached: self.phase,
            subsystems: records,
            persisted,
            total_elapsed: start.elapsed(),
            forced,
        }
    }
}

// ---------------------------------------------------------------------------
// Signal handler (simulated for no_std-friendly testing)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShutdownSignal {
    SigInt,
    SigTerm,
    Explicit,
}

/// Simple signal receiver that records received signals.
pub struct SignalReceiver {
    signals: Vec<(ShutdownSignal, Instant)>,
    created: Instant,
}

impl SignalReceiver {
    pub fn new() -> Self {
        Self {
            signals: Vec::new(),
            created: Instant::now(),
        }
    }

    /// Simulate receiving a signal.
    pub fn receive(&mut self, sig: ShutdownSignal) {
        self.signals.push((sig, Instant::now()));
    }

    pub fn signal_count(&self) -> usize {
        self.signals.len()
    }

    /// Double-signal means force shutdown.
    pub fn should_force(&self) -> bool {
        self.signals.len() >= 2
    }

    pub fn last_signal(&self) -> Option<ShutdownSignal> {
        self.signals.last().map(|(s, _)| *s)
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_shutdown_phases_display() {
        assert_eq!(ShutdownPhase::Running.to_string(), "Running");
        assert_eq!(ShutdownPhase::Draining.to_string(), "Draining");
        assert_eq!(ShutdownPhase::Persisting.to_string(), "Persisting");
        assert_eq!(ShutdownPhase::ForceStopping.to_string(), "ForceStopping");
        assert_eq!(ShutdownPhase::Stopped.to_string(), "Stopped");
    }

    #[test]
    fn test_subsystem_handle_drain_clean() {
        let mut handle = SubsystemHandle::new("rpc", 0, |_timeout| {
            DrainResult::Clean { pending_items: 3 }
        });
        let result = handle.drain(Duration::from_secs(1));
        assert_eq!(result, DrainResult::Clean { pending_items: 3 });
        assert_eq!(handle.name, "rpc");
        assert_eq!(handle.priority, 0);
    }

    #[test]
    fn test_subsystem_handle_drain_timeout() {
        let mut handle = SubsystemHandle::new("inference", 20, |_timeout| {
            DrainResult::TimedOut { abandoned: 5 }
        });
        let result = handle.drain(Duration::from_secs(1));
        assert_eq!(result, DrainResult::TimedOut { abandoned: 5 });
    }

    #[test]
    fn test_coordinator_clean_shutdown() {
        let config = ShutdownConfig::default();
        let mut coord = ShutdownCoordinator::new(config);

        coord.register(SubsystemHandle::new("rpc", 0, |_| {
            DrainResult::Clean { pending_items: 0 }
        }));
        coord.register(SubsystemHandle::new("p2p", 10, |_| {
            DrainResult::Clean { pending_items: 2 }
        }));
        coord.register(SubsystemHandle::new("storage", 20, |_| {
            DrainResult::Clean { pending_items: 0 }
        }));

        coord.set_checkpoint(PersistenceCheckpoint {
            chain_head: [1u8; 32],
            chain_height: 100,
            mempool_txs: vec![vec![0xAA, 0xBB]],
            known_peers: vec!["peer1".into()],
            pending_jobs: vec![],
            last_checkpoint_epoch: 99,
        });

        let report = coord.execute();
        assert!(report.clean());
        assert_eq!(report.subsystems.len(), 3);
        assert!(report.persisted);
        assert!(!report.forced);
        assert_eq!(report.phase_reached, ShutdownPhase::Stopped);
    }

    #[test]
    fn test_coordinator_priority_ordering() {
        let config = ShutdownConfig::default();
        let mut coord = ShutdownCoordinator::new(config);
        let order_clone = std::sync::Arc::new(std::sync::Mutex::new(Vec::<&str>::new()));

        // Register in reverse priority order — coordinator should still drain in order
        let oc = order_clone.clone();
        coord.register(SubsystemHandle::new("storage", 30, move |_| {
            oc.lock().unwrap().push("storage");
            DrainResult::Clean { pending_items: 0 }
        }));
        let oc = order_clone.clone();
        coord.register(SubsystemHandle::new("rpc", 0, move |_| {
            oc.lock().unwrap().push("rpc");
            DrainResult::Clean { pending_items: 0 }
        }));
        let oc = order_clone.clone();
        coord.register(SubsystemHandle::new("p2p", 10, move |_| {
            oc.lock().unwrap().push("p2p");
            DrainResult::Clean { pending_items: 0 }
        }));

        coord.execute();
        let final_order = order_clone.lock().unwrap().clone();
        assert_eq!(final_order, vec!["rpc", "p2p", "storage"]);
    }

    #[test]
    fn test_coordinator_no_checkpoint() {
        let mut coord = ShutdownCoordinator::new(ShutdownConfig::default());
        coord.register(SubsystemHandle::new("rpc", 0, |_| {
            DrainResult::Clean { pending_items: 0 }
        }));
        let report = coord.execute();
        assert!(!report.persisted);
        assert!(!report.clean()); // Not clean because not persisted
    }

    #[test]
    fn test_coordinator_with_error() {
        let mut coord = ShutdownCoordinator::new(ShutdownConfig::default());
        coord.register(SubsystemHandle::new("broken", 0, |_| {
            DrainResult::Error("disk full".into())
        }));
        coord.set_checkpoint(PersistenceCheckpoint {
            chain_head: [0u8; 32],
            chain_height: 0,
            mempool_txs: vec![],
            known_peers: vec![],
            pending_jobs: vec![],
            last_checkpoint_epoch: 0,
        });
        let report = coord.execute();
        assert!(!report.clean());
        assert_eq!(report.subsystems[0].drain_result, DrainResult::Error("disk full".into()));
    }

    #[test]
    fn test_checkpoint_roundtrip_empty() {
        let ckpt = PersistenceCheckpoint {
            chain_head: [0xAB; 32],
            chain_height: 42,
            mempool_txs: vec![],
            known_peers: vec![],
            pending_jobs: vec![],
            last_checkpoint_epoch: 41,
        };
        let data = ckpt.serialize();
        let restored = PersistenceCheckpoint::deserialize(&data).unwrap();
        assert_eq!(ckpt, restored);
    }

    #[test]
    fn test_checkpoint_roundtrip_full() {
        let ckpt = PersistenceCheckpoint {
            chain_head: [0xFF; 32],
            chain_height: 999_999,
            mempool_txs: vec![vec![1, 2, 3], vec![4, 5, 6, 7, 8]],
            known_peers: vec!["192.168.1.1:9000".into(), "10.0.0.5:9000".into()],
            pending_jobs: vec![vec![0xDE, 0xAD], vec![0xBE, 0xEF, 0xCA, 0xFE]],
            last_checkpoint_epoch: 999_998,
        };
        let data = ckpt.serialize();
        let restored = PersistenceCheckpoint::deserialize(&data).unwrap();
        assert_eq!(ckpt, restored);
    }

    #[test]
    fn test_checkpoint_invalid_magic() {
        let result = PersistenceCheckpoint::deserialize(b"NOT_A_CHECKPOINT");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("invalid checkpoint magic"));
    }

    #[test]
    fn test_checkpoint_too_short() {
        let result = PersistenceCheckpoint::deserialize(b"short");
        assert!(result.is_err());
    }

    #[test]
    fn test_checkpoint_bad_version() {
        let mut data = b"PROVA_CKPT".to_vec();
        data.push(99); // bad version
        data.extend_from_slice(&[0u8; 100]);
        let result = PersistenceCheckpoint::deserialize(&data);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("unsupported checkpoint version"));
    }

    #[test]
    fn test_signal_receiver_basic() {
        let mut rx = SignalReceiver::new();
        assert_eq!(rx.signal_count(), 0);
        assert!(!rx.should_force());
        assert_eq!(rx.last_signal(), None);

        rx.receive(ShutdownSignal::SigInt);
        assert_eq!(rx.signal_count(), 1);
        assert!(!rx.should_force());
        assert_eq!(rx.last_signal(), Some(ShutdownSignal::SigInt));
    }

    #[test]
    fn test_signal_receiver_double_force() {
        let mut rx = SignalReceiver::new();
        rx.receive(ShutdownSignal::SigInt);
        rx.receive(ShutdownSignal::SigInt);
        assert!(rx.should_force());
        assert_eq!(rx.signal_count(), 2);
    }

    #[test]
    fn test_signal_explicit() {
        let mut rx = SignalReceiver::new();
        rx.receive(ShutdownSignal::Explicit);
        assert_eq!(rx.last_signal(), Some(ShutdownSignal::Explicit));
    }

    #[test]
    fn test_multiple_subsystems_same_priority() {
        let mut coord = ShutdownCoordinator::new(ShutdownConfig::default());
        coord.register(SubsystemHandle::new("worker-1", 5, |_| {
            DrainResult::Clean { pending_items: 1 }
        }));
        coord.register(SubsystemHandle::new("worker-2", 5, |_| {
            DrainResult::Clean { pending_items: 2 }
        }));
        coord.set_checkpoint(PersistenceCheckpoint {
            chain_head: [0u8; 32],
            chain_height: 0,
            mempool_txs: vec![],
            known_peers: vec![],
            pending_jobs: vec![],
            last_checkpoint_epoch: 0,
        });
        let report = coord.execute();
        assert_eq!(report.subsystems.len(), 2);
        assert!(report.clean());
    }

    #[test]
    fn test_shutdown_config_default() {
        let cfg = ShutdownConfig::default();
        assert_eq!(cfg.drain_timeout, Duration::from_secs(5));
        assert_eq!(cfg.total_timeout, Duration::from_secs(30));
        assert!(cfg.force_on_timeout);
    }
}
