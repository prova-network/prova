// SPDX-License-Identifier: MIT
// Generated from contracts/out/ProvaToken.sol/ProvaToken.json via abigen.
// Do not edit by hand; run ./scripts/gen-bindings.sh instead.

// Code generated - DO NOT EDIT.
// This file is a generated binding and any manual changes will be lost.

package provatoken

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

// ProvaTokenMetaData contains all meta data concerning the ProvaToken contract.
var ProvaTokenMetaData = &bind.MetaData{
	ABI: "[{\"type\":\"constructor\",\"inputs\":[{\"name\":\"treasury\",\"type\":\"address\",\"internalType\":\"address\"}],\"stateMutability\":\"nonpayable\"},{\"type\":\"function\",\"name\":\"DOMAIN_SEPARATOR\",\"inputs\":[],\"outputs\":[{\"name\":\"\",\"type\":\"bytes32\",\"internalType\":\"bytes32\"}],\"stateMutability\":\"view\"},{\"type\":\"function\",\"name\":\"TOTAL_SUPPLY\",\"inputs\":[],\"outputs\":[{\"name\":\"\",\"type\":\"uint256\",\"internalType\":\"uint256\"}],\"stateMutability\":\"view\"},{\"type\":\"function\",\"name\":\"allowance\",\"inputs\":[{\"name\":\"owner\",\"type\":\"address\",\"internalType\":\"address\"},{\"name\":\"spender\",\"type\":\"address\",\"internalType\":\"address\"}],\"outputs\":[{\"name\":\"\",\"type\":\"uint256\",\"internalType\":\"uint256\"}],\"stateMutability\":\"view\"},{\"type\":\"function\",\"name\":\"approve\",\"inputs\":[{\"name\":\"spender\",\"type\":\"address\",\"internalType\":\"address\"},{\"name\":\"value\",\"type\":\"uint256\",\"internalType\":\"uint256\"}],\"outputs\":[{\"name\":\"\",\"type\":\"bool\",\"internalType\":\"bool\"}],\"stateMutability\":\"nonpayable\"},{\"type\":\"function\",\"name\":\"balanceOf\",\"inputs\":[{\"name\":\"account\",\"type\":\"address\",\"internalType\":\"address\"}],\"outputs\":[{\"name\":\"\",\"type\":\"uint256\",\"internalType\":\"uint256\"}],\"stateMutability\":\"view\"},{\"type\":\"function\",\"name\":\"burn\",\"inputs\":[{\"name\":\"value\",\"type\":\"uint256\",\"internalType\":\"uint256\"}],\"outputs\":[],\"stateMutability\":\"nonpayable\"},{\"type\":\"function\",\"name\":\"burnFrom\",\"inputs\":[{\"name\":\"account\",\"type\":\"address\",\"internalType\":\"address\"},{\"name\":\"value\",\"type\":\"uint256\",\"internalType\":\"uint256\"}],\"outputs\":[],\"stateMutability\":\"nonpayable\"},{\"type\":\"function\",\"name\":\"decimals\",\"inputs\":[],\"outputs\":[{\"name\":\"\",\"type\":\"uint8\",\"internalType\":\"uint8\"}],\"stateMutability\":\"view\"},{\"type\":\"function\",\"name\":\"eip712Domain\",\"inputs\":[],\"outputs\":[{\"name\":\"fields\",\"type\":\"bytes1\",\"internalType\":\"bytes1\"},{\"name\":\"name\",\"type\":\"string\",\"internalType\":\"string\"},{\"name\":\"version\",\"type\":\"string\",\"internalType\":\"string\"},{\"name\":\"chainId\",\"type\":\"uint256\",\"internalType\":\"uint256\"},{\"name\":\"verifyingContract\",\"type\":\"address\",\"internalType\":\"address\"},{\"name\":\"salt\",\"type\":\"bytes32\",\"internalType\":\"bytes32\"},{\"name\":\"extensions\",\"type\":\"uint256[]\",\"internalType\":\"uint256[]\"}],\"stateMutability\":\"view\"},{\"type\":\"function\",\"name\":\"name\",\"inputs\":[],\"outputs\":[{\"name\":\"\",\"type\":\"string\",\"internalType\":\"string\"}],\"stateMutability\":\"view\"},{\"type\":\"function\",\"name\":\"nonces\",\"inputs\":[{\"name\":\"owner\",\"type\":\"address\",\"internalType\":\"address\"}],\"outputs\":[{\"name\":\"\",\"type\":\"uint256\",\"internalType\":\"uint256\"}],\"stateMutability\":\"view\"},{\"type\":\"function\",\"name\":\"permit\",\"inputs\":[{\"name\":\"owner\",\"type\":\"address\",\"internalType\":\"address\"},{\"name\":\"spender\",\"type\":\"address\",\"internalType\":\"address\"},{\"name\":\"value\",\"type\":\"uint256\",\"internalType\":\"uint256\"},{\"name\":\"deadline\",\"type\":\"uint256\",\"internalType\":\"uint256\"},{\"name\":\"v\",\"type\":\"uint8\",\"internalType\":\"uint8\"},{\"name\":\"r\",\"type\":\"bytes32\",\"internalType\":\"bytes32\"},{\"name\":\"s\",\"type\":\"bytes32\",\"internalType\":\"bytes32\"}],\"outputs\":[],\"stateMutability\":\"nonpayable\"},{\"type\":\"function\",\"name\":\"symbol\",\"inputs\":[],\"outputs\":[{\"name\":\"\",\"type\":\"string\",\"internalType\":\"string\"}],\"stateMutability\":\"view\"},{\"type\":\"function\",\"name\":\"totalSupply\",\"inputs\":[],\"outputs\":[{\"name\":\"\",\"type\":\"uint256\",\"internalType\":\"uint256\"}],\"stateMutability\":\"view\"},{\"type\":\"function\",\"name\":\"transfer\",\"inputs\":[{\"name\":\"to\",\"type\":\"address\",\"internalType\":\"address\"},{\"name\":\"value\",\"type\":\"uint256\",\"internalType\":\"uint256\"}],\"outputs\":[{\"name\":\"\",\"type\":\"bool\",\"internalType\":\"bool\"}],\"stateMutability\":\"nonpayable\"},{\"type\":\"function\",\"name\":\"transferFrom\",\"inputs\":[{\"name\":\"from\",\"type\":\"address\",\"internalType\":\"address\"},{\"name\":\"to\",\"type\":\"address\",\"internalType\":\"address\"},{\"name\":\"value\",\"type\":\"uint256\",\"internalType\":\"uint256\"}],\"outputs\":[{\"name\":\"\",\"type\":\"bool\",\"internalType\":\"bool\"}],\"stateMutability\":\"nonpayable\"},{\"type\":\"event\",\"name\":\"Approval\",\"inputs\":[{\"name\":\"owner\",\"type\":\"address\",\"indexed\":true,\"internalType\":\"address\"},{\"name\":\"spender\",\"type\":\"address\",\"indexed\":true,\"internalType\":\"address\"},{\"name\":\"value\",\"type\":\"uint256\",\"indexed\":false,\"internalType\":\"uint256\"}],\"anonymous\":false},{\"type\":\"event\",\"name\":\"EIP712DomainChanged\",\"inputs\":[],\"anonymous\":false},{\"type\":\"event\",\"name\":\"Transfer\",\"inputs\":[{\"name\":\"from\",\"type\":\"address\",\"indexed\":true,\"internalType\":\"address\"},{\"name\":\"to\",\"type\":\"address\",\"indexed\":true,\"internalType\":\"address\"},{\"name\":\"value\",\"type\":\"uint256\",\"indexed\":false,\"internalType\":\"uint256\"}],\"anonymous\":false},{\"type\":\"error\",\"name\":\"ECDSAInvalidSignature\",\"inputs\":[]},{\"type\":\"error\",\"name\":\"ECDSAInvalidSignatureLength\",\"inputs\":[{\"name\":\"length\",\"type\":\"uint256\",\"internalType\":\"uint256\"}]},{\"type\":\"error\",\"name\":\"ECDSAInvalidSignatureS\",\"inputs\":[{\"name\":\"s\",\"type\":\"bytes32\",\"internalType\":\"bytes32\"}]},{\"type\":\"error\",\"name\":\"ERC20InsufficientAllowance\",\"inputs\":[{\"name\":\"spender\",\"type\":\"address\",\"internalType\":\"address\"},{\"name\":\"allowance\",\"type\":\"uint256\",\"internalType\":\"uint256\"},{\"name\":\"needed\",\"type\":\"uint256\",\"internalType\":\"uint256\"}]},{\"type\":\"error\",\"name\":\"ERC20InsufficientBalance\",\"inputs\":[{\"name\":\"sender\",\"type\":\"address\",\"internalType\":\"address\"},{\"name\":\"balance\",\"type\":\"uint256\",\"internalType\":\"uint256\"},{\"name\":\"needed\",\"type\":\"uint256\",\"internalType\":\"uint256\"}]},{\"type\":\"error\",\"name\":\"ERC20InvalidApprover\",\"inputs\":[{\"name\":\"approver\",\"type\":\"address\",\"internalType\":\"address\"}]},{\"type\":\"error\",\"name\":\"ERC20InvalidReceiver\",\"inputs\":[{\"name\":\"receiver\",\"type\":\"address\",\"internalType\":\"address\"}]},{\"type\":\"error\",\"name\":\"ERC20InvalidSender\",\"inputs\":[{\"name\":\"sender\",\"type\":\"address\",\"internalType\":\"address\"}]},{\"type\":\"error\",\"name\":\"ERC20InvalidSpender\",\"inputs\":[{\"name\":\"spender\",\"type\":\"address\",\"internalType\":\"address\"}]},{\"type\":\"error\",\"name\":\"ERC2612ExpiredSignature\",\"inputs\":[{\"name\":\"deadline\",\"type\":\"uint256\",\"internalType\":\"uint256\"}]},{\"type\":\"error\",\"name\":\"ERC2612InvalidSigner\",\"inputs\":[{\"name\":\"signer\",\"type\":\"address\",\"internalType\":\"address\"},{\"name\":\"owner\",\"type\":\"address\",\"internalType\":\"address\"}]},{\"type\":\"error\",\"name\":\"InvalidAccountNonce\",\"inputs\":[{\"name\":\"account\",\"type\":\"address\",\"internalType\":\"address\"},{\"name\":\"currentNonce\",\"type\":\"uint256\",\"internalType\":\"uint256\"}]},{\"type\":\"error\",\"name\":\"InvalidShortString\",\"inputs\":[]},{\"type\":\"error\",\"name\":\"StringTooLong\",\"inputs\":[{\"name\":\"str\",\"type\":\"string\",\"internalType\":\"string\"}]}]",
	Bin: "0x6101608060405234610516576020816115338038038091610020828561051a565b83398101031261051657516001600160a01b038116908190036105165760405161004b60408261051a565b60058152602081016450726f766160d81b81526040519061006d60408361051a565b600582526450726f766160d81b60208301526040519261008e60408561051a565b600584526450524f564160d81b6020850152604051936100af60408661051a565b60018552603160f81b60208601908152845190946001600160401b0382116104195760035490600182811c9216801561050c575b60208310146103fb5781601f84931161049e575b50602090601f8311600114610438575f9261042d575b50508160011b915f199060031b1c1916176003555b8051906001600160401b0382116104195760045490600182811c9216801561040f575b60208310146103fb5781601f84931161038d575b50602090601f8311600114610327575f9261031c575b50508160011b915f199060031b1c1916176004555b61018d8161053d565b6101205261019a846106c4565b61014052519020918260e05251902080610100524660a0526040519060208201927f8b73c3c69bb8fe3d512ecc4cf759cc79239f7b179b0ffacaa9a75d522b39400f8452604083015260608201524660808201523060a082015260a0815261020360c08261051a565b5190206080523060c05280156102e8576002546b033b2e3c9fd0803ce800000081018091116102d457600255805f525f60205260405f206b033b2e3c9fd0803ce800000081540190555f7fddf252ad1be2c89b69c2b068fc378daa952ba7f163c4a11628f55a4df523b3ef60206040516b033b2e3c9fd0803ce80000008152a3604051610d3690816107fd823960805181610998015260a05181610a55015260c05181610962015260e051816109e701526101005181610a0d015261012051816103b6015261014051816103df0152f35b634e487b7160e01b5f52601160045260245ffd5b60405162461bcd60e51b815260206004820152600c60248201526b5a65726f206164647265737360a01b6044820152606490fd5b015190505f8061016f565b60045f9081528281209350601f198516905b818110610375575090846001959493921061035d575b505050811b01600455610184565b01515f1960f88460031b161c191690555f808061034f565b92936020600181928786015181550195019301610339565b60045f529091507f8a35acfbc15ff81a39ae7d344fd709f28e8600b4aa8c65c6b64bfe7fe36bd19b601f840160051c810191602085106103f1575b90601f859493920160051c01905b8181106103e35750610159565b5f81558493506001016103d6565b90915081906103c8565b634e487b7160e01b5f52602260045260245ffd5b91607f1691610145565b634e487b7160e01b5f52604160045260245ffd5b015190505f8061010d565b60035f9081528281209350601f198516905b818110610486575090846001959493921061046e575b505050811b01600355610122565b01515f1960f88460031b161c191690555f8080610460565b9293602060018192878601518155019501930161044a565b60035f529091507fc2575a0e9e593c00f959f8c92f12db2869c3395a3b0502d05e2516446f71f85b601f840160051c81019160208510610502575b90601f859493920160051c01905b8181106104f457506100f7565b5f81558493506001016104e7565b90915081906104d9565b91607f16916100e3565b5f80fd5b601f909101601f19168101906001600160401b0382119082101761041957604052565b908151602081105f146105b7575090601f815111610577576020815191015160208210610568571790565b5f198260200360031b1b161790565b604460209160405192839163305a27a960e01b83528160048401528051918291826024860152018484015e5f828201840152601f01601f19168101030190fd5b6001600160401b03811161041957600554600181811c911680156106ba575b60208210146103fb57601f8111610687575b50602092601f821160011461062657928192935f9261061b575b50508160011b915f199060031b1c19161760055560ff90565b015190505f80610602565b601f1982169360055f52805f20915f5b86811061066f5750836001959610610657575b505050811b0160055560ff90565b01515f1960f88460031b161c191690555f8080610649565b91926020600181928685015181550194019201610636565b60055f52601f60205f20910160051c810190601f830160051c015b8181106106af57506105e8565b5f81556001016106a2565b90607f16906105d6565b908151602081105f146106ef575090601f815111610577576020815191015160208210610568571790565b6001600160401b03811161041957600654600181811c911680156107f2575b60208210146103fb57601f81116107bf575b50602092601f821160011461075e57928192935f92610753575b50508160011b915f199060031b1c19161760065560ff90565b015190505f8061073a565b601f1982169360065f52805f20915f5b8681106107a7575083600195961061078f575b505050811b0160065560ff90565b01515f1960f88460031b161c191690555f8080610781565b9192602060018192868501518155019401920161076e565b60065f52601f60205f20910160051c810190601f830160051c015b8181106107e75750610720565b5f81556001016107da565b90607f169061070e56fe6080806040526004361015610012575f80fd5b5f3560e01c90816306fdde031461060a57508063095ea7b3146105e457806318160ddd146105c757806323b872dd1461058f578063313ce567146105745780633644e5151461055257806342966c681461053557806370a08231146104fe57806379cc6790146104ce5780637ecebe001461049657806384b0196e1461039e578063902d55a51461037857806395d89b4114610296578063a9059cbb14610265578063d505accf146101205763dd62ed3e146100cc575f80fd5b3461011c57604036600319011261011c576100e56106d0565b6100ed6106e6565b6001600160a01b039182165f908152600160209081526040808320949093168252928352819020549051908152f35b5f80fd5b3461011c5760e036600319011261011c576101396106d0565b6101416106e6565b604435906064359260843560ff8116810361011c578442116102525761021561021e9160018060a01b03841696875f52600760205260405f20908154916001830190556040519060208201927f6e71edae12b1b97f4d1f60370fef10105fa2faae0126114a169c64845d6126c984528a604084015260018060a01b038916606084015289608084015260a083015260c082015260c081526101e360e0826107b5565b5190206101ee61095f565b906040519161190160f01b83526002830152602282015260c43591604260a4359220610be9565b90929192610c6c565b6001600160a01b031684810361023b57506102399350610aec565b005b84906325c0072360e11b5f5260045260245260445ffd5b8463313c898160e11b5f5260045260245ffd5b3461011c57604036600319011261011c5761028b6102816106d0565b60243590336108b5565b602060405160018152f35b3461011c575f36600319011261011c576040515f6004546102b6816106fc565b808452906001811690811561035457506001146102f6575b6102f2836102de818503826107b5565b6040519182916020835260208301906106ac565b0390f35b60045f9081527f8a35acfbc15ff81a39ae7d344fd709f28e8600b4aa8c65c6b64bfe7fe36bd19b939250905b80821061033a575090915081016020016102de6102ce565b919260018160209254838588010152019101909291610322565b60ff191660208086019190915291151560051b840190910191506102de90506102ce565b3461011c575f36600319011261011c57604051676765c793fa10079d601b1b8152602090f35b3461011c575f36600319011261011c5761043a6103da7f0000000000000000000000000000000000000000000000000000000000000000610b4f565b6104037f0000000000000000000000000000000000000000000000000000000000000000610bb2565b60206104486040519261041683856107b5565b5f84525f368137604051958695600f60f81b875260e08588015260e08701906106ac565b9085820360408701526106ac565b4660608501523060808501525f60a085015283810360c08501528180845192838152019301915f5b82811061047f57505050500390f35b835185528695509381019392810192600101610470565b3461011c57602036600319011261011c576001600160a01b036104b76106d0565b165f526007602052602060405f2054604051908152f35b3461011c57604036600319011261011c576102396104ea6106d0565b602435906104f98233836107ec565b610a7b565b3461011c57602036600319011261011c576001600160a01b0361051f6106d0565b165f525f602052602060405f2054604051908152f35b3461011c57602036600319011261011c5761023960043533610a7b565b3461011c575f36600319011261011c57602061056c61095f565b604051908152f35b3461011c575f36600319011261011c57602060405160128152f35b3461011c57606036600319011261011c5761028b6105ab6106d0565b6105b36106e6565b604435916105c28333836107ec565b6108b5565b3461011c575f36600319011261011c576020600254604051908152f35b3461011c57604036600319011261011c5761028b6106006106d0565b6024359033610aec565b3461011c575f36600319011261011c575f600354610627816106fc565b8084529060018116908115610354575060011461064e576102f2836102de818503826107b5565b60035f9081527fc2575a0e9e593c00f959f8c92f12db2869c3395a3b0502d05e2516446f71f85b939250905b808210610692575090915081016020016102de6102ce565b91926001816020925483858801015201910190929161067a565b805180835260209291819084018484015e5f828201840152601f01601f1916010190565b600435906001600160a01b038216820361011c57565b602435906001600160a01b038216820361011c57565b90600182811c9216801561072a575b602083101461071657565b634e487b7160e01b5f52602260045260245ffd5b91607f169161070b565b5f9291815491610743836106fc565b8083529260018116908115610798575060011461075f57505050565b5f9081526020812093945091925b83831061077e575060209250010190565b60018160209294939454838587010152019101919061076d565b915050602093945060ff929192191683830152151560051b010190565b601f909101601f19168101906001600160401b038211908210176107d857604052565b634e487b7160e01b5f52604160045260245ffd5b6001600160a01b039081165f818152600160209081526040808320948616835293905291909120549291905f198410610826575b50505050565b82841061089257801561087f576001600160a01b0382161561086c575f52600160205260405f209060018060a01b03165f5260205260405f20910390555f808080610820565b634a1406b160e11b5f525f60045260245ffd5b63e602df0560e01b5f525f60045260245ffd5b508290637dc7a0d960e11b5f5260018060a01b031660045260245260445260645ffd5b6001600160a01b031690811561094c576001600160a01b031691821561093957815f525f60205260405f205481811061092057815f516020610ce15f395f51905f5292602092855f525f84520360405f2055845f525f825260405f20818154019055604051908152a3565b8263391434e360e21b5f5260045260245260445260645ffd5b63ec442f0560e01b5f525f60045260245ffd5b634b637e8f60e11b5f525f60045260245ffd5b307f00000000000000000000000000000000000000000000000000000000000000006001600160a01b03161480610a52575b156109ba577f000000000000000000000000000000000000000000000000000000000000000090565b60405160208101907f8b73c3c69bb8fe3d512ecc4cf759cc79239f7b179b0ffacaa9a75d522b39400f82527f000000000000000000000000000000000000000000000000000000000000000060408201527f000000000000000000000000000000000000000000000000000000000000000060608201524660808201523060a082015260a08152610a4c60c0826107b5565b51902090565b507f00000000000000000000000000000000000000000000000000000000000000004614610991565b9091906001600160a01b0316801561094c57805f525f60205260405f2054838110610ad2576020845f94955f516020610ce15f395f51905f52938587528684520360408620558060025403600255604051908152a3565b915063391434e360e21b5f5260045260245260445260645ffd5b6001600160a01b031690811561087f576001600160a01b031691821561086c5760207f8c5be1e5ebec7d5bd14f71427d1e84f3dd0314c0f7b2291e5b200ac8c7c3b92591835f526001825260405f20855f5282528060405f2055604051908152a3565b60ff8114610b955760ff811690601f8211610b865760405191610b736040846107b5565b6020808452838101919036833783525290565b632cd44ac360e21b5f5260045ffd5b50604051610baf81610ba8816005610734565b03826107b5565b90565b60ff8114610bd65760ff811690601f8211610b865760405191610b736040846107b5565b50604051610baf81610ba8816006610734565b91906fa2a8918ca85bafe22016d0b997e4df60600160ff1b038411610c61579160209360809260ff5f9560405194855216868401526040830152606082015282805260015afa15610c56575f516001600160a01b03811615610c4c57905f905f90565b505f906001905f90565b6040513d5f823e3d90fd5b5050505f9160039190565b6004811015610ccc5780610c7e575050565b60018103610c955763f645eedf60e01b5f5260045ffd5b60028103610cb0575063fce698f760e01b5f5260045260245ffd5b600314610cba5750565b6335e2f38360e21b5f5260045260245ffd5b634e487b7160e01b5f52602160045260245ffdfeddf252ad1be2c89b69c2b068fc378daa952ba7f163c4a11628f55a4df523b3efa2646970667358221220b8385b0d874c2c1b41510561cb719a6d3b9a70e4d2c0c1a2b75427ccfc14aaf764736f6c634300081e0033",
}

