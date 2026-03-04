//! Snapshot serving over P2P — advertise snapshots and serve chunks on request.
//!
//! Implements NODE-019: nodes that have completed a state snapshot can:
//! - Advertise available snapshots to peers via gossip
//! - Handle chunk requests from syncing peers
//! - Rate-limit serving to prevent bandwidth abuse
//! - Verify chunk integrity before serving
//! - Track download progress for requesters
//!
//! Protocol flow:
//! ```text
//! 1. Seeder creates snapshot → registers in SnapshotServer
//! 2. Seeder gossips SnapshotAdvertisement to peers
//! 3. Syncing node sends ChunkRequest for specific chunk indices
//! 4. Seeder responds with ChunkResponse (data + proof)
//! 5. Syncing node verifies chunk hash against manifest
//! 6. Repeat until all chunks downloaded
//! ```

use std::collections::{HashMap, HashSet, VecDeque};

use sha2::{Digest, Sha256};

use prova_chain::snapshot::{SnapshotChunk, SnapshotHeader, StateSnapshot};
use prova_chain::types::Hash;
use crate::network::PeerId;

// ── Wire protocol messages ────────────────────────────────────────

/// Advertisement of an available snapshot.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SnapshotAdvertisement {
    /// Peer offering the snapshot.
    pub seeder: PeerId,
    /// Height at which snapshot was taken.
    pub height: u64,
    /// State root of the snapshot.
    pub state_root: Hash,
    /// Manifest root (hash of all chunk hashes).
    pub manifest_root: Hash,
    /// Number of chunks.
    pub chunk_count: u32,
    /// Total account count.
    pub account_count: u64,
}

impl SnapshotAdvertisement {
    pub fn from_header(seeder: PeerId, header: &SnapshotHeader) -> Self {
        Self {
            seeder,
            height: header.height,
            state_root: header.state_root,
            manifest_root: header.manifest_root,
            chunk_count: header.chunk_count,
            account_count: header.account_count,
        }
    }

    /// Deterministic ID for dedup.
    pub fn id(&self) -> Hash {
        let mut hasher = Sha256::new();
        hasher.update(self.seeder.0);
        hasher.update(self.height.to_be_bytes());
        hasher.update(self.state_root);
        hasher.update(self.manifest_root);
        hasher.finalize().into()
    }
}

/// Request for a specific chunk.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChunkRequest {
    pub requester: PeerId,
    pub manifest_root: Hash,
    pub chunk_index: u32,
}

/// Response carrying chunk data.
#[derive(Debug, Clone)]
pub struct ChunkResponse {
    pub manifest_root: Hash,
    pub chunk_index: u32,
    pub chunk: SnapshotChunk,
    /// Whether this is the last chunk.
    pub is_last: bool,
}

impl ChunkResponse {
    /// Verify chunk hash matches expected hash.
    pub fn verify_against_hash(&self, expected_hash: &Hash) -> bool {
        self.chunk.verify() && &self.chunk.hash == expected_hash
    }
}

/// Errors from snapshot serving.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ServeError {
    /// Requested snapshot not found.
    SnapshotNotFound,
    /// Chunk index out of range.
    ChunkOutOfRange { requested: u32, total: u32 },
    /// Peer is rate-limited.
    RateLimited { peer: PeerId, retry_after_ms: u64 },
    /// Snapshot data corrupted.
    IntegrityError,
    /// Server at capacity.
    AtCapacity,
}

// ── Rate limiter ──────────────────────────────────────────────────

/// Simple token-bucket rate limiter per peer.
#[derive(Debug, Clone)]
pub struct RateLimiter {
    /// Max chunks per window per peer.
    pub max_chunks_per_window: u32,
    /// Window duration in simulated ms.
    pub window_ms: u64,
    /// Peer → (chunks_served, window_start).
    buckets: HashMap<PeerId, (u32, u64)>,
}

