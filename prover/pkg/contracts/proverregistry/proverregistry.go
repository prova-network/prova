// SPDX-License-Identifier: MIT
// Generated from contracts/out/ProverRegistry.sol/ProverRegistry.json via abigen.
// Do not edit by hand; run ./scripts/gen-bindings.sh instead.

// Code generated - DO NOT EDIT.
// This file is a generated binding and any manual changes will be lost.

package proverregistry

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

// ProverRegistryProver is an auto generated low-level Go binding around an user-defined struct.
type ProverRegistryProver struct {
	Owner              common.Address
	Endpoint           string
	Features           uint64
	PricePerGibDay     *big.Int
	PricePerByteServed *big.Int
	RegisteredAt       uint64
	UpdatedAt          uint64
	Active             bool
	EnsNode            [32]byte
	Metadata           string
}

// ProverRegistryMetaData contains all meta data concerning the ProverRegistry contract.
var ProverRegistryMetaData = &bind.MetaData{
	ABI: "[{\"type\":\"constructor\",\"inputs\":[],\"stateMutability\":\"nonpayable\"},{\"type\":\"function\",\"name\":\"FEATURE_HTTPS_SERVING\",\"inputs\":[],\"outputs\":[{\"name\":\"\",\"type\":\"uint64\",\"internalType\":\"uint64\"}],\"stateMutability\":\"view\"},{\"type\":\"function\",\"name\":\"FEATURE_PDP\",\"inputs\":[],\"outputs\":[{\"name\":\"\",\"type\":\"uint64\",\"internalType\":\"uint64\"}],\"stateMutability\":\"view\"},{\"type\":\"function\",\"name\":\"FEATURE_QBP\",\"inputs\":[],\"outputs\":[{\"name\":\"\",\"type\":\"uint64\",\"internalType\":\"uint64\"}],\"stateMutability\":\"view\"},{\"type\":\"function\",\"name\":\"FEATURE_TEE\",\"inputs\":[],\"outputs\":[{\"name\":\"\",\"type\":\"uint64\",\"internalType\":\"uint64\"}],\"stateMutability\":\"view\"},{\"type\":\"function\",\"name\":\"MAX_ENDPOINT_LENGTH\",\"inputs\":[],\"outputs\":[{\"name\":\"\",\"type\":\"uint256\",\"internalType\":\"uint256\"}],\"stateMutability\":\"view\"},{\"type\":\"function\",\"name\":\"MAX_METADATA_LENGTH\",\"inputs\":[],\"outputs\":[{\"name\":\"\",\"type\":\"uint256\",\"internalType\":\"uint256\"}],\"stateMutability\":\"view\"},{\"type\":\"function\",\"name\":\"bindENS\",\"inputs\":[{\"name\":\"ensNode\",\"type\":\"bytes32\",\"internalType\":\"bytes32\"}],\"outputs\":[],\"stateMutability\":\"nonpayable\"},{\"type\":\"function\",\"name\":\"deregister\",\"inputs\":[],\"outputs\":[],\"stateMutability\":\"nonpayable\"},{\"type\":\"function\",\"name\":\"getProver\",\"inputs\":[{\"name\":\"prover\",\"type\":\"address\",\"internalType\":\"address\"}],\"outputs\":[{\"name\":\"\",\"type\":\"tuple\",\"internalType\":\"structProverRegistry.Prover\",\"components\":[{\"name\":\"owner\",\"type\":\"address\",\"internalType\":\"address\"},{\"name\":\"endpoint\",\"type\":\"string\",\"internalType\":\"string\"},{\"name\":\"features\",\"type\":\"uint64\",\"internalType\":\"uint64\"},{\"name\":\"pricePerGibDay\",\"type\":\"uint128\",\"internalType\":\"uint128\"},{\"name\":\"pricePerByteServed\",\"type\":\"uint128\",\"internalType\":\"uint128\"},{\"name\":\"registeredAt\",\"type\":\"uint64\",\"internalType\":\"uint64\"},{\"name\":\"updatedAt\",\"type\":\"uint64\",\"internalType\":\"uint64\"},{\"name\":\"active\",\"type\":\"bool\",\"internalType\":\"bool\"},{\"name\":\"ensNode\",\"type\":\"bytes32\",\"internalType\":\"bytes32\"},{\"name\":\"metadata\",\"type\":\"string\",\"internalType\":\"string\"}]}],\"stateMutability\":\"view\"},{\"type\":\"function\",\"name\":\"isActive\",\"inputs\":[{\"name\":\"prover\",\"type\":\"address\",\"internalType\":\"address\"}],\"outputs\":[{\"name\":\"\",\"type\":\"bool\",\"internalType\":\"bool\"}],\"stateMutability\":\"view\"},{\"type\":\"function\",\"name\":\"known\",\"inputs\":[{\"name\":\"\",\"type\":\"address\",\"internalType\":\"address\"}],\"outputs\":[{\"name\":\"\",\"type\":\"bool\",\"internalType\":\"bool\"}],\"stateMutability\":\"view\"},{\"type\":\"function\",\"name\":\"listActive\",\"inputs\":[{\"name\":\"offset\",\"type\":\"uint256\",\"internalType\":\"uint256\"},{\"name\":\"limit\",\"type\":\"uint256\",\"internalType\":\"uint256\"}],\"outputs\":[{\"name\":\"result\",\"type\":\"address[]\",\"internalType\":\"address[]\"},{\"name\":\"nextOffset\",\"type\":\"uint256\",\"internalType\":\"uint256\"}],\"stateMutability\":\"view\"},{\"type\":\"function\",\"name\":\"owner\",\"inputs\":[],\"outputs\":[{\"name\":\"\",\"type\":\"address\",\"internalType\":\"address\"}],\"stateMutability\":\"view\"},{\"type\":\"function\",\"name\":\"proverAddresses\",\"inputs\":[{\"name\":\"\",\"type\":\"uint256\",\"internalType\":\"uint256\"}],\"outputs\":[{\"name\":\"\",\"type\":\"address\",\"internalType\":\"address\"}],\"stateMutability\":\"view\"},{\"type\":\"function\",\"name\":\"provers\",\"inputs\":[{\"name\":\"\",\"type\":\"address\",\"internalType\":\"address\"}],\"outputs\":[{\"name\":\"owner\",\"type\":\"address\",\"internalType\":\"address\"},{\"name\":\"endpoint\",\"type\":\"string\",\"internalType\":\"string\"},{\"name\":\"features\",\"type\":\"uint64\",\"internalType\":\"uint64\"},{\"name\":\"pricePerGibDay\",\"type\":\"uint128\",\"internalType\":\"uint128\"},{\"name\":\"pricePerByteServed\",\"type\":\"uint128\",\"internalType\":\"uint128\"},{\"name\":\"registeredAt\",\"type\":\"uint64\",\"internalType\":\"uint64\"},{\"name\":\"updatedAt\",\"type\":\"uint64\",\"internalType\":\"uint64\"},{\"name\":\"active\",\"type\":\"bool\",\"internalType\":\"bool\"},{\"name\":\"ensNode\",\"type\":\"bytes32\",\"internalType\":\"bytes32\"},{\"name\":\"metadata\",\"type\":\"string\",\"internalType\":\"string\"}],\"stateMutability\":\"view\"},{\"type\":\"function\",\"name\":\"register\",\"inputs\":[{\"name\":\"endpoint\",\"type\":\"string\",\"internalType\":\"string\"},{\"name\":\"features\",\"type\":\"uint64\",\"internalType\":\"uint64\"},{\"name\":\"pricePerGibDay\",\"type\":\"uint128\",\"internalType\":\"uint128\"},{\"name\":\"pricePerByteServed\",\"type\":\"uint128\",\"internalType\":\"uint128\"},{\"name\":\"metadata\",\"type\":\"string\",\"internalType\":\"string\"}],\"outputs\":[],\"stateMutability\":\"nonpayable\"},{\"type\":\"function\",\"name\":\"renounceOwnership\",\"inputs\":[],\"outputs\":[],\"stateMutability\":\"nonpayable\"},{\"type\":\"function\",\"name\":\"setPrice\",\"inputs\":[{\"name\":\"pricePerGibDay\",\"type\":\"uint128\",\"internalType\":\"uint128\"},{\"name\":\"pricePerByteServed\",\"type\":\"uint128\",\"internalType\":\"uint128\"}],\"outputs\":[],\"stateMutability\":\"nonpayable\"},{\"type\":\"function\",\"name\":\"supportsFeature\",\"inputs\":[{\"name\":\"prover\",\"type\":\"address\",\"internalType\":\"address\"},{\"name\":\"feature\",\"type\":\"uint64\",\"internalType\":\"uint64\"}],\"outputs\":[{\"name\":\"\",\"type\":\"bool\",\"internalType\":\"bool\"}],\"stateMutability\":\"view\"},{\"type\":\"function\",\"name\":\"totalRegistered\",\"inputs\":[],\"outputs\":[{\"name\":\"\",\"type\":\"uint256\",\"internalType\":\"uint256\"}],\"stateMutability\":\"view\"},{\"type\":\"function\",\"name\":\"transferOwnership\",\"inputs\":[{\"name\":\"newOwner\",\"type\":\"address\",\"internalType\":\"address\"}],\"outputs\":[],\"stateMutability\":\"nonpayable\"},{\"type\":\"function\",\"name\":\"updateEndpoint\",\"inputs\":[{\"name\":\"endpoint\",\"type\":\"string\",\"internalType\":\"string\"},{\"name\":\"features\",\"type\":\"uint64\",\"internalType\":\"uint64\"},{\"name\":\"metadata\",\"type\":\"string\",\"internalType\":\"string\"}],\"outputs\":[],\"stateMutability\":\"nonpayable\"},{\"type\":\"event\",\"name\":\"ENSBound\",\"inputs\":[{\"name\":\"prover\",\"type\":\"address\",\"indexed\":true,\"internalType\":\"address\"},{\"name\":\"ensNode\",\"type\":\"bytes32\",\"indexed\":false,\"internalType\":\"bytes32\"}],\"anonymous\":false},{\"type\":\"event\",\"name\":\"OwnershipTransferred\",\"inputs\":[{\"name\":\"previousOwner\",\"type\":\"address\",\"indexed\":true,\"internalType\":\"address\"},{\"name\":\"newOwner\",\"type\":\"address\",\"indexed\":true,\"internalType\":\"address\"}],\"anonymous\":false},{\"type\":\"event\",\"name\":\"PriceChanged\",\"inputs\":[{\"name\":\"prover\",\"type\":\"address\",\"indexed\":true,\"internalType\":\"address\"},{\"name\":\"pricePerGibDay\",\"type\":\"uint128\",\"indexed\":false,\"internalType\":\"uint128\"},{\"name\":\"pricePerByteServed\",\"type\":\"uint128\",\"indexed\":false,\"internalType\":\"uint128\"}],\"anonymous\":false},{\"type\":\"event\",\"name\":\"ProverDeregistered\",\"inputs\":[{\"name\":\"prover\",\"type\":\"address\",\"indexed\":true,\"internalType\":\"address\"}],\"anonymous\":false},{\"type\":\"event\",\"name\":\"ProverRegistered\",\"inputs\":[{\"name\":\"prover\",\"type\":\"address\",\"indexed\":true,\"internalType\":\"address\"},{\"name\":\"endpoint\",\"type\":\"string\",\"indexed\":false,\"internalType\":\"string\"},{\"name\":\"features\",\"type\":\"uint64\",\"indexed\":false,\"internalType\":\"uint64\"}],\"anonymous\":false},{\"type\":\"event\",\"name\":\"ProverUpdated\",\"inputs\":[{\"name\":\"prover\",\"type\":\"address\",\"indexed\":true,\"internalType\":\"address\"},{\"name\":\"endpoint\",\"type\":\"string\",\"indexed\":false,\"internalType\":\"string\"},{\"name\":\"features\",\"type\":\"uint64\",\"indexed\":false,\"internalType\":\"uint64\"}],\"anonymous\":false},{\"type\":\"error\",\"name\":\"AlreadyRegistered\",\"inputs\":[]},{\"type\":\"error\",\"name\":\"EndpointTooLong\",\"inputs\":[]},{\"type\":\"error\",\"name\":\"InvalidFeatures\",\"inputs\":[]},{\"type\":\"error\",\"name\":\"MetadataTooLong\",\"inputs\":[]},{\"type\":\"error\",\"name\":\"NotOwner\",\"inputs\":[]},{\"type\":\"error\",\"name\":\"NotRegistered\",\"inputs\":[]},{\"type\":\"error\",\"name\":\"OwnableInvalidOwner\",\"inputs\":[{\"name\":\"owner\",\"type\":\"address\",\"internalType\":\"address\"}]},{\"type\":\"error\",\"name\":\"OwnableUnauthorizedAccount\",\"inputs\":[{\"name\":\"account\",\"type\":\"address\",\"internalType\":\"address\"}]}]",
	Bin: "0x608080604052346071573315605e575f8054336001600160a01b0319821681178355916001600160a01b03909116907f8be0079c531659141344cd1fd0a4f28419497f9722a3daafe3b4186f6b6457e09080a36114d190816100768239f35b631e4fbdf760e01b5f525f60045260245ffd5b5f80fdfe6080806040526004361015610012575f80fd5b5f3560e01c9081631200c7f514610f88575080631dcb0bd914610f6d5780631dec844b14610e8157806322dfc94414610e4457806343eb4d9414610a0f5780634546356c1461095257806349d2982b146108dd578063715018a61461089957806384691fab146108205780638da5cb5b146107f9578063927416c0146107dc57806399105669146107c15780639f8a13d714610781578063a91a97021461070e578063ad50b8a014610461578063aff5edb1146103de578063c85d3eb0146103c2578063d2c7f2ac14610380578063e8868e9f14610364578063e95aff8214610349578063f2fde38b146102d75763faab193c1461010e575f80fd5b346102d35760203660031901126102d357610127610fa1565b606061012060405161013881610fef565b5f81528260208201525f60408201525f838201525f60808201525f60a08201525f60c08201525f60e08201525f610100820152015260018060a01b03165f52600160205260405f2060405161018c81610fef565b81546001600160a01b031681526102cf6101a86001840161102e565b92602083019384526002810154604084019160018060401b0382168352606085019160018060801b039060401c1682526003810154608086019060018060801b038116825260a0870160018060401b038260801c16815260c088019160c01c825260ff6004850154169260e08901931515845261023460066005870154966101008c019788520161102e565b956101208a0196875261026d6040519b8c9b60208d5260018060a01b0390511660208d01525161014060408d01526101608c01906110ce565b97516001600160401b0390811660608c015290516001600160801b0390811660808c0152915190911660a08a01529051811660c089015290511660e08701525115156101008601525161012085015251838203601f19016101408501526110ce565b0390f35b5f80fd5b346102d35760203660031901126102d3576102f0610fa1565b6102f8611415565b6001600160a01b03168015610336575f80546001600160a01b03198116831782556001600160a01b0316905f51602061147c5f395f51905f529080a3005b631e4fbdf760e01b5f525f60045260245ffd5b346102d3575f3660031901126102d357602060405160028152f35b346102d3575f3660031901126102d35760206040516108008152f35b346102d35760203660031901126102d3576004356002548110156102d3576103a9602091611135565b905460405160039290921b1c6001600160a01b03168152f35b346102d3575f3660031901126102d35760206040516102008152f35b346102d3575f3660031901126102d357335f52600160205260405f20600481019081549160ff8316156104525760ff1990921690915561042b90426001600160401b031690600301611230565b337fba8692c6ccbc20602c00b106b981eb4c6e6b0957d070a30814d626956dba28c75f80a2005b63aba4733960e01b5f5260045ffd5b346102d35760603660031901126102d3576004356001600160401b0381116102d3576104919036906004016110f2565b9061049a61111f565b6044356001600160401b0381116102d3576104b99036906004016110f2565b9390335f52600160205260405f209460ff600487015416156104525761020083116106ff5761080081116106f05760018416156106e157600186016001600160401b03841161065f57610516846105108354610fb7565b836111a6565b5f84601f811160011461067e5780610535925f91610673575b506111f5565b90555b6002860180546001600160401b0319166001600160401b0386811691909117909155600687019290821161065f5761057a826105748554610fb7565b856111a6565b5f90601f83116001146105e757825f51602061143c5f395f51905f529798936105c695936105b0935f926105dc575b50506111f5565b90555b426001600160401b031690600301611230565b6105d7604051928392339684611253565b0390a2005b013590508a806105a9565b601f19831691845f5260205f20925f5b81811061064757509260019285925f51602061143c5f395f51905f529a9b966105c698961061062e575b505050811b0190556105b3565b01355f19600384901b60f8161c19169055898080610621565b919360206001819287870135815501950192016105f7565b634e487b7160e01b5f52604160045260245ffd5b90508801358a61052f565b50601f19851690825f528560205f20925f5b8181106106c65750106106ad575b5050600184811b019055610538565b8701355f19600387901b60f8161c19169055878061069e565b8a840135855560019094019360209384019389935001610690565b630990044760e41b5f5260045ffd5b63216e38bd60e21b5f5260045ffd5b639049092560e01b5f5260045ffd5b346102d35760403660031901126102d3576020610729610fa1565b61073161111f565b6001600160a01b039091165f9081526001835260409020600481015460ff16919082610764575b50506040519015158152f35b6002015481166001600160401b0390811691161490508280610758565b346102d35760203660031901126102d3576001600160a01b036107a2610fa1565b165f526001602052602060ff600460405f200154166040519015158152f35b346102d3575f3660031901126102d357602060405160018152f35b346102d3575f3660031901126102d3576020600254604051908152f35b346102d3575f3660031901126102d3575f546040516001600160a01b039091168152602090f35b346102d35760203660031901126102d357600435335f52600160205260405f2060ff60048201541615610452576005810182905561086b90426001600160401b031690600301611230565b6040519081527fb9e6158028d8a0c7cf69ed321b5d7a2b6c2d5d18395df5667bca51f046e0c62d60203392a2005b346102d3575f3660031901126102d3576108b1611415565b5f80546001600160a01b0319811682556001600160a01b03165f51602061147c5f395f51905f528280a3005b346102d35760403660031901126102d3576108fc6024356004356112c4565b9060405190604082019260408352815180945260206060840192015f945b80861061092f57505082935060208301520390f35b81516001600160a01b03168452600195909501946020938401939091019061091a565b346102d35760403660031901126102d3576004356001600160801b038116908181036102d3576024356001600160801b03811691908290036102d357335f52600160205260405f209060ff6004830154161561045257816109bb60039260026109dc9501611207565b0180546001600160801b03191683178155426001600160401b031690611230565b60405191825260208201527fafc22a6379ae3c9de76300bcf15145466f4ae54c3db09ed5492e957f44f6c56b60403392a2005b346102d35760a03660031901126102d3576004356001600160401b0381116102d357610a3f9036906004016110f2565b610a4761111f565b6044356001600160801b038116908190036102d3576064356001600160801b038116908190036102d3576084356001600160401b0381116102d357610a909036906004016110f2565b90335f52600160205260ff600460405f20015416610e355761020086116106ff5761080082116106f05760018516156106e15760405192426001600160401b0316610ada85610fef565b338552610ae836898b611161565b95602086019687526040860160018060401b0389168152606087019182526080870193845260a087019183835260c08801938452610b3860e0890196600188526101008a01985f8a523691611161565b6101208901908152335f90815260016020819052604090912099518a546001600160a01b0319166001600160a01b0391909116178a5599518051919a8a0191906001600160401b03821161065f57610b94826105748554610fb7565b602090601f8311600114610dc15792610bc783610c4999979460069e9d9c9b999794610bff975f92610db65750506111f5565b90555b905160028a0180546001600160401b0319166001600160401b039290921691909117815590516001600160801b031690611207565b91516003870180549351600160801b600160c01b0360809190911b166001600160801b039092166001600160c01b031990941693909317178255516001600160401b031690611230565b600483019051151560ff8019835416911617905551600582015501905180519060018060401b03821161065f57610c84826105748554610fb7565b602090601f8311600114610d5357610ca592915f9183610d485750506111f5565b90555b335f52600360205260ff60405f20541615610cde575b5f51602061145c5f395f51905f52916105d7604051928392339684611253565b335f908152600360205260409020805460ff1916600117905560025491600160401b83101561065f57610d258360015f51602061145c5f395f51905f529501600255611135565b81546001600160a01b0360039290921b91821b19163390911b1790559150610cbe565b0151905087806105a9565b90601f19831691845f52815f20925f5b818110610d9e5750908460019594939210610d86575b505050811b019055610ca8565b01515f1960f88460031b161c19169055868080610d79565b92936020600181928786015181550195019301610d63565b015190505f806105a9565b90601f19831691845f52815f20925f5b818110610e1d57509360069d9c9b9a989693610bff969360019383610c499d9b9810610e05575b505050811b019055610bca565b01515f1960f88460031b161c191690555f8080610df8565b92936020600181928786015181550195019301610dd1565b630ea075bf60e21b5f5260045ffd5b346102d35760203660031901126102d3576001600160a01b03610e65610fa1565b165f526003602052602060ff60405f2054166040519015158152f35b346102d35760203660031901126102d3576001600160a01b03610ea2610fa1565b165f52600160205260405f2060018060a01b038154166102cf610ec76001840161102e565b9260028101549060038101549060ff600482015416610eed60066005840154930161102e565b92610f0c604051988998895261014060208a01526101408901906110ce565b6001600160401b0386811660408a8101919091526001600160801b0397901c871660608a01529582166080808a019190915282901c90951660a088015260c090811c90870152151560e08601526101008501528382036101208501526110ce565b346102d3575f3660031901126102d357602060405160048152f35b346102d3575f3660031901126102d35780600860209252f35b600435906001600160a01b03821682036102d357565b90600182811c92168015610fe5575b6020831014610fd157565b634e487b7160e01b5f52602260045260245ffd5b91607f1691610fc6565b61014081019081106001600160401b0382111761065f57604052565b601f909101601f19168101906001600160401b0382119082101761065f57604052565b9060405191825f82549261104184610fb7565b80845293600181169081156110ac5750600114611068575b506110669250038361100b565b565b90505f9291925260205f20905f915b818310611090575050906020611066928201015f611059565b6020919350806001915483858901015201910190918492611077565b90506020925061106694915060ff191682840152151560051b8201015f611059565b805180835260209291819084018484015e5f828201840152601f01601f1916010190565b9181601f840112156102d3578235916001600160401b0383116102d357602083818601950101116102d357565b602435906001600160401b03821682036102d357565b60025481101561114d5760025f5260205f2001905f90565b634e487b7160e01b5f52603260045260245ffd5b9192916001600160401b03821161065f576040519161118a601f8201601f19166020018461100b565b8294818452818301116102d3578281602093845f960137010152565b601f82116111b357505050565b5f5260205f20906020601f840160051c830193106111eb575b601f0160051c01905b8181106111e0575050565b5f81556001016111d5565b90915081906111cc565b8160011b915f199060031b1c19161790565b8054600160401b600160c01b03191660409290921b600160401b600160c01b0316919091179055565b80546001600160c01b031660c09290921b6001600160c01b031916919091179055565b918060609160209396959660408652816040870152838601375f828286010152601f80199101168301019360018060401b0316910152565b6001600160401b03811161065f5760051b60200190565b5f1981146112b05760010190565b634e487b7160e01b5f52601160045260245ffd5b6002549291838210156113fa578101928382116112b0578084116113f2575b505f90805b8481106113a757506112f98261128b565b91611307604051938461100b565b808352611316601f199161128b565b013660208401375f905b84811061132d5750509190565b61133681611135565b60018060a01b0391549060031b1c165f52600160205260ff600460405f20015416611364575b600101611320565b9061136e82611135565b905460039190911b1c6001600160a01b03169161138a826112a2565b92845183101561114d57602060019360051b86010152905061135c565b6113b081611135565b60018060a01b0391549060031b1c165f52600160205260ff600460405f200154166113de575b6001016112e8565b916113ea6001916112a2565b9290506113d6565b92505f6112e3565b505060405161140a60208261100b565b5f81525f3681379190565b5f546001600160a01b0316330361142857565b63118cdaa760e01b5f523360045260245ffdfed38712fdbf30167b0c646ff649ac2873c75b20c0db08aa569e7c3eb37c18fa40e08f02adbc8dd2124ea677973e24fc7ac7fe8d4074d6bb120d7fd06ad4ebdd588be0079c531659141344cd1fd0a4f28419497f9722a3daafe3b4186f6b6457e0a2646970667358221220f6d5b8924f8bd0b24bf215c06125893a4abe5590534d2809efb3a9403e25b7cd64736f6c634300081e0033",
}

