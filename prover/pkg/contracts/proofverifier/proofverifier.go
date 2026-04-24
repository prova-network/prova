// SPDX-License-Identifier: MIT
// Generated from contracts/out/ProofVerifier.sol/ProofVerifier.json via abigen.
// Do not edit by hand; run ./scripts/gen-bindings.sh instead.

// Code generated - DO NOT EDIT.
// This file is a generated binding and any manual changes will be lost.

package proofverifier

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

// IPDPTypesPieceIdAndOffset is an auto generated low-level Go binding around an user-defined struct.
type IPDPTypesPieceIdAndOffset struct {
	PieceId *big.Int
	Offset  *big.Int
}

// IPDPTypesProof is an auto generated low-level Go binding around an user-defined struct.
type IPDPTypesProof struct {
	Leaf  [32]byte
	Proof [][32]byte
}

// ProofVerifierPlannedUpgrade is an auto generated low-level Go binding around an user-defined struct.
type ProofVerifierPlannedUpgrade struct {
	NextImplementation common.Address
	AfterEpoch         *big.Int
}

// ProofVerifierMetaData contains all meta data concerning the ProofVerifier contract.
var ProofVerifierMetaData = &bind.MetaData{
	ABI: "[{\"type\":\"constructor\",\"inputs\":[{\"name\":\"_initializerVersion\",\"type\":\"uint64\",\"internalType\":\"uint64\"},{\"name\":\"_usdfcTokenAddress\",\"type\":\"address\",\"internalType\":\"address\"},{\"name\":\"_usdfcSybilFee\",\"type\":\"uint256\",\"internalType\":\"uint256\"},{\"name\":\"_paymentsContractAddress\",\"type\":\"address\",\"internalType\":\"address\"}],\"stateMutability\":\"nonpayable\"},{\"type\":\"function\",\"name\":\"FIL_SYBIL_FEE\",\"inputs\":[],\"outputs\":[{\"name\":\"\",\"type\":\"uint256\",\"internalType\":\"uint256\"}],\"stateMutability\":\"pure\"},{\"type\":\"function\",\"name\":\"MAX_ENQUEUED_REMOVALS\",\"inputs\":[],\"outputs\":[{\"name\":\"\",\"type\":\"uint256\",\"internalType\":\"uint256\"}],\"stateMutability\":\"view\"},{\"type\":\"function\",\"name\":\"MAX_PIECE_SIZE_LOG2\",\"inputs\":[],\"outputs\":[{\"name\":\"\",\"type\":\"uint256\",\"internalType\":\"uint256\"}],\"stateMutability\":\"view\"},{\"type\":\"function\",\"name\":\"NO_CHALLENGE_SCHEDULED\",\"inputs\":[],\"outputs\":[{\"name\":\"\",\"type\":\"uint256\",\"internalType\":\"uint256\"}],\"stateMutability\":\"view\"},{\"type\":\"function\",\"name\":\"NO_PROVEN_EPOCH\",\"inputs\":[],\"outputs\":[{\"name\":\"\",\"type\":\"uint256\",\"internalType\":\"uint256\"}],\"stateMutability\":\"view\"},{\"type\":\"function\",\"name\":\"PAYMENTS_CONTRACT_ADDRESS\",\"inputs\":[],\"outputs\":[{\"name\":\"\",\"type\":\"address\",\"internalType\":\"address\"}],\"stateMutability\":\"view\"},{\"type\":\"function\",\"name\":\"UPGRADE_INTERFACE_VERSION\",\"inputs\":[],\"outputs\":[{\"name\":\"\",\"type\":\"string\",\"internalType\":\"string\"}],\"stateMutability\":\"view\"},{\"type\":\"function\",\"name\":\"USDFC_SYBIL_FEE\",\"inputs\":[],\"outputs\":[{\"name\":\"\",\"type\":\"uint256\",\"internalType\":\"uint256\"}],\"stateMutability\":\"view\"},{\"type\":\"function\",\"name\":\"USDFC_TOKEN_ADDRESS\",\"inputs\":[],\"outputs\":[{\"name\":\"\",\"type\":\"address\",\"internalType\":\"address\"}],\"stateMutability\":\"view\"},{\"type\":\"function\",\"name\":\"VERSION\",\"inputs\":[],\"outputs\":[{\"name\":\"\",\"type\":\"string\",\"internalType\":\"string\"}],\"stateMutability\":\"view\"},{\"type\":\"function\",\"name\":\"addPieces\",\"inputs\":[{\"name\":\"setId\",\"type\":\"uint256\",\"internalType\":\"uint256\"},{\"name\":\"listenerAddr\",\"type\":\"address\",\"internalType\":\"address\"},{\"name\":\"pieceData\",\"type\":\"tuple[]\",\"internalType\":\"structCids.Cid[]\",\"components\":[{\"name\":\"data\",\"type\":\"bytes\",\"internalType\":\"bytes\"}]},{\"name\":\"extraData\",\"type\":\"bytes\",\"internalType\":\"bytes\"}],\"outputs\":[{\"name\":\"\",\"type\":\"uint256\",\"internalType\":\"uint256\"}],\"stateMutability\":\"payable\"},{\"type\":\"function\",\"name\":\"announcePlannedUpgrade\",\"inputs\":[{\"name\":\"plannedUpgrade\",\"type\":\"tuple\",\"internalType\":\"structProofVerifier.PlannedUpgrade\",\"components\":[{\"name\":\"nextImplementation\",\"type\":\"address\",\"internalType\":\"address\"},{\"name\":\"afterEpoch\",\"type\":\"uint96\",\"internalType\":\"uint96\"}]}],\"outputs\":[],\"stateMutability\":\"nonpayable\"},{\"type\":\"function\",\"name\":\"calculateProofFee\",\"inputs\":[{\"name\":\"setId\",\"type\":\"uint256\",\"internalType\":\"uint256\"}],\"outputs\":[{\"name\":\"\",\"type\":\"uint256\",\"internalType\":\"uint256\"}],\"stateMutability\":\"view\"},{\"type\":\"function\",\"name\":\"calculateProofFeeForSize\",\"inputs\":[{\"name\":\"proofSize\",\"type\":\"uint256\",\"internalType\":\"uint256\"}],\"outputs\":[{\"name\":\"\",\"type\":\"uint256\",\"internalType\":\"uint256\"}],\"stateMutability\":\"view\"},{\"type\":\"function\",\"name\":\"claimDataSetStorageProvider\",\"inputs\":[{\"name\":\"setId\",\"type\":\"uint256\",\"internalType\":\"uint256\"},{\"name\":\"extraData\",\"type\":\"bytes\",\"internalType\":\"bytes\"}],\"outputs\":[],\"stateMutability\":\"nonpayable\"},{\"type\":\"function\",\"name\":\"createDataSet\",\"inputs\":[{\"name\":\"listenerAddr\",\"type\":\"address\",\"internalType\":\"address\"},{\"name\":\"extraData\",\"type\":\"bytes\",\"internalType\":\"bytes\"}],\"outputs\":[{\"name\":\"\",\"type\":\"uint256\",\"internalType\":\"uint256\"}],\"stateMutability\":\"payable\"},{\"type\":\"function\",\"name\":\"dataSetLive\",\"inputs\":[{\"name\":\"setId\",\"type\":\"uint256\",\"internalType\":\"uint256\"}],\"outputs\":[{\"name\":\"\",\"type\":\"bool\",\"internalType\":\"bool\"}],\"stateMutability\":\"view\"},{\"type\":\"function\",\"name\":\"deleteDataSet\",\"inputs\":[{\"name\":\"setId\",\"type\":\"uint256\",\"internalType\":\"uint256\"},{\"name\":\"extraData\",\"type\":\"bytes\",\"internalType\":\"bytes\"}],\"outputs\":[],\"stateMutability\":\"nonpayable\"},{\"type\":\"function\",\"name\":\"feeEffectiveTime\",\"inputs\":[],\"outputs\":[{\"name\":\"\",\"type\":\"uint64\",\"internalType\":\"uint64\"}],\"stateMutability\":\"view\"},{\"type\":\"function\",\"name\":\"feePerTiB\",\"inputs\":[],\"outputs\":[{\"name\":\"\",\"type\":\"uint96\",\"internalType\":\"uint96\"}],\"stateMutability\":\"view\"},{\"type\":\"function\",\"name\":\"findPieceIds\",\"inputs\":[{\"name\":\"setId\",\"type\":\"uint256\",\"internalType\":\"uint256\"},{\"name\":\"leafIndexs\",\"type\":\"uint256[]\",\"internalType\":\"uint256[]\"}],\"outputs\":[{\"name\":\"\",\"type\":\"tuple[]\",\"internalType\":\"structIPDPTypes.PieceIdAndOffset[]\",\"components\":[{\"name\":\"pieceId\",\"type\":\"uint256\",\"internalType\":\"uint256\"},{\"name\":\"offset\",\"type\":\"uint256\",\"internalType\":\"uint256\"}]}],\"stateMutability\":\"view\"},{\"type\":\"function\",\"name\":\"findPieceIdsByCid\",\"inputs\":[{\"name\":\"setId\",\"type\":\"uint256\",\"internalType\":\"uint256\"},{\"name\":\"pieceCid\",\"type\":\"tuple\",\"internalType\":\"structCids.Cid\",\"components\":[{\"name\":\"data\",\"type\":\"bytes\",\"internalType\":\"bytes\"}]},{\"name\":\"startPieceId\",\"type\":\"uint256\",\"internalType\":\"uint256\"},{\"name\":\"limit\",\"type\":\"uint256\",\"internalType\":\"uint256\"}],\"outputs\":[{\"name\":\"pieceIds\",\"type\":\"uint256[]\",\"internalType\":\"uint256[]\"}],\"stateMutability\":\"view\"},{\"type\":\"function\",\"name\":\"getActivePieceCount\",\"inputs\":[{\"name\":\"setId\",\"type\":\"uint256\",\"internalType\":\"uint256\"}],\"outputs\":[{\"name\":\"activeCount\",\"type\":\"uint256\",\"internalType\":\"uint256\"}],\"stateMutability\":\"view\"},{\"type\":\"function\",\"name\":\"getActivePieces\",\"inputs\":[{\"name\":\"setId\",\"type\":\"uint256\",\"internalType\":\"uint256\"},{\"name\":\"offset\",\"type\":\"uint256\",\"internalType\":\"uint256\"},{\"name\":\"limit\",\"type\":\"uint256\",\"internalType\":\"uint256\"}],\"outputs\":[{\"name\":\"pieces\",\"type\":\"tuple[]\",\"internalType\":\"structCids.Cid[]\",\"components\":[{\"name\":\"data\",\"type\":\"bytes\",\"internalType\":\"bytes\"}]},{\"name\":\"pieceIds\",\"type\":\"uint256[]\",\"internalType\":\"uint256[]\"},{\"name\":\"hasMore\",\"type\":\"bool\",\"internalType\":\"bool\"}],\"stateMutability\":\"view\"},{\"type\":\"function\",\"name\":\"getActivePiecesByCursor\",\"inputs\":[{\"name\":\"setId\",\"type\":\"uint256\",\"internalType\":\"uint256\"},{\"name\":\"startPieceId\",\"type\":\"uint256\",\"internalType\":\"uint256\"},{\"name\":\"limit\",\"type\":\"uint256\",\"internalType\":\"uint256\"}],\"outputs\":[{\"name\":\"pieces\",\"type\":\"tuple[]\",\"internalType\":\"structCids.Cid[]\",\"components\":[{\"name\":\"data\",\"type\":\"bytes\",\"internalType\":\"bytes\"}]},{\"name\":\"pieceIds\",\"type\":\"uint256[]\",\"internalType\":\"uint256[]\"},{\"name\":\"hasMore\",\"type\":\"bool\",\"internalType\":\"bool\"}],\"stateMutability\":\"view\"},{\"type\":\"function\",\"name\":\"getChallengeFinality\",\"inputs\":[],\"outputs\":[{\"name\":\"\",\"type\":\"uint256\",\"internalType\":\"uint256\"}],\"stateMutability\":\"view\"},{\"type\":\"function\",\"name\":\"getChallengeRange\",\"inputs\":[{\"name\":\"setId\",\"type\":\"uint256\",\"internalType\":\"uint256\"}],\"outputs\":[{\"name\":\"\",\"type\":\"uint256\",\"internalType\":\"uint256\"}],\"stateMutability\":\"view\"},{\"type\":\"function\",\"name\":\"getDataSetLastProvenEpoch\",\"inputs\":[{\"name\":\"setId\",\"type\":\"uint256\",\"internalType\":\"uint256\"}],\"outputs\":[{\"name\":\"\",\"type\":\"uint256\",\"internalType\":\"uint256\"}],\"stateMutability\":\"view\"},{\"type\":\"function\",\"name\":\"getDataSetLeafCount\",\"inputs\":[{\"name\":\"setId\",\"type\":\"uint256\",\"internalType\":\"uint256\"}],\"outputs\":[{\"name\":\"\",\"type\":\"uint256\",\"internalType\":\"uint256\"}],\"stateMutability\":\"view\"},{\"type\":\"function\",\"name\":\"getDataSetListener\",\"inputs\":[{\"name\":\"setId\",\"type\":\"uint256\",\"internalType\":\"uint256\"}],\"outputs\":[{\"name\":\"\",\"type\":\"address\",\"internalType\":\"address\"}],\"stateMutability\":\"view\"},{\"type\":\"function\",\"name\":\"getDataSetStorageProvider\",\"inputs\":[{\"name\":\"setId\",\"type\":\"uint256\",\"internalType\":\"uint256\"}],\"outputs\":[{\"name\":\"\",\"type\":\"address\",\"internalType\":\"address\"},{\"name\":\"\",\"type\":\"address\",\"internalType\":\"address\"}],\"stateMutability\":\"view\"},{\"type\":\"function\",\"name\":\"getNextChallengeEpoch\",\"inputs\":[{\"name\":\"setId\",\"type\":\"uint256\",\"internalType\":\"uint256\"}],\"outputs\":[{\"name\":\"\",\"type\":\"uint256\",\"internalType\":\"uint256\"}],\"stateMutability\":\"view\"},{\"type\":\"function\",\"name\":\"getNextDataSetId\",\"inputs\":[],\"outputs\":[{\"name\":\"\",\"type\":\"uint64\",\"internalType\":\"uint64\"}],\"stateMutability\":\"view\"},{\"type\":\"function\",\"name\":\"getNextPieceId\",\"inputs\":[{\"name\":\"setId\",\"type\":\"uint256\",\"internalType\":\"uint256\"}],\"outputs\":[{\"name\":\"\",\"type\":\"uint256\",\"internalType\":\"uint256\"}],\"stateMutability\":\"view\"},{\"type\":\"function\",\"name\":\"getPieceCid\",\"inputs\":[{\"name\":\"setId\",\"type\":\"uint256\",\"internalType\":\"uint256\"},{\"name\":\"pieceId\",\"type\":\"uint256\",\"internalType\":\"uint256\"}],\"outputs\":[{\"name\":\"\",\"type\":\"tuple\",\"internalType\":\"structCids.Cid\",\"components\":[{\"name\":\"data\",\"type\":\"bytes\",\"internalType\":\"bytes\"}]}],\"stateMutability\":\"view\"},{\"type\":\"function\",\"name\":\"getPieceLeafCount\",\"inputs\":[{\"name\":\"setId\",\"type\":\"uint256\",\"internalType\":\"uint256\"},{\"name\":\"pieceId\",\"type\":\"uint256\",\"internalType\":\"uint256\"}],\"outputs\":[{\"name\":\"\",\"type\":\"uint256\",\"internalType\":\"uint256\"}],\"stateMutability\":\"view\"},{\"type\":\"function\",\"name\":\"getRandomness\",\"inputs\":[{\"name\":\"epoch\",\"type\":\"uint256\",\"internalType\":\"uint256\"}],\"outputs\":[{\"name\":\"\",\"type\":\"uint256\",\"internalType\":\"uint256\"}],\"stateMutability\":\"view\"},{\"type\":\"function\",\"name\":\"getScheduledRemovals\",\"inputs\":[{\"name\":\"setId\",\"type\":\"uint256\",\"internalType\":\"uint256\"}],\"outputs\":[{\"name\":\"\",\"type\":\"uint256[]\",\"internalType\":\"uint256[]\"}],\"stateMutability\":\"view\"},{\"type\":\"function\",\"name\":\"initialize\",\"inputs\":[{\"name\":\"_challengeFinality\",\"type\":\"uint256\",\"internalType\":\"uint256\"}],\"outputs\":[],\"stateMutability\":\"nonpayable\"},{\"type\":\"function\",\"name\":\"migrate\",\"inputs\":[],\"outputs\":[],\"stateMutability\":\"nonpayable\"},{\"type\":\"function\",\"name\":\"nextProvingPeriod\",\"inputs\":[{\"name\":\"setId\",\"type\":\"uint256\",\"internalType\":\"uint256\"},{\"name\":\"challengeEpoch\",\"type\":\"uint256\",\"internalType\":\"uint256\"},{\"name\":\"extraData\",\"type\":\"bytes\",\"internalType\":\"bytes\"}],\"outputs\":[],\"stateMutability\":\"nonpayable\"},{\"type\":\"function\",\"name\":\"nextUpgrade\",\"inputs\":[],\"outputs\":[{\"name\":\"nextImplementation\",\"type\":\"address\",\"internalType\":\"address\"},{\"name\":\"afterEpoch\",\"type\":\"uint96\",\"internalType\":\"uint96\"}],\"stateMutability\":\"view\"},{\"type\":\"function\",\"name\":\"owner\",\"inputs\":[],\"outputs\":[{\"name\":\"\",\"type\":\"address\",\"internalType\":\"address\"}],\"stateMutability\":\"view\"},{\"type\":\"function\",\"name\":\"pieceChallengable\",\"inputs\":[{\"name\":\"setId\",\"type\":\"uint256\",\"internalType\":\"uint256\"},{\"name\":\"pieceId\",\"type\":\"uint256\",\"internalType\":\"uint256\"}],\"outputs\":[{\"name\":\"\",\"type\":\"bool\",\"internalType\":\"bool\"}],\"stateMutability\":\"view\"},{\"type\":\"function\",\"name\":\"pieceLive\",\"inputs\":[{\"name\":\"setId\",\"type\":\"uint256\",\"internalType\":\"uint256\"},{\"name\":\"pieceId\",\"type\":\"uint256\",\"internalType\":\"uint256\"}],\"outputs\":[{\"name\":\"\",\"type\":\"bool\",\"internalType\":\"bool\"}],\"stateMutability\":\"view\"},{\"type\":\"function\",\"name\":\"proposeDataSetStorageProvider\",\"inputs\":[{\"name\":\"setId\",\"type\":\"uint256\",\"internalType\":\"uint256\"},{\"name\":\"newStorageProvider\",\"type\":\"address\",\"internalType\":\"address\"}],\"outputs\":[],\"stateMutability\":\"nonpayable\"},{\"type\":\"function\",\"name\":\"proposedFeePerTiB\",\"inputs\":[],\"outputs\":[{\"name\":\"\",\"type\":\"uint96\",\"internalType\":\"uint96\"}],\"stateMutability\":\"view\"},{\"type\":\"function\",\"name\":\"provePossession\",\"inputs\":[{\"name\":\"setId\",\"type\":\"uint256\",\"internalType\":\"uint256\"},{\"name\":\"proofs\",\"type\":\"tuple[]\",\"internalType\":\"structIPDPTypes.Proof[]\",\"components\":[{\"name\":\"leaf\",\"type\":\"bytes32\",\"internalType\":\"bytes32\"},{\"name\":\"proof\",\"type\":\"bytes32[]\",\"internalType\":\"bytes32[]\"}]}],\"outputs\":[],\"stateMutability\":\"payable\"},{\"type\":\"function\",\"name\":\"proxiableUUID\",\"inputs\":[],\"outputs\":[{\"name\":\"\",\"type\":\"bytes32\",\"internalType\":\"bytes32\"}],\"stateMutability\":\"view\"},{\"type\":\"function\",\"name\":\"renounceOwnership\",\"inputs\":[],\"outputs\":[],\"stateMutability\":\"nonpayable\"},{\"type\":\"function\",\"name\":\"schedulePieceDeletions\",\"inputs\":[{\"name\":\"setId\",\"type\":\"uint256\",\"internalType\":\"uint256\"},{\"name\":\"pieceIds\",\"type\":\"uint256[]\",\"internalType\":\"uint256[]\"},{\"name\":\"extraData\",\"type\":\"bytes\",\"internalType\":\"bytes\"}],\"outputs\":[],\"stateMutability\":\"nonpayable\"},{\"type\":\"function\",\"name\":\"transferOwnership\",\"inputs\":[{\"name\":\"newOwner\",\"type\":\"address\",\"internalType\":\"address\"}],\"outputs\":[],\"stateMutability\":\"nonpayable\"},{\"type\":\"function\",\"name\":\"updateProofFee\",\"inputs\":[{\"name\":\"newFeePerTiB\",\"type\":\"uint256\",\"internalType\":\"uint256\"}],\"outputs\":[],\"stateMutability\":\"nonpayable\"},{\"type\":\"function\",\"name\":\"upgradeToAndCall\",\"inputs\":[{\"name\":\"newImplementation\",\"type\":\"address\",\"internalType\":\"address\"},{\"name\":\"data\",\"type\":\"bytes\",\"internalType\":\"bytes\"}],\"outputs\":[],\"stateMutability\":\"payable\"},{\"type\":\"event\",\"name\":\"ContractUpgraded\",\"inputs\":[{\"name\":\"version\",\"type\":\"string\",\"indexed\":false,\"internalType\":\"string\"},{\"name\":\"implementation\",\"type\":\"address\",\"indexed\":false,\"internalType\":\"address\"}],\"anonymous\":false},{\"type\":\"event\",\"name\":\"DataSetCreated\",\"inputs\":[{\"name\":\"setId\",\"type\":\"uint256\",\"indexed\":true,\"internalType\":\"uint256\"},{\"name\":\"storageProvider\",\"type\":\"address\",\"indexed\":true,\"internalType\":\"address\"}],\"anonymous\":false},{\"type\":\"event\",\"name\":\"DataSetDeleted\",\"inputs\":[{\"name\":\"setId\",\"type\":\"uint256\",\"indexed\":true,\"internalType\":\"uint256\"},{\"name\":\"deletedLeafCount\",\"type\":\"uint256\",\"indexed\":false,\"internalType\":\"uint256\"}],\"anonymous\":false},{\"type\":\"event\",\"name\":\"DataSetEmpty\",\"inputs\":[{\"name\":\"setId\",\"type\":\"uint256\",\"indexed\":true,\"internalType\":\"uint256\"}],\"anonymous\":false},{\"type\":\"event\",\"name\":\"FeeUpdateProposed\",\"inputs\":[{\"name\":\"currentFee\",\"type\":\"uint256\",\"indexed\":false,\"internalType\":\"uint256\"},{\"name\":\"newFee\",\"type\":\"uint256\",\"indexed\":false,\"internalType\":\"uint256\"},{\"name\":\"effectiveTime\",\"type\":\"uint256\",\"indexed\":false,\"internalType\":\"uint256\"}],\"anonymous\":false},{\"type\":\"event\",\"name\":\"Initialized\",\"inputs\":[{\"name\":\"version\",\"type\":\"uint64\",\"indexed\":false,\"internalType\":\"uint64\"}],\"anonymous\":false},{\"type\":\"event\",\"name\":\"NextProvingPeriod\",\"inputs\":[{\"name\":\"setId\",\"type\":\"uint256\",\"indexed\":true,\"internalType\":\"uint256\"},{\"name\":\"challengeEpoch\",\"type\":\"uint256\",\"indexed\":false,\"internalType\":\"uint256\"},{\"name\":\"leafCount\",\"type\":\"uint256\",\"indexed\":false,\"internalType\":\"uint256\"}],\"anonymous\":false},{\"type\":\"event\",\"name\":\"OwnershipTransferred\",\"inputs\":[{\"name\":\"previousOwner\",\"type\":\"address\",\"indexed\":true,\"internalType\":\"address\"},{\"name\":\"newOwner\",\"type\":\"address\",\"indexed\":true,\"internalType\":\"address\"}],\"anonymous\":false},{\"type\":\"event\",\"name\":\"PiecesAdded\",\"inputs\":[{\"name\":\"setId\",\"type\":\"uint256\",\"indexed\":true,\"internalType\":\"uint256\"},{\"name\":\"pieceIds\",\"type\":\"uint256[]\",\"indexed\":false,\"internalType\":\"uint256[]\"},{\"name\":\"pieceCids\",\"type\":\"tuple[]\",\"indexed\":false,\"internalType\":\"structCids.Cid[]\",\"components\":[{\"name\":\"data\",\"type\":\"bytes\",\"internalType\":\"bytes\"}]}],\"anonymous\":false},{\"type\":\"event\",\"name\":\"PiecesRemoved\",\"inputs\":[{\"name\":\"setId\",\"type\":\"uint256\",\"indexed\":true,\"internalType\":\"uint256\"},{\"name\":\"pieceIds\",\"type\":\"uint256[]\",\"indexed\":false,\"internalType\":\"uint256[]\"}],\"anonymous\":false},{\"type\":\"event\",\"name\":\"PossessionProven\",\"inputs\":[{\"name\":\"setId\",\"type\":\"uint256\",\"indexed\":true,\"internalType\":\"uint256\"},{\"name\":\"challenges\",\"type\":\"tuple[]\",\"indexed\":false,\"internalType\":\"structIPDPTypes.PieceIdAndOffset[]\",\"components\":[{\"name\":\"pieceId\",\"type\":\"uint256\",\"internalType\":\"uint256\"},{\"name\":\"offset\",\"type\":\"uint256\",\"internalType\":\"uint256\"}]}],\"anonymous\":false},{\"type\":\"event\",\"name\":\"ProofFeePaid\",\"inputs\":[{\"name\":\"setId\",\"type\":\"uint256\",\"indexed\":true,\"internalType\":\"uint256\"},{\"name\":\"fee\",\"type\":\"uint256\",\"indexed\":false,\"internalType\":\"uint256\"}],\"anonymous\":false},{\"type\":\"event\",\"name\":\"StorageProviderChanged\",\"inputs\":[{\"name\":\"setId\",\"type\":\"uint256\",\"indexed\":true,\"internalType\":\"uint256\"},{\"name\":\"oldStorageProvider\",\"type\":\"address\",\"indexed\":true,\"internalType\":\"address\"},{\"name\":\"newStorageProvider\",\"type\":\"address\",\"indexed\":true,\"internalType\":\"address\"}],\"anonymous\":false},{\"type\":\"event\",\"name\":\"UpgradeAnnounced\",\"inputs\":[{\"name\":\"plannedUpgrade\",\"type\":\"tuple\",\"indexed\":false,\"internalType\":\"structProofVerifier.PlannedUpgrade\",\"components\":[{\"name\":\"nextImplementation\",\"type\":\"address\",\"internalType\":\"address\"},{\"name\":\"afterEpoch\",\"type\":\"uint96\",\"internalType\":\"uint96\"}]}],\"anonymous\":false},{\"type\":\"event\",\"name\":\"Upgraded\",\"inputs\":[{\"name\":\"implementation\",\"type\":\"address\",\"indexed\":true,\"internalType\":\"address\"}],\"anonymous\":false},{\"type\":\"error\",\"name\":\"AddressEmptyCode\",\"inputs\":[{\"name\":\"target\",\"type\":\"address\",\"internalType\":\"address\"}]},{\"type\":\"error\",\"name\":\"ERC1967InvalidImplementation\",\"inputs\":[{\"name\":\"implementation\",\"type\":\"address\",\"internalType\":\"address\"}]},{\"type\":\"error\",\"name\":\"ERC1967NonPayable\",\"inputs\":[]},{\"type\":\"error\",\"name\":\"FailedCall\",\"inputs\":[]},{\"type\":\"error\",\"name\":\"FilRefundFailed\",\"inputs\":[]},{\"type\":\"error\",\"name\":\"IndexedError\",\"inputs\":[{\"name\":\"idx\",\"type\":\"uint256\",\"internalType\":\"uint256\"},{\"name\":\"msg\",\"type\":\"string\",\"internalType\":\"string\"}]},{\"type\":\"error\",\"name\":\"InvalidInitialization\",\"inputs\":[]},{\"type\":\"error\",\"name\":\"NotInitializing\",\"inputs\":[]},{\"type\":\"error\",\"name\":\"OwnableInvalidOwner\",\"inputs\":[{\"name\":\"owner\",\"type\":\"address\",\"internalType\":\"address\"}]},{\"type\":\"error\",\"name\":\"OwnableUnauthorizedAccount\",\"inputs\":[{\"name\":\"account\",\"type\":\"address\",\"internalType\":\"address\"}]},{\"type\":\"error\",\"name\":\"UUPSUnauthorizedCallContext\",\"inputs\":[]},{\"type\":\"error\",\"name\":\"UUPSUnsupportedProxiableUUID\",\"inputs\":[{\"name\":\"slot\",\"type\":\"bytes32\",\"internalType\":\"bytes32\"}]},{\"type\":\"error\",\"name\":\"UsdfcSybilFeeNotMet\",\"inputs\":[]}]",
	Bin: "0x610120346101db57601f614e8a38819003918201601f19168301916001600160401b038311848410176101df578084926080946040528339810103126101db578051906001600160401b03821682036101db5761005e602082016101f3565b61006f6060604084015193016101f3565b92306080525f516020614e6a5f395f51905f525460ff8160401c166101cc576002600160401b03196001600160401b03821601610176575b5082156101225760a05260c05260e05261010052604051614c62908161020882396080518181816118c40152613c2c015260a05181611267015260c051818181610ef001528181614529015261459e015260e05181818161109d01528181611abc01526133f701526101005181818161232c01526144d80152f35b60405162461bcd60e51b815260206004820152602660248201527f555344464320737962696c20666565206d75737420626520677265617465722060448201526507468616e20360d41b6064820152608490fd5b6001600160401b0319166001600160401b039081175f516020614e6a5f395f51905f52556040519081527fc7f505b2f371ae2175ee4913f4499e1f2633a7b5936321eed1cdaeb6115181d290602090a15f6100a7565b63f92ee8a960e01b5f5260045ffd5b5f80fd5b634e487b7160e01b5f52604160045260245ffd5b51906001600160a01b03821682036101db5756fe60e06040526004361015610011575f80fd5b5f5f3560e01c806304595c1a146129d95780630c292024146125c15780630cd7b880146125885780631a2712251461256e5780631c5ae80f1461253857806321b7cd1c146124dd57806322ef3f73146124b057806325bbbedf1461247a578063264b14cc146124555780632b3129bb14612416578063315e49ea146123e4578063349c91791461235b5780633684528d1461231657806339f51544146122fb57806343186080146121e6578063442cded3146121bd578063453f4f621461219e57806345c0b92d14611bb6578063462dd44914610baf57806346bf7ed314611adf5780634db5e17714611aa45780634f1ef2861461191857806352d1902d146118b15780635353bdfd146118355780636a4991a1146117045780636ba4608f146116ce5780636fa4469214611652578063715018a6146115fc5780637a1e29901461141657806386981308146113d657806389208ba9146113a05780638da5cb5b1461136b5780638fd3ab8014611244578063996ad96a146112235780639afd37f2146111bb5780639f8cb3bd1461119e5780639fbc9f1b14611174578063a531998c1461113e578063ad3cb1cc146110f6578063ba74d94c146110c9578063bbae41cb14611032578063bd00382714610f48578063ca759f2714610f1f578063cc45769014610eda578063dc63526614610da0578063df0f324814610bdb578063e9a31a5514610bb4578063f178b1be14610baf578063f2fde38b14610b82578063f58f952b14610442578063f83758fe14610425578063f8eb827614610409578063fe4b84df146102a25763ffa1ad741461026c575f80fd5b3461029f578060031936011261029f5761029b610287612d54565b604051918291602083526020830190612a9c565b0390f35b80fd5b503461029f57602036600319011261029f575f516020614c0d5f395f51905f5254604081901c60ff1615906001600160401b03811680159081610401575b60011490816103f7575b1590816103ee575b506103df576001600160401b031981166001175f516020614c0d5f395f51905f5255816103b7575b50610323614891565b61032b614891565b6103343361383a565b61033c614891565b6004358255600180546001600160401b03191681179055600f8054600160601b600160c01b031916640689786263606d1b1790556103775780f35b60ff60401b195f516020614c0d5f395f51905f5254165f516020614c0d5f395f51905f52555f516020614bad5f395f51905f52602060405160018152a180f35b6001600160481b0319166001600160401b01175f516020614c0d5f395f51905f52555f61031a565b63f92ee8a960e01b8352600483fd5b9050155f6102f2565b303b1591506102ea565b8391506102e0565b503461029f578060031936011261029f57602060405160328152f35b503461029f578060031936011261029f5760209054604051908152f35b5061044c36612ac0565b909192838152600c60205260018060a01b036040822054163303610b26578115610af35783815260076020526040812054804310610abc5715610a7e5761049282612fd5565b9184825260076020526104a860408320546132d7565b90858352600960205260408320549580845260056020526104cc6040852054613969565b610100036101008111610a6a579194958493919794966040945b6001600160401b038916881115610903578551602081018581528188018c905260c08b901b6001600160c01b0319166060830152604882529061052a606882612c93565b51902087156108ef57858861054092068c613a65565b6105536001600160401b038b1684613030565b526105676001600160401b038a1683613030565b5061058661057e6001600160401b038b1684613030565b51518b612f67565b9761059660208a515110156147ac565b8651986105a3888b612c93565b6020808b528a0199601f198901368c37835b6020811061089857505199519960208110610883575b506105db602182515110156147ac565b51805180601f1981011161086f57601f19810160201982011161086f576001916106099160201901906147f0565b5160f81c019a60ff8c1161085b5761062b6001600160401b038c1683876138b0565b60a08190526020810135903603601e19018112156108575760a0516001600160401b0390820135116108575760a0518101803560051b3603602090910113610857579a9b999a98998c999060206106a061068f6001600160401b038e16878b6138b0565b359c6001600160401b031688613030565b510151916106c26106b58260a0510135612fbe565b8c51608052608051612c93565b608051508060a0510135608051526020608051013660208360a051013560051b8460a051010101116108535760208260a0510101905b60a0518301803560051b0160200182106108435750505060ff16608051515f19820191821161082f57036107db579298929689935b608051518b1015610776576001906107478c608051613030565b51908a8316610767579061075a91614948565b985b811c9a01999761072d565b61077091614948565b9861075c565b91969c939950949a919650989298036107a05761079290613898565b9791939690959892986104e6565b855162461bcd60e51b815260206004820152601460248201527370726f6f6620646964206e6f742076657269667960601b6044820152606490fd5b885162461bcd60e51b815260206004820152602760248201527f70726f6f66206c656e67746820646f6573206e6f74206d617463682074726565604482015266081a195a59da1d60ca1b6064820152608490fd5b634e487b7160e01b86526011600452602486fd5b81358152602091820191016106f8565b8680fd5b8380fd5b634e487b7160e01b83526011600452602483fd5b634e487b7160e01b84526011600452602484fd5b5f9a919a199060200360031b1b16985f6105cb565b82518051601f198101919082116108db57906108c683926108c060019560ff60f81b94612dd9565b906147f0565b5116861a6108d482856147f0565b53016105b5565b634e487b7160e01b87526011600452602487fd5b634e487b7160e01b82526012600452602482fd5b838a83888b83865260096020528186205460018060fb1b03811681036108db579061096d610934889360051b6137aa565b61093d81614801565b867f58b7742b13c8873fc0ba58f695b33ca0044b2db7ff9c5208181dbaec2a5b291e60208751848152a234612fb1565b95858352600e6020524384842055858352600860205260018060a01b03848420541690816109f3575b505050507f1acf7df9f0c1b0208c23be6178950c0273f89b766805a2c0bd1e53d25c700e50916109ca915191829182612af3565b0390a281816109d65780f35b8080806109ee94335af16109e86138d2565b50613901565b818180f35b86845260066020528484205491803b15610a5c578492836084928851968795869463356de02b60e01b86528d60048701526024860152604485015260648401525af18015610a6057610a47575b8080610996565b81610a5191612c93565b610a5c578486610a40565b8480fd5b83513d84823e3d90fd5b634e487b7160e01b85526011600452602485fd5b60405162461bcd60e51b81526020600482015260166024820152751b9bc818da185b1b195b99d9481cd8da19591d5b195960521b6044820152606490fd5b60405162461bcd60e51b815260206004820152600f60248201526e383932b6b0ba3ab93290383937b7b360891b6044820152606490fd5b60405162461bcd60e51b815260206004820152600b60248201526a32b6b83a3c90383937b7b360a91b6044820152606490fd5b60405162461bcd60e51b815260206004820152602e60248201527f4f6e6c79207468652073746f726167652070726f76696465722063616e20707260448201526d37bb32903837b9b9b2b9b9b4b7b760911b6064820152608490fd5b503461029f57602036600319011261029f57610bac610b9f612c2d565b610ba7613bef565b61383a565b80f35b612c43565b503461029f57602036600319011261029f576020610bd36004356137aa565b604051908152f35b503461029f57610bea36612d25565b610bfb610bf684613774565b612d77565b828452600d60205260408420546001600160a01b03163303610d2a57828452600c60208181526040808720548688529282528087208054336001600160a01b03199182168117909255878952600d909352818820805490931690925551869590946001600160a01b03909316939092909184847f686146a80f2bf4dc855942926481871515b39b508826d7982a2e0212d20552c98980a4828652600860205260408620546001600160a01b03169182610cb2578680f35b823b156108535785610cf281959389979388948496634059b6d760e01b865260048601526024850152336044850152608060648501526084840191612e1f565b03925af18015610d1f57610d0a575b80808080808680f35b81610d1491612c93565b61029f57805f610d01565b6040513d84823e3d90fd5b60405162461bcd60e51b815260206004820152604260248201527f4f6e6c79207468652070726f706f7365642073746f726167652070726f76696460448201527f65722063616e20636c61696d2073746f726167652070726f766964657220726f6064820152616c6560f01b608482015260a490fd5b503461029f57610daf36612a6c565b81839293526005602052610dc66040832054613969565b61010003610100811161085b57838352600960205260408320545f19810190811161086f5790610df69185613a65565b916020830151848252600360205260408220845183526020526040822054915f198301928311610ec6575003610e5157610e3281602094612e3f565b9182610e45575b50506040519015158152f35b51101590505f80610e39565b60405162461bcd60e51b815260206004820152604160248201527f6368616c6c656e676552616e6765202d312073686f756c6420616c69676e207760448201527f697468207468652076657279206c617374206c656166206f66206120706965636064820152606560f81b608482015260a490fd5b634e487b7160e01b81526011600452602490fd5b503461029f578060031936011261029f576040517f00000000000000000000000000000000000000000000000000000000000000006001600160a01b03168152602090f35b503461029f57602036600319011261029f576020610f3e600435613774565b6040519015158152f35b503461029f57604036600319011261029f57610f62613bef565b610bb8610f6d613748565b3b111561029f57610f7c61375e565b436001600160601b03909116111561029f576001600160a01b03610f9e613748565b1660018060a01b03196010541617601055610fb761375e565b601080546001600160a01b0390811660a09390931b6001600160a01b03191692909217905560405190610fe8612c2d565b1681526024356001600160601b0381169081900361102e578160409160207fbcf8666408d712c75c2cbd790925afbec6495ca9e04186b1182902260a1d53cd940152a180f35b8280fd5b50604036600319011261029f57611047612c2d565b90602435906001600160401b03821161029f576020611085610bd361108b866110733660048901612a3f565b949061107d6144d6565b953691612cd1565b906145cc565b916110946144d6565b6110c2341515927f000000000000000000000000000000000000000000000000000000000000000090612dd9565b11156146f0565b503461029f578060031936011261029f57600f5460405160609190911c6001600160601b03168152602090f35b503461029f578060031936011261029f5761029b604051611118604082612c93565b60058152640352e302e360dc1b6020820152604051918291602083526020830190612a9c565b503461029f57602036600319011261029f576040602091600435611164610bf682613774565b8152600683522054604051908152f35b503461029f5761029b61118f61118936612b38565b916135cf565b60409391935193849384612be5565b503461029f578060031936011261029f5760206040516107d08152f35b50608036600319011261029f576111d0612c17565b6044356001600160401b03811161102e576111ef903690600401612a0f565b9092606435906001600160401b03821161029f576020610bd38686866112183660048901612a3f565b939092600435613364565b503461029f578060031936011261029f576020600f5460c01c604051908152f35b503461029f578060031936011261029f5761125d613c22565b611265613bef565b7f00000000000000000000000000000000000000000000000000000000000000005f516020614c0d5f395f51905f52549060ff8260401c168015611355575b6103df575f516020614bad5f395f51905f52916020916001600160401b0316907f2b51ff7c4cc8e6fe1c72e9d9685b7d2a88a5d82ad3a644afbdceb0272c89c1c361131b6112f0612d54565b60018060a01b035f516020614bcd5f395f51905f525416604051928392604084526040840190612a9c565b90878301520390a16001600160481b0319168117600160401b1760ff60401b19165f516020614c0d5f395f51905f5255604051908152a180f35b506001600160401b0381811690831610156112a4565b503461029f578060031936011261029f575f516020614b8d5f395f51905f52546040516001600160a01b039091168152602090f35b503461029f57602036600319011261029f5760406020916004356113c6610bf682613774565b8152600983522054604051908152f35b503461029f57602036600319011261029f57600435815260096020526040812054906001600160fb1b0382168203610ec6576020610bd38360051b6137aa565b503461029f5761142536612d25565b6001549091906001600160401b03168310156115bb57828452600c60205260408420546001600160a01b0316330361155f57839083825260066020526040822054928483526006602052826040812055848352600c6020526040832060018060a01b031981541690558483526007602052826040812055848352600e602052826040812055848352600860205260018060a01b0360408420541691826114f6575b83867f14eeeef7679fcb051c6572811f61c07bedccd0f1cfc1f9b79b23e47c5c52aeb7602088604051908152a280f35b823b156108575761153492849283604051809681958294630aaf519760e21b84528c60048501528b6024850152606060448501526064840191612e1f565b03925af18015610d1f5761154a575b80806114c6565b8161155491612c93565b61102e57825f611543565b60405162461bcd60e51b815260206004820152602e60248201527f4f6e6c79207468652073746f726167652070726f76696465722063616e20646560448201526d6c6574652064617461207365747360901b6064820152608490fd5b60405162461bcd60e51b81526020600482015260196024820152786461746120736574206964206f7574206f6620626f756e647360381b6044820152606490fd5b503461029f578060031936011261029f57611615613bef565b5f516020614b8d5f395f51905f5280546001600160a01b0319811690915581906001600160a01b03165f516020614bed5f395f51905f528280a380f35b503461029f57602036600319011261029f57600435611673610bf682613774565b8152600a6020526040812080549061168a82613127565b925b8281106116a9576040516020808252819061029b90820187612bb2565b806116b660019284612e0a565b90549060031b1c6116c78287613030565b520161168c565b503461029f57602036600319011261029f5760406020916004356116f4610bf682613774565b8152600783522054604051908152f35b503461029f57608036600319011261029f57600435906024356001600160401b038111611831576020816004019160031990360301126118315761176c61176560643592611754610bf687613774565b61175f841515613044565b80613332565b3691612cd1565b60208151910120838352600560205260408320549261178a83613127565b9481936044355b86811080611828575b1561180f576117f0908385526003602052604085208186526020526040852054156117eb57838552600260205260408520818652602052856117de60408720612ec7565b60208151910120146117f5575b613159565b611791565b8061180961180289613159565b988b613030565b52613159565b8588526040516020808252819061029b9082018b612bb2565b5081861061179a565b5080fd5b503461029f57602036600319011261029f5760043581611857610bf683613774565b8181526005602052604081205491815b83811061187957602085604051908152f35b818352600360205260408320818452602052604083205461189d575b600101611867565b936118a9600191613159565b949050611895565b503461029f578060031936011261029f577f00000000000000000000000000000000000000000000000000000000000000006001600160a01b031630036119095760206040515f516020614bcd5f395f51905f528152f35b63703e46dd60e11b8152600490fd5b50604036600319011261029f5761192d612c2d565b906024356001600160401b0381116118315761194d903690600401612d07565b91611956613c22565b61195e613bef565b6010546001600160a01b03808316929190811683036108575760a01c431061102e5760108390556040516352d1902d60e01b815293602085600481865afa80958596611a70575b506119be57634c9c8ce360e01b84526004839052602484fd5b9091845f516020614bcd5f395f51905f528103611a5e5750823b15611a4c575f516020614bcd5f395f51905f5280546001600160a01b031916821790557fbc7cd75a20ee27fd9adebab32041f755214dbc6bffa90cc0225b39da2e5c2d3b8480a2805115611a3357611a2f916148bc565b5080f35b505034611a3d5780f35b63b398979f60e01b8152600490fd5b634c9c8ce360e01b8452600452602483fd5b632a87526960e21b8552600452602484fd5b9095506020813d602011611a9c575b81611a8c60209383612c93565b81010312610a5c5751945f6119a5565b3d9150611a7f565b503461029f578060031936011261029f5760206040517f00000000000000000000000000000000000000000000000000000000000000008152f35b503461029f57602036600319011261029f57600435611afc613bef565b600f548060c01c421015611b92575b50600f549062093a8042019182421161086f57606082811b600160601b600160c01b03166001600160601b0390921691821760c094851b6001600160c01b03191617600f81905560408051938452602084019490945290931c918101919091527f239c396012e4038117d18910fba2aab3452e37696f685a457098e4c4864d8bcb9190a180f35b6001600160601b0319811660609190911c6001600160601b031617600f555f611b0b565b503461029f57606036600319011261029f576004356024356044356001600160401b03811161085757611bed903690600401612a3f565b838552600c60205260408520546001600160a01b031633036121375783855260066020526040852054156120dd57838552600e6020526040852054156120ca575b838552600a60205260408520805480611e3d575b50508385526006602052604085205484865260096020526040862055611c69855443612dd9565b8310611dc257849184835260076020528360408420558483526006602052604083205415611d7c575b848352600860205260408320546001600160a01b031680611cef575b5050507fc099ffec4e3e773644a4d1dda368c46af853a0eeb15babde217f53a657396e1e91836040925260066020528185205482519182526020820152a280f35b85845260076020526040842054908685526006602052604085205491813b15611d78578588611d4d829660405198899788968795632a89faf360e21b8752600487015260248601526044850152608060648501526084840191612e1f565b03925af18015610d1f57611d63575b8080611cae565b81611d6d91612c93565b61102e57825f611d5c565b8580fd5b847f02a8400fc343f45098cb00c3a6ea694174771939a5503f663e0ff6f4eb7c28428480a2848352600e6020528260408120558483526007602052826040812055611c92565b60405162461bcd60e51b815260206004820152604760248201527f6368616c6c656e67652065706f6368206d757374206265206174206c6561737460448201527f206368616c6c656e676546696e616c6974792065706f63687320696e207468656064820152662066757475726560c81b608482015260a490fd5b611e4c81969592939496613127565b92875b82811061208957505050838652600a6020526040862080548782558061206f575b50509290611e80610bf683613774565b8586915b805183101561200c57611e978382613030565b5196848952600360205260408920888a526020526040892054958892868b526005602052611ec860408c2054613969565b61010003986101008a11611ff85793611ee8611ee38c612db7565b614965565b8c8b82111580611fe2575b15611f3d57611f2892916040828c60019452600460205281812085825260205220611f1f8d8254612fb1565b90551b90612dd9565b93611f35611ee386612db7565b949094611ee8565b50505095975095611f9591936001939599898c52600360205260408c20818d526020528b6040812055898c52600260205260408c20908c526020528a60408120611f878154612e8f565b80611fa2575b505050612dd9565b9601919492959095611e84565b601f81118714611fb75750555b8a5f80611f8d565b81835260208320611fd291601f0160051c810190880161331c565b8082528160208120915555611faf565b5089905260056020528c60408120548310611ef3565b634e487b7160e01b8c52601160045260248cfd5b839694939250612065907f6e87df804629ac17804b57ba7abbdfac8bdc36bab504fb8a8801eb313a8ce7b192848a52600660205261204f60408b20918254612fb1565b9055604051918291602083526020830190612bb2565b0390a25f80611c42565b612082918852602088209081019061331c565b5f80611e70565b8061209660019284612e0a565b90549060031b1c806120a88389613030565b52888b52600b60205260408b209060081c8b5260205289604081205501611e4f565b838552600e602052436040862055611c2e565b60405162461bcd60e51b815260206004820152602c60248201527f63616e206f6e6c792073746172742070726f76696e67206f6e6365206c65617660448201526b195cc8185c9948185919195960a21b6064820152608490fd5b60405162461bcd60e51b815260206004820152603960248201527f6f6e6c79207468652073746f726167652070726f76696465722063616e206d6f6044820152781d99481d1bc81b995e1d081c1c9bdd9a5b99c81c195c9a5bd9603a1b6064820152608490fd5b503461029f57602036600319011261029f576020610bd36004356132d7565b503461029f578060031936011261029f576001546040516001600160401b039091168152602090f35b503461029f57604036600319011261029f57600435612203612c17565b61220f610bf683613774565b818352600c60205260408320546001600160a01b0316338103612284576001600160a01b0382160361225857508152600d6020526040812080546001600160a01b031916905580f35b908252600d6020526040822080546001600160a01b0319166001600160a01b0390921691909117905580f35b60a460405162461bcd60e51b815260206004820152604460248201527f4f6e6c79207468652063757272656e742073746f726167652070726f7669646560448201527f722063616e2070726f706f73652061206e65772073746f726167652070726f7660648201526334b232b960e11b6084820152fd5b503461029f5761029b61118f61231036612b38565b91613167565b503461029f578060031936011261029f576040517f00000000000000000000000000000000000000000000000000000000000000006001600160a01b03168152602090f35b503461029f5761236a36612ac0565b909180845260056020526123816040852054613969565b61010003916101008311610a6a5761239881612fd5565b945b8181106123af576040518061029b8882612af3565b806123c8856123c1600194868a612de6565b3586613a65565b6123d28289613030565b526123dd8188613030565b500161239a565b503461029f578060031936011261029f57601054604080516001600160a01b038316815260a09290921c602083015290f35b503461029f57602036600319011261029f5760209060043561243a610bf682613774565b815260088252604060018060a01b0391205416604051908152f35b503461029f578060031936011261029f5750602067016345785d8a0000604051908152f35b503461029f5761029b61249561248f36612a6c565b90612f67565b60405191829160208352516020808401526040830190612a9c565b503461029f578060031936011261029f5760206124cb613940565b6040516001600160601b039091168152f35b503461029f57602036600319011261029f57600435906124ff610bf683613774565b818152600c6020908152604080832054938352600d9091529081902054905191829161029b916001600160a01b03918216911683612a82565b503461029f57602036600319011261029f57604060209160043561255e610bf682613774565b8152600583522054604051908152f35b503461029f576020610f3e61258236612a6c565b90612e3f565b503461029f57604060209161259c36612a6c565b906125a9610bf682613774565b82526003845282822090825283522054604051908152f35b5034612725576060366003190112612725576004356024356001600160401b038111612725576125f5903690600401612a0f565b906044356001600160401b03811161272557612615903690600401612a3f565b90612622610bf686613774565b5f858152600c60205260409020546001600160a01b0316330361297357845f52600a6020526107d061265860405f205486612dd9565b1161290b575f5b84811061272957505f858152600860205260409020546001600160a01b03169182612688578680f35b823b156127255760405163e7954aa760e01b8152600481019690965260606024870152606486018590526001600160fb1b03851161272557856126f08195935f9793608484968a9660051b809183880137850185810382016003190160448701520191612e1f565b03925af1801561271a576127075780808080808680f35b61271391505f90612c93565b5f5f610d01565b6040513d5f823e3d90fd5b5f80fd5b612734818686612de6565b3590865f52600560205260405f20548210156128b157865f52600360205260405f20825f5260205260405f20541561285b578160081c600160ff84161b90885f52600b60205260405f20815f526020528160405f20541661280757885f52600b60205260405f20905f5260205260405f20908154179055865f52600a60205260405f2091825492600160401b8410156127f357836127d89160018096018155612e0a565b819291549060031b91821b915f19901b19161790550161265f565b634e487b7160e01b5f52604160045260245ffd5b60405162461bcd60e51b815260206004820152602660248201527f506965636520494420616c7265616479207363686564756c656420666f722072604482015265195b5bdd985b60d21b6064820152608490fd5b60405162461bcd60e51b815260206004820152602860248201527f43616e206f6e6c79207363686564756c652072656d6f76616c206f66206c6976604482015267652070696563657360c01b6064820152608490fd5b60405162461bcd60e51b815260206004820152602c60248201527f43616e206f6e6c79207363686564756c652072656d6f76616c206f662065786960448201526b7374696e672070696563657360a01b6064820152608490fd5b60405162461bcd60e51b815260206004820152603a60248201527f546f6f206d616e792072656d6f76616c73207761697420666f72206e6578742060448201527970726f76696e6720706572696f6420746f207363686564756c6560301b6064820152608490fd5b60405162461bcd60e51b815260206004820152603860248201527f4f6e6c79207468652073746f726167652070726f76696465722063616e207363604482015277686564756c652072656d6f76616c206f662070696563657360401b6064820152608490fd5b34612725576020366003190112612725576004356129f9610bf682613774565b5f52600e602052602060405f2054604051908152f35b9181601f84011215612725578235916001600160401b038311612725576020808501948460051b01011161272557565b9181601f84011215612725578235916001600160401b038311612725576020838186019501011161272557565b6040906003190112612725576004359060243590565b6001600160a01b0391821681529116602082015260400190565b805180835260209291819084018484015e5f828201840152601f01601f1916010190565b9060406003198301126127255760043591602435906001600160401b03821161272557612aef91600401612a0f565b9091565b60206040818301928281528451809452019201905f5b818110612b165750505090565b8251805185526020908101518186015260409094019390920191600101612b09565b606090600319011261272557600435906024359060443590565b9080602083519182815201916020808360051b8301019401925f915b838310612b7d57505050505090565b9091929394602080612ba3600193601f19868203018752828a5151918181520190612a9c565b97019301930191939290612b6e565b90602080835192838152019201905f5b818110612bcf5750505090565b8251845260209384019390920191600101612bc2565b91612c0f90612c01604093969596606086526060860190612b52565b908482036020860152612bb2565b931515910152565b602435906001600160a01b038216820361272557565b600435906001600160a01b038216820361272557565b34612725575f3660031901126127255760206040515f8152f35b602081019081106001600160401b038211176127f357604052565b604081019081106001600160401b038211176127f357604052565b601f909101601f19168101906001600160401b038211908210176127f357604052565b6001600160401b0381116127f357601f01601f191660200190565b929192612cdd82612cb6565b91612ceb6040519384612c93565b829481845281830111612725578281602093845f960137010152565b9080601f8301121561272557816020612d2293359101612cd1565b90565b9060406003198301126127255760043591602435906001600160401b03821161272557612aef91600401612a3f565b60405190612d63604083612c93565b60058252640332e322e360dc1b6020830152565b15612d7e57565b60405162461bcd60e51b81526020600482015260116024820152704461746120736574206e6f74206c69766560781b6044820152606490fd5b9060018201809211612dc557565b634e487b7160e01b5f52601160045260245ffd5b91908201809211612dc557565b9190811015612df65760051b0190565b634e487b7160e01b5f52603260045260245ffd5b8054821015612df6575f5260205f2001905f90565b908060209392818452848401375f828201840152601f01601f1916010190565b90612e4982613774565b9182612e78575b82612e5a57505090565b9091505f52600360205260405f20905f5260205260405f2054151590565b8092505f52600560205260405f2054811091612e50565b90600182811c92168015612ebd575b6020831014612ea957565b634e487b7160e01b5f52602260045260245ffd5b91607f1691612e9e565b9060405191825f825492612eda84612e8f565b8084529360018116908115612f455750600114612f01575b50612eff92500383612c93565b565b90505f9291925260205f20905f915b818310612f29575050906020612eff928201015f612ef2565b6020919350806001915483858901015201910190918492612f10565b905060209250612eff94915060ff191682840152151560051b8201015f612ef2565b6060604051612f7581612c5d565b52612f82610bf682613774565b5f52600260205260405f20905f5260205260405f20612fac60405191612fa783612c5d565b612ec7565b815290565b91908203918211612dc557565b6001600160401b0381116127f35760051b60200190565b90612fdf82612fbe565b612fec6040519182612c93565b8281528092612ffd601f1991612fbe565b01905f5b82811061300d57505050565b60209060405161301c81612c78565b5f81525f8382015282828501015201613001565b8051821015612df65760209160051b010190565b1561304b57565b60405162461bcd60e51b815260206004820152601c60248201527b04c696d6974206d7573742062652067726561746572207468616e20360241b6044820152606490fd5b6040519061309e602083612c93565b5f80835282815b8281106130b157505050565b6020906040516130c081612c5d565b60608152828285010152016130a5565b906130da82612fbe565b6130e76040519182612c93565b82815280926130f8601f1991612fbe565b01905f5b82811061310857505050565b60209060405161311781612c5d565b60608152828285010152016130fc565b9061313182612fbe565b61313e6040519182612c93565b828152809261314f601f1991612fbe565b0190602036910137565b5f198114612dc55760010190565b905f90613176610bf684613774565b613181841515613044565b825f52600560205260405f205493613198816130d0565b936131a282613127565b955f915f945f5b838110613204575b505050505081155f146131e85750505050506131cb61308f565b90604051916131db602084612c93565b5f83525f36813791905f90565b819592939495105f146131fe5783948181955252565b50919291565b825f52600360205260405f20815f5260205260405f2054613228575b6001016131a9565b9395898983891015806132ce575b1561329b578287611809826132939561328760019861328d978c5f52600260205260405f20875f5260205260405f2061327560405191612fa783612c5d565b81526132818383613030565b52613030565b50613030565b97613159565b949050613220565b5050956132a88683612dd9565b8110156132ba57613293600191613159565b505050505091506001915f808080806131b1565b50878310613236565b43106132e1574490565b60405162461bcd60e51b815260206004820152601360248201527265706f636820696e207468652066757475726560681b6044820152606490fd5b818110613327575050565b5f815560010161331c565b903590601e198136030182121561272557018035906001600160401b0382116127255760200191813603831361272557565b91949092918061347e57508301926040818503126127255780356001600160401b0381116127255784613398918301612d07565b9360208201356001600160401b038111612725576133b69201612d07565b936001600160a01b0383161561342f576133db612d22946133d56144d6565b946145cc565b946133e46144d6565b928061341c575b5050506110c2341515927f000000000000000000000000000000000000000000000000000000000000000090612dd9565b6134269287613cee565b505f80806133eb565b60405162461bcd60e51b815260206004820152602160248201527f6c697374656e657220726571756972656420666f72206e6577206461746173656044820152601d60fa1b6064820152608490fd5b939491926001600160a01b03166135775734613528576134a0610bf685613774565b5f848152600c60205260409020546001600160a01b031633036134d257612d22946134cc913691612cd1565b92613cee565b60405162461bcd60e51b815260206004820152602860248201527f4f6e6c79207468652073746f726167652070726f76696465722063616e206164604482015267642070696563657360c01b6064820152608490fd5b60405162461bcd60e51b815260206004820152602160248201527f6e6f20666565206f6e2061646420746f206578697374696e67206461746173656044820152601d60fa1b6064820152608490fd5b60405162461bcd60e51b815260206004820152602a60248201527f6c697374656e6572206d757374206265207a65726f20666f72206578697374696044820152691b99c819185d185cd95d60b21b6064820152608490fd5b9291905f6135df610bf686613774565b6135ea831515613044565b845f52600560205260405f2054938483101561373a57613609846130d0565b9161361385613127565b945f945b87811080613731575b156136a857885f52600360205260405f20815f5260205260405f205461364f575b61364a90613159565b613617565b946136a061364a918a5f52600260205260405f20885f5260205260405f2061367d60405191612fa783612c5d565b81526136898289613030565b526136948188613030565b5087611809828b613030565b959050613641565b509196909392949583159182156136ce575b5050156131e85750505050506131cb61308f565b5f198501858111612dc5576136e39089613030565b5160018101809111612dc5575b8281106136fe575b506136ba565b815f52600360205260405f20815f5260205260405f2054613721576001016136f0565b50505092506001925f80806136f8565b50818610613620565b9450505050506131cb61308f565b6004356001600160a01b03811681036127255790565b6024356001600160601b03811681036127255790565b6001546001600160401b03168110908161378c575090565b5f908152600c60205260409020546001600160a01b03161515919050565b80156137d7576137b8613940565b6001600160601b0316818102918115918304141715612dc55760281c90565b60405162461bcd60e51b815260206004820152603560248201527f6661696c656420746f2076616c69646174653a2070726f6f662073697a65206d60448201527407573742062652067726561746572207468616e203605c1b6064820152608490fd5b6001600160a01b03168015613885575f516020614b8d5f395f51905f5280546001600160a01b0319811683179091556001600160a01b03165f516020614bed5f395f51905f525f80a3565b631e4fbdf760e01b5f525f60045260245ffd5b6001600160401b03908116908114612dc55760010190565b9190811015612df65760051b81013590603e1981360301821215612725570190565b3d156138fc573d906138e382612cb6565b916138f16040519384612c93565b82523d5f602084013e565b606090565b1561390857565b60405162461bcd60e51b815260206004820152601060248201526f2a3930b739b332b9103330b4b632b21760811b6044820152606490fd5b600f5460c081901c421061395d5760601c6001600160601b031690565b6001600160601b031690565b610100908060801c80613a59575b508060401c80613a44575b508060201c80613a2f575b508060101c80613a1a575b508060081c80613a05575b508060041c806139f0575b508060021c806139db575b508060011c6139cb57612d2291612fb1565b506001198101908111612dc55790565b91600119810191508111612dc557905f6139b9565b91600319810191508111612dc557905f6139ae565b91600719810191508111612dc557905f6139a3565b91600f19810191508111612dc557905f613998565b91601f19810191508111612dc557905f61398d565b91603f19810191508111612dc557905f613982565b9150506080905f613977565b91604051613a7281612c78565b5f81525f602082015250825f52600660205260405f2054821015613baf576001811b5f198101908111612dc557925f91805b613b0757505f52600460205260405f20835f52602052613ac860405f205482612dd9565b82811115613af15750613ada91612fb1565b60405191613ae783612c78565b8252602082015290565b905060018301809311612dc557613ada91612fb1565b9391815f52600560205260405f2054831015613b8f57815f52600460205260405f20835f5260205283613b3e60405f205483612dd9565b11613b8f57613b6490825f52600460205260405f20845f5260205260405f205490612dd9565b915f19850190858211612dc5576001613b7e921b90612dd9565b935b8015612dc5575f190180613aa4565b915f19850190858211612dc5576001613ba9921b90612fb1565b93613b80565b60405162461bcd60e51b81526020600482015260186024820152774c65616620696e646578206f7574206f6620626f756e647360401b6044820152606490fd5b5f516020614b8d5f395f51905f52546001600160a01b03163303613c0f57565b63118cdaa760e01b5f523360045260245ffd5b6001600160a01b037f000000000000000000000000000000000000000000000000000000000000000016308114908115613c6d575b50613c5e57565b63703e46dd60e11b5f5260045ffd5b5f516020614bcd5f395f51905f52546001600160a01b0316141590505f613c57565b9190811015612df65760051b81013590601e1981360301821215612725570190565b9190916020818403126127255760405190613ccb82612c5d565b909283919081356001600160401b03811161272557613cea9201612d07565b9052565b9190939293811561449357825f52600560205260405f205494613d1083613127565b613d19846130d0565b5f5b858110613eaa5750613d618692613d6f7f396df50222a87662e94bb7d173792d5e61fe0b193b6ccf791f7ce433f0b2820793604051938493604085526040850190612bb2565b908382036020850152612b52565b0390a25f848152600860205260409020546001600160a01b03169283613d97575b5050505050565b833b156127255760405163f6814d7960e01b81526004810195909552602485018790526080604486015260848501819052849260a4600583901b8501810193929085015f83601e1936829003015b848310613e35575050505050505f83613e0d8296948294600319848303016064850152612a9c565b03925af1801561271a57613e25575b80808080613d90565b5f613e2f91612c93565b5f613e1c565b91939596909294975060a3198a82030185528735828112156127255783018035601e198236030181121561272557016020810190356001600160401b03811161272557803603821361272557613e976020928392838681600198520191612e1f565b9901950193019091899796959392613de5565b919492969593613ebb838583613c8f565b93613ec63686613cb1565b965f5b6004811061442357508751995f908b5160041015612df65760248c015160f81c607f16915b8060040180600411612dc5578d613f0f600160ff1b9260ff60f81b926147f0565b511610613f5757613f1f90613159565b918260040180600411612dc557613f38607f918f6147f0565b5160f81c166007840284810460071485151715612dc5571b1791613eee565b613f6e919b929d9496989a9c509892949698613159565b6004019b8c600411612dc557602281106143c9578c613f8c91612dd9565b8b515103614368578b908b519a5f60c0528b607f613faa85836147f0565b5160f81c16935b60c051600160ff1b926001600160f81b031992613fd1926108c091612dd9565b51161061402857607f613ffa8f6108c08f91613fee60c051613159565b60c05260c05190612dd9565b5160f81c1660c051600760c051020460071460c051151715612dc5578e938d91600760c051021b1793613fb1565b979599939c614063919c939b50614051614059919a96989a61404b60c051613159565b90612dd9565b8094516147f0565b5160f81c92613159565b506001600160f91b0381168103612dc557607f9060071b046005820160ff8111612dc55760ff600191161b81101561432557603282116142cf579060016140ae9260051c911b612fb1565b90885f52600560205260405f20908154916140c883613159565b90556140d6611ee383612db7565b83905f905b8c818310614295579150505f52600460205260405f20835f5260205260405f2055895f52600260205260405f20825f5260205261411c60405f209180613332565b906001600160401b0382116127f3576141358354612e8f565b601f811161425a575b505f90601f83116001146141f05760019695949392915f91836141e5575b50505f19600383901b1c191690861b1790555b895f52600360205260405f20905f526020528060405f2055885f52600660205261419e60405f20918254612dd9565b90556141aa818b612dd9565b6141b48286613030565b526141c9366141c4838a8a613c8f565b613cb1565b6141d38285613030565b526141de8184613030565b5001613d1b565b013590505f8061415c565b601f19831691845f5260205f20925f5b818110614242575091600198979695949291838a959310614229575b505050811b01905561416f565b01355f19600384901b60f8161c191690555f808061421c565b91936020600181928787013581550195019201614200565b61428590845f5260205f20601f850160051c8101916020861061428b575b601f0160051c019061331c565b5f61413e565b9091508190614278565b82936142c7916142aa60018095961b89612fb1565b905f52600460205260405f20905f5260205260405f205490612dd9565b9201906140db565b60a4846040519063c7b67cf360e01b8252600482015260406024820152602160448201527f50696563652073697a65206d757374206265206c657373207468616e20325e356064820152600360fc1b6084820152fd5b6084846040519063c7b67cf360e01b8252600482015260406024820152601460448201527350616464696e6720697320746f6f206c6172676560601b6064820152fd5b60405162461bcd60e51b815260206004820152603360248201527f436f6d6d507632206d756c746968617368206c656e67746820646f6573206e6f6044820152720e840dac2e8c6d040c8c2e8c240d8cadccee8d606b1b6064820152608490fd5b60405162461bcd60e51b815260206004820152602c60248201527f436f6d6d507632206d756c746968617368206c656e677468206d75737420626560448201526b08185d081b19585cdd080ccd60a21b6064820152608490fd5b60ff60f81b614433828b516147f0565b5116620aac8960e51b821a60f81b6001600160f81b0319160361445857600101613ec9565b60405162461bcd60e51b815260206004820152601360248201527221b4b21036bab9ba1031329021b7b6b6a83b1960691b6044820152606490fd5b60405162461bcd60e51b815260206004820152601b60248201527a4d75737420616464206174206c65617374206f6e6520706965636560281b6044820152606490fd5b7f00000000000000000000000000000000000000000000000000000000000000006001600160a01b0381168015801561459b575b6145955761455191608091604051808095819463ad74b77560e01b83527f000000000000000000000000000000000000000000000000000000000000000060048401612a82565b03915afa90811561271a575f91614566575090565b90506080813d60801161458d575b8161458160809383612c93565b81010312612725575190565b3d9150614574565b50505f90565b507f00000000000000000000000000000000000000000000000000000000000000006001600160a01b03161561450a565b6001546001600160401b03808216935f93909290916145ea86613898565b6001600160401b03199092169116176001555f84815260066020908152604080832083905560078252808320839055600c825280832080546001600160a01b031990811633179091556008835281842080549091166001600160a01b03959095169485179055600e90915281205580614689575b5050817f11369440e1b7135015c16acb9bc14b55b0f4b23b02010c363d34aec2e5b96281339280a390565b803b15612725576146c85f9291839260405194858094819363101c1eab60e01b83528a6004840152336024840152606060448401526064830190612a9c565b03925af1801561271a576146dd575b8061465e565b6146e991505f90612c93565b5f5f6146d7565b61478057156147715767016345785d8a00008034106147385761471281614801565b80341161471c5750565b3403348111612dc5575f808080612eff94335af16109e86138d2565b60405162461bcd60e51b81526020600482015260116024820152701cde589a5b08199959481b9bdd081b595d607a1b6044820152606490fd5b6357b1b20f60e11b5f5260045ffd5b503461478857565b5f80808034335af16147986138d2565b50612eff57633b39b94960e21b5f5260045ffd5b156147b357565b60405162461bcd60e51b815260206004820152601560248201527410da590819185d18481a5cc81d1bdbc81cda1bdc9d605a1b6044820152606490fd5b908151811015612df6570160200190565b803410614855575f8080809361dead5af161481a6138d2565b501561482257565b60405162461bcd60e51b815260206004820152600b60248201526a109d5c9b8819985a5b195960aa1b6044820152606490fd5b60405162461bcd60e51b8152602060048201526014602482015273125b98dbdc9c9958dd0819995948185b5bdd5b9d60621b6044820152606490fd5b60ff5f516020614c0d5f395f51905f525460401c16156148ad57565b631afcd79f60e31b5f5260045ffd5b905f8091602081519101845af48080614935575b156148f05750506040513d81523d5f602083013e60203d82010160405290565b1561491557639996b31560e01b5f9081526001600160a01b0391909116600452602490fd5b3d15614926576040513d5f823e3d90fd5b63d6bda27560e01b5f5260045ffd5b503d1515806148d05750813b15156148d0565b5f5260205260205f60408160025afa15612725575f5160c0191690565b6001600160ff1b038111614b3c5761010090600160ff1b8114612dc557805f031680614b33575b6001600160801b038116614b20575b6001600160401b03600160801b03600160c01b038116614b0d575b7bffffffff00000000ffffffff00000000ffffffff00000000ffffffff8116614afa575b7dffff0000ffff0000ffff0000ffff0000ffff0000ffff0000ffff0000ffff8116614ae7575b7eff00ff00ff00ff00ff00ff00ff00ff00ff00ff00ff00ff00ff00ff00ff00ff8116614ad4575b7f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f8116614ac1575b7f33333333333333333333333333333333333333333333333333333333333333338116614aae575b7f555555555555555555555555555555555555555555555555555555555555555516614aa05790565b5f198101908111612dc55790565b906001198101908111612dc55790614a77565b906003198101908111612dc55790614a4f565b906007198101908111612dc55790614a27565b90600f198101908111612dc55790614a00565b90601f198101908111612dc557906149da565b90603f198101908111612dc557906149b6565b90607f198101908111612dc5579061499b565b60ff915061498c565b60405162461bcd60e51b815260206004820152602260248201527f496e7075742065786365656473206d6178696d756d20696e743235362076616c604482015261756560f01b6064820152608490fdfe9016d09d72d40fdae2fd8ceac6b6234c7706214fd39c1cd1e609a0528c199300c7f505b2f371ae2175ee4913f4499e1f2633a7b5936321eed1cdaeb6115181d2360894a13ba1a3210667c828492db98dca3e2076cc3735a920a3ca505d382bbc8be0079c531659141344cd1fd0a4f28419497f9722a3daafe3b4186f6b6457e0f0c57e16840df040f15088dc2f81fe391c3923bec73e23a9662efc9c229c6a00a264697066735822122055c935d427e2624ae79c584c6c1b28b46b44cf8f9c65020c59fde0619c3a8a8864736f6c634300081e0033f0c57e16840df040f15088dc2f81fe391c3923bec73e23a9662efc9c229c6a00",
}

