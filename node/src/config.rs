//! Node configuration manager.
//!
//! Loads configuration from a TOML file with defaults, validates all fields,
//! and supports environment variable overrides (PROVA_ prefix).

use std::collections::HashMap;
use std::fmt;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::path::{Path, PathBuf};

// ── Core configuration struct ───────────────────────────────────────────────

/// Top-level node configuration.
#[derive(Debug, Clone, PartialEq)]
pub struct NodeConfig {
    pub network: NetworkConfig,
    pub chain: ChainConfig,
    pub storage: StorageConfig,
    pub rpc: RpcConfig,
    pub metrics: MetricsConfig,
    pub inference: InferenceConfig,
    pub logging: LoggingConfig,
}

#[derive(Debug, Clone, PartialEq)]
pub struct NetworkConfig {
    pub listen_addr: SocketAddr,
    pub external_addr: Option<SocketAddr>,
    pub max_peers: u32,
    pub bootstrap_peers: Vec<String>,
    pub gossip_interval_ms: u64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ChainConfig {
    pub chain_id: u64,
    pub genesis_hash: Option<String>,
    pub checkpoint_interval: u64,
    pub finality_depth: u64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct StorageConfig {
    pub data_dir: PathBuf,
    pub max_db_size_gb: u64,
    pub cache_size_mb: u64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RpcConfig {
    pub enabled: bool,
    pub listen_addr: SocketAddr,
    pub max_connections: u32,
    pub cors_origins: Vec<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct MetricsConfig {
    pub enabled: bool,
    pub listen_addr: SocketAddr,
    pub push_gateway: Option<String>,
    pub push_interval_secs: u64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct InferenceConfig {
    pub backend: InferenceBackend,
    pub max_batch_size: u32,
    pub timeout_secs: u64,
    pub gpu_device_ids: Vec<u32>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum InferenceBackend {
    Mock,
    LlamaCpp,
    TensorRT,
}

#[derive(Debug, Clone, PartialEq)]
pub enum LogLevel {
    Trace,
    Debug,
    Info,
    Warn,
    Error,
}

#[derive(Debug, Clone, PartialEq)]
pub struct LoggingConfig {
    pub level: LogLevel,
    pub file: Option<PathBuf>,
    pub json: bool,
}

// ── Defaults ────────────────────────────────────────────────────────────────

impl Default for NodeConfig {
    fn default() -> Self {
        Self {
            network: NetworkConfig::default(),
            chain: ChainConfig::default(),
            storage: StorageConfig::default(),
            rpc: RpcConfig::default(),
            metrics: MetricsConfig::default(),
            inference: InferenceConfig::default(),
            logging: LoggingConfig::default(),
        }
    }
}

impl Default for NetworkConfig {
    fn default() -> Self {
        Self {
            listen_addr: SocketAddr::new(IpAddr::V4(Ipv4Addr::UNSPECIFIED), 30303),
            external_addr: None,
            max_peers: 50,
            bootstrap_peers: Vec::new(),
            gossip_interval_ms: 1000,
        }
    }
}

impl Default for ChainConfig {
    fn default() -> Self {
        Self {
            chain_id: 1,
            genesis_hash: None,
            checkpoint_interval: 900,
            finality_depth: 30,
        }
    }
}

impl Default for StorageConfig {
    fn default() -> Self {
        Self {
            data_dir: PathBuf::from("./prova-data"),
            max_db_size_gb: 100,
            cache_size_mb: 512,
        }
    }
}

impl Default for RpcConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            listen_addr: SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 8545),
            max_connections: 100,
            cors_origins: vec!["*".to_string()],
        }
    }
}

impl Default for MetricsConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            listen_addr: SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 9090),
            push_gateway: None,
            push_interval_secs: 15,
        }
    }
}

impl Default for InferenceConfig {
    fn default() -> Self {
        Self {
            backend: InferenceBackend::Mock,
            max_batch_size: 8,
            timeout_secs: 300,
            gpu_device_ids: Vec::new(),
        }
    }
}

impl Default for LoggingConfig {
    fn default() -> Self {
        Self {
            level: LogLevel::Info,
            file: None,
            json: false,
        }
    }
}

// ── Error type ──────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq)]
pub struct ConfigError {
    pub field: String,
    pub message: String,
}

impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "config error [{}]: {}", self.field, self.message)
    }
}

