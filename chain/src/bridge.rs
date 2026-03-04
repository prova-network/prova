//! Cross-chain bridge message format (Prova ↔ Filecoin).
//!
//! Defines the message envelope, state proofs, and relay logic for
//! bidirectional communication between Prova L2 and Filecoin L1.
//!
//! Design:
//! - Messages are Merkle-proven against checkpoint state roots
//! - Outbound (Prova→Filecoin): queued in an outbox trie, proven against anchored checkpoints
//! - Inbound (Filecoin→Prova): L1 events parsed and verified against Filecoin tipset CIDs
//! - Nonce-ordered per (source_chain, sender) to prevent replay
//! - TTL-bounded: messages expire if not relayed within `MAX_MESSAGE_AGE` epochs

use crate::types::{Address, Epoch, Hash};
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;

/// Maximum age (in epochs) before an unrelayed message expires.
pub const MAX_MESSAGE_AGE: Epoch = 2880; // ~1 day at 30s epochs

/// Chain identifiers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ChainId {
    Prova,
    Filecoin,
}

/// Cross-chain message payload types.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MessagePayload {
    /// Transfer tokens between chains.
    TokenTransfer { recipient: Address, amount: u128 },
    /// Relay a checkpoint attestation to L1.
    CheckpointAttestation { sequence: u64, state_root: Hash },
    /// Relay a dispute result to L1 (for slashing finality).
    DisputeResult { commit_id: u64, slashed: Address, amount: u128 },
    /// Relay an L1 stake deposit notification to Prova.
    StakeDeposit { staker: Address, amount: u128 },
    /// Relay an L1 governance action to Prova.
    GovernanceAction { proposal_id: u64, action_hash: Hash },
    /// Generic data relay.
    RawData(Vec<u8>),
}

/// A cross-chain bridge message.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BridgeMessage {
    /// Source chain.
    pub source: ChainId,
    /// Destination chain.
    pub destination: ChainId,
    /// Sender on source chain.
    pub sender: Address,
    /// Per-sender nonce (monotonically increasing).
    pub nonce: u64,
    /// Epoch when the message was created.
    pub created_at: Epoch,
    /// Payload.
    pub payload: MessagePayload,
}

impl BridgeMessage {
    /// Compute the message hash (used for Merkle inclusion).
    pub fn hash(&self) -> Hash {
        let mut h = Sha256::new();
        h.update(match self.source {
            ChainId::Prova => [0u8],
            ChainId::Filecoin => [1u8],
        });
        h.update(match self.destination {
            ChainId::Prova => [0u8],
            ChainId::Filecoin => [1u8],
        });
        h.update(self.sender.0);
        h.update(self.nonce.to_le_bytes());
        h.update(self.created_at.to_le_bytes());
        // Hash the payload discriminant + key fields
        match &self.payload {
            MessagePayload::TokenTransfer { recipient, amount } => {
                h.update([0u8]);
                h.update(recipient.0);
                h.update(amount.to_le_bytes());
            }
            MessagePayload::CheckpointAttestation { sequence, state_root } => {
                h.update([1u8]);
                h.update(sequence.to_le_bytes());
                h.update(state_root);
            }
            MessagePayload::DisputeResult { commit_id, slashed, amount } => {
                h.update([2u8]);
                h.update(commit_id.to_le_bytes());
                h.update(slashed.0);
                h.update(amount.to_le_bytes());
            }
            MessagePayload::StakeDeposit { staker, amount } => {
                h.update([3u8]);
                h.update(staker.0);
                h.update(amount.to_le_bytes());
            }
            MessagePayload::GovernanceAction { proposal_id, action_hash } => {
                h.update([4u8]);
                h.update(proposal_id.to_le_bytes());
                h.update(action_hash);
            }
            MessagePayload::RawData(data) => {
                h.update([5u8]);
                h.update((data.len() as u64).to_le_bytes());
                h.update(data);
            }
        }
        h.finalize().into()
    }

    /// Check if the message has expired relative to `current_epoch`.
    pub fn is_expired(&self, current_epoch: Epoch) -> bool {
        current_epoch > self.created_at && current_epoch - self.created_at > MAX_MESSAGE_AGE
    }
}

/// Merkle proof for a bridge message against a state root.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StateProof {
    /// The message hash being proven.
    pub message_hash: Hash,
    /// Merkle path (sibling hashes, bottom-up).
    pub siblings: Vec<Hash>,
    /// Index in the leaf layer.
    pub index: u64,
    /// Total leaves in the tree.
    pub total_leaves: u64,
}

