// SPDX-License-Identifier: MIT
// Generated from contracts/out/StorageMarketplace.sol/StorageMarketplace.json via abigen.
// Do not edit by hand; run ./scripts/gen-bindings.sh instead.

// Code generated - DO NOT EDIT.
// This file is a generated binding and any manual changes will be lost.

package storagemarketplace

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

// CidsCid is an auto generated low-level Go binding around an user-defined struct.
type CidsCid struct {
	Data []byte
}

// StorageMarketplaceDeal is an auto generated low-level Go binding around an user-defined struct.
type StorageMarketplaceDeal struct {
	Client       common.Address
	Prover       common.Address
	CommpHash    [32]byte
	PieceSize    uint64
	StartedAt    uint64
	EndsAt       uint64
	DataSetId    *big.Int
	TotalPayment *big.Int
	PaidOut      *big.Int
	LastProofAt  *big.Int
	ProofCount   *big.Int
	Status       uint8
}

// StorageMarketplaceMetaData contains all meta data concerning the StorageMarketplace contract.
var StorageMarketplaceMetaData = &bind.MetaData{
	ABI: "[{\"type\":\"constructor\",\"inputs\":[{\"name\":\"_proofVerifier\",\"type\":\"address\",\"internalType\":\"address\"},{\"name\":\"_paymentToken\",\"type\":\"address\",\"internalType\":\"contractIERC20\"},{\"name\":\"_proverRegistry\",\"type\":\"address\",\"internalType\":\"contractProverRegistry\"},{\"name\":\"_proverStaking\",\"type\":\"address\",\"internalType\":\"contractProverStaking\"},{\"name\":\"_contentRegistry\",\"type\":\"address\",\"internalType\":\"contractContentRegistry\"},{\"name\":\"_treasury\",\"type\":\"address\",\"internalType\":\"address\"},{\"name\":\"_slashPerFault\",\"type\":\"uint256\",\"internalType\":\"uint256\"}],\"stateMutability\":\"nonpayable\"},{\"type\":\"function\",\"name\":\"BPS_DENOMINATOR\",\"inputs\":[],\"outputs\":[{\"name\":\"\",\"type\":\"uint256\",\"internalType\":\"uint256\"}],\"stateMutability\":\"view\"},{\"type\":\"function\",\"name\":\"MAX_DEAL_DURATION\",\"inputs\":[],\"outputs\":[{\"name\":\"\",\"type\":\"uint256\",\"internalType\":\"uint256\"}],\"stateMutability\":\"view\"},{\"type\":\"function\",\"name\":\"MAX_PROOF_GAP\",\"inputs\":[],\"outputs\":[{\"name\":\"\",\"type\":\"uint256\",\"internalType\":\"uint256\"}],\"stateMutability\":\"view\"},{\"type\":\"function\",\"name\":\"MIN_DEAL_DURATION\",\"inputs\":[],\"outputs\":[{\"name\":\"\",\"type\":\"uint256\",\"internalType\":\"uint256\"}],\"stateMutability\":\"view\"},{\"type\":\"function\",\"name\":\"cancelProposedDeal\",\"inputs\":[{\"name\":\"dealId\",\"type\":\"uint256\",\"internalType\":\"uint256\"}],\"outputs\":[],\"stateMutability\":\"nonpayable\"},{\"type\":\"function\",\"name\":\"completeDeal\",\"inputs\":[{\"name\":\"dealId\",\"type\":\"uint256\",\"internalType\":\"uint256\"}],\"outputs\":[],\"stateMutability\":\"nonpayable\"},{\"type\":\"function\",\"name\":\"contentRegistry\",\"inputs\":[],\"outputs\":[{\"name\":\"\",\"type\":\"address\",\"internalType\":\"contractContentRegistry\"}],\"stateMutability\":\"view\"},{\"type\":\"function\",\"name\":\"dataSetCreated\",\"inputs\":[{\"name\":\"dataSetId\",\"type\":\"uint256\",\"internalType\":\"uint256\"},{\"name\":\"creator\",\"type\":\"address\",\"internalType\":\"address\"},{\"name\":\"extraData\",\"type\":\"bytes\",\"internalType\":\"bytes\"}],\"outputs\":[],\"stateMutability\":\"nonpayable\"},{\"type\":\"function\",\"name\":\"dataSetDeleted\",\"inputs\":[{\"name\":\"dataSetId\",\"type\":\"uint256\",\"internalType\":\"uint256\"},{\"name\":\"\",\"type\":\"uint256\",\"internalType\":\"uint256\"},{\"name\":\"\",\"type\":\"bytes\",\"internalType\":\"bytes\"}],\"outputs\":[],\"stateMutability\":\"nonpayable\"},{\"type\":\"function\",\"name\":\"dealIdByDataSet\",\"inputs\":[{\"name\":\"\",\"type\":\"uint256\",\"internalType\":\"uint256\"}],\"outputs\":[{\"name\":\"\",\"type\":\"uint256\",\"internalType\":\"uint256\"}],\"stateMutability\":\"view\"},{\"type\":\"function\",\"name\":\"deals\",\"inputs\":[{\"name\":\"\",\"type\":\"uint256\",\"internalType\":\"uint256\"}],\"outputs\":[{\"name\":\"client\",\"type\":\"address\",\"internalType\":\"address\"},{\"name\":\"prover\",\"type\":\"address\",\"internalType\":\"address\"},{\"name\":\"commpHash\",\"type\":\"bytes32\",\"internalType\":\"bytes32\"},{\"name\":\"pieceSize\",\"type\":\"uint64\",\"internalType\":\"uint64\"},{\"name\":\"startedAt\",\"type\":\"uint64\",\"internalType\":\"uint64\"},{\"name\":\"endsAt\",\"type\":\"uint64\",\"internalType\":\"uint64\"},{\"name\":\"dataSetId\",\"type\":\"uint256\",\"internalType\":\"uint256\"},{\"name\":\"totalPayment\",\"type\":\"uint256\",\"internalType\":\"uint256\"},{\"name\":\"paidOut\",\"type\":\"uint256\",\"internalType\":\"uint256\"},{\"name\":\"lastProofAt\",\"type\":\"uint256\",\"internalType\":\"uint256\"},{\"name\":\"proofCount\",\"type\":\"uint256\",\"internalType\":\"uint256\"},{\"name\":\"status\",\"type\":\"uint8\",\"internalType\":\"enumStorageMarketplace.DealStatus\"}],\"stateMutability\":\"view\"},{\"type\":\"function\",\"name\":\"faultDeal\",\"inputs\":[{\"name\":\"dealId\",\"type\":\"uint256\",\"internalType\":\"uint256\"}],\"outputs\":[],\"stateMutability\":\"nonpayable\"},{\"type\":\"function\",\"name\":\"getDeal\",\"inputs\":[{\"name\":\"dealId\",\"type\":\"uint256\",\"internalType\":\"uint256\"}],\"outputs\":[{\"name\":\"\",\"type\":\"tuple\",\"internalType\":\"structStorageMarketplace.Deal\",\"components\":[{\"name\":\"client\",\"type\":\"address\",\"internalType\":\"address\"},{\"name\":\"prover\",\"type\":\"address\",\"internalType\":\"address\"},{\"name\":\"commpHash\",\"type\":\"bytes32\",\"internalType\":\"bytes32\"},{\"name\":\"pieceSize\",\"type\":\"uint64\",\"internalType\":\"uint64\"},{\"name\":\"startedAt\",\"type\":\"uint64\",\"internalType\":\"uint64\"},{\"name\":\"endsAt\",\"type\":\"uint64\",\"internalType\":\"uint64\"},{\"name\":\"dataSetId\",\"type\":\"uint256\",\"internalType\":\"uint256\"},{\"name\":\"totalPayment\",\"type\":\"uint256\",\"internalType\":\"uint256\"},{\"name\":\"paidOut\",\"type\":\"uint256\",\"internalType\":\"uint256\"},{\"name\":\"lastProofAt\",\"type\":\"uint256\",\"internalType\":\"uint256\"},{\"name\":\"proofCount\",\"type\":\"uint256\",\"internalType\":\"uint256\"},{\"name\":\"status\",\"type\":\"uint8\",\"internalType\":\"enumStorageMarketplace.DealStatus\"}]}],\"stateMutability\":\"view\"},{\"type\":\"function\",\"name\":\"isActive\",\"inputs\":[{\"name\":\"dealId\",\"type\":\"uint256\",\"internalType\":\"uint256\"}],\"outputs\":[{\"name\":\"\",\"type\":\"bool\",\"internalType\":\"bool\"}],\"stateMutability\":\"view\"},{\"type\":\"function\",\"name\":\"nextDealId\",\"inputs\":[],\"outputs\":[{\"name\":\"\",\"type\":\"uint256\",\"internalType\":\"uint256\"}],\"stateMutability\":\"view\"},{\"type\":\"function\",\"name\":\"nextProvingPeriod\",\"inputs\":[{\"name\":\"\",\"type\":\"uint256\",\"internalType\":\"uint256\"},{\"name\":\"\",\"type\":\"uint256\",\"internalType\":\"uint256\"},{\"name\":\"\",\"type\":\"uint256\",\"internalType\":\"uint256\"},{\"name\":\"\",\"type\":\"bytes\",\"internalType\":\"bytes\"}],\"outputs\":[],\"stateMutability\":\"nonpayable\"},{\"type\":\"function\",\"name\":\"owner\",\"inputs\":[],\"outputs\":[{\"name\":\"\",\"type\":\"address\",\"internalType\":\"address\"}],\"stateMutability\":\"view\"},{\"type\":\"function\",\"name\":\"paymentToken\",\"inputs\":[],\"outputs\":[{\"name\":\"\",\"type\":\"address\",\"internalType\":\"contractIERC20\"}],\"stateMutability\":\"view\"},{\"type\":\"function\",\"name\":\"pendingRelease\",\"inputs\":[{\"name\":\"dealId\",\"type\":\"uint256\",\"internalType\":\"uint256\"}],\"outputs\":[{\"name\":\"\",\"type\":\"uint256\",\"internalType\":\"uint256\"}],\"stateMutability\":\"view\"},{\"type\":\"function\",\"name\":\"piecesAdded\",\"inputs\":[{\"name\":\"\",\"type\":\"uint256\",\"internalType\":\"uint256\"},{\"name\":\"\",\"type\":\"uint256\",\"internalType\":\"uint256\"},{\"name\":\"\",\"type\":\"tuple[]\",\"internalType\":\"structCids.Cid[]\",\"components\":[{\"name\":\"data\",\"type\":\"bytes\",\"internalType\":\"bytes\"}]},{\"name\":\"\",\"type\":\"bytes\",\"internalType\":\"bytes\"}],\"outputs\":[],\"stateMutability\":\"nonpayable\"},{\"type\":\"function\",\"name\":\"piecesScheduledRemove\",\"inputs\":[{\"name\":\"\",\"type\":\"uint256\",\"internalType\":\"uint256\"},{\"name\":\"\",\"type\":\"uint256[]\",\"internalType\":\"uint256[]\"},{\"name\":\"\",\"type\":\"bytes\",\"internalType\":\"bytes\"}],\"outputs\":[],\"stateMutability\":\"nonpayable\"},{\"type\":\"function\",\"name\":\"possessionProven\",\"inputs\":[{\"name\":\"dataSetId\",\"type\":\"uint256\",\"internalType\":\"uint256\"},{\"name\":\"\",\"type\":\"uint256\",\"internalType\":\"uint256\"},{\"name\":\"\",\"type\":\"uint256\",\"internalType\":\"uint256\"},{\"name\":\"\",\"type\":\"uint256\",\"internalType\":\"uint256\"}],\"outputs\":[],\"stateMutability\":\"nonpayable\"},{\"type\":\"function\",\"name\":\"proofVerifier\",\"inputs\":[],\"outputs\":[{\"name\":\"\",\"type\":\"address\",\"internalType\":\"address\"}],\"stateMutability\":\"view\"},{\"type\":\"function\",\"name\":\"proposeDeal\",\"inputs\":[{\"name\":\"prover\",\"type\":\"address\",\"internalType\":\"address\"},{\"name\":\"commpHash\",\"type\":\"bytes32\",\"internalType\":\"bytes32\"},{\"name\":\"pieceSize\",\"type\":\"uint64\",\"internalType\":\"uint64\"},{\"name\":\"durationSeconds\",\"type\":\"uint64\",\"internalType\":\"uint64\"},{\"name\":\"totalPayment\",\"type\":\"uint256\",\"internalType\":\"uint256\"}],\"outputs\":[{\"name\":\"dealId\",\"type\":\"uint256\",\"internalType\":\"uint256\"}],\"stateMutability\":\"nonpayable\"},{\"type\":\"function\",\"name\":\"protocolFeeBps\",\"inputs\":[],\"outputs\":[{\"name\":\"\",\"type\":\"uint256\",\"internalType\":\"uint256\"}],\"stateMutability\":\"view\"},{\"type\":\"function\",\"name\":\"proverRegistry\",\"inputs\":[],\"outputs\":[{\"name\":\"\",\"type\":\"address\",\"internalType\":\"contractProverRegistry\"}],\"stateMutability\":\"view\"},{\"type\":\"function\",\"name\":\"proverStaking\",\"inputs\":[],\"outputs\":[{\"name\":\"\",\"type\":\"address\",\"internalType\":\"contractProverStaking\"}],\"stateMutability\":\"view\"},{\"type\":\"function\",\"name\":\"renounceOwnership\",\"inputs\":[],\"outputs\":[],\"stateMutability\":\"nonpayable\"},{\"type\":\"function\",\"name\":\"setProtocolFeeBps\",\"inputs\":[{\"name\":\"newBps\",\"type\":\"uint256\",\"internalType\":\"uint256\"}],\"outputs\":[],\"stateMutability\":\"nonpayable\"},{\"type\":\"function\",\"name\":\"setSlashPerFault\",\"inputs\":[{\"name\":\"newValue\",\"type\":\"uint256\",\"internalType\":\"uint256\"}],\"outputs\":[],\"stateMutability\":\"nonpayable\"},{\"type\":\"function\",\"name\":\"setTreasury\",\"inputs\":[{\"name\":\"newTreasury\",\"type\":\"address\",\"internalType\":\"address\"}],\"outputs\":[],\"stateMutability\":\"nonpayable\"},{\"type\":\"function\",\"name\":\"slashPerFault\",\"inputs\":[],\"outputs\":[{\"name\":\"\",\"type\":\"uint256\",\"internalType\":\"uint256\"}],\"stateMutability\":\"view\"},{\"type\":\"function\",\"name\":\"storageProviderChanged\",\"inputs\":[{\"name\":\"dataSetId\",\"type\":\"uint256\",\"internalType\":\"uint256\"},{\"name\":\"\",\"type\":\"address\",\"internalType\":\"address\"},{\"name\":\"newStorageProvider\",\"type\":\"address\",\"internalType\":\"address\"},{\"name\":\"\",\"type\":\"bytes\",\"internalType\":\"bytes\"}],\"outputs\":[],\"stateMutability\":\"nonpayable\"},{\"type\":\"function\",\"name\":\"transferOwnership\",\"inputs\":[{\"name\":\"newOwner\",\"type\":\"address\",\"internalType\":\"address\"}],\"outputs\":[],\"stateMutability\":\"nonpayable\"},{\"type\":\"function\",\"name\":\"treasury\",\"inputs\":[],\"outputs\":[{\"name\":\"\",\"type\":\"address\",\"internalType\":\"address\"}],\"stateMutability\":\"view\"},{\"type\":\"event\",\"name\":\"DealAccepted\",\"inputs\":[{\"name\":\"dealId\",\"type\":\"uint256\",\"indexed\":true,\"internalType\":\"uint256\"},{\"name\":\"prover\",\"type\":\"address\",\"indexed\":true,\"internalType\":\"address\"},{\"name\":\"dataSetId\",\"type\":\"uint256\",\"indexed\":false,\"internalType\":\"uint256\"},{\"name\":\"endsAt\",\"type\":\"uint64\",\"indexed\":false,\"internalType\":\"uint64\"}],\"anonymous\":false},{\"type\":\"event\",\"name\":\"DealCancelled\",\"inputs\":[{\"name\":\"dealId\",\"type\":\"uint256\",\"indexed\":true,\"internalType\":\"uint256\"},{\"name\":\"refund\",\"type\":\"uint256\",\"indexed\":false,\"internalType\":\"uint256\"}],\"anonymous\":false},{\"type\":\"event\",\"name\":\"DealCompleted\",\"inputs\":[{\"name\":\"dealId\",\"type\":\"uint256\",\"indexed\":true,\"internalType\":\"uint256\"},{\"name\":\"finalPaidOut\",\"type\":\"uint256\",\"indexed\":false,\"internalType\":\"uint256\"}],\"anonymous\":false},{\"type\":\"event\",\"name\":\"DealProposed\",\"inputs\":[{\"name\":\"dealId\",\"type\":\"uint256\",\"indexed\":true,\"internalType\":\"uint256\"},{\"name\":\"client\",\"type\":\"address\",\"indexed\":true,\"internalType\":\"address\"},{\"name\":\"prover\",\"type\":\"address\",\"indexed\":true,\"internalType\":\"address\"},{\"name\":\"commpHash\",\"type\":\"bytes32\",\"indexed\":false,\"internalType\":\"bytes32\"},{\"name\":\"pieceSize\",\"type\":\"uint64\",\"indexed\":false,\"internalType\":\"uint64\"},{\"name\":\"durationSeconds\",\"type\":\"uint64\",\"indexed\":false,\"internalType\":\"uint64\"},{\"name\":\"totalPayment\",\"type\":\"uint256\",\"indexed\":false,\"internalType\":\"uint256\"}],\"anonymous\":false},{\"type\":\"event\",\"name\":\"DealSlashed\",\"inputs\":[{\"name\":\"dealId\",\"type\":\"uint256\",\"indexed\":true,\"internalType\":\"uint256\"},{\"name\":\"prover\",\"type\":\"address\",\"indexed\":true,\"internalType\":\"address\"},{\"name\":\"slashedAmount\",\"type\":\"uint256\",\"indexed\":false,\"internalType\":\"uint256\"},{\"name\":\"refunded\",\"type\":\"uint256\",\"indexed\":false,\"internalType\":\"uint256\"}],\"anonymous\":false},{\"type\":\"event\",\"name\":\"OwnershipTransferred\",\"inputs\":[{\"name\":\"previousOwner\",\"type\":\"address\",\"indexed\":true,\"internalType\":\"address\"},{\"name\":\"newOwner\",\"type\":\"address\",\"indexed\":true,\"internalType\":\"address\"}],\"anonymous\":false},{\"type\":\"event\",\"name\":\"ProofRecorded\",\"inputs\":[{\"name\":\"dealId\",\"type\":\"uint256\",\"indexed\":true,\"internalType\":\"uint256\"},{\"name\":\"proofCount\",\"type\":\"uint256\",\"indexed\":false,\"internalType\":\"uint256\"},{\"name\":\"paymentReleased\",\"type\":\"uint256\",\"indexed\":false,\"internalType\":\"uint256\"}],\"anonymous\":false},{\"type\":\"event\",\"name\":\"ProtocolFeeChanged\",\"inputs\":[{\"name\":\"oldBps\",\"type\":\"uint256\",\"indexed\":false,\"internalType\":\"uint256\"},{\"name\":\"newBps\",\"type\":\"uint256\",\"indexed\":false,\"internalType\":\"uint256\"}],\"anonymous\":false},{\"type\":\"event\",\"name\":\"SlashPerFaultChanged\",\"inputs\":[{\"name\":\"oldValue\",\"type\":\"uint256\",\"indexed\":false,\"internalType\":\"uint256\"},{\"name\":\"newValue\",\"type\":\"uint256\",\"indexed\":false,\"internalType\":\"uint256\"}],\"anonymous\":false},{\"type\":\"event\",\"name\":\"TreasuryChanged\",\"inputs\":[{\"name\":\"oldTreasury\",\"type\":\"address\",\"indexed\":true,\"internalType\":\"address\"},{\"name\":\"newTreasury\",\"type\":\"address\",\"indexed\":true,\"internalType\":\"address\"}],\"anonymous\":false},{\"type\":\"error\",\"name\":\"DealNotActive\",\"inputs\":[]},{\"type\":\"error\",\"name\":\"DealNotProposed\",\"inputs\":[]},{\"type\":\"error\",\"name\":\"InvalidDuration\",\"inputs\":[]},{\"type\":\"error\",\"name\":\"InvalidPayment\",\"inputs\":[]},{\"type\":\"error\",\"name\":\"OnlyClient\",\"inputs\":[]},{\"type\":\"error\",\"name\":\"OnlyProofVerifier\",\"inputs\":[]},{\"type\":\"error\",\"name\":\"OnlyProver\",\"inputs\":[]},{\"type\":\"error\",\"name\":\"OwnableInvalidOwner\",\"inputs\":[{\"name\":\"owner\",\"type\":\"address\",\"internalType\":\"address\"}]},{\"type\":\"error\",\"name\":\"OwnableUnauthorizedAccount\",\"inputs\":[{\"name\":\"account\",\"type\":\"address\",\"internalType\":\"address\"}]},{\"type\":\"error\",\"name\":\"ProofGapTooSmall\",\"inputs\":[]},{\"type\":\"error\",\"name\":\"ProverCannotCommit\",\"inputs\":[]},{\"type\":\"error\",\"name\":\"ProverMismatch\",\"inputs\":[]},{\"type\":\"error\",\"name\":\"ProverNotActive\",\"inputs\":[]},{\"type\":\"error\",\"name\":\"ReentrancyGuardReentrantCall\",\"inputs\":[]},{\"type\":\"error\",\"name\":\"SafeERC20FailedOperation\",\"inputs\":[{\"name\":\"token\",\"type\":\"address\",\"internalType\":\"address\"}]},{\"type\":\"error\",\"name\":\"WrongDataSetOwner\",\"inputs\":[]}]",
	Bin: "0x6101203461023157601f6122f138819003918201601f19168301916001600160401b038311848410176102355780849260e0946040528339810103126102315761004881610249565b602082015190916001600160a01b03821682036102315760408101516001600160a01b03811681036102315760608201516001600160a01b0381168103610231576080830151916001600160a01b03831683036102315760c06100ad60a08601610249565b94015194331561021e575f8054336001600160a01b0319821681178355604051999290916001600160a01b0316907f8be0079c531659141344cd1fd0a4f28419497f9722a3daafe3b4186f6b6457e09080a360017f9b779b17422d0df92223018b32b4d1fa46e071723d6817e2486d003becc55f008190556064815560035560805260a05260c05260e05261010052600680546001600160a01b0319166001600160a01b0392909216919091179055600255612093908161025e82396080518181816111b2015281816113220152818161137e015281816115420152611c59015260a0518181816105dc0152818161089f015281816108dd01528181610c030152818161144001528181611b8d01528181611bd30152611e97015260c05181818161045701526109ff015260e05181818161070401528181610a5c015281816112670152818161162f0152611ce5015261010051818181610775015281816114d4015281816116c40152611da30152f35b631e4fbdf760e01b5f525f60045260245ffd5b5f80fd5b634e487b7160e01b5f52604160045260245ffd5b51906001600160a01b03821682036102315756fe6080806040526004361015610012575f80fd5b5f905f3560e01c908163015d0a87146118c65750806303988f84146117f2578063101c1eab1461150357806327a4f2bb146114be5780632abd465c1461148d5780632c1df2541461146f5780633013ce291461142a57806334b4f2c7146113d557806335659fb8146113b7578063356de02b1461136a5780634059b6d7146112db5780634d9879e3146112bd5780635872aa52146112965780635d9749771461125157806361d027b314611228578063715018a6146111e15780637fa417b31461119c57806382afd23b1461115d57806382fd5bac14610f875780638da5cb5b14610f60578063aa27ebcc14610f2f578063b01eefd414610e71578063c0417e5814610ddf578063d04aaa4d14610960578063dec3456b1461066b578063e1a452181461064e578063e658771814610540578063e7954aa7146104a4578063ecc87a7a14610486578063eda3150414610441578063f0f44260146103d7578063f2fde38b14610364578063f42e6e801461033a578063f602b50e1461031c5763f6814d791461019f575f80fd5b346102f55760803660031901126102f5576044356001600160401b038111610239573660238201121561023957806004013590602460206101df846119b1565b6101ec604051918261198e565b848152019260051b820101903682116103185760248101925b82841061023d57846064356001600160401b0381116102395761022c903690600401611931565b5050610236611c57565b80f35b5080fd5b83356001600160401b038111610314578201602060231982360301126103145760405190602082016001600160401b038111838210176102f85760405260248101356001600160401b0381116103105760249101019136601f8401121561030c578235916001600160401b0383116102f8576040516102c6601f8501601f19166020018261198e565b8381528836602086880101116102f5576020858197968280980183860137830101528152815201930192610205565b80fd5b634e487b7160e01b88526041600452602488fd5b8680fd5b8780fd5b8580fd5b8380fd5b50346102f557806003193601126102f5576020604051620151808152f35b50346102f55760203660031901126102f55760406020916004358152600583522054604051908152f35b50346102f55760203660031901126102f55761037e61191b565b610386611ec0565b6001600160a01b031680156103c35781546001600160a01b03198116821783556001600160a01b03165f51602061201e5f395f51905f528380a380f35b631e4fbdf760e01b82526004829052602482fd5b50346102f55760203660031901126102f5576103f161191b565b6103f9611ec0565b6006546001600160a01b0391821691829082167f8c3aa5f43a388513435861bf27dfad7829cd248696fed367c62d441f629544968580a36001600160a01b0319161760065580f35b50346102f557806003193601126102f5576040517f00000000000000000000000000000000000000000000000000000000000000006001600160a01b03168152602090f35b50346102f557806003193601126102f55760206040516203f4808152f35b50346102f55760603660031901126102f5576024356001600160401b0381116102395736602382011215610239578060040135602460206104e4836119b1565b6104f1604051918261198e565b838152019160051b8301019136831161031857602401905b82821061053057836044356001600160401b0381116102395761022c903690600401611931565b8135815260209182019101610509565b50346102f55760203660031901126102f55760043561055d611fe5565b8082526004602052604082206009810160ff815416600681101561063a5760010361062b5781546001600160a01b0316330361061c57805460ff191660041790558054600590910180547faf3855a84ba7ae9060a15c82675adab08caab3cb5ba10b102c3f0dd8279da0219260209291610600916001600160a01b03167f0000000000000000000000000000000000000000000000000000000000000000611f64565b54604051908152a260015f51602061203e5f395f51905f525580f35b63d6d90f8960e01b8452600484fd5b6307da9d3760e11b8452600484fd5b634e487b7160e01b85526021600452602485fd5b50346102f557806003193601126102f55760206040516127108152f35b50346102f55760203660031901126102f557600435610688611fe5565b8082526004602052604082206009810160ff815416600681101561063a576002036109515760038201805490929060801c6001600160401b0316421061091b578491600360ff1982541617905560058101549260068201936106eb855482611ab0565b8061085f575b5050600182015490546001600160a01b037f000000000000000000000000000000000000000000000000000000000000000081169216906001600160401b0316823b1561085b5761075b92859283604051809681958294633f99803160e11b845260048401611a3b565b03925af190811561085057839161083b575b5050600201547f00000000000000000000000000000000000000000000000000000000000000006001600160a01b031690813b1561082c5782916044839260405194859384926363c4099560e11b845260048401528960248401525af1801561083057610817575b505060207fd78ea1895a54fef557b21f240ae42cdf7bed32528d9d4f42f81458e17a1db4829154604051908152a260015f51602061203e5f395f51905f525580f35b816108219161198e565b61082c57825f6107d5565b8280fd5b6040513d84823e3d90fd5b816108459161198e565b61023957815f61076d565b6040513d85823e3d90fd5b8480fd5b6108c391865561087f61271061087760015484611a9d565b048092611ab0565b9080151580610907575b6108ca575b5060018401546001600160a01b03167f0000000000000000000000000000000000000000000000000000000000000000611f64565b5f806106f1565b60065461090191906001600160a01b03167f0000000000000000000000000000000000000000000000000000000000000000611f64565b5f61088e565b506006546001600160a01b03161515610889565b60405162461bcd60e51b815260206004820152600e60248201526d1919585b081b9bdd08195b99195960921b6044820152606490fd5b63299ca2e360e01b8452600484fd5b50346102f55760a03660031901126102f55761097a61191b565b604435916001600160401b0383169160243583850361082c57606435946001600160401b03861680870361085b57608435916109b4611fe5565b6201518082108015610dd2575b610dc3578215610db4578615610db457604051639f8a13d760e01b81526001600160a01b0380871660048301819052969190602090829060249082907f0000000000000000000000000000000000000000000000000000000000000000165afa908115610da9578891610d8a575b5015610d7b57604051638d543c9360e01b81529160209183918291610a58919060048401611a3b565b03817f00000000000000000000000000000000000000000000000000000000000000006001600160a01b03165afa908115610d70578691610d41575b5015610d3257600354955f198714610d1e576001870160035586604051610aba8161195e565b33815287602082018881526040830188815260608401868152608085019084825260a086019085825260c087019186835260e08801938c85526101008901958887526101208a01978989526101408b0199808b526101608c019c60018e5281526004602052604090209a600160a01b6001900390600160a01b6001900390511616600160a01b60019003198c5416178b55600160a01b6001900390511660018b0190600160a01b6001900316600160a01b60019003198254161790555160028a01556003890192600160401b60019003905116600160401b6001900319845416178355600160401b60019003905116610bb390836119c8565b51610bc9916001600160401b0390911690611a12565b51600486015551600585015551600684015551600783015551600882015560090190516006811015610d0a5760ff801983541691161790557f000000000000000000000000000000000000000000000000000000000000000093604051946323b872dd60e01b885233600452306024528460445260208860648180855af16001895114811615610ceb575b866040528860605215610cd0575093604093610cb7979360209a979360039784528b840152858301526060820152877fcb8ee79aef97f2ea6c1c5672fedae874e3a5f900e255a5458c9c7d22e1042a3460803393a4858152600487522001611a12565b60015f51602061203e5f395f51905f5255604051908152f35b635274afe760e01b88526001600160a01b0316600452602487fd5b6001811516610d0157813b15153d151616610c54565b863d8a823e3d90fd5b634e487b7160e01b88526021600452602488fd5b634e487b7160e01b86526011600452602486fd5b6371682a6f60e01b8552600485fd5b610d63915060203d602011610d69575b610d5b818361198e565b810190611c89565b5f610a94565b503d610d51565b6040513d88823e3d90fd5b630e5ea1c160e31b8752600487fd5b610da3915060203d602011610d6957610d5b818361198e565b5f610a2f565b6040513d8a823e3d90fd5b63078d696560e31b8652600486fd5b637616640160e01b8652600486fd5b506312cc030082116109c1565b50346102f55760203660031901126102f557600435610dfc611ec0565b6103e88111610e3d577fb51bef650ff5ad43303dbe2e500a74d4fd1bdc9ae05f046bece330e82ae0ba8760406001548151908152836020820152a160015580f35b60405162461bcd60e51b815260206004820152600c60248201526b0cccaca40e8dede40d0d2ced60a31b6044820152606490fd5b50346102f55760203660031901126102f557600435610e8e611fe5565b8082526004602052604082209060ff6009830154166006811015610f1b57600203610f0c5760078201546203f4808101809111610ef8574210610ee95790610ed591611ca1565b60015f51602061203e5f395f51905f525580f35b63d92f610160e01b8352600483fd5b634e487b7160e01b84526011600452602484fd5b63299ca2e360e01b8352600483fd5b634e487b7160e01b84526021600452602484fd5b50346102f55760803660031901126102f5576064356001600160401b0381116102395761022c903690600401611931565b50346102f557806003193601126102f557546040516001600160a01b039091168152602090f35b50346102f55760203660031901126102f55780610160604051610fa98161195e565b8281528260208201528260408201528260608201528260808201528260a08201528260c08201528260e08201528261010082015282610120820152826101408201520152600435815260046020526040812060405180916110098261195e565b60018060a01b03815416825260018060a01b03600182015416602083019081526002820154604084019081526003830154606085019060018060401b03811682526080860160018060401b038260401c16815260a087019160018060401b039060801c16825260048601549260c0880193845260058701549460e089019586526006880154966101008a0197885260ff600960078b01549a6101208d019b8c5261014060088201549d019c8d520154166101608c019c600682101561114957508c52604080519b516001600160a01b039081168d52915190911660208c01529051908a0152516001600160401b0390811660608a015290518116608089015290511660a08701525160c08601525160e085015251610100840152516101208301525161014082015290516101809190611147906101608301906118e2565bf35b634e487b7160e01b81526021600452602490fd5b50346102f55760203660031901126102f5576004358152600460205260ff60096040832001541690600682101561114957602082600260405191148152f35b50346102f557806003193601126102f5576040517f00000000000000000000000000000000000000000000000000000000000000006001600160a01b03168152602090f35b50346102f557806003193601126102f5576111fa611ec0565b80546001600160a01b03198116825581906001600160a01b03165f51602061201e5f395f51905f528280a380f35b50346102f557806003193601126102f5576006546040516001600160a01b039091168152602090f35b50346102f557806003193601126102f5576040517f00000000000000000000000000000000000000000000000000000000000000006001600160a01b03168152602090f35b50346102f55760203660031901126102f55760206112b5600435611c23565b604051908152f35b50346102f557806003193601126102f5576020600354604051908152f35b50346102f55760803660031901126102f5576112f56118ef565b506112fe611905565b506064356001600160401b0381116102395761131e903690600401611931565b50507f00000000000000000000000000000000000000000000000000000000000000006001600160a01b0316330361135b57610236600435611a5e565b63572a1d1b60e01b8152600490fd5b50346102f55760803660031901126102f5577f00000000000000000000000000000000000000000000000000000000000000006001600160a01b0316330361135b57610236600435611abd565b50346102f557806003193601126102f5576020600154604051908152f35b50346102f55760203660031901126102f5576004356113f2611ec0565b7f20761421f245b09f460443e59d7927ac61146d996e01b596dc2abc206450ac2b60406002548151908152836020820152a160025580f35b50346102f557806003193601126102f5576040517f00000000000000000000000000000000000000000000000000000000000000006001600160a01b03168152602090f35b50346102f557806003193601126102f5576020600254604051908152f35b50346102f55760603660031901126102f5576044356001600160401b0381116102395761131e903690600401611931565b50346102f557806003193601126102f5576040517f00000000000000000000000000000000000000000000000000000000000000006001600160a01b03168152602090f35b50346117ad5760603660031901126117ad576004356115206118ef565b906044356001600160401b0381116117ad57611540903690600401611931565b7f00000000000000000000000000000000000000000000000000000000000000006001600160a01b031633036117e35781602091810103126117ad573590815f52600460205260405f2092600984019360ff85541660068110156117cf576001036117c05760018101805490926001600160a01b039182169116036117b1576003810180549095906116019060801c6001600160401b03908116906115e7904216896119c8565b6001600160401b03906115fa90426119f1565b1687611a12565b60048201849055805460ff191660021790554260078201555f838152600560205260409020849055815485547f00000000000000000000000000000000000000000000000000000000000000006001600160a01b039081169216906001600160401b0316823b156117ad5761168f925f92836040518096819582946332d71bfd60e21b845260048401611a3b565b03925af180156117a25761178d575b5060028101549054855487926001600160401b03909116916001600160a01b03908116907f000000000000000000000000000000000000000000000000000000000000000016803b1561085b5784928360849260405196879586946392decf6f60e01b8652600486015260248501528b604485015260648401525af1801561083057611778575b50505492546040805192835260809190911c6001600160401b031660208301526001600160a01b0393909316927fdf671d8135dd67d13d0a978c3a6f6b0016a55c084d9e4ce79862332ecc0e5a9b91a380f35b816117829161198e565b61085b57845f611725565b61179a9196505f9061198e565b5f945f61169e565b6040513d5f823e3d90fd5b5f80fd5b63744a3ab160e01b5f5260045ffd5b6307da9d3760e11b5f5260045ffd5b634e487b7160e01b5f52602160045260245ffd5b63572a1d1b60e01b5f5260045ffd5b346117ad5760203660031901126117ad576004355f52600460205261018060405f2061114760018060a01b038254169160018060a01b036001820154169060028101546003820154600483015460058401549060068501549260078601549460ff6009600889015498015416976040519a8b5260208b015260408a015260018060401b03811660608a015260018060401b038160401c1660808a015260018060401b039060801c1660a089015260c088015260e08701526101008601526101208501526101408401526101608301906118e2565b346117ad575f3660031901126117ad57806312cc030060209252f35b9060068210156117cf5752565b602435906001600160a01b03821682036117ad57565b604435906001600160a01b03821682036117ad57565b600435906001600160a01b03821682036117ad57565b9181601f840112156117ad578235916001600160401b0383116117ad57602083818601950101116117ad57565b61018081019081106001600160401b0382111761197a57604052565b634e487b7160e01b5f52604160045260245ffd5b601f909101601f19168101906001600160401b0382119082101761197a57604052565b6001600160401b03811161197a5760051b60200190565b8054600160401b600160801b03191660409290921b600160401b600160801b0316919091179055565b919082018092116119fe57565b634e487b7160e01b5f52601160045260245ffd5b8054600160801b600160c01b03191660809290921b600160801b600160c01b0316919091179055565b6001600160a01b0390911681526001600160401b03909116602082015260400190565b5f52600560205260405f2054805f52600460205260405f2060ff60098201541660068110156117cf57600203611a9957611a9791611ca1565b565b5050565b818102929181159184041417156119fe57565b919082039182116119fe57565b5f52600560205260405f2054805f52600460205260405f2060ff60098201541660068110156117cf57600203611a995760088101908154600181018091116119fe5782604092827fcd3b3c1736fe7f5b05b3965d64e2a10662b20741cf825afc3e2ed2122369d1689555426007820155611b3681611ee6565b928315611c115750806006611bb19201611b518582546119f1565b9055612710611b6260015486611a9d565b0490611b6e8286611ab0565b9180151580611bfd575b611bc0575b50600101546001600160a01b03167f0000000000000000000000000000000000000000000000000000000000000000611f64565b549082519182526020820152a2565b600654611bf791906001600160a01b03167f0000000000000000000000000000000000000000000000000000000000000000611f64565b5f611b7d565b506006546001600160a01b03161515611b78565b9250505081519081525f6020820152a2565b5f52600460205260405f2060ff60098201541660068110156117cf57600203611c5257611c4f90611ee6565b90565b505f90565b7f00000000000000000000000000000000000000000000000000000000000000006001600160a01b031633036117e357565b908160209103126117ad575180151581036117ad5790565b60098201600560ff198254161790556005820154915f6006820193611cc7855482611ab0565b945583611e84575b6001820180546002549193916001600160a01b037f00000000000000000000000000000000000000000000000000000000000000008116921690823b156117ad5760405191637dd1602d60e11b8352600483015260248201528560448201525f8160648183865af180156117a257611e6f575b50835460038301546001600160401b031691906001600160a01b0316813b1561085b57918491611d899383604051809681958294633f99803160e11b845260048401611a3b565b03925af1801561085057908391611e5a575b5050600201547f00000000000000000000000000000000000000000000000000000000000000006001600160a01b031690813b1561082c5782916044839260405194859384926363c4099560e11b845260048401528960248401525af1801561083057611e45575b505060407f91ee6d1cfad4d94b4a8498c4fc55e34ed316908db60b97d781848357450bc5c29160018060a01b03905416936002549082519182526020820152a3565b611e5082809261198e565b6102f55780611e03565b81611e649161198e565b61023957815f611d9b565b611e7c9193505f9061198e565b5f915f611d42565b8154611ebb9085906001600160a01b03167f0000000000000000000000000000000000000000000000000000000000000000611f64565b611ccf565b5f546001600160a01b03163303611ed357565b63118cdaa760e01b5f523360045260245ffd5b60038101546001600160401b03604082901c81169160801c81168290039081116119fe576001600160401b03168015611f5d57611f2560069242611ab0565b818111611f56575b611f3b906005850154611a9d565b0491015480821115611f5057611c4f91611ab0565b50505f90565b5080611f2d565b5050505f90565b916040519163a9059cbb60e01b5f5260018060a01b031660045260245260205f60448180865af19060015f5114821615611fc4575b60405215611fa45750565b635274afe760e01b5f9081526001600160a01b0391909116600452602490fd5b906001811516611fdc57823b15153d15161690611f99565b503d5f823e3d90fd5b60025f51602061203e5f395f51905f52541461200e5760025f51602061203e5f395f51905f5255565b633ee5aeb560e01b5f5260045ffdfe8be0079c531659141344cd1fd0a4f28419497f9722a3daafe3b4186f6b6457e09b779b17422d0df92223018b32b4d1fa46e071723d6817e2486d003becc55f00a26469706673582212201a117bc1a55d7e3085840676f46b40109c2f95b06ced36faa85a54b5d592881864736f6c634300081e0033",
}

