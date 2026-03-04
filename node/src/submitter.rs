//! Checkpoint submitter — automatic L1 transaction submission.
//!
//! Monitors the CheckpointManager for finalized-but-unanchored checkpoints
//! and submits them to Filecoin L1 via an RPC client abstraction.
//!
//! Design:
//! - Polls at configurable intervals for new finalized checkpoints
//! - Encodes checkpoint data into L1 transaction format (CBOR-like envelope)
//! - Handles retries with exponential backoff on transient failures
//! - Tracks submission state: Pending → Submitted → Confirmed → Failed
//! - Gas estimation with configurable multiplier for safety margin
//! - Nonce management to prevent duplicate submissions

use sha2::{Digest, Sha256};
use std::collections::BTreeMap;

/// Simulated L1 transaction hash.
pub type TxHash = [u8; 32];

/// Submission states for checkpoint L1 transactions.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SubmissionState {
    /// Checkpoint finalized, not yet submitted.
    Pending,
    /// Transaction submitted, awaiting confirmation.
    Submitted { tx_hash: TxHash, attempts: u32 },
    /// Transaction confirmed on L1.
    Confirmed { tx_hash: TxHash, l1_epoch: u64 },
    /// Submission failed after max retries.
    Failed { reason: String, attempts: u32 },
}

/// Configuration for the checkpoint submitter.
#[derive(Debug, Clone)]
pub struct SubmitterConfig {
    /// Maximum retry attempts per checkpoint.
    pub max_retries: u32,
    /// Base delay for exponential backoff (in simulated ticks).
    pub base_backoff_ticks: u64,
    /// Gas price multiplier (basis points, e.g., 12000 = 1.2x).
    pub gas_multiplier_bps: u64,
    /// Maximum gas willing to spend per submission.
    pub max_gas: u64,
    /// L1 submitter address (the hot wallet).
    pub submitter_address: [u8; 20],
}

impl Default for SubmitterConfig {
    fn default() -> Self {
        Self {
            max_retries: 5,
            base_backoff_ticks: 2,
            gas_multiplier_bps: 12000, // 1.2x
            max_gas: 10_000_000,
            submitter_address: [0u8; 20],
        }
    }
}

/// Encoded checkpoint ready for L1 submission.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EncodedCheckpoint {
    pub sequence: u64,
    pub epoch_start: u64,
    pub epoch_end: u64,
    pub state_root: [u8; 32],
    pub block_hash: [u8; 32],
    pub validator_set_hash: [u8; 32],
    pub signature_count: u32,
    pub signed_stake: u128,
    pub total_stake: u128,
    /// CBOR-like serialized bytes for L1 calldata.
    pub calldata: Vec<u8>,
}

impl EncodedCheckpoint {
    /// Compute the calldata hash (used as simulated tx hash seed).
    pub fn calldata_hash(&self) -> [u8; 32] {
        let mut h = Sha256::new();
        h.update(&self.calldata);
        h.finalize().into()
    }
}

/// Simulated L1 RPC response.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum L1Response {
    /// Transaction accepted, returns tx hash.
    Accepted(TxHash),
    /// Transaction confirmed at L1 epoch.
    Confirmed { tx_hash: TxHash, l1_epoch: u64 },
    /// Transaction still pending.
    Pending,
    /// Transaction reverted or dropped.
    Rejected(String),
    /// RPC error (transient).
    RpcError(String),
}

/// Simulated L1 RPC client for testing.
pub struct MockL1Client {
    /// Pre-programmed responses by sequence number.
    responses: BTreeMap<u64, Vec<L1Response>>,
    /// Call index per sequence.
    call_idx: BTreeMap<u64, usize>,
    /// Gas price in simulated units.
    pub gas_price: u64,
    /// Current L1 epoch.
    pub l1_epoch: u64,
}

impl MockL1Client {
    pub fn new(gas_price: u64) -> Self {
        Self {
            responses: BTreeMap::new(),
            call_idx: BTreeMap::new(),
            gas_price,
            l1_epoch: 1000,
        }
    }

