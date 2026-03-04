//! Multi-sig CLI Commands (NODE-033)
//!
//! CLI subcommands for creating and managing multi-signature wallets.
//! Integrates with the multi-sig wallet system (CHAIN-037).
//!
//! Subcommands:
//!   create <owners...> --threshold <M> [--daily-limit <amount>] [--ttl <epochs>]
//!   propose <wallet> <target> <value> [--data <hex>]
//!   approve <wallet> <proposal-id>
//!   reject <wallet> <proposal-id>
//!   execute <wallet> <proposal-id>
//!   cancel <wallet> <proposal-id>
//!   list <wallet> [--pending | --executed | --all]
//!   info <wallet>
//!   owners <wallet> [--add <addr> | --remove <addr>]
//!   threshold <wallet> <new-threshold>

use std::fmt;
use std::collections::HashMap;

pub type Address = [u8; 32];
pub type WalletId = [u8; 32];

// ── Subcommand Definitions ──────────────────────────────────────────

#[derive(Debug, Clone, PartialEq)]
pub enum MultisigCmd {
    /// Create a new multi-sig wallet.
    Create {
        owners: Vec<String>,
        threshold: u32,
        daily_limit: Option<u64>,
        proposal_ttl: u64,
    },
    /// Submit a new spending proposal.
    Propose {
        wallet: String,
        target: String,
        value: u64,
        data: Option<Vec<u8>>,
    },
    /// Approve a pending proposal.
    Approve {
        wallet: String,
        proposal_id: u64,
    },
    /// Reject a pending proposal.
    Reject {
        wallet: String,
        proposal_id: u64,
    },
    /// Execute a fully-approved proposal.
    Execute {
        wallet: String,
        proposal_id: u64,
    },
    /// Cancel a proposal (only proposer).
    Cancel {
        wallet: String,
        proposal_id: u64,
    },
    /// List proposals in a wallet.
    List {
        wallet: String,
        filter: ProposalFilter,
    },
    /// Show wallet info (owners, threshold, nonce, limits).
    Info {
        wallet: String,
    },
    /// Manage owners (add or remove).
    Owners {
        wallet: String,
        action: OwnerAction,
    },
    /// Change the approval threshold.
    Threshold {
        wallet: String,
        new_threshold: u32,
    },
    /// Show help for multisig subcommands.
    Help,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ProposalFilter {
    Pending,
    Executed,
    All,
}

#[derive(Debug, Clone, PartialEq)]
pub enum OwnerAction {
    List,
    Add(String),
    Remove(String),
}

// ── Parse Errors ────────────────────────────────────────────────────

#[derive(Debug, PartialEq)]
pub enum ParseError {
    MissingSubcommand,
    UnknownSubcommand(String),
    MissingArgument(String),
    InvalidNumber(String),
    InvalidHex(String),
    TooFewOwners,
    InvalidFlag(String),
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingSubcommand => write!(f, "missing subcommand"),
            Self::UnknownSubcommand(s) => write!(f, "unknown subcommand: {s}"),
            Self::MissingArgument(s) => write!(f, "missing argument: {s}"),
            Self::InvalidNumber(s) => write!(f, "invalid number: {s}"),
            Self::InvalidHex(s) => write!(f, "invalid hex: {s}"),
            Self::TooFewOwners => write!(f, "need at least 2 owners"),
            Self::InvalidFlag(s) => write!(f, "invalid flag: {s}"),
        }
    }
}

// ── Parser ──────────────────────────────────────────────────────────

pub fn parse_multisig_cmd(args: &[&str]) -> Result<MultisigCmd, ParseError> {
    if args.is_empty() {
        return Err(ParseError::MissingSubcommand);
    }
    match args[0] {
        "create" => parse_create(&args[1..]),
        "propose" => parse_propose(&args[1..]),
        "approve" => parse_wallet_proposal(&args[1..], "approve"),
        "reject" => parse_wallet_proposal(&args[1..], "reject"),
        "execute" => parse_wallet_proposal(&args[1..], "execute"),
        "cancel" => parse_wallet_proposal(&args[1..], "cancel"),
        "list" => parse_list(&args[1..]),
        "info" => parse_info(&args[1..]),
        "owners" => parse_owners(&args[1..]),
        "threshold" => parse_threshold(&args[1..]),
        "help" | "--help" | "-h" => Ok(MultisigCmd::Help),
        other => Err(ParseError::UnknownSubcommand(other.to_string())),
    }
}

fn parse_create(args: &[&str]) -> Result<MultisigCmd, ParseError> {
    let mut owners = Vec::new();
    let mut threshold: Option<u32> = None;
    let mut daily_limit: Option<u64> = None;
    let mut proposal_ttl: u64 = 2880; // default ~24h
    let mut i = 0;
    while i < args.len() {
        match args[i] {
            "--threshold" | "-t" => {
                i += 1;
                threshold = Some(parse_u32(args.get(i).ok_or(ParseError::MissingArgument("threshold".into()))?)?);
            }
            "--daily-limit" => {
                i += 1;
                daily_limit = Some(parse_u64(args.get(i).ok_or(ParseError::MissingArgument("daily-limit".into()))?)?);
            }
            "--ttl" => {
                i += 1;
                proposal_ttl = parse_u64(args.get(i).ok_or(ParseError::MissingArgument("ttl".into()))?)?;
            }
            s if s.starts_with('-') => return Err(ParseError::InvalidFlag(s.to_string())),
            owner => owners.push(owner.to_string()),
        }
        i += 1;
    }
    if owners.len() < 2 {
        return Err(ParseError::TooFewOwners);
    }
    let threshold = threshold.ok_or(ParseError::MissingArgument("--threshold".into()))?;
    Ok(MultisigCmd::Create { owners, threshold, daily_limit, proposal_ttl })
}