// StorageMarketplaceABI is the input ABI used to generate the binding from.
// Deprecated: Use StorageMarketplaceMetaData.ABI instead.
var StorageMarketplaceABI = StorageMarketplaceMetaData.ABI

// StorageMarketplaceBin is the compiled bytecode used for deploying new contracts.
// Deprecated: Use StorageMarketplaceMetaData.Bin instead.
var StorageMarketplaceBin = StorageMarketplaceMetaData.Bin

// DeployStorageMarketplace deploys a new Ethereum contract, binding an instance of StorageMarketplace to it.
func DeployStorageMarketplace(auth *bind.TransactOpts, backend bind.ContractBackend, _proofVerifier common.Address, _paymentToken common.Address, _proverRegistry common.Address, _proverStaking common.Address, _contentRegistry common.Address, _treasury common.Address, _slashPerFault *big.Int) (common.Address, *types.Transaction, *StorageMarketplace, error) {
	parsed, err := StorageMarketplaceMetaData.GetAbi()
	if err != nil {
		return common.Address{}, nil, nil, err
	}
	if parsed == nil {
		return common.Address{}, nil, nil, errors.New("GetABI returned nil")
	}

	address, tx, contract, err := bind.DeployContract(auth, *parsed, common.FromHex(StorageMarketplaceBin), backend, _proofVerifier, _paymentToken, _proverRegistry, _proverStaking, _contentRegistry, _treasury, _slashPerFault)
	if err != nil {
		return common.Address{}, nil, nil, err
	}
	return address, tx, &StorageMarketplace{StorageMarketplaceCaller: StorageMarketplaceCaller{contract: contract}, StorageMarketplaceTransactor: StorageMarketplaceTransactor{contract: contract}, StorageMarketplaceFilterer: StorageMarketplaceFilterer{contract: contract}}, nil
}

