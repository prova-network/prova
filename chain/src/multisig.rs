// chain/src/multisig.rs — Multi-signature wallet & account abstraction
//
// Supports M-of-N multi-sig wallets for governance, treasury, and operator safety.
// Features: proposal lifecycle, signature collection, expiry, nonce replay protection.

use std::collections::{HashMap, HashSet};

/// Unique identifier for a multi-sig wallet.
pub type WalletId = [u8; 32];
/// Unique identifier for a proposal within a wallet.
pub type ProposalId = u64;
/// Abstract address (Ed25519 pubkey hash).
pub type Address = [u8; 32];

/// A pending transaction proposal.
#[derive(Clone, Debug)]
pub struct Proposal {
    pub id: ProposalId,
    pub proposer: Address,
    pub target: Address,
    pub value: u64,
    pub calldata: Vec<u8>,
    pub approvals: HashSet<Address>,
    pub rejections: HashSet<Address>,
    pub created_at: u64, // epoch
    pub executed: bool,
    pub cancelled: bool,
}

/// Multi-sig wallet configuration + state.
#[derive(Clone, Debug)]
pub struct MultisigWallet {
    pub id: WalletId,
    pub owners: Vec<Address>,
    pub threshold: u32, // M of N
    pub proposals: HashMap<ProposalId, Proposal>,
    pub next_proposal_id: ProposalId,
    pub nonce: u64,
    pub daily_limit: Option<u64>,     // optional daily spend limit
    pub daily_spent: u64,
    pub daily_reset_epoch: u64,
    pub proposal_ttl: u64,            // epochs before proposal expires
}

/// Errors from multi-sig operations.
#[derive(Debug, PartialEq)]
pub enum MultisigError {
    NotOwner,
    InvalidThreshold,
    DuplicateOwner,
    ProposalNotFound,
    AlreadyApproved,
    AlreadyRejected,
    AlreadyExecuted,
    ProposalExpired,
    ProposalCancelled,
    InsufficientApprovals,
    OnlyProposerCanCancel,
    DailyLimitExceeded,
    TooFewOwners,
    OwnerAlreadyExists,
    OwnerNotFound,
    ThresholdWouldExceedOwners,
}

/// Global registry of multi-sig wallets.
#[derive(Default)]
pub struct MultisigRegistry {
    pub wallets: HashMap<WalletId, MultisigWallet>,
}

impl MultisigWallet {
    pub fn new(
        id: WalletId,
        owners: Vec<Address>,
        threshold: u32,
        proposal_ttl: u64,
        daily_limit: Option<u64>,
    ) -> Result<Self, MultisigError> {
        if owners.len() < 2 {
            return Err(MultisigError::TooFewOwners);
        }
        if threshold == 0 || threshold as usize > owners.len() {
            return Err(MultisigError::InvalidThreshold);
        }
        let mut seen = HashSet::new();
        for o in &owners {
            if !seen.insert(*o) {
                return Err(MultisigError::DuplicateOwner);
            }
        }
        Ok(Self {
            id,
            owners,
            threshold,
            proposals: HashMap::new(),
            next_proposal_id: 1,
            nonce: 0,
            daily_limit,
            daily_spent: 0,
            daily_reset_epoch: 0,
            proposal_ttl,
        })
    }

    fn is_owner(&self, addr: &Address) -> bool {
        self.owners.contains(addr)
    }

    fn reset_daily_if_needed(&mut self, epoch: u64) {
        // Reset daily spending every 2880 epochs (~24h at 30s blocks)
        let day_epochs = 2880;
        if epoch >= self.daily_reset_epoch + day_epochs {
            self.daily_spent = 0;
            self.daily_reset_epoch = epoch - (epoch % day_epochs);
        }
    }

    /// Submit a new proposal. Returns proposal ID.
    pub fn propose(
        &mut self,
        proposer: Address,
        target: Address,
        value: u64,
        calldata: Vec<u8>,
        epoch: u64,
    ) -> Result<ProposalId, MultisigError> {
        if !self.is_owner(&proposer) {
            return Err(MultisigError::NotOwner);
        }
        let id = self.next_proposal_id;
        self.next_proposal_id += 1;
        let mut approvals = HashSet::new();
        approvals.insert(proposer); // proposer auto-approves
        self.proposals.insert(id, Proposal {
            id,
            proposer,
            target,
            value,
            calldata,
            approvals,
            rejections: HashSet::new(),
            created_at: epoch,
            executed: false,
            cancelled: false,
        });
        Ok(id)
    }