// ── TOML-like parser (no external dependency) ───────────────────────────────

/// Parses a simple TOML file into a flat key-value map.
/// Supports `[section]` headers and `key = value` pairs.
/// Values: strings (quoted), integers, booleans, arrays (comma-separated in brackets).
pub fn parse_toml(input: &str) -> Result<HashMap<String, String>, ConfigError> {
    let mut map = HashMap::new();
    let mut section = String::new();

    for (line_num, raw_line) in input.lines().enumerate() {
        let line = raw_line.split('#').next().unwrap_or("").trim();
        if line.is_empty() {
            continue;
        }

        // Section header
        if line.starts_with('[') && line.ends_with(']') {
            section = line[1..line.len() - 1].trim().to_string();
            continue;
        }

        // Key = Value
        let eq_pos = line.find('=').ok_or_else(|| ConfigError {
            field: format!("line {}", line_num + 1),
            message: format!("expected key = value, got: {}", line),
        })?;

        let key = line[..eq_pos].trim();
        let val = line[eq_pos + 1..].trim();

        let full_key = if section.is_empty() {
            key.to_string()
        } else {
            format!("{}.{}", section, key)
        };

        // Strip quotes from string values
        let val = if (val.starts_with('"') && val.ends_with('"'))
            || (val.starts_with('\'') && val.ends_with('\''))
        {
            val[1..val.len() - 1].to_string()
        } else {
            val.to_string()
        };

        map.insert(full_key, val);
    }

    Ok(map)
}

// ── Config builder ──────────────────────────────────────────────────────────

impl NodeConfig {
    /// Load config from TOML string, applying defaults for missing fields.
    pub fn from_toml(toml: &str) -> Result<Self, ConfigError> {
        let map = parse_toml(toml)?;
        Self::from_map(&map)
    }