// ProvaTokenABI is the input ABI used to generate the binding from.
// Deprecated: Use ProvaTokenMetaData.ABI instead.
var ProvaTokenABI = ProvaTokenMetaData.ABI

// ProvaTokenBin is the compiled bytecode used for deploying new contracts.
// Deprecated: Use ProvaTokenMetaData.Bin instead.
var ProvaTokenBin = ProvaTokenMetaData.Bin

// DeployProvaToken deploys a new Ethereum contract, binding an instance of ProvaToken to it.
func DeployProvaToken(auth *bind.TransactOpts, backend bind.ContractBackend, treasury common.Address) (common.Address, *types.Transaction, *ProvaToken, error) {
	parsed, err := ProvaTokenMetaData.GetAbi()
	if err != nil {
		return common.Address{}, nil, nil, err
	}
	if parsed == nil {
		return common.Address{}, nil, nil, errors.New("GetABI returned nil")
	}

	address, tx, contract, err := bind.DeployContract(auth, *parsed, common.FromHex(ProvaTokenBin), backend, treasury)
	if err != nil {
		return common.Address{}, nil, nil, err
	}
	return address, tx, &ProvaToken{ProvaTokenCaller: ProvaTokenCaller{contract: contract}, ProvaTokenTransactor: ProvaTokenTransactor{contract: contract}, ProvaTokenFilterer: ProvaTokenFilterer{contract: contract}}, nil
}

