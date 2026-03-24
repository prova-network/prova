//! Delegation CLI Commands (NODE-030)
//!
//! CLI subcommands for managing stake delegation to inference providers.
//! Integrates with the delegation system (CHAIN-031) and liquid staking (CHAIN-032).
//!
//! Subcommands:
//!   delegate <provider> <amount> [--auto-compound]
//!   undelegate <provider> <amount>
//!   redelegate <from-provider> <to-provider> <amount>
//!   rewards [--provider <addr>] [--claim]
//!   list [--provider <addr>] [--unbonding]
//!   providers [--active-only]

use std::fmt;

// Re-use types compatible with delegation module
pub type Amount = u64;
pub type Address = [u8; 32];

// ── Subcommand Definitions ──────────────────────────────────────────

#[derive(Debug, Clone, PartialEq)]
pub enum DelegationCmd {
    /// Delegate stake to an inference provider.
    Delegate {
        provider: String,
        amount: Amount,
        auto_compound: bool,
    },
    /// Undelegate (begin unbonding) from a provider.
    Undelegate { provider: String, amount: Amount },
    /// Redelegate from one provider to another without full unbonding.
    Redelegate {
        from_provider: String,
        to_provider: String,
        amount: Amount,
    },
    /// Query or claim delegation rewards.
    Rewards {
        provider: Option<String>,
        claim: bool,
    },
    /// List active delegations and optionally unbonding entries.
    List {
        provider: Option<String>,
        unbonding: bool,
    },
    /// List available providers accepting delegations.
    Providers { active_only: bool },
    /// Show help for delegation subcommands.
    Help,
}

// ── Parse Errors ────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq)]
pub enum DelegationParseError {
    NoSubcommand,
    UnknownSubcommand(String),
    MissingArg(String),
    InvalidAmount(String),
    UnknownFlag(String),
}

impl fmt::Display for DelegationParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NoSubcommand => write!(f, "no delegation subcommand (try 'delegate help')"),
            Self::UnknownSubcommand(s) => write!(f, "unknown delegation subcommand: '{s}'"),
            Self::MissingArg(a) => write!(f, "missing required argument: {a}"),
            Self::InvalidAmount(v) => write!(f, "invalid amount: '{v}'"),
            Self::UnknownFlag(fl) => write!(f, "unknown flag: '{fl}'"),
        }
    }
}

// ── Parser ──────────────────────────────────────────────────────────

pub fn parse_delegation(args: &[String]) -> Result<DelegationCmd, DelegationParseError> {
    if args.is_empty() {
        return Err(DelegationParseError::NoSubcommand);
    }

    match args[0].as_str() {
        "delegate" => parse_delegate(&args[1..]),
        "undelegate" => parse_undelegate(&args[1..]),
        "redelegate" => parse_redelegate(&args[1..]),
        "rewards" => parse_rewards(&args[1..]),
        "list" => parse_list(&args[1..]),
        "providers" => parse_providers(&args[1..]),
        "help" | "--help" | "-h" => Ok(DelegationCmd::Help),
        other => Err(DelegationParseError::UnknownSubcommand(other.into())),
    }
}

fn parse_amount(s: &str) -> Result<Amount, DelegationParseError> {
    // Support decimal notation: "1.5" -> 1_500_000 (6 decimal places)
    if let Some(dot_pos) = s.find('.') {
        let integer_part: u64 = s[..dot_pos]
            .parse()
            .map_err(|_| DelegationParseError::InvalidAmount(s.into()))?;
        let frac_str = &s[dot_pos + 1..];
        if frac_str.len() > 6 {
            return Err(DelegationParseError::InvalidAmount(s.into()));
        }
        let padded = format!("{:0<6}", frac_str);
        let frac_part: u64 = padded
            .parse()
            .map_err(|_| DelegationParseError::InvalidAmount(s.into()))?;
        Ok(integer_part * 1_000_000 + frac_part)
    } else {
        let v: u64 = s
            .parse()
            .map_err(|_| DelegationParseError::InvalidAmount(s.into()))?;
        Ok(v * 1_000_000)
    }
}

