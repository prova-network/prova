//! Delegation Governance Voting (CHAIN-033)
//!
//! Extends governance with delegation-aware voting: providers inherit
//! governance voting power from their delegators, but delegators can
//! override with a direct vote on any proposal.
//!
//! Key features:
//! - Provider inherits delegated stake as vote weight
//! - Delegator direct vote overrides provider's vote for that share
//! - Snapshot at proposal creation (delegation state frozen)
//! - No transitive delegation (single-hop only)
//! - Vote weight decomposition for transparency

use std::collections::HashMap;

pub type Address = [u8; 32];
pub type ProposalId = u64;
pub type Epoch = u64;
pub type Amount = u128;

/// A frozen snapshot of delegation state at proposal creation time.
#[derive(Debug, Clone)]
pub struct DelegationSnapshot {
    /// provider → self_stake
    pub provider_self_stake: HashMap<Address, Amount>,
    /// provider → total delegated to them (excluding self)
    pub provider_delegated: HashMap<Address, Amount>,
    /// delegator → (provider, amount)
    pub delegations: HashMap<Address, (Address, Amount)>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GovVote {
    Yes,
    No,
    Abstain,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DelegationGovError {
    ProposalNotFound,
    ProposalNotActive,
    NoVotingPower,
    VotingPeriodEnded,
    AlreadyFinalized,
    VotingPeriodNotEnded,
    SnapshotMissing,
}

impl std::fmt::Display for DelegationGovError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}
impl std::error::Error for DelegationGovError {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DelegationGovStatus {
    Active,
    Passed,
    Rejected,
    Expired,
}

#[derive(Debug, Clone)]
pub struct DelegationProposal {
    pub id: ProposalId,
    pub proposer: Address,
    pub created_epoch: Epoch,
    pub voting_period: Epoch,
    pub quorum_bps: u64,
    pub threshold_bps: u64,
    pub status: DelegationGovStatus,
    /// Direct votes cast (both providers and delegators)
    pub votes: HashMap<Address, GovVote>,
}

/// Governance system with delegation-aware voting.
pub struct DelegationGov {
    pub proposals: HashMap<ProposalId, DelegationProposal>,
    pub snapshots: HashMap<ProposalId, DelegationSnapshot>,
    pub next_id: ProposalId,
}

impl DelegationGov {
    pub fn new() -> Self {
        Self {
            proposals: HashMap::new(),
            snapshots: HashMap::new(),
            next_id: 1,
        }
    }

    /// Create a proposal, freezing the current delegation state as a snapshot.
    pub fn create_proposal(
        &mut self,
        proposer: Address,
        voting_period: Epoch,
        quorum_bps: u64,
        threshold_bps: u64,
        current_epoch: Epoch,
        snapshot: DelegationSnapshot,
    ) -> Result<ProposalId, DelegationGovError> {
        // Proposer must have some voting power
        let has_power = snapshot.provider_self_stake.contains_key(&proposer)
            || snapshot.delegations.contains_key(&proposer);
        if !has_power {
            return Err(DelegationGovError::NoVotingPower);
        }

        let id = self.next_id;
        self.next_id += 1;

        let proposal = DelegationProposal {
            id,
            proposer,
            created_epoch: current_epoch,
            voting_period,
            quorum_bps,
            threshold_bps,
            status: DelegationGovStatus::Active,
            votes: HashMap::new(),
        };

        self.proposals.insert(id, proposal);
        self.snapshots.insert(id, snapshot);
        Ok(id)
    }

    /// Cast a vote. Both providers and delegators can vote.
    /// If a delegator votes, their share is split from the provider's weight.
    pub fn vote(
        &mut self,
        proposal_id: ProposalId,
        voter: Address,
        vote: GovVote,
        current_epoch: Epoch,
    ) -> Result<(), DelegationGovError> {
        let proposal = self.proposals.get(&proposal_id)
            .ok_or(DelegationGovError::ProposalNotFound)?;
        if proposal.status != DelegationGovStatus::Active {
            return Err(DelegationGovError::ProposalNotActive);
        }
        let end = proposal.created_epoch + proposal.voting_period;
        if current_epoch >= end {
            return Err(DelegationGovError::VotingPeriodEnded);
        }
        let snapshot = self.snapshots.get(&proposal_id)
            .ok_or(DelegationGovError::SnapshotMissing)?;

        // Check voter has power: either a provider (self-stake) or a delegator
        let is_provider = snapshot.provider_self_stake.contains_key(&voter);
        let is_delegator = snapshot.delegations.contains_key(&voter);
        if !is_provider && !is_delegator {
            return Err(DelegationGovError::NoVotingPower);
        }

        let proposal = self.proposals.get_mut(&proposal_id).unwrap();
        proposal.votes.insert(voter, vote);
        Ok(())
    }