// ProvaToken is an auto generated Go binding around an Ethereum contract.
type ProvaToken struct {
	ProvaTokenCaller     // Read-only binding to the contract
	ProvaTokenTransactor // Write-only binding to the contract
	ProvaTokenFilterer   // Log filterer for contract events
}

// ProvaTokenCaller is an auto generated read-only Go binding around an Ethereum contract.
type ProvaTokenCaller struct {
	contract *bind.BoundContract // Generic contract wrapper for the low level calls
}

// ProvaTokenTransactor is an auto generated write-only Go binding around an Ethereum contract.
type ProvaTokenTransactor struct {
	contract *bind.BoundContract // Generic contract wrapper for the low level calls
}

// ProvaTokenFilterer is an auto generated log filtering Go binding around an Ethereum contract events.
type ProvaTokenFilterer struct {
	contract *bind.BoundContract // Generic contract wrapper for the low level calls
}

// ProvaTokenSession is an auto generated Go binding around an Ethereum contract,
// with pre-set call and transact options.
type ProvaTokenSession struct {
	Contract     *ProvaToken       // Generic contract binding to set the session for
	CallOpts     bind.CallOpts     // Call options to use throughout this session
	TransactOpts bind.TransactOpts // Transaction auth options to use throughout this session
}

// ProvaTokenCallerSession is an auto generated read-only Go binding around an Ethereum contract,
// with pre-set call options.
type ProvaTokenCallerSession struct {
	Contract *ProvaTokenCaller // Generic contract caller binding to set the session for
	CallOpts bind.CallOpts     // Call options to use throughout this session
}