// ProofVerifierABI is the input ABI used to generate the binding from.
// Deprecated: Use ProofVerifierMetaData.ABI instead.
var ProofVerifierABI = ProofVerifierMetaData.ABI

// ProofVerifierBin is the compiled bytecode used for deploying new contracts.
// Deprecated: Use ProofVerifierMetaData.Bin instead.
var ProofVerifierBin = ProofVerifierMetaData.Bin

// DeployProofVerifier deploys a new Ethereum contract, binding an instance of ProofVerifier to it.
func DeployProofVerifier(auth *bind.TransactOpts, backend bind.ContractBackend, _initializerVersion uint64, _usdfcTokenAddress common.Address, _usdfcSybilFee *big.Int, _paymentsContractAddress common.Address) (common.Address, *types.Transaction, *ProofVerifier, error) {
	parsed, err := ProofVerifierMetaData.GetAbi()
	if err != nil {
		return common.Address{}, nil, nil, err
	}
	if parsed == nil {
		return common.Address{}, nil, nil, errors.New("GetABI returned nil")
	}

	address, tx, contract, err := bind.DeployContract(auth, *parsed, common.FromHex(ProofVerifierBin), backend, _initializerVersion, _usdfcTokenAddress, _usdfcSybilFee, _paymentsContractAddress)
	if err != nil {
		return common.Address{}, nil, nil, err
	}
	return address, tx, &ProofVerifier{ProofVerifierCaller: ProofVerifierCaller{contract: contract}, ProofVerifierTransactor: ProofVerifierTransactor{contract: contract}, ProofVerifierFilterer: ProofVerifierFilterer{contract: contract}}, nil
}