    /// Approve a pending proposal.
    pub fn approve(
        &mut self,
        signer: Address,
        proposal_id: ProposalId,
        epoch: u64,
    ) -> Result<(), MultisigError> {
        if !self.is_owner(&signer) {
            return Err(MultisigError::NotOwner);
        }
        let p = self.proposals.get_mut(&proposal_id)
            .ok_or(MultisigError::ProposalNotFound)?;
        if p.executed { return Err(MultisigError::AlreadyExecuted); }
        if p.cancelled { return Err(MultisigError::ProposalCancelled); }
        if epoch > p.created_at + self.proposal_ttl {
            return Err(MultisigError::ProposalExpired);
        }
        if p.approvals.contains(&signer) {
            return Err(MultisigError::AlreadyApproved);
        }
        if p.rejections.contains(&signer) {
            return Err(MultisigError::AlreadyRejected);
        }
        p.approvals.insert(signer);
        Ok(())
    }

    /// Reject a pending proposal.
    pub fn reject(
        &mut self,
        signer: Address,
        proposal_id: ProposalId,
        epoch: u64,
    ) -> Result<(), MultisigError> {
        if !self.is_owner(&signer) {
            return Err(MultisigError::NotOwner);
        }
        let p = self.proposals.get_mut(&proposal_id)
            .ok_or(MultisigError::ProposalNotFound)?;
        if p.executed { return Err(MultisigError::AlreadyExecuted); }
        if p.cancelled { return Err(MultisigError::ProposalCancelled); }
        if epoch > p.created_at + self.proposal_ttl {
            return Err(MultisigError::ProposalExpired);
        }
        if p.rejections.contains(&signer) {
            return Err(MultisigError::AlreadyRejected);
        }
        if p.approvals.contains(&signer) {
            return Err(MultisigError::AlreadyApproved);
        }
        p.rejections.insert(signer);
        Ok(())
    }

    /// Cancel a proposal (only proposer can cancel).
    pub fn cancel(
        &mut self,
        signer: Address,
        proposal_id: ProposalId,
    ) -> Result<(), MultisigError> {
        let p = self.proposals.get_mut(&proposal_id)
            .ok_or(MultisigError::ProposalNotFound)?;
        if p.executed { return Err(MultisigError::AlreadyExecuted); }
        if p.proposer != signer {
            return Err(MultisigError::OnlyProposerCanCancel);
        }
        p.cancelled = true;
        Ok(())
    }

    /// Execute a proposal if threshold is met. Returns (target, value, calldata).
    pub fn execute(
        &mut self,
        proposal_id: ProposalId,
        epoch: u64,
    ) -> Result<(Address, u64, Vec<u8>), MultisigError> {
        self.reset_daily_if_needed(epoch);
        let p = self.proposals.get(&proposal_id)
            .ok_or(MultisigError::ProposalNotFound)?;
        if p.executed { return Err(MultisigError::AlreadyExecuted); }
        if p.cancelled { return Err(MultisigError::ProposalCancelled); }
        if epoch > p.created_at + self.proposal_ttl {
            return Err(MultisigError::ProposalExpired);
        }
        if (p.approvals.len() as u32) < self.threshold {
            return Err(MultisigError::InsufficientApprovals);
        }
        if let Some(limit) = self.daily_limit {
            if self.daily_spent + p.value > limit {
                return Err(MultisigError::DailyLimitExceeded);
            }
        }
        let target = p.target;
        let value = p.value;
        let calldata = p.calldata.clone();
        let p = self.proposals.get_mut(&proposal_id).unwrap();
        p.executed = true;
        self.nonce += 1;
        self.daily_spent += value;
        Ok((target, value, calldata))
    }

    /// Add a new owner (requires an executed proposal — caller must verify).
    pub fn add_owner(&mut self, new_owner: Address) -> Result<(), MultisigError> {
        if self.owners.contains(&new_owner) {
            return Err(MultisigError::OwnerAlreadyExists);
        }
        self.owners.push(new_owner);
        Ok(())
    }