// ProverRegistryABI is the input ABI used to generate the binding from.
// Deprecated: Use ProverRegistryMetaData.ABI instead.
var ProverRegistryABI = ProverRegistryMetaData.ABI

// ProverRegistryBin is the compiled bytecode used for deploying new contracts.
// Deprecated: Use ProverRegistryMetaData.Bin instead.
var ProverRegistryBin = ProverRegistryMetaData.Bin

// DeployProverRegistry deploys a new Ethereum contract, binding an instance of ProverRegistry to it.
func DeployProverRegistry(auth *bind.TransactOpts, backend bind.ContractBackend) (common.Address, *types.Transaction, *ProverRegistry, error) {
	parsed, err := ProverRegistryMetaData.GetAbi()
	if err != nil {
		return common.Address{}, nil, nil, err
	}
	if parsed == nil {
		return common.Address{}, nil, nil, errors.New("GetABI returned nil")
	}

	address, tx, contract, err := bind.DeployContract(auth, *parsed, common.FromHex(ProverRegistryBin), backend)
	if err != nil {
		return common.Address{}, nil, nil, err
	}
	return address, tx, &ProverRegistry{ProverRegistryCaller: ProverRegistryCaller{contract: contract}, ProverRegistryTransactor: ProverRegistryTransactor{contract: contract}, ProverRegistryFilterer: ProverRegistryFilterer{contract: contract}}, nil
}