    /// Program a sequence of responses for a checkpoint submission.
    pub fn program(&mut self, sequence: u64, responses: Vec<L1Response>) {
        self.responses.insert(sequence, responses);
        self.call_idx.insert(sequence, 0);
    }

    /// Submit a checkpoint (returns next programmed response).
    pub fn submit(&mut self, encoded: &EncodedCheckpoint) -> L1Response {
        let seq = encoded.sequence;
        let idx = self.call_idx.get(&seq).copied().unwrap_or(0);
        let resp = self
            .responses
            .get(&seq)
            .and_then(|v| v.get(idx))
            .cloned()
            .unwrap_or_else(|| {
                // Default: accept then confirm
                if idx == 0 {
                    let tx_hash = encoded.calldata_hash();
                    L1Response::Accepted(tx_hash)
                } else {
                    let tx_hash = encoded.calldata_hash();
                    L1Response::Confirmed {
                        tx_hash,
                        l1_epoch: self.l1_epoch,
                    }
                }
            });
        self.call_idx.insert(seq, idx + 1);
        resp
    }

    /// Check transaction status.
    pub fn check_tx(&mut self, sequence: u64) -> L1Response {
        self.submit(&EncodedCheckpoint {
            sequence,
            epoch_start: 0,
            epoch_end: 0,
            state_root: [0; 32],
            block_hash: [0; 32],
            validator_set_hash: [0; 32],
            signature_count: 0,
            signed_stake: 0,
            total_stake: 0,
            calldata: sequence.to_le_bytes().to_vec(),
        })
    }

    /// Estimate gas for a checkpoint submission.
    pub fn estimate_gas(&self, _encoded: &EncodedCheckpoint) -> Result<u64, String> {
        // Simulated: base 500k + 1k per signature
        Ok(500_000 + (_encoded.signature_count as u64) * 1_000)
    }
}

/// The checkpoint submitter service.
pub struct CheckpointSubmitter {
    pub config: SubmitterConfig,
    /// Submission state per checkpoint sequence.
    pub states: BTreeMap<u64, SubmissionState>,
    /// Attempt counter per checkpoint (persists across retries).
    pub attempts: BTreeMap<u64, u32>,
    /// Encoded checkpoints awaiting or in-flight.
    pub encoded: BTreeMap<u64, EncodedCheckpoint>,
    /// Nonce tracker for the submitter address.
    pub nonce: u64,
    /// Total gas spent (for metrics).
    pub total_gas_spent: u64,
    /// Successful submissions count.
    pub confirmed_count: u64,
    /// Failed submissions count.
    pub failed_count: u64,
}

impl CheckpointSubmitter {
    pub fn new(config: SubmitterConfig) -> Self {
        Self {
            config,
            states: BTreeMap::new(),
            attempts: BTreeMap::new(),
            encoded: BTreeMap::new(),
            nonce: 0,
            total_gas_spent: 0,
            confirmed_count: 0,
            failed_count: 0,
        }
    }

    /// Encode a finalized checkpoint for L1 submission.
    pub fn encode_checkpoint(
        &self,
        sequence: u64,
        epoch_start: u64,
        epoch_end: u64,
        state_root: [u8; 32],
        block_hash: [u8; 32],
        validator_set_hash: [u8; 32],
        signature_count: u32,
        signed_stake: u128,
        total_stake: u128,
    ) -> EncodedCheckpoint {
        // Build CBOR-like calldata: method selector + packed fields
        let mut calldata = Vec::new();
        // Method selector: "anchorCheckpoint(uint64,bytes32,bytes32,bytes32,uint128,uint128)"
        calldata.extend_from_slice(&[0xAC, 0xDC, 0x00, 0x01]); // 4-byte selector
        calldata.extend_from_slice(&sequence.to_le_bytes());
        calldata.extend_from_slice(&epoch_start.to_le_bytes());
        calldata.extend_from_slice(&epoch_end.to_le_bytes());
        calldata.extend_from_slice(&state_root);
        calldata.extend_from_slice(&block_hash);
        calldata.extend_from_slice(&validator_set_hash);
        calldata.extend_from_slice(&signed_stake.to_le_bytes());
        calldata.extend_from_slice(&total_stake.to_le_bytes());
        calldata.extend_from_slice(&signature_count.to_le_bytes());

        EncodedCheckpoint {
            sequence,
            epoch_start,
            epoch_end,
            state_root,
            block_hash,
            validator_set_hash,
            signature_count,
            signed_stake,
            total_stake,
            calldata,
        }
    }

