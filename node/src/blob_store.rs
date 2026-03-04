//! Blob Storage Backend (NODE-029)
//!
//! Chunked blob storage with garbage collection, disk quotas,
//! and integrity verification for DAS data availability.
//!
//! # Design
//!
//! - Stores erasure-coded chunks keyed by (BlobId, chunk_index)
//! - LRU eviction when disk quota is exceeded (unpinned blobs first)
//! - Periodic garbage collection of expired blobs past retention window
//! - Integrity checks on read (SHA-256 per chunk)
//! - Pin/unpin API for blobs that must survive GC
//! - Statistics tracking for quota enforcement

use prova_chain::das::{BlobId, TOTAL_CHUNKS};
use prova_chain::types::{Epoch, Hash};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, HashMap, HashSet};

// ─── Constants ───────────────────────────────────────────────────────────────

/// Default disk quota: 10 GiB in bytes.
pub const DEFAULT_QUOTA_BYTES: u64 = 10 * 1024 * 1024 * 1024;
/// Maximum single chunk size: 256 KiB.
pub const MAX_CHUNK_SIZE: usize = 256 * 1024;
/// GC batch size (chunks processed per GC pass).
pub const GC_BATCH_SIZE: usize = 1024;
/// Integrity check probability on read (1.0 = always).
pub const INTEGRITY_CHECK_RATE: f64 = 1.0;
/// Minimum free space before triggering eviction (10% of quota).
pub const EVICTION_THRESHOLD_RATIO: f64 = 0.10;

// ─── Types ───────────────────────────────────────────────────────────────────

/// A stored chunk with metadata.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StoredChunk {
    pub blob_id: BlobId,
    pub index: usize,
    pub data: Vec<u8>,
    pub checksum: Hash,
    pub stored_at: Epoch,
}

/// Metadata for a stored blob (aggregate over chunks).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BlobMetadata {
    pub blob_id: BlobId,
    pub total_size: u64,
    pub chunk_count: usize,
    pub chunks_stored: usize,
    pub stored_at: Epoch,
    pub expires_at: Epoch,
    pub pinned: bool,
    pub last_accessed: Epoch,
}

/// Storage statistics.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StoreStats {
    pub total_bytes: u64,
    pub blob_count: usize,
    pub chunk_count: usize,
    pub pinned_blobs: usize,
    pub quota_bytes: u64,
    pub utilization_pct: u8,
}

/// Errors from blob store operations.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StoreError {
    QuotaExceeded { used: u64, quota: u64 },
    BlobNotFound(BlobId),
    ChunkNotFound { blob_id: BlobId, index: usize },
    ChunkTooLarge { size: usize, max: usize },
    IntegrityFailure { blob_id: BlobId, index: usize },
    InvalidIndex { index: usize, max: usize },
    BlobExpired(BlobId),
}

/// Result of a GC pass.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GcResult {
    pub blobs_removed: usize,
    pub chunks_removed: usize,
    pub bytes_freed: u64,
}

/// Result of an eviction pass.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EvictionResult {
    pub blobs_evicted: usize,
    pub bytes_freed: u64,
}

// ─── Blob Store ──────────────────────────────────────────────────────────────

/// In-memory blob storage backend with quota, GC, and integrity checks.
pub struct BlobStore {
    /// Chunk storage: (blob_id, index) → chunk data.
    chunks: HashMap<(BlobId, usize), StoredChunk>,
    /// Blob metadata.
    metadata: HashMap<BlobId, BlobMetadata>,
    /// Access order for LRU eviction (epoch → blobs accessed at that epoch).
    access_order: BTreeMap<Epoch, Vec<BlobId>>,
    /// Pinned blob set.
    pinned: HashSet<BlobId>,
    /// Total stored bytes.
    total_bytes: u64,
    /// Disk quota in bytes.
    quota_bytes: u64,
}

impl BlobStore {
    /// Create a new blob store with the given quota.
    pub fn new(quota_bytes: u64) -> Self {
        Self {
            chunks: HashMap::new(),
            metadata: HashMap::new(),
            access_order: BTreeMap::new(),
            pinned: HashSet::new(),
            total_bytes: 0,
            quota_bytes,
        }
    }

