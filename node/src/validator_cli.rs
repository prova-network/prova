//! Validator CLI Commands (NODE-028b)
//!
//! CLI subcommands for validator set management.
//! Integrates with the validator set manager (CHAIN-034).
//!
//! Subcommands:
//!   register <stake> [--capacity <ops>]
//!   exit [--force]
//!   status [<address>] [--json]
//!   list [--active-only] [--json]
//!   stake add <amount>
//!   stake withdraw (after unbonding)
//!   epoch [--history <n>]
//!   help

use std::fmt;

// ── Subcommand Definitions ──────────────────────────────────────────

#[derive(Debug, Clone, PartialEq)]
pub enum ValidatorCmd {
    /// Register as a validator candidate with initial stake.
    Register {
        stake: u64,
        capacity: u64,
    },
    /// Begin voluntary exit (starts unbonding period).
    Exit {
        force: bool,
    },
    /// Complete exit after unbonding — withdraw remaining stake.
    Withdraw,
    /// Show status of a validator (self or by address).
    Status {
        address: Option<String>,
        json: bool,
    },
    /// List validators in the current set.
    List {
        active_only: bool,
        json: bool,
    },
    /// Add stake to your validator.
    StakeAdd {
        amount: u64,
    },
    /// Show current epoch info and optionally history.
    Epoch {
        history: usize,
    },
    /// Show help for validator subcommands.
    Help,
}

// ── Parse Errors ────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq)]
pub enum ValidatorParseError {
    NoSubcommand,
    UnknownSubcommand(String),
    MissingArg(String),
    InvalidValue(String, String),
}

impl fmt::Display for ValidatorParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NoSubcommand => write!(f, "no validator subcommand (try 'validator help')"),
            Self::UnknownSubcommand(s) => write!(f, "unknown validator subcommand: '{s}'"),
            Self::MissingArg(a) => write!(f, "missing required argument: {a}"),
            Self::InvalidValue(k, v) => write!(f, "invalid value for {k}: '{v}'"),
        }
    }
}

// ── Parser ──────────────────────────────────────────────────────────

pub fn parse_validator_cmd(args: &[&str]) -> Result<ValidatorCmd, ValidatorParseError> {
    if args.is_empty() {
        return Err(ValidatorParseError::NoSubcommand);
    }

    match args[0] {
        "register" => parse_register(&args[1..]),
        "exit" => parse_exit(&args[1..]),
        "withdraw" => Ok(ValidatorCmd::Withdraw),
        "status" => parse_status(&args[1..]),
        "list" => parse_list(&args[1..]),
        "stake" => parse_stake(&args[1..]),
        "epoch" => parse_epoch(&args[1..]),
        "help" | "--help" | "-h" => Ok(ValidatorCmd::Help),
        other => Err(ValidatorParseError::UnknownSubcommand(other.into())),
    }
}

fn parse_register(args: &[&str]) -> Result<ValidatorCmd, ValidatorParseError> {
    if args.is_empty() {
        return Err(ValidatorParseError::MissingArg("stake amount".into()));
    }
    let stake: u64 = args[0]
        .parse()
        .map_err(|_| ValidatorParseError::InvalidValue("stake".into(), args[0].into()))?;

    let mut capacity = 100; // default
    let mut i = 1;
    while i < args.len() {
        match args[i] {
            "--capacity" => {
                i += 1;
                if i >= args.len() {
                    return Err(ValidatorParseError::MissingArg("--capacity value".into()));
                }
                capacity = args[i].parse().map_err(|_| {
                    ValidatorParseError::InvalidValue("capacity".into(), args[i].into())
                })?;
            }
            _ => {}
        }
        i += 1;
    }

    Ok(ValidatorCmd::Register { stake, capacity })
}

fn parse_exit(args: &[&str]) -> Result<ValidatorCmd, ValidatorParseError> {
    let force = args.iter().any(|a| *a == "--force");
    Ok(ValidatorCmd::Exit { force })
}

fn parse_status(args: &[&str]) -> Result<ValidatorCmd, ValidatorParseError> {
    let mut address = None;
    let mut json = false;
    let mut i = 0;
    while i < args.len() {
        match args[i] {
            "--json" => json = true,
            s if !s.starts_with('-') && address.is_none() => address = Some(s.to_string()),
            _ => {}
        }
        i += 1;
    }
    Ok(ValidatorCmd::Status { address, json })
}