fn parse_propose(args: &[&str]) -> Result<MultisigCmd, ParseError> {
    if args.len() < 3 {
        return Err(ParseError::MissingArgument("wallet target value".into()));
    }
    let wallet = args[0].to_string();
    let target = args[1].to_string();
    let value = parse_u64(args[2])?;
    let mut data = None;
    let mut i = 3;
    while i < args.len() {
        match args[i] {
            "--data" => {
                i += 1;
                let hex_str = args.get(i).ok_or(ParseError::MissingArgument("data".into()))?;
                data = Some(parse_hex(hex_str)?);
            }
            s if s.starts_with('-') => return Err(ParseError::InvalidFlag(s.to_string())),
            _ => {}
        }
        i += 1;
    }
    Ok(MultisigCmd::Propose { wallet, target, value, data })
}

fn parse_wallet_proposal(args: &[&str], cmd: &str) -> Result<MultisigCmd, ParseError> {
    if args.len() < 2 {
        return Err(ParseError::MissingArgument(format!("wallet proposal-id for {cmd}")));
    }
    let wallet = args[0].to_string();
    let proposal_id = parse_u64(args[1])?;
    match cmd {
        "approve" => Ok(MultisigCmd::Approve { wallet, proposal_id }),
        "reject" => Ok(MultisigCmd::Reject { wallet, proposal_id }),
        "execute" => Ok(MultisigCmd::Execute { wallet, proposal_id }),
        "cancel" => Ok(MultisigCmd::Cancel { wallet, proposal_id }),
        _ => unreachable!(),
    }
}

fn parse_list(args: &[&str]) -> Result<MultisigCmd, ParseError> {
    if args.is_empty() {
        return Err(ParseError::MissingArgument("wallet".into()));
    }
    let wallet = args[0].to_string();
    let mut filter = ProposalFilter::Pending;
    for &a in &args[1..] {
        match a {
            "--pending" => filter = ProposalFilter::Pending,
            "--executed" => filter = ProposalFilter::Executed,
            "--all" => filter = ProposalFilter::All,
            s if s.starts_with('-') => return Err(ParseError::InvalidFlag(s.to_string())),
            _ => {}
        }
    }
    Ok(MultisigCmd::List { wallet, filter })
}

fn parse_info(args: &[&str]) -> Result<MultisigCmd, ParseError> {
    if args.is_empty() {
        return Err(ParseError::MissingArgument("wallet".into()));
    }
    Ok(MultisigCmd::Info { wallet: args[0].to_string() })
}

fn parse_owners(args: &[&str]) -> Result<MultisigCmd, ParseError> {
    if args.is_empty() {
        return Err(ParseError::MissingArgument("wallet".into()));
    }
    let wallet = args[0].to_string();
    let mut action = OwnerAction::List;
    let mut i = 1;
    while i < args.len() {
        match args[i] {
            "--add" => {
                i += 1;
                let addr = args.get(i).ok_or(ParseError::MissingArgument("address".into()))?;
                action = OwnerAction::Add(addr.to_string());
            }
            "--remove" => {
                i += 1;
                let addr = args.get(i).ok_or(ParseError::MissingArgument("address".into()))?;
                action = OwnerAction::Remove(addr.to_string());
            }
            s if s.starts_with('-') => return Err(ParseError::InvalidFlag(s.to_string())),
            _ => {}
        }
        i += 1;
    }
    Ok(MultisigCmd::Owners { wallet, action })
}

fn parse_threshold(args: &[&str]) -> Result<MultisigCmd, ParseError> {
    if args.len() < 2 {
        return Err(ParseError::MissingArgument("wallet new-threshold".into()));
    }
    let wallet = args[0].to_string();
    let new_threshold = parse_u32(args[1])?;
    Ok(MultisigCmd::Threshold { wallet, new_threshold })
}

fn parse_u32(s: &str) -> Result<u32, ParseError> {
    s.parse().map_err(|_| ParseError::InvalidNumber(s.to_string()))
}

fn parse_u64(s: &str) -> Result<u64, ParseError> {
    s.parse().map_err(|_| ParseError::InvalidNumber(s.to_string()))
}

fn parse_hex(s: &str) -> Result<Vec<u8>, ParseError> {
    let s = s.strip_prefix("0x").unwrap_or(s);
    if s.len() % 2 != 0 {
        return Err(ParseError::InvalidHex(s.to_string()));
    }
    (0..s.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&s[i..i + 2], 16).map_err(|_| ParseError::InvalidHex(s.to_string())))
        .collect()
}

// ── Execution Context ───────────────────────────────────────────────

/// Result of executing a multi-sig CLI command.
#[derive(Debug, Clone, PartialEq)]
pub enum MultisigResult {
    WalletCreated { wallet_id: String, owners: usize, threshold: u32 },
    ProposalCreated { wallet: String, proposal_id: u64 },
    ProposalApproved { wallet: String, proposal_id: u64, approvals: u32, threshold: u32 },
    ProposalRejected { wallet: String, proposal_id: u64, rejections: u32 },
    ProposalExecuted { wallet: String, proposal_id: u64, target: String, value: u64 },
    ProposalCancelled { wallet: String, proposal_id: u64 },
    ProposalList { wallet: String, proposals: Vec<ProposalSummary> },
    WalletInfo { wallet: String, owners: Vec<String>, threshold: u32, nonce: u64, daily_limit: Option<u64>, daily_spent: u64, pending_count: usize },
    OwnersListed { wallet: String, owners: Vec<String> },
    OwnerAdded { wallet: String, new_owner: String, total_owners: usize },
    OwnerRemoved { wallet: String, removed: String, total_owners: usize },
    ThresholdChanged { wallet: String, old: u32, new_threshold: u32 },
    HelpText(String),
}

