//! NODE-026: Marketplace CLI commands — list, bid, discover
//!
//! Provides a CLI interface over the chain marketplace module.
//! Subcommands: list, bid, discover, show, cancel, my-listings

use std::fmt;

// ── Command definitions ──────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq)]
pub enum MarketCmd {
    /// List models on the marketplace (optionally filter by model).
    List(ListOpts),
    /// Place a bid for inference.
    Bid(BidOpts),
    /// Discover providers for a model.
    Discover(DiscoverOpts),
    /// Show details of a specific listing.
    Show(ShowOpts),
    /// Create a new listing (provider-side).
    CreateListing(CreateListingOpts),
    /// Deactivate own listing.
    Deactivate(DeactivateOpts),
    /// Show own listings (requires --address).
    MyListings(MyListingsOpts),
}

#[derive(Debug, Clone, PartialEq)]
pub struct ListOpts {
    pub model_id: Option<String>,
    pub limit: usize,
    pub json: bool,
}

impl Default for ListOpts {
    fn default() -> Self {
        Self {
            model_id: None,
            limit: 20,
            json: false,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BidOpts {
    pub model_id: String,
    pub max_price_input: u128,
    pub max_price_output: u128,
    pub expires_in: u64,
    pub rpc_url: String,
}

impl Default for BidOpts {
    fn default() -> Self {
        Self {
            model_id: String::new(),
            max_price_input: 0,
            max_price_output: 0,
            expires_in: 0,
            rpc_url: "http://127.0.0.1:9944".into(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DiscoverOpts {
    pub model_id: String,
    pub max_price_input: Option<u128>,
    pub max_price_output: Option<u128>,
    pub min_stake: Option<u128>,
    pub max_latency_ms: Option<u64>,
    pub arch_group: Option<String>,
    pub sort_by: SortField,
    pub limit: usize,
    pub json: bool,
}

impl Default for DiscoverOpts {
    fn default() -> Self {
        Self {
            model_id: String::new(),
            max_price_input: None,
            max_price_output: None,
            min_stake: None,
            max_latency_ms: None,
            arch_group: None,
            sort_by: SortField::Price,
            limit: 10,
            json: false,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ShowOpts {
    pub listing_id: u64,
    pub rpc_url: String,
    pub json: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CreateListingOpts {
    pub model_id: String,
    pub price_per_m_input: u128,
    pub price_per_m_output: u128,
    pub max_concurrency: u32,
    pub latency_sla_ms: u64,
    pub arch_group: String,
    pub rpc_url: String,
}

impl Default for CreateListingOpts {
    fn default() -> Self {
        Self {
            model_id: String::new(),
            price_per_m_input: 0,
            price_per_m_output: 0,
            max_concurrency: 1,
            latency_sla_ms: 100,
            arch_group: "sm90".into(),
            rpc_url: "http://127.0.0.1:9944".into(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DeactivateOpts {
    pub listing_id: u64,
    pub rpc_url: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct MyListingsOpts {
    pub address: String,
    pub rpc_url: String,
    pub json: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SortField {
    Price,
    Latency,
    Stake,
    Completed,
}

impl SortField {
    fn from_str(s: &str) -> Option<Self> {
        match s {
            "price" => Some(Self::Price),
            "latency" => Some(Self::Latency),
            "stake" => Some(Self::Stake),
            "completed" => Some(Self::Completed),
            _ => None,
        }
    }
}

impl fmt::Display for SortField {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SortField::Price => write!(f, "price"),
            SortField::Latency => write!(f, "latency"),
            SortField::Stake => write!(f, "stake"),
            SortField::Completed => write!(f, "completed"),
        }
    }
}

// ── Errors ──────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq)]
pub enum MarketParseError {
    NoSubcommand,
    UnknownSubcommand(String),
    MissingArg(String),
    InvalidValue(String, String),
}

impl fmt::Display for MarketParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MarketParseError::NoSubcommand => write!(f, "marketplace subcommand required (list, bid, discover, show, create, deactivate, my-listings)"),
            MarketParseError::UnknownSubcommand(s) => write!(f, "unknown marketplace subcommand: '{s}'"),
            MarketParseError::MissingArg(a) => write!(f, "missing required argument: {a}"),
            MarketParseError::InvalidValue(k, v) => write!(f, "invalid value for {k}: '{v}'"),
        }
    }
}

// ── Parser ──────────────────────────────────────────────────────────

fn take_value(args: &[String], idx: usize, name: &str) -> Result<String, MarketParseError> {
    args.get(idx)
        .cloned()
        .ok_or_else(|| MarketParseError::MissingArg(name.into()))
}

fn parse_u128(args: &[String], idx: usize, name: &str) -> Result<u128, MarketParseError> {
    let v = take_value(args, idx, name)?;
    v.parse()
        .map_err(|_| MarketParseError::InvalidValue(name.into(), v))
}

fn parse_u64(args: &[String], idx: usize, name: &str) -> Result<u64, MarketParseError> {
    let v = take_value(args, idx, name)?;
    v.parse()
        .map_err(|_| MarketParseError::InvalidValue(name.into(), v))
}

fn parse_u32(args: &[String], idx: usize, name: &str) -> Result<u32, MarketParseError> {
    let v = take_value(args, idx, name)?;
    v.parse()
        .map_err(|_| MarketParseError::InvalidValue(name.into(), v))
}

pub fn parse_market_args(args: &[String]) -> Result<MarketCmd, MarketParseError> {
    if args.is_empty() {
        return Err(MarketParseError::NoSubcommand);
    }

    match args[0].as_str() {
        "list" => parse_list(&args[1..]),
        "bid" => parse_bid(&args[1..]),
        "discover" => parse_discover(&args[1..]),
        "show" => parse_show(&args[1..]),
        "create" => parse_create(&args[1..]),
        "deactivate" => parse_deactivate(&args[1..]),
        "my-listings" => parse_my_listings(&args[1..]),
        other => Err(MarketParseError::UnknownSubcommand(other.into())),
    }
}

fn parse_list(args: &[String]) -> Result<MarketCmd, MarketParseError> {
    let mut opts = ListOpts::default();
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--model" | "-m" => {
                i += 1;
                opts.model_id = Some(take_value(args, i, "--model")?);
            }
            "--limit" | "-n" => {
                i += 1;
                let v = take_value(args, i, "--limit")?;
                opts.limit = v
                    .parse()
                    .map_err(|_| MarketParseError::InvalidValue("--limit".into(), v))?;
            }
            "--json" => opts.json = true,
            _ => {}
        }
        i += 1;
    }
    Ok(MarketCmd::List(opts))
}

fn parse_bid(args: &[String]) -> Result<MarketCmd, MarketParseError> {
    let mut opts = BidOpts::default();
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--model" | "-m" => {
                i += 1;
                opts.model_id = take_value(args, i, "--model")?;
            }
            "--max-price-input" => {
                i += 1;
                opts.max_price_input = parse_u128(args, i, "--max-price-input")?;
            }
            "--max-price-output" => {
                i += 1;
                opts.max_price_output = parse_u128(args, i, "--max-price-output")?;
            }
            "--expires-in" => {
                i += 1;
                opts.expires_in = parse_u64(args, i, "--expires-in")?;
            }
            "--rpc-url" => {
                i += 1;
                opts.rpc_url = take_value(args, i, "--rpc-url")?;
            }
            _ => {}
        }
        i += 1;
    }
    if opts.model_id.is_empty() {
        return Err(MarketParseError::MissingArg("--model".into()));
    }
    if opts.max_price_input == 0 {
        return Err(MarketParseError::MissingArg("--max-price-input".into()));
    }
    if opts.max_price_output == 0 {
        return Err(MarketParseError::MissingArg("--max-price-output".into()));
    }
    Ok(MarketCmd::Bid(opts))
}

fn parse_discover(args: &[String]) -> Result<MarketCmd, MarketParseError> {
    let mut opts = DiscoverOpts::default();
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--model" | "-m" => {
                i += 1;
                opts.model_id = take_value(args, i, "--model")?;
            }
            "--max-price-input" => {
                i += 1;
                opts.max_price_input = Some(parse_u128(args, i, "--max-price-input")?);
            }
            "--max-price-output" => {
                i += 1;
                opts.max_price_output = Some(parse_u128(args, i, "--max-price-output")?);
            }
            "--min-stake" => {
                i += 1;
                opts.min_stake = Some(parse_u128(args, i, "--min-stake")?);
            }
            "--max-latency" => {
                i += 1;
                opts.max_latency_ms = Some(parse_u64(args, i, "--max-latency")?);
            }
            "--arch" => {
                i += 1;
                opts.arch_group = Some(take_value(args, i, "--arch")?);
            }
            "--sort" => {
                i += 1;
                let v = take_value(args, i, "--sort")?;
                opts.sort_by = SortField::from_str(&v)
                    .ok_or_else(|| MarketParseError::InvalidValue("--sort".into(), v))?;
            }
            "--limit" | "-n" => {
                i += 1;
                let v = take_value(args, i, "--limit")?;
                opts.limit = v
                    .parse()
                    .map_err(|_| MarketParseError::InvalidValue("--limit".into(), v))?;
            }
            "--json" => opts.json = true,
            _ => {}
        }
        i += 1;
    }
    if opts.model_id.is_empty() {
        return Err(MarketParseError::MissingArg("--model".into()));
    }
    Ok(MarketCmd::Discover(opts))
}

fn parse_show(args: &[String]) -> Result<MarketCmd, MarketParseError> {
    let mut listing_id: Option<u64> = None;
    let mut rpc_url = "http://127.0.0.1:9944".to_string();
    let mut json = false;
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--id" => {
                i += 1;
                listing_id = Some(parse_u64(args, i, "--id")?);
            }
            "--rpc-url" => {
                i += 1;
                rpc_url = take_value(args, i, "--rpc-url")?;
            }
            "--json" => json = true,
            _ => {
                // Positional: first non-flag is listing ID
                if listing_id.is_none() {
                    listing_id = Some(args[i].parse().map_err(|_| {
                        MarketParseError::InvalidValue("listing-id".into(), args[i].clone())
                    })?);
                }
            }
        }
        i += 1;
    }
    let listing_id = listing_id.ok_or_else(|| MarketParseError::MissingArg("listing-id".into()))?;
    Ok(MarketCmd::Show(ShowOpts {
        listing_id,
        rpc_url,
        json,
    }))
}