// ProofVerifier is an auto generated Go binding around an Ethereum contract.
type ProofVerifier struct {
	ProofVerifierCaller     // Read-only binding to the contract
	ProofVerifierTransactor // Write-only binding to the contract
	ProofVerifierFilterer   // Log filterer for contract events
}

// ProofVerifierCaller is an auto generated read-only Go binding around an Ethereum contract.
type ProofVerifierCaller struct {
	contract *bind.BoundContract // Generic contract wrapper for the low level calls
}

// ProofVerifierTransactor is an auto generated write-only Go binding around an Ethereum contract.
type ProofVerifierTransactor struct {
	contract *bind.BoundContract // Generic contract wrapper for the low level calls
}

// ProofVerifierFilterer is an auto generated log filtering Go binding around an Ethereum contract events.
type ProofVerifierFilterer struct {
	contract *bind.BoundContract // Generic contract wrapper for the low level calls
}

// ProofVerifierSession is an auto generated Go binding around an Ethereum contract,
// with pre-set call and transact options.
type ProofVerifierSession struct {
	Contract     *ProofVerifier    // Generic contract binding to set the session for
	CallOpts     bind.CallOpts     // Call options to use throughout this session
	TransactOpts bind.TransactOpts // Transaction auth options to use throughout this session
}

// ProofVerifierCallerSession is an auto generated read-only Go binding around an Ethereum contract,
// with pre-set call options.
type ProofVerifierCallerSession struct {
	Contract *ProofVerifierCaller // Generic contract caller binding to set the session for
	CallOpts bind.CallOpts        // Call options to use throughout this session
}

// ProofVerifierTransactorSession is an auto generated write-only Go binding around an Ethereum contract,
// with pre-set transact options.
type ProofVerifierTransactorSession struct {
	Contract     *ProofVerifierTransactor // Generic contract transactor binding to set the session for
	TransactOpts bind.TransactOpts        // Transaction auth options to use throughout this session
}

// ProofVerifierRaw is an auto generated low-level Go binding around an Ethereum contract.
type ProofVerifierRaw struct {
	Contract *ProofVerifier // Generic contract binding to access the raw methods on
}

// ProofVerifierCallerRaw is an auto generated low-level read-only Go binding around an Ethereum contract.
type ProofVerifierCallerRaw struct {
	Contract *ProofVerifierCaller // Generic read-only contract binding to access the raw methods on
}

// ProofVerifierTransactorRaw is an auto generated low-level write-only Go binding around an Ethereum contract.
type ProofVerifierTransactorRaw struct {
	Contract *ProofVerifierTransactor // Generic write-only contract binding to access the raw methods on
}

// NewProofVerifier creates a new instance of ProofVerifier, bound to a specific deployed contract.
func NewProofVerifier(address common.Address, backend bind.ContractBackend) (*ProofVerifier, error) {
	contract, err := bindProofVerifier(address, backend, backend, backend)
	if err != nil {
		return nil, err
	}
	return &ProofVerifier{ProofVerifierCaller: ProofVerifierCaller{contract: contract}, ProofVerifierTransactor: ProofVerifierTransactor{contract: contract}, ProofVerifierFilterer: ProofVerifierFilterer{contract: contract}}, nil
}

// NewProofVerifierCaller creates a new read-only instance of ProofVerifier, bound to a specific deployed contract.
func NewProofVerifierCaller(address common.Address, caller bind.ContractCaller) (*ProofVerifierCaller, error) {
	contract, err := bindProofVerifier(address, caller, nil, nil)
	if err != nil {
		return nil, err
	}
	return &ProofVerifierCaller{contract: contract}, nil
}

// NewProofVerifierTransactor creates a new write-only instance of ProofVerifier, bound to a specific deployed contract.
func NewProofVerifierTransactor(address common.Address, transactor bind.ContractTransactor) (*ProofVerifierTransactor, error) {
	contract, err := bindProofVerifier(address, nil, transactor, nil)
	if err != nil {
		return nil, err
	}
	return &ProofVerifierTransactor{contract: contract}, nil
}

// NewProofVerifierFilterer creates a new log filterer instance of ProofVerifier, bound to a specific deployed contract.
func NewProofVerifierFilterer(address common.Address, filterer bind.ContractFilterer) (*ProofVerifierFilterer, error) {
	contract, err := bindProofVerifier(address, nil, nil, filterer)
	if err != nil {
		return nil, err
	}
	return &ProofVerifierFilterer{contract: contract}, nil
}

// bindProofVerifier binds a generic wrapper to an already deployed contract.
func bindProofVerifier(address common.Address, caller bind.ContractCaller, transactor bind.ContractTransactor, filterer bind.ContractFilterer) (*bind.BoundContract, error) {
	parsed, err := ProofVerifierMetaData.GetAbi()
	if err != nil {
		return nil, err
	}
	return bind.NewBoundContract(address, *parsed, caller, transactor, filterer), nil
}

// Call invokes the (constant) contract method with params as input values and
// sets the output to result. The result type might be a single field for simple
// returns, a slice of interfaces for anonymous returns and a struct for named
// returns.
func (_ProofVerifier *ProofVerifierRaw) Call(opts *bind.CallOpts, result *[]interface{}, method string, params ...interface{}) error {
	return _ProofVerifier.Contract.ProofVerifierCaller.contract.Call(opts, result, method, params...)
}

// Transfer initiates a plain transaction to move funds to the contract, calling
// its default method if one is available.
func (_ProofVerifier *ProofVerifierRaw) Transfer(opts *bind.TransactOpts) (*types.Transaction, error) {
	return _ProofVerifier.Contract.ProofVerifierTransactor.contract.Transfer(opts)
}

// Transact invokes the (paid) contract method with params as input values.
func (_ProofVerifier *ProofVerifierRaw) Transact(opts *bind.TransactOpts, method string, params ...interface{}) (*types.Transaction, error) {
	return _ProofVerifier.Contract.ProofVerifierTransactor.contract.Transact(opts, method, params...)
}

// Call invokes the (constant) contract method with params as input values and
// sets the output to result. The result type might be a single field for simple
// returns, a slice of interfaces for anonymous returns and a struct for named
// returns.
func (_ProofVerifier *ProofVerifierCallerRaw) Call(opts *bind.CallOpts, result *[]interface{}, method string, params ...interface{}) error {
	return _ProofVerifier.Contract.contract.Call(opts, result, method, params...)
}

// Transfer initiates a plain transaction to move funds to the contract, calling
// its default method if one is available.
func (_ProofVerifier *ProofVerifierTransactorRaw) Transfer(opts *bind.TransactOpts) (*types.Transaction, error) {
	return _ProofVerifier.Contract.contract.Transfer(opts)
}

// Transact invokes the (paid) contract method with params as input values.
func (_ProofVerifier *ProofVerifierTransactorRaw) Transact(opts *bind.TransactOpts, method string, params ...interface{}) (*types.Transaction, error) {
	return _ProofVerifier.Contract.contract.Transact(opts, method, params...)
}

// FILSYBILFEE is a free data retrieval call binding the contract method 0x264b14cc.
//
// Solidity: function FIL_SYBIL_FEE() pure returns(uint256)
func (_ProofVerifier *ProofVerifierCaller) FILSYBILFEE(opts *bind.CallOpts) (*big.Int, error) {
	var out []interface{}
	err := _ProofVerifier.contract.Call(opts, &out, "FIL_SYBIL_FEE")

	if err != nil {
		return *new(*big.Int), err
	}

	out0 := *abi.ConvertType(out[0], new(*big.Int)).(**big.Int)

	return out0, err

}

// FILSYBILFEE is a free data retrieval call binding the contract method 0x264b14cc.
//
// Solidity: function FIL_SYBIL_FEE() pure returns(uint256)
func (_ProofVerifier *ProofVerifierSession) FILSYBILFEE() (*big.Int, error) {
	return _ProofVerifier.Contract.FILSYBILFEE(&_ProofVerifier.CallOpts)
}

// FILSYBILFEE is a free data retrieval call binding the contract method 0x264b14cc.
//
// Solidity: function FIL_SYBIL_FEE() pure returns(uint256)
func (_ProofVerifier *ProofVerifierCallerSession) FILSYBILFEE() (*big.Int, error) {
	return _ProofVerifier.Contract.FILSYBILFEE(&_ProofVerifier.CallOpts)
}

