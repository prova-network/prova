// SPDX-License-Identifier: MIT
// Generated from contracts/out/MockProofVerifier.sol/MockProofVerifier.json via abigen.
// Do not edit by hand; run ./scripts/gen-bindings.sh instead.

// Code generated - DO NOT EDIT.
// This file is a generated binding and any manual changes will be lost.

package mockproofverifier

import (
	"errors"
	"math/big"
	"strings"

	ethereum "github.com/ethereum/go-ethereum"
	"github.com/ethereum/go-ethereum/accounts/abi"
	"github.com/ethereum/go-ethereum/accounts/abi/bind"
	"github.com/ethereum/go-ethereum/common"
	"github.com/ethereum/go-ethereum/core/types"
	"github.com/ethereum/go-ethereum/event"
)

// Reference imports to suppress errors if they are not otherwise used.
var (
	_ = errors.New
	_ = big.NewInt
	_ = strings.NewReader
	_ = ethereum.NotFound
	_ = bind.Bind
	_ = common.Big1
	_ = types.BloomLookup
	_ = event.NewSubscription
	_ = abi.ConvertType
)

// MockProofVerifierMetaData contains all meta data concerning the MockProofVerifier contract.
var MockProofVerifierMetaData = &bind.MetaData{
	ABI: "[{\"type\":\"function\",\"name\":\"createDataSet\",\"inputs\":[{\"name\":\"listenerAddr\",\"type\":\"address\",\"internalType\":\"address\"},{\"name\":\"extraData\",\"type\":\"bytes\",\"internalType\":\"bytes\"}],\"outputs\":[{\"name\":\"setId\",\"type\":\"uint256\",\"internalType\":\"uint256\"}],\"stateMutability\":\"payable\"},{\"type\":\"function\",\"name\":\"listener\",\"inputs\":[{\"name\":\"\",\"type\":\"uint256\",\"internalType\":\"uint256\"}],\"outputs\":[{\"name\":\"\",\"type\":\"address\",\"internalType\":\"address\"}],\"stateMutability\":\"view\"},{\"type\":\"function\",\"name\":\"nextDataSetId\",\"inputs\":[],\"outputs\":[{\"name\":\"\",\"type\":\"uint256\",\"internalType\":\"uint256\"}],\"stateMutability\":\"view\"},{\"type\":\"function\",\"name\":\"simulateProof\",\"inputs\":[{\"name\":\"setId\",\"type\":\"uint256\",\"internalType\":\"uint256\"}],\"outputs\":[],\"stateMutability\":\"nonpayable\"},{\"type\":\"function\",\"name\":\"storageProvider\",\"inputs\":[{\"name\":\"\",\"type\":\"uint256\",\"internalType\":\"uint256\"}],\"outputs\":[{\"name\":\"\",\"type\":\"address\",\"internalType\":\"address\"}],\"stateMutability\":\"view\"},{\"type\":\"event\",\"name\":\"DataSetCreated\",\"inputs\":[{\"name\":\"setId\",\"type\":\"uint256\",\"indexed\":true,\"internalType\":\"uint256\"},{\"name\":\"storageProvider\",\"type\":\"address\",\"indexed\":true,\"internalType\":\"address\"}],\"anonymous\":false},{\"type\":\"event\",\"name\":\"PossessionProven\",\"inputs\":[{\"name\":\"setId\",\"type\":\"uint256\",\"indexed\":true,\"internalType\":\"uint256\"}],\"anonymous\":false}]",
	Bin: "0x6080806040523460195760015f556103ab908161001e8239f35b5f80fdfe6080806040526004361015610012575f80fd5b5f905f3560e01c908163271221b3146103105750806392a6e08f146102f4578063b31662db146102c2578063bbae41cb1461014d5763cbd646d614610055575f80fd5b3461014a57602036600319011261014a57600435808252600160205260408220546001600160a01b03168015610117578083913b156101085781809160846040518094819363356de02b60e01b835288600484015260016024840152816044840152600160648401525af1801561010c576100f3575b50807f8fcfe2d16538be9d0fbb9a19b1a403567f6cb498484d8c2605777d1fa92cc6ac91a280f35b816100fd9161033e565b61010857815f6100cb565b5080fd5b6040513d84823e3d90fd5b60405162461bcd60e51b815260206004820152600b60248201526a3737903634b9ba32b732b960a91b6044820152606490fd5b80fd5b60403660031901126102aa576004356001600160a01b038116908190036102aa576024356001600160401b0381116102aa57366023820112156102aa5760048101356001600160401b0381116102aa5736602482840101116102aa575f54915f1983146102ae5760018381015f9081558481526020918252604080822080546001600160a01b0319908116891790915560029093528082208054339416841790555195919085907f11369440e1b7135015c16acb9bc14b55b0f4b23b02010c363d34aec2e5b962819080a380610229575b602084604051908152f35b803b156102aa57845f6084828683976024839863101c1eab60e01b85528b600486015233828601526060604486015282606486015201848401378181018301849052601f01601f191681010301925af191821561029f5760209261028f575b808061021e565b5f6102999161033e565b5f610288565b6040513d5f823e3d90fd5b5f80fd5b634e487b7160e01b5f52601160045260245ffd5b346102aa5760203660031901126102aa576004355f526002602052602060018060a01b0360405f205416604051908152f35b346102aa575f3660031901126102aa5760205f54604051908152f35b346102aa5760203660031901126102aa576020906004355f526001825260018060a01b0360405f2054168152f35b601f909101601f19168101906001600160401b0382119082101761036157604052565b634e487b7160e01b5f52604160045260245ffdfea2646970667358221220aaf5f01056ce8f868388f98ff0911acafa4dfe9d3ee0756ec2c0606f7b6d71cd64736f6c634300081e0033",
}

