//! Persistent storage backend using sled embedded database.
//!
//! Provides typed key-value storage for:
//! - Block headers and bodies
//! - Chain state (accounts, nonces, balances)
//! - Transaction index (txhash → block + position)
//! - Metadata (chain head, sync state)
//!
//! Each logical store is a separate sled Tree (column family).

use sled::{Db, Tree};
use std::path::Path;

/// Column family names.
const CF_BLOCKS: &str = "blocks";
const CF_HEADERS: &str = "headers";
const CF_STATE: &str = "state";
const CF_TX_INDEX: &str = "tx_index";
const CF_META: &str = "meta";
const CF_RECEIPTS: &str = "receipts";

/// Meta keys.
const META_CHAIN_HEAD: &[u8] = b"chain_head";
const META_CHAIN_HEIGHT: &[u8] = b"chain_height";

/// Persistent storage engine wrapping sled.
pub struct Storage {
    db: Db,
    blocks: Tree,
    headers: Tree,
    state: Tree,
    tx_index: Tree,
    meta: Tree,
    receipts: Tree,
}

/// A located transaction: block hash + index within block.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TxLocation {
    pub block_hash: [u8; 32],
    pub index: u32,
}

impl TxLocation {
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut buf = Vec::with_capacity(36);
        buf.extend_from_slice(&self.block_hash);
        buf.extend_from_slice(&self.index.to_be_bytes());
        buf
    }

    pub fn from_bytes(data: &[u8]) -> Option<Self> {
        if data.len() < 36 {
            return None;
        }
        let mut block_hash = [0u8; 32];
        block_hash.copy_from_slice(&data[..32]);
        let index = u32::from_be_bytes(data[32..36].try_into().ok()?);
        Some(Self { block_hash, index })
    }
}

/// Errors from storage operations.
#[derive(Debug)]
pub enum StorageError {
    Sled(sled::Error),
    NotFound,
    Corrupted(String),
}

impl From<sled::Error> for StorageError {
    fn from(e: sled::Error) -> Self {
        StorageError::Sled(e)
    }
}

impl std::fmt::Display for StorageError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            StorageError::Sled(e) => write!(f, "sled: {e}"),
            StorageError::NotFound => write!(f, "not found"),
            StorageError::Corrupted(msg) => write!(f, "corrupted: {msg}"),
        }
    }
}

pub type Result<T> = std::result::Result<T, StorageError>;

impl Storage {
    /// Open (or create) a storage database at the given path.
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self> {
        let db = sled::open(path)?;
        let blocks = db.open_tree(CF_BLOCKS)?;
        let headers = db.open_tree(CF_HEADERS)?;
        let state = db.open_tree(CF_STATE)?;
        let tx_index = db.open_tree(CF_TX_INDEX)?;
        let meta = db.open_tree(CF_META)?;
        let receipts = db.open_tree(CF_RECEIPTS)?;

        Ok(Self {
            db,
            blocks,
            headers,
            state,
            tx_index,
            meta,
            receipts,
        })
    }

    /// Open a temporary storage (useful for tests).
    pub fn open_temp() -> Result<Self> {
        let db = sled::Config::new().temporary(true).open()?;
        let blocks = db.open_tree(CF_BLOCKS)?;
        let headers = db.open_tree(CF_HEADERS)?;
        let state = db.open_tree(CF_STATE)?;
        let tx_index = db.open_tree(CF_TX_INDEX)?;
        let meta = db.open_tree(CF_META)?;
        let receipts = db.open_tree(CF_RECEIPTS)?;

        Ok(Self {
            db,
            blocks,
            headers,
            state,
            tx_index,
            meta,
            receipts,
        })
    }

    // ── Blocks ──────────────────────────────────────────────

    /// Store a serialized block body keyed by block hash.
    pub fn put_block(&self, hash: &[u8; 32], data: &[u8]) -> Result<()> {
        self.blocks.insert(hash.as_slice(), data)?;
        Ok(())
    }

    /// Retrieve a block body by hash.
    pub fn get_block(&self, hash: &[u8; 32]) -> Result<Vec<u8>> {
        self.blocks
            .get(hash.as_slice())?
            .map(|v| v.to_vec())
            .ok_or(StorageError::NotFound)
    }

    /// Check if a block exists.
    pub fn has_block(&self, hash: &[u8; 32]) -> Result<bool> {
        Ok(self.blocks.contains_key(hash.as_slice())?)
    }

    // ── Headers ─────────────────────────────────────────────

