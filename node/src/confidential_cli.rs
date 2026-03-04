//! Confidential Commit CLI (NODE-031)
//!
//! CLI subcommands for submitting encrypted inference commitments and
//! revealing plaintext on dispute. Integrates with the confidential
//! inference system (CHAIN-035).
//!
//! Subcommands:
//!   commit <model-id> <encrypted-root> --blinding-factor <hex>
//!   reveal <commit-id> <plaintext-root> --blinding-factor <hex>
//!   dispute <commit-id>
//!   status <commit-id>
//!   list [--provider <addr>] [--status <filter>]
//!   finalize [--epoch <n>]

use std::fmt;

pub type Hash = [u8; 32];
pub type Address = [u8; 32];

// ── Subcommand Definitions ──────────────────────────────────────────

#[derive(Debug, Clone, PartialEq)]
pub enum ConfidentialCmd {
    /// Submit an encrypted inference commitment.
    Commit {
        model_id: String,
        encrypted_root: String,
        blinding_factor: String,
        provider: Option<String>,
    },
    /// Reveal plaintext root after a dispute.
    Reveal {
        commit_id: u64,
        plaintext_root: String,
        blinding_factor: String,
    },
    /// Open a dispute on a committed inference.
    Dispute {
        commit_id: u64,
        challenger: Option<String>,
    },
    /// Query status of a confidential commit.
    Status {
        commit_id: u64,
        json: bool,
    },
    /// List confidential commits with optional filters.
    List {
        provider: Option<String>,
        status_filter: Option<StatusFilter>,
        json: bool,
        limit: usize,
    },
    /// Trigger finalization for commits past the challenge window.
    Finalize {
        epoch: Option<u64>,
    },
    /// Enforce defaults (slash providers who missed reveal window).
    EnforceDefaults {
        epoch: Option<u64>,
    },
    /// Show help for confidential subcommands.
    Help,
}

#[derive(Debug, Clone, PartialEq)]
pub enum StatusFilter {
    Committed,
    Disputed,
    Revealed,
    Finalized,
    Defaulted,
    All,
}

impl fmt::Display for StatusFilter {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Committed => write!(f, "committed"),
            Self::Disputed => write!(f, "disputed"),
            Self::Revealed => write!(f, "revealed"),
            Self::Finalized => write!(f, "finalized"),
            Self::Defaulted => write!(f, "defaulted"),
            Self::All => write!(f, "all"),
        }
    }
}

// ── Parsed Commit Info (for display) ────────────────────────────────

#[derive(Debug, Clone)]
pub struct CommitInfo {
    pub id: u64,
    pub provider: String,
    pub model_id: String,
    pub encrypted_root: String,
    pub epoch: u64,
    pub status: String,
    pub dispute_epoch: Option<u64>,
    pub challenger: Option<String>,
    pub plaintext_root: Option<String>,
}

impl fmt::Display for CommitInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Commit #{}", self.id)?;
        writeln!(f, "  Provider:       {}", self.provider)?;
        writeln!(f, "  Model:          {}", self.model_id)?;
        writeln!(f, "  Encrypted Root: {}", self.encrypted_root)?;
        writeln!(f, "  Epoch:          {}", self.epoch)?;
        writeln!(f, "  Status:         {}", self.status)?;
        if let Some(de) = self.dispute_epoch {
            writeln!(f, "  Dispute Epoch:  {}", de)?;
        }
        if let Some(ref c) = self.challenger {
            writeln!(f, "  Challenger:     {}", c)?;
        }
        if let Some(ref pr) = self.plaintext_root {
            writeln!(f, "  Plaintext Root: {}", pr)?;
        }
        Ok(())
    }
}

// ── Parser ──────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq)]
pub struct ParseError(pub String);

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "parse error: {}", self.0)
    }
}

pub fn parse_confidential_cmd(args: &[&str]) -> Result<ConfidentialCmd, ParseError> {
    if args.is_empty() {
        return Ok(ConfidentialCmd::Help);
    }

    match args[0] {
        "commit" => parse_commit(&args[1..]),
        "reveal" => parse_reveal(&args[1..]),
        "dispute" => parse_dispute(&args[1..]),
        "status" => parse_status(&args[1..]),
        "list" => parse_list(&args[1..]),
        "finalize" => parse_finalize(&args[1..]),
        "enforce-defaults" => parse_enforce_defaults(&args[1..]),
        "help" | "--help" | "-h" => Ok(ConfidentialCmd::Help),
        other => Err(ParseError(format!("unknown subcommand: {}", other))),
    }
}

