// SPDX-License-Identifier: MIT
// Generated from contracts/out/ProverStaking.sol/ProverStaking.json via abigen.
// Do not edit by hand; run ./scripts/gen-bindings.sh instead.

// Code generated - DO NOT EDIT.
// This file is a generated binding and any manual changes will be lost.

package proverstaking

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

// ProverStakingStakeInfo is an auto generated low-level Go binding around an user-defined struct.
type ProverStakingStakeInfo struct {
	Staked          *big.Int
	Unbonding       *big.Int
	UnbondingEndsAt *big.Int
	CommittedBytes  *big.Int
}

// ProverStakingMetaData contains all meta data concerning the ProverStaking contract.
var ProverStakingMetaData = &bind.MetaData{
	ABI: "[{\"type\":\"constructor\",\"inputs\":[{\"name\":\"_token\",\"type\":\"address\",\"internalType\":\"contractIERC20\"},{\"name\":\"_minStakePerGib\",\"type\":\"uint256\",\"internalType\":\"uint256\"}],\"stateMutability\":\"nonpayable\"},{\"type\":\"function\",\"name\":\"GIB\",\"inputs\":[],\"outputs\":[{\"name\":\"\",\"type\":\"uint256\",\"internalType\":\"uint256\"}],\"stateMutability\":\"view\"},{\"type\":\"function\",\"name\":\"UNBONDING_PERIOD\",\"inputs\":[],\"outputs\":[{\"name\":\"\",\"type\":\"uint256\",\"internalType\":\"uint256\"}],\"stateMutability\":\"view\"},{\"type\":\"function\",\"name\":\"authorizedControllers\",\"inputs\":[{\"name\":\"\",\"type\":\"address\",\"internalType\":\"address\"}],\"outputs\":[{\"name\":\"\",\"type\":\"bool\",\"internalType\":\"bool\"}],\"stateMutability\":\"view\"},{\"type\":\"function\",\"name\":\"availableCapacityBytes\",\"inputs\":[{\"name\":\"prover\",\"type\":\"address\",\"internalType\":\"address\"}],\"outputs\":[{\"name\":\"\",\"type\":\"uint256\",\"internalType\":\"uint256\"}],\"stateMutability\":\"view\"},{\"type\":\"function\",\"name\":\"canCommit\",\"inputs\":[{\"name\":\"prover\",\"type\":\"address\",\"internalType\":\"address\"},{\"name\":\"bytesNeeded\",\"type\":\"uint256\",\"internalType\":\"uint256\"}],\"outputs\":[{\"name\":\"\",\"type\":\"bool\",\"internalType\":\"bool\"}],\"stateMutability\":\"view\"},{\"type\":\"function\",\"name\":\"commitBytes\",\"inputs\":[{\"name\":\"prover\",\"type\":\"address\",\"internalType\":\"address\"},{\"name\":\"newBytes\",\"type\":\"uint256\",\"internalType\":\"uint256\"}],\"outputs\":[],\"stateMutability\":\"nonpayable\"},{\"type\":\"function\",\"name\":\"getStake\",\"inputs\":[{\"name\":\"prover\",\"type\":\"address\",\"internalType\":\"address\"}],\"outputs\":[{\"name\":\"\",\"type\":\"tuple\",\"internalType\":\"structProverStaking.StakeInfo\",\"components\":[{\"name\":\"staked\",\"type\":\"uint256\",\"internalType\":\"uint256\"},{\"name\":\"unbonding\",\"type\":\"uint256\",\"internalType\":\"uint256\"},{\"name\":\"unbondingEndsAt\",\"type\":\"uint256\",\"internalType\":\"uint256\"},{\"name\":\"committedBytes\",\"type\":\"uint256\",\"internalType\":\"uint256\"}]}],\"stateMutability\":\"view\"},{\"type\":\"function\",\"name\":\"minStakeFor\",\"inputs\":[{\"name\":\"committedBytes\",\"type\":\"uint256\",\"internalType\":\"uint256\"}],\"outputs\":[{\"name\":\"\",\"type\":\"uint256\",\"internalType\":\"uint256\"}],\"stateMutability\":\"view\"},{\"type\":\"function\",\"name\":\"minStakePerGib\",\"inputs\":[],\"outputs\":[{\"name\":\"\",\"type\":\"uint256\",\"internalType\":\"uint256\"}],\"stateMutability\":\"view\"},{\"type\":\"function\",\"name\":\"owner\",\"inputs\":[],\"outputs\":[{\"name\":\"\",\"type\":\"address\",\"internalType\":\"address\"}],\"stateMutability\":\"view\"},{\"type\":\"function\",\"name\":\"releaseBytes\",\"inputs\":[{\"name\":\"prover\",\"type\":\"address\",\"internalType\":\"address\"},{\"name\":\"freedBytes\",\"type\":\"uint256\",\"internalType\":\"uint256\"}],\"outputs\":[],\"stateMutability\":\"nonpayable\"},{\"type\":\"function\",\"name\":\"renounceOwnership\",\"inputs\":[],\"outputs\":[],\"stateMutability\":\"nonpayable\"},{\"type\":\"function\",\"name\":\"requestUnstake\",\"inputs\":[{\"name\":\"amount\",\"type\":\"uint256\",\"internalType\":\"uint256\"}],\"outputs\":[],\"stateMutability\":\"nonpayable\"},{\"type\":\"function\",\"name\":\"setAuthorizedController\",\"inputs\":[{\"name\":\"controller\",\"type\":\"address\",\"internalType\":\"address\"},{\"name\":\"authorized\",\"type\":\"bool\",\"internalType\":\"bool\"}],\"outputs\":[],\"stateMutability\":\"nonpayable\"},{\"type\":\"function\",\"name\":\"setMinStakePerGib\",\"inputs\":[{\"name\":\"newValue\",\"type\":\"uint256\",\"internalType\":\"uint256\"}],\"outputs\":[],\"stateMutability\":\"nonpayable\"},{\"type\":\"function\",\"name\":\"slash\",\"inputs\":[{\"name\":\"prover\",\"type\":\"address\",\"internalType\":\"address\"},{\"name\":\"amount\",\"type\":\"uint256\",\"internalType\":\"uint256\"},{\"name\":\"reason\",\"type\":\"bytes32\",\"internalType\":\"bytes32\"}],\"outputs\":[],\"stateMutability\":\"nonpayable\"},{\"type\":\"function\",\"name\":\"slashedPool\",\"inputs\":[],\"outputs\":[{\"name\":\"\",\"type\":\"uint256\",\"internalType\":\"uint256\"}],\"stateMutability\":\"view\"},{\"type\":\"function\",\"name\":\"stake\",\"inputs\":[{\"name\":\"amount\",\"type\":\"uint256\",\"internalType\":\"uint256\"}],\"outputs\":[],\"stateMutability\":\"nonpayable\"},{\"type\":\"function\",\"name\":\"stakes\",\"inputs\":[{\"name\":\"\",\"type\":\"address\",\"internalType\":\"address\"}],\"outputs\":[{\"name\":\"staked\",\"type\":\"uint256\",\"internalType\":\"uint256\"},{\"name\":\"unbonding\",\"type\":\"uint256\",\"internalType\":\"uint256\"},{\"name\":\"unbondingEndsAt\",\"type\":\"uint256\",\"internalType\":\"uint256\"},{\"name\":\"committedBytes\",\"type\":\"uint256\",\"internalType\":\"uint256\"}],\"stateMutability\":\"view\"},{\"type\":\"function\",\"name\":\"token\",\"inputs\":[],\"outputs\":[{\"name\":\"\",\"type\":\"address\",\"internalType\":\"contractIERC20\"}],\"stateMutability\":\"view\"},{\"type\":\"function\",\"name\":\"totalStaked\",\"inputs\":[],\"outputs\":[{\"name\":\"\",\"type\":\"uint256\",\"internalType\":\"uint256\"}],\"stateMutability\":\"view\"},{\"type\":\"function\",\"name\":\"transferOwnership\",\"inputs\":[{\"name\":\"newOwner\",\"type\":\"address\",\"internalType\":\"address\"}],\"outputs\":[],\"stateMutability\":\"nonpayable\"},{\"type\":\"function\",\"name\":\"withdraw\",\"inputs\":[],\"outputs\":[],\"stateMutability\":\"nonpayable\"},{\"type\":\"function\",\"name\":\"withdrawSlashed\",\"inputs\":[{\"name\":\"to\",\"type\":\"address\",\"internalType\":\"address\"},{\"name\":\"amount\",\"type\":\"uint256\",\"internalType\":\"uint256\"}],\"outputs\":[],\"stateMutability\":\"nonpayable\"},{\"type\":\"event\",\"name\":\"AuthorizedControllerSet\",\"inputs\":[{\"name\":\"controller\",\"type\":\"address\",\"indexed\":true,\"internalType\":\"address\"},{\"name\":\"authorized\",\"type\":\"bool\",\"indexed\":false,\"internalType\":\"bool\"}],\"anonymous\":false},{\"type\":\"event\",\"name\":\"CommittedBytesChanged\",\"inputs\":[{\"name\":\"prover\",\"type\":\"address\",\"indexed\":true,\"internalType\":\"address\"},{\"name\":\"newCommittedBytes\",\"type\":\"uint256\",\"indexed\":false,\"internalType\":\"uint256\"}],\"anonymous\":false},{\"type\":\"event\",\"name\":\"MinStakePerGibChanged\",\"inputs\":[{\"name\":\"oldValue\",\"type\":\"uint256\",\"indexed\":false,\"internalType\":\"uint256\"},{\"name\":\"newValue\",\"type\":\"uint256\",\"indexed\":false,\"internalType\":\"uint256\"}],\"anonymous\":false},{\"type\":\"event\",\"name\":\"OwnershipTransferred\",\"inputs\":[{\"name\":\"previousOwner\",\"type\":\"address\",\"indexed\":true,\"internalType\":\"address\"},{\"name\":\"newOwner\",\"type\":\"address\",\"indexed\":true,\"internalType\":\"address\"}],\"anonymous\":false},{\"type\":\"event\",\"name\":\"Slashed\",\"inputs\":[{\"name\":\"prover\",\"type\":\"address\",\"indexed\":true,\"internalType\":\"address\"},{\"name\":\"amount\",\"type\":\"uint256\",\"indexed\":false,\"internalType\":\"uint256\"},{\"name\":\"by\",\"type\":\"address\",\"indexed\":true,\"internalType\":\"address\"},{\"name\":\"reason\",\"type\":\"bytes32\",\"indexed\":false,\"internalType\":\"bytes32\"}],\"anonymous\":false},{\"type\":\"event\",\"name\":\"SlashedPoolWithdrawn\",\"inputs\":[{\"name\":\"to\",\"type\":\"address\",\"indexed\":true,\"internalType\":\"address\"},{\"name\":\"amount\",\"type\":\"uint256\",\"indexed\":false,\"internalType\":\"uint256\"}],\"anonymous\":false},{\"type\":\"event\",\"name\":\"Staked\",\"inputs\":[{\"name\":\"prover\",\"type\":\"address\",\"indexed\":true,\"internalType\":\"address\"},{\"name\":\"amount\",\"type\":\"uint256\",\"indexed\":false,\"internalType\":\"uint256\"},{\"name\":\"newTotal\",\"type\":\"uint256\",\"indexed\":false,\"internalType\":\"uint256\"}],\"anonymous\":false},{\"type\":\"event\",\"name\":\"UnstakeRequested\",\"inputs\":[{\"name\":\"prover\",\"type\":\"address\",\"indexed\":true,\"internalType\":\"address\"},{\"name\":\"amount\",\"type\":\"uint256\",\"indexed\":false,\"internalType\":\"uint256\"},{\"name\":\"endsAt\",\"type\":\"uint256\",\"indexed\":false,\"internalType\":\"uint256\"}],\"anonymous\":false},{\"type\":\"event\",\"name\":\"Withdrawn\",\"inputs\":[{\"name\":\"prover\",\"type\":\"address\",\"indexed\":true,\"internalType\":\"address\"},{\"name\":\"amount\",\"type\":\"uint256\",\"indexed\":false,\"internalType\":\"uint256\"}],\"anonymous\":false},{\"type\":\"error\",\"name\":\"InsufficientBonded\",\"inputs\":[]},{\"type\":\"error\",\"name\":\"InsufficientStake\",\"inputs\":[]},{\"type\":\"error\",\"name\":\"NotAuthorized\",\"inputs\":[]},{\"type\":\"error\",\"name\":\"NothingToWithdraw\",\"inputs\":[]},{\"type\":\"error\",\"name\":\"OwnableInvalidOwner\",\"inputs\":[{\"name\":\"owner\",\"type\":\"address\",\"internalType\":\"address\"}]},{\"type\":\"error\",\"name\":\"OwnableUnauthorizedAccount\",\"inputs\":[{\"name\":\"account\",\"type\":\"address\",\"internalType\":\"address\"}]},{\"type\":\"error\",\"name\":\"ReentrancyGuardReentrantCall\",\"inputs\":[]},{\"type\":\"error\",\"name\":\"SafeERC20FailedOperation\",\"inputs\":[{\"name\":\"token\",\"type\":\"address\",\"internalType\":\"address\"}]},{\"type\":\"error\",\"name\":\"StillUnbonding\",\"inputs\":[]},{\"type\":\"error\",\"name\":\"WouldDropBelowMinimum\",\"inputs\":[]},{\"type\":\"error\",\"name\":\"ZeroAmount\",\"inputs\":[]}]",
	Bin: "0x60a03461010957601f610f1138819003918201601f19168301916001600160401b0383118484101761010d578084926040948552833981010312610109578051906001600160a01b0382168203610109576020015133156100f6575f8054336001600160a01b0319821681178355604051949290916001600160a01b0316907f8be0079c531659141344cd1fd0a4f28419497f9722a3daafe3b4186f6b6457e09080a360017f9b779b17422d0df92223018b32b4d1fa46e071723d6817e2486d003becc55f0055608052600155610def90816101228239608051818181610136015281816103680152818161058201526109420152f35b631e4fbdf760e01b5f525f60045260245ffd5b5f80fd5b634e487b7160e01b5f52604160045260245ffdfe60806040526004361015610011575f80fd5b5f3560e01c806309c661bb14610b445780630a39676f14610b2657806316934fc414610acc57806323095721146109c35780633ccfd60b146108fa578063466b28d3146108d757806362ada8ec146108ba5780636b5f0a2314610894578063715018a6146108505780637a766460146107ba5780637f33006214610730578063817b1cd2146107135780638d543c93146106c05780638da5cb5b14610699578063a694fc3a1461055e578063c281bcc814610521578063cb5c6ff414610492578063cfc0218d14610475578063d9a912ec14610458578063e07f0f14146103db578063f0bfee4c14610301578063f2fde38b1461028f578063fba2c05a146101695763fc0c546a14610121575f80fd5b34610165575f366003190112610165576040517f00000000000000000000000000000000000000000000000000000000000000006001600160a01b03168152602090f35b5f80fd5b3461016557606036600319011261016557610182610b97565b60243590335f52600360205260ff60405f205416156102805781156102715760018060a01b0316805f52600260205260405f20805460018201906101c7825482610bba565b808611610269575b5080851161023b5750506101e4838254610bad565b90556101f282600454610bad565b6004555b61020282600554610bba565b60055560405191825260443560208301527f365a9a525a3155795a8654bb76715d57cb544a4c9053c66d468ad288684e050860403393a3005b5f610262929361025761024e8489610bad565b93600454610bad565b600455558254610bad565b90556101f6565b9450856101cf565b631f2a200560e01b5f5260045ffd5b63ea8e4eb560e01b5f5260045ffd5b34610165576020366003190112610165576102a8610b97565b6102b0610c58565b6001600160a01b031680156102ee575f80546001600160a01b03198116831782556001600160a01b0316905f516020610d7a5f395f51905f529080a3005b631e4fbdf760e01b5f525f60045260245ffd5b346101655760403660031901126101655761031a610b97565b60243590610326610c58565b60055480831161039f577f5c10e125938f053d189065da760df2726397998496fd7b13ae8e334ffdea7c7b9161035e84602093610bad565b60055561038c84827f0000000000000000000000000000000000000000000000000000000000000000610d01565b6040519384526001600160a01b031692a2005b60405162461bcd60e51b8152602060048201526014602482015273195e18d959591cc81cdb185cda1959081c1bdbdb60621b6044820152606490fd5b34610165576040366003190112610165576103f4610b97565b602435908115158092036101655760207fd767b21ed9c209f0b4e3280af8a32f4f55b4c81948bd8fb0e2e7df92f2f9c85d9161042e610c58565b60018060a01b031692835f526003825260405f2060ff1981541660ff8316179055604051908152a2005b34610165575f366003190112610165576020604051621275008152f35b34610165575f366003190112610165576020600154604051908152f35b34610165576040366003190112610165576104ab610b97565b335f52600360205260ff60405f205416156102805760018060a01b0316805f52600260205260405f2060038101906104e66024358354610bba565b809255546104f382610cb6565b116105125760205f516020610d5a5f395f51905f5291604051908152a2005b6378de4a6960e11b5f5260045ffd5b34610165576020366003190112610165576001600160a01b03610542610b97565b165f526003602052602060ff60405f2054166040519015158152f35b346101655760203660031901126101655760043561057a610c7e565b8015610271577f00000000000000000000000000000000000000000000000000000000000000006040516323b872dd60e01b5f5233600452306024528260445260205f60648180865af19060015f5114821615610678575b6040525f606052156106585750335f52600260205260405f206105f6828254610bba565b905561060481600454610bba565b600455335f52600260205260405f205460405191825260208201527f1449c6dd7851abc30abf37f57715f492010519147cc2652fbc38202c18a6ee9060403392a260015f516020610d9a5f395f51905f5255005b635274afe760e01b5f9081526001600160a01b0391909116600452602490fd5b90600181151661069057823b15153d151616906105d2565b503d5f823e3d90fd5b34610165575f366003190112610165575f546040516001600160a01b039091168152602090f35b34610165576040366003190112610165576001600160a01b036106e1610b97565b165f526002602052602060405f206107076107026024356003840154610bba565b610cb6565b90541015604051908152f35b34610165575f366003190112610165576020600454604051908152f35b3461016557604036600319011261016557610749610b97565b60243590335f52600360205260ff60405f20541615610280576001600160a01b03165f818152600260209081526040909120600301805492935f516020610d5a5f395f51905f529390818111156107aa5750505f81555b54604051908152a2005b6107b391610bad565b81556107a0565b34610165576020366003190112610165576107d3610b97565b5f60606107de610c25565b828152826020820152826040820152015260018060a01b03165f526002602052608060405f2061080c610c25565b815491828252600181015460208301908152606060036002840154936040860194855201549301928352604051938452516020840152516040830152516060820152f35b34610165575f36600319011261016557610868610c58565b5f80546001600160a01b0319811682556001600160a01b03165f516020610d7a5f395f51905f528280a3005b346101655760203660031901126101655760206108b2600435610cb6565b604051908152f35b34610165575f366003190112610165576020600554604051908152f35b346101655760203660031901126101655760206108b26108f5610b97565b610bc7565b34610165575f36600319011261016557610912610c7e565b335f52600260205260405f206001810180549182156109b457600201805442106109a5575f8092555561096681337f0000000000000000000000000000000000000000000000000000000000000000610d01565b6040519081527f7084f5476618d8e60b11ef0d7d3f06914655adb8793e28ff7f018d4c76d505d560203392a260015f516020610d9a5f395f51905f5255005b633fc3ac2b60e01b5f5260045ffd5b630686827b60e51b5f5260045ffd5b34610165576020366003190112610165576004356109df610c7e565b801561027157335f52600260205260405f208054808311610abd57610a076003830154610cb6565b610a118483610bad565b10610aae5782610a2091610bad565b8155610a2e82600454610bad565b60045560018101610a40838254610bba565b905562127500420190814211610a9a5760028291015560405191825260208201527f57e41df54512c76148b5ba9b643d149752b0d35e493b969bd017d0a3fe5228cf60403392a260015f516020610d9a5f395f51905f5255005b634e487b7160e01b5f52601160045260245ffd5b63500c862960e01b5f5260045ffd5b636d281bcd60e11b5f5260045ffd5b34610165576020366003190112610165576001600160a01b03610aed610b97565b165f526002602052608060405f208054906001810154906003600282015491015491604051938452602084015260408301526060820152f35b34610165575f36600319011261016557602060405163400000008152f35b3461016557602036600319011261016557600435610b60610c58565b7fdcbfa78fc8ad2deb48caa409ffb7fa7fbcf8064fa61d57eb204b3fd0a56a3ab660406001548151908152836020820152a1600155005b600435906001600160a01b038216820361016557565b91908203918211610a9a57565b91908201809211610a9a57565b6001600160a01b03165f9081526002602052604090206001548015610c1e578154049081601e1b9180830463400000001490151715610a9a576003015480821115610c1857610c1591610bad565b90565b50505f90565b50505f1990565b60405190608082016001600160401b03811183821017610c4457604052565b634e487b7160e01b5f52604160045260245ffd5b5f546001600160a01b03163303610c6b57565b63118cdaa760e01b5f523360045260245ffd5b60025f516020610d9a5f395f51905f525414610ca75760025f516020610d9a5f395f51905f5255565b633ee5aeb560e01b5f5260045ffd5b6001549081158015610cf9575b610c18576340000000810190818111610a9a57633fffffff01908111610a9a57601e1c818102918183041490151715610a9a5790565b508015610cc3565b916040519163a9059cbb60e01b5f5260018060a01b031660045260245260205f60448180865af19060015f5114821615610d41575b604052156106585750565b90600181151661069057823b15153d15161690610d3656fefaa44185b159c9e959d2e94c26eb27c317e9f607e05d37d206b775eece86bbe58be0079c531659141344cd1fd0a4f28419497f9722a3daafe3b4186f6b6457e09b779b17422d0df92223018b32b4d1fa46e071723d6817e2486d003becc55f00a2646970667358221220dfc8cbaca453d0b610d85a5469707d90cb0ad31080ae196f26a5efa4d20c92e264736f6c634300081e0033",
}

