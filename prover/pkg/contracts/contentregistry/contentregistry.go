// SPDX-License-Identifier: MIT
// Generated from contracts/out/ContentRegistry.sol/ContentRegistry.json via abigen.
// Do not edit by hand; run ./scripts/gen-bindings.sh instead.

// Code generated - DO NOT EDIT.
// This file is a generated binding and any manual changes will be lost.

package contentregistry

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

// ContentRegistryContent is an auto generated low-level Go binding around an user-defined struct.
type ContentRegistryContent struct {
	Owner        common.Address
	ActiveDealId *big.Int
	PieceSize    uint64
	FirstSeen    uint64
	LastUpdated  uint64
	EnsNode      [32]byte
}

// ContentRegistryMetaData contains all meta data concerning the ContentRegistry contract.
var ContentRegistryMetaData = &bind.MetaData{
	ABI: "[{\"type\":\"constructor\",\"inputs\":[],\"stateMutability\":\"nonpayable\"},{\"type\":\"function\",\"name\":\"admin\",\"inputs\":[],\"outputs\":[{\"name\":\"\",\"type\":\"address\",\"internalType\":\"address\"}],\"stateMutability\":\"view\"},{\"type\":\"function\",\"name\":\"bindENS\",\"inputs\":[{\"name\":\"commpHash\",\"type\":\"bytes32\",\"internalType\":\"bytes32\"},{\"name\":\"ensNode\",\"type\":\"bytes32\",\"internalType\":\"bytes32\"}],\"outputs\":[],\"stateMutability\":\"nonpayable\"},{\"type\":\"function\",\"name\":\"clearActiveDeal\",\"inputs\":[{\"name\":\"commpHash\",\"type\":\"bytes32\",\"internalType\":\"bytes32\"},{\"name\":\"expectedDealId\",\"type\":\"uint256\",\"internalType\":\"uint256\"}],\"outputs\":[],\"stateMutability\":\"nonpayable\"},{\"type\":\"function\",\"name\":\"commpByENS\",\"inputs\":[{\"name\":\"\",\"type\":\"bytes32\",\"internalType\":\"bytes32\"}],\"outputs\":[{\"name\":\"\",\"type\":\"bytes32\",\"internalType\":\"bytes32\"}],\"stateMutability\":\"view\"},{\"type\":\"function\",\"name\":\"contentByHash\",\"inputs\":[{\"name\":\"\",\"type\":\"bytes32\",\"internalType\":\"bytes32\"}],\"outputs\":[{\"name\":\"owner\",\"type\":\"address\",\"internalType\":\"address\"},{\"name\":\"activeDealId\",\"type\":\"uint256\",\"internalType\":\"uint256\"},{\"name\":\"pieceSize\",\"type\":\"uint64\",\"internalType\":\"uint64\"},{\"name\":\"firstSeen\",\"type\":\"uint64\",\"internalType\":\"uint64\"},{\"name\":\"lastUpdated\",\"type\":\"uint64\",\"internalType\":\"uint64\"},{\"name\":\"ensNode\",\"type\":\"bytes32\",\"internalType\":\"bytes32\"}],\"stateMutability\":\"view\"},{\"type\":\"function\",\"name\":\"getContent\",\"inputs\":[{\"name\":\"commpHash\",\"type\":\"bytes32\",\"internalType\":\"bytes32\"}],\"outputs\":[{\"name\":\"\",\"type\":\"tuple\",\"internalType\":\"structContentRegistry.Content\",\"components\":[{\"name\":\"owner\",\"type\":\"address\",\"internalType\":\"address\"},{\"name\":\"activeDealId\",\"type\":\"uint256\",\"internalType\":\"uint256\"},{\"name\":\"pieceSize\",\"type\":\"uint64\",\"internalType\":\"uint64\"},{\"name\":\"firstSeen\",\"type\":\"uint64\",\"internalType\":\"uint64\"},{\"name\":\"lastUpdated\",\"type\":\"uint64\",\"internalType\":\"uint64\"},{\"name\":\"ensNode\",\"type\":\"bytes32\",\"internalType\":\"bytes32\"}]}],\"stateMutability\":\"view\"},{\"type\":\"function\",\"name\":\"hasActiveDeal\",\"inputs\":[{\"name\":\"commpHash\",\"type\":\"bytes32\",\"internalType\":\"bytes32\"}],\"outputs\":[{\"name\":\"\",\"type\":\"bool\",\"internalType\":\"bool\"}],\"stateMutability\":\"view\"},{\"type\":\"function\",\"name\":\"marketplace\",\"inputs\":[],\"outputs\":[{\"name\":\"\",\"type\":\"address\",\"internalType\":\"address\"}],\"stateMutability\":\"view\"},{\"type\":\"function\",\"name\":\"registerContent\",\"inputs\":[{\"name\":\"commpHash\",\"type\":\"bytes32\",\"internalType\":\"bytes32\"},{\"name\":\"owner\",\"type\":\"address\",\"internalType\":\"address\"},{\"name\":\"dealId\",\"type\":\"uint256\",\"internalType\":\"uint256\"},{\"name\":\"pieceSize\",\"type\":\"uint64\",\"internalType\":\"uint64\"}],\"outputs\":[],\"stateMutability\":\"nonpayable\"},{\"type\":\"function\",\"name\":\"resolveENS\",\"inputs\":[{\"name\":\"ensNode\",\"type\":\"bytes32\",\"internalType\":\"bytes32\"}],\"outputs\":[{\"name\":\"\",\"type\":\"tuple\",\"internalType\":\"structContentRegistry.Content\",\"components\":[{\"name\":\"owner\",\"type\":\"address\",\"internalType\":\"address\"},{\"name\":\"activeDealId\",\"type\":\"uint256\",\"internalType\":\"uint256\"},{\"name\":\"pieceSize\",\"type\":\"uint64\",\"internalType\":\"uint64\"},{\"name\":\"firstSeen\",\"type\":\"uint64\",\"internalType\":\"uint64\"},{\"name\":\"lastUpdated\",\"type\":\"uint64\",\"internalType\":\"uint64\"},{\"name\":\"ensNode\",\"type\":\"bytes32\",\"internalType\":\"bytes32\"}]}],\"stateMutability\":\"view\"},{\"type\":\"function\",\"name\":\"setMarketplace\",\"inputs\":[{\"name\":\"_marketplace\",\"type\":\"address\",\"internalType\":\"address\"}],\"outputs\":[],\"stateMutability\":\"nonpayable\"},{\"type\":\"function\",\"name\":\"transferAdmin\",\"inputs\":[{\"name\":\"newAdmin\",\"type\":\"address\",\"internalType\":\"address\"}],\"outputs\":[],\"stateMutability\":\"nonpayable\"},{\"type\":\"function\",\"name\":\"unbindENS\",\"inputs\":[{\"name\":\"commpHash\",\"type\":\"bytes32\",\"internalType\":\"bytes32\"}],\"outputs\":[],\"stateMutability\":\"nonpayable\"},{\"type\":\"event\",\"name\":\"ContentDealUpdated\",\"inputs\":[{\"name\":\"commpHash\",\"type\":\"bytes32\",\"indexed\":true,\"internalType\":\"bytes32\"},{\"name\":\"oldDealId\",\"type\":\"uint256\",\"indexed\":false,\"internalType\":\"uint256\"},{\"name\":\"newDealId\",\"type\":\"uint256\",\"indexed\":false,\"internalType\":\"uint256\"}],\"anonymous\":false},{\"type\":\"event\",\"name\":\"ContentRegistered\",\"inputs\":[{\"name\":\"commpHash\",\"type\":\"bytes32\",\"indexed\":true,\"internalType\":\"bytes32\"},{\"name\":\"owner\",\"type\":\"address\",\"indexed\":true,\"internalType\":\"address\"},{\"name\":\"dealId\",\"type\":\"uint256\",\"indexed\":true,\"internalType\":\"uint256\"},{\"name\":\"pieceSize\",\"type\":\"uint64\",\"indexed\":false,\"internalType\":\"uint64\"}],\"anonymous\":false},{\"type\":\"event\",\"name\":\"ENSBound\",\"inputs\":[{\"name\":\"commpHash\",\"type\":\"bytes32\",\"indexed\":true,\"internalType\":\"bytes32\"},{\"name\":\"ensNode\",\"type\":\"bytes32\",\"indexed\":true,\"internalType\":\"bytes32\"},{\"name\":\"by\",\"type\":\"address\",\"indexed\":true,\"internalType\":\"address\"}],\"anonymous\":false},{\"type\":\"event\",\"name\":\"ENSUnbound\",\"inputs\":[{\"name\":\"commpHash\",\"type\":\"bytes32\",\"indexed\":true,\"internalType\":\"bytes32\"},{\"name\":\"ensNode\",\"type\":\"bytes32\",\"indexed\":true,\"internalType\":\"bytes32\"}],\"anonymous\":false},{\"type\":\"event\",\"name\":\"MarketplaceSet\",\"inputs\":[{\"name\":\"oldMarketplace\",\"type\":\"address\",\"indexed\":true,\"internalType\":\"address\"},{\"name\":\"newMarketplace\",\"type\":\"address\",\"indexed\":true,\"internalType\":\"address\"}],\"anonymous\":false},{\"type\":\"error\",\"name\":\"ContentNotFound\",\"inputs\":[]},{\"type\":\"error\",\"name\":\"ENSAlreadyBound\",\"inputs\":[]},{\"type\":\"error\",\"name\":\"ENSNotBoundHere\",\"inputs\":[]},{\"type\":\"error\",\"name\":\"NotContentOwner\",\"inputs\":[]},{\"type\":\"error\",\"name\":\"OnlyAdmin\",\"inputs\":[]},{\"type\":\"error\",\"name\":\"OnlyMarketplace\",\"inputs\":[]}]",
	Bin: "0x60808060405234602757600380546001600160a01b031916331790556108a3908161002c8239f35b5f80fdfe6080806040526004361015610012575f80fd5b5f3560e01c90816306d3e11514610611575080634349300a146105425780635cc15001146104b157806373ad6c2d1461043f57806375829def146103e7578063922bba251461036f57806392decf6f14610246578063a92df3d314610217578063abc8c7af146101ef578063b6fbdfa814610122578063c788132a14610101578063edf3f8bc146100d75763f851a440146100ab575f80fd5b346100d3575f3660031901126100d3576003546040516001600160a01b039091168152602090f35b5f80fd5b346100d35760203660031901126100d3576004355f526001602052602060405f2054604051908152f35b346100d35760403660031901126100d3576101206024356004356107df565b005b346100d35760203660031901126100d357600435805f525f60205260405f2090600282019160018060401b03835460401c16156101e05780546001600160a01b031633036101d15760030180549283156101c2575f90915561018e90426001600160401b0316906106ac565b815f5260016020525f60408120557fbc6171f1c4547d68d95e787256d2a4d44cfb690e90d35cf07721d2fbb341d69b5f80a3005b636bf0fdc160e01b5f5260045ffd5b63796d3af960e11b5f5260045ffd5b6315f2470160e31b5f5260045ffd5b346100d3575f3660031901126100d3576002546040516001600160a01b039091168152602090f35b346100d35760203660031901126100d357610242610236600435610734565b6040519182918261063b565b0390f35b346100d35760803660031901126100d3576004356024356001600160a01b038116908190036100d357604435916064356001600160401b038116908190036100d3576002546001600160a01b0316330361036057815f525f60205260405f206102d960016002830192818060401b03845460401c161561032f575b0180549087905591426001600160401b0316906106ac565b8061030c575060207fd1814539fff4d0a0cbe33001338582c1dc3e5eea8b734484cd8daa48a1b05b5091604051908152a4005b90505f51602061084e5f395f51905f5292506040919382519182526020820152a2005b86828060a01b03198254161781558484548360401b8460801b034260401b1690848060801b031916171784556102c1565b632b35006d60e21b5f5260045ffd5b346100d35760203660031901126100d3576004355f908152602081815260409182902080546001820154600283015460039093015485516001600160a01b039093168352938201526001600160401b03808316828601529382901c84166060820152608091821c9093169083015260a082015260c090f35b346100d35760203660031901126100d357610400610696565b600354906001600160a01b0382163303610430576001600160a01b03166001600160a01b03199190911617600355005b634755657960e01b5f5260045ffd5b346100d35760203660031901126100d357610458610696565b6003546001600160a01b03163303610430576002546001600160a01b0391821691829082167fe928dbb5d9a59bebb2adca04fb75586d6ad3c04cbf312f3b9ea24086c4b93a065f80a36001600160a01b03191617600255005b346100d35760203660031901126100d3576104ca610708565b506004355f525f60205261024260405f2060036104e56106d5565b82546001600160a01b031681526001830154602082015260028301546001600160401b0380821660408085019190915282811c82166060850152608092831c90911691830191909152919092015460a0830152519182918261063b565b346100d35760403660031901126100d357600435602435815f525f60205260405f206002810160018060401b03815460401c16156101e05781546001600160a01b031633036101d157825f52600160205260405f2054848115159182610606575b50506105f7578260036105c193015560018060401b034216906106ac565b805f5260016020528160405f205533917ff7f84a553ca03faa4c3fcd10b5c62bcf67b1e55fc52fed74344b699cfd2bb1e45f80a4005b630172f88d60e61b5f5260045ffd5b1415905084866105a3565b346100d35760203660031901126100d3576020906004355f525f8252600160405f20015415158152f35b81516001600160a01b03168152602080830151908201526040808301516001600160401b03908116918301919091526060808401518216908301526080808401519091169082015260a0918201519181019190915260c00190565b600435906001600160a01b03821682036100d357565b8054600160801b600160c01b03191660809290921b600160801b600160c01b0316919091179055565b6040519060c082016001600160401b038111838210176106f457604052565b634e487b7160e01b5f52604160045260245ffd5b6107106106d5565b905f82525f60208301525f60408301525f60608301525f60808301525f60a0830152565b61073c610708565b505f52600160205260405f205480156107b2575f525f60205260405f2060036107636106d5565b82546001600160a01b031681526001830154602082015260028301546001600160401b0380821660408085019190915282901c81166060840152608091821c169082015291015460a082015290565b506107bb6106d5565b5f81525f60208201525f60408201525f60608201525f60808201525f60a082015290565b6002549091906001600160a01b0316330361036057815f525f60205260405f20600181019180835403610847575f9092555f51602061084e5f395f51905f529160409161083990426001600160401b0316906002016106ac565b81519081525f6020820152a2565b5050505056fe2d3f3ed8bebe96ed332e19ca2fb9a9793f64ca54196529bdb5b5d8c428c8676ba2646970667358221220383ceabb98be0a731604954e8dd1c5a46b26a653984d0b57ac1151efa9c167b464736f6c634300081e0033",
}

