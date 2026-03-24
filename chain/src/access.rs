//! CHAIN-024: Access control layer
//!
//! Role-based capability system for transaction authorization.
//! Supports: Admin, Operator, Provider, Challenger, Observer roles.
//! Capabilities are bitflags — roles map to capability sets.
//! Accounts can have multiple roles; capabilities union.

use crate::types::Address;
use std::collections::HashMap;

/// Individual capabilities (bitflags).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u64)]
pub enum Capability {
    /// Submit inference commits
    SubmitCommit = 1 << 0,
    /// Challenge an inference
    Challenge = 1 << 1,
    /// Register/update models
    RegisterModel = 1 << 2,
    /// Stake tokens
    Stake = 1 << 3,
    /// Unstake tokens
    Unstake = 1 << 4,
    /// Submit checkpoints to L1
    SubmitCheckpoint = 1 << 5,
    /// Create governance proposals
    Propose = 1 << 6,
    /// Vote on proposals
    Vote = 1 << 7,
    /// Transfer tokens
    Transfer = 1 << 8,
    /// Schedule inference jobs
    ScheduleJob = 1 << 9,
    /// Manage protocol parameters
    ManageParams = 1 << 10,
    /// Grant/revoke roles
    ManageRoles = 1 << 11,
    /// Upgrade contracts
    Upgrade = 1 << 12,
    /// Emergency pause
    EmergencyPause = 1 << 13,
    /// Read-only queries (everyone has this)
    Query = 1 << 14,
}

/// Packed capability set.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct CapabilitySet(pub u64);

impl CapabilitySet {
    pub fn empty() -> Self {
        Self(0)
    }

    pub fn has(&self, cap: Capability) -> bool {
        self.0 & (cap as u64) != 0
    }

    pub fn grant(&mut self, cap: Capability) {
        self.0 |= cap as u64;
    }

    pub fn revoke(&mut self, cap: Capability) {
        self.0 &= !(cap as u64);
    }

    pub fn union(&self, other: &Self) -> Self {
        Self(self.0 | other.0)
    }

    pub fn count(&self) -> u32 {
        self.0.count_ones()
    }
}

/// Pre-defined roles with capability sets.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Role {
    /// Full system control
    Admin,
    /// Node operator: commits, checkpoints, staking
    Operator,
    /// Inference provider: commits, model registration, jobs
    Provider,
    /// Dispute challenger
    Challenger,
    /// Read-only + transfer + vote
    Observer,
}

impl Role {
    /// Default capabilities for this role.
    pub fn capabilities(&self) -> CapabilitySet {
        use Capability::*;
        let bits = match self {
            Role::Admin => {
                ManageParams as u64
                    | ManageRoles as u64
                    | Upgrade as u64
                    | EmergencyPause as u64
                    | Query as u64
                    | Transfer as u64
                    | Propose as u64
                    | Vote as u64
            }
            Role::Operator => {
                SubmitCommit as u64
                    | SubmitCheckpoint as u64
                    | Stake as u64
                    | Unstake as u64
                    | Transfer as u64
                    | Vote as u64
                    | Query as u64
            }
            Role::Provider => {
                SubmitCommit as u64
                    | RegisterModel as u64
                    | Stake as u64
                    | Unstake as u64
                    | ScheduleJob as u64
                    | Transfer as u64
                    | Vote as u64
                    | Query as u64
            }
            Role::Challenger => Challenge as u64 | Stake as u64 | Transfer as u64 | Query as u64,
            Role::Observer => Transfer as u64 | Vote as u64 | Query as u64,
        };
        CapabilitySet(bits)
    }

    pub fn all() -> &'static [Role] {
        &[
            Role::Admin,
            Role::Operator,
            Role::Provider,
            Role::Challenger,
            Role::Observer,
        ]
    }
}

/// Per-account role assignment with optional expiry.
#[derive(Debug, Clone)]
pub struct RoleGrant {
    pub role: Role,
    pub granted_by: Address,
    pub granted_at: u64,
    /// Epoch at which this grant expires (0 = never).
    pub expires_at: u64,
}

impl RoleGrant {
    pub fn is_expired(&self, current_epoch: u64) -> bool {
        self.expires_at > 0 && current_epoch >= self.expires_at
    }
}

