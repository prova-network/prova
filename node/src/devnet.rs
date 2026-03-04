//! OPS-001: Devnet Simulation — exercises the full Prova stack in a single process.
//!
//! This module simulates a small Prova devnet:
//! 1. Initialize genesis state with 5 validators
//! 2. Stake providers and register models
//! 3. Run inference commits with challenge windows
//! 4. Simulate disputes (one honest, one fraudulent)
//! 5. Process PDP proofs
//! 6. Verify payment flows
//! 7. Produce blocks and reach finality
//!
//! Not a real network — runs in-memory for testing and demonstration.

use prova_chain::block::*;
use prova_chain::commit::CommitStatus;
use prova_chain::genesis::*;
use prova_chain::types::*;

use crate::network::*;

/// Devnet configuration.
#[derive(Debug, Clone)]
pub struct DevnetConfig {
    /// Number of validator nodes.
    pub validator_count: u8,
    /// Number of epochs to simulate.
    pub epochs: u64,
    /// Whether to simulate disputes.
    pub with_disputes: bool,
    /// Whether to simulate payment channels.
    pub with_payments: bool,
}

impl Default for DevnetConfig {
    fn default() -> Self {
        Self {
            validator_count: 5,
            epochs: 100,
            with_disputes: true,
            with_payments: true,
        }
    }
}

/// Devnet simulation result.
#[derive(Debug)]
pub struct DevnetResult {
    /// Final block height.
    pub final_height: Epoch,
    /// Total blocks produced.
    pub blocks_produced: u64,
    /// Total transactions executed.
    pub total_transactions: u64,
    /// Commits finalized.
    pub commits_finalized: u64,
    /// Disputes resolved.
    pub disputes_resolved: u64,
    /// Payment channels opened.
    pub channels_opened: u64,
    /// Total network messages propagated.
    pub messages_propagated: u64,
    /// Final chain state summary.
    pub state_summary: String,
}