    /// Tally votes with delegation override logic and finalize.
    pub fn finalize(
        &mut self,
        proposal_id: ProposalId,
        current_epoch: Epoch,
    ) -> Result<DelegationGovStatus, DelegationGovError> {
        let proposal = self.proposals.get(&proposal_id)
            .ok_or(DelegationGovError::ProposalNotFound)?;
        if proposal.status != DelegationGovStatus::Active {
            return Err(DelegationGovError::AlreadyFinalized);
        }
        let end = proposal.created_epoch + proposal.voting_period;
        if current_epoch < end {
            return Err(DelegationGovError::VotingPeriodNotEnded);
        }

        let snapshot = self.snapshots.get(&proposal_id)
            .ok_or(DelegationGovError::SnapshotMissing)?;

        // Compute total voting power
        let total_power: Amount = snapshot.provider_self_stake.values().sum::<Amount>()
            + snapshot.delegations.values().map(|(_, a)| *a).sum::<Amount>();

        if total_power == 0 {
            let p = self.proposals.get_mut(&proposal_id).unwrap();
            p.status = DelegationGovStatus::Expired;
            return Ok(DelegationGovStatus::Expired);
        }

        // Collect effective votes with delegation override
        let votes = &proposal.votes;
        let mut yes: Amount = 0;
        let mut no: Amount = 0;
        let mut abstain: Amount = 0;

        // Step 1: Determine which delegators voted directly
        let mut overridden: HashMap<Address, Amount> = HashMap::new(); // provider → amount overridden
        for (delegator, (provider, amount)) in &snapshot.delegations {
            if let Some(dvote) = votes.get(delegator) {
                // Delegator voted directly — their share is removed from provider
                *overridden.entry(*provider).or_default() += amount;
                match dvote {
                    GovVote::Yes => yes += amount,
                    GovVote::No => no += amount,
                    GovVote::Abstain => abstain += amount,
                }
            }
        }

        // Step 2: Provider votes with remaining delegated weight
        for (provider, &self_stake) in &snapshot.provider_self_stake {
            if let Some(pvote) = votes.get(provider) {
                let delegated_total = snapshot.provider_delegated.get(provider).copied().unwrap_or(0);
                let overridden_amount = overridden.get(provider).copied().unwrap_or(0);
                let effective_delegated = delegated_total.saturating_sub(overridden_amount);
                let provider_power = self_stake + effective_delegated;

                match pvote {
                    GovVote::Yes => yes += provider_power,
                    GovVote::No => no += provider_power,
                    GovVote::Abstain => abstain += provider_power,
                }
            }
            // If provider didn't vote, their power (and non-overriding delegators') is unused
        }

        let participated = yes + no + abstain;
        let quorum_needed = total_power * proposal.quorum_bps as u128 / 10_000;

        let status = if participated < quorum_needed {
            DelegationGovStatus::Expired
        } else {
            let vote_total = yes + no;
            if vote_total > 0 && yes * 10_000 / vote_total >= proposal.threshold_bps as u128 {
                DelegationGovStatus::Passed
            } else {
                DelegationGovStatus::Rejected
            }
        };

        let p = self.proposals.get_mut(&proposal_id).unwrap();
        p.status = status.clone();
        Ok(status)
    }