/// Access control registry.
#[derive(Debug, Clone)]
pub struct AccessControl {
    /// Roles granted to each address.
    grants: HashMap<Address, Vec<RoleGrant>>,
    /// Additional per-address capability overrides (beyond roles).
    overrides: HashMap<Address, CapabilitySet>,
    /// Global pause flag — blocks all non-admin operations.
    paused: bool,
    /// Protocol epoch for expiry checks.
    current_epoch: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AccessError {
    /// Missing required capability.
    Denied {
        required: &'static str,
        address: Address,
    },
    /// System is paused (only Admin can act).
    Paused,
    /// Cannot grant role without ManageRoles capability.
    Unauthorized,
    /// Role already granted.
    AlreadyGranted,
    /// Role not found on address.
    RoleNotFound,
    /// Cannot revoke own Admin role (safety).
    SelfRevoke,
}

impl std::fmt::Display for AccessError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Denied { required, address } => {
                write!(f, "{address} lacks capability: {required}")
            }
            Self::Paused => write!(f, "protocol is paused"),
            Self::Unauthorized => write!(f, "unauthorized to manage roles"),
            Self::AlreadyGranted => write!(f, "role already granted"),
            Self::RoleNotFound => write!(f, "role not found"),
            Self::SelfRevoke => write!(f, "cannot revoke own admin role"),
        }
    }
}

impl AccessControl {
    pub fn new() -> Self {
        Self {
            grants: HashMap::new(),
            overrides: HashMap::new(),
            paused: false,
            current_epoch: 0,
        }
    }

    /// Bootstrap with genesis admin.
    pub fn with_admin(admin: Address) -> Self {
        let mut ac = Self::new();
        ac.grants.insert(
            admin,
            vec![RoleGrant {
                role: Role::Admin,
                granted_by: admin,
                granted_at: 0,
                expires_at: 0,
            }],
        );
        ac
    }

    pub fn set_epoch(&mut self, epoch: u64) {
        self.current_epoch = epoch;
    }

    /// Get effective capabilities for an address (union of all active roles + overrides).
    pub fn effective_capabilities(&self, addr: &Address) -> CapabilitySet {
        let mut caps = CapabilitySet::empty();

        if let Some(grants) = self.grants.get(addr) {
            for g in grants {
                if !g.is_expired(self.current_epoch) {
                    caps = caps.union(&g.role.capabilities());
                }
            }
        }

        if let Some(over) = self.overrides.get(addr) {
            caps = caps.union(over);
        }

        caps
    }

    /// Check if address has a specific capability. Respects pause state.
    pub fn check(
        &self,
        addr: &Address,
        cap: Capability,
        cap_name: &'static str,
    ) -> Result<(), AccessError> {
        if self.paused {
            // Only admin can operate during pause
            let caps = self.effective_capabilities(addr);
            if !caps.has(Capability::EmergencyPause) {
                return Err(AccessError::Paused);
            }
        }

        let caps = self.effective_capabilities(addr);
        if caps.has(cap) {
            Ok(())
        } else {
            Err(AccessError::Denied {
                required: cap_name,
                address: *addr,
            })
        }
    }

    /// Grant a role. Caller must have ManageRoles capability.
    pub fn grant_role(
        &mut self,
        granter: &Address,
        target: &Address,
        role: Role,
        expires_at: u64,
    ) -> Result<(), AccessError> {
        self.check(granter, Capability::ManageRoles, "ManageRoles")?;

        let grants = self.grants.entry(*target).or_default();
        if grants
            .iter()
            .any(|g| g.role == role && !g.is_expired(self.current_epoch))
        {
            return Err(AccessError::AlreadyGranted);
        }

        grants.push(RoleGrant {
            role,
            granted_by: *granter,
            granted_at: self.current_epoch,
            expires_at,
        });
        Ok(())
    }

    /// Revoke a role. Caller must have ManageRoles. Cannot revoke own Admin.
    pub fn revoke_role(
        &mut self,
        revoker: &Address,
        target: &Address,
        role: Role,
    ) -> Result<(), AccessError> {
        self.check(revoker, Capability::ManageRoles, "ManageRoles")?;

        if revoker == target && role == Role::Admin {
            return Err(AccessError::SelfRevoke);
        }

        let grants = self
            .grants
            .get_mut(target)
            .ok_or(AccessError::RoleNotFound)?;
        let before = grants.len();
        grants.retain(|g| g.role != role);
        if grants.len() == before {
            return Err(AccessError::RoleNotFound);
        }
        Ok(())
    }

    /// Grant an additional capability override (beyond roles).
    pub fn grant_capability_override(
        &mut self,
        granter: &Address,
        target: &Address,
        cap: Capability,
    ) -> Result<(), AccessError> {
        self.check(granter, Capability::ManageRoles, "ManageRoles")?;
        self.overrides.entry(*target).or_default().grant(cap);
        Ok(())
    }