fn parse_create(args: &[String]) -> Result<MarketCmd, MarketParseError> {
    let mut opts = CreateListingOpts::default();
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--model" | "-m" => {
                i += 1;
                opts.model_id = take_value(args, i, "--model")?;
            }
            "--price-input" => {
                i += 1;
                opts.price_per_m_input = parse_u128(args, i, "--price-input")?;
            }
            "--price-output" => {
                i += 1;
                opts.price_per_m_output = parse_u128(args, i, "--price-output")?;
            }
            "--concurrency" => {
                i += 1;
                opts.max_concurrency = parse_u32(args, i, "--concurrency")?;
            }
            "--latency-sla" => {
                i += 1;
                opts.latency_sla_ms = parse_u64(args, i, "--latency-sla")?;
            }
            "--arch" => {
                i += 1;
                opts.arch_group = take_value(args, i, "--arch")?;
            }
            "--rpc-url" => {
                i += 1;
                opts.rpc_url = take_value(args, i, "--rpc-url")?;
            }
            _ => {}
        }
        i += 1;
    }
    if opts.model_id.is_empty() {
        return Err(MarketParseError::MissingArg("--model".into()));
    }
    if opts.price_per_m_input == 0 {
        return Err(MarketParseError::MissingArg("--price-input".into()));
    }
    if opts.price_per_m_output == 0 {
        return Err(MarketParseError::MissingArg("--price-output".into()));
    }
    Ok(MarketCmd::CreateListing(opts))
}

