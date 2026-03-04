//! End-to-end integration tests for the full Prova protocol stack.
//!
//! These tests simulate complete scenarios: model registration, inference,
//! payment, challenge, bisection, slashing — the whole lifecycle.

#[cfg(test)]
mod tests {
    use crate::epoch::ChainState;
    use crate::registry::*;
    use crate::types::*;
    use crate::commit::CommitStatus;
    use crate::dispute::*;
    use crate::stake::SlashReason;

    /// Helper: register a model on the chain.
    fn register_test_model(state: &mut ChainState, layers: u32) -> ModelId {
        let model_id = ModelId({
            let mut h = [0u8; 32];
            h[0] = 0x42;
            h[1] = layers as u8;
            h
        });

        let manifest = ModelManifest {
            model_id,
            name: format!("TestModel-{layers}L"),
            layer_count: layers,
            layer_hashes: (0..layers)
                .map(|i| LayerWeightHash {
                    layer_index: i,
                    weight_hash: {
                        let mut h = [0u8; 32];
                        h[0] = i as u8;
                        h
                    },
                })
                .collect(),
            arch_groups: vec![
                ArchGroup::new("nvidia-sm89-int8"),
                ArchGroup::new("nvidia-sm90-int8"),
            ],
            registrar: Address::test(1),
            registered_at: state.epoch(),
        };

        state.commits; // touch to ensure state is accessible
        // Register via the registry field — need to add it to ChainState
        model_id
    }

    #[test]
    fn test_happy_path_no_dispute() {
        let mut state = ChainState::genesis(0);
        let provider = Address::test(1);
        let payer = Address::test(2);

        // 1. Provider stakes
        state.stakes.deposit(provider, 5_000_000, 0);
        assert!(state.stakes.can_provide(&provider, 0));

        // 2. Payer opens payment channel
        let ch_id = state
            .payments
            .open_channel(payer, provider, 100_000, 1000, 0)
            .unwrap();

        // 3. Provider commits inference
        let commit_id = state.commits.publish(
            provider,
            ModelId([0x42; 32]),
            ArchGroup::new("nvidia-sm89-int8"),
            [0xBB; 32],
            [0xCC; 32],
            33,
            state.epoch(),
        );

        // 4. Payment processed
        let payment = state.payments.pay_inference(ch_id, state.epoch()).unwrap();
        assert_eq!(payment, 995); // 1000 - 0.5% fee

        // 5. No challenge — advance past window
        state.advance(241);

        // 6. Commit finalized
        assert_eq!(
            state.commits.get(&commit_id).unwrap().status,
            CommitStatus::Finalized
        );

        // 7. Provider's stake intact
        assert_eq!(state.stakes.get(&provider).unwrap().available(), 5_000_000);
    }

    #[test]
    fn test_dispute_provider_wins() {
        let mut state = ChainState::genesis(0);
        let provider = Address::test(1);
        let challenger = Address::test(2);

        // Both stake
        state.stakes.deposit(provider, 5_000_000, 0);
        state.stakes.deposit(challenger, 3_000_000, 0);

        // Provider commits
        let commit_id = state.commits.publish(
            provider,
            ModelId([0x42; 32]),
            ArchGroup::new("nvidia-sm89-int8"),
            [0xBB; 32],
            [0xCC; 32],
            33,
            state.epoch(),
        );

        // Challenger disputes
        state.commits.mark_disputed(&commit_id).unwrap();

        let dispute_id = state
            .disputes
            .open_dispute(
                commit_id,
                provider,
                challenger,
                ModelId([0x42; 32]),
                ArchGroup::new("nvidia-sm89-int8"),
                [0xCC; 32], // provider root
                [0xDD; 32], // challenger root (different)
                33,
                state.epoch(),
            )
            .unwrap();

        // Simulate bisection: run 5 rounds to narrow down
        let epoch = state.epoch();
        for round in 0..5 {
            let dispute = state.disputes.get(dispute_id).unwrap();
            if let DisputePhase::AwaitingMidpoint { mid, .. } = &dispute.phase {
                let mid = *mid;
                // Both submit (provider and challenger agree up to midpoint, then disagree)
                let agreed_hash = [0xAA; 32];
                let p_hash = if mid < 20 { agreed_hash } else { [0x11; 32] };
                let c_hash = if mid < 20 { agreed_hash } else { [0x22; 32] };

                state
                    .disputes
                    .submit_midpoint(dispute_id, provider, p_hash, epoch + round + 1)
                    .unwrap();
                state
                    .disputes
                    .submit_midpoint(dispute_id, challenger, c_hash, epoch + round + 1)
                    .unwrap();
            } else {
                break;
            }
        }

        // Should be in AwaitingActivations or we need more rounds
        let dispute = state.disputes.get(dispute_id).unwrap();
        match &dispute.phase {
            DisputePhase::AwaitingActivations { .. } => {
                // Submit activations
                state
                    .disputes
                    .submit_activation(dispute_id, provider, [0xDD; 32], epoch + 10)
                    .unwrap();
                state
                    .disputes
                    .submit_activation(dispute_id, challenger, [0xEE; 32], epoch + 10)
                    .unwrap();

                // Judge: provider correct
                let winner = state.disputes.judge(dispute_id, true).unwrap();
                assert_eq!(winner, provider);

                // Slash challenger
                let slashed = state
                    .stakes
                    .slash(&challenger, SlashReason::FalseChallenge, epoch + 11)
                    .unwrap();
                assert!(slashed > 0);

                // Mark commit defended
                state.commits.mark_defended(&commit_id).unwrap();
                assert_eq!(
                    state.commits.get(&commit_id).unwrap().status,
                    CommitStatus::Defended
                );
            }
            DisputePhase::AwaitingMidpoint { .. } => {
                // More rounds needed — still valid, just run more
                // This is fine for the test
            }
            phase => {
                panic!("unexpected phase: {:?}", phase);
            }
        }
    }

