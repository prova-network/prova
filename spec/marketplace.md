# 3.1 Marketplace

The Prova marketplace is the on-chain registry of storage deals between clients and provers. This section specifies the deal data structure, the state machine, the payment flow, and the integration points with the other contracts.

Source of truth: [`StorageMarketplace.sol`](https://github.com/prova-network/contracts/blob/main/src/StorageMarketplace.sol).

## 3.1.1 Deal data structure

```solidity
struct Deal {
    address client;          // who proposed the deal
    address prover;          // who accepts and stores
    bytes32 commpHash;       // 32-byte CommP digest of the piece
    uint64  pieceSize;       // size in bytes of the piece (Fr32-padded)
    uint64  startedAt;       // unix when prover accepted
    uint64  endsAt;          // unix when deal expires
    uint128 totalPayment;    // total USDC the client deposited (escrow)
    uint128 paidOut;         // USDC released to prover so far
    uint64  proofCount;      // number of valid proofs posted
    uint64  lastProofAt;     // unix of most recent proof
    uint256 dataSetId;       // ProofVerifier identifier for this deal
    DealStatus status;       // see §3.1.2
}
```

The `commpHash` is the 32-byte CommP digest, not the full CIDv1. Clients MAY pass either; the marketplace stores the digest only. Reverse-encoding to printable `baga…` is a UI concern.

## 3.1.2 Deal status

```solidity
enum DealStatus {
    Proposed,    // client deposited escrow; awaiting prover acceptance
    Active,      // prover accepted and posted first proof
    Completed,   // term ended successfully; final payment released
    Cancelled,   // client cancelled before acceptance; full refund
    Slashed      // prover failed; client refunded, prover stake slashed
}
```

Permitted transitions:

```
Proposed → Active     (prover acceptance triggers ProofVerifier listener)
Proposed → Cancelled  (client cancellation)
Active   → Completed  (deal end + completeDeal call)
Active   → Slashed    (faultDeal triggered after MAX_PROOF_GAP)
```

No other transitions are valid. Implementations MUST revert on attempted invalid transitions.

## 3.1.3 Lifecycle

### Propose

```solidity
function proposeDeal(
    address prover,
    bytes32 commp,
    uint64  pieceSize,
    uint64  duration,
    uint256 totalPayment
) external returns (uint256 dealId);
```

The client MUST have approved `totalPayment` of the payment token (USDC) to the marketplace. The full payment is moved into escrow at proposal time.

The marketplace MUST emit `DealProposed(dealId, client, prover, commp, pieceSize, duration, totalPayment)`.

### Cancel (client, before acceptance)

```solidity
function cancelProposedDeal(uint256 dealId) external;
```

The client MAY cancel a deal at any time before the prover transitions it to Active. The full escrow is refunded.

### Accept

The prover does NOT call the marketplace directly. Acceptance is triggered when the prover calls `ProofVerifier.createDataSet(commp, ...extraData)` — the verifier's `dataSetCreated` listener calls back into `marketplace.dataSetCreated(dataSetId, prover, extraData)`. The `extraData` blob carries the `dealId`.

The marketplace then:

1. Verifies the prover registered in `extraData` matches the `prover` field of the deal.
2. Checks the prover's stake covers the piece size (`proverStaking.commitBytes(prover, pieceSize)`).
3. Sets `startedAt = block.timestamp`, `endsAt = startedAt + duration`, `status = Active`.
4. Records the deal in `ContentRegistry`.
5. Emits `DealAccepted(dealId, prover, dataSetId, endsAt)`.

### Prove

Each successful PDP proof flows through:

```
ProofVerifier.verifyProof()
    → marketplace.possessionProven(dataSetId, ...)
        → streaming USDC release to prover (99%) + treasury (1%)
        → optional: ProverRewards.recordProof(prover, client, commp, pieceSize)
        → emit ProofRecorded(dealId, proofCount, paymentReleased)
```

The streaming release is **linear in time elapsed** since the deal started, capped at `totalPayment`. A prover that posts every challenge on time receives the full payment over the term; a prover that posts late catches up on the next successful proof.

### Complete

After `endsAt`, anyone MAY call `completeDeal(dealId)`. The marketplace:

1. Releases any remaining unpaid fraction of `totalPayment` to the prover (subject to the same 99/1 protocol fee split).
2. Releases `pieceSize` from the prover's committed bytes via `proverStaking.releaseBytes`.
3. Clears the active deal in `ContentRegistry`.
4. Sets `status = Completed`.
5. Emits `DealCompleted(dealId, finalPaidOut)`.

### Fault

If `block.timestamp - lastProofAt > MAX_PROOF_GAP` while the deal is Active, anyone MAY call `faultDeal(dealId)`. The marketplace:

1. Sets `status = Slashed`.
2. Refunds `totalPayment - paidOut` to the client.
3. Slashes `slashPerFault` PROVA from the prover via `proverStaking.slash`.
4. Releases `pieceSize` from the prover's committed bytes.
5. Clears `ContentRegistry`.
6. Optionally calls `proverRewards.recordMiss(prover)` for the quality multiplier.
7. Emits `DealSlashed(dealId, prover, slashedAmount, refunded)`.

`MAX_PROOF_GAP` is governance-tunable. Default: 6 hours.

## 3.1.4 Payment token

The marketplace's `paymentToken` is **USDC** at v1. Provers earn USDC. Clients pay USDC. Refunds are USDC.

The marketplace's `treasury` is the **`FeeRouter`** address (§5.1.4). The 1% protocol fee streams there continuously and is later swapped to PROVA and burned.

## 3.1.5 Stake locking

When a deal transitions to Active, the prover's `committedBytes` increase by `pieceSize`. When the deal terminates (Completed or Slashed), `committedBytes` decrease by `pieceSize`.

`maxCommittedBytes(prover) = staked(prover) / minStakePerGiB` — the prover MUST have enough stake to cover all active deals; the marketplace enforces this by reverting on `commitBytes` if it would exceed the cap.

## 3.1.6 Concurrency and reentrancy

- All state-mutating external functions are `nonReentrant`.
- Cross-contract calls happen in a fixed order: stake → content → payment, never in a loop.
- The optional `proverRewards.recordProof` call is wrapped in `try/catch` so a misconfigured rewards contract can never block a payment.

## 3.1.7 Admin parameters

| Parameter | Default | Hard cap | Set by | Timelock |
| --- | --- | --- | --- | --- |
| `protocolFeeBps` | 100 (1%) | 300 (3%) | governance | 2 days |
| `slashPerFault` | 50 PROVA | governance-set | governance | 2 days |
| `treasury` | `FeeRouter` | n/a | owner | none (admin) |
| `proverRewards` | set at deploy | n/a | owner | none (admin) |
| `MAX_PROOF_GAP` | 6 hours | governance-set | governance | 2 days |

## 3.1.8 Events

See [§3.2 Event schema](/event-schema).