fn parse_deactivate(args: &[String]) -> Result<MarketCmd, MarketParseError> {
    let mut listing_id: Option<u64> = None;
    let mut rpc_url = "http://127.0.0.1:9944".to_string();
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--id" => {
                i += 1;
                listing_id = Some(parse_u64(args, i, "--id")?);
            }
            "--rpc-url" => {
                i += 1;
                rpc_url = take_value(args, i, "--rpc-url")?;
            }
            _ => {
                if listing_id.is_none() {
                    listing_id = Some(args[i].parse().map_err(|_| {
                        MarketParseError::InvalidValue("listing-id".into(), args[i].clone())
                    })?);
                }
            }
        }
        i += 1;
    }
    let listing_id = listing_id.ok_or_else(|| MarketParseError::MissingArg("listing-id".into()))?;
    Ok(MarketCmd::Deactivate(DeactivateOpts {
        listing_id,
        rpc_url,
    }))
}

fn parse_my_listings(args: &[String]) -> Result<MarketCmd, MarketParseError> {
    let mut address = String::new();
    let mut rpc_url = "http://127.0.0.1:9944".to_string();
    let mut json = false;
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--address" | "-a" => {
                i += 1;
                address = take_value(args, i, "--address")?;
            }
            "--rpc-url" => {
                i += 1;
                rpc_url = take_value(args, i, "--rpc-url")?;
            }
            "--json" => json = true,
            _ => {}
        }
        i += 1;
    }
    if address.is_empty() {
        return Err(MarketParseError::MissingArg("--address".into()));
    }
    Ok(MarketCmd::MyListings(MyListingsOpts {
        address,
        rpc_url,
        json,
    }))
}

