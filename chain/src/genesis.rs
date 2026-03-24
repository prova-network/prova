//! Genesis State — initial chain state and configuration.
//!
//! Genesis defines the initial state of the Prova chain:
//! - Initial token allocations
//! - Pre-registered models
//! - System parameters
//! - The genesis block itself
//!
//! Genesis state is deterministic — any node running with the same
//! GenesisConfig produces the same genesis block and state root.

use crate::block::*;
use crate::commit::CommitConfig;
use crate::dispute::DisputeConfig;
use crate::epoch::ChainState;
use crate::payment::PaymentManager;
use crate::stake::StakeLedger;
use crate::types::*;
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;

/// Genesis configuration — everything needed to initialize the chain.
#[derive(Debug, Clone)]
pub struct GenesisConfig {
    /// Chain identifier (e.g., "prova-mainnet", "prova-devnet").
    pub chain_id: String,
    /// Genesis timestamp (Unix seconds).
    pub genesis_time: u64,
    /// Initial token allocations: address → amount.
    /// Using BTreeMap for deterministic ordering.
    pub allocations: BTreeMap<Address, StakeAmount>,
    /// Total token supply (sum of all allocations must match).
    pub total_supply: StakeAmount,
    /// Epoch duration in seconds (default: 30).
    pub epoch_duration_secs: u64,
    /// Challenge window in epochs.
    pub challenge_window: EpochDuration,
    /// Minimum provider stake.
    pub min_provider_stake: StakeAmount,
    /// Minimum challenger stake.
    pub min_challenger_stake: StakeAmount,
    /// Network fee rate in basis points (default: 50 = 0.5%).
    pub network_fee_bps: u32,
    /// Storage/compute reward split (α): 0..10000 basis points for storage share.
    pub storage_reward_bps: u32,
    /// Block reward per epoch (initial emission rate).
    pub block_reward: StakeAmount,
}

impl GenesisConfig {
    /// Create a devnet genesis config for testing.
    pub fn devnet() -> Self {
        let mut allocations = BTreeMap::new();
        // Devnet faucet
        allocations.insert(Address::test(0), 100_000_000_000);
        // 10 test validators
        for i in 1..=10u8 {
            allocations.insert(Address::test(i), 10_000_000_000);
        }

        Self {
            chain_id: "prova-devnet".to_string(),
            genesis_time: 1_700_000_000,
            allocations,
            total_supply: 200_000_000_000,
            epoch_duration_secs: 30,
            challenge_window: 240,
            min_provider_stake: 1_000_000,
            min_challenger_stake: 500_000,
            network_fee_bps: 50,
            storage_reward_bps: 5000, // 50% storage, 50% compute
            block_reward: 38,         // ~38 PROVA per epoch in devnet
        }
    }

    /// Create a minimal testnet config.
    pub fn testnet() -> Self {
        let mut allocations = BTreeMap::new();
        allocations.insert(Address::test(0), 500_000_000_000);
        for i in 1..=5u8 {
            allocations.insert(Address::test(i), 100_000_000_000);
        }

        Self {
            chain_id: "prova-testnet".to_string(),
            genesis_time: 1_700_000_000,
            allocations,
            total_supply: 1_000_000_000_000,
            epoch_duration_secs: 30,
            challenge_window: 240,
            min_provider_stake: 10_000_000,
            min_challenger_stake: 5_000_000,
            network_fee_bps: 50,
            storage_reward_bps: 5000,
            block_reward: 380,
        }
    }

    /// Validate the genesis config.
    pub fn validate(&self) -> Result<(), GenesisError> {
        // Check total supply matches allocations
        let allocated: StakeAmount = self.allocations.values().sum();
        if allocated > self.total_supply {
            return Err(GenesisError::AllocationExceedsSupply {
                allocated,
                supply: self.total_supply,
            });
        }

        if self.allocations.is_empty() {
            return Err(GenesisError::NoAllocations);
        }

        if self.epoch_duration_secs == 0 {
            return Err(GenesisError::InvalidParam(
                "epoch_duration_secs cannot be 0".into(),
            ));
        }

        if self.network_fee_bps > 10000 {
            return Err(GenesisError::InvalidParam("network_fee_bps > 10000".into()));
        }

        if self.storage_reward_bps > 10000 {
            return Err(GenesisError::InvalidParam(
                "storage_reward_bps > 10000".into(),
            ));
        }

        Ok(())
    }

    /// Compute the genesis state root — deterministic hash of the initial state.
    pub fn state_root(&self) -> Hash {
        let mut hasher = Sha256::new();
        hasher.update(self.chain_id.as_bytes());
        hasher.update(self.genesis_time.to_le_bytes());
        hasher.update(self.total_supply.to_le_bytes());
        hasher.update(self.epoch_duration_secs.to_le_bytes());
        hasher.update(self.challenge_window.to_le_bytes());
        hasher.update(self.network_fee_bps.to_le_bytes());
        hasher.update(self.storage_reward_bps.to_le_bytes());
        hasher.update(self.block_reward.to_le_bytes());

        // Allocations are from BTreeMap so ordering is deterministic
        for (addr, amount) in &self.allocations {
            hasher.update(addr.0);
            hasher.update(amount.to_le_bytes());
        }

        hasher.finalize().into()
    }