fn parse_list(args: &[&str]) -> Result<ValidatorCmd, ValidatorParseError> {
    let mut active_only = false;
    let mut json = false;
    for a in args {
        match *a {
            "--active-only" | "--active" => active_only = true,
            "--json" => json = true,
            _ => {}
        }
    }
    Ok(ValidatorCmd::List { active_only, json })
}

fn parse_stake(args: &[&str]) -> Result<ValidatorCmd, ValidatorParseError> {
    if args.is_empty() {
        return Err(ValidatorParseError::MissingArg("stake subcommand (add)".into()));
    }
    match args[0] {
        "add" => {
            if args.len() < 2 {
                return Err(ValidatorParseError::MissingArg("amount".into()));
            }
            let amount: u64 = args[1]
                .parse()
                .map_err(|_| ValidatorParseError::InvalidValue("amount".into(), args[1].into()))?;
            Ok(ValidatorCmd::StakeAdd { amount })
        }
        other => Err(ValidatorParseError::UnknownSubcommand(format!(
            "stake {other}"
        ))),
    }
}

fn parse_epoch(args: &[&str]) -> Result<ValidatorCmd, ValidatorParseError> {
    let mut history = 0;
    let mut i = 0;
    while i < args.len() {
        if args[i] == "--history" {
            i += 1;
            if i >= args.len() {
                return Err(ValidatorParseError::MissingArg("--history count".into()));
            }
            history = args[i].parse().map_err(|_| {
                ValidatorParseError::InvalidValue("history".into(), args[i].into())
            })?;
        }
        i += 1;
    }
    Ok(ValidatorCmd::Epoch { history })
}

// ── Execution layer ─────────────────────────────────────────────────
// Simulates RPC calls to the chain's ValidatorSet.

/// Validator info returned by status/list commands.
#[derive(Debug, Clone, PartialEq)]
pub struct ValidatorInfo {
    pub address: String,
    pub stake: u64,
    pub reputation: f64,
    pub status: String,
    pub capacity: u64,
    pub blocks_produced: u64,
    pub consecutive_misses: u64,
    pub score: f64,
}

/// Epoch summary returned by epoch command.
#[derive(Debug, Clone, PartialEq)]
pub struct EpochSummary {
    pub epoch: u64,
    pub active_count: usize,
    pub total_stake: u64,
}

/// Result of executing a validator command.
#[derive(Debug, Clone, PartialEq)]
pub enum ValidatorResult {
    Registered {
        stake: u64,
        capacity: u64,
    },
    ExitInitiated {
        ready_epoch: u64,
    },
    WithdrawComplete {
        returned_stake: u64,
    },
    StakeAdded {
        new_total: u64,
    },
    Status(ValidatorInfo),
    List(Vec<ValidatorInfo>),
    Epoch {
        current: EpochSummary,
        history: Vec<EpochSummary>,
    },
    Help(String),
}

impl fmt::Display for ValidatorResult {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Registered { stake, capacity } => {
                write!(f, "✓ Registered as validator candidate (stake: {stake}, capacity: {capacity} ops/s)")
            }
            Self::ExitInitiated { ready_epoch } => {
                write!(f, "✓ Exit initiated. Stake withdrawable at epoch {ready_epoch}")
            }
            Self::WithdrawComplete { returned_stake } => {
                write!(f, "✓ Withdraw complete. Returned: {returned_stake}")
            }
            Self::StakeAdded { new_total } => {
                write!(f, "✓ Stake added. New total: {new_total}")
            }
            Self::Status(info) => {
                write!(
                    f,
                    "Validator {}\n  Status: {}\n  Stake: {}\n  Reputation: {:.3}\n  Score: {:.4}\n  Capacity: {} ops/s\n  Blocks: {}\n  Misses: {}",
                    info.address, info.status, info.stake, info.reputation,
                    info.score, info.capacity, info.blocks_produced, info.consecutive_misses
                )
            }
            Self::List(validators) => {
                if validators.is_empty() {
                    return write!(f, "No validators found.");
                }
                writeln!(f, "{:<20} {:>10} {:>6} {:>8} {:>8}", "ADDRESS", "STAKE", "REP", "SCORE", "STATUS")?;
                writeln!(f, "{}", "-".repeat(56))?;
                for v in validators {
                    writeln!(
                        f,
                        "{:<20} {:>10} {:>6.3} {:>8.4} {:>8}",
                        truncate_addr(&v.address, 20),
                        v.stake,
                        v.reputation,
                        v.score,
                        v.status
                    )?;
                }
                Ok(())
            }
            Self::Epoch { current, history } => {
                writeln!(f, "Current epoch: {}", current.epoch)?;
                writeln!(f, "  Active validators: {}", current.active_count)?;
                writeln!(f, "  Total stake: {}", current.total_stake)?;
                if !history.is_empty() {
                    writeln!(f, "\nHistory:")?;
                    for h in history {
                        writeln!(f, "  Epoch {}: {} validators, {} stake", h.epoch, h.active_count, h.total_stake)?;
                    }
                }
                Ok(())
            }
            Self::Help(text) => write!(f, "{text}"),
        }
    }
}