fn parse_commit(args: &[&str]) -> Result<ConfidentialCmd, ParseError> {
    if args.len() < 2 {
        return Err(ParseError("usage: commit <model-id> <encrypted-root> --blinding-factor <hex>".into()));
    }
    let model_id = args[0].to_string();
    let encrypted_root = args[1].to_string();
    let mut blinding_factor = None;
    let mut provider = None;
    let mut i = 2;
    while i < args.len() {
        match args[i] {
            "--blinding-factor" | "-b" => {
                i += 1;
                blinding_factor = Some(args.get(i).ok_or_else(|| ParseError("--blinding-factor requires value".into()))?.to_string());
            }
            "--provider" | "-p" => {
                i += 1;
                provider = Some(args.get(i).ok_or_else(|| ParseError("--provider requires value".into()))?.to_string());
            }
            _ => return Err(ParseError(format!("unknown flag: {}", args[i]))),
        }
        i += 1;
    }
    let blinding_factor = blinding_factor.ok_or_else(|| ParseError("--blinding-factor is required".into()))?;
    Ok(ConfidentialCmd::Commit { model_id, encrypted_root, blinding_factor, provider })
}

fn parse_reveal(args: &[&str]) -> Result<ConfidentialCmd, ParseError> {
    if args.is_empty() {
        return Err(ParseError("usage: reveal <commit-id> <plaintext-root> --blinding-factor <hex>".into()));
    }
    let commit_id = args[0].parse::<u64>().map_err(|_| ParseError("invalid commit-id".into()))?;
    let plaintext_root = args.get(1).ok_or_else(|| ParseError("plaintext-root required".into()))?.to_string();
    let mut blinding_factor = None;
    let mut i = 2;
    while i < args.len() {
        match args[i] {
            "--blinding-factor" | "-b" => {
                i += 1;
                blinding_factor = Some(args.get(i).ok_or_else(|| ParseError("--blinding-factor requires value".into()))?.to_string());
            }
            _ => return Err(ParseError(format!("unknown flag: {}", args[i]))),
        }
        i += 1;
    }
    let blinding_factor = blinding_factor.ok_or_else(|| ParseError("--blinding-factor is required".into()))?;
    Ok(ConfidentialCmd::Reveal { commit_id, plaintext_root, blinding_factor })
}

fn parse_dispute(args: &[&str]) -> Result<ConfidentialCmd, ParseError> {
    if args.is_empty() {
        return Err(ParseError("usage: dispute <commit-id> [--challenger <addr>]".into()));
    }
    let commit_id = args[0].parse::<u64>().map_err(|_| ParseError("invalid commit-id".into()))?;
    let mut challenger = None;
    let mut i = 1;
    while i < args.len() {
        match args[i] {
            "--challenger" | "-c" => {
                i += 1;
                challenger = Some(args.get(i).ok_or_else(|| ParseError("--challenger requires value".into()))?.to_string());
            }
            _ => return Err(ParseError(format!("unknown flag: {}", args[i]))),
        }
        i += 1;
    }
    Ok(ConfidentialCmd::Dispute { commit_id, challenger })
}

fn parse_status(args: &[&str]) -> Result<ConfidentialCmd, ParseError> {
    if args.is_empty() {
        return Err(ParseError("usage: status <commit-id> [--json]".into()));
    }
    let commit_id = args[0].parse::<u64>().map_err(|_| ParseError("invalid commit-id".into()))?;
    let json = args[1..].contains(&"--json");
    Ok(ConfidentialCmd::Status { commit_id, json })
}

fn parse_list(args: &[&str]) -> Result<ConfidentialCmd, ParseError> {
    let mut provider = None;
    let mut status_filter = None;
    let mut json = false;
    let mut limit = 50;
    let mut i = 0;
    while i < args.len() {
        match args[i] {
            "--provider" | "-p" => {
                i += 1;
                provider = Some(args.get(i).ok_or_else(|| ParseError("--provider requires value".into()))?.to_string());
            }
            "--status" | "-s" => {
                i += 1;
                let s = args.get(i).ok_or_else(|| ParseError("--status requires value".into()))?;
                status_filter = Some(parse_status_filter(s)?);
            }
            "--json" => json = true,
            "--limit" | "-n" => {
                i += 1;
                limit = args.get(i).ok_or_else(|| ParseError("--limit requires value".into()))?
                    .parse().map_err(|_| ParseError("invalid limit".into()))?;
            }
            _ => return Err(ParseError(format!("unknown flag: {}", args[i]))),
        }
        i += 1;
    }
    Ok(ConfidentialCmd::List { provider, status_filter, json, limit })
}

fn parse_finalize(args: &[&str]) -> Result<ConfidentialCmd, ParseError> {
    let mut epoch = None;
    let mut i = 0;
    while i < args.len() {
        match args[i] {
            "--epoch" | "-e" => {
                i += 1;
                epoch = Some(args.get(i).ok_or_else(|| ParseError("--epoch requires value".into()))?
                    .parse().map_err(|_| ParseError("invalid epoch".into()))?);
            }
            _ => return Err(ParseError(format!("unknown flag: {}", args[i]))),
        }
        i += 1;
    }
    Ok(ConfidentialCmd::Finalize { epoch })
}

fn parse_enforce_defaults(args: &[&str]) -> Result<ConfidentialCmd, ParseError> {
    let mut epoch = None;
    let mut i = 0;
    while i < args.len() {
        match args[i] {
            "--epoch" | "-e" => {
                i += 1;
                epoch = Some(args.get(i).ok_or_else(|| ParseError("--epoch requires value".into()))?
                    .parse().map_err(|_| ParseError("invalid epoch".into()))?);
            }
            _ => return Err(ParseError(format!("unknown flag: {}", args[i]))),
        }
        i += 1;
    }
    Ok(ConfidentialCmd::EnforceDefaults { epoch })
}

