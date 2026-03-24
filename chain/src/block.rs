//! Block Production + Consensus — weighted round-robin block production with stake-based finality.
//!
//! Prova uses a deterministic weighted round-robin for block production:
//! - Each epoch (30s), exactly one block producer is selected
//! - Selection weight = storage_power + compute_power
//! - Producer is chosen via `hash(epoch || stake_snapshot_hash) mod total_weight`
//! - Finality requires 2/3 stake-weighted signatures
//!
//! Block structure:
//! - Header: parent hash, state root, epoch, producer, signature
//! - Body: ordered list of transactions

use crate::types::*;
use sha2::{Digest, Sha256};
use std::collections::HashMap;

/// Transaction types that can appear in a block.
#[derive(Debug, Clone)]
pub enum Transaction {
    /// Provider publishes an inference commitment.
    InferenceCommit {
        provider: Address,
        model_id: ModelId,
        arch_group: ArchGroup,
        input_hash: Hash,
        activation_root: Hash,
        leaf_count: u32,
    },
    /// Someone challenges an inference commit.
    Challenge {
        challenger: Address,
        commit_id: CommitId,
    },
    /// Bisection game move — provider or challenger reveals a layer hash.
    BisectionMove {
        dispute_id: u64,
        mover: Address,
        layer: u32,
        hash: Hash,
    },
    /// PDP proof submission.
    PdpProof {
        provider: Address,
        proof_set_id: u64,
        challenged_roots: Vec<u32>,
    },
    /// Payment channel operation.
    PaymentOp(PaymentOp),
    /// Stake operation.
    StakeOp(StakeOp),
    /// Model registration.
    RegisterModel {
        owner: Address,
        model_hash: Hash,
        name: String,
        layer_count: u32,
        arch_group: ArchGroup,
    },
}

/// Payment channel operations.
#[derive(Debug, Clone)]
pub enum PaymentOp {
    Open {
        payer: Address,
        payee: Address,
        deposit: StakeAmount,
        rate_per_epoch: StakeAmount,
    },
    Pay {
        channel_id: u64,
    },
    Close {
        channel_id: u64,
    },
}

/// Stake operations.
#[derive(Debug, Clone)]
pub enum StakeOp {
    Deposit {
        provider: Address,
        amount: StakeAmount,
    },
    Withdraw {
        provider: Address,
        amount: StakeAmount,
    },
    Lock {
        provider: Address,
        amount: StakeAmount,
        reason: String,
    },
}

/// Block header — contains all metadata needed for validation.
#[derive(Debug, Clone)]
pub struct BlockHeader {
    /// Hash of the parent block header.
    pub parent_hash: Hash,
    /// Merkle root of the post-execution state.
    pub state_root: Hash,
    /// Block epoch (height).
    pub epoch: Epoch,
    /// Address of the block producer.
    pub producer: Address,
    /// SHA-256 of the ordered transaction list.
    pub tx_root: Hash,
    /// Number of transactions in the body.
    pub tx_count: u32,
    /// Timestamp (Unix seconds) — informational, not consensus-critical.
    pub timestamp: u64,
}

impl BlockHeader {
    /// Compute the block hash (SHA-256 of the serialized header).
    pub fn hash(&self) -> Hash {
        let mut hasher = Sha256::new();
        hasher.update(self.parent_hash);
        hasher.update(self.state_root);
        hasher.update(self.epoch.to_le_bytes());
        hasher.update(self.producer.0);
        hasher.update(self.tx_root);
        hasher.update(self.tx_count.to_le_bytes());
        hasher.update(self.timestamp.to_le_bytes());
        hasher.finalize().into()
    }
}

/// A complete block.
#[derive(Debug, Clone)]
pub struct Block {
    pub header: BlockHeader,
    pub transactions: Vec<Transaction>,
}

impl Block {
    /// Compute the transaction root (SHA-256 of sequential tx hashes).
    pub fn compute_tx_root(transactions: &[Transaction]) -> Hash {
        let mut hasher = Sha256::new();
        for (i, tx) in transactions.iter().enumerate() {
            hasher.update((i as u64).to_le_bytes());
            hasher.update(tx.tag_bytes());
        }
        if transactions.is_empty() {
            // Empty block has a zero tx root
            return [0u8; 32];
        }
        hasher.finalize().into()
    }

