//! State trie — account balances and nonce tracking.
//!
//! Implements a simple Merkle Patricia-style state store with:
//! - Account balances (u128 token units)
//! - Nonces (replay protection)
//! - State root computation (deterministic SHA-256 hash of all accounts)
//!
//! This is a simplified in-memory trie suitable for devnet/simulation.
//! A production implementation would use a persistent key-sorted backend.

use std::collections::BTreeMap;

use crate::types::{Address, Hash};

/// Account state stored in the trie.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AccountState {
    pub balance: u128,
    pub nonce: u64,
    /// Optional code hash for future smart contract support.
    pub code_hash: Option<Hash>,
    /// Arbitrary storage slots (key → value).
    pub storage: BTreeMap<Hash, Hash>,
}

impl AccountState {
    pub fn new(balance: u128) -> Self {
        Self {
            balance,
            nonce: 0,
            code_hash: None,
            storage: BTreeMap::new(),
        }
    }

    /// Serialize account to deterministic bytes for hashing.
    fn to_bytes(&self) -> Vec<u8> {
        let mut buf = Vec::with_capacity(64);
        buf.extend_from_slice(&self.balance.to_be_bytes());
        buf.extend_from_slice(&self.nonce.to_be_bytes());
        match &self.code_hash {
            Some(h) => {
                buf.push(1);
                buf.extend_from_slice(h);
            }
            None => buf.push(0),
        }
        // Storage: sorted by key (BTreeMap guarantees order).
        for (k, v) in &self.storage {
            buf.extend_from_slice(k);
            buf.extend_from_slice(v);
        }
        buf
    }
}

impl Default for AccountState {
    fn default() -> Self {
        Self::new(0)
    }
}

/// In-memory state trie.
#[derive(Debug, Clone)]
pub struct StateTrie {
    accounts: BTreeMap<Address, AccountState>,
    /// Cached state root (invalidated on mutation).
    cached_root: Option<Hash>,
}

impl StateTrie {
    pub fn new() -> Self {
        Self {
            accounts: BTreeMap::new(),
            cached_root: None,
        }
    }

    /// Get account state (returns default for nonexistent accounts).
    pub fn get(&self, addr: &Address) -> AccountState {
        self.accounts.get(addr).cloned().unwrap_or_default()
    }

    /// Check if account exists in the trie.
    pub fn exists(&self, addr: &Address) -> bool {
        self.accounts.contains_key(addr)
    }

    /// Set balance for an account. Creates account if nonexistent.
    pub fn set_balance(&mut self, addr: Address, balance: u128) {
        self.cached_root = None;
        let acct = self.accounts.entry(addr).or_default();
        acct.balance = balance;
    }

    /// Add to balance. Returns new balance.
    pub fn credit(&mut self, addr: Address, amount: u128) -> u128 {
        self.cached_root = None;
        let acct = self.accounts.entry(addr).or_default();
        acct.balance = acct.balance.saturating_add(amount);
        acct.balance
    }

    /// Subtract from balance. Returns Err if insufficient funds.
    pub fn debit(&mut self, addr: Address, amount: u128) -> Result<u128, StateError> {
        self.cached_root = None;
        let acct = self.accounts.entry(addr).or_default();
        if acct.balance < amount {
            return Err(StateError::InsufficientBalance {
                addr,
                have: acct.balance,
                need: amount,
            });
        }
        acct.balance -= amount;
        Ok(acct.balance)
    }

    /// Transfer tokens between accounts. Atomic: fails if insufficient balance.
    pub fn transfer(&mut self, from: Address, to: Address, amount: u128) -> Result<(), StateError> {
        if from == to {
            return Ok(());
        }
        // Check balance first without mutating.
        let from_bal = self.get(&from).balance;
        if from_bal < amount {
            return Err(StateError::InsufficientBalance {
                addr: from,
                have: from_bal,
                need: amount,
            });
        }
        self.debit(from, amount)?;
        self.credit(to, amount);
        Ok(())
    }

    /// Get and increment nonce. Returns the nonce to use (pre-increment value).
    pub fn use_nonce(&mut self, addr: Address) -> u64 {
        self.cached_root = None;
        let acct = self.accounts.entry(addr).or_default();
        let n = acct.nonce;
        acct.nonce += 1;
        n
    }

    /// Check expected nonce without consuming it.
    pub fn expected_nonce(&self, addr: &Address) -> u64 {
        self.get(addr).nonce
    }

    /// Validate and consume a nonce. Returns Err on mismatch.
    pub fn validate_nonce(&mut self, addr: Address, provided: u64) -> Result<(), StateError> {
        let expected = self.expected_nonce(&addr);
        if provided != expected {
            return Err(StateError::NonceMismatch {
                addr,
                expected,
                provided,
            });
        }
        self.use_nonce(addr);
        Ok(())
    }

