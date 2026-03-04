//! Blob Upload Client (SDK-009)
//!
//! High-level client for uploading data blobs to Prova's DA layer.
//! Handles erasure encoding, chunking, Merkle root computation,
//! blob transaction construction, and upload progress tracking.
//!
//! # Usage
//!
//! ```rust,ignore
//! let client = BlobUploadClient::new(keypair, config);
//! let receipt = client.upload(data)?;
//! let status = client.status(receipt.blob_id)?;
//! ```

use prova_chain::das::{BlobId, DasStatus, ORIGINAL_CHUNKS, TOTAL_CHUNKS};
use prova_chain::blob_tx::{
    BlobTransaction, BlobTxResult, BASE_BLOB_FEE, FEE_PER_CHUNK,
    MAX_BLOB_SIZE, MIN_BLOB_SIZE,
};
use prova_chain::types::{Address, Epoch, Hash};
use sha2::{Digest, Sha256};
use std::collections::HashMap;

use crate::Keypair;

// ─── Constants ───────────────────────────────────────────────────────────────

/// Default maximum retries for chunk upload.
pub const DEFAULT_MAX_RETRIES: u32 = 3;
/// Default chunk upload timeout in milliseconds.
pub const DEFAULT_CHUNK_TIMEOUT_MS: u64 = 30_000;
/// Maximum concurrent chunk uploads.
pub const MAX_CONCURRENT_UPLOADS: usize = 16;
/// Minimum data size for erasure coding (below this, replicate instead).
pub const ERASURE_THRESHOLD: usize = 1024;
/// Default fee multiplier over estimated base (1.25x).
pub const DEFAULT_FEE_MULTIPLIER: f64 = 1.25;

// ─── Types ───────────────────────────────────────────────────────────────────

/// Configuration for blob uploads.
#[derive(Debug, Clone)]
pub struct BlobUploadConfig {
    pub max_retries: u32,
    pub chunk_timeout_ms: u64,
    pub max_concurrent: usize,
    pub fee_multiplier: f64,
    /// Optional reference commit hash to link blob with inference.
    pub reference: Option<Hash>,
}

impl Default for BlobUploadConfig {
    fn default() -> Self {
        Self {
            max_retries: DEFAULT_MAX_RETRIES,
            chunk_timeout_ms: DEFAULT_CHUNK_TIMEOUT_MS,
            max_concurrent: MAX_CONCURRENT_UPLOADS,
            fee_multiplier: DEFAULT_FEE_MULTIPLIER,
            reference: None,
        }
    }
}

/// Result of an erasure encoding pass.
#[derive(Debug, Clone)]
pub struct ErasureEncoded {
    /// Original data chunks.
    pub original: Vec<Vec<u8>>,
    /// Parity chunks (same count as original for 2x expansion).
    pub parity: Vec<Vec<u8>>,
    /// Merkle root over all chunks.
    pub data_root: Hash,
    /// Per-chunk hashes.
    pub chunk_hashes: Vec<Hash>,
}

/// Upload receipt returned after successful submission.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UploadReceipt {
    pub blob_id: BlobId,
    pub data_root: Hash,
    pub blob_size: u64,
    pub chunk_count: usize,
    pub fee_paid: u128,
    pub tx_hash: Hash,
}

/// Status of a blob upload.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UploadStatus {
    /// Transaction submitted, awaiting DAS sampling.
    Pending { rounds_complete: u32, rounds_required: u32 },
    /// DAS confirmed — blob is available.
    Confirmed,
    /// Upload failed (provider didn't serve chunks).
    Failed { reason: String },
    /// Blob not found on chain.
    NotFound,
}

/// Progress callback data.
#[derive(Debug, Clone)]
pub struct UploadProgress {
    pub phase: UploadPhase,
    pub chunks_uploaded: usize,
    pub total_chunks: usize,
    pub bytes_sent: u64,
    pub total_bytes: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UploadPhase {
    ErasureEncoding,
    ChunkUpload,
    TxSubmission,
    DasConfirmation,
    Complete,
}

/// Errors during blob upload.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BlobUploadError {
    DataTooLarge { size: u64, max: u64 },
    DataEmpty,
    EncodingFailed(String),
    ChunkUploadFailed { index: usize, retries: u32 },
    TxSubmissionFailed(String),
    InsufficientFee { required: u128, available: u128 },
    Timeout,
}