impl RateLimiter {
    pub fn new(max_chunks_per_window: u32, window_ms: u64) -> Self {
        Self {
            max_chunks_per_window,
            window_ms,
            buckets: HashMap::new(),
        }
    }

    /// Check if peer can request, and if so consume a token. Returns Ok or retry time.
    pub fn check_and_consume(&mut self, peer: &PeerId, now_ms: u64) -> Result<(), u64> {
        let entry = self.buckets.entry(*peer).or_insert((0, now_ms));
        // Reset window if expired.
        if now_ms >= entry.1 + self.window_ms {
            entry.0 = 0;
            entry.1 = now_ms;
        }
        if entry.0 >= self.max_chunks_per_window {
            let retry_after = (entry.1 + self.window_ms).saturating_sub(now_ms);
            Err(retry_after)
        } else {
            entry.0 += 1;
            Ok(())
        }
    }

    /// Reset a peer's bucket.
    pub fn reset_peer(&mut self, peer: &PeerId) {
        self.buckets.remove(peer);
    }
}

// ── Snapshot server (seeder side) ─────────────────────────────────

/// Metadata for a registered snapshot (header + chunk data).
#[derive(Debug)]
struct RegisteredSnapshot {
    header: SnapshotHeader,
    chunks: Vec<SnapshotChunk>,
    /// Chunk hashes for manifest verification by downloaders.
    chunk_hashes: Vec<Hash>,
}

impl RegisteredSnapshot {
    fn from_state_snapshot(snap: StateSnapshot) -> Self {
        let chunk_hashes: Vec<Hash> = snap.chunks.iter().map(|c| c.hash).collect();
        Self {
            header: snap.header,
            chunks: snap.chunks,
            chunk_hashes,
        }
    }
}

/// Serves snapshots to requesting peers.
#[derive(Debug)]
pub struct SnapshotServer {
    /// Local peer ID.
    pub local_id: PeerId,
    /// Available snapshots keyed by manifest_root.
    snapshots: HashMap<Hash, RegisteredSnapshot>,
    /// Rate limiter.
    rate_limiter: RateLimiter,
    /// Outbound advertisements queue.
    outbound_ads: VecDeque<SnapshotAdvertisement>,
    /// Max concurrent downloads (peers being served).
    max_concurrent: usize,
    /// Currently serving peers (manifest_root → set of peers).
    active_downloads: HashMap<Hash, HashSet<PeerId>>,
    /// Total chunks served (stats).
    pub chunks_served: u64,
}

impl SnapshotServer {
    pub fn new(local_id: PeerId, max_concurrent: usize) -> Self {
        Self {
            local_id,
            snapshots: HashMap::new(),
            rate_limiter: RateLimiter::new(50, 10_000), // 50 chunks per 10s
            outbound_ads: VecDeque::new(),
            max_concurrent,
            active_downloads: HashMap::new(),
            chunks_served: 0,
        }
    }

    /// Register a snapshot for serving. Queues an advertisement.
    pub fn register_snapshot(&mut self, snapshot: StateSnapshot) {
        let ad = SnapshotAdvertisement::from_header(self.local_id, &snapshot.header);
        let key = snapshot.header.manifest_root;
        self.snapshots.insert(key, RegisteredSnapshot::from_state_snapshot(snapshot));
        self.outbound_ads.push_back(ad);
    }

    /// Drain queued advertisements.
    pub fn drain_advertisements(&mut self) -> Vec<SnapshotAdvertisement> {
        self.outbound_ads.drain(..).collect()
    }

    /// Get chunk hashes for a snapshot (so downloaders can verify chunks).
    pub fn chunk_hashes(&self, manifest_root: &Hash) -> Option<&[Hash]> {
        self.snapshots.get(manifest_root).map(|s| s.chunk_hashes.as_slice())
    }