    /// Load from a flat key-value map (post-parse).
    pub fn from_map(map: &HashMap<String, String>) -> Result<Self, ConfigError> {
        let mut cfg = NodeConfig::default();

        // Network
        if let Some(v) = map.get("network.listen_addr") {
            cfg.network.listen_addr = parse_socket_addr(v, "network.listen_addr")?;
        }
        if let Some(v) = map.get("network.external_addr") {
            cfg.network.external_addr = Some(parse_socket_addr(v, "network.external_addr")?);
        }
        if let Some(v) = map.get("network.max_peers") {
            cfg.network.max_peers = parse_u32(v, "network.max_peers")?;
        }
        if let Some(v) = map.get("network.bootstrap_peers") {
            cfg.network.bootstrap_peers = parse_string_array(v);
        }
        if let Some(v) = map.get("network.gossip_interval_ms") {
            cfg.network.gossip_interval_ms = parse_u64(v, "network.gossip_interval_ms")?;
        }

        // Chain
        if let Some(v) = map.get("chain.chain_id") {
            cfg.chain.chain_id = parse_u64(v, "chain.chain_id")?;
        }
        if let Some(v) = map.get("chain.genesis_hash") {
            cfg.chain.genesis_hash = Some(v.clone());
        }
        if let Some(v) = map.get("chain.checkpoint_interval") {
            cfg.chain.checkpoint_interval = parse_u64(v, "chain.checkpoint_interval")?;
        }
        if let Some(v) = map.get("chain.finality_depth") {
            cfg.chain.finality_depth = parse_u64(v, "chain.finality_depth")?;
        }

        // Storage
        if let Some(v) = map.get("storage.data_dir") {
            cfg.storage.data_dir = PathBuf::from(v);
        }
        if let Some(v) = map.get("storage.max_db_size_gb") {
            cfg.storage.max_db_size_gb = parse_u64(v, "storage.max_db_size_gb")?;
        }
        if let Some(v) = map.get("storage.cache_size_mb") {
            cfg.storage.cache_size_mb = parse_u64(v, "storage.cache_size_mb")?;
        }

        // RPC
        if let Some(v) = map.get("rpc.enabled") {
            cfg.rpc.enabled = parse_bool(v, "rpc.enabled")?;
        }
        if let Some(v) = map.get("rpc.listen_addr") {
            cfg.rpc.listen_addr = parse_socket_addr(v, "rpc.listen_addr")?;
        }
        if let Some(v) = map.get("rpc.max_connections") {
            cfg.rpc.max_connections = parse_u32(v, "rpc.max_connections")?;
        }
        if let Some(v) = map.get("rpc.cors_origins") {
            cfg.rpc.cors_origins = parse_string_array(v);
        }

        // Metrics
        if let Some(v) = map.get("metrics.enabled") {
            cfg.metrics.enabled = parse_bool(v, "metrics.enabled")?;
        }
        if let Some(v) = map.get("metrics.listen_addr") {
            cfg.metrics.listen_addr = parse_socket_addr(v, "metrics.listen_addr")?;
        }
        if let Some(v) = map.get("metrics.push_gateway") {
            cfg.metrics.push_gateway = Some(v.clone());
        }
        if let Some(v) = map.get("metrics.push_interval_secs") {
            cfg.metrics.push_interval_secs = parse_u64(v, "metrics.push_interval_secs")?;
        }

        // Inference
        if let Some(v) = map.get("inference.backend") {
            cfg.inference.backend = match v.as_str() {
                "mock" => InferenceBackend::Mock,
                "llamacpp" | "llama.cpp" => InferenceBackend::LlamaCpp,
                "tensorrt" => InferenceBackend::TensorRT,
                _ => {
                    return Err(ConfigError {
                        field: "inference.backend".into(),
                        message: format!(
                            "unknown backend: {} (expected mock|llamacpp|tensorrt)",
                            v
                        ),
                    })
                }
            };
        }
        if let Some(v) = map.get("inference.max_batch_size") {
            cfg.inference.max_batch_size = parse_u32(v, "inference.max_batch_size")?;
        }
        if let Some(v) = map.get("inference.timeout_secs") {
            cfg.inference.timeout_secs = parse_u64(v, "inference.timeout_secs")?;
        }
        if let Some(v) = map.get("inference.gpu_device_ids") {
            cfg.inference.gpu_device_ids = parse_u32_array(v, "inference.gpu_device_ids")?;
        }

        // Logging
        if let Some(v) = map.get("logging.level") {
            cfg.logging.level = match v.to_lowercase().as_str() {
                "trace" => LogLevel::Trace,
                "debug" => LogLevel::Debug,
                "info" => LogLevel::Info,
                "warn" | "warning" => LogLevel::Warn,
                "error" => LogLevel::Error,
                _ => {
                    return Err(ConfigError {
                        field: "logging.level".into(),
                        message: format!(
                            "unknown level: {} (expected trace|debug|info|warn|error)",
                            v
                        ),
                    })
                }
            };
        }
        if let Some(v) = map.get("logging.file") {
            cfg.logging.file = Some(PathBuf::from(v));
        }
        if let Some(v) = map.get("logging.json") {
            cfg.logging.json = parse_bool(v, "logging.json")?;
        }

        cfg.validate()?;
        Ok(cfg)
    }

    /// Apply environment variable overrides (PROVA_ prefix).
    /// e.g. PROVA_NETWORK_LISTEN_ADDR=0.0.0.0:30303
    pub fn apply_env_overrides(map: &mut HashMap<String, String>, env_vars: &[(String, String)]) {
        for (key, val) in env_vars {
            if let Some(suffix) = key.strip_prefix("PROVA_") {
                // Convention: use dots for section separators in env vars
                // e.g. PROVA_network.max_peers or double-underscore:
                // PROVA_NETWORK__MAX_PEERS → network.max_peers
                let config_key = if suffix.contains("__") {
                    suffix
                        .to_lowercase()
                        .replacen("__", ".", 1)
                        .replace("__", "_")
                } else if suffix.contains('.') {
                    suffix.to_lowercase()
                } else {
                    // Single underscore: first _ becomes dot (section.key)
                    let lower = suffix.to_lowercase();
                    if let Some(pos) = lower.find('_') {
                        format!("{}.{}", &lower[..pos], &lower[pos + 1..])
                    } else {
                        lower
                    }
                };
                map.insert(config_key, val.clone());
            }
        }
    }

