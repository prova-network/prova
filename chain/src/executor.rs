//! Transaction execution engine — applies transactions to the state trie.
//!
//! Supports transaction types:
//! - **Transfer**: Move tokens between accounts
//! - **Stake**: Lock tokens for network participation (delegates to stake ledger)
//! - **RegisterModel**: Add a model to the registry
//! - **InferenceCommit**: Publish an inference activation root
//! - **ClaimReward**: Withdraw accumulated rewards
//!
//! Execution is atomic per-transaction: if any step fails, state is unchanged.
//! Batch execution processes transactions sequentially, collecting receipts.

use crate::state::{StateError, StateTrie};
use crate::types::*;

/// Transaction types supported by the execution engine.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TxKind {
    /// Transfer tokens from sender to recipient.
    Transfer { to: Address, amount: u128 },
    /// Stake tokens for network participation.
    Stake { amount: u128 },
    /// Unstake tokens (begins unlock period).
    Unstake { amount: u128 },
    /// Register a model in the on-chain registry.
    RegisterModel {
        model_id: ModelId,
        weight_hash: Hash,
        arch_group: String,
    },
    /// Publish an inference commitment.
    InferenceCommit {
        model_id: ModelId,
        activation_root: Hash,
    },
    /// Claim pending rewards.
    ClaimReward,
    /// Pay an inference fee (client → provider + treasury split).
    PayInferenceFee { provider: Address, amount: u128 },
}

/// A signed transaction ready for execution.
#[derive(Debug, Clone)]
pub struct Transaction {
    pub sender: Address,
    pub nonce: u64,
    pub kind: TxKind,
    /// Gas limit (simplified: flat cost per tx type).
    pub gas_limit: u64,
    /// Gas price in smallest token unit.
    pub gas_price: u128,
}

impl Transaction {
    pub fn new(sender: Address, nonce: u64, kind: TxKind) -> Self {
        // Default gas limit matches the tx type cost.
        let gas_limit = match &kind {
            TxKind::Transfer { .. } => 21_000,
            TxKind::Stake { .. } | TxKind::Unstake { .. } => 40_000,
            TxKind::RegisterModel { .. } => 60_000,
            TxKind::InferenceCommit { .. } => 50_000,
            TxKind::ClaimReward => 30_000,
            TxKind::PayInferenceFee { .. } => 25_000,
        };
        Self {
            sender,
            nonce,
            kind,
            gas_limit,
            gas_price: 1,
        }
    }

    pub fn with_gas(mut self, limit: u64, price: u128) -> Self {
        self.gas_limit = limit;
        self.gas_price = price;
        self
    }

    /// Compute gas cost for this transaction type.
    fn gas_cost(&self) -> u64 {
        match &self.kind {
            TxKind::Transfer { .. } => 21_000,
            TxKind::Stake { .. } | TxKind::Unstake { .. } => 40_000,
            TxKind::RegisterModel { .. } => 60_000,
            TxKind::InferenceCommit { .. } => 50_000,
            TxKind::ClaimReward => 30_000,
            TxKind::PayInferenceFee { .. } => 25_000,
        }
    }
}

/// Execution receipt for a processed transaction.
#[derive(Debug, Clone)]
pub struct Receipt {
    pub tx_index: usize,
    pub success: bool,
    pub gas_used: u64,
    pub error: Option<ExecError>,
    /// Post-execution state root.
    pub state_root: Hash,
}

/// Errors during transaction execution.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExecError {
    /// Nonce doesn't match expected value.
    NonceMismatch { expected: u64, provided: u64 },
    /// Insufficient balance for transfer + gas.
    InsufficientBalance { have: u128, need: u128 },
    /// Gas limit too low for this transaction type.
    OutOfGas { required: u64, limit: u64 },
    /// Transfer to self is a no-op but still costs gas.
    SelfTransfer,
    /// Zero-amount transfer or stake.
    ZeroAmount,
    /// Model already registered.
    ModelAlreadyRegistered,
    /// No rewards to claim.
    NoRewardsToClaim,
    /// Generic state error.
    State(String),
}

impl From<StateError> for ExecError {
    fn from(e: StateError) -> Self {
        ExecError::State(e.to_string())
    }
}