// ProvaTokenTransactorSession is an auto generated write-only Go binding around an Ethereum contract,
// with pre-set transact options.
type ProvaTokenTransactorSession struct {
	Contract     *ProvaTokenTransactor // Generic contract transactor binding to set the session for
	TransactOpts bind.TransactOpts     // Transaction auth options to use throughout this session
}

// ProvaTokenRaw is an auto generated low-level Go binding around an Ethereum contract.
type ProvaTokenRaw struct {
	Contract *ProvaToken // Generic contract binding to access the raw methods on
}

// ProvaTokenCallerRaw is an auto generated low-level read-only Go binding around an Ethereum contract.
type ProvaTokenCallerRaw struct {
	Contract *ProvaTokenCaller // Generic read-only contract binding to access the raw methods on
}

// ProvaTokenTransactorRaw is an auto generated low-level write-only Go binding around an Ethereum contract.
type ProvaTokenTransactorRaw struct {
	Contract *ProvaTokenTransactor // Generic write-only contract binding to access the raw methods on
}

// NewProvaToken creates a new instance of ProvaToken, bound to a specific deployed contract.
func NewProvaToken(address common.Address, backend bind.ContractBackend) (*ProvaToken, error) {
	contract, err := bindProvaToken(address, backend, backend, backend)
	if err != nil {
		return nil, err
	}
	return &ProvaToken{ProvaTokenCaller: ProvaTokenCaller{contract: contract}, ProvaTokenTransactor: ProvaTokenTransactor{contract: contract}, ProvaTokenFilterer: ProvaTokenFilterer{contract: contract}}, nil
}

// NewProvaTokenCaller creates a new read-only instance of ProvaToken, bound to a specific deployed contract.
func NewProvaTokenCaller(address common.Address, caller bind.ContractCaller) (*ProvaTokenCaller, error) {
	contract, err := bindProvaToken(address, caller, nil, nil)
	if err != nil {
		return nil, err
	}
	return &ProvaTokenCaller{contract: contract}, nil
}

// NewProvaTokenTransactor creates a new write-only instance of ProvaToken, bound to a specific deployed contract.
func NewProvaTokenTransactor(address common.Address, transactor bind.ContractTransactor) (*ProvaTokenTransactor, error) {
	contract, err := bindProvaToken(address, nil, transactor, nil)
	if err != nil {
		return nil, err
	}
	return &ProvaTokenTransactor{contract: contract}, nil
}

// NewProvaTokenFilterer creates a new log filterer instance of ProvaToken, bound to a specific deployed contract.
func NewProvaTokenFilterer(address common.Address, filterer bind.ContractFilterer) (*ProvaTokenFilterer, error) {
	contract, err := bindProvaToken(address, nil, nil, filterer)
	if err != nil {
		return nil, err
	}
	return &ProvaTokenFilterer{contract: contract}, nil
}

// bindProvaToken binds a generic wrapper to an already deployed contract.
func bindProvaToken(address common.Address, caller bind.ContractCaller, transactor bind.ContractTransactor, filterer bind.ContractFilterer) (*bind.BoundContract, error) {
	parsed, err := ProvaTokenMetaData.GetAbi()
	if err != nil {
		return nil, err
	}
	return bind.NewBoundContract(address, *parsed, caller, transactor, filterer), nil
}

// Call invokes the (constant) contract method with params as input values and
// sets the output to result. The result type might be a single field for simple
// returns, a slice of interfaces for anonymous returns and a struct for named
// returns.
func (_ProvaToken *ProvaTokenRaw) Call(opts *bind.CallOpts, result *[]interface{}, method string, params ...interface{}) error {
	return _ProvaToken.Contract.ProvaTokenCaller.contract.Call(opts, result, method, params...)
}

// Transfer initiates a plain transaction to move funds to the contract, calling
// its default method if one is available.
func (_ProvaToken *ProvaTokenRaw) Transfer(opts *bind.TransactOpts) (*types.Transaction, error) {
	return _ProvaToken.Contract.ProvaTokenTransactor.contract.Transfer(opts)
}

// Transact invokes the (paid) contract method with params as input values.
func (_ProvaToken *ProvaTokenRaw) Transact(opts *bind.TransactOpts, method string, params ...interface{}) (*types.Transaction, error) {
	return _ProvaToken.Contract.ProvaTokenTransactor.contract.Transact(opts, method, params...)
}

// Call invokes the (constant) contract method with params as input values and
// sets the output to result. The result type might be a single field for simple
// returns, a slice of interfaces for anonymous returns and a struct for named
// returns.
func (_ProvaToken *ProvaTokenCallerRaw) Call(opts *bind.CallOpts, result *[]interface{}, method string, params ...interface{}) error {
	return _ProvaToken.Contract.contract.Call(opts, result, method, params...)
}

// Transfer initiates a plain transaction to move funds to the contract, calling
// its default method if one is available.
func (_ProvaToken *ProvaTokenTransactorRaw) Transfer(opts *bind.TransactOpts) (*types.Transaction, error) {
	return _ProvaToken.Contract.contract.Transfer(opts)
}

// Transact invokes the (paid) contract method with params as input values.
func (_ProvaToken *ProvaTokenTransactorRaw) Transact(opts *bind.TransactOpts, method string, params ...interface{}) (*types.Transaction, error) {
	return _ProvaToken.Contract.contract.Transact(opts, method, params...)
}

// DOMAINSEPARATOR is a free data retrieval call binding the contract method 0x3644e515.
//
// Solidity: function DOMAIN_SEPARATOR() view returns(bytes32)
func (_ProvaToken *ProvaTokenCaller) DOMAINSEPARATOR(opts *bind.CallOpts) ([32]byte, error) {
	var out []interface{}
	err := _ProvaToken.contract.Call(opts, &out, "DOMAIN_SEPARATOR")

	if err != nil {
		return *new([32]byte), err
	}

	out0 := *abi.ConvertType(out[0], new([32]byte)).(*[32]byte)

	return out0, err

}

// DOMAINSEPARATOR is a free data retrieval call binding the contract method 0x3644e515.
//
// Solidity: function DOMAIN_SEPARATOR() view returns(bytes32)
func (_ProvaToken *ProvaTokenSession) DOMAINSEPARATOR() ([32]byte, error) {
	return _ProvaToken.Contract.DOMAINSEPARATOR(&_ProvaToken.CallOpts)
}