    /// Validate the configuration for consistency.
    pub fn validate(&self) -> Result<(), ConfigError> {
        if self.network.max_peers == 0 {
            return Err(ConfigError {
                field: "network.max_peers".into(),
                message: "must be > 0".into(),
            });
        }
        if self.network.gossip_interval_ms == 0 {
            return Err(ConfigError {
                field: "network.gossip_interval_ms".into(),
                message: "must be > 0".into(),
            });
        }
        if self.chain.chain_id == 0 {
            return Err(ConfigError {
                field: "chain.chain_id".into(),
                message: "must be > 0".into(),
            });
        }
        if self.chain.checkpoint_interval == 0 {
            return Err(ConfigError {
                field: "chain.checkpoint_interval".into(),
                message: "must be > 0".into(),
            });
        }
        if self.chain.finality_depth == 0 {
            return Err(ConfigError {
                field: "chain.finality_depth".into(),
                message: "must be > 0".into(),
            });
        }
        if self.storage.max_db_size_gb == 0 {
            return Err(ConfigError {
                field: "storage.max_db_size_gb".into(),
                message: "must be > 0".into(),
            });
        }
        if self.storage.cache_size_mb == 0 {
            return Err(ConfigError {
                field: "storage.cache_size_mb".into(),
                message: "must be > 0".into(),
            });
        }
        if self.rpc.max_connections == 0 {
            return Err(ConfigError {
                field: "rpc.max_connections".into(),
                message: "must be > 0".into(),
            });
        }
        if self.inference.max_batch_size == 0 {
            return Err(ConfigError {
                field: "inference.max_batch_size".into(),
                message: "must be > 0".into(),
            });
        }
        if self.inference.timeout_secs == 0 {
            return Err(ConfigError {
                field: "inference.timeout_secs".into(),
                message: "must be > 0".into(),
            });
        }
        if self.metrics.push_interval_secs == 0 {
            return Err(ConfigError {
                field: "metrics.push_interval_secs".into(),
                message: "must be > 0".into(),
            });
        }

        // Validate genesis hash format if provided
        if let Some(ref h) = self.chain.genesis_hash {
            if !h.chars().all(|c| c.is_ascii_hexdigit()) || h.len() != 64 {
                return Err(ConfigError {
                    field: "chain.genesis_hash".into(),
                    message: "must be 64 hex characters".into(),
                });
            }
        }

        Ok(())
    }

    /// Generate a default TOML config string.
    pub fn to_toml(&self) -> String {
        let mut out = String::new();
        out.push_str("# Prova Node Configuration\n\n");

        out.push_str("[network]\n");
        out.push_str(&format!("listen_addr = \"{}\"\n", self.network.listen_addr));
        if let Some(ref ext) = self.network.external_addr {
            out.push_str(&format!("external_addr = \"{}\"\n", ext));
        }
        out.push_str(&format!("max_peers = {}\n", self.network.max_peers));
        out.push_str(&format!(
            "bootstrap_peers = [{}]\n",
            self.network
                .bootstrap_peers
                .iter()
                .map(|p| format!("\"{}\"", p))
                .collect::<Vec<_>>()
                .join(", ")
        ));
        out.push_str(&format!(
            "gossip_interval_ms = {}\n",
            self.network.gossip_interval_ms
        ));

        out.push_str("\n[chain]\n");
        out.push_str(&format!("chain_id = {}\n", self.chain.chain_id));
        if let Some(ref h) = self.chain.genesis_hash {
            out.push_str(&format!("genesis_hash = \"{}\"\n", h));
        }
        out.push_str(&format!(
            "checkpoint_interval = {}\n",
            self.chain.checkpoint_interval
        ));
        out.push_str(&format!("finality_depth = {}\n", self.chain.finality_depth));

        out.push_str("\n[storage]\n");
        out.push_str(&format!(
            "data_dir = \"{}\"\n",
            self.storage.data_dir.display()
        ));
        out.push_str(&format!(
            "max_db_size_gb = {}\n",
            self.storage.max_db_size_gb
        ));
        out.push_str(&format!("cache_size_mb = {}\n", self.storage.cache_size_mb));

        out.push_str("\n[rpc]\n");
        out.push_str(&format!("enabled = {}\n", self.rpc.enabled));
        out.push_str(&format!("listen_addr = \"{}\"\n", self.rpc.listen_addr));
        out.push_str(&format!("max_connections = {}\n", self.rpc.max_connections));
        out.push_str(&format!(
            "cors_origins = [{}]\n",
            self.rpc
                .cors_origins
                .iter()
                .map(|o| format!("\"{}\"", o))
                .collect::<Vec<_>>()
                .join(", ")
        ));

        out.push_str("\n[metrics]\n");
        out.push_str(&format!("enabled = {}\n", self.metrics.enabled));
        out.push_str(&format!("listen_addr = \"{}\"\n", self.metrics.listen_addr));
        if let Some(ref gw) = self.metrics.push_gateway {
            out.push_str(&format!("push_gateway = \"{}\"\n", gw));
        }
        out.push_str(&format!(
            "push_interval_secs = {}\n",
            self.metrics.push_interval_secs
        ));

        out.push_str("\n[inference]\n");
        let backend_str = match self.inference.backend {
            InferenceBackend::Mock => "mock",
            InferenceBackend::LlamaCpp => "llamacpp",
            InferenceBackend::TensorRT => "tensorrt",
        };
        out.push_str(&format!("backend = \"{}\"\n", backend_str));
        out.push_str(&format!(
            "max_batch_size = {}\n",
            self.inference.max_batch_size
        ));
        out.push_str(&format!("timeout_secs = {}\n", self.inference.timeout_secs));
        out.push_str(&format!(
            "gpu_device_ids = [{}]\n",
            self.inference
                .gpu_device_ids
                .iter()
                .map(|id| id.to_string())
                .collect::<Vec<_>>()
                .join(", ")
        ));

        out.push_str("\n[logging]\n");
        let level_str = match self.logging.level {
            LogLevel::Trace => "trace",
            LogLevel::Debug => "debug",
            LogLevel::Info => "info",
            LogLevel::Warn => "warn",
            LogLevel::Error => "error",
        };
        out.push_str(&format!("level = \"{}\"\n", level_str));
        if let Some(ref f) = self.logging.file {
            out.push_str(&format!("file = \"{}\"\n", f.display()));
        }
        out.push_str(&format!("json = {}\n", self.logging.json));

        out
    }

