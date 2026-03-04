//! Bisection Game Participant — plays the QBP dispute game.
//!
//! Connects the inference runner to the chain's dispute arena,
//! enabling a node to participate as either provider or challenger
//! in a bisection dispute.

use crate::merkle::Hash;
use crate::runner::InferenceResult;
use prova_chain::dispute::*;
use prova_chain::types::*;

/// A participant in the QBP protocol.
#[derive(Debug)]
pub struct QbpParticipant {
    /// This participant's address.
    pub address: Address,
    /// Role in the current dispute.
    pub role: ParticipantRole,
    /// Cached inference result with activations.
    inference: InferenceResult,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParticipantRole {
    Provider,
    Challenger,
}

impl QbpParticipant {
    /// Create a new participant with their inference result.
    pub fn new(address: Address, role: ParticipantRole, inference: InferenceResult) -> Self {
        Self {
            address,
            role,
            inference,
        }
    }

    /// Get the activation hash at a specific layer index.
    pub fn activation_at(&self, layer: u32) -> Hash {
        self.inference.activation_hashes[layer as usize]
    }

    /// Get the Merkle root.
    pub fn root(&self) -> Hash {
        self.inference.activation_root()
    }

    /// Generate a Merkle proof for a layer.
    pub fn prove_layer(&self, layer: u32) -> crate::merkle::MerkleProof {
        self.inference.merkle_tree.prove(layer as usize)
    }
}

/// Orchestrate a complete bisection dispute between two participants.
///
/// Returns the disputed layer index and the winner's address.
pub fn run_dispute(
    provider: &QbpParticipant,
    challenger: &QbpParticipant,
    arena: &mut DisputeArena,
    commit_id: CommitId,
    model_id: ModelId,
    arch_group: ArchGroup,
    start_epoch: Epoch,
) -> DisputeOutcome {
    assert_eq!(provider.role, ParticipantRole::Provider);
    assert_eq!(challenger.role, ParticipantRole::Challenger);

    let leaf_count = provider.inference.activation_hashes.len() as u32;

    let dispute_id = arena
        .open_dispute(
            commit_id,
            provider.address,
            challenger.address,
            model_id,
            arch_group,
            provider.root(),
            challenger.root(),
            leaf_count,
            start_epoch,
        )
        .expect("failed to open dispute");

    let mut epoch = start_epoch;
    let mut rounds = 0u32;

    loop {
        let dispute = arena.get(dispute_id).unwrap();

        match &dispute.phase {
            DisputePhase::AwaitingMidpoint { mid, .. } => {
                let mid = *mid;
                epoch += 1;

                // Each participant responds with their activation hash at midpoint
                let p_hash = provider.activation_at(mid);
                let c_hash = challenger.activation_at(mid);

                arena
                    .submit_midpoint(dispute_id, provider.address, p_hash, epoch)
                    .expect("provider midpoint");

                let step = arena
                    .submit_midpoint(dispute_id, challenger.address, c_hash, epoch)
                    .expect("challenger midpoint");

                rounds += 1;

                match step {
                    BisectionStep::NarrowedToLayer { layer, .. } => {
                        // Submit activations for final judgment
                        let p_activation = provider.activation_at(layer);
                        let c_activation = challenger.activation_at(layer);

                        arena
                            .submit_activation(dispute_id, provider.address, p_activation, epoch)
                            .unwrap();
                        arena
                            .submit_activation(dispute_id, challenger.address, c_activation, epoch)
                            .unwrap();

                        // In a real system, a verifier would re-execute the single layer.
                        // Here we simulate: the "correct" party is the provider (convention).
                        // In tests, the caller can determine who's actually correct.
                        let provider_correct = true; // Default assumption for simulation

                        let winner = arena.judge(dispute_id, provider_correct).unwrap();

                        return DisputeOutcome {
                            dispute_id,
                            disputed_layer: layer,
                            rounds,
                            winner,
                            provider_won: winner == provider.address,
                            epochs_elapsed: epoch - start_epoch,
                        };
                    }
                    _ => continue,
                }
            }
            phase => {
                panic!("unexpected dispute phase: {:?}", phase);
            }
        }
    }
}

/// Find the first layer where two inference results diverge.
pub fn find_divergence(a: &InferenceResult, b: &InferenceResult) -> Option<u32> {
    a.activation_hashes
        .iter()
        .zip(b.activation_hashes.iter())
        .position(|(x, y)| x != y)
        .map(|i| i as u32)
}

/// Outcome of a bisection dispute.
#[derive(Debug)]
pub struct DisputeOutcome {
    pub dispute_id: u64,
    pub disputed_layer: u32,
    pub rounds: u32,
    pub winner: Address,
    pub provider_won: bool,
    pub epochs_elapsed: u64,
}

impl std::fmt::Display for DisputeOutcome {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Dispute #{}: layer {} found in {} rounds ({} epochs). Winner: {} ({})",
            self.dispute_id,
            self.disputed_layer,
            self.rounds,
            self.epochs_elapsed,
            self.winner,
            if self.provider_won {
                "provider"
            } else {
                "challenger"
            }
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::merkle::verify_proof;
    use crate::runner::MockRunner;