    /// Handle an incoming chunk request. Returns response or error.
    pub fn handle_request(
        &mut self,
        req: &ChunkRequest,
        now_ms: u64,
    ) -> Result<ChunkResponse, ServeError> {
        // Rate limit check.
        if let Err(retry_after) = self.rate_limiter.check_and_consume(&req.requester, now_ms) {
            return Err(ServeError::RateLimited {
                peer: req.requester,
                retry_after_ms: retry_after,
            });
        }

        // Find snapshot.
        let snapshot = self
            .snapshots
            .get(&req.manifest_root)
            .ok_or(ServeError::SnapshotNotFound)?;

        // Bounds check.
        let total = snapshot.chunks.len() as u32;
        if req.chunk_index >= total {
            return Err(ServeError::ChunkOutOfRange {
                requested: req.chunk_index,
                total,
            });
        }

        // Capacity check.
        let total_active: usize = self.active_downloads.values().map(|s| s.len()).sum();
        if total_active >= self.max_concurrent {
            let already_active = self
                .active_downloads
                .get(&req.manifest_root)
                .map_or(false, |s| s.contains(&req.requester));
            if !already_active {
                return Err(ServeError::AtCapacity);
            }
        }

        // Serve chunk.
        let chunk = &snapshot.chunks[req.chunk_index as usize];
        if !chunk.verify() {
            return Err(ServeError::IntegrityError);
        }

        // Track active download.
        self.active_downloads
            .entry(req.manifest_root)
            .or_default()
            .insert(req.requester);

        self.chunks_served += 1;

        Ok(ChunkResponse {
            manifest_root: req.manifest_root,
            chunk_index: req.chunk_index,
            chunk: chunk.clone(),
            is_last: req.chunk_index == total - 1,
        })
    }

    /// Remove a peer from active downloads.
    pub fn peer_finished(&mut self, manifest_root: &Hash, peer: &PeerId) {
        if let Some(peers) = self.active_downloads.get_mut(manifest_root) {
            peers.remove(peer);
            if peers.is_empty() {
                self.active_downloads.remove(manifest_root);
            }
        }
        self.rate_limiter.reset_peer(peer);
    }

    /// Number of snapshots available.
    pub fn snapshot_count(&self) -> usize {
        self.snapshots.len()
    }

    /// Get available snapshot heights.
    pub fn available_heights(&self) -> Vec<u64> {
        self.snapshots.values().map(|s| s.header.height).collect()
    }
}

// ── Snapshot downloader (syncing side) ────────────────────────────

/// Tracks download progress for a single snapshot.
#[derive(Debug)]
pub struct SnapshotDownload {
    /// The advertisement we're downloading from.
    pub ad: SnapshotAdvertisement,
    /// Expected chunk hashes (from seeder, verified via manifest_root).
    pub expected_hashes: Vec<Hash>,
    /// Downloaded chunks (index → chunk).
    pub downloaded: HashMap<u32, SnapshotChunk>,
    /// Pending request indices.
    pub pending: VecDeque<u32>,
    /// Failed chunks (to retry).
    pub failed: Vec<u32>,
    /// In-flight requests.
    pub in_flight: HashSet<u32>,
    /// Max parallel chunk requests.
    pub max_parallel: usize,
}

impl SnapshotDownload {
    pub fn new(ad: SnapshotAdvertisement, chunk_hashes: Vec<Hash>, max_parallel: usize) -> Self {
        let pending: VecDeque<u32> = (0..ad.chunk_count).collect();
        Self {
            ad,
            expected_hashes: chunk_hashes,
            downloaded: HashMap::new(),
            pending,
            failed: Vec::new(),
            in_flight: HashSet::new(),
            max_parallel,
        }
    }

    /// Verify that the chunk hashes we received match the advertised manifest root.
    pub fn verify_manifest(&self) -> bool {
        let mut hasher = Sha256::new();
        for h in &self.expected_hashes {
            hasher.update(h);
        }
        let computed: Hash = hasher.finalize().into();
        computed == self.ad.manifest_root
    }

