//! State Migration System — versioned state transformations for protocol upgrades.
//!
//! When a protocol upgrade changes the state format (e.g., new account fields,
//! renamed storage keys, rebalanced economics), migrations run at the activation
//! epoch to deterministically transform chain state.
//!
//! Features:
//! - Named, versioned migrations with pre/post validation
//! - Dry-run mode (preview changes without committing)
//! - Rollback support (undo migration if activation is reverted)
//! - Migration dependency graph (ordered execution)
//! - Progress tracking for long-running migrations
//! - Deterministic execution (same input → same output across all nodes)

use crate::types::{Address, Hash};
use std::collections::{BTreeMap, HashMap, HashSet};

// ── Types ──────────────────────────────────────────────────────────────

/// Unique migration identifier: "<upgrade_name>/<sequence>".
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct MigrationId {
    pub upgrade: String,
    pub sequence: u32,
}

impl MigrationId {
    pub fn new(upgrade: &str, sequence: u32) -> Self {
        Self {
            upgrade: upgrade.to_string(),
            sequence,
        }
    }

    pub fn canonical(&self) -> String {
        format!("{}/{:04}", self.upgrade, self.sequence)
    }
}

/// State version — incremented after each migration.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct StateVersion(pub u64);

impl StateVersion {
    pub fn genesis() -> Self {
        Self(0)
    }
    pub fn next(&self) -> Self {
        Self(self.0 + 1)
    }
}

/// Simplified account state for migration purposes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MigrationAccountState {
    pub balance: u128,
    pub nonce: u64,
    pub storage: BTreeMap<String, Vec<u8>>,
    pub metadata: BTreeMap<String, String>,
}

impl MigrationAccountState {
    pub fn new(balance: u128) -> Self {
        Self {
            balance,
            nonce: 0,
            storage: BTreeMap::new(),
            metadata: BTreeMap::new(),
        }
    }
}

/// The full state snapshot that migrations operate on.
#[derive(Debug, Clone)]
pub struct MigrationState {
    pub version: StateVersion,
    pub accounts: BTreeMap<Address, MigrationAccountState>,
    /// Global parameters (chain-wide config values).
    pub params: BTreeMap<String, Vec<u8>>,
    /// History of applied migrations.
    pub applied: Vec<MigrationRecord>,
}

impl MigrationState {
    pub fn new() -> Self {
        Self {
            version: StateVersion::genesis(),
            accounts: BTreeMap::new(),
            params: BTreeMap::new(),
            applied: Vec::new(),
        }
    }

    pub fn state_root(&self) -> Hash {
        use std::hash::{Hash as _, Hasher};
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        self.version.0.hash(&mut hasher);
        for (addr, acct) in &self.accounts {
            addr.hash(&mut hasher);
            acct.balance.hash(&mut hasher);
            acct.nonce.hash(&mut hasher);
        }
        for (k, v) in &self.params {
            k.hash(&mut hasher);
            v.hash(&mut hasher);
        }
        let h = hasher.finish();
        let mut hash = [0u8; 32];
        hash[..8].copy_from_slice(&h.to_be_bytes());
        hash
    }
}

/// Record of an applied migration.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MigrationRecord {
    pub id: MigrationId,
    pub from_version: StateVersion,
    pub to_version: StateVersion,
    pub pre_state_root: Hash,
    pub post_state_root: Hash,
    pub accounts_modified: u64,
    pub params_modified: u64,
}

// ── Migration Trait ────────────────────────────────────────────────────

/// Describes a single state migration.
pub trait Migration {
    fn id(&self) -> MigrationId;
    /// Human-readable description.
    fn description(&self) -> &str;
    /// Migrations that must run before this one.
    fn depends_on(&self) -> Vec<MigrationId> {
        vec![]
    }
    /// Minimum state version required.
    fn requires_version(&self) -> StateVersion {
        StateVersion::genesis()
    }
    /// Target state version after this migration.
    fn target_version(&self) -> StateVersion;

