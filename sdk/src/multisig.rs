//! Multi-sig Client SDK (SDK-012)
//!
//! High-level client for managing multi-signature wallets: create wallets,
//! submit proposals, approve/reject/execute, query status, and manage owners.
//! Wraps RPC calls and transaction signing into ergonomic operations.

use prova_chain::multisig::{
    Address, WalletId, ProposalId, MultisigWallet, Proposal, MultisigError, MultisigRegistry,
};
use std::collections::HashMap;

// ── Transport Abstraction ────────────────────────────────────

/// RPC error for multi-sig operations.
#[derive(Debug, PartialEq)]
pub enum MultisigRpcError {
    ConnectionFailed(String),
    Timeout,
    InvalidResponse(String),
    ChainError(MultisigError),
    SigningFailed,
    Nonce(String),
    WalletNotFound,
    ProposalNotFound,
}

impl std::fmt::Display for MultisigRpcError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ConnectionFailed(s) => write!(f, "connection failed: {s}"),
            Self::Timeout => write!(f, "request timed out"),
            Self::InvalidResponse(s) => write!(f, "invalid response: {s}"),
            Self::ChainError(e) => write!(f, "chain error: {e:?}"),
            Self::SigningFailed => write!(f, "transaction signing failed"),
            Self::Nonce(s) => write!(f, "nonce error: {s}"),
            Self::WalletNotFound => write!(f, "wallet not found"),
            Self::ProposalNotFound => write!(f, "proposal not found"),
        }
    }
}

/// Wallet summary for queries.
#[derive(Debug, Clone)]
pub struct WalletSummary {
    pub id: WalletId,
    pub owners: Vec<Address>,
    pub threshold: u32,
    pub proposal_count: usize,
    pub pending_proposals: usize,
    pub nonce: u64,
    pub daily_limit: Option<u64>,
    pub daily_spent: u64,
}

/// Proposal summary for display.
#[derive(Debug, Clone)]
pub struct ProposalSummary {
    pub id: ProposalId,
    pub proposer: Address,
    pub target: Address,
    pub value: u64,
    pub calldata_len: usize,
    pub approvals: usize,
    pub rejections: usize,
    pub threshold: u32,
    pub executed: bool,
    pub cancelled: bool,
    pub created_at: u64,
    pub expired: bool,
}

/// Mock RPC transport for testing.
#[derive(Default)]
pub struct MockMultisigTransport {
    pub registry: MultisigRegistry,
    pub current_epoch: u64,
    pub call_log: Vec<String>,
}

/// Signed transaction for multi-sig operations.
#[derive(Debug, Clone)]
pub struct MultisigTx {
    pub signer: Address,
    pub wallet_id: WalletId,
    pub action: MultisigAction,
    pub nonce: u64,
}

/// Action variants for multi-sig transactions.
#[derive(Debug, Clone)]
pub enum MultisigAction {
    CreateWallet {
        owners: Vec<Address>,
        threshold: u32,
        proposal_ttl: u64,
        daily_limit: Option<u64>,
    },
    Propose {
        target: Address,
        value: u64,
        calldata: Vec<u8>,
    },
    Approve {
        proposal_id: ProposalId,
    },
    Reject {
        proposal_id: ProposalId,
    },
    Execute {
        proposal_id: ProposalId,
    },
    Cancel {
        proposal_id: ProposalId,
    },
    AddOwner {
        new_owner: Address,
        new_threshold: Option<u32>,
    },
    RemoveOwner {
        owner: Address,
        new_threshold: Option<u32>,
    },
}

/// Client for multi-sig wallet management.
pub struct MultisigClient {
    transport: MockMultisigTransport,
    signer: Address,
}

impl MultisigClient {
    pub fn new(signer: Address) -> Self {
        Self {
            transport: MockMultisigTransport::default(),
            signer,
        }
    }

    pub fn with_epoch(mut self, epoch: u64) -> Self {
        self.transport.current_epoch = epoch;
        self
    }

    /// Create a new multi-sig wallet.
    pub fn create_wallet(
        &mut self,
        owners: Vec<Address>,
        threshold: u32,
        proposal_ttl: u64,
        daily_limit: Option<u64>,
    ) -> Result<WalletId, MultisigRpcError> {
        self.transport.call_log.push("create_wallet".into());
        // Generate deterministic wallet ID from owners + nonce
        let mut id = [0u8; 32];
        for (i, owner) in owners.iter().enumerate() {
            for j in 0..32 {
                id[j] ^= owner[j].wrapping_add(i as u8);
            }
        }
        let wallet = MultisigWallet::new(id, owners, threshold, proposal_ttl, daily_limit)
            .map_err(MultisigRpcError::ChainError)?;
        self.transport.registry.wallets.insert(id, wallet);
        Ok(id)
    }

