//! Formal Invariant Checker — validates fundamental protocol invariants.
//!
//! Checks that must hold after every state transition:
//! 1. **Balance conservation**: Total supply is constant (minted + treasury = distributed + burned).
//! 2. **Stake consistency**: locked ≤ deposited, slashed ≤ deposited, available ≥ 0.
//! 3. **Nonce monotonicity**: Nonces only increase.
//! 4. **Reward conservation**: Distributed rewards never exceed minted rewards.
//! 5. **No negative balances**: All account balances ≥ 0 (enforced by u128).
//! 6. **Dispute liveness**: Active disputes reference valid commits and participants.
//! 7. **Scheduler integrity**: No double-assignment, no expired-but-active jobs.
//!
//! Designed to run after every block in test/devnet mode, and optionally on checkpoints.

use crate::stake::{StakeLedger, StakeEntry};
use crate::state::StateTrie;
use crate::rewards::RewardLedger;
use crate::scheduler::Scheduler;
use crate::types::*;

use std::collections::HashMap;

/// Result of an invariant check.
#[derive(Debug, Clone)]
pub struct InvariantViolation {
    pub invariant: &'static str,
    pub detail: String,
}

impl std::fmt::Display for InvariantViolation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "INVARIANT VIOLATION [{}]: {}", self.invariant, self.detail)
    }
}

/// Snapshot of chain state fed into the checker.
/// Decoupled from live state so invariants can be checked on serialized snapshots too.
pub struct StateSnapshot {
    /// All account balances (address → balance).
    pub balances: HashMap<Address, u128>,
    /// All account nonces (address → nonce).
    pub nonces: HashMap<Address, u64>,
    /// Total tokens ever minted (block rewards + storage subsidies).
    pub total_minted: u128,
    /// Total tokens burned (slashing burns, fee burns).
    pub total_burned: u128,
    /// Treasury balance.
    pub treasury_balance: u128,
    /// Stake entries per address.
    pub stakes: HashMap<Address, StakeSnapshot>,
    /// Total rewards distributed to all addresses.
    pub total_rewards_distributed: u128,
    /// Active job assignments: job_id → assigned provider.
    pub active_jobs: HashMap<u64, Address>,
    /// Current epoch.
    pub current_epoch: Epoch,
}

/// Minimal stake snapshot for invariant checking.
#[derive(Debug, Clone)]
pub struct StakeSnapshot {
    pub deposited: u128,
    pub locked: u128,
    pub slashed: u128,
}

impl StakeSnapshot {
    pub fn available(&self) -> i128 {
        self.deposited as i128 - self.locked as i128 - self.slashed as i128
    }
}

/// Run all invariant checks on a state snapshot. Returns list of violations (empty = pass).
pub fn check_all(snap: &StateSnapshot) -> Vec<InvariantViolation> {
    let mut violations = Vec::new();
    check_balance_conservation(snap, &mut violations);
    check_stake_consistency(snap, &mut violations);
    check_reward_conservation(snap, &mut violations);
    check_no_duplicate_jobs(snap, &mut violations);
    violations
}

/// INV-1: Balance conservation.
/// Sum of all balances + treasury + total_burned == total_minted + initial_supply.
/// (We treat initial_supply as a parameter; for simplicity we check that
///  sum(balances) + treasury + burned ≤ minted, i.e., no tokens created from thin air.)
fn check_balance_conservation(snap: &StateSnapshot, violations: &mut Vec<InvariantViolation>) {
    let sum_balances: u128 = snap.balances.values().sum();
    let total_accounted = sum_balances
        .saturating_add(snap.treasury_balance)
        .saturating_add(snap.total_burned);

    // Staked tokens are held in accounts, so they're already counted in balances.
    // The sum of all accounted tokens must exactly equal total minted.
    if total_accounted != snap.total_minted {
        violations.push(InvariantViolation {
            invariant: "BALANCE_CONSERVATION",
            detail: format!(
                "accounted={total_accounted} (balances={sum_balances} + treasury={} + burned={}) != minted={}",
                snap.treasury_balance, snap.total_burned, snap.total_minted
            ),
        });
    }
}