fn parse_delegate(args: &[String]) -> Result<DelegationCmd, DelegationParseError> {
    let mut provider: Option<String> = None;
    let mut amount: Option<Amount> = None;
    let mut auto_compound = false;
    let mut i = 0;

    while i < args.len() {
        match args[i].as_str() {
            "--auto-compound" => auto_compound = true,
            s if s.starts_with('-') => {
                return Err(DelegationParseError::UnknownFlag(s.into()));
            }
            _ => {
                if provider.is_none() {
                    provider = Some(args[i].clone());
                } else if amount.is_none() {
                    amount = Some(parse_amount(&args[i])?);
                }
            }
        }
        i += 1;
    }

    Ok(DelegationCmd::Delegate {
        provider: provider.ok_or(DelegationParseError::MissingArg("provider".into()))?,
        amount: amount.ok_or(DelegationParseError::MissingArg("amount".into()))?,
        auto_compound,
    })
}

fn parse_undelegate(args: &[String]) -> Result<DelegationCmd, DelegationParseError> {
    let provider = args
        .first()
        .ok_or(DelegationParseError::MissingArg("provider".into()))?
        .clone();
    let amount_str = args
        .get(1)
        .ok_or(DelegationParseError::MissingArg("amount".into()))?;
    let amount = parse_amount(amount_str)?;
    Ok(DelegationCmd::Undelegate { provider, amount })
}

fn parse_redelegate(args: &[String]) -> Result<DelegationCmd, DelegationParseError> {
    let from = args
        .first()
        .ok_or(DelegationParseError::MissingArg("from-provider".into()))?
        .clone();
    let to = args
        .get(1)
        .ok_or(DelegationParseError::MissingArg("to-provider".into()))?
        .clone();
    let amount_str = args
        .get(2)
        .ok_or(DelegationParseError::MissingArg("amount".into()))?;
    let amount = parse_amount(amount_str)?;
    Ok(DelegationCmd::Redelegate {
        from_provider: from,
        to_provider: to,
        amount,
    })
}

fn parse_rewards(args: &[String]) -> Result<DelegationCmd, DelegationParseError> {
    let mut provider: Option<String> = None;
    let mut claim = false;
    let mut i = 0;

    while i < args.len() {
        match args[i].as_str() {
            "--claim" => claim = true,
            "--provider" => {
                i += 1;
                provider = Some(
                    args.get(i)
                        .ok_or(DelegationParseError::MissingArg("provider address".into()))?
                        .clone(),
                );
            }
            s if s.starts_with('-') => {
                return Err(DelegationParseError::UnknownFlag(s.into()));
            }
            _ => {}
        }
        i += 1;
    }

    Ok(DelegationCmd::Rewards { provider, claim })
}

fn parse_list(args: &[String]) -> Result<DelegationCmd, DelegationParseError> {
    let mut provider: Option<String> = None;
    let mut unbonding = false;
    let mut i = 0;

    while i < args.len() {
        match args[i].as_str() {
            "--unbonding" => unbonding = true,
            "--provider" => {
                i += 1;
                provider = Some(
                    args.get(i)
                        .ok_or(DelegationParseError::MissingArg("provider address".into()))?
                        .clone(),
                );
            }
            s if s.starts_with('-') => {
                return Err(DelegationParseError::UnknownFlag(s.into()));
            }
            _ => {}
        }
        i += 1;
    }

    Ok(DelegationCmd::List {
        provider,
        unbonding,
    })
}

fn parse_providers(args: &[String]) -> Result<DelegationCmd, DelegationParseError> {
    let mut active_only = false;
    for arg in args {
        match arg.as_str() {
            "--active-only" => active_only = true,
            s if s.starts_with('-') => {
                return Err(DelegationParseError::UnknownFlag(s.into()));
            }
            _ => {}
        }
    }
    Ok(DelegationCmd::Providers { active_only })
}