// MockProofVerifierABI is the input ABI used to generate the binding from.
// Deprecated: Use MockProofVerifierMetaData.ABI instead.
var MockProofVerifierABI = MockProofVerifierMetaData.ABI

// MockProofVerifierBin is the compiled bytecode used for deploying new contracts.
// Deprecated: Use MockProofVerifierMetaData.Bin instead.
var MockProofVerifierBin = MockProofVerifierMetaData.Bin

// DeployMockProofVerifier deploys a new Ethereum contract, binding an instance of MockProofVerifier to it.
func DeployMockProofVerifier(auth *bind.TransactOpts, backend bind.ContractBackend) (common.Address, *types.Transaction, *MockProofVerifier, error) {
	parsed, err := MockProofVerifierMetaData.GetAbi()
	if err != nil {
		return common.Address{}, nil, nil, err
	}
	if parsed == nil {
		return common.Address{}, nil, nil, errors.New("GetABI returned nil")
	}

	address, tx, contract, err := bind.DeployContract(auth, *parsed, common.FromHex(MockProofVerifierBin), backend)
	if err != nil {
		return common.Address{}, nil, nil, err
	}
	return address, tx, &MockProofVerifier{MockProofVerifierCaller: MockProofVerifierCaller{contract: contract}, MockProofVerifierTransactor: MockProofVerifierTransactor{contract: contract}, MockProofVerifierFilterer: MockProofVerifierFilterer{contract: contract}}, nil
}

// MockProofVerifier is an auto generated Go binding around an Ethereum contract.
type MockProofVerifier struct {
	MockProofVerifierCaller     // Read-only binding to the contract
	MockProofVerifierTransactor // Write-only binding to the contract
	MockProofVerifierFilterer   // Log filterer for contract events
}

// MockProofVerifierCaller is an auto generated read-only Go binding around an Ethereum contract.
type MockProofVerifierCaller struct {
	contract *bind.BoundContract // Generic contract wrapper for the low level calls
}

// MockProofVerifierTransactor is an auto generated write-only Go binding around an Ethereum contract.
type MockProofVerifierTransactor struct {
	contract *bind.BoundContract // Generic contract wrapper for the low level calls
}

// MockProofVerifierFilterer is an auto generated log filtering Go binding around an Ethereum contract events.
type MockProofVerifierFilterer struct {
	contract *bind.BoundContract // Generic contract wrapper for the low level calls
}