    /// Validate preconditions. Returns Ok(()) or an error description.
    fn validate_pre(&self, state: &MigrationState) -> Result<(), String>;
    /// Apply the migration (mutate state).
    fn apply(&self, state: &mut MigrationState) -> Result<MigrationEffect, String>;
    /// Validate postconditions.
    fn validate_post(&self, state: &MigrationState) -> Result<(), String>;
    /// Undo the migration (if possible).
    fn rollback(&self, state: &mut MigrationState) -> Result<(), String>;
}

/// Summary of what a migration changed.
#[derive(Debug, Clone, Default)]
pub struct MigrationEffect {
    pub accounts_modified: u64,
    pub accounts_created: u64,
    pub accounts_deleted: u64,
    pub params_modified: u64,
    pub description: String,
}

// ── Migration Runner ───────────────────────────────────────────────────

/// Execution mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RunMode {
    /// Apply changes to state.
    Live,
    /// Preview changes without modifying state.
    DryRun,
}

/// Progress callback info.
#[derive(Debug, Clone)]
pub struct MigrationProgress {
    pub migration_id: MigrationId,
    pub step: &'static str,
    pub detail: String,
}

/// Result of running a migration plan.
#[derive(Debug, Clone)]
pub struct PlanResult {
    pub mode: RunMode,
    pub migrations_run: usize,
    pub final_version: StateVersion,
    pub effects: Vec<(MigrationId, MigrationEffect)>,
    pub errors: Vec<(MigrationId, String)>,
}

impl PlanResult {
    pub fn success(&self) -> bool {
        self.errors.is_empty()
    }
}

/// The migration runner: registers migrations and executes them in order.
pub struct MigrationRunner {
    migrations: Vec<Box<dyn Migration>>,
    progress_log: Vec<MigrationProgress>,
}

impl MigrationRunner {
    pub fn new() -> Self {
        Self {
            migrations: Vec::new(),
            progress_log: Vec::new(),
        }
    }

    pub fn register(&mut self, m: Box<dyn Migration>) {
        self.migrations.push(m);
    }

    pub fn registered_count(&self) -> usize {
        self.migrations.len()
    }

    pub fn progress_log(&self) -> &[MigrationProgress] {
        &self.progress_log
    }

    /// Topologically sort migrations respecting dependencies.
    fn resolve_order(&self) -> Result<Vec<usize>, String> {
        let id_to_idx: HashMap<MigrationId, usize> = self
            .migrations
            .iter()
            .enumerate()
            .map(|(i, m)| (m.id(), i))
            .collect();

        // Kahn's algorithm
        let n = self.migrations.len();
        let mut in_degree = vec![0usize; n];
        let mut adj: Vec<Vec<usize>> = vec![vec![]; n];

        for (i, m) in self.migrations.iter().enumerate() {
            for dep in m.depends_on() {
                if let Some(&j) = id_to_idx.get(&dep) {
                    adj[j].push(i);
                    in_degree[i] += 1;
                } else {
                    return Err(format!(
                        "Migration {} depends on unknown {}",
                        m.id().canonical(),
                        dep.canonical()
                    ));
                }
            }
        }

        let mut queue: Vec<usize> = (0..n).filter(|&i| in_degree[i] == 0).collect();
        // Sort queue for determinism
        queue.sort_by(|a, b| self.migrations[*a].id().cmp(&self.migrations[*b].id()));
        let mut order = Vec::with_capacity(n);

        while let Some(idx) = queue.pop() {
            order.push(idx);
            let mut next = Vec::new();
            for &neighbor in &adj[idx] {
                in_degree[neighbor] -= 1;
                if in_degree[neighbor] == 0 {
                    next.push(neighbor);
                }
            }
            next.sort_by(|a, b| self.migrations[*a].id().cmp(&self.migrations[*b].id()));
            // Push in reverse so smallest pops first
            for n in next.into_iter().rev() {
                queue.push(n);
            }
        }

        if order.len() != n {
            return Err("Circular dependency detected in migrations".to_string());
        }
        Ok(order)
    }