fn parse_status_filter(s: &str) -> Result<StatusFilter, ParseError> {
    match s.to_lowercase().as_str() {
        "committed" => Ok(StatusFilter::Committed),
        "disputed" => Ok(StatusFilter::Disputed),
        "revealed" => Ok(StatusFilter::Revealed),
        "finalized" => Ok(StatusFilter::Finalized),
        "defaulted" => Ok(StatusFilter::Defaulted),
        "all" => Ok(StatusFilter::All),
        _ => Err(ParseError(format!("unknown status filter: {}", s))),
    }
}

// ── Hex helpers ─────────────────────────────────────────────────────

pub fn hex_encode(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{:02x}", b)).collect()
}

pub fn hex_decode(s: &str) -> Result<Vec<u8>, ParseError> {
    let s = s.strip_prefix("0x").unwrap_or(s);
    if s.len() % 2 != 0 {
        return Err(ParseError("hex string must have even length".into()));
    }
    (0..s.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&s[i..i + 2], 16).map_err(|_| ParseError("invalid hex".into())))
        .collect()
}

pub fn hex_to_hash(s: &str) -> Result<Hash, ParseError> {
    let bytes = hex_decode(s)?;
    if bytes.len() != 32 {
        return Err(ParseError(format!("expected 32 bytes, got {}", bytes.len())));
    }
    let mut h = [0u8; 32];
    h.copy_from_slice(&bytes);
    Ok(h)
}

// ── Execution Engine ────────────────────────────────────────────────

/// Simulated chain state for CLI operations.
pub struct ConfidentialCli {
    commits: Vec<CommitInfo>,
    next_id: u64,
    current_epoch: u64,
}

/// Result of a CLI command execution.
#[derive(Debug, Clone)]
pub struct CmdResult {
    pub success: bool,
    pub message: String,
    pub commit_info: Option<CommitInfo>,
    pub commit_list: Vec<CommitInfo>,
}

impl fmt::Display for CmdResult {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message)?;
        if let Some(ref info) = self.commit_info {
            write!(f, "\n{}", info)?;
        }
        for info in &self.commit_list {
            write!(f, "\n{}", info)?;
        }
        Ok(())
    }
}

impl ConfidentialCli {
    pub fn new(current_epoch: u64) -> Self {
        Self {
            commits: Vec::new(),
            next_id: 1,
            current_epoch,
        }
    }

    pub fn set_epoch(&mut self, epoch: u64) {
        self.current_epoch = epoch;
    }

    pub fn epoch(&self) -> u64 {
        self.current_epoch
    }

    /// Execute a parsed confidential CLI command.
    pub fn execute(&mut self, cmd: ConfidentialCmd) -> CmdResult {
        match cmd {
            ConfidentialCmd::Commit { model_id, encrypted_root, blinding_factor, provider } => {
                self.exec_commit(model_id, encrypted_root, blinding_factor, provider)
            }
            ConfidentialCmd::Reveal { commit_id, plaintext_root, blinding_factor } => {
                self.exec_reveal(commit_id, plaintext_root, blinding_factor)
            }
            ConfidentialCmd::Dispute { commit_id, challenger } => {
                self.exec_dispute(commit_id, challenger)
            }
            ConfidentialCmd::Status { commit_id, json: _ } => {
                self.exec_status(commit_id)
            }
            ConfidentialCmd::List { provider, status_filter, json: _, limit } => {
                self.exec_list(provider, status_filter, limit)
            }
            ConfidentialCmd::Finalize { epoch } => {
                self.exec_finalize(epoch)
            }
            ConfidentialCmd::EnforceDefaults { epoch } => {
                self.exec_enforce_defaults(epoch)
            }
            ConfidentialCmd::Help => {
                CmdResult {
                    success: true,
                    message: HELP_TEXT.to_string(),
                    commit_info: None,
                    commit_list: Vec::new(),
                }
            }
        }
    }

    fn exec_commit(&mut self, model_id: String, encrypted_root: String, blinding_factor: String, provider: Option<String>) -> CmdResult {
        // Validate hex inputs
        if hex_decode(&encrypted_root).is_err() {
            return CmdResult { success: false, message: "invalid hex for encrypted-root".into(), commit_info: None, commit_list: Vec::new() };
        }
        if hex_decode(&blinding_factor).is_err() {
            return CmdResult { success: false, message: "invalid hex for blinding-factor".into(), commit_info: None, commit_list: Vec::new() };
        }

        let id = self.next_id;
        self.next_id += 1;
        let provider_str = provider.unwrap_or_else(|| "local-wallet".into());

        let info = CommitInfo {
            id,
            provider: provider_str,
            model_id,
            encrypted_root,
            epoch: self.current_epoch,
            status: "committed".into(),
            dispute_epoch: None,
            challenger: None,
            plaintext_root: None,
        };
        self.commits.push(info.clone());

        CmdResult {
            success: true,
            message: format!("✓ Confidential commit #{} submitted at epoch {}", id, self.current_epoch),
            commit_info: Some(info),
            commit_list: Vec::new(),
        }
    }