    /// Create with default quota.
    pub fn with_default_quota() -> Self {
        Self::new(DEFAULT_QUOTA_BYTES)
    }

    /// Store a single chunk for a blob.
    pub fn put_chunk(
        &mut self,
        blob_id: BlobId,
        index: usize,
        data: Vec<u8>,
        current_epoch: Epoch,
        retention_epochs: Epoch,
    ) -> Result<(), StoreError> {
        if data.len() > MAX_CHUNK_SIZE {
            return Err(StoreError::ChunkTooLarge {
                size: data.len(),
                max: MAX_CHUNK_SIZE,
            });
        }
        if index >= TOTAL_CHUNKS {
            return Err(StoreError::InvalidIndex {
                index,
                max: TOTAL_CHUNKS,
            });
        }

        let chunk_size = data.len() as u64;

        // Check if replacing existing chunk
        let existing_size = self
            .chunks
            .get(&(blob_id, index))
            .map(|c| c.data.len() as u64)
            .unwrap_or(0);

        let new_total = self.total_bytes - existing_size + chunk_size;
        if new_total > self.quota_bytes {
            return Err(StoreError::QuotaExceeded {
                used: new_total,
                quota: self.quota_bytes,
            });
        }

        let checksum = compute_checksum(&data);
        let chunk = StoredChunk {
            blob_id,
            index,
            data,
            checksum,
            stored_at: current_epoch,
        };

        let is_new_chunk = !self.chunks.contains_key(&(blob_id, index));
        self.chunks.insert((blob_id, index), chunk);
        self.total_bytes = new_total;

        // Update metadata
        let meta = self.metadata.entry(blob_id).or_insert_with(|| BlobMetadata {
            blob_id,
            total_size: 0,
            chunk_count: TOTAL_CHUNKS,
            chunks_stored: 0,
            stored_at: current_epoch,
            expires_at: current_epoch + retention_epochs,
            pinned: false,
            last_accessed: current_epoch,
        });
        meta.total_size = meta.total_size - existing_size + chunk_size;
        if is_new_chunk {
            meta.chunks_stored += 1;
        }
        meta.last_accessed = current_epoch;

        // Update access order
        self.access_order
            .entry(current_epoch)
            .or_default()
            .push(blob_id);

        Ok(())
    }

    /// Retrieve a chunk, verifying integrity.
    pub fn get_chunk(
        &mut self,
        blob_id: BlobId,
        index: usize,
        current_epoch: Epoch,
    ) -> Result<&[u8], StoreError> {
        // Check blob exists and not expired
        if let Some(meta) = self.metadata.get(&blob_id) {
            if current_epoch > meta.expires_at && !meta.pinned {
                return Err(StoreError::BlobExpired(blob_id));
            }
        } else {
            return Err(StoreError::BlobNotFound(blob_id));
        }

        let chunk = self
            .chunks
            .get(&(blob_id, index))
            .ok_or(StoreError::ChunkNotFound { blob_id, index })?;

        // Integrity check
        let actual = compute_checksum(&chunk.data);
        if actual != chunk.checksum {
            return Err(StoreError::IntegrityFailure { blob_id, index });
        }

        // Update access time
        if let Some(meta) = self.metadata.get_mut(&blob_id) {
            meta.last_accessed = current_epoch;
        }

        // Re-borrow to satisfy borrow checker
        Ok(&self.chunks.get(&(blob_id, index)).unwrap().data)
    }

    /// Pin a blob (survives GC and eviction).
    pub fn pin(&mut self, blob_id: BlobId) -> Result<(), StoreError> {
        let meta = self
            .metadata
            .get_mut(&blob_id)
            .ok_or(StoreError::BlobNotFound(blob_id))?;
        meta.pinned = true;
        self.pinned.insert(blob_id);
        Ok(())
    }

    /// Unpin a blob.
    pub fn unpin(&mut self, blob_id: BlobId) -> Result<(), StoreError> {
        let meta = self
            .metadata
            .get_mut(&blob_id)
            .ok_or(StoreError::BlobNotFound(blob_id))?;
        meta.pinned = false;
        self.pinned.remove(&blob_id);
        Ok(())
    }