#[derive(Debug, Clone, PartialEq)]
pub struct ProposalSummary {
    pub id: u64,
    pub proposer: String,
    pub target: String,
    pub value: u64,
    pub approvals: u32,
    pub rejections: u32,
    pub executed: bool,
    pub cancelled: bool,
    pub expired: bool,
}

impl fmt::Display for MultisigResult {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::WalletCreated { wallet_id, owners, threshold } =>
                write!(f, "✅ Wallet {wallet_id} created ({threshold}-of-{owners})"),
            Self::ProposalCreated { wallet, proposal_id } =>
                write!(f, "📝 Proposal #{proposal_id} created in {wallet}"),
            Self::ProposalApproved { wallet, proposal_id, approvals, threshold } =>
                write!(f, "👍 Proposal #{proposal_id} approved ({approvals}/{threshold}) in {wallet}"),
            Self::ProposalRejected { wallet, proposal_id, rejections } =>
                write!(f, "👎 Proposal #{proposal_id} rejected ({rejections} rejections) in {wallet}"),
            Self::ProposalExecuted { wallet, proposal_id, target, value } =>
                write!(f, "⚡ Proposal #{proposal_id} executed: {value} → {target} in {wallet}"),
            Self::ProposalCancelled { wallet, proposal_id } =>
                write!(f, "🚫 Proposal #{proposal_id} cancelled in {wallet}"),
            Self::ProposalList { wallet, proposals } => {
                write!(f, "📋 Proposals in {wallet} ({} total):", proposals.len())?;
                for p in proposals {
                    let status = if p.executed { "✅" } else if p.cancelled { "🚫" } else if p.expired { "⏰" } else { "⏳" };
                    write!(f, "\n  {status} #{}: {} → {} ({} approve, {} reject)",
                        p.id, p.value, p.target, p.approvals, p.rejections)?;
                }
                Ok(())
            }
            Self::WalletInfo { wallet, owners, threshold, nonce, daily_limit, daily_spent, pending_count } => {
                write!(f, "🔐 Wallet {wallet}\n  Threshold: {threshold}-of-{}\n  Nonce: {nonce}\n  Pending: {pending_count}",
                    owners.len())?;
                if let Some(limit) = daily_limit {
                    write!(f, "\n  Daily: {daily_spent}/{limit}")?;
                }
                Ok(())
            }
            Self::OwnersListed { wallet, owners } => {
                write!(f, "👥 Owners of {wallet}:")?;
                for o in owners { write!(f, "\n  • {o}")?; }
                Ok(())
            }
            Self::OwnerAdded { wallet, new_owner, total_owners } =>
                write!(f, "➕ Added {new_owner} to {wallet} ({total_owners} owners)"),
            Self::OwnerRemoved { wallet, removed, total_owners } =>
                write!(f, "➖ Removed {removed} from {wallet} ({total_owners} owners)"),
            Self::ThresholdChanged { wallet, old, new_threshold } =>
                write!(f, "🔧 Threshold changed {old} → {new_threshold} in {wallet}"),
            Self::HelpText(s) => write!(f, "{s}"),
        }
    }
}

// ── Simulated Executor ──────────────────────────────────────────────

/// In-memory multi-sig CLI executor (for testing without real chain connection).
pub struct MultisigExecutor {
    wallets: HashMap<String, WalletState>,
    signer: String,
    current_epoch: u64,
}

struct WalletState {
    owners: Vec<String>,
    threshold: u32,
    daily_limit: Option<u64>,
    daily_spent: u64,
    daily_reset_epoch: u64,
    proposal_ttl: u64,
    proposals: HashMap<u64, SimProposal>,
    next_id: u64,
    nonce: u64,
}

#[derive(Clone)]
struct SimProposal {
    id: u64,
    proposer: String,
    target: String,
    value: u64,
    data: Vec<u8>,
    approvals: Vec<String>,
    rejections: Vec<String>,
    created_at: u64,
    executed: bool,
    cancelled: bool,
}

#[derive(Debug, PartialEq)]
pub enum ExecError {
    WalletNotFound(String),
    WalletAlreadyExists(String),
    NotOwner,
    ProposalNotFound(u64),
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
    InvalidThreshold,
}

impl fmt::Display for ExecError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::WalletNotFound(w) => write!(f, "wallet not found: {w}"),
            Self::WalletAlreadyExists(w) => write!(f, "wallet already exists: {w}"),
            Self::NotOwner => write!(f, "signer is not a wallet owner"),
            Self::ProposalNotFound(id) => write!(f, "proposal #{id} not found"),
            Self::AlreadyApproved => write!(f, "already approved"),
            Self::AlreadyRejected => write!(f, "already rejected"),
            Self::AlreadyExecuted => write!(f, "proposal already executed"),
            Self::ProposalExpired => write!(f, "proposal expired"),
            Self::ProposalCancelled => write!(f, "proposal cancelled"),
            Self::InsufficientApprovals => write!(f, "insufficient approvals"),
            Self::OnlyProposerCanCancel => write!(f, "only proposer can cancel"),
            Self::DailyLimitExceeded => write!(f, "daily spending limit exceeded"),
            Self::TooFewOwners => write!(f, "need at least 2 owners"),
            Self::OwnerAlreadyExists => write!(f, "owner already exists"),
            Self::OwnerNotFound => write!(f, "owner not found"),
            Self::ThresholdWouldExceedOwners => write!(f, "threshold would exceed owner count"),
            Self::InvalidThreshold => write!(f, "invalid threshold"),
        }
    }
}

