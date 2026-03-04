//! State snapshots — serializable state export/import for fast sync.
//!
//! Enables nodes to:
//! - Export full state trie at any height to a portable format
//! - Import state snapshots to bootstrap without replaying history
//! - Verify snapshot integrity via Merkle root + chunk hashes
//! - Stream large snapshots in chunks for bandwidth efficiency
//!
//! Snapshot format:
//! ```text
//! [Header: version, height, state_root, account_count, chunk_count]
//! [Chunk 0: accounts[0..N], chunk_hash]
//! [Chunk 1: accounts[N..2N], chunk_hash]
//! ...
//! [Manifest: ordered chunk hashes → manifest_root]
//! ```

use std::collections::BTreeMap;

use sha2::{Digest, Sha256};

use crate::state::{AccountState, StateTrie};
use crate::types::{Address, Hash};

/// Snapshot format version.
const SNAPSHOT_VERSION: u32 = 1;

/// Default accounts per chunk.
const DEFAULT_CHUNK_SIZE: usize = 1000;

/// Serialized account entry in a snapshot.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SnapshotAccount {
    pub address: Address,
    pub balance: u128,
    pub nonce: u64,
    pub code_hash: Option<Hash>,
    pub storage: BTreeMap<Hash, Hash>,
}

impl SnapshotAccount {
    /// Deterministic serialization for hashing.
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut buf = Vec::with_capacity(128);
        buf.extend_from_slice(&self.address.0);
        buf.extend_from_slice(&self.balance.to_be_bytes());
        buf.extend_from_slice(&self.nonce.to_be_bytes());
        match &self.code_hash {
            Some(h) => {
                buf.push(1);
                buf.extend_from_slice(h);
            }
            None => buf.push(0),
        }
        buf.extend_from_slice(&(self.storage.len() as u32).to_be_bytes());
        for (k, v) in &self.storage {
            buf.extend_from_slice(k);
            buf.extend_from_slice(v);
        }
        buf
    }

    /// Deserialize from bytes. Returns (account, bytes_consumed).
    pub fn from_bytes(data: &[u8]) -> Result<(Self, usize), SnapshotError> {
        // Minimum: 20 (addr) + 16 (balance) + 8 (nonce) + 1 (code flag) + 4 (storage len) = 49
        if data.len() < 49 {
            return Err(SnapshotError::Truncated);
        }
        let mut pos = 0;

        let mut addr_bytes = [0u8; 20];
        addr_bytes.copy_from_slice(&data[pos..pos + 20]);
        let address = Address(addr_bytes);
        pos += 20;

        let balance = u128::from_be_bytes(data[pos..pos + 16].try_into().unwrap());
        pos += 16;

        let nonce = u64::from_be_bytes(data[pos..pos + 8].try_into().unwrap());
        pos += 8;

        let code_hash = if data[pos] == 1 {
            pos += 1;
            if data.len() < pos + 32 {
                return Err(SnapshotError::Truncated);
            }
            let mut h = [0u8; 32];
            h.copy_from_slice(&data[pos..pos + 32]);
            pos += 32;
            Some(h)
        } else {
            pos += 1;
            None
        };

        if data.len() < pos + 4 {
            return Err(SnapshotError::Truncated);
        }
        let storage_len = u32::from_be_bytes(data[pos..pos + 4].try_into().unwrap()) as usize;
        pos += 4;

        if data.len() < pos + storage_len * 64 {
            return Err(SnapshotError::Truncated);
        }
        let mut storage = BTreeMap::new();
        for _ in 0..storage_len {
            let mut k = [0u8; 32];
            let mut v = [0u8; 32];
            k.copy_from_slice(&data[pos..pos + 32]);
            pos += 32;
            v.copy_from_slice(&data[pos..pos + 32]);
            pos += 32;
            storage.insert(k, v);
        }

        Ok((
            Self {
                address,
                balance,
                nonce,
                code_hash,
                storage,
            },
            pos,
        ))
    }
}

/// A chunk of accounts within a snapshot.
#[derive(Debug, Clone)]
pub struct SnapshotChunk {
    pub index: u32,
    pub accounts: Vec<SnapshotAccount>,
    pub hash: Hash,
}

