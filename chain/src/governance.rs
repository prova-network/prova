// chain/src/governance.rs — On-chain governance (SPEC-011)
//
// Token-weighted governance: proposals, voting, delegation, execution.

use crate::types::Address;
use std::collections::HashMap;

pub type ProposalId = u64;
pub type Epoch = u64;

#[derive(Debug, Clone, PartialEq)]
pub enum ProposalType {
    ParameterChange,
    TreasurySpend,
    ModelPolicy,
    EmergencyAction,
}

impl ProposalType {
    pub fn quorum_bps(&self) -> u64 {
        match self {
            Self::ParameterChange => 1000,
            Self::TreasurySpend => 1500,
            Self::ModelPolicy => 500,
            Self::EmergencyAction => 3300,
        }
    }

    pub fn threshold_bps(&self) -> u64 {
        match self {
            Self::ParameterChange | Self::TreasurySpend => 6670,
            Self::ModelPolicy => 5010,
            Self::EmergencyAction => 7500,
        }
    }

    pub fn voting_period(&self) -> Epoch {
        match self {
            Self::ParameterChange => 20_160,
            Self::TreasurySpend => 40_320,
            Self::ModelPolicy => 8_640,
            Self::EmergencyAction => 2_880,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum ProposalStatus {
    Active,
    Passed,
    Rejected,
    Expired,
    Executed,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Vote {
    Yes,
    No,
    Abstain,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ProposalPayload {
    ParameterChange {
        key: String,
        value: u128,
    },
    TreasurySpend {
        recipient: Address,
        amount: u128,
        memo: String,
    },
    ModelPolicy {
        key: String,
        value: u128,
    },
    EmergencyPause {
        module: String,
    },
}

#[derive(Debug, Clone)]
pub struct Proposal {
    pub id: ProposalId,
    pub proposer: Address,
    pub proposal_type: ProposalType,
    pub payload: ProposalPayload,
    pub description: String,
    pub created_epoch: Epoch,
    pub deposit: u128,
    pub votes: HashMap<Address, Vote>,
    pub status: ProposalStatus,
}

pub const PROPOSAL_DEPOSIT: u128 = 1_000_000_000_000_000_000_000;
pub const TIMELOCK_EPOCHS: Epoch = 2_880;
pub const MIN_PARTICIPATION_BPS: u64 = 500;

pub struct GovernanceState {
    pub proposals: HashMap<ProposalId, Proposal>,
    pub next_id: ProposalId,
    pub delegations: HashMap<Address, Address>,
    pub snapshots: HashMap<ProposalId, HashMap<Address, u128>>,
    pub treasury: u128,
    pub parameters: HashMap<String, u128>,
}

impl GovernanceState {
    pub fn new() -> Self {
        let mut parameters = HashMap::new();
        parameters.insert("challenge_window".into(), 100);
        parameters.insert("min_provider_stake".into(), 1000);
        parameters.insert("block_reward".into(), 10);
        parameters.insert("slash_fraction".into(), 10);
        parameters.insert("proof_reward".into(), 5);
        parameters.insert("payment_network_fee_bps".into(), 50);

        Self {
            proposals: HashMap::new(),
            next_id: 1,
            delegations: HashMap::new(),
            snapshots: HashMap::new(),
            treasury: 0,
            parameters,
        }
    }

    pub fn create_proposal(
        &mut self,
        proposer: Address,
        proposal_type: ProposalType,
        payload: ProposalPayload,
        description: String,
        current_epoch: Epoch,
        stakes: &HashMap<Address, u128>,
    ) -> Result<ProposalId, &'static str> {
        let id = self.next_id;
        self.next_id += 1;

        let mut snapshot: HashMap<Address, u128> = HashMap::new();
        for (addr, &stake) in stakes {
            if stake == 0 {
                continue;
            }
            let voter = self.delegations.get(addr).copied().unwrap_or(*addr);
            *snapshot.entry(voter).or_default() += stake;
        }

        let proposal = Proposal {
            id,
            proposer,
            proposal_type,
            payload,
            description,
            created_epoch: current_epoch,
            deposit: PROPOSAL_DEPOSIT,
            votes: HashMap::new(),
            status: ProposalStatus::Active,
        };

        self.proposals.insert(id, proposal);
        self.snapshots.insert(id, snapshot);
        Ok(id)
    }

    pub fn vote(
        &mut self,
        proposal_id: ProposalId,
        voter: Address,
        vote: Vote,
    ) -> Result<(), &'static str> {
        let proposal = self
            .proposals
            .get_mut(&proposal_id)
            .ok_or("proposal not found")?;
        if proposal.status != ProposalStatus::Active {
            return Err("proposal not active");
        }
        let snapshot = self.snapshots.get(&proposal_id).ok_or("no snapshot")?;
        if !snapshot.contains_key(&voter) {
            return Err("no voting power");
        }
        proposal.votes.insert(voter, vote);
        Ok(())
    }

    pub fn delegate(&mut self, delegator: Address, delegatee: Address) -> Result<(), &'static str> {
        if delegator == delegatee {
            return Err("cannot self-delegate");
        }
        if self.delegations.contains_key(&delegatee) {
            return Err("delegatee has own delegation");
        }
        self.delegations.insert(delegator, delegatee);
        Ok(())
    }

    pub fn undelegate(&mut self, delegator: Address) {
        self.delegations.remove(&delegator);
    }

    pub fn finalize(
        &mut self,
        proposal_id: ProposalId,
        current_epoch: Epoch,
    ) -> Result<ProposalStatus, &'static str> {
        let proposal = self
            .proposals
            .get(&proposal_id)
            .ok_or("proposal not found")?;
        if proposal.status != ProposalStatus::Active {
            return Err("proposal not active");
        }
        let end_epoch = proposal.created_epoch + proposal.proposal_type.voting_period();
        if current_epoch < end_epoch {
            return Err("voting period not ended");
        }

        let snapshot = self.snapshots.get(&proposal_id).ok_or("no snapshot")?;
        let total_power: u128 = snapshot.values().sum();
        if total_power == 0 {
            let p = self.proposals.get_mut(&proposal_id).unwrap();
            p.status = ProposalStatus::Expired;
            return Ok(ProposalStatus::Expired);
        }

        let (mut yes_power, mut no_power, mut abstain_power) = (0u128, 0u128, 0u128);
        for (addr, vote) in &proposal.votes {
            let power = snapshot.get(addr).copied().unwrap_or(0);
            match vote {
                Vote::Yes => yes_power += power,
                Vote::No => no_power += power,
                Vote::Abstain => abstain_power += power,
            }
        }

        let participated = yes_power + no_power + abstain_power;
        let quorum_needed = total_power * proposal.proposal_type.quorum_bps() as u128 / 10_000;

        let status = if participated < quorum_needed {
            let min_participation = total_power * MIN_PARTICIPATION_BPS as u128 / 10_000;
            if participated < min_participation {
                self.treasury += PROPOSAL_DEPOSIT;
            }
            ProposalStatus::Expired
        } else {
            let vote_total = yes_power + no_power;
            let threshold = proposal.proposal_type.threshold_bps() as u128;
            if vote_total > 0 && yes_power * 10_000 / vote_total >= threshold {
                ProposalStatus::Passed
            } else {
                ProposalStatus::Rejected
            }
        };

        let p = self.proposals.get_mut(&proposal_id).unwrap();
        p.status = status.clone();
        Ok(status)
    }

    pub fn execute(
        &mut self,
        proposal_id: ProposalId,
        current_epoch: Epoch,
    ) -> Result<(), &'static str> {
        let proposal = self
            .proposals
            .get(&proposal_id)
            .ok_or("proposal not found")?;
        if proposal.status != ProposalStatus::Passed {
            return Err("proposal not passed");
        }
        let end_epoch = proposal.created_epoch + proposal.proposal_type.voting_period();
        if current_epoch < end_epoch + TIMELOCK_EPOCHS {
            return Err("timelock not expired");
        }

        match &proposal.payload {
            ProposalPayload::ParameterChange { key, value } => {
                self.parameters.insert(key.clone(), *value);
            }
            ProposalPayload::TreasurySpend { amount, .. } => {
                if self.treasury < *amount {
                    return Err("insufficient treasury");
                }
                let max_spend = self.treasury / 10;
                if *amount > max_spend {
                    return Err("exceeds 10% treasury cap");
                }
                self.treasury -= amount;
            }
            ProposalPayload::ModelPolicy { key, value } => {
                self.parameters.insert(format!("model_{}", key), *value);
            }
            ProposalPayload::EmergencyPause { .. } => {}
        }

        let p = self.proposals.get_mut(&proposal_id).unwrap();
        p.status = ProposalStatus::Executed;
        Ok(())
    }