// MockProofVerifierSession is an auto generated Go binding around an Ethereum contract,
// with pre-set call and transact options.
type MockProofVerifierSession struct {
	Contract     *MockProofVerifier // Generic contract binding to set the session for
	CallOpts     bind.CallOpts      // Call options to use throughout this session
	TransactOpts bind.TransactOpts  // Transaction auth options to use throughout this session
}

// MockProofVerifierCallerSession is an auto generated read-only Go binding around an Ethereum contract,
// with pre-set call options.
type MockProofVerifierCallerSession struct {
	Contract *MockProofVerifierCaller // Generic contract caller binding to set the session for
	CallOpts bind.CallOpts            // Call options to use throughout this session
}

// MockProofVerifierTransactorSession is an auto generated write-only Go binding around an Ethereum contract,
// with pre-set transact options.
type MockProofVerifierTransactorSession struct {
	Contract     *MockProofVerifierTransactor // Generic contract transactor binding to set the session for
	TransactOpts bind.TransactOpts            // Transaction auth options to use throughout this session
}

// MockProofVerifierRaw is an auto generated low-level Go binding around an Ethereum contract.
type MockProofVerifierRaw struct {
	Contract *MockProofVerifier // Generic contract binding to access the raw methods on
}

// MockProofVerifierCallerRaw is an auto generated low-level read-only Go binding around an Ethereum contract.
type MockProofVerifierCallerRaw struct {
	Contract *MockProofVerifierCaller // Generic read-only contract binding to access the raw methods on
}

// MockProofVerifierTransactorRaw is an auto generated low-level write-only Go binding around an Ethereum contract.
type MockProofVerifierTransactorRaw struct {
	Contract *MockProofVerifierTransactor // Generic write-only contract binding to access the raw methods on
}

// NewMockProofVerifier creates a new instance of MockProofVerifier, bound to a specific deployed contract.
func NewMockProofVerifier(address common.Address, backend bind.ContractBackend) (*MockProofVerifier, error) {
	contract, err := bindMockProofVerifier(address, backend, backend, backend)
	if err != nil {
		return nil, err
	}
	return &MockProofVerifier{MockProofVerifierCaller: MockProofVerifierCaller{contract: contract}, MockProofVerifierTransactor: MockProofVerifierTransactor{contract: contract}, MockProofVerifierFilterer: MockProofVerifierFilterer{contract: contract}}, nil
}

// NewMockProofVerifierCaller creates a new read-only instance of MockProofVerifier, bound to a specific deployed contract.
func NewMockProofVerifierCaller(address common.Address, caller bind.ContractCaller) (*MockProofVerifierCaller, error) {
	contract, err := bindMockProofVerifier(address, caller, nil, nil)
	if err != nil {
		return nil, err
	}
	return &MockProofVerifierCaller{contract: contract}, nil
}

// NewMockProofVerifierTransactor creates a new write-only instance of MockProofVerifier, bound to a specific deployed contract.
func NewMockProofVerifierTransactor(address common.Address, transactor bind.ContractTransactor) (*MockProofVerifierTransactor, error) {
	contract, err := bindMockProofVerifier(address, nil, transactor, nil)
	if err != nil {
		return nil, err
	}
	return &MockProofVerifierTransactor{contract: contract}, nil
}

// NewMockProofVerifierFilterer creates a new log filterer instance of MockProofVerifier, bound to a specific deployed contract.
func NewMockProofVerifierFilterer(address common.Address, filterer bind.ContractFilterer) (*MockProofVerifierFilterer, error) {
	contract, err := bindMockProofVerifier(address, nil, nil, filterer)
	if err != nil {
		return nil, err
	}
	return &MockProofVerifierFilterer{contract: contract}, nil
}

// bindMockProofVerifier binds a generic wrapper to an already deployed contract.
func bindMockProofVerifier(address common.Address, caller bind.ContractCaller, transactor bind.ContractTransactor, filterer bind.ContractFilterer) (*bind.BoundContract, error) {
	parsed, err := MockProofVerifierMetaData.GetAbi()
	if err != nil {
		return nil, err
	}
	return bind.NewBoundContract(address, *parsed, caller, transactor, filterer), nil
}