impl StateProof {
    /// Verify this proof against an expected root.
    pub fn verify(&self, expected_root: Hash) -> bool {
        if self.total_leaves == 0 {
            return false;
        }
        let mut current = self.message_hash;
        let mut idx = self.index;
        for sibling in &self.siblings {
            let mut h = Sha256::new();
            if idx % 2 == 0 {
                h.update(current);
                h.update(sibling);
            } else {
                h.update(sibling);
                h.update(current);
            }
            current = h.finalize().into();
            idx /= 2;
        }
        current == expected_root
    }
}

/// Outbox — accumulates outbound messages and builds Merkle roots.
#[derive(Debug)]
pub struct Outbox {
    /// Queued messages not yet included in a checkpoint.
    pub pending: Vec<BridgeMessage>,
    /// Nonce tracker per sender.
    pub nonces: BTreeMap<Address, u64>,
    /// Finalized message batches (checkpoint_seq → batch root + messages).
    pub batches: BTreeMap<u64, (Hash, Vec<BridgeMessage>)>,
}

impl Outbox {
    pub fn new() -> Self {
        Self {
            pending: Vec::new(),
            nonces: BTreeMap::new(),
            batches: BTreeMap::new(),
        }
    }

    /// Queue a message for outbound relay. Validates nonce ordering.
    pub fn queue(&mut self, msg: BridgeMessage) -> Result<Hash, BridgeError> {
        if msg.source == msg.destination {
            return Err(BridgeError::SameChain);
        }
        let expected_nonce = self.nonces.get(&msg.sender).copied().unwrap_or(0);
        if msg.nonce != expected_nonce {
            return Err(BridgeError::InvalidNonce {
                expected: expected_nonce,
                got: msg.nonce,
            });
        }
        let hash = msg.hash();
        self.nonces.insert(msg.sender, expected_nonce + 1);
        self.pending.push(msg);
        Ok(hash)
    }

    /// Seal pending messages into a batch for a checkpoint.
    /// Returns the Merkle root of the batch (or zero hash if empty).
    pub fn seal_batch(&mut self, checkpoint_seq: u64) -> Hash {
        if self.pending.is_empty() {
            return [0u8; 32];
        }
        let batch: Vec<BridgeMessage> = self.pending.drain(..).collect();
        let root = compute_merkle_root(&batch.iter().map(|m| m.hash()).collect::<Vec<_>>());
        self.batches.insert(checkpoint_seq, (root, batch));
        root
    }

    /// Generate a state proof for a message in a sealed batch.
    pub fn prove(
        &self,
        checkpoint_seq: u64,
        message_index: usize,
    ) -> Result<StateProof, BridgeError> {
        let (_, messages) = self
            .batches
            .get(&checkpoint_seq)
            .ok_or(BridgeError::BatchNotFound)?;
        if message_index >= messages.len() {
            return Err(BridgeError::MessageNotFound);
        }
        let leaves: Vec<Hash> = messages.iter().map(|m| m.hash()).collect();
        let proof = build_merkle_proof(&leaves, message_index);
        Ok(proof)
    }

    /// Total pending messages.
    pub fn pending_count(&self) -> usize {
        self.pending.len()
    }

    /// Total sealed batches.
    pub fn batch_count(&self) -> usize {
        self.batches.len()
    }
}

/// Inbox — receives and validates inbound messages with state proofs.
#[derive(Debug)]
pub struct Inbox {
    /// Processed message hashes (replay protection).
    pub processed: BTreeMap<Hash, Epoch>,
    /// Expected nonces per (sender).
    pub nonces: BTreeMap<Address, u64>,
}

impl Inbox {
    pub fn new() -> Self {
        Self {
            processed: BTreeMap::new(),
            nonces: BTreeMap::new(),
        }
    }

    /// Receive a bridged message with its state proof.
    /// Verifies: proof validity, nonce, expiry, replay.
    pub fn receive(
        &mut self,
        msg: &BridgeMessage,
        proof: &StateProof,
        batch_root: Hash,
        current_epoch: Epoch,
    ) -> Result<(), BridgeError> {
        // Check expiry
        if msg.is_expired(current_epoch) {
            return Err(BridgeError::Expired);
        }

        let msg_hash = msg.hash();

        // Replay protection
        if self.processed.contains_key(&msg_hash) {
            return Err(BridgeError::Replay);
        }

        // Verify Merkle proof
        if proof.message_hash != msg_hash {
            return Err(BridgeError::ProofMismatch);
        }
        if !proof.verify(batch_root) {
            return Err(BridgeError::InvalidProof);
        }

        // Nonce check
        let expected = self.nonces.get(&msg.sender).copied().unwrap_or(0);
        if msg.nonce != expected {
            return Err(BridgeError::InvalidNonce {
                expected,
                got: msg.nonce,
            });
        }

        self.nonces.insert(msg.sender, expected + 1);
        self.processed.insert(msg_hash, current_epoch);
        Ok(())
    }