    /// Validate internal consistency (tx_root matches, tx_count matches).
    pub fn validate_internal(&self) -> Result<(), BlockError> {
        if self.header.tx_count as usize != self.transactions.len() {
            return Err(BlockError::TxCountMismatch {
                header: self.header.tx_count,
                actual: self.transactions.len() as u32,
            });
        }
        let expected_root = Self::compute_tx_root(&self.transactions);
        if self.header.tx_root != expected_root {
            return Err(BlockError::TxRootMismatch);
        }
        Ok(())
    }
}

impl Transaction {
    /// Tag bytes for hashing — unique per variant to prevent collisions.
    fn tag_bytes(&self) -> Vec<u8> {
        let mut out = Vec::new();
        match self {
            Transaction::InferenceCommit {
                provider,
                model_id,
                input_hash,
                activation_root,
                leaf_count,
                ..
            } => {
                out.push(0x01);
                out.extend_from_slice(&provider.0);
                out.extend_from_slice(&model_id.0);
                out.extend_from_slice(input_hash);
                out.extend_from_slice(activation_root);
                out.extend_from_slice(&leaf_count.to_le_bytes());
            }
            Transaction::Challenge {
                challenger,
                commit_id,
            } => {
                out.push(0x02);
                out.extend_from_slice(&challenger.0);
                out.extend_from_slice(&commit_id.0.to_le_bytes());
            }
            Transaction::BisectionMove {
                dispute_id,
                mover,
                layer,
                hash,
            } => {
                out.push(0x03);
                out.extend_from_slice(&dispute_id.to_le_bytes());
                out.extend_from_slice(&mover.0);
                out.extend_from_slice(&layer.to_le_bytes());
                out.extend_from_slice(hash);
            }
            Transaction::PdpProof {
                provider,
                proof_set_id,
                challenged_roots,
            } => {
                out.push(0x04);
                out.extend_from_slice(&provider.0);
                out.extend_from_slice(&proof_set_id.to_le_bytes());
                for r in challenged_roots {
                    out.extend_from_slice(&r.to_le_bytes());
                }
            }
            Transaction::PaymentOp(op) => {
                out.push(0x05);
                match op {
                    PaymentOp::Open {
                        payer,
                        payee,
                        deposit,
                        rate_per_epoch,
                    } => {
                        out.push(0x01);
                        out.extend_from_slice(&payer.0);
                        out.extend_from_slice(&payee.0);
                        out.extend_from_slice(&deposit.to_le_bytes());
                        out.extend_from_slice(&rate_per_epoch.to_le_bytes());
                    }
                    PaymentOp::Pay { channel_id } => {
                        out.push(0x02);
                        out.extend_from_slice(&channel_id.to_le_bytes());
                    }
                    PaymentOp::Close { channel_id } => {
                        out.push(0x03);
                        out.extend_from_slice(&channel_id.to_le_bytes());
                    }
                }
            }
            Transaction::StakeOp(op) => {
                out.push(0x06);
                match op {
                    StakeOp::Deposit { provider, amount } => {
                        out.push(0x01);
                        out.extend_from_slice(&provider.0);
                        out.extend_from_slice(&amount.to_le_bytes());
                    }
                    StakeOp::Withdraw { provider, amount } => {
                        out.push(0x02);
                        out.extend_from_slice(&provider.0);
                        out.extend_from_slice(&amount.to_le_bytes());
                    }
                    StakeOp::Lock {
                        provider, amount, ..
                    } => {
                        out.push(0x03);
                        out.extend_from_slice(&provider.0);
                        out.extend_from_slice(&amount.to_le_bytes());
                    }
                }
            }
            Transaction::RegisterModel {
                owner,
                model_hash,
                name,
                layer_count,
                ..
            } => {
                out.push(0x07);
                out.extend_from_slice(&owner.0);
                out.extend_from_slice(model_hash);
                out.extend_from_slice(name.as_bytes());
                out.extend_from_slice(&layer_count.to_le_bytes());
            }
        }
        out
    }
}

