// SPDX-License-Identifier: MIT
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
	"github.com/prova-network/prova/prover/pkg/contracts/proofverifier"
	"github.com/prova-network/prova/prover/pkg/contracts/proverregistry"
	"github.com/prova-network/prova/prover/pkg/contracts/proverstaking"
	"github.com/prova-network/prova/prover/pkg/contracts/storagemarketplace"
	"github.com/prova-network/prova/prover/pkg/daemon"
	"github.com/prova-network/prova/prover/pkg/deal"
	"github.com/prova-network/prova/prover/pkg/ethclient"
	"github.com/prova-network/prova/prover/pkg/httpserver"
	"github.com/prova-network/prova/prover/pkg/metrics"
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
  start     Run the prover daemon (main mode)
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

	// Metrics collector (always built; enabling the HTTP metrics endpoint
	// is opt-in via config).
	mcol := metrics.New()

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
	// Build accepter: real OnChainAccepter when ProofVerifier is
	// configured, stub when the address is zero.
	var accepter deal.Accepter = stubAccepter{logger: logger}
	var proofVerifier *proofverifier.ProofVerifier
	if cfg.Chain.Contracts.ProofVerifier != "" &&
		cfg.Chain.Contracts.ProofVerifier != "0x0000000000000000000000000000000000000000" {
		pvAddr, err := parseContractAddress(cfg.Chain.Contracts.ProofVerifier, "proof_verifier")
		if err != nil {
			return err
		}
		proofVerifier, err = proofverifier.NewProofVerifier(pvAddr, cl.Raw())
		if err != nil {
			return fmt.Errorf("bind ProofVerifier: %w", err)
		}
		transactor, err := cl.NewTransactor(w.PrivateKey)
		if err != nil {
			return fmt.Errorf("transactor: %w", err)
		}
		onchain, err := deal.NewOnChainAccepter(deal.OnChainAccepterOptions{
			Verifier:           proofVerifier,
			VerifierAddress:    pvAddr,
			MarketplaceAddress: marketAddr,
			Transactor:         transactor,
			Waiter:             &waiterAdapter{cl: cl},
		})
		if err != nil {
			return fmt.Errorf("build accepter: %w", err)
		}
		accepter = onchain
		logger.Info("on-chain accepter wired", "proofVerifier", pvAddr.Hex())
	}

	fetcher := deal.NewFetcher(deal.FetcherOptions{
		AllowInsecure: cfg.SourceURL.AllowInsecure,
	})

	engine, err := deal.NewEngine(deal.EngineOptions{
		OurAddress: w.Address,
		Deals:      deal.NewMemStore(),
		Pieces:     pieces,
		Fetcher:    fetcher,
		Accepter:   accepter,
		Metrics:    metrics.NewDealSink(mcol),
		Logger:     logger,
	})
	if err != nil {
		return fmt.Errorf("engine: %w", err)
	}

	poller, err := deal.NewEventPoller(deal.EventPollerOptions{
		Engine:            engine,
		Marketplace:       market,
		OurAddress:        w.Address,
		BlockLookback:     cfg.Chain.BlockLookback,
		SourceURLResolver: deal.NewSourceURLResolver(cfg.SourceURL.Template),
		Logger:            logger,
	})
	if err != nil {
		return fmt.Errorf("poller: %w", err)
	}

	// Optional HTTP server for piece retrieval.
	var httpSrv *httpserver.Server
	if cfg.HTTP.Enabled {
		httpSrv, err = httpserver.New(httpserver.Options{
			Pieces:     pieces,
			ListenAddr: cfg.HTTP.ListenAddr,
			PublicURL:  cfg.HTTP.PublicURL,
			CertPath:   cfg.HTTP.CertPath,
			KeyPath:    cfg.HTTP.KeyPath,
			Metrics:    metrics.NewHTTPSink(mcol),
			Logger:     logger,
		})
		if err != nil {
			return fmt.Errorf("http server: %w", err)
		}
	}

	// Optional metrics HTTP server.
	var mSrv *metrics.Server
	if cfg.Metrics.Enabled {
		mSrv, err = metrics.NewServer(metrics.Options{
			Collector:  mcol,
			ListenAddr: cfg.Metrics.ListenAddr,
			Logger:     logger,
		})
		if err != nil {
			return fmt.Errorf("metrics server: %w", err)
		}
	}

	d, err := daemon.New(daemon.Options{
		Config: daemon.Config{
			ProverAddress: w.Address,
			PollInterval:  time.Duration(cfg.Chain.PollIntervalSeconds) * time.Second,
		},
		Engine:     engine,
		Poller:     poller,
		Eth:        cl,
		HTTP:       httpSrv,
		Metrics:    mcol,
		MetricsSrv: mSrv,
		Logger:     logger,
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

// waiterAdapter bridges *ethclient.Client to the deal.ReceiptWaiter
// interface without forcing pkg/deal to import pkg/ethclient.
type waiterAdapter struct {
	cl *ethclient.Client
}

func (w *waiterAdapter) WaitReceiptInfo(ctx context.Context, txHash common.Hash) (deal.TxResult, error) {
	res, err := w.cl.WaitReceiptInfo(ctx, txHash)
	if err != nil {
		return deal.TxResult{}, err
	}
	logs := make([]deal.TxLog, len(res.Logs))
	for i, lg := range res.Logs {
		logs[i] = deal.TxLog{Topics: lg.Topics, Data: lg.Data}
	}
	return deal.TxResult{
		OK:          res.OK,
		BlockNumber: res.BlockNumber,
		Logs:        logs,
	}, nil
}

// stubAccepter is kept as a fallback when ProofVerifier is not yet
// configured in TOML (placeholder 0x0 address). Emits a clear warning
// so operators know they're not actually accepting deals on-chain.
type stubAccepter struct {
	logger *slog.Logger
}

func (s stubAccepter) Accept(_ context.Context, id deal.DealID) (uint64, error) {
	s.logger.Warn("accept stub: proof_verifier address is not configured (zero)",
		"dealID", uint64(id),
	)
	return 0, fmt.Errorf("acceptance tx unavailable: proof_verifier address is zero in config")
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
		fmt.Printf("  endpoint  %s\n", existing.Endpoint)
		fmt.Printf("  features  0x%016x\n", existing.Features)
		return nil
	}

	// Registration needs a publicly-reachable endpoint URL. We pull this
	// from HTTP.PublicURL since that's where clients will fetch pieces.
	if !cfg.HTTP.Enabled || cfg.HTTP.PublicURL == "" {
		return fmt.Errorf("registration requires [http].enabled=true and [http].public_url set")
	}

	transactor, err := cl.NewTransactor(w.PrivateKey)
	if err != nil {
		return fmt.Errorf("transactor: %w", err)
	}
	transactor.Context = ctx

	// Feature bitmap: PDP is required; include HTTPS if the retrieval
	// endpoint is enabled and its public URL uses TLS.
	var features uint64 = 1 // FEATURE_PDP
	if strings.HasPrefix(cfg.HTTP.PublicURL, "https://") {
		features |= 2 // FEATURE_HTTPS_SERVING
	}

	// Prices default to 0 for now; operators can update later via
	// ProverRegistry.setPrice when pricing is understood.
	pricePerGibDay := big.NewInt(0)
	pricePerByteServed := big.NewInt(0)

	fmt.Printf("registering prover\n")
	fmt.Printf("  address   %s\n", w.Address.Hex())
	fmt.Printf("  endpoint  %s\n", cfg.HTTP.PublicURL)
	fmt.Printf("  features  0x%016x\n", features)

	tx, err := reg.Register(transactor, cfg.HTTP.PublicURL, features, pricePerGibDay, pricePerByteServed, "")
	if err != nil {
		return fmt.Errorf("Register tx: %w", err)
	}
	fmt.Printf("  tx        %s\n", tx.Hash().Hex())

	receipt, err := cl.WaitReceiptInfo(ctx, tx.Hash())
	if err != nil {
		return fmt.Errorf("wait receipt: %w", err)
	}
	if !receipt.OK {
		return fmt.Errorf("Register reverted (tx %s)", tx.Hash().Hex())
	}
	fmt.Printf("  block     %s\n", receipt.BlockNumber.String())
	fmt.Printf("registered ok\n")
	return nil
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

	if err := printRecentDeals(ctx, cfg, w, cl); err != nil {
		fmt.Printf("deals       error: %v\n", err)
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

// printRecentDeals queries the StorageMarketplace for deals targeting us
// and prints a brief summary. Uses the on-chain nextDealId counter to
// bound the iteration.
func printRecentDeals(_ context.Context, cfg *config.Config, w *wallet.Wallet, cl *ethclient.Client) error {
	if cfg.Chain.Contracts.StorageMarketplace == "" ||
		cfg.Chain.Contracts.StorageMarketplace == "0x0000000000000000000000000000000000000000" {
		fmt.Printf("deals       (marketplace not configured)\n")
		return nil
	}
	addr, err := parseContractAddress(cfg.Chain.Contracts.StorageMarketplace, "storage_marketplace")
	if err != nil {
		return err
	}
	market, err := storagemarketplace.NewStorageMarketplace(addr, cl.Raw())
	if err != nil {
		return err
	}
	nextID, err := market.NextDealId(nil)
	if err != nil {
		return err
	}
	// Walk backwards up to 10 deals
	const maxScan = 10
	total := nextID.Uint64() - 1 // nextDealId is the id to be assigned next
	if total == 0 || nextID.Sign() == 0 {
		fmt.Printf("deals       no deals yet\n")
		return nil
	}

	var ours []struct {
		id     uint64
		status uint8
		proofs uint64
	}
	scanned := 0
	for id := total; id >= 1 && scanned < maxScan; id-- {
		d, err := market.GetDeal(nil, big.NewInt(int64(id)))
		if err != nil {
			continue
		}
		if d.Prover == w.Address {
			ours = append(ours, struct {
				id     uint64
				status uint8
				proofs uint64
			}{id, d.Status, d.ProofCount.Uint64()})
		}
		scanned++
	}
	if len(ours) == 0 {
		fmt.Printf("deals       no deals target this prover in the last %d\n", maxScan)
		return nil
	}
	fmt.Printf("deals       %d visible in last %d total deals\n", len(ours), maxScan)
	for _, d := range ours {
		fmt.Printf("  deal #%-4d status=%s proofs=%d\n",
			d.id, dealStatusName(d.status), d.proofs)
	}
	return nil
}

// dealStatusName maps the on-chain DealStatus enum to a string. Matches
// the enum ordering in contracts/src/StorageMarketplace.sol.
func dealStatusName(s uint8) string {
	switch s {
	case 0:
		return "None"
	case 1:
		return "Proposed"
	case 2:
		return "Active"
	case 3:
		return "Completed"
	case 4:
		return "Cancelled"
	case 5:
		return "Slashed"
	default:
		return fmt.Sprintf("Unknown(%d)", s)
	}
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