// ProverStakingABI is the input ABI used to generate the binding from.
// Deprecated: Use ProverStakingMetaData.ABI instead.
var ProverStakingABI = ProverStakingMetaData.ABI

// ProverStakingBin is the compiled bytecode used for deploying new contracts.
// Deprecated: Use ProverStakingMetaData.Bin instead.
var ProverStakingBin = ProverStakingMetaData.Bin

// DeployProverStaking deploys a new Ethereum contract, binding an instance of ProverStaking to it.
func DeployProverStaking(auth *bind.TransactOpts, backend bind.ContractBackend, _token common.Address, _minStakePerGib *big.Int) (common.Address, *types.Transaction, *ProverStaking, error) {
	parsed, err := ProverStakingMetaData.GetAbi()
	if err != nil {
		return common.Address{}, nil, nil, err
	}
	if parsed == nil {
		return common.Address{}, nil, nil, errors.New("GetABI returned nil")
	}

	address, tx, contract, err := bind.DeployContract(auth, *parsed, common.FromHex(ProverStakingBin), backend, _token, _minStakePerGib)
	if err != nil {
		return common.Address{}, nil, nil, err
	}
	return address, tx, &ProverStaking{ProverStakingCaller: ProverStakingCaller{contract: contract}, ProverStakingTransactor: ProverStakingTransactor{contract: contract}, ProverStakingFilterer: ProverStakingFilterer{contract: contract}}, nil
}

// ProverStaking is an auto generated Go binding around an Ethereum contract.
type ProverStaking struct {
	ProverStakingCaller     // Read-only binding to the contract
	ProverStakingTransactor // Write-only binding to the contract
	ProverStakingFilterer   // Log filterer for contract events
}

// ProverStakingCaller is an auto generated read-only Go binding around an Ethereum contract.
type ProverStakingCaller struct {
	contract *bind.BoundContract // Generic contract wrapper for the low level calls
}

// ProverStakingTransactor is an auto generated write-only Go binding around an Ethereum contract.
type ProverStakingTransactor struct {
	contract *bind.BoundContract // Generic contract wrapper for the low level calls
}