fn truncate_addr(addr: &str, max: usize) -> String {
    if addr.len() <= max {
        addr.to_string()
    } else {
        let half = (max - 1) / 2;
        format!("{}…{}", &addr[..half], &addr[addr.len() - half..])
    }
}

// ── MockRpc: simulated node interaction for testing ─────────────────

use std::collections::HashMap;

/// Mock RPC backend wrapping a ValidatorSet-like state for CLI testing.
#[derive(Debug)]
pub struct MockValidatorRpc {
    validators: HashMap<String, ValidatorInfo>,
    current_epoch: u64,
    epoch_records: Vec<EpochSummary>,
    self_address: String,
}

impl MockValidatorRpc {
    pub fn new(self_address: &str) -> Self {
        Self {
            validators: HashMap::new(),
            current_epoch: 0,
            epoch_records: Vec::new(),
            self_address: self_address.to_string(),
        }
    }

    pub fn with_epoch(mut self, epoch: u64) -> Self {
        self.current_epoch = epoch;
        self
    }

    pub fn with_validator(mut self, info: ValidatorInfo) -> Self {
        self.validators.insert(info.address.clone(), info);
        self
    }

    pub fn with_epoch_record(mut self, summary: EpochSummary) -> Self {
        self.epoch_records.push(summary);
        self
    }