    /// Run all pending migrations on the given state.
    pub fn run(&mut self, state: &mut MigrationState, mode: RunMode) -> PlanResult {
        let order = match self.resolve_order() {
            Ok(o) => o,
            Err(e) => {
                return PlanResult {
                    mode,
                    migrations_run: 0,
                    final_version: state.version,
                    effects: vec![],
                    errors: vec![(MigrationId::new("__resolve__", 0), e)],
                }
            }
        };

        let already_applied: HashSet<MigrationId> =
            state.applied.iter().map(|r| r.id.clone()).collect();

        let mut effects = Vec::new();
        let mut errors = Vec::new();
        let mut count = 0;

        // For dry-run, clone state
        let work_state: &mut MigrationState = if mode == RunMode::DryRun {
            // We'll track on a clone but report against the original
            // This is a simplification — real impl would use CoW
            state
        } else {
            state
        };

        let mut dry_clone = if mode == RunMode::DryRun {
            Some(work_state.clone())
        } else {
            None
        };

        let target = if mode == RunMode::DryRun {
            dry_clone.as_mut().unwrap()
        } else {
            work_state
        };

        for idx in order {
            let m = &self.migrations[idx];
            let mid = m.id();

            if already_applied.contains(&mid) {
                continue;
            }

            if target.version < m.requires_version() {
                continue; // Not yet eligible
            }

            // Pre-validate
            self.progress_log.push(MigrationProgress {
                migration_id: mid.clone(),
                step: "pre_validate",
                detail: m.description().to_string(),
            });

            if let Err(e) = m.validate_pre(target) {
                errors.push((mid.clone(), format!("pre-validation failed: {}", e)));
                break; // Stop on first error
            }

            let pre_root = target.state_root();
            let from_ver = target.version;

            // Apply
            self.progress_log.push(MigrationProgress {
                migration_id: mid.clone(),
                step: "apply",
                detail: format!("applying from v{}", from_ver.0),
            });

            match m.apply(target) {
                Ok(effect) => {
                    target.version = m.target_version();
                    let post_root = target.state_root();

                    // Post-validate
                    if let Err(e) = m.validate_post(target) {
                        errors.push((mid.clone(), format!("post-validation failed: {}", e)));
                        break;
                    }

                    target.applied.push(MigrationRecord {
                        id: mid.clone(),
                        from_version: from_ver,
                        to_version: target.version,
                        pre_state_root: pre_root,
                        post_state_root: post_root,
                        accounts_modified: effect.accounts_modified,
                        params_modified: effect.params_modified,
                    });

                    effects.push((mid.clone(), effect));
                    count += 1;

                    self.progress_log.push(MigrationProgress {
                        migration_id: mid,
                        step: "complete",
                        detail: format!("v{} → v{}", from_ver.0, target.version.0),
                    });
                }
                Err(e) => {
                    errors.push((mid, format!("apply failed: {}", e)));
                    break;
                }
            }
        }

        let final_version = target.version;

        PlanResult {
            mode,
            migrations_run: count,
            final_version,
            effects,
            errors,
        }
    }

    /// Rollback the last N applied migrations.
    pub fn rollback_last(
        &mut self,
        state: &mut MigrationState,
        count: usize,
    ) -> Result<usize, String> {
        let id_map: HashMap<MigrationId, usize> = self
            .migrations
            .iter()
            .enumerate()
            .map(|(i, m)| (m.id(), i))
            .collect();

        let to_rollback: Vec<MigrationRecord> =
            state.applied.iter().rev().take(count).cloned().collect();

        let mut rolled = 0;
        for record in &to_rollback {
            let idx = id_map.get(&record.id).ok_or_else(|| {
                format!("Migration {} not found in registry", record.id.canonical())
            })?;

            self.migrations[*idx].rollback(state)?;
            state.version = record.from_version;
            state.applied.pop();
            rolled += 1;
        }

        Ok(rolled)
    }

    /// Check which migrations are pending for the current state.
    pub fn pending(&self, state: &MigrationState) -> Vec<MigrationId> {
        let applied: HashSet<MigrationId> = state.applied.iter().map(|r| r.id.clone()).collect();
        self.migrations
            .iter()
            .filter(|m| !applied.contains(&m.id()))
            .map(|m| m.id())
            .collect()
    }
}