// ProverStakingFilterer is an auto generated log filtering Go binding around an Ethereum contract events.
type ProverStakingFilterer struct {
	contract *bind.BoundContract // Generic contract wrapper for the low level calls
}

// ProverStakingSession is an auto generated Go binding around an Ethereum contract,
// with pre-set call and transact options.
type ProverStakingSession struct {
	Contract     *ProverStaking    // Generic contract binding to set the session for
	CallOpts     bind.CallOpts     // Call options to use throughout this session
	TransactOpts bind.TransactOpts // Transaction auth options to use throughout this session
}

// ProverStakingCallerSession is an auto generated read-only Go binding around an Ethereum contract,
// with pre-set call options.
type ProverStakingCallerSession struct {
	Contract *ProverStakingCaller // Generic contract caller binding to set the session for
	CallOpts bind.CallOpts        // Call options to use throughout this session
}

// ProverStakingTransactorSession is an auto generated write-only Go binding around an Ethereum contract,
// with pre-set transact options.
type ProverStakingTransactorSession struct {
	Contract     *ProverStakingTransactor // Generic contract transactor binding to set the session for
	TransactOpts bind.TransactOpts        // Transaction auth options to use throughout this session
}

// ProverStakingRaw is an auto generated low-level Go binding around an Ethereum contract.
type ProverStakingRaw struct {
	Contract *ProverStaking // Generic contract binding to access the raw methods on
}

// ProverStakingCallerRaw is an auto generated low-level read-only Go binding around an Ethereum contract.
type ProverStakingCallerRaw struct {
	Contract *ProverStakingCaller // Generic read-only contract binding to access the raw methods on
}

// ProverStakingTransactorRaw is an auto generated low-level write-only Go binding around an Ethereum contract.
type ProverStakingTransactorRaw struct {
	Contract *ProverStakingTransactor // Generic write-only contract binding to access the raw methods on
}

// NewProverStaking creates a new instance of ProverStaking, bound to a specific deployed contract.
func NewProverStaking(address common.Address, backend bind.ContractBackend) (*ProverStaking, error) {
	contract, err := bindProverStaking(address, backend, backend, backend)
	if err != nil {
		return nil, err
	}
	return &ProverStaking{ProverStakingCaller: ProverStakingCaller{contract: contract}, ProverStakingTransactor: ProverStakingTransactor{contract: contract}, ProverStakingFilterer: ProverStakingFilterer{contract: contract}}, nil
}

// NewProverStakingCaller creates a new read-only instance of ProverStaking, bound to a specific deployed contract.
func NewProverStakingCaller(address common.Address, caller bind.ContractCaller) (*ProverStakingCaller, error) {
	contract, err := bindProverStaking(address, caller, nil, nil)
	if err != nil {
		return nil, err
	}
	return &ProverStakingCaller{contract: contract}, nil
}

// NewProverStakingTransactor creates a new write-only instance of ProverStaking, bound to a specific deployed contract.
func NewProverStakingTransactor(address common.Address, transactor bind.ContractTransactor) (*ProverStakingTransactor, error) {
	contract, err := bindProverStaking(address, nil, transactor, nil)
	if err != nil {
		return nil, err
	}
	return &ProverStakingTransactor{contract: contract}, nil
}

// NewProverStakingFilterer creates a new log filterer instance of ProverStaking, bound to a specific deployed contract.
func NewProverStakingFilterer(address common.Address, filterer bind.ContractFilterer) (*ProverStakingFilterer, error) {
	contract, err := bindProverStaking(address, nil, nil, filterer)
	if err != nil {
		return nil, err
	}
	return &ProverStakingFilterer{contract: contract}, nil
}

// bindProverStaking binds a generic wrapper to an already deployed contract.
func bindProverStaking(address common.Address, caller bind.ContractCaller, transactor bind.ContractTransactor, filterer bind.ContractFilterer) (*bind.BoundContract, error) {
	parsed, err := ProverStakingMetaData.GetAbi()
	if err != nil {
		return nil, err
	}
	return bind.NewBoundContract(address, *parsed, caller, transactor, filterer), nil
}

// Call invokes the (constant) contract method with params as input values and
// sets the output to result. The result type might be a single field for simple
// returns, a slice of interfaces for anonymous returns and a struct for named
// returns.
func (_ProverStaking *ProverStakingRaw) Call(opts *bind.CallOpts, result *[]interface{}, method string, params ...interface{}) error {
	return _ProverStaking.Contract.ProverStakingCaller.contract.Call(opts, result, method, params...)
}

// Transfer initiates a plain transaction to move funds to the contract, calling
// its default method if one is available.
func (_ProverStaking *ProverStakingRaw) Transfer(opts *bind.TransactOpts) (*types.Transaction, error) {
	return _ProverStaking.Contract.ProverStakingTransactor.contract.Transfer(opts)
}

// Transact invokes the (paid) contract method with params as input values.
func (_ProverStaking *ProverStakingRaw) Transact(opts *bind.TransactOpts, method string, params ...interface{}) (*types.Transaction, error) {
	return _ProverStaking.Contract.ProverStakingTransactor.contract.Transact(opts, method, params...)
}

// Call invokes the (constant) contract method with params as input values and
// sets the output to result. The result type might be a single field for simple
// returns, a slice of interfaces for anonymous returns and a struct for named
// returns.
func (_ProverStaking *ProverStakingCallerRaw) Call(opts *bind.CallOpts, result *[]interface{}, method string, params ...interface{}) error {
	return _ProverStaking.Contract.contract.Call(opts, result, method, params...)
}

// Transfer initiates a plain transaction to move funds to the contract, calling
// its default method if one is available.
func (_ProverStaking *ProverStakingTransactorRaw) Transfer(opts *bind.TransactOpts) (*types.Transaction, error) {
	return _ProverStaking.Contract.contract.Transfer(opts)
}

// Transact invokes the (paid) contract method with params as input values.
func (_ProverStaking *ProverStakingTransactorRaw) Transact(opts *bind.TransactOpts, method string, params ...interface{}) (*types.Transaction, error) {
	return _ProverStaking.Contract.contract.Transact(opts, method, params...)
}

// GIB is a free data retrieval call binding the contract method 0x0a39676f.
//
// Solidity: function GIB() view returns(uint256)
func (_ProverStaking *ProverStakingCaller) GIB(opts *bind.CallOpts) (*big.Int, error) {
	var out []interface{}
	err := _ProverStaking.contract.Call(opts, &out, "GIB")

	if err != nil {
		return *new(*big.Int), err
	}

	out0 := *abi.ConvertType(out[0], new(*big.Int)).(**big.Int)

	return out0, err

}

// GIB is a free data retrieval call binding the contract method 0x0a39676f.
//
// Solidity: function GIB() view returns(uint256)
func (_ProverStaking *ProverStakingSession) GIB() (*big.Int, error) {
	return _ProverStaking.Contract.GIB(&_ProverStaking.CallOpts)
}

// GIB is a free data retrieval call binding the contract method 0x0a39676f.
//
// Solidity: function GIB() view returns(uint256)
func (_ProverStaking *ProverStakingCallerSession) GIB() (*big.Int, error) {
	return _ProverStaking.Contract.GIB(&_ProverStaking.CallOpts)
}

// UNBONDINGPERIOD is a free data retrieval call binding the contract method 0xd9a912ec.
//
// Solidity: function UNBONDING_PERIOD() view returns(uint256)
func (_ProverStaking *ProverStakingCaller) UNBONDINGPERIOD(opts *bind.CallOpts) (*big.Int, error) {
	var out []interface{}
	err := _ProverStaking.contract.Call(opts, &out, "UNBONDING_PERIOD")

	if err != nil {
		return *new(*big.Int), err
	}

	out0 := *abi.ConvertType(out[0], new(*big.Int)).(**big.Int)

	return out0, err

}

// UNBONDINGPERIOD is a free data retrieval call binding the contract method 0xd9a912ec.
//
// Solidity: function UNBONDING_PERIOD() view returns(uint256)
func (_ProverStaking *ProverStakingSession) UNBONDINGPERIOD() (*big.Int, error) {
	return _ProverStaking.Contract.UNBONDINGPERIOD(&_ProverStaking.CallOpts)
}

// UNBONDINGPERIOD is a free data retrieval call binding the contract method 0xd9a912ec.
//
// Solidity: function UNBONDING_PERIOD() view returns(uint256)
func (_ProverStaking *ProverStakingCallerSession) UNBONDINGPERIOD() (*big.Int, error) {
	return _ProverStaking.Contract.UNBONDINGPERIOD(&_ProverStaking.CallOpts)
}

// AuthorizedControllers is a free data retrieval call binding the contract method 0xc281bcc8.
//
// Solidity: function authorizedControllers(address ) view returns(bool)
func (_ProverStaking *ProverStakingCaller) AuthorizedControllers(opts *bind.CallOpts, arg0 common.Address) (bool, error) {
	var out []interface{}
	err := _ProverStaking.contract.Call(opts, &out, "authorizedControllers", arg0)

	if err != nil {
		return *new(bool), err
	}

	out0 := *abi.ConvertType(out[0], new(bool)).(*bool)

	return out0, err

}

// AuthorizedControllers is a free data retrieval call binding the contract method 0xc281bcc8.
//
// Solidity: function authorizedControllers(address ) view returns(bool)
func (_ProverStaking *ProverStakingSession) AuthorizedControllers(arg0 common.Address) (bool, error) {
	return _ProverStaking.Contract.AuthorizedControllers(&_ProverStaking.CallOpts, arg0)
}

// AuthorizedControllers is a free data retrieval call binding the contract method 0xc281bcc8.
//
// Solidity: function authorizedControllers(address ) view returns(bool)
func (_ProverStaking *ProverStakingCallerSession) AuthorizedControllers(arg0 common.Address) (bool, error) {
	return _ProverStaking.Contract.AuthorizedControllers(&_ProverStaking.CallOpts, arg0)
}

// AvailableCapacityBytes is a free data retrieval call binding the contract method 0x466b28d3.
//
// Solidity: function availableCapacityBytes(address prover) view returns(uint256)
func (_ProverStaking *ProverStakingCaller) AvailableCapacityBytes(opts *bind.CallOpts, prover common.Address) (*big.Int, error) {
	var out []interface{}
	err := _ProverStaking.contract.Call(opts, &out, "availableCapacityBytes", prover)

	if err != nil {
		return *new(*big.Int), err
	}

	out0 := *abi.ConvertType(out[0], new(*big.Int)).(**big.Int)

	return out0, err

}

// AvailableCapacityBytes is a free data retrieval call binding the contract method 0x466b28d3.
//
// Solidity: function availableCapacityBytes(address prover) view returns(uint256)
func (_ProverStaking *ProverStakingSession) AvailableCapacityBytes(prover common.Address) (*big.Int, error) {
	return _ProverStaking.Contract.AvailableCapacityBytes(&_ProverStaking.CallOpts, prover)
}

// AvailableCapacityBytes is a free data retrieval call binding the contract method 0x466b28d3.
//
// Solidity: function availableCapacityBytes(address prover) view returns(uint256)
func (_ProverStaking *ProverStakingCallerSession) AvailableCapacityBytes(prover common.Address) (*big.Int, error) {
	return _ProverStaking.Contract.AvailableCapacityBytes(&_ProverStaking.CallOpts, prover)
}

// CanCommit is a free data retrieval call binding the contract method 0x8d543c93.
//
// Solidity: function canCommit(address prover, uint256 bytesNeeded) view returns(bool)
func (_ProverStaking *ProverStakingCaller) CanCommit(opts *bind.CallOpts, prover common.Address, bytesNeeded *big.Int) (bool, error) {
	var out []interface{}
	err := _ProverStaking.contract.Call(opts, &out, "canCommit", prover, bytesNeeded)

	if err != nil {
		return *new(bool), err
	}

	out0 := *abi.ConvertType(out[0], new(bool)).(*bool)

	return out0, err

}