// MAXENQUEUEDREMOVALS is a free data retrieval call binding the contract method 0x9f8cb3bd.
//
// Solidity: function MAX_ENQUEUED_REMOVALS() view returns(uint256)
func (_ProofVerifier *ProofVerifierCaller) MAXENQUEUEDREMOVALS(opts *bind.CallOpts) (*big.Int, error) {
	var out []interface{}
	err := _ProofVerifier.contract.Call(opts, &out, "MAX_ENQUEUED_REMOVALS")

	if err != nil {
		return *new(*big.Int), err
	}

	out0 := *abi.ConvertType(out[0], new(*big.Int)).(**big.Int)

	return out0, err

}

// MAXENQUEUEDREMOVALS is a free data retrieval call binding the contract method 0x9f8cb3bd.
//
// Solidity: function MAX_ENQUEUED_REMOVALS() view returns(uint256)
func (_ProofVerifier *ProofVerifierSession) MAXENQUEUEDREMOVALS() (*big.Int, error) {
	return _ProofVerifier.Contract.MAXENQUEUEDREMOVALS(&_ProofVerifier.CallOpts)
}

// MAXENQUEUEDREMOVALS is a free data retrieval call binding the contract method 0x9f8cb3bd.
//
// Solidity: function MAX_ENQUEUED_REMOVALS() view returns(uint256)
func (_ProofVerifier *ProofVerifierCallerSession) MAXENQUEUEDREMOVALS() (*big.Int, error) {
	return _ProofVerifier.Contract.MAXENQUEUEDREMOVALS(&_ProofVerifier.CallOpts)
}

// MAXPIECESIZELOG2 is a free data retrieval call binding the contract method 0xf8eb8276.
//
// Solidity: function MAX_PIECE_SIZE_LOG2() view returns(uint256)
func (_ProofVerifier *ProofVerifierCaller) MAXPIECESIZELOG2(opts *bind.CallOpts) (*big.Int, error) {
	var out []interface{}
	err := _ProofVerifier.contract.Call(opts, &out, "MAX_PIECE_SIZE_LOG2")

	if err != nil {
		return *new(*big.Int), err
	}

	out0 := *abi.ConvertType(out[0], new(*big.Int)).(**big.Int)

	return out0, err

}

// MAXPIECESIZELOG2 is a free data retrieval call binding the contract method 0xf8eb8276.
//
// Solidity: function MAX_PIECE_SIZE_LOG2() view returns(uint256)
func (_ProofVerifier *ProofVerifierSession) MAXPIECESIZELOG2() (*big.Int, error) {
	return _ProofVerifier.Contract.MAXPIECESIZELOG2(&_ProofVerifier.CallOpts)
}

// MAXPIECESIZELOG2 is a free data retrieval call binding the contract method 0xf8eb8276.
//
// Solidity: function MAX_PIECE_SIZE_LOG2() view returns(uint256)
func (_ProofVerifier *ProofVerifierCallerSession) MAXPIECESIZELOG2() (*big.Int, error) {
	return _ProofVerifier.Contract.MAXPIECESIZELOG2(&_ProofVerifier.CallOpts)
}

// NOCHALLENGESCHEDULED is a free data retrieval call binding the contract method 0x462dd449.
//
// Solidity: function NO_CHALLENGE_SCHEDULED() view returns(uint256)
func (_ProofVerifier *ProofVerifierCaller) NOCHALLENGESCHEDULED(opts *bind.CallOpts) (*big.Int, error) {
	var out []interface{}
	err := _ProofVerifier.contract.Call(opts, &out, "NO_CHALLENGE_SCHEDULED")

	if err != nil {
		return *new(*big.Int), err
	}

	out0 := *abi.ConvertType(out[0], new(*big.Int)).(**big.Int)

	return out0, err

}

// NOCHALLENGESCHEDULED is a free data retrieval call binding the contract method 0x462dd449.
//
// Solidity: function NO_CHALLENGE_SCHEDULED() view returns(uint256)
func (_ProofVerifier *ProofVerifierSession) NOCHALLENGESCHEDULED() (*big.Int, error) {
	return _ProofVerifier.Contract.NOCHALLENGESCHEDULED(&_ProofVerifier.CallOpts)
}

// NOCHALLENGESCHEDULED is a free data retrieval call binding the contract method 0x462dd449.
//
// Solidity: function NO_CHALLENGE_SCHEDULED() view returns(uint256)
func (_ProofVerifier *ProofVerifierCallerSession) NOCHALLENGESCHEDULED() (*big.Int, error) {
	return _ProofVerifier.Contract.NOCHALLENGESCHEDULED(&_ProofVerifier.CallOpts)
}

// NOPROVENEPOCH is a free data retrieval call binding the contract method 0xf178b1be.
//
// Solidity: function NO_PROVEN_EPOCH() view returns(uint256)
func (_ProofVerifier *ProofVerifierCaller) NOPROVENEPOCH(opts *bind.CallOpts) (*big.Int, error) {
	var out []interface{}
	err := _ProofVerifier.contract.Call(opts, &out, "NO_PROVEN_EPOCH")

	if err != nil {
		return *new(*big.Int), err
	}

	out0 := *abi.ConvertType(out[0], new(*big.Int)).(**big.Int)

	return out0, err

}

// NOPROVENEPOCH is a free data retrieval call binding the contract method 0xf178b1be.
//
// Solidity: function NO_PROVEN_EPOCH() view returns(uint256)
func (_ProofVerifier *ProofVerifierSession) NOPROVENEPOCH() (*big.Int, error) {
	return _ProofVerifier.Contract.NOPROVENEPOCH(&_ProofVerifier.CallOpts)
}

// NOPROVENEPOCH is a free data retrieval call binding the contract method 0xf178b1be.
//
// Solidity: function NO_PROVEN_EPOCH() view returns(uint256)
func (_ProofVerifier *ProofVerifierCallerSession) NOPROVENEPOCH() (*big.Int, error) {
	return _ProofVerifier.Contract.NOPROVENEPOCH(&_ProofVerifier.CallOpts)
}

// PAYMENTSCONTRACTADDRESS is a free data retrieval call binding the contract method 0x3684528d.
//
// Solidity: function PAYMENTS_CONTRACT_ADDRESS() view returns(address)
func (_ProofVerifier *ProofVerifierCaller) PAYMENTSCONTRACTADDRESS(opts *bind.CallOpts) (common.Address, error) {
	var out []interface{}
	err := _ProofVerifier.contract.Call(opts, &out, "PAYMENTS_CONTRACT_ADDRESS")

	if err != nil {
		return *new(common.Address), err
	}

	out0 := *abi.ConvertType(out[0], new(common.Address)).(*common.Address)

	return out0, err

}

// PAYMENTSCONTRACTADDRESS is a free data retrieval call binding the contract method 0x3684528d.
//
// Solidity: function PAYMENTS_CONTRACT_ADDRESS() view returns(address)
func (_ProofVerifier *ProofVerifierSession) PAYMENTSCONTRACTADDRESS() (common.Address, error) {
	return _ProofVerifier.Contract.PAYMENTSCONTRACTADDRESS(&_ProofVerifier.CallOpts)
}

// PAYMENTSCONTRACTADDRESS is a free data retrieval call binding the contract method 0x3684528d.
//
// Solidity: function PAYMENTS_CONTRACT_ADDRESS() view returns(address)
func (_ProofVerifier *ProofVerifierCallerSession) PAYMENTSCONTRACTADDRESS() (common.Address, error) {
	return _ProofVerifier.Contract.PAYMENTSCONTRACTADDRESS(&_ProofVerifier.CallOpts)
}

// UPGRADEINTERFACEVERSION is a free data retrieval call binding the contract method 0xad3cb1cc.
//
// Solidity: function UPGRADE_INTERFACE_VERSION() view returns(string)
func (_ProofVerifier *ProofVerifierCaller) UPGRADEINTERFACEVERSION(opts *bind.CallOpts) (string, error) {
	var out []interface{}
	err := _ProofVerifier.contract.Call(opts, &out, "UPGRADE_INTERFACE_VERSION")

	if err != nil {
		return *new(string), err
	}

	out0 := *abi.ConvertType(out[0], new(string)).(*string)

	return out0, err

}

// UPGRADEINTERFACEVERSION is a free data retrieval call binding the contract method 0xad3cb1cc.
//
// Solidity: function UPGRADE_INTERFACE_VERSION() view returns(string)
func (_ProofVerifier *ProofVerifierSession) UPGRADEINTERFACEVERSION() (string, error) {
	return _ProofVerifier.Contract.UPGRADEINTERFACEVERSION(&_ProofVerifier.CallOpts)
}

// UPGRADEINTERFACEVERSION is a free data retrieval call binding the contract method 0xad3cb1cc.
//
// Solidity: function UPGRADE_INTERFACE_VERSION() view returns(string)
func (_ProofVerifier *ProofVerifierCallerSession) UPGRADEINTERFACEVERSION() (string, error) {
	return _ProofVerifier.Contract.UPGRADEINTERFACEVERSION(&_ProofVerifier.CallOpts)
}

// USDFCSYBILFEE is a free data retrieval call binding the contract method 0x4db5e177.
//
// Solidity: function USDFC_SYBIL_FEE() view returns(uint256)
func (_ProofVerifier *ProofVerifierCaller) USDFCSYBILFEE(opts *bind.CallOpts) (*big.Int, error) {
	var out []interface{}
	err := _ProofVerifier.contract.Call(opts, &out, "USDFC_SYBIL_FEE")

	if err != nil {
		return *new(*big.Int), err
	}

	out0 := *abi.ConvertType(out[0], new(*big.Int)).(**big.Int)

	return out0, err

}

// USDFCSYBILFEE is a free data retrieval call binding the contract method 0x4db5e177.
//
// Solidity: function USDFC_SYBIL_FEE() view returns(uint256)
func (_ProofVerifier *ProofVerifierSession) USDFCSYBILFEE() (*big.Int, error) {
	return _ProofVerifier.Contract.USDFCSYBILFEE(&_ProofVerifier.CallOpts)
}

// USDFCSYBILFEE is a free data retrieval call binding the contract method 0x4db5e177.
//
// Solidity: function USDFC_SYBIL_FEE() view returns(uint256)
func (_ProofVerifier *ProofVerifierCallerSession) USDFCSYBILFEE() (*big.Int, error) {
	return _ProofVerifier.Contract.USDFCSYBILFEE(&_ProofVerifier.CallOpts)
}

// USDFCTOKENADDRESS is a free data retrieval call binding the contract method 0xcc457690.
//
// Solidity: function USDFC_TOKEN_ADDRESS() view returns(address)
func (_ProofVerifier *ProofVerifierCaller) USDFCTOKENADDRESS(opts *bind.CallOpts) (common.Address, error) {
	var out []interface{}
	err := _ProofVerifier.contract.Call(opts, &out, "USDFC_TOKEN_ADDRESS")

	if err != nil {
		return *new(common.Address), err
	}

	out0 := *abi.ConvertType(out[0], new(common.Address)).(*common.Address)

	return out0, err

}

// USDFCTOKENADDRESS is a free data retrieval call binding the contract method 0xcc457690.
//
// Solidity: function USDFC_TOKEN_ADDRESS() view returns(address)
func (_ProofVerifier *ProofVerifierSession) USDFCTOKENADDRESS() (common.Address, error) {
	return _ProofVerifier.Contract.USDFCTOKENADDRESS(&_ProofVerifier.CallOpts)
}

// USDFCTOKENADDRESS is a free data retrieval call binding the contract method 0xcc457690.
//
// Solidity: function USDFC_TOKEN_ADDRESS() view returns(address)
func (_ProofVerifier *ProofVerifierCallerSession) USDFCTOKENADDRESS() (common.Address, error) {
	return _ProofVerifier.Contract.USDFCTOKENADDRESS(&_ProofVerifier.CallOpts)
}

// VERSION is a free data retrieval call binding the contract method 0xffa1ad74.
//
// Solidity: function VERSION() view returns(string)
func (_ProofVerifier *ProofVerifierCaller) VERSION(opts *bind.CallOpts) (string, error) {
	var out []interface{}
	err := _ProofVerifier.contract.Call(opts, &out, "VERSION")

	if err != nil {
		return *new(string), err
	}

	out0 := *abi.ConvertType(out[0], new(string)).(*string)

	return out0, err

}

// VERSION is a free data retrieval call binding the contract method 0xffa1ad74.
//
// Solidity: function VERSION() view returns(string)
func (_ProofVerifier *ProofVerifierSession) VERSION() (string, error) {
	return _ProofVerifier.Contract.VERSION(&_ProofVerifier.CallOpts)
}

// VERSION is a free data retrieval call binding the contract method 0xffa1ad74.
//
// Solidity: function VERSION() view returns(string)
func (_ProofVerifier *ProofVerifierCallerSession) VERSION() (string, error) {
	return _ProofVerifier.Contract.VERSION(&_ProofVerifier.CallOpts)
}

// CalculateProofFee is a free data retrieval call binding the contract method 0x86981308.
//
// Solidity: function calculateProofFee(uint256 setId) view returns(uint256)
func (_ProofVerifier *ProofVerifierCaller) CalculateProofFee(opts *bind.CallOpts, setId *big.Int) (*big.Int, error) {
	var out []interface{}
	err := _ProofVerifier.contract.Call(opts, &out, "calculateProofFee", setId)

	if err != nil {
		return *new(*big.Int), err
	}

	out0 := *abi.ConvertType(out[0], new(*big.Int)).(**big.Int)

	return out0, err

}

// CalculateProofFee is a free data retrieval call binding the contract method 0x86981308.
//
// Solidity: function calculateProofFee(uint256 setId) view returns(uint256)
func (_ProofVerifier *ProofVerifierSession) CalculateProofFee(setId *big.Int) (*big.Int, error) {
	return _ProofVerifier.Contract.CalculateProofFee(&_ProofVerifier.CallOpts, setId)
}

// CalculateProofFee is a free data retrieval call binding the contract method 0x86981308.
//
// Solidity: function calculateProofFee(uint256 setId) view returns(uint256)
func (_ProofVerifier *ProofVerifierCallerSession) CalculateProofFee(setId *big.Int) (*big.Int, error) {
	return _ProofVerifier.Contract.CalculateProofFee(&_ProofVerifier.CallOpts, setId)
}

// CalculateProofFeeForSize is a free data retrieval call binding the contract method 0xe9a31a55.
//
// Solidity: function calculateProofFeeForSize(uint256 proofSize) view returns(uint256)
func (_ProofVerifier *ProofVerifierCaller) CalculateProofFeeForSize(opts *bind.CallOpts, proofSize *big.Int) (*big.Int, error) {
	var out []interface{}
	err := _ProofVerifier.contract.Call(opts, &out, "calculateProofFeeForSize", proofSize)

	if err != nil {
		return *new(*big.Int), err
	}

	out0 := *abi.ConvertType(out[0], new(*big.Int)).(**big.Int)

	return out0, err

}

// CalculateProofFeeForSize is a free data retrieval call binding the contract method 0xe9a31a55.
//
// Solidity: function calculateProofFeeForSize(uint256 proofSize) view returns(uint256)
func (_ProofVerifier *ProofVerifierSession) CalculateProofFeeForSize(proofSize *big.Int) (*big.Int, error) {
	return _ProofVerifier.Contract.CalculateProofFeeForSize(&_ProofVerifier.CallOpts, proofSize)
}

// CalculateProofFeeForSize is a free data retrieval call binding the contract method 0xe9a31a55.
//
// Solidity: function calculateProofFeeForSize(uint256 proofSize) view returns(uint256)
func (_ProofVerifier *ProofVerifierCallerSession) CalculateProofFeeForSize(proofSize *big.Int) (*big.Int, error) {
	return _ProofVerifier.Contract.CalculateProofFeeForSize(&_ProofVerifier.CallOpts, proofSize)
}

// DataSetLive is a free data retrieval call binding the contract method 0xca759f27.
//
// Solidity: function dataSetLive(uint256 setId) view returns(bool)
func (_ProofVerifier *ProofVerifierCaller) DataSetLive(opts *bind.CallOpts, setId *big.Int) (bool, error) {
	var out []interface{}
	err := _ProofVerifier.contract.Call(opts, &out, "dataSetLive", setId)

	if err != nil {
		return *new(bool), err
	}

	out0 := *abi.ConvertType(out[0], new(bool)).(*bool)

	return out0, err

}

// DataSetLive is a free data retrieval call binding the contract method 0xca759f27.
//
// Solidity: function dataSetLive(uint256 setId) view returns(bool)
func (_ProofVerifier *ProofVerifierSession) DataSetLive(setId *big.Int) (bool, error) {
	return _ProofVerifier.Contract.DataSetLive(&_ProofVerifier.CallOpts, setId)
}

// DataSetLive is a free data retrieval call binding the contract method 0xca759f27.
//
// Solidity: function dataSetLive(uint256 setId) view returns(bool)
func (_ProofVerifier *ProofVerifierCallerSession) DataSetLive(setId *big.Int) (bool, error) {
	return _ProofVerifier.Contract.DataSetLive(&_ProofVerifier.CallOpts, setId)
}

// FeeEffectiveTime is a free data retrieval call binding the contract method 0x996ad96a.
//
// Solidity: function feeEffectiveTime() view returns(uint64)
func (_ProofVerifier *ProofVerifierCaller) FeeEffectiveTime(opts *bind.CallOpts) (uint64, error) {
	var out []interface{}
	err := _ProofVerifier.contract.Call(opts, &out, "feeEffectiveTime")

	if err != nil {
		return *new(uint64), err
	}

	out0 := *abi.ConvertType(out[0], new(uint64)).(*uint64)

	return out0, err

}

// FeeEffectiveTime is a free data retrieval call binding the contract method 0x996ad96a.
//
// Solidity: function feeEffectiveTime() view returns(uint64)
func (_ProofVerifier *ProofVerifierSession) FeeEffectiveTime() (uint64, error) {
	return _ProofVerifier.Contract.FeeEffectiveTime(&_ProofVerifier.CallOpts)
}

// FeeEffectiveTime is a free data retrieval call binding the contract method 0x996ad96a.
//
// Solidity: function feeEffectiveTime() view returns(uint64)
func (_ProofVerifier *ProofVerifierCallerSession) FeeEffectiveTime() (uint64, error) {
	return _ProofVerifier.Contract.FeeEffectiveTime(&_ProofVerifier.CallOpts)
}

// FeePerTiB is a free data retrieval call binding the contract method 0x22ef3f73.
//
// Solidity: function feePerTiB() view returns(uint96)
func (_ProofVerifier *ProofVerifierCaller) FeePerTiB(opts *bind.CallOpts) (*big.Int, error) {
	var out []interface{}
	err := _ProofVerifier.contract.Call(opts, &out, "feePerTiB")

	if err != nil {
		return *new(*big.Int), err
	}

	out0 := *abi.ConvertType(out[0], new(*big.Int)).(**big.Int)

	return out0, err

}

// FeePerTiB is a free data retrieval call binding the contract method 0x22ef3f73.
//
// Solidity: function feePerTiB() view returns(uint96)
func (_ProofVerifier *ProofVerifierSession) FeePerTiB() (*big.Int, error) {
	return _ProofVerifier.Contract.FeePerTiB(&_ProofVerifier.CallOpts)
}

// FeePerTiB is a free data retrieval call binding the contract method 0x22ef3f73.
//
// Solidity: function feePerTiB() view returns(uint96)
func (_ProofVerifier *ProofVerifierCallerSession) FeePerTiB() (*big.Int, error) {
	return _ProofVerifier.Contract.FeePerTiB(&_ProofVerifier.CallOpts)
}

// FindPieceIds is a free data retrieval call binding the contract method 0x349c9179.
//
// Solidity: function findPieceIds(uint256 setId, uint256[] leafIndexs) view returns((uint256,uint256)[])
func (_ProofVerifier *ProofVerifierCaller) FindPieceIds(opts *bind.CallOpts, setId *big.Int, leafIndexs []*big.Int) ([]IPDPTypesPieceIdAndOffset, error) {
	var out []interface{}
	err := _ProofVerifier.contract.Call(opts, &out, "findPieceIds", setId, leafIndexs)

	if err != nil {
		return *new([]IPDPTypesPieceIdAndOffset), err
	}

	out0 := *abi.ConvertType(out[0], new([]IPDPTypesPieceIdAndOffset)).(*[]IPDPTypesPieceIdAndOffset)

	return out0, err

}

// FindPieceIds is a free data retrieval call binding the contract method 0x349c9179.
//
// Solidity: function findPieceIds(uint256 setId, uint256[] leafIndexs) view returns((uint256,uint256)[])
func (_ProofVerifier *ProofVerifierSession) FindPieceIds(setId *big.Int, leafIndexs []*big.Int) ([]IPDPTypesPieceIdAndOffset, error) {
	return _ProofVerifier.Contract.FindPieceIds(&_ProofVerifier.CallOpts, setId, leafIndexs)
}

// FindPieceIds is a free data retrieval call binding the contract method 0x349c9179.
//
// Solidity: function findPieceIds(uint256 setId, uint256[] leafIndexs) view returns((uint256,uint256)[])
func (_ProofVerifier *ProofVerifierCallerSession) FindPieceIds(setId *big.Int, leafIndexs []*big.Int) ([]IPDPTypesPieceIdAndOffset, error) {
	return _ProofVerifier.Contract.FindPieceIds(&_ProofVerifier.CallOpts, setId, leafIndexs)
}

// FindPieceIdsByCid is a free data retrieval call binding the contract method 0x6a4991a1.
//
// Solidity: function findPieceIdsByCid(uint256 setId, (bytes) pieceCid, uint256 startPieceId, uint256 limit) view returns(uint256[] pieceIds)
func (_ProofVerifier *ProofVerifierCaller) FindPieceIdsByCid(opts *bind.CallOpts, setId *big.Int, pieceCid CidsCid, startPieceId *big.Int, limit *big.Int) ([]*big.Int, error) {
	var out []interface{}
	err := _ProofVerifier.contract.Call(opts, &out, "findPieceIdsByCid", setId, pieceCid, startPieceId, limit)

	if err != nil {
		return *new([]*big.Int), err
	}

	out0 := *abi.ConvertType(out[0], new([]*big.Int)).(*[]*big.Int)

	return out0, err

}

// FindPieceIdsByCid is a free data retrieval call binding the contract method 0x6a4991a1.
//
// Solidity: function findPieceIdsByCid(uint256 setId, (bytes) pieceCid, uint256 startPieceId, uint256 limit) view returns(uint256[] pieceIds)
func (_ProofVerifier *ProofVerifierSession) FindPieceIdsByCid(setId *big.Int, pieceCid CidsCid, startPieceId *big.Int, limit *big.Int) ([]*big.Int, error) {
	return _ProofVerifier.Contract.FindPieceIdsByCid(&_ProofVerifier.CallOpts, setId, pieceCid, startPieceId, limit)
}

// FindPieceIdsByCid is a free data retrieval call binding the contract method 0x6a4991a1.
//
// Solidity: function findPieceIdsByCid(uint256 setId, (bytes) pieceCid, uint256 startPieceId, uint256 limit) view returns(uint256[] pieceIds)
func (_ProofVerifier *ProofVerifierCallerSession) FindPieceIdsByCid(setId *big.Int, pieceCid CidsCid, startPieceId *big.Int, limit *big.Int) ([]*big.Int, error) {
	return _ProofVerifier.Contract.FindPieceIdsByCid(&_ProofVerifier.CallOpts, setId, pieceCid, startPieceId, limit)
}

// GetActivePieceCount is a free data retrieval call binding the contract method 0x5353bdfd.
//
// Solidity: function getActivePieceCount(uint256 setId) view returns(uint256 activeCount)
func (_ProofVerifier *ProofVerifierCaller) GetActivePieceCount(opts *bind.CallOpts, setId *big.Int) (*big.Int, error) {
	var out []interface{}
	err := _ProofVerifier.contract.Call(opts, &out, "getActivePieceCount", setId)

	if err != nil {
		return *new(*big.Int), err
	}

	out0 := *abi.ConvertType(out[0], new(*big.Int)).(**big.Int)

	return out0, err

}

// GetActivePieceCount is a free data retrieval call binding the contract method 0x5353bdfd.
//
// Solidity: function getActivePieceCount(uint256 setId) view returns(uint256 activeCount)
func (_ProofVerifier *ProofVerifierSession) GetActivePieceCount(setId *big.Int) (*big.Int, error) {
	return _ProofVerifier.Contract.GetActivePieceCount(&_ProofVerifier.CallOpts, setId)
}

// GetActivePieceCount is a free data retrieval call binding the contract method 0x5353bdfd.
//
// Solidity: function getActivePieceCount(uint256 setId) view returns(uint256 activeCount)
func (_ProofVerifier *ProofVerifierCallerSession) GetActivePieceCount(setId *big.Int) (*big.Int, error) {
	return _ProofVerifier.Contract.GetActivePieceCount(&_ProofVerifier.CallOpts, setId)
}

// GetActivePieces is a free data retrieval call binding the contract method 0x39f51544.
//
// Solidity: function getActivePieces(uint256 setId, uint256 offset, uint256 limit) view returns((bytes)[] pieces, uint256[] pieceIds, bool hasMore)
func (_ProofVerifier *ProofVerifierCaller) GetActivePieces(opts *bind.CallOpts, setId *big.Int, offset *big.Int, limit *big.Int) (struct {
	Pieces   []CidsCid
	PieceIds []*big.Int
	HasMore  bool
}, error) {
	var out []interface{}
	err := _ProofVerifier.contract.Call(opts, &out, "getActivePieces", setId, offset, limit)

	outstruct := new(struct {
		Pieces   []CidsCid
		PieceIds []*big.Int
		HasMore  bool
	})
	if err != nil {
		return *outstruct, err
	}

	outstruct.Pieces = *abi.ConvertType(out[0], new([]CidsCid)).(*[]CidsCid)
	outstruct.PieceIds = *abi.ConvertType(out[1], new([]*big.Int)).(*[]*big.Int)
	outstruct.HasMore = *abi.ConvertType(out[2], new(bool)).(*bool)

	return *outstruct, err

}

// GetActivePieces is a free data retrieval call binding the contract method 0x39f51544.
//
// Solidity: function getActivePieces(uint256 setId, uint256 offset, uint256 limit) view returns((bytes)[] pieces, uint256[] pieceIds, bool hasMore)
func (_ProofVerifier *ProofVerifierSession) GetActivePieces(setId *big.Int, offset *big.Int, limit *big.Int) (struct {
	Pieces   []CidsCid
	PieceIds []*big.Int
	HasMore  bool
}, error) {
	return _ProofVerifier.Contract.GetActivePieces(&_ProofVerifier.CallOpts, setId, offset, limit)
}

// GetActivePieces is a free data retrieval call binding the contract method 0x39f51544.
//
// Solidity: function getActivePieces(uint256 setId, uint256 offset, uint256 limit) view returns((bytes)[] pieces, uint256[] pieceIds, bool hasMore)
func (_ProofVerifier *ProofVerifierCallerSession) GetActivePieces(setId *big.Int, offset *big.Int, limit *big.Int) (struct {
	Pieces   []CidsCid
	PieceIds []*big.Int
	HasMore  bool
}, error) {
	return _ProofVerifier.Contract.GetActivePieces(&_ProofVerifier.CallOpts, setId, offset, limit)
}