impl MultisigExecutor {
    pub fn new(signer: String, epoch: u64) -> Self {
        Self { wallets: HashMap::new(), signer, current_epoch: epoch }
    }

    pub fn set_epoch(&mut self, epoch: u64) { self.current_epoch = epoch; }
    pub fn set_signer(&mut self, signer: String) { self.signer = signer; }

    pub fn execute_cmd(&mut self, cmd: MultisigCmd) -> Result<MultisigResult, ExecError> {
        match cmd {
            MultisigCmd::Create { owners, threshold, daily_limit, proposal_ttl } => {
                if owners.len() < 2 { return Err(ExecError::TooFewOwners); }
                if threshold == 0 || threshold as usize > owners.len() {
                    return Err(ExecError::InvalidThreshold);
                }
                let id = format!("msig-{:04x}", self.wallets.len());
                if self.wallets.contains_key(&id) {
                    return Err(ExecError::WalletAlreadyExists(id));
                }
                let n_owners = owners.len();
                self.wallets.insert(id.clone(), WalletState {
                    owners,
                    threshold,
                    daily_limit,
                    daily_spent: 0,
                    daily_reset_epoch: self.current_epoch,
                    proposal_ttl,
                    proposals: HashMap::new(),
                    next_id: 1,
                    nonce: 0,
                });
                Ok(MultisigResult::WalletCreated { wallet_id: id, owners: n_owners, threshold })
            }

            MultisigCmd::Propose { wallet, target, value, data } => {
                let w = self.wallets.get_mut(&wallet).ok_or(ExecError::WalletNotFound(wallet.clone()))?;
                if !w.owners.contains(&self.signer) { return Err(ExecError::NotOwner); }
                let id = w.next_id;
                w.next_id += 1;
                w.proposals.insert(id, SimProposal {
                    id,
                    proposer: self.signer.clone(),
                    target,
                    value,
                    data: data.unwrap_or_default(),
                    approvals: vec![self.signer.clone()],
                    rejections: vec![],
                    created_at: self.current_epoch,
                    executed: false,
                    cancelled: false,
                });
                Ok(MultisigResult::ProposalCreated { wallet, proposal_id: id })
            }

            MultisigCmd::Approve { wallet, proposal_id } => {
                let w = self.wallets.get_mut(&wallet).ok_or(ExecError::WalletNotFound(wallet.clone()))?;
                if !w.owners.contains(&self.signer) { return Err(ExecError::NotOwner); }
                let p = w.proposals.get_mut(&proposal_id).ok_or(ExecError::ProposalNotFound(proposal_id))?;
                if p.executed { return Err(ExecError::AlreadyExecuted); }
                if p.cancelled { return Err(ExecError::ProposalCancelled); }
                if self.current_epoch > p.created_at + w.proposal_ttl { return Err(ExecError::ProposalExpired); }
                if p.approvals.contains(&self.signer) { return Err(ExecError::AlreadyApproved); }
                if p.rejections.contains(&self.signer) { return Err(ExecError::AlreadyRejected); }
                p.approvals.push(self.signer.clone());
                Ok(MultisigResult::ProposalApproved {
                    wallet, proposal_id,
                    approvals: p.approvals.len() as u32,
                    threshold: w.threshold,
                })
            }

            MultisigCmd::Reject { wallet, proposal_id } => {
                let w = self.wallets.get_mut(&wallet).ok_or(ExecError::WalletNotFound(wallet.clone()))?;
                if !w.owners.contains(&self.signer) { return Err(ExecError::NotOwner); }
                let p = w.proposals.get_mut(&proposal_id).ok_or(ExecError::ProposalNotFound(proposal_id))?;
                if p.executed { return Err(ExecError::AlreadyExecuted); }
                if p.cancelled { return Err(ExecError::ProposalCancelled); }
                if self.current_epoch > p.created_at + w.proposal_ttl { return Err(ExecError::ProposalExpired); }
                if p.rejections.contains(&self.signer) { return Err(ExecError::AlreadyRejected); }
                if p.approvals.contains(&self.signer) { return Err(ExecError::AlreadyApproved); }
                p.rejections.push(self.signer.clone());
                Ok(MultisigResult::ProposalRejected {
                    wallet, proposal_id,
                    rejections: p.rejections.len() as u32,
                })
            }

            MultisigCmd::Execute { wallet, proposal_id } => {
                let w = self.wallets.get_mut(&wallet).ok_or(ExecError::WalletNotFound(wallet.clone()))?;
                // Reset daily if needed
                let day_epochs = 2880u64;
                if self.current_epoch >= w.daily_reset_epoch + day_epochs {
                    w.daily_spent = 0;
                    w.daily_reset_epoch = self.current_epoch - (self.current_epoch % day_epochs);
                }
                let p = w.proposals.get(&proposal_id).ok_or(ExecError::ProposalNotFound(proposal_id))?;
                if p.executed { return Err(ExecError::AlreadyExecuted); }
                if p.cancelled { return Err(ExecError::ProposalCancelled); }
                if self.current_epoch > p.created_at + w.proposal_ttl { return Err(ExecError::ProposalExpired); }
                if (p.approvals.len() as u32) < w.threshold { return Err(ExecError::InsufficientApprovals); }
                if let Some(limit) = w.daily_limit {
                    if w.daily_spent + p.value > limit { return Err(ExecError::DailyLimitExceeded); }
                }
                let target = p.target.clone();
                let value = p.value;
                let p = w.proposals.get_mut(&proposal_id).unwrap();
                p.executed = true;
                w.nonce += 1;
                w.daily_spent += value;
                Ok(MultisigResult::ProposalExecuted { wallet, proposal_id, target, value })
            }

            MultisigCmd::Cancel { wallet, proposal_id } => {
                let w = self.wallets.get_mut(&wallet).ok_or(ExecError::WalletNotFound(wallet.clone()))?;
                let p = w.proposals.get_mut(&proposal_id).ok_or(ExecError::ProposalNotFound(proposal_id))?;
                if p.executed { return Err(ExecError::AlreadyExecuted); }
                if p.proposer != self.signer { return Err(ExecError::OnlyProposerCanCancel); }
                p.cancelled = true;
                Ok(MultisigResult::ProposalCancelled { wallet, proposal_id })
            }

            MultisigCmd::List { wallet, filter } => {
                let w = self.wallets.get(&wallet).ok_or(ExecError::WalletNotFound(wallet.clone()))?;
                let proposals: Vec<ProposalSummary> = w.proposals.values()
                    .filter(|p| match filter {
                        ProposalFilter::Pending => !p.executed && !p.cancelled,
                        ProposalFilter::Executed => p.executed,
                        ProposalFilter::All => true,
                    })
                    .map(|p| ProposalSummary {
                        id: p.id,
                        proposer: p.proposer.clone(),
                        target: p.target.clone(),
                        value: p.value,
                        approvals: p.approvals.len() as u32,
                        rejections: p.rejections.len() as u32,
                        executed: p.executed,
                        cancelled: p.cancelled,
                        expired: self.current_epoch > p.created_at + w.proposal_ttl,
                    })
                    .collect();
                Ok(MultisigResult::ProposalList { wallet, proposals })
            }

            MultisigCmd::Info { wallet } => {
                let w = self.wallets.get(&wallet).ok_or(ExecError::WalletNotFound(wallet.clone()))?;
                let pending_count = w.proposals.values().filter(|p| !p.executed && !p.cancelled).count();
                Ok(MultisigResult::WalletInfo {
                    wallet,
                    owners: w.owners.clone(),
                    threshold: w.threshold,
                    nonce: w.nonce,
                    daily_limit: w.daily_limit,
                    daily_spent: w.daily_spent,
                    pending_count,
                })
            }

            MultisigCmd::Owners { wallet, action } => {
                match action {
                    OwnerAction::List => {
                        let w = self.wallets.get(&wallet).ok_or(ExecError::WalletNotFound(wallet.clone()))?;
                        Ok(MultisigResult::OwnersListed { wallet, owners: w.owners.clone() })
                    }
                    OwnerAction::Add(new_owner) => {
                        let w = self.wallets.get_mut(&wallet).ok_or(ExecError::WalletNotFound(wallet.clone()))?;
                        if w.owners.contains(&new_owner) { return Err(ExecError::OwnerAlreadyExists); }
                        w.owners.push(new_owner.clone());
                        let total = w.owners.len();
                        Ok(MultisigResult::OwnerAdded { wallet, new_owner, total_owners: total })
                    }
                    OwnerAction::Remove(owner) => {
                        let w = self.wallets.get_mut(&wallet).ok_or(ExecError::WalletNotFound(wallet.clone()))?;
                        let idx = w.owners.iter().position(|o| o == &owner).ok_or(ExecError::OwnerNotFound)?;
                        if w.owners.len() - 1 < w.threshold as usize { return Err(ExecError::ThresholdWouldExceedOwners); }
                        if w.owners.len() - 1 < 2 { return Err(ExecError::TooFewOwners); }
                        w.owners.remove(idx);
                        let total = w.owners.len();
                        Ok(MultisigResult::OwnerRemoved { wallet, removed: owner, total_owners: total })
                    }
                }
            }

            MultisigCmd::Threshold { wallet, new_threshold } => {
                let w = self.wallets.get_mut(&wallet).ok_or(ExecError::WalletNotFound(wallet.clone()))?;
                if new_threshold == 0 || new_threshold as usize > w.owners.len() {
                    return Err(ExecError::InvalidThreshold);
                }
                let old = w.threshold;
                w.threshold = new_threshold;
                Ok(MultisigResult::ThresholdChanged { wallet, old, new_threshold })
            }

            MultisigCmd::Help => Ok(MultisigResult::HelpText(HELP_TEXT.to_string())),
        }
    }
}