// StorageMarketplace is an auto generated Go binding around an Ethereum contract.
type StorageMarketplace struct {
	StorageMarketplaceCaller     // Read-only binding to the contract
	StorageMarketplaceTransactor // Write-only binding to the contract
	StorageMarketplaceFilterer   // Log filterer for contract events
}

// StorageMarketplaceCaller is an auto generated read-only Go binding around an Ethereum contract.
type StorageMarketplaceCaller struct {
	contract *bind.BoundContract // Generic contract wrapper for the low level calls
}

// StorageMarketplaceTransactor is an auto generated write-only Go binding around an Ethereum contract.
type StorageMarketplaceTransactor struct {
	contract *bind.BoundContract // Generic contract wrapper for the low level calls
}

// StorageMarketplaceFilterer is an auto generated log filtering Go binding around an Ethereum contract events.
type StorageMarketplaceFilterer struct {
	contract *bind.BoundContract // Generic contract wrapper for the low level calls
}

// StorageMarketplaceSession is an auto generated Go binding around an Ethereum contract,
// with pre-set call and transact options.
type StorageMarketplaceSession struct {
	Contract     *StorageMarketplace // Generic contract binding to set the session for
	CallOpts     bind.CallOpts       // Call options to use throughout this session
	TransactOpts bind.TransactOpts   // Transaction auth options to use throughout this session
}

// StorageMarketplaceCallerSession is an auto generated read-only Go binding around an Ethereum contract,
// with pre-set call options.
type StorageMarketplaceCallerSession struct {
	Contract *StorageMarketplaceCaller // Generic contract caller binding to set the session for
	CallOpts bind.CallOpts             // Call options to use throughout this session
}

// StorageMarketplaceTransactorSession is an auto generated write-only Go binding around an Ethereum contract,
// with pre-set transact options.
type StorageMarketplaceTransactorSession struct {
	Contract     *StorageMarketplaceTransactor // Generic contract transactor binding to set the session for
	TransactOpts bind.TransactOpts             // Transaction auth options to use throughout this session
}

// StorageMarketplaceRaw is an auto generated low-level Go binding around an Ethereum contract.
type StorageMarketplaceRaw struct {
	Contract *StorageMarketplace // Generic contract binding to access the raw methods on
}

// StorageMarketplaceCallerRaw is an auto generated low-level read-only Go binding around an Ethereum contract.
type StorageMarketplaceCallerRaw struct {
	Contract *StorageMarketplaceCaller // Generic read-only contract binding to access the raw methods on
}

// StorageMarketplaceTransactorRaw is an auto generated low-level write-only Go binding around an Ethereum contract.
type StorageMarketplaceTransactorRaw struct {
	Contract *StorageMarketplaceTransactor // Generic write-only contract binding to access the raw methods on
}

// NewStorageMarketplace creates a new instance of StorageMarketplace, bound to a specific deployed contract.
func NewStorageMarketplace(address common.Address, backend bind.ContractBackend) (*StorageMarketplace, error) {
	contract, err := bindStorageMarketplace(address, backend, backend, backend)
	if err != nil {
		return nil, err
	}
	return &StorageMarketplace{StorageMarketplaceCaller: StorageMarketplaceCaller{contract: contract}, StorageMarketplaceTransactor: StorageMarketplaceTransactor{contract: contract}, StorageMarketplaceFilterer: StorageMarketplaceFilterer{contract: contract}}, nil
}

// NewStorageMarketplaceCaller creates a new read-only instance of StorageMarketplace, bound to a specific deployed contract.
func NewStorageMarketplaceCaller(address common.Address, caller bind.ContractCaller) (*StorageMarketplaceCaller, error) {
	contract, err := bindStorageMarketplace(address, caller, nil, nil)
	if err != nil {
		return nil, err
	}
	return &StorageMarketplaceCaller{contract: contract}, nil
}

// NewStorageMarketplaceTransactor creates a new write-only instance of StorageMarketplace, bound to a specific deployed contract.
func NewStorageMarketplaceTransactor(address common.Address, transactor bind.ContractTransactor) (*StorageMarketplaceTransactor, error) {
	contract, err := bindStorageMarketplace(address, nil, transactor, nil)
	if err != nil {
		return nil, err
	}
	return &StorageMarketplaceTransactor{contract: contract}, nil
}

// NewStorageMarketplaceFilterer creates a new log filterer instance of StorageMarketplace, bound to a specific deployed contract.
func NewStorageMarketplaceFilterer(address common.Address, filterer bind.ContractFilterer) (*StorageMarketplaceFilterer, error) {
	contract, err := bindStorageMarketplace(address, nil, nil, filterer)
	if err != nil {
		return nil, err
	}
	return &StorageMarketplaceFilterer{contract: contract}, nil
}

// bindStorageMarketplace binds a generic wrapper to an already deployed contract.
func bindStorageMarketplace(address common.Address, caller bind.ContractCaller, transactor bind.ContractTransactor, filterer bind.ContractFilterer) (*bind.BoundContract, error) {
	parsed, err := StorageMarketplaceMetaData.GetAbi()
	if err != nil {
		return nil, err
	}
	return bind.NewBoundContract(address, *parsed, caller, transactor, filterer), nil
}

// Call invokes the (constant) contract method with params as input values and
// sets the output to result. The result type might be a single field for simple
// returns, a slice of interfaces for anonymous returns and a struct for named
// returns.
func (_StorageMarketplace *StorageMarketplaceRaw) Call(opts *bind.CallOpts, result *[]interface{}, method string, params ...interface{}) error {
	return _StorageMarketplace.Contract.StorageMarketplaceCaller.contract.Call(opts, result, method, params...)
}

// Transfer initiates a plain transaction to move funds to the contract, calling
// its default method if one is available.
func (_StorageMarketplace *StorageMarketplaceRaw) Transfer(opts *bind.TransactOpts) (*types.Transaction, error) {
	return _StorageMarketplace.Contract.StorageMarketplaceTransactor.contract.Transfer(opts)
}

// Transact invokes the (paid) contract method with params as input values.
func (_StorageMarketplace *StorageMarketplaceRaw) Transact(opts *bind.TransactOpts, method string, params ...interface{}) (*types.Transaction, error) {
	return _StorageMarketplace.Contract.StorageMarketplaceTransactor.contract.Transact(opts, method, params...)
}

// Call invokes the (constant) contract method with params as input values and
// sets the output to result. The result type might be a single field for simple
// returns, a slice of interfaces for anonymous returns and a struct for named
// returns.
func (_StorageMarketplace *StorageMarketplaceCallerRaw) Call(opts *bind.CallOpts, result *[]interface{}, method string, params ...interface{}) error {
	return _StorageMarketplace.Contract.contract.Call(opts, result, method, params...)
}

// Transfer initiates a plain transaction to move funds to the contract, calling
// its default method if one is available.
func (_StorageMarketplace *StorageMarketplaceTransactorRaw) Transfer(opts *bind.TransactOpts) (*types.Transaction, error) {
	return _StorageMarketplace.Contract.contract.Transfer(opts)
}

// Transact invokes the (paid) contract method with params as input values.
func (_StorageMarketplace *StorageMarketplaceTransactorRaw) Transact(opts *bind.TransactOpts, method string, params ...interface{}) (*types.Transaction, error) {
	return _StorageMarketplace.Contract.contract.Transact(opts, method, params...)
}

// BPSDENOMINATOR is a free data retrieval call binding the contract method 0xe1a45218.
//
// Solidity: function BPS_DENOMINATOR() view returns(uint256)
func (_StorageMarketplace *StorageMarketplaceCaller) BPSDENOMINATOR(opts *bind.CallOpts) (*big.Int, error) {
	var out []interface{}
	err := _StorageMarketplace.contract.Call(opts, &out, "BPS_DENOMINATOR")

	if err != nil {
		return *new(*big.Int), err
	}

	out0 := *abi.ConvertType(out[0], new(*big.Int)).(**big.Int)

	return out0, err

}

// BPSDENOMINATOR is a free data retrieval call binding the contract method 0xe1a45218.
//
// Solidity: function BPS_DENOMINATOR() view returns(uint256)
func (_StorageMarketplace *StorageMarketplaceSession) BPSDENOMINATOR() (*big.Int, error) {
	return _StorageMarketplace.Contract.BPSDENOMINATOR(&_StorageMarketplace.CallOpts)
}

// BPSDENOMINATOR is a free data retrieval call binding the contract method 0xe1a45218.
//
// Solidity: function BPS_DENOMINATOR() view returns(uint256)
func (_StorageMarketplace *StorageMarketplaceCallerSession) BPSDENOMINATOR() (*big.Int, error) {
	return _StorageMarketplace.Contract.BPSDENOMINATOR(&_StorageMarketplace.CallOpts)
}

// MAXDEALDURATION is a free data retrieval call binding the contract method 0x015d0a87.
//
// Solidity: function MAX_DEAL_DURATION() view returns(uint256)
func (_StorageMarketplace *StorageMarketplaceCaller) MAXDEALDURATION(opts *bind.CallOpts) (*big.Int, error) {
	var out []interface{}
	err := _StorageMarketplace.contract.Call(opts, &out, "MAX_DEAL_DURATION")

	if err != nil {
		return *new(*big.Int), err
	}

	out0 := *abi.ConvertType(out[0], new(*big.Int)).(**big.Int)

	return out0, err

}

// MAXDEALDURATION is a free data retrieval call binding the contract method 0x015d0a87.
//
// Solidity: function MAX_DEAL_DURATION() view returns(uint256)
func (_StorageMarketplace *StorageMarketplaceSession) MAXDEALDURATION() (*big.Int, error) {
	return _StorageMarketplace.Contract.MAXDEALDURATION(&_StorageMarketplace.CallOpts)
}

// MAXDEALDURATION is a free data retrieval call binding the contract method 0x015d0a87.
//
// Solidity: function MAX_DEAL_DURATION() view returns(uint256)
func (_StorageMarketplace *StorageMarketplaceCallerSession) MAXDEALDURATION() (*big.Int, error) {
	return _StorageMarketplace.Contract.MAXDEALDURATION(&_StorageMarketplace.CallOpts)
}

// MAXPROOFGAP is a free data retrieval call binding the contract method 0xecc87a7a.
//
// Solidity: function MAX_PROOF_GAP() view returns(uint256)
func (_StorageMarketplace *StorageMarketplaceCaller) MAXPROOFGAP(opts *bind.CallOpts) (*big.Int, error) {
	var out []interface{}
	err := _StorageMarketplace.contract.Call(opts, &out, "MAX_PROOF_GAP")

	if err != nil {
		return *new(*big.Int), err
	}

	out0 := *abi.ConvertType(out[0], new(*big.Int)).(**big.Int)

	return out0, err

}

// MAXPROOFGAP is a free data retrieval call binding the contract method 0xecc87a7a.
//
// Solidity: function MAX_PROOF_GAP() view returns(uint256)
func (_StorageMarketplace *StorageMarketplaceSession) MAXPROOFGAP() (*big.Int, error) {
	return _StorageMarketplace.Contract.MAXPROOFGAP(&_StorageMarketplace.CallOpts)
}

// MAXPROOFGAP is a free data retrieval call binding the contract method 0xecc87a7a.
//
// Solidity: function MAX_PROOF_GAP() view returns(uint256)
func (_StorageMarketplace *StorageMarketplaceCallerSession) MAXPROOFGAP() (*big.Int, error) {
	return _StorageMarketplace.Contract.MAXPROOFGAP(&_StorageMarketplace.CallOpts)
}

