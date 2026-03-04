//! Testnet configuration loader and launcher.
//!
//! Parses genesis.toml + bootnodes.toml → GenesisConfig → GenesisState.
//! This module bridges the TOML config files to the chain's genesis system.

use prova_chain::genesis::{GenesisConfig, GenesisState, GenesisError};
use prova_chain::types::*;
use std::collections::BTreeMap;
use std::path::Path;

// ── TOML config structures ──────────────────────────────────────────────────

/// Parsed genesis.toml allocation entry.
#[derive(Debug, Clone)]
pub struct AllocationEntry {
    pub label: String,
    pub address: String,
    pub amount: u64,
}

/// Parsed genesis.toml model entry.
#[derive(Debug, Clone)]
pub struct ModelEntry {
    pub name: String,
    pub layer_count: u32,
    pub arch_group: String,
    pub owner: String,
}

/// Boot node definition from bootnodes.toml.
#[derive(Debug, Clone)]
pub struct BootNode {
    pub id: String,
    pub label: String,
    pub peer_id: String,
    pub addrs: Vec<String>,
    pub validator_addr: String,
    pub region: String,
}

/// Full testnet configuration parsed from TOML files.
#[derive(Debug, Clone)]
pub struct TestnetConfig {
    pub chain_id: String,
    pub genesis_time: u64,
    pub total_supply: u64,
    pub epoch_duration_secs: u64,
    pub challenge_window: u64,
    pub min_provider_stake: u64,
    pub min_challenger_stake: u64,
    pub network_fee_bps: u32,
    pub storage_reward_bps: u32,
    pub block_reward: u64,
    pub allocations: Vec<AllocationEntry>,
    pub models: Vec<ModelEntry>,
    pub bootnodes: Vec<BootNode>,
    pub gas_base_fee: u64,
    pub gas_max_block: u64,
}

impl TestnetConfig {
    /// Load the hardcoded testnet-1 configuration.
    /// In production, this would parse TOML files from disk.
    pub fn testnet_1() -> Self {
        Self {
            chain_id: "prova-testnet-1".into(),
            genesis_time: 1741046400,
            total_supply: 1_000_000_000_000,
            epoch_duration_secs: 30,
            challenge_window: 240,
            min_provider_stake: 10_000_000,
            min_challenger_stake: 5_000_000,
            network_fee_bps: 50,
            storage_reward_bps: 5000,
            block_reward: 380,
            allocations: vec![
                AllocationEntry { label: "faucet".into(), address: "faucet".into(), amount: 500_000_000_000 },
                AllocationEntry { label: "foundation".into(), address: "foundation".into(), amount: 200_000_000_000 },
                AllocationEntry { label: "boot-validator-1".into(), address: "val1".into(), amount: 50_000_000_000 },
                AllocationEntry { label: "boot-validator-2".into(), address: "val2".into(), amount: 50_000_000_000 },
                AllocationEntry { label: "boot-validator-3".into(), address: "val3".into(), amount: 50_000_000_000 },
                AllocationEntry { label: "boot-validator-4".into(), address: "val4".into(), amount: 50_000_000_000 },
                AllocationEntry { label: "boot-validator-5".into(), address: "val5".into(), amount: 50_000_000_000 },
                AllocationEntry { label: "community-reserve".into(), address: "community".into(), amount: 50_000_000_000 },
            ],
            models: vec![
                ModelEntry { name: "llama-7b-q8".into(), layer_count: 32, arch_group: "nvidia-sm89-int8".into(), owner: "foundation".into() },
                ModelEntry { name: "mistral-7b-q8".into(), layer_count: 32, arch_group: "nvidia-sm89-int8".into(), owner: "foundation".into() },
            ],
            bootnodes: vec![
                BootNode { id: "boot-1".into(), label: "EU-West".into(), peer_id: "boot1".into(), addrs: vec!["/dns4/boot1.testnet.prova.network/tcp/9000".into()], validator_addr: "val1".into(), region: "eu-west".into() },
                BootNode { id: "boot-2".into(), label: "US-East".into(), peer_id: "boot2".into(), addrs: vec!["/dns4/boot2.testnet.prova.network/tcp/9000".into()], validator_addr: "val2".into(), region: "us-east".into() },
                BootNode { id: "boot-3".into(), label: "AP-Southeast".into(), peer_id: "boot3".into(), addrs: vec!["/dns4/boot3.testnet.prova.network/tcp/9000".into()], validator_addr: "val3".into(), region: "ap-southeast".into() },
                BootNode { id: "boot-4".into(), label: "US-West".into(), peer_id: "boot4".into(), addrs: vec!["/dns4/boot4.testnet.prova.network/tcp/9000".into()], validator_addr: "val4".into(), region: "us-west".into() },
                BootNode { id: "boot-5".into(), label: "EU-Central".into(), peer_id: "boot5".into(), addrs: vec!["/dns4/boot5.testnet.prova.network/tcp/9000".into()], validator_addr: "val5".into(), region: "eu-central".into() },
            ],
            gas_base_fee: 100,
            gas_max_block: 30_000_000,
        }
    }