// ── Display formatting ──────────────────────────────────────────────

/// Format a listing for human-readable display.
pub fn format_listing(
    id: u64,
    provider: &str,
    model: &str,
    price_in: u128,
    price_out: u128,
    concurrency: u32,
    active_reqs: u32,
    latency_ms: u64,
    completed: u64,
    arch: &str,
    active: bool,
) -> String {
    let status = if active { "●" } else { "○" };
    format!(
        "{status} Listing #{id}\n  Provider:    {provider}\n  Model:       {model}\n  Price:       {price_in}/M in, {price_out}/M out\n  Capacity:    {active_reqs}/{concurrency}\n  Latency SLA: {latency_ms}ms (p95)\n  Completed:   {completed}\n  Arch:        {arch}"
    )
}

/// Format a bid for human-readable display.
pub fn format_bid(
    id: u64,
    client: &str,
    model: &str,
    max_in: u128,
    max_out: u128,
    expires: u64,
    matched: bool,
    matched_listing: Option<u64>,
) -> String {
    let status = if matched {
        format!("✓ matched → listing #{}", matched_listing.unwrap_or(0))
    } else if expires > 0 {
        format!("pending (expires epoch {expires})")
    } else {
        "pending".to_string()
    };
    format!(
        "Bid #{id} [{status}]\n  Client:    {client}\n  Model:     {model}\n  Max price: {max_in}/M in, {max_out}/M out"
    )
}

/// Format discovery results as a table.
pub fn format_discovery_table(entries: &[(u64, String, u128, u128, u64, u64, String)]) -> String {
    if entries.is_empty() {
        return "No providers found matching criteria.".to_string();
    }
    let mut lines = vec![format!(
        "{:<8} {:<12} {:>10} {:>10} {:>8} {:>10} {}",
        "ID", "Provider", "Price/In", "Price/Out", "Latency", "Completed", "Arch"
    )];
    lines.push("-".repeat(76));
    for (id, provider, pin, pout, lat, comp, arch) in entries {
        lines.push(format!(
            "{:<8} {:<12} {:>10} {:>10} {:>6}ms {:>10} {}",
            id, provider, pin, pout, lat, comp, arch
        ));
    }
    lines.join("\n")
}

// ── Help text ───────────────────────────────────────────────────────

pub fn help_text() -> &'static str {
    r#"prova marketplace — Model marketplace commands

USAGE:
    prova market <SUBCOMMAND> [OPTIONS]

SUBCOMMANDS:
    list              List marketplace listings
    bid               Place a bid for inference
    discover          Discover providers for a model
    show <ID>         Show listing details
    create            Create a new listing (provider)
    deactivate <ID>   Deactivate a listing
    my-listings       Show own listings

COMMON OPTIONS:
    --json            Output as JSON
    --rpc-url <URL>   RPC endpoint (default: http://127.0.0.1:9944)

EXAMPLES:
    prova market list --model llama-70b --json
    prova market discover --model llama-70b --sort price --max-latency 50
    prova market bid --model llama-70b --max-price-input 100 --max-price-output 200
    prova market create --model llama-70b --price-input 80 --price-output 150 --concurrency 4
    prova market show 42
    prova market deactivate 42
    prova market my-listings --address 0x1234...
"#
}