// Call invokes the (constant) contract method with params as input values and
// sets the output to result. The result type might be a single field for simple
// returns, a slice of interfaces for anonymous returns and a struct for named
// returns.
func (_MockProofVerifier *MockProofVerifierRaw) Call(opts *bind.CallOpts, result *[]interface{}, method string, params ...interface{}) error {
	return _MockProofVerifier.Contract.MockProofVerifierCaller.contract.Call(opts, result, method, params...)
}

// Transfer initiates a plain transaction to move funds to the contract, calling
// its default method if one is available.
func (_MockProofVerifier *MockProofVerifierRaw) Transfer(opts *bind.TransactOpts) (*types.Transaction, error) {
	return _MockProofVerifier.Contract.MockProofVerifierTransactor.contract.Transfer(opts)
}

// Transact invokes the (paid) contract method with params as input values.
func (_MockProofVerifier *MockProofVerifierRaw) Transact(opts *bind.TransactOpts, method string, params ...interface{}) (*types.Transaction, error) {
	return _MockProofVerifier.Contract.MockProofVerifierTransactor.contract.Transact(opts, method, params...)
}

// Call invokes the (constant) contract method with params as input values and
// sets the output to result. The result type might be a single field for simple
// returns, a slice of interfaces for anonymous returns and a struct for named
// returns.
func (_MockProofVerifier *MockProofVerifierCallerRaw) Call(opts *bind.CallOpts, result *[]interface{}, method string, params ...interface{}) error {
	return _MockProofVerifier.Contract.contract.Call(opts, result, method, params...)
}

// Transfer initiates a plain transaction to move funds to the contract, calling
// its default method if one is available.
func (_MockProofVerifier *MockProofVerifierTransactorRaw) Transfer(opts *bind.TransactOpts) (*types.Transaction, error) {
	return _MockProofVerifier.Contract.contract.Transfer(opts)
}

// Transact invokes the (paid) contract method with params as input values.
func (_MockProofVerifier *MockProofVerifierTransactorRaw) Transact(opts *bind.TransactOpts, method string, params ...interface{}) (*types.Transaction, error) {
	return _MockProofVerifier.Contract.contract.Transact(opts, method, params...)
}

// Listener is a free data retrieval call binding the contract method 0x271221b3.
//
// Solidity: function listener(uint256 ) view returns(address)
func (_MockProofVerifier *MockProofVerifierCaller) Listener(opts *bind.CallOpts, arg0 *big.Int) (common.Address, error) {
	var out []interface{}
	err := _MockProofVerifier.contract.Call(opts, &out, "listener", arg0)

	if err != nil {
		return *new(common.Address), err
	}

	out0 := *abi.ConvertType(out[0], new(common.Address)).(*common.Address)

	return out0, err

}

// Listener is a free data retrieval call binding the contract method 0x271221b3.
//
// Solidity: function listener(uint256 ) view returns(address)
func (_MockProofVerifier *MockProofVerifierSession) Listener(arg0 *big.Int) (common.Address, error) {
	return _MockProofVerifier.Contract.Listener(&_MockProofVerifier.CallOpts, arg0)
}

// Listener is a free data retrieval call binding the contract method 0x271221b3.
//
// Solidity: function listener(uint256 ) view returns(address)
func (_MockProofVerifier *MockProofVerifierCallerSession) Listener(arg0 *big.Int) (common.Address, error) {
	return _MockProofVerifier.Contract.Listener(&_MockProofVerifier.CallOpts, arg0)
}

// NextDataSetId is a free data retrieval call binding the contract method 0x92a6e08f.
//
// Solidity: function nextDataSetId() view returns(uint256)
func (_MockProofVerifier *MockProofVerifierCaller) NextDataSetId(opts *bind.CallOpts) (*big.Int, error) {
	var out []interface{}
	err := _MockProofVerifier.contract.Call(opts, &out, "nextDataSetId")

	if err != nil {
		return *new(*big.Int), err
	}

	out0 := *abi.ConvertType(out[0], new(*big.Int)).(**big.Int)

	return out0, err

}