    /// Set a storage slot for an account.
    pub fn set_storage(&mut self, addr: Address, key: Hash, value: Hash) {
        self.cached_root = None;
        let acct = self.accounts.entry(addr).or_default();
        if value == [0u8; 32] {
            acct.storage.remove(&key);
        } else {
            acct.storage.insert(key, value);
        }
    }

    /// Get a storage slot value.
    pub fn get_storage(&self, addr: &Address, key: &Hash) -> Hash {
        self.get(addr)
            .storage
            .get(key)
            .copied()
            .unwrap_or([0u8; 32])
    }

    /// Compute the state root hash. Cached until next mutation.
    pub fn root(&mut self) -> Hash {
        if let Some(r) = self.cached_root {
            return r;
        }
        let root = self.compute_root();
        self.cached_root = Some(root);
        root
    }

    fn compute_root(&self) -> Hash {
        use sha2::{Digest, Sha256};

        let mut hasher = Sha256::new();
        // Empty trie has a well-known root.
        if self.accounts.is_empty() {
            return [0u8; 32];
        }
        for (addr, acct) in &self.accounts {
            hasher.update(&addr.0);
            hasher.update(&acct.to_bytes());
        }
        let result = hasher.finalize();
        let mut hash = [0u8; 32];
        hash.copy_from_slice(&result);
        hash
    }

    /// Number of accounts in the trie.
    pub fn account_count(&self) -> usize {
        self.accounts.len()
    }

    /// Total supply across all accounts.
    pub fn total_supply(&self) -> u128 {
        self.accounts.values().map(|a| a.balance).sum()
    }

    /// Iterate over all accounts (sorted by address).
    pub fn iter(&self) -> impl Iterator<Item = (&Address, &AccountState)> {
        self.accounts.iter()
    }

    /// Create a snapshot (clone) of the current state.
    pub fn snapshot(&self) -> Self {
        self.clone()
    }

    /// Remove an account entirely (e.g., empty account pruning).
    pub fn remove(&mut self, addr: &Address) -> Option<AccountState> {
        self.cached_root = None;
        self.accounts.remove(addr)
    }

    /// Prune zero-balance, zero-nonce, no-code, no-storage accounts.
    pub fn prune_empty(&mut self) -> usize {
        self.cached_root = None;
        let before = self.accounts.len();
        self.accounts.retain(|_, acct| {
            acct.balance > 0
                || acct.nonce > 0
                || acct.code_hash.is_some()
                || !acct.storage.is_empty()
        });
        before - self.accounts.len()
    }
}

impl Default for StateTrie {
    fn default() -> Self {
        Self::new()
    }
}

/// Errors from state operations.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StateError {
    InsufficientBalance {
        addr: Address,
        have: u128,
        need: u128,
    },
    NonceMismatch {
        addr: Address,
        expected: u64,
        provided: u64,
    },
}

impl std::fmt::Display for StateError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InsufficientBalance { addr, have, need } => {
                write!(
                    f,
                    "insufficient balance for {addr}: have {have}, need {need}"
                )
            }
            Self::NonceMismatch {
                addr,
                expected,
                provided,
            } => {
                write!(
                    f,
                    "nonce mismatch for {addr}: expected {expected}, got {provided}"
                )
            }
        }
    }
}

impl std::error::Error for StateError {}

#[cfg(test)]
mod tests {
    use super::*;

    fn addr(id: u8) -> Address {
        Address::test(id)
    }

    #[test]
    fn test_empty_trie() {
        let mut trie = StateTrie::new();
        assert_eq!(trie.account_count(), 0);
        assert_eq!(trie.total_supply(), 0);
        assert_eq!(trie.root(), [0u8; 32]);
    }

    #[test]
    fn test_credit_debit() {
        let mut trie = StateTrie::new();
        let a = addr(1);
        trie.credit(a, 1000);
        assert_eq!(trie.get(&a).balance, 1000);
        trie.debit(a, 300).unwrap();
        assert_eq!(trie.get(&a).balance, 700);
    }

    #[test]
    fn test_insufficient_balance() {
        let mut trie = StateTrie::new();
        let a = addr(1);
        trie.credit(a, 100);
        let err = trie.debit(a, 200).unwrap_err();
        assert!(matches!(err, StateError::InsufficientBalance { .. }));
    }

    #[test]
    fn test_transfer() {
        let mut trie = StateTrie::new();
        let a = addr(1);
        let b = addr(2);
        trie.credit(a, 1000);
        trie.transfer(a, b, 400).unwrap();
        assert_eq!(trie.get(&a).balance, 600);
        assert_eq!(trie.get(&b).balance, 400);
        assert_eq!(trie.total_supply(), 1000);
    }

