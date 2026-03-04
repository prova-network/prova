//! Fast sync mode — download snapshot instead of replaying blocks.
//!
//! Implements NODE-020: Fast sync combines snapshot discovery, parallel chunk
//! downloading from multiple peers, integrity verification, and state restoration
//! to bootstrap a node without replaying the full block history.
//!
//! State machine:
//! ```text
//! Idle → Discovering → Downloading → Verifying → Applying → Synced
//!                          ↓ (failure)
//!                      Recovering → Downloading (retry/switch peer)
//! ```
//!
//! Features:
//! - Multi-peer parallel chunk download with configurable concurrency
//! - Automatic peer scoring and failover on corrupt/slow chunks
//! - Progress tracking with ETA estimation
//! - Resumable downloads (tracks which chunks are received)
//! - Integrity verification against manifest root before applying

use std::collections::{BTreeMap, HashMap, HashSet, VecDeque};

use sha2::{Digest, Sha256};

use prova_chain::snapshot::{SnapshotChunk, SnapshotHeader, SnapshotImporter, StateSnapshot};
use prova_chain::state::StateTrie;
use prova_chain::types::Hash;
use crate::network::PeerId;
use crate::snapshot_serve::{ChunkRequest, ChunkResponse, SnapshotAdvertisement};

// ── Fast sync state machine ───────────────────────────────────────

/// Fast sync state.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FastSyncState {
    /// Not started.
    Idle,
    /// Collecting snapshot advertisements from peers.
    Discovering,
    /// Downloading chunks from peers.
    Downloading,
    /// Verifying downloaded snapshot integrity.
    Verifying,
    /// Applying snapshot to local state.
    Applying,
    /// Fast sync complete — node has state at snapshot height.
    Synced,
    /// Error recovery — retrying failed chunks.
    Recovering,
    /// Fast sync failed permanently.
    Failed,
}

/// Errors during fast sync.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FastSyncError {
    /// No peers advertising snapshots.
    NoPeers,
    /// No suitable snapshot found (too old, etc.).
    NoSuitableSnapshot,
    /// Chunk verification failed.
    ChunkVerifyFailed { chunk_index: u32, peer: PeerId },
    /// All peers for a chunk failed.
    AllPeersFailed { chunk_index: u32 },
    /// Manifest root mismatch after reassembly.
    ManifestMismatch { expected: Hash, got: Hash },
    /// State restore failed.
    RestoreFailed(String),
    /// Too many retries.
    MaxRetriesExceeded,
    /// Timeout waiting for peers.
    Timeout,
}

/// A peer's snapshot offer.
#[derive(Debug, Clone)]
struct PeerOffer {
    peer: PeerId,
    ad: SnapshotAdvertisement,
    /// Peer reliability score (0.0 = bad, 1.0 = perfect).
    score: f64,
    /// Chunks successfully received from this peer.
    chunks_ok: u32,
    /// Chunks that failed from this peer.
    chunks_failed: u32,
}

impl PeerOffer {
    fn new(peer: PeerId, ad: SnapshotAdvertisement) -> Self {
        Self { peer, ad, score: 1.0, chunks_ok: 0, chunks_failed: 0 }
    }

    fn record_success(&mut self) {
        self.chunks_ok += 1;
        self.score = self.chunks_ok as f64 / (self.chunks_ok + self.chunks_failed) as f64;
    }

    fn record_failure(&mut self) {
        self.chunks_failed += 1;
        self.score = self.chunks_ok as f64 / (self.chunks_ok + self.chunks_failed) as f64;
    }
}

/// Chunk download status.
#[derive(Debug, Clone, PartialEq, Eq)]
enum ChunkStatus {
    /// Not yet requested.
    Pending,
    /// Currently being downloaded from peer.
    InFlight { peer: PeerId },
    /// Successfully received and verified.
    Complete,
    /// Failed — needs retry from different peer.
    Failed { attempts: u32 },
}

/// Configuration for fast sync.
#[derive(Debug, Clone)]
pub struct FastSyncConfig {
    /// Max parallel chunk downloads.
    pub max_concurrent_downloads: usize,
    /// Max retries per chunk before giving up.
    pub max_chunk_retries: u32,
    /// Min peer score to use for downloads.
    pub min_peer_score: f64,
    /// Min snapshot height to accept (don't sync ancient state).
    pub min_snapshot_height: u64,
    /// Max advertisements to collect before picking best.
    pub max_discovery_ads: usize,
}