// NextDataSetId is a free data retrieval call binding the contract method 0x92a6e08f.
//
// Solidity: function nextDataSetId() view returns(uint256)
func (_MockProofVerifier *MockProofVerifierSession) NextDataSetId() (*big.Int, error) {
	return _MockProofVerifier.Contract.NextDataSetId(&_MockProofVerifier.CallOpts)
}

// NextDataSetId is a free data retrieval call binding the contract method 0x92a6e08f.
//
// Solidity: function nextDataSetId() view returns(uint256)
func (_MockProofVerifier *MockProofVerifierCallerSession) NextDataSetId() (*big.Int, error) {
	return _MockProofVerifier.Contract.NextDataSetId(&_MockProofVerifier.CallOpts)
}

// StorageProvider is a free data retrieval call binding the contract method 0xb31662db.
//
// Solidity: function storageProvider(uint256 ) view returns(address)
func (_MockProofVerifier *MockProofVerifierCaller) StorageProvider(opts *bind.CallOpts, arg0 *big.Int) (common.Address, error) {
	var out []interface{}
	err := _MockProofVerifier.contract.Call(opts, &out, "storageProvider", arg0)

	if err != nil {
		return *new(common.Address), err
	}

	out0 := *abi.ConvertType(out[0], new(common.Address)).(*common.Address)

	return out0, err

}

// StorageProvider is a free data retrieval call binding the contract method 0xb31662db.
//
// Solidity: function storageProvider(uint256 ) view returns(address)
func (_MockProofVerifier *MockProofVerifierSession) StorageProvider(arg0 *big.Int) (common.Address, error) {
	return _MockProofVerifier.Contract.StorageProvider(&_MockProofVerifier.CallOpts, arg0)
}

// StorageProvider is a free data retrieval call binding the contract method 0xb31662db.
//
// Solidity: function storageProvider(uint256 ) view returns(address)
func (_MockProofVerifier *MockProofVerifierCallerSession) StorageProvider(arg0 *big.Int) (common.Address, error) {
	return _MockProofVerifier.Contract.StorageProvider(&_MockProofVerifier.CallOpts, arg0)
}

// CreateDataSet is a paid mutator transaction binding the contract method 0xbbae41cb.
//
// Solidity: function createDataSet(address listenerAddr, bytes extraData) payable returns(uint256 setId)
func (_MockProofVerifier *MockProofVerifierTransactor) CreateDataSet(opts *bind.TransactOpts, listenerAddr common.Address, extraData []byte) (*types.Transaction, error) {
	return _MockProofVerifier.contract.Transact(opts, "createDataSet", listenerAddr, extraData)
}

// CreateDataSet is a paid mutator transaction binding the contract method 0xbbae41cb.
//
// Solidity: function createDataSet(address listenerAddr, bytes extraData) payable returns(uint256 setId)
func (_MockProofVerifier *MockProofVerifierSession) CreateDataSet(listenerAddr common.Address, extraData []byte) (*types.Transaction, error) {
	return _MockProofVerifier.Contract.CreateDataSet(&_MockProofVerifier.TransactOpts, listenerAddr, extraData)
}

// CreateDataSet is a paid mutator transaction binding the contract method 0xbbae41cb.
//
// Solidity: function createDataSet(address listenerAddr, bytes extraData) payable returns(uint256 setId)
func (_MockProofVerifier *MockProofVerifierTransactorSession) CreateDataSet(listenerAddr common.Address, extraData []byte) (*types.Transaction, error) {
	return _MockProofVerifier.Contract.CreateDataSet(&_MockProofVerifier.TransactOpts, listenerAddr, extraData)
}

// SimulateProof is a paid mutator transaction binding the contract method 0xcbd646d6.
//
// Solidity: function simulateProof(uint256 setId) returns()
func (_MockProofVerifier *MockProofVerifierTransactor) SimulateProof(opts *bind.TransactOpts, setId *big.Int) (*types.Transaction, error) {
	return _MockProofVerifier.contract.Transact(opts, "simulateProof", setId)
}