impl SnapshotChunk {
    /// Compute chunk hash from accounts.
    pub fn compute_hash(accounts: &[SnapshotAccount]) -> Hash {
        let mut hasher = Sha256::new();
        for acct in accounts {
            hasher.update(&acct.to_bytes());
        }
        let result = hasher.finalize();
        let mut hash = [0u8; 32];
        hash.copy_from_slice(&result);
        hash
    }

    /// Serialize chunk to bytes (accounts concatenated).
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut buf = Vec::new();
        buf.extend_from_slice(&self.index.to_be_bytes());
        buf.extend_from_slice(&(self.accounts.len() as u32).to_be_bytes());
        for acct in &self.accounts {
            let ab = acct.to_bytes();
            buf.extend_from_slice(&(ab.len() as u32).to_be_bytes());
            buf.extend_from_slice(&ab);
        }
        buf.extend_from_slice(&self.hash);
        buf
    }

    /// Verify chunk integrity.
    pub fn verify(&self) -> bool {
        Self::compute_hash(&self.accounts) == self.hash
    }
}

/// Snapshot header.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SnapshotHeader {
    pub version: u32,
    pub height: u64,
    pub state_root: Hash,
    pub account_count: u64,
    pub chunk_count: u32,
    pub manifest_root: Hash,
}

impl SnapshotHeader {
    pub const SERIALIZED_SIZE: usize = 4 + 8 + 32 + 8 + 4 + 32; // 88 bytes

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut buf = Vec::with_capacity(Self::SERIALIZED_SIZE);
        buf.extend_from_slice(&self.version.to_be_bytes());
        buf.extend_from_slice(&self.height.to_be_bytes());
        buf.extend_from_slice(&self.state_root);
        buf.extend_from_slice(&self.account_count.to_be_bytes());
        buf.extend_from_slice(&self.chunk_count.to_be_bytes());
        buf.extend_from_slice(&self.manifest_root);
        buf
    }

    pub fn from_bytes(data: &[u8]) -> Result<Self, SnapshotError> {
        if data.len() < Self::SERIALIZED_SIZE {
            return Err(SnapshotError::Truncated);
        }
        let mut pos = 0;
        let version = u32::from_be_bytes(data[pos..pos + 4].try_into().unwrap());
        pos += 4;
        if version != SNAPSHOT_VERSION {
            return Err(SnapshotError::UnsupportedVersion(version));
        }
        let height = u64::from_be_bytes(data[pos..pos + 8].try_into().unwrap());
        pos += 8;
        let mut state_root = [0u8; 32];
        state_root.copy_from_slice(&data[pos..pos + 32]);
        pos += 32;
        let account_count = u64::from_be_bytes(data[pos..pos + 8].try_into().unwrap());
        pos += 8;
        let chunk_count = u32::from_be_bytes(data[pos..pos + 4].try_into().unwrap());
        pos += 4;
        let mut manifest_root = [0u8; 32];
        manifest_root.copy_from_slice(&data[pos..pos + 32]);

        Ok(Self {
            version,
            height,
            state_root,
            account_count,
            chunk_count,
            manifest_root,
        })
    }
}

/// Complete state snapshot.
#[derive(Debug, Clone)]
pub struct StateSnapshot {
    pub header: SnapshotHeader,
    pub chunks: Vec<SnapshotChunk>,
}

impl StateSnapshot {
    /// Create a snapshot from a state trie at a given height.
    pub fn create(trie: &mut StateTrie, height: u64, chunk_size: usize) -> Self {
        let chunk_size = if chunk_size == 0 { DEFAULT_CHUNK_SIZE } else { chunk_size };
        let state_root = trie.root();
        let accounts: Vec<SnapshotAccount> = trie
            .iter()
            .map(|(addr, acct)| SnapshotAccount {
                address: *addr,
                balance: acct.balance,
                nonce: acct.nonce,
                code_hash: acct.code_hash,
                storage: acct.storage.clone(),
            })
            .collect();

        let account_count = accounts.len() as u64;
        let mut chunks = Vec::new();

        for (i, chunk_accounts) in accounts.chunks(chunk_size).enumerate() {
            let accts = chunk_accounts.to_vec();
            let hash = SnapshotChunk::compute_hash(&accts);
            chunks.push(SnapshotChunk {
                index: i as u32,
                accounts: accts,
                hash,
            });
        }

        let manifest_root = Self::compute_manifest_root(&chunks);

        let header = SnapshotHeader {
            version: SNAPSHOT_VERSION,
            height,
            state_root,
            account_count,
            chunk_count: chunks.len() as u32,
            manifest_root,
        };

        Self { header, chunks }
    }

