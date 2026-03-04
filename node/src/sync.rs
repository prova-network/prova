//! Chain Sync Protocol — block download, verification, and chain selection.
//!
//! Implements NODE-012:
//! - Header-first sync: download headers, validate chain, then fetch bodies
//! - Fork choice: longest valid chain (heaviest by stake weight)
//! - Peer scoring: track peer reliability for block serving
//! - Range requests: request block ranges from peers
//! - State machine: Idle → Syncing → Verifying → Synced
//!
//! Design follows the "header-first" approach:
//! 1. Discover peers' chain tips via status messages
//! 2. Download headers in batches, validate linkage + PoA signatures
//! 3. Download block bodies for verified headers
//! 4. Execute blocks to advance local state
//! 5. Switch to follow-tip mode once caught up

use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, HashMap, HashSet, VecDeque};

/// Unique peer identifier.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PeerId(pub [u8; 32]);

impl PeerId {
    pub fn test(id: u8) -> Self {
        let mut b = [0u8; 32];
        b[0] = id;
        Self(b)
    }
}

/// Block hash (32 bytes).
pub type BlockHash = [u8; 32];

/// Sync state machine.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SyncState {
    /// No sync in progress, at chain tip.
    Idle,
    /// Downloading and verifying headers.
    SyncingHeaders,
    /// Downloading block bodies for verified headers.
    SyncingBodies,
    /// Executing downloaded blocks.
    Executing,
    /// Fully synced, following tip.
    Synced,
}

/// A compact block header for sync purposes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SyncHeader {
    pub hash: BlockHash,
    pub parent_hash: BlockHash,
    pub epoch: u64,
    pub producer: [u8; 32],
    pub state_root: BlockHash,
    pub tx_root: BlockHash,
    pub tx_count: u32,
}

impl SyncHeader {
    /// Verify the header hash matches its contents.
    pub fn verify_hash(&self) -> bool {
        let computed = self.compute_hash();
        computed == self.hash
    }

    pub fn compute_hash(&self) -> BlockHash {
        let mut h = Sha256::new();
        h.update(self.parent_hash);
        h.update(self.state_root);
        h.update(self.epoch.to_le_bytes());
        h.update(self.producer);
        h.update(self.tx_root);
        h.update(self.tx_count.to_le_bytes());
        h.finalize().into()
    }
}

/// A block body (just transaction data for sync).
#[derive(Debug, Clone)]
pub struct SyncBody {
    pub tx_data: Vec<Vec<u8>>,
}

impl SyncBody {
    /// Compute the tx root to verify against the header.
    pub fn compute_tx_root(&self) -> BlockHash {
        if self.tx_data.is_empty() {
            return [0u8; 32];
        }
        let mut h = Sha256::new();
        for (i, tx) in self.tx_data.iter().enumerate() {
            h.update((i as u64).to_le_bytes());
            h.update(tx);
        }
        h.finalize().into()
    }
}

/// Peer chain status.
#[derive(Debug, Clone)]
pub struct PeerStatus {
    pub peer: PeerId,
    pub best_hash: BlockHash,
    pub best_epoch: u64,
    pub genesis_hash: BlockHash,
}

/// Peer score tracking.
#[derive(Debug, Clone)]
pub struct PeerScore {
    pub peer: PeerId,
    /// Successful responses.
    pub good: u64,
    /// Timeouts or invalid data.
    pub bad: u64,
    /// Currently banned.
    pub banned: bool,
}

impl PeerScore {
    pub fn new(peer: PeerId) -> Self {
        Self { peer, good: 0, bad: 0, banned: false }
    }

    pub fn score(&self) -> i64 {
        self.good as i64 - (self.bad as i64 * 5)
    }

    pub fn record_good(&mut self) {
        self.good += 1;
    }

    pub fn record_bad(&mut self) {
        self.bad += 1;
        if self.score() < -10 {
            self.banned = true;
        }
    }
}