    fn exec_reveal(&mut self, commit_id: u64, plaintext_root: String, blinding_factor: String) -> CmdResult {
        if hex_decode(&plaintext_root).is_err() {
            return CmdResult { success: false, message: "invalid hex for plaintext-root".into(), commit_info: None, commit_list: Vec::new() };
        }
        if hex_decode(&blinding_factor).is_err() {
            return CmdResult { success: false, message: "invalid hex for blinding-factor".into(), commit_info: None, commit_list: Vec::new() };
        }

        let commit = self.commits.iter_mut().find(|c| c.id == commit_id);
        match commit {
            None => CmdResult { success: false, message: format!("commit #{} not found", commit_id), commit_info: None, commit_list: Vec::new() },
            Some(c) => {
                if c.status != "disputed" {
                    return CmdResult { success: false, message: format!("commit #{} is not disputed (status: {})", commit_id, c.status), commit_info: None, commit_list: Vec::new() };
                }
                // Check reveal window (5 epochs from dispute)
                if let Some(de) = c.dispute_epoch {
                    if self.current_epoch > de + 5 {
                        return CmdResult { success: false, message: format!("reveal window expired for commit #{}", commit_id), commit_info: None, commit_list: Vec::new() };
                    }
                }
                c.status = "revealed".into();
                c.plaintext_root = Some(plaintext_root);
                CmdResult {
                    success: true,
                    message: format!("✓ Commit #{} revealed at epoch {}", commit_id, self.current_epoch),
                    commit_info: Some(c.clone()),
                    commit_list: Vec::new(),
                }
            }
        }
    }

    fn exec_dispute(&mut self, commit_id: u64, challenger: Option<String>) -> CmdResult {
        let commit = self.commits.iter_mut().find(|c| c.id == commit_id);
        match commit {
            None => CmdResult { success: false, message: format!("commit #{} not found", commit_id), commit_info: None, commit_list: Vec::new() },
            Some(c) => {
                if c.status != "committed" {
                    return CmdResult { success: false, message: format!("commit #{} cannot be disputed (status: {})", commit_id, c.status), commit_info: None, commit_list: Vec::new() };
                }
                // Check challenge window (10 epochs)
                if self.current_epoch > c.epoch + 10 {
                    return CmdResult { success: false, message: format!("challenge window expired for commit #{}", commit_id), commit_info: None, commit_list: Vec::new() };
                }
                let challenger_str = challenger.unwrap_or_else(|| "local-challenger".into());
                c.status = "disputed".into();
                c.dispute_epoch = Some(self.current_epoch);
                c.challenger = Some(challenger_str);
                CmdResult {
                    success: true,
                    message: format!("✓ Dispute opened on commit #{} at epoch {}", commit_id, self.current_epoch),
                    commit_info: Some(c.clone()),
                    commit_list: Vec::new(),
                }
            }
        }
    }

    fn exec_status(&self, commit_id: u64) -> CmdResult {
        match self.commits.iter().find(|c| c.id == commit_id) {
            None => CmdResult { success: false, message: format!("commit #{} not found", commit_id), commit_info: None, commit_list: Vec::new() },
            Some(c) => CmdResult { success: true, message: format!("Commit #{} status: {}", commit_id, c.status), commit_info: Some(c.clone()), commit_list: Vec::new() },
        }
    }

    fn exec_list(&self, provider: Option<String>, status_filter: Option<StatusFilter>, limit: usize) -> CmdResult {
        let mut results: Vec<&CommitInfo> = self.commits.iter().collect();

        if let Some(ref p) = provider {
            results.retain(|c| c.provider == *p);
        }
        if let Some(ref sf) = status_filter {
            if *sf != StatusFilter::All {
                let s = sf.to_string();
                results.retain(|c| c.status == s);
            }
        }

        let total = results.len();
        results.truncate(limit);
        let list: Vec<CommitInfo> = results.into_iter().cloned().collect();

        CmdResult {
            success: true,
            message: format!("Found {} commits (showing {})", total, list.len()),
            commit_info: None,
            commit_list: list,
        }
    }

    fn exec_finalize(&mut self, epoch: Option<u64>) -> CmdResult {
        let epoch = epoch.unwrap_or(self.current_epoch);
        let mut finalized = 0;
        for c in &mut self.commits {
            if c.status == "committed" && epoch > c.epoch + 10 {
                c.status = "finalized".into();
                finalized += 1;
            }
        }
        CmdResult {
            success: true,
            message: format!("✓ Finalized {} commits at epoch {}", finalized, epoch),
            commit_info: None,
            commit_list: Vec::new(),
        }
    }