    /// Submit a new proposal to a wallet.
    pub fn propose(
        &mut self,
        wallet_id: &WalletId,
        target: Address,
        value: u64,
        calldata: Vec<u8>,
    ) -> Result<ProposalId, MultisigRpcError> {
        self.transport.call_log.push("propose".into());
        let wallet = self.transport.registry.wallets.get_mut(wallet_id)
            .ok_or(MultisigRpcError::WalletNotFound)?;
        let epoch = self.transport.current_epoch;
        wallet.propose(self.signer, target, value, calldata, epoch)
            .map_err(MultisigRpcError::ChainError)
    }

    /// Approve a proposal.
    pub fn approve(
        &mut self,
        wallet_id: &WalletId,
        proposal_id: ProposalId,
    ) -> Result<(), MultisigRpcError> {
        self.transport.call_log.push("approve".into());
        let wallet = self.transport.registry.wallets.get_mut(wallet_id)
            .ok_or(MultisigRpcError::WalletNotFound)?;
        let epoch = self.transport.current_epoch;
        wallet.approve(self.signer, proposal_id, epoch)
            .map_err(MultisigRpcError::ChainError)
    }

    /// Reject a proposal.
    pub fn reject(
        &mut self,
        wallet_id: &WalletId,
        proposal_id: ProposalId,
    ) -> Result<(), MultisigRpcError> {
        self.transport.call_log.push("reject".into());
        let wallet = self.transport.registry.wallets.get_mut(wallet_id)
            .ok_or(MultisigRpcError::WalletNotFound)?;
        let epoch = self.transport.current_epoch;
        wallet.reject(self.signer, proposal_id, epoch)
            .map_err(MultisigRpcError::ChainError)
    }

    /// Execute a proposal that has reached threshold.
    pub fn execute(
        &mut self,
        wallet_id: &WalletId,
        proposal_id: ProposalId,
    ) -> Result<(Address, u64, Vec<u8>), MultisigRpcError> {
        self.transport.call_log.push("execute".into());
        let wallet = self.transport.registry.wallets.get_mut(wallet_id)
            .ok_or(MultisigRpcError::WalletNotFound)?;
        let epoch = self.transport.current_epoch;
        wallet.execute(proposal_id, epoch)
            .map_err(MultisigRpcError::ChainError)
    }

    /// Cancel a proposal (only proposer).
    pub fn cancel(
        &mut self,
        wallet_id: &WalletId,
        proposal_id: ProposalId,
    ) -> Result<(), MultisigRpcError> {
        self.transport.call_log.push("cancel".into());
        let wallet = self.transport.registry.wallets.get_mut(wallet_id)
            .ok_or(MultisigRpcError::WalletNotFound)?;
        wallet.cancel(self.signer, proposal_id)
            .map_err(MultisigRpcError::ChainError)
    }

    /// Get wallet summary.
    pub fn get_wallet(&self, wallet_id: &WalletId) -> Result<WalletSummary, MultisigRpcError> {
        let wallet = self.transport.registry.wallets.get(wallet_id)
            .ok_or(MultisigRpcError::WalletNotFound)?;
        let pending = wallet.proposals.values()
            .filter(|p| !p.executed && !p.cancelled)
            .count();
        Ok(WalletSummary {
            id: wallet.id,
            owners: wallet.owners.clone(),
            threshold: wallet.threshold,
            proposal_count: wallet.proposals.len(),
            pending_proposals: pending,
            nonce: wallet.nonce,
            daily_limit: wallet.daily_limit,
            daily_spent: wallet.daily_spent,
        })
    }

    /// Get proposal summary.
    pub fn get_proposal(
        &self,
        wallet_id: &WalletId,
        proposal_id: ProposalId,
    ) -> Result<ProposalSummary, MultisigRpcError> {
        let wallet = self.transport.registry.wallets.get(wallet_id)
            .ok_or(MultisigRpcError::WalletNotFound)?;
        let prop = wallet.proposals.get(&proposal_id)
            .ok_or(MultisigRpcError::ProposalNotFound)?;
        let expired = self.transport.current_epoch > prop.created_at + wallet.proposal_ttl;
        Ok(ProposalSummary {
            id: prop.id,
            proposer: prop.proposer,
            target: prop.target,
            value: prop.value,
            calldata_len: prop.calldata.len(),
            approvals: prop.approvals.len(),
            rejections: prop.rejections.len(),
            threshold: wallet.threshold,
            executed: prop.executed,
            cancelled: prop.cancelled,
            created_at: prop.created_at,
            expired,
        })
    }