    /// Convert to chain GenesisConfig.
    pub fn to_genesis_config(&self) -> GenesisConfig {
        let mut allocations = BTreeMap::new();
        for (i, alloc) in self.allocations.iter().enumerate() {
            allocations.insert(Address::test(i as u8), alloc.amount);
        }

        GenesisConfig {
            chain_id: self.chain_id.clone(),
            genesis_time: self.genesis_time,
            allocations,
            total_supply: self.total_supply,
            epoch_duration_secs: self.epoch_duration_secs,
            challenge_window: self.challenge_window,
            min_provider_stake: self.min_provider_stake,
            min_challenger_stake: self.min_challenger_stake,
            network_fee_bps: self.network_fee_bps,
            storage_reward_bps: self.storage_reward_bps,
            block_reward: self.block_reward,
        }
    }

    /// Build full genesis state from this testnet config.
    pub fn build_genesis(&self) -> Result<GenesisState, GenesisError> {
        let config = self.to_genesis_config();
        config.build_chain_state()
    }

    /// Validate the testnet config (allocations sum, bootnode coverage, etc).
    pub fn validate(&self) -> Result<(), TestnetConfigError> {
        // Check allocations sum
        let total_alloc: u64 = self.allocations.iter().map(|a| a.amount).sum();
        if total_alloc > self.total_supply {
            return Err(TestnetConfigError::AllocationOverflow { total_alloc, supply: self.total_supply });
        }

        // Need at least 1 bootnode
        if self.bootnodes.is_empty() {
            return Err(TestnetConfigError::NoBootnodes);
        }

        // Each bootnode must have at least 1 address
        for bn in &self.bootnodes {
            if bn.addrs.is_empty() {
                return Err(TestnetConfigError::BootnodeNoAddr(bn.id.clone()));
            }
        }

        // Need at least 3 bootnodes for byzantine tolerance (3f+1 with f=0 → min 1, but 3 for meaningful decentralization)
        if self.bootnodes.len() < 3 {
            return Err(TestnetConfigError::InsufficientBootnodes(self.bootnodes.len()));
        }

        // Validate underlying genesis config
        let gc = self.to_genesis_config();
        gc.validate().map_err(TestnetConfigError::Genesis)?;

        Ok(())
    }

    /// Get bootstrap peer addresses for node config.
    pub fn bootstrap_addrs(&self) -> Vec<String> {
        self.bootnodes.iter()
            .flat_map(|bn| bn.addrs.clone())
            .collect()
    }

    /// Number of pre-registered models.
    pub fn model_count(&self) -> usize {
        self.models.len()
    }

    /// Geographic diversity score: number of unique regions.
    pub fn region_diversity(&self) -> usize {
        let regions: std::collections::HashSet<_> = self.bootnodes.iter().map(|bn| &bn.region).collect();
        regions.len()
    }
}