// DOMAINSEPARATOR is a free data retrieval call binding the contract method 0x3644e515.
//
// Solidity: function DOMAIN_SEPARATOR() view returns(bytes32)
func (_ProvaToken *ProvaTokenCallerSession) DOMAINSEPARATOR() ([32]byte, error) {
	return _ProvaToken.Contract.DOMAINSEPARATOR(&_ProvaToken.CallOpts)
}

// TOTALSUPPLY is a free data retrieval call binding the contract method 0x902d55a5.
//
// Solidity: function TOTAL_SUPPLY() view returns(uint256)
func (_ProvaToken *ProvaTokenCaller) TOTALSUPPLY(opts *bind.CallOpts) (*big.Int, error) {
	var out []interface{}
	err := _ProvaToken.contract.Call(opts, &out, "TOTAL_SUPPLY")

	if err != nil {
		return *new(*big.Int), err
	}

	out0 := *abi.ConvertType(out[0], new(*big.Int)).(**big.Int)

	return out0, err

}

// TOTALSUPPLY is a free data retrieval call binding the contract method 0x902d55a5.
//
// Solidity: function TOTAL_SUPPLY() view returns(uint256)
func (_ProvaToken *ProvaTokenSession) TOTALSUPPLY() (*big.Int, error) {
	return _ProvaToken.Contract.TOTALSUPPLY(&_ProvaToken.CallOpts)
}

// TOTALSUPPLY is a free data retrieval call binding the contract method 0x902d55a5.
//
// Solidity: function TOTAL_SUPPLY() view returns(uint256)
func (_ProvaToken *ProvaTokenCallerSession) TOTALSUPPLY() (*big.Int, error) {
	return _ProvaToken.Contract.TOTALSUPPLY(&_ProvaToken.CallOpts)
}

// Allowance is a free data retrieval call binding the contract method 0xdd62ed3e.
//
// Solidity: function allowance(address owner, address spender) view returns(uint256)
func (_ProvaToken *ProvaTokenCaller) Allowance(opts *bind.CallOpts, owner common.Address, spender common.Address) (*big.Int, error) {
	var out []interface{}
	err := _ProvaToken.contract.Call(opts, &out, "allowance", owner, spender)

	if err != nil {
		return *new(*big.Int), err
	}

	out0 := *abi.ConvertType(out[0], new(*big.Int)).(**big.Int)

	return out0, err

}

// Allowance is a free data retrieval call binding the contract method 0xdd62ed3e.
//
// Solidity: function allowance(address owner, address spender) view returns(uint256)
func (_ProvaToken *ProvaTokenSession) Allowance(owner common.Address, spender common.Address) (*big.Int, error) {
	return _ProvaToken.Contract.Allowance(&_ProvaToken.CallOpts, owner, spender)
}

// Allowance is a free data retrieval call binding the contract method 0xdd62ed3e.
//
// Solidity: function allowance(address owner, address spender) view returns(uint256)
func (_ProvaToken *ProvaTokenCallerSession) Allowance(owner common.Address, spender common.Address) (*big.Int, error) {
	return _ProvaToken.Contract.Allowance(&_ProvaToken.CallOpts, owner, spender)
}

// BalanceOf is a free data retrieval call binding the contract method 0x70a08231.
//
// Solidity: function balanceOf(address account) view returns(uint256)
func (_ProvaToken *ProvaTokenCaller) BalanceOf(opts *bind.CallOpts, account common.Address) (*big.Int, error) {
	var out []interface{}
	err := _ProvaToken.contract.Call(opts, &out, "balanceOf", account)

	if err != nil {
		return *new(*big.Int), err
	}

	out0 := *abi.ConvertType(out[0], new(*big.Int)).(**big.Int)

	return out0, err

}

// BalanceOf is a free data retrieval call binding the contract method 0x70a08231.
//
// Solidity: function balanceOf(address account) view returns(uint256)
func (_ProvaToken *ProvaTokenSession) BalanceOf(account common.Address) (*big.Int, error) {
	return _ProvaToken.Contract.BalanceOf(&_ProvaToken.CallOpts, account)
}

// BalanceOf is a free data retrieval call binding the contract method 0x70a08231.
//
// Solidity: function balanceOf(address account) view returns(uint256)
func (_ProvaToken *ProvaTokenCallerSession) BalanceOf(account common.Address) (*big.Int, error) {
	return _ProvaToken.Contract.BalanceOf(&_ProvaToken.CallOpts, account)
}

// Decimals is a free data retrieval call binding the contract method 0x313ce567.
//
// Solidity: function decimals() view returns(uint8)
func (_ProvaToken *ProvaTokenCaller) Decimals(opts *bind.CallOpts) (uint8, error) {
	var out []interface{}
	err := _ProvaToken.contract.Call(opts, &out, "decimals")

	if err != nil {
		return *new(uint8), err
	}

	out0 := *abi.ConvertType(out[0], new(uint8)).(*uint8)

	return out0, err

}

// Decimals is a free data retrieval call binding the contract method 0x313ce567.
//
// Solidity: function decimals() view returns(uint8)
func (_ProvaToken *ProvaTokenSession) Decimals() (uint8, error) {
	return _ProvaToken.Contract.Decimals(&_ProvaToken.CallOpts)
}

// Decimals is a free data retrieval call binding the contract method 0x313ce567.
//
// Solidity: function decimals() view returns(uint8)
func (_ProvaToken *ProvaTokenCallerSession) Decimals() (uint8, error) {
	return _ProvaToken.Contract.Decimals(&_ProvaToken.CallOpts)
}

// Eip712Domain is a free data retrieval call binding the contract method 0x84b0196e.
//
// Solidity: function eip712Domain() view returns(bytes1 fields, string name, string version, uint256 chainId, address verifyingContract, bytes32 salt, uint256[] extensions)
func (_ProvaToken *ProvaTokenCaller) Eip712Domain(opts *bind.CallOpts) (struct {
	Fields            [1]byte
	Name              string
	Version           string
	ChainId           *big.Int
	VerifyingContract common.Address
	Salt              [32]byte
	Extensions        []*big.Int
}, error) {
	var out []interface{}
	err := _ProvaToken.contract.Call(opts, &out, "eip712Domain")

	outstruct := new(struct {
		Fields            [1]byte
		Name              string
		Version           string
		ChainId           *big.Int
		VerifyingContract common.Address
		Salt              [32]byte
		Extensions        []*big.Int
	})
	if err != nil {
		return *outstruct, err
	}

	outstruct.Fields = *abi.ConvertType(out[0], new([1]byte)).(*[1]byte)
	outstruct.Name = *abi.ConvertType(out[1], new(string)).(*string)
	outstruct.Version = *abi.ConvertType(out[2], new(string)).(*string)
	outstruct.ChainId = *abi.ConvertType(out[3], new(*big.Int)).(**big.Int)
	outstruct.VerifyingContract = *abi.ConvertType(out[4], new(common.Address)).(*common.Address)
	outstruct.Salt = *abi.ConvertType(out[5], new([32]byte)).(*[32]byte)
	outstruct.Extensions = *abi.ConvertType(out[6], new([]*big.Int)).(*[]*big.Int)

	return *outstruct, err

}

// Eip712Domain is a free data retrieval call binding the contract method 0x84b0196e.
//
// Solidity: function eip712Domain() view returns(bytes1 fields, string name, string version, uint256 chainId, address verifyingContract, bytes32 salt, uint256[] extensions)
func (_ProvaToken *ProvaTokenSession) Eip712Domain() (struct {
	Fields            [1]byte
	Name              string
	Version           string
	ChainId           *big.Int
	VerifyingContract common.Address
	Salt              [32]byte
	Extensions        []*big.Int
}, error) {
	return _ProvaToken.Contract.Eip712Domain(&_ProvaToken.CallOpts)
}

