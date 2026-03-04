// chain/src/integration_test.rs — INT-002: Full-chain integration test
//
// Genesis → staking → blocks → transactions → rewards → governance

#[cfg(test)]
mod tests {
    use crate::types::Address;
    use crate::state::StateTrie;
    use crate::rewards::RewardLedger;
    use crate::executor::{Executor, Transaction as ExTx, TxKind};
    use crate::stake::{StakeLedger, SlashReason};
    use crate::governance::{GovernanceState, ProposalType, ProposalPayload, Vote, ProposalStatus};
    use crate::epoch::ChainState;
    use crate::mempool::{self, Mempool, MempoolConfig};
    use std::collections::HashMap;

    fn a(n: u8) -> Address { Address::test(n) }

    #[test]
    fn test_full_chain_lifecycle() {
        // Phase 1: Genesis — set up initial state
        let mut state = StateTrie::new();
        state.set_balance(a(1), 100_000); // provider
        state.set_balance(a(2), 50_000);  // challenger
        state.set_balance(a(3), 100_000); // user
        let treasury = a(99);
        state.set_balance(treasury, 0);
        assert_eq!(state.get(&a(1)).balance, 100_000);

        // Phase 2: Staking
        let mut stake_ledger = StakeLedger::new(1_000, 500);
        stake_ledger.deposit(a(1), 20_000, 0);
        stake_ledger.deposit(a(2), 5_000, 0);
        assert!(stake_ledger.can_provide(&a(1), 0));
        assert!(stake_ledger.can_challenge(&a(2), 0));

        // Phase 3: Transaction execution
        let mut executor = Executor::new(treasury);
        let tx = ExTx::new(a(3), 0, TxKind::Transfer { to: a(1), amount: 1_000 })
            .with_gas(21_000, 1);
        let receipt = executor.execute(&mut state, &tx, 0);
        assert!(receipt.success);
        assert_eq!(state.get(&a(1)).balance, 101_000);
        assert!(state.get(&a(3)).balance < 100_000);

        // Phase 4: Rewards
        let mut rewards = RewardLedger::new();
        let result = rewards.distribute_block_reward(a(1), 0);
        assert!(result.to_producer > 0);
        let pending = rewards.pending_for(&a(1));
        assert!(pending > 0);
        let claimed = rewards.claim(&a(1));
        assert_eq!(claimed, pending);

        // Phase 5: Governance — change a parameter
        let mut gov = GovernanceState::new();
        let stakes: HashMap<Address, u128> = vec![
            (a(1), 20_000), (a(2), 5_000),
        ].into_iter().collect();

        let pid = gov.create_proposal(
            a(1), ProposalType::ParameterChange,
            ProposalPayload::ParameterChange { key: "block_reward".into(), value: 20 },
            "Double block reward".into(), 0, &stakes,
        ).unwrap();

        gov.vote(pid, a(1), Vote::Yes).unwrap();
        gov.vote(pid, a(2), Vote::Yes).unwrap();
        let status = gov.finalize(pid, 20_160).unwrap();
        assert_eq!(status, ProposalStatus::Passed);

        gov.execute(pid, 20_160 + 2_880).unwrap();
        assert_eq!(gov.get_parameter("block_reward"), Some(20));

        // Phase 6: Epoch progression
        let mut chain = ChainState::genesis(0);
        let summaries = chain.advance(100);
        assert_eq!(summaries.len(), 100);
        assert_eq!(chain.epoch(), 100);
    }

    #[test]
    fn test_stake_slash_and_reward_cycle() {
        let mut stake_ledger = StakeLedger::new(1_000, 500);
        stake_ledger.deposit(a(1), 10_000, 0);

        // Slash for lost dispute (10%)
        let slashed = stake_ledger.slash(&a(1), SlashReason::DisputeLost, 0).unwrap();
        assert_eq!(slashed, 1_000);

        let entry = stake_ledger.get(&a(1)).unwrap();
        assert_eq!(entry.deposited, 10_000);
        assert_eq!(entry.slashed, 1_000);
        assert_eq!(entry.available(), 9_000);

        // Provider still earns rewards on remaining stake
        let mut rewards = RewardLedger::new();
        let result = rewards.distribute_block_reward(a(1), 0);
        assert!(result.to_producer > 0);
    }

    #[test]
    fn test_multi_transfer_state_consistency() {
        let mut state = StateTrie::new();
        state.set_balance(a(1), 1_000_000);
        state.set_balance(a(2), 0);
        state.set_balance(a(99), 0);

        let mut executor = Executor::new(a(99));

        for i in 0..10u64 {
            let tx = ExTx::new(a(1), i, TxKind::Transfer { to: a(2), amount: 1_000 })
                .with_gas(21_000, 1);
            let receipt = executor.execute(&mut state, &tx, i as usize);
            assert!(receipt.success);
        }

        assert_eq!(state.get(&a(2)).balance, 10_000);
        let bal1 = state.get(&a(1)).balance;
        assert!(bal1 < 990_000); // transfers + gas
        assert!(bal1 > 700_000); // sanity (10 transfers + 10 gas fees)
    }

    #[test]
    fn test_mempool_to_execution_flow() {
        let config = MempoolConfig {
            max_txs: 100,
            max_per_sender: 16,
            ..MempoolConfig::default()
        };
        let mut pool = Mempool::new(config);

        let tx = mempool::Transaction {
            hash: [1u8; 32],
            sender: a(1),
            nonce: 0,
            gas_price: 10,
            gas_limit: 21_000,
            kind: mempool::TxKind::Transfer,
            submitted_at: 0,
            size: 100,
        };
        let result = pool.add(tx);
        assert_eq!(result, mempool::AddResult::Added);

        let top = pool.top(10);
        assert_eq!(top.len(), 1);
        assert_eq!(top[0].sender, a(1));
    }

    #[test]
    fn test_governance_treasury_lifecycle() {
        let mut gov = GovernanceState::new();
        gov.treasury = 1_000_000;

        let stakes: HashMap<Address, u128> = vec![
            (a(1), 8_000), (a(2), 2_000),
        ].into_iter().collect();

        let pid = gov.create_proposal(
            a(1), ProposalType::TreasurySpend,
            ProposalPayload::TreasurySpend {
                recipient: a(5), amount: 50_000, memo: "dev grant".into(),
            },
            "Fund development".into(), 0, &stakes,
        ).unwrap();

        gov.vote(pid, a(1), Vote::Yes).unwrap();
        gov.vote(pid, a(2), Vote::Yes).unwrap();
        gov.finalize(pid, 40_320).unwrap();
        gov.execute(pid, 40_320 + 2_880).unwrap();
        assert_eq!(gov.treasury, 950_000);
    }
}