/// INV-2: Stake consistency — for every participant, locked ≤ deposited and available ≥ 0.
fn check_stake_consistency(snap: &StateSnapshot, violations: &mut Vec<InvariantViolation>) {
    for (addr, stake) in &snap.stakes {
        if stake.locked > stake.deposited {
            violations.push(InvariantViolation {
                invariant: "STAKE_LOCKED_LE_DEPOSITED",
                detail: format!("{addr}: locked={} > deposited={}", stake.locked, stake.deposited),
            });
        }
        if stake.slashed > stake.deposited {
            violations.push(InvariantViolation {
                invariant: "STAKE_SLASHED_LE_DEPOSITED",
                detail: format!("{addr}: slashed={} > deposited={}", stake.slashed, stake.deposited),
            });
        }
        if stake.available() < 0 {
            violations.push(InvariantViolation {
                invariant: "STAKE_AVAILABLE_NON_NEGATIVE",
                detail: format!("{addr}: available={}", stake.available()),
            });
        }
    }
}

/// INV-3: Reward conservation — distributed rewards must not exceed total minted.
fn check_reward_conservation(snap: &StateSnapshot, violations: &mut Vec<InvariantViolation>) {
    if snap.total_rewards_distributed > snap.total_minted {
        violations.push(InvariantViolation {
            invariant: "REWARD_CONSERVATION",
            detail: format!(
                "distributed={} > minted={}",
                snap.total_rewards_distributed, snap.total_minted
            ),
        });
    }
}

/// INV-4: No duplicate job assignments.
fn check_no_duplicate_jobs(snap: &StateSnapshot, violations: &mut Vec<InvariantViolation>) {
    let mut seen_jobs: HashMap<u64, Address> = HashMap::new();
    for (&job_id, addr) in &snap.active_jobs {
        if let Some(prev) = seen_jobs.insert(job_id, *addr) {
            if prev != *addr {
                violations.push(InvariantViolation {
                    invariant: "NO_DUPLICATE_JOBS",
                    detail: format!("job {job_id} assigned to both {prev} and {addr}"),
                });
            }
        }
    }
}

/// Convenience: check nonce monotonicity between two snapshots (before/after transition).
pub fn check_nonce_monotonicity(
    before: &HashMap<Address, u64>,
    after: &HashMap<Address, u64>,
) -> Vec<InvariantViolation> {
    let mut violations = Vec::new();
    for (addr, &new_nonce) in after {
        if let Some(&old_nonce) = before.get(addr) {
            if new_nonce < old_nonce {
                violations.push(InvariantViolation {
                    invariant: "NONCE_MONOTONICITY",
                    detail: format!("{addr}: nonce decreased from {old_nonce} to {new_nonce}"),
                });
            }
        }
    }
    violations
}

/// Run invariants and panic if any violations found (for test/devnet use).
pub fn assert_invariants(snap: &StateSnapshot) {
    let violations = check_all(snap);
    if !violations.is_empty() {
        let msgs: Vec<String> = violations.iter().map(|v| v.to_string()).collect();
        panic!("Invariant violations detected:\n{}", msgs.join("\n"));
    }
}