// ContentRegistryABI is the input ABI used to generate the binding from.
// Deprecated: Use ContentRegistryMetaData.ABI instead.
var ContentRegistryABI = ContentRegistryMetaData.ABI

// ContentRegistryBin is the compiled bytecode used for deploying new contracts.
// Deprecated: Use ContentRegistryMetaData.Bin instead.
var ContentRegistryBin = ContentRegistryMetaData.Bin

// DeployContentRegistry deploys a new Ethereum contract, binding an instance of ContentRegistry to it.
func DeployContentRegistry(auth *bind.TransactOpts, backend bind.ContractBackend) (common.Address, *types.Transaction, *ContentRegistry, error) {
	parsed, err := ContentRegistryMetaData.GetAbi()
	if err != nil {
		return common.Address{}, nil, nil, err
	}
	if parsed == nil {
		return common.Address{}, nil, nil, errors.New("GetABI returned nil")
	}

	address, tx, contract, err := bind.DeployContract(auth, *parsed, common.FromHex(ContentRegistryBin), backend)
	if err != nil {
		return common.Address{}, nil, nil, err
	}
	return address, tx, &ContentRegistry{ContentRegistryCaller: ContentRegistryCaller{contract: contract}, ContentRegistryTransactor: ContentRegistryTransactor{contract: contract}, ContentRegistryFilterer: ContentRegistryFilterer{contract: contract}}, nil
}

// ContentRegistry is an auto generated Go binding around an Ethereum contract.
type ContentRegistry struct {
	ContentRegistryCaller     // Read-only binding to the contract
	ContentRegistryTransactor // Write-only binding to the contract
	ContentRegistryFilterer   // Log filterer for contract events
}