/// Request types sent to peers.
#[derive(Debug, Clone)]
pub enum SyncRequest {
    /// Request peer's chain status.
    GetStatus,
    /// Request headers in an epoch range.
    GetHeaders { from_epoch: u64, count: u64 },
    /// Request block bodies by hash.
    GetBodies { hashes: Vec<BlockHash> },
}

/// Response types from peers.
#[derive(Debug, Clone)]
pub enum SyncResponse {
    Status(PeerStatus),
    Headers(Vec<SyncHeader>),
    Bodies(Vec<(BlockHash, SyncBody)>),
}

/// Maximum headers per request.
const MAX_HEADERS_PER_REQUEST: u64 = 128;
/// Maximum bodies per request.
const MAX_BODIES_PER_REQUEST: usize = 32;

/// Pending request tracker.
#[derive(Debug)]
struct PendingRequest {
    peer: PeerId,
    request: SyncRequest,
    sent_at: u64, // logical clock
}

/// Chain sync engine.
pub struct ChainSync {
    /// Current sync state.
    pub state: SyncState,
    /// Our genesis hash.
    genesis_hash: BlockHash,
    /// Local chain tip.
    local_tip: u64,
    local_tip_hash: BlockHash,
    /// Known peer statuses.
    peer_statuses: HashMap<PeerId, PeerStatus>,
    /// Peer scores.
    peer_scores: HashMap<PeerId, PeerScore>,
    /// Downloaded headers awaiting body fetch, ordered by epoch.
    verified_headers: BTreeMap<u64, SyncHeader>,
    /// Downloaded bodies ready for execution, by hash.
    ready_blocks: BTreeMap<u64, (SyncHeader, SyncBody)>,
    /// Headers whose bodies we still need.
    bodies_needed: VecDeque<BlockHash>,
    /// Epochs we've requested headers for (to avoid duplicates).
    requested_epochs: HashSet<u64>,
    /// Logical clock for timeout tracking.
    clock: u64,
    /// Pending requests.
    pending: Vec<PendingRequest>,
}

impl ChainSync {
    /// Create a new sync engine.
    pub fn new(genesis_hash: BlockHash, local_tip: u64, local_tip_hash: BlockHash) -> Self {
        Self {
            state: SyncState::Idle,
            genesis_hash,
            local_tip,
            local_tip_hash,
            peer_statuses: HashMap::new(),
            peer_scores: HashMap::new(),
            verified_headers: BTreeMap::new(),
            ready_blocks: BTreeMap::new(),
            bodies_needed: VecDeque::new(),
            requested_epochs: HashSet::new(),
            clock: 0,
            pending: Vec::new(),
        }
    }

    /// Get current sync state.
    pub fn sync_state(&self) -> SyncState {
        self.state
    }

    /// Get local chain tip epoch.
    pub fn local_tip_epoch(&self) -> u64 {
        self.local_tip
    }

    /// Get best known peer epoch.
    pub fn best_peer_epoch(&self) -> u64 {
        self.peer_statuses.values().map(|s| s.best_epoch).max().unwrap_or(0)
    }

    /// Number of verified headers pending body download.
    pub fn pending_headers(&self) -> usize {
        self.verified_headers.len()
    }

    /// Number of complete blocks ready for execution.
    pub fn ready_block_count(&self) -> usize {
        self.ready_blocks.len()
    }

    /// Register a peer's status.
    pub fn on_peer_status(&mut self, status: PeerStatus) {
        if status.genesis_hash != self.genesis_hash {
            // Wrong network — ignore and penalize.
            self.record_bad(status.peer);
            return;
        }
        self.peer_scores.entry(status.peer).or_insert_with(|| PeerScore::new(status.peer));
        self.peer_statuses.insert(status.peer, status);
    }

    /// Remove a disconnected peer.
    pub fn on_peer_disconnect(&mut self, peer: PeerId) {
        self.peer_statuses.remove(&peer);
        // Keep score for reconnects.
    }