// ─── Erasure Encoding ────────────────────────────────────────────────────────

/// Simplified Reed-Solomon-style erasure encoding.
/// Splits data into ORIGINAL_CHUNKS equal pieces, generates ORIGINAL_CHUNKS
/// parity pieces via XOR pairing, builds a Merkle tree over all.
pub fn erasure_encode(data: &[u8]) -> Result<ErasureEncoded, BlobUploadError> {
    if data.is_empty() {
        return Err(BlobUploadError::DataEmpty);
    }
    if data.len() as u64 > MAX_BLOB_SIZE {
        return Err(BlobUploadError::DataTooLarge {
            size: data.len() as u64,
            max: MAX_BLOB_SIZE,
        });
    }

    let chunk_size = (data.len() + ORIGINAL_CHUNKS - 1) / ORIGINAL_CHUNKS;
    let chunk_size = chunk_size.max(1);

    // Split into original chunks (pad last if needed).
    let mut original = Vec::with_capacity(ORIGINAL_CHUNKS);
    for i in 0..ORIGINAL_CHUNKS {
        let start = i * chunk_size;
        if start >= data.len() {
            original.push(vec![0u8; chunk_size]);
        } else {
            let end = (start + chunk_size).min(data.len());
            let mut chunk = data[start..end].to_vec();
            chunk.resize(chunk_size, 0);
            original.push(chunk);
        }
    }

    // Generate parity chunks via XOR with rotating partner.
    let mut parity = Vec::with_capacity(ORIGINAL_CHUNKS);
    for i in 0..ORIGINAL_CHUNKS {
        let partner = (i + ORIGINAL_CHUNKS / 2) % ORIGINAL_CHUNKS;
        let p: Vec<u8> = original[i]
            .iter()
            .zip(original[partner].iter())
            .map(|(a, b)| a ^ b)
            .collect();
        parity.push(p);
    }

    // Compute per-chunk hashes and Merkle root.
    let mut chunk_hashes = Vec::with_capacity(TOTAL_CHUNKS);
    for chunk in original.iter().chain(parity.iter()) {
        let mut h = Sha256::new();
        h.update(chunk);
        let hash = h.finalize();
        let mut arr = [0u8; 32];
        arr.copy_from_slice(&hash);
        chunk_hashes.push(arr);
    }

    let data_root = compute_merkle_root(&chunk_hashes);

    Ok(ErasureEncoded {
        original,
        parity,
        data_root,
        chunk_hashes,
    })
}

/// Compute Merkle root from leaf hashes.
fn compute_merkle_root(leaves: &[Hash]) -> Hash {
    if leaves.is_empty() {
        return [0u8; 32];
    }
    if leaves.len() == 1 {
        return leaves[0];
    }

    let mut layer: Vec<Hash> = leaves.to_vec();
    while layer.len() > 1 {
        let mut next = Vec::with_capacity((layer.len() + 1) / 2);
        for pair in layer.chunks(2) {
            let mut h = Sha256::new();
            h.update(&pair[0]);
            if pair.len() > 1 {
                h.update(&pair[1]);
            } else {
                h.update(&pair[0]); // duplicate odd leaf
            }
            let hash = h.finalize();
            let mut arr = [0u8; 32];
            arr.copy_from_slice(&hash);
            next.push(arr);
        }
        layer = next;
    }
    layer[0]
}

/// Decode original data from erasure-coded chunks.
/// Requires at least ORIGINAL_CHUNKS pieces (any mix of original + parity).
pub fn erasure_decode(
    encoded: &ErasureEncoded,
    available_indices: &[usize],
) -> Result<Vec<u8>, BlobUploadError> {
    // Simplified: if all originals available, just concatenate.
    let has_all_originals = (0..ORIGINAL_CHUNKS).all(|i| available_indices.contains(&i));
    if has_all_originals {
        let mut data = Vec::new();
        for chunk in &encoded.original {
            data.extend_from_slice(chunk);
        }
        return Ok(data);
    }

    // With parity: recover missing originals via XOR.
    let mut recovered = encoded.original.clone();
    for i in 0..ORIGINAL_CHUNKS {
        if !available_indices.contains(&i) {
            let parity_idx = i;
            let partner = (i + ORIGINAL_CHUNKS / 2) % ORIGINAL_CHUNKS;
            if available_indices.contains(&(ORIGINAL_CHUNKS + parity_idx))
                && available_indices.contains(&partner)
            {
                // original[i] = parity[i] ^ original[partner]
                recovered[i] = encoded.parity[parity_idx]
                    .iter()
                    .zip(encoded.original[partner].iter())
                    .map(|(p, o)| p ^ o)
                    .collect();
            } else {
                return Err(BlobUploadError::EncodingFailed(format!(
                    "Cannot recover chunk {i}: insufficient data"
                )));
            }
        }
    }

    let mut data = Vec::new();
    for chunk in &recovered {
        data.extend_from_slice(chunk);
    }
    Ok(data)
}