    /// Build the genesis block.
    pub fn build_genesis_block(&self) -> Block {
        let state_root = self.state_root();
        let header = BlockHeader {
            parent_hash: [0u8; 32],
            state_root,
            epoch: 0,
            producer: Address::test(0), // system address
            tx_root: [0u8; 32],         // no transactions in genesis
            tx_count: 0,
            timestamp: self.genesis_time,
        };
        Block {
            header,
            transactions: vec![],
        }
    }

    /// Initialize full chain state from genesis.
    pub fn build_chain_state(&self) -> Result<GenesisState, GenesisError> {
        self.validate()?;

        let genesis_block = self.build_genesis_block();
        let genesis_hash = genesis_block.header.hash();

        let chain = BlockChain::new(genesis_block);

        let commit_config = CommitConfig {
            challenge_window: self.challenge_window,
            min_provider_stake: self.min_provider_stake,
            min_challenger_stake: self.min_challenger_stake,
        };

        let chain_state = ChainState {
            ticker: crate::epoch::EpochTicker::new(0),
            commits: crate::commit::CommitStore::new(commit_config),
            disputes: crate::dispute::DisputeArena::new(DisputeConfig::default()),
            stakes: StakeLedger::new(self.min_provider_stake, self.min_challenger_stake),
            payments: PaymentManager::new(),
        };

        // Compute initial producer schedule from allocations
        // (everyone with tokens can potentially produce blocks after staking)
        let providers: Vec<(Address, u64)> = self
            .allocations
            .iter()
            .map(|(addr, amount)| {
                // Initial power is proportional to allocation (simplified for genesis)
                (*addr, (*amount / 1_000_000) as u64)
            })
            .collect();

        let schedule = ProducerSchedule::new(providers, self.state_root());

        Ok(GenesisState {
            config: self.clone(),
            chain,
            state: chain_state,
            genesis_hash,
            schedule,
        })
    }
}

/// The fully initialized genesis state.
#[derive(Debug)]
pub struct GenesisState {
    /// The genesis configuration.
    pub config: GenesisConfig,
    /// The block chain (starting with genesis block).
    pub chain: BlockChain,
    /// The chain state (commit store, disputes, stakes, payments).
    pub state: ChainState,
    /// Hash of the genesis block.
    pub genesis_hash: Hash,
    /// Initial producer schedule (may be None if no valid providers).
    pub schedule: Option<ProducerSchedule>,
}

impl GenesisState {
    /// Convenience: produce the next block with given transactions.
    pub fn produce_block(&mut self, transactions: Vec<Transaction>) -> Result<Hash, BlockError> {
        let next_epoch = self.chain.height() + 1;
        let parent_hash = self.chain.tip();
        let timestamp = self.config.genesis_time + next_epoch * self.config.epoch_duration_secs;

        // Select producer
        let producer = match &self.schedule {
            Some(s) => s.producer_for_epoch(next_epoch),
            None => return Err(BlockError::NoEligibleProducers),
        };

        let mut builder = BlockBuilder::new(next_epoch, parent_hash, producer, timestamp);
        for tx in transactions {
            builder.push_tx(tx);
        }

        // Tick the chain state
        let _summary = self.state.tick();

        // Build block with current state root (simplified — real impl would compute from state)
        let state_root = self.compute_state_root();
        let block = builder.build(state_root);

        self.chain.append(block)
    }

    /// Compute a simplified state root from current chain state.
    fn compute_state_root(&self) -> Hash {
        let mut hasher = Sha256::new();
        hasher.update(b"state:");
        hasher.update(self.state.epoch().to_le_bytes());
        hasher.update(self.chain.tip());
        hasher.finalize().into()
    }
}