    /// Number of processed messages.
    pub fn processed_count(&self) -> usize {
        self.processed.len()
    }
}

/// Compute Merkle root from leaf hashes.
pub fn compute_merkle_root(leaves: &[Hash]) -> Hash {
    if leaves.is_empty() {
        return [0u8; 32];
    }
    if leaves.len() == 1 {
        return leaves[0];
    }
    // Pad to power of 2
    let mut layer: Vec<Hash> = leaves.to_vec();
    while layer.len().count_ones() != 1 {
        layer.push([0u8; 32]);
    }
    while layer.len() > 1 {
        let mut next = Vec::new();
        for pair in layer.chunks(2) {
            let mut h = Sha256::new();
            h.update(pair[0]);
            h.update(pair[1]);
            next.push(h.finalize().into());
        }
        layer = next;
    }
    layer[0]
}

/// Build a Merkle proof for a leaf at `index`.
fn build_merkle_proof(leaves: &[Hash], index: usize) -> StateProof {
    let msg_hash = leaves[index];
    let mut padded: Vec<Hash> = leaves.to_vec();
    let total_leaves = padded.len() as u64;
    while padded.len().count_ones() != 1 {
        padded.push([0u8; 32]);
    }

    let mut siblings = Vec::new();
    let mut layer = padded;
    let mut idx = index;

    while layer.len() > 1 {
        let sibling_idx = if idx % 2 == 0 { idx + 1 } else { idx - 1 };
        siblings.push(layer[sibling_idx]);
        let mut next = Vec::new();
        for pair in layer.chunks(2) {
            let mut h = Sha256::new();
            h.update(pair[0]);
            h.update(pair[1]);
            next.push(h.finalize().into());
        }
        layer = next;
        idx /= 2;
    }

    StateProof {
        message_hash: msg_hash,
        siblings,
        index: index as u64,
        total_leaves,
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BridgeError {
    SameChain,
    InvalidNonce { expected: u64, got: u64 },
    Expired,
    Replay,
    InvalidProof,
    ProofMismatch,
    BatchNotFound,
    MessageNotFound,
}

impl std::fmt::Display for BridgeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::SameChain => write!(f, "source and destination are the same chain"),
            Self::InvalidNonce { expected, got } => {
                write!(f, "invalid nonce: expected {expected}, got {got}")
            }
            Self::Expired => write!(f, "message expired"),
            Self::Replay => write!(f, "message already processed"),
            Self::InvalidProof => write!(f, "Merkle proof verification failed"),
            Self::ProofMismatch => write!(f, "proof message hash does not match"),
            Self::BatchNotFound => write!(f, "batch not found for checkpoint"),
            Self::MessageNotFound => write!(f, "message not found in batch"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_msg(sender: u8, nonce: u64, epoch: Epoch) -> BridgeMessage {
        BridgeMessage {
            source: ChainId::Prova,
            destination: ChainId::Filecoin,
            sender: Address::test(sender),
            nonce,
            created_at: epoch,
            payload: MessagePayload::TokenTransfer {
                recipient: Address::test(sender + 100),
                amount: 1000,
            },
        }
    }

    fn make_inbound_msg(sender: u8, nonce: u64, epoch: Epoch) -> BridgeMessage {
        BridgeMessage {
            source: ChainId::Filecoin,
            destination: ChainId::Prova,
            sender: Address::test(sender),
            nonce,
            created_at: epoch,
            payload: MessagePayload::StakeDeposit {
                staker: Address::test(sender),
                amount: 5000,
            },
        }
    }

    #[test]
    fn test_message_hash_deterministic() {
        let msg = make_msg(1, 0, 100);
        assert_eq!(msg.hash(), msg.hash());
        assert_ne!(msg.hash(), [0u8; 32]);
    }

    #[test]
    fn test_message_hash_varies_by_nonce() {
        let m1 = make_msg(1, 0, 100);
        let m2 = make_msg(1, 1, 100);
        assert_ne!(m1.hash(), m2.hash());
    }

    #[test]
    fn test_message_expiry() {
        let msg = make_msg(1, 0, 100);
        assert!(!msg.is_expired(100));
        assert!(!msg.is_expired(100 + MAX_MESSAGE_AGE));
        assert!(msg.is_expired(100 + MAX_MESSAGE_AGE + 1));
    }

    #[test]
    fn test_outbox_queue_and_nonce() {
        let mut outbox = Outbox::new();
        let m0 = make_msg(1, 0, 10);
        let m1 = make_msg(1, 1, 11);
        outbox.queue(m0).unwrap();
        outbox.queue(m1).unwrap();
        assert_eq!(outbox.pending_count(), 2);
    }

    #[test]
    fn test_outbox_rejects_wrong_nonce() {
        let mut outbox = Outbox::new();
        let msg = make_msg(1, 5, 10); // nonce 5 but expected 0
        let err = outbox.queue(msg).unwrap_err();
        assert_eq!(
            err,
            BridgeError::InvalidNonce {
                expected: 0,
                got: 5
            }
        );
    }

    #[test]
    fn test_outbox_rejects_same_chain() {
        let mut outbox = Outbox::new();
        let msg = BridgeMessage {
            source: ChainId::Prova,
            destination: ChainId::Prova,
            sender: Address::test(1),
            nonce: 0,
            created_at: 10,
            payload: MessagePayload::RawData(vec![1, 2, 3]),
        };
        assert_eq!(outbox.queue(msg).unwrap_err(), BridgeError::SameChain);
    }

    #[test]
    fn test_seal_batch_and_prove() {
        let mut outbox = Outbox::new();
        for i in 0..4u64 {
            outbox.queue(make_msg(1, i, 10 + i)).unwrap();
        }
        let root = outbox.seal_batch(0);
        assert_ne!(root, [0u8; 32]);
        assert_eq!(outbox.pending_count(), 0);
        assert_eq!(outbox.batch_count(), 1);

        // Prove each message
        for i in 0..4 {
            let proof = outbox.prove(0, i).unwrap();
            assert!(proof.verify(root));
        }
    }

    #[test]
    fn test_seal_empty_batch() {
        let mut outbox = Outbox::new();
        let root = outbox.seal_batch(0);
        assert_eq!(root, [0u8; 32]);
    }

    #[test]
    fn test_proof_fails_wrong_root() {
        let mut outbox = Outbox::new();
        outbox.queue(make_msg(1, 0, 10)).unwrap();
        let root = outbox.seal_batch(0);
        let proof = outbox.prove(0, 0).unwrap();
        assert!(proof.verify(root));
        assert!(!proof.verify([0xffu8; 32])); // wrong root
    }

    #[test]
    fn test_inbox_receive_and_replay_protection() {
        let mut outbox = Outbox::new();
        let msg = make_inbound_msg(1, 0, 100);
        // Build a mini outbox just for proof generation
        let outbound_copy = BridgeMessage {
            source: ChainId::Filecoin,
            destination: ChainId::Prova,
            sender: msg.sender,
            nonce: msg.nonce,
            created_at: msg.created_at,
            payload: msg.payload.clone(),
        };
        // Manually build proof
        let leaves = vec![outbound_copy.hash()];
        let root = compute_merkle_root(&leaves);
        let proof = build_merkle_proof(&leaves, 0);

        let mut inbox = Inbox::new();
        inbox.receive(&msg, &proof, root, 200).unwrap();
        assert_eq!(inbox.processed_count(), 1);

        // Replay should fail
        let err = inbox.receive(&msg, &proof, root, 201).unwrap_err();
        assert_eq!(err, BridgeError::Replay);
    }

    #[test]
    fn test_inbox_rejects_expired() {
        let msg = make_inbound_msg(1, 0, 100);
        let leaves = vec![msg.hash()];
        let root = compute_merkle_root(&leaves);
        let proof = build_merkle_proof(&leaves, 0);

        let mut inbox = Inbox::new();
        let err = inbox
            .receive(&msg, &proof, root, 100 + MAX_MESSAGE_AGE + 1)
            .unwrap_err();
        assert_eq!(err, BridgeError::Expired);
    }

    #[test]
    fn test_inbox_rejects_bad_proof() {
        let msg = make_inbound_msg(1, 0, 100);
        let proof = StateProof {
            message_hash: msg.hash(),
            siblings: vec![[0xaau8; 32]],
            index: 0,
            total_leaves: 2,
        };
        let mut inbox = Inbox::new();
        let err = inbox.receive(&msg, &proof, [0xbbu8; 32], 200).unwrap_err();
        assert_eq!(err, BridgeError::InvalidProof);
    }

    #[test]
    fn test_inbox_rejects_wrong_nonce() {
        let msg = make_inbound_msg(1, 1, 100); // nonce 1 but expected 0
        let leaves = vec![msg.hash()];
        let root = compute_merkle_root(&leaves);
        let proof = build_merkle_proof(&leaves, 0);

        let mut inbox = Inbox::new();
        let err = inbox.receive(&msg, &proof, root, 200).unwrap_err();
        assert_eq!(
            err,
            BridgeError::InvalidNonce {
                expected: 0,
                got: 1
            }
        );
    }

    #[test]
    fn test_merkle_root_single_leaf() {
        let leaf = [42u8; 32];
        assert_eq!(compute_merkle_root(&[leaf]), leaf);
    }

    #[test]
    fn test_merkle_root_empty() {
        assert_eq!(compute_merkle_root(&[]), [0u8; 32]);
    }

    #[test]
    fn test_all_payload_types_hash_differently() {
        let base = BridgeMessage {
            source: ChainId::Prova,
            destination: ChainId::Filecoin,
            sender: Address::test(1),
            nonce: 0,
            created_at: 100,
            payload: MessagePayload::TokenTransfer {
                recipient: Address::test(2),
                amount: 100,
            },
        };
        let payloads = vec![
            MessagePayload::TokenTransfer { recipient: Address::test(2), amount: 100 },
            MessagePayload::CheckpointAttestation { sequence: 1, state_root: [1u8; 32] },
            MessagePayload::DisputeResult { commit_id: 1, slashed: Address::test(3), amount: 50 },
            MessagePayload::StakeDeposit { staker: Address::test(4), amount: 200 },
            MessagePayload::GovernanceAction { proposal_id: 1, action_hash: [2u8; 32] },
            MessagePayload::RawData(vec![1, 2, 3]),
        ];
        let hashes: Vec<Hash> = payloads
            .into_iter()
            .map(|p| {
                let mut m = base.clone();
                m.payload = p;
                m.hash()
            })
            .collect();
        // All should be unique
        for i in 0..hashes.len() {
            for j in (i + 1)..hashes.len() {
                assert_ne!(hashes[i], hashes[j], "payloads {i} and {j} collided");
            }
        }
    }

    #[test]
    fn test_prove_nonexistent_batch() {
        let outbox = Outbox::new();
        assert_eq!(outbox.prove(99, 0).unwrap_err(), BridgeError::BatchNotFound);
    }

    #[test]
    fn test_prove_out_of_bounds() {
        let mut outbox = Outbox::new();
        outbox.queue(make_msg(1, 0, 10)).unwrap();
        outbox.seal_batch(0);
        assert_eq!(
            outbox.prove(0, 5).unwrap_err(),
            BridgeError::MessageNotFound
        );
    }

    #[test]
    fn test_multi_sender_nonces() {
        let mut outbox = Outbox::new();
        // Two senders, independent nonces
        outbox.queue(make_msg(1, 0, 10)).unwrap();
        outbox.queue(make_msg(2, 0, 10)).unwrap();
        outbox.queue(make_msg(1, 1, 11)).unwrap();
        outbox.queue(make_msg(2, 1, 11)).unwrap();
        assert_eq!(outbox.pending_count(), 4);
    }

    #[test]
    fn test_e2e_outbox_to_inbox() {
        // Full flow: queue → seal → prove → receive
        let mut outbox = Outbox::new();
        let msgs: Vec<BridgeMessage> = (0..3)
            .map(|i| make_inbound_msg(1, i, 100 + i))
            .collect();
        // Use outbox as the source chain's outbox
        let mut src_outbox = Outbox::new();
        for m in &msgs {
            // Re-create as outbound from Filecoin perspective
            src_outbox
                .queue(BridgeMessage {
                    source: ChainId::Filecoin,
                    destination: ChainId::Prova,
                    sender: m.sender,
                    nonce: m.nonce,
                    created_at: m.created_at,
                    payload: m.payload.clone(),
                })
                .unwrap();
        }
        let root = src_outbox.seal_batch(0);

        let mut inbox = Inbox::new();
        for (i, m) in msgs.iter().enumerate() {
            let proof = src_outbox.prove(0, i).unwrap();
            inbox.receive(m, &proof, root, 200).unwrap();
        }
        assert_eq!(inbox.processed_count(), 3);
    }
}