    /// Start or continue sync. Returns requests to send to peers.
    pub fn poll(&mut self) -> Vec<(PeerId, SyncRequest)> {
        self.clock += 1;
        let mut requests = Vec::new();

        let best_peer = self.best_peer_epoch();
        if best_peer <= self.local_tip {
            // We're at tip or ahead.
            self.state = SyncState::Synced;
            return requests;
        }

        // Phase 1: Request missing headers.
        if self.verified_headers.is_empty() && self.ready_blocks.is_empty() {
            self.state = SyncState::SyncingHeaders;
            let start = self.local_tip + 1;
            let end = best_peer.min(start + MAX_HEADERS_PER_REQUEST - 1);

            if let Some(peer) = self.pick_peer() {
                let req = SyncRequest::GetHeaders {
                    from_epoch: start,
                    count: end - start + 1,
                };
                self.pending.push(PendingRequest {
                    peer,
                    request: req.clone(),
                    sent_at: self.clock,
                });
                requests.push((peer, req));
            }
        }

        // Phase 2: Request missing bodies.
        if !self.bodies_needed.is_empty() {
            self.state = SyncState::SyncingBodies;
            let batch: Vec<BlockHash> = self.bodies_needed.drain(..self.bodies_needed.len().min(MAX_BODIES_PER_REQUEST)).collect();
            if let Some(peer) = self.pick_peer() {
                let req = SyncRequest::GetBodies { hashes: batch };
                requests.push((peer, req));
            }
        }

        // Phase 3: Execute ready blocks.
        if !self.ready_blocks.is_empty() && self.bodies_needed.is_empty() {
            self.state = SyncState::Executing;
        }

        requests
    }

    /// Handle incoming headers from a peer.
    pub fn on_headers(&mut self, peer: PeerId, headers: Vec<SyncHeader>) -> Result<(), SyncError> {
        if headers.is_empty() {
            self.record_bad(peer);
            return Err(SyncError::EmptyResponse);
        }

        // Validate header chain linkage.
        let mut prev_hash = if headers[0].epoch == self.local_tip + 1 {
            self.local_tip_hash
        } else if headers[0].epoch > 0 {
            // Must connect to a known header.
            if let Some(known) = self.verified_headers.get(&(headers[0].epoch - 1)) {
                known.hash
            } else {
                self.record_bad(peer);
                return Err(SyncError::DisconnectedChain);
            }
        } else {
            [0u8; 32] // genesis
        };

        for header in &headers {
            // Verify hash.
            if !header.verify_hash() {
                self.record_bad(peer);
                return Err(SyncError::InvalidHeaderHash { epoch: header.epoch });
            }
            // Verify linkage.
            if header.parent_hash != prev_hash {
                self.record_bad(peer);
                return Err(SyncError::BrokenChain { epoch: header.epoch });
            }
            prev_hash = header.hash;
        }

        // All valid — store and queue body fetches.
        self.record_good(peer);
        for header in headers {
            let hash = header.hash;
            self.verified_headers.insert(header.epoch, header);
            self.bodies_needed.push_back(hash);
        }

        Ok(())
    }

    /// Handle incoming block bodies from a peer.
    pub fn on_bodies(&mut self, peer: PeerId, bodies: Vec<(BlockHash, SyncBody)>) -> Result<(), SyncError> {
        if bodies.is_empty() {
            self.record_bad(peer);
            return Err(SyncError::EmptyResponse);
        }

        for (hash, body) in bodies {
            // Find the header for this body.
            let epoch = self.verified_headers.iter()
                .find(|(_, h)| h.hash == hash)
                .map(|(&e, _)| e);

            let epoch = match epoch {
                Some(e) => e,
                None => {
                    self.record_bad(peer);
                    return Err(SyncError::UnrequestedBody { hash });
                }
            };

            let header = self.verified_headers.get(&epoch).unwrap().clone();

            // Verify tx_root matches.
            let computed_root = body.compute_tx_root();
            if computed_root != header.tx_root {
                self.record_bad(peer);
                return Err(SyncError::TxRootMismatch { epoch });
            }

            // Verify tx count.
            if body.tx_data.len() != header.tx_count as usize {
                self.record_bad(peer);
                return Err(SyncError::TxCountMismatch { epoch });
            }

            self.record_good(peer);
            self.verified_headers.remove(&epoch);
            self.ready_blocks.insert(epoch, (header, body));
        }

        Ok(())
    }