// ContentRegistryCaller is an auto generated read-only Go binding around an Ethereum contract.
type ContentRegistryCaller struct {
	contract *bind.BoundContract // Generic contract wrapper for the low level calls
}

// ContentRegistryTransactor is an auto generated write-only Go binding around an Ethereum contract.
type ContentRegistryTransactor struct {
	contract *bind.BoundContract // Generic contract wrapper for the low level calls
}

// ContentRegistryFilterer is an auto generated log filtering Go binding around an Ethereum contract events.
type ContentRegistryFilterer struct {
	contract *bind.BoundContract // Generic contract wrapper for the low level calls
}

// ContentRegistrySession is an auto generated Go binding around an Ethereum contract,
// with pre-set call and transact options.
type ContentRegistrySession struct {
	Contract     *ContentRegistry  // Generic contract binding to set the session for
	CallOpts     bind.CallOpts     // Call options to use throughout this session
	TransactOpts bind.TransactOpts // Transaction auth options to use throughout this session
}

// ContentRegistryCallerSession is an auto generated read-only Go binding around an Ethereum contract,
// with pre-set call options.
type ContentRegistryCallerSession struct {
	Contract *ContentRegistryCaller // Generic contract caller binding to set the session for
	CallOpts bind.CallOpts          // Call options to use throughout this session
}

// ContentRegistryTransactorSession is an auto generated write-only Go binding around an Ethereum contract,
// with pre-set transact options.
type ContentRegistryTransactorSession struct {
	Contract     *ContentRegistryTransactor // Generic contract transactor binding to set the session for
	TransactOpts bind.TransactOpts          // Transaction auth options to use throughout this session
}

// ContentRegistryRaw is an auto generated low-level Go binding around an Ethereum contract.
type ContentRegistryRaw struct {
	Contract *ContentRegistry // Generic contract binding to access the raw methods on
}

// ContentRegistryCallerRaw is an auto generated low-level read-only Go binding around an Ethereum contract.
type ContentRegistryCallerRaw struct {
	Contract *ContentRegistryCaller // Generic read-only contract binding to access the raw methods on
}

// ContentRegistryTransactorRaw is an auto generated low-level write-only Go binding around an Ethereum contract.
type ContentRegistryTransactorRaw struct {
	Contract *ContentRegistryTransactor // Generic write-only contract binding to access the raw methods on
}

// NewContentRegistry creates a new instance of ContentRegistry, bound to a specific deployed contract.
func NewContentRegistry(address common.Address, backend bind.ContractBackend) (*ContentRegistry, error) {
	contract, err := bindContentRegistry(address, backend, backend, backend)
	if err != nil {
		return nil, err
	}
	return &ContentRegistry{ContentRegistryCaller: ContentRegistryCaller{contract: contract}, ContentRegistryTransactor: ContentRegistryTransactor{contract: contract}, ContentRegistryFilterer: ContentRegistryFilterer{contract: contract}}, nil
}

// NewContentRegistryCaller creates a new read-only instance of ContentRegistry, bound to a specific deployed contract.
func NewContentRegistryCaller(address common.Address, caller bind.ContractCaller) (*ContentRegistryCaller, error) {
	contract, err := bindContentRegistry(address, caller, nil, nil)
	if err != nil {
		return nil, err
	}
	return &ContentRegistryCaller{contract: contract}, nil
}

// NewContentRegistryTransactor creates a new write-only instance of ContentRegistry, bound to a specific deployed contract.
func NewContentRegistryTransactor(address common.Address, transactor bind.ContractTransactor) (*ContentRegistryTransactor, error) {
	contract, err := bindContentRegistry(address, nil, transactor, nil)
	if err != nil {
		return nil, err
	}
	return &ContentRegistryTransactor{contract: contract}, nil
}

// NewContentRegistryFilterer creates a new log filterer instance of ContentRegistry, bound to a specific deployed contract.
func NewContentRegistryFilterer(address common.Address, filterer bind.ContractFilterer) (*ContentRegistryFilterer, error) {
	contract, err := bindContentRegistry(address, nil, nil, filterer)
	if err != nil {
		return nil, err
	}
	return &ContentRegistryFilterer{contract: contract}, nil
}

// bindContentRegistry binds a generic wrapper to an already deployed contract.
func bindContentRegistry(address common.Address, caller bind.ContractCaller, transactor bind.ContractTransactor, filterer bind.ContractFilterer) (*bind.BoundContract, error) {
	parsed, err := ContentRegistryMetaData.GetAbi()
	if err != nil {
		return nil, err
	}
	return bind.NewBoundContract(address, *parsed, caller, transactor, filterer), nil
}

// Call invokes the (constant) contract method with params as input values and
// sets the output to result. The result type might be a single field for simple
// returns, a slice of interfaces for anonymous returns and a struct for named
// returns.
func (_ContentRegistry *ContentRegistryRaw) Call(opts *bind.CallOpts, result *[]interface{}, method string, params ...interface{}) error {
	return _ContentRegistry.Contract.ContentRegistryCaller.contract.Call(opts, result, method, params...)
}

// Transfer initiates a plain transaction to move funds to the contract, calling
// its default method if one is available.
func (_ContentRegistry *ContentRegistryRaw) Transfer(opts *bind.TransactOpts) (*types.Transaction, error) {
	return _ContentRegistry.Contract.ContentRegistryTransactor.contract.Transfer(opts)
}

// Transact invokes the (paid) contract method with params as input values.
func (_ContentRegistry *ContentRegistryRaw) Transact(opts *bind.TransactOpts, method string, params ...interface{}) (*types.Transaction, error) {
	return _ContentRegistry.Contract.ContentRegistryTransactor.contract.Transact(opts, method, params...)
}

// Call invokes the (constant) contract method with params as input values and
// sets the output to result. The result type might be a single field for simple
// returns, a slice of interfaces for anonymous returns and a struct for named
// returns.
func (_ContentRegistry *ContentRegistryCallerRaw) Call(opts *bind.CallOpts, result *[]interface{}, method string, params ...interface{}) error {
	return _ContentRegistry.Contract.contract.Call(opts, result, method, params...)
}

// Transfer initiates a plain transaction to move funds to the contract, calling
// its default method if one is available.
func (_ContentRegistry *ContentRegistryTransactorRaw) Transfer(opts *bind.TransactOpts) (*types.Transaction, error) {
	return _ContentRegistry.Contract.contract.Transfer(opts)
}

// Transact invokes the (paid) contract method with params as input values.
func (_ContentRegistry *ContentRegistryTransactorRaw) Transact(opts *bind.TransactOpts, method string, params ...interface{}) (*types.Transaction, error) {
	return _ContentRegistry.Contract.contract.Transact(opts, method, params...)
}

// Admin is a free data retrieval call binding the contract method 0xf851a440.
//
// Solidity: function admin() view returns(address)
func (_ContentRegistry *ContentRegistryCaller) Admin(opts *bind.CallOpts) (common.Address, error) {
	var out []interface{}
	err := _ContentRegistry.contract.Call(opts, &out, "admin")

	if err != nil {
		return *new(common.Address), err
	}

	out0 := *abi.ConvertType(out[0], new(common.Address)).(*common.Address)

	return out0, err

}

// Admin is a free data retrieval call binding the contract method 0xf851a440.
//
// Solidity: function admin() view returns(address)
func (_ContentRegistry *ContentRegistrySession) Admin() (common.Address, error) {
	return _ContentRegistry.Contract.Admin(&_ContentRegistry.CallOpts)
}

// Admin is a free data retrieval call binding the contract method 0xf851a440.
//
// Solidity: function admin() view returns(address)
func (_ContentRegistry *ContentRegistryCallerSession) Admin() (common.Address, error) {
	return _ContentRegistry.Contract.Admin(&_ContentRegistry.CallOpts)
}

// CommpByENS is a free data retrieval call binding the contract method 0xedf3f8bc.
//
// Solidity: function commpByENS(bytes32 ) view returns(bytes32)
func (_ContentRegistry *ContentRegistryCaller) CommpByENS(opts *bind.CallOpts, arg0 [32]byte) ([32]byte, error) {
	var out []interface{}
	err := _ContentRegistry.contract.Call(opts, &out, "commpByENS", arg0)

	if err != nil {
		return *new([32]byte), err
	}

	out0 := *abi.ConvertType(out[0], new([32]byte)).(*[32]byte)

	return out0, err

}