    /// Run garbage collection: remove expired, unpinned blobs.
    pub fn gc(&mut self, current_epoch: Epoch) -> GcResult {
        let expired: Vec<BlobId> = self
            .metadata
            .values()
            .filter(|m| current_epoch > m.expires_at && !m.pinned)
            .map(|m| m.blob_id)
            .collect();

        let mut result = GcResult {
            blobs_removed: 0,
            chunks_removed: 0,
            bytes_freed: 0,
        };

        for blob_id in expired {
            let freed = self.remove_blob(blob_id);
            result.blobs_removed += 1;
            result.chunks_removed += freed.0;
            result.bytes_freed += freed.1;
        }

        result
    }

    /// Evict LRU unpinned blobs until we're below eviction threshold.
    pub fn evict(&mut self) -> EvictionResult {
        let threshold = self.quota_bytes - (self.quota_bytes as f64 * EVICTION_THRESHOLD_RATIO) as u64;
        let mut result = EvictionResult {
            blobs_evicted: 0,
            bytes_freed: 0,
        };

        if self.total_bytes <= threshold {
            return result;
        }

        // Sort blobs by last_accessed ascending (LRU first), skip pinned
        let mut candidates: Vec<(Epoch, [u8; 32], BlobId)> = self
            .metadata
            .values()
            .filter(|m| !m.pinned)
            .map(|m| (m.last_accessed, m.blob_id.0, m.blob_id))
            .collect();
        candidates.sort_by(|a, b| a.0.cmp(&b.0).then(a.1.cmp(&b.1)));

        for (_, _, blob_id) in candidates {
            if self.total_bytes <= threshold {
                break;
            }
            let freed = self.remove_blob(blob_id);
            result.blobs_evicted += 1;
            result.bytes_freed += freed.1;
        }

        result
    }

    /// Remove all chunks for a blob, returning (chunks_removed, bytes_freed).
    fn remove_blob(&mut self, blob_id: BlobId) -> (usize, u64) {
        let mut chunks_removed = 0;
        let mut bytes_freed = 0u64;

        // Remove all chunks
        let indices: Vec<usize> = (0..TOTAL_CHUNKS)
            .filter(|i| self.chunks.contains_key(&(blob_id, *i)))
            .collect();

        for index in &indices {
            if let Some(chunk) = self.chunks.remove(&(blob_id, *index)) {
                bytes_freed += chunk.data.len() as u64;
                chunks_removed += 1;
            }
        }

        self.total_bytes -= bytes_freed;
        self.metadata.remove(&blob_id);
        self.pinned.remove(&blob_id);

        (chunks_removed, bytes_freed)
    }

    /// Check if a blob is fully stored (all chunks present).
    pub fn is_complete(&self, blob_id: BlobId) -> bool {
        self.metadata
            .get(&blob_id)
            .map(|m| m.chunks_stored == m.chunk_count)
            .unwrap_or(false)
    }

    /// Get blob metadata.
    pub fn get_metadata(&self, blob_id: BlobId) -> Option<&BlobMetadata> {
        self.metadata.get(&blob_id)
    }

    /// Get store statistics.
    pub fn stats(&self) -> StoreStats {
        let utilization = if self.quota_bytes > 0 {
            ((self.total_bytes as f64 / self.quota_bytes as f64) * 100.0) as u8
        } else {
            0
        };

        StoreStats {
            total_bytes: self.total_bytes,
            blob_count: self.metadata.len(),
            chunk_count: self.chunks.len(),
            pinned_blobs: self.pinned.len(),
            quota_bytes: self.quota_bytes,
            utilization_pct: utilization,
        }
    }

    /// List all stored blob IDs.
    pub fn list_blobs(&self) -> Vec<BlobId> {
        self.metadata.keys().copied().collect()
    }

    /// Delete a specific blob (regardless of pin status).
    pub fn delete(&mut self, blob_id: BlobId) -> Result<(usize, u64), StoreError> {
        if !self.metadata.contains_key(&blob_id) {
            return Err(StoreError::BlobNotFound(blob_id));
        }
        Ok(self.remove_blob(blob_id))
    }
}