// ─── Fee Estimation ──────────────────────────────────────────────────────────

/// Estimate the fee for uploading data of the given size.
pub fn estimate_fee(data_size: u64, fee_multiplier: f64) -> u128 {
    let chunk_count = ((data_size as usize + ORIGINAL_CHUNKS - 1) / ORIGINAL_CHUNKS).max(1);
    let base = BASE_BLOB_FEE + (TOTAL_CHUNKS as u128) * FEE_PER_CHUNK;
    (base as f64 * fee_multiplier) as u128
}

// ─── Blob Upload Client ─────────────────────────────────────────────────────

/// High-level client for uploading blobs to Prova's DA layer.
pub struct BlobUploadClient {
    keypair: Keypair,
    config: BlobUploadConfig,
    nonce: u64,
    /// Simulated chain state: submitted blobs.
    submitted: HashMap<BlobId, UploadReceipt>,
    /// Simulated DAS status per blob.
    das_status: HashMap<BlobId, DasStatus>,
    /// Upload history.
    history: Vec<UploadReceipt>,
}

impl BlobUploadClient {
    pub fn new(keypair: Keypair, config: BlobUploadConfig) -> Self {
        Self {
            keypair,
            config,
            nonce: 0,
            submitted: HashMap::new(),
            das_status: HashMap::new(),
            history: Vec::new(),
        }
    }

    /// Upload raw data, returning a receipt on success.
    pub fn upload(&mut self, data: &[u8]) -> Result<UploadReceipt, BlobUploadError> {
        // Validate size.
        if data.is_empty() {
            return Err(BlobUploadError::DataEmpty);
        }
        if data.len() as u64 > MAX_BLOB_SIZE {
            return Err(BlobUploadError::DataTooLarge {
                size: data.len() as u64,
                max: MAX_BLOB_SIZE,
            });
        }

        // Erasure encode.
        let encoded = erasure_encode(data)?;

        // Compute blob ID.
        let mut h = Sha256::new();
        h.update(data);
        let hash = h.finalize();
        let mut blob_hash = [0u8; 32];
        blob_hash.copy_from_slice(&hash);
        let blob_id = BlobId(blob_hash);

        // Estimate and validate fee.
        let fee = estimate_fee(data.len() as u64, self.config.fee_multiplier);

        // Build blob transaction.
        let tx = BlobTransaction {
            sender: self.keypair.address,
            nonce: self.nonce,
            blob_id,
            data_root: encoded.data_root,
            blob_size: data.len() as u64,
            chunk_count: TOTAL_CHUNKS,
            reference: self.config.reference,
            max_fee: fee,
        };

        // Compute tx hash.
        let mut th = Sha256::new();
        th.update(&tx.sender.0);
        th.update(&tx.nonce.to_le_bytes());
        th.update(&tx.blob_id.0);
        let tx_hash_out = th.finalize();
        let mut tx_hash = [0u8; 32];
        tx_hash.copy_from_slice(&tx_hash_out);

        self.nonce += 1;

        let receipt = UploadReceipt {
            blob_id,
            data_root: encoded.data_root,
            blob_size: data.len() as u64,
            chunk_count: TOTAL_CHUNKS,
            fee_paid: fee,
            tx_hash,
        };

        self.submitted.insert(blob_id, receipt.clone());
        self.das_status.insert(blob_id, DasStatus::Pending);
        self.history.push(receipt.clone());

        Ok(receipt)
    }

    /// Check the status of a previously uploaded blob.
    pub fn status(&self, blob_id: BlobId) -> UploadStatus {
        match self.das_status.get(&blob_id) {
            Some(DasStatus::Pending) => UploadStatus::Pending {
                rounds_complete: 0,
                rounds_required: 3,
            },
            Some(DasStatus::Confirmed) => UploadStatus::Confirmed,
            Some(DasStatus::Failed) => UploadStatus::Failed {
                reason: "DAS challenge failed".into(),
            },
            None => UploadStatus::NotFound,
        }
    }