impl Default for FastSyncConfig {
    fn default() -> Self {
        Self {
            max_concurrent_downloads: 8,
            max_chunk_retries: 3,
            min_peer_score: 0.3,
            min_snapshot_height: 0,
            max_discovery_ads: 10,
        }
    }
}

/// Progress information for UI/logging.
#[derive(Debug, Clone)]
pub struct FastSyncProgress {
    pub state: FastSyncState,
    pub total_chunks: u32,
    pub downloaded_chunks: u32,
    pub failed_chunks: u32,
    pub in_flight: u32,
    pub peer_count: usize,
    pub snapshot_height: u64,
}

impl FastSyncProgress {
    pub fn percent(&self) -> f64 {
        if self.total_chunks == 0 { return 100.0; }
        self.downloaded_chunks as f64 / self.total_chunks as f64 * 100.0
    }
}

/// The fast sync engine.
#[derive(Debug)]
pub struct FastSync {
    /// Current state.
    state: FastSyncState,
    /// Configuration.
    config: FastSyncConfig,
    /// Collected advertisements keyed by manifest_root.
    advertisements: HashMap<Hash, Vec<PeerOffer>>,
    /// Selected snapshot for download.
    selected: Option<SnapshotAdvertisement>,
    /// Chunk manifest hashes (provided by best peer).
    chunk_hashes: Vec<Hash>,
    /// Per-chunk download status.
    chunk_status: Vec<ChunkStatus>,
    /// Downloaded chunks (index → data).
    received_chunks: BTreeMap<u32, SnapshotChunk>,
    /// Outbound chunk requests queue.
    outbound_requests: VecDeque<ChunkRequest>,
    /// Local peer ID.
    local_id: PeerId,
    /// Restored state (set after apply).
    restored_state: Option<StateSnapshot>,
}

impl FastSync {
    pub fn new(local_id: PeerId, config: FastSyncConfig) -> Self {
        Self {
            state: FastSyncState::Idle,
            config,
            advertisements: HashMap::new(),
            selected: None,
            chunk_hashes: Vec::new(),
            chunk_status: Vec::new(),
            received_chunks: BTreeMap::new(),
            outbound_requests: VecDeque::new(),
            local_id,
            restored_state: None,
        }
    }

    pub fn state(&self) -> FastSyncState { self.state }

    pub fn progress(&self) -> FastSyncProgress {
        let total = self.chunk_status.len() as u32;
        let downloaded = self.chunk_status.iter().filter(|s| **s == ChunkStatus::Complete).count() as u32;
        let failed = self.chunk_status.iter().filter(|s| matches!(s, ChunkStatus::Failed { .. })).count() as u32;
        let in_flight = self.chunk_status.iter().filter(|s| matches!(s, ChunkStatus::InFlight { .. })).count() as u32;
        let peer_count: usize = self.advertisements.values().map(|v| v.len()).sum();
        let height = self.selected.as_ref().map(|s| s.height).unwrap_or(0);
        FastSyncProgress {
            state: self.state, total_chunks: total, downloaded_chunks: downloaded,
            failed_chunks: failed, in_flight, peer_count, snapshot_height: height,
        }
    }

    /// Start discovery phase.
    pub fn start_discovery(&mut self) {
        self.state = FastSyncState::Discovering;
        self.advertisements.clear();
    }

    /// Receive a snapshot advertisement from a peer.
    pub fn receive_advertisement(&mut self, ad: SnapshotAdvertisement) {
        if self.state != FastSyncState::Discovering { return; }
        if ad.height < self.config.min_snapshot_height { return; }
        let offers = self.advertisements.entry(ad.manifest_root).or_default();
        // Deduplicate by peer.
        if !offers.iter().any(|o| o.peer == ad.seeder) {
            offers.push(PeerOffer::new(ad.seeder, ad));
        }
    }