    fn setup_participants(fault_layer: u32) -> (QbpParticipant, QbpParticipant) {
        let model_id = ModelId([0x42; 32]);
        let prompt = "The meaning of life is";
        let layer_count = 32;

        let provider_result = MockRunner::run(&model_id, prompt, layer_count);
        let challenger_result = MockRunner::run_faulty(&model_id, prompt, layer_count, fault_layer);

        let provider =
            QbpParticipant::new(Address::test(1), ParticipantRole::Provider, provider_result);
        let challenger = QbpParticipant::new(
            Address::test(2),
            ParticipantRole::Challenger,
            challenger_result,
        );

        (provider, challenger)
    }

    #[test]
    fn test_dispute_finds_correct_layer() {
        let (provider, challenger) = setup_participants(21);
        let mut arena = DisputeArena::new(DisputeConfig::default());

        let outcome = run_dispute(
            &provider,
            &challenger,
            &mut arena,
            CommitId(1),
            ModelId([0x42; 32]),
            ArchGroup::new("nvidia-sm89-int8"),
            1000,
        );

        assert_eq!(outcome.disputed_layer, 21);
        assert!(outcome.rounds <= 6, "should take ≤6 rounds for 33 leaves");
    }

    #[test]
    fn test_dispute_at_layer_0() {
        // Fault at layer 0 (input activation) → bisection finds layer 1
        // because it locates the *transition* between last-agree and first-disagree.
        // When activation[0] differs, the first transition (0→1) is disputed.
        // In production, input mismatches are caught pre-bisection (input is public).
        let (provider, challenger) = setup_participants(0);
        let mut arena = DisputeArena::new(DisputeConfig::default());

        let outcome = run_dispute(
            &provider,
            &challenger,
            &mut arena,
            CommitId(1),
            ModelId([0x42; 32]),
            ArchGroup::new("test"),
            1000,
        );

        // Bisection returns 1 (the first layer transition)
        assert_eq!(outcome.disputed_layer, 1);
    }

    #[test]
    fn test_dispute_at_last_layer() {
        let (provider, challenger) = setup_participants(32);
        let mut arena = DisputeArena::new(DisputeConfig::default());

        let outcome = run_dispute(
            &provider,
            &challenger,
            &mut arena,
            CommitId(1),
            ModelId([0x42; 32]),
            ArchGroup::new("test"),
            1000,
        );

        assert_eq!(outcome.disputed_layer, 32);
    }

    #[test]
    fn test_find_divergence() {
        let model_id = ModelId([0x42; 32]);
        let correct = MockRunner::run(&model_id, "test", 32);
        let faulty = MockRunner::run_faulty(&model_id, "test", 32, 15);

        assert_eq!(find_divergence(&correct, &faulty), Some(15));
    }

    #[test]
    fn test_no_divergence_identical_runs() {
        let model_id = ModelId([0x42; 32]);
        let r1 = MockRunner::run(&model_id, "test", 32);
        let r2 = MockRunner::run(&model_id, "test", 32);

        assert_eq!(find_divergence(&r1, &r2), None);
    }

    #[test]
    fn test_dispute_efficiency() {
        // Test across multiple fault positions to verify O(log L) holds
        // Start from 1: layer 0 faults are caught pre-bisection (input is public)
        for fault_layer in [1, 5, 10, 15, 20, 25, 30, 32] {
            let (provider, challenger) = setup_participants(fault_layer);
            let mut arena = DisputeArena::new(DisputeConfig::default());

            let outcome = run_dispute(
                &provider,
                &challenger,
                &mut arena,
                CommitId(fault_layer as u64),
                ModelId([0x42; 32]),
                ArchGroup::new("test"),
                1000,
            );

            assert_eq!(outcome.disputed_layer, fault_layer);
            // 33 leaves → padded to 64 → max 6 rounds
            assert!(
                outcome.rounds <= 6,
                "fault at layer {}: took {} rounds (expected ≤6)",
                fault_layer,
                outcome.rounds
            );
        }
    }
}