// ─── Helpers ─────────────────────────────────────────────────────────────────

fn compute_checksum(data: &[u8]) -> Hash {
    let mut hasher = Sha256::new();
    hasher.update(data);
    let result = hasher.finalize();
    let mut hash = [0u8; 32];
    hash.copy_from_slice(&result);
    hash
}

// ─── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn blob_id(n: u8) -> BlobId {
        let mut h = [0u8; 32];
        h[0] = n;
        BlobId(h)
    }

    fn chunk_data(size: usize) -> Vec<u8> {
        vec![0xAB; size]
    }

    #[test]
    fn test_put_and_get_chunk() {
        let mut store = BlobStore::new(1_000_000);
        let bid = blob_id(1);
        store.put_chunk(bid, 0, chunk_data(100), 1, 100).unwrap();
        let data = store.get_chunk(bid, 0, 1).unwrap();
        assert_eq!(data.len(), 100);
    }

    #[test]
    fn test_quota_enforcement() {
        let mut store = BlobStore::new(200);
        let bid = blob_id(1);
        store.put_chunk(bid, 0, chunk_data(100), 1, 100).unwrap();
        let err = store.put_chunk(bid, 1, chunk_data(150), 1, 100).unwrap_err();
        assert!(matches!(err, StoreError::QuotaExceeded { .. }));
    }

    #[test]
    fn test_chunk_too_large() {
        let mut store = BlobStore::new(1_000_000);
        let err = store
            .put_chunk(blob_id(1), 0, chunk_data(MAX_CHUNK_SIZE + 1), 1, 100)
            .unwrap_err();
        assert!(matches!(err, StoreError::ChunkTooLarge { .. }));
    }

    #[test]
    fn test_invalid_index() {
        let mut store = BlobStore::new(1_000_000);
        let err = store
            .put_chunk(blob_id(1), TOTAL_CHUNKS, chunk_data(10), 1, 100)
            .unwrap_err();
        assert!(matches!(err, StoreError::InvalidIndex { .. }));
    }

    #[test]
    fn test_gc_removes_expired() {
        let mut store = BlobStore::new(1_000_000);
        let bid = blob_id(1);
        store.put_chunk(bid, 0, chunk_data(100), 1, 10).unwrap();
        // Epoch 12 > expires_at (1+10=11)
        let result = store.gc(12);
        assert_eq!(result.blobs_removed, 1);
        assert_eq!(result.bytes_freed, 100);
        assert!(store.get_metadata(bid).is_none());
    }

    #[test]
    fn test_gc_preserves_pinned() {
        let mut store = BlobStore::new(1_000_000);
        let bid = blob_id(1);
        store.put_chunk(bid, 0, chunk_data(100), 1, 10).unwrap();
        store.pin(bid).unwrap();
        let result = store.gc(12);
        assert_eq!(result.blobs_removed, 0);
        assert!(store.get_metadata(bid).is_some());
    }

    #[test]
    fn test_pin_unpin() {
        let mut store = BlobStore::new(1_000_000);
        let bid = blob_id(1);
        store.put_chunk(bid, 0, chunk_data(100), 1, 100).unwrap();
        store.pin(bid).unwrap();
        assert!(store.get_metadata(bid).unwrap().pinned);
        store.unpin(bid).unwrap();
        assert!(!store.get_metadata(bid).unwrap().pinned);
    }

    #[test]
    fn test_expired_blob_not_accessible() {
        let mut store = BlobStore::new(1_000_000);
        let bid = blob_id(1);
        store.put_chunk(bid, 0, chunk_data(100), 1, 10).unwrap();
        let err = store.get_chunk(bid, 0, 20).unwrap_err();
        assert!(matches!(err, StoreError::BlobExpired(_)));
    }

    #[test]
    fn test_pinned_blob_accessible_after_expiry() {
        let mut store = BlobStore::new(1_000_000);
        let bid = blob_id(1);
        store.put_chunk(bid, 0, chunk_data(100), 1, 10).unwrap();
        store.pin(bid).unwrap();
        let data = store.get_chunk(bid, 0, 20).unwrap();
        assert_eq!(data.len(), 100);
    }

    #[test]
    fn test_eviction_lru() {
        let mut store = BlobStore::new(400);
        // Store 3 blobs, ~150 bytes each (total 450 > 400, but put one at a time)
        for i in 0..3 {
            // Temporarily raise quota to allow insertion, then shrink
            store.quota_bytes = 1_000_000;
            store
                .put_chunk(blob_id(i), 0, chunk_data(150), (i + 1) as Epoch, 1000)
                .unwrap();
        }
        store.quota_bytes = 400;
        // Access blob 0 recently to make it NOT the LRU
        let _ = store.get_chunk(blob_id(0), 0, 10);
        let result = store.evict();
        // Should evict oldest-accessed first (blob 1 or 2)
        assert!(result.blobs_evicted > 0);
        assert!(store.total_bytes <= 500);
    }

    #[test]
    fn test_eviction_skips_pinned() {
        let mut store = BlobStore::new(300);
        store
            .put_chunk(blob_id(1), 0, chunk_data(200), 1, 1000)
            .unwrap();
        store.pin(blob_id(1)).unwrap();
        store
            .put_chunk(blob_id(2), 0, chunk_data(90), 2, 1000)
            .unwrap();
        // Over threshold but only unpinned can be evicted
        let result = store.evict();
        // blob_id(2) should be evicted, blob_id(1) preserved
        assert!(store.get_metadata(blob_id(1)).is_some());
    }

    #[test]
    fn test_stats() {
        let mut store = BlobStore::new(10_000);
        store
            .put_chunk(blob_id(1), 0, chunk_data(100), 1, 100)
            .unwrap();
        store
            .put_chunk(blob_id(1), 1, chunk_data(200), 1, 100)
            .unwrap();
        store.pin(blob_id(1)).unwrap();
        let stats = store.stats();
        assert_eq!(stats.total_bytes, 300);
        assert_eq!(stats.blob_count, 1);
        assert_eq!(stats.chunk_count, 2);
        assert_eq!(stats.pinned_blobs, 1);
        assert_eq!(stats.utilization_pct, 3);
    }

    #[test]
    fn test_is_complete() {
        let mut store = BlobStore::new(100_000_000);
        let bid = blob_id(1);
        for i in 0..TOTAL_CHUNKS {
            store.put_chunk(bid, i, chunk_data(10), 1, 100).unwrap();
        }
        assert!(store.is_complete(bid));
    }

    #[test]
    fn test_delete() {
        let mut store = BlobStore::new(1_000_000);
        let bid = blob_id(1);
        store.put_chunk(bid, 0, chunk_data(100), 1, 100).unwrap();
        store.pin(bid).unwrap();
        let (chunks, bytes) = store.delete(bid).unwrap();
        assert_eq!(chunks, 1);
        assert_eq!(bytes, 100);
        assert!(store.get_metadata(bid).is_none());
    }

    #[test]
    fn test_replace_chunk() {
        let mut store = BlobStore::new(1_000_000);
        let bid = blob_id(1);
        store.put_chunk(bid, 0, chunk_data(100), 1, 100).unwrap();
        store.put_chunk(bid, 0, chunk_data(200), 2, 100).unwrap();
        let stats = store.stats();
        assert_eq!(stats.total_bytes, 200);
        assert_eq!(stats.chunk_count, 1);
    }

    #[test]
    fn test_integrity_check() {
        let mut store = BlobStore::new(1_000_000);
        let bid = blob_id(1);
        store.put_chunk(bid, 0, chunk_data(100), 1, 100).unwrap();
        // Corrupt the chunk
        store.chunks.get_mut(&(bid, 0)).unwrap().data[0] = 0xFF;
        let err = store.get_chunk(bid, 0, 1).unwrap_err();
        assert!(matches!(err, StoreError::IntegrityFailure { .. }));
    }

    #[test]
    fn test_blob_not_found() {
        let mut store = BlobStore::new(1_000_000);
        let err = store.get_chunk(blob_id(99), 0, 1).unwrap_err();
        assert!(matches!(err, StoreError::BlobNotFound(_)));
    }
}
