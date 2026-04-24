// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright (c) 2026 Prova Network contributors.

// provad is the Prova prover daemon.
package main

import (
	"context"
	"flag"
	"fmt"
	"log/slog"
	"math/big"
	"os"
	"os/signal"
	"runtime"
	"strings"
	"syscall"
	"time"

	"github.com/ethereum/go-ethereum/common"

	"github.com/prova-network/prova/prover/pkg/config"
	"github.com/prova-network/prova/prover/pkg/contracts/proverregistry"
	"github.com/prova-network/prova/prover/pkg/contracts/proverstaking"
	"github.com/prova-network/prova/prover/pkg/contracts/storagemarketplace"
	"github.com/prova-network/prova/prover/pkg/daemon"
	"github.com/prova-network/prova/prover/pkg/deal"
	"github.com/prova-network/prova/prover/pkg/ethclient"
	"github.com/prova-network/prova/prover/pkg/store"
	"github.com/prova-network/prova/prover/pkg/wallet"
)

// These are set via -ldflags at build time:
//
//	go build -ldflags "-X main.version=0.1.0 -X main.commit=$(git rev-parse --short HEAD)"
var (
	version = "dev"
	commit  = "unknown"
)

func main() {
	if err := run(); err != nil {
		fmt.Fprintf(os.Stderr, "provad: %v\n", err)
		os.Exit(1)
	}
}

func run() error {
	var configPath string
	flag.StringVar(&configPath, "config", "", "path to prover TOML config (required for start/register/status)")
	flag.Usage = usage
	flag.Parse()

	args := flag.Args()
	if len(args) == 0 {
		usage()
		return fmt.Errorf("missing subcommand")
	}

	ctx, cancel := signal.NotifyContext(context.Background(), os.Interrupt, syscall.SIGTERM)
	defer cancel()

	cmd := args[0]
	switch cmd {
	case "version":
		return cmdVersion()
	case "start":
		return cmdStart(ctx, configPath)
	case "register":
		return cmdRegister(ctx, configPath)
	case "status":
		return cmdStatus(ctx, configPath)
	case "help", "-h", "--help":
		usage()
		return nil
	default:
		usage()
		return fmt.Errorf("unknown subcommand: %q", cmd)
	}
}

func usage() {
	fmt.Fprintf(os.Stderr, `provad - Prova Network prover daemon

Usage:
  provad [flags] <subcommand>

Subcommands:
  start     Run the prover daemon (main mode, not yet implemented)
  register  Register this prover in ProverRegistry on-chain
  status    Print current prover status and exit
  version   Print version info and exit
  help      Print this help

Flags:
  --config  Path to prover TOML config file

Environment:
  PROVA_KEYSTORE_PASSPHRASE  Decrypt keystore file (preferred over config value)
  PROVA_PRIVATE_KEY          Raw hex private key; overrides config (dev only)

Examples:
  provad version
  provad --config /etc/prova/prover.toml status
`)
}

func cmdVersion() error {
	fmt.Printf("provad %s (commit %s, %s/%s, %s)\n",
		version, commit, runtime.GOOS, runtime.GOARCH, runtime.Version())
	return nil
}

// loadEnvironment brings up (config, wallet, ethclient) — the common setup
// for every subcommand that actually touches the chain.
func loadEnvironment(ctx context.Context, configPath string) (*config.Config, *wallet.Wallet, *ethclient.Client, error) {
	cfg, err := loadConfig(configPath)
	if err != nil {
		return nil, nil, nil, err
	}

	w, err := loadWallet(cfg)
	if err != nil {
		return nil, nil, nil, fmt.Errorf("wallet: %w", err)
	}

	cl, err := ethclient.Dial(ctx, ethclient.Options{
		RPCURL:          cfg.Chain.RPCURL,
		ExpectedChainID: cfg.Chain.ChainID,
	})
	if err != nil {
		return nil, nil, nil, fmt.Errorf("ethclient: %w", err)
	}

	return cfg, w, cl, nil
}

func loadWallet(cfg *config.Config) (*wallet.Wallet, error) {
	// Env override has precedence (useful for testing and keyless CI)
	if w, ok, err := wallet.LoadFromEnv(); ok {
		if err != nil {
			return nil, err
		}
		return w, nil
	}
	if cfg.Identity.KeystorePath != "" {
		return wallet.LoadKeystore(cfg.Identity.KeystorePath, cfg.Identity.Passphrase)
	}
	if cfg.Identity.PrivateKeyHex != "" {
		return wallet.LoadHex(cfg.Identity.PrivateKeyHex)
	}
	return nil, fmt.Errorf("no identity source configured")
}