// ProverRegistry is an auto generated Go binding around an Ethereum contract.
type ProverRegistry struct {
	ProverRegistryCaller     // Read-only binding to the contract
	ProverRegistryTransactor // Write-only binding to the contract
	ProverRegistryFilterer   // Log filterer for contract events
}

// ProverRegistryCaller is an auto generated read-only Go binding around an Ethereum contract.
type ProverRegistryCaller struct {
	contract *bind.BoundContract // Generic contract wrapper for the low level calls
}

// ProverRegistryTransactor is an auto generated write-only Go binding around an Ethereum contract.
type ProverRegistryTransactor struct {
	contract *bind.BoundContract // Generic contract wrapper for the low level calls
}

// ProverRegistryFilterer is an auto generated log filtering Go binding around an Ethereum contract events.
type ProverRegistryFilterer struct {
	contract *bind.BoundContract // Generic contract wrapper for the low level calls
}

// ProverRegistrySession is an auto generated Go binding around an Ethereum contract,
// with pre-set call and transact options.
type ProverRegistrySession struct {
	Contract     *ProverRegistry   // Generic contract binding to set the session for
	CallOpts     bind.CallOpts     // Call options to use throughout this session
	TransactOpts bind.TransactOpts // Transaction auth options to use throughout this session
}

// ProverRegistryCallerSession is an auto generated read-only Go binding around an Ethereum contract,
// with pre-set call options.
type ProverRegistryCallerSession struct {
	Contract *ProverRegistryCaller // Generic contract caller binding to set the session for
	CallOpts bind.CallOpts         // Call options to use throughout this session
}

// ProverRegistryTransactorSession is an auto generated write-only Go binding around an Ethereum contract,
// with pre-set transact options.
type ProverRegistryTransactorSession struct {
	Contract     *ProverRegistryTransactor // Generic contract transactor binding to set the session for
	TransactOpts bind.TransactOpts         // Transaction auth options to use throughout this session
}

// ProverRegistryRaw is an auto generated low-level Go binding around an Ethereum contract.
type ProverRegistryRaw struct {
	Contract *ProverRegistry // Generic contract binding to access the raw methods on
}

// ProverRegistryCallerRaw is an auto generated low-level read-only Go binding around an Ethereum contract.
type ProverRegistryCallerRaw struct {
	Contract *ProverRegistryCaller // Generic read-only contract binding to access the raw methods on
}

// ProverRegistryTransactorRaw is an auto generated low-level write-only Go binding around an Ethereum contract.
type ProverRegistryTransactorRaw struct {
	Contract *ProverRegistryTransactor // Generic write-only contract binding to access the raw methods on
}

// NewProverRegistry creates a new instance of ProverRegistry, bound to a specific deployed contract.
func NewProverRegistry(address common.Address, backend bind.ContractBackend) (*ProverRegistry, error) {
	contract, err := bindProverRegistry(address, backend, backend, backend)
	if err != nil {
		return nil, err
	}
	return &ProverRegistry{ProverRegistryCaller: ProverRegistryCaller{contract: contract}, ProverRegistryTransactor: ProverRegistryTransactor{contract: contract}, ProverRegistryFilterer: ProverRegistryFilterer{contract: contract}}, nil
}

// NewProverRegistryCaller creates a new read-only instance of ProverRegistry, bound to a specific deployed contract.
func NewProverRegistryCaller(address common.Address, caller bind.ContractCaller) (*ProverRegistryCaller, error) {
	contract, err := bindProverRegistry(address, caller, nil, nil)
	if err != nil {
		return nil, err
	}
	return &ProverRegistryCaller{contract: contract}, nil
}

// NewProverRegistryTransactor creates a new write-only instance of ProverRegistry, bound to a specific deployed contract.
func NewProverRegistryTransactor(address common.Address, transactor bind.ContractTransactor) (*ProverRegistryTransactor, error) {
	contract, err := bindProverRegistry(address, nil, transactor, nil)
	if err != nil {
		return nil, err
	}
	return &ProverRegistryTransactor{contract: contract}, nil
}

// NewProverRegistryFilterer creates a new log filterer instance of ProverRegistry, bound to a specific deployed contract.
func NewProverRegistryFilterer(address common.Address, filterer bind.ContractFilterer) (*ProverRegistryFilterer, error) {
	contract, err := bindProverRegistry(address, nil, nil, filterer)
	if err != nil {
		return nil, err
	}
	return &ProverRegistryFilterer{contract: contract}, nil
}

// bindProverRegistry binds a generic wrapper to an already deployed contract.
func bindProverRegistry(address common.Address, caller bind.ContractCaller, transactor bind.ContractTransactor, filterer bind.ContractFilterer) (*bind.BoundContract, error) {
	parsed, err := ProverRegistryMetaData.GetAbi()
	if err != nil {
		return nil, err
	}
	return bind.NewBoundContract(address, *parsed, caller, transactor, filterer), nil
}

// Call invokes the (constant) contract method with params as input values and
// sets the output to result. The result type might be a single field for simple
// returns, a slice of interfaces for anonymous returns and a struct for named
// returns.
func (_ProverRegistry *ProverRegistryRaw) Call(opts *bind.CallOpts, result *[]interface{}, method string, params ...interface{}) error {
	return _ProverRegistry.Contract.ProverRegistryCaller.contract.Call(opts, result, method, params...)
}

// Transfer initiates a plain transaction to move funds to the contract, calling
// its default method if one is available.
func (_ProverRegistry *ProverRegistryRaw) Transfer(opts *bind.TransactOpts) (*types.Transaction, error) {
	return _ProverRegistry.Contract.ProverRegistryTransactor.contract.Transfer(opts)
}

// Transact invokes the (paid) contract method with params as input values.
func (_ProverRegistry *ProverRegistryRaw) Transact(opts *bind.TransactOpts, method string, params ...interface{}) (*types.Transaction, error) {
	return _ProverRegistry.Contract.ProverRegistryTransactor.contract.Transact(opts, method, params...)
}

// Call invokes the (constant) contract method with params as input values and
// sets the output to result. The result type might be a single field for simple
// returns, a slice of interfaces for anonymous returns and a struct for named
// returns.
func (_ProverRegistry *ProverRegistryCallerRaw) Call(opts *bind.CallOpts, result *[]interface{}, method string, params ...interface{}) error {
	return _ProverRegistry.Contract.contract.Call(opts, result, method, params...)
}

// Transfer initiates a plain transaction to move funds to the contract, calling
// its default method if one is available.
func (_ProverRegistry *ProverRegistryTransactorRaw) Transfer(opts *bind.TransactOpts) (*types.Transaction, error) {
	return _ProverRegistry.Contract.contract.Transfer(opts)
}

// Transact invokes the (paid) contract method with params as input values.
func (_ProverRegistry *ProverRegistryTransactorRaw) Transact(opts *bind.TransactOpts, method string, params ...interface{}) (*types.Transaction, error) {
	return _ProverRegistry.Contract.contract.Transact(opts, method, params...)
}