// MINDEALDURATION is a free data retrieval call binding the contract method 0xf602b50e.
//
// Solidity: function MIN_DEAL_DURATION() view returns(uint256)
func (_StorageMarketplace *StorageMarketplaceCaller) MINDEALDURATION(opts *bind.CallOpts) (*big.Int, error) {
	var out []interface{}
	err := _StorageMarketplace.contract.Call(opts, &out, "MIN_DEAL_DURATION")

	if err != nil {
		return *new(*big.Int), err
	}

	out0 := *abi.ConvertType(out[0], new(*big.Int)).(**big.Int)

	return out0, err

}

// MINDEALDURATION is a free data retrieval call binding the contract method 0xf602b50e.
//
// Solidity: function MIN_DEAL_DURATION() view returns(uint256)
func (_StorageMarketplace *StorageMarketplaceSession) MINDEALDURATION() (*big.Int, error) {
	return _StorageMarketplace.Contract.MINDEALDURATION(&_StorageMarketplace.CallOpts)
}

// MINDEALDURATION is a free data retrieval call binding the contract method 0xf602b50e.
//
// Solidity: function MIN_DEAL_DURATION() view returns(uint256)
func (_StorageMarketplace *StorageMarketplaceCallerSession) MINDEALDURATION() (*big.Int, error) {
	return _StorageMarketplace.Contract.MINDEALDURATION(&_StorageMarketplace.CallOpts)
}

// ContentRegistry is a free data retrieval call binding the contract method 0x27a4f2bb.
//
// Solidity: function contentRegistry() view returns(address)
func (_StorageMarketplace *StorageMarketplaceCaller) ContentRegistry(opts *bind.CallOpts) (common.Address, error) {
	var out []interface{}
	err := _StorageMarketplace.contract.Call(opts, &out, "contentRegistry")

	if err != nil {
		return *new(common.Address), err
	}

	out0 := *abi.ConvertType(out[0], new(common.Address)).(*common.Address)

	return out0, err

}

// ContentRegistry is a free data retrieval call binding the contract method 0x27a4f2bb.
//
// Solidity: function contentRegistry() view returns(address)
func (_StorageMarketplace *StorageMarketplaceSession) ContentRegistry() (common.Address, error) {
	return _StorageMarketplace.Contract.ContentRegistry(&_StorageMarketplace.CallOpts)
}

// ContentRegistry is a free data retrieval call binding the contract method 0x27a4f2bb.
//
// Solidity: function contentRegistry() view returns(address)
func (_StorageMarketplace *StorageMarketplaceCallerSession) ContentRegistry() (common.Address, error) {
	return _StorageMarketplace.Contract.ContentRegistry(&_StorageMarketplace.CallOpts)
}

// DealIdByDataSet is a free data retrieval call binding the contract method 0xf42e6e80.
//
// Solidity: function dealIdByDataSet(uint256 ) view returns(uint256)
func (_StorageMarketplace *StorageMarketplaceCaller) DealIdByDataSet(opts *bind.CallOpts, arg0 *big.Int) (*big.Int, error) {
	var out []interface{}
	err := _StorageMarketplace.contract.Call(opts, &out, "dealIdByDataSet", arg0)

	if err != nil {
		return *new(*big.Int), err
	}

	out0 := *abi.ConvertType(out[0], new(*big.Int)).(**big.Int)

	return out0, err

}

// DealIdByDataSet is a free data retrieval call binding the contract method 0xf42e6e80.
//
// Solidity: function dealIdByDataSet(uint256 ) view returns(uint256)
func (_StorageMarketplace *StorageMarketplaceSession) DealIdByDataSet(arg0 *big.Int) (*big.Int, error) {
	return _StorageMarketplace.Contract.DealIdByDataSet(&_StorageMarketplace.CallOpts, arg0)
}

// DealIdByDataSet is a free data retrieval call binding the contract method 0xf42e6e80.
//
// Solidity: function dealIdByDataSet(uint256 ) view returns(uint256)
func (_StorageMarketplace *StorageMarketplaceCallerSession) DealIdByDataSet(arg0 *big.Int) (*big.Int, error) {
	return _StorageMarketplace.Contract.DealIdByDataSet(&_StorageMarketplace.CallOpts, arg0)
}

// Deals is a free data retrieval call binding the contract method 0x03988f84.
//
// Solidity: function deals(uint256 ) view returns(address client, address prover, bytes32 commpHash, uint64 pieceSize, uint64 startedAt, uint64 endsAt, uint256 dataSetId, uint256 totalPayment, uint256 paidOut, uint256 lastProofAt, uint256 proofCount, uint8 status)
func (_StorageMarketplace *StorageMarketplaceCaller) Deals(opts *bind.CallOpts, arg0 *big.Int) (struct {
	Client       common.Address
	Prover       common.Address
	CommpHash    [32]byte
	PieceSize    uint64
	StartedAt    uint64
	EndsAt       uint64
	DataSetId    *big.Int
	TotalPayment *big.Int
	PaidOut      *big.Int
	LastProofAt  *big.Int
	ProofCount   *big.Int
	Status       uint8
}, error) {
	var out []interface{}
	err := _StorageMarketplace.contract.Call(opts, &out, "deals", arg0)

	outstruct := new(struct {
		Client       common.Address
		Prover       common.Address
		CommpHash    [32]byte
		PieceSize    uint64
		StartedAt    uint64
		EndsAt       uint64
		DataSetId    *big.Int
		TotalPayment *big.Int
		PaidOut      *big.Int
		LastProofAt  *big.Int
		ProofCount   *big.Int
		Status       uint8
	})
	if err != nil {
		return *outstruct, err
	}

	outstruct.Client = *abi.ConvertType(out[0], new(common.Address)).(*common.Address)
	outstruct.Prover = *abi.ConvertType(out[1], new(common.Address)).(*common.Address)
	outstruct.CommpHash = *abi.ConvertType(out[2], new([32]byte)).(*[32]byte)
	outstruct.PieceSize = *abi.ConvertType(out[3], new(uint64)).(*uint64)
	outstruct.StartedAt = *abi.ConvertType(out[4], new(uint64)).(*uint64)
	outstruct.EndsAt = *abi.ConvertType(out[5], new(uint64)).(*uint64)
	outstruct.DataSetId = *abi.ConvertType(out[6], new(*big.Int)).(**big.Int)
	outstruct.TotalPayment = *abi.ConvertType(out[7], new(*big.Int)).(**big.Int)
	outstruct.PaidOut = *abi.ConvertType(out[8], new(*big.Int)).(**big.Int)
	outstruct.LastProofAt = *abi.ConvertType(out[9], new(*big.Int)).(**big.Int)
	outstruct.ProofCount = *abi.ConvertType(out[10], new(*big.Int)).(**big.Int)
	outstruct.Status = *abi.ConvertType(out[11], new(uint8)).(*uint8)

	return *outstruct, err

}

// Deals is a free data retrieval call binding the contract method 0x03988f84.
//
// Solidity: function deals(uint256 ) view returns(address client, address prover, bytes32 commpHash, uint64 pieceSize, uint64 startedAt, uint64 endsAt, uint256 dataSetId, uint256 totalPayment, uint256 paidOut, uint256 lastProofAt, uint256 proofCount, uint8 status)
func (_StorageMarketplace *StorageMarketplaceSession) Deals(arg0 *big.Int) (struct {
	Client       common.Address
	Prover       common.Address
	CommpHash    [32]byte
	PieceSize    uint64
	StartedAt    uint64
	EndsAt       uint64
	DataSetId    *big.Int
	TotalPayment *big.Int
	PaidOut      *big.Int
	LastProofAt  *big.Int
	ProofCount   *big.Int
	Status       uint8
}, error) {
	return _StorageMarketplace.Contract.Deals(&_StorageMarketplace.CallOpts, arg0)
}

// Deals is a free data retrieval call binding the contract method 0x03988f84.
//
// Solidity: function deals(uint256 ) view returns(address client, address prover, bytes32 commpHash, uint64 pieceSize, uint64 startedAt, uint64 endsAt, uint256 dataSetId, uint256 totalPayment, uint256 paidOut, uint256 lastProofAt, uint256 proofCount, uint8 status)
func (_StorageMarketplace *StorageMarketplaceCallerSession) Deals(arg0 *big.Int) (struct {
	Client       common.Address
	Prover       common.Address
	CommpHash    [32]byte
	PieceSize    uint64
	StartedAt    uint64
	EndsAt       uint64
	DataSetId    *big.Int
	TotalPayment *big.Int
	PaidOut      *big.Int
	LastProofAt  *big.Int
	ProofCount   *big.Int
	Status       uint8
}, error) {
	return _StorageMarketplace.Contract.Deals(&_StorageMarketplace.CallOpts, arg0)
}

// GetDeal is a free data retrieval call binding the contract method 0x82fd5bac.
//
// Solidity: function getDeal(uint256 dealId) view returns((address,address,bytes32,uint64,uint64,uint64,uint256,uint256,uint256,uint256,uint256,uint8))
func (_StorageMarketplace *StorageMarketplaceCaller) GetDeal(opts *bind.CallOpts, dealId *big.Int) (StorageMarketplaceDeal, error) {
	var out []interface{}
	err := _StorageMarketplace.contract.Call(opts, &out, "getDeal", dealId)

	if err != nil {
		return *new(StorageMarketplaceDeal), err
	}

	out0 := *abi.ConvertType(out[0], new(StorageMarketplaceDeal)).(*StorageMarketplaceDeal)

	return out0, err

}

// GetDeal is a free data retrieval call binding the contract method 0x82fd5bac.
//
// Solidity: function getDeal(uint256 dealId) view returns((address,address,bytes32,uint64,uint64,uint64,uint256,uint256,uint256,uint256,uint256,uint8))
func (_StorageMarketplace *StorageMarketplaceSession) GetDeal(dealId *big.Int) (StorageMarketplaceDeal, error) {
	return _StorageMarketplace.Contract.GetDeal(&_StorageMarketplace.CallOpts, dealId)
}

// GetDeal is a free data retrieval call binding the contract method 0x82fd5bac.
//
// Solidity: function getDeal(uint256 dealId) view returns((address,address,bytes32,uint64,uint64,uint64,uint256,uint256,uint256,uint256,uint256,uint8))
func (_StorageMarketplace *StorageMarketplaceCallerSession) GetDeal(dealId *big.Int) (StorageMarketplaceDeal, error) {
	return _StorageMarketplace.Contract.GetDeal(&_StorageMarketplace.CallOpts, dealId)
}

// IsActive is a free data retrieval call binding the contract method 0x82afd23b.
//
// Solidity: function isActive(uint256 dealId) view returns(bool)
func (_StorageMarketplace *StorageMarketplaceCaller) IsActive(opts *bind.CallOpts, dealId *big.Int) (bool, error) {
	var out []interface{}
	err := _StorageMarketplace.contract.Call(opts, &out, "isActive", dealId)

	if err != nil {
		return *new(bool), err
	}

	out0 := *abi.ConvertType(out[0], new(bool)).(*bool)

	return out0, err

}

// IsActive is a free data retrieval call binding the contract method 0x82afd23b.
//
// Solidity: function isActive(uint256 dealId) view returns(bool)
func (_StorageMarketplace *StorageMarketplaceSession) IsActive(dealId *big.Int) (bool, error) {
	return _StorageMarketplace.Contract.IsActive(&_StorageMarketplace.CallOpts, dealId)
}

// IsActive is a free data retrieval call binding the contract method 0x82afd23b.
//
// Solidity: function isActive(uint256 dealId) view returns(bool)
func (_StorageMarketplace *StorageMarketplaceCallerSession) IsActive(dealId *big.Int) (bool, error) {
	return _StorageMarketplace.Contract.IsActive(&_StorageMarketplace.CallOpts, dealId)
}

// NextDealId is a free data retrieval call binding the contract method 0x4d9879e3.
//
// Solidity: function nextDealId() view returns(uint256)
func (_StorageMarketplace *StorageMarketplaceCaller) NextDealId(opts *bind.CallOpts) (*big.Int, error) {
	var out []interface{}
	err := _StorageMarketplace.contract.Call(opts, &out, "nextDealId")

	if err != nil {
		return *new(*big.Int), err
	}

	out0 := *abi.ConvertType(out[0], new(*big.Int)).(**big.Int)

	return out0, err

}

// NextDealId is a free data retrieval call binding the contract method 0x4d9879e3.
//
// Solidity: function nextDealId() view returns(uint256)
func (_StorageMarketplace *StorageMarketplaceSession) NextDealId() (*big.Int, error) {
	return _StorageMarketplace.Contract.NextDealId(&_StorageMarketplace.CallOpts)
}

// NextDealId is a free data retrieval call binding the contract method 0x4d9879e3.
//
// Solidity: function nextDealId() view returns(uint256)
func (_StorageMarketplace *StorageMarketplaceCallerSession) NextDealId() (*big.Int, error) {
	return _StorageMarketplace.Contract.NextDealId(&_StorageMarketplace.CallOpts)
}

// Owner is a free data retrieval call binding the contract method 0x8da5cb5b.
//
// Solidity: function owner() view returns(address)
func (_StorageMarketplace *StorageMarketplaceCaller) Owner(opts *bind.CallOpts) (common.Address, error) {
	var out []interface{}
	err := _StorageMarketplace.contract.Call(opts, &out, "owner")

	if err != nil {
		return *new(common.Address), err
	}

	out0 := *abi.ConvertType(out[0], new(common.Address)).(*common.Address)

	return out0, err

}

// Owner is a free data retrieval call binding the contract method 0x8da5cb5b.
//
// Solidity: function owner() view returns(address)
func (_StorageMarketplace *StorageMarketplaceSession) Owner() (common.Address, error) {
	return _StorageMarketplace.Contract.Owner(&_StorageMarketplace.CallOpts)
}

// Owner is a free data retrieval call binding the contract method 0x8da5cb5b.
//
// Solidity: function owner() view returns(address)
func (_StorageMarketplace *StorageMarketplaceCallerSession) Owner() (common.Address, error) {
	return _StorageMarketplace.Contract.Owner(&_StorageMarketplace.CallOpts)
}

// PaymentToken is a free data retrieval call binding the contract method 0x3013ce29.
//
// Solidity: function paymentToken() view returns(address)
func (_StorageMarketplace *StorageMarketplaceCaller) PaymentToken(opts *bind.CallOpts) (common.Address, error) {
	var out []interface{}
	err := _StorageMarketplace.contract.Call(opts, &out, "paymentToken")

	if err != nil {
		return *new(common.Address), err
	}

	out0 := *abi.ConvertType(out[0], new(common.Address)).(*common.Address)

	return out0, err

}

// PaymentToken is a free data retrieval call binding the contract method 0x3013ce29.
//
// Solidity: function paymentToken() view returns(address)
func (_StorageMarketplace *StorageMarketplaceSession) PaymentToken() (common.Address, error) {
	return _StorageMarketplace.Contract.PaymentToken(&_StorageMarketplace.CallOpts)
}

// PaymentToken is a free data retrieval call binding the contract method 0x3013ce29.
//
// Solidity: function paymentToken() view returns(address)
func (_StorageMarketplace *StorageMarketplaceCallerSession) PaymentToken() (common.Address, error) {
	return _StorageMarketplace.Contract.PaymentToken(&_StorageMarketplace.CallOpts)
}

// PendingRelease is a free data retrieval call binding the contract method 0x5872aa52.
//
// Solidity: function pendingRelease(uint256 dealId) view returns(uint256)
func (_StorageMarketplace *StorageMarketplaceCaller) PendingRelease(opts *bind.CallOpts, dealId *big.Int) (*big.Int, error) {
	var out []interface{}
	err := _StorageMarketplace.contract.Call(opts, &out, "pendingRelease", dealId)

	if err != nil {
		return *new(*big.Int), err
	}

	out0 := *abi.ConvertType(out[0], new(*big.Int)).(**big.Int)

	return out0, err

}

// PendingRelease is a free data retrieval call binding the contract method 0x5872aa52.
//
// Solidity: function pendingRelease(uint256 dealId) view returns(uint256)
func (_StorageMarketplace *StorageMarketplaceSession) PendingRelease(dealId *big.Int) (*big.Int, error) {
	return _StorageMarketplace.Contract.PendingRelease(&_StorageMarketplace.CallOpts, dealId)
}

// PendingRelease is a free data retrieval call binding the contract method 0x5872aa52.
//
// Solidity: function pendingRelease(uint256 dealId) view returns(uint256)
func (_StorageMarketplace *StorageMarketplaceCallerSession) PendingRelease(dealId *big.Int) (*big.Int, error) {
	return _StorageMarketplace.Contract.PendingRelease(&_StorageMarketplace.CallOpts, dealId)
}

// ProofVerifier is a free data retrieval call binding the contract method 0x7fa417b3.
//
// Solidity: function proofVerifier() view returns(address)
func (_StorageMarketplace *StorageMarketplaceCaller) ProofVerifier(opts *bind.CallOpts) (common.Address, error) {
	var out []interface{}
	err := _StorageMarketplace.contract.Call(opts, &out, "proofVerifier")

	if err != nil {
		return *new(common.Address), err
	}

	out0 := *abi.ConvertType(out[0], new(common.Address)).(*common.Address)

	return out0, err

}

// ProofVerifier is a free data retrieval call binding the contract method 0x7fa417b3.
//
// Solidity: function proofVerifier() view returns(address)
func (_StorageMarketplace *StorageMarketplaceSession) ProofVerifier() (common.Address, error) {
	return _StorageMarketplace.Contract.ProofVerifier(&_StorageMarketplace.CallOpts)
}

// ProofVerifier is a free data retrieval call binding the contract method 0x7fa417b3.
//
// Solidity: function proofVerifier() view returns(address)
func (_StorageMarketplace *StorageMarketplaceCallerSession) ProofVerifier() (common.Address, error) {
	return _StorageMarketplace.Contract.ProofVerifier(&_StorageMarketplace.CallOpts)
}

// ProtocolFeeBps is a free data retrieval call binding the contract method 0x35659fb8.
//
// Solidity: function protocolFeeBps() view returns(uint256)
func (_StorageMarketplace *StorageMarketplaceCaller) ProtocolFeeBps(opts *bind.CallOpts) (*big.Int, error) {
	var out []interface{}
	err := _StorageMarketplace.contract.Call(opts, &out, "protocolFeeBps")

	if err != nil {
		return *new(*big.Int), err
	}

	out0 := *abi.ConvertType(out[0], new(*big.Int)).(**big.Int)

	return out0, err

}

