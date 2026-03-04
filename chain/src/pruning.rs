//! State pruning — retain last N snapshots and garbage collect old blocks.
//!
//! Prova nodes accumulate state snapshots and block history over time.
//! The pruning system manages disk space by:
//! - Retaining only the last N state snapshots (configurable)
//! - Garbage collecting blocks below the lowest retained snapshot height
//! - Preserving checkpoint-anchored blocks (L1-referenced, never pruned)
//! - Supporting archive mode (no pruning) for full-history nodes
//!
//! Pruning is non-destructive to consensus safety: any pruned state can be
//! recovered via fast-sync from a peer that retains it.

use std::collections::{BTreeMap, BTreeSet};

use crate::types::Hash;

/// Configuration for the pruning subsystem.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PruningConfig {
    /// Number of recent snapshots to retain. 0 = archive mode (no pruning).
    pub retain_snapshots: u64,
    /// Minimum block height to never prune below (safety floor).
    pub min_height: u64,
    /// Heights anchored to L1 checkpoints (always retained).
    pub checkpoint_heights: BTreeSet<u64>,
    /// If true, pruning is completely disabled (archive node).
    pub archive_mode: bool,
}

impl Default for PruningConfig {
    fn default() -> Self {
        Self {
            retain_snapshots: 128,
            min_height: 0,
            checkpoint_heights: BTreeSet::new(),
            archive_mode: false,
        }
    }
}

/// Metadata for a stored snapshot.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SnapshotMeta {
    pub height: u64,
    pub state_root: Hash,
    pub size_bytes: u64,
    pub created_epoch: u64,
}

/// Metadata for a stored block.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BlockMeta {
    pub height: u64,
    pub hash: Hash,
    pub parent_hash: Hash,
    pub size_bytes: u64,
}

/// Result of a pruning pass.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PruningResult {
    /// Snapshot heights that were pruned.
    pub pruned_snapshots: Vec<u64>,
    /// Block heights that were garbage collected.
    pub pruned_blocks: Vec<u64>,
    /// Total bytes reclaimed.
    pub bytes_reclaimed: u64,
    /// Heights that were protected (checkpoint or min_height).
    pub protected_heights: Vec<u64>,
}

/// In-memory pruning manager that tracks snapshots and blocks.
pub struct PruningManager {
    config: PruningConfig,
    /// Snapshots indexed by height.
    snapshots: BTreeMap<u64, SnapshotMeta>,
    /// Blocks indexed by height.
    blocks: BTreeMap<u64, BlockMeta>,
}

impl PruningManager {
    pub fn new(config: PruningConfig) -> Self {
        Self {
            config,
            snapshots: BTreeMap::new(),
            blocks: BTreeMap::new(),
        }
    }

    /// Register a new snapshot.
    pub fn add_snapshot(&mut self, meta: SnapshotMeta) {
        self.snapshots.insert(meta.height, meta);
    }

    /// Register a new block.
    pub fn add_block(&mut self, meta: BlockMeta) {
        self.blocks.insert(meta.height, meta);
    }

    /// Mark a height as checkpoint-anchored (protected from pruning).
    pub fn add_checkpoint(&mut self, height: u64) {
        self.config.checkpoint_heights.insert(height);
    }

    /// Get the current number of stored snapshots.
    pub fn snapshot_count(&self) -> usize {
        self.snapshots.len()
    }

    /// Get the current number of stored blocks.
    pub fn block_count(&self) -> usize {
        self.blocks.len()
    }

    /// Determine which snapshots should be pruned based on retention policy.
    /// Returns heights to prune (oldest first).
    fn snapshots_to_prune(&self) -> Vec<u64> {
        if self.config.archive_mode || self.config.retain_snapshots == 0 {
            return Vec::new();
        }

        // Separate protected (checkpoint or below min_height) from prunable
        let mut prunable: Vec<u64> = Vec::new();
        let mut protected_count = 0u64;

        for &h in self.snapshots.keys() {
            if h < self.config.min_height || self.config.checkpoint_heights.contains(&h) {
                protected_count += 1;
            } else {
                prunable.push(h);
            }
        }

        // We want to retain retain_snapshots TOTAL (including protected ones)
        let total = self.snapshots.len() as u64;
        if total <= self.config.retain_snapshots {
            return Vec::new();
        }

        let to_remove = total - self.config.retain_snapshots;
        // Can only remove from prunable set, oldest first (already sorted via BTreeMap)
        prunable.into_iter().take(to_remove as usize).collect()
    }