    pub fn get_parameter(&self, key: &str) -> Option<u128> {
        self.parameters.get(key).copied()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn a(n: u8) -> Address {
        Address::test(n)
    }

    fn stakes_map(pairs: &[(u8, u128)]) -> HashMap<Address, u128> {
        pairs.iter().map(|&(n, s)| (a(n), s)).collect()
    }

    #[test]
    fn test_create_proposal() {
        let mut gov = GovernanceState::new();
        let stakes = stakes_map(&[(1, 500), (2, 300), (3, 200)]);
        let id = gov
            .create_proposal(
                a(1),
                ProposalType::ParameterChange,
                ProposalPayload::ParameterChange {
                    key: "block_reward".into(),
                    value: 20,
                },
                "Increase reward".into(),
                100,
                &stakes,
            )
            .unwrap();
        assert_eq!(id, 1);
        assert_eq!(gov.proposals[&1].status, ProposalStatus::Active);
    }

    #[test]
    fn test_vote_and_pass() {
        let mut gov = GovernanceState::new();
        let stakes = stakes_map(&[(1, 500), (2, 300), (3, 200)]);
        let id = gov
            .create_proposal(
                a(1),
                ProposalType::ParameterChange,
                ProposalPayload::ParameterChange {
                    key: "block_reward".into(),
                    value: 20,
                },
                "Increase".into(),
                100,
                &stakes,
            )
            .unwrap();
        gov.vote(id, a(1), Vote::Yes).unwrap();
        gov.vote(id, a(2), Vote::Yes).unwrap();
        let status = gov.finalize(id, 100 + 20_160).unwrap();
        assert_eq!(status, ProposalStatus::Passed);
    }

    #[test]
    fn test_vote_rejected() {
        let mut gov = GovernanceState::new();
        let stakes = stakes_map(&[(1, 500), (2, 300), (3, 200)]);
        let id = gov
            .create_proposal(
                a(1),
                ProposalType::ParameterChange,
                ProposalPayload::ParameterChange {
                    key: "block_reward".into(),
                    value: 20,
                },
                "Increase".into(),
                100,
                &stakes,
            )
            .unwrap();
        gov.vote(id, a(1), Vote::No).unwrap();
        gov.vote(id, a(2), Vote::Yes).unwrap();
        gov.vote(id, a(3), Vote::No).unwrap();
        let status = gov.finalize(id, 100 + 20_160).unwrap();
        assert_eq!(status, ProposalStatus::Rejected);
    }

    #[test]
    fn test_quorum_not_met_expired() {
        let mut gov = GovernanceState::new();
        let stakes = stakes_map(&[(1, 50), (2, 50), (3, 900)]);
        let id = gov
            .create_proposal(
                a(1),
                ProposalType::ParameterChange,
                ProposalPayload::ParameterChange {
                    key: "block_reward".into(),
                    value: 20,
                },
                "Increase".into(),
                100,
                &stakes,
            )
            .unwrap();
        gov.vote(id, a(1), Vote::Yes).unwrap();
        let status = gov.finalize(id, 100 + 20_160).unwrap();
        assert_eq!(status, ProposalStatus::Expired);
    }

    #[test]
    fn test_deposit_slashed_low_participation() {
        let mut gov = GovernanceState::new();
        let stakes = stakes_map(&[(1, 10), (2, 10), (3, 980)]);
        let id = gov
            .create_proposal(
                a(1),
                ProposalType::ParameterChange,
                ProposalPayload::ParameterChange {
                    key: "x".into(),
                    value: 1,
                },
                "test".into(),
                0,
                &stakes,
            )
            .unwrap();
        let status = gov.finalize(id, 20_160).unwrap();
        assert_eq!(status, ProposalStatus::Expired);
        assert_eq!(gov.treasury, PROPOSAL_DEPOSIT);
    }

    #[test]
    fn test_delegation() {
        let mut gov = GovernanceState::new();
        gov.delegate(a(1), a(2)).unwrap();
        let stakes = stakes_map(&[(1, 400), (2, 300), (3, 300)]);
        let id = gov
            .create_proposal(
                a(2),
                ProposalType::ModelPolicy,
                ProposalPayload::ModelPolicy {
                    key: "min_stake".into(),
                    value: 500,
                },
                "Raise min".into(),
                0,
                &stakes,
            )
            .unwrap();
        gov.vote(id, a(2), Vote::Yes).unwrap();
        let status = gov.finalize(id, 8_640).unwrap();
        assert_eq!(status, ProposalStatus::Passed);
    }

    #[test]
    fn test_cannot_self_delegate() {
        let mut gov = GovernanceState::new();
        assert!(gov.delegate(a(1), a(1)).is_err());
    }

    #[test]
    fn test_no_redelegation_chain() {
        let mut gov = GovernanceState::new();
        gov.delegate(a(2), a(3)).unwrap();
        assert!(gov.delegate(a(1), a(2)).is_err());
    }

    #[test]
    fn test_execute_parameter_change() {
        let mut gov = GovernanceState::new();
        let stakes = stakes_map(&[(1, 1000)]);
        let id = gov
            .create_proposal(
                a(1),
                ProposalType::ParameterChange,
                ProposalPayload::ParameterChange {
                    key: "block_reward".into(),
                    value: 20,
                },
                "Double reward".into(),
                100,
                &stakes,
            )
            .unwrap();
        gov.vote(id, a(1), Vote::Yes).unwrap();
        gov.finalize(id, 100 + 20_160).unwrap();
        assert!(gov.execute(id, 100 + 20_160).is_err());
        gov.execute(id, 100 + 20_160 + TIMELOCK_EPOCHS).unwrap();
        assert_eq!(gov.get_parameter("block_reward"), Some(20));
        assert_eq!(gov.proposals[&id].status, ProposalStatus::Executed);
    }

    #[test]
    fn test_treasury_spend() {
        let mut gov = GovernanceState::new();
        gov.treasury = 100_000;
        let stakes = stakes_map(&[(1, 600), (2, 400)]);
        let id = gov
            .create_proposal(
                a(1),
                ProposalType::TreasurySpend,
                ProposalPayload::TreasurySpend {
                    recipient: a(5),
                    amount: 5_000,
                    memo: "grant".into(),
                },
                "Fund dev".into(),
                0,
                &stakes,
            )
            .unwrap();
        gov.vote(id, a(1), Vote::Yes).unwrap();
        gov.vote(id, a(2), Vote::Yes).unwrap();
        gov.finalize(id, 40_320).unwrap();
        gov.execute(id, 40_320 + TIMELOCK_EPOCHS).unwrap();
        assert_eq!(gov.treasury, 95_000);
    }

    #[test]
    fn test_treasury_spend_exceeds_cap() {
        let mut gov = GovernanceState::new();
        gov.treasury = 100_000;
        let stakes = stakes_map(&[(1, 1000)]);
        let id = gov
            .create_proposal(
                a(1),
                ProposalType::TreasurySpend,
                ProposalPayload::TreasurySpend {
                    recipient: a(5),
                    amount: 20_000,
                    memo: "too much".into(),
                },
                "Big spend".into(),
                0,
                &stakes,
            )
            .unwrap();
        gov.vote(id, a(1), Vote::Yes).unwrap();
        gov.finalize(id, 40_320).unwrap();
        assert!(gov.execute(id, 40_320 + TIMELOCK_EPOCHS).is_err());
    }

    #[test]
    fn test_emergency_high_quorum() {
        let mut gov = GovernanceState::new();
        let stakes = stakes_map(&[(1, 200), (2, 100), (3, 700)]);
        let id = gov
            .create_proposal(
                a(1),
                ProposalType::EmergencyAction,
                ProposalPayload::EmergencyPause {
                    module: "disputes".into(),
                },
                "Pause disputes".into(),
                0,
                &stakes,
            )
            .unwrap();
        gov.vote(id, a(1), Vote::Yes).unwrap();
        gov.vote(id, a(2), Vote::Yes).unwrap();
        let status = gov.finalize(id, 2_880).unwrap();
        assert_eq!(status, ProposalStatus::Expired);
    }

    #[test]
    fn test_abstain_counts_quorum_not_threshold() {
        let mut gov = GovernanceState::new();
        let stakes = stakes_map(&[(1, 400), (2, 300), (3, 300)]);
        let id = gov
            .create_proposal(
                a(1),
                ProposalType::ParameterChange,
                ProposalPayload::ParameterChange {
                    key: "x".into(),
                    value: 1,
                },
                "test".into(),
                0,
                &stakes,
            )
            .unwrap();
        gov.vote(id, a(1), Vote::Yes).unwrap();
        gov.vote(id, a(2), Vote::No).unwrap();
        gov.vote(id, a(3), Vote::Abstain).unwrap();
        let status = gov.finalize(id, 20_160).unwrap();
        assert_eq!(status, ProposalStatus::Rejected);
    }

    #[test]
    fn test_vote_change() {
        let mut gov = GovernanceState::new();
        let stakes = stakes_map(&[(1, 500), (2, 500)]);
        let id = gov
            .create_proposal(
                a(1),
                ProposalType::ModelPolicy,
                ProposalPayload::ModelPolicy {
                    key: "x".into(),
                    value: 1,
                },
                "test".into(),
                0,
                &stakes,
            )
            .unwrap();
        gov.vote(id, a(1), Vote::No).unwrap();
        gov.vote(id, a(1), Vote::Yes).unwrap();
        gov.vote(id, a(2), Vote::Yes).unwrap();
        let status = gov.finalize(id, 8_640).unwrap();
        assert_eq!(status, ProposalStatus::Passed);
    }

    #[test]
    fn test_cannot_vote_without_power() {
        let mut gov = GovernanceState::new();
        let stakes = stakes_map(&[(1, 1000)]);
        let id = gov
            .create_proposal(
                a(1),
                ProposalType::ParameterChange,
                ProposalPayload::ParameterChange {
                    key: "x".into(),
                    value: 1,
                },
                "test".into(),
                0,
                &stakes,
            )
            .unwrap();
        assert!(gov.vote(id, a(9), Vote::Yes).is_err());
    }

    #[test]
    fn test_cannot_finalize_early() {
        let mut gov = GovernanceState::new();
        let stakes = stakes_map(&[(1, 1000)]);
        let id = gov
            .create_proposal(
                a(1),
                ProposalType::ParameterChange,
                ProposalPayload::ParameterChange {
                    key: "x".into(),
                    value: 1,
                },
                "test".into(),
                100,
                &stakes,
            )
            .unwrap();
        assert!(gov.finalize(id, 100 + 10_000).is_err());
    }

    #[test]
    fn test_undelegate() {
        let mut gov = GovernanceState::new();
        gov.delegate(a(1), a(2)).unwrap();
        gov.undelegate(a(1));
        let stakes = stakes_map(&[(1, 500), (2, 500)]);
        let id = gov
            .create_proposal(
                a(1),
                ProposalType::ModelPolicy,
                ProposalPayload::ModelPolicy {
                    key: "x".into(),
                    value: 1,
                },
                "test".into(),
                0,
                &stakes,
            )
            .unwrap();
        let snapshot = &gov.snapshots[&id];
        assert_eq!(snapshot[&a(1)], 500);
        assert_eq!(snapshot[&a(2)], 500);
    }
}
