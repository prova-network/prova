// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright (c) 2026 Prova Network contributors.

// Package config loads and validates the prover daemon configuration.
package config

import (
	"fmt"
	"os"

	"github.com/BurntSushi/toml"
)

// Config is the top-level prover configuration.
type Config struct {
	Identity  IdentityConfig  `toml:"identity"`
	Chain     ChainConfig     `toml:"chain"`
	Storage   StorageConfig   `toml:"storage"`
	HTTP      HTTPConfig      `toml:"http"`
	Metrics   MetricsConfig   `toml:"metrics"`
	SourceURL SourceURLPolicy `toml:"source_url"`
}

// IdentityConfig controls prover identity and signing.
type IdentityConfig struct {
	// KeystorePath is a path to an encrypted private key file (go-ethereum keystore).
	// Mutually exclusive with PrivateKeyHex.
	KeystorePath string `toml:"keystore_path"`

	// PrivateKeyHex is the hex-encoded ECDSA private key. Insecure, for testnet only.
	PrivateKeyHex string `toml:"private_key_hex"`

	// Passphrase for decrypting KeystorePath, if set. Prefer $PROVA_KEYSTORE_PASSPHRASE env var.
	Passphrase string `toml:"passphrase"`
}

// ChainConfig controls how the prover talks to Base (or other EVM chain).
type ChainConfig struct {
	// RPCURL is the Base (or Base Sepolia) JSON-RPC endpoint.
	RPCURL string `toml:"rpc_url"`

	// ChainID is the expected chain ID (8453 = Base mainnet, 84532 = Base Sepolia).
	ChainID uint64 `toml:"chain_id"`

	// Contract addresses (set after `forge script` deploy or hard-coded per chain).
	Contracts Contracts `toml:"contracts"`

	// PollInterval is how often to poll for new events (if not subscribing via websocket).
	PollIntervalSeconds int `toml:"poll_interval_seconds"`

	// BlockLookback is the reorg-safety margin applied when filtering events.
	// Events from blocks > (currentBlock - BlockLookback) are not yet
	// considered final. Use *uint64 so 0 is a valid value (useful on anvil
	// and other single-proposer testnets where blocks are final immediately).
	// nil = use default (6).
	BlockLookback *uint64 `toml:"block_lookback"`
}

// Contracts holds deployed Prova contract addresses on the target chain.
type Contracts struct {
	ProvaToken         string `toml:"prova_token"`
	ProofVerifier      string `toml:"proof_verifier"`
	ProverRegistry     string `toml:"prover_registry"`
	ProverStaking      string `toml:"prover_staking"`
	ContentRegistry    string `toml:"content_registry"`
	StorageMarketplace string `toml:"storage_marketplace"`
}

// StorageConfig controls local blob storage.
type StorageConfig struct {
	// DataDir is the directory where stored pieces live.
	DataDir string `toml:"data_dir"`

	// MaxBytes is the soft cap on total storage. 0 = no cap.
	MaxBytes uint64 `toml:"max_bytes"`

	// IndexPath is where the piece-CID -> on-disk-path index database lives.
	// Default: <DataDir>/index.sqlite
	IndexPath string `toml:"index_path"`
}

// HTTPConfig controls the optional HTTPS retrieval endpoint.
type HTTPConfig struct {
	// Enabled exposes this prover for HTTPS retrieval.
	Enabled bool `toml:"enabled"`

	// ListenAddr is the bind address for HTTPS.
	ListenAddr string `toml:"listen_addr"`

	// CertPath and KeyPath for TLS. If both empty, use ACME via LetsEncrypt on port 80.
	CertPath string `toml:"cert_path"`
	KeyPath  string `toml:"key_path"`

	// PublicURL is the URL that clients use to reach this prover, advertised
	// in the ProverRegistry. e.g., https://prover.example.com
	PublicURL string `toml:"public_url"`
}

// MetricsConfig controls Prometheus metrics export.
type MetricsConfig struct {
	Enabled    bool   `toml:"enabled"`
	ListenAddr string `toml:"listen_addr"`
}

// SourceURLPolicy controls how the prover discovers piece source URLs
// given only the on-chain DealProposed event (which does not carry a URL
// field in v1).
type SourceURLPolicy struct {
	// Template, when non-empty, derives a piece URL from the client address
	// and CommP hash. Supported substitutions:
	//   {client}     lowercase hex client address with 0x prefix
	//   {clientRaw}  lowercase hex client address without 0x prefix
	//   {commpHex}   64-char hex of the 32-byte commP hash
	//   {commpCid}   the full CID string
	//
	// Example: "https://clients.example.com/{client}/{commpCid}"
	Template string `toml:"template"`

	// AllowInsecure permits http:// and private IP hosts in derived URLs.
	// Never set this in production.
	AllowInsecure bool `toml:"allow_insecure"`
}

// Load reads and validates a config file.
func Load(path string) (*Config, error) {
	if path == "" {
		return nil, fmt.Errorf("config path is required")
	}

	data, err := os.ReadFile(path)
	if err != nil {
		return nil, fmt.Errorf("read config %q: %w", path, err)
	}

	var c Config
	if _, err := toml.Decode(string(data), &c); err != nil {
		return nil, fmt.Errorf("parse config %q: %w", path, err)
	}

	if err := c.validate(); err != nil {
		return nil, fmt.Errorf("invalid config: %w", err)
	}

	c.applyDefaults()
	return &c, nil
}

func (c *Config) validate() error {
	if c.Chain.RPCURL == "" {
		return fmt.Errorf("chain.rpc_url is required")
	}
	if c.Chain.ChainID == 0 {
		return fmt.Errorf("chain.chain_id is required")
	}
	if c.Storage.DataDir == "" {
		return fmt.Errorf("storage.data_dir is required")
	}
	if c.Identity.KeystorePath == "" && c.Identity.PrivateKeyHex == "" {
		return fmt.Errorf("identity.keystore_path or identity.private_key_hex is required")
	}
	if c.Identity.KeystorePath != "" && c.Identity.PrivateKeyHex != "" {
		return fmt.Errorf("identity: cannot set both keystore_path and private_key_hex")
	}
	return nil
}

func (c *Config) applyDefaults() {
	if c.Chain.PollIntervalSeconds == 0 {
		c.Chain.PollIntervalSeconds = 12 // Base block time is ~2s, 12s is 6 blocks
	}
	if c.HTTP.ListenAddr == "" {
		c.HTTP.ListenAddr = ":8443"
	}
	if c.Metrics.ListenAddr == "" {
		c.Metrics.ListenAddr = "127.0.0.1:9090"
	}
	if c.Storage.IndexPath == "" && c.Storage.DataDir != "" {
		c.Storage.IndexPath = c.Storage.DataDir + "/index.sqlite"
	}
}