// ProtocolFeeBps is a free data retrieval call binding the contract method 0x35659fb8.
//
// Solidity: function protocolFeeBps() view returns(uint256)
func (_StorageMarketplace *StorageMarketplaceSession) ProtocolFeeBps() (*big.Int, error) {
	return _StorageMarketplace.Contract.ProtocolFeeBps(&_StorageMarketplace.CallOpts)
}

// ProtocolFeeBps is a free data retrieval call binding the contract method 0x35659fb8.
//
// Solidity: function protocolFeeBps() view returns(uint256)
func (_StorageMarketplace *StorageMarketplaceCallerSession) ProtocolFeeBps() (*big.Int, error) {
	return _StorageMarketplace.Contract.ProtocolFeeBps(&_StorageMarketplace.CallOpts)
}

// ProverRegistry is a free data retrieval call binding the contract method 0xeda31504.
//
// Solidity: function proverRegistry() view returns(address)
func (_StorageMarketplace *StorageMarketplaceCaller) ProverRegistry(opts *bind.CallOpts) (common.Address, error) {
	var out []interface{}
	err := _StorageMarketplace.contract.Call(opts, &out, "proverRegistry")

	if err != nil {
		return *new(common.Address), err
	}

	out0 := *abi.ConvertType(out[0], new(common.Address)).(*common.Address)

	return out0, err

}

// ProverRegistry is a free data retrieval call binding the contract method 0xeda31504.
//
// Solidity: function proverRegistry() view returns(address)
func (_StorageMarketplace *StorageMarketplaceSession) ProverRegistry() (common.Address, error) {
	return _StorageMarketplace.Contract.ProverRegistry(&_StorageMarketplace.CallOpts)
}

// ProverRegistry is a free data retrieval call binding the contract method 0xeda31504.
//
// Solidity: function proverRegistry() view returns(address)
func (_StorageMarketplace *StorageMarketplaceCallerSession) ProverRegistry() (common.Address, error) {
	return _StorageMarketplace.Contract.ProverRegistry(&_StorageMarketplace.CallOpts)
}

// ProverStaking is a free data retrieval call binding the contract method 0x5d974977.
//
// Solidity: function proverStaking() view returns(address)
func (_StorageMarketplace *StorageMarketplaceCaller) ProverStaking(opts *bind.CallOpts) (common.Address, error) {
	var out []interface{}
	err := _StorageMarketplace.contract.Call(opts, &out, "proverStaking")

	if err != nil {
		return *new(common.Address), err
	}

	out0 := *abi.ConvertType(out[0], new(common.Address)).(*common.Address)

	return out0, err

}

// ProverStaking is a free data retrieval call binding the contract method 0x5d974977.
//
// Solidity: function proverStaking() view returns(address)
func (_StorageMarketplace *StorageMarketplaceSession) ProverStaking() (common.Address, error) {
	return _StorageMarketplace.Contract.ProverStaking(&_StorageMarketplace.CallOpts)
}

// ProverStaking is a free data retrieval call binding the contract method 0x5d974977.
//
// Solidity: function proverStaking() view returns(address)
func (_StorageMarketplace *StorageMarketplaceCallerSession) ProverStaking() (common.Address, error) {
	return _StorageMarketplace.Contract.ProverStaking(&_StorageMarketplace.CallOpts)
}

// SlashPerFault is a free data retrieval call binding the contract method 0x2c1df254.
//
// Solidity: function slashPerFault() view returns(uint256)
func (_StorageMarketplace *StorageMarketplaceCaller) SlashPerFault(opts *bind.CallOpts) (*big.Int, error) {
	var out []interface{}
	err := _StorageMarketplace.contract.Call(opts, &out, "slashPerFault")

	if err != nil {
		return *new(*big.Int), err
	}

	out0 := *abi.ConvertType(out[0], new(*big.Int)).(**big.Int)

	return out0, err

}

// SlashPerFault is a free data retrieval call binding the contract method 0x2c1df254.
//
// Solidity: function slashPerFault() view returns(uint256)
func (_StorageMarketplace *StorageMarketplaceSession) SlashPerFault() (*big.Int, error) {
	return _StorageMarketplace.Contract.SlashPerFault(&_StorageMarketplace.CallOpts)
}

// SlashPerFault is a free data retrieval call binding the contract method 0x2c1df254.
//
// Solidity: function slashPerFault() view returns(uint256)
func (_StorageMarketplace *StorageMarketplaceCallerSession) SlashPerFault() (*big.Int, error) {
	return _StorageMarketplace.Contract.SlashPerFault(&_StorageMarketplace.CallOpts)
}

// Treasury is a free data retrieval call binding the contract method 0x61d027b3.
//
// Solidity: function treasury() view returns(address)
func (_StorageMarketplace *StorageMarketplaceCaller) Treasury(opts *bind.CallOpts) (common.Address, error) {
	var out []interface{}
	err := _StorageMarketplace.contract.Call(opts, &out, "treasury")

	if err != nil {
		return *new(common.Address), err
	}

	out0 := *abi.ConvertType(out[0], new(common.Address)).(*common.Address)

	return out0, err

}

// Treasury is a free data retrieval call binding the contract method 0x61d027b3.
//
// Solidity: function treasury() view returns(address)
func (_StorageMarketplace *StorageMarketplaceSession) Treasury() (common.Address, error) {
	return _StorageMarketplace.Contract.Treasury(&_StorageMarketplace.CallOpts)
}

// Treasury is a free data retrieval call binding the contract method 0x61d027b3.
//
// Solidity: function treasury() view returns(address)
func (_StorageMarketplace *StorageMarketplaceCallerSession) Treasury() (common.Address, error) {
	return _StorageMarketplace.Contract.Treasury(&_StorageMarketplace.CallOpts)
}

// CancelProposedDeal is a paid mutator transaction binding the contract method 0xe6587718.
//
// Solidity: function cancelProposedDeal(uint256 dealId) returns()
func (_StorageMarketplace *StorageMarketplaceTransactor) CancelProposedDeal(opts *bind.TransactOpts, dealId *big.Int) (*types.Transaction, error) {
	return _StorageMarketplace.contract.Transact(opts, "cancelProposedDeal", dealId)
}

// CancelProposedDeal is a paid mutator transaction binding the contract method 0xe6587718.
//
// Solidity: function cancelProposedDeal(uint256 dealId) returns()
func (_StorageMarketplace *StorageMarketplaceSession) CancelProposedDeal(dealId *big.Int) (*types.Transaction, error) {
	return _StorageMarketplace.Contract.CancelProposedDeal(&_StorageMarketplace.TransactOpts, dealId)
}

// CancelProposedDeal is a paid mutator transaction binding the contract method 0xe6587718.
//
// Solidity: function cancelProposedDeal(uint256 dealId) returns()
func (_StorageMarketplace *StorageMarketplaceTransactorSession) CancelProposedDeal(dealId *big.Int) (*types.Transaction, error) {
	return _StorageMarketplace.Contract.CancelProposedDeal(&_StorageMarketplace.TransactOpts, dealId)
}

// CompleteDeal is a paid mutator transaction binding the contract method 0xdec3456b.
//
// Solidity: function completeDeal(uint256 dealId) returns()
func (_StorageMarketplace *StorageMarketplaceTransactor) CompleteDeal(opts *bind.TransactOpts, dealId *big.Int) (*types.Transaction, error) {
	return _StorageMarketplace.contract.Transact(opts, "completeDeal", dealId)
}

// CompleteDeal is a paid mutator transaction binding the contract method 0xdec3456b.
//
// Solidity: function completeDeal(uint256 dealId) returns()
func (_StorageMarketplace *StorageMarketplaceSession) CompleteDeal(dealId *big.Int) (*types.Transaction, error) {
	return _StorageMarketplace.Contract.CompleteDeal(&_StorageMarketplace.TransactOpts, dealId)
}

// CompleteDeal is a paid mutator transaction binding the contract method 0xdec3456b.
//
// Solidity: function completeDeal(uint256 dealId) returns()
func (_StorageMarketplace *StorageMarketplaceTransactorSession) CompleteDeal(dealId *big.Int) (*types.Transaction, error) {
	return _StorageMarketplace.Contract.CompleteDeal(&_StorageMarketplace.TransactOpts, dealId)
}

// DataSetCreated is a paid mutator transaction binding the contract method 0x101c1eab.
//
// Solidity: function dataSetCreated(uint256 dataSetId, address creator, bytes extraData) returns()
func (_StorageMarketplace *StorageMarketplaceTransactor) DataSetCreated(opts *bind.TransactOpts, dataSetId *big.Int, creator common.Address, extraData []byte) (*types.Transaction, error) {
	return _StorageMarketplace.contract.Transact(opts, "dataSetCreated", dataSetId, creator, extraData)
}

// DataSetCreated is a paid mutator transaction binding the contract method 0x101c1eab.
//
// Solidity: function dataSetCreated(uint256 dataSetId, address creator, bytes extraData) returns()
func (_StorageMarketplace *StorageMarketplaceSession) DataSetCreated(dataSetId *big.Int, creator common.Address, extraData []byte) (*types.Transaction, error) {
	return _StorageMarketplace.Contract.DataSetCreated(&_StorageMarketplace.TransactOpts, dataSetId, creator, extraData)
}

// DataSetCreated is a paid mutator transaction binding the contract method 0x101c1eab.
//
// Solidity: function dataSetCreated(uint256 dataSetId, address creator, bytes extraData) returns()
func (_StorageMarketplace *StorageMarketplaceTransactorSession) DataSetCreated(dataSetId *big.Int, creator common.Address, extraData []byte) (*types.Transaction, error) {
	return _StorageMarketplace.Contract.DataSetCreated(&_StorageMarketplace.TransactOpts, dataSetId, creator, extraData)
}

// DataSetDeleted is a paid mutator transaction binding the contract method 0x2abd465c.
//
// Solidity: function dataSetDeleted(uint256 dataSetId, uint256 , bytes ) returns()
func (_StorageMarketplace *StorageMarketplaceTransactor) DataSetDeleted(opts *bind.TransactOpts, dataSetId *big.Int, arg1 *big.Int, arg2 []byte) (*types.Transaction, error) {
	return _StorageMarketplace.contract.Transact(opts, "dataSetDeleted", dataSetId, arg1, arg2)
}

// DataSetDeleted is a paid mutator transaction binding the contract method 0x2abd465c.
//
// Solidity: function dataSetDeleted(uint256 dataSetId, uint256 , bytes ) returns()
func (_StorageMarketplace *StorageMarketplaceSession) DataSetDeleted(dataSetId *big.Int, arg1 *big.Int, arg2 []byte) (*types.Transaction, error) {
	return _StorageMarketplace.Contract.DataSetDeleted(&_StorageMarketplace.TransactOpts, dataSetId, arg1, arg2)
}

// DataSetDeleted is a paid mutator transaction binding the contract method 0x2abd465c.
//
// Solidity: function dataSetDeleted(uint256 dataSetId, uint256 , bytes ) returns()
func (_StorageMarketplace *StorageMarketplaceTransactorSession) DataSetDeleted(dataSetId *big.Int, arg1 *big.Int, arg2 []byte) (*types.Transaction, error) {
	return _StorageMarketplace.Contract.DataSetDeleted(&_StorageMarketplace.TransactOpts, dataSetId, arg1, arg2)
}

// FaultDeal is a paid mutator transaction binding the contract method 0xb01eefd4.
//
// Solidity: function faultDeal(uint256 dealId) returns()
func (_StorageMarketplace *StorageMarketplaceTransactor) FaultDeal(opts *bind.TransactOpts, dealId *big.Int) (*types.Transaction, error) {
	return _StorageMarketplace.contract.Transact(opts, "faultDeal", dealId)
}

// FaultDeal is a paid mutator transaction binding the contract method 0xb01eefd4.
//
// Solidity: function faultDeal(uint256 dealId) returns()
func (_StorageMarketplace *StorageMarketplaceSession) FaultDeal(dealId *big.Int) (*types.Transaction, error) {
	return _StorageMarketplace.Contract.FaultDeal(&_StorageMarketplace.TransactOpts, dealId)
}

// FaultDeal is a paid mutator transaction binding the contract method 0xb01eefd4.
//
// Solidity: function faultDeal(uint256 dealId) returns()
func (_StorageMarketplace *StorageMarketplaceTransactorSession) FaultDeal(dealId *big.Int) (*types.Transaction, error) {
	return _StorageMarketplace.Contract.FaultDeal(&_StorageMarketplace.TransactOpts, dealId)
}

// NextProvingPeriod is a paid mutator transaction binding the contract method 0xaa27ebcc.
//
// Solidity: function nextProvingPeriod(uint256 , uint256 , uint256 , bytes ) returns()
func (_StorageMarketplace *StorageMarketplaceTransactor) NextProvingPeriod(opts *bind.TransactOpts, arg0 *big.Int, arg1 *big.Int, arg2 *big.Int, arg3 []byte) (*types.Transaction, error) {
	return _StorageMarketplace.contract.Transact(opts, "nextProvingPeriod", arg0, arg1, arg2, arg3)
}

// NextProvingPeriod is a paid mutator transaction binding the contract method 0xaa27ebcc.
//
// Solidity: function nextProvingPeriod(uint256 , uint256 , uint256 , bytes ) returns()
func (_StorageMarketplace *StorageMarketplaceSession) NextProvingPeriod(arg0 *big.Int, arg1 *big.Int, arg2 *big.Int, arg3 []byte) (*types.Transaction, error) {
	return _StorageMarketplace.Contract.NextProvingPeriod(&_StorageMarketplace.TransactOpts, arg0, arg1, arg2, arg3)
}

// NextProvingPeriod is a paid mutator transaction binding the contract method 0xaa27ebcc.
//
// Solidity: function nextProvingPeriod(uint256 , uint256 , uint256 , bytes ) returns()
func (_StorageMarketplace *StorageMarketplaceTransactorSession) NextProvingPeriod(arg0 *big.Int, arg1 *big.Int, arg2 *big.Int, arg3 []byte) (*types.Transaction, error) {
	return _StorageMarketplace.Contract.NextProvingPeriod(&_StorageMarketplace.TransactOpts, arg0, arg1, arg2, arg3)
}

// PiecesAdded is a paid mutator transaction binding the contract method 0xf6814d79.
//
// Solidity: function piecesAdded(uint256 , uint256 , (bytes)[] , bytes ) returns()
func (_StorageMarketplace *StorageMarketplaceTransactor) PiecesAdded(opts *bind.TransactOpts, arg0 *big.Int, arg1 *big.Int, arg2 []CidsCid, arg3 []byte) (*types.Transaction, error) {
	return _StorageMarketplace.contract.Transact(opts, "piecesAdded", arg0, arg1, arg2, arg3)
}

// PiecesAdded is a paid mutator transaction binding the contract method 0xf6814d79.
//
// Solidity: function piecesAdded(uint256 , uint256 , (bytes)[] , bytes ) returns()
func (_StorageMarketplace *StorageMarketplaceSession) PiecesAdded(arg0 *big.Int, arg1 *big.Int, arg2 []CidsCid, arg3 []byte) (*types.Transaction, error) {
	return _StorageMarketplace.Contract.PiecesAdded(&_StorageMarketplace.TransactOpts, arg0, arg1, arg2, arg3)
}

// PiecesAdded is a paid mutator transaction binding the contract method 0xf6814d79.
//
// Solidity: function piecesAdded(uint256 , uint256 , (bytes)[] , bytes ) returns()
func (_StorageMarketplace *StorageMarketplaceTransactorSession) PiecesAdded(arg0 *big.Int, arg1 *big.Int, arg2 []CidsCid, arg3 []byte) (*types.Transaction, error) {
	return _StorageMarketplace.Contract.PiecesAdded(&_StorageMarketplace.TransactOpts, arg0, arg1, arg2, arg3)
}

// PiecesScheduledRemove is a paid mutator transaction binding the contract method 0xe7954aa7.
//
// Solidity: function piecesScheduledRemove(uint256 , uint256[] , bytes ) returns()
func (_StorageMarketplace *StorageMarketplaceTransactor) PiecesScheduledRemove(opts *bind.TransactOpts, arg0 *big.Int, arg1 []*big.Int, arg2 []byte) (*types.Transaction, error) {
	return _StorageMarketplace.contract.Transact(opts, "piecesScheduledRemove", arg0, arg1, arg2)
}

// PiecesScheduledRemove is a paid mutator transaction binding the contract method 0xe7954aa7.
//
// Solidity: function piecesScheduledRemove(uint256 , uint256[] , bytes ) returns()
func (_StorageMarketplace *StorageMarketplaceSession) PiecesScheduledRemove(arg0 *big.Int, arg1 []*big.Int, arg2 []byte) (*types.Transaction, error) {
	return _StorageMarketplace.Contract.PiecesScheduledRemove(&_StorageMarketplace.TransactOpts, arg0, arg1, arg2)
}

// PiecesScheduledRemove is a paid mutator transaction binding the contract method 0xe7954aa7.
//
// Solidity: function piecesScheduledRemove(uint256 , uint256[] , bytes ) returns()
func (_StorageMarketplace *StorageMarketplaceTransactorSession) PiecesScheduledRemove(arg0 *big.Int, arg1 []*big.Int, arg2 []byte) (*types.Transaction, error) {
	return _StorageMarketplace.Contract.PiecesScheduledRemove(&_StorageMarketplace.TransactOpts, arg0, arg1, arg2)
}

// PossessionProven is a paid mutator transaction binding the contract method 0x356de02b.
//
// Solidity: function possessionProven(uint256 dataSetId, uint256 , uint256 , uint256 ) returns()
func (_StorageMarketplace *StorageMarketplaceTransactor) PossessionProven(opts *bind.TransactOpts, dataSetId *big.Int, arg1 *big.Int, arg2 *big.Int, arg3 *big.Int) (*types.Transaction, error) {
	return _StorageMarketplace.contract.Transact(opts, "possessionProven", dataSetId, arg1, arg2, arg3)
}