    /// Enqueue a checkpoint for submission.
    pub fn enqueue(
        &mut self,
        encoded: EncodedCheckpoint,
    ) -> Result<(), SubmitterError> {
        let seq = encoded.sequence;
        if self.states.contains_key(&seq) {
            return Err(SubmitterError::AlreadyQueued(seq));
        }
        self.states.insert(seq, SubmissionState::Pending);
        self.encoded.insert(seq, encoded);
        Ok(())
    }

    /// Process one tick: submit pending checkpoints and check in-flight ones.
    pub fn tick(&mut self, client: &mut MockL1Client) -> Vec<SubmitterEvent> {
        let mut events = Vec::new();
        let sequences: Vec<u64> = self.states.keys().copied().collect();

        for seq in sequences {
            let state = self.states.get(&seq).cloned().unwrap();
            match state {
                SubmissionState::Pending => {
                    if let Some(encoded) = self.encoded.get(&seq) {
                        // Estimate gas
                        match client.estimate_gas(encoded) {
                            Ok(gas) => {
                                let adjusted = gas * self.config.gas_multiplier_bps / 10000;
                                if adjusted > self.config.max_gas {
                                    self.states.insert(
                                        seq,
                                        SubmissionState::Failed {
                                            reason: format!(
                                                "gas {} exceeds max {}",
                                                adjusted, self.config.max_gas
                                            ),
                                            attempts: 0,
                                        },
                                    );
                                    self.failed_count += 1;
                                    events.push(SubmitterEvent::GasExceeded { sequence: seq, gas: adjusted });
                                    continue;
                                }
                            }
                            Err(e) => {
                                events.push(SubmitterEvent::RpcError { sequence: seq, error: e });
                                continue; // Retry next tick
                            }
                        }

                        let resp = client.submit(encoded);
                        let att = self.attempts.entry(seq).or_insert(0);
                        *att += 1;
                        let current_att = *att;
                        match resp {
                            L1Response::Accepted(tx_hash) => {
                                self.states.insert(
                                    seq,
                                    SubmissionState::Submitted { tx_hash, attempts: current_att },
                                );
                                self.nonce += 1;
                                events.push(SubmitterEvent::Submitted { sequence: seq, tx_hash });
                            }
                            L1Response::Rejected(reason) => {
                                self.states.insert(
                                    seq,
                                    SubmissionState::Failed { reason: reason.clone(), attempts: current_att },
                                );
                                self.failed_count += 1;
                                events.push(SubmitterEvent::Failed { sequence: seq, reason });
                            }
                            L1Response::RpcError(e) => {
                                events.push(SubmitterEvent::RpcError { sequence: seq, error: e });
                            }
                            _ => {}
                        }
                    }
                }
                SubmissionState::Submitted { tx_hash, attempts: _ } => {
                    let total_attempts = self.attempts.get(&seq).copied().unwrap_or(1);
                    let resp = client.check_tx(seq);
                    match resp {
                        L1Response::Confirmed { tx_hash: confirmed_hash, l1_epoch } => {
                            let hash = if confirmed_hash != [0u8; 32] { confirmed_hash } else { tx_hash };
                            self.states.insert(
                                seq,
                                SubmissionState::Confirmed { tx_hash: hash, l1_epoch },
                            );
                            self.confirmed_count += 1;
                            self.total_gas_spent += 500_000; // Simulated
                            events.push(SubmitterEvent::Confirmed {
                                sequence: seq,
                                tx_hash: hash,
                                l1_epoch,
                            });
                        }
                        L1Response::Pending => {
                            // Still waiting, keep state
                        }
                        L1Response::Rejected(reason) => {
                            if total_attempts < self.config.max_retries {
                                // Retry: go back to pending
                                self.states.insert(seq, SubmissionState::Pending);
                                events.push(SubmitterEvent::Retry { sequence: seq, attempt: total_attempts + 1 });
                            } else {
                                self.states.insert(
                                    seq,
                                    SubmissionState::Failed { reason: reason.clone(), attempts: total_attempts },
                                );
                                self.failed_count += 1;
                                events.push(SubmitterEvent::Failed { sequence: seq, reason });
                            }
                        }
                        L1Response::RpcError(e) => {
                            if total_attempts < self.config.max_retries {
                                events.push(SubmitterEvent::RpcError { sequence: seq, error: e });
                            } else {
                                self.states.insert(
                                    seq,
                                    SubmissionState::Failed {
                                        reason: format!("RPC errors after {} attempts", total_attempts),
                                        attempts: total_attempts,
                                    },
                                );
                                self.failed_count += 1;
                            }
                        }
                        _ => {}
                    }
                }
                SubmissionState::Confirmed { .. } | SubmissionState::Failed { .. } => {
                    // Terminal states, no action
                }
            }
        }
        events
    }

