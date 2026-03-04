/// CLI scaffold for the Prova node.
///
/// Subcommands: run, status, account, tx
/// Uses a simple hand-rolled parser (no external deps).

use std::fmt;

// ── Subcommand definitions ──────────────────────────────────────────

#[derive(Debug, Clone, PartialEq)]
pub enum Command {
    Run(RunOpts),
    Status(StatusOpts),
    Account(AccountCmd),
    Tx(TxCmd),
    Help,
    Version,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RunOpts {
    pub rpc_addr: String,
    pub data_dir: String,
    pub chain_id: u64,
    pub validator: bool,
}

impl Default for RunOpts {
    fn default() -> Self {
        Self {
            rpc_addr: "127.0.0.1:9944".into(),
            data_dir: "./data".into(),
            chain_id: 1,
            validator: false,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StatusOpts {
    pub rpc_url: String,
    pub json: bool,
}

impl Default for StatusOpts {
    fn default() -> Self {
        Self {
            rpc_url: "http://127.0.0.1:9944".into(),
            json: false,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum AccountCmd {
    Create,
    Balance { address: String },
    List,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TxCmd {
    Send {
        to: String,
        amount: u64,
        nonce: Option<u64>,
    },
    Status {
        hash: String,
    },
    List {
        address: String,
        limit: usize,
    },
}

// ── Errors ──────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq)]
pub enum ParseError {
    NoCommand,
    UnknownCommand(String),
    UnknownSubcommand(String, String),
    MissingArg(String),
    InvalidValue(String, String),
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ParseError::NoCommand => write!(f, "no command provided (try --help)"),
            ParseError::UnknownCommand(c) => write!(f, "unknown command: '{c}'"),
            ParseError::UnknownSubcommand(cmd, sub) => {
                write!(f, "unknown {cmd} subcommand: '{sub}'")
            }
            ParseError::MissingArg(a) => write!(f, "missing required argument: {a}"),
            ParseError::InvalidValue(k, v) => write!(f, "invalid value for {k}: '{v}'"),
        }
    }
}

// ── Parser ──────────────────────────────────────────────────────────

pub fn parse_args(args: &[String]) -> Result<Command, ParseError> {
    // Skip binary name if present (convention: caller passes &args[1..])
    if args.is_empty() {
        return Err(ParseError::NoCommand);
    }

    match args[0].as_str() {
        "run" => parse_run(&args[1..]),
        "status" => parse_status(&args[1..]),
        "account" => parse_account(&args[1..]),
        "tx" => parse_tx(&args[1..]),
        "--help" | "-h" | "help" => Ok(Command::Help),
        "--version" | "-V" | "version" => Ok(Command::Version),
        other => Err(ParseError::UnknownCommand(other.into())),
    }
}

fn parse_run(args: &[String]) -> Result<Command, ParseError> {
    let mut opts = RunOpts::default();
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--rpc-addr" => {
                i += 1;
                opts.rpc_addr = take_value(args, i, "--rpc-addr")?;
            }
            "--data-dir" => {
                i += 1;
                opts.data_dir = take_value(args, i, "--data-dir")?;
            }
            "--chain-id" => {
                i += 1;
                let v = take_value(args, i, "--chain-id")?;
                opts.chain_id = v
                    .parse()
                    .map_err(|_| ParseError::InvalidValue("--chain-id".into(), v))?;
            }
            "--validator" => opts.validator = true,
            _ => {}
        }
        i += 1;
    }
    Ok(Command::Run(opts))
}

fn parse_status(args: &[String]) -> Result<Command, ParseError> {
    let mut opts = StatusOpts::default();
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--rpc-url" => {
                i += 1;
                opts.rpc_url = take_value(args, i, "--rpc-url")?;
            }
            "--json" => opts.json = true,
            _ => {}
        }
        i += 1;
    }
    Ok(Command::Status(opts))
}

fn parse_account(args: &[String]) -> Result<Command, ParseError> {
    if args.is_empty() {
        return Err(ParseError::NoCommand);
    }
    match args[0].as_str() {
        "create" => Ok(Command::Account(AccountCmd::Create)),
        "balance" => {
            let addr = args
                .get(1)
                .ok_or_else(|| ParseError::MissingArg("address".into()))?;
            Ok(Command::Account(AccountCmd::Balance {
                address: addr.clone(),
            }))
        }
        "list" => Ok(Command::Account(AccountCmd::List)),
        other => Err(ParseError::UnknownSubcommand("account".into(), other.into())),
    }
}