// SimulateProof is a paid mutator transaction binding the contract method 0xcbd646d6.
//
// Solidity: function simulateProof(uint256 setId) returns()
func (_MockProofVerifier *MockProofVerifierSession) SimulateProof(setId *big.Int) (*types.Transaction, error) {
	return _MockProofVerifier.Contract.SimulateProof(&_MockProofVerifier.TransactOpts, setId)
}

// SimulateProof is a paid mutator transaction binding the contract method 0xcbd646d6.
//
// Solidity: function simulateProof(uint256 setId) returns()
func (_MockProofVerifier *MockProofVerifierTransactorSession) SimulateProof(setId *big.Int) (*types.Transaction, error) {
	return _MockProofVerifier.Contract.SimulateProof(&_MockProofVerifier.TransactOpts, setId)
}

// MockProofVerifierDataSetCreatedIterator is returned from FilterDataSetCreated and is used to iterate over the raw logs and unpacked data for DataSetCreated events raised by the MockProofVerifier contract.
type MockProofVerifierDataSetCreatedIterator struct {
	Event *MockProofVerifierDataSetCreated // Event containing the contract specifics and raw log

	contract *bind.BoundContract // Generic contract to use for unpacking event data
	event    string              // Event name to use for unpacking event data

	logs chan types.Log        // Log channel receiving the found contract events
	sub  ethereum.Subscription // Subscription for errors, completion and termination
	done bool                  // Whether the subscription completed delivering logs
	fail error                 // Occurred error to stop iteration
}

// Next advances the iterator to the subsequent event, returning whether there
// are any more events found. In case of a retrieval or parsing error, false is
// returned and Error() can be queried for the exact failure.
func (it *MockProofVerifierDataSetCreatedIterator) Next() bool {
	// If the iterator failed, stop iterating
	if it.fail != nil {
		return false
	}
	// If the iterator completed, deliver directly whatever's available
	if it.done {
		select {
		case log := <-it.logs:
			it.Event = new(MockProofVerifierDataSetCreated)
			if err := it.contract.UnpackLog(it.Event, it.event, log); err != nil {
				it.fail = err
				return false
			}
			it.Event.Raw = log
			return true

		default:
			return false
		}
	}
	// Iterator still in progress, wait for either a data or an error event
	select {
	case log := <-it.logs:
		it.Event = new(MockProofVerifierDataSetCreated)
		if err := it.contract.UnpackLog(it.Event, it.event, log); err != nil {
			it.fail = err
			return false
		}
		it.Event.Raw = log
		return true

	case err := <-it.sub.Err():
		it.done = true
		it.fail = err
		return it.Next()
	}
}

// Error returns any retrieval or parsing error occurred during filtering.
func (it *MockProofVerifierDataSetCreatedIterator) Error() error {
	return it.fail
}

// Close terminates the iteration process, releasing any pending underlying
// resources.
func (it *MockProofVerifierDataSetCreatedIterator) Close() error {
	it.sub.Unsubscribe()
	return nil
}

// MockProofVerifierDataSetCreated represents a DataSetCreated event raised by the MockProofVerifier contract.
type MockProofVerifierDataSetCreated struct {
	SetId           *big.Int
	StorageProvider common.Address
	Raw             types.Log // Blockchain specific contextual infos
}

// FilterDataSetCreated is a free log retrieval operation binding the contract event 0x11369440e1b7135015c16acb9bc14b55b0f4b23b02010c363d34aec2e5b96281.
//
// Solidity: event DataSetCreated(uint256 indexed setId, address indexed storageProvider)
func (_MockProofVerifier *MockProofVerifierFilterer) FilterDataSetCreated(opts *bind.FilterOpts, setId []*big.Int, storageProvider []common.Address) (*MockProofVerifierDataSetCreatedIterator, error) {

	var setIdRule []interface{}
	for _, setIdItem := range setId {
		setIdRule = append(setIdRule, setIdItem)
	}
	var storageProviderRule []interface{}
	for _, storageProviderItem := range storageProvider {
		storageProviderRule = append(storageProviderRule, storageProviderItem)
	}

	logs, sub, err := _MockProofVerifier.contract.FilterLogs(opts, "DataSetCreated", setIdRule, storageProviderRule)
	if err != nil {
		return nil, err
	}
	return &MockProofVerifierDataSetCreatedIterator{contract: _MockProofVerifier.contract, event: "DataSetCreated", logs: logs, sub: sub}, nil
}