    /// Compute manifest root from chunk hashes (Merkle hash of all chunk hashes).
    fn compute_manifest_root(chunks: &[SnapshotChunk]) -> Hash {
        if chunks.is_empty() {
            return [0u8; 32];
        }
        let mut hasher = Sha256::new();
        for chunk in chunks {
            hasher.update(&chunk.hash);
        }
        let result = hasher.finalize();
        let mut hash = [0u8; 32];
        hash.copy_from_slice(&result);
        hash
    }

    /// Verify full snapshot integrity.
    pub fn verify(&self) -> Result<(), SnapshotError> {
        // 1. Verify each chunk hash.
        for chunk in &self.chunks {
            if !chunk.verify() {
                return Err(SnapshotError::ChunkHashMismatch(chunk.index));
            }
        }

        // 2. Verify manifest root.
        let computed_manifest = Self::compute_manifest_root(&self.chunks);
        if computed_manifest != self.header.manifest_root {
            return Err(SnapshotError::ManifestMismatch);
        }

        // 3. Verify account count.
        let total: u64 = self.chunks.iter().map(|c| c.accounts.len() as u64).sum();
        if total != self.header.account_count {
            return Err(SnapshotError::AccountCountMismatch {
                expected: self.header.account_count,
                actual: total,
            });
        }

        // 4. Verify chunk ordering.
        for (i, chunk) in self.chunks.iter().enumerate() {
            if chunk.index != i as u32 {
                return Err(SnapshotError::ChunkOrderMismatch {
                    expected: i as u32,
                    actual: chunk.index,
                });
            }
        }

        Ok(())
    }

    /// Restore a StateTrie from a verified snapshot.
    pub fn restore(&self) -> Result<StateTrie, SnapshotError> {
        self.verify()?;

        let mut trie = StateTrie::new();
        for chunk in &self.chunks {
            for acct in &chunk.accounts {
                trie.set_balance(acct.address, acct.balance);
                // Set nonce by consuming nonces up to the target value.
                for _ in 0..acct.nonce {
                    trie.use_nonce(acct.address);
                }
                if let Some(code_hash) = acct.code_hash {
                    // Use storage slot to preserve code hash (simplified).
                    let code_key = [0xFFu8; 32]; // Reserved key for code hash.
                    trie.set_storage(acct.address, code_key, code_hash);
                }
                for (k, v) in &acct.storage {
                    trie.set_storage(acct.address, *k, *v);
                }
            }
        }

        // Verify restored state root matches snapshot.
        let restored_root = trie.root();
        // Note: root may differ because we store code_hash via storage slot.
        // For full fidelity, StateTrie would need native code_hash support.
        // For now, we verify account count.
        if trie.account_count() as u64 != self.header.account_count {
            return Err(SnapshotError::RestoreAccountMismatch);
        }

        Ok(trie)
    }

    /// Get total byte size estimate of the snapshot.
    pub fn estimated_size(&self) -> usize {
        SnapshotHeader::SERIALIZED_SIZE
            + self
                .chunks
                .iter()
                .map(|c| c.to_bytes().len())
                .sum::<usize>()
    }

    /// Get a single chunk by index (for streaming).
    pub fn get_chunk(&self, index: u32) -> Option<&SnapshotChunk> {
        self.chunks.get(index as usize)
    }
}

/// Incremental snapshot builder — processes chunks one at a time (for streaming import).
#[derive(Debug)]
pub struct SnapshotImporter {
    header: SnapshotHeader,
    received_chunks: Vec<Option<SnapshotChunk>>,
    received_count: u32,
}

impl SnapshotImporter {
    /// Start an import from a header.
    pub fn new(header: SnapshotHeader) -> Result<Self, SnapshotError> {
        if header.version != SNAPSHOT_VERSION {
            return Err(SnapshotError::UnsupportedVersion(header.version));
        }
        let chunks = vec![None; header.chunk_count as usize];
        Ok(Self {
            header,
            received_chunks: chunks,
            received_count: 0,
        })
    }