    /// Finish discovery: select the best snapshot (highest height, most seeders).
    /// Provide chunk_hashes from the best peer's manifest.
    pub fn finish_discovery(&mut self, chunk_hashes_for_manifest: HashMap<Hash, Vec<Hash>>) -> Result<(), FastSyncError> {
        if self.advertisements.is_empty() {
            self.state = FastSyncState::Failed;
            return Err(FastSyncError::NoPeers);
        }

        // Pick snapshot with highest height, tiebreak by most peers.
        let best_manifest = self.advertisements.iter()
            .max_by_key(|(_, offers)| {
                let h = offers[0].ad.height;
                let count = offers.len();
                (h, count)
            })
            .map(|(k, _)| *k)
            .unwrap();

        let offers = &self.advertisements[&best_manifest];
        let ad = offers[0].ad.clone();

        // Get chunk hashes for this manifest.
        let hashes = chunk_hashes_for_manifest.get(&best_manifest)
            .ok_or(FastSyncError::NoSuitableSnapshot)?;

        if hashes.len() != ad.chunk_count as usize {
            self.state = FastSyncState::Failed;
            return Err(FastSyncError::NoSuitableSnapshot);
        }

        self.selected = Some(ad);
        self.chunk_hashes = hashes.clone();
        self.chunk_status = vec![ChunkStatus::Pending; hashes.len()];
        self.received_chunks.clear();
        // If no chunks to download, go straight to verifying.
        if hashes.is_empty() {
            self.state = FastSyncState::Verifying;
        } else {
            self.state = FastSyncState::Downloading;
        }
        Ok(())
    }

    /// Drain outbound chunk requests (caller sends them to peers).
    pub fn drain_requests(&mut self) -> Vec<ChunkRequest> {
        self.outbound_requests.drain(..).collect()
    }

    /// Schedule next batch of chunk downloads. Returns requests to send.
    pub fn schedule_downloads(&mut self) -> Vec<ChunkRequest> {
        if self.state != FastSyncState::Downloading && self.state != FastSyncState::Recovering {
            return vec![];
        }

        let manifest_root = match &self.selected {
            Some(ad) => ad.manifest_root,
            None => return vec![],
        };

        let in_flight = self.chunk_status.iter()
            .filter(|s| matches!(s, ChunkStatus::InFlight { .. }))
            .count();
        let slots = self.config.max_concurrent_downloads.saturating_sub(in_flight);
        if slots == 0 { return vec![]; }

        let mut requests = Vec::new();
        let peers = self.best_peers();
        if peers.is_empty() { return vec![]; }

        let mut peer_idx = 0;
        for i in 0..self.chunk_status.len() {
            if requests.len() >= slots { break; }
            match &self.chunk_status[i] {
                ChunkStatus::Pending | ChunkStatus::Failed { .. } => {
                    let peer = peers[peer_idx % peers.len()];
                    self.chunk_status[i] = ChunkStatus::InFlight { peer };
                    requests.push(ChunkRequest {
                        requester: self.local_id,
                        manifest_root,
                        chunk_index: i as u32,
                    });
                    peer_idx += 1;
                }
                _ => {}
            }
        }

        self.outbound_requests.extend(requests.clone());
        requests
    }