    #[test]
    fn test_transfer_insufficient() {
        let mut trie = StateTrie::new();
        let a = addr(1);
        let b = addr(2);
        trie.credit(a, 100);
        assert!(trie.transfer(a, b, 200).is_err());
        // Atomic: nothing should change.
        assert_eq!(trie.get(&a).balance, 100);
        assert_eq!(trie.get(&b).balance, 0);
    }

    #[test]
    fn test_self_transfer() {
        let mut trie = StateTrie::new();
        let a = addr(1);
        trie.credit(a, 500);
        trie.transfer(a, a, 100).unwrap();
        assert_eq!(trie.get(&a).balance, 500);
    }

    #[test]
    fn test_nonce_tracking() {
        let mut trie = StateTrie::new();
        let a = addr(1);
        assert_eq!(trie.expected_nonce(&a), 0);
        assert_eq!(trie.use_nonce(a), 0);
        assert_eq!(trie.use_nonce(a), 1);
        assert_eq!(trie.expected_nonce(&a), 2);
    }

    #[test]
    fn test_validate_nonce() {
        let mut trie = StateTrie::new();
        let a = addr(1);
        trie.validate_nonce(a, 0).unwrap();
        trie.validate_nonce(a, 1).unwrap();
        let err = trie.validate_nonce(a, 5).unwrap_err();
        assert!(matches!(
            err,
            StateError::NonceMismatch {
                expected: 2,
                provided: 5,
                ..
            }
        ));
    }

    #[test]
    fn test_storage_slots() {
        let mut trie = StateTrie::new();
        let a = addr(1);
        let key = [1u8; 32];
        let val = [42u8; 32];
        trie.set_storage(a, key, val);
        assert_eq!(trie.get_storage(&a, &key), val);
        // Zero value deletes the slot.
        trie.set_storage(a, key, [0u8; 32]);
        assert_eq!(trie.get_storage(&a, &key), [0u8; 32]);
    }

    #[test]
    fn test_state_root_deterministic() {
        let mut t1 = StateTrie::new();
        let mut t2 = StateTrie::new();
        // Same operations, same order → same root.
        for i in 0..5 {
            t1.credit(addr(i), (i as u128 + 1) * 100);
            t2.credit(addr(i), (i as u128 + 1) * 100);
        }
        assert_eq!(t1.root(), t2.root());
        assert_ne!(t1.root(), [0u8; 32]);
    }

    #[test]
    fn test_state_root_changes_on_mutation() {
        let mut trie = StateTrie::new();
        trie.credit(addr(1), 1000);
        let r1 = trie.root();
        trie.credit(addr(2), 500);
        let r2 = trie.root();
        assert_ne!(r1, r2);
    }

    #[test]
    fn test_snapshot_isolation() {
        let mut trie = StateTrie::new();
        trie.credit(addr(1), 1000);
        let snap = trie.snapshot();
        trie.credit(addr(1), 500);
        assert_eq!(snap.get(&addr(1)).balance, 1000);
        assert_eq!(trie.get(&addr(1)).balance, 1500);
    }

    #[test]
    fn test_prune_empty() {
        let mut trie = StateTrie::new();
        trie.credit(addr(1), 1000);
        trie.credit(addr(2), 500);
        trie.debit(addr(2), 500).unwrap();
        let pruned = trie.prune_empty();
        assert_eq!(pruned, 1);
        assert_eq!(trie.account_count(), 1);
    }

    #[test]
    fn test_prune_keeps_nonzero_nonce() {
        let mut trie = StateTrie::new();
        trie.use_nonce(addr(1)); // nonce=1, balance=0
        let pruned = trie.prune_empty();
        assert_eq!(pruned, 0);
        assert!(trie.exists(&addr(1)));
    }

    #[test]
    fn test_remove_account() {
        let mut trie = StateTrie::new();
        trie.credit(addr(1), 1000);
        let removed = trie.remove(&addr(1));
        assert!(removed.is_some());
        assert_eq!(removed.unwrap().balance, 1000);
        assert!(!trie.exists(&addr(1)));
    }

    #[test]
    fn test_total_supply() {
        let mut trie = StateTrie::new();
        trie.credit(addr(1), 1000);
        trie.credit(addr(2), 2000);
        trie.credit(addr(3), 3000);
        assert_eq!(trie.total_supply(), 6000);
        trie.transfer(addr(1), addr(2), 500).unwrap();
        assert_eq!(trie.total_supply(), 6000); // Conservation.
    }
}