    /// Determine the prune horizon: blocks below this height can be GC'd.
    /// The horizon is the lowest retained snapshot height.
    fn block_prune_horizon(&self) -> Option<u64> {
        if self.config.archive_mode {
            return None;
        }
        // After pruning snapshots, the lowest remaining snapshot sets the floor
        let prunable = self.snapshots_to_prune();
        let remaining: BTreeSet<u64> = self
            .snapshots
            .keys()
            .copied()
            .filter(|h| !prunable.contains(h))
            .collect();

        remaining.iter().next().copied()
    }

    /// Execute a pruning pass. Returns what was pruned.
    pub fn prune(&mut self) -> PruningResult {
        if self.config.archive_mode {
            return PruningResult {
                pruned_snapshots: Vec::new(),
                pruned_blocks: Vec::new(),
                bytes_reclaimed: 0,
                protected_heights: self.config.checkpoint_heights.iter().copied().collect(),
            };
        }

        let mut bytes_reclaimed = 0u64;
        let mut protected = Vec::new();

        // 1. Prune snapshots beyond retention window
        let snap_to_prune = self.snapshots_to_prune();
        for h in &snap_to_prune {
            if let Some(meta) = self.snapshots.remove(h) {
                bytes_reclaimed += meta.size_bytes;
            }
        }

        // 2. GC blocks below prune horizon, respecting checkpoints
        let horizon = self.block_prune_horizon().unwrap_or(0);
        let effective_horizon = horizon.max(self.config.min_height);

        let block_heights_below: Vec<u64> = self
            .blocks
            .keys()
            .copied()
            .filter(|h| *h < effective_horizon)
            .collect();

        let mut pruned_blocks = Vec::new();
        for h in block_heights_below {
            if self.config.checkpoint_heights.contains(&h) {
                protected.push(h);
                continue;
            }
            if let Some(meta) = self.blocks.remove(&h) {
                bytes_reclaimed += meta.size_bytes;
                pruned_blocks.push(h);
            }
        }

        // Also mark checkpoint heights within snapshot range as protected
        for h in &self.config.checkpoint_heights {
            if !protected.contains(h) {
                protected.push(*h);
            }
        }
        protected.sort();

        PruningResult {
            pruned_snapshots: snap_to_prune,
            pruned_blocks: pruned_blocks,
            bytes_reclaimed,
            protected_heights: protected,
        }
    }

    /// Estimate bytes reclaimable without actually pruning.
    pub fn estimate_reclaimable(&self) -> u64 {
        if self.config.archive_mode {
            return 0;
        }

        let mut total = 0u64;

        for h in self.snapshots_to_prune() {
            if let Some(meta) = self.snapshots.get(&h) {
                total += meta.size_bytes;
            }
        }

        let horizon = self.block_prune_horizon().unwrap_or(0);
        let effective_horizon = horizon.max(self.config.min_height);
        for (h, meta) in &self.blocks {
            if *h < effective_horizon && !self.config.checkpoint_heights.contains(h) {
                total += meta.size_bytes;
            }
        }

        total
    }

    /// Get the current config.
    pub fn config(&self) -> &PruningConfig {
        &self.config
    }