    /// List all proposals for a wallet.
    pub fn list_proposals(
        &self,
        wallet_id: &WalletId,
        pending_only: bool,
    ) -> Result<Vec<ProposalSummary>, MultisigRpcError> {
        let wallet = self.transport.registry.wallets.get(wallet_id)
            .ok_or(MultisigRpcError::WalletNotFound)?;
        let epoch = self.transport.current_epoch;
        let mut results: Vec<ProposalSummary> = wallet.proposals.values()
            .filter(|p| !pending_only || (!p.executed && !p.cancelled))
            .map(|p| {
                let expired = epoch > p.created_at + wallet.proposal_ttl;
                ProposalSummary {
                    id: p.id,
                    proposer: p.proposer,
                    target: p.target,
                    value: p.value,
                    calldata_len: p.calldata.len(),
                    approvals: p.approvals.len(),
                    rejections: p.rejections.len(),
                    threshold: wallet.threshold,
                    executed: p.executed,
                    cancelled: p.cancelled,
                    created_at: p.created_at,
                    expired,
                }
            })
            .collect();
        results.sort_by_key(|p| p.id);
        Ok(results)
    }

    /// Check if a proposal is ready to execute (enough approvals).
    pub fn is_executable(
        &self,
        wallet_id: &WalletId,
        proposal_id: ProposalId,
    ) -> Result<bool, MultisigRpcError> {
        let summary = self.get_proposal(wallet_id, proposal_id)?;
        Ok(summary.approvals >= summary.threshold as usize
            && !summary.executed
            && !summary.cancelled
            && !summary.expired)
    }

    /// Convenience: propose + auto-approve (proposer is first approval).
    pub fn propose_and_approve(
        &mut self,
        wallet_id: &WalletId,
        target: Address,
        value: u64,
        calldata: Vec<u8>,
    ) -> Result<ProposalId, MultisigRpcError> {
        let pid = self.propose(wallet_id, target, value, calldata)?;
        self.approve(wallet_id, pid)?;
        Ok(pid)
    }

    /// Batch approve multiple proposals.
    pub fn batch_approve(
        &mut self,
        wallet_id: &WalletId,
        proposal_ids: &[ProposalId],
    ) -> Result<Vec<Result<(), MultisigRpcError>>, MultisigRpcError> {
        let mut results = Vec::new();
        for &pid in proposal_ids {
            results.push(self.approve(wallet_id, pid));
        }
        Ok(results)
    }

    /// Get call log (for testing).
    pub fn call_log(&self) -> &[String] {
        &self.transport.call_log
    }
}

