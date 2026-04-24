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
	ABI: "[{\"type\":\"constructor\",\"inputs\":[],\"stateMutability\":\"nonpayable\"},{\"type\":\"function\",\"name\":\"FEATURE_HTTPS_SERVING\",\"inputs\":[],\"outputs\":[{\"name\":\"\",\"type\":\"uint64\",\"internalType\":\"uint64\"}],\"stateMutability\":\"view\"},{\"type\":\"function\",\"name\":\"FEATURE_PDP\",\"inputs\":[],\"outputs\":[{\"name\":\"\",\"type\":\"uint64\",\"internalType\":\"uint64\"}],\"stateMutability\":\"view\"},{\"type\":\"function\",\"name\":\"MAX_ENDPOINT_LENGTH\",\"inputs\":[],\"outputs\":[{\"name\":\"\",\"type\":\"uint256\",\"internalType\":\"uint256\"}],\"stateMutability\":\"view\"},{\"type\":\"function\",\"name\":\"MAX_METADATA_LENGTH\",\"inputs\":[],\"outputs\":[{\"name\":\"\",\"type\":\"uint256\",\"internalType\":\"uint256\"}],\"stateMutability\":\"view\"},{\"type\":\"function\",\"name\":\"bindENS\",\"inputs\":[{\"name\":\"ensNode\",\"type\":\"bytes32\",\"internalType\":\"bytes32\"}],\"outputs\":[],\"stateMutability\":\"nonpayable\"},{\"type\":\"function\",\"name\":\"deregister\",\"inputs\":[],\"outputs\":[],\"stateMutability\":\"nonpayable\"},{\"type\":\"function\",\"name\":\"getProver\",\"inputs\":[{\"name\":\"prover\",\"type\":\"address\",\"internalType\":\"address\"}],\"outputs\":[{\"name\":\"\",\"type\":\"tuple\",\"internalType\":\"structProverRegistry.Prover\",\"components\":[{\"name\":\"owner\",\"type\":\"address\",\"internalType\":\"address\"},{\"name\":\"endpoint\",\"type\":\"string\",\"internalType\":\"string\"},{\"name\":\"features\",\"type\":\"uint64\",\"internalType\":\"uint64\"},{\"name\":\"pricePerGibDay\",\"type\":\"uint128\",\"internalType\":\"uint128\"},{\"name\":\"pricePerByteServed\",\"type\":\"uint128\",\"internalType\":\"uint128\"},{\"name\":\"registeredAt\",\"type\":\"uint64\",\"internalType\":\"uint64\"},{\"name\":\"updatedAt\",\"type\":\"uint64\",\"internalType\":\"uint64\"},{\"name\":\"active\",\"type\":\"bool\",\"internalType\":\"bool\"},{\"name\":\"ensNode\",\"type\":\"bytes32\",\"internalType\":\"bytes32\"},{\"name\":\"metadata\",\"type\":\"string\",\"internalType\":\"string\"}]}],\"stateMutability\":\"view\"},{\"type\":\"function\",\"name\":\"isActive\",\"inputs\":[{\"name\":\"prover\",\"type\":\"address\",\"internalType\":\"address\"}],\"outputs\":[{\"name\":\"\",\"type\":\"bool\",\"internalType\":\"bool\"}],\"stateMutability\":\"view\"},{\"type\":\"function\",\"name\":\"known\",\"inputs\":[{\"name\":\"\",\"type\":\"address\",\"internalType\":\"address\"}],\"outputs\":[{\"name\":\"\",\"type\":\"bool\",\"internalType\":\"bool\"}],\"stateMutability\":\"view\"},{\"type\":\"function\",\"name\":\"listActive\",\"inputs\":[{\"name\":\"offset\",\"type\":\"uint256\",\"internalType\":\"uint256\"},{\"name\":\"limit\",\"type\":\"uint256\",\"internalType\":\"uint256\"}],\"outputs\":[{\"name\":\"result\",\"type\":\"address[]\",\"internalType\":\"address[]\"},{\"name\":\"nextOffset\",\"type\":\"uint256\",\"internalType\":\"uint256\"}],\"stateMutability\":\"view\"},{\"type\":\"function\",\"name\":\"owner\",\"inputs\":[],\"outputs\":[{\"name\":\"\",\"type\":\"address\",\"internalType\":\"address\"}],\"stateMutability\":\"view\"},{\"type\":\"function\",\"name\":\"proverAddresses\",\"inputs\":[{\"name\":\"\",\"type\":\"uint256\",\"internalType\":\"uint256\"}],\"outputs\":[{\"name\":\"\",\"type\":\"address\",\"internalType\":\"address\"}],\"stateMutability\":\"view\"},{\"type\":\"function\",\"name\":\"provers\",\"inputs\":[{\"name\":\"\",\"type\":\"address\",\"internalType\":\"address\"}],\"outputs\":[{\"name\":\"owner\",\"type\":\"address\",\"internalType\":\"address\"},{\"name\":\"endpoint\",\"type\":\"string\",\"internalType\":\"string\"},{\"name\":\"features\",\"type\":\"uint64\",\"internalType\":\"uint64\"},{\"name\":\"pricePerGibDay\",\"type\":\"uint128\",\"internalType\":\"uint128\"},{\"name\":\"pricePerByteServed\",\"type\":\"uint128\",\"internalType\":\"uint128\"},{\"name\":\"registeredAt\",\"type\":\"uint64\",\"internalType\":\"uint64\"},{\"name\":\"updatedAt\",\"type\":\"uint64\",\"internalType\":\"uint64\"},{\"name\":\"active\",\"type\":\"bool\",\"internalType\":\"bool\"},{\"name\":\"ensNode\",\"type\":\"bytes32\",\"internalType\":\"bytes32\"},{\"name\":\"metadata\",\"type\":\"string\",\"internalType\":\"string\"}],\"stateMutability\":\"view\"},{\"type\":\"function\",\"name\":\"register\",\"inputs\":[{\"name\":\"endpoint\",\"type\":\"string\",\"internalType\":\"string\"},{\"name\":\"features\",\"type\":\"uint64\",\"internalType\":\"uint64\"},{\"name\":\"pricePerGibDay\",\"type\":\"uint128\",\"internalType\":\"uint128\"},{\"name\":\"pricePerByteServed\",\"type\":\"uint128\",\"internalType\":\"uint128\"},{\"name\":\"metadata\",\"type\":\"string\",\"internalType\":\"string\"}],\"outputs\":[],\"stateMutability\":\"nonpayable\"},{\"type\":\"function\",\"name\":\"renounceOwnership\",\"inputs\":[],\"outputs\":[],\"stateMutability\":\"nonpayable\"},{\"type\":\"function\",\"name\":\"setPrice\",\"inputs\":[{\"name\":\"pricePerGibDay\",\"type\":\"uint128\",\"internalType\":\"uint128\"},{\"name\":\"pricePerByteServed\",\"type\":\"uint128\",\"internalType\":\"uint128\"}],\"outputs\":[],\"stateMutability\":\"nonpayable\"},{\"type\":\"function\",\"name\":\"supportsFeature\",\"inputs\":[{\"name\":\"prover\",\"type\":\"address\",\"internalType\":\"address\"},{\"name\":\"feature\",\"type\":\"uint64\",\"internalType\":\"uint64\"}],\"outputs\":[{\"name\":\"\",\"type\":\"bool\",\"internalType\":\"bool\"}],\"stateMutability\":\"view\"},{\"type\":\"function\",\"name\":\"totalRegistered\",\"inputs\":[],\"outputs\":[{\"name\":\"\",\"type\":\"uint256\",\"internalType\":\"uint256\"}],\"stateMutability\":\"view\"},{\"type\":\"function\",\"name\":\"transferOwnership\",\"inputs\":[{\"name\":\"newOwner\",\"type\":\"address\",\"internalType\":\"address\"}],\"outputs\":[],\"stateMutability\":\"nonpayable\"},{\"type\":\"function\",\"name\":\"updateEndpoint\",\"inputs\":[{\"name\":\"endpoint\",\"type\":\"string\",\"internalType\":\"string\"},{\"name\":\"features\",\"type\":\"uint64\",\"internalType\":\"uint64\"},{\"name\":\"metadata\",\"type\":\"string\",\"internalType\":\"string\"}],\"outputs\":[],\"stateMutability\":\"nonpayable\"},{\"type\":\"event\",\"name\":\"ENSBound\",\"inputs\":[{\"name\":\"prover\",\"type\":\"address\",\"indexed\":true,\"internalType\":\"address\"},{\"name\":\"ensNode\",\"type\":\"bytes32\",\"indexed\":false,\"internalType\":\"bytes32\"}],\"anonymous\":false},{\"type\":\"event\",\"name\":\"OwnershipTransferred\",\"inputs\":[{\"name\":\"previousOwner\",\"type\":\"address\",\"indexed\":true,\"internalType\":\"address\"},{\"name\":\"newOwner\",\"type\":\"address\",\"indexed\":true,\"internalType\":\"address\"}],\"anonymous\":false},{\"type\":\"event\",\"name\":\"PriceChanged\",\"inputs\":[{\"name\":\"prover\",\"type\":\"address\",\"indexed\":true,\"internalType\":\"address\"},{\"name\":\"pricePerGibDay\",\"type\":\"uint128\",\"indexed\":false,\"internalType\":\"uint128\"},{\"name\":\"pricePerByteServed\",\"type\":\"uint128\",\"indexed\":false,\"internalType\":\"uint128\"}],\"anonymous\":false},{\"type\":\"event\",\"name\":\"ProverDeregistered\",\"inputs\":[{\"name\":\"prover\",\"type\":\"address\",\"indexed\":true,\"internalType\":\"address\"}],\"anonymous\":false},{\"type\":\"event\",\"name\":\"ProverRegistered\",\"inputs\":[{\"name\":\"prover\",\"type\":\"address\",\"indexed\":true,\"internalType\":\"address\"},{\"name\":\"endpoint\",\"type\":\"string\",\"indexed\":false,\"internalType\":\"string\"},{\"name\":\"features\",\"type\":\"uint64\",\"indexed\":false,\"internalType\":\"uint64\"}],\"anonymous\":false},{\"type\":\"event\",\"name\":\"ProverUpdated\",\"inputs\":[{\"name\":\"prover\",\"type\":\"address\",\"indexed\":true,\"internalType\":\"address\"},{\"name\":\"endpoint\",\"type\":\"string\",\"indexed\":false,\"internalType\":\"string\"},{\"name\":\"features\",\"type\":\"uint64\",\"indexed\":false,\"internalType\":\"uint64\"}],\"anonymous\":false},{\"type\":\"error\",\"name\":\"AlreadyRegistered\",\"inputs\":[]},{\"type\":\"error\",\"name\":\"EndpointTooLong\",\"inputs\":[]},{\"type\":\"error\",\"name\":\"InvalidFeatures\",\"inputs\":[]},{\"type\":\"error\",\"name\":\"MetadataTooLong\",\"inputs\":[]},{\"type\":\"error\",\"name\":\"NotOwner\",\"inputs\":[]},{\"type\":\"error\",\"name\":\"NotRegistered\",\"inputs\":[]},{\"type\":\"error\",\"name\":\"OwnableInvalidOwner\",\"inputs\":[{\"name\":\"owner\",\"type\":\"address\",\"internalType\":\"address\"}]},{\"type\":\"error\",\"name\":\"OwnableUnauthorizedAccount\",\"inputs\":[{\"name\":\"account\",\"type\":\"address\",\"internalType\":\"address\"}]}]",
	Bin: "0x608080604052346071573315605e575f8054336001600160a01b0319821681178355916001600160a01b03909116907f8be0079c531659141344cd1fd0a4f28419497f9722a3daafe3b4186f6b6457e09080a361148490816100768239f35b631e4fbdf760e01b5f525f60045260245ffd5b5f80fdfe60806040526004361015610011575f80fd5b5f3560e01c80631dec844b14610e6857806322dfc94414610e2b57806343eb4d94146109f65780634546356c1461093957806349d2982b146108c4578063715018a61461088057806384691fab146108075780638da5cb5b146107e0578063927416c0146107c357806399105669146107a85780639f8a13d714610768578063a91a9702146106f5578063ad50b8a014610448578063aff5edb1146103c5578063c85d3eb0146103a9578063d2c7f2ac14610367578063e8868e9f1461034b578063e95aff8214610330578063f2fde38b146102be5763faab193c146100f5575f80fd5b346102ba5760203660031901126102ba5761010e610f54565b606061012060405161011f81610fa2565b5f81528260208201525f60408201525f838201525f60808201525f60a08201525f60c08201525f60e08201525f610100820152015260018060a01b03165f52600160205260405f2060405161017381610fa2565b81546001600160a01b031681526102b661018f60018401610fe1565b92602083019384526002810154604084019160018060401b0382168352606085019160018060801b039060401c1682526003810154608086019060018060801b038116825260a0870160018060401b038260801c16815260c088019160c01c825260ff6004850154169260e08901931515845261021b60066005870154966101008c0197885201610fe1565b956101208a019687526102546040519b8c9b60208d5260018060a01b0390511660208d01525161014060408d01526101608c0190611081565b97516001600160401b0390811660608c015290516001600160801b0390811660808c0152915190911660a08a01529051811660c089015290511660e08701525115156101008601525161012085015251838203601f1901610140850152611081565b0390f35b5f80fd5b346102ba5760203660031901126102ba576102d7610f54565b6102df6113c8565b6001600160a01b0316801561031d575f80546001600160a01b03198116831782556001600160a01b0316905f51602061142f5f395f51905f529080a3005b631e4fbdf760e01b5f525f60045260245ffd5b346102ba575f3660031901126102ba57602060405160028152f35b346102ba575f3660031901126102ba5760206040516108008152f35b346102ba5760203660031901126102ba576004356002548110156102ba576103906020916110e8565b905460405160039290921b1c6001600160a01b03168152f35b346102ba575f3660031901126102ba5760206040516102008152f35b346102ba575f3660031901126102ba57335f52600160205260405f20600481019081549160ff8316156104395760ff1990921690915561041290426001600160401b0316906003016111e3565b337fba8692c6ccbc20602c00b106b981eb4c6e6b0957d070a30814d626956dba28c75f80a2005b63aba4733960e01b5f5260045ffd5b346102ba5760603660031901126102ba576004356001600160401b0381116102ba576104789036906004016110a5565b906104816110d2565b6044356001600160401b0381116102ba576104a09036906004016110a5565b9390335f52600160205260405f209460ff600487015416156104395761020083116106e65761080081116106d75760018416156106c857600186016001600160401b038411610646576104fd846104f78354610f6a565b83611159565b5f84601f8111600114610665578061051c925f9161065a575b506111a8565b90555b6002860180546001600160401b0319166001600160401b03868116919091179091556006870192908211610646576105618261055b8554610f6a565b85611159565b5f90601f83116001146105ce57825f5160206113ef5f395f51905f529798936105ad9593610597935f926105c3575b50506111a8565b90555b426001600160401b0316906003016111e3565b6105be604051928392339684611206565b0390a2005b013590508a80610590565b601f19831691845f5260205f20925f5b81811061062e57509260019285925f5160206113ef5f395f51905f529a9b966105ad989610610615575b505050811b01905561059a565b01355f19600384901b60f8161c19169055898080610608565b919360206001819287870135815501950192016105de565b634e487b7160e01b5f52604160045260245ffd5b90508801358a610516565b50601f19851690825f528560205f20925f5b8181106106ad575010610694575b5050600184811b01905561051f565b8701355f19600387901b60f8161c191690558780610685565b8a840135855560019094019360209384019389935001610677565b630990044760e41b5f5260045ffd5b63216e38bd60e21b5f5260045ffd5b639049092560e01b5f5260045ffd5b346102ba5760403660031901126102ba576020610710610f54565b6107186110d2565b6001600160a01b039091165f9081526001835260409020600481015460ff1691908261074b575b50506040519015158152f35b6002015481166001600160401b039081169116149050828061073f565b346102ba5760203660031901126102ba576001600160a01b03610789610f54565b165f526001602052602060ff600460405f200154166040519015158152f35b346102ba575f3660031901126102ba57602060405160018152f35b346102ba575f3660031901126102ba576020600254604051908152f35b346102ba575f3660031901126102ba575f546040516001600160a01b039091168152602090f35b346102ba5760203660031901126102ba57600435335f52600160205260405f2060ff60048201541615610439576005810182905561085290426001600160401b0316906003016111e3565b6040519081527fb9e6158028d8a0c7cf69ed321b5d7a2b6c2d5d18395df5667bca51f046e0c62d60203392a2005b346102ba575f3660031901126102ba576108986113c8565b5f80546001600160a01b0319811682556001600160a01b03165f51602061142f5f395f51905f528280a3005b346102ba5760403660031901126102ba576108e3602435600435611277565b9060405190604082019260408352815180945260206060840192015f945b80861061091657505082935060208301520390f35b81516001600160a01b031684526001959095019460209384019390910190610901565b346102ba5760403660031901126102ba576004356001600160801b038116908181036102ba576024356001600160801b03811691908290036102ba57335f52600160205260405f209060ff6004830154161561043957816109a260039260026109c395016111ba565b0180546001600160801b03191683178155426001600160401b0316906111e3565b60405191825260208201527fafc22a6379ae3c9de76300bcf15145466f4ae54c3db09ed5492e957f44f6c56b60403392a2005b346102ba5760a03660031901126102ba576004356001600160401b0381116102ba57610a269036906004016110a5565b610a2e6110d2565b6044356001600160801b038116908190036102ba576064356001600160801b038116908190036102ba576084356001600160401b0381116102ba57610a779036906004016110a5565b90335f52600160205260ff600460405f20015416610e1c5761020086116106e65761080082116106d75760018516156106c85760405192426001600160401b0316610ac185610fa2565b338552610acf36898b611114565b95602086019687526040860160018060401b0389168152606087019182526080870193845260a087019183835260c08801938452610b1f60e0890196600188526101008a01985f8a523691611114565b6101208901908152335f90815260016020819052604090912099518a546001600160a01b0319166001600160a01b0391909116178a5599518051919a8a0191906001600160401b03821161064657610b7b8261055b8554610f6a565b602090601f8311600114610da85792610bae83610c3099979460069e9d9c9b999794610be6975f92610d9d5750506111a8565b90555b905160028a0180546001600160401b0319166001600160401b039290921691909117815590516001600160801b0316906111ba565b91516003870180549351600160801b600160c01b0360809190911b166001600160801b039092166001600160c01b031990941693909317178255516001600160401b0316906111e3565b600483019051151560ff8019835416911617905551600582015501905180519060018060401b03821161064657610c6b8261055b8554610f6a565b602090601f8311600114610d3a57610c8c92915f9183610d2f5750506111a8565b90555b335f52600360205260ff60405f20541615610cc5575b5f51602061140f5f395f51905f52916105be604051928392339684611206565b335f908152600360205260409020805460ff1916600117905560025491600160401b83101561064657610d0c8360015f51602061140f5f395f51905f5295016002556110e8565b81546001600160a01b0360039290921b91821b19163390911b1790559150610ca5565b015190508780610590565b90601f19831691845f52815f20925f5b818110610d855750908460019594939210610d6d575b505050811b019055610c8f565b01515f1960f88460031b161c19169055868080610d60565b92936020600181928786015181550195019301610d4a565b015190505f80610590565b90601f19831691845f52815f20925f5b818110610e0457509360069d9c9b9a989693610be6969360019383610c309d9b9810610dec575b505050811b019055610bb1565b01515f1960f88460031b161c191690555f8080610ddf565b92936020600181928786015181550195019301610db8565b630ea075bf60e21b5f5260045ffd5b346102ba5760203660031901126102ba576001600160a01b03610e4c610f54565b165f526003602052602060ff60405f2054166040519015158152f35b346102ba5760203660031901126102ba576001600160a01b03610e89610f54565b165f52600160205260405f2060018060a01b038154166102b6610eae60018401610fe1565b9260028101549060038101549060ff600482015416610ed4600660058401549301610fe1565b92610ef3604051988998895261014060208a0152610140890190611081565b6001600160401b0386811660408a8101919091526001600160801b0397901c871660608a01529582166080808a019190915282901c90951660a088015260c090811c90870152151560e0860152610100850152838203610120850152611081565b600435906001600160a01b03821682036102ba57565b90600182811c92168015610f98575b6020831014610f8457565b634e487b7160e01b5f52602260045260245ffd5b91607f1691610f79565b61014081019081106001600160401b0382111761064657604052565b601f909101601f19168101906001600160401b0382119082101761064657604052565b9060405191825f825492610ff484610f6a565b808452936001811690811561105f575060011461101b575b5061101992500383610fbe565b565b90505f9291925260205f20905f915b818310611043575050906020611019928201015f61100c565b602091935080600191548385890101520191019091849261102a565b90506020925061101994915060ff191682840152151560051b8201015f61100c565b805180835260209291819084018484015e5f828201840152601f01601f1916010190565b9181601f840112156102ba578235916001600160401b0383116102ba57602083818601950101116102ba57565b602435906001600160401b03821682036102ba57565b6002548110156111005760025f5260205f2001905f90565b634e487b7160e01b5f52603260045260245ffd5b9192916001600160401b038211610646576040519161113d601f8201601f191660200184610fbe565b8294818452818301116102ba578281602093845f960137010152565b601f821161116657505050565b5f5260205f20906020601f840160051c8301931061119e575b601f0160051c01905b818110611193575050565b5f8155600101611188565b909150819061117f565b8160011b915f199060031b1c19161790565b8054600160401b600160c01b03191660409290921b600160401b600160c01b0316919091179055565b80546001600160c01b031660c09290921b6001600160c01b031916919091179055565b918060609160209396959660408652816040870152838601375f828286010152601f80199101168301019360018060401b0316910152565b6001600160401b0381116106465760051b60200190565b5f1981146112635760010190565b634e487b7160e01b5f52601160045260245ffd5b6002549291838210156113ad57810192838211611263578084116113a5575b505f90805b84811061135a57506112ac8261123e565b916112ba6040519384610fbe565b8083526112c9601f199161123e565b013660208401375f905b8481106112e05750509190565b6112e9816110e8565b60018060a01b0391549060031b1c165f52600160205260ff600460405f20015416611317575b6001016112d3565b90611321826110e8565b905460039190911b1c6001600160a01b03169161133d82611255565b92845183101561110057602060019360051b86010152905061130f565b611363816110e8565b60018060a01b0391549060031b1c165f52600160205260ff600460405f20015416611391575b60010161129b565b9161139d600191611255565b929050611389565b92505f611296565b50506040516113bd602082610fbe565b5f81525f3681379190565b5f546001600160a01b031633036113db57565b63118cdaa760e01b5f523360045260245ffdfed38712fdbf30167b0c646ff649ac2873c75b20c0db08aa569e7c3eb37c18fa40e08f02adbc8dd2124ea677973e24fc7ac7fe8d4074d6bb120d7fd06ad4ebdd588be0079c531659141344cd1fd0a4f28419497f9722a3daafe3b4186f6b6457e0a2646970667358221220d928a8c4e8f343ec8f96235cad2e90b084734f626a072137e418cee806c1739064736f6c634300081e0033",
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