#[derive(Debug, PartialEq)]
pub enum TestnetConfigError {
    AllocationOverflow { total_alloc: u64, supply: u64 },
    NoBootnodes,
    BootnodeNoAddr(String),
    InsufficientBootnodes(usize),
    Genesis(GenesisError),
}

// ── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_testnet1_valid() {
        let config = TestnetConfig::testnet_1();
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_testnet1_chain_id() {
        let config = TestnetConfig::testnet_1();
        assert_eq!(config.chain_id, "prova-testnet-1");
    }

    #[test]
    fn test_testnet1_allocations_sum() {
        let config = TestnetConfig::testnet_1();
        let total: u64 = config.allocations.iter().map(|a| a.amount).sum();
        assert_eq!(total, config.total_supply);
    }

    #[test]
    fn test_testnet1_bootnode_count() {
        let config = TestnetConfig::testnet_1();
        assert_eq!(config.bootnodes.len(), 5);
    }

    #[test]
    fn test_testnet1_region_diversity() {
        let config = TestnetConfig::testnet_1();
        assert_eq!(config.region_diversity(), 5);
    }

    #[test]
    fn test_testnet1_model_count() {
        let config = TestnetConfig::testnet_1();
        assert_eq!(config.model_count(), 2);
    }

    #[test]
    fn test_testnet1_bootstrap_addrs() {
        let config = TestnetConfig::testnet_1();
        let addrs = config.bootstrap_addrs();
        assert_eq!(addrs.len(), 5);
        assert!(addrs[0].contains("boot1.testnet.prova.network"));
    }

    #[test]
    fn test_testnet1_genesis_build() {
        let config = TestnetConfig::testnet_1();
        let genesis = config.build_genesis().unwrap();
        assert_eq!(genesis.chain.height(), 0);
        assert_eq!(genesis.config.chain_id, "prova-testnet-1");
    }

    #[test]
    fn test_testnet1_genesis_deterministic() {
        let c1 = TestnetConfig::testnet_1();
        let c2 = TestnetConfig::testnet_1();
        let g1 = c1.build_genesis().unwrap();
        let g2 = c2.build_genesis().unwrap();
        assert_eq!(g1.genesis_hash, g2.genesis_hash);
    }

    #[test]
    fn test_testnet1_produce_blocks() {
        let config = TestnetConfig::testnet_1();
        let mut genesis = config.build_genesis().unwrap();
        for _ in 0..5 {
            genesis.produce_block(vec![]).unwrap();
        }
        assert_eq!(genesis.chain.height(), 5);
    }

    #[test]
    fn test_allocation_overflow() {
        let mut config = TestnetConfig::testnet_1();
        config.total_supply = 1; // too small
        assert!(matches!(config.validate(), Err(TestnetConfigError::AllocationOverflow { .. })));
    }

    #[test]
    fn test_no_bootnodes() {
        let mut config = TestnetConfig::testnet_1();
        config.bootnodes.clear();
        assert_eq!(config.validate(), Err(TestnetConfigError::NoBootnodes));
    }

    #[test]
    fn test_insufficient_bootnodes() {
        let mut config = TestnetConfig::testnet_1();
        config.bootnodes.truncate(2);
        assert_eq!(config.validate(), Err(TestnetConfigError::InsufficientBootnodes(2)));
    }

    #[test]
    fn test_bootnode_no_addr() {
        let mut config = TestnetConfig::testnet_1();
        config.bootnodes[0].addrs.clear();
        assert_eq!(config.validate(), Err(TestnetConfigError::BootnodeNoAddr("boot-1".into())));
    }

    #[test]
    fn test_to_genesis_config() {
        let config = TestnetConfig::testnet_1();
        let gc = config.to_genesis_config();
        assert_eq!(gc.chain_id, "prova-testnet-1");
        assert_eq!(gc.allocations.len(), 8);
        assert_eq!(gc.challenge_window, 240);
        assert!(gc.validate().is_ok());
    }
}