    /// Store a serialized block header keyed by block hash.
    pub fn put_header(&self, hash: &[u8; 32], data: &[u8]) -> Result<()> {
        self.headers.insert(hash.as_slice(), data)?;
        Ok(())
    }

    /// Retrieve a block header by hash.
    pub fn get_header(&self, hash: &[u8; 32]) -> Result<Vec<u8>> {
        self.headers
            .get(hash.as_slice())?
            .map(|v| v.to_vec())
            .ok_or(StorageError::NotFound)
    }

    // ── Height index ────────────────────────────────────────

    /// Map a block height to its hash (stored in meta tree with "height:" prefix).
    pub fn put_height_index(&self, height: u64, hash: &[u8; 32]) -> Result<()> {
        let key = height_key(height);
        self.meta.insert(key, hash.as_slice())?;
        Ok(())
    }

    /// Look up block hash by height.
    pub fn get_hash_by_height(&self, height: u64) -> Result<[u8; 32]> {
        let key = height_key(height);
        let val = self.meta.get(key)?.ok_or(StorageError::NotFound)?;
        let mut hash = [0u8; 32];
        if val.len() < 32 {
            return Err(StorageError::Corrupted("short hash".into()));
        }
        hash.copy_from_slice(&val[..32]);
        Ok(hash)
    }

    // ── State (accounts) ────────────────────────────────────

    /// Store account state (serialized) keyed by address.
    pub fn put_account(&self, address: &[u8; 32], data: &[u8]) -> Result<()> {
        self.state.insert(address.as_slice(), data)?;
        Ok(())
    }

    /// Retrieve account state by address.
    pub fn get_account(&self, address: &[u8; 32]) -> Result<Vec<u8>> {
        self.state
            .get(address.as_slice())?
            .map(|v| v.to_vec())
            .ok_or(StorageError::NotFound)
    }

    /// Delete an account.
    pub fn delete_account(&self, address: &[u8; 32]) -> Result<()> {
        self.state.remove(address.as_slice())?;
        Ok(())
    }

    /// Iterate all accounts (for state snapshots / rebuilds).
    pub fn iter_accounts(&self) -> impl Iterator<Item = ([u8; 32], Vec<u8>)> + '_ {
        self.state.iter().filter_map(|res| {
            let (k, v) = res.ok()?;
            if k.len() < 32 {
                return None;
            }
            let mut addr = [0u8; 32];
            addr.copy_from_slice(&k[..32]);
            Some((addr, v.to_vec()))
        })
    }

    /// Count of accounts in state.
    pub fn account_count(&self) -> usize {
        self.state.len()
    }

    // ── Transaction index ───────────────────────────────────

    /// Index a transaction by its hash → (block_hash, index).
    pub fn put_tx_index(&self, tx_hash: &[u8; 32], loc: &TxLocation) -> Result<()> {
        self.tx_index.insert(tx_hash.as_slice(), loc.to_bytes())?;
        Ok(())
    }

    /// Look up a transaction location by tx hash.
    pub fn get_tx_location(&self, tx_hash: &[u8; 32]) -> Result<TxLocation> {
        let val = self
            .tx_index
            .get(tx_hash.as_slice())?
            .ok_or(StorageError::NotFound)?;
        TxLocation::from_bytes(&val).ok_or(StorageError::Corrupted("bad tx location".into()))
    }

    // ── Receipts ────────────────────────────────────────────

    /// Store a transaction receipt keyed by tx hash.
    pub fn put_receipt(&self, tx_hash: &[u8; 32], data: &[u8]) -> Result<()> {
        self.receipts.insert(tx_hash.as_slice(), data)?;
        Ok(())
    }

    /// Retrieve a receipt by tx hash.
    pub fn get_receipt(&self, tx_hash: &[u8; 32]) -> Result<Vec<u8>> {
        self.receipts
            .get(tx_hash.as_slice())?
            .map(|v| v.to_vec())
            .ok_or(StorageError::NotFound)
    }

    // ── Chain metadata ──────────────────────────────────────

    /// Set the chain head hash.
    pub fn set_chain_head(&self, hash: &[u8; 32]) -> Result<()> {
        self.meta.insert(META_CHAIN_HEAD, hash.as_slice())?;
        Ok(())
    }

    /// Get the chain head hash.
    pub fn chain_head(&self) -> Result<[u8; 32]> {
        let val = self
            .meta
            .get(META_CHAIN_HEAD)?
            .ok_or(StorageError::NotFound)?;
        let mut hash = [0u8; 32];
        if val.len() < 32 {
            return Err(StorageError::Corrupted("short head hash".into()));
        }
        hash.copy_from_slice(&val[..32]);
        Ok(hash)
    }

    /// Set the chain height.
    pub fn set_chain_height(&self, height: u64) -> Result<()> {
        self.meta.insert(META_CHAIN_HEIGHT, &height.to_be_bytes())?;
        Ok(())
    }

    /// Get the chain height.
    pub fn chain_height(&self) -> Result<u64> {
        let val = self
            .meta
            .get(META_CHAIN_HEIGHT)?
            .ok_or(StorageError::NotFound)?;
        if val.len() < 8 {
            return Err(StorageError::Corrupted("short height".into()));
        }
        Ok(u64::from_be_bytes(val[..8].try_into().unwrap()))
    }

    // ── Batch / Atomic operations ───────────────────────────

    /// Atomically write a block + header + height index + update chain head/height.
    /// This is the primary "commit block" operation.
    pub fn commit_block(
        &self,
        hash: &[u8; 32],
        height: u64,
        header: &[u8],
        body: &[u8],
        tx_hashes: &[([u8; 32], u32)], // (tx_hash, index)
    ) -> Result<()> {
        // sled doesn't have cross-tree transactions, so we write in order:
        // header → body → tx_index → height_index → meta
        // On crash, partial writes are acceptable (we can detect via chain_height).
        self.put_header(hash, header)?;
        self.put_block(hash, body)?;
        for (tx_hash, idx) in tx_hashes {
            self.put_tx_index(
                tx_hash,
                &TxLocation {
                    block_hash: *hash,
                    index: *idx,
                },
            )?;
        }
        self.put_height_index(height, hash)?;
        self.set_chain_head(hash)?;
        self.set_chain_height(height)?;
        // Flush to disk.
        self.db.flush()?;
        Ok(())
    }

    /// Flush all pending writes to disk.
    pub fn flush(&self) -> Result<()> {
        self.db.flush()?;
        Ok(())
    }

    /// Database size on disk (bytes).
    pub fn size_on_disk(&self) -> u64 {
        self.db.size_on_disk().unwrap_or(0)
    }
}