    /// Load from file path (reads file, parses TOML, applies env overrides).
    pub fn from_file(path: &Path, env_vars: &[(String, String)]) -> Result<Self, ConfigError> {
        let content = std::fs::read_to_string(path).map_err(|e| ConfigError {
            field: "file".into(),
            message: format!("cannot read {}: {}", path.display(), e),
        })?;
        let mut map = parse_toml(&content)?;
        Self::apply_env_overrides(&mut map, env_vars);
        Self::from_map(&map)
    }
}

// ── Parse helpers ───────────────────────────────────────────────────────────

fn parse_socket_addr(s: &str, field: &str) -> Result<SocketAddr, ConfigError> {
    s.parse().map_err(|_| ConfigError {
        field: field.into(),
        message: format!("invalid socket address: {}", s),
    })
}

fn parse_u32(s: &str, field: &str) -> Result<u32, ConfigError> {
    s.parse().map_err(|_| ConfigError {
        field: field.into(),
        message: format!("invalid u32: {}", s),
    })
}

fn parse_u64(s: &str, field: &str) -> Result<u64, ConfigError> {
    s.parse().map_err(|_| ConfigError {
        field: field.into(),
        message: format!("invalid u64: {}", s),
    })
}

fn parse_bool(s: &str, field: &str) -> Result<bool, ConfigError> {
    match s {
        "true" => Ok(true),
        "false" => Ok(false),
        _ => Err(ConfigError {
            field: field.into(),
            message: format!("invalid bool: {} (expected true|false)", s),
        }),
    }
}

fn parse_string_array(s: &str) -> Vec<String> {
    let s = s.trim();
    let s = if s.starts_with('[') && s.ends_with(']') {
        &s[1..s.len() - 1]
    } else {
        s
    };
    s.split(',')
        .map(|item| {
            let item = item.trim();
            if (item.starts_with('"') && item.ends_with('"'))
                || (item.starts_with('\'') && item.ends_with('\''))
            {
                item[1..item.len() - 1].to_string()
            } else {
                item.to_string()
            }
        })
        .filter(|item| !item.is_empty())
        .collect()
}

fn parse_u32_array(s: &str, field: &str) -> Result<Vec<u32>, ConfigError> {
    let s = s.trim();
    let s = if s.starts_with('[') && s.ends_with(']') {
        &s[1..s.len() - 1]
    } else {
        s
    };
    if s.trim().is_empty() {
        return Ok(Vec::new());
    }
    s.split(',')
        .map(|item| {
            item.trim().parse::<u32>().map_err(|_| ConfigError {
                field: field.into(),
                message: format!("invalid u32 in array: {}", item.trim()),
            })
        })
        .collect()
}

// ── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let cfg = NodeConfig::default();
        assert_eq!(cfg.network.listen_addr.port(), 30303);
        assert_eq!(cfg.rpc.listen_addr.port(), 8545);
        assert_eq!(cfg.chain.chain_id, 1);
        assert_eq!(cfg.inference.backend, InferenceBackend::Mock);
        assert_eq!(cfg.logging.level, LogLevel::Info);
        assert!(cfg.validate().is_ok());
    }

    #[test]
    fn test_parse_empty_toml() {
        let cfg = NodeConfig::from_toml("").unwrap();
        assert_eq!(cfg, NodeConfig::default());
    }

    #[test]
    fn test_parse_full_toml() {
        let toml = r#"
[network]
listen_addr = "0.0.0.0:9999"
max_peers = 100
gossip_interval_ms = 500

[chain]
chain_id = 42
finality_depth = 60

[storage]
data_dir = "/var/prova"
max_db_size_gb = 500

[rpc]
enabled = false
listen_addr = "127.0.0.1:9000"

[inference]
backend = "tensorrt"
gpu_device_ids = [0, 1, 2]
max_batch_size = 16

[logging]
level = "debug"
json = true
"#;
        let cfg = NodeConfig::from_toml(toml).unwrap();
        assert_eq!(cfg.network.listen_addr.port(), 9999);
        assert_eq!(cfg.network.max_peers, 100);
        assert_eq!(cfg.chain.chain_id, 42);
        assert_eq!(cfg.chain.finality_depth, 60);
        assert_eq!(cfg.storage.data_dir, PathBuf::from("/var/prova"));
        assert_eq!(cfg.storage.max_db_size_gb, 500);
        assert!(!cfg.rpc.enabled);
        assert_eq!(cfg.inference.backend, InferenceBackend::TensorRT);
        assert_eq!(cfg.inference.gpu_device_ids, vec![0, 1, 2]);
        assert_eq!(cfg.inference.max_batch_size, 16);
        assert_eq!(cfg.logging.level, LogLevel::Debug);
        assert!(cfg.logging.json);
    }

    #[test]
    fn test_parse_with_comments() {
        let toml = r#"
# Main network config
[network]
listen_addr = "0.0.0.0:30303" # P2P port
max_peers = 25 # Keep it small
"#;
        let cfg = NodeConfig::from_toml(toml).unwrap();
        assert_eq!(cfg.network.max_peers, 25);
    }

    #[test]
    fn test_parse_quoted_strings() {
        let toml = r#"
[chain]
genesis_hash = "abcdef0123456789abcdef0123456789abcdef0123456789abcdef0123456789"
"#;
        let cfg = NodeConfig::from_toml(toml).unwrap();
        assert_eq!(
            cfg.chain.genesis_hash.as_deref(),
            Some("abcdef0123456789abcdef0123456789abcdef0123456789abcdef0123456789")
        );
    }

    #[test]
    fn test_validate_zero_peers() {
        let mut cfg = NodeConfig::default();
        cfg.network.max_peers = 0;
        assert_eq!(cfg.validate().unwrap_err().field, "network.max_peers");
    }

    #[test]
    fn test_validate_zero_chain_id() {
        let mut cfg = NodeConfig::default();
        cfg.chain.chain_id = 0;
        assert_eq!(cfg.validate().unwrap_err().field, "chain.chain_id");
    }

    #[test]
    fn test_validate_bad_genesis_hash() {
        let mut cfg = NodeConfig::default();
        cfg.chain.genesis_hash = Some("tooshort".into());
        assert_eq!(cfg.validate().unwrap_err().field, "chain.genesis_hash");
    }

    #[test]
    fn test_validate_zero_batch_size() {
        let mut cfg = NodeConfig::default();
        cfg.inference.max_batch_size = 0;
        assert_eq!(
            cfg.validate().unwrap_err().field,
            "inference.max_batch_size"
        );
    }

    #[test]
    fn test_validate_zero_timeout() {
        let mut cfg = NodeConfig::default();
        cfg.inference.timeout_secs = 0;
        assert_eq!(cfg.validate().unwrap_err().field, "inference.timeout_secs");
    }

    #[test]
    fn test_validate_zero_cache() {
        let mut cfg = NodeConfig::default();
        cfg.storage.cache_size_mb = 0;
        assert_eq!(cfg.validate().unwrap_err().field, "storage.cache_size_mb");
    }

    #[test]
    fn test_env_overrides() {
        let mut map = parse_toml("[network]\nmax_peers = 10").unwrap();
        // Env override uses lowercase dot notation
        let env2: Vec<(String, String)> = vec![("PROVA_network.max_peers".into(), "200".into())];
        NodeConfig::apply_env_overrides(&mut map, &env2);
        let cfg = NodeConfig::from_map(&map).unwrap();
        assert_eq!(cfg.network.max_peers, 200);
    }

    #[test]
    fn test_invalid_backend() {
        let toml = r#"
[inference]
backend = "pytorch"
"#;
        let err = NodeConfig::from_toml(toml).unwrap_err();
        assert_eq!(err.field, "inference.backend");
    }

    #[test]
    fn test_invalid_log_level() {
        let toml = r#"
[logging]
level = "verbose"
"#;
        let err = NodeConfig::from_toml(toml).unwrap_err();
        assert_eq!(err.field, "logging.level");
    }

    #[test]
    fn test_invalid_socket_addr() {
        let toml = r#"
[network]
listen_addr = "not-an-address"
"#;
        let err = NodeConfig::from_toml(toml).unwrap_err();
        assert_eq!(err.field, "network.listen_addr");
    }

    #[test]
    fn test_to_toml_roundtrip() {
        let cfg = NodeConfig::default();
        let toml_str = cfg.to_toml();
        let cfg2 = NodeConfig::from_toml(&toml_str).unwrap();
        assert_eq!(cfg, cfg2);
    }

    #[test]
    fn test_to_toml_roundtrip_custom() {
        let mut cfg = NodeConfig::default();
        cfg.chain.chain_id = 99;
        cfg.inference.backend = InferenceBackend::LlamaCpp;
        cfg.inference.gpu_device_ids = vec![0, 1];
        cfg.logging.level = LogLevel::Trace;
        cfg.logging.json = true;
        cfg.network.bootstrap_peers = vec![
            "/ip4/1.2.3.4/tcp/30303".into(),
            "/ip4/5.6.7.8/tcp/30303".into(),
        ];
        let toml_str = cfg.to_toml();
        let cfg2 = NodeConfig::from_toml(&toml_str).unwrap();
        assert_eq!(cfg, cfg2);
    }

    #[test]
    fn test_parse_string_array() {
        let arr = parse_string_array(r#"["a", "b", "c"]"#);
        assert_eq!(arr, vec!["a", "b", "c"]);
    }

    #[test]
    fn test_parse_u32_array_empty() {
        let arr = parse_u32_array("[]", "test").unwrap();
        assert!(arr.is_empty());
    }

    #[test]
    fn test_parse_u32_array_values() {
        let arr = parse_u32_array("[0, 1, 3]", "test").unwrap();
        assert_eq!(arr, vec![0, 1, 3]);
    }

    #[test]
    fn test_bootstrap_peers_array() {
        let toml = r#"
[network]
bootstrap_peers = ["/ip4/1.2.3.4/tcp/30303", "/ip4/5.6.7.8/tcp/30303"]
"#;
        let cfg = NodeConfig::from_toml(toml).unwrap();
        assert_eq!(cfg.network.bootstrap_peers.len(), 2);
    }

    #[test]
    fn test_metrics_push_gateway() {
        let toml = r#"
[metrics]
push_gateway = "http://prometheus:9091"
push_interval_secs = 30
"#;
        let cfg = NodeConfig::from_toml(toml).unwrap();
        assert_eq!(
            cfg.metrics.push_gateway.as_deref(),
            Some("http://prometheus:9091")
        );
        assert_eq!(cfg.metrics.push_interval_secs, 30);
    }

    #[test]
    fn test_logging_file() {
        let toml = r#"
[logging]
level = "warn"
file = "/var/log/prova.log"
"#;
        let cfg = NodeConfig::from_toml(toml).unwrap();
        assert_eq!(cfg.logging.level, LogLevel::Warn);
        assert_eq!(
            cfg.logging.file.as_deref(),
            Some(Path::new("/var/log/prova.log"))
        );
    }

    #[test]
    fn test_error_display() {
        let err = ConfigError {
            field: "test.field".into(),
            message: "bad value".into(),
        };
        assert_eq!(format!("{}", err), "config error [test.field]: bad value");
    }
}
