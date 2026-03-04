//! End-to-end simulation of the Prova QBP protocol.
//!
//! Demonstrates the full lifecycle:
//! 1. Register a model
//! 2. Provider commits inference result
//! 3. Challenger disputes
//! 4. Bisection narrows to single layer
//! 5. Judgment resolves dispute

use crate::types::*;
use crate::registry::*;
use crate::commit::*;
use crate::dispute::*;

/// Run a complete QBP protocol simulation.
pub fn run_simulation() -> SimulationResult {
    let mut registry = ModelRegistry::new();
    let mut commits = CommitStore::new(CommitConfig::default());
    let mut arena = DisputeArena::new(DisputeConfig::default());

    let provider = Address::test(1);
    let challenger = Address::test(2);
    let mut epoch: Epoch = 1000;

    // Step 1: Register model
    let model_id = ModelId({
        let mut h = [0u8; 32];
        h[0] = 0x42;
        h
    });

    let layer_count = 32u32;
    let manifest = ModelManifest {
        model_id,
        name: "TinyLlama-1.1B-Q8_0".to_string(),
        layer_count,
        layer_hashes: (0..layer_count)
            .map(|i| LayerWeightHash {
                layer_index: i,
                weight_hash: {
                    let mut h = [0u8; 32];
                    h[0] = i as u8;
                    h[1] = 0xFF;
                    h
                },
            })
            .collect(),
        arch_groups: vec![
            ArchGroup::new("nvidia-sm89-int8"),
            ArchGroup::new("nvidia-sm90-int8"),
        ],
        registrar: provider,
        registered_at: epoch,
    };

    registry.register(manifest).expect("registration should succeed");

    // Step 2: Provider commits inference
    epoch += 10;
    let leaf_count = layer_count + 1; // input + layers
    let provider_root = [0x11; 32]; // Provider's activation root

    let commit_id = commits.publish(
        provider,
        model_id,
        ArchGroup::new("nvidia-sm89-int8"),
        [0xBB; 32], // input hash
        provider_root,
        leaf_count,
        epoch,
    );

    // Step 3: Challenger disputes (different root)
    epoch += 5;
    let challenger_root = [0x22; 32];

    commits.mark_disputed(&commit_id).expect("should mark disputed");

    let dispute_id = arena
        .open_dispute(
            commit_id,
            provider,
            challenger,
            model_id,
            ArchGroup::new("nvidia-sm89-int8"),
            provider_root,
            challenger_root,
            leaf_count,
            epoch,
        )
        .expect("should open dispute");

    // Step 4: Bisection game
    // Simulate: provider and challenger agree on layers 0..20, disagree from 21
    let actual_dispute_layer = 21u32;
    let mut rounds = 0u32;

    loop {
        let dispute = arena.get(dispute_id).unwrap();
        match &dispute.phase {
            DisputePhase::AwaitingMidpoint { mid, .. } => {
                let mid = *mid;
                epoch += 1;

                // Both parties compute honestly: agree if mid < dispute layer
                let provider_hash = if mid < actual_dispute_layer {
                    [0xAA; 32] // Agreed hash
                } else {
                    [0x11; 32] // Provider's version
                };

                let challenger_hash = if mid < actual_dispute_layer {
                    [0xAA; 32] // Agreed hash
                } else {
                    [0x22; 32] // Challenger's version
                };

                arena
                    .submit_midpoint(dispute_id, provider, provider_hash, epoch)
                    .expect("provider submit");

                let step = arena
                    .submit_midpoint(dispute_id, challenger, challenger_hash, epoch)
                    .expect("challenger submit");

                rounds += 1;

                match step {
                    BisectionStep::NarrowedToLayer { layer, .. } => {
                        // Submit activations
                        arena
                            .submit_activation(dispute_id, provider, [0xDD; 32], epoch)
                            .unwrap();
                        arena
                            .submit_activation(dispute_id, challenger, [0xEE; 32], epoch)
                            .unwrap();

                        // Judge: provider was correct
                        let winner = arena.judge(dispute_id, true).unwrap();

                        return SimulationResult {
                            model_name: "TinyLlama-1.1B-Q8_0".to_string(),
                            layer_count,
                            disputed_layer: layer,
                            bisection_rounds: rounds,
                            expected_rounds: (leaf_count.next_power_of_two() as f64).log2() as u32,
                            winner,
                            provider_won: winner == provider,
                            total_epochs: epoch - 1000,
                        };
                    }
                    _ => continue,
                }
            }
            _ => break,
        }
    }

    unreachable!("bisection should always complete")
}

/// Results from a simulation run.
#[derive(Debug)]
pub struct SimulationResult {
    pub model_name: String,
    pub layer_count: u32,
    pub disputed_layer: u32,
    pub bisection_rounds: u32,
    pub expected_rounds: u32,
    pub winner: Address,
    pub provider_won: bool,
    pub total_epochs: u64,
}

impl std::fmt::Display for SimulationResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "╔══════════════════════════════════════════╗")?;
        writeln!(f, "║     Prova QBP Simulation Results         ║")?;
        writeln!(f, "╠══════════════════════════════════════════╣")?;
        writeln!(f, "║ Model:      {:<28} ║", self.model_name)?;
        writeln!(f, "║ Layers:     {:<28} ║", self.layer_count)?;
        writeln!(f, "║ Disputed:   layer {:<22} ║", self.disputed_layer)?;
        writeln!(f, "║ Rounds:     {}/{} (actual/expected)     ║", self.bisection_rounds, self.expected_rounds)?;
        writeln!(f, "║ Winner:     {:<28} ║", if self.provider_won { "Provider ✓" } else { "Challenger ✓" })?;
        writeln!(f, "║ Duration:   {} epochs                   ║", self.total_epochs)?;
        writeln!(f, "║ Efficiency: 1/{} layer re-execution    ║", self.layer_count)?;
        writeln!(f, "╚══════════════════════════════════════════╝")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_full_simulation() {
        let result = run_simulation();
        assert_eq!(result.model_name, "TinyLlama-1.1B-Q8_0");
        assert_eq!(result.layer_count, 32);
        assert_eq!(result.disputed_layer, 21);
        assert!(result.provider_won);
        // 33 leaves padded to 64 → 6 rounds max. Bisection should find layer 21 in ≤6 rounds
        assert!(result.bisection_rounds <= 6, "took {} rounds", result.bisection_rounds);
    }
}