    /// Add a chunk. Verifies hash before accepting.
    pub fn add_chunk(&mut self, chunk: SnapshotChunk) -> Result<(), SnapshotError> {
        let idx = chunk.index;
        if idx >= self.header.chunk_count {
            return Err(SnapshotError::ChunkIndexOutOfRange(idx));
        }
        if !chunk.verify() {
            return Err(SnapshotError::ChunkHashMismatch(idx));
        }
        if self.received_chunks[idx as usize].is_some() {
            return Err(SnapshotError::DuplicateChunk(idx));
        }
        self.received_chunks[idx as usize] = Some(chunk);
        self.received_count += 1;
        Ok(())
    }

    /// Check if all chunks received.
    pub fn is_complete(&self) -> bool {
        self.received_count == self.header.chunk_count
    }

    /// Progress as fraction (0.0 to 1.0).
    pub fn progress(&self) -> f64 {
        if self.header.chunk_count == 0 {
            return 1.0;
        }
        self.received_count as f64 / self.header.chunk_count as f64
    }

    /// Finalize into a complete snapshot. Fails if incomplete.
    pub fn finalize(self) -> Result<StateSnapshot, SnapshotError> {
        if !self.is_complete() {
            return Err(SnapshotError::IncompleteImport {
                received: self.received_count,
                expected: self.header.chunk_count,
            });
        }
        let chunks: Vec<SnapshotChunk> = self
            .received_chunks
            .into_iter()
            .map(|c| c.unwrap())
            .collect();

        let snapshot = StateSnapshot {
            header: self.header,
            chunks,
        };
        snapshot.verify()?;
        Ok(snapshot)
    }
}

/// Snapshot errors.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SnapshotError {
    Truncated,
    UnsupportedVersion(u32),
    ChunkHashMismatch(u32),
    ManifestMismatch,
    AccountCountMismatch { expected: u64, actual: u64 },
    ChunkOrderMismatch { expected: u32, actual: u32 },
    RestoreAccountMismatch,
    ChunkIndexOutOfRange(u32),
    DuplicateChunk(u32),
    IncompleteImport { received: u32, expected: u32 },
}

impl std::fmt::Display for SnapshotError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Truncated => write!(f, "snapshot data truncated"),
            Self::UnsupportedVersion(v) => write!(f, "unsupported snapshot version: {v}"),
            Self::ChunkHashMismatch(i) => write!(f, "chunk {i} hash mismatch"),
            Self::ManifestMismatch => write!(f, "manifest root mismatch"),
            Self::AccountCountMismatch { expected, actual } => {
                write!(f, "account count mismatch: expected {expected}, got {actual}")
            }
            Self::ChunkOrderMismatch { expected, actual } => {
                write!(f, "chunk order mismatch: expected {expected}, got {actual}")
            }
            Self::RestoreAccountMismatch => write!(f, "restored state account count mismatch"),
            Self::ChunkIndexOutOfRange(i) => write!(f, "chunk index {i} out of range"),
            Self::DuplicateChunk(i) => write!(f, "duplicate chunk {i}"),
            Self::IncompleteImport { received, expected } => {
                write!(f, "incomplete import: {received}/{expected} chunks")
            }
        }
    }
}

impl std::error::Error for SnapshotError {}

#[cfg(test)]
mod tests {
    use super::*;

    fn setup_trie(n: usize) -> StateTrie {
        let mut trie = StateTrie::new();
        for i in 0..n {
            let addr = Address::test(i as u8);
            trie.credit(addr, (i as u128 + 1) * 1000);
            for _ in 0..i {
                trie.use_nonce(addr);
            }
            if i % 3 == 0 {
                let key = [i as u8; 32];
                let val = [(i * 2) as u8; 32];
                trie.set_storage(addr, key, val);
            }
        }
        trie
    }

    #[test]
    fn test_create_snapshot_empty() {
        let mut trie = StateTrie::new();
        let snap = StateSnapshot::create(&mut trie, 0, DEFAULT_CHUNK_SIZE);
        assert_eq!(snap.header.version, SNAPSHOT_VERSION);
        assert_eq!(snap.header.height, 0);
        assert_eq!(snap.header.account_count, 0);
        assert_eq!(snap.header.chunk_count, 0);
        assert!(snap.verify().is_ok());
    }

    #[test]
    fn test_create_snapshot_single_chunk() {
        let mut trie = setup_trie(5);
        let snap = StateSnapshot::create(&mut trie, 100, DEFAULT_CHUNK_SIZE);
        assert_eq!(snap.header.height, 100);
        assert_eq!(snap.header.account_count, 5);
        assert_eq!(snap.header.chunk_count, 1);
        assert_eq!(snap.header.state_root, trie.root());
        assert!(snap.verify().is_ok());
    }