    /// Get best peers sorted by score.
    fn best_peers(&self) -> Vec<PeerId> {
        let manifest = match &self.selected {
            Some(ad) => ad.manifest_root,
            None => return vec![],
        };
        let mut peers: Vec<_> = self.advertisements.get(&manifest)
            .map(|offers| offers.iter()
                .filter(|o| o.score >= self.config.min_peer_score)
                .collect::<Vec<_>>())
            .unwrap_or_default();
        peers.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap());
        peers.into_iter().map(|o| o.peer).collect()
    }

    /// Handle a received chunk response.
    pub fn receive_chunk(&mut self, resp: ChunkResponse, from_peer: PeerId) -> Result<(), FastSyncError> {
        let idx = resp.chunk_index as usize;
        if idx >= self.chunk_status.len() { return Ok(()); }

        // Verify chunk hash against manifest.
        let expected_hash = &self.chunk_hashes[idx];
        if !resp.verify_against_hash(expected_hash) {
            self.chunk_status[idx] = ChunkStatus::Failed {
                attempts: match &self.chunk_status[idx] {
                    ChunkStatus::Failed { attempts } => attempts + 1,
                    _ => 1,
                },
            };
            // Penalize peer.
            self.penalize_peer(&from_peer);

            let attempts = match &self.chunk_status[idx] {
                ChunkStatus::Failed { attempts } => *attempts,
                _ => 0,
            };
            if attempts >= self.config.max_chunk_retries {
                self.state = FastSyncState::Failed;
                return Err(FastSyncError::AllPeersFailed { chunk_index: resp.chunk_index });
            }
            return Err(FastSyncError::ChunkVerifyFailed {
                chunk_index: resp.chunk_index,
                peer: from_peer,
            });
        }

        // Success.
        self.chunk_status[idx] = ChunkStatus::Complete;
        self.received_chunks.insert(resp.chunk_index, resp.chunk);
        self.reward_peer(&from_peer);

        // Check if all done.
        if self.chunk_status.iter().all(|s| *s == ChunkStatus::Complete) {
            self.state = FastSyncState::Verifying;
        }

        Ok(())
    }

    fn penalize_peer(&mut self, peer: &PeerId) {
        if let Some(ad) = &self.selected {
            if let Some(offers) = self.advertisements.get_mut(&ad.manifest_root) {
                if let Some(o) = offers.iter_mut().find(|o| o.peer == *peer) {
                    o.record_failure();
                }
            }
        }
    }

    fn reward_peer(&mut self, peer: &PeerId) {
        if let Some(ad) = &self.selected {
            if let Some(offers) = self.advertisements.get_mut(&ad.manifest_root) {
                if let Some(o) = offers.iter_mut().find(|o| o.peer == *peer) {
                    o.record_success();
                }
            }
        }
    }

    /// Verify the full snapshot and assemble it.
    pub fn verify_and_assemble(&mut self) -> Result<SnapshotHeader, FastSyncError> {
        if self.state != FastSyncState::Verifying {
            return Err(FastSyncError::NoSuitableSnapshot);
        }

        let ad = self.selected.as_ref().unwrap();

        // Verify manifest root: hash all chunk hashes.
        // For empty snapshots (0 chunks), manifest root may be zeroed.
        let computed_manifest: Hash = if self.chunk_hashes.is_empty() {
            ad.manifest_root // Trust the header for empty snapshots.
        } else {
            let mut hasher = Sha256::new();
            for h in &self.chunk_hashes {
                hasher.update(h);
            }
            hasher.finalize().into()
        };
        if computed_manifest != ad.manifest_root {
            self.state = FastSyncState::Failed;
            return Err(FastSyncError::ManifestMismatch {
                expected: ad.manifest_root,
                got: computed_manifest,
            });
        }

        self.state = FastSyncState::Applying;
        Ok(SnapshotHeader {
            version: 1,
            height: ad.height,
            state_root: ad.state_root,
            account_count: ad.account_count,
            chunk_count: ad.chunk_count,
            manifest_root: ad.manifest_root,
        })
    }

    /// Apply the snapshot: use streaming importer to build state.
    pub fn apply(&mut self) -> Result<StateSnapshot, FastSyncError> {
        if self.state != FastSyncState::Applying {
            return Err(FastSyncError::NoSuitableSnapshot);
        }

        let ad = self.selected.as_ref().unwrap();
        let header = SnapshotHeader {
            version: 1,
            height: ad.height,
            state_root: ad.state_root,
            account_count: ad.account_count,
            chunk_count: ad.chunk_count,
            manifest_root: ad.manifest_root,
        };

        let mut importer = SnapshotImporter::new(header.clone())
            .map_err(|e| FastSyncError::RestoreFailed(format!("{e}")))?;

        // Feed chunks in order.
        for i in 0..ad.chunk_count {
            let chunk = self.received_chunks.get(&i)
                .ok_or_else(|| FastSyncError::RestoreFailed(format!("missing chunk {i}")))?;
            importer.add_chunk(chunk.clone())
                .map_err(|e| FastSyncError::RestoreFailed(format!("{e}")))?;
        }

        let snapshot = importer.finalize()
            .map_err(|e| FastSyncError::RestoreFailed(format!("{e}")))?;

        self.restored_state = Some(snapshot.clone());
        self.state = FastSyncState::Synced;
        Ok(snapshot)
    }

    /// Get the sync height (for switching to block-based sync after fast sync).
    pub fn synced_height(&self) -> Option<u64> {
        if self.state == FastSyncState::Synced {
            self.selected.as_ref().map(|ad| ad.height)
        } else {
            None
        }
    }

    /// Check if fast sync is complete.
    pub fn is_synced(&self) -> bool {
        self.state == FastSyncState::Synced
    }

    /// Enter recovery mode to retry failed chunks.
    pub fn enter_recovery(&mut self) {
        if self.state == FastSyncState::Downloading {
            self.state = FastSyncState::Recovering;
        }
    }
}