impl std::fmt::Display for ExecError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NonceMismatch { expected, provided } => {
                write!(f, "nonce mismatch: expected {expected}, got {provided}")
            }
            Self::InsufficientBalance { have, need } => {
                write!(f, "insufficient balance: have {have}, need {need}")
            }
            Self::OutOfGas { required, limit } => {
                write!(f, "out of gas: need {required}, limit {limit}")
            }
            Self::SelfTransfer => write!(f, "self-transfer"),
            Self::ZeroAmount => write!(f, "zero amount"),
            Self::ModelAlreadyRegistered => write!(f, "model already registered"),
            Self::NoRewardsToClaim => write!(f, "no rewards to claim"),
            Self::State(msg) => write!(f, "state error: {msg}"),
        }
    }
}

/// Transaction executor — applies transactions to state.
///
/// Maintains its own view of registered models and pending rewards
/// (simplified — production would read from dedicated state subtrees).
pub struct Executor {
    /// Registered model IDs (simplified tracking).
    registered_models: std::collections::HashSet<[u8; 32]>,
    /// Pending rewards per address (simplified — mirrors RewardLedger).
    pending_rewards: std::collections::HashMap<Address, u128>,
    /// Treasury address for gas fee collection.
    pub treasury: Address,
    /// Total gas fees collected.
    pub total_gas_fees: u128,
    /// Total transactions executed.
    pub tx_count: u64,
    /// Total successful transactions.
    pub success_count: u64,
}

impl Executor {
    pub fn new(treasury: Address) -> Self {
        Self {
            registered_models: std::collections::HashSet::new(),
            pending_rewards: std::collections::HashMap::new(),
            treasury,
            total_gas_fees: 0,
            tx_count: 0,
            success_count: 0,
        }
    }

    /// Seed pending rewards (e.g., from RewardLedger sync).
    pub fn set_pending_reward(&mut self, addr: Address, amount: u128) {
        self.pending_rewards.insert(addr, amount);
    }

    /// Pre-register a model (e.g., from genesis state).
    pub fn register_model(&mut self, model_id: &ModelId) {
        self.registered_models.insert(model_id.0);
    }

    /// Execute a single transaction against the state trie.
    /// Returns a receipt. State is only modified on success (atomic).
    pub fn execute(&mut self, state: &mut StateTrie, tx: &Transaction, tx_index: usize) -> Receipt {
        // Take snapshot for rollback on failure.
        let snapshot = state.snapshot();
        let models_snap = self.registered_models.clone();
        let rewards_snap = self.pending_rewards.clone();

        self.tx_count += 1;

        match self.execute_inner(state, tx) {
            Ok(gas_used) => {
                self.success_count += 1;
                Receipt {
                    tx_index,
                    success: true,
                    gas_used,
                    error: None,
                    state_root: state.root(),
                }
            }
            Err(e) => {
                // Rollback state.
                *state = snapshot;
                self.registered_models = models_snap;
                self.pending_rewards = rewards_snap;
                Receipt {
                    tx_index,
                    success: false,
                    gas_used: 0,
                    error: Some(e),
                    state_root: state.root(),
                }
            }
        }
    }