// CommpByENS is a free data retrieval call binding the contract method 0xedf3f8bc.
//
// Solidity: function commpByENS(bytes32 ) view returns(bytes32)
func (_ContentRegistry *ContentRegistrySession) CommpByENS(arg0 [32]byte) ([32]byte, error) {
	return _ContentRegistry.Contract.CommpByENS(&_ContentRegistry.CallOpts, arg0)
}

// CommpByENS is a free data retrieval call binding the contract method 0xedf3f8bc.
//
// Solidity: function commpByENS(bytes32 ) view returns(bytes32)
func (_ContentRegistry *ContentRegistryCallerSession) CommpByENS(arg0 [32]byte) ([32]byte, error) {
	return _ContentRegistry.Contract.CommpByENS(&_ContentRegistry.CallOpts, arg0)
}

// ContentByHash is a free data retrieval call binding the contract method 0x922bba25.
//
// Solidity: function contentByHash(bytes32 ) view returns(address owner, uint256 activeDealId, uint64 pieceSize, uint64 firstSeen, uint64 lastUpdated, bytes32 ensNode)
func (_ContentRegistry *ContentRegistryCaller) ContentByHash(opts *bind.CallOpts, arg0 [32]byte) (struct {
	Owner        common.Address
	ActiveDealId *big.Int
	PieceSize    uint64
	FirstSeen    uint64
	LastUpdated  uint64
	EnsNode      [32]byte
}, error) {
	var out []interface{}
	err := _ContentRegistry.contract.Call(opts, &out, "contentByHash", arg0)

	outstruct := new(struct {
		Owner        common.Address
		ActiveDealId *big.Int
		PieceSize    uint64
		FirstSeen    uint64
		LastUpdated  uint64
		EnsNode      [32]byte
	})
	if err != nil {
		return *outstruct, err
	}

	outstruct.Owner = *abi.ConvertType(out[0], new(common.Address)).(*common.Address)
	outstruct.ActiveDealId = *abi.ConvertType(out[1], new(*big.Int)).(**big.Int)
	outstruct.PieceSize = *abi.ConvertType(out[2], new(uint64)).(*uint64)
	outstruct.FirstSeen = *abi.ConvertType(out[3], new(uint64)).(*uint64)
	outstruct.LastUpdated = *abi.ConvertType(out[4], new(uint64)).(*uint64)
	outstruct.EnsNode = *abi.ConvertType(out[5], new([32]byte)).(*[32]byte)

	return *outstruct, err

}

// ContentByHash is a free data retrieval call binding the contract method 0x922bba25.
//
// Solidity: function contentByHash(bytes32 ) view returns(address owner, uint256 activeDealId, uint64 pieceSize, uint64 firstSeen, uint64 lastUpdated, bytes32 ensNode)
func (_ContentRegistry *ContentRegistrySession) ContentByHash(arg0 [32]byte) (struct {
	Owner        common.Address
	ActiveDealId *big.Int
	PieceSize    uint64
	FirstSeen    uint64
	LastUpdated  uint64
	EnsNode      [32]byte
}, error) {
	return _ContentRegistry.Contract.ContentByHash(&_ContentRegistry.CallOpts, arg0)
}

// ContentByHash is a free data retrieval call binding the contract method 0x922bba25.
//
// Solidity: function contentByHash(bytes32 ) view returns(address owner, uint256 activeDealId, uint64 pieceSize, uint64 firstSeen, uint64 lastUpdated, bytes32 ensNode)
func (_ContentRegistry *ContentRegistryCallerSession) ContentByHash(arg0 [32]byte) (struct {
	Owner        common.Address
	ActiveDealId *big.Int
	PieceSize    uint64
	FirstSeen    uint64
	LastUpdated  uint64
	EnsNode      [32]byte
}, error) {
	return _ContentRegistry.Contract.ContentByHash(&_ContentRegistry.CallOpts, arg0)
}

// GetContent is a free data retrieval call binding the contract method 0x5cc15001.
//
// Solidity: function getContent(bytes32 commpHash) view returns((address,uint256,uint64,uint64,uint64,bytes32))
func (_ContentRegistry *ContentRegistryCaller) GetContent(opts *bind.CallOpts, commpHash [32]byte) (ContentRegistryContent, error) {
	var out []interface{}
	err := _ContentRegistry.contract.Call(opts, &out, "getContent", commpHash)

	if err != nil {
		return *new(ContentRegistryContent), err
	}

	out0 := *abi.ConvertType(out[0], new(ContentRegistryContent)).(*ContentRegistryContent)

	return out0, err

}

// GetContent is a free data retrieval call binding the contract method 0x5cc15001.
//
// Solidity: function getContent(bytes32 commpHash) view returns((address,uint256,uint64,uint64,uint64,bytes32))
func (_ContentRegistry *ContentRegistrySession) GetContent(commpHash [32]byte) (ContentRegistryContent, error) {
	return _ContentRegistry.Contract.GetContent(&_ContentRegistry.CallOpts, commpHash)
}

// GetContent is a free data retrieval call binding the contract method 0x5cc15001.
//
// Solidity: function getContent(bytes32 commpHash) view returns((address,uint256,uint64,uint64,uint64,bytes32))
func (_ContentRegistry *ContentRegistryCallerSession) GetContent(commpHash [32]byte) (ContentRegistryContent, error) {
	return _ContentRegistry.Contract.GetContent(&_ContentRegistry.CallOpts, commpHash)
}

// HasActiveDeal is a free data retrieval call binding the contract method 0x06d3e115.
//
// Solidity: function hasActiveDeal(bytes32 commpHash) view returns(bool)
func (_ContentRegistry *ContentRegistryCaller) HasActiveDeal(opts *bind.CallOpts, commpHash [32]byte) (bool, error) {
	var out []interface{}
	err := _ContentRegistry.contract.Call(opts, &out, "hasActiveDeal", commpHash)

	if err != nil {
		return *new(bool), err
	}

	out0 := *abi.ConvertType(out[0], new(bool)).(*bool)

	return out0, err

}

// HasActiveDeal is a free data retrieval call binding the contract method 0x06d3e115.
//
// Solidity: function hasActiveDeal(bytes32 commpHash) view returns(bool)
func (_ContentRegistry *ContentRegistrySession) HasActiveDeal(commpHash [32]byte) (bool, error) {
	return _ContentRegistry.Contract.HasActiveDeal(&_ContentRegistry.CallOpts, commpHash)
}

// HasActiveDeal is a free data retrieval call binding the contract method 0x06d3e115.
//
// Solidity: function hasActiveDeal(bytes32 commpHash) view returns(bool)
func (_ContentRegistry *ContentRegistryCallerSession) HasActiveDeal(commpHash [32]byte) (bool, error) {
	return _ContentRegistry.Contract.HasActiveDeal(&_ContentRegistry.CallOpts, commpHash)
}

// Marketplace is a free data retrieval call binding the contract method 0xabc8c7af.
//
// Solidity: function marketplace() view returns(address)
func (_ContentRegistry *ContentRegistryCaller) Marketplace(opts *bind.CallOpts) (common.Address, error) {
	var out []interface{}
	err := _ContentRegistry.contract.Call(opts, &out, "marketplace")

	if err != nil {
		return *new(common.Address), err
	}

	out0 := *abi.ConvertType(out[0], new(common.Address)).(*common.Address)

	return out0, err

}

// Marketplace is a free data retrieval call binding the contract method 0xabc8c7af.
//
// Solidity: function marketplace() view returns(address)
func (_ContentRegistry *ContentRegistrySession) Marketplace() (common.Address, error) {
	return _ContentRegistry.Contract.Marketplace(&_ContentRegistry.CallOpts)
}

// Marketplace is a free data retrieval call binding the contract method 0xabc8c7af.
//
// Solidity: function marketplace() view returns(address)
func (_ContentRegistry *ContentRegistryCallerSession) Marketplace() (common.Address, error) {
	return _ContentRegistry.Contract.Marketplace(&_ContentRegistry.CallOpts)
}