// GetActivePiecesByCursor is a free data retrieval call binding the contract method 0x9fbc9f1b.
//
// Solidity: function getActivePiecesByCursor(uint256 setId, uint256 startPieceId, uint256 limit) view returns((bytes)[] pieces, uint256[] pieceIds, bool hasMore)
func (_ProofVerifier *ProofVerifierCaller) GetActivePiecesByCursor(opts *bind.CallOpts, setId *big.Int, startPieceId *big.Int, limit *big.Int) (struct {
	Pieces   []CidsCid
	PieceIds []*big.Int
	HasMore  bool
}, error) {
	var out []interface{}
	err := _ProofVerifier.contract.Call(opts, &out, "getActivePiecesByCursor", setId, startPieceId, limit)

	outstruct := new(struct {
		Pieces   []CidsCid
		PieceIds []*big.Int
		HasMore  bool
	})
	if err != nil {
		return *outstruct, err
	}

	outstruct.Pieces = *abi.ConvertType(out[0], new([]CidsCid)).(*[]CidsCid)
	outstruct.PieceIds = *abi.ConvertType(out[1], new([]*big.Int)).(*[]*big.Int)
	outstruct.HasMore = *abi.ConvertType(out[2], new(bool)).(*bool)

	return *outstruct, err

}

// GetActivePiecesByCursor is a free data retrieval call binding the contract method 0x9fbc9f1b.
//
// Solidity: function getActivePiecesByCursor(uint256 setId, uint256 startPieceId, uint256 limit) view returns((bytes)[] pieces, uint256[] pieceIds, bool hasMore)
func (_ProofVerifier *ProofVerifierSession) GetActivePiecesByCursor(setId *big.Int, startPieceId *big.Int, limit *big.Int) (struct {
	Pieces   []CidsCid
	PieceIds []*big.Int
	HasMore  bool
}, error) {
	return _ProofVerifier.Contract.GetActivePiecesByCursor(&_ProofVerifier.CallOpts, setId, startPieceId, limit)
}

// GetActivePiecesByCursor is a free data retrieval call binding the contract method 0x9fbc9f1b.
//
// Solidity: function getActivePiecesByCursor(uint256 setId, uint256 startPieceId, uint256 limit) view returns((bytes)[] pieces, uint256[] pieceIds, bool hasMore)
func (_ProofVerifier *ProofVerifierCallerSession) GetActivePiecesByCursor(setId *big.Int, startPieceId *big.Int, limit *big.Int) (struct {
	Pieces   []CidsCid
	PieceIds []*big.Int
	HasMore  bool
}, error) {
	return _ProofVerifier.Contract.GetActivePiecesByCursor(&_ProofVerifier.CallOpts, setId, startPieceId, limit)
}

// GetChallengeFinality is a free data retrieval call binding the contract method 0xf83758fe.
//
// Solidity: function getChallengeFinality() view returns(uint256)
func (_ProofVerifier *ProofVerifierCaller) GetChallengeFinality(opts *bind.CallOpts) (*big.Int, error) {
	var out []interface{}
	err := _ProofVerifier.contract.Call(opts, &out, "getChallengeFinality")

	if err != nil {
		return *new(*big.Int), err
	}

	out0 := *abi.ConvertType(out[0], new(*big.Int)).(**big.Int)

	return out0, err

}

// GetChallengeFinality is a free data retrieval call binding the contract method 0xf83758fe.
//
// Solidity: function getChallengeFinality() view returns(uint256)
func (_ProofVerifier *ProofVerifierSession) GetChallengeFinality() (*big.Int, error) {
	return _ProofVerifier.Contract.GetChallengeFinality(&_ProofVerifier.CallOpts)
}

// GetChallengeFinality is a free data retrieval call binding the contract method 0xf83758fe.
//
// Solidity: function getChallengeFinality() view returns(uint256)
func (_ProofVerifier *ProofVerifierCallerSession) GetChallengeFinality() (*big.Int, error) {
	return _ProofVerifier.Contract.GetChallengeFinality(&_ProofVerifier.CallOpts)
}

// GetChallengeRange is a free data retrieval call binding the contract method 0x89208ba9.
//
// Solidity: function getChallengeRange(uint256 setId) view returns(uint256)
func (_ProofVerifier *ProofVerifierCaller) GetChallengeRange(opts *bind.CallOpts, setId *big.Int) (*big.Int, error) {
	var out []interface{}
	err := _ProofVerifier.contract.Call(opts, &out, "getChallengeRange", setId)

	if err != nil {
		return *new(*big.Int), err
	}

	out0 := *abi.ConvertType(out[0], new(*big.Int)).(**big.Int)

	return out0, err

}

// GetChallengeRange is a free data retrieval call binding the contract method 0x89208ba9.
//
// Solidity: function getChallengeRange(uint256 setId) view returns(uint256)
func (_ProofVerifier *ProofVerifierSession) GetChallengeRange(setId *big.Int) (*big.Int, error) {
	return _ProofVerifier.Contract.GetChallengeRange(&_ProofVerifier.CallOpts, setId)
}

// GetChallengeRange is a free data retrieval call binding the contract method 0x89208ba9.
//
// Solidity: function getChallengeRange(uint256 setId) view returns(uint256)
func (_ProofVerifier *ProofVerifierCallerSession) GetChallengeRange(setId *big.Int) (*big.Int, error) {
	return _ProofVerifier.Contract.GetChallengeRange(&_ProofVerifier.CallOpts, setId)
}

// GetDataSetLastProvenEpoch is a free data retrieval call binding the contract method 0x04595c1a.
//
// Solidity: function getDataSetLastProvenEpoch(uint256 setId) view returns(uint256)
func (_ProofVerifier *ProofVerifierCaller) GetDataSetLastProvenEpoch(opts *bind.CallOpts, setId *big.Int) (*big.Int, error) {
	var out []interface{}
	err := _ProofVerifier.contract.Call(opts, &out, "getDataSetLastProvenEpoch", setId)

	if err != nil {
		return *new(*big.Int), err
	}

	out0 := *abi.ConvertType(out[0], new(*big.Int)).(**big.Int)

	return out0, err

}

// GetDataSetLastProvenEpoch is a free data retrieval call binding the contract method 0x04595c1a.
//
// Solidity: function getDataSetLastProvenEpoch(uint256 setId) view returns(uint256)
func (_ProofVerifier *ProofVerifierSession) GetDataSetLastProvenEpoch(setId *big.Int) (*big.Int, error) {
	return _ProofVerifier.Contract.GetDataSetLastProvenEpoch(&_ProofVerifier.CallOpts, setId)
}

// GetDataSetLastProvenEpoch is a free data retrieval call binding the contract method 0x04595c1a.
//
// Solidity: function getDataSetLastProvenEpoch(uint256 setId) view returns(uint256)
func (_ProofVerifier *ProofVerifierCallerSession) GetDataSetLastProvenEpoch(setId *big.Int) (*big.Int, error) {
	return _ProofVerifier.Contract.GetDataSetLastProvenEpoch(&_ProofVerifier.CallOpts, setId)
}

// GetDataSetLeafCount is a free data retrieval call binding the contract method 0xa531998c.
//
// Solidity: function getDataSetLeafCount(uint256 setId) view returns(uint256)
func (_ProofVerifier *ProofVerifierCaller) GetDataSetLeafCount(opts *bind.CallOpts, setId *big.Int) (*big.Int, error) {
	var out []interface{}
	err := _ProofVerifier.contract.Call(opts, &out, "getDataSetLeafCount", setId)

	if err != nil {
		return *new(*big.Int), err
	}

	out0 := *abi.ConvertType(out[0], new(*big.Int)).(**big.Int)

	return out0, err

}

// GetDataSetLeafCount is a free data retrieval call binding the contract method 0xa531998c.
//
// Solidity: function getDataSetLeafCount(uint256 setId) view returns(uint256)
func (_ProofVerifier *ProofVerifierSession) GetDataSetLeafCount(setId *big.Int) (*big.Int, error) {
	return _ProofVerifier.Contract.GetDataSetLeafCount(&_ProofVerifier.CallOpts, setId)
}

// GetDataSetLeafCount is a free data retrieval call binding the contract method 0xa531998c.
//
// Solidity: function getDataSetLeafCount(uint256 setId) view returns(uint256)
func (_ProofVerifier *ProofVerifierCallerSession) GetDataSetLeafCount(setId *big.Int) (*big.Int, error) {
	return _ProofVerifier.Contract.GetDataSetLeafCount(&_ProofVerifier.CallOpts, setId)
}

// GetDataSetListener is a free data retrieval call binding the contract method 0x2b3129bb.
//
// Solidity: function getDataSetListener(uint256 setId) view returns(address)
func (_ProofVerifier *ProofVerifierCaller) GetDataSetListener(opts *bind.CallOpts, setId *big.Int) (common.Address, error) {
	var out []interface{}
	err := _ProofVerifier.contract.Call(opts, &out, "getDataSetListener", setId)

	if err != nil {
		return *new(common.Address), err
	}

	out0 := *abi.ConvertType(out[0], new(common.Address)).(*common.Address)

	return out0, err

}

// GetDataSetListener is a free data retrieval call binding the contract method 0x2b3129bb.
//
// Solidity: function getDataSetListener(uint256 setId) view returns(address)
func (_ProofVerifier *ProofVerifierSession) GetDataSetListener(setId *big.Int) (common.Address, error) {
	return _ProofVerifier.Contract.GetDataSetListener(&_ProofVerifier.CallOpts, setId)
}

// GetDataSetListener is a free data retrieval call binding the contract method 0x2b3129bb.
//
// Solidity: function getDataSetListener(uint256 setId) view returns(address)
func (_ProofVerifier *ProofVerifierCallerSession) GetDataSetListener(setId *big.Int) (common.Address, error) {
	return _ProofVerifier.Contract.GetDataSetListener(&_ProofVerifier.CallOpts, setId)
}

// GetDataSetStorageProvider is a free data retrieval call binding the contract method 0x21b7cd1c.
//
// Solidity: function getDataSetStorageProvider(uint256 setId) view returns(address, address)
func (_ProofVerifier *ProofVerifierCaller) GetDataSetStorageProvider(opts *bind.CallOpts, setId *big.Int) (common.Address, common.Address, error) {
	var out []interface{}
	err := _ProofVerifier.contract.Call(opts, &out, "getDataSetStorageProvider", setId)

	if err != nil {
		return *new(common.Address), *new(common.Address), err
	}

	out0 := *abi.ConvertType(out[0], new(common.Address)).(*common.Address)
	out1 := *abi.ConvertType(out[1], new(common.Address)).(*common.Address)

	return out0, out1, err

}

// GetDataSetStorageProvider is a free data retrieval call binding the contract method 0x21b7cd1c.
//
// Solidity: function getDataSetStorageProvider(uint256 setId) view returns(address, address)
func (_ProofVerifier *ProofVerifierSession) GetDataSetStorageProvider(setId *big.Int) (common.Address, common.Address, error) {
	return _ProofVerifier.Contract.GetDataSetStorageProvider(&_ProofVerifier.CallOpts, setId)
}

// GetDataSetStorageProvider is a free data retrieval call binding the contract method 0x21b7cd1c.
//
// Solidity: function getDataSetStorageProvider(uint256 setId) view returns(address, address)
func (_ProofVerifier *ProofVerifierCallerSession) GetDataSetStorageProvider(setId *big.Int) (common.Address, common.Address, error) {
	return _ProofVerifier.Contract.GetDataSetStorageProvider(&_ProofVerifier.CallOpts, setId)
}

// GetNextChallengeEpoch is a free data retrieval call binding the contract method 0x6ba4608f.
//
// Solidity: function getNextChallengeEpoch(uint256 setId) view returns(uint256)
func (_ProofVerifier *ProofVerifierCaller) GetNextChallengeEpoch(opts *bind.CallOpts, setId *big.Int) (*big.Int, error) {
	var out []interface{}
	err := _ProofVerifier.contract.Call(opts, &out, "getNextChallengeEpoch", setId)

	if err != nil {
		return *new(*big.Int), err
	}

	out0 := *abi.ConvertType(out[0], new(*big.Int)).(**big.Int)

	return out0, err

}

// GetNextChallengeEpoch is a free data retrieval call binding the contract method 0x6ba4608f.
//
// Solidity: function getNextChallengeEpoch(uint256 setId) view returns(uint256)
func (_ProofVerifier *ProofVerifierSession) GetNextChallengeEpoch(setId *big.Int) (*big.Int, error) {
	return _ProofVerifier.Contract.GetNextChallengeEpoch(&_ProofVerifier.CallOpts, setId)
}

// GetNextChallengeEpoch is a free data retrieval call binding the contract method 0x6ba4608f.
//
// Solidity: function getNextChallengeEpoch(uint256 setId) view returns(uint256)
func (_ProofVerifier *ProofVerifierCallerSession) GetNextChallengeEpoch(setId *big.Int) (*big.Int, error) {
	return _ProofVerifier.Contract.GetNextChallengeEpoch(&_ProofVerifier.CallOpts, setId)
}

// GetNextDataSetId is a free data retrieval call binding the contract method 0x442cded3.
//
// Solidity: function getNextDataSetId() view returns(uint64)
func (_ProofVerifier *ProofVerifierCaller) GetNextDataSetId(opts *bind.CallOpts) (uint64, error) {
	var out []interface{}
	err := _ProofVerifier.contract.Call(opts, &out, "getNextDataSetId")

	if err != nil {
		return *new(uint64), err
	}

	out0 := *abi.ConvertType(out[0], new(uint64)).(*uint64)

	return out0, err

}

// GetNextDataSetId is a free data retrieval call binding the contract method 0x442cded3.
//
// Solidity: function getNextDataSetId() view returns(uint64)
func (_ProofVerifier *ProofVerifierSession) GetNextDataSetId() (uint64, error) {
	return _ProofVerifier.Contract.GetNextDataSetId(&_ProofVerifier.CallOpts)
}

// GetNextDataSetId is a free data retrieval call binding the contract method 0x442cded3.
//
// Solidity: function getNextDataSetId() view returns(uint64)
func (_ProofVerifier *ProofVerifierCallerSession) GetNextDataSetId() (uint64, error) {
	return _ProofVerifier.Contract.GetNextDataSetId(&_ProofVerifier.CallOpts)
}

// GetNextPieceId is a free data retrieval call binding the contract method 0x1c5ae80f.
//
// Solidity: function getNextPieceId(uint256 setId) view returns(uint256)
func (_ProofVerifier *ProofVerifierCaller) GetNextPieceId(opts *bind.CallOpts, setId *big.Int) (*big.Int, error) {
	var out []interface{}
	err := _ProofVerifier.contract.Call(opts, &out, "getNextPieceId", setId)

	if err != nil {
		return *new(*big.Int), err
	}

	out0 := *abi.ConvertType(out[0], new(*big.Int)).(**big.Int)

	return out0, err

}

// GetNextPieceId is a free data retrieval call binding the contract method 0x1c5ae80f.
//
// Solidity: function getNextPieceId(uint256 setId) view returns(uint256)
func (_ProofVerifier *ProofVerifierSession) GetNextPieceId(setId *big.Int) (*big.Int, error) {
	return _ProofVerifier.Contract.GetNextPieceId(&_ProofVerifier.CallOpts, setId)
}

// GetNextPieceId is a free data retrieval call binding the contract method 0x1c5ae80f.
//
// Solidity: function getNextPieceId(uint256 setId) view returns(uint256)
func (_ProofVerifier *ProofVerifierCallerSession) GetNextPieceId(setId *big.Int) (*big.Int, error) {
	return _ProofVerifier.Contract.GetNextPieceId(&_ProofVerifier.CallOpts, setId)
}

// GetPieceCid is a free data retrieval call binding the contract method 0x25bbbedf.
//
// Solidity: function getPieceCid(uint256 setId, uint256 pieceId) view returns((bytes))
func (_ProofVerifier *ProofVerifierCaller) GetPieceCid(opts *bind.CallOpts, setId *big.Int, pieceId *big.Int) (CidsCid, error) {
	var out []interface{}
	err := _ProofVerifier.contract.Call(opts, &out, "getPieceCid", setId, pieceId)

	if err != nil {
		return *new(CidsCid), err
	}

	out0 := *abi.ConvertType(out[0], new(CidsCid)).(*CidsCid)

	return out0, err

}

// GetPieceCid is a free data retrieval call binding the contract method 0x25bbbedf.
//
// Solidity: function getPieceCid(uint256 setId, uint256 pieceId) view returns((bytes))
func (_ProofVerifier *ProofVerifierSession) GetPieceCid(setId *big.Int, pieceId *big.Int) (CidsCid, error) {
	return _ProofVerifier.Contract.GetPieceCid(&_ProofVerifier.CallOpts, setId, pieceId)
}

// GetPieceCid is a free data retrieval call binding the contract method 0x25bbbedf.
//
// Solidity: function getPieceCid(uint256 setId, uint256 pieceId) view returns((bytes))
func (_ProofVerifier *ProofVerifierCallerSession) GetPieceCid(setId *big.Int, pieceId *big.Int) (CidsCid, error) {
	return _ProofVerifier.Contract.GetPieceCid(&_ProofVerifier.CallOpts, setId, pieceId)
}

// GetPieceLeafCount is a free data retrieval call binding the contract method 0x0cd7b880.
//
// Solidity: function getPieceLeafCount(uint256 setId, uint256 pieceId) view returns(uint256)
func (_ProofVerifier *ProofVerifierCaller) GetPieceLeafCount(opts *bind.CallOpts, setId *big.Int, pieceId *big.Int) (*big.Int, error) {
	var out []interface{}
	err := _ProofVerifier.contract.Call(opts, &out, "getPieceLeafCount", setId, pieceId)

	if err != nil {
		return *new(*big.Int), err
	}

	out0 := *abi.ConvertType(out[0], new(*big.Int)).(**big.Int)

	return out0, err

}

// GetPieceLeafCount is a free data retrieval call binding the contract method 0x0cd7b880.
//
// Solidity: function getPieceLeafCount(uint256 setId, uint256 pieceId) view returns(uint256)
func (_ProofVerifier *ProofVerifierSession) GetPieceLeafCount(setId *big.Int, pieceId *big.Int) (*big.Int, error) {
	return _ProofVerifier.Contract.GetPieceLeafCount(&_ProofVerifier.CallOpts, setId, pieceId)
}

// GetPieceLeafCount is a free data retrieval call binding the contract method 0x0cd7b880.
//
// Solidity: function getPieceLeafCount(uint256 setId, uint256 pieceId) view returns(uint256)
func (_ProofVerifier *ProofVerifierCallerSession) GetPieceLeafCount(setId *big.Int, pieceId *big.Int) (*big.Int, error) {
	return _ProofVerifier.Contract.GetPieceLeafCount(&_ProofVerifier.CallOpts, setId, pieceId)
}

// GetRandomness is a free data retrieval call binding the contract method 0x453f4f62.
//
// Solidity: function getRandomness(uint256 epoch) view returns(uint256)
func (_ProofVerifier *ProofVerifierCaller) GetRandomness(opts *bind.CallOpts, epoch *big.Int) (*big.Int, error) {
	var out []interface{}
	err := _ProofVerifier.contract.Call(opts, &out, "getRandomness", epoch)

	if err != nil {
		return *new(*big.Int), err
	}

	out0 := *abi.ConvertType(out[0], new(*big.Int)).(**big.Int)

	return out0, err

}

// GetRandomness is a free data retrieval call binding the contract method 0x453f4f62.
//
// Solidity: function getRandomness(uint256 epoch) view returns(uint256)
func (_ProofVerifier *ProofVerifierSession) GetRandomness(epoch *big.Int) (*big.Int, error) {
	return _ProofVerifier.Contract.GetRandomness(&_ProofVerifier.CallOpts, epoch)
}

// GetRandomness is a free data retrieval call binding the contract method 0x453f4f62.
//
// Solidity: function getRandomness(uint256 epoch) view returns(uint256)
func (_ProofVerifier *ProofVerifierCallerSession) GetRandomness(epoch *big.Int) (*big.Int, error) {
	return _ProofVerifier.Contract.GetRandomness(&_ProofVerifier.CallOpts, epoch)
}

// GetScheduledRemovals is a free data retrieval call binding the contract method 0x6fa44692.
//
// Solidity: function getScheduledRemovals(uint256 setId) view returns(uint256[])
func (_ProofVerifier *ProofVerifierCaller) GetScheduledRemovals(opts *bind.CallOpts, setId *big.Int) ([]*big.Int, error) {
	var out []interface{}
	err := _ProofVerifier.contract.Call(opts, &out, "getScheduledRemovals", setId)

	if err != nil {
		return *new([]*big.Int), err
	}

	out0 := *abi.ConvertType(out[0], new([]*big.Int)).(*[]*big.Int)

	return out0, err

}

// GetScheduledRemovals is a free data retrieval call binding the contract method 0x6fa44692.
//
// Solidity: function getScheduledRemovals(uint256 setId) view returns(uint256[])
func (_ProofVerifier *ProofVerifierSession) GetScheduledRemovals(setId *big.Int) ([]*big.Int, error) {
	return _ProofVerifier.Contract.GetScheduledRemovals(&_ProofVerifier.CallOpts, setId)
}

// GetScheduledRemovals is a free data retrieval call binding the contract method 0x6fa44692.
//
// Solidity: function getScheduledRemovals(uint256 setId) view returns(uint256[])
func (_ProofVerifier *ProofVerifierCallerSession) GetScheduledRemovals(setId *big.Int) ([]*big.Int, error) {
	return _ProofVerifier.Contract.GetScheduledRemovals(&_ProofVerifier.CallOpts, setId)
}

// NextUpgrade is a free data retrieval call binding the contract method 0x315e49ea.
//
// Solidity: function nextUpgrade() view returns(address nextImplementation, uint96 afterEpoch)
func (_ProofVerifier *ProofVerifierCaller) NextUpgrade(opts *bind.CallOpts) (struct {
	NextImplementation common.Address
	AfterEpoch         *big.Int
}, error) {
	var out []interface{}
	err := _ProofVerifier.contract.Call(opts, &out, "nextUpgrade")

	outstruct := new(struct {
		NextImplementation common.Address
		AfterEpoch         *big.Int
	})
	if err != nil {
		return *outstruct, err
	}

	outstruct.NextImplementation = *abi.ConvertType(out[0], new(common.Address)).(*common.Address)
	outstruct.AfterEpoch = *abi.ConvertType(out[1], new(*big.Int)).(**big.Int)

	return *outstruct, err

}

// NextUpgrade is a free data retrieval call binding the contract method 0x315e49ea.
//
// Solidity: function nextUpgrade() view returns(address nextImplementation, uint96 afterEpoch)
func (_ProofVerifier *ProofVerifierSession) NextUpgrade() (struct {
	NextImplementation common.Address
	AfterEpoch         *big.Int
}, error) {
	return _ProofVerifier.Contract.NextUpgrade(&_ProofVerifier.CallOpts)
}

// NextUpgrade is a free data retrieval call binding the contract method 0x315e49ea.
//
// Solidity: function nextUpgrade() view returns(address nextImplementation, uint96 afterEpoch)
func (_ProofVerifier *ProofVerifierCallerSession) NextUpgrade() (struct {
	NextImplementation common.Address
	AfterEpoch         *big.Int
}, error) {
	return _ProofVerifier.Contract.NextUpgrade(&_ProofVerifier.CallOpts)
}

// Owner is a free data retrieval call binding the contract method 0x8da5cb5b.
//
// Solidity: function owner() view returns(address)
func (_ProofVerifier *ProofVerifierCaller) Owner(opts *bind.CallOpts) (common.Address, error) {
	var out []interface{}
	err := _ProofVerifier.contract.Call(opts, &out, "owner")

	if err != nil {
		return *new(common.Address), err
	}

	out0 := *abi.ConvertType(out[0], new(common.Address)).(*common.Address)

	return out0, err

}

// Owner is a free data retrieval call binding the contract method 0x8da5cb5b.
//
// Solidity: function owner() view returns(address)
func (_ProofVerifier *ProofVerifierSession) Owner() (common.Address, error) {
	return _ProofVerifier.Contract.Owner(&_ProofVerifier.CallOpts)
}

// Owner is a free data retrieval call binding the contract method 0x8da5cb5b.
//
// Solidity: function owner() view returns(address)
func (_ProofVerifier *ProofVerifierCallerSession) Owner() (common.Address, error) {
	return _ProofVerifier.Contract.Owner(&_ProofVerifier.CallOpts)
}

// PieceChallengable is a free data retrieval call binding the contract method 0xdc635266.
//
// Solidity: function pieceChallengable(uint256 setId, uint256 pieceId) view returns(bool)
func (_ProofVerifier *ProofVerifierCaller) PieceChallengable(opts *bind.CallOpts, setId *big.Int, pieceId *big.Int) (bool, error) {
	var out []interface{}
	err := _ProofVerifier.contract.Call(opts, &out, "pieceChallengable", setId, pieceId)

	if err != nil {
		return *new(bool), err
	}

	out0 := *abi.ConvertType(out[0], new(bool)).(*bool)

	return out0, err

}

// PieceChallengable is a free data retrieval call binding the contract method 0xdc635266.
//
// Solidity: function pieceChallengable(uint256 setId, uint256 pieceId) view returns(bool)
func (_ProofVerifier *ProofVerifierSession) PieceChallengable(setId *big.Int, pieceId *big.Int) (bool, error) {
	return _ProofVerifier.Contract.PieceChallengable(&_ProofVerifier.CallOpts, setId, pieceId)
}

// PieceChallengable is a free data retrieval call binding the contract method 0xdc635266.
//
// Solidity: function pieceChallengable(uint256 setId, uint256 pieceId) view returns(bool)
func (_ProofVerifier *ProofVerifierCallerSession) PieceChallengable(setId *big.Int, pieceId *big.Int) (bool, error) {
	return _ProofVerifier.Contract.PieceChallengable(&_ProofVerifier.CallOpts, setId, pieceId)
}

// PieceLive is a free data retrieval call binding the contract method 0x1a271225.
//
// Solidity: function pieceLive(uint256 setId, uint256 pieceId) view returns(bool)
func (_ProofVerifier *ProofVerifierCaller) PieceLive(opts *bind.CallOpts, setId *big.Int, pieceId *big.Int) (bool, error) {
	var out []interface{}
	err := _ProofVerifier.contract.Call(opts, &out, "pieceLive", setId, pieceId)

	if err != nil {
		return *new(bool), err
	}

	out0 := *abi.ConvertType(out[0], new(bool)).(*bool)

	return out0, err

}

// PieceLive is a free data retrieval call binding the contract method 0x1a271225.
//
// Solidity: function pieceLive(uint256 setId, uint256 pieceId) view returns(bool)
func (_ProofVerifier *ProofVerifierSession) PieceLive(setId *big.Int, pieceId *big.Int) (bool, error) {
	return _ProofVerifier.Contract.PieceLive(&_ProofVerifier.CallOpts, setId, pieceId)
}

// PieceLive is a free data retrieval call binding the contract method 0x1a271225.
//
// Solidity: function pieceLive(uint256 setId, uint256 pieceId) view returns(bool)
func (_ProofVerifier *ProofVerifierCallerSession) PieceLive(setId *big.Int, pieceId *big.Int) (bool, error) {
	return _ProofVerifier.Contract.PieceLive(&_ProofVerifier.CallOpts, setId, pieceId)
}

// ProposedFeePerTiB is a free data retrieval call binding the contract method 0xba74d94c.
//
// Solidity: function proposedFeePerTiB() view returns(uint96)
func (_ProofVerifier *ProofVerifierCaller) ProposedFeePerTiB(opts *bind.CallOpts) (*big.Int, error) {
	var out []interface{}
	err := _ProofVerifier.contract.Call(opts, &out, "proposedFeePerTiB")

	if err != nil {
		return *new(*big.Int), err
	}

	out0 := *abi.ConvertType(out[0], new(*big.Int)).(**big.Int)

	return out0, err

}