#[cfg(test)]
mod tests {
    use super::*;

    fn args(s: &str) -> Vec<String> {
        s.split_whitespace().map(String::from).collect()
    }

    // ── list ──

    #[test]
    fn test_list_defaults() {
        let cmd = parse_market_args(&args("list")).unwrap();
        assert_eq!(
            cmd,
            MarketCmd::List(ListOpts {
                model_id: None,
                limit: 20,
                json: false
            })
        );
    }

    #[test]
    fn test_list_with_model_and_json() {
        let cmd = parse_market_args(&args("list --model llama-70b --json --limit 5")).unwrap();
        assert_eq!(
            cmd,
            MarketCmd::List(ListOpts {
                model_id: Some("llama-70b".into()),
                limit: 5,
                json: true,
            })
        );
    }

    #[test]
    fn test_list_short_flags() {
        let cmd = parse_market_args(&args("list -m llama-70b -n 3")).unwrap();
        if let MarketCmd::List(opts) = cmd {
            assert_eq!(opts.model_id, Some("llama-70b".into()));
            assert_eq!(opts.limit, 3);
        } else {
            panic!("expected List");
        }
    }

    // ── bid ──

    #[test]
    fn test_bid_full() {
        let cmd = parse_market_args(&args(
            "bid --model llama-70b --max-price-input 100 --max-price-output 200 --expires-in 50",
        ))
        .unwrap();
        assert_eq!(
            cmd,
            MarketCmd::Bid(BidOpts {
                model_id: "llama-70b".into(),
                max_price_input: 100,
                max_price_output: 200,
                expires_in: 50,
                rpc_url: "http://127.0.0.1:9944".into(),
            })
        );
    }

    #[test]
    fn test_bid_missing_model() {
        let err = parse_market_args(&args("bid --max-price-input 100 --max-price-output 200"))
            .unwrap_err();
        assert_eq!(err, MarketParseError::MissingArg("--model".into()));
    }

    #[test]
    fn test_bid_missing_price_input() {
        let err = parse_market_args(&args("bid --model x --max-price-output 200")).unwrap_err();
        assert_eq!(
            err,
            MarketParseError::MissingArg("--max-price-input".into())
        );
    }

    #[test]
    fn test_bid_missing_price_output() {
        let err = parse_market_args(&args("bid --model x --max-price-input 100")).unwrap_err();
        assert_eq!(
            err,
            MarketParseError::MissingArg("--max-price-output".into())
        );
    }

    // ── discover ──

    #[test]
    fn test_discover_minimal() {
        let cmd = parse_market_args(&args("discover --model llama-70b")).unwrap();
        if let MarketCmd::Discover(opts) = cmd {
            assert_eq!(opts.model_id, "llama-70b");
            assert_eq!(opts.sort_by, SortField::Price);
            assert_eq!(opts.limit, 10);
            assert!(opts.max_price_input.is_none());
        } else {
            panic!("expected Discover");
        }
    }

    #[test]
    fn test_discover_all_filters() {
        let cmd = parse_market_args(&args(
            "discover --model llama-70b --max-price-input 500 --max-price-output 600 --min-stake 1000 --max-latency 50 --arch sm90 --sort stake --limit 3 --json"
        )).unwrap();
        assert_eq!(
            cmd,
            MarketCmd::Discover(DiscoverOpts {
                model_id: "llama-70b".into(),
                max_price_input: Some(500),
                max_price_output: Some(600),
                min_stake: Some(1000),
                max_latency_ms: Some(50),
                arch_group: Some("sm90".into()),
                sort_by: SortField::Stake,
                limit: 3,
                json: true,
            })
        );
    }

    #[test]
    fn test_discover_missing_model() {
        let err = parse_market_args(&args("discover --sort price")).unwrap_err();
        assert_eq!(err, MarketParseError::MissingArg("--model".into()));
    }