func cmdStart(ctx context.Context, configPath string) error {
	cfg, w, cl, err := loadEnvironment(ctx, configPath)
	if err != nil {
		return err
	}
	defer cl.Close()

	logger := slog.New(slog.NewTextHandler(os.Stdout, &slog.HandlerOptions{Level: slog.LevelInfo}))

	// Marketplace binding (needed for the event poller)
	marketAddr, err := parseContractAddress(cfg.Chain.Contracts.StorageMarketplace, "storage_marketplace")
	if err != nil {
		return err
	}
	market, err := storagemarketplace.NewStorageMarketplace(marketAddr, cl.Raw())
	if err != nil {
		return fmt.Errorf("bind StorageMarketplace: %w", err)
	}

	// Local disk-backed piece store
	pieces, err := store.NewDiskStore(cfg.Storage.DataDir)
	if err != nil {
		return fmt.Errorf("piece store: %w", err)
	}
	defer pieces.Close()

	// Deal engine with in-memory store for now (SQLite in a later phase).
	// Accepter is a stub until Phase D.2 wires the real tx path through
	// ProofVerifier.createDataSet. A nil-safe placeholder keeps the daemon
	// runnable without accidentally submitting transactions.
	engine, err := deal.NewEngine(deal.EngineOptions{
		OurAddress: w.Address,
		Deals:      deal.NewMemStore(),
		Pieces:     pieces,
		Accepter:   stubAccepter{logger: logger},
		Logger:     logger,
	})
	if err != nil {
		return fmt.Errorf("engine: %w", err)
	}

	poller, err := deal.NewEventPoller(deal.EventPollerOptions{
		Engine:      engine,
		Marketplace: market,
		OurAddress:  w.Address,
		Logger:      logger,
	})
	if err != nil {
		return fmt.Errorf("poller: %w", err)
	}

	d, err := daemon.New(daemon.Options{
		Config: daemon.Config{
			ProverAddress: w.Address,
			PollInterval:  time.Duration(cfg.Chain.PollIntervalSeconds) * time.Second,
		},
		Engine: engine,
		Poller: poller,
		Eth:    cl,
		Logger: logger,
	})
	if err != nil {
		return fmt.Errorf("daemon: %w", err)
	}

	logger.Info("provad start",
		"version", version,
		"commit", commit,
		"chain", ethclient.ChainName(cfg.Chain.ChainID),
		"dataDir", cfg.Storage.DataDir,
	)

	return d.Run(ctx)
}

// stubAccepter logs but does not actually submit the acceptance tx.
// Replaced with a real implementation in Phase D.2 once the Merkle
// builder lands. Keeping the daemon runnable today without risk of
// submitting broken txs is worth the explicit stub.
type stubAccepter struct {
	logger *slog.Logger
}

func (s stubAccepter) Accept(_ context.Context, id deal.DealID) (uint64, error) {
	s.logger.Warn("accept stub: real ProofVerifier.createDataSet path is Phase D.2",
		"dealID", uint64(id),
	)
	return 0, fmt.Errorf("acceptance tx not yet wired; see Phase D.2")
}

func cmdRegister(ctx context.Context, configPath string) error {
	cfg, w, cl, err := loadEnvironment(ctx, configPath)
	if err != nil {
		return err
	}
	defer cl.Close()

	regAddr, err := parseContractAddress(cfg.Chain.Contracts.ProverRegistry, "prover_registry")
	if err != nil {
		return err
	}

	reg, err := proverregistry.NewProverRegistry(regAddr, cl.Raw())
	if err != nil {
		return fmt.Errorf("bind ProverRegistry: %w", err)
	}

	// Check if already registered
	existing, err := reg.GetProver(nil, w.Address)
	if err == nil && existing.Active {
		fmt.Printf("already registered as active prover: %s\n", w.Address.Hex())
		fmt.Printf("  endpoint: %s\n", existing.Endpoint)
		fmt.Printf("  features: 0x%016x\n", existing.Features)
		return nil
	}

	// Would call reg.Register(...) here. Holding off so that a stray
	// invocation doesn't accidentally register the prover before we're ready.
	return fmt.Errorf("register: not yet wired (prover registration must be explicit; coming in a later change)")
}