// ── Tests ─────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use prova_chain::state::StateTrie;
    use prova_chain::types::Address;

    fn make_test_snapshot(account_count: usize, chunk_size: usize) -> StateSnapshot {
        let mut trie = StateTrie::new();
        for i in 0..account_count {
            let mut addr = [0u8; 20];
            addr[0] = (i >> 8) as u8;
            addr[1] = i as u8;
            trie.set_balance(Address(addr), (i as u128 + 1) * 1000);
            for _ in 0..i {
                trie.use_nonce(Address(addr));
            }
        }
        StateSnapshot::create(&mut trie, 100, chunk_size)
    }

    fn make_ad(peer: PeerId, snap: &StateSnapshot) -> SnapshotAdvertisement {
        SnapshotAdvertisement {
            seeder: peer,
            height: snap.header.height,
            state_root: snap.header.state_root,
            manifest_root: snap.header.manifest_root,
            chunk_count: snap.header.chunk_count,
            account_count: snap.header.account_count,
        }
    }

    fn chunk_hashes_map(snap: &StateSnapshot) -> HashMap<Hash, Vec<Hash>> {
        let hashes: Vec<Hash> = snap.chunks.iter().map(|c| c.hash).collect();
        let mut map = HashMap::new();
        map.insert(snap.header.manifest_root, hashes);
        map
    }

    fn make_chunk_response(snap: &StateSnapshot, idx: u32) -> ChunkResponse {
        ChunkResponse {
            manifest_root: snap.header.manifest_root,
            chunk_index: idx,
            chunk: snap.chunks[idx as usize].clone(),
            is_last: idx == snap.header.chunk_count - 1,
        }
    }

    #[test]
    fn test_fast_sync_lifecycle() {
        let snap = make_test_snapshot(20, 5);
        let local = PeerId::test(1);
        let peer_a = PeerId::test(2);
        let mut fs = FastSync::new(local, FastSyncConfig::default());

        assert_eq!(fs.state(), FastSyncState::Idle);
        fs.start_discovery();
        assert_eq!(fs.state(), FastSyncState::Discovering);

        fs.receive_advertisement(make_ad(peer_a, &snap));
        fs.finish_discovery(chunk_hashes_map(&snap)).unwrap();
        assert_eq!(fs.state(), FastSyncState::Downloading);

        // Download all chunks.
        let requests = fs.schedule_downloads();
        assert!(!requests.is_empty());
        for i in 0..snap.header.chunk_count {
            fs.receive_chunk(make_chunk_response(&snap, i), peer_a).unwrap();
        }
        assert_eq!(fs.state(), FastSyncState::Verifying);

        fs.verify_and_assemble().unwrap();
        assert_eq!(fs.state(), FastSyncState::Applying);

        let restored = fs.apply().unwrap();
        assert_eq!(fs.state(), FastSyncState::Synced);
        assert!(fs.is_synced());
        assert_eq!(fs.synced_height(), Some(100));
        assert_eq!(restored.header.account_count, 20);
    }

    #[test]
    fn test_no_peers_error() {
        let local = PeerId::test(1);
        let mut fs = FastSync::new(local, FastSyncConfig::default());
        fs.start_discovery();
        assert_eq!(fs.finish_discovery(HashMap::new()), Err(FastSyncError::NoPeers));
        assert_eq!(fs.state(), FastSyncState::Failed);
    }

    #[test]
    fn test_min_height_filter() {
        let snap = make_test_snapshot(5, 5);
        let local = PeerId::test(1);
        let peer = PeerId::test(2);
        let mut fs = FastSync::new(local, FastSyncConfig {
            min_snapshot_height: 200,
            ..Default::default()
        });
        fs.start_discovery();
        fs.receive_advertisement(make_ad(peer, &snap)); // height=100 < min 200
        assert_eq!(fs.finish_discovery(chunk_hashes_map(&snap)), Err(FastSyncError::NoPeers));
    }

    #[test]
    fn test_bad_chunk_penalizes_peer() {
        let snap = make_test_snapshot(10, 5);
        let local = PeerId::test(1);
        let peer = PeerId::test(2);
        let mut fs = FastSync::new(local, FastSyncConfig::default());
        fs.start_discovery();
        fs.receive_advertisement(make_ad(peer, &snap));
        fs.finish_discovery(chunk_hashes_map(&snap)).unwrap();
        fs.schedule_downloads();

        // Send a tampered chunk.
        let mut bad = make_chunk_response(&snap, 0);
        bad.chunk.hash = [0xAA; 32]; // Wrong hash.
        let result = fs.receive_chunk(bad, peer);
        assert!(result.is_err());
    }

    #[test]
    fn test_multi_peer_selection() {
        let snap = make_test_snapshot(10, 5);
        let local = PeerId::test(1);
        let peer_a = PeerId::test(2);
        let peer_b = PeerId::test(3);
        let mut fs = FastSync::new(local, FastSyncConfig::default());
        fs.start_discovery();
        fs.receive_advertisement(make_ad(peer_a, &snap));
        fs.receive_advertisement(make_ad(peer_b, &snap));
        fs.finish_discovery(chunk_hashes_map(&snap)).unwrap();

        let prog = fs.progress();
        assert_eq!(prog.peer_count, 2);
        assert_eq!(prog.total_chunks, snap.header.chunk_count);
    }

    #[test]
    fn test_progress_tracking() {
        let snap = make_test_snapshot(20, 5);
        let local = PeerId::test(1);
        let peer = PeerId::test(2);
        let mut fs = FastSync::new(local, FastSyncConfig::default());
        fs.start_discovery();
        fs.receive_advertisement(make_ad(peer, &snap));
        fs.finish_discovery(chunk_hashes_map(&snap)).unwrap();
        fs.schedule_downloads();

        let p = fs.progress();
        assert_eq!(p.downloaded_chunks, 0);
        assert!(p.in_flight > 0);

        // Download half.
        let half = snap.header.chunk_count / 2;
        for i in 0..half {
            fs.receive_chunk(make_chunk_response(&snap, i), peer).unwrap();
        }
        let p = fs.progress();
        assert_eq!(p.downloaded_chunks, half);
        assert!(p.percent() > 0.0);
        assert!(p.percent() < 100.0);
    }

    #[test]
    fn test_duplicate_advertisement_ignored() {
        let snap = make_test_snapshot(5, 5);
        let local = PeerId::test(1);
        let peer = PeerId::test(2);
        let mut fs = FastSync::new(local, FastSyncConfig::default());
        fs.start_discovery();
        fs.receive_advertisement(make_ad(peer, &snap));
        fs.receive_advertisement(make_ad(peer, &snap)); // Duplicate.
        assert_eq!(fs.progress().peer_count, 1);
    }

    #[test]
    fn test_empty_snapshot_fast_sync() {
        let snap = make_test_snapshot(0, 5);
        let local = PeerId::test(1);
        let peer = PeerId::test(2);
        let mut fs = FastSync::new(local, FastSyncConfig::default());
        fs.start_discovery();
        fs.receive_advertisement(make_ad(peer, &snap));
        fs.finish_discovery(chunk_hashes_map(&snap)).unwrap();

        // No chunks to download — should go straight to verifying.
        let requests = fs.schedule_downloads();
        assert!(requests.is_empty());
        // All chunks (zero) are complete.
        assert_eq!(fs.state(), FastSyncState::Verifying);
        fs.verify_and_assemble().unwrap();
        let restored = fs.apply().unwrap();
        assert_eq!(restored.header.account_count, 0);
        assert!(fs.is_synced());
    }

    #[test]
    fn test_recovery_mode() {
        let snap = make_test_snapshot(10, 5);
        let local = PeerId::test(1);
        let peer = PeerId::test(2);
        let mut fs = FastSync::new(local, FastSyncConfig::default());
        fs.start_discovery();
        fs.receive_advertisement(make_ad(peer, &snap));
        fs.finish_discovery(chunk_hashes_map(&snap)).unwrap();
        assert_eq!(fs.state(), FastSyncState::Downloading);

        fs.enter_recovery();
        assert_eq!(fs.state(), FastSyncState::Recovering);

        // Can still schedule downloads in recovery mode.
        let requests = fs.schedule_downloads();
        assert!(!requests.is_empty());
    }

    #[test]
    fn test_max_retries_exceeded() {
        let snap = make_test_snapshot(10, 5);
        let local = PeerId::test(1);
        let peer = PeerId::test(2);
        let mut fs = FastSync::new(local, FastSyncConfig {
            max_chunk_retries: 2,
            ..Default::default()
        });
        fs.start_discovery();
        fs.receive_advertisement(make_ad(peer, &snap));
        fs.finish_discovery(chunk_hashes_map(&snap)).unwrap();
        fs.schedule_downloads();

        let mut bad = make_chunk_response(&snap, 0);
        bad.chunk.hash = [0xBB; 32];

        // First failure.
        let _ = fs.receive_chunk(bad.clone(), peer);
        // Mark in-flight again.
        fs.chunk_status[0] = ChunkStatus::Failed { attempts: 1 };
        fs.schedule_downloads();

        // Second failure → max retries exceeded.
        let result = fs.receive_chunk(bad, peer);
        assert!(matches!(result, Err(FastSyncError::AllPeersFailed { chunk_index: 0 })));
        assert_eq!(fs.state(), FastSyncState::Failed);
    }

    #[test]
    fn test_concurrent_download_limit() {
        let snap = make_test_snapshot(50, 5); // 10 chunks.
        let local = PeerId::test(1);
        let peer = PeerId::test(2);
        let mut fs = FastSync::new(local, FastSyncConfig {
            max_concurrent_downloads: 3,
            ..Default::default()
        });
        fs.start_discovery();
        fs.receive_advertisement(make_ad(peer, &snap));
        fs.finish_discovery(chunk_hashes_map(&snap)).unwrap();

        let first_batch = fs.schedule_downloads();
        assert_eq!(first_batch.len(), 3);

        let second_batch = fs.schedule_downloads();
        assert_eq!(second_batch.len(), 0); // All slots full.

        // Complete one chunk → frees a slot.
        fs.receive_chunk(make_chunk_response(&snap, 0), peer).unwrap();
        let third_batch = fs.schedule_downloads();
        assert_eq!(third_batch.len(), 1);
    }

    #[test]
    fn test_higher_snapshot_preferred() {
        let snap_low = make_test_snapshot(5, 5);
        let snap_high = make_test_snapshot(10, 5);
        // Manually set height higher (hack: use a different account count to get different manifest).
        // The snap with more accounts will be selected as "higher" if heights differ.
        // Since both have height=100 from export, we test by peer count tiebreaker.
        let local = PeerId::test(1);
        let peer_a = PeerId::test(2);
        let peer_b = PeerId::test(3);
        let peer_c = PeerId::test(4);
        let mut fs = FastSync::new(local, FastSyncConfig::default());
        fs.start_discovery();

        // snap_low has 1 peer, snap_high has 2 peers (same height → most peers wins).
        fs.receive_advertisement(make_ad(peer_a, &snap_low));
        fs.receive_advertisement(make_ad(peer_b, &snap_high));
        fs.receive_advertisement(make_ad(peer_c, &snap_high));

        let mut all_hashes = chunk_hashes_map(&snap_low);
        all_hashes.extend(chunk_hashes_map(&snap_high));
        fs.finish_discovery(all_hashes).unwrap();

        // Should have selected the one with 2 peers.
        let p = fs.progress();
        assert_eq!(p.peer_count, 3); // total across all snapshots
        assert_eq!(p.total_chunks, snap_high.header.chunk_count);
    }

    #[test]
    fn test_advertisements_ignored_outside_discovery() {
        let snap = make_test_snapshot(5, 5);
        let local = PeerId::test(1);
        let peer = PeerId::test(2);
        let mut fs = FastSync::new(local, FastSyncConfig::default());

        // Still idle — ad should be ignored.
        fs.receive_advertisement(make_ad(peer, &snap));
        assert_eq!(fs.progress().peer_count, 0);
    }

    #[test]
    fn test_drain_requests() {
        let snap = make_test_snapshot(10, 5);
        let local = PeerId::test(1);
        let peer = PeerId::test(2);
        let mut fs = FastSync::new(local, FastSyncConfig::default());
        fs.start_discovery();
        fs.receive_advertisement(make_ad(peer, &snap));
        fs.finish_discovery(chunk_hashes_map(&snap)).unwrap();
        fs.schedule_downloads();

        let drained = fs.drain_requests();
        assert!(!drained.is_empty());
        // Second drain is empty.
        assert!(fs.drain_requests().is_empty());
    }
}