// ProposedFeePerTiB is a free data retrieval call binding the contract method 0xba74d94c.
//
// Solidity: function proposedFeePerTiB() view returns(uint96)
func (_ProofVerifier *ProofVerifierSession) ProposedFeePerTiB() (*big.Int, error) {
	return _ProofVerifier.Contract.ProposedFeePerTiB(&_ProofVerifier.CallOpts)
}

// ProposedFeePerTiB is a free data retrieval call binding the contract method 0xba74d94c.
//
// Solidity: function proposedFeePerTiB() view returns(uint96)
func (_ProofVerifier *ProofVerifierCallerSession) ProposedFeePerTiB() (*big.Int, error) {
	return _ProofVerifier.Contract.ProposedFeePerTiB(&_ProofVerifier.CallOpts)
}

// ProxiableUUID is a free data retrieval call binding the contract method 0x52d1902d.
//
// Solidity: function proxiableUUID() view returns(bytes32)
func (_ProofVerifier *ProofVerifierCaller) ProxiableUUID(opts *bind.CallOpts) ([32]byte, error) {
	var out []interface{}
	err := _ProofVerifier.contract.Call(opts, &out, "proxiableUUID")

	if err != nil {
		return *new([32]byte), err
	}

	out0 := *abi.ConvertType(out[0], new([32]byte)).(*[32]byte)

	return out0, err

}

// ProxiableUUID is a free data retrieval call binding the contract method 0x52d1902d.
//
// Solidity: function proxiableUUID() view returns(bytes32)
func (_ProofVerifier *ProofVerifierSession) ProxiableUUID() ([32]byte, error) {
	return _ProofVerifier.Contract.ProxiableUUID(&_ProofVerifier.CallOpts)
}

// ProxiableUUID is a free data retrieval call binding the contract method 0x52d1902d.
//
// Solidity: function proxiableUUID() view returns(bytes32)
func (_ProofVerifier *ProofVerifierCallerSession) ProxiableUUID() ([32]byte, error) {
	return _ProofVerifier.Contract.ProxiableUUID(&_ProofVerifier.CallOpts)
}

// AddPieces is a paid mutator transaction binding the contract method 0x9afd37f2.
//
// Solidity: function addPieces(uint256 setId, address listenerAddr, (bytes)[] pieceData, bytes extraData) payable returns(uint256)
func (_ProofVerifier *ProofVerifierTransactor) AddPieces(opts *bind.TransactOpts, setId *big.Int, listenerAddr common.Address, pieceData []CidsCid, extraData []byte) (*types.Transaction, error) {
	return _ProofVerifier.contract.Transact(opts, "addPieces", setId, listenerAddr, pieceData, extraData)
}

// AddPieces is a paid mutator transaction binding the contract method 0x9afd37f2.
//
// Solidity: function addPieces(uint256 setId, address listenerAddr, (bytes)[] pieceData, bytes extraData) payable returns(uint256)
func (_ProofVerifier *ProofVerifierSession) AddPieces(setId *big.Int, listenerAddr common.Address, pieceData []CidsCid, extraData []byte) (*types.Transaction, error) {
	return _ProofVerifier.Contract.AddPieces(&_ProofVerifier.TransactOpts, setId, listenerAddr, pieceData, extraData)
}

// AddPieces is a paid mutator transaction binding the contract method 0x9afd37f2.
//
// Solidity: function addPieces(uint256 setId, address listenerAddr, (bytes)[] pieceData, bytes extraData) payable returns(uint256)
func (_ProofVerifier *ProofVerifierTransactorSession) AddPieces(setId *big.Int, listenerAddr common.Address, pieceData []CidsCid, extraData []byte) (*types.Transaction, error) {
	return _ProofVerifier.Contract.AddPieces(&_ProofVerifier.TransactOpts, setId, listenerAddr, pieceData, extraData)
}

// AnnouncePlannedUpgrade is a paid mutator transaction binding the contract method 0xbd003827.
//
// Solidity: function announcePlannedUpgrade((address,uint96) plannedUpgrade) returns()
func (_ProofVerifier *ProofVerifierTransactor) AnnouncePlannedUpgrade(opts *bind.TransactOpts, plannedUpgrade ProofVerifierPlannedUpgrade) (*types.Transaction, error) {
	return _ProofVerifier.contract.Transact(opts, "announcePlannedUpgrade", plannedUpgrade)
}

// AnnouncePlannedUpgrade is a paid mutator transaction binding the contract method 0xbd003827.
//
// Solidity: function announcePlannedUpgrade((address,uint96) plannedUpgrade) returns()
func (_ProofVerifier *ProofVerifierSession) AnnouncePlannedUpgrade(plannedUpgrade ProofVerifierPlannedUpgrade) (*types.Transaction, error) {
	return _ProofVerifier.Contract.AnnouncePlannedUpgrade(&_ProofVerifier.TransactOpts, plannedUpgrade)
}

// AnnouncePlannedUpgrade is a paid mutator transaction binding the contract method 0xbd003827.
//
// Solidity: function announcePlannedUpgrade((address,uint96) plannedUpgrade) returns()
func (_ProofVerifier *ProofVerifierTransactorSession) AnnouncePlannedUpgrade(plannedUpgrade ProofVerifierPlannedUpgrade) (*types.Transaction, error) {
	return _ProofVerifier.Contract.AnnouncePlannedUpgrade(&_ProofVerifier.TransactOpts, plannedUpgrade)
}

// ClaimDataSetStorageProvider is a paid mutator transaction binding the contract method 0xdf0f3248.
//
// Solidity: function claimDataSetStorageProvider(uint256 setId, bytes extraData) returns()
func (_ProofVerifier *ProofVerifierTransactor) ClaimDataSetStorageProvider(opts *bind.TransactOpts, setId *big.Int, extraData []byte) (*types.Transaction, error) {
	return _ProofVerifier.contract.Transact(opts, "claimDataSetStorageProvider", setId, extraData)
}

// ClaimDataSetStorageProvider is a paid mutator transaction binding the contract method 0xdf0f3248.
//
// Solidity: function claimDataSetStorageProvider(uint256 setId, bytes extraData) returns()
func (_ProofVerifier *ProofVerifierSession) ClaimDataSetStorageProvider(setId *big.Int, extraData []byte) (*types.Transaction, error) {
	return _ProofVerifier.Contract.ClaimDataSetStorageProvider(&_ProofVerifier.TransactOpts, setId, extraData)
}

// ClaimDataSetStorageProvider is a paid mutator transaction binding the contract method 0xdf0f3248.
//
// Solidity: function claimDataSetStorageProvider(uint256 setId, bytes extraData) returns()
func (_ProofVerifier *ProofVerifierTransactorSession) ClaimDataSetStorageProvider(setId *big.Int, extraData []byte) (*types.Transaction, error) {
	return _ProofVerifier.Contract.ClaimDataSetStorageProvider(&_ProofVerifier.TransactOpts, setId, extraData)
}

// CreateDataSet is a paid mutator transaction binding the contract method 0xbbae41cb.
//
// Solidity: function createDataSet(address listenerAddr, bytes extraData) payable returns(uint256)
func (_ProofVerifier *ProofVerifierTransactor) CreateDataSet(opts *bind.TransactOpts, listenerAddr common.Address, extraData []byte) (*types.Transaction, error) {
	return _ProofVerifier.contract.Transact(opts, "createDataSet", listenerAddr, extraData)
}

// CreateDataSet is a paid mutator transaction binding the contract method 0xbbae41cb.
//
// Solidity: function createDataSet(address listenerAddr, bytes extraData) payable returns(uint256)
func (_ProofVerifier *ProofVerifierSession) CreateDataSet(listenerAddr common.Address, extraData []byte) (*types.Transaction, error) {
	return _ProofVerifier.Contract.CreateDataSet(&_ProofVerifier.TransactOpts, listenerAddr, extraData)
}

// CreateDataSet is a paid mutator transaction binding the contract method 0xbbae41cb.
//
// Solidity: function createDataSet(address listenerAddr, bytes extraData) payable returns(uint256)
func (_ProofVerifier *ProofVerifierTransactorSession) CreateDataSet(listenerAddr common.Address, extraData []byte) (*types.Transaction, error) {
	return _ProofVerifier.Contract.CreateDataSet(&_ProofVerifier.TransactOpts, listenerAddr, extraData)
}

// DeleteDataSet is a paid mutator transaction binding the contract method 0x7a1e2990.
//
// Solidity: function deleteDataSet(uint256 setId, bytes extraData) returns()
func (_ProofVerifier *ProofVerifierTransactor) DeleteDataSet(opts *bind.TransactOpts, setId *big.Int, extraData []byte) (*types.Transaction, error) {
	return _ProofVerifier.contract.Transact(opts, "deleteDataSet", setId, extraData)
}

// DeleteDataSet is a paid mutator transaction binding the contract method 0x7a1e2990.
//
// Solidity: function deleteDataSet(uint256 setId, bytes extraData) returns()
func (_ProofVerifier *ProofVerifierSession) DeleteDataSet(setId *big.Int, extraData []byte) (*types.Transaction, error) {
	return _ProofVerifier.Contract.DeleteDataSet(&_ProofVerifier.TransactOpts, setId, extraData)
}

// DeleteDataSet is a paid mutator transaction binding the contract method 0x7a1e2990.
//
// Solidity: function deleteDataSet(uint256 setId, bytes extraData) returns()
func (_ProofVerifier *ProofVerifierTransactorSession) DeleteDataSet(setId *big.Int, extraData []byte) (*types.Transaction, error) {
	return _ProofVerifier.Contract.DeleteDataSet(&_ProofVerifier.TransactOpts, setId, extraData)
}

// Initialize is a paid mutator transaction binding the contract method 0xfe4b84df.
//
// Solidity: function initialize(uint256 _challengeFinality) returns()
func (_ProofVerifier *ProofVerifierTransactor) Initialize(opts *bind.TransactOpts, _challengeFinality *big.Int) (*types.Transaction, error) {
	return _ProofVerifier.contract.Transact(opts, "initialize", _challengeFinality)
}

// Initialize is a paid mutator transaction binding the contract method 0xfe4b84df.
//
// Solidity: function initialize(uint256 _challengeFinality) returns()
func (_ProofVerifier *ProofVerifierSession) Initialize(_challengeFinality *big.Int) (*types.Transaction, error) {
	return _ProofVerifier.Contract.Initialize(&_ProofVerifier.TransactOpts, _challengeFinality)
}

// Initialize is a paid mutator transaction binding the contract method 0xfe4b84df.
//
// Solidity: function initialize(uint256 _challengeFinality) returns()
func (_ProofVerifier *ProofVerifierTransactorSession) Initialize(_challengeFinality *big.Int) (*types.Transaction, error) {
	return _ProofVerifier.Contract.Initialize(&_ProofVerifier.TransactOpts, _challengeFinality)
}

// Migrate is a paid mutator transaction binding the contract method 0x8fd3ab80.
//
// Solidity: function migrate() returns()
func (_ProofVerifier *ProofVerifierTransactor) Migrate(opts *bind.TransactOpts) (*types.Transaction, error) {
	return _ProofVerifier.contract.Transact(opts, "migrate")
}

// Migrate is a paid mutator transaction binding the contract method 0x8fd3ab80.
//
// Solidity: function migrate() returns()
func (_ProofVerifier *ProofVerifierSession) Migrate() (*types.Transaction, error) {
	return _ProofVerifier.Contract.Migrate(&_ProofVerifier.TransactOpts)
}

// Migrate is a paid mutator transaction binding the contract method 0x8fd3ab80.
//
// Solidity: function migrate() returns()
func (_ProofVerifier *ProofVerifierTransactorSession) Migrate() (*types.Transaction, error) {
	return _ProofVerifier.Contract.Migrate(&_ProofVerifier.TransactOpts)
}

// NextProvingPeriod is a paid mutator transaction binding the contract method 0x45c0b92d.
//
// Solidity: function nextProvingPeriod(uint256 setId, uint256 challengeEpoch, bytes extraData) returns()
func (_ProofVerifier *ProofVerifierTransactor) NextProvingPeriod(opts *bind.TransactOpts, setId *big.Int, challengeEpoch *big.Int, extraData []byte) (*types.Transaction, error) {
	return _ProofVerifier.contract.Transact(opts, "nextProvingPeriod", setId, challengeEpoch, extraData)
}

// NextProvingPeriod is a paid mutator transaction binding the contract method 0x45c0b92d.
//
// Solidity: function nextProvingPeriod(uint256 setId, uint256 challengeEpoch, bytes extraData) returns()
func (_ProofVerifier *ProofVerifierSession) NextProvingPeriod(setId *big.Int, challengeEpoch *big.Int, extraData []byte) (*types.Transaction, error) {
	return _ProofVerifier.Contract.NextProvingPeriod(&_ProofVerifier.TransactOpts, setId, challengeEpoch, extraData)
}

// NextProvingPeriod is a paid mutator transaction binding the contract method 0x45c0b92d.
//
// Solidity: function nextProvingPeriod(uint256 setId, uint256 challengeEpoch, bytes extraData) returns()
func (_ProofVerifier *ProofVerifierTransactorSession) NextProvingPeriod(setId *big.Int, challengeEpoch *big.Int, extraData []byte) (*types.Transaction, error) {
	return _ProofVerifier.Contract.NextProvingPeriod(&_ProofVerifier.TransactOpts, setId, challengeEpoch, extraData)
}

// ProposeDataSetStorageProvider is a paid mutator transaction binding the contract method 0x43186080.
//
// Solidity: function proposeDataSetStorageProvider(uint256 setId, address newStorageProvider) returns()
func (_ProofVerifier *ProofVerifierTransactor) ProposeDataSetStorageProvider(opts *bind.TransactOpts, setId *big.Int, newStorageProvider common.Address) (*types.Transaction, error) {
	return _ProofVerifier.contract.Transact(opts, "proposeDataSetStorageProvider", setId, newStorageProvider)
}

// ProposeDataSetStorageProvider is a paid mutator transaction binding the contract method 0x43186080.
//
// Solidity: function proposeDataSetStorageProvider(uint256 setId, address newStorageProvider) returns()
func (_ProofVerifier *ProofVerifierSession) ProposeDataSetStorageProvider(setId *big.Int, newStorageProvider common.Address) (*types.Transaction, error) {
	return _ProofVerifier.Contract.ProposeDataSetStorageProvider(&_ProofVerifier.TransactOpts, setId, newStorageProvider)
}

// ProposeDataSetStorageProvider is a paid mutator transaction binding the contract method 0x43186080.
//
// Solidity: function proposeDataSetStorageProvider(uint256 setId, address newStorageProvider) returns()
func (_ProofVerifier *ProofVerifierTransactorSession) ProposeDataSetStorageProvider(setId *big.Int, newStorageProvider common.Address) (*types.Transaction, error) {
	return _ProofVerifier.Contract.ProposeDataSetStorageProvider(&_ProofVerifier.TransactOpts, setId, newStorageProvider)
}

// ProvePossession is a paid mutator transaction binding the contract method 0xf58f952b.
//
// Solidity: function provePossession(uint256 setId, (bytes32,bytes32[])[] proofs) payable returns()
func (_ProofVerifier *ProofVerifierTransactor) ProvePossession(opts *bind.TransactOpts, setId *big.Int, proofs []IPDPTypesProof) (*types.Transaction, error) {
	return _ProofVerifier.contract.Transact(opts, "provePossession", setId, proofs)
}

// ProvePossession is a paid mutator transaction binding the contract method 0xf58f952b.
//
// Solidity: function provePossession(uint256 setId, (bytes32,bytes32[])[] proofs) payable returns()
func (_ProofVerifier *ProofVerifierSession) ProvePossession(setId *big.Int, proofs []IPDPTypesProof) (*types.Transaction, error) {
	return _ProofVerifier.Contract.ProvePossession(&_ProofVerifier.TransactOpts, setId, proofs)
}

// ProvePossession is a paid mutator transaction binding the contract method 0xf58f952b.
//
// Solidity: function provePossession(uint256 setId, (bytes32,bytes32[])[] proofs) payable returns()
func (_ProofVerifier *ProofVerifierTransactorSession) ProvePossession(setId *big.Int, proofs []IPDPTypesProof) (*types.Transaction, error) {
	return _ProofVerifier.Contract.ProvePossession(&_ProofVerifier.TransactOpts, setId, proofs)
}

// RenounceOwnership is a paid mutator transaction binding the contract method 0x715018a6.
//
// Solidity: function renounceOwnership() returns()
func (_ProofVerifier *ProofVerifierTransactor) RenounceOwnership(opts *bind.TransactOpts) (*types.Transaction, error) {
	return _ProofVerifier.contract.Transact(opts, "renounceOwnership")
}

// RenounceOwnership is a paid mutator transaction binding the contract method 0x715018a6.
//
// Solidity: function renounceOwnership() returns()
func (_ProofVerifier *ProofVerifierSession) RenounceOwnership() (*types.Transaction, error) {
	return _ProofVerifier.Contract.RenounceOwnership(&_ProofVerifier.TransactOpts)
}

// RenounceOwnership is a paid mutator transaction binding the contract method 0x715018a6.
//
// Solidity: function renounceOwnership() returns()
func (_ProofVerifier *ProofVerifierTransactorSession) RenounceOwnership() (*types.Transaction, error) {
	return _ProofVerifier.Contract.RenounceOwnership(&_ProofVerifier.TransactOpts)
}

// SchedulePieceDeletions is a paid mutator transaction binding the contract method 0x0c292024.
//
// Solidity: function schedulePieceDeletions(uint256 setId, uint256[] pieceIds, bytes extraData) returns()
func (_ProofVerifier *ProofVerifierTransactor) SchedulePieceDeletions(opts *bind.TransactOpts, setId *big.Int, pieceIds []*big.Int, extraData []byte) (*types.Transaction, error) {
	return _ProofVerifier.contract.Transact(opts, "schedulePieceDeletions", setId, pieceIds, extraData)
}

// SchedulePieceDeletions is a paid mutator transaction binding the contract method 0x0c292024.
//
// Solidity: function schedulePieceDeletions(uint256 setId, uint256[] pieceIds, bytes extraData) returns()
func (_ProofVerifier *ProofVerifierSession) SchedulePieceDeletions(setId *big.Int, pieceIds []*big.Int, extraData []byte) (*types.Transaction, error) {
	return _ProofVerifier.Contract.SchedulePieceDeletions(&_ProofVerifier.TransactOpts, setId, pieceIds, extraData)
}

// SchedulePieceDeletions is a paid mutator transaction binding the contract method 0x0c292024.
//
// Solidity: function schedulePieceDeletions(uint256 setId, uint256[] pieceIds, bytes extraData) returns()
func (_ProofVerifier *ProofVerifierTransactorSession) SchedulePieceDeletions(setId *big.Int, pieceIds []*big.Int, extraData []byte) (*types.Transaction, error) {
	return _ProofVerifier.Contract.SchedulePieceDeletions(&_ProofVerifier.TransactOpts, setId, pieceIds, extraData)
}

// TransferOwnership is a paid mutator transaction binding the contract method 0xf2fde38b.
//
// Solidity: function transferOwnership(address newOwner) returns()
func (_ProofVerifier *ProofVerifierTransactor) TransferOwnership(opts *bind.TransactOpts, newOwner common.Address) (*types.Transaction, error) {
	return _ProofVerifier.contract.Transact(opts, "transferOwnership", newOwner)
}

// TransferOwnership is a paid mutator transaction binding the contract method 0xf2fde38b.
//
// Solidity: function transferOwnership(address newOwner) returns()
func (_ProofVerifier *ProofVerifierSession) TransferOwnership(newOwner common.Address) (*types.Transaction, error) {
	return _ProofVerifier.Contract.TransferOwnership(&_ProofVerifier.TransactOpts, newOwner)
}

// TransferOwnership is a paid mutator transaction binding the contract method 0xf2fde38b.
//
// Solidity: function transferOwnership(address newOwner) returns()
func (_ProofVerifier *ProofVerifierTransactorSession) TransferOwnership(newOwner common.Address) (*types.Transaction, error) {
	return _ProofVerifier.Contract.TransferOwnership(&_ProofVerifier.TransactOpts, newOwner)
}

// UpdateProofFee is a paid mutator transaction binding the contract method 0x46bf7ed3.
//
// Solidity: function updateProofFee(uint256 newFeePerTiB) returns()
func (_ProofVerifier *ProofVerifierTransactor) UpdateProofFee(opts *bind.TransactOpts, newFeePerTiB *big.Int) (*types.Transaction, error) {
	return _ProofVerifier.contract.Transact(opts, "updateProofFee", newFeePerTiB)
}

// UpdateProofFee is a paid mutator transaction binding the contract method 0x46bf7ed3.
//
// Solidity: function updateProofFee(uint256 newFeePerTiB) returns()
func (_ProofVerifier *ProofVerifierSession) UpdateProofFee(newFeePerTiB *big.Int) (*types.Transaction, error) {
	return _ProofVerifier.Contract.UpdateProofFee(&_ProofVerifier.TransactOpts, newFeePerTiB)
}

// UpdateProofFee is a paid mutator transaction binding the contract method 0x46bf7ed3.
//
// Solidity: function updateProofFee(uint256 newFeePerTiB) returns()
func (_ProofVerifier *ProofVerifierTransactorSession) UpdateProofFee(newFeePerTiB *big.Int) (*types.Transaction, error) {
	return _ProofVerifier.Contract.UpdateProofFee(&_ProofVerifier.TransactOpts, newFeePerTiB)
}

// UpgradeToAndCall is a paid mutator transaction binding the contract method 0x4f1ef286.
//
// Solidity: function upgradeToAndCall(address newImplementation, bytes data) payable returns()
func (_ProofVerifier *ProofVerifierTransactor) UpgradeToAndCall(opts *bind.TransactOpts, newImplementation common.Address, data []byte) (*types.Transaction, error) {
	return _ProofVerifier.contract.Transact(opts, "upgradeToAndCall", newImplementation, data)
}

// UpgradeToAndCall is a paid mutator transaction binding the contract method 0x4f1ef286.
//
// Solidity: function upgradeToAndCall(address newImplementation, bytes data) payable returns()
func (_ProofVerifier *ProofVerifierSession) UpgradeToAndCall(newImplementation common.Address, data []byte) (*types.Transaction, error) {
	return _ProofVerifier.Contract.UpgradeToAndCall(&_ProofVerifier.TransactOpts, newImplementation, data)
}

// UpgradeToAndCall is a paid mutator transaction binding the contract method 0x4f1ef286.
//
// Solidity: function upgradeToAndCall(address newImplementation, bytes data) payable returns()
func (_ProofVerifier *ProofVerifierTransactorSession) UpgradeToAndCall(newImplementation common.Address, data []byte) (*types.Transaction, error) {
	return _ProofVerifier.Contract.UpgradeToAndCall(&_ProofVerifier.TransactOpts, newImplementation, data)
}