// ResolveENS is a free data retrieval call binding the contract method 0xa92df3d3.
//
// Solidity: function resolveENS(bytes32 ensNode) view returns((address,uint256,uint64,uint64,uint64,bytes32))
func (_ContentRegistry *ContentRegistryCaller) ResolveENS(opts *bind.CallOpts, ensNode [32]byte) (ContentRegistryContent, error) {
	var out []interface{}
	err := _ContentRegistry.contract.Call(opts, &out, "resolveENS", ensNode)

	if err != nil {
		return *new(ContentRegistryContent), err
	}

	out0 := *abi.ConvertType(out[0], new(ContentRegistryContent)).(*ContentRegistryContent)

	return out0, err

}

// ResolveENS is a free data retrieval call binding the contract method 0xa92df3d3.
//
// Solidity: function resolveENS(bytes32 ensNode) view returns((address,uint256,uint64,uint64,uint64,bytes32))
func (_ContentRegistry *ContentRegistrySession) ResolveENS(ensNode [32]byte) (ContentRegistryContent, error) {
	return _ContentRegistry.Contract.ResolveENS(&_ContentRegistry.CallOpts, ensNode)
}

// ResolveENS is a free data retrieval call binding the contract method 0xa92df3d3.
//
// Solidity: function resolveENS(bytes32 ensNode) view returns((address,uint256,uint64,uint64,uint64,bytes32))
func (_ContentRegistry *ContentRegistryCallerSession) ResolveENS(ensNode [32]byte) (ContentRegistryContent, error) {
	return _ContentRegistry.Contract.ResolveENS(&_ContentRegistry.CallOpts, ensNode)
}

// BindENS is a paid mutator transaction binding the contract method 0x4349300a.
//
// Solidity: function bindENS(bytes32 commpHash, bytes32 ensNode) returns()
func (_ContentRegistry *ContentRegistryTransactor) BindENS(opts *bind.TransactOpts, commpHash [32]byte, ensNode [32]byte) (*types.Transaction, error) {
	return _ContentRegistry.contract.Transact(opts, "bindENS", commpHash, ensNode)
}

// BindENS is a paid mutator transaction binding the contract method 0x4349300a.
//
// Solidity: function bindENS(bytes32 commpHash, bytes32 ensNode) returns()
func (_ContentRegistry *ContentRegistrySession) BindENS(commpHash [32]byte, ensNode [32]byte) (*types.Transaction, error) {
	return _ContentRegistry.Contract.BindENS(&_ContentRegistry.TransactOpts, commpHash, ensNode)
}

// BindENS is a paid mutator transaction binding the contract method 0x4349300a.
//
// Solidity: function bindENS(bytes32 commpHash, bytes32 ensNode) returns()
func (_ContentRegistry *ContentRegistryTransactorSession) BindENS(commpHash [32]byte, ensNode [32]byte) (*types.Transaction, error) {
	return _ContentRegistry.Contract.BindENS(&_ContentRegistry.TransactOpts, commpHash, ensNode)
}

// ClearActiveDeal is a paid mutator transaction binding the contract method 0xc788132a.
//
// Solidity: function clearActiveDeal(bytes32 commpHash, uint256 expectedDealId) returns()
func (_ContentRegistry *ContentRegistryTransactor) ClearActiveDeal(opts *bind.TransactOpts, commpHash [32]byte, expectedDealId *big.Int) (*types.Transaction, error) {
	return _ContentRegistry.contract.Transact(opts, "clearActiveDeal", commpHash, expectedDealId)
}

// ClearActiveDeal is a paid mutator transaction binding the contract method 0xc788132a.
//
// Solidity: function clearActiveDeal(bytes32 commpHash, uint256 expectedDealId) returns()
func (_ContentRegistry *ContentRegistrySession) ClearActiveDeal(commpHash [32]byte, expectedDealId *big.Int) (*types.Transaction, error) {
	return _ContentRegistry.Contract.ClearActiveDeal(&_ContentRegistry.TransactOpts, commpHash, expectedDealId)
}

// ClearActiveDeal is a paid mutator transaction binding the contract method 0xc788132a.
//
// Solidity: function clearActiveDeal(bytes32 commpHash, uint256 expectedDealId) returns()
func (_ContentRegistry *ContentRegistryTransactorSession) ClearActiveDeal(commpHash [32]byte, expectedDealId *big.Int) (*types.Transaction, error) {
	return _ContentRegistry.Contract.ClearActiveDeal(&_ContentRegistry.TransactOpts, commpHash, expectedDealId)
}

// RegisterContent is a paid mutator transaction binding the contract method 0x92decf6f.
//
// Solidity: function registerContent(bytes32 commpHash, address owner, uint256 dealId, uint64 pieceSize) returns()
func (_ContentRegistry *ContentRegistryTransactor) RegisterContent(opts *bind.TransactOpts, commpHash [32]byte, owner common.Address, dealId *big.Int, pieceSize uint64) (*types.Transaction, error) {
	return _ContentRegistry.contract.Transact(opts, "registerContent", commpHash, owner, dealId, pieceSize)
}

// RegisterContent is a paid mutator transaction binding the contract method 0x92decf6f.
//
// Solidity: function registerContent(bytes32 commpHash, address owner, uint256 dealId, uint64 pieceSize) returns()
func (_ContentRegistry *ContentRegistrySession) RegisterContent(commpHash [32]byte, owner common.Address, dealId *big.Int, pieceSize uint64) (*types.Transaction, error) {
	return _ContentRegistry.Contract.RegisterContent(&_ContentRegistry.TransactOpts, commpHash, owner, dealId, pieceSize)
}

// RegisterContent is a paid mutator transaction binding the contract method 0x92decf6f.
//
// Solidity: function registerContent(bytes32 commpHash, address owner, uint256 dealId, uint64 pieceSize) returns()
func (_ContentRegistry *ContentRegistryTransactorSession) RegisterContent(commpHash [32]byte, owner common.Address, dealId *big.Int, pieceSize uint64) (*types.Transaction, error) {
	return _ContentRegistry.Contract.RegisterContent(&_ContentRegistry.TransactOpts, commpHash, owner, dealId, pieceSize)
}

// SetMarketplace is a paid mutator transaction binding the contract method 0x73ad6c2d.
//
// Solidity: function setMarketplace(address _marketplace) returns()
func (_ContentRegistry *ContentRegistryTransactor) SetMarketplace(opts *bind.TransactOpts, _marketplace common.Address) (*types.Transaction, error) {
	return _ContentRegistry.contract.Transact(opts, "setMarketplace", _marketplace)
}

// SetMarketplace is a paid mutator transaction binding the contract method 0x73ad6c2d.
//
// Solidity: function setMarketplace(address _marketplace) returns()
func (_ContentRegistry *ContentRegistrySession) SetMarketplace(_marketplace common.Address) (*types.Transaction, error) {
	return _ContentRegistry.Contract.SetMarketplace(&_ContentRegistry.TransactOpts, _marketplace)
}

// SetMarketplace is a paid mutator transaction binding the contract method 0x73ad6c2d.
//
// Solidity: function setMarketplace(address _marketplace) returns()
func (_ContentRegistry *ContentRegistryTransactorSession) SetMarketplace(_marketplace common.Address) (*types.Transaction, error) {
	return _ContentRegistry.Contract.SetMarketplace(&_ContentRegistry.TransactOpts, _marketplace)
}

// TransferAdmin is a paid mutator transaction binding the contract method 0x75829def.
//
// Solidity: function transferAdmin(address newAdmin) returns()
func (_ContentRegistry *ContentRegistryTransactor) TransferAdmin(opts *bind.TransactOpts, newAdmin common.Address) (*types.Transaction, error) {
	return _ContentRegistry.contract.Transact(opts, "transferAdmin", newAdmin)
}

// TransferAdmin is a paid mutator transaction binding the contract method 0x75829def.
//
// Solidity: function transferAdmin(address newAdmin) returns()
func (_ContentRegistry *ContentRegistrySession) TransferAdmin(newAdmin common.Address) (*types.Transaction, error) {
	return _ContentRegistry.Contract.TransferAdmin(&_ContentRegistry.TransactOpts, newAdmin)
}