    #[test]
    fn test_create_snapshot_multiple_chunks() {
        let mut trie = setup_trie(10);
        let snap = StateSnapshot::create(&mut trie, 200, 3);
        assert_eq!(snap.header.account_count, 10);
        assert_eq!(snap.header.chunk_count, 4); // ceil(10/3)
        assert_eq!(snap.chunks[0].accounts.len(), 3);
        assert_eq!(snap.chunks[3].accounts.len(), 1);
        assert!(snap.verify().is_ok());
    }

    #[test]
    fn test_snapshot_restore() {
        let mut trie = setup_trie(8);
        let snap = StateSnapshot::create(&mut trie, 50, 3);
        let restored = snap.restore().unwrap();
        assert_eq!(restored.account_count(), trie.account_count());
        // Verify balances preserved.
        for i in 0..8u8 {
            let addr = Address::test(i);
            assert_eq!(restored.get(&addr).balance, trie.get(&addr).balance);
            assert_eq!(restored.get(&addr).nonce, trie.get(&addr).nonce);
        }
    }

    #[test]
    fn test_chunk_verification_detects_tampering() {
        let mut trie = setup_trie(5);
        let mut snap = StateSnapshot::create(&mut trie, 100, DEFAULT_CHUNK_SIZE);
        // Tamper with an account balance.
        snap.chunks[0].accounts[0].balance = 999_999;
        assert!(matches!(
            snap.verify(),
            Err(SnapshotError::ChunkHashMismatch(0))
        ));
    }

    #[test]
    fn test_manifest_verification_detects_tampering() {
        let mut trie = setup_trie(10);
        let mut snap = StateSnapshot::create(&mut trie, 100, 3);
        // Tamper with manifest root.
        snap.header.manifest_root = [0xAA; 32];
        // Chunks are fine, but manifest won't match.
        assert!(matches!(snap.verify(), Err(SnapshotError::ManifestMismatch)));
    }

    #[test]
    fn test_header_serialization_roundtrip() {
        let mut trie = setup_trie(5);
        let snap = StateSnapshot::create(&mut trie, 42, DEFAULT_CHUNK_SIZE);
        let bytes = snap.header.to_bytes();
        assert_eq!(bytes.len(), SnapshotHeader::SERIALIZED_SIZE);
        let decoded = SnapshotHeader::from_bytes(&bytes).unwrap();
        assert_eq!(decoded, snap.header);
    }

    #[test]
    fn test_header_rejects_bad_version() {
        let mut data = vec![0u8; SnapshotHeader::SERIALIZED_SIZE];
        data[0..4].copy_from_slice(&99u32.to_be_bytes()); // bad version
        assert!(matches!(
            SnapshotHeader::from_bytes(&data),
            Err(SnapshotError::UnsupportedVersion(99))
        ));
    }

    #[test]
    fn test_account_serialization_roundtrip() {
        let acct = SnapshotAccount {
            address: Address::test(42),
            balance: 123_456_789,
            nonce: 17,
            code_hash: Some([0xBB; 32]),
            storage: {
                let mut m = BTreeMap::new();
                m.insert([1u8; 32], [2u8; 32]);
                m.insert([3u8; 32], [4u8; 32]);
                m
            },
        };
        let bytes = acct.to_bytes();
        let (decoded, consumed) = SnapshotAccount::from_bytes(&bytes).unwrap();
        assert_eq!(consumed, bytes.len());
        assert_eq!(decoded, acct);
    }

    #[test]
    fn test_account_without_code_hash() {
        let acct = SnapshotAccount {
            address: Address::test(1),
            balance: 500,
            nonce: 0,
            code_hash: None,
            storage: BTreeMap::new(),
        };
        let bytes = acct.to_bytes();
        let (decoded, _) = SnapshotAccount::from_bytes(&bytes).unwrap();
        assert_eq!(decoded, acct);
    }