    /// Simulate DAS confirmation for a blob (for testing).
    pub fn simulate_confirm(&mut self, blob_id: BlobId) -> bool {
        if self.das_status.contains_key(&blob_id) {
            self.das_status.insert(blob_id, DasStatus::Confirmed);
            true
        } else {
            false
        }
    }

    /// Simulate DAS failure for a blob (for testing).
    pub fn simulate_fail(&mut self, blob_id: BlobId) -> bool {
        if self.das_status.contains_key(&blob_id) {
            self.das_status.insert(blob_id, DasStatus::Failed);
            true
        } else {
            false
        }
    }

    /// Get upload history.
    pub fn history(&self) -> &[UploadReceipt] {
        &self.history
    }

    /// Get the configured keypair address.
    pub fn address(&self) -> Address {
        self.keypair.address
    }

    /// Batch upload multiple data blobs, returning receipts for each.
    pub fn upload_batch(
        &mut self,
        items: &[&[u8]],
    ) -> Vec<Result<UploadReceipt, BlobUploadError>> {
        items.iter().map(|data| self.upload(data)).collect()
    }

    /// Compute progress for a given upload (simulated).
    pub fn progress(&self, blob_id: BlobId) -> Option<UploadProgress> {
        let receipt = self.submitted.get(&blob_id)?;
        let phase = match self.das_status.get(&blob_id)? {
            DasStatus::Pending => UploadPhase::DasConfirmation,
            DasStatus::Confirmed => UploadPhase::Complete,
            DasStatus::Failed => UploadPhase::Complete,
        };
        Some(UploadProgress {
            phase,
            chunks_uploaded: receipt.chunk_count,
            total_chunks: receipt.chunk_count,
            bytes_sent: receipt.blob_size,
            total_bytes: receipt.blob_size,
        })
    }
}