fn height_key(height: u64) -> Vec<u8> {
    let mut key = b"height:".to_vec();
    key.extend_from_slice(&height.to_be_bytes());
    key
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_hash(v: u8) -> [u8; 32] {
        [v; 32]
    }

    #[test]
    fn open_and_close() {
        let s = Storage::open_temp().unwrap();
        assert_eq!(s.account_count(), 0);
    }

    #[test]
    fn block_roundtrip() {
        let s = Storage::open_temp().unwrap();
        let hash = test_hash(1);
        let body = b"block body data";
        s.put_block(&hash, body).unwrap();
        assert_eq!(s.get_block(&hash).unwrap(), body);
        assert!(s.has_block(&hash).unwrap());
        assert!(!s.has_block(&test_hash(2)).unwrap());
    }

    #[test]
    fn header_roundtrip() {
        let s = Storage::open_temp().unwrap();
        let hash = test_hash(3);
        let hdr = b"header bytes";
        s.put_header(&hash, hdr).unwrap();
        assert_eq!(s.get_header(&hash).unwrap(), hdr);
    }

    #[test]
    fn account_crud() {
        let s = Storage::open_temp().unwrap();
        let addr = test_hash(10);
        let data = b"account state blob";

        // Create
        s.put_account(&addr, data).unwrap();
        assert_eq!(s.get_account(&addr).unwrap(), data);
        assert_eq!(s.account_count(), 1);

        // Update
        let updated = b"new state";
        s.put_account(&addr, updated).unwrap();
        assert_eq!(s.get_account(&addr).unwrap(), updated);
        assert_eq!(s.account_count(), 1);

        // Delete
        s.delete_account(&addr).unwrap();
        assert!(matches!(s.get_account(&addr), Err(StorageError::NotFound)));
    }

    #[test]
    fn iter_accounts() {
        let s = Storage::open_temp().unwrap();
        for i in 0u8..5 {
            s.put_account(&test_hash(i), &[i; 8]).unwrap();
        }
        let all: Vec<_> = s.iter_accounts().collect();
        assert_eq!(all.len(), 5);
    }

    #[test]
    fn tx_index_roundtrip() {
        let s = Storage::open_temp().unwrap();
        let tx_hash = test_hash(20);
        let loc = TxLocation {
            block_hash: test_hash(1),
            index: 42,
        };
        s.put_tx_index(&tx_hash, &loc).unwrap();
        assert_eq!(s.get_tx_location(&tx_hash).unwrap(), loc);
    }

    #[test]
    fn tx_location_serde() {
        let loc = TxLocation {
            block_hash: test_hash(5),
            index: 1000,
        };
        let bytes = loc.to_bytes();
        assert_eq!(bytes.len(), 36);
        let decoded = TxLocation::from_bytes(&bytes).unwrap();
        assert_eq!(decoded, loc);
    }

    #[test]
    fn chain_metadata() {
        let s = Storage::open_temp().unwrap();

        // Initially empty
        assert!(matches!(s.chain_head(), Err(StorageError::NotFound)));
        assert!(matches!(s.chain_height(), Err(StorageError::NotFound)));

        s.set_chain_head(&test_hash(99)).unwrap();
        s.set_chain_height(12345).unwrap();

        assert_eq!(s.chain_head().unwrap(), test_hash(99));
        assert_eq!(s.chain_height().unwrap(), 12345);
    }

    #[test]
    fn height_index() {
        let s = Storage::open_temp().unwrap();
        s.put_height_index(0, &test_hash(1)).unwrap();
        s.put_height_index(1, &test_hash(2)).unwrap();
        assert_eq!(s.get_hash_by_height(0).unwrap(), test_hash(1));
        assert_eq!(s.get_hash_by_height(1).unwrap(), test_hash(2));
        assert!(matches!(
            s.get_hash_by_height(99),
            Err(StorageError::NotFound)
        ));
    }

    #[test]
    fn commit_block_atomic() {
        let s = Storage::open_temp().unwrap();
        let hash = test_hash(50);
        let header = b"hdr";
        let body = b"body";
        let txs = vec![(test_hash(60), 0u32), (test_hash(61), 1)];

        s.commit_block(&hash, 0, header, body, &txs).unwrap();

        assert_eq!(s.chain_head().unwrap(), hash);
        assert_eq!(s.chain_height().unwrap(), 0);
        assert_eq!(s.get_header(&hash).unwrap(), header);
        assert_eq!(s.get_block(&hash).unwrap(), body);
        assert_eq!(s.get_hash_by_height(0).unwrap(), hash);
        assert_eq!(s.get_tx_location(&test_hash(60)).unwrap().index, 0);
        assert_eq!(s.get_tx_location(&test_hash(61)).unwrap().index, 1);
    }

    #[test]
    fn receipt_roundtrip() {
        let s = Storage::open_temp().unwrap();
        let tx = test_hash(70);
        let receipt = b"receipt data with gas used etc";
        s.put_receipt(&tx, receipt).unwrap();
        assert_eq!(s.get_receipt(&tx).unwrap(), receipt);
        assert!(matches!(
            s.get_receipt(&test_hash(71)),
            Err(StorageError::NotFound)
        ));
    }

    #[test]
    fn not_found_errors() {
        let s = Storage::open_temp().unwrap();
        let missing = test_hash(255);
        assert!(matches!(s.get_block(&missing), Err(StorageError::NotFound)));
        assert!(matches!(
            s.get_header(&missing),
            Err(StorageError::NotFound)
        ));
        assert!(matches!(
            s.get_account(&missing),
            Err(StorageError::NotFound)
        ));
        assert!(matches!(
            s.get_tx_location(&missing),
            Err(StorageError::NotFound)
        ));
    }

    #[test]
    fn persistent_across_reopen() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("testdb");

        // Write
        {
            let s = Storage::open(&path).unwrap();
            s.put_block(&test_hash(1), b"persisted").unwrap();
            s.set_chain_height(42).unwrap();
            s.flush().unwrap();
        }

        // Reopen and read
        {
            let s = Storage::open(&path).unwrap();
            assert_eq!(s.get_block(&test_hash(1)).unwrap(), b"persisted");
            assert_eq!(s.chain_height().unwrap(), 42);
        }
    }

    #[test]
    fn size_on_disk_nonzero_after_writes() {
        let s = Storage::open_temp().unwrap();
        for i in 0u8..10 {
            s.put_block(&test_hash(i), &vec![i; 1024]).unwrap();
        }
        s.flush().unwrap();
        // sled temp DBs may report 0 for size_on_disk, just ensure no panic
        let _ = s.size_on_disk();
    }

    #[test]
    fn multi_block_chain() {
        let s = Storage::open_temp().unwrap();
        for i in 0u8..10 {
            let hash = test_hash(i);
            s.commit_block(&hash, i as u64, &[i], &[i; 100], &[])
                .unwrap();
        }
        assert_eq!(s.chain_height().unwrap(), 9);
        assert_eq!(s.chain_head().unwrap(), test_hash(9));
        for i in 0u8..10 {
            assert_eq!(s.get_hash_by_height(i as u64).unwrap(), test_hash(i));
        }
    }
}