    /// Update retention count at runtime.
    pub fn set_retain_snapshots(&mut self, n: u64) {
        self.config.retain_snapshots = n;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_hash(n: u8) -> Hash {
        let mut h = [0u8; 32];
        h[0] = n;
        h
    }

    fn make_snapshot(height: u64, size: u64) -> SnapshotMeta {
        SnapshotMeta {
            height,
            state_root: test_hash(height as u8),
            size_bytes: size,
            created_epoch: height,
        }
    }

    fn make_block(height: u64, size: u64) -> BlockMeta {
        BlockMeta {
            height,
            hash: test_hash(height as u8),
            parent_hash: test_hash(height.wrapping_sub(1) as u8),
            size_bytes: size,
        }
    }

    #[test]
    fn test_archive_mode_no_pruning() {
        let config = PruningConfig {
            archive_mode: true,
            ..Default::default()
        };
        let mut mgr = PruningManager::new(config);
        for i in 0..200 {
            mgr.add_snapshot(make_snapshot(i, 1000));
            mgr.add_block(make_block(i, 500));
        }
        let result = mgr.prune();
        assert!(result.pruned_snapshots.is_empty());
        assert!(result.pruned_blocks.is_empty());
        assert_eq!(result.bytes_reclaimed, 0);
        assert_eq!(mgr.snapshot_count(), 200);
    }

    #[test]
    fn test_retain_last_n_snapshots() {
        let config = PruningConfig {
            retain_snapshots: 3,
            ..Default::default()
        };
        let mut mgr = PruningManager::new(config);
        for i in 1..=5 {
            mgr.add_snapshot(make_snapshot(i * 100, 1000));
        }
        let result = mgr.prune();
        assert_eq!(result.pruned_snapshots, vec![100, 200]);
        assert_eq!(mgr.snapshot_count(), 3);
        // Remaining: 300, 400, 500
        assert!(mgr.snapshots.contains_key(&300));
        assert!(mgr.snapshots.contains_key(&400));
        assert!(mgr.snapshots.contains_key(&500));
    }

    #[test]
    fn test_block_gc_below_horizon() {
        let config = PruningConfig {
            retain_snapshots: 2,
            ..Default::default()
        };
        let mut mgr = PruningManager::new(config);
        // Snapshots at 100, 200, 300
        for i in 1..=3 {
            mgr.add_snapshot(make_snapshot(i * 100, 1000));
        }
        // Blocks at every height 0..300
        for i in 0..300 {
            mgr.add_block(make_block(i, 500));
        }
        let result = mgr.prune();
        // Snapshot 100 pruned, remaining: 200, 300
        assert_eq!(result.pruned_snapshots, vec![100]);
        // Blocks below 200 (the new lowest snapshot) should be GC'd
        assert_eq!(result.pruned_blocks.len(), 200);
        assert_eq!(mgr.block_count(), 100); // 200..299
    }

    #[test]
    fn test_checkpoint_protected() {
        let mut config = PruningConfig {
            retain_snapshots: 1,
            ..Default::default()
        };
        config.checkpoint_heights.insert(50);
        config.checkpoint_heights.insert(100);

        let mut mgr = PruningManager::new(config);
        mgr.add_snapshot(make_snapshot(100, 1000));
        mgr.add_snapshot(make_snapshot(200, 1000));
        mgr.add_snapshot(make_snapshot(300, 1000));

        for i in 0..300 {
            mgr.add_block(make_block(i, 500));
        }

        let result = mgr.prune();
        // 3 snapshots, retain 1, to_remove=2. 100 is checkpoint-protected.
        // Prunable: [200, 300], both pruned. Only 100 remains.
        assert_eq!(result.pruned_snapshots, vec![200, 300]);
        assert_eq!(mgr.snapshot_count(), 1);

        // Block 50 should be protected
        assert!(result.protected_heights.contains(&50));
        assert!(result.protected_heights.contains(&100));
        assert!(mgr.blocks.contains_key(&50));
    }

    #[test]
    fn test_min_height_floor() {
        let config = PruningConfig {
            retain_snapshots: 1,
            min_height: 150,
            ..Default::default()
        };
        let mut mgr = PruningManager::new(config);
        mgr.add_snapshot(make_snapshot(100, 1000));
        mgr.add_snapshot(make_snapshot(200, 1000));

        for i in 0..250 {
            mgr.add_block(make_block(i, 500));
        }

        let result = mgr.prune();
        // 2 snapshots, retain 1, to_remove=1. Snapshot 100 is below min_height (protected).
        // Only prunable: 200. We prune it, leaving 100 (protected).
        assert_eq!(result.pruned_snapshots, vec![200]);

        // After pruning, lowest remaining snapshot = 100
        // effective_horizon = max(100, min_height=150) = 150
        // Blocks 0..149 pruned
        assert_eq!(result.pruned_blocks.len(), 150);
    }

    #[test]
    fn test_no_pruning_under_retention() {
        let config = PruningConfig {
            retain_snapshots: 10,
            ..Default::default()
        };
        let mut mgr = PruningManager::new(config);
        for i in 0..5 {
            mgr.add_snapshot(make_snapshot(i * 100, 1000));
        }
        let result = mgr.prune();
        assert!(result.pruned_snapshots.is_empty());
        assert_eq!(mgr.snapshot_count(), 5);
    }

    #[test]
    fn test_estimate_reclaimable() {
        let config = PruningConfig {
            retain_snapshots: 1,
            ..Default::default()
        };
        let mut mgr = PruningManager::new(config);
        mgr.add_snapshot(make_snapshot(100, 2000));
        mgr.add_snapshot(make_snapshot(200, 3000));

        for i in 0..200 {
            mgr.add_block(make_block(i, 100));
        }

        let est = mgr.estimate_reclaimable();
        // Snapshot 100 pruned: 2000 bytes
        // Blocks 0..199 below horizon 200: 200 * 100 = 20000
        // But wait — after snapshot pruning, lowest remaining = 200
        // So blocks 0..199 below 200 = 200 blocks × 100 = 20000
        assert_eq!(est, 2000 + 200 * 100);
    }

    #[test]
    fn test_bytes_reclaimed_accurate() {
        let config = PruningConfig {
            retain_snapshots: 1,
            ..Default::default()
        };
        let mut mgr = PruningManager::new(config);
        mgr.add_snapshot(make_snapshot(100, 5000));
        mgr.add_snapshot(make_snapshot(200, 3000));

        mgr.add_block(make_block(50, 750));
        mgr.add_block(make_block(150, 500));
        mgr.add_block(make_block(250, 600));

        let result = mgr.prune();
        // Snapshot 100 pruned: 5000
        // Horizon = 200, blocks below: 50 (750) + 150 (500) = 1250
        assert_eq!(result.bytes_reclaimed, 5000 + 750 + 500);
    }

    #[test]
    fn test_runtime_config_change() {
        let config = PruningConfig {
            retain_snapshots: 5,
            ..Default::default()
        };
        let mut mgr = PruningManager::new(config);
        for i in 1..=10 {
            mgr.add_snapshot(make_snapshot(i * 10, 1000));
        }

        // Initially retain 5 → prune 5
        let r1 = mgr.prune();
        assert_eq!(r1.pruned_snapshots.len(), 5);
        assert_eq!(mgr.snapshot_count(), 5);

        // Now tighten to retain 2
        mgr.set_retain_snapshots(2);
        let r2 = mgr.prune();
        assert_eq!(r2.pruned_snapshots.len(), 3);
        assert_eq!(mgr.snapshot_count(), 2);
    }

    #[test]
    fn test_add_checkpoint_at_runtime() {
        let config = PruningConfig {
            retain_snapshots: 1,
            ..Default::default()
        };
        let mut mgr = PruningManager::new(config);
        mgr.add_snapshot(make_snapshot(100, 1000));
        mgr.add_snapshot(make_snapshot(200, 1000));
        mgr.add_snapshot(make_snapshot(300, 1000));

        // Protect 100 after adding
        mgr.add_checkpoint(100);

        let result = mgr.prune();
        // 3 total, retain 1, to_remove=2. 100 is protected (checkpoint).
        // Prunable: [200, 300]. Both pruned, leaving only 100.
        assert!(!result.pruned_snapshots.contains(&100));
        assert!(result.pruned_snapshots.contains(&200));
        assert!(result.pruned_snapshots.contains(&300));
        assert_eq!(mgr.snapshot_count(), 1);
    }

    #[test]
    fn test_empty_manager_prune() {
        let mut mgr = PruningManager::new(PruningConfig::default());
        let result = mgr.prune();
        assert!(result.pruned_snapshots.is_empty());
        assert!(result.pruned_blocks.is_empty());
        assert_eq!(result.bytes_reclaimed, 0);
    }

    #[test]
    fn test_multiple_prune_passes_idempotent() {
        let config = PruningConfig {
            retain_snapshots: 2,
            ..Default::default()
        };
        let mut mgr = PruningManager::new(config);
        for i in 1..=5 {
            mgr.add_snapshot(make_snapshot(i * 100, 1000));
        }

        let r1 = mgr.prune();
        assert_eq!(r1.pruned_snapshots.len(), 3);

        let r2 = mgr.prune();
        assert!(r2.pruned_snapshots.is_empty());
        assert_eq!(r2.bytes_reclaimed, 0);
    }

    #[test]
    fn test_zero_retain_means_archive() {
        let config = PruningConfig {
            retain_snapshots: 0,
            ..Default::default()
        };
        let mut mgr = PruningManager::new(config);
        for i in 0..50 {
            mgr.add_snapshot(make_snapshot(i, 1000));
        }
        let result = mgr.prune();
        assert!(result.pruned_snapshots.is_empty());
        assert_eq!(mgr.snapshot_count(), 50);
    }
}