    /// Remove an owner (threshold must still be satisfiable).
    pub fn remove_owner(&mut self, owner: Address) -> Result<(), MultisigError> {
        let idx = self.owners.iter().position(|o| *o == owner)
            .ok_or(MultisigError::OwnerNotFound)?;
        if self.owners.len() - 1 < self.threshold as usize {
            return Err(MultisigError::ThresholdWouldExceedOwners);
        }
        if self.owners.len() - 1 < 2 {
            return Err(MultisigError::TooFewOwners);
        }
        self.owners.remove(idx);
        Ok(())
    }

    /// Change threshold (must be valid for current owner count).
    pub fn change_threshold(&mut self, new_threshold: u32) -> Result<(), MultisigError> {
        if new_threshold == 0 || new_threshold as usize > self.owners.len() {
            return Err(MultisigError::InvalidThreshold);
        }
        self.threshold = new_threshold;
        Ok(())
    }
}

impl MultisigRegistry {
    pub fn create_wallet(
        &mut self,
        id: WalletId,
        owners: Vec<Address>,
        threshold: u32,
        proposal_ttl: u64,
        daily_limit: Option<u64>,
    ) -> Result<(), MultisigError> {
        let wallet = MultisigWallet::new(id, owners, threshold, proposal_ttl, daily_limit)?;
        self.wallets.insert(id, wallet);
        Ok(())
    }

    pub fn get(&self, id: &WalletId) -> Option<&MultisigWallet> {
        self.wallets.get(id)
    }