// ProofVerifierContractUpgradedIterator is returned from FilterContractUpgraded and is used to iterate over the raw logs and unpacked data for ContractUpgraded events raised by the ProofVerifier contract.
type ProofVerifierContractUpgradedIterator struct {
	Event *ProofVerifierContractUpgraded // Event containing the contract specifics and raw log

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
func (it *ProofVerifierContractUpgradedIterator) Next() bool {
	// If the iterator failed, stop iterating
	if it.fail != nil {
		return false
	}
	// If the iterator completed, deliver directly whatever's available
	if it.done {
		select {
		case log := <-it.logs:
			it.Event = new(ProofVerifierContractUpgraded)
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
		it.Event = new(ProofVerifierContractUpgraded)
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
func (it *ProofVerifierContractUpgradedIterator) Error() error {
	return it.fail
}

// Close terminates the iteration process, releasing any pending underlying
// resources.
func (it *ProofVerifierContractUpgradedIterator) Close() error {
	it.sub.Unsubscribe()
	return nil
}

// ProofVerifierContractUpgraded represents a ContractUpgraded event raised by the ProofVerifier contract.
type ProofVerifierContractUpgraded struct {
	Version        string
	Implementation common.Address
	Raw            types.Log // Blockchain specific contextual infos
}

// FilterContractUpgraded is a free log retrieval operation binding the contract event 0x2b51ff7c4cc8e6fe1c72e9d9685b7d2a88a5d82ad3a644afbdceb0272c89c1c3.
//
// Solidity: event ContractUpgraded(string version, address implementation)
func (_ProofVerifier *ProofVerifierFilterer) FilterContractUpgraded(opts *bind.FilterOpts) (*ProofVerifierContractUpgradedIterator, error) {

	logs, sub, err := _ProofVerifier.contract.FilterLogs(opts, "ContractUpgraded")
	if err != nil {
		return nil, err
	}
	return &ProofVerifierContractUpgradedIterator{contract: _ProofVerifier.contract, event: "ContractUpgraded", logs: logs, sub: sub}, nil
}

// WatchContractUpgraded is a free log subscription operation binding the contract event 0x2b51ff7c4cc8e6fe1c72e9d9685b7d2a88a5d82ad3a644afbdceb0272c89c1c3.
//
// Solidity: event ContractUpgraded(string version, address implementation)
func (_ProofVerifier *ProofVerifierFilterer) WatchContractUpgraded(opts *bind.WatchOpts, sink chan<- *ProofVerifierContractUpgraded) (event.Subscription, error) {

	logs, sub, err := _ProofVerifier.contract.WatchLogs(opts, "ContractUpgraded")
	if err != nil {
		return nil, err
	}
	return event.NewSubscription(func(quit <-chan struct{}) error {
		defer sub.Unsubscribe()
		for {
			select {
			case log := <-logs:
				// New log arrived, parse the event and forward to the user
				event := new(ProofVerifierContractUpgraded)
				if err := _ProofVerifier.contract.UnpackLog(event, "ContractUpgraded", log); err != nil {
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

// ParseContractUpgraded is a log parse operation binding the contract event 0x2b51ff7c4cc8e6fe1c72e9d9685b7d2a88a5d82ad3a644afbdceb0272c89c1c3.
//
// Solidity: event ContractUpgraded(string version, address implementation)
func (_ProofVerifier *ProofVerifierFilterer) ParseContractUpgraded(log types.Log) (*ProofVerifierContractUpgraded, error) {
	event := new(ProofVerifierContractUpgraded)
	if err := _ProofVerifier.contract.UnpackLog(event, "ContractUpgraded", log); err != nil {
		return nil, err
	}
	event.Raw = log
	return event, nil
}

// ProofVerifierDataSetCreatedIterator is returned from FilterDataSetCreated and is used to iterate over the raw logs and unpacked data for DataSetCreated events raised by the ProofVerifier contract.
type ProofVerifierDataSetCreatedIterator struct {
	Event *ProofVerifierDataSetCreated // Event containing the contract specifics and raw log

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
func (it *ProofVerifierDataSetCreatedIterator) Next() bool {
	// If the iterator failed, stop iterating
	if it.fail != nil {
		return false
	}
	// If the iterator completed, deliver directly whatever's available
	if it.done {
		select {
		case log := <-it.logs:
			it.Event = new(ProofVerifierDataSetCreated)
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
		it.Event = new(ProofVerifierDataSetCreated)
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
func (it *ProofVerifierDataSetCreatedIterator) Error() error {
	return it.fail
}

// Close terminates the iteration process, releasing any pending underlying
// resources.
func (it *ProofVerifierDataSetCreatedIterator) Close() error {
	it.sub.Unsubscribe()
	return nil
}

// ProofVerifierDataSetCreated represents a DataSetCreated event raised by the ProofVerifier contract.
type ProofVerifierDataSetCreated struct {
	SetId           *big.Int
	StorageProvider common.Address
	Raw             types.Log // Blockchain specific contextual infos
}

// FilterDataSetCreated is a free log retrieval operation binding the contract event 0x11369440e1b7135015c16acb9bc14b55b0f4b23b02010c363d34aec2e5b96281.
//
// Solidity: event DataSetCreated(uint256 indexed setId, address indexed storageProvider)
func (_ProofVerifier *ProofVerifierFilterer) FilterDataSetCreated(opts *bind.FilterOpts, setId []*big.Int, storageProvider []common.Address) (*ProofVerifierDataSetCreatedIterator, error) {

	var setIdRule []interface{}
	for _, setIdItem := range setId {
		setIdRule = append(setIdRule, setIdItem)
	}
	var storageProviderRule []interface{}
	for _, storageProviderItem := range storageProvider {
		storageProviderRule = append(storageProviderRule, storageProviderItem)
	}

	logs, sub, err := _ProofVerifier.contract.FilterLogs(opts, "DataSetCreated", setIdRule, storageProviderRule)
	if err != nil {
		return nil, err
	}
	return &ProofVerifierDataSetCreatedIterator{contract: _ProofVerifier.contract, event: "DataSetCreated", logs: logs, sub: sub}, nil
}

// WatchDataSetCreated is a free log subscription operation binding the contract event 0x11369440e1b7135015c16acb9bc14b55b0f4b23b02010c363d34aec2e5b96281.
//
// Solidity: event DataSetCreated(uint256 indexed setId, address indexed storageProvider)
func (_ProofVerifier *ProofVerifierFilterer) WatchDataSetCreated(opts *bind.WatchOpts, sink chan<- *ProofVerifierDataSetCreated, setId []*big.Int, storageProvider []common.Address) (event.Subscription, error) {

	var setIdRule []interface{}
	for _, setIdItem := range setId {
		setIdRule = append(setIdRule, setIdItem)
	}
	var storageProviderRule []interface{}
	for _, storageProviderItem := range storageProvider {
		storageProviderRule = append(storageProviderRule, storageProviderItem)
	}

	logs, sub, err := _ProofVerifier.contract.WatchLogs(opts, "DataSetCreated", setIdRule, storageProviderRule)
	if err != nil {
		return nil, err
	}
	return event.NewSubscription(func(quit <-chan struct{}) error {
		defer sub.Unsubscribe()
		for {
			select {
			case log := <-logs:
				// New log arrived, parse the event and forward to the user
				event := new(ProofVerifierDataSetCreated)
				if err := _ProofVerifier.contract.UnpackLog(event, "DataSetCreated", log); err != nil {
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
func (_ProofVerifier *ProofVerifierFilterer) ParseDataSetCreated(log types.Log) (*ProofVerifierDataSetCreated, error) {
	event := new(ProofVerifierDataSetCreated)
	if err := _ProofVerifier.contract.UnpackLog(event, "DataSetCreated", log); err != nil {
		return nil, err
	}
	event.Raw = log
	return event, nil
}

// ProofVerifierDataSetDeletedIterator is returned from FilterDataSetDeleted and is used to iterate over the raw logs and unpacked data for DataSetDeleted events raised by the ProofVerifier contract.
type ProofVerifierDataSetDeletedIterator struct {
	Event *ProofVerifierDataSetDeleted // Event containing the contract specifics and raw log

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
func (it *ProofVerifierDataSetDeletedIterator) Next() bool {
	// If the iterator failed, stop iterating
	if it.fail != nil {
		return false
	}
	// If the iterator completed, deliver directly whatever's available
	if it.done {
		select {
		case log := <-it.logs:
			it.Event = new(ProofVerifierDataSetDeleted)
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
		it.Event = new(ProofVerifierDataSetDeleted)
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
func (it *ProofVerifierDataSetDeletedIterator) Error() error {
	return it.fail
}

// Close terminates the iteration process, releasing any pending underlying
// resources.
func (it *ProofVerifierDataSetDeletedIterator) Close() error {
	it.sub.Unsubscribe()
	return nil
}

// ProofVerifierDataSetDeleted represents a DataSetDeleted event raised by the ProofVerifier contract.
type ProofVerifierDataSetDeleted struct {
	SetId            *big.Int
	DeletedLeafCount *big.Int
	Raw              types.Log // Blockchain specific contextual infos
}

// FilterDataSetDeleted is a free log retrieval operation binding the contract event 0x14eeeef7679fcb051c6572811f61c07bedccd0f1cfc1f9b79b23e47c5c52aeb7.
//
// Solidity: event DataSetDeleted(uint256 indexed setId, uint256 deletedLeafCount)
func (_ProofVerifier *ProofVerifierFilterer) FilterDataSetDeleted(opts *bind.FilterOpts, setId []*big.Int) (*ProofVerifierDataSetDeletedIterator, error) {

	var setIdRule []interface{}
	for _, setIdItem := range setId {
		setIdRule = append(setIdRule, setIdItem)
	}

	logs, sub, err := _ProofVerifier.contract.FilterLogs(opts, "DataSetDeleted", setIdRule)
	if err != nil {
		return nil, err
	}
	return &ProofVerifierDataSetDeletedIterator{contract: _ProofVerifier.contract, event: "DataSetDeleted", logs: logs, sub: sub}, nil
}

// WatchDataSetDeleted is a free log subscription operation binding the contract event 0x14eeeef7679fcb051c6572811f61c07bedccd0f1cfc1f9b79b23e47c5c52aeb7.
//
// Solidity: event DataSetDeleted(uint256 indexed setId, uint256 deletedLeafCount)
func (_ProofVerifier *ProofVerifierFilterer) WatchDataSetDeleted(opts *bind.WatchOpts, sink chan<- *ProofVerifierDataSetDeleted, setId []*big.Int) (event.Subscription, error) {

	var setIdRule []interface{}
	for _, setIdItem := range setId {
		setIdRule = append(setIdRule, setIdItem)
	}

	logs, sub, err := _ProofVerifier.contract.WatchLogs(opts, "DataSetDeleted", setIdRule)
	if err != nil {
		return nil, err
	}
	return event.NewSubscription(func(quit <-chan struct{}) error {
		defer sub.Unsubscribe()
		for {
			select {
			case log := <-logs:
				// New log arrived, parse the event and forward to the user
				event := new(ProofVerifierDataSetDeleted)
				if err := _ProofVerifier.contract.UnpackLog(event, "DataSetDeleted", log); err != nil {
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

// ParseDataSetDeleted is a log parse operation binding the contract event 0x14eeeef7679fcb051c6572811f61c07bedccd0f1cfc1f9b79b23e47c5c52aeb7.
//
// Solidity: event DataSetDeleted(uint256 indexed setId, uint256 deletedLeafCount)
func (_ProofVerifier *ProofVerifierFilterer) ParseDataSetDeleted(log types.Log) (*ProofVerifierDataSetDeleted, error) {
	event := new(ProofVerifierDataSetDeleted)
	if err := _ProofVerifier.contract.UnpackLog(event, "DataSetDeleted", log); err != nil {
		return nil, err
	}
	event.Raw = log
	return event, nil
}

// ProofVerifierDataSetEmptyIterator is returned from FilterDataSetEmpty and is used to iterate over the raw logs and unpacked data for DataSetEmpty events raised by the ProofVerifier contract.
type ProofVerifierDataSetEmptyIterator struct {
	Event *ProofVerifierDataSetEmpty // Event containing the contract specifics and raw log

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
func (it *ProofVerifierDataSetEmptyIterator) Next() bool {
	// If the iterator failed, stop iterating
	if it.fail != nil {
		return false
	}
	// If the iterator completed, deliver directly whatever's available
	if it.done {
		select {
		case log := <-it.logs:
			it.Event = new(ProofVerifierDataSetEmpty)
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
		it.Event = new(ProofVerifierDataSetEmpty)
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
func (it *ProofVerifierDataSetEmptyIterator) Error() error {
	return it.fail
}

// Close terminates the iteration process, releasing any pending underlying
// resources.
func (it *ProofVerifierDataSetEmptyIterator) Close() error {
	it.sub.Unsubscribe()
	return nil
}

// ProofVerifierDataSetEmpty represents a DataSetEmpty event raised by the ProofVerifier contract.
type ProofVerifierDataSetEmpty struct {
	SetId *big.Int
	Raw   types.Log // Blockchain specific contextual infos
}

// FilterDataSetEmpty is a free log retrieval operation binding the contract event 0x02a8400fc343f45098cb00c3a6ea694174771939a5503f663e0ff6f4eb7c2842.
//
// Solidity: event DataSetEmpty(uint256 indexed setId)
func (_ProofVerifier *ProofVerifierFilterer) FilterDataSetEmpty(opts *bind.FilterOpts, setId []*big.Int) (*ProofVerifierDataSetEmptyIterator, error) {

	var setIdRule []interface{}
	for _, setIdItem := range setId {
		setIdRule = append(setIdRule, setIdItem)
	}

	logs, sub, err := _ProofVerifier.contract.FilterLogs(opts, "DataSetEmpty", setIdRule)
	if err != nil {
		return nil, err
	}
	return &ProofVerifierDataSetEmptyIterator{contract: _ProofVerifier.contract, event: "DataSetEmpty", logs: logs, sub: sub}, nil
}

// WatchDataSetEmpty is a free log subscription operation binding the contract event 0x02a8400fc343f45098cb00c3a6ea694174771939a5503f663e0ff6f4eb7c2842.
//
// Solidity: event DataSetEmpty(uint256 indexed setId)
func (_ProofVerifier *ProofVerifierFilterer) WatchDataSetEmpty(opts *bind.WatchOpts, sink chan<- *ProofVerifierDataSetEmpty, setId []*big.Int) (event.Subscription, error) {

	var setIdRule []interface{}
	for _, setIdItem := range setId {
		setIdRule = append(setIdRule, setIdItem)
	}

	logs, sub, err := _ProofVerifier.contract.WatchLogs(opts, "DataSetEmpty", setIdRule)
	if err != nil {
		return nil, err
	}
	return event.NewSubscription(func(quit <-chan struct{}) error {
		defer sub.Unsubscribe()
		for {
			select {
			case log := <-logs:
				// New log arrived, parse the event and forward to the user
				event := new(ProofVerifierDataSetEmpty)
				if err := _ProofVerifier.contract.UnpackLog(event, "DataSetEmpty", log); err != nil {
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

// ParseDataSetEmpty is a log parse operation binding the contract event 0x02a8400fc343f45098cb00c3a6ea694174771939a5503f663e0ff6f4eb7c2842.
//
// Solidity: event DataSetEmpty(uint256 indexed setId)
func (_ProofVerifier *ProofVerifierFilterer) ParseDataSetEmpty(log types.Log) (*ProofVerifierDataSetEmpty, error) {
	event := new(ProofVerifierDataSetEmpty)
	if err := _ProofVerifier.contract.UnpackLog(event, "DataSetEmpty", log); err != nil {
		return nil, err
	}
	event.Raw = log
	return event, nil
}

// ProofVerifierFeeUpdateProposedIterator is returned from FilterFeeUpdateProposed and is used to iterate over the raw logs and unpacked data for FeeUpdateProposed events raised by the ProofVerifier contract.
type ProofVerifierFeeUpdateProposedIterator struct {
	Event *ProofVerifierFeeUpdateProposed // Event containing the contract specifics and raw log

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
func (it *ProofVerifierFeeUpdateProposedIterator) Next() bool {
	// If the iterator failed, stop iterating
	if it.fail != nil {
		return false
	}
	// If the iterator completed, deliver directly whatever's available
	if it.done {
		select {
		case log := <-it.logs:
			it.Event = new(ProofVerifierFeeUpdateProposed)
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
		it.Event = new(ProofVerifierFeeUpdateProposed)
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
func (it *ProofVerifierFeeUpdateProposedIterator) Error() error {
	return it.fail
}

// Close terminates the iteration process, releasing any pending underlying
// resources.
func (it *ProofVerifierFeeUpdateProposedIterator) Close() error {
	it.sub.Unsubscribe()
	return nil
}

// ProofVerifierFeeUpdateProposed represents a FeeUpdateProposed event raised by the ProofVerifier contract.
type ProofVerifierFeeUpdateProposed struct {
	CurrentFee    *big.Int
	NewFee        *big.Int
	EffectiveTime *big.Int
	Raw           types.Log // Blockchain specific contextual infos
}

// FilterFeeUpdateProposed is a free log retrieval operation binding the contract event 0x239c396012e4038117d18910fba2aab3452e37696f685a457098e4c4864d8bcb.
//
// Solidity: event FeeUpdateProposed(uint256 currentFee, uint256 newFee, uint256 effectiveTime)
func (_ProofVerifier *ProofVerifierFilterer) FilterFeeUpdateProposed(opts *bind.FilterOpts) (*ProofVerifierFeeUpdateProposedIterator, error) {

	logs, sub, err := _ProofVerifier.contract.FilterLogs(opts, "FeeUpdateProposed")
	if err != nil {
		return nil, err
	}
	return &ProofVerifierFeeUpdateProposedIterator{contract: _ProofVerifier.contract, event: "FeeUpdateProposed", logs: logs, sub: sub}, nil
}

// WatchFeeUpdateProposed is a free log subscription operation binding the contract event 0x239c396012e4038117d18910fba2aab3452e37696f685a457098e4c4864d8bcb.
//
// Solidity: event FeeUpdateProposed(uint256 currentFee, uint256 newFee, uint256 effectiveTime)
func (_ProofVerifier *ProofVerifierFilterer) WatchFeeUpdateProposed(opts *bind.WatchOpts, sink chan<- *ProofVerifierFeeUpdateProposed) (event.Subscription, error) {

	logs, sub, err := _ProofVerifier.contract.WatchLogs(opts, "FeeUpdateProposed")
	if err != nil {
		return nil, err
	}
	return event.NewSubscription(func(quit <-chan struct{}) error {
		defer sub.Unsubscribe()
		for {
			select {
			case log := <-logs:
				// New log arrived, parse the event and forward to the user
				event := new(ProofVerifierFeeUpdateProposed)
				if err := _ProofVerifier.contract.UnpackLog(event, "FeeUpdateProposed", log); err != nil {
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

// ParseFeeUpdateProposed is a log parse operation binding the contract event 0x239c396012e4038117d18910fba2aab3452e37696f685a457098e4c4864d8bcb.
//
// Solidity: event FeeUpdateProposed(uint256 currentFee, uint256 newFee, uint256 effectiveTime)
func (_ProofVerifier *ProofVerifierFilterer) ParseFeeUpdateProposed(log types.Log) (*ProofVerifierFeeUpdateProposed, error) {
	event := new(ProofVerifierFeeUpdateProposed)
	if err := _ProofVerifier.contract.UnpackLog(event, "FeeUpdateProposed", log); err != nil {
		return nil, err
	}
	event.Raw = log
	return event, nil
}

// ProofVerifierInitializedIterator is returned from FilterInitialized and is used to iterate over the raw logs and unpacked data for Initialized events raised by the ProofVerifier contract.
type ProofVerifierInitializedIterator struct {
	Event *ProofVerifierInitialized // Event containing the contract specifics and raw log

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
func (it *ProofVerifierInitializedIterator) Next() bool {
	// If the iterator failed, stop iterating
	if it.fail != nil {
		return false
	}
	// If the iterator completed, deliver directly whatever's available
	if it.done {
		select {
		case log := <-it.logs:
			it.Event = new(ProofVerifierInitialized)
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
		it.Event = new(ProofVerifierInitialized)
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
func (it *ProofVerifierInitializedIterator) Error() error {
	return it.fail
}

// Close terminates the iteration process, releasing any pending underlying
// resources.
func (it *ProofVerifierInitializedIterator) Close() error {
	it.sub.Unsubscribe()
	return nil
}

// ProofVerifierInitialized represents a Initialized event raised by the ProofVerifier contract.
type ProofVerifierInitialized struct {
	Version uint64
	Raw     types.Log // Blockchain specific contextual infos
}

// FilterInitialized is a free log retrieval operation binding the contract event 0xc7f505b2f371ae2175ee4913f4499e1f2633a7b5936321eed1cdaeb6115181d2.
//
// Solidity: event Initialized(uint64 version)
func (_ProofVerifier *ProofVerifierFilterer) FilterInitialized(opts *bind.FilterOpts) (*ProofVerifierInitializedIterator, error) {

	logs, sub, err := _ProofVerifier.contract.FilterLogs(opts, "Initialized")
	if err != nil {
		return nil, err
	}
	return &ProofVerifierInitializedIterator{contract: _ProofVerifier.contract, event: "Initialized", logs: logs, sub: sub}, nil
}

// WatchInitialized is a free log subscription operation binding the contract event 0xc7f505b2f371ae2175ee4913f4499e1f2633a7b5936321eed1cdaeb6115181d2.
//
// Solidity: event Initialized(uint64 version)
func (_ProofVerifier *ProofVerifierFilterer) WatchInitialized(opts *bind.WatchOpts, sink chan<- *ProofVerifierInitialized) (event.Subscription, error) {

	logs, sub, err := _ProofVerifier.contract.WatchLogs(opts, "Initialized")
	if err != nil {
		return nil, err
	}
	return event.NewSubscription(func(quit <-chan struct{}) error {
		defer sub.Unsubscribe()
		for {
			select {
			case log := <-logs:
				// New log arrived, parse the event and forward to the user
				event := new(ProofVerifierInitialized)
				if err := _ProofVerifier.contract.UnpackLog(event, "Initialized", log); err != nil {
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

// ParseInitialized is a log parse operation binding the contract event 0xc7f505b2f371ae2175ee4913f4499e1f2633a7b5936321eed1cdaeb6115181d2.
//
// Solidity: event Initialized(uint64 version)
func (_ProofVerifier *ProofVerifierFilterer) ParseInitialized(log types.Log) (*ProofVerifierInitialized, error) {
	event := new(ProofVerifierInitialized)
	if err := _ProofVerifier.contract.UnpackLog(event, "Initialized", log); err != nil {
		return nil, err
	}
	event.Raw = log
	return event, nil
}

// ProofVerifierNextProvingPeriodIterator is returned from FilterNextProvingPeriod and is used to iterate over the raw logs and unpacked data for NextProvingPeriod events raised by the ProofVerifier contract.
type ProofVerifierNextProvingPeriodIterator struct {
	Event *ProofVerifierNextProvingPeriod // Event containing the contract specifics and raw log

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
func (it *ProofVerifierNextProvingPeriodIterator) Next() bool {
	// If the iterator failed, stop iterating
	if it.fail != nil {
		return false
	}
	// If the iterator completed, deliver directly whatever's available
	if it.done {
		select {
		case log := <-it.logs:
			it.Event = new(ProofVerifierNextProvingPeriod)
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
		it.Event = new(ProofVerifierNextProvingPeriod)
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
func (it *ProofVerifierNextProvingPeriodIterator) Error() error {
	return it.fail
}

// Close terminates the iteration process, releasing any pending underlying
// resources.
func (it *ProofVerifierNextProvingPeriodIterator) Close() error {
	it.sub.Unsubscribe()
	return nil
}

// ProofVerifierNextProvingPeriod represents a NextProvingPeriod event raised by the ProofVerifier contract.
type ProofVerifierNextProvingPeriod struct {
	SetId          *big.Int
	ChallengeEpoch *big.Int
	LeafCount      *big.Int
	Raw            types.Log // Blockchain specific contextual infos
}

// FilterNextProvingPeriod is a free log retrieval operation binding the contract event 0xc099ffec4e3e773644a4d1dda368c46af853a0eeb15babde217f53a657396e1e.
//
// Solidity: event NextProvingPeriod(uint256 indexed setId, uint256 challengeEpoch, uint256 leafCount)
func (_ProofVerifier *ProofVerifierFilterer) FilterNextProvingPeriod(opts *bind.FilterOpts, setId []*big.Int) (*ProofVerifierNextProvingPeriodIterator, error) {

	var setIdRule []interface{}
	for _, setIdItem := range setId {
		setIdRule = append(setIdRule, setIdItem)
	}

	logs, sub, err := _ProofVerifier.contract.FilterLogs(opts, "NextProvingPeriod", setIdRule)
	if err != nil {
		return nil, err
	}
	return &ProofVerifierNextProvingPeriodIterator{contract: _ProofVerifier.contract, event: "NextProvingPeriod", logs: logs, sub: sub}, nil
}

// WatchNextProvingPeriod is a free log subscription operation binding the contract event 0xc099ffec4e3e773644a4d1dda368c46af853a0eeb15babde217f53a657396e1e.
//
// Solidity: event NextProvingPeriod(uint256 indexed setId, uint256 challengeEpoch, uint256 leafCount)
func (_ProofVerifier *ProofVerifierFilterer) WatchNextProvingPeriod(opts *bind.WatchOpts, sink chan<- *ProofVerifierNextProvingPeriod, setId []*big.Int) (event.Subscription, error) {

	var setIdRule []interface{}
	for _, setIdItem := range setId {
		setIdRule = append(setIdRule, setIdItem)
	}

	logs, sub, err := _ProofVerifier.contract.WatchLogs(opts, "NextProvingPeriod", setIdRule)
	if err != nil {
		return nil, err
	}
	return event.NewSubscription(func(quit <-chan struct{}) error {
		defer sub.Unsubscribe()
		for {
			select {
			case log := <-logs:
				// New log arrived, parse the event and forward to the user
				event := new(ProofVerifierNextProvingPeriod)
				if err := _ProofVerifier.contract.UnpackLog(event, "NextProvingPeriod", log); err != nil {
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

// ParseNextProvingPeriod is a log parse operation binding the contract event 0xc099ffec4e3e773644a4d1dda368c46af853a0eeb15babde217f53a657396e1e.
//
// Solidity: event NextProvingPeriod(uint256 indexed setId, uint256 challengeEpoch, uint256 leafCount)
func (_ProofVerifier *ProofVerifierFilterer) ParseNextProvingPeriod(log types.Log) (*ProofVerifierNextProvingPeriod, error) {
	event := new(ProofVerifierNextProvingPeriod)
	if err := _ProofVerifier.contract.UnpackLog(event, "NextProvingPeriod", log); err != nil {
		return nil, err
	}
	event.Raw = log
	return event, nil
}

// ProofVerifierOwnershipTransferredIterator is returned from FilterOwnershipTransferred and is used to iterate over the raw logs and unpacked data for OwnershipTransferred events raised by the ProofVerifier contract.
type ProofVerifierOwnershipTransferredIterator struct {
	Event *ProofVerifierOwnershipTransferred // Event containing the contract specifics and raw log

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
func (it *ProofVerifierOwnershipTransferredIterator) Next() bool {
	// If the iterator failed, stop iterating
	if it.fail != nil {
		return false
	}
	// If the iterator completed, deliver directly whatever's available
	if it.done {
		select {
		case log := <-it.logs:
			it.Event = new(ProofVerifierOwnershipTransferred)
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
		it.Event = new(ProofVerifierOwnershipTransferred)
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
func (it *ProofVerifierOwnershipTransferredIterator) Error() error {
	return it.fail
}

// Close terminates the iteration process, releasing any pending underlying
// resources.
func (it *ProofVerifierOwnershipTransferredIterator) Close() error {
	it.sub.Unsubscribe()
	return nil
}

// ProofVerifierOwnershipTransferred represents a OwnershipTransferred event raised by the ProofVerifier contract.
type ProofVerifierOwnershipTransferred struct {
	PreviousOwner common.Address
	NewOwner      common.Address
	Raw           types.Log // Blockchain specific contextual infos
}

// FilterOwnershipTransferred is a free log retrieval operation binding the contract event 0x8be0079c531659141344cd1fd0a4f28419497f9722a3daafe3b4186f6b6457e0.
//
// Solidity: event OwnershipTransferred(address indexed previousOwner, address indexed newOwner)
func (_ProofVerifier *ProofVerifierFilterer) FilterOwnershipTransferred(opts *bind.FilterOpts, previousOwner []common.Address, newOwner []common.Address) (*ProofVerifierOwnershipTransferredIterator, error) {

	var previousOwnerRule []interface{}
	for _, previousOwnerItem := range previousOwner {
		previousOwnerRule = append(previousOwnerRule, previousOwnerItem)
	}
	var newOwnerRule []interface{}
	for _, newOwnerItem := range newOwner {
		newOwnerRule = append(newOwnerRule, newOwnerItem)
	}

	logs, sub, err := _ProofVerifier.contract.FilterLogs(opts, "OwnershipTransferred", previousOwnerRule, newOwnerRule)
	if err != nil {
		return nil, err
	}
	return &ProofVerifierOwnershipTransferredIterator{contract: _ProofVerifier.contract, event: "OwnershipTransferred", logs: logs, sub: sub}, nil
}

// WatchOwnershipTransferred is a free log subscription operation binding the contract event 0x8be0079c531659141344cd1fd0a4f28419497f9722a3daafe3b4186f6b6457e0.
//
// Solidity: event OwnershipTransferred(address indexed previousOwner, address indexed newOwner)
func (_ProofVerifier *ProofVerifierFilterer) WatchOwnershipTransferred(opts *bind.WatchOpts, sink chan<- *ProofVerifierOwnershipTransferred, previousOwner []common.Address, newOwner []common.Address) (event.Subscription, error) {

	var previousOwnerRule []interface{}
	for _, previousOwnerItem := range previousOwner {
		previousOwnerRule = append(previousOwnerRule, previousOwnerItem)
	}
	var newOwnerRule []interface{}
	for _, newOwnerItem := range newOwner {
		newOwnerRule = append(newOwnerRule, newOwnerItem)
	}

	logs, sub, err := _ProofVerifier.contract.WatchLogs(opts, "OwnershipTransferred", previousOwnerRule, newOwnerRule)
	if err != nil {
		return nil, err
	}
	return event.NewSubscription(func(quit <-chan struct{}) error {
		defer sub.Unsubscribe()
		for {
			select {
			case log := <-logs:
				// New log arrived, parse the event and forward to the user
				event := new(ProofVerifierOwnershipTransferred)
				if err := _ProofVerifier.contract.UnpackLog(event, "OwnershipTransferred", log); err != nil {
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
func (_ProofVerifier *ProofVerifierFilterer) ParseOwnershipTransferred(log types.Log) (*ProofVerifierOwnershipTransferred, error) {
	event := new(ProofVerifierOwnershipTransferred)
	if err := _ProofVerifier.contract.UnpackLog(event, "OwnershipTransferred", log); err != nil {
		return nil, err
	}
	event.Raw = log
	return event, nil
}

// ProofVerifierPiecesAddedIterator is returned from FilterPiecesAdded and is used to iterate over the raw logs and unpacked data for PiecesAdded events raised by the ProofVerifier contract.
type ProofVerifierPiecesAddedIterator struct {
	Event *ProofVerifierPiecesAdded // Event containing the contract specifics and raw log

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
func (it *ProofVerifierPiecesAddedIterator) Next() bool {
	// If the iterator failed, stop iterating
	if it.fail != nil {
		return false
	}
	// If the iterator completed, deliver directly whatever's available
	if it.done {
		select {
		case log := <-it.logs:
			it.Event = new(ProofVerifierPiecesAdded)
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
		it.Event = new(ProofVerifierPiecesAdded)
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
func (it *ProofVerifierPiecesAddedIterator) Error() error {
	return it.fail
}

// Close terminates the iteration process, releasing any pending underlying
// resources.
func (it *ProofVerifierPiecesAddedIterator) Close() error {
	it.sub.Unsubscribe()
	return nil
}

// ProofVerifierPiecesAdded represents a PiecesAdded event raised by the ProofVerifier contract.
type ProofVerifierPiecesAdded struct {
	SetId     *big.Int
	PieceIds  []*big.Int
	PieceCids []CidsCid
	Raw       types.Log // Blockchain specific contextual infos
}

// FilterPiecesAdded is a free log retrieval operation binding the contract event 0x396df50222a87662e94bb7d173792d5e61fe0b193b6ccf791f7ce433f0b28207.
//
// Solidity: event PiecesAdded(uint256 indexed setId, uint256[] pieceIds, (bytes)[] pieceCids)
func (_ProofVerifier *ProofVerifierFilterer) FilterPiecesAdded(opts *bind.FilterOpts, setId []*big.Int) (*ProofVerifierPiecesAddedIterator, error) {

	var setIdRule []interface{}
	for _, setIdItem := range setId {
		setIdRule = append(setIdRule, setIdItem)
	}

	logs, sub, err := _ProofVerifier.contract.FilterLogs(opts, "PiecesAdded", setIdRule)
	if err != nil {
		return nil, err
	}
	return &ProofVerifierPiecesAddedIterator{contract: _ProofVerifier.contract, event: "PiecesAdded", logs: logs, sub: sub}, nil
}

// WatchPiecesAdded is a free log subscription operation binding the contract event 0x396df50222a87662e94bb7d173792d5e61fe0b193b6ccf791f7ce433f0b28207.
//
// Solidity: event PiecesAdded(uint256 indexed setId, uint256[] pieceIds, (bytes)[] pieceCids)
func (_ProofVerifier *ProofVerifierFilterer) WatchPiecesAdded(opts *bind.WatchOpts, sink chan<- *ProofVerifierPiecesAdded, setId []*big.Int) (event.Subscription, error) {

	var setIdRule []interface{}
	for _, setIdItem := range setId {
		setIdRule = append(setIdRule, setIdItem)
	}

	logs, sub, err := _ProofVerifier.contract.WatchLogs(opts, "PiecesAdded", setIdRule)
	if err != nil {
		return nil, err
	}
	return event.NewSubscription(func(quit <-chan struct{}) error {
		defer sub.Unsubscribe()
		for {
			select {
			case log := <-logs:
				// New log arrived, parse the event and forward to the user
				event := new(ProofVerifierPiecesAdded)
				if err := _ProofVerifier.contract.UnpackLog(event, "PiecesAdded", log); err != nil {
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

// ParsePiecesAdded is a log parse operation binding the contract event 0x396df50222a87662e94bb7d173792d5e61fe0b193b6ccf791f7ce433f0b28207.
//
// Solidity: event PiecesAdded(uint256 indexed setId, uint256[] pieceIds, (bytes)[] pieceCids)
func (_ProofVerifier *ProofVerifierFilterer) ParsePiecesAdded(log types.Log) (*ProofVerifierPiecesAdded, error) {
	event := new(ProofVerifierPiecesAdded)
	if err := _ProofVerifier.contract.UnpackLog(event, "PiecesAdded", log); err != nil {
		return nil, err
	}
	event.Raw = log
	return event, nil
}

// ProofVerifierPiecesRemovedIterator is returned from FilterPiecesRemoved and is used to iterate over the raw logs and unpacked data for PiecesRemoved events raised by the ProofVerifier contract.
type ProofVerifierPiecesRemovedIterator struct {
	Event *ProofVerifierPiecesRemoved // Event containing the contract specifics and raw log

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
func (it *ProofVerifierPiecesRemovedIterator) Next() bool {
	// If the iterator failed, stop iterating
	if it.fail != nil {
		return false
	}
	// If the iterator completed, deliver directly whatever's available
	if it.done {
		select {
		case log := <-it.logs:
			it.Event = new(ProofVerifierPiecesRemoved)
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
		it.Event = new(ProofVerifierPiecesRemoved)
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
func (it *ProofVerifierPiecesRemovedIterator) Error() error {
	return it.fail
}

// Close terminates the iteration process, releasing any pending underlying
// resources.
func (it *ProofVerifierPiecesRemovedIterator) Close() error {
	it.sub.Unsubscribe()
	return nil
}

// ProofVerifierPiecesRemoved represents a PiecesRemoved event raised by the ProofVerifier contract.
type ProofVerifierPiecesRemoved struct {
	SetId    *big.Int
	PieceIds []*big.Int
	Raw      types.Log // Blockchain specific contextual infos
}

// FilterPiecesRemoved is a free log retrieval operation binding the contract event 0x6e87df804629ac17804b57ba7abbdfac8bdc36bab504fb8a8801eb313a8ce7b1.
//
// Solidity: event PiecesRemoved(uint256 indexed setId, uint256[] pieceIds)
func (_ProofVerifier *ProofVerifierFilterer) FilterPiecesRemoved(opts *bind.FilterOpts, setId []*big.Int) (*ProofVerifierPiecesRemovedIterator, error) {

	var setIdRule []interface{}
	for _, setIdItem := range setId {
		setIdRule = append(setIdRule, setIdItem)
	}

	logs, sub, err := _ProofVerifier.contract.FilterLogs(opts, "PiecesRemoved", setIdRule)
	if err != nil {
		return nil, err
	}
	return &ProofVerifierPiecesRemovedIterator{contract: _ProofVerifier.contract, event: "PiecesRemoved", logs: logs, sub: sub}, nil
}

// WatchPiecesRemoved is a free log subscription operation binding the contract event 0x6e87df804629ac17804b57ba7abbdfac8bdc36bab504fb8a8801eb313a8ce7b1.
//
// Solidity: event PiecesRemoved(uint256 indexed setId, uint256[] pieceIds)
func (_ProofVerifier *ProofVerifierFilterer) WatchPiecesRemoved(opts *bind.WatchOpts, sink chan<- *ProofVerifierPiecesRemoved, setId []*big.Int) (event.Subscription, error) {

	var setIdRule []interface{}
	for _, setIdItem := range setId {
		setIdRule = append(setIdRule, setIdItem)
	}

	logs, sub, err := _ProofVerifier.contract.WatchLogs(opts, "PiecesRemoved", setIdRule)
	if err != nil {
		return nil, err
	}
	return event.NewSubscription(func(quit <-chan struct{}) error {
		defer sub.Unsubscribe()
		for {
			select {
			case log := <-logs:
				// New log arrived, parse the event and forward to the user
				event := new(ProofVerifierPiecesRemoved)
				if err := _ProofVerifier.contract.UnpackLog(event, "PiecesRemoved", log); err != nil {
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

// ParsePiecesRemoved is a log parse operation binding the contract event 0x6e87df804629ac17804b57ba7abbdfac8bdc36bab504fb8a8801eb313a8ce7b1.
//
// Solidity: event PiecesRemoved(uint256 indexed setId, uint256[] pieceIds)
func (_ProofVerifier *ProofVerifierFilterer) ParsePiecesRemoved(log types.Log) (*ProofVerifierPiecesRemoved, error) {
	event := new(ProofVerifierPiecesRemoved)
	if err := _ProofVerifier.contract.UnpackLog(event, "PiecesRemoved", log); err != nil {
		return nil, err
	}
	event.Raw = log
	return event, nil
}

// ProofVerifierPossessionProvenIterator is returned from FilterPossessionProven and is used to iterate over the raw logs and unpacked data for PossessionProven events raised by the ProofVerifier contract.
type ProofVerifierPossessionProvenIterator struct {
	Event *ProofVerifierPossessionProven // Event containing the contract specifics and raw log

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
func (it *ProofVerifierPossessionProvenIterator) Next() bool {
	// If the iterator failed, stop iterating
	if it.fail != nil {
		return false
	}
	// If the iterator completed, deliver directly whatever's available
	if it.done {
		select {
		case log := <-it.logs:
			it.Event = new(ProofVerifierPossessionProven)
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
		it.Event = new(ProofVerifierPossessionProven)
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
func (it *ProofVerifierPossessionProvenIterator) Error() error {
	return it.fail
}

// Close terminates the iteration process, releasing any pending underlying
// resources.
func (it *ProofVerifierPossessionProvenIterator) Close() error {
	it.sub.Unsubscribe()
	return nil
}

// ProofVerifierPossessionProven represents a PossessionProven event raised by the ProofVerifier contract.
type ProofVerifierPossessionProven struct {
	SetId      *big.Int
	Challenges []IPDPTypesPieceIdAndOffset
	Raw        types.Log // Blockchain specific contextual infos
}

// FilterPossessionProven is a free log retrieval operation binding the contract event 0x1acf7df9f0c1b0208c23be6178950c0273f89b766805a2c0bd1e53d25c700e50.
//
// Solidity: event PossessionProven(uint256 indexed setId, (uint256,uint256)[] challenges)
func (_ProofVerifier *ProofVerifierFilterer) FilterPossessionProven(opts *bind.FilterOpts, setId []*big.Int) (*ProofVerifierPossessionProvenIterator, error) {

	var setIdRule []interface{}
	for _, setIdItem := range setId {
		setIdRule = append(setIdRule, setIdItem)
	}

	logs, sub, err := _ProofVerifier.contract.FilterLogs(opts, "PossessionProven", setIdRule)
	if err != nil {
		return nil, err
	}
	return &ProofVerifierPossessionProvenIterator{contract: _ProofVerifier.contract, event: "PossessionProven", logs: logs, sub: sub}, nil
}

// WatchPossessionProven is a free log subscription operation binding the contract event 0x1acf7df9f0c1b0208c23be6178950c0273f89b766805a2c0bd1e53d25c700e50.
//
// Solidity: event PossessionProven(uint256 indexed setId, (uint256,uint256)[] challenges)
func (_ProofVerifier *ProofVerifierFilterer) WatchPossessionProven(opts *bind.WatchOpts, sink chan<- *ProofVerifierPossessionProven, setId []*big.Int) (event.Subscription, error) {

	var setIdRule []interface{}
	for _, setIdItem := range setId {
		setIdRule = append(setIdRule, setIdItem)
	}

	logs, sub, err := _ProofVerifier.contract.WatchLogs(opts, "PossessionProven", setIdRule)
	if err != nil {
		return nil, err
	}
	return event.NewSubscription(func(quit <-chan struct{}) error {
		defer sub.Unsubscribe()
		for {
			select {
			case log := <-logs:
				// New log arrived, parse the event and forward to the user
				event := new(ProofVerifierPossessionProven)
				if err := _ProofVerifier.contract.UnpackLog(event, "PossessionProven", log); err != nil {
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

// ParsePossessionProven is a log parse operation binding the contract event 0x1acf7df9f0c1b0208c23be6178950c0273f89b766805a2c0bd1e53d25c700e50.
//
// Solidity: event PossessionProven(uint256 indexed setId, (uint256,uint256)[] challenges)
func (_ProofVerifier *ProofVerifierFilterer) ParsePossessionProven(log types.Log) (*ProofVerifierPossessionProven, error) {
	event := new(ProofVerifierPossessionProven)
	if err := _ProofVerifier.contract.UnpackLog(event, "PossessionProven", log); err != nil {
		return nil, err
	}
	event.Raw = log
	return event, nil
}

// ProofVerifierProofFeePaidIterator is returned from FilterProofFeePaid and is used to iterate over the raw logs and unpacked data for ProofFeePaid events raised by the ProofVerifier contract.
type ProofVerifierProofFeePaidIterator struct {
	Event *ProofVerifierProofFeePaid // Event containing the contract specifics and raw log

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
func (it *ProofVerifierProofFeePaidIterator) Next() bool {
	// If the iterator failed, stop iterating
	if it.fail != nil {
		return false
	}
	// If the iterator completed, deliver directly whatever's available
	if it.done {
		select {
		case log := <-it.logs:
			it.Event = new(ProofVerifierProofFeePaid)
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
		it.Event = new(ProofVerifierProofFeePaid)
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
func (it *ProofVerifierProofFeePaidIterator) Error() error {
	return it.fail
}

// Close terminates the iteration process, releasing any pending underlying
// resources.
func (it *ProofVerifierProofFeePaidIterator) Close() error {
	it.sub.Unsubscribe()
	return nil
}

// ProofVerifierProofFeePaid represents a ProofFeePaid event raised by the ProofVerifier contract.
type ProofVerifierProofFeePaid struct {
	SetId *big.Int
	Fee   *big.Int
	Raw   types.Log // Blockchain specific contextual infos
}

// FilterProofFeePaid is a free log retrieval operation binding the contract event 0x58b7742b13c8873fc0ba58f695b33ca0044b2db7ff9c5208181dbaec2a5b291e.
//
// Solidity: event ProofFeePaid(uint256 indexed setId, uint256 fee)
func (_ProofVerifier *ProofVerifierFilterer) FilterProofFeePaid(opts *bind.FilterOpts, setId []*big.Int) (*ProofVerifierProofFeePaidIterator, error) {

	var setIdRule []interface{}
	for _, setIdItem := range setId {
		setIdRule = append(setIdRule, setIdItem)
	}

	logs, sub, err := _ProofVerifier.contract.FilterLogs(opts, "ProofFeePaid", setIdRule)
	if err != nil {
		return nil, err
	}
	return &ProofVerifierProofFeePaidIterator{contract: _ProofVerifier.contract, event: "ProofFeePaid", logs: logs, sub: sub}, nil
}

// WatchProofFeePaid is a free log subscription operation binding the contract event 0x58b7742b13c8873fc0ba58f695b33ca0044b2db7ff9c5208181dbaec2a5b291e.
//
// Solidity: event ProofFeePaid(uint256 indexed setId, uint256 fee)
func (_ProofVerifier *ProofVerifierFilterer) WatchProofFeePaid(opts *bind.WatchOpts, sink chan<- *ProofVerifierProofFeePaid, setId []*big.Int) (event.Subscription, error) {

	var setIdRule []interface{}
	for _, setIdItem := range setId {
		setIdRule = append(setIdRule, setIdItem)
	}

	logs, sub, err := _ProofVerifier.contract.WatchLogs(opts, "ProofFeePaid", setIdRule)
	if err != nil {
		return nil, err
	}
	return event.NewSubscription(func(quit <-chan struct{}) error {
		defer sub.Unsubscribe()
		for {
			select {
			case log := <-logs:
				// New log arrived, parse the event and forward to the user
				event := new(ProofVerifierProofFeePaid)
				if err := _ProofVerifier.contract.UnpackLog(event, "ProofFeePaid", log); err != nil {
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

// ParseProofFeePaid is a log parse operation binding the contract event 0x58b7742b13c8873fc0ba58f695b33ca0044b2db7ff9c5208181dbaec2a5b291e.
//
// Solidity: event ProofFeePaid(uint256 indexed setId, uint256 fee)
func (_ProofVerifier *ProofVerifierFilterer) ParseProofFeePaid(log types.Log) (*ProofVerifierProofFeePaid, error) {
	event := new(ProofVerifierProofFeePaid)
	if err := _ProofVerifier.contract.UnpackLog(event, "ProofFeePaid", log); err != nil {
		return nil, err
	}
	event.Raw = log
	return event, nil
}

// ProofVerifierStorageProviderChangedIterator is returned from FilterStorageProviderChanged and is used to iterate over the raw logs and unpacked data for StorageProviderChanged events raised by the ProofVerifier contract.
type ProofVerifierStorageProviderChangedIterator struct {
	Event *ProofVerifierStorageProviderChanged // Event containing the contract specifics and raw log

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
func (it *ProofVerifierStorageProviderChangedIterator) Next() bool {
	// If the iterator failed, stop iterating
	if it.fail != nil {
		return false
	}
	// If the iterator completed, deliver directly whatever's available
	if it.done {
		select {
		case log := <-it.logs:
			it.Event = new(ProofVerifierStorageProviderChanged)
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
		it.Event = new(ProofVerifierStorageProviderChanged)
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
func (it *ProofVerifierStorageProviderChangedIterator) Error() error {
	return it.fail
}

// Close terminates the iteration process, releasing any pending underlying
// resources.
func (it *ProofVerifierStorageProviderChangedIterator) Close() error {
	it.sub.Unsubscribe()
	return nil
}

// ProofVerifierStorageProviderChanged represents a StorageProviderChanged event raised by the ProofVerifier contract.
type ProofVerifierStorageProviderChanged struct {
	SetId              *big.Int
	OldStorageProvider common.Address
	NewStorageProvider common.Address
	Raw                types.Log // Blockchain specific contextual infos
}

// FilterStorageProviderChanged is a free log retrieval operation binding the contract event 0x686146a80f2bf4dc855942926481871515b39b508826d7982a2e0212d20552c9.
//
// Solidity: event StorageProviderChanged(uint256 indexed setId, address indexed oldStorageProvider, address indexed newStorageProvider)
func (_ProofVerifier *ProofVerifierFilterer) FilterStorageProviderChanged(opts *bind.FilterOpts, setId []*big.Int, oldStorageProvider []common.Address, newStorageProvider []common.Address) (*ProofVerifierStorageProviderChangedIterator, error) {

	var setIdRule []interface{}
	for _, setIdItem := range setId {
		setIdRule = append(setIdRule, setIdItem)
	}
	var oldStorageProviderRule []interface{}
	for _, oldStorageProviderItem := range oldStorageProvider {
		oldStorageProviderRule = append(oldStorageProviderRule, oldStorageProviderItem)
	}
	var newStorageProviderRule []interface{}
	for _, newStorageProviderItem := range newStorageProvider {
		newStorageProviderRule = append(newStorageProviderRule, newStorageProviderItem)
	}

	logs, sub, err := _ProofVerifier.contract.FilterLogs(opts, "StorageProviderChanged", setIdRule, oldStorageProviderRule, newStorageProviderRule)
	if err != nil {
		return nil, err
	}
	return &ProofVerifierStorageProviderChangedIterator{contract: _ProofVerifier.contract, event: "StorageProviderChanged", logs: logs, sub: sub}, nil
}

// WatchStorageProviderChanged is a free log subscription operation binding the contract event 0x686146a80f2bf4dc855942926481871515b39b508826d7982a2e0212d20552c9.
//
// Solidity: event StorageProviderChanged(uint256 indexed setId, address indexed oldStorageProvider, address indexed newStorageProvider)
func (_ProofVerifier *ProofVerifierFilterer) WatchStorageProviderChanged(opts *bind.WatchOpts, sink chan<- *ProofVerifierStorageProviderChanged, setId []*big.Int, oldStorageProvider []common.Address, newStorageProvider []common.Address) (event.Subscription, error) {

	var setIdRule []interface{}
	for _, setIdItem := range setId {
		setIdRule = append(setIdRule, setIdItem)
	}
	var oldStorageProviderRule []interface{}
	for _, oldStorageProviderItem := range oldStorageProvider {
		oldStorageProviderRule = append(oldStorageProviderRule, oldStorageProviderItem)
	}
	var newStorageProviderRule []interface{}
	for _, newStorageProviderItem := range newStorageProvider {
		newStorageProviderRule = append(newStorageProviderRule, newStorageProviderItem)
	}

	logs, sub, err := _ProofVerifier.contract.WatchLogs(opts, "StorageProviderChanged", setIdRule, oldStorageProviderRule, newStorageProviderRule)
	if err != nil {
		return nil, err
	}
	return event.NewSubscription(func(quit <-chan struct{}) error {
		defer sub.Unsubscribe()
		for {
			select {
			case log := <-logs:
				// New log arrived, parse the event and forward to the user
				event := new(ProofVerifierStorageProviderChanged)
				if err := _ProofVerifier.contract.UnpackLog(event, "StorageProviderChanged", log); err != nil {
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

// ParseStorageProviderChanged is a log parse operation binding the contract event 0x686146a80f2bf4dc855942926481871515b39b508826d7982a2e0212d20552c9.
//
// Solidity: event StorageProviderChanged(uint256 indexed setId, address indexed oldStorageProvider, address indexed newStorageProvider)
func (_ProofVerifier *ProofVerifierFilterer) ParseStorageProviderChanged(log types.Log) (*ProofVerifierStorageProviderChanged, error) {
	event := new(ProofVerifierStorageProviderChanged)
	if err := _ProofVerifier.contract.UnpackLog(event, "StorageProviderChanged", log); err != nil {
		return nil, err
	}
	event.Raw = log
	return event, nil
}

// ProofVerifierUpgradeAnnouncedIterator is returned from FilterUpgradeAnnounced and is used to iterate over the raw logs and unpacked data for UpgradeAnnounced events raised by the ProofVerifier contract.
type ProofVerifierUpgradeAnnouncedIterator struct {
	Event *ProofVerifierUpgradeAnnounced // Event containing the contract specifics and raw log

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
func (it *ProofVerifierUpgradeAnnouncedIterator) Next() bool {
	// If the iterator failed, stop iterating
	if it.fail != nil {
		return false
	}
	// If the iterator completed, deliver directly whatever's available
	if it.done {
		select {
		case log := <-it.logs:
			it.Event = new(ProofVerifierUpgradeAnnounced)
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
		it.Event = new(ProofVerifierUpgradeAnnounced)
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
func (it *ProofVerifierUpgradeAnnouncedIterator) Error() error {
	return it.fail
}

// Close terminates the iteration process, releasing any pending underlying
// resources.
func (it *ProofVerifierUpgradeAnnouncedIterator) Close() error {
	it.sub.Unsubscribe()
	return nil
}

// ProofVerifierUpgradeAnnounced represents a UpgradeAnnounced event raised by the ProofVerifier contract.
type ProofVerifierUpgradeAnnounced struct {
	PlannedUpgrade ProofVerifierPlannedUpgrade
	Raw            types.Log // Blockchain specific contextual infos
}

// FilterUpgradeAnnounced is a free log retrieval operation binding the contract event 0xbcf8666408d712c75c2cbd790925afbec6495ca9e04186b1182902260a1d53cd.
//
// Solidity: event UpgradeAnnounced((address,uint96) plannedUpgrade)
func (_ProofVerifier *ProofVerifierFilterer) FilterUpgradeAnnounced(opts *bind.FilterOpts) (*ProofVerifierUpgradeAnnouncedIterator, error) {

	logs, sub, err := _ProofVerifier.contract.FilterLogs(opts, "UpgradeAnnounced")
	if err != nil {
		return nil, err
	}
	return &ProofVerifierUpgradeAnnouncedIterator{contract: _ProofVerifier.contract, event: "UpgradeAnnounced", logs: logs, sub: sub}, nil
}

// WatchUpgradeAnnounced is a free log subscription operation binding the contract event 0xbcf8666408d712c75c2cbd790925afbec6495ca9e04186b1182902260a1d53cd.
//
// Solidity: event UpgradeAnnounced((address,uint96) plannedUpgrade)
func (_ProofVerifier *ProofVerifierFilterer) WatchUpgradeAnnounced(opts *bind.WatchOpts, sink chan<- *ProofVerifierUpgradeAnnounced) (event.Subscription, error) {

	logs, sub, err := _ProofVerifier.contract.WatchLogs(opts, "UpgradeAnnounced")
	if err != nil {
		return nil, err
	}
	return event.NewSubscription(func(quit <-chan struct{}) error {
		defer sub.Unsubscribe()
		for {
			select {
			case log := <-logs:
				// New log arrived, parse the event and forward to the user
				event := new(ProofVerifierUpgradeAnnounced)
				if err := _ProofVerifier.contract.UnpackLog(event, "UpgradeAnnounced", log); err != nil {
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

// ParseUpgradeAnnounced is a log parse operation binding the contract event 0xbcf8666408d712c75c2cbd790925afbec6495ca9e04186b1182902260a1d53cd.
//
// Solidity: event UpgradeAnnounced((address,uint96) plannedUpgrade)
func (_ProofVerifier *ProofVerifierFilterer) ParseUpgradeAnnounced(log types.Log) (*ProofVerifierUpgradeAnnounced, error) {
	event := new(ProofVerifierUpgradeAnnounced)
	if err := _ProofVerifier.contract.UnpackLog(event, "UpgradeAnnounced", log); err != nil {
		return nil, err
	}
	event.Raw = log
	return event, nil
}

// ProofVerifierUpgradedIterator is returned from FilterUpgraded and is used to iterate over the raw logs and unpacked data for Upgraded events raised by the ProofVerifier contract.
type ProofVerifierUpgradedIterator struct {
	Event *ProofVerifierUpgraded // Event containing the contract specifics and raw log

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
func (it *ProofVerifierUpgradedIterator) Next() bool {
	// If the iterator failed, stop iterating
	if it.fail != nil {
		return false
	}
	// If the iterator completed, deliver directly whatever's available
	if it.done {
		select {
		case log := <-it.logs:
			it.Event = new(ProofVerifierUpgraded)
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
		it.Event = new(ProofVerifierUpgraded)
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
func (it *ProofVerifierUpgradedIterator) Error() error {
	return it.fail
}

// Close terminates the iteration process, releasing any pending underlying
// resources.
func (it *ProofVerifierUpgradedIterator) Close() error {
	it.sub.Unsubscribe()
	return nil
}

// ProofVerifierUpgraded represents a Upgraded event raised by the ProofVerifier contract.
type ProofVerifierUpgraded struct {
	Implementation common.Address
	Raw            types.Log // Blockchain specific contextual infos
}

// FilterUpgraded is a free log retrieval operation binding the contract event 0xbc7cd75a20ee27fd9adebab32041f755214dbc6bffa90cc0225b39da2e5c2d3b.
//
// Solidity: event Upgraded(address indexed implementation)
func (_ProofVerifier *ProofVerifierFilterer) FilterUpgraded(opts *bind.FilterOpts, implementation []common.Address) (*ProofVerifierUpgradedIterator, error) {

	var implementationRule []interface{}
	for _, implementationItem := range implementation {
		implementationRule = append(implementationRule, implementationItem)
	}

	logs, sub, err := _ProofVerifier.contract.FilterLogs(opts, "Upgraded", implementationRule)
	if err != nil {
		return nil, err
	}
	return &ProofVerifierUpgradedIterator{contract: _ProofVerifier.contract, event: "Upgraded", logs: logs, sub: sub}, nil
}

// WatchUpgraded is a free log subscription operation binding the contract event 0xbc7cd75a20ee27fd9adebab32041f755214dbc6bffa90cc0225b39da2e5c2d3b.
//
// Solidity: event Upgraded(address indexed implementation)
func (_ProofVerifier *ProofVerifierFilterer) WatchUpgraded(opts *bind.WatchOpts, sink chan<- *ProofVerifierUpgraded, implementation []common.Address) (event.Subscription, error) {

	var implementationRule []interface{}
	for _, implementationItem := range implementation {
		implementationRule = append(implementationRule, implementationItem)
	}

	logs, sub, err := _ProofVerifier.contract.WatchLogs(opts, "Upgraded", implementationRule)
	if err != nil {
		return nil, err
	}
	return event.NewSubscription(func(quit <-chan struct{}) error {
		defer sub.Unsubscribe()
		for {
			select {
			case log := <-logs:
				// New log arrived, parse the event and forward to the user
				event := new(ProofVerifierUpgraded)
				if err := _ProofVerifier.contract.UnpackLog(event, "Upgraded", log); err != nil {
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

// ParseUpgraded is a log parse operation binding the contract event 0xbc7cd75a20ee27fd9adebab32041f755214dbc6bffa90cc0225b39da2e5c2d3b.
//
// Solidity: event Upgraded(address indexed implementation)
func (_ProofVerifier *ProofVerifierFilterer) ParseUpgraded(log types.Log) (*ProofVerifierUpgraded, error) {
	event := new(ProofVerifierUpgraded)
	if err := _ProofVerifier.contract.UnpackLog(event, "Upgraded", log); err != nil {
		return nil, err
	}
	event.Raw = log
	return event, nil
}