// TransferAdmin is a paid mutator transaction binding the contract method 0x75829def.
//
// Solidity: function transferAdmin(address newAdmin) returns()
func (_ContentRegistry *ContentRegistryTransactorSession) TransferAdmin(newAdmin common.Address) (*types.Transaction, error) {
	return _ContentRegistry.Contract.TransferAdmin(&_ContentRegistry.TransactOpts, newAdmin)
}

// UnbindENS is a paid mutator transaction binding the contract method 0xb6fbdfa8.
//
// Solidity: function unbindENS(bytes32 commpHash) returns()
func (_ContentRegistry *ContentRegistryTransactor) UnbindENS(opts *bind.TransactOpts, commpHash [32]byte) (*types.Transaction, error) {
	return _ContentRegistry.contract.Transact(opts, "unbindENS", commpHash)
}

// UnbindENS is a paid mutator transaction binding the contract method 0xb6fbdfa8.
//
// Solidity: function unbindENS(bytes32 commpHash) returns()
func (_ContentRegistry *ContentRegistrySession) UnbindENS(commpHash [32]byte) (*types.Transaction, error) {
	return _ContentRegistry.Contract.UnbindENS(&_ContentRegistry.TransactOpts, commpHash)
}

// UnbindENS is a paid mutator transaction binding the contract method 0xb6fbdfa8.
//
// Solidity: function unbindENS(bytes32 commpHash) returns()
func (_ContentRegistry *ContentRegistryTransactorSession) UnbindENS(commpHash [32]byte) (*types.Transaction, error) {
	return _ContentRegistry.Contract.UnbindENS(&_ContentRegistry.TransactOpts, commpHash)
}

