//! Fuzz testing harness — property-based testing for chain state.
//!
//! Generates random sequences of transactions and verifies that all
//! protocol invariants hold after every block execution. Uses a
//! deterministic PRNG seeded per test case for reproducibility.
//!
//! Properties tested:
//! 1. **No panics** — arbitrary input never causes an unwind
//! 2. **Nonce monotonicity** — nonces only increase
//! 3. **Replay protection** — duplicate nonces always rejected
//! 4. **Deterministic state root** — same txs in same order = same root
//! 5. **Self-transfer safety** — self-sends don't inflate balance
//! 6. **Insufficient balance safety** — near-empty accounts stay sane
//! 7. **Mixed validity streams** — invalid txs don't corrupt state
//! 8. **Large-scale stress** — 1000 txs, no crash
//! 9. **Rapid seed sweep** — 500 seeds, broad coverage

use prova_chain::types::*;
use prova_chain::state::StateTrie;
use prova_chain::executor::{Transaction, TxKind, Executor};

/// Simple deterministic PRNG (xorshift64) for reproducible fuzz runs.
struct Rng(u64);

impl Rng {
    fn new(seed: u64) -> Self {
        Self(if seed == 0 { 1 } else { seed })
    }

    fn next(&mut self) -> u64 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        self.0
    }

    fn range(&mut self, lo: u64, hi: u64) -> u64 {
        if lo >= hi { return lo; }
        lo + self.next() % (hi - lo)
    }

    fn pick<'a, T>(&mut self, items: &'a [T]) -> &'a T {
        let idx = self.next() as usize % items.len();
        &items[idx]
    }

    fn bytes20(&mut self) -> [u8; 20] {
        let mut b = [0u8; 20];
        for chunk in b.chunks_mut(8) {
            let v = self.next().to_le_bytes();
            chunk.copy_from_slice(&v[..chunk.len()]);
        }
        b
    }

    fn bytes32(&mut self) -> [u8; 32] {
        let mut b = [0u8; 32];
        for chunk in b.chunks_mut(8) {
            let v = self.next().to_le_bytes();
            chunk.copy_from_slice(&v[..chunk.len()]);
        }
        b
    }
}

/// A fuzzable chain environment bundling all state components.
struct FuzzEnv {
    state: StateTrie,
    executor: Executor,
    accounts: Vec<Address>,
    nonces: Vec<u64>,
    rng: Rng,
    treasury: Address,
    tx_counter: usize,
}

impl FuzzEnv {
    fn new(seed: u64, num_accounts: usize) -> Self {
        let mut rng = Rng::new(seed);
        let mut state = StateTrie::new();
        let mut accounts = Vec::with_capacity(num_accounts);
        let initial_balance: u128 = 1_000_000;

        for _ in 0..num_accounts {
            let addr = Address::new(rng.bytes20());
            state.set_balance(addr, initial_balance);
            accounts.push(addr);
        }

        let treasury = Address::test(0xFF);
        state.set_balance(treasury, 0);

        FuzzEnv {
            state,
            executor: Executor::new(treasury),
            accounts,
            nonces: vec![0; num_accounts],
            rng,
            treasury,
            tx_counter: 0,
        }
    }

    /// Generate a random transaction kind.
    fn random_tx(&mut self) -> Transaction {
        let idx = self.rng.next() as usize % self.accounts.len();
        let sender = self.accounts[idx];
        let nonce = self.nonces[idx];
        let kind_roll = self.rng.range(0, 7);

        let kind = match kind_roll {
            0 => {
                let to = *self.rng.pick(&self.accounts);
                let amount = self.rng.range(0, 10_000) as u128;
                TxKind::Transfer { to, amount }
            }
            1 => {
                let amount = self.rng.range(0, 5_000) as u128;
                TxKind::Stake { amount }
            }
            2 => {
                let amount = self.rng.range(0, 5_000) as u128;
                TxKind::Unstake { amount }
            }
            3 => {
                let model_id = ModelId(self.rng.bytes32());
                let weight_hash = self.rng.bytes32();
                TxKind::RegisterModel {
                    model_id,
                    weight_hash,
                    arch_group: "fuzz-arch".into(),
                }
            }
            4 => {
                let model_id = ModelId(self.rng.bytes32());
                let activation_root = self.rng.bytes32();
                TxKind::InferenceCommit {
                    model_id,
                    activation_root,
                }
            }
            5 => TxKind::ClaimReward,
            _ => {
                let provider = *self.rng.pick(&self.accounts);
                let amount = self.rng.range(0, 2_000) as u128;
                TxKind::PayInferenceFee { provider, amount }
            }
        };

        Transaction::new(sender, nonce, kind)
    }