// WatchDataSetCreated is a free log subscription operation binding the contract event 0x11369440e1b7135015c16acb9bc14b55b0f4b23b02010c363d34aec2e5b96281.
//
// Solidity: event DataSetCreated(uint256 indexed setId, address indexed storageProvider)
func (_MockProofVerifier *MockProofVerifierFilterer) WatchDataSetCreated(opts *bind.WatchOpts, sink chan<- *MockProofVerifierDataSetCreated, setId []*big.Int, storageProvider []common.Address) (event.Subscription, error) {

	var setIdRule []interface{}
	for _, setIdItem := range setId {
		setIdRule = append(setIdRule, setIdItem)
	}
	var storageProviderRule []interface{}
	for _, storageProviderItem := range storageProvider {
		storageProviderRule = append(storageProviderRule, storageProviderItem)
	}

	logs, sub, err := _MockProofVerifier.contract.WatchLogs(opts, "DataSetCreated", setIdRule, storageProviderRule)
	if err != nil {
		return nil, err
	}
	return event.NewSubscription(func(quit <-chan struct{}) error {
		defer sub.Unsubscribe()
		for {
			select {
			case log := <-logs:
				// New log arrived, parse the event and forward to the user
				event := new(MockProofVerifierDataSetCreated)
				if err := _MockProofVerifier.contract.UnpackLog(event, "DataSetCreated", log); err != nil {
					return err
				}
				event.Raw = log

				select {
				case sink <- event:
				case err := <-sub.Err():
					return err
				case <-quit:
					return nil
				}
			case err := <-sub.Err():
				return err
			case <-quit:
				return nil
			}
		}
	}), nil
}

// ParseDataSetCreated is a log parse operation binding the contract event 0x11369440e1b7135015c16acb9bc14b55b0f4b23b02010c363d34aec2e5b96281.
//
// Solidity: event DataSetCreated(uint256 indexed setId, address indexed storageProvider)
func (_MockProofVerifier *MockProofVerifierFilterer) ParseDataSetCreated(log types.Log) (*MockProofVerifierDataSetCreated, error) {
	event := new(MockProofVerifierDataSetCreated)
	if err := _MockProofVerifier.contract.UnpackLog(event, "DataSetCreated", log); err != nil {
		return nil, err
	}
	event.Raw = log
	return event, nil
}

// MockProofVerifierPossessionProvenIterator is returned from FilterPossessionProven and is used to iterate over the raw logs and unpacked data for PossessionProven events raised by the MockProofVerifier contract.
type MockProofVerifierPossessionProvenIterator struct {
	Event *MockProofVerifierPossessionProven // Event containing the contract specifics and raw log

	contract *bind.BoundContract // Generic contract to use for unpacking event data
	event    string              // Event name to use for unpacking event data

	logs chan types.Log        // Log channel receiving the found contract events
	sub  ethereum.Subscription // Subscription for errors, completion and termination
	done bool                  // Whether the subscription completed delivering logs
	fail error                 // Occurred error to stop iteration
}

// Next advances the iterator to the subsequent event, returning whether there
// are any more events found. In case of a retrieval or parsing error, false is
// returned and Error() can be queried for the exact failure.
func (it *MockProofVerifierPossessionProvenIterator) Next() bool {
	// If the iterator failed, stop iterating
	if it.fail != nil {
		return false
	}
	// If the iterator completed, deliver directly whatever's available
	if it.done {
		select {
		case log := <-it.logs:
			it.Event = new(MockProofVerifierPossessionProven)
			if err := it.contract.UnpackLog(it.Event, it.event, log); err != nil {
				it.fail = err
				return false
			}
			it.Event.Raw = log
			return true

		default:
			return false
		}
	}
	// Iterator still in progress, wait for either a data or an error event
	select {
	case log := <-it.logs:
		it.Event = new(MockProofVerifierPossessionProven)
		if err := it.contract.UnpackLog(it.Event, it.event, log); err != nil {
			it.fail = err
			return false
		}
		it.Event.Raw = log
		return true

	case err := <-it.sub.Err():
		it.done = true
		it.fail = err
		return it.Next()
	}
}