fn parse_tx(args: &[String]) -> Result<Command, ParseError> {
    if args.is_empty() {
        return Err(ParseError::NoCommand);
    }
    match args[0].as_str() {
        "send" => {
            let mut to = None;
            let mut amount = None;
            let mut nonce = None;
            let mut i = 1;
            while i < args.len() {
                match args[i].as_str() {
                    "--to" => {
                        i += 1;
                        to = Some(take_value(args, i, "--to")?);
                    }
                    "--amount" => {
                        i += 1;
                        let v = take_value(args, i, "--amount")?;
                        amount = Some(
                            v.parse()
                                .map_err(|_| ParseError::InvalidValue("--amount".into(), v))?,
                        );
                    }
                    "--nonce" => {
                        i += 1;
                        let v = take_value(args, i, "--nonce")?;
                        nonce = Some(
                            v.parse()
                                .map_err(|_| ParseError::InvalidValue("--nonce".into(), v))?,
                        );
                    }
                    _ => {}
                }
                i += 1;
            }
            Ok(Command::Tx(TxCmd::Send {
                to: to.ok_or_else(|| ParseError::MissingArg("--to".into()))?,
                amount: amount.ok_or_else(|| ParseError::MissingArg("--amount".into()))?,
                nonce,
            }))
        }
        "status" => {
            let hash = args
                .get(1)
                .ok_or_else(|| ParseError::MissingArg("tx-hash".into()))?;
            Ok(Command::Tx(TxCmd::Status { hash: hash.clone() }))
        }
        "list" => {
            let addr = args
                .get(1)
                .ok_or_else(|| ParseError::MissingArg("address".into()))?;
            let limit = args
                .get(2)
                .and_then(|v| v.parse().ok())
                .unwrap_or(20);
            Ok(Command::Tx(TxCmd::List {
                address: addr.clone(),
                limit,
            }))
        }
        other => Err(ParseError::UnknownSubcommand("tx".into(), other.into())),
    }
}

fn take_value(args: &[String], i: usize, flag: &str) -> Result<String, ParseError> {
    args.get(i)
        .cloned()
        .ok_or_else(|| ParseError::MissingArg(flag.into()))
}

// ── Help text ───────────────────────────────────────────────────────

pub fn help_text() -> &'static str {
    "prova-node — Prova network node

USAGE:
    prova-node <COMMAND> [OPTIONS]

COMMANDS:
    run        Start the node
    status     Query node status
    account    Manage accounts (create, balance, list)
    tx         Transactions (send, status, list)
    help       Show this help
    version    Show version

RUN OPTIONS:
    --rpc-addr <ADDR>    Listen address (default: 127.0.0.1:9944)
    --data-dir <PATH>    Data directory (default: ./data)
    --chain-id <ID>      Chain ID (default: 1)
    --validator           Enable validator mode

STATUS OPTIONS:
    --rpc-url <URL>      RPC endpoint (default: http://127.0.0.1:9944)
    --json               Output as JSON

TX SEND OPTIONS:
    --to <ADDR>          Recipient address (required)
    --amount <AMT>       Amount in smallest unit (required)
    --nonce <N>          Override nonce (optional)"
}

pub const VERSION: &str = "prova-node 0.1.0";