// ── Execution Engine ────────────────────────────────────────────────

/// Result of executing a delegation CLI command.
#[derive(Debug, Clone, PartialEq)]
pub struct DelegationResult {
    pub success: bool,
    pub message: String,
    pub details: Vec<DelegationDetail>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct DelegationDetail {
    pub provider: String,
    pub delegated: Amount,
    pub rewards: Amount,
    pub unbonding: Amount,
    pub auto_compound: bool,
    pub st_balance: Amount,
}

/// Format amount with 6 decimal places.
pub fn format_amount(amount: Amount) -> String {
    let whole = amount / 1_000_000;
    let frac = amount % 1_000_000;
    if frac == 0 {
        format!("{}.0", whole)
    } else {
        let frac_str = format!("{:06}", frac).trim_end_matches('0').to_string();
        format!("{}.{}", whole, frac_str)
    }
}

/// Execute a delegation command against the given state.
/// In production this would make RPC calls; here we operate on in-memory state.
pub fn execute_delegation(
    cmd: &DelegationCmd,
    state: &mut DelegationState,
) -> Result<DelegationResult, String> {
    match cmd {
        DelegationCmd::Delegate {
            provider,
            amount,
            auto_compound,
        } => {
            if *amount == 0 {
                return Err("amount must be non-zero".into());
            }
            let st_minted = state.mint_st_tokens(provider, *amount);
            let entry = state
                .delegations
                .entry(provider.clone())
                .or_insert(DelegationEntry::default());
            entry.delegated += amount;
            entry.auto_compound = *auto_compound;
            entry.st_balance += st_minted;
            Ok(DelegationResult {
                success: true,
                message: format!(
                    "delegated {} PROVA to {} (stPROVA: {})",
                    format_amount(*amount),
                    provider,
                    format_amount(st_minted)
                ),
                details: vec![entry.to_detail(provider)],
            })
        }
        DelegationCmd::Undelegate { provider, amount } => {
            {
                let entry = state
                    .delegations
                    .get(provider)
                    .ok_or_else(|| format!("no delegation to provider {}", provider))?;
                if *amount > entry.delegated {
                    return Err("insufficient delegation balance".into());
                }
            }
            let st_burned = state.burn_st_tokens(provider, *amount);
            let entry = state.delegations.get_mut(provider).unwrap();
            entry.delegated -= amount;
            entry.unbonding += amount;
            entry.st_balance = entry.st_balance.saturating_sub(st_burned);
            Ok(DelegationResult {
                success: true,
                message: format!(
                    "undelegating {} PROVA from {} (unbonding period: 14400 epochs)",
                    format_amount(*amount),
                    provider
                ),
                details: vec![entry.to_detail(provider)],
            })
        }
        DelegationCmd::Redelegate {
            from_provider,
            to_provider,
            amount,
        } => {
            if from_provider == to_provider {
                return Err("cannot redelegate to same provider".into());
            }
            // Validate
            {
                let from = state
                    .delegations
                    .get(from_provider)
                    .ok_or_else(|| format!("no delegation to provider {}", from_provider))?;
                if *amount > from.delegated {
                    return Err("insufficient delegation balance".into());
                }
            }
            // Compute tokens before mutating
            let st_burned = state.burn_st_tokens(from_provider, *amount);
            let st_minted = state.mint_st_tokens(to_provider, *amount);

            // Remove from source
            let from = state.delegations.get_mut(from_provider).unwrap();
            from.delegated -= amount;
            from.st_balance = from.st_balance.saturating_sub(st_burned);

            // Add to destination
            let to = state
                .delegations
                .entry(to_provider.clone())
                .or_insert(DelegationEntry::default());
            to.delegated += amount;
            to.st_balance += st_minted;

            Ok(DelegationResult {
                success: true,
                message: format!(
                    "redelegated {} PROVA from {} to {}",
                    format_amount(*amount),
                    from_provider,
                    to_provider
                ),
                details: vec![],
            })
        }
        DelegationCmd::Rewards { provider, claim } => {
            let mut details = Vec::new();
            let mut total_rewards: Amount = 0;

            let providers: Vec<String> = match provider {
                Some(p) => vec![p.clone()],
                None => state.delegations.keys().cloned().collect(),
            };

            for p in &providers {
                if let Some(entry) = state.delegations.get(p) {
                    total_rewards += entry.rewards;
                    details.push(entry.to_detail(p));
                }
            }

            if *claim && total_rewards > 0 {
                for p in &providers {
                    if let Some(entry) = state.delegations.get_mut(p) {
                        entry.rewards = 0;
                    }
                }
                Ok(DelegationResult {
                    success: true,
                    message: format!("claimed {} PROVA in rewards", format_amount(total_rewards)),
                    details,
                })
            } else {
                Ok(DelegationResult {
                    success: true,
                    message: format!("pending rewards: {} PROVA", format_amount(total_rewards)),
                    details,
                })
            }
        }
        DelegationCmd::List {
            provider,
            unbonding,
        } => {
            let entries: Vec<(String, &DelegationEntry)> = match provider {
                Some(p) => state
                    .delegations
                    .get(p)
                    .map(|e| vec![(p.clone(), e)])
                    .unwrap_or_default(),
                None => state
                    .delegations
                    .iter()
                    .map(|(k, v)| (k.clone(), v))
                    .collect(),
            };

            let mut details: Vec<DelegationDetail> = entries
                .iter()
                .filter(|(_, e)| !*unbonding || e.unbonding > 0)
                .map(|(p, e)| e.to_detail(p))
                .collect();

            details.sort_by(|a, b| b.delegated.cmp(&a.delegated));

            Ok(DelegationResult {
                success: true,
                message: format!("{} delegation(s) found", details.len()),
                details,
            })
        }
        DelegationCmd::Providers { active_only } => {
            let providers: Vec<DelegationDetail> = state
                .providers
                .iter()
                .filter(|(_, p)| !*active_only || p.accepting)
                .map(|(addr, p)| DelegationDetail {
                    provider: addr.clone(),
                    delegated: p.total_delegated,
                    rewards: 0,
                    unbonding: 0,
                    auto_compound: false,
                    st_balance: 0,
                })
                .collect();

            Ok(DelegationResult {
                success: true,
                message: format!("{} provider(s) found", providers.len()),
                details: providers,
            })
        }
        DelegationCmd::Help => Ok(DelegationResult {
            success: true,
            message: HELP_TEXT.to_string(),
            details: vec![],
        }),
    }
}

const HELP_TEXT: &str = "\
USAGE: prova delegate <subcommand> [options]

SUBCOMMANDS:
    delegate <provider> <amount> [--auto-compound]
        Delegate stake to an inference provider
    undelegate <provider> <amount>
        Begin unbonding from a provider (14400 epoch cooldown)
    redelegate <from> <to> <amount>
        Move delegation between providers without full unbonding
    rewards [--provider <addr>] [--claim]
        Query pending rewards, optionally claim them
    list [--provider <addr>] [--unbonding]
        List active delegations
    providers [--active-only]
        List inference providers accepting delegations
    help
        Show this help message";

// ── In-memory State (for testing / local execution) ─────────────────

use std::collections::HashMap;

#[derive(Debug, Clone, Default)]
pub struct DelegationEntry {
    pub delegated: Amount,
    pub unbonding: Amount,
    pub rewards: Amount,
    pub auto_compound: bool,
    pub st_balance: Amount,
}

impl DelegationEntry {
    fn to_detail(&self, provider: &str) -> DelegationDetail {
        DelegationDetail {
            provider: provider.to_string(),
            delegated: self.delegated,
            rewards: self.rewards,
            unbonding: self.unbonding,
            auto_compound: self.auto_compound,
            st_balance: self.st_balance,
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct ProviderEntry {
    pub total_delegated: Amount,
    pub commission_bps: u16,
    pub accepting: bool,
}

#[derive(Debug, Clone, Default)]
pub struct DelegationState {
    pub delegations: HashMap<String, DelegationEntry>,
    pub providers: HashMap<String, ProviderEntry>,
    /// stPROVA exchange rates per provider (basis: 1_000_000 = 1:1)
    pub exchange_rates: HashMap<String, u64>,
}

impl DelegationState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register_provider(&mut self, address: &str, commission_bps: u16) {
        self.providers.insert(
            address.to_string(),
            ProviderEntry {
                total_delegated: 0,
                commission_bps,
                accepting: true,
            },
        );
        self.exchange_rates.insert(address.to_string(), 1_000_000); // 1:1 initial rate
    }

    pub fn add_rewards(&mut self, provider: &str, amount: Amount) {
        if let Some(entry) = self.delegations.get_mut(provider) {
            if entry.auto_compound {
                entry.delegated += amount;
                // Increase exchange rate to reflect compounding
                if let Some(rate) = self.exchange_rates.get_mut(provider) {
                    if entry.delegated > 0 {
                        *rate = (entry.delegated as u128 * 1_000_000
                            / (entry.delegated - amount).max(1) as u128)
                            as u64;
                    }
                }
            } else {
                entry.rewards += amount;
            }
        }
    }

    fn mint_st_tokens(&self, provider: &str, amount: Amount) -> Amount {
        let rate = self
            .exchange_rates
            .get(provider)
            .copied()
            .unwrap_or(1_000_000);
        if rate == 0 {
            return 0;
        }
        (amount as u128 * 1_000_000 / rate as u128) as Amount
    }

    fn burn_st_tokens(&self, provider: &str, amount: Amount) -> Amount {
        let rate = self
            .exchange_rates
            .get(provider)
            .copied()
            .unwrap_or(1_000_000);
        (amount as u128 * 1_000_000 / rate as u128) as Amount
    }
}

// ── Tests ───────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn args(s: &str) -> Vec<String> {
        s.split_whitespace().map(String::from).collect()
    }