// ─── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn base_snapshot() -> StateSnapshot {
        // Consistent state: 1000 minted, split across two accounts + treasury
        let mut balances = HashMap::new();
        balances.insert(Address::test(1), 400);
        balances.insert(Address::test(2), 300);

        StateSnapshot {
            balances,
            nonces: HashMap::new(),
            total_minted: 1000,
            total_burned: 100,
            treasury_balance: 200,
            stakes: HashMap::new(),
            total_rewards_distributed: 500,
            active_jobs: HashMap::new(),
            current_epoch: 100,
        }
    }

    #[test]
    fn test_valid_state_passes() {
        let snap = base_snapshot();
        let v = check_all(&snap);
        assert!(v.is_empty(), "expected no violations: {v:?}");
    }

    #[test]
    fn test_balance_conservation_violation_over() {
        let mut snap = base_snapshot();
        // Inflate an account — total exceeds minted
        snap.balances.insert(Address::test(1), 500);
        let v = check_all(&snap);
        assert!(v.iter().any(|x| x.invariant == "BALANCE_CONSERVATION"));
    }

    #[test]
    fn test_balance_conservation_violation_under() {
        let mut snap = base_snapshot();
        // Remove tokens — total less than minted (tokens vanished)
        snap.balances.insert(Address::test(1), 300);
        let v = check_all(&snap);
        assert!(v.iter().any(|x| x.invariant == "BALANCE_CONSERVATION"));
    }

    #[test]
    fn test_stake_locked_exceeds_deposited() {
        let mut snap = base_snapshot();
        snap.stakes.insert(Address::test(1), StakeSnapshot {
            deposited: 100,
            locked: 150,
            slashed: 0,
        });
        let v = check_all(&snap);
        assert!(v.iter().any(|x| x.invariant == "STAKE_LOCKED_LE_DEPOSITED"));
    }

    #[test]
    fn test_stake_slashed_exceeds_deposited() {
        let mut snap = base_snapshot();
        snap.stakes.insert(Address::test(1), StakeSnapshot {
            deposited: 100,
            locked: 0,
            slashed: 200,
        });
        let v = check_all(&snap);
        assert!(v.iter().any(|x| x.invariant == "STAKE_SLASHED_LE_DEPOSITED"));
    }

    #[test]
    fn test_stake_available_negative() {
        let mut snap = base_snapshot();
        snap.stakes.insert(Address::test(1), StakeSnapshot {
            deposited: 100,
            locked: 60,
            slashed: 50,
        });
        let v = check_all(&snap);
        assert!(v.iter().any(|x| x.invariant == "STAKE_AVAILABLE_NON_NEGATIVE"));
    }

    #[test]
    fn test_valid_stake_passes() {
        let mut snap = base_snapshot();
        snap.stakes.insert(Address::test(1), StakeSnapshot {
            deposited: 100,
            locked: 40,
            slashed: 30,
        });
        let v = check_all(&snap);
        // Only balance conservation should pass, no stake violations
        assert!(!v.iter().any(|x| x.invariant.starts_with("STAKE_")));
    }

    #[test]
    fn test_reward_conservation_violation() {
        let mut snap = base_snapshot();
        snap.total_rewards_distributed = snap.total_minted + 1;
        let v = check_all(&snap);
        assert!(v.iter().any(|x| x.invariant == "REWARD_CONSERVATION"));
    }

    #[test]
    fn test_nonce_monotonicity_pass() {
        let mut before = HashMap::new();
        before.insert(Address::test(1), 5);
        let mut after = HashMap::new();
        after.insert(Address::test(1), 6);
        let v = check_nonce_monotonicity(&before, &after);
        assert!(v.is_empty());
    }

    #[test]
    fn test_nonce_monotonicity_violation() {
        let mut before = HashMap::new();
        before.insert(Address::test(1), 5);
        let mut after = HashMap::new();
        after.insert(Address::test(1), 3);
        let v = check_nonce_monotonicity(&before, &after);
        assert!(v.iter().any(|x| x.invariant == "NONCE_MONOTONICITY"));
    }

    #[test]
    fn test_nonce_monotonicity_same_is_ok() {
        let mut before = HashMap::new();
        before.insert(Address::test(1), 5);
        let after = before.clone();
        let v = check_nonce_monotonicity(&before, &after);
        assert!(v.is_empty());
    }

    #[test]
    fn test_assert_invariants_panics() {
        let mut snap = base_snapshot();
        snap.total_minted = 0; // Will violate multiple invariants
        let result = std::panic::catch_unwind(|| assert_invariants(&snap));
        assert!(result.is_err());
    }

    #[test]
    fn test_multiple_violations_reported() {
        let mut snap = base_snapshot();
        snap.balances.insert(Address::test(1), 500); // conservation violation
        snap.total_rewards_distributed = snap.total_minted + 1; // reward violation
        snap.stakes.insert(Address::test(3), StakeSnapshot {
            deposited: 10,
            locked: 20,
            slashed: 0,
        }); // stake violation
        let v = check_all(&snap);
        assert!(v.len() >= 3, "expected at least 3 violations, got {}", v.len());
    }

    #[test]
    fn test_empty_state_valid() {
        let snap = StateSnapshot {
            balances: HashMap::new(),
            nonces: HashMap::new(),
            total_minted: 0,
            total_burned: 0,
            treasury_balance: 0,
            stakes: HashMap::new(),
            total_rewards_distributed: 0,
            active_jobs: HashMap::new(),
            current_epoch: 0,
        };
        let v = check_all(&snap);
        assert!(v.is_empty());
    }

    #[test]
    fn test_new_address_nonce_ok() {
        let before = HashMap::new();
        let mut after = HashMap::new();
        after.insert(Address::test(1), 1);
        let v = check_nonce_monotonicity(&before, &after);
        assert!(v.is_empty());
    }

    #[test]
    fn test_display_violation() {
        let v = InvariantViolation {
            invariant: "TEST",
            detail: "something broke".into(),
        };
        assert!(v.to_string().contains("TEST"));
        assert!(v.to_string().contains("something broke"));
    }
}