// ── Tests ───────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn args(s: &str) -> Vec<String> {
        s.split_whitespace().map(String::from).collect()
    }

    #[test]
    fn test_help() {
        assert_eq!(parse_args(&args("help")).unwrap(), Command::Help);
        assert_eq!(parse_args(&args("--help")).unwrap(), Command::Help);
        assert_eq!(parse_args(&args("-h")).unwrap(), Command::Help);
    }

    #[test]
    fn test_version() {
        assert_eq!(parse_args(&args("version")).unwrap(), Command::Version);
        assert_eq!(parse_args(&args("--version")).unwrap(), Command::Version);
    }

    #[test]
    fn test_run_defaults() {
        let cmd = parse_args(&args("run")).unwrap();
        assert_eq!(cmd, Command::Run(RunOpts::default()));
    }

    #[test]
    fn test_run_all_flags() {
        let cmd = parse_args(&args(
            "run --rpc-addr 0.0.0.0:8080 --data-dir /tmp/prova --chain-id 42 --validator",
        ))
        .unwrap();
        assert_eq!(
            cmd,
            Command::Run(RunOpts {
                rpc_addr: "0.0.0.0:8080".into(),
                data_dir: "/tmp/prova".into(),
                chain_id: 42,
                validator: true,
            })
        );
    }

    #[test]
    fn test_run_invalid_chain_id() {
        let r = parse_args(&args("run --chain-id abc"));
        assert!(matches!(r, Err(ParseError::InvalidValue(_, _))));
    }

    #[test]
    fn test_status_defaults() {
        let cmd = parse_args(&args("status")).unwrap();
        assert_eq!(cmd, Command::Status(StatusOpts::default()));
    }

    #[test]
    fn test_status_json() {
        let cmd = parse_args(&args("status --json")).unwrap();
        assert_eq!(
            cmd,
            Command::Status(StatusOpts {
                json: true,
                ..Default::default()
            })
        );
    }

    #[test]
    fn test_account_create() {
        let cmd = parse_args(&args("account create")).unwrap();
        assert_eq!(cmd, Command::Account(AccountCmd::Create));
    }

    #[test]
    fn test_account_balance() {
        let cmd = parse_args(&args("account balance 0xdead")).unwrap();
        assert_eq!(
            cmd,
            Command::Account(AccountCmd::Balance {
                address: "0xdead".into()
            })
        );
    }

    #[test]
    fn test_account_balance_missing_addr() {
        let r = parse_args(&args("account balance"));
        assert!(matches!(r, Err(ParseError::MissingArg(_))));
    }

    #[test]
    fn test_account_list() {
        let cmd = parse_args(&args("account list")).unwrap();
        assert_eq!(cmd, Command::Account(AccountCmd::List));
    }

    #[test]
    fn test_account_unknown_sub() {
        let r = parse_args(&args("account nope"));
        assert!(matches!(r, Err(ParseError::UnknownSubcommand(_, _))));
    }

    #[test]
    fn test_tx_send() {
        let cmd = parse_args(&args("tx send --to 0xbeef --amount 1000")).unwrap();
        assert_eq!(
            cmd,
            Command::Tx(TxCmd::Send {
                to: "0xbeef".into(),
                amount: 1000,
                nonce: None,
            })
        );
    }

    #[test]
    fn test_tx_send_with_nonce() {
        let cmd = parse_args(&args("tx send --to 0xbeef --amount 500 --nonce 7")).unwrap();
        assert_eq!(
            cmd,
            Command::Tx(TxCmd::Send {
                to: "0xbeef".into(),
                amount: 500,
                nonce: Some(7),
            })
        );
    }

    #[test]
    fn test_tx_send_missing_to() {
        let r = parse_args(&args("tx send --amount 100"));
        assert!(matches!(r, Err(ParseError::MissingArg(_))));
    }

    #[test]
    fn test_tx_send_missing_amount() {
        let r = parse_args(&args("tx send --to 0xbeef"));
        assert!(matches!(r, Err(ParseError::MissingArg(_))));
    }

    #[test]
    fn test_tx_status() {
        let cmd = parse_args(&args("tx status 0xabc123")).unwrap();
        assert_eq!(
            cmd,
            Command::Tx(TxCmd::Status {
                hash: "0xabc123".into()
            })
        );
    }

    #[test]
    fn test_tx_list() {
        let cmd = parse_args(&args("tx list 0xdead")).unwrap();
        assert_eq!(
            cmd,
            Command::Tx(TxCmd::List {
                address: "0xdead".into(),
                limit: 20,
            })
        );
    }

    #[test]
    fn test_tx_list_with_limit() {
        let cmd = parse_args(&args("tx list 0xdead 5")).unwrap();
        assert_eq!(
            cmd,
            Command::Tx(TxCmd::List {
                address: "0xdead".into(),
                limit: 5,
            })
        );
    }

    #[test]
    fn test_empty_args() {
        let r = parse_args(&[]);
        assert!(matches!(r, Err(ParseError::NoCommand)));
    }

    #[test]
    fn test_unknown_command() {
        let r = parse_args(&args("frobnicate"));
        assert!(matches!(r, Err(ParseError::UnknownCommand(_))));
    }

    #[test]
    fn test_error_display() {
        assert_eq!(
            ParseError::NoCommand.to_string(),
            "no command provided (try --help)"
        );
        assert_eq!(
            ParseError::UnknownCommand("foo".into()).to_string(),
            "unknown command: 'foo'"
        );
    }
}