/// Block production errors.
#[derive(Debug, PartialEq, Eq)]
pub enum BlockError {
    TxCountMismatch { header: u32, actual: u32 },
    TxRootMismatch,
    WrongProducer { expected: Address, got: Address },
    EpochMismatch { expected: Epoch, got: Epoch },
    ParentHashMismatch,
    InsufficientStake,
    NoEligibleProducers,
    DoubleSign { epoch: Epoch, producer: Address },
    NotFinalized,
}

/// Producer selection — weighted round-robin based on stake power.
#[derive(Debug)]
pub struct ProducerSchedule {
    /// Ordered list of eligible producers and their weights.
    entries: Vec<(Address, u64)>,
    /// Total weight across all producers.
    total_weight: u64,
    /// Snapshot hash — for deterministic selection.
    snapshot_hash: Hash,
}

impl ProducerSchedule {
    /// Build a schedule from a snapshot of provider powers.
    ///
    /// `providers` must be sorted by address for deterministic ordering.
    pub fn new(mut providers: Vec<(Address, u64)>, snapshot_hash: Hash) -> Option<Self> {
        // Sort by address bytes for deterministic ordering
        providers.sort_by_key(|(addr, _)| addr.0);

        // Filter out zero-weight providers
        let entries: Vec<(Address, u64)> = providers.into_iter().filter(|(_, w)| *w > 0).collect();

        if entries.is_empty() {
            return None;
        }

        let total_weight = entries.iter().map(|(_, w)| *w).sum();

        Some(Self {
            entries,
            total_weight,
            snapshot_hash,
        })
    }

    /// Select the block producer for a given epoch.
    ///
    /// Uses `SHA-256(epoch || snapshot_hash) mod total_weight` to pick
    /// a random-looking but deterministic selection point, then walks
    /// the cumulative weight distribution.
    pub fn producer_for_epoch(&self, epoch: Epoch) -> Address {
        let mut hasher = Sha256::new();
        hasher.update(epoch.to_le_bytes());
        hasher.update(self.snapshot_hash);
        let selection_hash: [u8; 32] = hasher.finalize().into();

        // Interpret first 8 bytes as u64 for the selection point
        let selection_bytes: [u8; 8] = selection_hash[..8].try_into().unwrap();
        let selection_point = u64::from_le_bytes(selection_bytes) % self.total_weight;

        let mut cumulative = 0u64;
        for (addr, weight) in &self.entries {
            cumulative += weight;
            if selection_point < cumulative {
                return *addr;
            }
        }

        // Should never reach here, but return last producer as fallback
        self.entries.last().unwrap().0
    }

    /// Get all entries in the schedule.
    pub fn entries(&self) -> &[(Address, u64)] {
        &self.entries
    }

    /// Total weight in the schedule.
    pub fn total_weight(&self) -> u64 {
        self.total_weight
    }
}

/// Finality tracker — collects 2/3 stake-weighted votes for blocks.
#[derive(Debug)]
pub struct FinalityTracker {
    /// Votes per block hash: address → weight that voted.
    votes: HashMap<Hash, Vec<(Address, u64)>>,
    /// Total votable weight (all staked providers).
    #[allow(dead_code)]
    total_weight: u64,
    /// Threshold for finality (2/3 of total weight).
    threshold: u64,
    /// Set of finalized block hashes.
    finalized: Vec<Hash>,
}

impl FinalityTracker {
    pub fn new(total_weight: u64) -> Self {
        // 2/3 threshold (ceiling)
        let threshold = (total_weight * 2).div_ceil(3);
        Self {
            votes: HashMap::new(),
            total_weight,
            threshold,
            finalized: Vec::new(),
        }
    }