// CanCommit is a free data retrieval call binding the contract method 0x8d543c93.
//
// Solidity: function canCommit(address prover, uint256 bytesNeeded) view returns(bool)
func (_ProverStaking *ProverStakingSession) CanCommit(prover common.Address, bytesNeeded *big.Int) (bool, error) {
	return _ProverStaking.Contract.CanCommit(&_ProverStaking.CallOpts, prover, bytesNeeded)
}

// CanCommit is a free data retrieval call binding the contract method 0x8d543c93.
//
// Solidity: function canCommit(address prover, uint256 bytesNeeded) view returns(bool)
func (_ProverStaking *ProverStakingCallerSession) CanCommit(prover common.Address, bytesNeeded *big.Int) (bool, error) {
	return _ProverStaking.Contract.CanCommit(&_ProverStaking.CallOpts, prover, bytesNeeded)
}

// GetStake is a free data retrieval call binding the contract method 0x7a766460.
//
// Solidity: function getStake(address prover) view returns((uint256,uint256,uint256,uint256))
func (_ProverStaking *ProverStakingCaller) GetStake(opts *bind.CallOpts, prover common.Address) (ProverStakingStakeInfo, error) {
	var out []interface{}
	err := _ProverStaking.contract.Call(opts, &out, "getStake", prover)

	if err != nil {
		return *new(ProverStakingStakeInfo), err
	}

	out0 := *abi.ConvertType(out[0], new(ProverStakingStakeInfo)).(*ProverStakingStakeInfo)

	return out0, err

}

// GetStake is a free data retrieval call binding the contract method 0x7a766460.
//
// Solidity: function getStake(address prover) view returns((uint256,uint256,uint256,uint256))
func (_ProverStaking *ProverStakingSession) GetStake(prover common.Address) (ProverStakingStakeInfo, error) {
	return _ProverStaking.Contract.GetStake(&_ProverStaking.CallOpts, prover)
}

// GetStake is a free data retrieval call binding the contract method 0x7a766460.
//
// Solidity: function getStake(address prover) view returns((uint256,uint256,uint256,uint256))
func (_ProverStaking *ProverStakingCallerSession) GetStake(prover common.Address) (ProverStakingStakeInfo, error) {
	return _ProverStaking.Contract.GetStake(&_ProverStaking.CallOpts, prover)
}

// MinStakeFor is a free data retrieval call binding the contract method 0x6b5f0a23.
//
// Solidity: function minStakeFor(uint256 committedBytes) view returns(uint256)
func (_ProverStaking *ProverStakingCaller) MinStakeFor(opts *bind.CallOpts, committedBytes *big.Int) (*big.Int, error) {
	var out []interface{}
	err := _ProverStaking.contract.Call(opts, &out, "minStakeFor", committedBytes)

	if err != nil {
		return *new(*big.Int), err
	}

	out0 := *abi.ConvertType(out[0], new(*big.Int)).(**big.Int)

	return out0, err

}

// MinStakeFor is a free data retrieval call binding the contract method 0x6b5f0a23.
//
// Solidity: function minStakeFor(uint256 committedBytes) view returns(uint256)
func (_ProverStaking *ProverStakingSession) MinStakeFor(committedBytes *big.Int) (*big.Int, error) {
	return _ProverStaking.Contract.MinStakeFor(&_ProverStaking.CallOpts, committedBytes)
}

// MinStakeFor is a free data retrieval call binding the contract method 0x6b5f0a23.
//
// Solidity: function minStakeFor(uint256 committedBytes) view returns(uint256)
func (_ProverStaking *ProverStakingCallerSession) MinStakeFor(committedBytes *big.Int) (*big.Int, error) {
	return _ProverStaking.Contract.MinStakeFor(&_ProverStaking.CallOpts, committedBytes)
}

// MinStakePerGib is a free data retrieval call binding the contract method 0xcfc0218d.
//
// Solidity: function minStakePerGib() view returns(uint256)
func (_ProverStaking *ProverStakingCaller) MinStakePerGib(opts *bind.CallOpts) (*big.Int, error) {
	var out []interface{}
	err := _ProverStaking.contract.Call(opts, &out, "minStakePerGib")

	if err != nil {
		return *new(*big.Int), err
	}

	out0 := *abi.ConvertType(out[0], new(*big.Int)).(**big.Int)

	return out0, err

}

// MinStakePerGib is a free data retrieval call binding the contract method 0xcfc0218d.
//
// Solidity: function minStakePerGib() view returns(uint256)
func (_ProverStaking *ProverStakingSession) MinStakePerGib() (*big.Int, error) {
	return _ProverStaking.Contract.MinStakePerGib(&_ProverStaking.CallOpts)
}

// MinStakePerGib is a free data retrieval call binding the contract method 0xcfc0218d.
//
// Solidity: function minStakePerGib() view returns(uint256)
func (_ProverStaking *ProverStakingCallerSession) MinStakePerGib() (*big.Int, error) {
	return _ProverStaking.Contract.MinStakePerGib(&_ProverStaking.CallOpts)
}

// Owner is a free data retrieval call binding the contract method 0x8da5cb5b.
//
// Solidity: function owner() view returns(address)
func (_ProverStaking *ProverStakingCaller) Owner(opts *bind.CallOpts) (common.Address, error) {
	var out []interface{}
	err := _ProverStaking.contract.Call(opts, &out, "owner")

	if err != nil {
		return *new(common.Address), err
	}

	out0 := *abi.ConvertType(out[0], new(common.Address)).(*common.Address)

	return out0, err

}

// Owner is a free data retrieval call binding the contract method 0x8da5cb5b.
//
// Solidity: function owner() view returns(address)
func (_ProverStaking *ProverStakingSession) Owner() (common.Address, error) {
	return _ProverStaking.Contract.Owner(&_ProverStaking.CallOpts)
}

// Owner is a free data retrieval call binding the contract method 0x8da5cb5b.
//
// Solidity: function owner() view returns(address)
func (_ProverStaking *ProverStakingCallerSession) Owner() (common.Address, error) {
	return _ProverStaking.Contract.Owner(&_ProverStaking.CallOpts)
}

// SlashedPool is a free data retrieval call binding the contract method 0x62ada8ec.
//
// Solidity: function slashedPool() view returns(uint256)
func (_ProverStaking *ProverStakingCaller) SlashedPool(opts *bind.CallOpts) (*big.Int, error) {
	var out []interface{}
	err := _ProverStaking.contract.Call(opts, &out, "slashedPool")

	if err != nil {
		return *new(*big.Int), err
	}

	out0 := *abi.ConvertType(out[0], new(*big.Int)).(**big.Int)

	return out0, err

}

// SlashedPool is a free data retrieval call binding the contract method 0x62ada8ec.
//
// Solidity: function slashedPool() view returns(uint256)
func (_ProverStaking *ProverStakingSession) SlashedPool() (*big.Int, error) {
	return _ProverStaking.Contract.SlashedPool(&_ProverStaking.CallOpts)
}

// SlashedPool is a free data retrieval call binding the contract method 0x62ada8ec.
//
// Solidity: function slashedPool() view returns(uint256)
func (_ProverStaking *ProverStakingCallerSession) SlashedPool() (*big.Int, error) {
	return _ProverStaking.Contract.SlashedPool(&_ProverStaking.CallOpts)
}

// Stakes is a free data retrieval call binding the contract method 0x16934fc4.
//
// Solidity: function stakes(address ) view returns(uint256 staked, uint256 unbonding, uint256 unbondingEndsAt, uint256 committedBytes)
func (_ProverStaking *ProverStakingCaller) Stakes(opts *bind.CallOpts, arg0 common.Address) (struct {
	Staked          *big.Int
	Unbonding       *big.Int
	UnbondingEndsAt *big.Int
	CommittedBytes  *big.Int
}, error) {
	var out []interface{}
	err := _ProverStaking.contract.Call(opts, &out, "stakes", arg0)

	outstruct := new(struct {
		Staked          *big.Int
		Unbonding       *big.Int
		UnbondingEndsAt *big.Int
		CommittedBytes  *big.Int
	})
	if err != nil {
		return *outstruct, err
	}

	outstruct.Staked = *abi.ConvertType(out[0], new(*big.Int)).(**big.Int)
	outstruct.Unbonding = *abi.ConvertType(out[1], new(*big.Int)).(**big.Int)
	outstruct.UnbondingEndsAt = *abi.ConvertType(out[2], new(*big.Int)).(**big.Int)
	outstruct.CommittedBytes = *abi.ConvertType(out[3], new(*big.Int)).(**big.Int)

	return *outstruct, err

}

// Stakes is a free data retrieval call binding the contract method 0x16934fc4.
//
// Solidity: function stakes(address ) view returns(uint256 staked, uint256 unbonding, uint256 unbondingEndsAt, uint256 committedBytes)
func (_ProverStaking *ProverStakingSession) Stakes(arg0 common.Address) (struct {
	Staked          *big.Int
	Unbonding       *big.Int
	UnbondingEndsAt *big.Int
	CommittedBytes  *big.Int
}, error) {
	return _ProverStaking.Contract.Stakes(&_ProverStaking.CallOpts, arg0)
}

// Stakes is a free data retrieval call binding the contract method 0x16934fc4.
//
// Solidity: function stakes(address ) view returns(uint256 staked, uint256 unbonding, uint256 unbondingEndsAt, uint256 committedBytes)
func (_ProverStaking *ProverStakingCallerSession) Stakes(arg0 common.Address) (struct {
	Staked          *big.Int
	Unbonding       *big.Int
	UnbondingEndsAt *big.Int
	CommittedBytes  *big.Int
}, error) {
	return _ProverStaking.Contract.Stakes(&_ProverStaking.CallOpts, arg0)
}

// Token is a free data retrieval call binding the contract method 0xfc0c546a.
//
// Solidity: function token() view returns(address)
func (_ProverStaking *ProverStakingCaller) Token(opts *bind.CallOpts) (common.Address, error) {
	var out []interface{}
	err := _ProverStaking.contract.Call(opts, &out, "token")

	if err != nil {
		return *new(common.Address), err
	}

	out0 := *abi.ConvertType(out[0], new(common.Address)).(*common.Address)

	return out0, err

}

// Token is a free data retrieval call binding the contract method 0xfc0c546a.
//
// Solidity: function token() view returns(address)
func (_ProverStaking *ProverStakingSession) Token() (common.Address, error) {
	return _ProverStaking.Contract.Token(&_ProverStaking.CallOpts)
}

// Token is a free data retrieval call binding the contract method 0xfc0c546a.
//
// Solidity: function token() view returns(address)
func (_ProverStaking *ProverStakingCallerSession) Token() (common.Address, error) {
	return _ProverStaking.Contract.Token(&_ProverStaking.CallOpts)
}

// TotalStaked is a free data retrieval call binding the contract method 0x817b1cd2.
//
// Solidity: function totalStaked() view returns(uint256)
func (_ProverStaking *ProverStakingCaller) TotalStaked(opts *bind.CallOpts) (*big.Int, error) {
	var out []interface{}
	err := _ProverStaking.contract.Call(opts, &out, "totalStaked")

	if err != nil {
		return *new(*big.Int), err
	}

	out0 := *abi.ConvertType(out[0], new(*big.Int)).(**big.Int)

	return out0, err

}

// TotalStaked is a free data retrieval call binding the contract method 0x817b1cd2.
//
// Solidity: function totalStaked() view returns(uint256)
func (_ProverStaking *ProverStakingSession) TotalStaked() (*big.Int, error) {
	return _ProverStaking.Contract.TotalStaked(&_ProverStaking.CallOpts)
}

// TotalStaked is a free data retrieval call binding the contract method 0x817b1cd2.
//
// Solidity: function totalStaked() view returns(uint256)
func (_ProverStaking *ProverStakingCallerSession) TotalStaked() (*big.Int, error) {
	return _ProverStaking.Contract.TotalStaked(&_ProverStaking.CallOpts)
}

// CommitBytes is a paid mutator transaction binding the contract method 0xcb5c6ff4.
//
// Solidity: function commitBytes(address prover, uint256 newBytes) returns()
func (_ProverStaking *ProverStakingTransactor) CommitBytes(opts *bind.TransactOpts, prover common.Address, newBytes *big.Int) (*types.Transaction, error) {
	return _ProverStaking.contract.Transact(opts, "commitBytes", prover, newBytes)
}