    // ── Parsing Tests ───────────────────────────────────────────────

    #[test]
    fn test_parse_delegate_basic() {
        let cmd = parse_delegation(&args("delegate provider1 100")).unwrap();
        assert_eq!(
            cmd,
            DelegationCmd::Delegate {
                provider: "provider1".into(),
                amount: 100_000_000,
                auto_compound: false,
            }
        );
    }

    #[test]
    fn test_parse_delegate_auto_compound() {
        let cmd = parse_delegation(&args("delegate prov1 50 --auto-compound")).unwrap();
        assert_eq!(
            cmd,
            DelegationCmd::Delegate {
                provider: "prov1".into(),
                amount: 50_000_000,
                auto_compound: true,
            }
        );
    }

    #[test]
    fn test_parse_delegate_decimal() {
        let cmd = parse_delegation(&args("delegate prov1 1.5")).unwrap();
        assert_eq!(
            cmd,
            DelegationCmd::Delegate {
                provider: "prov1".into(),
                amount: 1_500_000,
                auto_compound: false,
            }
        );
    }

    #[test]
    fn test_parse_undelegate() {
        let cmd = parse_delegation(&args("undelegate prov1 25")).unwrap();
        assert_eq!(
            cmd,
            DelegationCmd::Undelegate {
                provider: "prov1".into(),
                amount: 25_000_000,
            }
        );
    }