    /// Cast a vote for a block. Returns true if the block just became finalized.
    pub fn vote(&mut self, block_hash: Hash, voter: Address, voter_weight: u64) -> bool {
        let entry = self.votes.entry(block_hash).or_default();

        // Prevent double-voting
        if entry.iter().any(|(a, _)| *a == voter) {
            return false;
        }

        entry.push((voter, voter_weight));

        let voted_weight: u64 = entry.iter().map(|(_, w)| *w).sum();

        if voted_weight >= self.threshold && !self.finalized.contains(&block_hash) {
            self.finalized.push(block_hash);
            return true;
        }

        false
    }

    /// Check if a block is finalized.
    pub fn is_finalized(&self, block_hash: &Hash) -> bool {
        self.finalized.contains(block_hash)
    }

    /// Total voted weight for a block.
    pub fn voted_weight(&self, block_hash: &Hash) -> u64 {
        self.votes
            .get(block_hash)
            .map(|v| v.iter().map(|(_, w)| *w).sum())
            .unwrap_or(0)
    }

    /// Threshold weight for finality.
    pub fn threshold(&self) -> u64 {
        self.threshold
    }

    /// All finalized block hashes.
    pub fn finalized_blocks(&self) -> &[Hash] {
        &self.finalized
    }
}

/// Block builder — constructs valid blocks for a given epoch.
pub struct BlockBuilder {
    epoch: Epoch,
    parent_hash: Hash,
    producer: Address,
    transactions: Vec<Transaction>,
    timestamp: u64,
}

impl BlockBuilder {
    pub fn new(epoch: Epoch, parent_hash: Hash, producer: Address, timestamp: u64) -> Self {
        Self {
            epoch,
            parent_hash,
            producer,
            transactions: Vec::new(),
            timestamp,
        }
    }

    /// Add a transaction to the block.
    pub fn push_tx(&mut self, tx: Transaction) {
        self.transactions.push(tx);
    }

    /// Finalize and build the block.
    pub fn build(self, state_root: Hash) -> Block {
        let tx_root = Block::compute_tx_root(&self.transactions);
        let header = BlockHeader {
            parent_hash: self.parent_hash,
            state_root,
            epoch: self.epoch,
            producer: self.producer,
            tx_root,
            tx_count: self.transactions.len() as u32,
            timestamp: self.timestamp,
        };
        Block {
            header,
            transactions: self.transactions,
        }
    }

    /// Number of transactions currently in the builder.
    pub fn tx_count(&self) -> usize {
        self.transactions.len()
    }
}

/// Chain — the linear sequence of blocks.
#[derive(Debug)]
pub struct BlockChain {
    /// All blocks by hash.
    blocks: HashMap<Hash, Block>,
    /// Block hash at each epoch.
    by_epoch: HashMap<Epoch, Hash>,
    /// Current tip (latest block hash).
    tip: Hash,
    /// Current height.
    height: Epoch,
}

impl BlockChain {
    /// Create a new chain starting from a genesis block.
    pub fn new(genesis: Block) -> Self {
        let hash = genesis.header.hash();
        let epoch = genesis.header.epoch;
        let mut blocks = HashMap::new();
        let mut by_epoch = HashMap::new();
        blocks.insert(hash, genesis);
        by_epoch.insert(epoch, hash);

        Self {
            blocks,
            by_epoch,
            tip: hash,
            height: epoch,
        }
    }

    /// Append a block to the chain.
    pub fn append(&mut self, block: Block) -> Result<Hash, BlockError> {
        let expected_epoch = self.height + 1;
        if block.header.epoch != expected_epoch {
            return Err(BlockError::EpochMismatch {
                expected: expected_epoch,
                got: block.header.epoch,
            });
        }

        if block.header.parent_hash != self.tip {
            return Err(BlockError::ParentHashMismatch);
        }

        block.validate_internal()?;

        let hash = block.header.hash();
        self.by_epoch.insert(block.header.epoch, hash);
        self.blocks.insert(hash, block);
        self.tip = hash;
        self.height = expected_epoch;

        Ok(hash)
    }

    /// Get a block by hash.
    pub fn get(&self, hash: &Hash) -> Option<&Block> {
        self.blocks.get(hash)
    }