    pub fn execute(&mut self, cmd: &ValidatorCmd) -> Result<ValidatorResult, ValidatorExecError> {
        match cmd {
            ValidatorCmd::Register { stake, capacity } => {
                if self.validators.contains_key(&self.self_address) {
                    return Err(ValidatorExecError::AlreadyRegistered);
                }
                if *stake < 100_000 {
                    return Err(ValidatorExecError::InsufficientStake);
                }
                let info = ValidatorInfo {
                    address: self.self_address.clone(),
                    stake: *stake,
                    reputation: 0.5,
                    status: "candidate".into(),
                    capacity: *capacity,
                    blocks_produced: 0,
                    consecutive_misses: 0,
                    score: 0.0,
                };
                self.validators.insert(self.self_address.clone(), info);
                Ok(ValidatorResult::Registered {
                    stake: *stake,
                    capacity: *capacity,
                })
            }
            ValidatorCmd::Exit { force: _ } => {
                let v = self
                    .validators
                    .get_mut(&self.self_address)
                    .ok_or(ValidatorExecError::NotRegistered)?;
                if v.status == "unbonding" || v.status == "exited" {
                    return Err(ValidatorExecError::InvalidState(v.status.clone()));
                }
                let ready = self.current_epoch + 14;
                v.status = "unbonding".into();
                Ok(ValidatorResult::ExitInitiated { ready_epoch: ready })
            }
            ValidatorCmd::Withdraw => {
                let v = self
                    .validators
                    .get_mut(&self.self_address)
                    .ok_or(ValidatorExecError::NotRegistered)?;
                if v.status != "unbonding" && v.status != "ejected" {
                    return Err(ValidatorExecError::InvalidState(v.status.clone()));
                }
                let returned = v.stake;
                v.stake = 0;
                v.status = "exited".into();
                Ok(ValidatorResult::WithdrawComplete {
                    returned_stake: returned,
                })
            }
            ValidatorCmd::Status { address, json: _ } => {
                let addr = address.as_deref().unwrap_or(&self.self_address);
                let v = self
                    .validators
                    .get(addr)
                    .ok_or(ValidatorExecError::NotFound(addr.into()))?;
                Ok(ValidatorResult::Status(v.clone()))
            }
            ValidatorCmd::List {
                active_only,
                json: _,
            } => {
                let mut list: Vec<ValidatorInfo> = if *active_only {
                    self.validators
                        .values()
                        .filter(|v| v.status == "active")
                        .cloned()
                        .collect()
                } else {
                    self.validators.values().cloned().collect()
                };
                list.sort_by(|a, b| {
                    b.score
                        .partial_cmp(&a.score)
                        .unwrap_or(std::cmp::Ordering::Equal)
                });
                Ok(ValidatorResult::List(list))
            }
            ValidatorCmd::StakeAdd { amount } => {
                let v = self
                    .validators
                    .get_mut(&self.self_address)
                    .ok_or(ValidatorExecError::NotRegistered)?;
                if v.status == "exited" {
                    return Err(ValidatorExecError::InvalidState(v.status.clone()));
                }
                v.stake += amount;
                Ok(ValidatorResult::StakeAdded {
                    new_total: v.stake,
                })
            }
            ValidatorCmd::Epoch { history } => {
                let current = EpochSummary {
                    epoch: self.current_epoch,
                    active_count: self
                        .validators
                        .values()
                        .filter(|v| v.status == "active")
                        .count(),
                    total_stake: self
                        .validators
                        .values()
                        .filter(|v| v.status == "active")
                        .map(|v| v.stake)
                        .sum(),
                };
                let hist = if *history > 0 {
                    let skip = self.epoch_records.len().saturating_sub(*history);
                    self.epoch_records[skip..].to_vec()
                } else {
                    Vec::new()
                };
                Ok(ValidatorResult::Epoch {
                    current,
                    history: hist,
                })
            }
            ValidatorCmd::Help => Ok(ValidatorResult::Help(help_text())),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum ValidatorExecError {
    AlreadyRegistered,
    InsufficientStake,
    NotRegistered,
    NotFound(String),
    InvalidState(String),
    RpcError(String),
}

impl fmt::Display for ValidatorExecError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::AlreadyRegistered => write!(f, "already registered as validator"),
            Self::InsufficientStake => write!(f, "stake below minimum (100,000)"),
            Self::NotRegistered => write!(f, "not registered as validator"),
            Self::NotFound(a) => write!(f, "validator not found: {a}"),
            Self::InvalidState(s) => write!(f, "invalid state for operation: {s}"),
            Self::RpcError(e) => write!(f, "RPC error: {e}"),
        }
    }
}

fn help_text() -> String {
    "\
prova validator — Manage validator registration and status

SUBCOMMANDS:
  register <stake> [--capacity <ops>]   Register as validator candidate
  exit [--force]                         Begin voluntary exit (unbonding)
  withdraw                               Complete exit, withdraw stake
  status [<address>] [--json]            Show validator status
  list [--active-only] [--json]          List validators
  stake add <amount>                     Add stake to your validator
  epoch [--history <n>]                  Show epoch info
  help                                   Show this help

EXAMPLES:
  prova validator register 200000 --capacity 500
  prova validator status
  prova validator list --active-only --json
  prova validator stake add 50000
  prova validator exit
  prova validator epoch --history 10"
        .to_string()
}