// ── Built-in Example Migrations ────────────────────────────────────────

/// Migration: Add metadata field to all accounts.
pub struct AddAccountMetadata {
    pub field_name: String,
    pub default_value: String,
}

impl Migration for AddAccountMetadata {
    fn id(&self) -> MigrationId {
        MigrationId::new("v1_metadata", 1)
    }
    fn description(&self) -> &str {
        "Add metadata field to all accounts"
    }
    fn target_version(&self) -> StateVersion {
        StateVersion(1)
    }

    fn validate_pre(&self, state: &MigrationState) -> Result<(), String> {
        if state.version != StateVersion::genesis() {
            return Err(format!("Expected version 0, got {}", state.version.0));
        }
        Ok(())
    }

    fn apply(&self, state: &mut MigrationState) -> Result<MigrationEffect, String> {
        let mut modified = 0u64;
        for (_addr, acct) in state.accounts.iter_mut() {
            acct.metadata
                .insert(self.field_name.clone(), self.default_value.clone());
            modified += 1;
        }
        Ok(MigrationEffect {
            accounts_modified: modified,
            description: format!(
                "Added '{}' metadata to {} accounts",
                self.field_name, modified
            ),
            ..Default::default()
        })
    }

    fn validate_post(&self, state: &MigrationState) -> Result<(), String> {
        for (addr, acct) in &state.accounts {
            if !acct.metadata.contains_key(&self.field_name) {
                return Err(format!(
                    "Account {} missing metadata '{}'",
                    hex::encode(&addr.0),
                    self.field_name
                ));
            }
        }
        Ok(())
    }

    fn rollback(&self, state: &mut MigrationState) -> Result<(), String> {
        for (_addr, acct) in state.accounts.iter_mut() {
            acct.metadata.remove(&self.field_name);
        }
        Ok(())
    }
}

/// Migration: Rebalance token supply (multiply all balances by a factor).
pub struct RebalanceSupply {
    pub numerator: u128,
    pub denominator: u128,
    /// Snapshot of original balances for rollback.
    original_balances: std::cell::RefCell<BTreeMap<Address, u128>>,
}

impl RebalanceSupply {
    pub fn new(numerator: u128, denominator: u128) -> Self {
        Self {
            numerator,
            denominator,
            original_balances: std::cell::RefCell::new(BTreeMap::new()),
        }
    }
}

impl Migration for RebalanceSupply {
    fn id(&self) -> MigrationId {
        MigrationId::new("v2_rebalance", 1)
    }
    fn description(&self) -> &str {
        "Rebalance token supply by scaling all balances"
    }
    fn depends_on(&self) -> Vec<MigrationId> {
        vec![MigrationId::new("v1_metadata", 1)]
    }
    fn requires_version(&self) -> StateVersion {
        StateVersion(1)
    }
    fn target_version(&self) -> StateVersion {
        StateVersion(2)
    }

    fn validate_pre(&self, state: &MigrationState) -> Result<(), String> {
        if state.version.0 < 1 {
            return Err("Requires state version >= 1".to_string());
        }
        if self.denominator == 0 {
            return Err("Denominator cannot be zero".to_string());
        }
        Ok(())
    }

    fn apply(&self, state: &mut MigrationState) -> Result<MigrationEffect, String> {
        let mut originals = self.original_balances.borrow_mut();
        originals.clear();
        let mut modified = 0u64;
        let mut total_before = 0u128;
        let mut total_after = 0u128;

        for (addr, acct) in state.accounts.iter_mut() {
            originals.insert(*addr, acct.balance);
            total_before += acct.balance;
            acct.balance = acct
                .balance
                .checked_mul(self.numerator)
                .and_then(|v| v.checked_div(self.denominator))
                .ok_or_else(|| "Overflow during rebalance".to_string())?;
            total_after += acct.balance;
            modified += 1;
        }

        // Store total supply delta in params
        state.params.insert(
            "supply_delta".to_string(),
            (total_after as i128 - total_before as i128)
                .to_be_bytes()
                .to_vec(),
        );

        Ok(MigrationEffect {
            accounts_modified: modified,
            params_modified: 1,
            description: format!(
                "Rebalanced {} accounts: {}×/{}",
                modified, self.numerator, self.denominator
            ),
            ..Default::default()
        })
    }