// PossessionProven is a paid mutator transaction binding the contract method 0x356de02b.
//
// Solidity: function possessionProven(uint256 dataSetId, uint256 , uint256 , uint256 ) returns()
func (_StorageMarketplace *StorageMarketplaceSession) PossessionProven(dataSetId *big.Int, arg1 *big.Int, arg2 *big.Int, arg3 *big.Int) (*types.Transaction, error) {
	return _StorageMarketplace.Contract.PossessionProven(&_StorageMarketplace.TransactOpts, dataSetId, arg1, arg2, arg3)
}

// PossessionProven is a paid mutator transaction binding the contract method 0x356de02b.
//
// Solidity: function possessionProven(uint256 dataSetId, uint256 , uint256 , uint256 ) returns()
func (_StorageMarketplace *StorageMarketplaceTransactorSession) PossessionProven(dataSetId *big.Int, arg1 *big.Int, arg2 *big.Int, arg3 *big.Int) (*types.Transaction, error) {
	return _StorageMarketplace.Contract.PossessionProven(&_StorageMarketplace.TransactOpts, dataSetId, arg1, arg2, arg3)
}

// ProposeDeal is a paid mutator transaction binding the contract method 0xd04aaa4d.
//
// Solidity: function proposeDeal(address prover, bytes32 commpHash, uint64 pieceSize, uint64 durationSeconds, uint256 totalPayment) returns(uint256 dealId)
func (_StorageMarketplace *StorageMarketplaceTransactor) ProposeDeal(opts *bind.TransactOpts, prover common.Address, commpHash [32]byte, pieceSize uint64, durationSeconds uint64, totalPayment *big.Int) (*types.Transaction, error) {
	return _StorageMarketplace.contract.Transact(opts, "proposeDeal", prover, commpHash, pieceSize, durationSeconds, totalPayment)
}

// ProposeDeal is a paid mutator transaction binding the contract method 0xd04aaa4d.
//
// Solidity: function proposeDeal(address prover, bytes32 commpHash, uint64 pieceSize, uint64 durationSeconds, uint256 totalPayment) returns(uint256 dealId)
func (_StorageMarketplace *StorageMarketplaceSession) ProposeDeal(prover common.Address, commpHash [32]byte, pieceSize uint64, durationSeconds uint64, totalPayment *big.Int) (*types.Transaction, error) {
	return _StorageMarketplace.Contract.ProposeDeal(&_StorageMarketplace.TransactOpts, prover, commpHash, pieceSize, durationSeconds, totalPayment)
}

// ProposeDeal is a paid mutator transaction binding the contract method 0xd04aaa4d.
//
// Solidity: function proposeDeal(address prover, bytes32 commpHash, uint64 pieceSize, uint64 durationSeconds, uint256 totalPayment) returns(uint256 dealId)
func (_StorageMarketplace *StorageMarketplaceTransactorSession) ProposeDeal(prover common.Address, commpHash [32]byte, pieceSize uint64, durationSeconds uint64, totalPayment *big.Int) (*types.Transaction, error) {
	return _StorageMarketplace.Contract.ProposeDeal(&_StorageMarketplace.TransactOpts, prover, commpHash, pieceSize, durationSeconds, totalPayment)
}

// RenounceOwnership is a paid mutator transaction binding the contract method 0x715018a6.
//
// Solidity: function renounceOwnership() returns()
func (_StorageMarketplace *StorageMarketplaceTransactor) RenounceOwnership(opts *bind.TransactOpts) (*types.Transaction, error) {
	return _StorageMarketplace.contract.Transact(opts, "renounceOwnership")
}

// RenounceOwnership is a paid mutator transaction binding the contract method 0x715018a6.
//
// Solidity: function renounceOwnership() returns()
func (_StorageMarketplace *StorageMarketplaceSession) RenounceOwnership() (*types.Transaction, error) {
	return _StorageMarketplace.Contract.RenounceOwnership(&_StorageMarketplace.TransactOpts)
}

// RenounceOwnership is a paid mutator transaction binding the contract method 0x715018a6.
//
// Solidity: function renounceOwnership() returns()
func (_StorageMarketplace *StorageMarketplaceTransactorSession) RenounceOwnership() (*types.Transaction, error) {
	return _StorageMarketplace.Contract.RenounceOwnership(&_StorageMarketplace.TransactOpts)
}

// SetProtocolFeeBps is a paid mutator transaction binding the contract method 0xc0417e58.
//
// Solidity: function setProtocolFeeBps(uint256 newBps) returns()
func (_StorageMarketplace *StorageMarketplaceTransactor) SetProtocolFeeBps(opts *bind.TransactOpts, newBps *big.Int) (*types.Transaction, error) {
	return _StorageMarketplace.contract.Transact(opts, "setProtocolFeeBps", newBps)
}

// SetProtocolFeeBps is a paid mutator transaction binding the contract method 0xc0417e58.
//
// Solidity: function setProtocolFeeBps(uint256 newBps) returns()
func (_StorageMarketplace *StorageMarketplaceSession) SetProtocolFeeBps(newBps *big.Int) (*types.Transaction, error) {
	return _StorageMarketplace.Contract.SetProtocolFeeBps(&_StorageMarketplace.TransactOpts, newBps)
}

// SetProtocolFeeBps is a paid mutator transaction binding the contract method 0xc0417e58.
//
// Solidity: function setProtocolFeeBps(uint256 newBps) returns()
func (_StorageMarketplace *StorageMarketplaceTransactorSession) SetProtocolFeeBps(newBps *big.Int) (*types.Transaction, error) {
	return _StorageMarketplace.Contract.SetProtocolFeeBps(&_StorageMarketplace.TransactOpts, newBps)
}

// SetSlashPerFault is a paid mutator transaction binding the contract method 0x34b4f2c7.
//
// Solidity: function setSlashPerFault(uint256 newValue) returns()
func (_StorageMarketplace *StorageMarketplaceTransactor) SetSlashPerFault(opts *bind.TransactOpts, newValue *big.Int) (*types.Transaction, error) {
	return _StorageMarketplace.contract.Transact(opts, "setSlashPerFault", newValue)
}

// SetSlashPerFault is a paid mutator transaction binding the contract method 0x34b4f2c7.
//
// Solidity: function setSlashPerFault(uint256 newValue) returns()
func (_StorageMarketplace *StorageMarketplaceSession) SetSlashPerFault(newValue *big.Int) (*types.Transaction, error) {
	return _StorageMarketplace.Contract.SetSlashPerFault(&_StorageMarketplace.TransactOpts, newValue)
}

// SetSlashPerFault is a paid mutator transaction binding the contract method 0x34b4f2c7.
//
// Solidity: function setSlashPerFault(uint256 newValue) returns()
func (_StorageMarketplace *StorageMarketplaceTransactorSession) SetSlashPerFault(newValue *big.Int) (*types.Transaction, error) {
	return _StorageMarketplace.Contract.SetSlashPerFault(&_StorageMarketplace.TransactOpts, newValue)
}

// SetTreasury is a paid mutator transaction binding the contract method 0xf0f44260.
//
// Solidity: function setTreasury(address newTreasury) returns()
func (_StorageMarketplace *StorageMarketplaceTransactor) SetTreasury(opts *bind.TransactOpts, newTreasury common.Address) (*types.Transaction, error) {
	return _StorageMarketplace.contract.Transact(opts, "setTreasury", newTreasury)
}

// SetTreasury is a paid mutator transaction binding the contract method 0xf0f44260.
//
// Solidity: function setTreasury(address newTreasury) returns()
func (_StorageMarketplace *StorageMarketplaceSession) SetTreasury(newTreasury common.Address) (*types.Transaction, error) {
	return _StorageMarketplace.Contract.SetTreasury(&_StorageMarketplace.TransactOpts, newTreasury)
}

// SetTreasury is a paid mutator transaction binding the contract method 0xf0f44260.
//
// Solidity: function setTreasury(address newTreasury) returns()
func (_StorageMarketplace *StorageMarketplaceTransactorSession) SetTreasury(newTreasury common.Address) (*types.Transaction, error) {
	return _StorageMarketplace.Contract.SetTreasury(&_StorageMarketplace.TransactOpts, newTreasury)
}

// StorageProviderChanged is a paid mutator transaction binding the contract method 0x4059b6d7.
//
// Solidity: function storageProviderChanged(uint256 dataSetId, address , address newStorageProvider, bytes ) returns()
func (_StorageMarketplace *StorageMarketplaceTransactor) StorageProviderChanged(opts *bind.TransactOpts, dataSetId *big.Int, arg1 common.Address, newStorageProvider common.Address, arg3 []byte) (*types.Transaction, error) {
	return _StorageMarketplace.contract.Transact(opts, "storageProviderChanged", dataSetId, arg1, newStorageProvider, arg3)
}

// StorageProviderChanged is a paid mutator transaction binding the contract method 0x4059b6d7.
//
// Solidity: function storageProviderChanged(uint256 dataSetId, address , address newStorageProvider, bytes ) returns()
func (_StorageMarketplace *StorageMarketplaceSession) StorageProviderChanged(dataSetId *big.Int, arg1 common.Address, newStorageProvider common.Address, arg3 []byte) (*types.Transaction, error) {
	return _StorageMarketplace.Contract.StorageProviderChanged(&_StorageMarketplace.TransactOpts, dataSetId, arg1, newStorageProvider, arg3)
}

// StorageProviderChanged is a paid mutator transaction binding the contract method 0x4059b6d7.
//
// Solidity: function storageProviderChanged(uint256 dataSetId, address , address newStorageProvider, bytes ) returns()
func (_StorageMarketplace *StorageMarketplaceTransactorSession) StorageProviderChanged(dataSetId *big.Int, arg1 common.Address, newStorageProvider common.Address, arg3 []byte) (*types.Transaction, error) {
	return _StorageMarketplace.Contract.StorageProviderChanged(&_StorageMarketplace.TransactOpts, dataSetId, arg1, newStorageProvider, arg3)
}

// TransferOwnership is a paid mutator transaction binding the contract method 0xf2fde38b.
//
// Solidity: function transferOwnership(address newOwner) returns()
func (_StorageMarketplace *StorageMarketplaceTransactor) TransferOwnership(opts *bind.TransactOpts, newOwner common.Address) (*types.Transaction, error) {
	return _StorageMarketplace.contract.Transact(opts, "transferOwnership", newOwner)
}

// TransferOwnership is a paid mutator transaction binding the contract method 0xf2fde38b.
//
// Solidity: function transferOwnership(address newOwner) returns()
func (_StorageMarketplace *StorageMarketplaceSession) TransferOwnership(newOwner common.Address) (*types.Transaction, error) {
	return _StorageMarketplace.Contract.TransferOwnership(&_StorageMarketplace.TransactOpts, newOwner)
}

// TransferOwnership is a paid mutator transaction binding the contract method 0xf2fde38b.
//
// Solidity: function transferOwnership(address newOwner) returns()
func (_StorageMarketplace *StorageMarketplaceTransactorSession) TransferOwnership(newOwner common.Address) (*types.Transaction, error) {
	return _StorageMarketplace.Contract.TransferOwnership(&_StorageMarketplace.TransactOpts, newOwner)
}