    /// Get a block by epoch.
    pub fn get_at_epoch(&self, epoch: Epoch) -> Option<&Block> {
        self.by_epoch.get(&epoch).and_then(|h| self.blocks.get(h))
    }

    /// Current tip hash.
    pub fn tip(&self) -> Hash {
        self.tip
    }

    /// Current height.
    pub fn height(&self) -> Epoch {
        self.height
    }

    /// Number of blocks in the chain.
    pub fn len(&self) -> usize {
        self.blocks.len()
    }

    /// Whether the chain is empty (should never be — always has genesis).
    pub fn is_empty(&self) -> bool {
        self.blocks.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn genesis_block() -> Block {
        let header = BlockHeader {
            parent_hash: [0u8; 32],
            state_root: [0xAA; 32],
            epoch: 0,
            producer: Address::test(0),
            tx_root: [0u8; 32],
            tx_count: 0,
            timestamp: 1_700_000_000,
        };
        Block {
            header,
            transactions: vec![],
        }
    }

    #[test]
    fn test_block_hash_deterministic() {
        let b = genesis_block();
        assert_eq!(b.header.hash(), b.header.hash());
    }

    #[test]
    fn test_block_hash_changes_with_epoch() {
        let mut b1 = genesis_block();
        let mut b2 = genesis_block();
        b2.header.epoch = 1;
        assert_ne!(b1.header.hash(), b2.header.hash());
    }

    #[test]
    fn test_empty_block_validates() {
        let b = genesis_block();
        assert!(b.validate_internal().is_ok());
    }

    #[test]
    fn test_block_with_transactions() {
        let txs = vec![
            Transaction::StakeOp(StakeOp::Deposit {
                provider: Address::test(1),
                amount: 1_000_000,
            }),
            Transaction::RegisterModel {
                owner: Address::test(1),
                model_hash: [0xBB; 32],
                name: "test-model".into(),
                layer_count: 32,
                arch_group: ArchGroup::new("nvidia-sm89-int8"),
            },
        ];
        let tx_root = Block::compute_tx_root(&txs);
        let header = BlockHeader {
            parent_hash: [0; 32],
            state_root: [0xCC; 32],
            epoch: 1,
            producer: Address::test(1),
            tx_root,
            tx_count: 2,
            timestamp: 1_700_000_030,
        };
        let block = Block {
            header,
            transactions: txs,
        };
        assert!(block.validate_internal().is_ok());
    }

    #[test]
    fn test_tx_count_mismatch_fails() {
        let header = BlockHeader {
            parent_hash: [0; 32],
            state_root: [0; 32],
            epoch: 0,
            producer: Address::test(0),
            tx_root: [0; 32],
            tx_count: 5, // wrong
            timestamp: 0,
        };
        let block = Block {
            header,
            transactions: vec![],
        };
        assert_eq!(
            block.validate_internal(),
            Err(BlockError::TxCountMismatch {
                header: 5,
                actual: 0
            })
        );
    }

    #[test]
    fn test_tx_root_mismatch_fails() {
        let txs = vec![Transaction::StakeOp(StakeOp::Deposit {
            provider: Address::test(1),
            amount: 100,
        })];
        let header = BlockHeader {
            parent_hash: [0; 32],
            state_root: [0; 32],
            epoch: 0,
            producer: Address::test(0),
            tx_root: [0xFF; 32], // wrong root
            tx_count: 1,
            timestamp: 0,
        };
        let block = Block {
            header,
            transactions: txs,
        };
        assert_eq!(block.validate_internal(), Err(BlockError::TxRootMismatch));
    }

    // --- ProducerSchedule tests ---

    #[test]
    fn test_producer_schedule_single() {
        let providers = vec![(Address::test(1), 100)];
        let schedule = ProducerSchedule::new(providers, [0; 32]).unwrap();

        // With only one provider, every epoch selects them
        for epoch in 0..10 {
            assert_eq!(schedule.producer_for_epoch(epoch), Address::test(1));
        }
    }

    #[test]
    fn test_producer_schedule_empty_returns_none() {
        let schedule = ProducerSchedule::new(vec![], [0; 32]);
        assert!(schedule.is_none());
    }

    #[test]
    fn test_producer_schedule_zero_weight_filtered() {
        let providers = vec![(Address::test(1), 0), (Address::test(2), 100)];
        let schedule = ProducerSchedule::new(providers, [0; 32]).unwrap();
        assert_eq!(schedule.entries().len(), 1);
        assert_eq!(schedule.total_weight(), 100);
    }

    #[test]
    fn test_producer_schedule_weighted_distribution() {
        // Provider A has 90% weight, Provider B has 10%
        let providers = vec![(Address::test(1), 900), (Address::test(2), 100)];
        let schedule = ProducerSchedule::new(providers, [0x42; 32]).unwrap();

        // Run 1000 epochs and check distribution is roughly proportional
        let mut counts = HashMap::new();
        for epoch in 0..1000 {
            let producer = schedule.producer_for_epoch(epoch);
            *counts.entry(producer).or_insert(0u32) += 1;
        }

        let a_count = *counts.get(&Address::test(1)).unwrap_or(&0);
        let b_count = *counts.get(&Address::test(2)).unwrap_or(&0);

        // A should get roughly 900/1000 = 90% of epochs (allow 5% tolerance)
        assert!(a_count > 800, "A got {a_count}/1000, expected ~900");
        assert!(b_count > 50, "B got {b_count}/1000, expected ~100");
        assert_eq!(a_count + b_count, 1000);
    }

    #[test]
    fn test_producer_schedule_deterministic() {
        let providers = vec![
            (Address::test(1), 500),
            (Address::test(2), 300),
            (Address::test(3), 200),
        ];
        let s1 = ProducerSchedule::new(providers.clone(), [0xAB; 32]).unwrap();
        let s2 = ProducerSchedule::new(providers, [0xAB; 32]).unwrap();

        for epoch in 0..100 {
            assert_eq!(s1.producer_for_epoch(epoch), s2.producer_for_epoch(epoch));
        }
    }

    // --- FinalityTracker tests ---

    #[test]
    fn test_finality_threshold() {
        let tracker = FinalityTracker::new(300);
        // 2/3 of 300 = 200, ceiling = 201
        assert!(tracker.threshold() >= 200);
    }

    #[test]
    fn test_finality_single_large_voter() {
        let mut tracker = FinalityTracker::new(300);
        let block_hash = [0xBB; 32];

        // One voter with 250 weight (> 2/3 of 300 = 200)
        let finalized = tracker.vote(block_hash, Address::test(1), 250);
        assert!(finalized);
        assert!(tracker.is_finalized(&block_hash));
    }

    #[test]
    fn test_finality_accumulating_votes() {
        let mut tracker = FinalityTracker::new(300);
        let block_hash = [0xCC; 32];

        // Three voters: 80 + 80 + 80 = 240 > 200
        assert!(!tracker.vote(block_hash, Address::test(1), 80));
        assert!(!tracker.vote(block_hash, Address::test(2), 80));
        assert!(tracker.vote(block_hash, Address::test(3), 80));
        assert!(tracker.is_finalized(&block_hash));
    }

    #[test]
    fn test_finality_double_vote_ignored() {
        let mut tracker = FinalityTracker::new(300);
        let block_hash = [0xDD; 32];

        tracker.vote(block_hash, Address::test(1), 150);
        // Same voter again — should be ignored
        let finalized = tracker.vote(block_hash, Address::test(1), 150);
        assert!(!finalized);
        assert_eq!(tracker.voted_weight(&block_hash), 150);
    }

    #[test]
    fn test_finality_not_reached() {
        let mut tracker = FinalityTracker::new(300);
        let block_hash = [0xEE; 32];

        tracker.vote(block_hash, Address::test(1), 50);
        tracker.vote(block_hash, Address::test(2), 50);
        assert!(!tracker.is_finalized(&block_hash));
        assert_eq!(tracker.voted_weight(&block_hash), 100);
    }

    // --- BlockChain tests ---

    #[test]
    fn test_blockchain_genesis() {
        let genesis = genesis_block();
        let chain = BlockChain::new(genesis);
        assert_eq!(chain.height(), 0);
        assert_eq!(chain.len(), 1);
    }

    #[test]
    fn test_blockchain_append() {
        let genesis = genesis_block();
        let genesis_hash = genesis.header.hash();
        let mut chain = BlockChain::new(genesis);

        let header = BlockHeader {
            parent_hash: genesis_hash,
            state_root: [0xBB; 32],
            epoch: 1,
            producer: Address::test(1),
            tx_root: [0; 32], // empty tx
            tx_count: 0,
            timestamp: 1_700_000_030,
        };
        let block = Block {
            header,
            transactions: vec![],
        };
        let hash = chain.append(block).unwrap();

        assert_eq!(chain.height(), 1);
        assert_eq!(chain.tip(), hash);
        assert_eq!(chain.len(), 2);
    }

    #[test]
    fn test_blockchain_wrong_parent_rejected() {
        let genesis = genesis_block();
        let mut chain = BlockChain::new(genesis);

        let header = BlockHeader {
            parent_hash: [0xFF; 32], // wrong parent
            state_root: [0; 32],
            epoch: 1,
            producer: Address::test(1),
            tx_root: [0; 32],
            tx_count: 0,
            timestamp: 0,
        };
        let block = Block {
            header,
            transactions: vec![],
        };
        assert_eq!(chain.append(block), Err(BlockError::ParentHashMismatch));
    }

    #[test]
    fn test_blockchain_wrong_epoch_rejected() {
        let genesis = genesis_block();
        let genesis_hash = genesis.header.hash();
        let mut chain = BlockChain::new(genesis);

        let header = BlockHeader {
            parent_hash: genesis_hash,
            state_root: [0; 32],
            epoch: 5, // should be 1
            producer: Address::test(1),
            tx_root: [0; 32],
            tx_count: 0,
            timestamp: 0,
        };
        let block = Block {
            header,
            transactions: vec![],
        };
        assert_eq!(
            chain.append(block),
            Err(BlockError::EpochMismatch {
                expected: 1,
                got: 5
            })
        );
    }

    #[test]
    fn test_blockchain_ten_blocks() {
        let genesis = genesis_block();
        let mut chain = BlockChain::new(genesis);

        for i in 1..=10u64 {
            let parent_hash = chain.tip();
            let header = BlockHeader {
                parent_hash,
                state_root: [i as u8; 32],
                epoch: i,
                producer: Address::test((i % 3) as u8),
                tx_root: [0; 32],
                tx_count: 0,
                timestamp: 1_700_000_000 + i * 30,
            };
            chain
                .append(Block {
                    header,
                    transactions: vec![],
                })
                .unwrap();
        }

        assert_eq!(chain.height(), 10);
        assert_eq!(chain.len(), 11);

        // Verify epoch lookup works
        for i in 0..=10u64 {
            assert!(chain.get_at_epoch(i).is_some());
        }
    }

    #[test]
    fn test_block_builder() {
        let builder = BlockBuilder::new(5, [0xAA; 32], Address::test(1), 1_700_000_150);
        let block = builder.build([0xBB; 32]);
        assert_eq!(block.header.epoch, 5);
        assert_eq!(block.header.tx_count, 0);
        assert!(block.validate_internal().is_ok());
    }

    #[test]
    fn test_block_builder_with_txs() {
        let mut builder = BlockBuilder::new(10, [0; 32], Address::test(2), 1_700_000_300);
        builder.push_tx(Transaction::StakeOp(StakeOp::Deposit {
            provider: Address::test(2),
            amount: 5_000_000,
        }));
        builder.push_tx(Transaction::InferenceCommit {
            provider: Address::test(2),
            model_id: ModelId([0x42; 32]),
            arch_group: ArchGroup::new("nvidia-sm89-int8"),
            input_hash: [0xAA; 32],
            activation_root: [0xBB; 32],
            leaf_count: 33,
        });
        assert_eq!(builder.tx_count(), 2);

        let block = builder.build([0xCC; 32]);
        assert_eq!(block.header.tx_count, 2);
        assert!(block.validate_internal().is_ok());
    }
}