    fn exec_enforce_defaults(&mut self, epoch: Option<u64>) -> CmdResult {
        let epoch = epoch.unwrap_or(self.current_epoch);
        let mut defaulted = 0;
        for c in &mut self.commits {
            if c.status == "disputed" {
                if let Some(de) = c.dispute_epoch {
                    if epoch > de + 5 {
                        c.status = "defaulted".into();
                        defaulted += 1;
                    }
                }
            }
        }
        CmdResult {
            success: true,
            message: format!("✓ Defaulted {} commits at epoch {}", defaulted, epoch),
            commit_info: None,
            commit_list: Vec::new(),
        }
    }

    pub fn commit_count(&self) -> usize {
        self.commits.len()
    }
}

const HELP_TEXT: &str = "\
prova confidential — Manage confidential inference commitments

SUBCOMMANDS:
  commit <model-id> <encrypted-root> --blinding-factor <hex>
      Submit an encrypted inference commitment

  reveal <commit-id> <plaintext-root> --blinding-factor <hex>
      Reveal plaintext root after a dispute

  dispute <commit-id> [--challenger <addr>]
      Open a dispute on a committed inference

  status <commit-id> [--json]
      Query status of a confidential commit

  list [--provider <addr>] [--status <filter>] [--limit <n>] [--json]
      List confidential commits

  finalize [--epoch <n>]
      Finalize commits past the challenge window

  enforce-defaults [--epoch <n>]
      Slash providers who missed the reveal window

  help
      Show this help message
";