// StorageMarketplaceDealAcceptedIterator is returned from FilterDealAccepted and is used to iterate over the raw logs and unpacked data for DealAccepted events raised by the StorageMarketplace contract.
type StorageMarketplaceDealAcceptedIterator struct {
	Event *StorageMarketplaceDealAccepted // Event containing the contract specifics and raw log

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
func (it *StorageMarketplaceDealAcceptedIterator) Next() bool {
	// If the iterator failed, stop iterating
	if it.fail != nil {
		return false
	}
	// If the iterator completed, deliver directly whatever's available
	if it.done {
		select {
		case log := <-it.logs:
			it.Event = new(StorageMarketplaceDealAccepted)
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
		it.Event = new(StorageMarketplaceDealAccepted)
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
func (it *StorageMarketplaceDealAcceptedIterator) Error() error {
	return it.fail
}

// Close terminates the iteration process, releasing any pending underlying
// resources.
func (it *StorageMarketplaceDealAcceptedIterator) Close() error {
	it.sub.Unsubscribe()
	return nil
}

// StorageMarketplaceDealAccepted represents a DealAccepted event raised by the StorageMarketplace contract.
type StorageMarketplaceDealAccepted struct {
	DealId    *big.Int
	Prover    common.Address
	DataSetId *big.Int
	EndsAt    uint64
	Raw       types.Log // Blockchain specific contextual infos
}

// FilterDealAccepted is a free log retrieval operation binding the contract event 0xdf671d8135dd67d13d0a978c3a6f6b0016a55c084d9e4ce79862332ecc0e5a9b.
//
// Solidity: event DealAccepted(uint256 indexed dealId, address indexed prover, uint256 dataSetId, uint64 endsAt)
func (_StorageMarketplace *StorageMarketplaceFilterer) FilterDealAccepted(opts *bind.FilterOpts, dealId []*big.Int, prover []common.Address) (*StorageMarketplaceDealAcceptedIterator, error) {

	var dealIdRule []interface{}
	for _, dealIdItem := range dealId {
		dealIdRule = append(dealIdRule, dealIdItem)
	}
	var proverRule []interface{}
	for _, proverItem := range prover {
		proverRule = append(proverRule, proverItem)
	}

	logs, sub, err := _StorageMarketplace.contract.FilterLogs(opts, "DealAccepted", dealIdRule, proverRule)
	if err != nil {
		return nil, err
	}
	return &StorageMarketplaceDealAcceptedIterator{contract: _StorageMarketplace.contract, event: "DealAccepted", logs: logs, sub: sub}, nil
}

// WatchDealAccepted is a free log subscription operation binding the contract event 0xdf671d8135dd67d13d0a978c3a6f6b0016a55c084d9e4ce79862332ecc0e5a9b.
//
// Solidity: event DealAccepted(uint256 indexed dealId, address indexed prover, uint256 dataSetId, uint64 endsAt)
func (_StorageMarketplace *StorageMarketplaceFilterer) WatchDealAccepted(opts *bind.WatchOpts, sink chan<- *StorageMarketplaceDealAccepted, dealId []*big.Int, prover []common.Address) (event.Subscription, error) {

	var dealIdRule []interface{}
	for _, dealIdItem := range dealId {
		dealIdRule = append(dealIdRule, dealIdItem)
	}
	var proverRule []interface{}
	for _, proverItem := range prover {
		proverRule = append(proverRule, proverItem)
	}

	logs, sub, err := _StorageMarketplace.contract.WatchLogs(opts, "DealAccepted", dealIdRule, proverRule)
	if err != nil {
		return nil, err
	}
	return event.NewSubscription(func(quit <-chan struct{}) error {
		defer sub.Unsubscribe()
		for {
			select {
			case log := <-logs:
				// New log arrived, parse the event and forward to the user
				event := new(StorageMarketplaceDealAccepted)
				if err := _StorageMarketplace.contract.UnpackLog(event, "DealAccepted", log); err != nil {
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

// ParseDealAccepted is a log parse operation binding the contract event 0xdf671d8135dd67d13d0a978c3a6f6b0016a55c084d9e4ce79862332ecc0e5a9b.
//
// Solidity: event DealAccepted(uint256 indexed dealId, address indexed prover, uint256 dataSetId, uint64 endsAt)
func (_StorageMarketplace *StorageMarketplaceFilterer) ParseDealAccepted(log types.Log) (*StorageMarketplaceDealAccepted, error) {
	event := new(StorageMarketplaceDealAccepted)
	if err := _StorageMarketplace.contract.UnpackLog(event, "DealAccepted", log); err != nil {
		return nil, err
	}
	event.Raw = log
	return event, nil
}

// StorageMarketplaceDealCancelledIterator is returned from FilterDealCancelled and is used to iterate over the raw logs and unpacked data for DealCancelled events raised by the StorageMarketplace contract.
type StorageMarketplaceDealCancelledIterator struct {
	Event *StorageMarketplaceDealCancelled // Event containing the contract specifics and raw log

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
func (it *StorageMarketplaceDealCancelledIterator) Next() bool {
	// If the iterator failed, stop iterating
	if it.fail != nil {
		return false
	}
	// If the iterator completed, deliver directly whatever's available
	if it.done {
		select {
		case log := <-it.logs:
			it.Event = new(StorageMarketplaceDealCancelled)
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
		it.Event = new(StorageMarketplaceDealCancelled)
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
func (it *StorageMarketplaceDealCancelledIterator) Error() error {
	return it.fail
}

// Close terminates the iteration process, releasing any pending underlying
// resources.
func (it *StorageMarketplaceDealCancelledIterator) Close() error {
	it.sub.Unsubscribe()
	return nil
}

// StorageMarketplaceDealCancelled represents a DealCancelled event raised by the StorageMarketplace contract.
type StorageMarketplaceDealCancelled struct {
	DealId *big.Int
	Refund *big.Int
	Raw    types.Log // Blockchain specific contextual infos
}

// FilterDealCancelled is a free log retrieval operation binding the contract event 0xaf3855a84ba7ae9060a15c82675adab08caab3cb5ba10b102c3f0dd8279da021.
//
// Solidity: event DealCancelled(uint256 indexed dealId, uint256 refund)
func (_StorageMarketplace *StorageMarketplaceFilterer) FilterDealCancelled(opts *bind.FilterOpts, dealId []*big.Int) (*StorageMarketplaceDealCancelledIterator, error) {

	var dealIdRule []interface{}
	for _, dealIdItem := range dealId {
		dealIdRule = append(dealIdRule, dealIdItem)
	}

	logs, sub, err := _StorageMarketplace.contract.FilterLogs(opts, "DealCancelled", dealIdRule)
	if err != nil {
		return nil, err
	}
	return &StorageMarketplaceDealCancelledIterator{contract: _StorageMarketplace.contract, event: "DealCancelled", logs: logs, sub: sub}, nil
}

// WatchDealCancelled is a free log subscription operation binding the contract event 0xaf3855a84ba7ae9060a15c82675adab08caab3cb5ba10b102c3f0dd8279da021.
//
// Solidity: event DealCancelled(uint256 indexed dealId, uint256 refund)
func (_StorageMarketplace *StorageMarketplaceFilterer) WatchDealCancelled(opts *bind.WatchOpts, sink chan<- *StorageMarketplaceDealCancelled, dealId []*big.Int) (event.Subscription, error) {

	var dealIdRule []interface{}
	for _, dealIdItem := range dealId {
		dealIdRule = append(dealIdRule, dealIdItem)
	}

	logs, sub, err := _StorageMarketplace.contract.WatchLogs(opts, "DealCancelled", dealIdRule)
	if err != nil {
		return nil, err
	}
	return event.NewSubscription(func(quit <-chan struct{}) error {
		defer sub.Unsubscribe()
		for {
			select {
			case log := <-logs:
				// New log arrived, parse the event and forward to the user
				event := new(StorageMarketplaceDealCancelled)
				if err := _StorageMarketplace.contract.UnpackLog(event, "DealCancelled", log); err != nil {
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

// ParseDealCancelled is a log parse operation binding the contract event 0xaf3855a84ba7ae9060a15c82675adab08caab3cb5ba10b102c3f0dd8279da021.
//
// Solidity: event DealCancelled(uint256 indexed dealId, uint256 refund)
func (_StorageMarketplace *StorageMarketplaceFilterer) ParseDealCancelled(log types.Log) (*StorageMarketplaceDealCancelled, error) {
	event := new(StorageMarketplaceDealCancelled)
	if err := _StorageMarketplace.contract.UnpackLog(event, "DealCancelled", log); err != nil {
		return nil, err
	}
	event.Raw = log
	return event, nil
}

// StorageMarketplaceDealCompletedIterator is returned from FilterDealCompleted and is used to iterate over the raw logs and unpacked data for DealCompleted events raised by the StorageMarketplace contract.
type StorageMarketplaceDealCompletedIterator struct {
	Event *StorageMarketplaceDealCompleted // Event containing the contract specifics and raw log

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
func (it *StorageMarketplaceDealCompletedIterator) Next() bool {
	// If the iterator failed, stop iterating
	if it.fail != nil {
		return false
	}
	// If the iterator completed, deliver directly whatever's available
	if it.done {
		select {
		case log := <-it.logs:
			it.Event = new(StorageMarketplaceDealCompleted)
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
		it.Event = new(StorageMarketplaceDealCompleted)
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
func (it *StorageMarketplaceDealCompletedIterator) Error() error {
	return it.fail
}

// Close terminates the iteration process, releasing any pending underlying
// resources.
func (it *StorageMarketplaceDealCompletedIterator) Close() error {
	it.sub.Unsubscribe()
	return nil
}

// StorageMarketplaceDealCompleted represents a DealCompleted event raised by the StorageMarketplace contract.
type StorageMarketplaceDealCompleted struct {
	DealId       *big.Int
	FinalPaidOut *big.Int
	Raw          types.Log // Blockchain specific contextual infos
}

// FilterDealCompleted is a free log retrieval operation binding the contract event 0xd78ea1895a54fef557b21f240ae42cdf7bed32528d9d4f42f81458e17a1db482.
//
// Solidity: event DealCompleted(uint256 indexed dealId, uint256 finalPaidOut)
func (_StorageMarketplace *StorageMarketplaceFilterer) FilterDealCompleted(opts *bind.FilterOpts, dealId []*big.Int) (*StorageMarketplaceDealCompletedIterator, error) {

	var dealIdRule []interface{}
	for _, dealIdItem := range dealId {
		dealIdRule = append(dealIdRule, dealIdItem)
	}

	logs, sub, err := _StorageMarketplace.contract.FilterLogs(opts, "DealCompleted", dealIdRule)
	if err != nil {
		return nil, err
	}
	return &StorageMarketplaceDealCompletedIterator{contract: _StorageMarketplace.contract, event: "DealCompleted", logs: logs, sub: sub}, nil
}

// WatchDealCompleted is a free log subscription operation binding the contract event 0xd78ea1895a54fef557b21f240ae42cdf7bed32528d9d4f42f81458e17a1db482.
//
// Solidity: event DealCompleted(uint256 indexed dealId, uint256 finalPaidOut)
func (_StorageMarketplace *StorageMarketplaceFilterer) WatchDealCompleted(opts *bind.WatchOpts, sink chan<- *StorageMarketplaceDealCompleted, dealId []*big.Int) (event.Subscription, error) {

	var dealIdRule []interface{}
	for _, dealIdItem := range dealId {
		dealIdRule = append(dealIdRule, dealIdItem)
	}

	logs, sub, err := _StorageMarketplace.contract.WatchLogs(opts, "DealCompleted", dealIdRule)
	if err != nil {
		return nil, err
	}
	return event.NewSubscription(func(quit <-chan struct{}) error {
		defer sub.Unsubscribe()
		for {
			select {
			case log := <-logs:
				// New log arrived, parse the event and forward to the user
				event := new(StorageMarketplaceDealCompleted)
				if err := _StorageMarketplace.contract.UnpackLog(event, "DealCompleted", log); err != nil {
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

// ParseDealCompleted is a log parse operation binding the contract event 0xd78ea1895a54fef557b21f240ae42cdf7bed32528d9d4f42f81458e17a1db482.
//
// Solidity: event DealCompleted(uint256 indexed dealId, uint256 finalPaidOut)
func (_StorageMarketplace *StorageMarketplaceFilterer) ParseDealCompleted(log types.Log) (*StorageMarketplaceDealCompleted, error) {
	event := new(StorageMarketplaceDealCompleted)
	if err := _StorageMarketplace.contract.UnpackLog(event, "DealCompleted", log); err != nil {
		return nil, err
	}
	event.Raw = log
	return event, nil
}

// StorageMarketplaceDealProposedIterator is returned from FilterDealProposed and is used to iterate over the raw logs and unpacked data for DealProposed events raised by the StorageMarketplace contract.
type StorageMarketplaceDealProposedIterator struct {
	Event *StorageMarketplaceDealProposed // Event containing the contract specifics and raw log

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
func (it *StorageMarketplaceDealProposedIterator) Next() bool {
	// If the iterator failed, stop iterating
	if it.fail != nil {
		return false
	}
	// If the iterator completed, deliver directly whatever's available
	if it.done {
		select {
		case log := <-it.logs:
			it.Event = new(StorageMarketplaceDealProposed)
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
		it.Event = new(StorageMarketplaceDealProposed)
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
func (it *StorageMarketplaceDealProposedIterator) Error() error {
	return it.fail
}

// Close terminates the iteration process, releasing any pending underlying
// resources.
func (it *StorageMarketplaceDealProposedIterator) Close() error {
	it.sub.Unsubscribe()
	return nil
}

// StorageMarketplaceDealProposed represents a DealProposed event raised by the StorageMarketplace contract.
type StorageMarketplaceDealProposed struct {
	DealId          *big.Int
	Client          common.Address
	Prover          common.Address
	CommpHash       [32]byte
	PieceSize       uint64
	DurationSeconds uint64
	TotalPayment    *big.Int
	Raw             types.Log // Blockchain specific contextual infos
}

// FilterDealProposed is a free log retrieval operation binding the contract event 0xcb8ee79aef97f2ea6c1c5672fedae874e3a5f900e255a5458c9c7d22e1042a34.
//
// Solidity: event DealProposed(uint256 indexed dealId, address indexed client, address indexed prover, bytes32 commpHash, uint64 pieceSize, uint64 durationSeconds, uint256 totalPayment)
func (_StorageMarketplace *StorageMarketplaceFilterer) FilterDealProposed(opts *bind.FilterOpts, dealId []*big.Int, client []common.Address, prover []common.Address) (*StorageMarketplaceDealProposedIterator, error) {

	var dealIdRule []interface{}
	for _, dealIdItem := range dealId {
		dealIdRule = append(dealIdRule, dealIdItem)
	}
	var clientRule []interface{}
	for _, clientItem := range client {
		clientRule = append(clientRule, clientItem)
	}
	var proverRule []interface{}
	for _, proverItem := range prover {
		proverRule = append(proverRule, proverItem)
	}

	logs, sub, err := _StorageMarketplace.contract.FilterLogs(opts, "DealProposed", dealIdRule, clientRule, proverRule)
	if err != nil {
		return nil, err
	}
	return &StorageMarketplaceDealProposedIterator{contract: _StorageMarketplace.contract, event: "DealProposed", logs: logs, sub: sub}, nil
}

// WatchDealProposed is a free log subscription operation binding the contract event 0xcb8ee79aef97f2ea6c1c5672fedae874e3a5f900e255a5458c9c7d22e1042a34.
//
// Solidity: event DealProposed(uint256 indexed dealId, address indexed client, address indexed prover, bytes32 commpHash, uint64 pieceSize, uint64 durationSeconds, uint256 totalPayment)
func (_StorageMarketplace *StorageMarketplaceFilterer) WatchDealProposed(opts *bind.WatchOpts, sink chan<- *StorageMarketplaceDealProposed, dealId []*big.Int, client []common.Address, prover []common.Address) (event.Subscription, error) {

	var dealIdRule []interface{}
	for _, dealIdItem := range dealId {
		dealIdRule = append(dealIdRule, dealIdItem)
	}
	var clientRule []interface{}
	for _, clientItem := range client {
		clientRule = append(clientRule, clientItem)
	}
	var proverRule []interface{}
	for _, proverItem := range prover {
		proverRule = append(proverRule, proverItem)
	}

	logs, sub, err := _StorageMarketplace.contract.WatchLogs(opts, "DealProposed", dealIdRule, clientRule, proverRule)
	if err != nil {
		return nil, err
	}
	return event.NewSubscription(func(quit <-chan struct{}) error {
		defer sub.Unsubscribe()
		for {
			select {
			case log := <-logs:
				// New log arrived, parse the event and forward to the user
				event := new(StorageMarketplaceDealProposed)
				if err := _StorageMarketplace.contract.UnpackLog(event, "DealProposed", log); err != nil {
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

// ParseDealProposed is a log parse operation binding the contract event 0xcb8ee79aef97f2ea6c1c5672fedae874e3a5f900e255a5458c9c7d22e1042a34.
//
// Solidity: event DealProposed(uint256 indexed dealId, address indexed client, address indexed prover, bytes32 commpHash, uint64 pieceSize, uint64 durationSeconds, uint256 totalPayment)
func (_StorageMarketplace *StorageMarketplaceFilterer) ParseDealProposed(log types.Log) (*StorageMarketplaceDealProposed, error) {
	event := new(StorageMarketplaceDealProposed)
	if err := _StorageMarketplace.contract.UnpackLog(event, "DealProposed", log); err != nil {
		return nil, err
	}
	event.Raw = log
	return event, nil
}

// StorageMarketplaceDealSlashedIterator is returned from FilterDealSlashed and is used to iterate over the raw logs and unpacked data for DealSlashed events raised by the StorageMarketplace contract.
type StorageMarketplaceDealSlashedIterator struct {
	Event *StorageMarketplaceDealSlashed // Event containing the contract specifics and raw log

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
func (it *StorageMarketplaceDealSlashedIterator) Next() bool {
	// If the iterator failed, stop iterating
	if it.fail != nil {
		return false
	}
	// If the iterator completed, deliver directly whatever's available
	if it.done {
		select {
		case log := <-it.logs:
			it.Event = new(StorageMarketplaceDealSlashed)
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
		it.Event = new(StorageMarketplaceDealSlashed)
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
func (it *StorageMarketplaceDealSlashedIterator) Error() error {
	return it.fail
}

// Close terminates the iteration process, releasing any pending underlying
// resources.
func (it *StorageMarketplaceDealSlashedIterator) Close() error {
	it.sub.Unsubscribe()
	return nil
}

// StorageMarketplaceDealSlashed represents a DealSlashed event raised by the StorageMarketplace contract.
type StorageMarketplaceDealSlashed struct {
	DealId        *big.Int
	Prover        common.Address
	SlashedAmount *big.Int
	Refunded      *big.Int
	Raw           types.Log // Blockchain specific contextual infos
}

// FilterDealSlashed is a free log retrieval operation binding the contract event 0x91ee6d1cfad4d94b4a8498c4fc55e34ed316908db60b97d781848357450bc5c2.
//
// Solidity: event DealSlashed(uint256 indexed dealId, address indexed prover, uint256 slashedAmount, uint256 refunded)
func (_StorageMarketplace *StorageMarketplaceFilterer) FilterDealSlashed(opts *bind.FilterOpts, dealId []*big.Int, prover []common.Address) (*StorageMarketplaceDealSlashedIterator, error) {

	var dealIdRule []interface{}
	for _, dealIdItem := range dealId {
		dealIdRule = append(dealIdRule, dealIdItem)
	}
	var proverRule []interface{}
	for _, proverItem := range prover {
		proverRule = append(proverRule, proverItem)
	}

	logs, sub, err := _StorageMarketplace.contract.FilterLogs(opts, "DealSlashed", dealIdRule, proverRule)
	if err != nil {
		return nil, err
	}
	return &StorageMarketplaceDealSlashedIterator{contract: _StorageMarketplace.contract, event: "DealSlashed", logs: logs, sub: sub}, nil
}

// WatchDealSlashed is a free log subscription operation binding the contract event 0x91ee6d1cfad4d94b4a8498c4fc55e34ed316908db60b97d781848357450bc5c2.
//
// Solidity: event DealSlashed(uint256 indexed dealId, address indexed prover, uint256 slashedAmount, uint256 refunded)
func (_StorageMarketplace *StorageMarketplaceFilterer) WatchDealSlashed(opts *bind.WatchOpts, sink chan<- *StorageMarketplaceDealSlashed, dealId []*big.Int, prover []common.Address) (event.Subscription, error) {

	var dealIdRule []interface{}
	for _, dealIdItem := range dealId {
		dealIdRule = append(dealIdRule, dealIdItem)
	}
	var proverRule []interface{}
	for _, proverItem := range prover {
		proverRule = append(proverRule, proverItem)
	}

	logs, sub, err := _StorageMarketplace.contract.WatchLogs(opts, "DealSlashed", dealIdRule, proverRule)
	if err != nil {
		return nil, err
	}
	return event.NewSubscription(func(quit <-chan struct{}) error {
		defer sub.Unsubscribe()
		for {
			select {
			case log := <-logs:
				// New log arrived, parse the event and forward to the user
				event := new(StorageMarketplaceDealSlashed)
				if err := _StorageMarketplace.contract.UnpackLog(event, "DealSlashed", log); err != nil {
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

// ParseDealSlashed is a log parse operation binding the contract event 0x91ee6d1cfad4d94b4a8498c4fc55e34ed316908db60b97d781848357450bc5c2.
//
// Solidity: event DealSlashed(uint256 indexed dealId, address indexed prover, uint256 slashedAmount, uint256 refunded)
func (_StorageMarketplace *StorageMarketplaceFilterer) ParseDealSlashed(log types.Log) (*StorageMarketplaceDealSlashed, error) {
	event := new(StorageMarketplaceDealSlashed)
	if err := _StorageMarketplace.contract.UnpackLog(event, "DealSlashed", log); err != nil {
		return nil, err
	}
	event.Raw = log
	return event, nil
}

// StorageMarketplaceOwnershipTransferredIterator is returned from FilterOwnershipTransferred and is used to iterate over the raw logs and unpacked data for OwnershipTransferred events raised by the StorageMarketplace contract.
type StorageMarketplaceOwnershipTransferredIterator struct {
	Event *StorageMarketplaceOwnershipTransferred // Event containing the contract specifics and raw log

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
func (it *StorageMarketplaceOwnershipTransferredIterator) Next() bool {
	// If the iterator failed, stop iterating
	if it.fail != nil {
		return false
	}
	// If the iterator completed, deliver directly whatever's available
	if it.done {
		select {
		case log := <-it.logs:
			it.Event = new(StorageMarketplaceOwnershipTransferred)
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
		it.Event = new(StorageMarketplaceOwnershipTransferred)
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
func (it *StorageMarketplaceOwnershipTransferredIterator) Error() error {
	return it.fail
}

// Close terminates the iteration process, releasing any pending underlying
// resources.
func (it *StorageMarketplaceOwnershipTransferredIterator) Close() error {
	it.sub.Unsubscribe()
	return nil
}

// StorageMarketplaceOwnershipTransferred represents a OwnershipTransferred event raised by the StorageMarketplace contract.
type StorageMarketplaceOwnershipTransferred struct {
	PreviousOwner common.Address
	NewOwner      common.Address
	Raw           types.Log // Blockchain specific contextual infos
}

// FilterOwnershipTransferred is a free log retrieval operation binding the contract event 0x8be0079c531659141344cd1fd0a4f28419497f9722a3daafe3b4186f6b6457e0.
//
// Solidity: event OwnershipTransferred(address indexed previousOwner, address indexed newOwner)
func (_StorageMarketplace *StorageMarketplaceFilterer) FilterOwnershipTransferred(opts *bind.FilterOpts, previousOwner []common.Address, newOwner []common.Address) (*StorageMarketplaceOwnershipTransferredIterator, error) {

	var previousOwnerRule []interface{}
	for _, previousOwnerItem := range previousOwner {
		previousOwnerRule = append(previousOwnerRule, previousOwnerItem)
	}
	var newOwnerRule []interface{}
	for _, newOwnerItem := range newOwner {
		newOwnerRule = append(newOwnerRule, newOwnerItem)
	}

	logs, sub, err := _StorageMarketplace.contract.FilterLogs(opts, "OwnershipTransferred", previousOwnerRule, newOwnerRule)
	if err != nil {
		return nil, err
	}
	return &StorageMarketplaceOwnershipTransferredIterator{contract: _StorageMarketplace.contract, event: "OwnershipTransferred", logs: logs, sub: sub}, nil
}

// WatchOwnershipTransferred is a free log subscription operation binding the contract event 0x8be0079c531659141344cd1fd0a4f28419497f9722a3daafe3b4186f6b6457e0.
//
// Solidity: event OwnershipTransferred(address indexed previousOwner, address indexed newOwner)
func (_StorageMarketplace *StorageMarketplaceFilterer) WatchOwnershipTransferred(opts *bind.WatchOpts, sink chan<- *StorageMarketplaceOwnershipTransferred, previousOwner []common.Address, newOwner []common.Address) (event.Subscription, error) {

	var previousOwnerRule []interface{}
	for _, previousOwnerItem := range previousOwner {
		previousOwnerRule = append(previousOwnerRule, previousOwnerItem)
	}
	var newOwnerRule []interface{}
	for _, newOwnerItem := range newOwner {
		newOwnerRule = append(newOwnerRule, newOwnerItem)
	}

	logs, sub, err := _StorageMarketplace.contract.WatchLogs(opts, "OwnershipTransferred", previousOwnerRule, newOwnerRule)
	if err != nil {
		return nil, err
	}
	return event.NewSubscription(func(quit <-chan struct{}) error {
		defer sub.Unsubscribe()
		for {
			select {
			case log := <-logs:
				// New log arrived, parse the event and forward to the user
				event := new(StorageMarketplaceOwnershipTransferred)
				if err := _StorageMarketplace.contract.UnpackLog(event, "OwnershipTransferred", log); err != nil {
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
func (_StorageMarketplace *StorageMarketplaceFilterer) ParseOwnershipTransferred(log types.Log) (*StorageMarketplaceOwnershipTransferred, error) {
	event := new(StorageMarketplaceOwnershipTransferred)
	if err := _StorageMarketplace.contract.UnpackLog(event, "OwnershipTransferred", log); err != nil {
		return nil, err
	}
	event.Raw = log
	return event, nil
}

// StorageMarketplaceProofRecordedIterator is returned from FilterProofRecorded and is used to iterate over the raw logs and unpacked data for ProofRecorded events raised by the StorageMarketplace contract.
type StorageMarketplaceProofRecordedIterator struct {
	Event *StorageMarketplaceProofRecorded // Event containing the contract specifics and raw log

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
func (it *StorageMarketplaceProofRecordedIterator) Next() bool {
	// If the iterator failed, stop iterating
	if it.fail != nil {
		return false
	}
	// If the iterator completed, deliver directly whatever's available
	if it.done {
		select {
		case log := <-it.logs:
			it.Event = new(StorageMarketplaceProofRecorded)
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
		it.Event = new(StorageMarketplaceProofRecorded)
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
func (it *StorageMarketplaceProofRecordedIterator) Error() error {
	return it.fail
}

// Close terminates the iteration process, releasing any pending underlying
// resources.
func (it *StorageMarketplaceProofRecordedIterator) Close() error {
	it.sub.Unsubscribe()
	return nil
}

// StorageMarketplaceProofRecorded represents a ProofRecorded event raised by the StorageMarketplace contract.
type StorageMarketplaceProofRecorded struct {
	DealId          *big.Int
	ProofCount      *big.Int
	PaymentReleased *big.Int
	Raw             types.Log // Blockchain specific contextual infos
}

// FilterProofRecorded is a free log retrieval operation binding the contract event 0xcd3b3c1736fe7f5b05b3965d64e2a10662b20741cf825afc3e2ed2122369d168.
//
// Solidity: event ProofRecorded(uint256 indexed dealId, uint256 proofCount, uint256 paymentReleased)
func (_StorageMarketplace *StorageMarketplaceFilterer) FilterProofRecorded(opts *bind.FilterOpts, dealId []*big.Int) (*StorageMarketplaceProofRecordedIterator, error) {

	var dealIdRule []interface{}
	for _, dealIdItem := range dealId {
		dealIdRule = append(dealIdRule, dealIdItem)
	}

	logs, sub, err := _StorageMarketplace.contract.FilterLogs(opts, "ProofRecorded", dealIdRule)
	if err != nil {
		return nil, err
	}
	return &StorageMarketplaceProofRecordedIterator{contract: _StorageMarketplace.contract, event: "ProofRecorded", logs: logs, sub: sub}, nil
}

// WatchProofRecorded is a free log subscription operation binding the contract event 0xcd3b3c1736fe7f5b05b3965d64e2a10662b20741cf825afc3e2ed2122369d168.
//
// Solidity: event ProofRecorded(uint256 indexed dealId, uint256 proofCount, uint256 paymentReleased)
func (_StorageMarketplace *StorageMarketplaceFilterer) WatchProofRecorded(opts *bind.WatchOpts, sink chan<- *StorageMarketplaceProofRecorded, dealId []*big.Int) (event.Subscription, error) {

	var dealIdRule []interface{}
	for _, dealIdItem := range dealId {
		dealIdRule = append(dealIdRule, dealIdItem)
	}

	logs, sub, err := _StorageMarketplace.contract.WatchLogs(opts, "ProofRecorded", dealIdRule)
	if err != nil {
		return nil, err
	}
	return event.NewSubscription(func(quit <-chan struct{}) error {
		defer sub.Unsubscribe()
		for {
			select {
			case log := <-logs:
				// New log arrived, parse the event and forward to the user
				event := new(StorageMarketplaceProofRecorded)
				if err := _StorageMarketplace.contract.UnpackLog(event, "ProofRecorded", log); err != nil {
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

// ParseProofRecorded is a log parse operation binding the contract event 0xcd3b3c1736fe7f5b05b3965d64e2a10662b20741cf825afc3e2ed2122369d168.
//
// Solidity: event ProofRecorded(uint256 indexed dealId, uint256 proofCount, uint256 paymentReleased)
func (_StorageMarketplace *StorageMarketplaceFilterer) ParseProofRecorded(log types.Log) (*StorageMarketplaceProofRecorded, error) {
	event := new(StorageMarketplaceProofRecorded)
	if err := _StorageMarketplace.contract.UnpackLog(event, "ProofRecorded", log); err != nil {
		return nil, err
	}
	event.Raw = log
	return event, nil
}

// StorageMarketplaceProtocolFeeChangedIterator is returned from FilterProtocolFeeChanged and is used to iterate over the raw logs and unpacked data for ProtocolFeeChanged events raised by the StorageMarketplace contract.
type StorageMarketplaceProtocolFeeChangedIterator struct {
	Event *StorageMarketplaceProtocolFeeChanged // Event containing the contract specifics and raw log

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
func (it *StorageMarketplaceProtocolFeeChangedIterator) Next() bool {
	// If the iterator failed, stop iterating
	if it.fail != nil {
		return false
	}
	// If the iterator completed, deliver directly whatever's available
	if it.done {
		select {
		case log := <-it.logs:
			it.Event = new(StorageMarketplaceProtocolFeeChanged)
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
		it.Event = new(StorageMarketplaceProtocolFeeChanged)
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
func (it *StorageMarketplaceProtocolFeeChangedIterator) Error() error {
	return it.fail
}

// Close terminates the iteration process, releasing any pending underlying
// resources.
func (it *StorageMarketplaceProtocolFeeChangedIterator) Close() error {
	it.sub.Unsubscribe()
	return nil
}

// StorageMarketplaceProtocolFeeChanged represents a ProtocolFeeChanged event raised by the StorageMarketplace contract.
type StorageMarketplaceProtocolFeeChanged struct {
	OldBps *big.Int
	NewBps *big.Int
	Raw    types.Log // Blockchain specific contextual infos
}

// FilterProtocolFeeChanged is a free log retrieval operation binding the contract event 0xb51bef650ff5ad43303dbe2e500a74d4fd1bdc9ae05f046bece330e82ae0ba87.
//
// Solidity: event ProtocolFeeChanged(uint256 oldBps, uint256 newBps)
func (_StorageMarketplace *StorageMarketplaceFilterer) FilterProtocolFeeChanged(opts *bind.FilterOpts) (*StorageMarketplaceProtocolFeeChangedIterator, error) {

	logs, sub, err := _StorageMarketplace.contract.FilterLogs(opts, "ProtocolFeeChanged")
	if err != nil {
		return nil, err
	}
	return &StorageMarketplaceProtocolFeeChangedIterator{contract: _StorageMarketplace.contract, event: "ProtocolFeeChanged", logs: logs, sub: sub}, nil
}

// WatchProtocolFeeChanged is a free log subscription operation binding the contract event 0xb51bef650ff5ad43303dbe2e500a74d4fd1bdc9ae05f046bece330e82ae0ba87.
//
// Solidity: event ProtocolFeeChanged(uint256 oldBps, uint256 newBps)
func (_StorageMarketplace *StorageMarketplaceFilterer) WatchProtocolFeeChanged(opts *bind.WatchOpts, sink chan<- *StorageMarketplaceProtocolFeeChanged) (event.Subscription, error) {

	logs, sub, err := _StorageMarketplace.contract.WatchLogs(opts, "ProtocolFeeChanged")
	if err != nil {
		return nil, err
	}
	return event.NewSubscription(func(quit <-chan struct{}) error {
		defer sub.Unsubscribe()
		for {
			select {
			case log := <-logs:
				// New log arrived, parse the event and forward to the user
				event := new(StorageMarketplaceProtocolFeeChanged)
				if err := _StorageMarketplace.contract.UnpackLog(event, "ProtocolFeeChanged", log); err != nil {
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

// ParseProtocolFeeChanged is a log parse operation binding the contract event 0xb51bef650ff5ad43303dbe2e500a74d4fd1bdc9ae05f046bece330e82ae0ba87.
//
// Solidity: event ProtocolFeeChanged(uint256 oldBps, uint256 newBps)
func (_StorageMarketplace *StorageMarketplaceFilterer) ParseProtocolFeeChanged(log types.Log) (*StorageMarketplaceProtocolFeeChanged, error) {
	event := new(StorageMarketplaceProtocolFeeChanged)
	if err := _StorageMarketplace.contract.UnpackLog(event, "ProtocolFeeChanged", log); err != nil {
		return nil, err
	}
	event.Raw = log
	return event, nil
}

// StorageMarketplaceSlashPerFaultChangedIterator is returned from FilterSlashPerFaultChanged and is used to iterate over the raw logs and unpacked data for SlashPerFaultChanged events raised by the StorageMarketplace contract.
type StorageMarketplaceSlashPerFaultChangedIterator struct {
	Event *StorageMarketplaceSlashPerFaultChanged // Event containing the contract specifics and raw log

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
func (it *StorageMarketplaceSlashPerFaultChangedIterator) Next() bool {
	// If the iterator failed, stop iterating
	if it.fail != nil {
		return false
	}
	// If the iterator completed, deliver directly whatever's available
	if it.done {
		select {
		case log := <-it.logs:
			it.Event = new(StorageMarketplaceSlashPerFaultChanged)
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
		it.Event = new(StorageMarketplaceSlashPerFaultChanged)
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
func (it *StorageMarketplaceSlashPerFaultChangedIterator) Error() error {
	return it.fail
}

// Close terminates the iteration process, releasing any pending underlying
// resources.
func (it *StorageMarketplaceSlashPerFaultChangedIterator) Close() error {
	it.sub.Unsubscribe()
	return nil
}

// StorageMarketplaceSlashPerFaultChanged represents a SlashPerFaultChanged event raised by the StorageMarketplace contract.
type StorageMarketplaceSlashPerFaultChanged struct {
	OldValue *big.Int
	NewValue *big.Int
	Raw      types.Log // Blockchain specific contextual infos
}

// FilterSlashPerFaultChanged is a free log retrieval operation binding the contract event 0x20761421f245b09f460443e59d7927ac61146d996e01b596dc2abc206450ac2b.
//
// Solidity: event SlashPerFaultChanged(uint256 oldValue, uint256 newValue)
func (_StorageMarketplace *StorageMarketplaceFilterer) FilterSlashPerFaultChanged(opts *bind.FilterOpts) (*StorageMarketplaceSlashPerFaultChangedIterator, error) {

	logs, sub, err := _StorageMarketplace.contract.FilterLogs(opts, "SlashPerFaultChanged")
	if err != nil {
		return nil, err
	}
	return &StorageMarketplaceSlashPerFaultChangedIterator{contract: _StorageMarketplace.contract, event: "SlashPerFaultChanged", logs: logs, sub: sub}, nil
}

// WatchSlashPerFaultChanged is a free log subscription operation binding the contract event 0x20761421f245b09f460443e59d7927ac61146d996e01b596dc2abc206450ac2b.
//
// Solidity: event SlashPerFaultChanged(uint256 oldValue, uint256 newValue)
func (_StorageMarketplace *StorageMarketplaceFilterer) WatchSlashPerFaultChanged(opts *bind.WatchOpts, sink chan<- *StorageMarketplaceSlashPerFaultChanged) (event.Subscription, error) {

	logs, sub, err := _StorageMarketplace.contract.WatchLogs(opts, "SlashPerFaultChanged")
	if err != nil {
		return nil, err
	}
	return event.NewSubscription(func(quit <-chan struct{}) error {
		defer sub.Unsubscribe()
		for {
			select {
			case log := <-logs:
				// New log arrived, parse the event and forward to the user
				event := new(StorageMarketplaceSlashPerFaultChanged)
				if err := _StorageMarketplace.contract.UnpackLog(event, "SlashPerFaultChanged", log); err != nil {
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

// ParseSlashPerFaultChanged is a log parse operation binding the contract event 0x20761421f245b09f460443e59d7927ac61146d996e01b596dc2abc206450ac2b.
//
// Solidity: event SlashPerFaultChanged(uint256 oldValue, uint256 newValue)
func (_StorageMarketplace *StorageMarketplaceFilterer) ParseSlashPerFaultChanged(log types.Log) (*StorageMarketplaceSlashPerFaultChanged, error) {
	event := new(StorageMarketplaceSlashPerFaultChanged)
	if err := _StorageMarketplace.contract.UnpackLog(event, "SlashPerFaultChanged", log); err != nil {
		return nil, err
	}
	event.Raw = log
	return event, nil
}

// StorageMarketplaceTreasuryChangedIterator is returned from FilterTreasuryChanged and is used to iterate over the raw logs and unpacked data for TreasuryChanged events raised by the StorageMarketplace contract.
type StorageMarketplaceTreasuryChangedIterator struct {
	Event *StorageMarketplaceTreasuryChanged // Event containing the contract specifics and raw log

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
func (it *StorageMarketplaceTreasuryChangedIterator) Next() bool {
	// If the iterator failed, stop iterating
	if it.fail != nil {
		return false
	}
	// If the iterator completed, deliver directly whatever's available
	if it.done {
		select {
		case log := <-it.logs:
			it.Event = new(StorageMarketplaceTreasuryChanged)
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
		it.Event = new(StorageMarketplaceTreasuryChanged)
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
func (it *StorageMarketplaceTreasuryChangedIterator) Error() error {
	return it.fail
}

// Close terminates the iteration process, releasing any pending underlying
// resources.
func (it *StorageMarketplaceTreasuryChangedIterator) Close() error {
	it.sub.Unsubscribe()
	return nil
}

// StorageMarketplaceTreasuryChanged represents a TreasuryChanged event raised by the StorageMarketplace contract.
type StorageMarketplaceTreasuryChanged struct {
	OldTreasury common.Address
	NewTreasury common.Address
	Raw         types.Log // Blockchain specific contextual infos
}

// FilterTreasuryChanged is a free log retrieval operation binding the contract event 0x8c3aa5f43a388513435861bf27dfad7829cd248696fed367c62d441f62954496.
//
// Solidity: event TreasuryChanged(address indexed oldTreasury, address indexed newTreasury)
func (_StorageMarketplace *StorageMarketplaceFilterer) FilterTreasuryChanged(opts *bind.FilterOpts, oldTreasury []common.Address, newTreasury []common.Address) (*StorageMarketplaceTreasuryChangedIterator, error) {

	var oldTreasuryRule []interface{}
	for _, oldTreasuryItem := range oldTreasury {
		oldTreasuryRule = append(oldTreasuryRule, oldTreasuryItem)
	}
	var newTreasuryRule []interface{}
	for _, newTreasuryItem := range newTreasury {
		newTreasuryRule = append(newTreasuryRule, newTreasuryItem)
	}

	logs, sub, err := _StorageMarketplace.contract.FilterLogs(opts, "TreasuryChanged", oldTreasuryRule, newTreasuryRule)
	if err != nil {
		return nil, err
	}
	return &StorageMarketplaceTreasuryChangedIterator{contract: _StorageMarketplace.contract, event: "TreasuryChanged", logs: logs, sub: sub}, nil
}

// WatchTreasuryChanged is a free log subscription operation binding the contract event 0x8c3aa5f43a388513435861bf27dfad7829cd248696fed367c62d441f62954496.
//
// Solidity: event TreasuryChanged(address indexed oldTreasury, address indexed newTreasury)
func (_StorageMarketplace *StorageMarketplaceFilterer) WatchTreasuryChanged(opts *bind.WatchOpts, sink chan<- *StorageMarketplaceTreasuryChanged, oldTreasury []common.Address, newTreasury []common.Address) (event.Subscription, error) {

	var oldTreasuryRule []interface{}
	for _, oldTreasuryItem := range oldTreasury {
		oldTreasuryRule = append(oldTreasuryRule, oldTreasuryItem)
	}
	var newTreasuryRule []interface{}
	for _, newTreasuryItem := range newTreasury {
		newTreasuryRule = append(newTreasuryRule, newTreasuryItem)
	}

	logs, sub, err := _StorageMarketplace.contract.WatchLogs(opts, "TreasuryChanged", oldTreasuryRule, newTreasuryRule)
	if err != nil {
		return nil, err
	}
	return event.NewSubscription(func(quit <-chan struct{}) error {
		defer sub.Unsubscribe()
		for {
			select {
			case log := <-logs:
				// New log arrived, parse the event and forward to the user
				event := new(StorageMarketplaceTreasuryChanged)
				if err := _StorageMarketplace.contract.UnpackLog(event, "TreasuryChanged", log); err != nil {
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

// ParseTreasuryChanged is a log parse operation binding the contract event 0x8c3aa5f43a388513435861bf27dfad7829cd248696fed367c62d441f62954496.
//
// Solidity: event TreasuryChanged(address indexed oldTreasury, address indexed newTreasury)
func (_StorageMarketplace *StorageMarketplaceFilterer) ParseTreasuryChanged(log types.Log) (*StorageMarketplaceTreasuryChanged, error) {
	event := new(StorageMarketplaceTreasuryChanged)
	if err := _StorageMarketplace.contract.UnpackLog(event, "TreasuryChanged", log); err != nil {
		return nil, err
	}
	event.Raw = log
	return event, nil
}