func cmdStatus(ctx context.Context, configPath string) error {
	cfg, w, cl, err := loadEnvironment(ctx, configPath)
	if err != nil {
		return err
	}
	defer cl.Close()

	ctx, cancel := context.WithTimeout(ctx, 10*time.Second)
	defer cancel()

	fmt.Printf("provad %s\n", version)
	fmt.Printf("chain       %s (id %d)\n", ethclient.ChainName(cfg.Chain.ChainID), cfg.Chain.ChainID)
	fmt.Printf("rpc         %s\n", cfg.Chain.RPCURL)

	blockNum, err := cl.BlockNumber(ctx)
	if err != nil {
		return fmt.Errorf("block number: %w", err)
	}
	fmt.Printf("head        block %d\n", blockNum)

	fmt.Printf("address     %s\n", w.Address.Hex())
	balance, err := cl.BalanceOf(ctx, w.Address)
	if err != nil {
		return fmt.Errorf("balance: %w", err)
	}
	fmt.Printf("balance     %s wei (%s ETH)\n", balance.String(), weiToETH(balance))

	// Try to read on-chain registry state if a registry address is configured
	if cfg.Chain.Contracts.ProverRegistry != "" &&
		cfg.Chain.Contracts.ProverRegistry != "0x0000000000000000000000000000000000000000" {
		if err := printRegistryStatus(ctx, cfg, w, cl); err != nil {
			fmt.Printf("registry    error: %v\n", err)
		}
	} else {
		fmt.Printf("registry    (not configured)\n")
	}

	if cfg.Chain.Contracts.ProverStaking != "" &&
		cfg.Chain.Contracts.ProverStaking != "0x0000000000000000000000000000000000000000" {
		if err := printStakingStatus(ctx, cfg, w, cl); err != nil {
			fmt.Printf("staking     error: %v\n", err)
		}
	} else {
		fmt.Printf("staking     (not configured)\n")
	}

	return nil
}

func printRegistryStatus(_ context.Context, cfg *config.Config, w *wallet.Wallet, cl *ethclient.Client) error {
	addr, err := parseContractAddress(cfg.Chain.Contracts.ProverRegistry, "prover_registry")
	if err != nil {
		return err
	}
	reg, err := proverregistry.NewProverRegistry(addr, cl.Raw())
	if err != nil {
		return err
	}
	p, err := reg.GetProver(nil, w.Address)
	if err != nil {
		return err
	}
	if !p.Active {
		fmt.Printf("registry    not registered\n")
		return nil
	}
	fmt.Printf("registry    registered\n")
	fmt.Printf("  endpoint  %s\n", p.Endpoint)
	fmt.Printf("  features  0x%016x\n", p.Features)
	return nil
}

func printStakingStatus(_ context.Context, cfg *config.Config, w *wallet.Wallet, cl *ethclient.Client) error {
	addr, err := parseContractAddress(cfg.Chain.Contracts.ProverStaking, "prover_staking")
	if err != nil {
		return err
	}
	stk, err := proverstaking.NewProverStaking(addr, cl.Raw())
	if err != nil {
		return err
	}
	s, err := stk.GetStake(nil, w.Address)
	if err != nil {
		return err
	}
	fmt.Printf("staking     %s staked, %s unbonding, %d bytes committed\n",
		s.Staked.String(), s.Unbonding.String(), s.CommittedBytes.Uint64())
	return nil
}

func loadConfig(path string) (*config.Config, error) {
	if path == "" {
		return nil, fmt.Errorf("--config is required")
	}
	return config.Load(path)
}

func parseContractAddress(s, fieldName string) (common.Address, error) {
	if s == "" {
		return common.Address{}, fmt.Errorf("%s: not configured", fieldName)
	}
	if !common.IsHexAddress(s) {
		return common.Address{}, fmt.Errorf("%s: invalid hex address %q", fieldName, s)
	}
	return common.HexToAddress(s), nil
}

// weiToETH formats a wei value as a decimal ETH string (18 decimals, up to 6
// fractional digits). For display only; do not use for calculations.
func weiToETH(wei *big.Int) string {
	if wei == nil {
		return "0"
	}
	// Divide by 1e18, keep 6 decimal places.
	const decimals = 18
	const show = 6
	neg := wei.Sign() < 0
	abs := new(big.Int).Abs(wei)
	div := new(big.Int).Exp(big.NewInt(10), big.NewInt(decimals), nil)
	whole, rem := new(big.Int).QuoRem(abs, div, new(big.Int))

	// Scale fractional part to `show` digits
	scale := new(big.Int).Exp(big.NewInt(10), big.NewInt(decimals-show), nil)
	frac := new(big.Int).Quo(rem, scale)

	sign := ""
	if neg {
		sign = "-"
	}
	s := fmt.Sprintf("%s%s.%0*d", sign, whole.String(), show, frac.Uint64())
	return strings.TrimRight(strings.TrimRight(s, "0"), ".")
}