// FEATUREHTTPSSERVING is a free data retrieval call binding the contract method 0xe95aff82.
//
// Solidity: function FEATURE_HTTPS_SERVING() view returns(uint64)
func (_ProverRegistry *ProverRegistryCaller) FEATUREHTTPSSERVING(opts *bind.CallOpts) (uint64, error) {
	var out []interface{}
	err := _ProverRegistry.contract.Call(opts, &out, "FEATURE_HTTPS_SERVING")

	if err != nil {
		return *new(uint64), err
	}

	out0 := *abi.ConvertType(out[0], new(uint64)).(*uint64)

	return out0, err

}

// FEATUREHTTPSSERVING is a free data retrieval call binding the contract method 0xe95aff82.
//
// Solidity: function FEATURE_HTTPS_SERVING() view returns(uint64)
func (_ProverRegistry *ProverRegistrySession) FEATUREHTTPSSERVING() (uint64, error) {
	return _ProverRegistry.Contract.FEATUREHTTPSSERVING(&_ProverRegistry.CallOpts)
}

// FEATUREHTTPSSERVING is a free data retrieval call binding the contract method 0xe95aff82.
//
// Solidity: function FEATURE_HTTPS_SERVING() view returns(uint64)
func (_ProverRegistry *ProverRegistryCallerSession) FEATUREHTTPSSERVING() (uint64, error) {
	return _ProverRegistry.Contract.FEATUREHTTPSSERVING(&_ProverRegistry.CallOpts)
}

// FEATUREPDP is a free data retrieval call binding the contract method 0x99105669.
//
// Solidity: function FEATURE_PDP() view returns(uint64)
func (_ProverRegistry *ProverRegistryCaller) FEATUREPDP(opts *bind.CallOpts) (uint64, error) {
	var out []interface{}
	err := _ProverRegistry.contract.Call(opts, &out, "FEATURE_PDP")

	if err != nil {
		return *new(uint64), err
	}

	out0 := *abi.ConvertType(out[0], new(uint64)).(*uint64)

	return out0, err

}

// FEATUREPDP is a free data retrieval call binding the contract method 0x99105669.
//
// Solidity: function FEATURE_PDP() view returns(uint64)
func (_ProverRegistry *ProverRegistrySession) FEATUREPDP() (uint64, error) {
	return _ProverRegistry.Contract.FEATUREPDP(&_ProverRegistry.CallOpts)
}

// FEATUREPDP is a free data retrieval call binding the contract method 0x99105669.
//
// Solidity: function FEATURE_PDP() view returns(uint64)
func (_ProverRegistry *ProverRegistryCallerSession) FEATUREPDP() (uint64, error) {
	return _ProverRegistry.Contract.FEATUREPDP(&_ProverRegistry.CallOpts)
}

// FEATUREQBP is a free data retrieval call binding the contract method 0x1200c7f5.
//
// Solidity: function FEATURE_QBP() view returns(uint64)
func (_ProverRegistry *ProverRegistryCaller) FEATUREQBP(opts *bind.CallOpts) (uint64, error) {
	var out []interface{}
	err := _ProverRegistry.contract.Call(opts, &out, "FEATURE_QBP")

	if err != nil {
		return *new(uint64), err
	}

	out0 := *abi.ConvertType(out[0], new(uint64)).(*uint64)

	return out0, err

}

// FEATUREQBP is a free data retrieval call binding the contract method 0x1200c7f5.
//
// Solidity: function FEATURE_QBP() view returns(uint64)
func (_ProverRegistry *ProverRegistrySession) FEATUREQBP() (uint64, error) {
	return _ProverRegistry.Contract.FEATUREQBP(&_ProverRegistry.CallOpts)
}

// FEATUREQBP is a free data retrieval call binding the contract method 0x1200c7f5.
//
// Solidity: function FEATURE_QBP() view returns(uint64)
func (_ProverRegistry *ProverRegistryCallerSession) FEATUREQBP() (uint64, error) {
	return _ProverRegistry.Contract.FEATUREQBP(&_ProverRegistry.CallOpts)
}

// FEATURETEE is a free data retrieval call binding the contract method 0x1dcb0bd9.
//
// Solidity: function FEATURE_TEE() view returns(uint64)
func (_ProverRegistry *ProverRegistryCaller) FEATURETEE(opts *bind.CallOpts) (uint64, error) {
	var out []interface{}
	err := _ProverRegistry.contract.Call(opts, &out, "FEATURE_TEE")

	if err != nil {
		return *new(uint64), err
	}

	out0 := *abi.ConvertType(out[0], new(uint64)).(*uint64)

	return out0, err

}

// FEATURETEE is a free data retrieval call binding the contract method 0x1dcb0bd9.
//
// Solidity: function FEATURE_TEE() view returns(uint64)
func (_ProverRegistry *ProverRegistrySession) FEATURETEE() (uint64, error) {
	return _ProverRegistry.Contract.FEATURETEE(&_ProverRegistry.CallOpts)
}

// FEATURETEE is a free data retrieval call binding the contract method 0x1dcb0bd9.
//
// Solidity: function FEATURE_TEE() view returns(uint64)
func (_ProverRegistry *ProverRegistryCallerSession) FEATURETEE() (uint64, error) {
	return _ProverRegistry.Contract.FEATURETEE(&_ProverRegistry.CallOpts)
}

// MAXENDPOINTLENGTH is a free data retrieval call binding the contract method 0xc85d3eb0.
//
// Solidity: function MAX_ENDPOINT_LENGTH() view returns(uint256)
func (_ProverRegistry *ProverRegistryCaller) MAXENDPOINTLENGTH(opts *bind.CallOpts) (*big.Int, error) {
	var out []interface{}
	err := _ProverRegistry.contract.Call(opts, &out, "MAX_ENDPOINT_LENGTH")

	if err != nil {
		return *new(*big.Int), err
	}

	out0 := *abi.ConvertType(out[0], new(*big.Int)).(**big.Int)

	return out0, err

}

// MAXENDPOINTLENGTH is a free data retrieval call binding the contract method 0xc85d3eb0.
//
// Solidity: function MAX_ENDPOINT_LENGTH() view returns(uint256)
func (_ProverRegistry *ProverRegistrySession) MAXENDPOINTLENGTH() (*big.Int, error) {
	return _ProverRegistry.Contract.MAXENDPOINTLENGTH(&_ProverRegistry.CallOpts)
}

// MAXENDPOINTLENGTH is a free data retrieval call binding the contract method 0xc85d3eb0.
//
// Solidity: function MAX_ENDPOINT_LENGTH() view returns(uint256)
func (_ProverRegistry *ProverRegistryCallerSession) MAXENDPOINTLENGTH() (*big.Int, error) {
	return _ProverRegistry.Contract.MAXENDPOINTLENGTH(&_ProverRegistry.CallOpts)
}

// MAXMETADATALENGTH is a free data retrieval call binding the contract method 0xe8868e9f.
//
// Solidity: function MAX_METADATA_LENGTH() view returns(uint256)
func (_ProverRegistry *ProverRegistryCaller) MAXMETADATALENGTH(opts *bind.CallOpts) (*big.Int, error) {
	var out []interface{}
	err := _ProverRegistry.contract.Call(opts, &out, "MAX_METADATA_LENGTH")

	if err != nil {
		return *new(*big.Int), err
	}

	out0 := *abi.ConvertType(out[0], new(*big.Int)).(**big.Int)

	return out0, err

}

// MAXMETADATALENGTH is a free data retrieval call binding the contract method 0xe8868e9f.
//
// Solidity: function MAX_METADATA_LENGTH() view returns(uint256)
func (_ProverRegistry *ProverRegistrySession) MAXMETADATALENGTH() (*big.Int, error) {
	return _ProverRegistry.Contract.MAXMETADATALENGTH(&_ProverRegistry.CallOpts)
}

// MAXMETADATALENGTH is a free data retrieval call binding the contract method 0xe8868e9f.
//
// Solidity: function MAX_METADATA_LENGTH() view returns(uint256)
func (_ProverRegistry *ProverRegistryCallerSession) MAXMETADATALENGTH() (*big.Int, error) {
	return _ProverRegistry.Contract.MAXMETADATALENGTH(&_ProverRegistry.CallOpts)
}

// GetProver is a free data retrieval call binding the contract method 0xfaab193c.
//
// Solidity: function getProver(address prover) view returns((address,string,uint64,uint128,uint128,uint64,uint64,bool,bytes32,string))
func (_ProverRegistry *ProverRegistryCaller) GetProver(opts *bind.CallOpts, prover common.Address) (ProverRegistryProver, error) {
	var out []interface{}
	err := _ProverRegistry.contract.Call(opts, &out, "getProver", prover)

	if err != nil {
		return *new(ProverRegistryProver), err
	}

	out0 := *abi.ConvertType(out[0], new(ProverRegistryProver)).(*ProverRegistryProver)

	return out0, err

}

// GetProver is a free data retrieval call binding the contract method 0xfaab193c.
//
// Solidity: function getProver(address prover) view returns((address,string,uint64,uint128,uint128,uint64,uint64,bool,bytes32,string))
func (_ProverRegistry *ProverRegistrySession) GetProver(prover common.Address) (ProverRegistryProver, error) {
	return _ProverRegistry.Contract.GetProver(&_ProverRegistry.CallOpts, prover)
}

// GetProver is a free data retrieval call binding the contract method 0xfaab193c.
//
// Solidity: function getProver(address prover) view returns((address,string,uint64,uint128,uint128,uint64,uint64,bool,bytes32,string))
func (_ProverRegistry *ProverRegistryCallerSession) GetProver(prover common.Address) (ProverRegistryProver, error) {
	return _ProverRegistry.Contract.GetProver(&_ProverRegistry.CallOpts, prover)
}

// IsActive is a free data retrieval call binding the contract method 0x9f8a13d7.
//
// Solidity: function isActive(address prover) view returns(bool)
func (_ProverRegistry *ProverRegistryCaller) IsActive(opts *bind.CallOpts, prover common.Address) (bool, error) {
	var out []interface{}
	err := _ProverRegistry.contract.Call(opts, &out, "isActive", prover)

	if err != nil {
		return *new(bool), err
	}

	out0 := *abi.ConvertType(out[0], new(bool)).(*bool)

	return out0, err

}

// IsActive is a free data retrieval call binding the contract method 0x9f8a13d7.
//
// Solidity: function isActive(address prover) view returns(bool)
func (_ProverRegistry *ProverRegistrySession) IsActive(prover common.Address) (bool, error) {
	return _ProverRegistry.Contract.IsActive(&_ProverRegistry.CallOpts, prover)
}

// IsActive is a free data retrieval call binding the contract method 0x9f8a13d7.
//
// Solidity: function isActive(address prover) view returns(bool)
func (_ProverRegistry *ProverRegistryCallerSession) IsActive(prover common.Address) (bool, error) {
	return _ProverRegistry.Contract.IsActive(&_ProverRegistry.CallOpts, prover)
}

// Known is a free data retrieval call binding the contract method 0x22dfc944.
//
// Solidity: function known(address ) view returns(bool)
func (_ProverRegistry *ProverRegistryCaller) Known(opts *bind.CallOpts, arg0 common.Address) (bool, error) {
	var out []interface{}
	err := _ProverRegistry.contract.Call(opts, &out, "known", arg0)

	if err != nil {
		return *new(bool), err
	}

	out0 := *abi.ConvertType(out[0], new(bool)).(*bool)

	return out0, err

}