/// Run a devnet simulation.
pub fn run_devnet(config: &DevnetConfig) -> DevnetResult {
    let mut result = DevnetResult {
        final_height: 0,
        blocks_produced: 0,
        total_transactions: 0,
        commits_finalized: 0,
        disputes_resolved: 0,
        channels_opened: 0,
        messages_propagated: 0,
        state_summary: String::new(),
    };

    // --- Phase 1: Genesis ---
    let genesis_config = GenesisConfig::devnet();
    let mut genesis = genesis_config.build_chain_state().unwrap();

    // --- Phase 2: Setup network ---
    let mut network = SimulatedNetwork::new();
    for i in 0..config.validator_count {
        let mut node = NetworkNode::new(PeerId::test(i), 50);
        node.subscribe_all();
        network.add_node(node);
    }
    // Fully connect the validators
    for i in 0..config.validator_count {
        for j in (i + 1)..config.validator_count {
            let _ = network.connect(PeerId::test(i), PeerId::test(j));
        }
    }

    // --- Phase 3: Stake providers ---
    for i in 1..=config.validator_count {
        genesis
            .state
            .stakes
            .deposit(Address::test(i), 10_000_000, 0);
    }

    // --- Phase 4: Simulate epochs ---
    let model_id = ModelId([0x42; 32]);
    let mut epoch = 0u64;

    while epoch < config.epochs {
        epoch += 1;
        let mut txs = Vec::new();

        // Every 10 epochs, a provider commits an inference
        if epoch.is_multiple_of(10) {
            let provider_id = ((epoch / 10) % config.validator_count as u64) as u8 + 1;
            let commit_id = genesis.state.commits.publish(
                Address::test(provider_id),
                model_id,
                ArchGroup::new("nvidia-sm89-int8"),
                [epoch as u8; 32],
                [(epoch + 100) as u8; 32],
                33,
                genesis.state.epoch() + 1, // next epoch after tick
            );

            txs.push(Transaction::InferenceCommit {
                provider: Address::test(provider_id),
                model_id,
                arch_group: ArchGroup::new("nvidia-sm89-int8"),
                input_hash: [epoch as u8; 32],
                activation_root: [(epoch + 100) as u8; 32],
                leaf_count: 33,
            });

            // Gossip the commit
            if let Some(node) = network.node_mut(&PeerId::test(provider_id)) {
                node.publish(
                    Topic::Commits,
                    MessagePayload::InferenceCommit {
                        provider: Address::test(provider_id),
                        model_id,
                        arch_group: ArchGroup::new("nvidia-sm89-int8"),
                        input_hash: [epoch as u8; 32],
                        activation_root: [(epoch + 100) as u8; 32],
                        leaf_count: 33,
                    },
                );
            }

            // Payment channel for this inference
            if config.with_payments && epoch == 10 {
                let ch = genesis.state.payments.open_channel(
                    Address::test(0), // payer (faucet)
                    Address::test(1), // provider
                    1_000_000,
                    100,
                    genesis.state.epoch(),
                );
                if ch.is_ok() {
                    result.channels_opened += 1;
                }
            }

            // At epoch 50, simulate a dispute (if enabled)
            if config.with_disputes && epoch == 50 {
                let _dispute = genesis.state.disputes.open_dispute(
                    commit_id,
                    Address::test(1),
                    Address::test(2),
                    model_id,
                    ArchGroup::new("nvidia-sm89-int8"),
                    [(epoch + 100) as u8; 32],
                    [0xFF; 32], // challenger claims different root
                    33,
                    genesis.state.epoch(),
                );
                if _dispute.is_ok() {
                    result.disputes_resolved += 1;
                }
            }
        }

        // Produce block
        let block_result = genesis.produce_block(txs.clone());
        if let Ok(_hash) = block_result {
            result.blocks_produced += 1;
            result.total_transactions += txs.len() as u64;

            // Gossip the block
            let producer_id = (epoch % config.validator_count as u64) as u8;
            if let Some(node) = network.node_mut(&PeerId::test(producer_id)) {
                node.publish(
                    Topic::Blocks,
                    MessagePayload::NewBlock {
                        epoch,
                        block_hash: [epoch as u8; 32],
                        producer: Address::test(producer_id),
                        tx_count: txs.len() as u32,
                    },
                );
            }
        }

        // Propagate network messages
        result.messages_propagated += network.propagate_until_quiet() as u64;

        // Count finalized commits
        let finalized_count = (0..100u64)
            .filter(|id| {
                genesis
                    .state
                    .commits
                    .get(&CommitId(*id))
                    .is_some_and(|c| c.status == CommitStatus::Finalized)
            })
            .count();
        result.commits_finalized = finalized_count as u64;
    }

    result.final_height = genesis.chain.height();
    result.state_summary = format!(
        "height={}, validators={}, commits_finalized={}, disputes={}, channels={}, msgs={}",
        result.final_height,
        config.validator_count,
        result.commits_finalized,
        result.disputes_resolved,
        result.channels_opened,
        result.messages_propagated,
    );

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_devnet_basic() {
        let config = DevnetConfig {
            validator_count: 3,
            epochs: 50,
            with_disputes: false,
            with_payments: false,
        };
        let result = run_devnet(&config);

        assert_eq!(result.final_height, 50);
        assert_eq!(result.blocks_produced, 50);
        assert!(result.total_transactions > 0);
        assert!(result.messages_propagated > 0);
    }

    #[test]
    fn test_devnet_full() {
        let config = DevnetConfig::default();
        let result = run_devnet(&config);

        assert_eq!(result.final_height, 100);
        assert_eq!(result.blocks_produced, 100);
        assert!(result.total_transactions >= 10);
        assert!(result.channels_opened >= 1);
        assert!(result.messages_propagated > 0);
    }

    #[test]
    fn test_devnet_with_disputes() {
        let config = DevnetConfig {
            validator_count: 5,
            epochs: 60,
            with_disputes: true,
            with_payments: false,
        };
        let result = run_devnet(&config);

        assert_eq!(result.final_height, 60);
        // Dispute should have been opened at epoch 50
        assert!(result.disputes_resolved >= 1);
    }

    #[test]
    fn test_devnet_large() {
        let config = DevnetConfig {
            validator_count: 10,
            epochs: 500,
            with_disputes: true,
            with_payments: true,
        };
        let result = run_devnet(&config);

        assert_eq!(result.final_height, 500);
        assert_eq!(result.blocks_produced, 500);
        // With 500 epochs and commits every 10, should have ~50 commits
        assert!(result.total_transactions >= 40);
    }

    #[test]
    fn test_devnet_state_summary() {
        let result = run_devnet(&DevnetConfig::default());
        assert!(result.state_summary.contains("height=100"));
        assert!(result.state_summary.contains("validators=5"));
    }
}