// ContentRegistryContentDealUpdatedIterator is returned from FilterContentDealUpdated and is used to iterate over the raw logs and unpacked data for ContentDealUpdated events raised by the ContentRegistry contract.
type ContentRegistryContentDealUpdatedIterator struct {
	Event *ContentRegistryContentDealUpdated // Event containing the contract specifics and raw log

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
func (it *ContentRegistryContentDealUpdatedIterator) Next() bool {
	// If the iterator failed, stop iterating
	if it.fail != nil {
		return false
	}
	// If the iterator completed, deliver directly whatever's available
	if it.done {
		select {
		case log := <-it.logs:
			it.Event = new(ContentRegistryContentDealUpdated)
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
		it.Event = new(ContentRegistryContentDealUpdated)
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
func (it *ContentRegistryContentDealUpdatedIterator) Error() error {
	return it.fail
}

// Close terminates the iteration process, releasing any pending underlying
// resources.
func (it *ContentRegistryContentDealUpdatedIterator) Close() error {
	it.sub.Unsubscribe()
	return nil
}

// ContentRegistryContentDealUpdated represents a ContentDealUpdated event raised by the ContentRegistry contract.
type ContentRegistryContentDealUpdated struct {
	CommpHash [32]byte
	OldDealId *big.Int
	NewDealId *big.Int
	Raw       types.Log // Blockchain specific contextual infos
}

// FilterContentDealUpdated is a free log retrieval operation binding the contract event 0x2d3f3ed8bebe96ed332e19ca2fb9a9793f64ca54196529bdb5b5d8c428c8676b.
//
// Solidity: event ContentDealUpdated(bytes32 indexed commpHash, uint256 oldDealId, uint256 newDealId)
func (_ContentRegistry *ContentRegistryFilterer) FilterContentDealUpdated(opts *bind.FilterOpts, commpHash [][32]byte) (*ContentRegistryContentDealUpdatedIterator, error) {

	var commpHashRule []interface{}
	for _, commpHashItem := range commpHash {
		commpHashRule = append(commpHashRule, commpHashItem)
	}

	logs, sub, err := _ContentRegistry.contract.FilterLogs(opts, "ContentDealUpdated", commpHashRule)
	if err != nil {
		return nil, err
	}
	return &ContentRegistryContentDealUpdatedIterator{contract: _ContentRegistry.contract, event: "ContentDealUpdated", logs: logs, sub: sub}, nil
}

// WatchContentDealUpdated is a free log subscription operation binding the contract event 0x2d3f3ed8bebe96ed332e19ca2fb9a9793f64ca54196529bdb5b5d8c428c8676b.
//
// Solidity: event ContentDealUpdated(bytes32 indexed commpHash, uint256 oldDealId, uint256 newDealId)
func (_ContentRegistry *ContentRegistryFilterer) WatchContentDealUpdated(opts *bind.WatchOpts, sink chan<- *ContentRegistryContentDealUpdated, commpHash [][32]byte) (event.Subscription, error) {

	var commpHashRule []interface{}
	for _, commpHashItem := range commpHash {
		commpHashRule = append(commpHashRule, commpHashItem)
	}

	logs, sub, err := _ContentRegistry.contract.WatchLogs(opts, "ContentDealUpdated", commpHashRule)
	if err != nil {
		return nil, err
	}
	return event.NewSubscription(func(quit <-chan struct{}) error {
		defer sub.Unsubscribe()
		for {
			select {
			case log := <-logs:
				// New log arrived, parse the event and forward to the user
				event := new(ContentRegistryContentDealUpdated)
				if err := _ContentRegistry.contract.UnpackLog(event, "ContentDealUpdated", log); err != nil {
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

// ParseContentDealUpdated is a log parse operation binding the contract event 0x2d3f3ed8bebe96ed332e19ca2fb9a9793f64ca54196529bdb5b5d8c428c8676b.
//
// Solidity: event ContentDealUpdated(bytes32 indexed commpHash, uint256 oldDealId, uint256 newDealId)
func (_ContentRegistry *ContentRegistryFilterer) ParseContentDealUpdated(log types.Log) (*ContentRegistryContentDealUpdated, error) {
	event := new(ContentRegistryContentDealUpdated)
	if err := _ContentRegistry.contract.UnpackLog(event, "ContentDealUpdated", log); err != nil {
		return nil, err
	}
	event.Raw = log
	return event, nil
}

// ContentRegistryContentRegisteredIterator is returned from FilterContentRegistered and is used to iterate over the raw logs and unpacked data for ContentRegistered events raised by the ContentRegistry contract.
type ContentRegistryContentRegisteredIterator struct {
	Event *ContentRegistryContentRegistered // Event containing the contract specifics and raw log

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
func (it *ContentRegistryContentRegisteredIterator) Next() bool {
	// If the iterator failed, stop iterating
	if it.fail != nil {
		return false
	}
	// If the iterator completed, deliver directly whatever's available
	if it.done {
		select {
		case log := <-it.logs:
			it.Event = new(ContentRegistryContentRegistered)
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
		it.Event = new(ContentRegistryContentRegistered)
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
func (it *ContentRegistryContentRegisteredIterator) Error() error {
	return it.fail
}

// Close terminates the iteration process, releasing any pending underlying
// resources.
func (it *ContentRegistryContentRegisteredIterator) Close() error {
	it.sub.Unsubscribe()
	return nil
}

// ContentRegistryContentRegistered represents a ContentRegistered event raised by the ContentRegistry contract.
type ContentRegistryContentRegistered struct {
	CommpHash [32]byte
	Owner     common.Address
	DealId    *big.Int
	PieceSize uint64
	Raw       types.Log // Blockchain specific contextual infos
}

// FilterContentRegistered is a free log retrieval operation binding the contract event 0xd1814539fff4d0a0cbe33001338582c1dc3e5eea8b734484cd8daa48a1b05b50.
//
// Solidity: event ContentRegistered(bytes32 indexed commpHash, address indexed owner, uint256 indexed dealId, uint64 pieceSize)
func (_ContentRegistry *ContentRegistryFilterer) FilterContentRegistered(opts *bind.FilterOpts, commpHash [][32]byte, owner []common.Address, dealId []*big.Int) (*ContentRegistryContentRegisteredIterator, error) {

	var commpHashRule []interface{}
	for _, commpHashItem := range commpHash {
		commpHashRule = append(commpHashRule, commpHashItem)
	}
	var ownerRule []interface{}
	for _, ownerItem := range owner {
		ownerRule = append(ownerRule, ownerItem)
	}
	var dealIdRule []interface{}
	for _, dealIdItem := range dealId {
		dealIdRule = append(dealIdRule, dealIdItem)
	}

	logs, sub, err := _ContentRegistry.contract.FilterLogs(opts, "ContentRegistered", commpHashRule, ownerRule, dealIdRule)
	if err != nil {
		return nil, err
	}
	return &ContentRegistryContentRegisteredIterator{contract: _ContentRegistry.contract, event: "ContentRegistered", logs: logs, sub: sub}, nil
}

// WatchContentRegistered is a free log subscription operation binding the contract event 0xd1814539fff4d0a0cbe33001338582c1dc3e5eea8b734484cd8daa48a1b05b50.
//
// Solidity: event ContentRegistered(bytes32 indexed commpHash, address indexed owner, uint256 indexed dealId, uint64 pieceSize)
func (_ContentRegistry *ContentRegistryFilterer) WatchContentRegistered(opts *bind.WatchOpts, sink chan<- *ContentRegistryContentRegistered, commpHash [][32]byte, owner []common.Address, dealId []*big.Int) (event.Subscription, error) {

	var commpHashRule []interface{}
	for _, commpHashItem := range commpHash {
		commpHashRule = append(commpHashRule, commpHashItem)
	}
	var ownerRule []interface{}
	for _, ownerItem := range owner {
		ownerRule = append(ownerRule, ownerItem)
	}
	var dealIdRule []interface{}
	for _, dealIdItem := range dealId {
		dealIdRule = append(dealIdRule, dealIdItem)
	}

	logs, sub, err := _ContentRegistry.contract.WatchLogs(opts, "ContentRegistered", commpHashRule, ownerRule, dealIdRule)
	if err != nil {
		return nil, err
	}
	return event.NewSubscription(func(quit <-chan struct{}) error {
		defer sub.Unsubscribe()
		for {
			select {
			case log := <-logs:
				// New log arrived, parse the event and forward to the user
				event := new(ContentRegistryContentRegistered)
				if err := _ContentRegistry.contract.UnpackLog(event, "ContentRegistered", log); err != nil {
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

// ParseContentRegistered is a log parse operation binding the contract event 0xd1814539fff4d0a0cbe33001338582c1dc3e5eea8b734484cd8daa48a1b05b50.
//
// Solidity: event ContentRegistered(bytes32 indexed commpHash, address indexed owner, uint256 indexed dealId, uint64 pieceSize)
func (_ContentRegistry *ContentRegistryFilterer) ParseContentRegistered(log types.Log) (*ContentRegistryContentRegistered, error) {
	event := new(ContentRegistryContentRegistered)
	if err := _ContentRegistry.contract.UnpackLog(event, "ContentRegistered", log); err != nil {
		return nil, err
	}
	event.Raw = log
	return event, nil
}

// ContentRegistryENSBoundIterator is returned from FilterENSBound and is used to iterate over the raw logs and unpacked data for ENSBound events raised by the ContentRegistry contract.
type ContentRegistryENSBoundIterator struct {
	Event *ContentRegistryENSBound // Event containing the contract specifics and raw log

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
func (it *ContentRegistryENSBoundIterator) Next() bool {
	// If the iterator failed, stop iterating
	if it.fail != nil {
		return false
	}
	// If the iterator completed, deliver directly whatever's available
	if it.done {
		select {
		case log := <-it.logs:
			it.Event = new(ContentRegistryENSBound)
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
		it.Event = new(ContentRegistryENSBound)
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
func (it *ContentRegistryENSBoundIterator) Error() error {
	return it.fail
}

// Close terminates the iteration process, releasing any pending underlying
// resources.
func (it *ContentRegistryENSBoundIterator) Close() error {
	it.sub.Unsubscribe()
	return nil
}

// ContentRegistryENSBound represents a ENSBound event raised by the ContentRegistry contract.
type ContentRegistryENSBound struct {
	CommpHash [32]byte
	EnsNode   [32]byte
	By        common.Address
	Raw       types.Log // Blockchain specific contextual infos
}

// FilterENSBound is a free log retrieval operation binding the contract event 0xf7f84a553ca03faa4c3fcd10b5c62bcf67b1e55fc52fed74344b699cfd2bb1e4.
//
// Solidity: event ENSBound(bytes32 indexed commpHash, bytes32 indexed ensNode, address indexed by)
func (_ContentRegistry *ContentRegistryFilterer) FilterENSBound(opts *bind.FilterOpts, commpHash [][32]byte, ensNode [][32]byte, by []common.Address) (*ContentRegistryENSBoundIterator, error) {

	var commpHashRule []interface{}
	for _, commpHashItem := range commpHash {
		commpHashRule = append(commpHashRule, commpHashItem)
	}
	var ensNodeRule []interface{}
	for _, ensNodeItem := range ensNode {
		ensNodeRule = append(ensNodeRule, ensNodeItem)
	}
	var byRule []interface{}
	for _, byItem := range by {
		byRule = append(byRule, byItem)
	}

	logs, sub, err := _ContentRegistry.contract.FilterLogs(opts, "ENSBound", commpHashRule, ensNodeRule, byRule)
	if err != nil {
		return nil, err
	}
	return &ContentRegistryENSBoundIterator{contract: _ContentRegistry.contract, event: "ENSBound", logs: logs, sub: sub}, nil
}

// WatchENSBound is a free log subscription operation binding the contract event 0xf7f84a553ca03faa4c3fcd10b5c62bcf67b1e55fc52fed74344b699cfd2bb1e4.
//
// Solidity: event ENSBound(bytes32 indexed commpHash, bytes32 indexed ensNode, address indexed by)
func (_ContentRegistry *ContentRegistryFilterer) WatchENSBound(opts *bind.WatchOpts, sink chan<- *ContentRegistryENSBound, commpHash [][32]byte, ensNode [][32]byte, by []common.Address) (event.Subscription, error) {

	var commpHashRule []interface{}
	for _, commpHashItem := range commpHash {
		commpHashRule = append(commpHashRule, commpHashItem)
	}
	var ensNodeRule []interface{}
	for _, ensNodeItem := range ensNode {
		ensNodeRule = append(ensNodeRule, ensNodeItem)
	}
	var byRule []interface{}
	for _, byItem := range by {
		byRule = append(byRule, byItem)
	}

	logs, sub, err := _ContentRegistry.contract.WatchLogs(opts, "ENSBound", commpHashRule, ensNodeRule, byRule)
	if err != nil {
		return nil, err
	}
	return event.NewSubscription(func(quit <-chan struct{}) error {
		defer sub.Unsubscribe()
		for {
			select {
			case log := <-logs:
				// New log arrived, parse the event and forward to the user
				event := new(ContentRegistryENSBound)
				if err := _ContentRegistry.contract.UnpackLog(event, "ENSBound", log); err != nil {
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

// ParseENSBound is a log parse operation binding the contract event 0xf7f84a553ca03faa4c3fcd10b5c62bcf67b1e55fc52fed74344b699cfd2bb1e4.
//
// Solidity: event ENSBound(bytes32 indexed commpHash, bytes32 indexed ensNode, address indexed by)
func (_ContentRegistry *ContentRegistryFilterer) ParseENSBound(log types.Log) (*ContentRegistryENSBound, error) {
	event := new(ContentRegistryENSBound)
	if err := _ContentRegistry.contract.UnpackLog(event, "ENSBound", log); err != nil {
		return nil, err
	}
	event.Raw = log
	return event, nil
}

// ContentRegistryENSUnboundIterator is returned from FilterENSUnbound and is used to iterate over the raw logs and unpacked data for ENSUnbound events raised by the ContentRegistry contract.
type ContentRegistryENSUnboundIterator struct {
	Event *ContentRegistryENSUnbound // Event containing the contract specifics and raw log

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
func (it *ContentRegistryENSUnboundIterator) Next() bool {
	// If the iterator failed, stop iterating
	if it.fail != nil {
		return false
	}
	// If the iterator completed, deliver directly whatever's available
	if it.done {
		select {
		case log := <-it.logs:
			it.Event = new(ContentRegistryENSUnbound)
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
		it.Event = new(ContentRegistryENSUnbound)
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
func (it *ContentRegistryENSUnboundIterator) Error() error {
	return it.fail
}

// Close terminates the iteration process, releasing any pending underlying
// resources.
func (it *ContentRegistryENSUnboundIterator) Close() error {
	it.sub.Unsubscribe()
	return nil
}

// ContentRegistryENSUnbound represents a ENSUnbound event raised by the ContentRegistry contract.
type ContentRegistryENSUnbound struct {
	CommpHash [32]byte
	EnsNode   [32]byte
	Raw       types.Log // Blockchain specific contextual infos
}

// FilterENSUnbound is a free log retrieval operation binding the contract event 0xbc6171f1c4547d68d95e787256d2a4d44cfb690e90d35cf07721d2fbb341d69b.
//
// Solidity: event ENSUnbound(bytes32 indexed commpHash, bytes32 indexed ensNode)
func (_ContentRegistry *ContentRegistryFilterer) FilterENSUnbound(opts *bind.FilterOpts, commpHash [][32]byte, ensNode [][32]byte) (*ContentRegistryENSUnboundIterator, error) {

	var commpHashRule []interface{}
	for _, commpHashItem := range commpHash {
		commpHashRule = append(commpHashRule, commpHashItem)
	}
	var ensNodeRule []interface{}
	for _, ensNodeItem := range ensNode {
		ensNodeRule = append(ensNodeRule, ensNodeItem)
	}

	logs, sub, err := _ContentRegistry.contract.FilterLogs(opts, "ENSUnbound", commpHashRule, ensNodeRule)
	if err != nil {
		return nil, err
	}
	return &ContentRegistryENSUnboundIterator{contract: _ContentRegistry.contract, event: "ENSUnbound", logs: logs, sub: sub}, nil
}

// WatchENSUnbound is a free log subscription operation binding the contract event 0xbc6171f1c4547d68d95e787256d2a4d44cfb690e90d35cf07721d2fbb341d69b.
//
// Solidity: event ENSUnbound(bytes32 indexed commpHash, bytes32 indexed ensNode)
func (_ContentRegistry *ContentRegistryFilterer) WatchENSUnbound(opts *bind.WatchOpts, sink chan<- *ContentRegistryENSUnbound, commpHash [][32]byte, ensNode [][32]byte) (event.Subscription, error) {

	var commpHashRule []interface{}
	for _, commpHashItem := range commpHash {
		commpHashRule = append(commpHashRule, commpHashItem)
	}
	var ensNodeRule []interface{}
	for _, ensNodeItem := range ensNode {
		ensNodeRule = append(ensNodeRule, ensNodeItem)
	}

	logs, sub, err := _ContentRegistry.contract.WatchLogs(opts, "ENSUnbound", commpHashRule, ensNodeRule)
	if err != nil {
		return nil, err
	}
	return event.NewSubscription(func(quit <-chan struct{}) error {
		defer sub.Unsubscribe()
		for {
			select {
			case log := <-logs:
				// New log arrived, parse the event and forward to the user
				event := new(ContentRegistryENSUnbound)
				if err := _ContentRegistry.contract.UnpackLog(event, "ENSUnbound", log); err != nil {
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

// ParseENSUnbound is a log parse operation binding the contract event 0xbc6171f1c4547d68d95e787256d2a4d44cfb690e90d35cf07721d2fbb341d69b.
//
// Solidity: event ENSUnbound(bytes32 indexed commpHash, bytes32 indexed ensNode)
func (_ContentRegistry *ContentRegistryFilterer) ParseENSUnbound(log types.Log) (*ContentRegistryENSUnbound, error) {
	event := new(ContentRegistryENSUnbound)
	if err := _ContentRegistry.contract.UnpackLog(event, "ENSUnbound", log); err != nil {
		return nil, err
	}
	event.Raw = log
	return event, nil
}

// ContentRegistryMarketplaceSetIterator is returned from FilterMarketplaceSet and is used to iterate over the raw logs and unpacked data for MarketplaceSet events raised by the ContentRegistry contract.
type ContentRegistryMarketplaceSetIterator struct {
	Event *ContentRegistryMarketplaceSet // Event containing the contract specifics and raw log

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
func (it *ContentRegistryMarketplaceSetIterator) Next() bool {
	// If the iterator failed, stop iterating
	if it.fail != nil {
		return false
	}
	// If the iterator completed, deliver directly whatever's available
	if it.done {
		select {
		case log := <-it.logs:
			it.Event = new(ContentRegistryMarketplaceSet)
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
		it.Event = new(ContentRegistryMarketplaceSet)
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
func (it *ContentRegistryMarketplaceSetIterator) Error() error {
	return it.fail
}

// Close terminates the iteration process, releasing any pending underlying
// resources.
func (it *ContentRegistryMarketplaceSetIterator) Close() error {
	it.sub.Unsubscribe()
	return nil
}

// ContentRegistryMarketplaceSet represents a MarketplaceSet event raised by the ContentRegistry contract.
type ContentRegistryMarketplaceSet struct {
	OldMarketplace common.Address
	NewMarketplace common.Address
	Raw            types.Log // Blockchain specific contextual infos
}

// FilterMarketplaceSet is a free log retrieval operation binding the contract event 0xe928dbb5d9a59bebb2adca04fb75586d6ad3c04cbf312f3b9ea24086c4b93a06.
//
// Solidity: event MarketplaceSet(address indexed oldMarketplace, address indexed newMarketplace)
func (_ContentRegistry *ContentRegistryFilterer) FilterMarketplaceSet(opts *bind.FilterOpts, oldMarketplace []common.Address, newMarketplace []common.Address) (*ContentRegistryMarketplaceSetIterator, error) {

	var oldMarketplaceRule []interface{}
	for _, oldMarketplaceItem := range oldMarketplace {
		oldMarketplaceRule = append(oldMarketplaceRule, oldMarketplaceItem)
	}
	var newMarketplaceRule []interface{}
	for _, newMarketplaceItem := range newMarketplace {
		newMarketplaceRule = append(newMarketplaceRule, newMarketplaceItem)
	}

	logs, sub, err := _ContentRegistry.contract.FilterLogs(opts, "MarketplaceSet", oldMarketplaceRule, newMarketplaceRule)
	if err != nil {
		return nil, err
	}
	return &ContentRegistryMarketplaceSetIterator{contract: _ContentRegistry.contract, event: "MarketplaceSet", logs: logs, sub: sub}, nil
}

// WatchMarketplaceSet is a free log subscription operation binding the contract event 0xe928dbb5d9a59bebb2adca04fb75586d6ad3c04cbf312f3b9ea24086c4b93a06.
//
// Solidity: event MarketplaceSet(address indexed oldMarketplace, address indexed newMarketplace)
func (_ContentRegistry *ContentRegistryFilterer) WatchMarketplaceSet(opts *bind.WatchOpts, sink chan<- *ContentRegistryMarketplaceSet, oldMarketplace []common.Address, newMarketplace []common.Address) (event.Subscription, error) {

	var oldMarketplaceRule []interface{}
	for _, oldMarketplaceItem := range oldMarketplace {
		oldMarketplaceRule = append(oldMarketplaceRule, oldMarketplaceItem)
	}
	var newMarketplaceRule []interface{}
	for _, newMarketplaceItem := range newMarketplace {
		newMarketplaceRule = append(newMarketplaceRule, newMarketplaceItem)
	}

	logs, sub, err := _ContentRegistry.contract.WatchLogs(opts, "MarketplaceSet", oldMarketplaceRule, newMarketplaceRule)
	if err != nil {
		return nil, err
	}
	return event.NewSubscription(func(quit <-chan struct{}) error {
		defer sub.Unsubscribe()
		for {
			select {
			case log := <-logs:
				// New log arrived, parse the event and forward to the user
				event := new(ContentRegistryMarketplaceSet)
				if err := _ContentRegistry.contract.UnpackLog(event, "MarketplaceSet", log); err != nil {
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

// ParseMarketplaceSet is a log parse operation binding the contract event 0xe928dbb5d9a59bebb2adca04fb75586d6ad3c04cbf312f3b9ea24086c4b93a06.
//
// Solidity: event MarketplaceSet(address indexed oldMarketplace, address indexed newMarketplace)
func (_ContentRegistry *ContentRegistryFilterer) ParseMarketplaceSet(log types.Log) (*ContentRegistryMarketplaceSet, error) {
	event := new(ContentRegistryMarketplaceSet)
	if err := _ContentRegistry.contract.UnpackLog(event, "MarketplaceSet", log); err != nil {
		return nil, err
	}
	event.Raw = log
	return event, nil
}