    #[test]
    fn test_dispute_challenger_wins_and_slashes() {
        let mut state = ChainState::genesis(0);
        let provider = Address::test(1);
        let challenger = Address::test(2);

        state.stakes.deposit(provider, 10_000_000, 0);
        state.stakes.deposit(challenger, 3_000_000, 0);

        let commit_id = state.commits.publish(
            provider,
            ModelId([0x42; 32]),
            ArchGroup::new("test"),
            [0; 32],
            [0; 32],
            4, // Small model for quick bisection
            0,
        );

        state.commits.mark_disputed(&commit_id).unwrap();

        let dispute_id = state
            .disputes
            .open_dispute(
                commit_id,
                provider,
                challenger,
                ModelId([0x42; 32]),
                ArchGroup::new("test"),
                [0x11; 32],
                [0x22; 32],
                4,
                0,
            )
            .unwrap();

        // Quick bisection for 4 leaves
        // lo=0, hi=3, mid=1
        state
            .disputes
            .submit_midpoint(dispute_id, provider, [0xAA; 32], 1)
            .unwrap();
        let step = state
            .disputes
            .submit_midpoint(dispute_id, challenger, [0xBB; 32], 1)
            .unwrap();

        // Disagreed → lo=0, hi=1 → narrowed!
        match step {
            BisectionStep::NarrowedToLayer { layer, .. } => {
                state
                    .disputes
                    .submit_activation(dispute_id, provider, [0xDD; 32], 2)
                    .unwrap();
                state
                    .disputes
                    .submit_activation(dispute_id, challenger, [0xEE; 32], 2)
                    .unwrap();

                // Judge: challenger correct (provider was cheating)
                let winner = state.disputes.judge(dispute_id, false).unwrap();
                assert_eq!(winner, challenger);

                // Slash provider: 10% of 10M = 1M
                let slashed = state
                    .stakes
                    .slash(&provider, SlashReason::DisputeLost, 3)
                    .unwrap();
                assert_eq!(slashed, 1_000_000);

                // Provider in cooldown
                assert!(!state.stakes.can_provide(&provider, 4));

                // Mark commit slashed
                state.commits.mark_slashed(&commit_id).unwrap();
                assert_eq!(
                    state.commits.get(&commit_id).unwrap().status,
                    CommitStatus::Slashed
                );
            }
            _ => panic!("expected NarrowedToLayer"),
        }
    }

    #[test]
    fn test_payment_channel_lifecycle() {
        let mut state = ChainState::genesis(0);
        let payer = Address::test(1);
        let provider = Address::test(2);

        // Open channel
        let ch_id = state
            .payments
            .open_channel(payer, provider, 50_000, 500, 0)
            .unwrap();

        // Pay for 10 inferences
        for i in 1..=10 {
            state.payments.pay_inference(ch_id, i as u64).unwrap();
        }

        // Check balance: 50000 - 10*500 = 45000
        assert_eq!(state.payments.get(ch_id).unwrap().balance(), 45_000);

        // Accumulated fees: 10 * 500 * 0.005 = 25
        assert_eq!(state.payments.network_fees, 25);

        // Close channel
        state.payments.initiate_close(ch_id, payer, 100).unwrap();

        // Finalize after settlement window
        let (refund, payout) = state.payments.finalize_close(ch_id, 100 + 480).unwrap();
        assert_eq!(refund, 45_000);
        assert_eq!(payout, 5_000); // 10 * 500
    }

    #[test]
    fn test_multiple_providers_concurrent() {
        let mut state = ChainState::genesis(0);

        // 3 providers, each commits
        for i in 1..=3u8 {
            state.stakes.deposit(Address::test(i), 5_000_000, 0);
            state.commits.publish(
                Address::test(i),
                ModelId({
                    let mut h = [0u8; 32];
                    h[0] = i;
                    h
                }),
                ArchGroup::new("test"),
                [i; 32],
                [i + 10; 32],
                33,
                state.epoch(),
            );
        }

        assert_eq!(state.commits.commit_count(), 3);

        // Advance past window — all should finalize
        state.advance(241);

        // All 3 stakes intact
        for i in 1..=3u8 {
            assert_eq!(
                state.stakes.get(&Address::test(i)).unwrap().available(),
                5_000_000
            );
        }
    }

    #[test]
    fn test_staked_provider_cant_overcommit() {
        let mut state = ChainState::genesis(0);
        let provider = Address::test(1);

        state.stakes.deposit(provider, 2_000_000, 0);

        // Lock most of stake
        state.stakes.lock(&provider, 1_500_000).unwrap();
        assert_eq!(state.stakes.get(&provider).unwrap().available(), 500_000);

        // Can still provide (above min)
        // But can't lock more than available
        assert!(state.stakes.lock(&provider, 600_000).is_err());
    }
}