    fn validate_post(&self, state: &MigrationState) -> Result<(), String> {
        // Verify no account has impossibly high balance
        for (addr, acct) in &state.accounts {
            if acct.balance > u128::MAX / 2 {
                return Err(format!(
                    "Account {} has suspiciously high balance",
                    hex::encode(&addr.0)
                ));
            }
        }
        Ok(())
    }

    fn rollback(&self, state: &mut MigrationState) -> Result<(), String> {
        let originals = self.original_balances.borrow();
        for (addr, acct) in state.accounts.iter_mut() {
            if let Some(&orig) = originals.get(addr) {
                acct.balance = orig;
            }
        }
        state.params.remove("supply_delta");
        Ok(())
    }
}

/// Migration: Add a new global parameter.
pub struct AddGlobalParam {
    pub key: String,
    pub value: Vec<u8>,
    pub required_version: StateVersion,
    pub new_version: StateVersion,
}

impl Migration for AddGlobalParam {
    fn id(&self) -> MigrationId {
        MigrationId::new("add_param", self.new_version.0 as u32)
    }
    fn description(&self) -> &str {
        "Add global parameter"
    }
    fn requires_version(&self) -> StateVersion {
        self.required_version
    }
    fn target_version(&self) -> StateVersion {
        self.new_version
    }

    fn validate_pre(&self, state: &MigrationState) -> Result<(), String> {
        if state.params.contains_key(&self.key) {
            return Err(format!("Parameter '{}' already exists", self.key));
        }
        Ok(())
    }

    fn apply(&self, state: &mut MigrationState) -> Result<MigrationEffect, String> {
        state.params.insert(self.key.clone(), self.value.clone());
        Ok(MigrationEffect {
            params_modified: 1,
            description: format!("Added param '{}'", self.key),
            ..Default::default()
        })
    }

    fn validate_post(&self, state: &MigrationState) -> Result<(), String> {
        if !state.params.contains_key(&self.key) {
            return Err(format!(
                "Parameter '{}' not found after migration",
                self.key
            ));
        }
        Ok(())
    }

    fn rollback(&self, state: &mut MigrationState) -> Result<(), String> {
        state.params.remove(&self.key);
        Ok(())
    }
}

// ── Hex helper ─────────────────────────────────────────────────────────

mod hex {
    pub fn encode(bytes: &[u8]) -> String {
        bytes.iter().map(|b| format!("{:02x}", b)).collect()
    }
}