// CommitBytes is a paid mutator transaction binding the contract method 0xcb5c6ff4.
//
// Solidity: function commitBytes(address prover, uint256 newBytes) returns()
func (_ProverStaking *ProverStakingSession) CommitBytes(prover common.Address, newBytes *big.Int) (*types.Transaction, error) {
	return _ProverStaking.Contract.CommitBytes(&_ProverStaking.TransactOpts, prover, newBytes)
}

// CommitBytes is a paid mutator transaction binding the contract method 0xcb5c6ff4.
//
// Solidity: function commitBytes(address prover, uint256 newBytes) returns()
func (_ProverStaking *ProverStakingTransactorSession) CommitBytes(prover common.Address, newBytes *big.Int) (*types.Transaction, error) {
	return _ProverStaking.Contract.CommitBytes(&_ProverStaking.TransactOpts, prover, newBytes)
}

// ReleaseBytes is a paid mutator transaction binding the contract method 0x7f330062.
//
// Solidity: function releaseBytes(address prover, uint256 freedBytes) returns()
func (_ProverStaking *ProverStakingTransactor) ReleaseBytes(opts *bind.TransactOpts, prover common.Address, freedBytes *big.Int) (*types.Transaction, error) {
	return _ProverStaking.contract.Transact(opts, "releaseBytes", prover, freedBytes)
}

// ReleaseBytes is a paid mutator transaction binding the contract method 0x7f330062.
//
// Solidity: function releaseBytes(address prover, uint256 freedBytes) returns()
func (_ProverStaking *ProverStakingSession) ReleaseBytes(prover common.Address, freedBytes *big.Int) (*types.Transaction, error) {
	return _ProverStaking.Contract.ReleaseBytes(&_ProverStaking.TransactOpts, prover, freedBytes)
}

// ReleaseBytes is a paid mutator transaction binding the contract method 0x7f330062.
//
// Solidity: function releaseBytes(address prover, uint256 freedBytes) returns()
func (_ProverStaking *ProverStakingTransactorSession) ReleaseBytes(prover common.Address, freedBytes *big.Int) (*types.Transaction, error) {
	return _ProverStaking.Contract.ReleaseBytes(&_ProverStaking.TransactOpts, prover, freedBytes)
}

// RenounceOwnership is a paid mutator transaction binding the contract method 0x715018a6.
//
// Solidity: function renounceOwnership() returns()
func (_ProverStaking *ProverStakingTransactor) RenounceOwnership(opts *bind.TransactOpts) (*types.Transaction, error) {
	return _ProverStaking.contract.Transact(opts, "renounceOwnership")
}

// RenounceOwnership is a paid mutator transaction binding the contract method 0x715018a6.
//
// Solidity: function renounceOwnership() returns()
func (_ProverStaking *ProverStakingSession) RenounceOwnership() (*types.Transaction, error) {
	return _ProverStaking.Contract.RenounceOwnership(&_ProverStaking.TransactOpts)
}

// RenounceOwnership is a paid mutator transaction binding the contract method 0x715018a6.
//
// Solidity: function renounceOwnership() returns()
func (_ProverStaking *ProverStakingTransactorSession) RenounceOwnership() (*types.Transaction, error) {
	return _ProverStaking.Contract.RenounceOwnership(&_ProverStaking.TransactOpts)
}

// RequestUnstake is a paid mutator transaction binding the contract method 0x23095721.
//
// Solidity: function requestUnstake(uint256 amount) returns()
func (_ProverStaking *ProverStakingTransactor) RequestUnstake(opts *bind.TransactOpts, amount *big.Int) (*types.Transaction, error) {
	return _ProverStaking.contract.Transact(opts, "requestUnstake", amount)
}

// RequestUnstake is a paid mutator transaction binding the contract method 0x23095721.
//
// Solidity: function requestUnstake(uint256 amount) returns()
func (_ProverStaking *ProverStakingSession) RequestUnstake(amount *big.Int) (*types.Transaction, error) {
	return _ProverStaking.Contract.RequestUnstake(&_ProverStaking.TransactOpts, amount)
}

// RequestUnstake is a paid mutator transaction binding the contract method 0x23095721.
//
// Solidity: function requestUnstake(uint256 amount) returns()
func (_ProverStaking *ProverStakingTransactorSession) RequestUnstake(amount *big.Int) (*types.Transaction, error) {
	return _ProverStaking.Contract.RequestUnstake(&_ProverStaking.TransactOpts, amount)
}

// SetAuthorizedController is a paid mutator transaction binding the contract method 0xe07f0f14.
//
// Solidity: function setAuthorizedController(address controller, bool authorized) returns()
func (_ProverStaking *ProverStakingTransactor) SetAuthorizedController(opts *bind.TransactOpts, controller common.Address, authorized bool) (*types.Transaction, error) {
	return _ProverStaking.contract.Transact(opts, "setAuthorizedController", controller, authorized)
}

// SetAuthorizedController is a paid mutator transaction binding the contract method 0xe07f0f14.
//
// Solidity: function setAuthorizedController(address controller, bool authorized) returns()
func (_ProverStaking *ProverStakingSession) SetAuthorizedController(controller common.Address, authorized bool) (*types.Transaction, error) {
	return _ProverStaking.Contract.SetAuthorizedController(&_ProverStaking.TransactOpts, controller, authorized)
}

// SetAuthorizedController is a paid mutator transaction binding the contract method 0xe07f0f14.
//
// Solidity: function setAuthorizedController(address controller, bool authorized) returns()
func (_ProverStaking *ProverStakingTransactorSession) SetAuthorizedController(controller common.Address, authorized bool) (*types.Transaction, error) {
	return _ProverStaking.Contract.SetAuthorizedController(&_ProverStaking.TransactOpts, controller, authorized)
}

// SetMinStakePerGib is a paid mutator transaction binding the contract method 0x09c661bb.
//
// Solidity: function setMinStakePerGib(uint256 newValue) returns()
func (_ProverStaking *ProverStakingTransactor) SetMinStakePerGib(opts *bind.TransactOpts, newValue *big.Int) (*types.Transaction, error) {
	return _ProverStaking.contract.Transact(opts, "setMinStakePerGib", newValue)
}

// SetMinStakePerGib is a paid mutator transaction binding the contract method 0x09c661bb.
//
// Solidity: function setMinStakePerGib(uint256 newValue) returns()
func (_ProverStaking *ProverStakingSession) SetMinStakePerGib(newValue *big.Int) (*types.Transaction, error) {
	return _ProverStaking.Contract.SetMinStakePerGib(&_ProverStaking.TransactOpts, newValue)
}

// SetMinStakePerGib is a paid mutator transaction binding the contract method 0x09c661bb.
//
// Solidity: function setMinStakePerGib(uint256 newValue) returns()
func (_ProverStaking *ProverStakingTransactorSession) SetMinStakePerGib(newValue *big.Int) (*types.Transaction, error) {
	return _ProverStaking.Contract.SetMinStakePerGib(&_ProverStaking.TransactOpts, newValue)
}

// Slash is a paid mutator transaction binding the contract method 0xfba2c05a.
//
// Solidity: function slash(address prover, uint256 amount, bytes32 reason) returns()
func (_ProverStaking *ProverStakingTransactor) Slash(opts *bind.TransactOpts, prover common.Address, amount *big.Int, reason [32]byte) (*types.Transaction, error) {
	return _ProverStaking.contract.Transact(opts, "slash", prover, amount, reason)
}

// Slash is a paid mutator transaction binding the contract method 0xfba2c05a.
//
// Solidity: function slash(address prover, uint256 amount, bytes32 reason) returns()
func (_ProverStaking *ProverStakingSession) Slash(prover common.Address, amount *big.Int, reason [32]byte) (*types.Transaction, error) {
	return _ProverStaking.Contract.Slash(&_ProverStaking.TransactOpts, prover, amount, reason)
}

// Slash is a paid mutator transaction binding the contract method 0xfba2c05a.
//
// Solidity: function slash(address prover, uint256 amount, bytes32 reason) returns()
func (_ProverStaking *ProverStakingTransactorSession) Slash(prover common.Address, amount *big.Int, reason [32]byte) (*types.Transaction, error) {
	return _ProverStaking.Contract.Slash(&_ProverStaking.TransactOpts, prover, amount, reason)
}

// Stake is a paid mutator transaction binding the contract method 0xa694fc3a.
//
// Solidity: function stake(uint256 amount) returns()
func (_ProverStaking *ProverStakingTransactor) Stake(opts *bind.TransactOpts, amount *big.Int) (*types.Transaction, error) {
	return _ProverStaking.contract.Transact(opts, "stake", amount)
}

// Stake is a paid mutator transaction binding the contract method 0xa694fc3a.
//
// Solidity: function stake(uint256 amount) returns()
func (_ProverStaking *ProverStakingSession) Stake(amount *big.Int) (*types.Transaction, error) {
	return _ProverStaking.Contract.Stake(&_ProverStaking.TransactOpts, amount)
}

// Stake is a paid mutator transaction binding the contract method 0xa694fc3a.
//
// Solidity: function stake(uint256 amount) returns()
func (_ProverStaking *ProverStakingTransactorSession) Stake(amount *big.Int) (*types.Transaction, error) {
	return _ProverStaking.Contract.Stake(&_ProverStaking.TransactOpts, amount)
}

// TransferOwnership is a paid mutator transaction binding the contract method 0xf2fde38b.
//
// Solidity: function transferOwnership(address newOwner) returns()
func (_ProverStaking *ProverStakingTransactor) TransferOwnership(opts *bind.TransactOpts, newOwner common.Address) (*types.Transaction, error) {
	return _ProverStaking.contract.Transact(opts, "transferOwnership", newOwner)
}

// TransferOwnership is a paid mutator transaction binding the contract method 0xf2fde38b.
//
// Solidity: function transferOwnership(address newOwner) returns()
func (_ProverStaking *ProverStakingSession) TransferOwnership(newOwner common.Address) (*types.Transaction, error) {
	return _ProverStaking.Contract.TransferOwnership(&_ProverStaking.TransactOpts, newOwner)
}

// TransferOwnership is a paid mutator transaction binding the contract method 0xf2fde38b.
//
// Solidity: function transferOwnership(address newOwner) returns()
func (_ProverStaking *ProverStakingTransactorSession) TransferOwnership(newOwner common.Address) (*types.Transaction, error) {
	return _ProverStaking.Contract.TransferOwnership(&_ProverStaking.TransactOpts, newOwner)
}

// Withdraw is a paid mutator transaction binding the contract method 0x3ccfd60b.
//
// Solidity: function withdraw() returns()
func (_ProverStaking *ProverStakingTransactor) Withdraw(opts *bind.TransactOpts) (*types.Transaction, error) {
	return _ProverStaking.contract.Transact(opts, "withdraw")
}

// Withdraw is a paid mutator transaction binding the contract method 0x3ccfd60b.
//
// Solidity: function withdraw() returns()
func (_ProverStaking *ProverStakingSession) Withdraw() (*types.Transaction, error) {
	return _ProverStaking.Contract.Withdraw(&_ProverStaking.TransactOpts)
}

// Withdraw is a paid mutator transaction binding the contract method 0x3ccfd60b.
//
// Solidity: function withdraw() returns()
func (_ProverStaking *ProverStakingTransactorSession) Withdraw() (*types.Transaction, error) {
	return _ProverStaking.Contract.Withdraw(&_ProverStaking.TransactOpts)
}

// WithdrawSlashed is a paid mutator transaction binding the contract method 0xf0bfee4c.
//
// Solidity: function withdrawSlashed(address to, uint256 amount) returns()
func (_ProverStaking *ProverStakingTransactor) WithdrawSlashed(opts *bind.TransactOpts, to common.Address, amount *big.Int) (*types.Transaction, error) {
	return _ProverStaking.contract.Transact(opts, "withdrawSlashed", to, amount)
}

// WithdrawSlashed is a paid mutator transaction binding the contract method 0xf0bfee4c.
//
// Solidity: function withdrawSlashed(address to, uint256 amount) returns()
func (_ProverStaking *ProverStakingSession) WithdrawSlashed(to common.Address, amount *big.Int) (*types.Transaction, error) {
	return _ProverStaking.Contract.WithdrawSlashed(&_ProverStaking.TransactOpts, to, amount)
}