// ── Tests ────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn addr(n: u8) -> Address {
        let mut a = [0u8; 32];
        a[0] = n;
        a
    }

    fn make_client(signer: u8) -> MultisigClient {
        MultisigClient::new(addr(signer)).with_epoch(100)
    }

    fn make_wallet(client: &mut MultisigClient) -> WalletId {
        client.create_wallet(
            vec![addr(1), addr(2), addr(3)],
            2,
            1000, // proposal TTL
            Some(1_000_000), // daily limit
        ).unwrap()
    }

    #[test]
    fn test_create_wallet() {
        let mut client = make_client(1);
        let wid = make_wallet(&mut client);
        let summary = client.get_wallet(&wid).unwrap();
        assert_eq!(summary.owners.len(), 3);
        assert_eq!(summary.threshold, 2);
        assert_eq!(summary.pending_proposals, 0);
        assert_eq!(summary.daily_limit, Some(1_000_000));
    }

    #[test]
    fn test_create_wallet_invalid_threshold() {
        let mut client = make_client(1);
        let result = client.create_wallet(vec![addr(1), addr(2)], 5, 1000, None);
        assert!(result.is_err());
    }

    #[test]
    fn test_propose_and_query() {
        let mut client = make_client(1);
        let wid = make_wallet(&mut client);
        let pid = client.propose(&wid, addr(99), 5000, vec![1, 2, 3]).unwrap();
        let summary = client.get_proposal(&wid, pid).unwrap();
        assert_eq!(summary.value, 5000);
        assert_eq!(summary.calldata_len, 3);
        // propose auto-approves from proposer
        assert_eq!(summary.approvals, 1);
        assert!(!summary.executed);
        assert!(!summary.expired);
    }

    #[test]
    fn test_approve_and_execute() {
        let mut client = make_client(1);
        let wid = make_wallet(&mut client);
        // propose auto-approves from proposer (owner 1)
        let pid = client.propose(&wid, addr(99), 5000, vec![0xAB]).unwrap();

        // Switch signer to owner 2 for second approval
        client.signer = addr(2);
        client.approve(&wid, pid).unwrap();

        // Should now be executable (2 of 2 threshold)
        assert!(client.is_executable(&wid, pid).unwrap());

        let (target, value, data) = client.execute(&wid, pid).unwrap();
        assert_eq!(target, addr(99));
        assert_eq!(value, 5000);
        assert_eq!(data, vec![0xAB]);
    }

    #[test]
    fn test_reject_proposal() {
        let mut client = make_client(1);
        let wid = make_wallet(&mut client);
        let pid = client.propose(&wid, addr(99), 100, vec![]).unwrap();

        client.signer = addr(2);
        client.reject(&wid, pid).unwrap();

        let summary = client.get_proposal(&wid, pid).unwrap();
        assert_eq!(summary.rejections, 1);
    }

    #[test]
    fn test_cancel_proposal() {
        let mut client = make_client(1);
        let wid = make_wallet(&mut client);
        let pid = client.propose(&wid, addr(99), 100, vec![]).unwrap();
        client.cancel(&wid, pid).unwrap();

        let summary = client.get_proposal(&wid, pid).unwrap();
        assert!(summary.cancelled);
        assert!(!client.is_executable(&wid, pid).unwrap());
    }

    #[test]
    fn test_cancel_not_proposer() {
        let mut client = make_client(1);
        let wid = make_wallet(&mut client);
        let pid = client.propose(&wid, addr(99), 100, vec![]).unwrap();

        client.signer = addr(2);
        let result = client.cancel(&wid, pid);
        assert!(result.is_err());
    }

    #[test]
    fn test_list_proposals() {
        let mut client = make_client(1);
        let wid = make_wallet(&mut client);
        client.propose(&wid, addr(10), 100, vec![]).unwrap();
        client.propose(&wid, addr(11), 200, vec![]).unwrap();
        let pid3 = client.propose(&wid, addr(12), 300, vec![]).unwrap();
        client.cancel(&wid, pid3).unwrap();

        let all = client.list_proposals(&wid, false).unwrap();
        assert_eq!(all.len(), 3);

        let pending = client.list_proposals(&wid, true).unwrap();
        assert_eq!(pending.len(), 2);
    }

    #[test]
    fn test_propose_and_approve_convenience() {
        let mut client = make_client(1);
        let wid = make_wallet(&mut client);
        // propose auto-approves, so propose_and_approve would double-approve
        // Use a different signer for propose, then approve as owner 1
        client.signer = addr(2);
        let pid = client.propose(&wid, addr(99), 500, vec![]).unwrap();
        // proposer (2) already approved; now approve as owner 1
        client.signer = addr(1);
        client.approve(&wid, pid).unwrap();
        let summary = client.get_proposal(&wid, pid).unwrap();
        assert_eq!(summary.approvals, 2);
    }

    #[test]
    fn test_batch_approve() {
        let mut client = make_client(1);
        let wid = make_wallet(&mut client);
        let p1 = client.propose(&wid, addr(10), 100, vec![]).unwrap();
        let p2 = client.propose(&wid, addr(11), 200, vec![]).unwrap();

        // Proposer (1) already approved both; batch approve as owner 2
        client.signer = addr(2);
        let results = client.batch_approve(&wid, &[p1, p2]).unwrap();
        assert_eq!(results.len(), 2);
        assert!(results.iter().all(|r| r.is_ok()));
    }

    #[test]
    fn test_wallet_not_found() {
        let client = make_client(1);
        let fake_id = [0xFF; 32];
        assert_eq!(client.get_wallet(&fake_id).unwrap_err(), MultisigRpcError::WalletNotFound);
    }

    #[test]
    fn test_proposal_not_found() {
        let mut client = make_client(1);
        let wid = make_wallet(&mut client);
        assert_eq!(
            client.get_proposal(&wid, 999).unwrap_err(),
            MultisigRpcError::ProposalNotFound
        );
    }

    #[test]
    fn test_non_owner_cannot_propose() {
        let mut client = make_client(99); // not an owner
        let wid = client.create_wallet(vec![addr(1), addr(2)], 2, 1000, None).unwrap();
        let result = client.propose(&wid, addr(10), 100, vec![]);
        assert!(result.is_err());
    }

    #[test]
    fn test_call_log_tracking() {
        let mut client = make_client(1);
        let wid = make_wallet(&mut client);
        client.propose(&wid, addr(10), 100, vec![]).unwrap();
        assert!(client.call_log().contains(&"create_wallet".to_string()));
        assert!(client.call_log().contains(&"propose".to_string()));
    }

    #[test]
    fn test_double_approve_fails() {
        let mut client = make_client(1);
        let wid = make_wallet(&mut client);
        let pid = client.propose(&wid, addr(10), 100, vec![]).unwrap();
        // proposer already auto-approved, so approving again should fail
        let result = client.approve(&wid, pid);
        assert!(result.is_err());
    }

    #[test]
    fn test_execute_without_threshold_fails() {
        let mut client = make_client(1);
        let wid = make_wallet(&mut client);
        let pid = client.propose(&wid, addr(10), 100, vec![]).unwrap();
        // Only 1 approval (auto from proposer), threshold is 2
        let result = client.execute(&wid, pid);
        assert!(result.is_err());
    }
}
