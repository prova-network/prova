// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Prova Network contributors.

// Package ethclient wraps go-ethereum's RPC client with Prova-specific
// conventions: known chain IDs, tx receipt polling with sensible defaults,
// and a shim for signing transactions against our Wallet.
package ethclient

import (
	"context"
	"crypto/ecdsa"
	"fmt"
	"math/big"
	"time"

	ethereum "github.com/ethereum/go-ethereum"
	"github.com/ethereum/go-ethereum/accounts/abi/bind"
	"github.com/ethereum/go-ethereum/common"
	"github.com/ethereum/go-ethereum/core/types"
	"github.com/ethereum/go-ethereum/ethclient"
)

// Known chain IDs.
const (
	ChainIDBaseMainnet uint64 = 8453
	ChainIDBaseSepolia uint64 = 84532
	ChainIDEthMainnet  uint64 = 1
	ChainIDEthSepolia  uint64 = 11155111
	ChainIDLocalAnvil  uint64 = 31337
)

// ChainName returns a human-readable name for known chain IDs.
func ChainName(id uint64) string {
	switch id {
	case ChainIDBaseMainnet:
		return "base-mainnet"
	case ChainIDBaseSepolia:
		return "base-sepolia"
	case ChainIDEthMainnet:
		return "eth-mainnet"
	case ChainIDEthSepolia:
		return "eth-sepolia"
	case ChainIDLocalAnvil:
		return "anvil-local"
	default:
		return fmt.Sprintf("unknown-%d", id)
	}
}

// Client is a Prova-flavored wrapper over go-ethereum's ethclient.
// It enforces that the RPC endpoint reports the expected chain ID at
// connection time, caches the chain ID for tx signing, and provides a
// small set of polling / receipt helpers.
type Client struct {
	raw         *ethclient.Client
	chainID     *big.Int
	rpcURL      string
	receiptWait time.Duration
}

// Options controls Client construction.
type Options struct {
	// RPCURL is the HTTP or WebSocket endpoint.
	RPCURL string
	// ExpectedChainID is validated against eth_chainId on connect. Pass 0 to skip.
	ExpectedChainID uint64
	// ReceiptTimeout is the default timeout for WaitReceipt. Default: 60s.
	ReceiptTimeout time.Duration
}

// Dial opens an RPC connection and verifies the chain ID.
func Dial(ctx context.Context, opts Options) (*Client, error) {
	if opts.RPCURL == "" {
		return nil, fmt.Errorf("RPCURL is required")
	}
	if opts.ReceiptTimeout == 0 {
		opts.ReceiptTimeout = 60 * time.Second
	}

	raw, err := ethclient.DialContext(ctx, opts.RPCURL)
	if err != nil {
		return nil, fmt.Errorf("dial %q: %w", opts.RPCURL, err)
	}

	id, err := raw.ChainID(ctx)
	if err != nil {
		raw.Close()
		return nil, fmt.Errorf("fetch chain id from %q: %w", opts.RPCURL, err)
	}

	if opts.ExpectedChainID != 0 && id.Uint64() != opts.ExpectedChainID {
		raw.Close()
		return nil, fmt.Errorf(
			"chain id mismatch: expected %d (%s), got %d (%s)",
			opts.ExpectedChainID, ChainName(opts.ExpectedChainID),
			id.Uint64(), ChainName(id.Uint64()),
		)
	}

	return &Client{
		raw:         raw,
		chainID:     id,
		rpcURL:      opts.RPCURL,
		receiptWait: opts.ReceiptTimeout,
	}, nil
}

// Close releases the underlying RPC connection.
func (c *Client) Close() {
	if c.raw != nil {
		c.raw.Close()
	}
}

// Raw exposes the underlying go-ethereum client for advanced usage.
// Prefer the Client wrappers where they exist.
func (c *Client) Raw() *ethclient.Client { return c.raw }

// ChainID returns the connected chain ID as a *big.Int.
func (c *Client) ChainID() *big.Int { return new(big.Int).Set(c.chainID) }

// NewTransactor builds a bind.TransactOpts for signing transactions as the
// given private key, on the connected chain. The returned opts has a default
// gas limit estimator and no explicit gas price (uses the node's suggestion).
func (c *Client) NewTransactor(priv *ecdsa.PrivateKey) (*bind.TransactOpts, error) {
	opts, err := bind.NewKeyedTransactorWithChainID(priv, c.chainID)
	if err != nil {
		return nil, fmt.Errorf("new transactor: %w", err)
	}
	return opts, nil
}

// WaitReceipt polls for a transaction receipt until one is available or
// the context deadline is hit.
func (c *Client) WaitReceipt(ctx context.Context, txHash common.Hash) (*types.Receipt, error) {
	ctx, cancel := context.WithTimeout(ctx, c.receiptWait)
	defer cancel()

	ticker := time.NewTicker(500 * time.Millisecond)
	defer ticker.Stop()

	for {
		receipt, err := c.raw.TransactionReceipt(ctx, txHash)
		if err == nil {
			return receipt, nil
		}
		if err != ethereum.NotFound {
			return nil, fmt.Errorf("fetch receipt: %w", err)
		}
		select {
		case <-ctx.Done():
			return nil, fmt.Errorf("waiting for receipt %s: %w", txHash.Hex(), ctx.Err())
		case <-ticker.C:
		}
	}
}

// BalanceOf returns the wei balance of addr.
func (c *Client) BalanceOf(ctx context.Context, addr common.Address) (*big.Int, error) {
	return c.raw.BalanceAt(ctx, addr, nil)
}

// BlockNumber returns the current head block number.
func (c *Client) BlockNumber(ctx context.Context) (uint64, error) {
	return c.raw.BlockNumber(ctx)
}

// TxResult is a tiny struct returned by WaitReceiptInfo. It carries the
// minimal receipt metadata the deal package needs, without forcing any
// consumer to import go-ethereum's types.Receipt.
type TxResult struct {
	OK          bool         // true iff receipt.Status == 1
	BlockNumber *big.Int
	Logs        []TxLog
}

// TxLog mirrors the subset of types.Log needed for event parsing.
type TxLog struct {
	Topics []common.Hash
	Data   []byte
}

// WaitReceiptInfo waits for a tx receipt and returns a plain struct
// the deal package can consume without importing go-ethereum types.
func (c *Client) WaitReceiptInfo(ctx context.Context, txHash common.Hash) (*TxResult, error) {
	r, err := c.WaitReceipt(ctx, txHash)
	if err != nil {
		return nil, err
	}
	out := &TxResult{
		OK:          r.Status == 1,
		BlockNumber: r.BlockNumber,
		Logs:        make([]TxLog, len(r.Logs)),
	}
	for i, lg := range r.Logs {
		out.Logs[i] = TxLog{Topics: lg.Topics, Data: lg.Data}
	}
	return out, nil
}