    /// Get next batch of chunk indices to request.
    pub fn next_requests(&mut self) -> Vec<u32> {
        let mut batch = Vec::new();
        while self.in_flight.len() < self.max_parallel {
            if let Some(idx) = self.pending.pop_front() {
                self.in_flight.insert(idx);
                batch.push(idx);
            } else {
                break;
            }
        }
        batch
    }

    /// Process a received chunk response.
    pub fn receive_chunk(&mut self, resp: ChunkResponse) -> Result<(), ServeError> {
        let idx = resp.chunk_index;
        self.in_flight.remove(&idx);

        if idx as usize >= self.expected_hashes.len() {
            return Err(ServeError::ChunkOutOfRange {
                requested: idx,
                total: self.expected_hashes.len() as u32,
            });
        }

        if !resp.verify_against_hash(&self.expected_hashes[idx as usize]) {
            self.failed.push(idx);
            return Err(ServeError::IntegrityError);
        }

        self.downloaded.insert(idx, resp.chunk);
        Ok(())
    }

    /// Re-queue failed chunks for retry.
    pub fn retry_failed(&mut self) {
        for idx in self.failed.drain(..) {
            self.pending.push_back(idx);
        }
    }

    /// Progress as fraction (0.0 → 1.0).
    pub fn progress(&self) -> f64 {
        if self.ad.chunk_count == 0 {
            return 1.0;
        }
        self.downloaded.len() as f64 / self.ad.chunk_count as f64
    }

    /// Whether all chunks have been downloaded and verified.
    pub fn is_complete(&self) -> bool {
        self.downloaded.len() == self.ad.chunk_count as usize
    }

    /// Assemble downloaded chunks into ordered Vec (only when complete).
    pub fn assemble(self) -> Option<Vec<SnapshotChunk>> {
        if !self.is_complete() {
            return None;
        }
        let mut chunks: Vec<_> = self.downloaded.into_iter().collect();
        chunks.sort_by_key(|(idx, _)| *idx);
        Some(chunks.into_iter().map(|(_, c)| c).collect())
    }
}