// ── Tests ──────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn addr(n: u8) -> Address {
        Address::test(n)
    }

    fn sample_state(n_accounts: u8) -> MigrationState {
        let mut state = MigrationState::new();
        for i in 1..=n_accounts {
            state
                .accounts
                .insert(addr(i), MigrationAccountState::new(1000 * i as u128));
        }
        state
    }

    #[test]
    fn test_migration_id_canonical() {
        let id = MigrationId::new("v1_metadata", 1);
        assert_eq!(id.canonical(), "v1_metadata/0001");
    }

    #[test]
    fn test_state_version_ordering() {
        assert!(StateVersion(0) < StateVersion(1));
        assert_eq!(StateVersion(5).next(), StateVersion(6));
    }

    #[test]
    fn test_state_root_deterministic() {
        let s1 = sample_state(3);
        let s2 = sample_state(3);
        assert_eq!(s1.state_root(), s2.state_root());
    }

    #[test]
    fn test_state_root_changes_on_mutation() {
        let s1 = sample_state(3);
        let mut s2 = sample_state(3);
        s2.accounts.get_mut(&addr(1)).unwrap().balance += 1;
        assert_ne!(s1.state_root(), s2.state_root());
    }

    #[test]
    fn test_add_metadata_migration() {
        let mut state = sample_state(3);
        let m = AddAccountMetadata {
            field_name: "tier".to_string(),
            default_value: "basic".to_string(),
        };

        assert!(m.validate_pre(&state).is_ok());
        let effect = m.apply(&mut state).unwrap();
        assert_eq!(effect.accounts_modified, 3);
        assert!(m.validate_post(&state).is_ok());

        for (_, acct) in &state.accounts {
            assert_eq!(acct.metadata.get("tier").unwrap(), "basic");
        }
    }

    #[test]
    fn test_add_metadata_rollback() {
        let mut state = sample_state(2);
        let m = AddAccountMetadata {
            field_name: "tier".to_string(),
            default_value: "basic".to_string(),
        };
        m.apply(&mut state).unwrap();
        assert!(state
            .accounts
            .values()
            .all(|a| a.metadata.contains_key("tier")));

        m.rollback(&mut state).unwrap();
        assert!(state
            .accounts
            .values()
            .all(|a| !a.metadata.contains_key("tier")));
    }

    #[test]
    fn test_rebalance_migration() {
        let mut state = sample_state(3);
        state.version = StateVersion(1);
        let m = RebalanceSupply::new(3, 1); // Triple all balances

        assert!(m.validate_pre(&state).is_ok());
        let effect = m.apply(&mut state).unwrap();
        assert_eq!(effect.accounts_modified, 3);

        assert_eq!(state.accounts[&addr(1)].balance, 3000);
        assert_eq!(state.accounts[&addr(2)].balance, 6000);
        assert_eq!(state.accounts[&addr(3)].balance, 9000);
    }

    #[test]
    fn test_rebalance_rollback() {
        let mut state = sample_state(2);
        state.version = StateVersion(1);
        let m = RebalanceSupply::new(2, 1);
        m.apply(&mut state).unwrap();
        assert_eq!(state.accounts[&addr(1)].balance, 2000);

        m.rollback(&mut state).unwrap();
        assert_eq!(state.accounts[&addr(1)].balance, 1000);
        assert_eq!(state.accounts[&addr(2)].balance, 2000);
    }

    #[test]
    fn test_runner_single_migration() {
        let mut state = sample_state(5);
        let mut runner = MigrationRunner::new();
        runner.register(Box::new(AddAccountMetadata {
            field_name: "version".to_string(),
            default_value: "1".to_string(),
        }));

        let result = runner.run(&mut state, RunMode::Live);
        assert!(result.success());
        assert_eq!(result.migrations_run, 1);
        assert_eq!(result.final_version, StateVersion(1));
        assert_eq!(state.applied.len(), 1);
    }

    #[test]
    fn test_runner_dependency_chain() {
        let mut state = sample_state(3);
        let mut runner = MigrationRunner::new();

        // Register in reverse order — runner should sort by dependencies
        runner.register(Box::new(RebalanceSupply::new(2, 1)));
        runner.register(Box::new(AddAccountMetadata {
            field_name: "tier".to_string(),
            default_value: "basic".to_string(),
        }));

        let result = runner.run(&mut state, RunMode::Live);
        assert!(result.success());
        assert_eq!(result.migrations_run, 2);
        assert_eq!(result.final_version, StateVersion(2));

        // Metadata added first, then rebalance
        assert_eq!(state.applied[0].id, MigrationId::new("v1_metadata", 1));
        assert_eq!(state.applied[1].id, MigrationId::new("v2_rebalance", 1));
    }

    #[test]
    fn test_runner_skips_already_applied() {
        let mut state = sample_state(2);
        let mut runner = MigrationRunner::new();
        runner.register(Box::new(AddAccountMetadata {
            field_name: "tier".to_string(),
            default_value: "basic".to_string(),
        }));

        let r1 = runner.run(&mut state, RunMode::Live);
        assert_eq!(r1.migrations_run, 1);

        let r2 = runner.run(&mut state, RunMode::Live);
        assert_eq!(r2.migrations_run, 0); // Already applied
    }

    #[test]
    fn test_runner_dry_run() {
        let mut state = sample_state(3);
        let original_root = state.state_root();

        let mut runner = MigrationRunner::new();
        runner.register(Box::new(AddAccountMetadata {
            field_name: "x".to_string(),
            default_value: "y".to_string(),
        }));

        let result = runner.run(&mut state, RunMode::DryRun);
        assert!(result.success());
        assert_eq!(result.migrations_run, 1);
        // Dry run does modify the clone (simplified impl), but in production would use CoW
    }

    #[test]
    fn test_runner_rollback() {
        let mut state = sample_state(3);
        let mut runner = MigrationRunner::new();
        runner.register(Box::new(AddAccountMetadata {
            field_name: "tier".to_string(),
            default_value: "basic".to_string(),
        }));

        runner.run(&mut state, RunMode::Live);
        assert_eq!(state.version, StateVersion(1));
        assert_eq!(state.applied.len(), 1);

        let rolled = runner.rollback_last(&mut state, 1).unwrap();
        assert_eq!(rolled, 1);
        assert_eq!(state.version, StateVersion::genesis());
        assert!(state.applied.is_empty());
    }

    #[test]
    fn test_pending_migrations() {
        let state = sample_state(2);
        let mut runner = MigrationRunner::new();
        runner.register(Box::new(AddAccountMetadata {
            field_name: "a".to_string(),
            default_value: "b".to_string(),
        }));
        runner.register(Box::new(RebalanceSupply::new(2, 1)));

        let pending = runner.pending(&state);
        assert_eq!(pending.len(), 2);
    }

    #[test]
    fn test_add_global_param() {
        let mut state = sample_state(1);
        let m = AddGlobalParam {
            key: "max_validators".to_string(),
            value: 100u64.to_be_bytes().to_vec(),
            required_version: StateVersion::genesis(),
            new_version: StateVersion(1),
        };

        assert!(m.validate_pre(&state).is_ok());
        let effect = m.apply(&mut state).unwrap();
        assert_eq!(effect.params_modified, 1);
        assert!(state.params.contains_key("max_validators"));

        m.rollback(&mut state).unwrap();
        assert!(!state.params.contains_key("max_validators"));
    }

    #[test]
    fn test_pre_validation_failure() {
        let mut state = sample_state(2);
        state.version = StateVersion(5); // Wrong version
        let m = AddAccountMetadata {
            field_name: "x".to_string(),
            default_value: "y".to_string(),
        };
        assert!(m.validate_pre(&state).is_err());
    }

    #[test]
    fn test_rebalance_zero_denominator() {
        let state = MigrationState {
            version: StateVersion(1),
            ..MigrationState::new()
        };
        let m = RebalanceSupply::new(1, 0);
        assert!(m.validate_pre(&state).is_err());
    }

    #[test]
    fn test_progress_tracking() {
        let mut state = sample_state(2);
        let mut runner = MigrationRunner::new();
        runner.register(Box::new(AddAccountMetadata {
            field_name: "x".to_string(),
            default_value: "y".to_string(),
        }));

        runner.run(&mut state, RunMode::Live);
        let log = runner.progress_log();
        assert!(log.len() >= 2); // pre_validate + apply + complete
        assert_eq!(log[0].step, "pre_validate");
    }

    #[test]
    fn test_migration_record_integrity() {
        let mut state = sample_state(3);
        let pre_root = state.state_root();

        let mut runner = MigrationRunner::new();
        runner.register(Box::new(AddAccountMetadata {
            field_name: "v".to_string(),
            default_value: "1".to_string(),
        }));

        runner.run(&mut state, RunMode::Live);
        let record = &state.applied[0];
        assert_eq!(record.pre_state_root, pre_root);
        assert_ne!(record.pre_state_root, record.post_state_root);
        assert_eq!(record.accounts_modified, 3);
    }

    #[test]
    fn test_duplicate_param_rejected() {
        let mut state = sample_state(1);
        state.params.insert("existing".to_string(), vec![1]);
        let m = AddGlobalParam {
            key: "existing".to_string(),
            value: vec![2],
            required_version: StateVersion::genesis(),
            new_version: StateVersion(1),
        };
        assert!(m.validate_pre(&state).is_err());
    }
}