    #[test]
    fn test_discover_invalid_sort() {
        let err = parse_market_args(&args("discover --model x --sort bogus")).unwrap_err();
        assert_eq!(
            err,
            MarketParseError::InvalidValue("--sort".into(), "bogus".into())
        );
    }

    // ── show ──

    #[test]
    fn test_show_positional() {
        let cmd = parse_market_args(&args("show 42")).unwrap();
        assert_eq!(
            cmd,
            MarketCmd::Show(ShowOpts {
                listing_id: 42,
                rpc_url: "http://127.0.0.1:9944".into(),
                json: false,
            })
        );
    }

    #[test]
    fn test_show_with_flag() {
        let cmd = parse_market_args(&args("show --id 7 --json")).unwrap();
        assert_eq!(
            cmd,
            MarketCmd::Show(ShowOpts {
                listing_id: 7,
                rpc_url: "http://127.0.0.1:9944".into(),
                json: true,
            })
        );
    }

    #[test]
    fn test_show_missing_id() {
        let err = parse_market_args(&args("show")).unwrap_err();
        assert_eq!(err, MarketParseError::MissingArg("listing-id".into()));
    }

    // ── create ──

    #[test]
    fn test_create_listing() {
        let cmd = parse_market_args(&args(
            "create --model llama-70b --price-input 80 --price-output 150 --concurrency 4 --latency-sla 30 --arch sm89"
        )).unwrap();
        assert_eq!(
            cmd,
            MarketCmd::CreateListing(CreateListingOpts {
                model_id: "llama-70b".into(),
                price_per_m_input: 80,
                price_per_m_output: 150,
                max_concurrency: 4,
                latency_sla_ms: 30,
                arch_group: "sm89".into(),
                rpc_url: "http://127.0.0.1:9944".into(),
            })
        );
    }

    #[test]
    fn test_create_missing_model() {
        let err =
            parse_market_args(&args("create --price-input 80 --price-output 150")).unwrap_err();
        assert_eq!(err, MarketParseError::MissingArg("--model".into()));
    }

    #[test]
    fn test_create_missing_price() {
        let err = parse_market_args(&args("create --model x --price-output 150")).unwrap_err();
        assert_eq!(err, MarketParseError::MissingArg("--price-input".into()));
    }

    // ── deactivate ──

    #[test]
    fn test_deactivate_positional() {
        let cmd = parse_market_args(&args("deactivate 5")).unwrap();
        assert_eq!(
            cmd,
            MarketCmd::Deactivate(DeactivateOpts {
                listing_id: 5,
                rpc_url: "http://127.0.0.1:9944".into(),
            })
        );
    }

    #[test]
    fn test_deactivate_with_flag() {
        let cmd = parse_market_args(&args("deactivate --id 9")).unwrap();
        assert_eq!(
            cmd,
            MarketCmd::Deactivate(DeactivateOpts {
                listing_id: 9,
                rpc_url: "http://127.0.0.1:9944".into(),
            })
        );
    }

    #[test]
    fn test_deactivate_missing_id() {
        let err = parse_market_args(&args("deactivate")).unwrap_err();
        assert_eq!(err, MarketParseError::MissingArg("listing-id".into()));
    }

    // ── my-listings ──

    #[test]
    fn test_my_listings() {
        let cmd = parse_market_args(&args("my-listings --address 0x1234abcd --json")).unwrap();
        assert_eq!(
            cmd,
            MarketCmd::MyListings(MyListingsOpts {
                address: "0x1234abcd".into(),
                rpc_url: "http://127.0.0.1:9944".into(),
                json: true,
            })
        );
    }

    #[test]
    fn test_my_listings_missing_address() {
        let err = parse_market_args(&args("my-listings")).unwrap_err();
        assert_eq!(err, MarketParseError::MissingArg("--address".into()));
    }

    // ── errors ──

    #[test]
    fn test_no_subcommand() {
        let err = parse_market_args(&[]).unwrap_err();
        assert_eq!(err, MarketParseError::NoSubcommand);
    }

    #[test]
    fn test_unknown_subcommand() {
        let err = parse_market_args(&args("foobar")).unwrap_err();
        assert_eq!(err, MarketParseError::UnknownSubcommand("foobar".into()));
    }

    // ── formatting ──