    /// Generate a random tx, advance nonce, execute it. Returns success.
    fn step(&mut self) -> bool {
        let tx = self.random_tx();
        let idx = self.accounts.iter().position(|a| *a == tx.sender).unwrap();
        self.nonces[idx] += 1;
        let ti = self.tx_counter;
        self.tx_counter += 1;
        let receipt = self.executor.execute(&mut self.state, &tx, ti);
        receipt.success
    }

    /// Execute a specific tx (without advancing nonce).
    fn exec_tx(&mut self, tx: &Transaction) -> bool {
        let ti = self.tx_counter;
        self.tx_counter += 1;
        let receipt = self.executor.execute(&mut self.state, tx, ti);
        receipt.success
    }
}

// ─── Tests ───────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    /// Property 1: Random transaction sequences never panic.
    #[test]
    fn fuzz_no_panics_100_seeds() {
        for seed in 1..=100 {
            let mut env = FuzzEnv::new(seed, 5);
            for _ in 0..50 {
                env.step();
            }
        }
    }

    /// Property 2: Nonces are monotonically increasing per account.
    #[test]
    fn fuzz_nonce_monotonicity() {
        for seed in 300..=310 {
            let mut env = FuzzEnv::new(seed, 4);
            let mut last_nonces = vec![0u64; env.accounts.len()];

            for _ in 0..60 {
                let tx = env.random_tx();
                let sender_idx = env.accounts.iter().position(|a| *a == tx.sender).unwrap();
                env.nonces[sender_idx] += 1;
                let ti = env.tx_counter;
                env.tx_counter += 1;
                let receipt = env.executor.execute(&mut env.state, &tx, ti);
                if receipt.success {
                    let current = env.state.expected_nonce(&env.accounts[sender_idx]);
                    assert!(
                        current >= last_nonces[sender_idx],
                        "seed={seed}: nonce went backwards for account {sender_idx}"
                    );
                    last_nonces[sender_idx] = current;
                }
            }
        }
    }

    /// Property 3: Replay protection — same sender+nonce always rejected second time.
    #[test]
    fn fuzz_replay_protection() {
        for seed in 400..=410 {
            let mut env = FuzzEnv::new(seed, 3);
            for round in 0..30 {
                let tx = env.random_tx();
                let idx = env.accounts.iter().position(|a| *a == tx.sender).unwrap();
                env.nonces[idx] += 1;

                // Execute first
                let ti = env.tx_counter;
                env.tx_counter += 1;
                let _r1 = env.executor.execute(&mut env.state, &tx, ti);

                // Replay with same nonce
                let replay = Transaction::new(tx.sender, tx.nonce, tx.kind.clone());
                let ti2 = env.tx_counter;
                env.tx_counter += 1;
                let r2 = env.executor.execute(&mut env.state, &replay, ti2);
                assert!(
                    !r2.success,
                    "seed={seed} round={round}: replay tx accepted — nonce reuse!"
                );
            }
        }
    }

    /// Property 4: Deterministic state root — same txs produce same root.
    #[test]
    fn fuzz_deterministic_root() {
        for seed in 500..=510 {
            let root1 = {
                let mut env = FuzzEnv::new(seed, 4);
                for _ in 0..20 { env.step(); }
                env.state.root()
            };
            let root2 = {
                let mut env = FuzzEnv::new(seed, 4);
                for _ in 0..20 { env.step(); }
                env.state.root()
            };
            assert_eq!(
                root1, root2,
                "seed={seed}: same transactions produced different state roots!"
            );
        }
    }

    /// Property 5: Self-transfer doesn't inflate balance.
    #[test]
    fn fuzz_self_transfer_noop() {
        for seed in 1000..=1010 {
            let mut env = FuzzEnv::new(seed, 3);
            let addr = env.accounts[0];
            let before = env.state.get(&addr).balance;
            let nonce = env.nonces[0];
            env.nonces[0] += 1;

            let tx = Transaction::new(addr, nonce, TxKind::Transfer { to: addr, amount: 500 });
            env.exec_tx(&tx);

            let after = env.state.get(&addr).balance;
            assert!(after <= before, "seed={seed}: self-transfer increased balance");
        }
    }

    /// Property 6: Insufficient balance never wraps around.
    #[test]
    fn fuzz_insufficient_balance_safety() {
        for seed in 800..=810 {
            let mut env = FuzzEnv::new(seed, 3);
            // Drain accounts
            for addr in &env.accounts {
                env.state.set_balance(*addr, 1);
            }
            for _ in 0..50 {
                env.step();
            }
            for addr in &env.accounts {
                let bal = env.state.get(addr).balance;
                assert!(bal < u128::MAX / 2, "seed={seed}: suspiciously large balance — possible underflow");
            }
        }
    }

    /// Property 7: Mixed valid/invalid tx streams don't corrupt state.
    #[test]
    fn fuzz_mixed_validity_stream() {
        for seed in 900..=910 {
            let mut env = FuzzEnv::new(seed, 5);
            for i in 0..40 {
                if i % 3 == 0 {
                    // Wrong nonce — should fail
                    let tx = Transaction::new(
                        env.accounts[0],
                        999_999,
                        TxKind::Transfer { to: env.accounts[1], amount: 100 },
                    );
                    let r = env.exec_tx(&tx);
                    assert!(!r, "Wrong nonce accepted at i={i}");
                } else {
                    env.step();
                }
            }
        }
    }

    /// Property 8: Large-scale fuzz — 1000 txs across 20 accounts, no crash.
    #[test]
    fn fuzz_large_scale_stress() {
        let mut env = FuzzEnv::new(7777, 20);
        for _ in 0..1000 {
            env.step();
        }
    }

    /// Property 9: Rapid seed sweep — 500 seeds, 10 txs each, no crash.
    #[test]
    fn fuzz_rapid_sweep_500() {
        for seed in 1..=500 {
            let mut env = FuzzEnv::new(seed * 31337, 3);
            for _ in 0..10 {
                env.step();
            }
        }
    }

    /// Property 10: Stake then unstake roundtrip.
    #[test]
    fn fuzz_stake_unstake_roundtrip() {
        let mut env = FuzzEnv::new(1100, 2);
        let addr = env.accounts[0];
        let before = env.state.get(&addr).balance;

        let tx1 = Transaction::new(addr, 0, TxKind::Stake { amount: 10_000 });
        env.nonces[0] = 1;
        env.exec_tx(&tx1);

        let tx2 = Transaction::new(addr, 1, TxKind::Unstake { amount: 10_000 });
        env.nonces[0] = 2;
        env.exec_tx(&tx2);

        let after = env.state.get(&addr).balance;
        assert!(after <= before, "stake→unstake increased balance");
    }

    /// Property 11: Executor success count matches observed successes.
    #[test]
    fn fuzz_executor_accounting() {
        let mut env = FuzzEnv::new(1200, 5);
        let mut observed_success = 0u64;
        for _ in 0..100 {
            if env.step() {
                observed_success += 1;
            }
        }
        assert_eq!(
            env.executor.success_count, observed_success,
            "Executor success count mismatch"
        );
    }

    /// Property 12: Total tx count always equals steps taken.
    #[test]
    fn fuzz_tx_count_tracking() {
        let mut env = FuzzEnv::new(1300, 4);
        let n = 77;
        for _ in 0..n {
            env.step();
        }
        assert_eq!(env.executor.tx_count, n, "Tx count mismatch");
    }
}