// Error returns any retrieval or parsing error occurred during filtering.
func (it *MockProofVerifierPossessionProvenIterator) Error() error {
	return it.fail
}

// Close terminates the iteration process, releasing any pending underlying
// resources.
func (it *MockProofVerifierPossessionProvenIterator) Close() error {
	it.sub.Unsubscribe()
	return nil
}

// MockProofVerifierPossessionProven represents a PossessionProven event raised by the MockProofVerifier contract.
type MockProofVerifierPossessionProven struct {
	SetId *big.Int
	Raw   types.Log // Blockchain specific contextual infos
}

// FilterPossessionProven is a free log retrieval operation binding the contract event 0x8fcfe2d16538be9d0fbb9a19b1a403567f6cb498484d8c2605777d1fa92cc6ac.
//
// Solidity: event PossessionProven(uint256 indexed setId)
func (_MockProofVerifier *MockProofVerifierFilterer) FilterPossessionProven(opts *bind.FilterOpts, setId []*big.Int) (*MockProofVerifierPossessionProvenIterator, error) {

	var setIdRule []interface{}
	for _, setIdItem := range setId {
		setIdRule = append(setIdRule, setIdItem)
	}

	logs, sub, err := _MockProofVerifier.contract.FilterLogs(opts, "PossessionProven", setIdRule)
	if err != nil {
		return nil, err
	}
	return &MockProofVerifierPossessionProvenIterator{contract: _MockProofVerifier.contract, event: "PossessionProven", logs: logs, sub: sub}, nil
}

// WatchPossessionProven is a free log subscription operation binding the contract event 0x8fcfe2d16538be9d0fbb9a19b1a403567f6cb498484d8c2605777d1fa92cc6ac.
//
// Solidity: event PossessionProven(uint256 indexed setId)
func (_MockProofVerifier *MockProofVerifierFilterer) WatchPossessionProven(opts *bind.WatchOpts, sink chan<- *MockProofVerifierPossessionProven, setId []*big.Int) (event.Subscription, error) {

	var setIdRule []interface{}
	for _, setIdItem := range setId {
		setIdRule = append(setIdRule, setIdItem)
	}

	logs, sub, err := _MockProofVerifier.contract.WatchLogs(opts, "PossessionProven", setIdRule)
	if err != nil {
		return nil, err
	}
	return event.NewSubscription(func(quit <-chan struct{}) error {
		defer sub.Unsubscribe()
		for {
			select {
			case log := <-logs:
				// New log arrived, parse the event and forward to the user
				event := new(MockProofVerifierPossessionProven)
				if err := _MockProofVerifier.contract.UnpackLog(event, "PossessionProven", log); err != nil {
					return err
				}
				event.Raw = log

				select {
				case sink <- event:
				case err := <-sub.Err():
					return err
				case <-quit:
					return nil
				}
			case err := <-sub.Err():
				return err
			case <-quit:
				return nil
			}
		}
	}), nil
}

// ParsePossessionProven is a log parse operation binding the contract event 0x8fcfe2d16538be9d0fbb9a19b1a403567f6cb498484d8c2605777d1fa92cc6ac.
//
// Solidity: event PossessionProven(uint256 indexed setId)
func (_MockProofVerifier *MockProofVerifierFilterer) ParsePossessionProven(log types.Log) (*MockProofVerifierPossessionProven, error) {
	event := new(MockProofVerifierPossessionProven)
	if err := _MockProofVerifier.contract.UnpackLog(event, "PossessionProven", log); err != nil {
		return nil, err
	}
	event.Raw = log
	return event, nil
}