    fn execute_inner(&mut self, state: &mut StateTrie, tx: &Transaction) -> Result<u64, ExecError> {
        // 1. Gas check.
        let gas_cost = tx.gas_cost();
        if tx.gas_limit < gas_cost {
            return Err(ExecError::OutOfGas {
                required: gas_cost,
                limit: tx.gas_limit,
            });
        }
        let gas_fee = gas_cost as u128 * tx.gas_price;

        // 2. Nonce validation.
        let expected = state.expected_nonce(&tx.sender);
        if tx.nonce != expected {
            return Err(ExecError::NonceMismatch {
                expected,
                provided: tx.nonce,
            });
        }

        // 3. Ensure sender can pay gas.
        let sender_bal = state.get(&tx.sender).balance;
        let total_needed = gas_fee
            + match &tx.kind {
                TxKind::Transfer { amount, .. } => *amount,
                TxKind::Stake { amount } => *amount,
                TxKind::PayInferenceFee { amount, .. } => *amount,
                _ => 0,
            };
        if sender_bal < total_needed {
            return Err(ExecError::InsufficientBalance {
                have: sender_bal,
                need: total_needed,
            });
        }

        // 4. Consume nonce.
        state.validate_nonce(tx.sender, tx.nonce)?;

        // 5. Deduct gas fee → treasury.
        if gas_fee > 0 {
            state.debit(tx.sender, gas_fee)?;
            state.credit(self.treasury, gas_fee);
            self.total_gas_fees += gas_fee;
        }

        // 6. Execute transaction body.
        match &tx.kind {
            TxKind::Transfer { to, amount } => {
                if *amount == 0 {
                    return Err(ExecError::ZeroAmount);
                }
                state.transfer(tx.sender, *to, *amount)?;
            }
            TxKind::Stake { amount } => {
                if *amount == 0 {
                    return Err(ExecError::ZeroAmount);
                }
                // Lock tokens by moving to treasury (simplified staking).
                state.transfer(tx.sender, self.treasury, *amount)?;
            }
            TxKind::Unstake { amount } => {
                if *amount == 0 {
                    return Err(ExecError::ZeroAmount);
                }
                // Unlock tokens (simplified — no unbonding period here).
                state.transfer(self.treasury, tx.sender, *amount)?;
            }
            TxKind::RegisterModel { model_id, .. } => {
                if self.registered_models.contains(&model_id.0) {
                    return Err(ExecError::ModelAlreadyRegistered);
                }
                self.registered_models.insert(model_id.0);
            }
            TxKind::InferenceCommit { .. } => {
                // Commit recorded — actual commit storage handled by CommitStore.
                // Execution engine just validates gas + nonce.
            }
            TxKind::ClaimReward => {
                let pending = self.pending_rewards.remove(&tx.sender).unwrap_or(0);
                if pending == 0 {
                    return Err(ExecError::NoRewardsToClaim);
                }
                state.credit(tx.sender, pending);
            }
            TxKind::PayInferenceFee { provider, amount } => {
                if *amount == 0 {
                    return Err(ExecError::ZeroAmount);
                }
                // 90% to provider, 10% to treasury.
                let provider_share = amount * 9 / 10;
                let treasury_share = amount - provider_share;
                state.debit(tx.sender, *amount)?;
                state.credit(*provider, provider_share);
                state.credit(self.treasury, treasury_share);
            }
        }

        Ok(gas_cost)
    }

    /// Execute a batch of transactions sequentially. Returns receipts for each.
    pub fn execute_batch(&mut self, state: &mut StateTrie, txs: &[Transaction]) -> Vec<Receipt> {
        txs.iter()
            .enumerate()
            .map(|(i, tx)| self.execute(state, tx, i))
            .collect()
    }
}