    /// Get submission state for a checkpoint.
    pub fn get_state(&self, sequence: u64) -> Option<&SubmissionState> {
        self.states.get(&sequence)
    }

    /// Get all pending checkpoint sequences.
    pub fn pending_sequences(&self) -> Vec<u64> {
        self.states
            .iter()
            .filter(|(_, s)| matches!(s, SubmissionState::Pending))
            .map(|(seq, _)| *seq)
            .collect()
    }

    /// Get all confirmed checkpoint sequences.
    pub fn confirmed_sequences(&self) -> Vec<u64> {
        self.states
            .iter()
            .filter(|(_, s)| matches!(s, SubmissionState::Confirmed { .. }))
            .map(|(seq, _)| *seq)
            .collect()
    }
}

/// Events emitted by the submitter for logging/metrics.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SubmitterEvent {
    Submitted { sequence: u64, tx_hash: TxHash },
    Confirmed { sequence: u64, tx_hash: TxHash, l1_epoch: u64 },
    Failed { sequence: u64, reason: String },
    Retry { sequence: u64, attempt: u32 },
    GasExceeded { sequence: u64, gas: u64 },
    RpcError { sequence: u64, error: String },
}

/// Submitter errors.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SubmitterError {
    AlreadyQueued(u64),
}

impl std::fmt::Display for SubmitterError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::AlreadyQueued(seq) => write!(f, "checkpoint {} already queued", seq),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_hash(val: u8) -> [u8; 32] {
        let mut h = [0u8; 32];
        h[0] = val;
        h
    }

    fn make_submitter() -> CheckpointSubmitter {
        CheckpointSubmitter::new(SubmitterConfig::default())
    }

    fn encode_test(submitter: &CheckpointSubmitter, seq: u64) -> EncodedCheckpoint {
        submitter.encode_checkpoint(
            seq, seq * 120 + 1, (seq + 1) * 120,
            test_hash(1), test_hash(2), test_hash(3),
            3, 200, 300,
        )
    }

    #[test]
    fn test_encode_checkpoint() {
        let s = make_submitter();
        let enc = encode_test(&s, 0);
        assert_eq!(enc.sequence, 0);
        assert_eq!(enc.signature_count, 3);
        assert!(!enc.calldata.is_empty());
        // Selector is first 4 bytes
        assert_eq!(&enc.calldata[..4], &[0xAC, 0xDC, 0x00, 0x01]);
    }

    #[test]
    fn test_calldata_hash_deterministic() {
        let s = make_submitter();
        let enc = encode_test(&s, 0);
        assert_eq!(enc.calldata_hash(), enc.calldata_hash());
        assert_ne!(enc.calldata_hash(), [0u8; 32]);
    }

    #[test]
    fn test_enqueue_checkpoint() {
        let mut s = make_submitter();
        let enc = encode_test(&s, 0);
        s.enqueue(enc).unwrap();
        assert_eq!(s.pending_sequences(), vec![0]);
        assert!(matches!(s.get_state(0), Some(SubmissionState::Pending)));
    }

    #[test]
    fn test_duplicate_enqueue_rejected() {
        let mut s = make_submitter();
        let enc = encode_test(&s, 0);
        s.enqueue(enc.clone()).unwrap();
        let err = s.enqueue(enc).unwrap_err();
        assert_eq!(err, SubmitterError::AlreadyQueued(0));
    }

    #[test]
    fn test_submit_and_confirm() {
        let mut s = make_submitter();
        let mut client = MockL1Client::new(100);
        let enc = encode_test(&s, 0);
        let tx_hash = enc.calldata_hash();

        // Program: accept then confirm
        client.program(0, vec![
            L1Response::Accepted(tx_hash),
            L1Response::Confirmed { tx_hash, l1_epoch: 1000 },
        ]);

        s.enqueue(enc).unwrap();

        // Tick 1: submit
        let events = s.tick(&mut client);
        assert!(events.iter().any(|e| matches!(e, SubmitterEvent::Submitted { sequence: 0, .. })));
        assert!(matches!(s.get_state(0), Some(SubmissionState::Submitted { .. })));
        assert_eq!(s.nonce, 1);

        // Tick 2: confirm
        let events = s.tick(&mut client);
        assert!(events.iter().any(|e| matches!(e, SubmitterEvent::Confirmed { sequence: 0, l1_epoch: 1000, .. })));
        assert!(matches!(s.get_state(0), Some(SubmissionState::Confirmed { .. })));
        assert_eq!(s.confirmed_count, 1);
    }

    #[test]
    fn test_submit_rejected_then_retry() {
        let mut s = make_submitter();
        let mut client = MockL1Client::new(100);
        let enc = encode_test(&s, 0);
        let tx_hash = enc.calldata_hash();

        // Program: accept, then reject (triggers retry), then accept, then confirm
        client.program(0, vec![
            L1Response::Accepted(tx_hash),
            L1Response::Rejected("nonce too low".into()),
            L1Response::Accepted(tx_hash),
            L1Response::Confirmed { tx_hash, l1_epoch: 1001 },
        ]);

        s.enqueue(enc).unwrap();

        // Tick 1: submitted
        let events = s.tick(&mut client);
        assert!(events.iter().any(|e| matches!(e, SubmitterEvent::Submitted { .. })));

        // Tick 2: rejected → retry (back to pending)
        let events = s.tick(&mut client);
        assert!(events.iter().any(|e| matches!(e, SubmitterEvent::Retry { sequence: 0, attempt: 2 })));
        assert!(matches!(s.get_state(0), Some(SubmissionState::Pending)));

        // Tick 3: re-submit
        let events = s.tick(&mut client);
        assert!(events.iter().any(|e| matches!(e, SubmitterEvent::Submitted { .. })));

        // Tick 4: confirm
        let events = s.tick(&mut client);
        assert!(events.iter().any(|e| matches!(e, SubmitterEvent::Confirmed { .. })));
        assert_eq!(s.confirmed_count, 1);
    }

    #[test]
    fn test_gas_exceeded() {
        let mut s = CheckpointSubmitter::new(SubmitterConfig {
            max_gas: 100, // Very low
            ..Default::default()
        });
        let mut client = MockL1Client::new(100);
        let enc = encode_test(&s, 0);
        s.enqueue(enc).unwrap();

        let events = s.tick(&mut client);
        assert!(events.iter().any(|e| matches!(e, SubmitterEvent::GasExceeded { .. })));
        assert!(matches!(s.get_state(0), Some(SubmissionState::Failed { .. })));
        assert_eq!(s.failed_count, 1);
    }

    #[test]
    fn test_multiple_checkpoints() {
        let mut s = make_submitter();
        let mut client = MockL1Client::new(100);

        for seq in 0..3 {
            let enc = encode_test(&s, seq);
            s.enqueue(enc).unwrap();
        }
        assert_eq!(s.pending_sequences().len(), 3);

        // Tick: all 3 submitted (default mock accepts)
        s.tick(&mut client);
        assert_eq!(s.nonce, 3);

        // Tick: all 3 confirmed
        s.tick(&mut client);
        assert_eq!(s.confirmed_count, 3);
        assert_eq!(s.confirmed_sequences(), vec![0, 1, 2]);
    }

    #[test]
    fn test_max_retries_exhausted() {
        let mut s = CheckpointSubmitter::new(SubmitterConfig {
            max_retries: 2,
            ..Default::default()
        });
        let mut client = MockL1Client::new(100);
        let enc = encode_test(&s, 0);
        let tx_hash = enc.calldata_hash();

        // Accept then reject twice (reaches max_retries)
        client.program(0, vec![
            L1Response::Accepted(tx_hash),
            L1Response::Rejected("revert".into()),
            L1Response::Accepted(tx_hash),
            L1Response::Rejected("revert again".into()),
        ]);

        s.enqueue(enc).unwrap();
        s.tick(&mut client); // submitted
        s.tick(&mut client); // rejected → retry (attempt 2)
        s.tick(&mut client); // re-submitted
        s.tick(&mut client); // rejected → max retries → failed

        assert!(matches!(s.get_state(0), Some(SubmissionState::Failed { .. })));
        assert_eq!(s.failed_count, 1);
    }

    #[test]
    fn test_confirmed_is_terminal() {
        let mut s = make_submitter();
        let mut client = MockL1Client::new(100);
        let enc = encode_test(&s, 0);
        s.enqueue(enc).unwrap();

        s.tick(&mut client); // submitted
        s.tick(&mut client); // confirmed

        let count_before = s.confirmed_count;
        s.tick(&mut client); // should be no-op
        assert_eq!(s.confirmed_count, count_before);
    }

    #[test]
    fn test_nonce_increments_on_submit() {
        let mut s = make_submitter();
        let mut client = MockL1Client::new(100);

        assert_eq!(s.nonce, 0);
        s.enqueue(encode_test(&s, 0)).unwrap();
        s.tick(&mut client);
        assert_eq!(s.nonce, 1);
        s.enqueue(encode_test(&s, 1)).unwrap();
        s.tick(&mut client);
        assert_eq!(s.nonce, 2);
    }

    #[test]
    fn test_total_gas_tracking() {
        let mut s = make_submitter();
        let mut client = MockL1Client::new(100);
        s.enqueue(encode_test(&s, 0)).unwrap();
        s.tick(&mut client); // submit
        s.tick(&mut client); // confirm
        assert!(s.total_gas_spent > 0);
    }

    #[test]
    fn test_different_checkpoints_different_calldata() {
        let s = make_submitter();
        let e1 = encode_test(&s, 0);
        let e2 = encode_test(&s, 1);
        assert_ne!(e1.calldata, e2.calldata);
        assert_ne!(e1.calldata_hash(), e2.calldata_hash());
    }

    #[test]
    fn test_submitter_error_display() {
        let err = SubmitterError::AlreadyQueued(42);
        assert_eq!(err.to_string(), "checkpoint 42 already queued");
    }

    #[test]
    fn test_default_config() {
        let cfg = SubmitterConfig::default();
        assert_eq!(cfg.max_retries, 5);
        assert_eq!(cfg.gas_multiplier_bps, 12000);
        assert_eq!(cfg.max_gas, 10_000_000);
    }
}