// Eip712Domain is a free data retrieval call binding the contract method 0x84b0196e.
//
// Solidity: function eip712Domain() view returns(bytes1 fields, string name, string version, uint256 chainId, address verifyingContract, bytes32 salt, uint256[] extensions)
func (_ProvaToken *ProvaTokenCallerSession) Eip712Domain() (struct {
	Fields            [1]byte
	Name              string
	Version           string
	ChainId           *big.Int
	VerifyingContract common.Address
	Salt              [32]byte
	Extensions        []*big.Int
}, error) {
	return _ProvaToken.Contract.Eip712Domain(&_ProvaToken.CallOpts)
}

// Name is a free data retrieval call binding the contract method 0x06fdde03.
//
// Solidity: function name() view returns(string)
func (_ProvaToken *ProvaTokenCaller) Name(opts *bind.CallOpts) (string, error) {
	var out []interface{}
	err := _ProvaToken.contract.Call(opts, &out, "name")

	if err != nil {
		return *new(string), err
	}

	out0 := *abi.ConvertType(out[0], new(string)).(*string)

	return out0, err

}

// Name is a free data retrieval call binding the contract method 0x06fdde03.
//
// Solidity: function name() view returns(string)
func (_ProvaToken *ProvaTokenSession) Name() (string, error) {
	return _ProvaToken.Contract.Name(&_ProvaToken.CallOpts)
}

// Name is a free data retrieval call binding the contract method 0x06fdde03.
//
// Solidity: function name() view returns(string)
func (_ProvaToken *ProvaTokenCallerSession) Name() (string, error) {
	return _ProvaToken.Contract.Name(&_ProvaToken.CallOpts)
}

// Nonces is a free data retrieval call binding the contract method 0x7ecebe00.
//
// Solidity: function nonces(address owner) view returns(uint256)
func (_ProvaToken *ProvaTokenCaller) Nonces(opts *bind.CallOpts, owner common.Address) (*big.Int, error) {
	var out []interface{}
	err := _ProvaToken.contract.Call(opts, &out, "nonces", owner)

	if err != nil {
		return *new(*big.Int), err
	}

	out0 := *abi.ConvertType(out[0], new(*big.Int)).(**big.Int)

	return out0, err

}

// Nonces is a free data retrieval call binding the contract method 0x7ecebe00.
//
// Solidity: function nonces(address owner) view returns(uint256)
func (_ProvaToken *ProvaTokenSession) Nonces(owner common.Address) (*big.Int, error) {
	return _ProvaToken.Contract.Nonces(&_ProvaToken.CallOpts, owner)
}

// Nonces is a free data retrieval call binding the contract method 0x7ecebe00.
//
// Solidity: function nonces(address owner) view returns(uint256)
func (_ProvaToken *ProvaTokenCallerSession) Nonces(owner common.Address) (*big.Int, error) {
	return _ProvaToken.Contract.Nonces(&_ProvaToken.CallOpts, owner)
}

// Symbol is a free data retrieval call binding the contract method 0x95d89b41.
//
// Solidity: function symbol() view returns(string)
func (_ProvaToken *ProvaTokenCaller) Symbol(opts *bind.CallOpts) (string, error) {
	var out []interface{}
	err := _ProvaToken.contract.Call(opts, &out, "symbol")

	if err != nil {
		return *new(string), err
	}

	out0 := *abi.ConvertType(out[0], new(string)).(*string)

	return out0, err

}

// Symbol is a free data retrieval call binding the contract method 0x95d89b41.
//
// Solidity: function symbol() view returns(string)
func (_ProvaToken *ProvaTokenSession) Symbol() (string, error) {
	return _ProvaToken.Contract.Symbol(&_ProvaToken.CallOpts)
}

// Symbol is a free data retrieval call binding the contract method 0x95d89b41.
//
// Solidity: function symbol() view returns(string)
func (_ProvaToken *ProvaTokenCallerSession) Symbol() (string, error) {
	return _ProvaToken.Contract.Symbol(&_ProvaToken.CallOpts)
}

// TotalSupply is a free data retrieval call binding the contract method 0x18160ddd.
//
// Solidity: function totalSupply() view returns(uint256)
func (_ProvaToken *ProvaTokenCaller) TotalSupply(opts *bind.CallOpts) (*big.Int, error) {
	var out []interface{}
	err := _ProvaToken.contract.Call(opts, &out, "totalSupply")

	if err != nil {
		return *new(*big.Int), err
	}

	out0 := *abi.ConvertType(out[0], new(*big.Int)).(**big.Int)

	return out0, err

}

// TotalSupply is a free data retrieval call binding the contract method 0x18160ddd.
//
// Solidity: function totalSupply() view returns(uint256)
func (_ProvaToken *ProvaTokenSession) TotalSupply() (*big.Int, error) {
	return _ProvaToken.Contract.TotalSupply(&_ProvaToken.CallOpts)
}

// TotalSupply is a free data retrieval call binding the contract method 0x18160ddd.
//
// Solidity: function totalSupply() view returns(uint256)
func (_ProvaToken *ProvaTokenCallerSession) TotalSupply() (*big.Int, error) {
	return _ProvaToken.Contract.TotalSupply(&_ProvaToken.CallOpts)
}

// Approve is a paid mutator transaction binding the contract method 0x095ea7b3.
//
// Solidity: function approve(address spender, uint256 value) returns(bool)
func (_ProvaToken *ProvaTokenTransactor) Approve(opts *bind.TransactOpts, spender common.Address, value *big.Int) (*types.Transaction, error) {
	return _ProvaToken.contract.Transact(opts, "approve", spender, value)
}

// Approve is a paid mutator transaction binding the contract method 0x095ea7b3.
//
// Solidity: function approve(address spender, uint256 value) returns(bool)
func (_ProvaToken *ProvaTokenSession) Approve(spender common.Address, value *big.Int) (*types.Transaction, error) {
	return _ProvaToken.Contract.Approve(&_ProvaToken.TransactOpts, spender, value)
}

// Approve is a paid mutator transaction binding the contract method 0x095ea7b3.
//
// Solidity: function approve(address spender, uint256 value) returns(bool)
func (_ProvaToken *ProvaTokenTransactorSession) Approve(spender common.Address, value *big.Int) (*types.Transaction, error) {
	return _ProvaToken.Contract.Approve(&_ProvaToken.TransactOpts, spender, value)
}

// Burn is a paid mutator transaction binding the contract method 0x42966c68.
//
// Solidity: function burn(uint256 value) returns()
func (_ProvaToken *ProvaTokenTransactor) Burn(opts *bind.TransactOpts, value *big.Int) (*types.Transaction, error) {
	return _ProvaToken.contract.Transact(opts, "burn", value)
}

// Burn is a paid mutator transaction binding the contract method 0x42966c68.
//
// Solidity: function burn(uint256 value) returns()
func (_ProvaToken *ProvaTokenSession) Burn(value *big.Int) (*types.Transaction, error) {
	return _ProvaToken.Contract.Burn(&_ProvaToken.TransactOpts, value)
}

// Burn is a paid mutator transaction binding the contract method 0x42966c68.
//
// Solidity: function burn(uint256 value) returns()
func (_ProvaToken *ProvaTokenTransactorSession) Burn(value *big.Int) (*types.Transaction, error) {
	return _ProvaToken.Contract.Burn(&_ProvaToken.TransactOpts, value)
}

// BurnFrom is a paid mutator transaction binding the contract method 0x79cc6790.
//
// Solidity: function burnFrom(address account, uint256 value) returns()
func (_ProvaToken *ProvaTokenTransactor) BurnFrom(opts *bind.TransactOpts, account common.Address, value *big.Int) (*types.Transaction, error) {
	return _ProvaToken.contract.Transact(opts, "burnFrom", account, value)
}

// BurnFrom is a paid mutator transaction binding the contract method 0x79cc6790.
//
// Solidity: function burnFrom(address account, uint256 value) returns()
func (_ProvaToken *ProvaTokenSession) BurnFrom(account common.Address, value *big.Int) (*types.Transaction, error) {
	return _ProvaToken.Contract.BurnFrom(&_ProvaToken.TransactOpts, account, value)
}

