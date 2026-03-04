pub mod config;
pub mod canonical_cpu;
pub mod cli;
pub mod determinism;
pub mod devnet;
pub mod executor;
pub mod llamacpp;
pub mod metrics;
pub mod merkle;
pub mod multinode;
pub mod network;
pub mod participant;
pub mod pdp;
pub mod rpc;
pub mod runner;
pub mod storage;
pub mod sync;
pub mod pricing;
pub mod submitter;
pub mod wallet;
pub mod snapshot_serve;
pub mod fast_sync;
pub mod watcher;
pub mod shutdown;
pub mod subscriptions;
pub mod explorer;
pub mod tls;

use merkle::{hash_tensor, verify_proof, ActivationMerkleTree, DType};
use prova_chain::types::ModelId;
use runner::MockRunner;

fn main() {
    println!("=== Prova Node — Activation Merkle Tree Demo ===\n");

    // Simulate a 32-layer model inference
    let layer_count = 32;
    println!("Simulating {layer_count}-layer model inference...\n");

    // Generate fake activation hashes (in production, these come from actual inference)
    let activation_hashes: Vec<[u8; 32]> = (0..=layer_count as u8)
        .map(|i| {
            // Simulate hash_tensor output for layer i
            let _shape = [1u64, 2048, 4096];
            let fake_data = vec![i; 4]; // Placeholder
            hash_tensor(DType::Fp32, &[1], &fake_data)
        })
        .collect();

    // Build the tree
    let tree = ActivationMerkleTree::build(&activation_hashes);
    let root = tree.root();

    println!("Tree built:");
    println!(
        "  Leaves: {} (input + {layer_count} layers)",
        layer_count + 1
    );
    println!("  Root:   {}", hex::encode(&root));
    println!();

    // Generate and verify proofs for a few layers
    for layer in [0, 15, 31, layer_count] {
        let proof = tree.prove(layer);
        let valid = verify_proof(&proof, &root);
        println!(
            "  Layer {layer:>2}: proof has {} siblings, valid = {valid}",
            proof.siblings.len()
        );
    }

    // ---- MockRunner demo ----
    println!("\n=== Mock Inference Runner ===\n");
    let model_id = ModelId([0x42; 32]);
    let prompt = "The meaning of life is";

    let result = MockRunner::run(&model_id, prompt, 32);
    println!("  Prompt:     \"{}\"", prompt);
    println!("  Layers:     {}", result.layer_count());
    println!("  Root:       {}", hex::encode(&result.activation_root()));
    println!("  Input hash: {}", hex::encode(&result.input_hash));

    // Faulty run
    let faulty = MockRunner::run_faulty(&model_id, prompt, 32, 21);
    println!("\n  Faulty run (fault at layer 21):");
    println!("  Root:       {}", hex::encode(&faulty.activation_root()));
    println!(
        "  Roots match: {}",
        result.activation_root() == faulty.activation_root()
    );

    let first_diff = result
        .activation_hashes
        .iter()
        .zip(faulty.activation_hashes.iter())
        .position(|(a, b)| a != b)
        .unwrap();
    println!("  First divergence at layer: {}", first_diff);

    println!("\n=== Bisection Simulation ===\n");

    // Simulate a dispute: prover and challenger agree up to layer 20, disagree after
    let dispute_layer = simulate_bisection(layer_count as u32 + 1);
    println!("Bisection isolated disputed layer: {dispute_layer}");
    println!(
        "Verification cost: 1/{} = {:.1}% of total inference",
        layer_count + 1,
        100.0 / (layer_count + 1) as f64
    );
}

/// Simulate the bisection algorithm finding a disputed layer.
fn simulate_bisection(total_layers: u32) -> u32 {
    let mut lo = 0u32;
    let mut hi = total_layers;
    let mut round = 0;

    // Simulate: parties agree on layers 0..20, disagree from 21 onwards
    let actual_dispute = 21;

    while hi - lo > 1 {
        let mid = (lo + hi) / 2;
        round += 1;

        let agree = mid < actual_dispute;
        println!(
            "  Round {round}: lo={lo}, hi={hi}, mid={mid} → {}",
            if agree {
                "agree (move lo)"
            } else {
                "disagree (move hi)"
            }
        );

        if agree {
            lo = mid;
        } else {
            hi = mid;
        }
    }

    println!("  Bisection complete in {round} rounds");
    hi
}

// Needed for hex encoding in the demo
mod hex {
    pub fn encode(data: &[u8]) -> String {
        data.iter().map(|b| format!("{b:02x}")).collect()
    }
}