    #[test]
    fn test_format_listing_active() {
        let s = format_listing(
            1,
            "0x01",
            "llama-70b",
            100,
            200,
            5,
            2,
            30,
            100,
            "sm90",
            true,
        );
        assert!(s.contains("●"));
        assert!(s.contains("llama-70b"));
        assert!(s.contains("2/5"));
    }

    #[test]
    fn test_format_listing_inactive() {
        let s = format_listing(1, "0x01", "llama-70b", 100, 200, 5, 0, 30, 0, "sm90", false);
        assert!(s.contains("○"));
    }

    #[test]
    fn test_format_bid_pending() {
        let s = format_bid(1, "0x02", "llama-70b", 100, 200, 50, false, None);
        assert!(s.contains("pending"));
        assert!(s.contains("expires epoch 50"));
    }

    #[test]
    fn test_format_bid_matched() {
        let s = format_bid(1, "0x02", "llama-70b", 100, 200, 0, true, Some(5));
        assert!(s.contains("matched"));
        assert!(s.contains("listing #5"));
    }

    #[test]
    fn test_format_discovery_table_empty() {
        let s = format_discovery_table(&[]);
        assert!(s.contains("No providers found"));
    }

    #[test]
    fn test_format_discovery_table() {
        let entries = vec![
            (
                1,
                "0x01…".to_string(),
                100u128,
                200u128,
                30u64,
                500u64,
                "sm90".to_string(),
            ),
            (
                2,
                "0x02…".to_string(),
                150,
                250,
                40,
                300,
                "sm89".to_string(),
            ),
        ];
        let s = format_discovery_table(&entries);
        assert!(s.contains("ID"));
        assert!(s.contains("0x01"));
        assert!(s.contains("0x02"));
        assert_eq!(s.lines().count(), 4); // header + separator + 2 rows
    }

    #[test]
    fn test_sort_field_display() {
        assert_eq!(format!("{}", SortField::Price), "price");
        assert_eq!(format!("{}", SortField::Latency), "latency");
        assert_eq!(format!("{}", SortField::Stake), "stake");
        assert_eq!(format!("{}", SortField::Completed), "completed");
    }

    #[test]
    fn test_sort_field_from_str() {
        assert_eq!(SortField::from_str("price"), Some(SortField::Price));
        assert_eq!(SortField::from_str("latency"), Some(SortField::Latency));
        assert_eq!(SortField::from_str("bogus"), None);
    }

    #[test]
    fn test_error_display() {
        assert!(format!("{}", MarketParseError::NoSubcommand).contains("subcommand required"));
        assert!(format!("{}", MarketParseError::UnknownSubcommand("x".into())).contains("'x'"));
        assert!(format!("{}", MarketParseError::MissingArg("--foo".into())).contains("--foo"));
        assert!(
            format!("{}", MarketParseError::InvalidValue("k".into(), "v".into())).contains("'v'")
        );
    }

    #[test]
    fn test_help_text() {
        let h = help_text();
        assert!(h.contains("marketplace"));
        assert!(h.contains("SUBCOMMANDS"));
        assert!(h.contains("discover"));
    }

    #[test]
    fn test_bid_custom_rpc() {
        let cmd = parse_market_args(&args(
            "bid --model x --max-price-input 1 --max-price-output 1 --rpc-url http://custom:1234",
        ))
        .unwrap();
        if let MarketCmd::Bid(opts) = cmd {
            assert_eq!(opts.rpc_url, "http://custom:1234");
        } else {
            panic!("expected Bid");
        }
    }

    #[test]
    fn test_create_defaults() {
        let cmd =
            parse_market_args(&args("create --model x --price-input 1 --price-output 1")).unwrap();
        if let MarketCmd::CreateListing(opts) = cmd {
            assert_eq!(opts.max_concurrency, 1);
            assert_eq!(opts.latency_sla_ms, 100);
            assert_eq!(opts.arch_group, "sm90");
        } else {
            panic!("expected CreateListing");
        }
    }

    #[test]
    fn test_invalid_listing_id() {
        let err = parse_market_args(&args("show notanumber")).unwrap_err();
        assert_eq!(
            err,
            MarketParseError::InvalidValue("listing-id".into(), "notanumber".into())
        );
    }
}