// BurnFrom is a paid mutator transaction binding the contract method 0x79cc6790.
//
// Solidity: function burnFrom(address account, uint256 value) returns()
func (_ProvaToken *ProvaTokenTransactorSession) BurnFrom(account common.Address, value *big.Int) (*types.Transaction, error) {
	return _ProvaToken.Contract.BurnFrom(&_ProvaToken.TransactOpts, account, value)
}

// Permit is a paid mutator transaction binding the contract method 0xd505accf.
//
// Solidity: function permit(address owner, address spender, uint256 value, uint256 deadline, uint8 v, bytes32 r, bytes32 s) returns()
func (_ProvaToken *ProvaTokenTransactor) Permit(opts *bind.TransactOpts, owner common.Address, spender common.Address, value *big.Int, deadline *big.Int, v uint8, r [32]byte, s [32]byte) (*types.Transaction, error) {
	return _ProvaToken.contract.Transact(opts, "permit", owner, spender, value, deadline, v, r, s)
}

// Permit is a paid mutator transaction binding the contract method 0xd505accf.
//
// Solidity: function permit(address owner, address spender, uint256 value, uint256 deadline, uint8 v, bytes32 r, bytes32 s) returns()
func (_ProvaToken *ProvaTokenSession) Permit(owner common.Address, spender common.Address, value *big.Int, deadline *big.Int, v uint8, r [32]byte, s [32]byte) (*types.Transaction, error) {
	return _ProvaToken.Contract.Permit(&_ProvaToken.TransactOpts, owner, spender, value, deadline, v, r, s)
}

// Permit is a paid mutator transaction binding the contract method 0xd505accf.
//
// Solidity: function permit(address owner, address spender, uint256 value, uint256 deadline, uint8 v, bytes32 r, bytes32 s) returns()
func (_ProvaToken *ProvaTokenTransactorSession) Permit(owner common.Address, spender common.Address, value *big.Int, deadline *big.Int, v uint8, r [32]byte, s [32]byte) (*types.Transaction, error) {
	return _ProvaToken.Contract.Permit(&_ProvaToken.TransactOpts, owner, spender, value, deadline, v, r, s)
}

// Transfer is a paid mutator transaction binding the contract method 0xa9059cbb.
//
// Solidity: function transfer(address to, uint256 value) returns(bool)
func (_ProvaToken *ProvaTokenTransactor) Transfer(opts *bind.TransactOpts, to common.Address, value *big.Int) (*types.Transaction, error) {
	return _ProvaToken.contract.Transact(opts, "transfer", to, value)
}

// Transfer is a paid mutator transaction binding the contract method 0xa9059cbb.
//
// Solidity: function transfer(address to, uint256 value) returns(bool)
func (_ProvaToken *ProvaTokenSession) Transfer(to common.Address, value *big.Int) (*types.Transaction, error) {
	return _ProvaToken.Contract.Transfer(&_ProvaToken.TransactOpts, to, value)
}

// Transfer is a paid mutator transaction binding the contract method 0xa9059cbb.
//
// Solidity: function transfer(address to, uint256 value) returns(bool)
func (_ProvaToken *ProvaTokenTransactorSession) Transfer(to common.Address, value *big.Int) (*types.Transaction, error) {
	return _ProvaToken.Contract.Transfer(&_ProvaToken.TransactOpts, to, value)
}

// TransferFrom is a paid mutator transaction binding the contract method 0x23b872dd.
//
// Solidity: function transferFrom(address from, address to, uint256 value) returns(bool)
func (_ProvaToken *ProvaTokenTransactor) TransferFrom(opts *bind.TransactOpts, from common.Address, to common.Address, value *big.Int) (*types.Transaction, error) {
	return _ProvaToken.contract.Transact(opts, "transferFrom", from, to, value)
}

// TransferFrom is a paid mutator transaction binding the contract method 0x23b872dd.
//
// Solidity: function transferFrom(address from, address to, uint256 value) returns(bool)
func (_ProvaToken *ProvaTokenSession) TransferFrom(from common.Address, to common.Address, value *big.Int) (*types.Transaction, error) {
	return _ProvaToken.Contract.TransferFrom(&_ProvaToken.TransactOpts, from, to, value)
}

// TransferFrom is a paid mutator transaction binding the contract method 0x23b872dd.
//
// Solidity: function transferFrom(address from, address to, uint256 value) returns(bool)
func (_ProvaToken *ProvaTokenTransactorSession) TransferFrom(from common.Address, to common.Address, value *big.Int) (*types.Transaction, error) {
	return _ProvaToken.Contract.TransferFrom(&_ProvaToken.TransactOpts, from, to, value)
}