    #[test]
    fn test_parse_redelegate() {
        let cmd = parse_delegation(&args("redelegate prov1 prov2 10")).unwrap();
        assert_eq!(
            cmd,
            DelegationCmd::Redelegate {
                from_provider: "prov1".into(),
                to_provider: "prov2".into(),
                amount: 10_000_000,
            }
        );
    }

    #[test]
    fn test_parse_rewards_claim() {
        let cmd = parse_delegation(&args("rewards --provider prov1 --claim")).unwrap();
        assert_eq!(
            cmd,
            DelegationCmd::Rewards {
                provider: Some("prov1".into()),
                claim: true,
            }
        );
    }

    #[test]
    fn test_parse_rewards_no_args() {
        let cmd = parse_delegation(&args("rewards")).unwrap();
        assert_eq!(
            cmd,
            DelegationCmd::Rewards {
                provider: None,
                claim: false,
            }
        );
    }

    #[test]
    fn test_parse_list_unbonding() {
        let cmd = parse_delegation(&args("list --unbonding")).unwrap();
        assert_eq!(
            cmd,
            DelegationCmd::List {
                provider: None,
                unbonding: true,
            }
        );
    }

    #[test]
    fn test_parse_providers_active() {
        let cmd = parse_delegation(&args("providers --active-only")).unwrap();
        assert_eq!(cmd, DelegationCmd::Providers { active_only: true });
    }