/// Genesis errors.
#[derive(Debug, PartialEq, Eq)]
pub enum GenesisError {
    AllocationExceedsSupply {
        allocated: StakeAmount,
        supply: StakeAmount,
    },
    NoAllocations,
    InvalidParam(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_devnet_genesis_valid() {
        let config = GenesisConfig::devnet();
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_testnet_genesis_valid() {
        let config = GenesisConfig::testnet();
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_genesis_state_root_deterministic() {
        let c1 = GenesisConfig::devnet();
        let c2 = GenesisConfig::devnet();
        assert_eq!(c1.state_root(), c2.state_root());
    }

    #[test]
    fn test_genesis_state_root_changes_with_chain_id() {
        let mut c1 = GenesisConfig::devnet();
        let mut c2 = GenesisConfig::devnet();
        c2.chain_id = "prova-other".to_string();
        assert_ne!(c1.state_root(), c2.state_root());
    }

    #[test]
    fn test_genesis_block() {
        let config = GenesisConfig::devnet();
        let block = config.build_genesis_block();
        assert_eq!(block.header.epoch, 0);
        assert_eq!(block.header.parent_hash, [0; 32]);
        assert_eq!(block.header.tx_count, 0);
        assert!(block.validate_internal().is_ok());
    }

    #[test]
    fn test_genesis_block_deterministic() {
        let c1 = GenesisConfig::devnet();
        let c2 = GenesisConfig::devnet();
        assert_eq!(
            c1.build_genesis_block().header.hash(),
            c2.build_genesis_block().header.hash()
        );
    }

    #[test]
    fn test_build_chain_state() {
        let config = GenesisConfig::devnet();
        let genesis = config.build_chain_state().unwrap();

        assert_eq!(genesis.chain.height(), 0);
        assert_eq!(genesis.state.epoch(), 0);
        assert!(genesis.schedule.is_some());

        let schedule = genesis.schedule.as_ref().unwrap();
        assert_eq!(schedule.entries().len(), 11); // faucet + 10 validators
    }

    #[test]
    fn test_produce_blocks() {
        let config = GenesisConfig::devnet();
        let mut genesis = config.build_chain_state().unwrap();

        // Produce 10 empty blocks
        for _ in 0..10 {
            genesis.produce_block(vec![]).unwrap();
        }

        assert_eq!(genesis.chain.height(), 10);
        assert_eq!(genesis.state.epoch(), 10);
    }

    #[test]
    fn test_produce_blocks_with_transactions() {
        let config = GenesisConfig::devnet();
        let mut genesis = config.build_chain_state().unwrap();

        // Block 1: stake deposit
        genesis
            .produce_block(vec![Transaction::StakeOp(StakeOp::Deposit {
                provider: Address::test(1),
                amount: 5_000_000,
            })])
            .unwrap();

        // Block 2: model registration + inference commit
        genesis
            .produce_block(vec![
                Transaction::RegisterModel {
                    owner: Address::test(1),
                    model_hash: [0x42; 32],
                    name: "llama-7b-q8".into(),
                    layer_count: 32,
                    arch_group: ArchGroup::new("nvidia-sm89-int8"),
                },
                Transaction::InferenceCommit {
                    provider: Address::test(1),
                    model_id: ModelId([0x42; 32]),
                    arch_group: ArchGroup::new("nvidia-sm89-int8"),
                    input_hash: [0xAA; 32],
                    activation_root: [0xBB; 32],
                    leaf_count: 33,
                },
            ])
            .unwrap();

        assert_eq!(genesis.chain.height(), 2);

        // Verify block 2 has 2 transactions
        let block2 = genesis.chain.get_at_epoch(2).unwrap();
        assert_eq!(block2.transactions.len(), 2);
    }

    #[test]
    fn test_allocation_exceeds_supply() {
        let mut config = GenesisConfig::devnet();
        config.total_supply = 1; // way too small
        assert_eq!(
            config.validate(),
            Err(GenesisError::AllocationExceedsSupply {
                allocated: 200_000_000_000,
                supply: 1,
            })
        );
    }

    #[test]
    fn test_no_allocations() {
        let mut config = GenesisConfig::devnet();
        config.allocations.clear();
        assert_eq!(config.validate(), Err(GenesisError::NoAllocations));
    }

    #[test]
    fn test_invalid_fee_bps() {
        let mut config = GenesisConfig::devnet();
        config.network_fee_bps = 20000;
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_chain_continuity() {
        let config = GenesisConfig::devnet();
        let mut genesis = config.build_chain_state().unwrap();

        let mut hashes = vec![genesis.genesis_hash];

        for _ in 0..5 {
            let hash = genesis.produce_block(vec![]).unwrap();
            hashes.push(hash);
        }

        // Verify chain links
        for i in 1..hashes.len() {
            let block = genesis.chain.get(&hashes[i]).unwrap();
            assert_eq!(block.header.parent_hash, hashes[i - 1]);
        }
    }

    #[test]
    fn test_producer_rotation() {
        let config = GenesisConfig::devnet();
        let mut genesis = config.build_chain_state().unwrap();

        let mut producers = Vec::new();
        for _ in 0..20 {
            genesis.produce_block(vec![]).unwrap();
            let block = genesis.chain.get_at_epoch(genesis.chain.height()).unwrap();
            producers.push(block.header.producer);
        }

        // With 11 providers, we should see at least 2 different producers in 20 blocks
        let unique: std::collections::HashSet<_> = producers.iter().collect();
        assert!(
            unique.len() >= 2,
            "expected producer rotation, got {} unique in 20 blocks",
            unique.len()
        );
    }
}
