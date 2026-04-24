// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Prova Network contributors.

package deal

import (
	"context"
	"fmt"
	"math/big"

	"github.com/ethereum/go-ethereum/accounts/abi"
	"github.com/ethereum/go-ethereum/accounts/abi/bind"
	"github.com/ethereum/go-ethereum/common"

	pv "github.com/prova-network/prova/prover/pkg/contracts/proofverifier"
)

// OnChainAccepter submits the ProofVerifier.createDataSet transaction that
// activates a proposed deal on-chain.
//
// Flow:
//   1. Our StorageMarketplace is registered as the PDPListener when we call
//      createDataSet. Passing `address(storageMarketplace)` as listenerAddr
//      tells ProofVerifier "route all callbacks for this data set to the
//      marketplace."
//   2. extraData = abi.encode(uint256(dealId)). The marketplace reads this
//      in its dataSetCreated() hook to resolve the deal and move it from
//      Proposed → Active.
//   3. On success we return the assigned dataSetID. The marketplace's
//      dataSetCreated hook has already updated on-chain state by the time
//      our tx receipt comes back; we confirm by reading the emitted
//      DataSetCreated event.
type OnChainAccepter struct {
	verifier       *pv.ProofVerifier
	verifierAddr   common.Address
	marketplaceAddr common.Address
	transactor     *bind.TransactOpts
	waiter         ReceiptWaiter
}

// ReceiptWaiter blocks until a tx is mined and returns a result struct.
// Defined here so deal/ doesn't depend on ethclient/ directly; the real
// implementation is ethclient.Client.WaitReceiptInfo.
type ReceiptWaiter interface {
	WaitReceiptInfo(ctx context.Context, txHash common.Hash) (TxResult, error)
}

// TxResult is the minimum receipt metadata the acceptance path needs.
// Matches ethclient.TxResult field-for-field.
type TxResult struct {
	OK          bool
	BlockNumber *big.Int
	Logs        []TxLog
}

// TxLog mirrors ethclient.TxLog.
type TxLog struct {
	Topics []common.Hash
	Data   []byte
}

// OnChainAccepterOptions configures a real Accepter.
type OnChainAccepterOptions struct {
	// Verifier is the ProofVerifier contract binding.
	Verifier *pv.ProofVerifier

	// VerifierAddress is the address of the deployed ProofVerifier proxy.
	// Used only for logging; the binding holds it internally.
	VerifierAddress common.Address

	// MarketplaceAddress is passed as the listenerAddr in createDataSet,
	// so the verifier routes PDPListener callbacks to the marketplace.
	MarketplaceAddress common.Address

	// Transactor is a bind.TransactOpts produced by ethclient.NewTransactor.
	Transactor *bind.TransactOpts

	// Waiter blocks on tx receipts. Usually an ethclient.Client wrapper.
	Waiter ReceiptWaiter
}

// NewOnChainAccepter constructs a real Accepter.
func NewOnChainAccepter(opts OnChainAccepterOptions) (*OnChainAccepter, error) {
	if opts.Verifier == nil {
		return nil, fmt.Errorf("verifier binding required")
	}
	if opts.MarketplaceAddress == (common.Address{}) {
		return nil, fmt.Errorf("marketplace address required")
	}
	if opts.Transactor == nil {
		return nil, fmt.Errorf("transactor required")
	}
	if opts.Waiter == nil {
		return nil, fmt.Errorf("receipt waiter required")
	}
	return &OnChainAccepter{
		verifier:        opts.Verifier,
		verifierAddr:    opts.VerifierAddress,
		marketplaceAddr: opts.MarketplaceAddress,
		transactor:      opts.Transactor,
		waiter:          opts.Waiter,
	}, nil
}

// Accept implements deal.Accepter. Submits createDataSet and waits for
// confirmation. Returns the assigned dataSetID extracted from the
// DataSetCreated event (topic[1]).
func (a *OnChainAccepter) Accept(ctx context.Context, dealID DealID) (uint64, error) {
	// Pack dealID as abi-encoded uint256
	uint256Ty, _ := abi.NewType("uint256", "", nil)
	args := abi.Arguments{{Type: uint256Ty}}
	extraData, err := args.Pack(new(big.Int).SetUint64(uint64(dealID)))
	if err != nil {
		return 0, fmt.Errorf("abi encode dealID: %w", err)
	}

	// Fresh opts per call so Context is current
	opts := *a.transactor
	opts.Context = ctx

	tx, err := a.verifier.CreateDataSet(&opts, a.marketplaceAddr, extraData)
	if err != nil {
		return 0, fmt.Errorf("createDataSet tx: %w", err)
	}

	receipt, err := a.waiter.WaitReceiptInfo(ctx, tx.Hash())
	if err != nil {
		return 0, fmt.Errorf("wait receipt: %w", err)
	}
	if !receipt.OK {
		return 0, fmt.Errorf("createDataSet reverted (tx %s)", tx.Hash().Hex())
	}

	// Find the DataSetCreated event: topics[0] = event sig,
	// topics[1] = indexed setId, topics[2] = indexed storageProvider.
	// ABI declares: event DataSetCreated(uint256 indexed setId, address indexed storageProvider);
	//
	// We don't need the event signature hash to match — the only log in
	// this tx with 3 topics is DataSetCreated.
	for _, lg := range receipt.Logs {
		if len(lg.Topics) == 3 {
			return new(big.Int).SetBytes(lg.Topics[1][:]).Uint64(), nil
		}
	}
	return 0, fmt.Errorf("createDataSet tx succeeded but DataSetCreated event not found in receipt logs")
}