    /// Emergency pause — requires EmergencyPause capability.
    pub fn pause(&mut self, caller: &Address) -> Result<(), AccessError> {
        self.check(caller, Capability::EmergencyPause, "EmergencyPause")?;
        self.paused = true;
        Ok(())
    }

    /// Unpause — requires EmergencyPause capability.
    pub fn unpause(&mut self, caller: &Address) -> Result<(), AccessError> {
        // During pause, only admin can act — check is handled inside check()
        self.check(caller, Capability::EmergencyPause, "EmergencyPause")?;
        self.paused = false;
        Ok(())
    }

    pub fn is_paused(&self) -> bool {
        self.paused
    }

    /// List active roles for an address.
    pub fn roles(&self, addr: &Address) -> Vec<Role> {
        self.grants
            .get(addr)
            .map(|gs| {
                gs.iter()
                    .filter(|g| !g.is_expired(self.current_epoch))
                    .map(|g| g.role)
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Garbage-collect expired grants.
    pub fn gc_expired(&mut self) {
        for grants in self.grants.values_mut() {
            grants.retain(|g| !g.is_expired(self.current_epoch));
        }
        self.grants.retain(|_, gs| !gs.is_empty());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn admin() -> Address {
        Address::test(1)
    }
    fn operator() -> Address {
        Address::test(2)
    }
    fn provider() -> Address {
        Address::test(3)
    }
    fn challenger() -> Address {
        Address::test(4)
    }
    fn observer() -> Address {
        Address::test(5)
    }
    fn nobody() -> Address {
        Address::test(99)
    }

    #[test]
    fn test_capability_set_operations() {
        let mut caps = CapabilitySet::empty();
        assert!(!caps.has(Capability::Transfer));
        caps.grant(Capability::Transfer);
        assert!(caps.has(Capability::Transfer));
        assert_eq!(caps.count(), 1);
        caps.revoke(Capability::Transfer);
        assert!(!caps.has(Capability::Transfer));
    }

    #[test]
    fn test_capability_set_union() {
        let mut a = CapabilitySet::empty();
        a.grant(Capability::Transfer);
        let mut b = CapabilitySet::empty();
        b.grant(Capability::Vote);
        let c = a.union(&b);
        assert!(c.has(Capability::Transfer));
        assert!(c.has(Capability::Vote));
    }

    #[test]
    fn test_role_capabilities() {
        let admin_caps = Role::Admin.capabilities();
        assert!(admin_caps.has(Capability::ManageRoles));
        assert!(admin_caps.has(Capability::EmergencyPause));
        assert!(!admin_caps.has(Capability::SubmitCommit)); // admin doesn't auto-get provider caps

        let provider_caps = Role::Provider.capabilities();
        assert!(provider_caps.has(Capability::SubmitCommit));
        assert!(provider_caps.has(Capability::RegisterModel));
        assert!(!provider_caps.has(Capability::ManageRoles));
    }

    #[test]
    fn test_genesis_admin() {
        let ac = AccessControl::with_admin(admin());
        let caps = ac.effective_capabilities(&admin());
        assert!(caps.has(Capability::ManageRoles));
        assert!(caps.has(Capability::EmergencyPause));
    }

    #[test]
    fn test_grant_and_check_role() {
        let mut ac = AccessControl::with_admin(admin());
        ac.grant_role(&admin(), &operator(), Role::Operator, 0)
            .unwrap();
        assert!(ac
            .check(&operator(), Capability::SubmitCommit, "SubmitCommit")
            .is_ok());
        assert!(ac
            .check(&operator(), Capability::ManageRoles, "ManageRoles")
            .is_err());
    }

    #[test]
    fn test_unauthorized_grant() {
        let mut ac = AccessControl::with_admin(admin());
        ac.grant_role(&admin(), &operator(), Role::Operator, 0)
            .unwrap();
        // Operator can't grant roles
        let err = ac.grant_role(&operator(), &provider(), Role::Provider, 0);
        assert!(matches!(err, Err(AccessError::Denied { .. })));
    }

    #[test]
    fn test_duplicate_grant() {
        let mut ac = AccessControl::with_admin(admin());
        ac.grant_role(&admin(), &operator(), Role::Operator, 0)
            .unwrap();
        let err = ac.grant_role(&admin(), &operator(), Role::Operator, 0);
        assert_eq!(err, Err(AccessError::AlreadyGranted));
    }

    #[test]
    fn test_revoke_role() {
        let mut ac = AccessControl::with_admin(admin());
        ac.grant_role(&admin(), &provider(), Role::Provider, 0)
            .unwrap();
        assert!(ac
            .check(&provider(), Capability::SubmitCommit, "SubmitCommit")
            .is_ok());
        ac.revoke_role(&admin(), &provider(), Role::Provider)
            .unwrap();
        assert!(ac
            .check(&provider(), Capability::SubmitCommit, "SubmitCommit")
            .is_err());
    }

    #[test]
    fn test_self_revoke_admin_blocked() {
        let mut ac = AccessControl::with_admin(admin());
        let err = ac.revoke_role(&admin(), &admin(), Role::Admin);
        assert_eq!(err, Err(AccessError::SelfRevoke));
    }

    #[test]
    fn test_role_expiry() {
        let mut ac = AccessControl::with_admin(admin());
        ac.grant_role(&admin(), &operator(), Role::Operator, 100)
            .unwrap();
        assert!(ac
            .check(&operator(), Capability::SubmitCommit, "SubmitCommit")
            .is_ok());
        ac.set_epoch(100);
        assert!(ac
            .check(&operator(), Capability::SubmitCommit, "SubmitCommit")
            .is_err());
    }

    #[test]
    fn test_pause_blocks_non_admin() {
        let mut ac = AccessControl::with_admin(admin());
        ac.grant_role(&admin(), &provider(), Role::Provider, 0)
            .unwrap();
        ac.pause(&admin()).unwrap();
        assert!(ac
            .check(&provider(), Capability::SubmitCommit, "SubmitCommit")
            .is_err());
        assert!(ac
            .check(&admin(), Capability::ManageRoles, "ManageRoles")
            .is_ok());
        ac.unpause(&admin()).unwrap();
        assert!(ac
            .check(&provider(), Capability::SubmitCommit, "SubmitCommit")
            .is_ok());
    }

    #[test]
    fn test_capability_override() {
        let mut ac = AccessControl::with_admin(admin());
        ac.grant_role(&admin(), &observer(), Role::Observer, 0)
            .unwrap();
        assert!(ac
            .check(&observer(), Capability::Challenge, "Challenge")
            .is_err());
        ac.grant_capability_override(&admin(), &observer(), Capability::Challenge)
            .unwrap();
        assert!(ac
            .check(&observer(), Capability::Challenge, "Challenge")
            .is_ok());
    }

    #[test]
    fn test_multi_role_union() {
        let mut ac = AccessControl::with_admin(admin());
        ac.grant_role(&admin(), &operator(), Role::Operator, 0)
            .unwrap();
        ac.grant_role(&admin(), &operator(), Role::Challenger, 0)
            .unwrap();
        let caps = ac.effective_capabilities(&operator());
        assert!(caps.has(Capability::SubmitCommit)); // from Operator
        assert!(caps.has(Capability::Challenge)); // from Challenger
        assert!(caps.has(Capability::SubmitCheckpoint)); // from Operator
    }

    #[test]
    fn test_nobody_has_no_caps() {
        let ac = AccessControl::with_admin(admin());
        let caps = ac.effective_capabilities(&nobody());
        assert_eq!(caps.count(), 0);
    }

    #[test]
    fn test_gc_expired() {
        let mut ac = AccessControl::with_admin(admin());
        ac.grant_role(&admin(), &operator(), Role::Operator, 50)
            .unwrap();
        ac.set_epoch(50);
        assert!(ac.roles(&operator()).is_empty());
        ac.gc_expired();
        assert!(ac.grants.get(&operator()).is_none());
    }

    #[test]
    fn test_roles_listing() {
        let mut ac = AccessControl::with_admin(admin());
        ac.grant_role(&admin(), &provider(), Role::Provider, 0)
            .unwrap();
        ac.grant_role(&admin(), &provider(), Role::Challenger, 0)
            .unwrap();
        let roles = ac.roles(&provider());
        assert_eq!(roles.len(), 2);
        assert!(roles.contains(&Role::Provider));
        assert!(roles.contains(&Role::Challenger));
    }

    #[test]
    fn test_pause_requires_capability() {
        let mut ac = AccessControl::with_admin(admin());
        ac.grant_role(&admin(), &operator(), Role::Operator, 0)
            .unwrap();
        let err = ac.pause(&operator());
        assert!(matches!(err, Err(AccessError::Denied { .. })));
    }
}