const HELP_TEXT: &str = "\
prova multisig — Multi-signature wallet management

USAGE:
  prova multisig <COMMAND> [OPTIONS]

COMMANDS:
  create <owners...> --threshold <M>   Create a new M-of-N wallet
  propose <wallet> <target> <value>    Submit a spending proposal
  approve <wallet> <proposal-id>       Approve a pending proposal
  reject <wallet> <proposal-id>        Reject a pending proposal
  execute <wallet> <proposal-id>       Execute a fully-approved proposal
  cancel <wallet> <proposal-id>        Cancel (only proposer)
  list <wallet> [--pending|--executed|--all]  List proposals
  info <wallet>                        Show wallet details
  owners <wallet> [--add|--remove <addr>]    Manage owners
  threshold <wallet> <new-threshold>   Change approval threshold

OPTIONS:
  --daily-limit <amount>    Set daily spending limit (create)
  --ttl <epochs>            Proposal time-to-live (default: 2880)
  --data <hex>              Attach calldata to proposal (propose)
  -h, --help                Show this help
";

// ── Tests ───────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    // ── Parser Tests ────────────────────────────────────────────────

    #[test]
    fn parse_create_basic() {
        let cmd = parse_multisig_cmd(&["create", "alice", "bob", "--threshold", "2"]).unwrap();
        assert_eq!(cmd, MultisigCmd::Create {
            owners: vec!["alice".into(), "bob".into()],
            threshold: 2,
            daily_limit: None,
            proposal_ttl: 2880,
        });
    }

    #[test]
    fn parse_create_with_options() {
        let cmd = parse_multisig_cmd(&["create", "a", "b", "c", "-t", "2", "--daily-limit", "5000", "--ttl", "100"]).unwrap();
        assert_eq!(cmd, MultisigCmd::Create {
            owners: vec!["a".into(), "b".into(), "c".into()],
            threshold: 2,
            daily_limit: Some(5000),
            proposal_ttl: 100,
        });
    }

    #[test]
    fn parse_create_too_few_owners() {
        assert_eq!(
            parse_multisig_cmd(&["create", "alice", "--threshold", "1"]).unwrap_err(),
            ParseError::TooFewOwners,
        );
    }

    #[test]
    fn parse_propose_basic() {
        let cmd = parse_multisig_cmd(&["propose", "msig-0000", "recipient", "1000"]).unwrap();
        assert_eq!(cmd, MultisigCmd::Propose {
            wallet: "msig-0000".into(),
            target: "recipient".into(),
            value: 1000,
            data: None,
        });
    }

    #[test]
    fn parse_propose_with_data() {
        let cmd = parse_multisig_cmd(&["propose", "w", "t", "0", "--data", "0xdeadbeef"]).unwrap();
        assert_eq!(cmd, MultisigCmd::Propose {
            wallet: "w".into(),
            target: "t".into(),
            value: 0,
            data: Some(vec![0xde, 0xad, 0xbe, 0xef]),
        });
    }

    #[test]
    fn parse_approve() {
        let cmd = parse_multisig_cmd(&["approve", "w", "5"]).unwrap();
        assert_eq!(cmd, MultisigCmd::Approve { wallet: "w".into(), proposal_id: 5 });
    }

    #[test]
    fn parse_reject() {
        let cmd = parse_multisig_cmd(&["reject", "w", "3"]).unwrap();
        assert_eq!(cmd, MultisigCmd::Reject { wallet: "w".into(), proposal_id: 3 });
    }

    #[test]
    fn parse_execute() {
        let cmd = parse_multisig_cmd(&["execute", "w", "1"]).unwrap();
        assert_eq!(cmd, MultisigCmd::Execute { wallet: "w".into(), proposal_id: 1 });
    }

    #[test]
    fn parse_cancel() {
        let cmd = parse_multisig_cmd(&["cancel", "w", "2"]).unwrap();
        assert_eq!(cmd, MultisigCmd::Cancel { wallet: "w".into(), proposal_id: 2 });
    }

    #[test]
    fn parse_list_default_pending() {
        let cmd = parse_multisig_cmd(&["list", "w"]).unwrap();
        assert_eq!(cmd, MultisigCmd::List { wallet: "w".into(), filter: ProposalFilter::Pending });
    }

    #[test]
    fn parse_list_all() {
        let cmd = parse_multisig_cmd(&["list", "w", "--all"]).unwrap();
        assert_eq!(cmd, MultisigCmd::List { wallet: "w".into(), filter: ProposalFilter::All });
    }

    #[test]
    fn parse_info() {
        let cmd = parse_multisig_cmd(&["info", "w"]).unwrap();
        assert_eq!(cmd, MultisigCmd::Info { wallet: "w".into() });
    }

    #[test]
    fn parse_owners_list() {
        let cmd = parse_multisig_cmd(&["owners", "w"]).unwrap();
        assert_eq!(cmd, MultisigCmd::Owners { wallet: "w".into(), action: OwnerAction::List });
    }

    #[test]
    fn parse_owners_add() {
        let cmd = parse_multisig_cmd(&["owners", "w", "--add", "newguy"]).unwrap();
        assert_eq!(cmd, MultisigCmd::Owners { wallet: "w".into(), action: OwnerAction::Add("newguy".into()) });
    }

    #[test]
    fn parse_owners_remove() {
        let cmd = parse_multisig_cmd(&["owners", "w", "--remove", "oldguy"]).unwrap();
        assert_eq!(cmd, MultisigCmd::Owners { wallet: "w".into(), action: OwnerAction::Remove("oldguy".into()) });
    }

    #[test]
    fn parse_threshold() {
        let cmd = parse_multisig_cmd(&["threshold", "w", "3"]).unwrap();
        assert_eq!(cmd, MultisigCmd::Threshold { wallet: "w".into(), new_threshold: 3 });
    }

    #[test]
    fn parse_help() {
        assert_eq!(parse_multisig_cmd(&["help"]).unwrap(), MultisigCmd::Help);
    }

    #[test]
    fn parse_missing_subcommand() {
        assert_eq!(parse_multisig_cmd(&[]).unwrap_err(), ParseError::MissingSubcommand);
    }

    #[test]
    fn parse_unknown_subcommand() {
        assert_eq!(
            parse_multisig_cmd(&["explode"]).unwrap_err(),
            ParseError::UnknownSubcommand("explode".into()),
        );
    }

    #[test]
    fn parse_invalid_hex() {
        assert_eq!(
            parse_multisig_cmd(&["propose", "w", "t", "0", "--data", "0xGG"]).unwrap_err(),
            ParseError::InvalidHex("GG".into()),
        );
    }

    #[test]
    fn parse_hex_odd_length() {
        assert_eq!(
            parse_multisig_cmd(&["propose", "w", "t", "0", "--data", "abc"]).unwrap_err(),
            ParseError::InvalidHex("abc".into()),
        );
    }

    // ── Executor Tests ──────────────────────────────────────────────

    fn make_executor() -> MultisigExecutor {
        MultisigExecutor::new("alice".into(), 100)
    }

    fn create_wallet(ex: &mut MultisigExecutor) -> String {
        let cmd = parse_multisig_cmd(&["create", "alice", "bob", "carol", "--threshold", "2"]).unwrap();
        match ex.execute_cmd(cmd).unwrap() {
            MultisigResult::WalletCreated { wallet_id, .. } => wallet_id,
            _ => panic!("expected WalletCreated"),
        }
    }

    #[test]
    fn exec_create_wallet() {
        let mut ex = make_executor();
        let wid = create_wallet(&mut ex);
        assert_eq!(wid, "msig-0000");
    }

    #[test]
    fn exec_full_lifecycle() {
        let mut ex = make_executor();
        let wid = create_wallet(&mut ex);

        // alice proposes
        let cmd = parse_multisig_cmd(&["propose", &wid, "treasury", "500"]).unwrap();
        let pid = match ex.execute_cmd(cmd).unwrap() {
            MultisigResult::ProposalCreated { proposal_id, .. } => proposal_id,
            _ => panic!("expected ProposalCreated"),
        };
        assert_eq!(pid, 1);

        // bob approves
        ex.set_signer("bob".into());
        let cmd = parse_multisig_cmd(&["approve", &wid, "1"]).unwrap();
        match ex.execute_cmd(cmd).unwrap() {
            MultisigResult::ProposalApproved { approvals, threshold, .. } => {
                assert_eq!(approvals, 2);
                assert_eq!(threshold, 2);
            }
            _ => panic!("expected ProposalApproved"),
        }

        // execute
        let cmd = parse_multisig_cmd(&["execute", &wid, "1"]).unwrap();
        match ex.execute_cmd(cmd).unwrap() {
            MultisigResult::ProposalExecuted { value, target, .. } => {
                assert_eq!(value, 500);
                assert_eq!(target, "treasury");
            }
            _ => panic!("expected ProposalExecuted"),
        }
    }

    #[test]
    fn exec_reject_and_insufficient() {
        let mut ex = make_executor();
        let wid = create_wallet(&mut ex);

        let cmd = parse_multisig_cmd(&["propose", &wid, "t", "100"]).unwrap();
        ex.execute_cmd(cmd).unwrap();

        // bob rejects
        ex.set_signer("bob".into());
        let cmd = parse_multisig_cmd(&["reject", &wid, "1"]).unwrap();
        match ex.execute_cmd(cmd).unwrap() {
            MultisigResult::ProposalRejected { rejections, .. } => assert_eq!(rejections, 1),
            _ => panic!("expected ProposalRejected"),
        }

        // try execute — only 1 approval (alice auto), threshold=2
        let cmd = parse_multisig_cmd(&["execute", &wid, "1"]).unwrap();
        assert_eq!(ex.execute_cmd(cmd).unwrap_err(), ExecError::InsufficientApprovals);
    }

    #[test]
    fn exec_cancel_only_proposer() {
        let mut ex = make_executor();
        let wid = create_wallet(&mut ex);

        let cmd = parse_multisig_cmd(&["propose", &wid, "t", "0"]).unwrap();
        ex.execute_cmd(cmd).unwrap();

        // bob cannot cancel alice's proposal
        ex.set_signer("bob".into());
        let cmd = parse_multisig_cmd(&["cancel", &wid, "1"]).unwrap();
        assert_eq!(ex.execute_cmd(cmd).unwrap_err(), ExecError::OnlyProposerCanCancel);

        // alice can
        ex.set_signer("alice".into());
        let cmd = parse_multisig_cmd(&["cancel", &wid, "1"]).unwrap();
        assert!(matches!(ex.execute_cmd(cmd).unwrap(), MultisigResult::ProposalCancelled { .. }));
    }

    #[test]
    fn exec_proposal_expiry() {
        let mut ex = MultisigExecutor::new("alice".into(), 100);
        let cmd = parse_multisig_cmd(&["create", "alice", "bob", "--threshold", "2", "--ttl", "50"]).unwrap();
        let wid = match ex.execute_cmd(cmd).unwrap() {
            MultisigResult::WalletCreated { wallet_id, .. } => wallet_id,
            _ => panic!(),
        };

        let cmd = parse_multisig_cmd(&["propose", &wid, "t", "0"]).unwrap();
        ex.execute_cmd(cmd).unwrap();

        ex.set_epoch(151);
        ex.set_signer("bob".into());
        let cmd = parse_multisig_cmd(&["approve", &wid, "1"]).unwrap();
        assert_eq!(ex.execute_cmd(cmd).unwrap_err(), ExecError::ProposalExpired);
    }

    #[test]
    fn exec_daily_limit() {
        let mut ex = make_executor();
        let cmd = parse_multisig_cmd(&["create", "alice", "bob", "--threshold", "1", "--daily-limit", "1000"]).unwrap();
        let wid = match ex.execute_cmd(cmd).unwrap() {
            MultisigResult::WalletCreated { wallet_id, .. } => wallet_id,
            _ => panic!(),
        };

        let cmd = parse_multisig_cmd(&["propose", &wid, "t", "800"]).unwrap();
        ex.execute_cmd(cmd).unwrap();
        let cmd = parse_multisig_cmd(&["execute", &wid, "1"]).unwrap();
        ex.execute_cmd(cmd).unwrap();

        let cmd = parse_multisig_cmd(&["propose", &wid, "t", "300"]).unwrap();
        ex.execute_cmd(cmd).unwrap();
        let cmd = parse_multisig_cmd(&["execute", &wid, "2"]).unwrap();
        assert_eq!(ex.execute_cmd(cmd).unwrap_err(), ExecError::DailyLimitExceeded);
    }

    #[test]
    fn exec_list_proposals() {
        let mut ex = make_executor();
        let wid = create_wallet(&mut ex);

        let cmd = parse_multisig_cmd(&["propose", &wid, "t1", "100"]).unwrap();
        ex.execute_cmd(cmd).unwrap();
        let cmd = parse_multisig_cmd(&["propose", &wid, "t2", "200"]).unwrap();
        ex.execute_cmd(cmd).unwrap();

        let cmd = parse_multisig_cmd(&["list", &wid, "--all"]).unwrap();
        match ex.execute_cmd(cmd).unwrap() {
            MultisigResult::ProposalList { proposals, .. } => assert_eq!(proposals.len(), 2),
            _ => panic!(),
        }
    }

    #[test]
    fn exec_wallet_info() {
        let mut ex = make_executor();
        let wid = create_wallet(&mut ex);

        let cmd = parse_multisig_cmd(&["info", &wid]).unwrap();
        match ex.execute_cmd(cmd).unwrap() {
            MultisigResult::WalletInfo { owners, threshold, nonce, pending_count, .. } => {
                assert_eq!(owners.len(), 3);
                assert_eq!(threshold, 2);
                assert_eq!(nonce, 0);
                assert_eq!(pending_count, 0);
            }
            _ => panic!(),
        }
    }

    #[test]
    fn exec_owner_management() {
        let mut ex = make_executor();
        let wid = create_wallet(&mut ex);

        // Add dave
        let cmd = parse_multisig_cmd(&["owners", &wid, "--add", "dave"]).unwrap();
        match ex.execute_cmd(cmd).unwrap() {
            MultisigResult::OwnerAdded { total_owners, .. } => assert_eq!(total_owners, 4),
            _ => panic!(),
        }

        // Remove dave
        let cmd = parse_multisig_cmd(&["owners", &wid, "--remove", "dave"]).unwrap();
        match ex.execute_cmd(cmd).unwrap() {
            MultisigResult::OwnerRemoved { total_owners, .. } => assert_eq!(total_owners, 3),
            _ => panic!(),
        }

        // Cannot remove below threshold
        let cmd = parse_multisig_cmd(&["owners", &wid, "--remove", "carol"]).unwrap();
        ex.execute_cmd(cmd).unwrap();
        let cmd = parse_multisig_cmd(&["owners", &wid, "--remove", "bob"]).unwrap();
        assert_eq!(ex.execute_cmd(cmd).unwrap_err(), ExecError::ThresholdWouldExceedOwners);
    }

    #[test]
    fn exec_threshold_change() {
        let mut ex = make_executor();
        let wid = create_wallet(&mut ex);

        let cmd = parse_multisig_cmd(&["threshold", &wid, "3"]).unwrap();
        match ex.execute_cmd(cmd).unwrap() {
            MultisigResult::ThresholdChanged { old, new_threshold, .. } => {
                assert_eq!(old, 2);
                assert_eq!(new_threshold, 3);
            }
            _ => panic!(),
        }

        // Invalid threshold
        let cmd = parse_multisig_cmd(&["threshold", &wid, "5"]).unwrap();
        assert_eq!(ex.execute_cmd(cmd).unwrap_err(), ExecError::InvalidThreshold);
    }

    #[test]
    fn exec_not_owner_errors() {
        let mut ex = make_executor();
        let wid = create_wallet(&mut ex);

        ex.set_signer("mallory".into());
        let cmd = parse_multisig_cmd(&["propose", &wid, "t", "0"]).unwrap();
        assert_eq!(ex.execute_cmd(cmd).unwrap_err(), ExecError::NotOwner);
    }

    #[test]
    fn exec_double_approve() {
        let mut ex = make_executor();
        let wid = create_wallet(&mut ex);

        let cmd = parse_multisig_cmd(&["propose", &wid, "t", "0"]).unwrap();
        ex.execute_cmd(cmd).unwrap();

        // alice auto-approved, try again
        let cmd = parse_multisig_cmd(&["approve", &wid, "1"]).unwrap();
        assert_eq!(ex.execute_cmd(cmd).unwrap_err(), ExecError::AlreadyApproved);
    }

    #[test]
    fn exec_wallet_not_found() {
        let mut ex = make_executor();
        let cmd = parse_multisig_cmd(&["info", "nonexistent"]).unwrap();
        assert_eq!(ex.execute_cmd(cmd).unwrap_err(), ExecError::WalletNotFound("nonexistent".into()));
    }

    #[test]
    fn exec_help() {
        let mut ex = make_executor();
        let cmd = parse_multisig_cmd(&["help"]).unwrap();
        match ex.execute_cmd(cmd).unwrap() {
            MultisigResult::HelpText(t) => assert!(t.contains("multisig")),
            _ => panic!(),
        }
    }

    #[test]
    fn display_formats() {
        let r = MultisigResult::WalletCreated { wallet_id: "w".into(), owners: 3, threshold: 2 };
        assert!(format!("{r}").contains("2-of-3"));

        let r = MultisigResult::ProposalExecuted { wallet: "w".into(), proposal_id: 1, target: "t".into(), value: 100 };
        assert!(format!("{r}").contains("100"));
    }
}