// ProvaTokenApprovalIterator is returned from FilterApproval and is used to iterate over the raw logs and unpacked data for Approval events raised by the ProvaToken contract.
type ProvaTokenApprovalIterator struct {
	Event *ProvaTokenApproval // Event containing the contract specifics and raw log

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
func (it *ProvaTokenApprovalIterator) Next() bool {
	// If the iterator failed, stop iterating
	if it.fail != nil {
		return false
	}
	// If the iterator completed, deliver directly whatever's available
	if it.done {
		select {
		case log := <-it.logs:
			it.Event = new(ProvaTokenApproval)
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
		it.Event = new(ProvaTokenApproval)
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
func (it *ProvaTokenApprovalIterator) Error() error {
	return it.fail
}

// Close terminates the iteration process, releasing any pending underlying
// resources.
func (it *ProvaTokenApprovalIterator) Close() error {
	it.sub.Unsubscribe()
	return nil
}

// ProvaTokenApproval represents a Approval event raised by the ProvaToken contract.
type ProvaTokenApproval struct {
	Owner   common.Address
	Spender common.Address
	Value   *big.Int
	Raw     types.Log // Blockchain specific contextual infos
}

// FilterApproval is a free log retrieval operation binding the contract event 0x8c5be1e5ebec7d5bd14f71427d1e84f3dd0314c0f7b2291e5b200ac8c7c3b925.
//
// Solidity: event Approval(address indexed owner, address indexed spender, uint256 value)
func (_ProvaToken *ProvaTokenFilterer) FilterApproval(opts *bind.FilterOpts, owner []common.Address, spender []common.Address) (*ProvaTokenApprovalIterator, error) {

	var ownerRule []interface{}
	for _, ownerItem := range owner {
		ownerRule = append(ownerRule, ownerItem)
	}
	var spenderRule []interface{}
	for _, spenderItem := range spender {
		spenderRule = append(spenderRule, spenderItem)
	}

	logs, sub, err := _ProvaToken.contract.FilterLogs(opts, "Approval", ownerRule, spenderRule)
	if err != nil {
		return nil, err
	}
	return &ProvaTokenApprovalIterator{contract: _ProvaToken.contract, event: "Approval", logs: logs, sub: sub}, nil
}

// WatchApproval is a free log subscription operation binding the contract event 0x8c5be1e5ebec7d5bd14f71427d1e84f3dd0314c0f7b2291e5b200ac8c7c3b925.
//
// Solidity: event Approval(address indexed owner, address indexed spender, uint256 value)
func (_ProvaToken *ProvaTokenFilterer) WatchApproval(opts *bind.WatchOpts, sink chan<- *ProvaTokenApproval, owner []common.Address, spender []common.Address) (event.Subscription, error) {

	var ownerRule []interface{}
	for _, ownerItem := range owner {
		ownerRule = append(ownerRule, ownerItem)
	}
	var spenderRule []interface{}
	for _, spenderItem := range spender {
		spenderRule = append(spenderRule, spenderItem)
	}

	logs, sub, err := _ProvaToken.contract.WatchLogs(opts, "Approval", ownerRule, spenderRule)
	if err != nil {
		return nil, err
	}
	return event.NewSubscription(func(quit <-chan struct{}) error {
		defer sub.Unsubscribe()
		for {
			select {
			case log := <-logs:
				// New log arrived, parse the event and forward to the user
				event := new(ProvaTokenApproval)
				if err := _ProvaToken.contract.UnpackLog(event, "Approval", log); err != nil {
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

// ParseApproval is a log parse operation binding the contract event 0x8c5be1e5ebec7d5bd14f71427d1e84f3dd0314c0f7b2291e5b200ac8c7c3b925.
//
// Solidity: event Approval(address indexed owner, address indexed spender, uint256 value)
func (_ProvaToken *ProvaTokenFilterer) ParseApproval(log types.Log) (*ProvaTokenApproval, error) {
	event := new(ProvaTokenApproval)
	if err := _ProvaToken.contract.UnpackLog(event, "Approval", log); err != nil {
		return nil, err
	}
	event.Raw = log
	return event, nil
}

// ProvaTokenEIP712DomainChangedIterator is returned from FilterEIP712DomainChanged and is used to iterate over the raw logs and unpacked data for EIP712DomainChanged events raised by the ProvaToken contract.
type ProvaTokenEIP712DomainChangedIterator struct {
	Event *ProvaTokenEIP712DomainChanged // Event containing the contract specifics and raw log

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
func (it *ProvaTokenEIP712DomainChangedIterator) Next() bool {
	// If the iterator failed, stop iterating
	if it.fail != nil {
		return false
	}
	// If the iterator completed, deliver directly whatever's available
	if it.done {
		select {
		case log := <-it.logs:
			it.Event = new(ProvaTokenEIP712DomainChanged)
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
		it.Event = new(ProvaTokenEIP712DomainChanged)
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
func (it *ProvaTokenEIP712DomainChangedIterator) Error() error {
	return it.fail
}

// Close terminates the iteration process, releasing any pending underlying
// resources.
func (it *ProvaTokenEIP712DomainChangedIterator) Close() error {
	it.sub.Unsubscribe()
	return nil
}

// ProvaTokenEIP712DomainChanged represents a EIP712DomainChanged event raised by the ProvaToken contract.
type ProvaTokenEIP712DomainChanged struct {
	Raw types.Log // Blockchain specific contextual infos
}

// FilterEIP712DomainChanged is a free log retrieval operation binding the contract event 0x0a6387c9ea3628b88a633bb4f3b151770f70085117a15f9bf3787cda53f13d31.
//
// Solidity: event EIP712DomainChanged()
func (_ProvaToken *ProvaTokenFilterer) FilterEIP712DomainChanged(opts *bind.FilterOpts) (*ProvaTokenEIP712DomainChangedIterator, error) {

	logs, sub, err := _ProvaToken.contract.FilterLogs(opts, "EIP712DomainChanged")
	if err != nil {
		return nil, err
	}
	return &ProvaTokenEIP712DomainChangedIterator{contract: _ProvaToken.contract, event: "EIP712DomainChanged", logs: logs, sub: sub}, nil
}

// WatchEIP712DomainChanged is a free log subscription operation binding the contract event 0x0a6387c9ea3628b88a633bb4f3b151770f70085117a15f9bf3787cda53f13d31.
//
// Solidity: event EIP712DomainChanged()
func (_ProvaToken *ProvaTokenFilterer) WatchEIP712DomainChanged(opts *bind.WatchOpts, sink chan<- *ProvaTokenEIP712DomainChanged) (event.Subscription, error) {

	logs, sub, err := _ProvaToken.contract.WatchLogs(opts, "EIP712DomainChanged")
	if err != nil {
		return nil, err
	}
	return event.NewSubscription(func(quit <-chan struct{}) error {
		defer sub.Unsubscribe()
		for {
			select {
			case log := <-logs:
				// New log arrived, parse the event and forward to the user
				event := new(ProvaTokenEIP712DomainChanged)
				if err := _ProvaToken.contract.UnpackLog(event, "EIP712DomainChanged", log); err != nil {
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

// ParseEIP712DomainChanged is a log parse operation binding the contract event 0x0a6387c9ea3628b88a633bb4f3b151770f70085117a15f9bf3787cda53f13d31.
//
// Solidity: event EIP712DomainChanged()
func (_ProvaToken *ProvaTokenFilterer) ParseEIP712DomainChanged(log types.Log) (*ProvaTokenEIP712DomainChanged, error) {
	event := new(ProvaTokenEIP712DomainChanged)
	if err := _ProvaToken.contract.UnpackLog(event, "EIP712DomainChanged", log); err != nil {
		return nil, err
	}
	event.Raw = log
	return event, nil
}

// ProvaTokenTransferIterator is returned from FilterTransfer and is used to iterate over the raw logs and unpacked data for Transfer events raised by the ProvaToken contract.
type ProvaTokenTransferIterator struct {
	Event *ProvaTokenTransfer // Event containing the contract specifics and raw log

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
func (it *ProvaTokenTransferIterator) Next() bool {
	// If the iterator failed, stop iterating
	if it.fail != nil {
		return false
	}
	// If the iterator completed, deliver directly whatever's available
	if it.done {
		select {
		case log := <-it.logs:
			it.Event = new(ProvaTokenTransfer)
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
		it.Event = new(ProvaTokenTransfer)
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
func (it *ProvaTokenTransferIterator) Error() error {
	return it.fail
}

// Close terminates the iteration process, releasing any pending underlying
// resources.
func (it *ProvaTokenTransferIterator) Close() error {
	it.sub.Unsubscribe()
	return nil
}

// ProvaTokenTransfer represents a Transfer event raised by the ProvaToken contract.
type ProvaTokenTransfer struct {
	From  common.Address
	To    common.Address
	Value *big.Int
	Raw   types.Log // Blockchain specific contextual infos
}

// FilterTransfer is a free log retrieval operation binding the contract event 0xddf252ad1be2c89b69c2b068fc378daa952ba7f163c4a11628f55a4df523b3ef.
//
// Solidity: event Transfer(address indexed from, address indexed to, uint256 value)
func (_ProvaToken *ProvaTokenFilterer) FilterTransfer(opts *bind.FilterOpts, from []common.Address, to []common.Address) (*ProvaTokenTransferIterator, error) {

	var fromRule []interface{}
	for _, fromItem := range from {
		fromRule = append(fromRule, fromItem)
	}
	var toRule []interface{}
	for _, toItem := range to {
		toRule = append(toRule, toItem)
	}

	logs, sub, err := _ProvaToken.contract.FilterLogs(opts, "Transfer", fromRule, toRule)
	if err != nil {
		return nil, err
	}
	return &ProvaTokenTransferIterator{contract: _ProvaToken.contract, event: "Transfer", logs: logs, sub: sub}, nil
}

// WatchTransfer is a free log subscription operation binding the contract event 0xddf252ad1be2c89b69c2b068fc378daa952ba7f163c4a11628f55a4df523b3ef.
//
// Solidity: event Transfer(address indexed from, address indexed to, uint256 value)
func (_ProvaToken *ProvaTokenFilterer) WatchTransfer(opts *bind.WatchOpts, sink chan<- *ProvaTokenTransfer, from []common.Address, to []common.Address) (event.Subscription, error) {

	var fromRule []interface{}
	for _, fromItem := range from {
		fromRule = append(fromRule, fromItem)
	}
	var toRule []interface{}
	for _, toItem := range to {
		toRule = append(toRule, toItem)
	}

	logs, sub, err := _ProvaToken.contract.WatchLogs(opts, "Transfer", fromRule, toRule)
	if err != nil {
		return nil, err
	}
	return event.NewSubscription(func(quit <-chan struct{}) error {
		defer sub.Unsubscribe()
		for {
			select {
			case log := <-logs:
				// New log arrived, parse the event and forward to the user
				event := new(ProvaTokenTransfer)
				if err := _ProvaToken.contract.UnpackLog(event, "Transfer", log); err != nil {
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

// ParseTransfer is a log parse operation binding the contract event 0xddf252ad1be2c89b69c2b068fc378daa952ba7f163c4a11628f55a4df523b3ef.
//
// Solidity: event Transfer(address indexed from, address indexed to, uint256 value)
func (_ProvaToken *ProvaTokenFilterer) ParseTransfer(log types.Log) (*ProvaTokenTransfer, error) {
	event := new(ProvaTokenTransfer)
	if err := _ProvaToken.contract.UnpackLog(event, "Transfer", log); err != nil {
		return nil, err
	}
	event.Raw = log
	return event, nil
}