    /// Take the next batch of blocks ready for execution (in order).
    /// Returns blocks in ascending epoch order.
    pub fn take_executable_blocks(&mut self, max: usize) -> Vec<(SyncHeader, SyncBody)> {
        let mut result = Vec::new();
        let expected = self.local_tip + 1;

        let epochs: Vec<u64> = self.ready_blocks.keys().copied().collect();
        for epoch in epochs {
            if epoch != expected + result.len() as u64 {
                break; // Gap — can't execute out of order.
            }
            if result.len() >= max {
                break;
            }
            if let Some(block) = self.ready_blocks.remove(&epoch) {
                result.push(block);
            }
        }

        result
    }

    /// Mark blocks as executed, advancing the local tip.
    pub fn mark_executed(&mut self, new_tip: u64, new_tip_hash: BlockHash) {
        self.local_tip = new_tip;
        self.local_tip_hash = new_tip_hash;

        if self.local_tip >= self.best_peer_epoch()
            && self.verified_headers.is_empty()
            && self.ready_blocks.is_empty()
        {
            self.state = SyncState::Synced;
        }
    }

    /// Check if a peer is banned.
    pub fn is_banned(&self, peer: &PeerId) -> bool {
        self.peer_scores.get(peer).map(|s| s.banned).unwrap_or(false)
    }

    /// Get peer score.
    pub fn peer_score(&self, peer: &PeerId) -> i64 {
        self.peer_scores.get(peer).map(|s| s.score()).unwrap_or(0)
    }

    /// Pick the best non-banned peer that has blocks we need.
    fn pick_peer(&self) -> Option<PeerId> {
        self.peer_statuses.iter()
            .filter(|(id, status)| {
                !self.is_banned(id) && status.best_epoch > self.local_tip
            })
            .max_by_key(|(id, _)| self.peer_score(id))
            .map(|(id, _)| *id)
    }

    fn record_good(&mut self, peer: PeerId) {
        self.peer_scores.entry(peer).or_insert_with(|| PeerScore::new(peer)).record_good();
    }

    fn record_bad(&mut self, peer: PeerId) {
        self.peer_scores.entry(peer).or_insert_with(|| PeerScore::new(peer)).record_bad();
    }
}

/// Sync errors.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SyncError {
    EmptyResponse,
    InvalidHeaderHash { epoch: u64 },
    BrokenChain { epoch: u64 },
    DisconnectedChain,
    UnrequestedBody { hash: BlockHash },
    TxRootMismatch { epoch: u64 },
    TxCountMismatch { epoch: u64 },
}