// Known is a free data retrieval call binding the contract method 0x22dfc944.
//
// Solidity: function known(address ) view returns(bool)
func (_ProverRegistry *ProverRegistrySession) Known(arg0 common.Address) (bool, error) {
	return _ProverRegistry.Contract.Known(&_ProverRegistry.CallOpts, arg0)
}

// Known is a free data retrieval call binding the contract method 0x22dfc944.
//
// Solidity: function known(address ) view returns(bool)
func (_ProverRegistry *ProverRegistryCallerSession) Known(arg0 common.Address) (bool, error) {
	return _ProverRegistry.Contract.Known(&_ProverRegistry.CallOpts, arg0)
}

// ListActive is a free data retrieval call binding the contract method 0x49d2982b.
//
// Solidity: function listActive(uint256 offset, uint256 limit) view returns(address[] result, uint256 nextOffset)
func (_ProverRegistry *ProverRegistryCaller) ListActive(opts *bind.CallOpts, offset *big.Int, limit *big.Int) (struct {
	Result     []common.Address
	NextOffset *big.Int
}, error) {
	var out []interface{}
	err := _ProverRegistry.contract.Call(opts, &out, "listActive", offset, limit)

	outstruct := new(struct {
		Result     []common.Address
		NextOffset *big.Int
	})
	if err != nil {
		return *outstruct, err
	}

	outstruct.Result = *abi.ConvertType(out[0], new([]common.Address)).(*[]common.Address)
	outstruct.NextOffset = *abi.ConvertType(out[1], new(*big.Int)).(**big.Int)

	return *outstruct, err

}

// ListActive is a free data retrieval call binding the contract method 0x49d2982b.
//
// Solidity: function listActive(uint256 offset, uint256 limit) view returns(address[] result, uint256 nextOffset)
func (_ProverRegistry *ProverRegistrySession) ListActive(offset *big.Int, limit *big.Int) (struct {
	Result     []common.Address
	NextOffset *big.Int
}, error) {
	return _ProverRegistry.Contract.ListActive(&_ProverRegistry.CallOpts, offset, limit)
}

// ListActive is a free data retrieval call binding the contract method 0x49d2982b.
//
// Solidity: function listActive(uint256 offset, uint256 limit) view returns(address[] result, uint256 nextOffset)
func (_ProverRegistry *ProverRegistryCallerSession) ListActive(offset *big.Int, limit *big.Int) (struct {
	Result     []common.Address
	NextOffset *big.Int
}, error) {
	return _ProverRegistry.Contract.ListActive(&_ProverRegistry.CallOpts, offset, limit)
}

// Owner is a free data retrieval call binding the contract method 0x8da5cb5b.
//
// Solidity: function owner() view returns(address)
func (_ProverRegistry *ProverRegistryCaller) Owner(opts *bind.CallOpts) (common.Address, error) {
	var out []interface{}
	err := _ProverRegistry.contract.Call(opts, &out, "owner")

	if err != nil {
		return *new(common.Address), err
	}

	out0 := *abi.ConvertType(out[0], new(common.Address)).(*common.Address)

	return out0, err

}

// Owner is a free data retrieval call binding the contract method 0x8da5cb5b.
//
// Solidity: function owner() view returns(address)
func (_ProverRegistry *ProverRegistrySession) Owner() (common.Address, error) {
	return _ProverRegistry.Contract.Owner(&_ProverRegistry.CallOpts)
}

// Owner is a free data retrieval call binding the contract method 0x8da5cb5b.
//
// Solidity: function owner() view returns(address)
func (_ProverRegistry *ProverRegistryCallerSession) Owner() (common.Address, error) {
	return _ProverRegistry.Contract.Owner(&_ProverRegistry.CallOpts)
}

// ProverAddresses is a free data retrieval call binding the contract method 0xd2c7f2ac.
//
// Solidity: function proverAddresses(uint256 ) view returns(address)
func (_ProverRegistry *ProverRegistryCaller) ProverAddresses(opts *bind.CallOpts, arg0 *big.Int) (common.Address, error) {
	var out []interface{}
	err := _ProverRegistry.contract.Call(opts, &out, "proverAddresses", arg0)

	if err != nil {
		return *new(common.Address), err
	}

	out0 := *abi.ConvertType(out[0], new(common.Address)).(*common.Address)

	return out0, err

}

// ProverAddresses is a free data retrieval call binding the contract method 0xd2c7f2ac.
//
// Solidity: function proverAddresses(uint256 ) view returns(address)
func (_ProverRegistry *ProverRegistrySession) ProverAddresses(arg0 *big.Int) (common.Address, error) {
	return _ProverRegistry.Contract.ProverAddresses(&_ProverRegistry.CallOpts, arg0)
}

// ProverAddresses is a free data retrieval call binding the contract method 0xd2c7f2ac.
//
// Solidity: function proverAddresses(uint256 ) view returns(address)
func (_ProverRegistry *ProverRegistryCallerSession) ProverAddresses(arg0 *big.Int) (common.Address, error) {
	return _ProverRegistry.Contract.ProverAddresses(&_ProverRegistry.CallOpts, arg0)
}

// Provers is a free data retrieval call binding the contract method 0x1dec844b.
//
// Solidity: function provers(address ) view returns(address owner, string endpoint, uint64 features, uint128 pricePerGibDay, uint128 pricePerByteServed, uint64 registeredAt, uint64 updatedAt, bool active, bytes32 ensNode, string metadata)
func (_ProverRegistry *ProverRegistryCaller) Provers(opts *bind.CallOpts, arg0 common.Address) (struct {
	Owner              common.Address
	Endpoint           string
	Features           uint64
	PricePerGibDay     *big.Int
	PricePerByteServed *big.Int
	RegisteredAt       uint64
	UpdatedAt          uint64
	Active             bool
	EnsNode            [32]byte
	Metadata           string
}, error) {
	var out []interface{}
	err := _ProverRegistry.contract.Call(opts, &out, "provers", arg0)

	outstruct := new(struct {
		Owner              common.Address
		Endpoint           string
		Features           uint64
		PricePerGibDay     *big.Int
		PricePerByteServed *big.Int
		RegisteredAt       uint64
		UpdatedAt          uint64
		Active             bool
		EnsNode            [32]byte
		Metadata           string
	})
	if err != nil {
		return *outstruct, err
	}

	outstruct.Owner = *abi.ConvertType(out[0], new(common.Address)).(*common.Address)
	outstruct.Endpoint = *abi.ConvertType(out[1], new(string)).(*string)
	outstruct.Features = *abi.ConvertType(out[2], new(uint64)).(*uint64)
	outstruct.PricePerGibDay = *abi.ConvertType(out[3], new(*big.Int)).(**big.Int)
	outstruct.PricePerByteServed = *abi.ConvertType(out[4], new(*big.Int)).(**big.Int)
	outstruct.RegisteredAt = *abi.ConvertType(out[5], new(uint64)).(*uint64)
	outstruct.UpdatedAt = *abi.ConvertType(out[6], new(uint64)).(*uint64)
	outstruct.Active = *abi.ConvertType(out[7], new(bool)).(*bool)
	outstruct.EnsNode = *abi.ConvertType(out[8], new([32]byte)).(*[32]byte)
	outstruct.Metadata = *abi.ConvertType(out[9], new(string)).(*string)

	return *outstruct, err

}

// Provers is a free data retrieval call binding the contract method 0x1dec844b.
//
// Solidity: function provers(address ) view returns(address owner, string endpoint, uint64 features, uint128 pricePerGibDay, uint128 pricePerByteServed, uint64 registeredAt, uint64 updatedAt, bool active, bytes32 ensNode, string metadata)
func (_ProverRegistry *ProverRegistrySession) Provers(arg0 common.Address) (struct {
	Owner              common.Address
	Endpoint           string
	Features           uint64
	PricePerGibDay     *big.Int
	PricePerByteServed *big.Int
	RegisteredAt       uint64
	UpdatedAt          uint64
	Active             bool
	EnsNode            [32]byte
	Metadata           string
}, error) {
	return _ProverRegistry.Contract.Provers(&_ProverRegistry.CallOpts, arg0)
}

// Provers is a free data retrieval call binding the contract method 0x1dec844b.
//
// Solidity: function provers(address ) view returns(address owner, string endpoint, uint64 features, uint128 pricePerGibDay, uint128 pricePerByteServed, uint64 registeredAt, uint64 updatedAt, bool active, bytes32 ensNode, string metadata)
func (_ProverRegistry *ProverRegistryCallerSession) Provers(arg0 common.Address) (struct {
	Owner              common.Address
	Endpoint           string
	Features           uint64
	PricePerGibDay     *big.Int
	PricePerByteServed *big.Int
	RegisteredAt       uint64
	UpdatedAt          uint64
	Active             bool
	EnsNode            [32]byte
	Metadata           string
}, error) {
	return _ProverRegistry.Contract.Provers(&_ProverRegistry.CallOpts, arg0)
}

// SupportsFeature is a free data retrieval call binding the contract method 0xa91a9702.
//
// Solidity: function supportsFeature(address prover, uint64 feature) view returns(bool)
func (_ProverRegistry *ProverRegistryCaller) SupportsFeature(opts *bind.CallOpts, prover common.Address, feature uint64) (bool, error) {
	var out []interface{}
	err := _ProverRegistry.contract.Call(opts, &out, "supportsFeature", prover, feature)

	if err != nil {
		return *new(bool), err
	}

	out0 := *abi.ConvertType(out[0], new(bool)).(*bool)

	return out0, err

}

// SupportsFeature is a free data retrieval call binding the contract method 0xa91a9702.
//
// Solidity: function supportsFeature(address prover, uint64 feature) view returns(bool)
func (_ProverRegistry *ProverRegistrySession) SupportsFeature(prover common.Address, feature uint64) (bool, error) {
	return _ProverRegistry.Contract.SupportsFeature(&_ProverRegistry.CallOpts, prover, feature)
}

// SupportsFeature is a free data retrieval call binding the contract method 0xa91a9702.
//
// Solidity: function supportsFeature(address prover, uint64 feature) view returns(bool)
func (_ProverRegistry *ProverRegistryCallerSession) SupportsFeature(prover common.Address, feature uint64) (bool, error) {
	return _ProverRegistry.Contract.SupportsFeature(&_ProverRegistry.CallOpts, prover, feature)
}

// TotalRegistered is a free data retrieval call binding the contract method 0x927416c0.
//
// Solidity: function totalRegistered() view returns(uint256)
func (_ProverRegistry *ProverRegistryCaller) TotalRegistered(opts *bind.CallOpts) (*big.Int, error) {
	var out []interface{}
	err := _ProverRegistry.contract.Call(opts, &out, "totalRegistered")

	if err != nil {
		return *new(*big.Int), err
	}

	out0 := *abi.ConvertType(out[0], new(*big.Int)).(**big.Int)

	return out0, err

}

// TotalRegistered is a free data retrieval call binding the contract method 0x927416c0.
//
// Solidity: function totalRegistered() view returns(uint256)
func (_ProverRegistry *ProverRegistrySession) TotalRegistered() (*big.Int, error) {
	return _ProverRegistry.Contract.TotalRegistered(&_ProverRegistry.CallOpts)
}