// WithdrawSlashed is a paid mutator transaction binding the contract method 0xf0bfee4c.
//
// Solidity: function withdrawSlashed(address to, uint256 amount) returns()
func (_ProverStaking *ProverStakingTransactorSession) WithdrawSlashed(to common.Address, amount *big.Int) (*types.Transaction, error) {
	return _ProverStaking.Contract.WithdrawSlashed(&_ProverStaking.TransactOpts, to, amount)
}

// ProverStakingAuthorizedControllerSetIterator is returned from FilterAuthorizedControllerSet and is used to iterate over the raw logs and unpacked data for AuthorizedControllerSet events raised by the ProverStaking contract.
type ProverStakingAuthorizedControllerSetIterator struct {
	Event *ProverStakingAuthorizedControllerSet // Event containing the contract specifics and raw log

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
func (it *ProverStakingAuthorizedControllerSetIterator) Next() bool {
	// If the iterator failed, stop iterating
	if it.fail != nil {
		return false
	}
	// If the iterator completed, deliver directly whatever's available
	if it.done {
		select {
		case log := <-it.logs:
			it.Event = new(ProverStakingAuthorizedControllerSet)
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
		it.Event = new(ProverStakingAuthorizedControllerSet)
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
func (it *ProverStakingAuthorizedControllerSetIterator) Error() error {
	return it.fail
}

// Close terminates the iteration process, releasing any pending underlying
// resources.
func (it *ProverStakingAuthorizedControllerSetIterator) Close() error {
	it.sub.Unsubscribe()
	return nil
}

// ProverStakingAuthorizedControllerSet represents a AuthorizedControllerSet event raised by the ProverStaking contract.
type ProverStakingAuthorizedControllerSet struct {
	Controller common.Address
	Authorized bool
	Raw        types.Log // Blockchain specific contextual infos
}

// FilterAuthorizedControllerSet is a free log retrieval operation binding the contract event 0xd767b21ed9c209f0b4e3280af8a32f4f55b4c81948bd8fb0e2e7df92f2f9c85d.
//
// Solidity: event AuthorizedControllerSet(address indexed controller, bool authorized)
func (_ProverStaking *ProverStakingFilterer) FilterAuthorizedControllerSet(opts *bind.FilterOpts, controller []common.Address) (*ProverStakingAuthorizedControllerSetIterator, error) {

	var controllerRule []interface{}
	for _, controllerItem := range controller {
		controllerRule = append(controllerRule, controllerItem)
	}

	logs, sub, err := _ProverStaking.contract.FilterLogs(opts, "AuthorizedControllerSet", controllerRule)
	if err != nil {
		return nil, err
	}
	return &ProverStakingAuthorizedControllerSetIterator{contract: _ProverStaking.contract, event: "AuthorizedControllerSet", logs: logs, sub: sub}, nil
}

// WatchAuthorizedControllerSet is a free log subscription operation binding the contract event 0xd767b21ed9c209f0b4e3280af8a32f4f55b4c81948bd8fb0e2e7df92f2f9c85d.
//
// Solidity: event AuthorizedControllerSet(address indexed controller, bool authorized)
func (_ProverStaking *ProverStakingFilterer) WatchAuthorizedControllerSet(opts *bind.WatchOpts, sink chan<- *ProverStakingAuthorizedControllerSet, controller []common.Address) (event.Subscription, error) {

	var controllerRule []interface{}
	for _, controllerItem := range controller {
		controllerRule = append(controllerRule, controllerItem)
	}

	logs, sub, err := _ProverStaking.contract.WatchLogs(opts, "AuthorizedControllerSet", controllerRule)
	if err != nil {
		return nil, err
	}
	return event.NewSubscription(func(quit <-chan struct{}) error {
		defer sub.Unsubscribe()
		for {
			select {
			case log := <-logs:
				// New log arrived, parse the event and forward to the user
				event := new(ProverStakingAuthorizedControllerSet)
				if err := _ProverStaking.contract.UnpackLog(event, "AuthorizedControllerSet", log); err != nil {
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

// ParseAuthorizedControllerSet is a log parse operation binding the contract event 0xd767b21ed9c209f0b4e3280af8a32f4f55b4c81948bd8fb0e2e7df92f2f9c85d.
//
// Solidity: event AuthorizedControllerSet(address indexed controller, bool authorized)
func (_ProverStaking *ProverStakingFilterer) ParseAuthorizedControllerSet(log types.Log) (*ProverStakingAuthorizedControllerSet, error) {
	event := new(ProverStakingAuthorizedControllerSet)
	if err := _ProverStaking.contract.UnpackLog(event, "AuthorizedControllerSet", log); err != nil {
		return nil, err
	}
	event.Raw = log
	return event, nil
}

// ProverStakingCommittedBytesChangedIterator is returned from FilterCommittedBytesChanged and is used to iterate over the raw logs and unpacked data for CommittedBytesChanged events raised by the ProverStaking contract.
type ProverStakingCommittedBytesChangedIterator struct {
	Event *ProverStakingCommittedBytesChanged // Event containing the contract specifics and raw log

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
func (it *ProverStakingCommittedBytesChangedIterator) Next() bool {
	// If the iterator failed, stop iterating
	if it.fail != nil {
		return false
	}
	// If the iterator completed, deliver directly whatever's available
	if it.done {
		select {
		case log := <-it.logs:
			it.Event = new(ProverStakingCommittedBytesChanged)
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
		it.Event = new(ProverStakingCommittedBytesChanged)
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
func (it *ProverStakingCommittedBytesChangedIterator) Error() error {
	return it.fail
}

// Close terminates the iteration process, releasing any pending underlying
// resources.
func (it *ProverStakingCommittedBytesChangedIterator) Close() error {
	it.sub.Unsubscribe()
	return nil
}

// ProverStakingCommittedBytesChanged represents a CommittedBytesChanged event raised by the ProverStaking contract.
type ProverStakingCommittedBytesChanged struct {
	Prover            common.Address
	NewCommittedBytes *big.Int
	Raw               types.Log // Blockchain specific contextual infos
}

// FilterCommittedBytesChanged is a free log retrieval operation binding the contract event 0xfaa44185b159c9e959d2e94c26eb27c317e9f607e05d37d206b775eece86bbe5.
//
// Solidity: event CommittedBytesChanged(address indexed prover, uint256 newCommittedBytes)
func (_ProverStaking *ProverStakingFilterer) FilterCommittedBytesChanged(opts *bind.FilterOpts, prover []common.Address) (*ProverStakingCommittedBytesChangedIterator, error) {

	var proverRule []interface{}
	for _, proverItem := range prover {
		proverRule = append(proverRule, proverItem)
	}

	logs, sub, err := _ProverStaking.contract.FilterLogs(opts, "CommittedBytesChanged", proverRule)
	if err != nil {
		return nil, err
	}
	return &ProverStakingCommittedBytesChangedIterator{contract: _ProverStaking.contract, event: "CommittedBytesChanged", logs: logs, sub: sub}, nil
}

// WatchCommittedBytesChanged is a free log subscription operation binding the contract event 0xfaa44185b159c9e959d2e94c26eb27c317e9f607e05d37d206b775eece86bbe5.
//
// Solidity: event CommittedBytesChanged(address indexed prover, uint256 newCommittedBytes)
func (_ProverStaking *ProverStakingFilterer) WatchCommittedBytesChanged(opts *bind.WatchOpts, sink chan<- *ProverStakingCommittedBytesChanged, prover []common.Address) (event.Subscription, error) {

	var proverRule []interface{}
	for _, proverItem := range prover {
		proverRule = append(proverRule, proverItem)
	}

	logs, sub, err := _ProverStaking.contract.WatchLogs(opts, "CommittedBytesChanged", proverRule)
	if err != nil {
		return nil, err
	}
	return event.NewSubscription(func(quit <-chan struct{}) error {
		defer sub.Unsubscribe()
		for {
			select {
			case log := <-logs:
				// New log arrived, parse the event and forward to the user
				event := new(ProverStakingCommittedBytesChanged)
				if err := _ProverStaking.contract.UnpackLog(event, "CommittedBytesChanged", log); err != nil {
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

// ParseCommittedBytesChanged is a log parse operation binding the contract event 0xfaa44185b159c9e959d2e94c26eb27c317e9f607e05d37d206b775eece86bbe5.
//
// Solidity: event CommittedBytesChanged(address indexed prover, uint256 newCommittedBytes)
func (_ProverStaking *ProverStakingFilterer) ParseCommittedBytesChanged(log types.Log) (*ProverStakingCommittedBytesChanged, error) {
	event := new(ProverStakingCommittedBytesChanged)
	if err := _ProverStaking.contract.UnpackLog(event, "CommittedBytesChanged", log); err != nil {
		return nil, err
	}
	event.Raw = log
	return event, nil
}

// ProverStakingMinStakePerGibChangedIterator is returned from FilterMinStakePerGibChanged and is used to iterate over the raw logs and unpacked data for MinStakePerGibChanged events raised by the ProverStaking contract.
type ProverStakingMinStakePerGibChangedIterator struct {
	Event *ProverStakingMinStakePerGibChanged // Event containing the contract specifics and raw log

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
func (it *ProverStakingMinStakePerGibChangedIterator) Next() bool {
	// If the iterator failed, stop iterating
	if it.fail != nil {
		return false
	}
	// If the iterator completed, deliver directly whatever's available
	if it.done {
		select {
		case log := <-it.logs:
			it.Event = new(ProverStakingMinStakePerGibChanged)
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
		it.Event = new(ProverStakingMinStakePerGibChanged)
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
func (it *ProverStakingMinStakePerGibChangedIterator) Error() error {
	return it.fail
}

// Close terminates the iteration process, releasing any pending underlying
// resources.
func (it *ProverStakingMinStakePerGibChangedIterator) Close() error {
	it.sub.Unsubscribe()
	return nil
}

// ProverStakingMinStakePerGibChanged represents a MinStakePerGibChanged event raised by the ProverStaking contract.
type ProverStakingMinStakePerGibChanged struct {
	OldValue *big.Int
	NewValue *big.Int
	Raw      types.Log // Blockchain specific contextual infos
}

// FilterMinStakePerGibChanged is a free log retrieval operation binding the contract event 0xdcbfa78fc8ad2deb48caa409ffb7fa7fbcf8064fa61d57eb204b3fd0a56a3ab6.
//
// Solidity: event MinStakePerGibChanged(uint256 oldValue, uint256 newValue)
func (_ProverStaking *ProverStakingFilterer) FilterMinStakePerGibChanged(opts *bind.FilterOpts) (*ProverStakingMinStakePerGibChangedIterator, error) {

	logs, sub, err := _ProverStaking.contract.FilterLogs(opts, "MinStakePerGibChanged")
	if err != nil {
		return nil, err
	}
	return &ProverStakingMinStakePerGibChangedIterator{contract: _ProverStaking.contract, event: "MinStakePerGibChanged", logs: logs, sub: sub}, nil
}

// WatchMinStakePerGibChanged is a free log subscription operation binding the contract event 0xdcbfa78fc8ad2deb48caa409ffb7fa7fbcf8064fa61d57eb204b3fd0a56a3ab6.
//
// Solidity: event MinStakePerGibChanged(uint256 oldValue, uint256 newValue)
func (_ProverStaking *ProverStakingFilterer) WatchMinStakePerGibChanged(opts *bind.WatchOpts, sink chan<- *ProverStakingMinStakePerGibChanged) (event.Subscription, error) {

	logs, sub, err := _ProverStaking.contract.WatchLogs(opts, "MinStakePerGibChanged")
	if err != nil {
		return nil, err
	}
	return event.NewSubscription(func(quit <-chan struct{}) error {
		defer sub.Unsubscribe()
		for {
			select {
			case log := <-logs:
				// New log arrived, parse the event and forward to the user
				event := new(ProverStakingMinStakePerGibChanged)
				if err := _ProverStaking.contract.UnpackLog(event, "MinStakePerGibChanged", log); err != nil {
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

// ParseMinStakePerGibChanged is a log parse operation binding the contract event 0xdcbfa78fc8ad2deb48caa409ffb7fa7fbcf8064fa61d57eb204b3fd0a56a3ab6.
//
// Solidity: event MinStakePerGibChanged(uint256 oldValue, uint256 newValue)
func (_ProverStaking *ProverStakingFilterer) ParseMinStakePerGibChanged(log types.Log) (*ProverStakingMinStakePerGibChanged, error) {
	event := new(ProverStakingMinStakePerGibChanged)
	if err := _ProverStaking.contract.UnpackLog(event, "MinStakePerGibChanged", log); err != nil {
		return nil, err
	}
	event.Raw = log
	return event, nil
}

// ProverStakingOwnershipTransferredIterator is returned from FilterOwnershipTransferred and is used to iterate over the raw logs and unpacked data for OwnershipTransferred events raised by the ProverStaking contract.
type ProverStakingOwnershipTransferredIterator struct {
	Event *ProverStakingOwnershipTransferred // Event containing the contract specifics and raw log

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
func (it *ProverStakingOwnershipTransferredIterator) Next() bool {
	// If the iterator failed, stop iterating
	if it.fail != nil {
		return false
	}
	// If the iterator completed, deliver directly whatever's available
	if it.done {
		select {
		case log := <-it.logs:
			it.Event = new(ProverStakingOwnershipTransferred)
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
		it.Event = new(ProverStakingOwnershipTransferred)
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
func (it *ProverStakingOwnershipTransferredIterator) Error() error {
	return it.fail
}

// Close terminates the iteration process, releasing any pending underlying
// resources.
func (it *ProverStakingOwnershipTransferredIterator) Close() error {
	it.sub.Unsubscribe()
	return nil
}

// ProverStakingOwnershipTransferred represents a OwnershipTransferred event raised by the ProverStaking contract.
type ProverStakingOwnershipTransferred struct {
	PreviousOwner common.Address
	NewOwner      common.Address
	Raw           types.Log // Blockchain specific contextual infos
}

// FilterOwnershipTransferred is a free log retrieval operation binding the contract event 0x8be0079c531659141344cd1fd0a4f28419497f9722a3daafe3b4186f6b6457e0.
//
// Solidity: event OwnershipTransferred(address indexed previousOwner, address indexed newOwner)
func (_ProverStaking *ProverStakingFilterer) FilterOwnershipTransferred(opts *bind.FilterOpts, previousOwner []common.Address, newOwner []common.Address) (*ProverStakingOwnershipTransferredIterator, error) {

	var previousOwnerRule []interface{}
	for _, previousOwnerItem := range previousOwner {
		previousOwnerRule = append(previousOwnerRule, previousOwnerItem)
	}
	var newOwnerRule []interface{}
	for _, newOwnerItem := range newOwner {
		newOwnerRule = append(newOwnerRule, newOwnerItem)
	}

	logs, sub, err := _ProverStaking.contract.FilterLogs(opts, "OwnershipTransferred", previousOwnerRule, newOwnerRule)
	if err != nil {
		return nil, err
	}
	return &ProverStakingOwnershipTransferredIterator{contract: _ProverStaking.contract, event: "OwnershipTransferred", logs: logs, sub: sub}, nil
}

// WatchOwnershipTransferred is a free log subscription operation binding the contract event 0x8be0079c531659141344cd1fd0a4f28419497f9722a3daafe3b4186f6b6457e0.
//
// Solidity: event OwnershipTransferred(address indexed previousOwner, address indexed newOwner)
func (_ProverStaking *ProverStakingFilterer) WatchOwnershipTransferred(opts *bind.WatchOpts, sink chan<- *ProverStakingOwnershipTransferred, previousOwner []common.Address, newOwner []common.Address) (event.Subscription, error) {

	var previousOwnerRule []interface{}
	for _, previousOwnerItem := range previousOwner {
		previousOwnerRule = append(previousOwnerRule, previousOwnerItem)
	}
	var newOwnerRule []interface{}
	for _, newOwnerItem := range newOwner {
		newOwnerRule = append(newOwnerRule, newOwnerItem)
	}

	logs, sub, err := _ProverStaking.contract.WatchLogs(opts, "OwnershipTransferred", previousOwnerRule, newOwnerRule)
	if err != nil {
		return nil, err
	}
	return event.NewSubscription(func(quit <-chan struct{}) error {
		defer sub.Unsubscribe()
		for {
			select {
			case log := <-logs:
				// New log arrived, parse the event and forward to the user
				event := new(ProverStakingOwnershipTransferred)
				if err := _ProverStaking.contract.UnpackLog(event, "OwnershipTransferred", log); err != nil {
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
func (_ProverStaking *ProverStakingFilterer) ParseOwnershipTransferred(log types.Log) (*ProverStakingOwnershipTransferred, error) {
	event := new(ProverStakingOwnershipTransferred)
	if err := _ProverStaking.contract.UnpackLog(event, "OwnershipTransferred", log); err != nil {
		return nil, err
	}
	event.Raw = log
	return event, nil
}

// ProverStakingSlashedIterator is returned from FilterSlashed and is used to iterate over the raw logs and unpacked data for Slashed events raised by the ProverStaking contract.
type ProverStakingSlashedIterator struct {
	Event *ProverStakingSlashed // Event containing the contract specifics and raw log

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
func (it *ProverStakingSlashedIterator) Next() bool {
	// If the iterator failed, stop iterating
	if it.fail != nil {
		return false
	}
	// If the iterator completed, deliver directly whatever's available
	if it.done {
		select {
		case log := <-it.logs:
			it.Event = new(ProverStakingSlashed)
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
		it.Event = new(ProverStakingSlashed)
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
func (it *ProverStakingSlashedIterator) Error() error {
	return it.fail
}

// Close terminates the iteration process, releasing any pending underlying
// resources.
func (it *ProverStakingSlashedIterator) Close() error {
	it.sub.Unsubscribe()
	return nil
}

// ProverStakingSlashed represents a Slashed event raised by the ProverStaking contract.
type ProverStakingSlashed struct {
	Prover common.Address
	Amount *big.Int
	By     common.Address
	Reason [32]byte
	Raw    types.Log // Blockchain specific contextual infos
}

// FilterSlashed is a free log retrieval operation binding the contract event 0x365a9a525a3155795a8654bb76715d57cb544a4c9053c66d468ad288684e0508.
//
// Solidity: event Slashed(address indexed prover, uint256 amount, address indexed by, bytes32 reason)
func (_ProverStaking *ProverStakingFilterer) FilterSlashed(opts *bind.FilterOpts, prover []common.Address, by []common.Address) (*ProverStakingSlashedIterator, error) {

	var proverRule []interface{}
	for _, proverItem := range prover {
		proverRule = append(proverRule, proverItem)
	}

	var byRule []interface{}
	for _, byItem := range by {
		byRule = append(byRule, byItem)
	}

	logs, sub, err := _ProverStaking.contract.FilterLogs(opts, "Slashed", proverRule, byRule)
	if err != nil {
		return nil, err
	}
	return &ProverStakingSlashedIterator{contract: _ProverStaking.contract, event: "Slashed", logs: logs, sub: sub}, nil
}

// WatchSlashed is a free log subscription operation binding the contract event 0x365a9a525a3155795a8654bb76715d57cb544a4c9053c66d468ad288684e0508.
//
// Solidity: event Slashed(address indexed prover, uint256 amount, address indexed by, bytes32 reason)
func (_ProverStaking *ProverStakingFilterer) WatchSlashed(opts *bind.WatchOpts, sink chan<- *ProverStakingSlashed, prover []common.Address, by []common.Address) (event.Subscription, error) {

	var proverRule []interface{}
	for _, proverItem := range prover {
		proverRule = append(proverRule, proverItem)
	}

	var byRule []interface{}
	for _, byItem := range by {
		byRule = append(byRule, byItem)
	}

	logs, sub, err := _ProverStaking.contract.WatchLogs(opts, "Slashed", proverRule, byRule)
	if err != nil {
		return nil, err
	}
	return event.NewSubscription(func(quit <-chan struct{}) error {
		defer sub.Unsubscribe()
		for {
			select {
			case log := <-logs:
				// New log arrived, parse the event and forward to the user
				event := new(ProverStakingSlashed)
				if err := _ProverStaking.contract.UnpackLog(event, "Slashed", log); err != nil {
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

// ParseSlashed is a log parse operation binding the contract event 0x365a9a525a3155795a8654bb76715d57cb544a4c9053c66d468ad288684e0508.
//
// Solidity: event Slashed(address indexed prover, uint256 amount, address indexed by, bytes32 reason)
func (_ProverStaking *ProverStakingFilterer) ParseSlashed(log types.Log) (*ProverStakingSlashed, error) {
	event := new(ProverStakingSlashed)
	if err := _ProverStaking.contract.UnpackLog(event, "Slashed", log); err != nil {
		return nil, err
	}
	event.Raw = log
	return event, nil
}

// ProverStakingSlashedPoolWithdrawnIterator is returned from FilterSlashedPoolWithdrawn and is used to iterate over the raw logs and unpacked data for SlashedPoolWithdrawn events raised by the ProverStaking contract.
type ProverStakingSlashedPoolWithdrawnIterator struct {
	Event *ProverStakingSlashedPoolWithdrawn // Event containing the contract specifics and raw log

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
func (it *ProverStakingSlashedPoolWithdrawnIterator) Next() bool {
	// If the iterator failed, stop iterating
	if it.fail != nil {
		return false
	}
	// If the iterator completed, deliver directly whatever's available
	if it.done {
		select {
		case log := <-it.logs:
			it.Event = new(ProverStakingSlashedPoolWithdrawn)
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
		it.Event = new(ProverStakingSlashedPoolWithdrawn)
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
func (it *ProverStakingSlashedPoolWithdrawnIterator) Error() error {
	return it.fail
}

// Close terminates the iteration process, releasing any pending underlying
// resources.
func (it *ProverStakingSlashedPoolWithdrawnIterator) Close() error {
	it.sub.Unsubscribe()
	return nil
}

// ProverStakingSlashedPoolWithdrawn represents a SlashedPoolWithdrawn event raised by the ProverStaking contract.
type ProverStakingSlashedPoolWithdrawn struct {
	To     common.Address
	Amount *big.Int
	Raw    types.Log // Blockchain specific contextual infos
}

// FilterSlashedPoolWithdrawn is a free log retrieval operation binding the contract event 0x5c10e125938f053d189065da760df2726397998496fd7b13ae8e334ffdea7c7b.
//
// Solidity: event SlashedPoolWithdrawn(address indexed to, uint256 amount)
func (_ProverStaking *ProverStakingFilterer) FilterSlashedPoolWithdrawn(opts *bind.FilterOpts, to []common.Address) (*ProverStakingSlashedPoolWithdrawnIterator, error) {

	var toRule []interface{}
	for _, toItem := range to {
		toRule = append(toRule, toItem)
	}

	logs, sub, err := _ProverStaking.contract.FilterLogs(opts, "SlashedPoolWithdrawn", toRule)
	if err != nil {
		return nil, err
	}
	return &ProverStakingSlashedPoolWithdrawnIterator{contract: _ProverStaking.contract, event: "SlashedPoolWithdrawn", logs: logs, sub: sub}, nil
}

// WatchSlashedPoolWithdrawn is a free log subscription operation binding the contract event 0x5c10e125938f053d189065da760df2726397998496fd7b13ae8e334ffdea7c7b.
//
// Solidity: event SlashedPoolWithdrawn(address indexed to, uint256 amount)
func (_ProverStaking *ProverStakingFilterer) WatchSlashedPoolWithdrawn(opts *bind.WatchOpts, sink chan<- *ProverStakingSlashedPoolWithdrawn, to []common.Address) (event.Subscription, error) {

	var toRule []interface{}
	for _, toItem := range to {
		toRule = append(toRule, toItem)
	}

	logs, sub, err := _ProverStaking.contract.WatchLogs(opts, "SlashedPoolWithdrawn", toRule)
	if err != nil {
		return nil, err
	}
	return event.NewSubscription(func(quit <-chan struct{}) error {
		defer sub.Unsubscribe()
		for {
			select {
			case log := <-logs:
				// New log arrived, parse the event and forward to the user
				event := new(ProverStakingSlashedPoolWithdrawn)
				if err := _ProverStaking.contract.UnpackLog(event, "SlashedPoolWithdrawn", log); err != nil {
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

// ParseSlashedPoolWithdrawn is a log parse operation binding the contract event 0x5c10e125938f053d189065da760df2726397998496fd7b13ae8e334ffdea7c7b.
//
// Solidity: event SlashedPoolWithdrawn(address indexed to, uint256 amount)
func (_ProverStaking *ProverStakingFilterer) ParseSlashedPoolWithdrawn(log types.Log) (*ProverStakingSlashedPoolWithdrawn, error) {
	event := new(ProverStakingSlashedPoolWithdrawn)
	if err := _ProverStaking.contract.UnpackLog(event, "SlashedPoolWithdrawn", log); err != nil {
		return nil, err
	}
	event.Raw = log
	return event, nil
}

// ProverStakingStakedIterator is returned from FilterStaked and is used to iterate over the raw logs and unpacked data for Staked events raised by the ProverStaking contract.
type ProverStakingStakedIterator struct {
	Event *ProverStakingStaked // Event containing the contract specifics and raw log

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
func (it *ProverStakingStakedIterator) Next() bool {
	// If the iterator failed, stop iterating
	if it.fail != nil {
		return false
	}
	// If the iterator completed, deliver directly whatever's available
	if it.done {
		select {
		case log := <-it.logs:
			it.Event = new(ProverStakingStaked)
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
		it.Event = new(ProverStakingStaked)
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
func (it *ProverStakingStakedIterator) Error() error {
	return it.fail
}

// Close terminates the iteration process, releasing any pending underlying
// resources.
func (it *ProverStakingStakedIterator) Close() error {
	it.sub.Unsubscribe()
	return nil
}

// ProverStakingStaked represents a Staked event raised by the ProverStaking contract.
type ProverStakingStaked struct {
	Prover   common.Address
	Amount   *big.Int
	NewTotal *big.Int
	Raw      types.Log // Blockchain specific contextual infos
}

// FilterStaked is a free log retrieval operation binding the contract event 0x1449c6dd7851abc30abf37f57715f492010519147cc2652fbc38202c18a6ee90.
//
// Solidity: event Staked(address indexed prover, uint256 amount, uint256 newTotal)
func (_ProverStaking *ProverStakingFilterer) FilterStaked(opts *bind.FilterOpts, prover []common.Address) (*ProverStakingStakedIterator, error) {

	var proverRule []interface{}
	for _, proverItem := range prover {
		proverRule = append(proverRule, proverItem)
	}

	logs, sub, err := _ProverStaking.contract.FilterLogs(opts, "Staked", proverRule)
	if err != nil {
		return nil, err
	}
	return &ProverStakingStakedIterator{contract: _ProverStaking.contract, event: "Staked", logs: logs, sub: sub}, nil
}

// WatchStaked is a free log subscription operation binding the contract event 0x1449c6dd7851abc30abf37f57715f492010519147cc2652fbc38202c18a6ee90.
//
// Solidity: event Staked(address indexed prover, uint256 amount, uint256 newTotal)
func (_ProverStaking *ProverStakingFilterer) WatchStaked(opts *bind.WatchOpts, sink chan<- *ProverStakingStaked, prover []common.Address) (event.Subscription, error) {

	var proverRule []interface{}
	for _, proverItem := range prover {
		proverRule = append(proverRule, proverItem)
	}

	logs, sub, err := _ProverStaking.contract.WatchLogs(opts, "Staked", proverRule)
	if err != nil {
		return nil, err
	}
	return event.NewSubscription(func(quit <-chan struct{}) error {
		defer sub.Unsubscribe()
		for {
			select {
			case log := <-logs:
				// New log arrived, parse the event and forward to the user
				event := new(ProverStakingStaked)
				if err := _ProverStaking.contract.UnpackLog(event, "Staked", log); err != nil {
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

// ParseStaked is a log parse operation binding the contract event 0x1449c6dd7851abc30abf37f57715f492010519147cc2652fbc38202c18a6ee90.
//
// Solidity: event Staked(address indexed prover, uint256 amount, uint256 newTotal)
func (_ProverStaking *ProverStakingFilterer) ParseStaked(log types.Log) (*ProverStakingStaked, error) {
	event := new(ProverStakingStaked)
	if err := _ProverStaking.contract.UnpackLog(event, "Staked", log); err != nil {
		return nil, err
	}
	event.Raw = log
	return event, nil
}

// ProverStakingUnstakeRequestedIterator is returned from FilterUnstakeRequested and is used to iterate over the raw logs and unpacked data for UnstakeRequested events raised by the ProverStaking contract.
type ProverStakingUnstakeRequestedIterator struct {
	Event *ProverStakingUnstakeRequested // Event containing the contract specifics and raw log

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
func (it *ProverStakingUnstakeRequestedIterator) Next() bool {
	// If the iterator failed, stop iterating
	if it.fail != nil {
		return false
	}
	// If the iterator completed, deliver directly whatever's available
	if it.done {
		select {
		case log := <-it.logs:
			it.Event = new(ProverStakingUnstakeRequested)
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
		it.Event = new(ProverStakingUnstakeRequested)
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
func (it *ProverStakingUnstakeRequestedIterator) Error() error {
	return it.fail
}

// Close terminates the iteration process, releasing any pending underlying
// resources.
func (it *ProverStakingUnstakeRequestedIterator) Close() error {
	it.sub.Unsubscribe()
	return nil
}

// ProverStakingUnstakeRequested represents a UnstakeRequested event raised by the ProverStaking contract.
type ProverStakingUnstakeRequested struct {
	Prover common.Address
	Amount *big.Int
	EndsAt *big.Int
	Raw    types.Log // Blockchain specific contextual infos
}

// FilterUnstakeRequested is a free log retrieval operation binding the contract event 0x57e41df54512c76148b5ba9b643d149752b0d35e493b969bd017d0a3fe5228cf.
//
// Solidity: event UnstakeRequested(address indexed prover, uint256 amount, uint256 endsAt)
func (_ProverStaking *ProverStakingFilterer) FilterUnstakeRequested(opts *bind.FilterOpts, prover []common.Address) (*ProverStakingUnstakeRequestedIterator, error) {

	var proverRule []interface{}
	for _, proverItem := range prover {
		proverRule = append(proverRule, proverItem)
	}

	logs, sub, err := _ProverStaking.contract.FilterLogs(opts, "UnstakeRequested", proverRule)
	if err != nil {
		return nil, err
	}
	return &ProverStakingUnstakeRequestedIterator{contract: _ProverStaking.contract, event: "UnstakeRequested", logs: logs, sub: sub}, nil
}

// WatchUnstakeRequested is a free log subscription operation binding the contract event 0x57e41df54512c76148b5ba9b643d149752b0d35e493b969bd017d0a3fe5228cf.
//
// Solidity: event UnstakeRequested(address indexed prover, uint256 amount, uint256 endsAt)
func (_ProverStaking *ProverStakingFilterer) WatchUnstakeRequested(opts *bind.WatchOpts, sink chan<- *ProverStakingUnstakeRequested, prover []common.Address) (event.Subscription, error) {

	var proverRule []interface{}
	for _, proverItem := range prover {
		proverRule = append(proverRule, proverItem)
	}

	logs, sub, err := _ProverStaking.contract.WatchLogs(opts, "UnstakeRequested", proverRule)
	if err != nil {
		return nil, err
	}
	return event.NewSubscription(func(quit <-chan struct{}) error {
		defer sub.Unsubscribe()
		for {
			select {
			case log := <-logs:
				// New log arrived, parse the event and forward to the user
				event := new(ProverStakingUnstakeRequested)
				if err := _ProverStaking.contract.UnpackLog(event, "UnstakeRequested", log); err != nil {
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

// ParseUnstakeRequested is a log parse operation binding the contract event 0x57e41df54512c76148b5ba9b643d149752b0d35e493b969bd017d0a3fe5228cf.
//
// Solidity: event UnstakeRequested(address indexed prover, uint256 amount, uint256 endsAt)
func (_ProverStaking *ProverStakingFilterer) ParseUnstakeRequested(log types.Log) (*ProverStakingUnstakeRequested, error) {
	event := new(ProverStakingUnstakeRequested)
	if err := _ProverStaking.contract.UnpackLog(event, "UnstakeRequested", log); err != nil {
		return nil, err
	}
	event.Raw = log
	return event, nil
}

// ProverStakingWithdrawnIterator is returned from FilterWithdrawn and is used to iterate over the raw logs and unpacked data for Withdrawn events raised by the ProverStaking contract.
type ProverStakingWithdrawnIterator struct {
	Event *ProverStakingWithdrawn // Event containing the contract specifics and raw log

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
func (it *ProverStakingWithdrawnIterator) Next() bool {
	// If the iterator failed, stop iterating
	if it.fail != nil {
		return false
	}
	// If the iterator completed, deliver directly whatever's available
	if it.done {
		select {
		case log := <-it.logs:
			it.Event = new(ProverStakingWithdrawn)
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
		it.Event = new(ProverStakingWithdrawn)
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
func (it *ProverStakingWithdrawnIterator) Error() error {
	return it.fail
}

// Close terminates the iteration process, releasing any pending underlying
// resources.
func (it *ProverStakingWithdrawnIterator) Close() error {
	it.sub.Unsubscribe()
	return nil
}

// ProverStakingWithdrawn represents a Withdrawn event raised by the ProverStaking contract.
type ProverStakingWithdrawn struct {
	Prover common.Address
	Amount *big.Int
	Raw    types.Log // Blockchain specific contextual infos
}

// FilterWithdrawn is a free log retrieval operation binding the contract event 0x7084f5476618d8e60b11ef0d7d3f06914655adb8793e28ff7f018d4c76d505d5.
//
// Solidity: event Withdrawn(address indexed prover, uint256 amount)
func (_ProverStaking *ProverStakingFilterer) FilterWithdrawn(opts *bind.FilterOpts, prover []common.Address) (*ProverStakingWithdrawnIterator, error) {

	var proverRule []interface{}
	for _, proverItem := range prover {
		proverRule = append(proverRule, proverItem)
	}

	logs, sub, err := _ProverStaking.contract.FilterLogs(opts, "Withdrawn", proverRule)
	if err != nil {
		return nil, err
	}
	return &ProverStakingWithdrawnIterator{contract: _ProverStaking.contract, event: "Withdrawn", logs: logs, sub: sub}, nil
}

// WatchWithdrawn is a free log subscription operation binding the contract event 0x7084f5476618d8e60b11ef0d7d3f06914655adb8793e28ff7f018d4c76d505d5.
//
// Solidity: event Withdrawn(address indexed prover, uint256 amount)
func (_ProverStaking *ProverStakingFilterer) WatchWithdrawn(opts *bind.WatchOpts, sink chan<- *ProverStakingWithdrawn, prover []common.Address) (event.Subscription, error) {

	var proverRule []interface{}
	for _, proverItem := range prover {
		proverRule = append(proverRule, proverItem)
	}

	logs, sub, err := _ProverStaking.contract.WatchLogs(opts, "Withdrawn", proverRule)
	if err != nil {
		return nil, err
	}
	return event.NewSubscription(func(quit <-chan struct{}) error {
		defer sub.Unsubscribe()
		for {
			select {
			case log := <-logs:
				// New log arrived, parse the event and forward to the user
				event := new(ProverStakingWithdrawn)
				if err := _ProverStaking.contract.UnpackLog(event, "Withdrawn", log); err != nil {
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

// ParseWithdrawn is a log parse operation binding the contract event 0x7084f5476618d8e60b11ef0d7d3f06914655adb8793e28ff7f018d4c76d505d5.
//
// Solidity: event Withdrawn(address indexed prover, uint256 amount)
func (_ProverStaking *ProverStakingFilterer) ParseWithdrawn(log types.Log) (*ProverStakingWithdrawn, error) {
	event := new(ProverStakingWithdrawn)
	if err := _ProverStaking.contract.UnpackLog(event, "Withdrawn", log); err != nil {
		return nil, err
	}
	event.Raw = log
	return event, nil
}