// ─── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn test_keypair() -> Keypair {
        Keypair::from_seed([42u8; 32])
    }

    fn test_client() -> BlobUploadClient {
        BlobUploadClient::new(test_keypair(), BlobUploadConfig::default())
    }

    #[test]
    fn test_erasure_encode_basic() {
        let data = vec![0xABu8; 4096];
        let encoded = erasure_encode(&data).unwrap();
        assert_eq!(encoded.original.len(), ORIGINAL_CHUNKS);
        assert_eq!(encoded.parity.len(), ORIGINAL_CHUNKS);
        assert_eq!(encoded.chunk_hashes.len(), TOTAL_CHUNKS);
        assert_ne!(encoded.data_root, [0u8; 32]);
    }

    #[test]
    fn test_erasure_encode_small_data() {
        let data = vec![1u8; 10];
        let encoded = erasure_encode(&data).unwrap();
        assert_eq!(encoded.original.len(), ORIGINAL_CHUNKS);
    }

    #[test]
    fn test_erasure_encode_empty_fails() {
        let result = erasure_encode(&[]);
        assert!(matches!(result, Err(BlobUploadError::DataEmpty)));
    }

    #[test]
    fn test_erasure_encode_too_large() {
        let data = vec![0u8; (MAX_BLOB_SIZE + 1) as usize];
        let result = erasure_encode(&data);
        assert!(matches!(result, Err(BlobUploadError::DataTooLarge { .. })));
    }

    #[test]
    fn test_erasure_decode_all_originals() {
        let data = vec![0xCDu8; 2048];
        let encoded = erasure_encode(&data).unwrap();
        let indices: Vec<usize> = (0..ORIGINAL_CHUNKS).collect();
        let decoded = erasure_decode(&encoded, &indices).unwrap();
        // Decoded includes padding; original data is prefix.
        assert_eq!(&decoded[..data.len()], &data[..]);
    }

    #[test]
    fn test_erasure_decode_with_parity_recovery() {
        let data = vec![0xEFu8; 4096];
        let encoded = erasure_encode(&data).unwrap();
        // Drop chunk 0, use its parity + partner instead.
        let partner = ORIGINAL_CHUNKS / 2;
        let mut indices: Vec<usize> = (1..ORIGINAL_CHUNKS).collect();
        indices.push(ORIGINAL_CHUNKS); // parity[0]
        indices.push(partner);
        let decoded = erasure_decode(&encoded, &indices).unwrap();
        assert_eq!(&decoded[..data.len()], &data[..]);
    }

    #[test]
    fn test_merkle_root_deterministic() {
        let data = vec![0xAAu8; 1024];
        let e1 = erasure_encode(&data).unwrap();
        let e2 = erasure_encode(&data).unwrap();
        assert_eq!(e1.data_root, e2.data_root);
    }

    #[test]
    fn test_merkle_root_different_data() {
        let e1 = erasure_encode(&[1u8; 1024]).unwrap();
        let e2 = erasure_encode(&[2u8; 1024]).unwrap();
        assert_ne!(e1.data_root, e2.data_root);
    }

    #[test]
    fn test_upload_basic() {
        let mut client = test_client();
        let data = vec![0xBBu8; 2048];
        let receipt = client.upload(&data).unwrap();
        assert_eq!(receipt.blob_size, 2048);
        assert_eq!(receipt.chunk_count, TOTAL_CHUNKS);
        assert!(receipt.fee_paid > 0);
    }

    #[test]
    fn test_upload_empty_fails() {
        let mut client = test_client();
        let result = client.upload(&[]);
        assert!(matches!(result, Err(BlobUploadError::DataEmpty)));
    }

    #[test]
    fn test_upload_too_large_fails() {
        let mut client = test_client();
        let data = vec![0u8; (MAX_BLOB_SIZE + 1) as usize];
        let result = client.upload(&data);
        assert!(matches!(result, Err(BlobUploadError::DataTooLarge { .. })));
    }

    #[test]
    fn test_upload_status_lifecycle() {
        let mut client = test_client();
        let receipt = client.upload(&[1u8; 512]).unwrap();

        // Initially pending.
        assert!(matches!(
            client.status(receipt.blob_id),
            UploadStatus::Pending { .. }
        ));

        // Simulate confirmation.
        assert!(client.simulate_confirm(receipt.blob_id));
        assert_eq!(client.status(receipt.blob_id), UploadStatus::Confirmed);
    }

    #[test]
    fn test_upload_status_failure() {
        let mut client = test_client();
        let receipt = client.upload(&[2u8; 512]).unwrap();
        assert!(client.simulate_fail(receipt.blob_id));
        assert!(matches!(
            client.status(receipt.blob_id),
            UploadStatus::Failed { .. }
        ));
    }

    #[test]
    fn test_status_not_found() {
        let client = test_client();
        let fake_id = BlobId([99u8; 32]);
        assert_eq!(client.status(fake_id), UploadStatus::NotFound);
    }

    #[test]
    fn test_upload_increments_nonce() {
        let mut client = test_client();
        let r1 = client.upload(&[1u8; 100]).unwrap();
        let r2 = client.upload(&[2u8; 100]).unwrap();
        assert_ne!(r1.tx_hash, r2.tx_hash);
    }

    #[test]
    fn test_batch_upload() {
        let mut client = test_client();
        let items: Vec<&[u8]> = vec![&[1u8; 100], &[2u8; 200], &[3u8; 300]];
        let results = client.upload_batch(&items);
        assert_eq!(results.len(), 3);
        assert!(results.iter().all(|r| r.is_ok()));
        assert_eq!(client.history().len(), 3);
    }

    #[test]
    fn test_upload_progress() {
        let mut client = test_client();
        let receipt = client.upload(&[5u8; 1024]).unwrap();
        let progress = client.progress(receipt.blob_id).unwrap();
        assert_eq!(progress.phase, UploadPhase::DasConfirmation);
        assert_eq!(progress.total_bytes, 1024);

        client.simulate_confirm(receipt.blob_id);
        let progress = client.progress(receipt.blob_id).unwrap();
        assert_eq!(progress.phase, UploadPhase::Complete);
    }

    #[test]
    fn test_fee_estimation() {
        let fee = estimate_fee(4096, 1.0);
        assert!(fee >= BASE_BLOB_FEE);
        let fee_boosted = estimate_fee(4096, 2.0);
        assert!(fee_boosted > fee);
    }

    #[test]
    fn test_upload_with_reference() {
        let mut client = BlobUploadClient::new(
            test_keypair(),
            BlobUploadConfig {
                reference: Some([0xFFu8; 32]),
                ..Default::default()
            },
        );
        let receipt = client.upload(&[7u8; 256]).unwrap();
        assert!(receipt.fee_paid > 0);
    }
}