// TotalRegistered is a free data retrieval call binding the contract method 0x927416c0.
//
// Solidity: function totalRegistered() view returns(uint256)
func (_ProverRegistry *ProverRegistryCallerSession) TotalRegistered() (*big.Int, error) {
	return _ProverRegistry.Contract.TotalRegistered(&_ProverRegistry.CallOpts)
}

// BindENS is a paid mutator transaction binding the contract method 0x84691fab.
//
// Solidity: function bindENS(bytes32 ensNode) returns()
func (_ProverRegistry *ProverRegistryTransactor) BindENS(opts *bind.TransactOpts, ensNode [32]byte) (*types.Transaction, error) {
	return _ProverRegistry.contract.Transact(opts, "bindENS", ensNode)
}

// BindENS is a paid mutator transaction binding the contract method 0x84691fab.
//
// Solidity: function bindENS(bytes32 ensNode) returns()
func (_ProverRegistry *ProverRegistrySession) BindENS(ensNode [32]byte) (*types.Transaction, error) {
	return _ProverRegistry.Contract.BindENS(&_ProverRegistry.TransactOpts, ensNode)
}

// BindENS is a paid mutator transaction binding the contract method 0x84691fab.
//
// Solidity: function bindENS(bytes32 ensNode) returns()
func (_ProverRegistry *ProverRegistryTransactorSession) BindENS(ensNode [32]byte) (*types.Transaction, error) {
	return _ProverRegistry.Contract.BindENS(&_ProverRegistry.TransactOpts, ensNode)
}

// Deregister is a paid mutator transaction binding the contract method 0xaff5edb1.
//
// Solidity: function deregister() returns()
func (_ProverRegistry *ProverRegistryTransactor) Deregister(opts *bind.TransactOpts) (*types.Transaction, error) {
	return _ProverRegistry.contract.Transact(opts, "deregister")
}

// Deregister is a paid mutator transaction binding the contract method 0xaff5edb1.
//
// Solidity: function deregister() returns()
func (_ProverRegistry *ProverRegistrySession) Deregister() (*types.Transaction, error) {
	return _ProverRegistry.Contract.Deregister(&_ProverRegistry.TransactOpts)
}

// Deregister is a paid mutator transaction binding the contract method 0xaff5edb1.
//
// Solidity: function deregister() returns()
func (_ProverRegistry *ProverRegistryTransactorSession) Deregister() (*types.Transaction, error) {
	return _ProverRegistry.Contract.Deregister(&_ProverRegistry.TransactOpts)
}

// Register is a paid mutator transaction binding the contract method 0x43eb4d94.
//
// Solidity: function register(string endpoint, uint64 features, uint128 pricePerGibDay, uint128 pricePerByteServed, string metadata) returns()
func (_ProverRegistry *ProverRegistryTransactor) Register(opts *bind.TransactOpts, endpoint string, features uint64, pricePerGibDay *big.Int, pricePerByteServed *big.Int, metadata string) (*types.Transaction, error) {
	return _ProverRegistry.contract.Transact(opts, "register", endpoint, features, pricePerGibDay, pricePerByteServed, metadata)
}

// Register is a paid mutator transaction binding the contract method 0x43eb4d94.
//
// Solidity: function register(string endpoint, uint64 features, uint128 pricePerGibDay, uint128 pricePerByteServed, string metadata) returns()
func (_ProverRegistry *ProverRegistrySession) Register(endpoint string, features uint64, pricePerGibDay *big.Int, pricePerByteServed *big.Int, metadata string) (*types.Transaction, error) {
	return _ProverRegistry.Contract.Register(&_ProverRegistry.TransactOpts, endpoint, features, pricePerGibDay, pricePerByteServed, metadata)
}

// Register is a paid mutator transaction binding the contract method 0x43eb4d94.
//
// Solidity: function register(string endpoint, uint64 features, uint128 pricePerGibDay, uint128 pricePerByteServed, string metadata) returns()
func (_ProverRegistry *ProverRegistryTransactorSession) Register(endpoint string, features uint64, pricePerGibDay *big.Int, pricePerByteServed *big.Int, metadata string) (*types.Transaction, error) {
	return _ProverRegistry.Contract.Register(&_ProverRegistry.TransactOpts, endpoint, features, pricePerGibDay, pricePerByteServed, metadata)
}

// RenounceOwnership is a paid mutator transaction binding the contract method 0x715018a6.
//
// Solidity: function renounceOwnership() returns()
func (_ProverRegistry *ProverRegistryTransactor) RenounceOwnership(opts *bind.TransactOpts) (*types.Transaction, error) {
	return _ProverRegistry.contract.Transact(opts, "renounceOwnership")
}

// RenounceOwnership is a paid mutator transaction binding the contract method 0x715018a6.
//
// Solidity: function renounceOwnership() returns()
func (_ProverRegistry *ProverRegistrySession) RenounceOwnership() (*types.Transaction, error) {
	return _ProverRegistry.Contract.RenounceOwnership(&_ProverRegistry.TransactOpts)
}

// RenounceOwnership is a paid mutator transaction binding the contract method 0x715018a6.
//
// Solidity: function renounceOwnership() returns()
func (_ProverRegistry *ProverRegistryTransactorSession) RenounceOwnership() (*types.Transaction, error) {
	return _ProverRegistry.Contract.RenounceOwnership(&_ProverRegistry.TransactOpts)
}

// SetPrice is a paid mutator transaction binding the contract method 0x4546356c.
//
// Solidity: function setPrice(uint128 pricePerGibDay, uint128 pricePerByteServed) returns()
func (_ProverRegistry *ProverRegistryTransactor) SetPrice(opts *bind.TransactOpts, pricePerGibDay *big.Int, pricePerByteServed *big.Int) (*types.Transaction, error) {
	return _ProverRegistry.contract.Transact(opts, "setPrice", pricePerGibDay, pricePerByteServed)
}

// SetPrice is a paid mutator transaction binding the contract method 0x4546356c.
//
// Solidity: function setPrice(uint128 pricePerGibDay, uint128 pricePerByteServed) returns()
func (_ProverRegistry *ProverRegistrySession) SetPrice(pricePerGibDay *big.Int, pricePerByteServed *big.Int) (*types.Transaction, error) {
	return _ProverRegistry.Contract.SetPrice(&_ProverRegistry.TransactOpts, pricePerGibDay, pricePerByteServed)
}

// SetPrice is a paid mutator transaction binding the contract method 0x4546356c.
//
// Solidity: function setPrice(uint128 pricePerGibDay, uint128 pricePerByteServed) returns()
func (_ProverRegistry *ProverRegistryTransactorSession) SetPrice(pricePerGibDay *big.Int, pricePerByteServed *big.Int) (*types.Transaction, error) {
	return _ProverRegistry.Contract.SetPrice(&_ProverRegistry.TransactOpts, pricePerGibDay, pricePerByteServed)
}

// TransferOwnership is a paid mutator transaction binding the contract method 0xf2fde38b.
//
// Solidity: function transferOwnership(address newOwner) returns()
func (_ProverRegistry *ProverRegistryTransactor) TransferOwnership(opts *bind.TransactOpts, newOwner common.Address) (*types.Transaction, error) {
	return _ProverRegistry.contract.Transact(opts, "transferOwnership", newOwner)
}

// TransferOwnership is a paid mutator transaction binding the contract method 0xf2fde38b.
//
// Solidity: function transferOwnership(address newOwner) returns()
func (_ProverRegistry *ProverRegistrySession) TransferOwnership(newOwner common.Address) (*types.Transaction, error) {
	return _ProverRegistry.Contract.TransferOwnership(&_ProverRegistry.TransactOpts, newOwner)
}

// TransferOwnership is a paid mutator transaction binding the contract method 0xf2fde38b.
//
// Solidity: function transferOwnership(address newOwner) returns()
func (_ProverRegistry *ProverRegistryTransactorSession) TransferOwnership(newOwner common.Address) (*types.Transaction, error) {
	return _ProverRegistry.Contract.TransferOwnership(&_ProverRegistry.TransactOpts, newOwner)
}

// UpdateEndpoint is a paid mutator transaction binding the contract method 0xad50b8a0.
//
// Solidity: function updateEndpoint(string endpoint, uint64 features, string metadata) returns()
func (_ProverRegistry *ProverRegistryTransactor) UpdateEndpoint(opts *bind.TransactOpts, endpoint string, features uint64, metadata string) (*types.Transaction, error) {
	return _ProverRegistry.contract.Transact(opts, "updateEndpoint", endpoint, features, metadata)
}

// UpdateEndpoint is a paid mutator transaction binding the contract method 0xad50b8a0.
//
// Solidity: function updateEndpoint(string endpoint, uint64 features, string metadata) returns()
func (_ProverRegistry *ProverRegistrySession) UpdateEndpoint(endpoint string, features uint64, metadata string) (*types.Transaction, error) {
	return _ProverRegistry.Contract.UpdateEndpoint(&_ProverRegistry.TransactOpts, endpoint, features, metadata)
}

// UpdateEndpoint is a paid mutator transaction binding the contract method 0xad50b8a0.
//
// Solidity: function updateEndpoint(string endpoint, uint64 features, string metadata) returns()
func (_ProverRegistry *ProverRegistryTransactorSession) UpdateEndpoint(endpoint string, features uint64, metadata string) (*types.Transaction, error) {
	return _ProverRegistry.Contract.UpdateEndpoint(&_ProverRegistry.TransactOpts, endpoint, features, metadata)
}