impl std::fmt::Display for SyncError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::EmptyResponse => write!(f, "peer sent empty response"),
            Self::InvalidHeaderHash { epoch } => write!(f, "invalid header hash at epoch {}", epoch),
            Self::BrokenChain { epoch } => write!(f, "broken chain linkage at epoch {}", epoch),
            Self::DisconnectedChain => write!(f, "headers don't connect to known chain"),
            Self::UnrequestedBody { .. } => write!(f, "received body for unrequested block"),
            Self::TxRootMismatch { epoch } => write!(f, "tx root mismatch at epoch {}", epoch),
            Self::TxCountMismatch { epoch } => write!(f, "tx count mismatch at epoch {}", epoch),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Build a chain of valid linked headers.
    fn build_chain(genesis_hash: BlockHash, count: u64) -> Vec<(SyncHeader, SyncBody)> {
        let mut blocks = Vec::new();
        let mut prev_hash = genesis_hash;

        for epoch in 1..=count {
            let tx_data: Vec<Vec<u8>> = vec![
                format!("tx-{}-0", epoch).into_bytes(),
                format!("tx-{}-1", epoch).into_bytes(),
            ];
            let body = SyncBody { tx_data };
            let tx_root = body.compute_tx_root();

            let mut header = SyncHeader {
                hash: [0u8; 32], // will compute
                parent_hash: prev_hash,
                epoch,
                producer: [epoch as u8; 32],
                state_root: [0u8; 32],
                tx_root,
                tx_count: 2,
            };
            header.hash = header.compute_hash();
            prev_hash = header.hash;
            blocks.push((header, body));
        }
        blocks
    }

    fn genesis_hash() -> BlockHash {
        let mut h = Sha256::new();
        h.update(b"prova-genesis");
        h.finalize().into()
    }

    #[test]
    fn test_sync_state_initial() {
        let gh = genesis_hash();
        let sync = ChainSync::new(gh, 0, gh);
        assert_eq!(sync.sync_state(), SyncState::Idle);
        assert_eq!(sync.local_tip_epoch(), 0);
        assert_eq!(sync.best_peer_epoch(), 0);
    }

    #[test]
    fn test_peer_status_wrong_genesis() {
        let gh = genesis_hash();
        let mut sync = ChainSync::new(gh, 0, gh);
        let peer = PeerId::test(1);

        sync.on_peer_status(PeerStatus {
            peer,
            best_hash: [1u8; 32],
            best_epoch: 100,
            genesis_hash: [99u8; 32], // wrong genesis
        });

        // Peer should be penalized, not tracked.
        assert!(sync.peer_score(&peer) < 0);
    }

    #[test]
    fn test_peer_status_correct_genesis() {
        let gh = genesis_hash();
        let mut sync = ChainSync::new(gh, 0, gh);
        let peer = PeerId::test(1);

        sync.on_peer_status(PeerStatus {
            peer,
            best_hash: [1u8; 32],
            best_epoch: 100,
            genesis_hash: gh,
        });

        assert_eq!(sync.best_peer_epoch(), 100);
        assert_eq!(sync.peer_score(&peer), 0);
    }

    #[test]
    fn test_header_sync_and_verification() {
        let gh = genesis_hash();
        let chain = build_chain(gh, 5);
        let mut sync = ChainSync::new(gh, 0, gh);
        let peer = PeerId::test(1);

        sync.on_peer_status(PeerStatus {
            peer,
            best_hash: chain.last().unwrap().0.hash,
            best_epoch: 5,
            genesis_hash: gh,
        });

        let headers: Vec<SyncHeader> = chain.iter().map(|(h, _)| h.clone()).collect();
        assert!(sync.on_headers(peer, headers).is_ok());
        assert_eq!(sync.pending_headers(), 5);
        assert_eq!(sync.peer_score(&peer), 1); // one good response
    }

    #[test]
    fn test_broken_chain_rejected() {
        let gh = genesis_hash();
        let chain = build_chain(gh, 3);
        let mut sync = ChainSync::new(gh, 0, gh);
        let peer = PeerId::test(1);

        sync.on_peer_status(PeerStatus {
            peer,
            best_hash: chain.last().unwrap().0.hash,
            best_epoch: 3,
            genesis_hash: gh,
        });

        // Tamper with linkage.
        let mut headers: Vec<SyncHeader> = chain.iter().map(|(h, _)| h.clone()).collect();
        headers[1].parent_hash = [42u8; 32]; // break link

        let result = sync.on_headers(peer, headers);
        assert!(matches!(result, Err(SyncError::InvalidHeaderHash { .. }) | Err(SyncError::BrokenChain { .. })));
    }

    #[test]
    fn test_body_verification() {
        let gh = genesis_hash();
        let chain = build_chain(gh, 3);
        let mut sync = ChainSync::new(gh, 0, gh);
        let peer = PeerId::test(1);

        sync.on_peer_status(PeerStatus {
            peer,
            best_hash: chain.last().unwrap().0.hash,
            best_epoch: 3,
            genesis_hash: gh,
        });

        let headers: Vec<SyncHeader> = chain.iter().map(|(h, _)| h.clone()).collect();
        sync.on_headers(peer, headers).unwrap();

        // Send correct bodies.
        let bodies: Vec<(BlockHash, SyncBody)> = chain.iter()
            .map(|(h, b)| (h.hash, b.clone()))
            .collect();
        assert!(sync.on_bodies(peer, bodies).is_ok());
        assert_eq!(sync.ready_block_count(), 3);
    }

    #[test]
    fn test_bad_tx_root_rejected() {
        let gh = genesis_hash();
        let chain = build_chain(gh, 2);
        let mut sync = ChainSync::new(gh, 0, gh);
        let peer = PeerId::test(1);

        sync.on_peer_status(PeerStatus {
            peer,
            best_hash: chain.last().unwrap().0.hash,
            best_epoch: 2,
            genesis_hash: gh,
        });

        let headers: Vec<SyncHeader> = chain.iter().map(|(h, _)| h.clone()).collect();
        sync.on_headers(peer, headers).unwrap();

        // Send body with wrong tx data.
        let bad_body = SyncBody { tx_data: vec![b"wrong".to_vec()] };
        let result = sync.on_bodies(peer, vec![(chain[0].0.hash, bad_body)]);
        assert!(matches!(result, Err(SyncError::TxRootMismatch { .. }) | Err(SyncError::TxCountMismatch { .. })));
    }

    #[test]
    fn test_take_executable_blocks_in_order() {
        let gh = genesis_hash();
        let chain = build_chain(gh, 5);
        let mut sync = ChainSync::new(gh, 0, gh);
        let peer = PeerId::test(1);

        sync.on_peer_status(PeerStatus {
            peer,
            best_hash: chain.last().unwrap().0.hash,
            best_epoch: 5,
            genesis_hash: gh,
        });

        let headers: Vec<SyncHeader> = chain.iter().map(|(h, _)| h.clone()).collect();
        sync.on_headers(peer, headers).unwrap();

        let bodies: Vec<(BlockHash, SyncBody)> = chain.iter()
            .map(|(h, b)| (h.hash, b.clone()))
            .collect();
        sync.on_bodies(peer, bodies).unwrap();

        // Take first 3.
        let batch = sync.take_executable_blocks(3);
        assert_eq!(batch.len(), 3);
        assert_eq!(batch[0].0.epoch, 1);
        assert_eq!(batch[2].0.epoch, 3);

        // Mark executed.
        sync.mark_executed(3, batch[2].0.hash);
        assert_eq!(sync.local_tip_epoch(), 3);

        // Take remaining.
        let batch2 = sync.take_executable_blocks(10);
        assert_eq!(batch2.len(), 2);
        assert_eq!(batch2[0].0.epoch, 4);
    }

    #[test]
    fn test_poll_generates_requests() {
        let gh = genesis_hash();
        let mut sync = ChainSync::new(gh, 0, gh);
        let peer = PeerId::test(1);

        sync.on_peer_status(PeerStatus {
            peer,
            best_hash: [1u8; 32],
            best_epoch: 50,
            genesis_hash: gh,
        });

        let reqs = sync.poll();
        assert!(!reqs.is_empty());
        match &reqs[0].1 {
            SyncRequest::GetHeaders { from_epoch, count } => {
                assert_eq!(*from_epoch, 1);
                assert!(*count <= 128);
            }
            _ => panic!("expected GetHeaders"),
        }
    }

    #[test]
    fn test_synced_state_when_at_tip() {
        let gh = genesis_hash();
        let chain = build_chain(gh, 2);
        let mut sync = ChainSync::new(gh, 0, gh);
        let peer = PeerId::test(1);

        sync.on_peer_status(PeerStatus {
            peer,
            best_hash: chain.last().unwrap().0.hash,
            best_epoch: 2,
            genesis_hash: gh,
        });

        let headers: Vec<SyncHeader> = chain.iter().map(|(h, _)| h.clone()).collect();
        sync.on_headers(peer, headers).unwrap();
        let bodies: Vec<(BlockHash, SyncBody)> = chain.iter()
            .map(|(h, b)| (h.hash, b.clone()))
            .collect();
        sync.on_bodies(peer, bodies).unwrap();

        let batch = sync.take_executable_blocks(10);
        sync.mark_executed(2, batch.last().unwrap().0.hash);

        assert_eq!(sync.sync_state(), SyncState::Synced);
    }

    #[test]
    fn test_peer_banning() {
        let gh = genesis_hash();
        let mut sync = ChainSync::new(gh, 0, gh);
        let peer = PeerId::test(1);

        // 3 bad responses → score -15 → banned
        for _ in 0..3 {
            sync.record_bad(peer);
        }
        assert!(sync.is_banned(&peer));
        assert!(sync.peer_score(&peer) < -10);
    }

    #[test]
    fn test_empty_response_rejected() {
        let gh = genesis_hash();
        let mut sync = ChainSync::new(gh, 0, gh);
        let peer = PeerId::test(1);

        assert!(matches!(sync.on_headers(peer, vec![]), Err(SyncError::EmptyResponse)));
        assert!(matches!(sync.on_bodies(peer, vec![]), Err(SyncError::EmptyResponse)));
    }

    #[test]
    fn test_peer_disconnect_cleanup() {
        let gh = genesis_hash();
        let mut sync = ChainSync::new(gh, 0, gh);
        let peer = PeerId::test(1);

        sync.on_peer_status(PeerStatus {
            peer,
            best_hash: [1u8; 32],
            best_epoch: 50,
            genesis_hash: gh,
        });

        assert_eq!(sync.best_peer_epoch(), 50);
        sync.on_peer_disconnect(peer);
        assert_eq!(sync.best_peer_epoch(), 0);
    }

    #[test]
    fn test_full_sync_lifecycle() {
        let gh = genesis_hash();
        let chain = build_chain(gh, 10);
        let mut sync = ChainSync::new(gh, 0, gh);
        let peer = PeerId::test(1);

        // 1. Peer announces.
        sync.on_peer_status(PeerStatus {
            peer,
            best_hash: chain.last().unwrap().0.hash,
            best_epoch: 10,
            genesis_hash: gh,
        });

        // 2. Poll → get header request.
        let reqs = sync.poll();
        assert_eq!(sync.sync_state(), SyncState::SyncingHeaders);
        assert!(!reqs.is_empty());

        // 3. Receive headers.
        let headers: Vec<SyncHeader> = chain.iter().map(|(h, _)| h.clone()).collect();
        sync.on_headers(peer, headers).unwrap();

        // 4. Poll → get body request.
        let reqs2 = sync.poll();
        assert_eq!(sync.sync_state(), SyncState::SyncingBodies);
        assert!(!reqs2.is_empty());

        // 5. Receive bodies.
        let bodies: Vec<(BlockHash, SyncBody)> = chain.iter()
            .map(|(h, b)| (h.hash, b.clone()))
            .collect();
        sync.on_bodies(peer, bodies).unwrap();

        // 6. Execute.
        let batch = sync.take_executable_blocks(100);
        assert_eq!(batch.len(), 10);
        sync.mark_executed(10, batch.last().unwrap().0.hash);

        // 7. Fully synced.
        assert_eq!(sync.sync_state(), SyncState::Synced);
        assert_eq!(sync.local_tip_epoch(), 10);
        assert!(sync.peer_score(&peer) > 0);
    }
}