// ── Tests ─────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use prova_chain::snapshot::{SnapshotAccount, SnapshotChunk, SnapshotHeader};
    use prova_chain::types::Address;
    use std::collections::BTreeMap;

    fn test_address(id: u8) -> Address {
        let mut bytes = [0u8; 20];
        bytes[0] = id;
        Address(bytes)
    }

    fn test_account(id: u8, balance: u128) -> SnapshotAccount {
        SnapshotAccount {
            address: test_address(id),
            balance,
            nonce: id as u64,
            code_hash: None,
            storage: BTreeMap::new(),
        }
    }

    fn make_chunk(index: u32, accounts: Vec<SnapshotAccount>) -> SnapshotChunk {
        let hash = SnapshotChunk::compute_hash(&accounts);
        SnapshotChunk { index, accounts, hash }
    }

    /// Build a StateSnapshot manually for testing.
    fn make_test_snapshot(num_chunks: u32, accounts_per_chunk: usize) -> StateSnapshot {
        let mut chunks = Vec::new();
        let mut total_accounts = 0u64;
        for i in 0..num_chunks {
            let accounts: Vec<_> = (0..accounts_per_chunk)
                .map(|j| test_account((i * accounts_per_chunk as u32 + j as u32) as u8, 1000 + j as u128))
                .collect();
            total_accounts += accounts.len() as u64;
            chunks.push(make_chunk(i, accounts));
        }
        // Compute manifest root same way as StateSnapshot::compute_manifest_root.
        let mut hasher = Sha256::new();
        for c in &chunks {
            hasher.update(&c.hash);
        }
        let manifest_root: Hash = hasher.finalize().into();

        let header = SnapshotHeader {
            version: 1,
            height: 100,
            state_root: [0xAA; 32],
            account_count: total_accounts,
            chunk_count: num_chunks,
            manifest_root,
        };
        StateSnapshot { header, chunks }
    }

    #[test]
    fn test_advertisement_from_header() {
        let snap = make_test_snapshot(3, 2);
        let peer = PeerId::test(1);
        let ad = SnapshotAdvertisement::from_header(peer, &snap.header);
        assert_eq!(ad.height, 100);
        assert_eq!(ad.chunk_count, 3);
        assert_eq!(ad.account_count, 6);
        assert_eq!(ad.state_root, [0xAA; 32]);
    }

    #[test]
    fn test_advertisement_id_deterministic() {
        let snap = make_test_snapshot(2, 1);
        let peer = PeerId::test(5);
        let ad = SnapshotAdvertisement::from_header(peer, &snap.header);
        let id1 = ad.id();
        let id2 = ad.id();
        assert_eq!(id1, id2);
    }

    #[test]
    fn test_rate_limiter_basic() {
        let mut rl = RateLimiter::new(3, 1000);
        let peer = PeerId::test(1);
        assert!(rl.check_and_consume(&peer, 0).is_ok());
        assert!(rl.check_and_consume(&peer, 100).is_ok());
        assert!(rl.check_and_consume(&peer, 200).is_ok());
        assert!(rl.check_and_consume(&peer, 300).is_err());
        // After window resets.
        assert!(rl.check_and_consume(&peer, 1000).is_ok());
    }

    #[test]
    fn test_rate_limiter_per_peer() {
        let mut rl = RateLimiter::new(1, 1000);
        let p1 = PeerId::test(1);
        let p2 = PeerId::test(2);
        assert!(rl.check_and_consume(&p1, 0).is_ok());
        assert!(rl.check_and_consume(&p1, 0).is_err());
        assert!(rl.check_and_consume(&p2, 0).is_ok());
    }

    #[test]
    fn test_server_register_and_advertise() {
        let snap = make_test_snapshot(2, 3);
        let mut server = SnapshotServer::new(PeerId::test(1), 10);
        server.register_snapshot(snap);
        let ads = server.drain_advertisements();
        assert_eq!(ads.len(), 1);
        assert_eq!(ads[0].chunk_count, 2);
        assert_eq!(server.snapshot_count(), 1);
    }

    #[test]
    fn test_server_handle_request_success() {
        let snap = make_test_snapshot(3, 2);
        let manifest_root = snap.header.manifest_root;
        let mut server = SnapshotServer::new(PeerId::test(1), 10);
        server.register_snapshot(snap);

        let req = ChunkRequest {
            requester: PeerId::test(2),
            manifest_root,
            chunk_index: 0,
        };
        let resp = server.handle_request(&req, 0).unwrap();
        assert_eq!(resp.chunk_index, 0);
        assert!(!resp.is_last);
        assert_eq!(server.chunks_served, 1);
    }

    #[test]
    fn test_server_handle_last_chunk() {
        let snap = make_test_snapshot(3, 2);
        let manifest_root = snap.header.manifest_root;
        let mut server = SnapshotServer::new(PeerId::test(1), 10);
        server.register_snapshot(snap);

        let req = ChunkRequest {
            requester: PeerId::test(2),
            manifest_root,
            chunk_index: 2,
        };
        let resp = server.handle_request(&req, 0).unwrap();
        assert!(resp.is_last);
    }

    #[test]
    fn test_server_chunk_out_of_range() {
        let snap = make_test_snapshot(2, 1);
        let manifest_root = snap.header.manifest_root;
        let mut server = SnapshotServer::new(PeerId::test(1), 10);
        server.register_snapshot(snap);

        let req = ChunkRequest {
            requester: PeerId::test(2),
            manifest_root,
            chunk_index: 5,
        };
        let err = server.handle_request(&req, 0).unwrap_err();
        assert_eq!(err, ServeError::ChunkOutOfRange { requested: 5, total: 2 });
    }

    #[test]
    fn test_server_snapshot_not_found() {
        let mut server = SnapshotServer::new(PeerId::test(1), 10);
        let req = ChunkRequest {
            requester: PeerId::test(2),
            manifest_root: [0xFF; 32],
            chunk_index: 0,
        };
        assert_eq!(server.handle_request(&req, 0).unwrap_err(), ServeError::SnapshotNotFound);
    }

    #[test]
    fn test_server_rate_limiting() {
        let snap = make_test_snapshot(2, 1);
        let manifest_root = snap.header.manifest_root;
        let mut server = SnapshotServer::new(PeerId::test(1), 100);
        server.rate_limiter = RateLimiter::new(2, 10_000);
        server.register_snapshot(snap);

        let peer = PeerId::test(2);
        let req = |idx| ChunkRequest { requester: peer, manifest_root, chunk_index: idx };
        assert!(server.handle_request(&req(0), 0).is_ok());
        assert!(server.handle_request(&req(1), 0).is_ok());
        let err = server.handle_request(&req(0), 0).unwrap_err();
        assert!(matches!(err, ServeError::RateLimited { .. }));
    }

    #[test]
    fn test_server_capacity_limit() {
        let snap = make_test_snapshot(2, 1);
        let manifest_root = snap.header.manifest_root;
        let mut server = SnapshotServer::new(PeerId::test(1), 1);
        server.register_snapshot(snap);

        let req1 = ChunkRequest { requester: PeerId::test(2), manifest_root, chunk_index: 0 };
        assert!(server.handle_request(&req1, 0).is_ok());

        let req2 = ChunkRequest { requester: PeerId::test(3), manifest_root, chunk_index: 0 };
        assert_eq!(server.handle_request(&req2, 100).unwrap_err(), ServeError::AtCapacity);

        // Existing peer can still continue.
        let req1b = ChunkRequest { requester: PeerId::test(2), manifest_root, chunk_index: 1 };
        assert!(server.handle_request(&req1b, 200).is_ok());
    }

    #[test]
    fn test_server_peer_finished_frees_slot() {
        let snap = make_test_snapshot(2, 1);
        let manifest_root = snap.header.manifest_root;
        let mut server = SnapshotServer::new(PeerId::test(1), 1);
        server.register_snapshot(snap);

        let peer = PeerId::test(2);
        let req = ChunkRequest { requester: peer, manifest_root, chunk_index: 0 };
        server.handle_request(&req, 0).unwrap();
        server.peer_finished(&manifest_root, &peer);

        let peer3 = PeerId::test(3);
        let req3 = ChunkRequest { requester: peer3, manifest_root, chunk_index: 0 };
        assert!(server.handle_request(&req3, 100).is_ok());
    }

    #[test]
    fn test_download_progress() {
        let snap = make_test_snapshot(4, 2);
        let chunk_hashes: Vec<Hash> = snap.chunks.iter().map(|c| c.hash).collect();
        let ad = SnapshotAdvertisement::from_header(PeerId::test(1), &snap.header);
        let mut dl = SnapshotDownload::new(ad, chunk_hashes, 2);
        assert_eq!(dl.progress(), 0.0);
        assert!(!dl.is_complete());

        let batch = dl.next_requests();
        assert_eq!(batch.len(), 2);
        assert_eq!(batch, vec![0, 1]);

        let resp0 = ChunkResponse {
            manifest_root: snap.header.manifest_root,
            chunk_index: 0,
            chunk: snap.chunks[0].clone(),
            is_last: false,
        };
        dl.receive_chunk(resp0).unwrap();
        assert_eq!(dl.progress(), 0.25);
    }

    #[test]
    fn test_download_full_cycle() {
        let snap = make_test_snapshot(3, 2);
        let chunk_hashes: Vec<Hash> = snap.chunks.iter().map(|c| c.hash).collect();
        let ad = SnapshotAdvertisement::from_header(PeerId::test(1), &snap.header);
        let mut dl = SnapshotDownload::new(ad, chunk_hashes, 10);

        let batch = dl.next_requests();
        assert_eq!(batch, vec![0, 1, 2]);

        for i in 0..3u32 {
            let resp = ChunkResponse {
                manifest_root: snap.header.manifest_root,
                chunk_index: i,
                chunk: snap.chunks[i as usize].clone(),
                is_last: i == 2,
            };
            dl.receive_chunk(resp).unwrap();
        }
        assert!(dl.is_complete());
        assert_eq!(dl.progress(), 1.0);

        let assembled = dl.assemble().unwrap();
        assert_eq!(assembled.len(), 3);
        assert_eq!(assembled[0].index, 0);
        assert_eq!(assembled[2].index, 2);
    }

    #[test]
    fn test_download_integrity_failure_and_retry() {
        let snap = make_test_snapshot(2, 1);
        let chunk_hashes: Vec<Hash> = snap.chunks.iter().map(|c| c.hash).collect();
        let ad = SnapshotAdvertisement::from_header(PeerId::test(1), &snap.header);
        let mut dl = SnapshotDownload::new(ad, chunk_hashes, 10);
        dl.next_requests();

        let mut bad_chunk = snap.chunks[0].clone();
        bad_chunk.hash = [0xFF; 32];
        let bad_resp = ChunkResponse {
            manifest_root: snap.header.manifest_root,
            chunk_index: 0,
            chunk: bad_chunk,
            is_last: false,
        };
        assert!(dl.receive_chunk(bad_resp).is_err());
        assert_eq!(dl.failed.len(), 1);

        dl.retry_failed();
        assert!(dl.failed.is_empty());
        let retries = dl.next_requests();
        assert!(retries.contains(&0));
    }

    #[test]
    fn test_download_verify_manifest() {
        let snap = make_test_snapshot(3, 2);
        let chunk_hashes: Vec<Hash> = snap.chunks.iter().map(|c| c.hash).collect();
        let ad = SnapshotAdvertisement::from_header(PeerId::test(1), &snap.header);
        let dl = SnapshotDownload::new(ad, chunk_hashes, 10);
        assert!(dl.verify_manifest());
    }

    #[test]
    fn test_download_verify_manifest_tampered() {
        let snap = make_test_snapshot(3, 2);
        let mut chunk_hashes: Vec<Hash> = snap.chunks.iter().map(|c| c.hash).collect();
        chunk_hashes[1] = [0xDE; 32]; // tamper
        let ad = SnapshotAdvertisement::from_header(PeerId::test(1), &snap.header);
        let dl = SnapshotDownload::new(ad, chunk_hashes, 10);
        assert!(!dl.verify_manifest());
    }

    #[test]
    fn test_chunk_response_verify() {
        let snap = make_test_snapshot(1, 3);
        let resp = ChunkResponse {
            manifest_root: snap.header.manifest_root,
            chunk_index: 0,
            chunk: snap.chunks[0].clone(),
            is_last: true,
        };
        assert!(resp.verify_against_hash(&snap.chunks[0].hash));
        assert!(!resp.verify_against_hash(&[0xBB; 32]));
    }

    #[test]
    fn test_available_heights() {
        let mut server = SnapshotServer::new(PeerId::test(1), 10);
        let s1 = make_test_snapshot(1, 1);
        let mut s2 = make_test_snapshot(1, 1);
        s2.header.height = 200;
        // Need different manifest_root for s2 to not overwrite.
        s2.header.manifest_root = [0xBB; 32];
        server.register_snapshot(s1);
        server.register_snapshot(s2);
        let mut heights = server.available_heights();
        heights.sort();
        assert_eq!(heights, vec![100, 200]);
    }
}