    #[test]
    fn test_parse_help() {
        let cmd = parse_delegation(&args("help")).unwrap();
        assert_eq!(cmd, DelegationCmd::Help);
    }

    #[test]
    fn test_parse_no_subcommand() {
        let result = parse_delegation(&[]);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_unknown_subcommand() {
        let result = parse_delegation(&args("stake prov1 100"));
        assert_eq!(
            result,
            Err(DelegationParseError::UnknownSubcommand("stake".into()))
        );
    }

    #[test]
    fn test_parse_missing_amount() {
        let result = parse_delegation(&args("delegate prov1"));
        assert_eq!(
            result,
            Err(DelegationParseError::MissingArg("amount".into()))
        );
    }

    #[test]
    fn test_parse_invalid_amount() {
        let result = parse_delegation(&args("delegate prov1 abc"));
        assert!(matches!(
            result,
            Err(DelegationParseError::InvalidAmount(_))
        ));
    }

    #[test]
    fn test_parse_unknown_flag() {
        let result = parse_delegation(&args("delegate prov1 100 --verbose"));
        assert_eq!(
            result,
            Err(DelegationParseError::UnknownFlag("--verbose".into()))
        );
    }

    // ── Execution Tests ─────────────────────────────────────────────

    #[test]
    fn test_execute_delegate() {
        let mut state = DelegationState::new();
        state.register_provider("prov1", 500);
        let cmd = DelegationCmd::Delegate {
            provider: "prov1".into(),
            amount: 10_000_000,
            auto_compound: false,
        };
        let result = execute_delegation(&cmd, &mut state).unwrap();
        assert!(result.success);
        assert_eq!(state.delegations["prov1"].delegated, 10_000_000);
        assert_eq!(state.delegations["prov1"].st_balance, 10_000_000); // 1:1 rate
    }

    #[test]
    fn test_execute_undelegate() {
        let mut state = DelegationState::new();
        state.register_provider("prov1", 500);
        let _ = execute_delegation(
            &DelegationCmd::Delegate {
                provider: "prov1".into(),
                amount: 10_000_000,
                auto_compound: false,
            },
            &mut state,
        );
        let result = execute_delegation(
            &DelegationCmd::Undelegate {
                provider: "prov1".into(),
                amount: 4_000_000,
            },
            &mut state,
        )
        .unwrap();
        assert!(result.success);
        assert_eq!(state.delegations["prov1"].delegated, 6_000_000);
        assert_eq!(state.delegations["prov1"].unbonding, 4_000_000);
    }

    #[test]
    fn test_execute_undelegate_insufficient() {
        let mut state = DelegationState::new();
        state.register_provider("prov1", 500);
        let _ = execute_delegation(
            &DelegationCmd::Delegate {
                provider: "prov1".into(),
                amount: 5_000_000,
                auto_compound: false,
            },
            &mut state,
        );
        let result = execute_delegation(
            &DelegationCmd::Undelegate {
                provider: "prov1".into(),
                amount: 10_000_000,
            },
            &mut state,
        );
        assert!(result.is_err());
    }

    #[test]
    fn test_execute_redelegate() {
        let mut state = DelegationState::new();
        state.register_provider("prov1", 500);
        state.register_provider("prov2", 300);
        let _ = execute_delegation(
            &DelegationCmd::Delegate {
                provider: "prov1".into(),
                amount: 20_000_000,
                auto_compound: false,
            },
            &mut state,
        );
        let result = execute_delegation(
            &DelegationCmd::Redelegate {
                from_provider: "prov1".into(),
                to_provider: "prov2".into(),
                amount: 8_000_000,
            },
            &mut state,
        )
        .unwrap();
        assert!(result.success);
        assert_eq!(state.delegations["prov1"].delegated, 12_000_000);
        assert_eq!(state.delegations["prov2"].delegated, 8_000_000);
    }

    #[test]
    fn test_execute_redelegate_same_provider() {
        let mut state = DelegationState::new();
        state.register_provider("prov1", 500);
        let _ = execute_delegation(
            &DelegationCmd::Delegate {
                provider: "prov1".into(),
                amount: 10_000_000,
                auto_compound: false,
            },
            &mut state,
        );
        let result = execute_delegation(
            &DelegationCmd::Redelegate {
                from_provider: "prov1".into(),
                to_provider: "prov1".into(),
                amount: 5_000_000,
            },
            &mut state,
        );
        assert!(result.is_err());
    }

    #[test]
    fn test_execute_rewards_query() {
        let mut state = DelegationState::new();
        state.register_provider("prov1", 500);
        let _ = execute_delegation(
            &DelegationCmd::Delegate {
                provider: "prov1".into(),
                amount: 10_000_000,
                auto_compound: false,
            },
            &mut state,
        );
        state.add_rewards("prov1", 500_000);
        let result = execute_delegation(
            &DelegationCmd::Rewards {
                provider: None,
                claim: false,
            },
            &mut state,
        )
        .unwrap();
        assert!(result.message.contains("0.5"));
        assert_eq!(state.delegations["prov1"].rewards, 500_000);
    }

    #[test]
    fn test_execute_rewards_claim() {
        let mut state = DelegationState::new();
        state.register_provider("prov1", 500);
        let _ = execute_delegation(
            &DelegationCmd::Delegate {
                provider: "prov1".into(),
                amount: 10_000_000,
                auto_compound: false,
            },
            &mut state,
        );
        state.add_rewards("prov1", 1_000_000);
        let result = execute_delegation(
            &DelegationCmd::Rewards {
                provider: None,
                claim: true,
            },
            &mut state,
        )
        .unwrap();
        assert!(result.message.contains("claimed"));
        assert_eq!(state.delegations["prov1"].rewards, 0);
    }

    #[test]
    fn test_execute_auto_compound() {
        let mut state = DelegationState::new();
        state.register_provider("prov1", 500);
        let _ = execute_delegation(
            &DelegationCmd::Delegate {
                provider: "prov1".into(),
                amount: 10_000_000,
                auto_compound: true,
            },
            &mut state,
        );
        state.add_rewards("prov1", 500_000);
        // Auto-compound adds rewards to delegated, not to rewards
        assert_eq!(state.delegations["prov1"].delegated, 10_500_000);
        assert_eq!(state.delegations["prov1"].rewards, 0);
    }

    #[test]
    fn test_execute_list_all() {
        let mut state = DelegationState::new();
        state.register_provider("prov1", 500);
        state.register_provider("prov2", 300);
        let _ = execute_delegation(
            &DelegationCmd::Delegate {
                provider: "prov1".into(),
                amount: 10_000_000,
                auto_compound: false,
            },
            &mut state,
        );
        let _ = execute_delegation(
            &DelegationCmd::Delegate {
                provider: "prov2".into(),
                amount: 5_000_000,
                auto_compound: false,
            },
            &mut state,
        );
        let result = execute_delegation(
            &DelegationCmd::List {
                provider: None,
                unbonding: false,
            },
            &mut state,
        )
        .unwrap();
        assert_eq!(result.details.len(), 2);
    }

    #[test]
    fn test_execute_list_unbonding_filter() {
        let mut state = DelegationState::new();
        state.register_provider("prov1", 500);
        state.register_provider("prov2", 300);
        let _ = execute_delegation(
            &DelegationCmd::Delegate {
                provider: "prov1".into(),
                amount: 10_000_000,
                auto_compound: false,
            },
            &mut state,
        );
        let _ = execute_delegation(
            &DelegationCmd::Delegate {
                provider: "prov2".into(),
                amount: 5_000_000,
                auto_compound: false,
            },
            &mut state,
        );
        let _ = execute_delegation(
            &DelegationCmd::Undelegate {
                provider: "prov1".into(),
                amount: 3_000_000,
            },
            &mut state,
        );
        let result = execute_delegation(
            &DelegationCmd::List {
                provider: None,
                unbonding: true,
            },
            &mut state,
        )
        .unwrap();
        assert_eq!(result.details.len(), 1);
        assert_eq!(result.details[0].provider, "prov1");
    }

    #[test]
    fn test_execute_providers() {
        let mut state = DelegationState::new();
        state.register_provider("prov1", 500);
        state.providers.insert(
            "prov_inactive".into(),
            ProviderEntry {
                total_delegated: 0,
                commission_bps: 1000,
                accepting: false,
            },
        );
        let result =
            execute_delegation(&DelegationCmd::Providers { active_only: true }, &mut state)
                .unwrap();
        assert_eq!(result.details.len(), 1);

        let result_all =
            execute_delegation(&DelegationCmd::Providers { active_only: false }, &mut state)
                .unwrap();
        assert_eq!(result_all.details.len(), 2);
    }

    #[test]
    fn test_execute_delegate_zero() {
        let mut state = DelegationState::new();
        let result = execute_delegation(
            &DelegationCmd::Delegate {
                provider: "prov1".into(),
                amount: 0,
                auto_compound: false,
            },
            &mut state,
        );
        assert!(result.is_err());
    }

    #[test]
    fn test_execute_help() {
        let mut state = DelegationState::new();
        let result = execute_delegation(&DelegationCmd::Help, &mut state).unwrap();
        assert!(result.message.contains("USAGE"));
    }

    #[test]
    fn test_format_amount() {
        assert_eq!(format_amount(1_000_000), "1.0");
        assert_eq!(format_amount(1_500_000), "1.5");
        assert_eq!(format_amount(10_123_456), "10.123456");
        assert_eq!(format_amount(0), "0.0");
        assert_eq!(format_amount(100), "0.0001");
    }

    #[test]
    fn test_st_token_minting() {
        let mut state = DelegationState::new();
        state.register_provider("prov1", 500);
        // First delegation at 1:1
        let _ = execute_delegation(
            &DelegationCmd::Delegate {
                provider: "prov1".into(),
                amount: 10_000_000,
                auto_compound: false,
            },
            &mut state,
        );
        assert_eq!(state.delegations["prov1"].st_balance, 10_000_000);
    }

    #[test]
    fn test_parse_decimal_edge_cases() {
        // "0.1" = 100_000
        let cmd = parse_delegation(&args("delegate prov1 0.1")).unwrap();
        match cmd {
            DelegationCmd::Delegate { amount, .. } => assert_eq!(amount, 100_000),
            _ => panic!("wrong variant"),
        }
        // "0.000001" = 1
        let cmd = parse_delegation(&args("delegate prov1 0.000001")).unwrap();
        match cmd {
            DelegationCmd::Delegate { amount, .. } => assert_eq!(amount, 1),
            _ => panic!("wrong variant"),
        }
    }
}