// ─── Tests ───────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn addr(id: u8) -> Address {
        Address::test(id)
    }

    fn treasury() -> Address {
        addr(255)
    }

    fn setup() -> (Executor, StateTrie) {
        let mut state = StateTrie::new();
        state.credit(addr(1), 1_000_000);
        state.credit(addr(2), 500_000);
        let exec = Executor::new(treasury());
        (exec, state)
    }

    fn transfer_tx(sender: u8, to: u8, amount: u128, nonce: u64) -> Transaction {
        Transaction::new(
            addr(sender),
            nonce,
            TxKind::Transfer {
                to: addr(to),
                amount,
            },
        )
    }

    #[test]
    fn test_simple_transfer() {
        let (mut exec, mut state) = setup();
        let tx = transfer_tx(1, 2, 100_000, 0);
        let receipt = exec.execute(&mut state, &tx, 0);
        assert!(receipt.success);
        assert_eq!(receipt.gas_used, 21_000);
        // Balance = initial - amount - gas_fee (21000 * 1).
        assert_eq!(state.get(&addr(1)).balance, 1_000_000 - 100_000 - 21_000);
        assert_eq!(state.get(&addr(2)).balance, 600_000);
    }

    #[test]
    fn test_nonce_enforcement() {
        let (mut exec, mut state) = setup();
        // Wrong nonce.
        let tx = transfer_tx(1, 2, 1000, 5);
        let receipt = exec.execute(&mut state, &tx, 0);
        assert!(!receipt.success);
        assert_eq!(
            receipt.error,
            Some(ExecError::NonceMismatch {
                expected: 0,
                provided: 5
            })
        );
        // Balance unchanged.
        assert_eq!(state.get(&addr(1)).balance, 1_000_000);
    }

    #[test]
    fn test_sequential_nonces() {
        let (mut exec, mut state) = setup();
        for i in 0..5 {
            let tx = transfer_tx(1, 2, 1000, i);
            let receipt = exec.execute(&mut state, &tx, i as usize);
            assert!(receipt.success, "tx {i} failed: {:?}", receipt.error);
        }
        assert_eq!(state.expected_nonce(&addr(1)), 5);
    }

    #[test]
    fn test_insufficient_balance_for_transfer_plus_gas() {
        let (mut exec, mut state) = setup();
        // Try to send entire balance — gas fee makes it fail.
        let tx = transfer_tx(1, 2, 1_000_000, 0);
        let receipt = exec.execute(&mut state, &tx, 0);
        assert!(!receipt.success);
        assert!(matches!(
            receipt.error,
            Some(ExecError::InsufficientBalance { .. })
        ));
        // Atomic rollback.
        assert_eq!(state.get(&addr(1)).balance, 1_000_000);
        assert_eq!(state.expected_nonce(&addr(1)), 0);
    }

    #[test]
    fn test_out_of_gas() {
        let (mut exec, mut state) = setup();
        let tx = Transaction::new(
            addr(1),
            0,
            TxKind::Transfer {
                to: addr(2),
                amount: 1000,
            },
        )
        .with_gas(100, 1); // Gas limit too low.
        let receipt = exec.execute(&mut state, &tx, 0);
        assert!(!receipt.success);
        assert_eq!(
            receipt.error,
            Some(ExecError::OutOfGas {
                required: 21_000,
                limit: 100
            })
        );
    }

    #[test]
    fn test_zero_amount_transfer() {
        let (mut exec, mut state) = setup();
        let tx = Transaction::new(
            addr(1),
            0,
            TxKind::Transfer {
                to: addr(2),
                amount: 0,
            },
        );
        let receipt = exec.execute(&mut state, &tx, 0);
        assert!(!receipt.success);
        assert_eq!(receipt.error, Some(ExecError::ZeroAmount));
    }

    #[test]
    fn test_stake_and_unstake() {
        let (mut exec, mut state) = setup();
        // Stake.
        let tx = Transaction::new(addr(1), 0, TxKind::Stake { amount: 200_000 });
        let receipt = exec.execute(&mut state, &tx, 0);
        assert!(receipt.success);
        let gas_fee = 40_000; // Stake gas cost * price(1).
        assert_eq!(state.get(&addr(1)).balance, 1_000_000 - 200_000 - gas_fee);

        // Unstake.
        let tx = Transaction::new(addr(1), 1, TxKind::Unstake { amount: 100_000 });
        let receipt = exec.execute(&mut state, &tx, 1);
        assert!(receipt.success);
        assert_eq!(
            state.get(&addr(1)).balance,
            1_000_000 - 200_000 - gas_fee + 100_000 - gas_fee
        );
    }

    #[test]
    fn test_register_model() {
        let (mut exec, mut state) = setup();
        let model_id = ModelId([42u8; 32]);
        let tx = Transaction::new(
            addr(1),
            0,
            TxKind::RegisterModel {
                model_id,
                weight_hash: [1u8; 32],
                arch_group: "nvidia-sm89-int8".to_string(),
            },
        );
        let receipt = exec.execute(&mut state, &tx, 0);
        assert!(receipt.success);

        // Duplicate registration fails.
        let tx2 = Transaction::new(
            addr(1),
            1,
            TxKind::RegisterModel {
                model_id,
                weight_hash: [1u8; 32],
                arch_group: "nvidia-sm89-int8".to_string(),
            },
        );
        let receipt2 = exec.execute(&mut state, &tx2, 1);
        assert!(!receipt2.success);
        assert_eq!(receipt2.error, Some(ExecError::ModelAlreadyRegistered));
    }

    #[test]
    fn test_claim_reward() {
        let (mut exec, mut state) = setup();
        exec.set_pending_reward(addr(1), 50_000);
        let tx = Transaction::new(addr(1), 0, TxKind::ClaimReward);
        let receipt = exec.execute(&mut state, &tx, 0);
        assert!(receipt.success);
        // Balance = initial + reward - gas_fee.
        assert_eq!(state.get(&addr(1)).balance, 1_000_000 + 50_000 - 30_000);
    }

    #[test]
    fn test_claim_no_rewards() {
        let (mut exec, mut state) = setup();
        let tx = Transaction::new(addr(1), 0, TxKind::ClaimReward);
        let receipt = exec.execute(&mut state, &tx, 0);
        assert!(!receipt.success);
        assert_eq!(receipt.error, Some(ExecError::NoRewardsToClaim));
    }

    #[test]
    fn test_pay_inference_fee() {
        let (mut exec, mut state) = setup();
        let provider = addr(3);
        let tx = Transaction::new(
            addr(1),
            0,
            TxKind::PayInferenceFee {
                provider,
                amount: 10_000,
            },
        );
        let receipt = exec.execute(&mut state, &tx, 0);
        assert!(receipt.success);
        // Provider gets 90%.
        assert_eq!(state.get(&provider).balance, 9_000);
        // Treasury gets gas_fee(25000) + 10% of fee(1000).
        assert_eq!(state.get(&treasury()).balance, 25_000 + 1_000);
        // Sender pays amount + gas.
        assert_eq!(state.get(&addr(1)).balance, 1_000_000 - 10_000 - 25_000);
    }

    #[test]
    fn test_batch_execution() {
        let (mut exec, mut state) = setup();
        let txs = vec![
            transfer_tx(1, 2, 10_000, 0),
            transfer_tx(1, 2, 20_000, 1),
            transfer_tx(2, 1, 5_000, 0),
        ];
        let receipts = exec.execute_batch(&mut state, &txs);
        assert_eq!(receipts.len(), 3);
        assert!(receipts.iter().all(|r| r.success));
        assert_eq!(exec.tx_count, 3);
        assert_eq!(exec.success_count, 3);
    }

    #[test]
    fn test_batch_partial_failure() {
        let (mut exec, mut state) = setup();
        let txs = vec![
            transfer_tx(1, 2, 10_000, 0), // OK
            transfer_tx(1, 2, 10_000, 5), // Bad nonce — fails
            transfer_tx(1, 2, 10_000, 1), // OK (nonce 1 still valid)
        ];
        let receipts = exec.execute_batch(&mut state, &txs);
        assert!(receipts[0].success);
        assert!(!receipts[1].success);
        assert!(receipts[2].success);
        assert_eq!(exec.success_count, 2);
    }

    #[test]
    fn test_gas_fees_accumulate() {
        let (mut exec, mut state) = setup();
        let txs = vec![transfer_tx(1, 2, 1000, 0), transfer_tx(1, 2, 1000, 1)];
        exec.execute_batch(&mut state, &txs);
        assert_eq!(exec.total_gas_fees, 21_000 * 2);
        assert_eq!(state.get(&treasury()).balance, 21_000 * 2);
    }

    #[test]
    fn test_state_root_changes_per_tx() {
        let (mut exec, mut state) = setup();
        let tx1 = transfer_tx(1, 2, 1000, 0);
        let tx2 = transfer_tx(1, 2, 2000, 1);
        let r1 = exec.execute(&mut state, &tx1, 0);
        let r2 = exec.execute(&mut state, &tx2, 1);
        assert!(r1.success && r2.success);
        assert_ne!(r1.state_root, r2.state_root);
    }

    #[test]
    fn test_atomic_rollback_on_failure() {
        let (mut exec, mut state) = setup();
        let root_before = state.root();
        let bal_before = state.get(&addr(1)).balance;
        // This will fail (bad nonce).
        let tx = transfer_tx(1, 2, 1000, 99);
        let receipt = exec.execute(&mut state, &tx, 0);
        assert!(!receipt.success);
        // State fully rolled back.
        assert_eq!(state.get(&addr(1)).balance, bal_before);
        assert_eq!(state.root(), root_before);
    }

    #[test]
    fn test_inference_commit() {
        let (mut exec, mut state) = setup();
        let tx = Transaction::new(
            addr(1),
            0,
            TxKind::InferenceCommit {
                model_id: ModelId([1u8; 32]),
                activation_root: [2u8; 32],
            },
        );
        let receipt = exec.execute(&mut state, &tx, 0);
        assert!(receipt.success);
        assert_eq!(receipt.gas_used, 50_000);
    }
}
