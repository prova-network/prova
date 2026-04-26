# 3.2 Event schema

All Prova on-chain events that indexers, dashboards, and SDK consumers should track. The Solidity declarations are authoritative; this section is the reading reference.

## 3.2.1 StorageMarketplace events

```solidity
event DealProposed(
    uint256 indexed dealId,
    address indexed client,
    address indexed prover,
    bytes32 commpHash,
    uint64  pieceSize,
    uint64  durationSeconds,
    uint256 totalPayment
);

event DealAccepted(
    uint256 indexed dealId,
    address indexed prover,
    uint256 dataSetId,
    uint64  endsAt
);

event DealCompleted(uint256 indexed dealId, uint256 finalPaidOut);
event DealCancelled(uint256 indexed dealId, uint256 refund);
event DealSlashed(
    uint256 indexed dealId,
    address indexed prover,
    uint256 slashedAmount,
    uint256 refunded
);

event ProofRecorded(
    uint256 indexed dealId,
    uint256 proofCount,
    uint256 paymentReleased
);

event ProtocolFeeChanged(uint256 oldBps, uint256 newBps);
event TreasuryChanged(address indexed oldTreasury, address indexed newTreasury);
event SlashPerFaultChanged(uint256 oldValue, uint256 newValue);
event ProverRewardsSet(address indexed previous, address indexed next);
```

## 3.2.2 ProverRegistry events

```solidity
event ProverRegistered(
    address indexed prover,
    string  endpoint,
    uint64  features,
    uint64  capacity,
    uint8   region,
    bytes   attestation
);

event ProverDeregistered(address indexed prover);
event ProverEndpointUpdated(address indexed prover, string newEndpoint);
event ProverFeaturesUpdated(address indexed prover, uint64 newFeatures);
```

## 3.2.3 ProverStaking events

```solidity
event Staked(address indexed prover, uint256 amount);
event UnstakeRequested(
    address indexed prover,
    uint256 amount,
    uint256 endsAt
);
event Unstaked(address indexed prover, uint256 amount);
event Slashed(
    address indexed prover,
    uint256 amount,
    bytes32 reason
);
event BytesCommitted(address indexed prover, uint256 bytesCommitted);
event BytesReleased(address indexed prover, uint256 bytesReleased);
```

## 3.2.4 ProverRewards events

```solidity
event MarketplaceSet(address indexed previous, address indexed next);
event RedundancyCapSet(uint8 previous, uint8 next);
event QualityCutoffSet(uint16 previous, uint16 next);

event ProofRecorded(
    uint256 indexed epoch,
    address indexed prover,
    bytes32 indexed pieceCid,
    uint256 bytesProven,
    bool counted
);

event QualityUpdated(
    address indexed prover,
    uint64 successes,
    uint64 failures
);

event Claimed(
    address indexed prover,
    uint256 indexed epoch,
    uint256 amount
);
```

Note: `ProverRewards.ProofRecorded` is a different event from `StorageMarketplace.ProofRecorded` — they share a name but live on different contracts. Indexers SHOULD distinguish by emitting contract address.

## 3.2.5 FeeRouter events

```solidity
event ModeChanged(Mode indexed oldMode, Mode indexed newMode);
event BurnShareChanged(uint16 oldBps, uint16 newBps);
event SwapRouterChanged(address oldRouter, address newRouter);
event SwapPoolFeeChanged(uint24 oldFee, uint24 newFee);
event MaxSwapPerCallChanged(uint256 oldMax, uint256 newMax);
event MaxSlippageChanged(uint16 oldBps, uint16 newBps);

event FeesBurned(uint256 usdcIn, uint256 provaOut);
event FeesHeld(uint256 usdcAmount);
event Withdrawn(address indexed token, address indexed to, uint256 amount);
```

## 3.2.6 ContentRegistry events

```solidity
event ContentRegistered(
    bytes32 indexed commp,
    uint256 indexed dealId,
    address indexed client,
    uint64  pieceSize
);

event ContentCleared(bytes32 indexed commp, uint256 indexed dealId);
event ENSBound(bytes32 indexed commp, bytes32 indexed ensNode);
event ENSCleared(bytes32 indexed commp, bytes32 indexed ensNode);
```

## 3.2.7 ProofVerifier (PDP) events

ProofVerifier is a fork of Filecoin's PDPVerifier. Its events follow the upstream convention with one Prova-specific addition (`ProvaListener` callbacks). The full set is documented in [`prova-network/contracts/src/ProofVerifier.sol`](https://github.com/prova-network/contracts/blob/main/src/ProofVerifier.sol).

## 3.2.8 Indexer guidance

- All events are indexed by `(contract, eventName, blockNumber, txIndex, logIndex)`.
- Deal state at block `n` is `lastEventForDeal(dealId, ≤ n)`.
- For point-in-time prover quality, sum `ProofRecorded(counted=true)` and `QualityUpdated.failures` over the trailing 30-day window.
- `Claimed` events on `ProverRewards` are the canonical record of how much PROVA emission has been distributed.