// ── Tests ───────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    // ── Parser Tests ────────────────────────────────────────────────

    #[test]
    fn test_parse_commit() {
        let args = vec!["commit", "llama-7b", "0xaa", "--blinding-factor", "0xbb"];
        let cmd = parse_confidential_cmd(&args).unwrap();
        assert_eq!(cmd, ConfidentialCmd::Commit {
            model_id: "llama-7b".into(),
            encrypted_root: "0xaa".into(),
            blinding_factor: "0xbb".into(),
            provider: None,
        });
    }

    #[test]
    fn test_parse_commit_with_provider() {
        let args = vec!["commit", "llama-7b", "0xaa", "--blinding-factor", "0xbb", "--provider", "0xcc"];
        let cmd = parse_confidential_cmd(&args).unwrap();
        assert_eq!(cmd, ConfidentialCmd::Commit {
            model_id: "llama-7b".into(),
            encrypted_root: "0xaa".into(),
            blinding_factor: "0xbb".into(),
            provider: Some("0xcc".into()),
        });
    }

    #[test]
    fn test_parse_commit_missing_blinding() {
        let args = vec!["commit", "llama-7b", "0xaa"];
        assert!(parse_confidential_cmd(&args).is_err());
    }

    #[test]
    fn test_parse_reveal() {
        let args = vec!["reveal", "42", "0xdd", "--blinding-factor", "0xee"];
        let cmd = parse_confidential_cmd(&args).unwrap();
        assert_eq!(cmd, ConfidentialCmd::Reveal {
            commit_id: 42,
            plaintext_root: "0xdd".into(),
            blinding_factor: "0xee".into(),
        });
    }

    #[test]
    fn test_parse_dispute() {
        let args = vec!["dispute", "7"];
        let cmd = parse_confidential_cmd(&args).unwrap();
        assert_eq!(cmd, ConfidentialCmd::Dispute { commit_id: 7, challenger: None });
    }

    #[test]
    fn test_parse_dispute_with_challenger() {
        let args = vec!["dispute", "7", "--challenger", "0xff"];
        let cmd = parse_confidential_cmd(&args).unwrap();
        assert_eq!(cmd, ConfidentialCmd::Dispute { commit_id: 7, challenger: Some("0xff".into()) });
    }

    #[test]
    fn test_parse_status() {
        let args = vec!["status", "3"];
        let cmd = parse_confidential_cmd(&args).unwrap();
        assert_eq!(cmd, ConfidentialCmd::Status { commit_id: 3, json: false });
    }

    #[test]
    fn test_parse_status_json() {
        let args = vec!["status", "3", "--json"];
        let cmd = parse_confidential_cmd(&args).unwrap();
        assert_eq!(cmd, ConfidentialCmd::Status { commit_id: 3, json: true });
    }

    #[test]
    fn test_parse_list_defaults() {
        let args = vec!["list"];
        let cmd = parse_confidential_cmd(&args).unwrap();
        assert_eq!(cmd, ConfidentialCmd::List { provider: None, status_filter: None, json: false, limit: 50 });
    }

    #[test]
    fn test_parse_list_filtered() {
        let args = vec!["list", "--provider", "0xaa", "--status", "disputed", "--limit", "10", "--json"];
        let cmd = parse_confidential_cmd(&args).unwrap();
        assert_eq!(cmd, ConfidentialCmd::List {
            provider: Some("0xaa".into()),
            status_filter: Some(StatusFilter::Disputed),
            json: true,
            limit: 10,
        });
    }

    #[test]
    fn test_parse_finalize() {
        let args = vec!["finalize", "--epoch", "200"];
        let cmd = parse_confidential_cmd(&args).unwrap();
        assert_eq!(cmd, ConfidentialCmd::Finalize { epoch: Some(200) });
    }

    #[test]
    fn test_parse_enforce_defaults() {
        let args = vec!["enforce-defaults"];
        let cmd = parse_confidential_cmd(&args).unwrap();
        assert_eq!(cmd, ConfidentialCmd::EnforceDefaults { epoch: None });
    }

    #[test]
    fn test_parse_help() {
        let args = vec!["help"];
        let cmd = parse_confidential_cmd(&args).unwrap();
        assert_eq!(cmd, ConfidentialCmd::Help);
    }

    #[test]
    fn test_parse_empty_is_help() {
        let args: Vec<&str> = vec![];
        let cmd = parse_confidential_cmd(&args).unwrap();
        assert_eq!(cmd, ConfidentialCmd::Help);
    }

    #[test]
    fn test_parse_unknown_subcommand() {
        let args = vec!["frobnicate"];
        assert!(parse_confidential_cmd(&args).is_err());
    }

    // ── Hex Tests ───────────────────────────────────────────────────

    #[test]
    fn test_hex_encode() {
        assert_eq!(hex_encode(&[0xde, 0xad, 0xbe, 0xef]), "deadbeef");
    }

    #[test]
    fn test_hex_decode() {
        assert_eq!(hex_decode("deadbeef").unwrap(), vec![0xde, 0xad, 0xbe, 0xef]);
        assert_eq!(hex_decode("0xdeadbeef").unwrap(), vec![0xde, 0xad, 0xbe, 0xef]);
    }

    #[test]
    fn test_hex_decode_odd_length() {
        assert!(hex_decode("abc").is_err());
    }

    #[test]
    fn test_hex_to_hash() {
        let hex = "aa".repeat(32);
        let h = hex_to_hash(&hex).unwrap();
        assert_eq!(h, [0xaa; 32]);
    }

    #[test]
    fn test_hex_to_hash_wrong_length() {
        assert!(hex_to_hash("aabb").is_err());
    }

    // ── Execution Tests ─────────────────────────────────────────────

    #[test]
    fn test_exec_commit() {
        let mut cli = ConfidentialCli::new(100);
        let cmd = ConfidentialCmd::Commit {
            model_id: "llama-7b".into(),
            encrypted_root: "aabb".into(),
            blinding_factor: "ccdd".into(),
            provider: None,
        };
        let result = cli.execute(cmd);
        assert!(result.success);
        assert!(result.message.contains("#1"));
        assert_eq!(cli.commit_count(), 1);
    }

    #[test]
    fn test_exec_commit_invalid_hex() {
        let mut cli = ConfidentialCli::new(100);
        let cmd = ConfidentialCmd::Commit {
            model_id: "llama-7b".into(),
            encrypted_root: "not-hex!".into(),
            blinding_factor: "ccdd".into(),
            provider: None,
        };
        let result = cli.execute(cmd);
        assert!(!result.success);
    }

    #[test]
    fn test_exec_dispute_and_reveal() {
        let mut cli = ConfidentialCli::new(100);

        // Commit
        let commit_cmd = ConfidentialCmd::Commit {
            model_id: "llama-7b".into(),
            encrypted_root: "aabb".into(),
            blinding_factor: "ccdd".into(),
            provider: Some("provider-1".into()),
        };
        let r = cli.execute(commit_cmd);
        assert!(r.success);

        // Dispute within window
        cli.set_epoch(105);
        let dispute_cmd = ConfidentialCmd::Dispute { commit_id: 1, challenger: Some("challenger-1".into()) };
        let r = cli.execute(dispute_cmd);
        assert!(r.success);
        assert!(r.message.contains("Dispute opened"));

        // Reveal within reveal window
        cli.set_epoch(108);
        let reveal_cmd = ConfidentialCmd::Reveal {
            commit_id: 1,
            plaintext_root: "eeff".into(),
            blinding_factor: "0011".into(),
        };
        let r = cli.execute(reveal_cmd);
        assert!(r.success);
        assert!(r.message.contains("revealed"));
    }

    #[test]
    fn test_exec_dispute_expired() {
        let mut cli = ConfidentialCli::new(100);
        cli.execute(ConfidentialCmd::Commit {
            model_id: "m".into(), encrypted_root: "aa".into(),
            blinding_factor: "bb".into(), provider: None,
        });

        cli.set_epoch(111); // Past challenge window
        let r = cli.execute(ConfidentialCmd::Dispute { commit_id: 1, challenger: None });
        assert!(!r.success);
        assert!(r.message.contains("challenge window expired"));
    }

    #[test]
    fn test_exec_reveal_window_expired() {
        let mut cli = ConfidentialCli::new(100);
        cli.execute(ConfidentialCmd::Commit {
            model_id: "m".into(), encrypted_root: "aa".into(),
            blinding_factor: "bb".into(), provider: None,
        });
        cli.set_epoch(105);
        cli.execute(ConfidentialCmd::Dispute { commit_id: 1, challenger: None });

        cli.set_epoch(111); // Past reveal window (105 + 5)
        let r = cli.execute(ConfidentialCmd::Reveal {
            commit_id: 1, plaintext_root: "cc".into(), blinding_factor: "dd".into(),
        });
        assert!(!r.success);
        assert!(r.message.contains("reveal window expired"));
    }

    #[test]
    fn test_exec_reveal_not_disputed() {
        let mut cli = ConfidentialCli::new(100);
        cli.execute(ConfidentialCmd::Commit {
            model_id: "m".into(), encrypted_root: "aa".into(),
            blinding_factor: "bb".into(), provider: None,
        });
        let r = cli.execute(ConfidentialCmd::Reveal {
            commit_id: 1, plaintext_root: "cc".into(), blinding_factor: "dd".into(),
        });
        assert!(!r.success);
        assert!(r.message.contains("not disputed"));
    }

    #[test]
    fn test_exec_status() {
        let mut cli = ConfidentialCli::new(100);
        cli.execute(ConfidentialCmd::Commit {
            model_id: "m".into(), encrypted_root: "aa".into(),
            blinding_factor: "bb".into(), provider: None,
        });
        let r = cli.execute(ConfidentialCmd::Status { commit_id: 1, json: false });
        assert!(r.success);
        assert!(r.commit_info.is_some());
    }

    #[test]
    fn test_exec_status_not_found() {
        let cli = ConfidentialCli::new(100);
        // Need mut for execute but status doesn't mutate — that's fine, Rust just needs mut ref
        let mut cli = cli;
        let r = cli.execute(ConfidentialCmd::Status { commit_id: 999, json: false });
        assert!(!r.success);
    }

    #[test]
    fn test_exec_list_empty() {
        let mut cli = ConfidentialCli::new(100);
        let r = cli.execute(ConfidentialCmd::List { provider: None, status_filter: None, json: false, limit: 50 });
        assert!(r.success);
        assert_eq!(r.commit_list.len(), 0);
    }

    #[test]
    fn test_exec_list_filtered() {
        let mut cli = ConfidentialCli::new(100);
        cli.execute(ConfidentialCmd::Commit {
            model_id: "m".into(), encrypted_root: "aa".into(),
            blinding_factor: "bb".into(), provider: Some("p1".into()),
        });
        cli.execute(ConfidentialCmd::Commit {
            model_id: "m".into(), encrypted_root: "cc".into(),
            blinding_factor: "dd".into(), provider: Some("p2".into()),
        });
        let r = cli.execute(ConfidentialCmd::List {
            provider: Some("p1".into()), status_filter: None, json: false, limit: 50,
        });
        assert_eq!(r.commit_list.len(), 1);
    }

    #[test]
    fn test_exec_finalize() {
        let mut cli = ConfidentialCli::new(100);
        cli.execute(ConfidentialCmd::Commit {
            model_id: "m".into(), encrypted_root: "aa".into(),
            blinding_factor: "bb".into(), provider: None,
        });
        cli.set_epoch(111);
        let r = cli.execute(ConfidentialCmd::Finalize { epoch: None });
        assert!(r.success);
        assert!(r.message.contains("Finalized 1"));
    }

    #[test]
    fn test_exec_enforce_defaults() {
        let mut cli = ConfidentialCli::new(100);
        cli.execute(ConfidentialCmd::Commit {
            model_id: "m".into(), encrypted_root: "aa".into(),
            blinding_factor: "bb".into(), provider: None,
        });
        cli.set_epoch(105);
        cli.execute(ConfidentialCmd::Dispute { commit_id: 1, challenger: None });
        cli.set_epoch(111);
        let r = cli.execute(ConfidentialCmd::EnforceDefaults { epoch: None });
        assert!(r.success);
        assert!(r.message.contains("Defaulted 1"));
    }

    #[test]
    fn test_exec_help() {
        let mut cli = ConfidentialCli::new(100);
        let r = cli.execute(ConfidentialCmd::Help);
        assert!(r.success);
        assert!(r.message.contains("SUBCOMMANDS"));
    }

    #[test]
    fn test_full_lifecycle_commit_finalize() {
        let mut cli = ConfidentialCli::new(100);
        // Commit
        cli.execute(ConfidentialCmd::Commit {
            model_id: "llama-7b".into(), encrypted_root: "aabb".into(),
            blinding_factor: "ccdd".into(), provider: Some("provider-1".into()),
        });
        // Advance past challenge window and finalize
        cli.set_epoch(111);
        cli.execute(ConfidentialCmd::Finalize { epoch: None });
        // Verify
        let r = cli.execute(ConfidentialCmd::Status { commit_id: 1, json: false });
        assert_eq!(r.commit_info.unwrap().status, "finalized");
    }

    #[test]
    fn test_full_lifecycle_dispute_default() {
        let mut cli = ConfidentialCli::new(100);
        cli.execute(ConfidentialCmd::Commit {
            model_id: "m".into(), encrypted_root: "aa".into(),
            blinding_factor: "bb".into(), provider: Some("lazy-provider".into()),
        });
        cli.set_epoch(105);
        cli.execute(ConfidentialCmd::Dispute { commit_id: 1, challenger: Some("vigilant-challenger".into()) });
        // Provider doesn't reveal → enforce defaults
        cli.set_epoch(111);
        cli.execute(ConfidentialCmd::EnforceDefaults { epoch: None });
        let r = cli.execute(ConfidentialCmd::Status { commit_id: 1, json: false });
        assert_eq!(r.commit_info.unwrap().status, "defaulted");
    }

    #[test]
    fn test_list_by_status_filter() {
        let mut cli = ConfidentialCli::new(100);
        cli.execute(ConfidentialCmd::Commit {
            model_id: "m".into(), encrypted_root: "aa".into(),
            blinding_factor: "bb".into(), provider: None,
        });
        cli.execute(ConfidentialCmd::Commit {
            model_id: "m".into(), encrypted_root: "cc".into(),
            blinding_factor: "dd".into(), provider: None,
        });
        // Finalize first
        cli.set_epoch(111);
        cli.execute(ConfidentialCmd::Finalize { epoch: Some(111) });

        // Add a new committed one
        cli.execute(ConfidentialCmd::Commit {
            model_id: "m".into(), encrypted_root: "ee".into(),
            blinding_factor: "ff".into(), provider: None,
        });

        let r = cli.execute(ConfidentialCmd::List {
            provider: None, status_filter: Some(StatusFilter::Finalized), json: false, limit: 50,
        });
        assert_eq!(r.commit_list.len(), 2);

        let r = cli.execute(ConfidentialCmd::List {
            provider: None, status_filter: Some(StatusFilter::Committed), json: false, limit: 50,
        });
        assert_eq!(r.commit_list.len(), 1);
    }

    #[test]
    fn test_list_limit() {
        let mut cli = ConfidentialCli::new(100);
        for i in 0..10 {
            cli.execute(ConfidentialCmd::Commit {
                model_id: format!("m{}", i), encrypted_root: "aa".into(),
                blinding_factor: "bb".into(), provider: None,
            });
        }
        let r = cli.execute(ConfidentialCmd::List {
            provider: None, status_filter: None, json: false, limit: 3,
        });
        assert_eq!(r.commit_list.len(), 3);
        assert!(r.message.contains("Found 10"));
    }

    #[test]
    fn test_dispute_not_found() {
        let mut cli = ConfidentialCli::new(100);
        let r = cli.execute(ConfidentialCmd::Dispute { commit_id: 999, challenger: None });
        assert!(!r.success);
        assert!(r.message.contains("not found"));
    }

    #[test]
    fn test_reveal_not_found() {
        let mut cli = ConfidentialCli::new(100);
        let r = cli.execute(ConfidentialCmd::Reveal {
            commit_id: 999, plaintext_root: "aa".into(), blinding_factor: "bb".into(),
        });
        assert!(!r.success);
    }

    #[test]
    fn test_dispute_already_disputed() {
        let mut cli = ConfidentialCli::new(100);
        cli.execute(ConfidentialCmd::Commit {
            model_id: "m".into(), encrypted_root: "aa".into(),
            blinding_factor: "bb".into(), provider: None,
        });
        cli.set_epoch(105);
        cli.execute(ConfidentialCmd::Dispute { commit_id: 1, challenger: None });
        let r = cli.execute(ConfidentialCmd::Dispute { commit_id: 1, challenger: None });
        assert!(!r.success);
        assert!(r.message.contains("cannot be disputed"));
    }

    #[test]
    fn test_commit_info_display() {
        let info = CommitInfo {
            id: 1,
            provider: "test-provider".into(),
            model_id: "llama-7b".into(),
            encrypted_root: "aabbccdd".into(),
            epoch: 100,
            status: "committed".into(),
            dispute_epoch: None,
            challenger: None,
            plaintext_root: None,
        };
        let s = format!("{}", info);
        assert!(s.contains("Commit #1"));
        assert!(s.contains("test-provider"));
        assert!(s.contains("llama-7b"));
    }

    #[test]
    fn test_cmd_result_display() {
        let r = CmdResult {
            success: true,
            message: "test message".into(),
            commit_info: None,
            commit_list: Vec::new(),
        };
        assert_eq!(format!("{}", r), "test message");
    }

    #[test]
    fn test_parse_invalid_commit_id() {
        let args = vec!["reveal", "not-a-number", "0xaa", "--blinding-factor", "0xbb"];
        assert!(parse_confidential_cmd(&args).is_err());
    }

    #[test]
    fn test_status_filter_display() {
        assert_eq!(format!("{}", StatusFilter::Committed), "committed");
        assert_eq!(format!("{}", StatusFilter::Disputed), "disputed");
        assert_eq!(format!("{}", StatusFilter::Revealed), "revealed");
        assert_eq!(format!("{}", StatusFilter::Finalized), "finalized");
        assert_eq!(format!("{}", StatusFilter::Defaulted), "defaulted");
        assert_eq!(format!("{}", StatusFilter::All), "all");
    }

    #[test]
    fn test_parse_status_filter_all_variants() {
        assert_eq!(parse_status_filter("committed").unwrap(), StatusFilter::Committed);
        assert_eq!(parse_status_filter("DISPUTED").unwrap(), StatusFilter::Disputed);
        assert_eq!(parse_status_filter("Revealed").unwrap(), StatusFilter::Revealed);
        assert!(parse_status_filter("unknown").is_err());
    }
}