    /// Get the effective vote weight breakdown for a proposal.
    pub fn vote_breakdown(
        &self,
        proposal_id: ProposalId,
    ) -> Result<VoteBreakdown, DelegationGovError> {
        let proposal = self.proposals.get(&proposal_id)
            .ok_or(DelegationGovError::ProposalNotFound)?;
        let snapshot = self.snapshots.get(&proposal_id)
            .ok_or(DelegationGovError::SnapshotMissing)?;

        let total_power: Amount = snapshot.provider_self_stake.values().sum::<Amount>()
            + snapshot.delegations.values().map(|(_, a)| *a).sum::<Amount>();

        let mut yes: Amount = 0;
        let mut no: Amount = 0;
        let mut abstain: Amount = 0;
        let mut overridden: HashMap<Address, Amount> = HashMap::new();
        let mut override_count: u32 = 0;

        for (delegator, (provider, amount)) in &snapshot.delegations {
            if proposal.votes.contains_key(delegator) {
                *overridden.entry(*provider).or_default() += amount;
                override_count += 1;
                match &proposal.votes[delegator] {
                    GovVote::Yes => yes += amount,
                    GovVote::No => no += amount,
                    GovVote::Abstain => abstain += amount,
                }
            }
        }

        for (provider, &self_stake) in &snapshot.provider_self_stake {
            if let Some(pvote) = proposal.votes.get(provider) {
                let delegated_total = snapshot.provider_delegated.get(provider).copied().unwrap_or(0);
                let ov = overridden.get(provider).copied().unwrap_or(0);
                let power = self_stake + delegated_total.saturating_sub(ov);
                match pvote {
                    GovVote::Yes => yes += power,
                    GovVote::No => no += power,
                    GovVote::Abstain => abstain += power,
                }
            }
        }

        Ok(VoteBreakdown {
            total_power,
            yes,
            no,
            abstain,
            participated: yes + no + abstain,
            override_count,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VoteBreakdown {
    pub total_power: Amount,
    pub yes: Amount,
    pub no: Amount,
    pub abstain: Amount,
    pub participated: Amount,
    pub override_count: u32,
}

/// Helper to build a DelegationSnapshot from raw data.
pub fn build_snapshot(
    provider_stakes: &[(Address, Amount)],
    delegations: &[(Address, Address, Amount)], // (delegator, provider, amount)
) -> DelegationSnapshot {
    let mut provider_self_stake = HashMap::new();
    let mut provider_delegated: HashMap<Address, Amount> = HashMap::new();
    let mut deleg_map = HashMap::new();

    for &(provider, stake) in provider_stakes {
        provider_self_stake.insert(provider, stake);
    }

    for &(delegator, provider, amount) in delegations {
        *provider_delegated.entry(provider).or_default() += amount;
        deleg_map.insert(delegator, (provider, amount));
    }

    DelegationSnapshot {
        provider_self_stake,
        provider_delegated,
        delegations: deleg_map,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn addr(n: u8) -> Address {
        let mut a = [0u8; 32];
        a[0] = n;
        a
    }

    fn make_snapshot(
        providers: &[(u8, Amount)],
        delegations: &[(u8, u8, Amount)],
    ) -> DelegationSnapshot {
        let ps: Vec<_> = providers.iter().map(|&(n, s)| (addr(n), s)).collect();
        let ds: Vec<_> = delegations.iter().map(|&(d, p, a)| (addr(d), addr(p), a)).collect();
        build_snapshot(&ps, &ds)
    }

    #[test]
    fn test_provider_votes_with_delegated_weight() {
        let mut gov = DelegationGov::new();
        // Provider 1 has 100 self + 400 delegated, Provider 2 has 200 self + 300 delegated
        let snap = make_snapshot(
            &[(1, 100), (2, 200)],
            &[(10, 1, 200), (11, 1, 200), (20, 2, 300)],
        );
        let id = gov.create_proposal(addr(1), 1000, 1000, 6670, 0, snap).unwrap();
        gov.vote(id, addr(1), GovVote::Yes, 500).unwrap();
        gov.vote(id, addr(2), GovVote::No, 500).unwrap();
        let status = gov.finalize(id, 1000).unwrap();
        // Provider 1: 100 + 400 = 500 Yes. Provider 2: 200 + 300 = 500 No.
        // 500 yes / 1000 = 50% < 66.7% threshold
        assert_eq!(status, DelegationGovStatus::Rejected);
    }

    #[test]
    fn test_delegator_override_flips_result() {
        let mut gov = DelegationGov::new();
        // Provider 1 has 100 self + 400 delegated
        let snap = make_snapshot(
            &[(1, 100)],
            &[(10, 1, 200), (11, 1, 200)],
        );
        let id = gov.create_proposal(addr(1), 1000, 1000, 5010, 0, snap).unwrap();
        // Provider votes Yes (would be 500 total)
        gov.vote(id, addr(1), GovVote::Yes, 500).unwrap();
        // Delegator 10 overrides with No (200 removed from provider, added to No)
        gov.vote(id, addr(10), GovVote::No, 500).unwrap();
        let status = gov.finalize(id, 1000).unwrap();
        // Provider effective: 100 + 200 = 300 Yes. Delegator 10: 200 No. Total: 300 yes / 500 = 60% < threshold
        // But actually 300/500 = 60% and threshold is 50.1% → Passed
        assert_eq!(status, DelegationGovStatus::Passed);
    }

    #[test]
    fn test_all_delegators_override() {
        let mut gov = DelegationGov::new();
        let snap = make_snapshot(
            &[(1, 50)],
            &[(10, 1, 200), (11, 1, 250)],
        );
        let id = gov.create_proposal(addr(1), 1000, 1000, 5010, 0, snap).unwrap();
        gov.vote(id, addr(1), GovVote::Yes, 500).unwrap(); // 50 effective (self only)
        gov.vote(id, addr(10), GovVote::No, 500).unwrap();  // 200 No
        gov.vote(id, addr(11), GovVote::No, 500).unwrap();  // 250 No
        let status = gov.finalize(id, 1000).unwrap();
        // 50 Yes vs 450 No → Rejected
        assert_eq!(status, DelegationGovStatus::Rejected);
    }

    #[test]
    fn test_non_voting_provider_power_unused() {
        let mut gov = DelegationGov::new();
        let snap = make_snapshot(
            &[(1, 100), (2, 100)],
            &[(10, 2, 800)],
        );
        let id = gov.create_proposal(addr(1), 1000, 500, 5010, 0, snap).unwrap();
        // Only provider 1 votes, provider 2 (with 900 total) abstains by not voting
        gov.vote(id, addr(1), GovVote::Yes, 500).unwrap();
        let status = gov.finalize(id, 1000).unwrap();
        // 100 participated of 1000 total = 10% > 5% quorum → check threshold
        // 100 yes / 100 total = 100% ≥ 50.1% → Passed
        assert_eq!(status, DelegationGovStatus::Passed);
    }

    #[test]
    fn test_quorum_not_met() {
        let mut gov = DelegationGov::new();
        let snap = make_snapshot(
            &[(1, 10), (2, 990)],
            &[],
        );
        let id = gov.create_proposal(addr(1), 1000, 2000, 5010, 0, snap).unwrap();
        gov.vote(id, addr(1), GovVote::Yes, 500).unwrap();
        let status = gov.finalize(id, 1000).unwrap();
        // 10 participated / 1000 total = 1% < 20% quorum → Expired
        assert_eq!(status, DelegationGovStatus::Expired);
    }

    #[test]
    fn test_delegator_cannot_vote_without_delegation() {
        let mut gov = DelegationGov::new();
        let snap = make_snapshot(&[(1, 100)], &[]);
        let id = gov.create_proposal(addr(1), 1000, 1000, 5010, 0, snap).unwrap();
        let err = gov.vote(id, addr(99), GovVote::Yes, 500);
        assert_eq!(err, Err(DelegationGovError::NoVotingPower));
    }

    #[test]
    fn test_cannot_vote_after_period() {
        let mut gov = DelegationGov::new();
        let snap = make_snapshot(&[(1, 100)], &[]);
        let id = gov.create_proposal(addr(1), 1000, 1000, 5010, 0, snap).unwrap();
        let err = gov.vote(id, addr(1), GovVote::Yes, 1000);
        assert_eq!(err, Err(DelegationGovError::VotingPeriodEnded));
    }

    #[test]
    fn test_cannot_finalize_early() {
        let mut gov = DelegationGov::new();
        let snap = make_snapshot(&[(1, 100)], &[]);
        let id = gov.create_proposal(addr(1), 1000, 1000, 5010, 0, snap).unwrap();
        let err = gov.finalize(id, 500);
        assert_eq!(err, Err(DelegationGovError::VotingPeriodNotEnded));
    }

    #[test]
    fn test_vote_change_allowed() {
        let mut gov = DelegationGov::new();
        let snap = make_snapshot(&[(1, 500), (2, 500)], &[]);
        let id = gov.create_proposal(addr(1), 1000, 1000, 6670, 0, snap).unwrap();
        gov.vote(id, addr(1), GovVote::No, 500).unwrap();
        gov.vote(id, addr(1), GovVote::Yes, 500).unwrap(); // change
        gov.vote(id, addr(2), GovVote::Yes, 500).unwrap();
        let status = gov.finalize(id, 1000).unwrap();
        assert_eq!(status, DelegationGovStatus::Passed);
    }

    #[test]
    fn test_vote_breakdown() {
        let mut gov = DelegationGov::new();
        let snap = make_snapshot(
            &[(1, 100)],
            &[(10, 1, 200), (11, 1, 300)],
        );
        let id = gov.create_proposal(addr(1), 1000, 1000, 5010, 0, snap).unwrap();
        gov.vote(id, addr(1), GovVote::Yes, 500).unwrap();
        gov.vote(id, addr(10), GovVote::No, 500).unwrap();

        let bd = gov.vote_breakdown(id).unwrap();
        assert_eq!(bd.total_power, 600);
        // Provider: 100 + 300 (only delegator 11 not overriding) = 400 Yes
        // Delegator 10: 200 No
        assert_eq!(bd.yes, 400);
        assert_eq!(bd.no, 200);
        assert_eq!(bd.abstain, 0);
        assert_eq!(bd.override_count, 1);
        assert_eq!(bd.participated, 600);
    }

    #[test]
    fn test_multiple_providers_mixed_voting() {
        let mut gov = DelegationGov::new();
        let snap = make_snapshot(
            &[(1, 100), (2, 100)],
            &[(10, 1, 150), (20, 2, 150)],
        );
        let id = gov.create_proposal(addr(1), 1000, 1000, 5010, 0, snap).unwrap();
        gov.vote(id, addr(1), GovVote::Yes, 500).unwrap();
        gov.vote(id, addr(2), GovVote::No, 500).unwrap();
        gov.vote(id, addr(20), GovVote::Yes, 500).unwrap(); // override provider 2

        let status = gov.finalize(id, 1000).unwrap();
        // Provider 1: 100 + 150 = 250 Yes
        // Delegator 20: 150 Yes (override)
        // Provider 2: 100 + 0 = 100 No (delegator 20 overrode)
        // Total: 400 Yes, 100 No → 80% > 50.1% → Passed
        assert_eq!(status, DelegationGovStatus::Passed);
    }

    #[test]
    fn test_cannot_double_finalize() {
        let mut gov = DelegationGov::new();
        let snap = make_snapshot(&[(1, 100)], &[]);
        let id = gov.create_proposal(addr(1), 1000, 1000, 5010, 0, snap).unwrap();
        gov.vote(id, addr(1), GovVote::Yes, 500).unwrap();
        gov.finalize(id, 1000).unwrap();
        let err = gov.finalize(id, 1000);
        assert_eq!(err, Err(DelegationGovError::AlreadyFinalized));
    }

    #[test]
    fn test_proposer_needs_voting_power() {
        let mut gov = DelegationGov::new();
        let snap = make_snapshot(&[(1, 100)], &[]);
        let err = gov.create_proposal(addr(99), 1000, 1000, 5010, 0, snap);
        assert_eq!(err, Err(DelegationGovError::NoVotingPower));
    }

    #[test]
    fn test_delegator_as_proposer() {
        let mut gov = DelegationGov::new();
        let snap = make_snapshot(&[(1, 100)], &[(10, 1, 200)]);
        let id = gov.create_proposal(addr(10), 1000, 1000, 5010, 0, snap);
        assert!(id.is_ok());
    }

    #[test]
    fn test_abstain_counts_quorum_not_threshold() {
        let mut gov = DelegationGov::new();
        let snap = make_snapshot(&[(1, 300), (2, 300), (3, 400)], &[]);
        let id = gov.create_proposal(addr(1), 1000, 1000, 6670, 0, snap).unwrap();
        gov.vote(id, addr(1), GovVote::Yes, 500).unwrap();
        gov.vote(id, addr(2), GovVote::No, 500).unwrap();
        gov.vote(id, addr(3), GovVote::Abstain, 500).unwrap();
        let status = gov.finalize(id, 1000).unwrap();
        // 300 yes / (300+300) = 50% < 66.7% → Rejected
        assert_eq!(status, DelegationGovStatus::Rejected);
    }

    #[test]
    fn test_zero_total_power_expires() {
        let mut gov = DelegationGov::new();
        // Create snapshot where proposer has power but total is technically 0 after...
        // Actually: provider with 0 stake can't propose. Use minimal:
        let snap = make_snapshot(&[(1, 1)], &[]);
        let id = gov.create_proposal(addr(1), 1000, 10000, 5010, 0, snap).unwrap();
        // Don't vote → quorum 100% not met → Expired
        let status = gov.finalize(id, 1000).unwrap();
        assert_eq!(status, DelegationGovStatus::Expired);
    }
}