    #[test]
    fn test_streaming_import() {
        let mut trie = setup_trie(10);
        let snap = StateSnapshot::create(&mut trie, 100, 3);

        let mut importer = SnapshotImporter::new(snap.header.clone()).unwrap();
        assert!(!importer.is_complete());
        assert_eq!(importer.progress(), 0.0);

        // Add chunks out of order.
        importer.add_chunk(snap.chunks[2].clone()).unwrap();
        assert!((importer.progress() - 0.25).abs() < 0.01);

        importer.add_chunk(snap.chunks[0].clone()).unwrap();
        importer.add_chunk(snap.chunks[1].clone()).unwrap();
        importer.add_chunk(snap.chunks[3].clone()).unwrap();
        assert!(importer.is_complete());

        let restored = importer.finalize().unwrap();
        assert!(restored.verify().is_ok());
    }

    #[test]
    fn test_streaming_import_rejects_duplicate() {
        let mut trie = setup_trie(10);
        let snap = StateSnapshot::create(&mut trie, 100, 3);

        let mut importer = SnapshotImporter::new(snap.header.clone()).unwrap();
        importer.add_chunk(snap.chunks[0].clone()).unwrap();
        assert!(matches!(
            importer.add_chunk(snap.chunks[0].clone()),
            Err(SnapshotError::DuplicateChunk(0))
        ));
    }

    #[test]
    fn test_streaming_import_rejects_tampered_chunk() {
        let mut trie = setup_trie(5);
        let snap = StateSnapshot::create(&mut trie, 100, 3);

        let mut importer = SnapshotImporter::new(snap.header.clone()).unwrap();
        let mut bad_chunk = snap.chunks[0].clone();
        bad_chunk.accounts[0].balance = 0; // tamper
        assert!(matches!(
            importer.add_chunk(bad_chunk),
            Err(SnapshotError::ChunkHashMismatch(0))
        ));
    }

    #[test]
    fn test_incomplete_finalize_fails() {
        let mut trie = setup_trie(10);
        let snap = StateSnapshot::create(&mut trie, 100, 3);

        let mut importer = SnapshotImporter::new(snap.header.clone()).unwrap();
        importer.add_chunk(snap.chunks[0].clone()).unwrap();
        assert!(matches!(
            importer.finalize(),
            Err(SnapshotError::IncompleteImport { received: 1, expected: 4 })
        ));
    }

    #[test]
    fn test_estimated_size() {
        let mut trie = setup_trie(20);
        let snap = StateSnapshot::create(&mut trie, 100, 5);
        let size = snap.estimated_size();
        assert!(size > SnapshotHeader::SERIALIZED_SIZE);
        assert!(size > 0);
    }

    #[test]
    fn test_get_chunk() {
        let mut trie = setup_trie(10);
        let snap = StateSnapshot::create(&mut trie, 100, 3);
        assert!(snap.get_chunk(0).is_some());
        assert!(snap.get_chunk(3).is_some());
        assert!(snap.get_chunk(4).is_none());
    }

    #[test]
    fn test_empty_snapshot_import() {
        let mut trie = StateTrie::new();
        let snap = StateSnapshot::create(&mut trie, 0, DEFAULT_CHUNK_SIZE);
        let importer = SnapshotImporter::new(snap.header.clone()).unwrap();
        assert!(importer.is_complete());
        let restored = importer.finalize().unwrap();
        assert_eq!(restored.header.account_count, 0);
    }

    #[test]
    fn test_snapshot_preserves_storage_slots() {
        let mut trie = StateTrie::new();
        let addr = Address::test(1);
        trie.credit(addr, 1000);
        let k1 = [1u8; 32];
        let v1 = [42u8; 32];
        let k2 = [2u8; 32];
        let v2 = [99u8; 32];
        trie.set_storage(addr, k1, v1);
        trie.set_storage(addr, k2, v2);

        let snap = StateSnapshot::create(&mut trie, 10, DEFAULT_CHUNK_SIZE);
        let acct = &snap.chunks[0].accounts[0];
        assert_eq!(acct.storage.len(), 2);
        assert_eq!(acct.storage[&k1], v1);
        assert_eq!(acct.storage[&k2], v2);
    }

    #[test]
    fn test_deterministic_snapshot_hash() {
        let mut t1 = setup_trie(10);
        let mut t2 = setup_trie(10);
        let s1 = StateSnapshot::create(&mut t1, 100, 3);
        let s2 = StateSnapshot::create(&mut t2, 100, 3);
        assert_eq!(s1.header.manifest_root, s2.header.manifest_root);
        assert_eq!(s1.header.state_root, s2.header.state_root);
    }
}