// ── Tests ───────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    // ── Parser tests ────────────────────────────────────────────────

    #[test]
    fn test_parse_register_basic() {
        let cmd = parse_validator_cmd(&["register", "200000"]).unwrap();
        assert_eq!(cmd, ValidatorCmd::Register { stake: 200_000, capacity: 100 });
    }

    #[test]
    fn test_parse_register_with_capacity() {
        let cmd = parse_validator_cmd(&["register", "500000", "--capacity", "1000"]).unwrap();
        assert_eq!(cmd, ValidatorCmd::Register { stake: 500_000, capacity: 1000 });
    }

    #[test]
    fn test_parse_register_missing_stake() {
        let err = parse_validator_cmd(&["register"]).unwrap_err();
        assert_eq!(err, ValidatorParseError::MissingArg("stake amount".into()));
    }

    #[test]
    fn test_parse_register_invalid_stake() {
        let err = parse_validator_cmd(&["register", "abc"]).unwrap_err();
        assert!(matches!(err, ValidatorParseError::InvalidValue(..)));
    }

    #[test]
    fn test_parse_exit() {
        let cmd = parse_validator_cmd(&["exit"]).unwrap();
        assert_eq!(cmd, ValidatorCmd::Exit { force: false });
    }

    #[test]
    fn test_parse_exit_force() {
        let cmd = parse_validator_cmd(&["exit", "--force"]).unwrap();
        assert_eq!(cmd, ValidatorCmd::Exit { force: true });
    }

    #[test]
    fn test_parse_withdraw() {
        let cmd = parse_validator_cmd(&["withdraw"]).unwrap();
        assert_eq!(cmd, ValidatorCmd::Withdraw);
    }

    #[test]
    fn test_parse_status_no_addr() {
        let cmd = parse_validator_cmd(&["status"]).unwrap();
        assert_eq!(cmd, ValidatorCmd::Status { address: None, json: false });
    }

    #[test]
    fn test_parse_status_with_addr() {
        let cmd = parse_validator_cmd(&["status", "val_123"]).unwrap();
        assert_eq!(cmd, ValidatorCmd::Status { address: Some("val_123".into()), json: false });
    }

    #[test]
    fn test_parse_status_json() {
        let cmd = parse_validator_cmd(&["status", "--json"]).unwrap();
        assert_eq!(cmd, ValidatorCmd::Status { address: None, json: true });
    }

    #[test]
    fn test_parse_list_defaults() {
        let cmd = parse_validator_cmd(&["list"]).unwrap();
        assert_eq!(cmd, ValidatorCmd::List { active_only: false, json: false });
    }

    #[test]
    fn test_parse_list_active_json() {
        let cmd = parse_validator_cmd(&["list", "--active-only", "--json"]).unwrap();
        assert_eq!(cmd, ValidatorCmd::List { active_only: true, json: true });
    }

    #[test]
    fn test_parse_stake_add() {
        let cmd = parse_validator_cmd(&["stake", "add", "50000"]).unwrap();
        assert_eq!(cmd, ValidatorCmd::StakeAdd { amount: 50_000 });
    }

    #[test]
    fn test_parse_stake_add_missing_amount() {
        let err = parse_validator_cmd(&["stake", "add"]).unwrap_err();
        assert_eq!(err, ValidatorParseError::MissingArg("amount".into()));
    }

    #[test]
    fn test_parse_epoch_default() {
        let cmd = parse_validator_cmd(&["epoch"]).unwrap();
        assert_eq!(cmd, ValidatorCmd::Epoch { history: 0 });
    }

    #[test]
    fn test_parse_epoch_history() {
        let cmd = parse_validator_cmd(&["epoch", "--history", "5"]).unwrap();
        assert_eq!(cmd, ValidatorCmd::Epoch { history: 5 });
    }

    #[test]
    fn test_parse_help() {
        let cmd = parse_validator_cmd(&["help"]).unwrap();
        assert_eq!(cmd, ValidatorCmd::Help);
    }

    #[test]
    fn test_parse_no_subcommand() {
        let err = parse_validator_cmd(&[]).unwrap_err();
        assert_eq!(err, ValidatorParseError::NoSubcommand);
    }

    #[test]
    fn test_parse_unknown_subcommand() {
        let err = parse_validator_cmd(&["foobar"]).unwrap_err();
        assert_eq!(err, ValidatorParseError::UnknownSubcommand("foobar".into()));
    }

    // ── Execution tests ─────────────────────────────────────────────

    fn mock_rpc() -> MockValidatorRpc {
        MockValidatorRpc::new("my_validator").with_epoch(10)
    }

    #[test]
    fn test_exec_register() {
        let mut rpc = mock_rpc();
        let cmd = ValidatorCmd::Register { stake: 200_000, capacity: 500 };
        let res = rpc.execute(&cmd).unwrap();
        assert_eq!(res, ValidatorResult::Registered { stake: 200_000, capacity: 500 });
        assert!(rpc.validators.contains_key("my_validator"));
    }

    #[test]
    fn test_exec_register_duplicate() {
        let mut rpc = mock_rpc();
        rpc.execute(&ValidatorCmd::Register { stake: 200_000, capacity: 100 }).unwrap();
        let err = rpc.execute(&ValidatorCmd::Register { stake: 200_000, capacity: 100 }).unwrap_err();
        assert_eq!(err, ValidatorExecError::AlreadyRegistered);
    }

    #[test]
    fn test_exec_register_insufficient_stake() {
        let mut rpc = mock_rpc();
        let err = rpc.execute(&ValidatorCmd::Register { stake: 50_000, capacity: 100 }).unwrap_err();
        assert_eq!(err, ValidatorExecError::InsufficientStake);
    }

    #[test]
    fn test_exec_exit() {
        let mut rpc = mock_rpc();
        rpc.execute(&ValidatorCmd::Register { stake: 200_000, capacity: 100 }).unwrap();
        let res = rpc.execute(&ValidatorCmd::Exit { force: false }).unwrap();
        assert_eq!(res, ValidatorResult::ExitInitiated { ready_epoch: 24 }); // 10 + 14
    }

    #[test]
    fn test_exec_exit_not_registered() {
        let mut rpc = mock_rpc();
        let err = rpc.execute(&ValidatorCmd::Exit { force: false }).unwrap_err();
        assert_eq!(err, ValidatorExecError::NotRegistered);
    }

    #[test]
    fn test_exec_withdraw() {
        let mut rpc = mock_rpc();
        rpc.execute(&ValidatorCmd::Register { stake: 200_000, capacity: 100 }).unwrap();
        rpc.execute(&ValidatorCmd::Exit { force: false }).unwrap();
        let res = rpc.execute(&ValidatorCmd::Withdraw).unwrap();
        assert_eq!(res, ValidatorResult::WithdrawComplete { returned_stake: 200_000 });
    }

    #[test]
    fn test_exec_withdraw_wrong_state() {
        let mut rpc = mock_rpc();
        rpc.execute(&ValidatorCmd::Register { stake: 200_000, capacity: 100 }).unwrap();
        let err = rpc.execute(&ValidatorCmd::Withdraw).unwrap_err();
        assert_eq!(err, ValidatorExecError::InvalidState("candidate".into()));
    }

    #[test]
    fn test_exec_status_self() {
        let mut rpc = mock_rpc();
        rpc.execute(&ValidatorCmd::Register { stake: 200_000, capacity: 100 }).unwrap();
        let res = rpc.execute(&ValidatorCmd::Status { address: None, json: false }).unwrap();
        match res {
            ValidatorResult::Status(info) => {
                assert_eq!(info.address, "my_validator");
                assert_eq!(info.stake, 200_000);
            }
            _ => panic!("expected Status"),
        }
    }

    #[test]
    fn test_exec_status_other() {
        let info = ValidatorInfo {
            address: "other_val".into(),
            stake: 300_000,
            reputation: 0.8,
            status: "active".into(),
            capacity: 200,
            blocks_produced: 50,
            consecutive_misses: 0,
            score: 0.59,
        };
        let mut rpc = mock_rpc().with_validator(info);
        let res = rpc.execute(&ValidatorCmd::Status { address: Some("other_val".into()), json: false }).unwrap();
        match res {
            ValidatorResult::Status(v) => assert_eq!(v.stake, 300_000),
            _ => panic!("expected Status"),
        }
    }

    #[test]
    fn test_exec_status_not_found() {
        let mut rpc = mock_rpc();
        let err = rpc.execute(&ValidatorCmd::Status { address: Some("ghost".into()), json: false }).unwrap_err();
        assert_eq!(err, ValidatorExecError::NotFound("ghost".into()));
    }

    #[test]
    fn test_exec_list_all() {
        let v1 = ValidatorInfo {
            address: "v1".into(), stake: 200_000, reputation: 0.5,
            status: "active".into(), capacity: 100, blocks_produced: 10,
            consecutive_misses: 0, score: 0.85,
        };
        let v2 = ValidatorInfo {
            address: "v2".into(), stake: 150_000, reputation: 0.3,
            status: "candidate".into(), capacity: 80, blocks_produced: 0,
            consecutive_misses: 0, score: 0.44,
        };
        let mut rpc = mock_rpc().with_validator(v1).with_validator(v2);
        let res = rpc.execute(&ValidatorCmd::List { active_only: false, json: false }).unwrap();
        match res {
            ValidatorResult::List(list) => {
                assert_eq!(list.len(), 2);
                assert_eq!(list[0].address, "v1"); // higher score first
            }
            _ => panic!("expected List"),
        }
    }

    #[test]
    fn test_exec_list_active_only() {
        let v1 = ValidatorInfo {
            address: "v1".into(), stake: 200_000, reputation: 0.5,
            status: "active".into(), capacity: 100, blocks_produced: 10,
            consecutive_misses: 0, score: 0.85,
        };
        let v2 = ValidatorInfo {
            address: "v2".into(), stake: 150_000, reputation: 0.3,
            status: "candidate".into(), capacity: 80, blocks_produced: 0,
            consecutive_misses: 0, score: 0.44,
        };
        let mut rpc = mock_rpc().with_validator(v1).with_validator(v2);
        let res = rpc.execute(&ValidatorCmd::List { active_only: true, json: false }).unwrap();
        match res {
            ValidatorResult::List(list) => {
                assert_eq!(list.len(), 1);
                assert_eq!(list[0].address, "v1");
            }
            _ => panic!("expected List"),
        }
    }

    #[test]
    fn test_exec_stake_add() {
        let mut rpc = mock_rpc();
        rpc.execute(&ValidatorCmd::Register { stake: 200_000, capacity: 100 }).unwrap();
        let res = rpc.execute(&ValidatorCmd::StakeAdd { amount: 50_000 }).unwrap();
        assert_eq!(res, ValidatorResult::StakeAdded { new_total: 250_000 });
    }

    #[test]
    fn test_exec_stake_add_not_registered() {
        let mut rpc = mock_rpc();
        let err = rpc.execute(&ValidatorCmd::StakeAdd { amount: 50_000 }).unwrap_err();
        assert_eq!(err, ValidatorExecError::NotRegistered);
    }

    #[test]
    fn test_exec_epoch_no_history() {
        let mut rpc = mock_rpc();
        let res = rpc.execute(&ValidatorCmd::Epoch { history: 0 }).unwrap();
        match res {
            ValidatorResult::Epoch { current, history } => {
                assert_eq!(current.epoch, 10);
                assert!(history.is_empty());
            }
            _ => panic!("expected Epoch"),
        }
    }

    #[test]
    fn test_exec_epoch_with_history() {
        let mut rpc = mock_rpc()
            .with_epoch_record(EpochSummary { epoch: 8, active_count: 5, total_stake: 500_000 })
            .with_epoch_record(EpochSummary { epoch: 9, active_count: 6, total_stake: 600_000 });
        let res = rpc.execute(&ValidatorCmd::Epoch { history: 1 }).unwrap();
        match res {
            ValidatorResult::Epoch { history, .. } => {
                assert_eq!(history.len(), 1);
                assert_eq!(history[0].epoch, 9);
            }
            _ => panic!("expected Epoch"),
        }
    }

    #[test]
    fn test_exec_help() {
        let mut rpc = mock_rpc();
        let res = rpc.execute(&ValidatorCmd::Help).unwrap();
        match res {
            ValidatorResult::Help(text) => assert!(text.contains("register")),
            _ => panic!("expected Help"),
        }
    }

    #[test]
    fn test_display_registered() {
        let r = ValidatorResult::Registered { stake: 200_000, capacity: 500 };
        let s = format!("{r}");
        assert!(s.contains("200000"));
        assert!(s.contains("500"));
    }

    #[test]
    fn test_display_list_empty() {
        let r = ValidatorResult::List(vec![]);
        assert_eq!(format!("{r}"), "No validators found.");
    }

    #[test]
    fn test_truncate_addr_short() {
        assert_eq!(truncate_addr("abc", 20), "abc");
    }

    #[test]
    fn test_truncate_addr_long() {
        let long = "a".repeat(30);
        let t = truncate_addr(&long, 20);
        assert!(t.contains('…'));
        // Truncated output should be shorter than original
        assert!(t.chars().count() <= 20);
    }

    #[test]
    fn test_parse_stake_unknown_sub() {
        let err = parse_validator_cmd(&["stake", "remove"]).unwrap_err();
        assert!(matches!(err, ValidatorParseError::UnknownSubcommand(..)));
    }

    #[test]
    fn test_exec_exit_already_unbonding() {
        let mut rpc = mock_rpc();
        rpc.execute(&ValidatorCmd::Register { stake: 200_000, capacity: 100 }).unwrap();
        rpc.execute(&ValidatorCmd::Exit { force: false }).unwrap();
        let err = rpc.execute(&ValidatorCmd::Exit { force: false }).unwrap_err();
        assert_eq!(err, ValidatorExecError::InvalidState("unbonding".into()));
    }
}