// ProverRegistryENSBoundIterator is returned from FilterENSBound and is used to iterate over the raw logs and unpacked data for ENSBound events raised by the ProverRegistry contract.
type ProverRegistryENSBoundIterator struct {
	Event *ProverRegistryENSBound // Event containing the contract specifics and raw log

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
func (it *ProverRegistryENSBoundIterator) Next() bool {
	// If the iterator failed, stop iterating
	if it.fail != nil {
		return false
	}
	// If the iterator completed, deliver directly whatever's available
	if it.done {
		select {
		case log := <-it.logs:
			it.Event = new(ProverRegistryENSBound)
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
		it.Event = new(ProverRegistryENSBound)
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
func (it *ProverRegistryENSBoundIterator) Error() error {
	return it.fail
}

// Close terminates the iteration process, releasing any pending underlying
// resources.
func (it *ProverRegistryENSBoundIterator) Close() error {
	it.sub.Unsubscribe()
	return nil
}

// ProverRegistryENSBound represents a ENSBound event raised by the ProverRegistry contract.
type ProverRegistryENSBound struct {
	Prover  common.Address
	EnsNode [32]byte
	Raw     types.Log // Blockchain specific contextual infos
}

// FilterENSBound is a free log retrieval operation binding the contract event 0xb9e6158028d8a0c7cf69ed321b5d7a2b6c2d5d18395df5667bca51f046e0c62d.
//
// Solidity: event ENSBound(address indexed prover, bytes32 ensNode)
func (_ProverRegistry *ProverRegistryFilterer) FilterENSBound(opts *bind.FilterOpts, prover []common.Address) (*ProverRegistryENSBoundIterator, error) {

	var proverRule []interface{}
	for _, proverItem := range prover {
		proverRule = append(proverRule, proverItem)
	}

	logs, sub, err := _ProverRegistry.contract.FilterLogs(opts, "ENSBound", proverRule)
	if err != nil {
		return nil, err
	}
	return &ProverRegistryENSBoundIterator{contract: _ProverRegistry.contract, event: "ENSBound", logs: logs, sub: sub}, nil
}

// WatchENSBound is a free log subscription operation binding the contract event 0xb9e6158028d8a0c7cf69ed321b5d7a2b6c2d5d18395df5667bca51f046e0c62d.
//
// Solidity: event ENSBound(address indexed prover, bytes32 ensNode)
func (_ProverRegistry *ProverRegistryFilterer) WatchENSBound(opts *bind.WatchOpts, sink chan<- *ProverRegistryENSBound, prover []common.Address) (event.Subscription, error) {

	var proverRule []interface{}
	for _, proverItem := range prover {
		proverRule = append(proverRule, proverItem)
	}

	logs, sub, err := _ProverRegistry.contract.WatchLogs(opts, "ENSBound", proverRule)
	if err != nil {
		return nil, err
	}
	return event.NewSubscription(func(quit <-chan struct{}) error {
		defer sub.Unsubscribe()
		for {
			select {
			case log := <-logs:
				// New log arrived, parse the event and forward to the user
				event := new(ProverRegistryENSBound)
				if err := _ProverRegistry.contract.UnpackLog(event, "ENSBound", log); err != nil {
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

// ParseENSBound is a log parse operation binding the contract event 0xb9e6158028d8a0c7cf69ed321b5d7a2b6c2d5d18395df5667bca51f046e0c62d.
//
// Solidity: event ENSBound(address indexed prover, bytes32 ensNode)
func (_ProverRegistry *ProverRegistryFilterer) ParseENSBound(log types.Log) (*ProverRegistryENSBound, error) {
	event := new(ProverRegistryENSBound)
	if err := _ProverRegistry.contract.UnpackLog(event, "ENSBound", log); err != nil {
		return nil, err
	}
	event.Raw = log
	return event, nil
}

// ProverRegistryOwnershipTransferredIterator is returned from FilterOwnershipTransferred and is used to iterate over the raw logs and unpacked data for OwnershipTransferred events raised by the ProverRegistry contract.
type ProverRegistryOwnershipTransferredIterator struct {
	Event *ProverRegistryOwnershipTransferred // Event containing the contract specifics and raw log

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
func (it *ProverRegistryOwnershipTransferredIterator) Next() bool {
	// If the iterator failed, stop iterating
	if it.fail != nil {
		return false
	}
	// If the iterator completed, deliver directly whatever's available
	if it.done {
		select {
		case log := <-it.logs:
			it.Event = new(ProverRegistryOwnershipTransferred)
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
		it.Event = new(ProverRegistryOwnershipTransferred)
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
func (it *ProverRegistryOwnershipTransferredIterator) Error() error {
	return it.fail
}

// Close terminates the iteration process, releasing any pending underlying
// resources.
func (it *ProverRegistryOwnershipTransferredIterator) Close() error {
	it.sub.Unsubscribe()
	return nil
}

// ProverRegistryOwnershipTransferred represents a OwnershipTransferred event raised by the ProverRegistry contract.
type ProverRegistryOwnershipTransferred struct {
	PreviousOwner common.Address
	NewOwner      common.Address
	Raw           types.Log // Blockchain specific contextual infos
}

// FilterOwnershipTransferred is a free log retrieval operation binding the contract event 0x8be0079c531659141344cd1fd0a4f28419497f9722a3daafe3b4186f6b6457e0.
//
// Solidity: event OwnershipTransferred(address indexed previousOwner, address indexed newOwner)
func (_ProverRegistry *ProverRegistryFilterer) FilterOwnershipTransferred(opts *bind.FilterOpts, previousOwner []common.Address, newOwner []common.Address) (*ProverRegistryOwnershipTransferredIterator, error) {

	var previousOwnerRule []interface{}
	for _, previousOwnerItem := range previousOwner {
		previousOwnerRule = append(previousOwnerRule, previousOwnerItem)
	}
	var newOwnerRule []interface{}
	for _, newOwnerItem := range newOwner {
		newOwnerRule = append(newOwnerRule, newOwnerItem)
	}

	logs, sub, err := _ProverRegistry.contract.FilterLogs(opts, "OwnershipTransferred", previousOwnerRule, newOwnerRule)
	if err != nil {
		return nil, err
	}
	return &ProverRegistryOwnershipTransferredIterator{contract: _ProverRegistry.contract, event: "OwnershipTransferred", logs: logs, sub: sub}, nil
}

// WatchOwnershipTransferred is a free log subscription operation binding the contract event 0x8be0079c531659141344cd1fd0a4f28419497f9722a3daafe3b4186f6b6457e0.
//
// Solidity: event OwnershipTransferred(address indexed previousOwner, address indexed newOwner)
func (_ProverRegistry *ProverRegistryFilterer) WatchOwnershipTransferred(opts *bind.WatchOpts, sink chan<- *ProverRegistryOwnershipTransferred, previousOwner []common.Address, newOwner []common.Address) (event.Subscription, error) {

	var previousOwnerRule []interface{}
	for _, previousOwnerItem := range previousOwner {
		previousOwnerRule = append(previousOwnerRule, previousOwnerItem)
	}
	var newOwnerRule []interface{}
	for _, newOwnerItem := range newOwner {
		newOwnerRule = append(newOwnerRule, newOwnerItem)
	}

	logs, sub, err := _ProverRegistry.contract.WatchLogs(opts, "OwnershipTransferred", previousOwnerRule, newOwnerRule)
	if err != nil {
		return nil, err
	}
	return event.NewSubscription(func(quit <-chan struct{}) error {
		defer sub.Unsubscribe()
		for {
			select {
			case log := <-logs:
				// New log arrived, parse the event and forward to the user
				event := new(ProverRegistryOwnershipTransferred)
				if err := _ProverRegistry.contract.UnpackLog(event, "OwnershipTransferred", log); err != nil {
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

// ParseOwnershipTransferred is a log parse operation binding the contract event 0x8be0079c531659141344cd1fd0a4f28419497f9722a3daafe3b4186f6b6457e0.
//
// Solidity: event OwnershipTransferred(address indexed previousOwner, address indexed newOwner)
func (_ProverRegistry *ProverRegistryFilterer) ParseOwnershipTransferred(log types.Log) (*ProverRegistryOwnershipTransferred, error) {
	event := new(ProverRegistryOwnershipTransferred)
	if err := _ProverRegistry.contract.UnpackLog(event, "OwnershipTransferred", log); err != nil {
		return nil, err
	}
	event.Raw = log
	return event, nil
}

// ProverRegistryPriceChangedIterator is returned from FilterPriceChanged and is used to iterate over the raw logs and unpacked data for PriceChanged events raised by the ProverRegistry contract.
type ProverRegistryPriceChangedIterator struct {
	Event *ProverRegistryPriceChanged // Event containing the contract specifics and raw log

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
func (it *ProverRegistryPriceChangedIterator) Next() bool {
	// If the iterator failed, stop iterating
	if it.fail != nil {
		return false
	}
	// If the iterator completed, deliver directly whatever's available
	if it.done {
		select {
		case log := <-it.logs:
			it.Event = new(ProverRegistryPriceChanged)
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
		it.Event = new(ProverRegistryPriceChanged)
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
func (it *ProverRegistryPriceChangedIterator) Error() error {
	return it.fail
}

// Close terminates the iteration process, releasing any pending underlying
// resources.
func (it *ProverRegistryPriceChangedIterator) Close() error {
	it.sub.Unsubscribe()
	return nil
}

// ProverRegistryPriceChanged represents a PriceChanged event raised by the ProverRegistry contract.
type ProverRegistryPriceChanged struct {
	Prover             common.Address
	PricePerGibDay     *big.Int
	PricePerByteServed *big.Int
	Raw                types.Log // Blockchain specific contextual infos
}

// FilterPriceChanged is a free log retrieval operation binding the contract event 0xafc22a6379ae3c9de76300bcf15145466f4ae54c3db09ed5492e957f44f6c56b.
//
// Solidity: event PriceChanged(address indexed prover, uint128 pricePerGibDay, uint128 pricePerByteServed)
func (_ProverRegistry *ProverRegistryFilterer) FilterPriceChanged(opts *bind.FilterOpts, prover []common.Address) (*ProverRegistryPriceChangedIterator, error) {

	var proverRule []interface{}
	for _, proverItem := range prover {
		proverRule = append(proverRule, proverItem)
	}

	logs, sub, err := _ProverRegistry.contract.FilterLogs(opts, "PriceChanged", proverRule)
	if err != nil {
		return nil, err
	}
	return &ProverRegistryPriceChangedIterator{contract: _ProverRegistry.contract, event: "PriceChanged", logs: logs, sub: sub}, nil
}

// WatchPriceChanged is a free log subscription operation binding the contract event 0xafc22a6379ae3c9de76300bcf15145466f4ae54c3db09ed5492e957f44f6c56b.
//
// Solidity: event PriceChanged(address indexed prover, uint128 pricePerGibDay, uint128 pricePerByteServed)
func (_ProverRegistry *ProverRegistryFilterer) WatchPriceChanged(opts *bind.WatchOpts, sink chan<- *ProverRegistryPriceChanged, prover []common.Address) (event.Subscription, error) {

	var proverRule []interface{}
	for _, proverItem := range prover {
		proverRule = append(proverRule, proverItem)
	}

	logs, sub, err := _ProverRegistry.contract.WatchLogs(opts, "PriceChanged", proverRule)
	if err != nil {
		return nil, err
	}
	return event.NewSubscription(func(quit <-chan struct{}) error {
		defer sub.Unsubscribe()
		for {
			select {
			case log := <-logs:
				// New log arrived, parse the event and forward to the user
				event := new(ProverRegistryPriceChanged)
				if err := _ProverRegistry.contract.UnpackLog(event, "PriceChanged", log); err != nil {
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

// ParsePriceChanged is a log parse operation binding the contract event 0xafc22a6379ae3c9de76300bcf15145466f4ae54c3db09ed5492e957f44f6c56b.
//
// Solidity: event PriceChanged(address indexed prover, uint128 pricePerGibDay, uint128 pricePerByteServed)
func (_ProverRegistry *ProverRegistryFilterer) ParsePriceChanged(log types.Log) (*ProverRegistryPriceChanged, error) {
	event := new(ProverRegistryPriceChanged)
	if err := _ProverRegistry.contract.UnpackLog(event, "PriceChanged", log); err != nil {
		return nil, err
	}
	event.Raw = log
	return event, nil
}

// ProverRegistryProverDeregisteredIterator is returned from FilterProverDeregistered and is used to iterate over the raw logs and unpacked data for ProverDeregistered events raised by the ProverRegistry contract.
type ProverRegistryProverDeregisteredIterator struct {
	Event *ProverRegistryProverDeregistered // Event containing the contract specifics and raw log

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
func (it *ProverRegistryProverDeregisteredIterator) Next() bool {
	// If the iterator failed, stop iterating
	if it.fail != nil {
		return false
	}
	// If the iterator completed, deliver directly whatever's available
	if it.done {
		select {
		case log := <-it.logs:
			it.Event = new(ProverRegistryProverDeregistered)
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
		it.Event = new(ProverRegistryProverDeregistered)
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
func (it *ProverRegistryProverDeregisteredIterator) Error() error {
	return it.fail
}

// Close terminates the iteration process, releasing any pending underlying
// resources.
func (it *ProverRegistryProverDeregisteredIterator) Close() error {
	it.sub.Unsubscribe()
	return nil
}

// ProverRegistryProverDeregistered represents a ProverDeregistered event raised by the ProverRegistry contract.
type ProverRegistryProverDeregistered struct {
	Prover common.Address
	Raw    types.Log // Blockchain specific contextual infos
}

// FilterProverDeregistered is a free log retrieval operation binding the contract event 0xba8692c6ccbc20602c00b106b981eb4c6e6b0957d070a30814d626956dba28c7.
//
// Solidity: event ProverDeregistered(address indexed prover)
func (_ProverRegistry *ProverRegistryFilterer) FilterProverDeregistered(opts *bind.FilterOpts, prover []common.Address) (*ProverRegistryProverDeregisteredIterator, error) {

	var proverRule []interface{}
	for _, proverItem := range prover {
		proverRule = append(proverRule, proverItem)
	}

	logs, sub, err := _ProverRegistry.contract.FilterLogs(opts, "ProverDeregistered", proverRule)
	if err != nil {
		return nil, err
	}
	return &ProverRegistryProverDeregisteredIterator{contract: _ProverRegistry.contract, event: "ProverDeregistered", logs: logs, sub: sub}, nil
}

// WatchProverDeregistered is a free log subscription operation binding the contract event 0xba8692c6ccbc20602c00b106b981eb4c6e6b0957d070a30814d626956dba28c7.
//
// Solidity: event ProverDeregistered(address indexed prover)
func (_ProverRegistry *ProverRegistryFilterer) WatchProverDeregistered(opts *bind.WatchOpts, sink chan<- *ProverRegistryProverDeregistered, prover []common.Address) (event.Subscription, error) {

	var proverRule []interface{}
	for _, proverItem := range prover {
		proverRule = append(proverRule, proverItem)
	}

	logs, sub, err := _ProverRegistry.contract.WatchLogs(opts, "ProverDeregistered", proverRule)
	if err != nil {
		return nil, err
	}
	return event.NewSubscription(func(quit <-chan struct{}) error {
		defer sub.Unsubscribe()
		for {
			select {
			case log := <-logs:
				// New log arrived, parse the event and forward to the user
				event := new(ProverRegistryProverDeregistered)
				if err := _ProverRegistry.contract.UnpackLog(event, "ProverDeregistered", log); err != nil {
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

// ParseProverDeregistered is a log parse operation binding the contract event 0xba8692c6ccbc20602c00b106b981eb4c6e6b0957d070a30814d626956dba28c7.
//
// Solidity: event ProverDeregistered(address indexed prover)
func (_ProverRegistry *ProverRegistryFilterer) ParseProverDeregistered(log types.Log) (*ProverRegistryProverDeregistered, error) {
	event := new(ProverRegistryProverDeregistered)
	if err := _ProverRegistry.contract.UnpackLog(event, "ProverDeregistered", log); err != nil {
		return nil, err
	}
	event.Raw = log
	return event, nil
}

// ProverRegistryProverRegisteredIterator is returned from FilterProverRegistered and is used to iterate over the raw logs and unpacked data for ProverRegistered events raised by the ProverRegistry contract.
type ProverRegistryProverRegisteredIterator struct {
	Event *ProverRegistryProverRegistered // Event containing the contract specifics and raw log

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
func (it *ProverRegistryProverRegisteredIterator) Next() bool {
	// If the iterator failed, stop iterating
	if it.fail != nil {
		return false
	}
	// If the iterator completed, deliver directly whatever's available
	if it.done {
		select {
		case log := <-it.logs:
			it.Event = new(ProverRegistryProverRegistered)
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
		it.Event = new(ProverRegistryProverRegistered)
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
func (it *ProverRegistryProverRegisteredIterator) Error() error {
	return it.fail
}

// Close terminates the iteration process, releasing any pending underlying
// resources.
func (it *ProverRegistryProverRegisteredIterator) Close() error {
	it.sub.Unsubscribe()
	return nil
}

// ProverRegistryProverRegistered represents a ProverRegistered event raised by the ProverRegistry contract.
type ProverRegistryProverRegistered struct {
	Prover   common.Address
	Endpoint string
	Features uint64
	Raw      types.Log // Blockchain specific contextual infos
}

// FilterProverRegistered is a free log retrieval operation binding the contract event 0xe08f02adbc8dd2124ea677973e24fc7ac7fe8d4074d6bb120d7fd06ad4ebdd58.
//
// Solidity: event ProverRegistered(address indexed prover, string endpoint, uint64 features)
func (_ProverRegistry *ProverRegistryFilterer) FilterProverRegistered(opts *bind.FilterOpts, prover []common.Address) (*ProverRegistryProverRegisteredIterator, error) {

	var proverRule []interface{}
	for _, proverItem := range prover {
		proverRule = append(proverRule, proverItem)
	}

	logs, sub, err := _ProverRegistry.contract.FilterLogs(opts, "ProverRegistered", proverRule)
	if err != nil {
		return nil, err
	}
	return &ProverRegistryProverRegisteredIterator{contract: _ProverRegistry.contract, event: "ProverRegistered", logs: logs, sub: sub}, nil
}

// WatchProverRegistered is a free log subscription operation binding the contract event 0xe08f02adbc8dd2124ea677973e24fc7ac7fe8d4074d6bb120d7fd06ad4ebdd58.
//
// Solidity: event ProverRegistered(address indexed prover, string endpoint, uint64 features)
func (_ProverRegistry *ProverRegistryFilterer) WatchProverRegistered(opts *bind.WatchOpts, sink chan<- *ProverRegistryProverRegistered, prover []common.Address) (event.Subscription, error) {

	var proverRule []interface{}
	for _, proverItem := range prover {
		proverRule = append(proverRule, proverItem)
	}

	logs, sub, err := _ProverRegistry.contract.WatchLogs(opts, "ProverRegistered", proverRule)
	if err != nil {
		return nil, err
	}
	return event.NewSubscription(func(quit <-chan struct{}) error {
		defer sub.Unsubscribe()
		for {
			select {
			case log := <-logs:
				// New log arrived, parse the event and forward to the user
				event := new(ProverRegistryProverRegistered)
				if err := _ProverRegistry.contract.UnpackLog(event, "ProverRegistered", log); err != nil {
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

// ParseProverRegistered is a log parse operation binding the contract event 0xe08f02adbc8dd2124ea677973e24fc7ac7fe8d4074d6bb120d7fd06ad4ebdd58.
//
// Solidity: event ProverRegistered(address indexed prover, string endpoint, uint64 features)
func (_ProverRegistry *ProverRegistryFilterer) ParseProverRegistered(log types.Log) (*ProverRegistryProverRegistered, error) {
	event := new(ProverRegistryProverRegistered)
	if err := _ProverRegistry.contract.UnpackLog(event, "ProverRegistered", log); err != nil {
		return nil, err
	}
	event.Raw = log
	return event, nil
}

// ProverRegistryProverUpdatedIterator is returned from FilterProverUpdated and is used to iterate over the raw logs and unpacked data for ProverUpdated events raised by the ProverRegistry contract.
type ProverRegistryProverUpdatedIterator struct {
	Event *ProverRegistryProverUpdated // Event containing the contract specifics and raw log

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
func (it *ProverRegistryProverUpdatedIterator) Next() bool {
	// If the iterator failed, stop iterating
	if it.fail != nil {
		return false
	}
	// If the iterator completed, deliver directly whatever's available
	if it.done {
		select {
		case log := <-it.logs:
			it.Event = new(ProverRegistryProverUpdated)
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
		it.Event = new(ProverRegistryProverUpdated)
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
func (it *ProverRegistryProverUpdatedIterator) Error() error {
	return it.fail
}

// Close terminates the iteration process, releasing any pending underlying
// resources.
func (it *ProverRegistryProverUpdatedIterator) Close() error {
	it.sub.Unsubscribe()
	return nil
}

// ProverRegistryProverUpdated represents a ProverUpdated event raised by the ProverRegistry contract.
type ProverRegistryProverUpdated struct {
	Prover   common.Address
	Endpoint string
	Features uint64
	Raw      types.Log // Blockchain specific contextual infos
}

// FilterProverUpdated is a free log retrieval operation binding the contract event 0xd38712fdbf30167b0c646ff649ac2873c75b20c0db08aa569e7c3eb37c18fa40.
//
// Solidity: event ProverUpdated(address indexed prover, string endpoint, uint64 features)
func (_ProverRegistry *ProverRegistryFilterer) FilterProverUpdated(opts *bind.FilterOpts, prover []common.Address) (*ProverRegistryProverUpdatedIterator, error) {

	var proverRule []interface{}
	for _, proverItem := range prover {
		proverRule = append(proverRule, proverItem)
	}

	logs, sub, err := _ProverRegistry.contract.FilterLogs(opts, "ProverUpdated", proverRule)
	if err != nil {
		return nil, err
	}
	return &ProverRegistryProverUpdatedIterator{contract: _ProverRegistry.contract, event: "ProverUpdated", logs: logs, sub: sub}, nil
}

// WatchProverUpdated is a free log subscription operation binding the contract event 0xd38712fdbf30167b0c646ff649ac2873c75b20c0db08aa569e7c3eb37c18fa40.
//
// Solidity: event ProverUpdated(address indexed prover, string endpoint, uint64 features)
func (_ProverRegistry *ProverRegistryFilterer) WatchProverUpdated(opts *bind.WatchOpts, sink chan<- *ProverRegistryProverUpdated, prover []common.Address) (event.Subscription, error) {

	var proverRule []interface{}
	for _, proverItem := range prover {
		proverRule = append(proverRule, proverItem)
	}

	logs, sub, err := _ProverRegistry.contract.WatchLogs(opts, "ProverUpdated", proverRule)
	if err != nil {
		return nil, err
	}
	return event.NewSubscription(func(quit <-chan struct{}) error {
		defer sub.Unsubscribe()
		for {
			select {
			case log := <-logs:
				// New log arrived, parse the event and forward to the user
				event := new(ProverRegistryProverUpdated)
				if err := _ProverRegistry.contract.UnpackLog(event, "ProverUpdated", log); err != nil {
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

// ParseProverUpdated is a log parse operation binding the contract event 0xd38712fdbf30167b0c646ff649ac2873c75b20c0db08aa569e7c3eb37c18fa40.
//
// Solidity: event ProverUpdated(address indexed prover, string endpoint, uint64 features)
func (_ProverRegistry *ProverRegistryFilterer) ParseProverUpdated(log types.Log) (*ProverRegistryProverUpdated, error) {
	event := new(ProverRegistryProverUpdated)
	if err := _ProverRegistry.contract.UnpackLog(event, "ProverUpdated", log); err != nil {
		return nil, err
	}
	event.Raw = log
	return event, nil
}