    pub fn get_mut(&mut self, id: &WalletId) -> Option<&mut MultisigWallet> {
        self.wallets.get_mut(id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn addr(n: u8) -> Address { let mut a = [0u8; 32]; a[0] = n; a }
    fn wid() -> WalletId { [1u8; 32] }

    #[test]
    fn create_2_of_3_wallet() {
        let w = MultisigWallet::new(wid(), vec![addr(1), addr(2), addr(3)], 2, 100, None).unwrap();
        assert_eq!(w.threshold, 2);
        assert_eq!(w.owners.len(), 3);
    }

    #[test]
    fn reject_invalid_threshold() {
        assert_eq!(
            MultisigWallet::new(wid(), vec![addr(1), addr(2)], 3, 100, None).unwrap_err(),
            MultisigError::InvalidThreshold
        );
        assert_eq!(
            MultisigWallet::new(wid(), vec![addr(1), addr(2)], 0, 100, None).unwrap_err(),
            MultisigError::InvalidThreshold
        );
    }

    #[test]
    fn reject_duplicate_owners() {
        assert_eq!(
            MultisigWallet::new(wid(), vec![addr(1), addr(1)], 1, 100, None).unwrap_err(),
            MultisigError::DuplicateOwner
        );
    }

    #[test]
    fn reject_too_few_owners() {
        assert_eq!(
            MultisigWallet::new(wid(), vec![addr(1)], 1, 100, None).unwrap_err(),
            MultisigError::TooFewOwners
        );
    }

    #[test]
    fn propose_and_execute_2_of_3() {
        let mut w = MultisigWallet::new(wid(), vec![addr(1), addr(2), addr(3)], 2, 100, None).unwrap();
        let pid = w.propose(addr(1), addr(99), 500, vec![0xAB], 10).unwrap();
        // proposer auto-approves, need one more
        assert_eq!(w.execute(pid, 10).unwrap_err(), MultisigError::InsufficientApprovals);
        w.approve(addr(2), pid, 10).unwrap();
        let (target, value, data) = w.execute(pid, 10).unwrap();
        assert_eq!(target, addr(99));
        assert_eq!(value, 500);
        assert_eq!(data, vec![0xAB]);
        assert_eq!(w.nonce, 1);
    }

    #[test]
    fn double_approve_rejected() {
        let mut w = MultisigWallet::new(wid(), vec![addr(1), addr(2)], 2, 100, None).unwrap();
        let pid = w.propose(addr(1), addr(99), 0, vec![], 10).unwrap();
        assert_eq!(w.approve(addr(1), pid, 10).unwrap_err(), MultisigError::AlreadyApproved);
    }

    #[test]
    fn non_owner_cannot_propose() {
        let mut w = MultisigWallet::new(wid(), vec![addr(1), addr(2)], 1, 100, None).unwrap();
        assert_eq!(w.propose(addr(99), addr(1), 0, vec![], 10).unwrap_err(), MultisigError::NotOwner);
    }

    #[test]
    fn proposal_expires() {
        let mut w = MultisigWallet::new(wid(), vec![addr(1), addr(2)], 2, 50, None).unwrap();
        let pid = w.propose(addr(1), addr(99), 0, vec![], 10).unwrap();
        assert_eq!(w.approve(addr(2), pid, 61).unwrap_err(), MultisigError::ProposalExpired);
    }

    #[test]
    fn cancel_proposal() {
        let mut w = MultisigWallet::new(wid(), vec![addr(1), addr(2)], 1, 100, None).unwrap();
        let pid = w.propose(addr(1), addr(99), 0, vec![], 10).unwrap();
        // non-proposer cannot cancel
        assert_eq!(w.cancel(addr(2), pid).unwrap_err(), MultisigError::OnlyProposerCanCancel);
        w.cancel(addr(1), pid).unwrap();
        assert_eq!(w.execute(pid, 10).unwrap_err(), MultisigError::ProposalCancelled);
    }

    #[test]
    fn reject_vote() {
        let mut w = MultisigWallet::new(wid(), vec![addr(1), addr(2), addr(3)], 2, 100, None).unwrap();
        let pid = w.propose(addr(1), addr(99), 0, vec![], 10).unwrap();
        w.reject(addr(2), pid, 10).unwrap();
        // addr(2) already rejected, can't approve
        assert_eq!(w.approve(addr(2), pid, 10).unwrap_err(), MultisigError::AlreadyRejected);
    }

    #[test]
    fn daily_limit_enforced() {
        let mut w = MultisigWallet::new(wid(), vec![addr(1), addr(2)], 1, 5000, Some(1000)).unwrap();
        let p1 = w.propose(addr(1), addr(99), 600, vec![], 10).unwrap();
        w.execute(p1, 10).unwrap();
        let p2 = w.propose(addr(1), addr(99), 500, vec![], 10).unwrap();
        assert_eq!(w.execute(p2, 10).unwrap_err(), MultisigError::DailyLimitExceeded);
        // After day reset (2880 epochs), limit resets
        w.execute(p2, 2880).unwrap();
    }

    #[test]
    fn add_and_remove_owner() {
        let mut w = MultisigWallet::new(wid(), vec![addr(1), addr(2), addr(3)], 2, 100, None).unwrap();
        w.add_owner(addr(4)).unwrap();
        assert_eq!(w.owners.len(), 4);
        assert_eq!(w.add_owner(addr(4)).unwrap_err(), MultisigError::OwnerAlreadyExists);
        w.remove_owner(addr(4)).unwrap();
        assert_eq!(w.owners.len(), 3);
        // Can't remove to below 2 owners
        w.remove_owner(addr(3)).unwrap();
        assert_eq!(w.owners.len(), 2);
        assert_eq!(w.remove_owner(addr(2)).unwrap_err(), MultisigError::ThresholdWouldExceedOwners);
    }

    #[test]
    fn change_threshold() {
        let mut w = MultisigWallet::new(wid(), vec![addr(1), addr(2), addr(3)], 2, 100, None).unwrap();
        w.change_threshold(3).unwrap();
        assert_eq!(w.threshold, 3);
        assert_eq!(w.change_threshold(4).unwrap_err(), MultisigError::InvalidThreshold);
    }

    #[test]
    fn registry_create_and_lookup() {
        let mut r = MultisigRegistry::default();
        r.create_wallet(wid(), vec![addr(1), addr(2)], 2, 100, None).unwrap();
        assert!(r.get(&wid()).is_some());
        let w = r.get_mut(&wid()).unwrap();
        let pid = w.propose(addr(1), addr(99), 0, vec![], 0).unwrap();
        w.approve(addr(2), pid, 0).unwrap();
        w.execute(pid, 0).unwrap();
        assert_eq!(r.get(&wid()).unwrap().nonce, 1);
    }

    #[test]
    fn cannot_execute_twice() {
        let mut w = MultisigWallet::new(wid(), vec![addr(1), addr(2)], 1, 100, None).unwrap();
        let pid = w.propose(addr(1), addr(99), 0, vec![], 10).unwrap();
        w.execute(pid, 10).unwrap();
        assert_eq!(w.execute(pid, 10).unwrap_err(), MultisigError::AlreadyExecuted);
    }
}
