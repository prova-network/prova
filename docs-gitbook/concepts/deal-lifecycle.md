---
description: From proposal to permanence.
---

# Deal lifecycle

A Prova deal moves through five observable on-chain states. Every transition is recorded on Base. Every state is queryable via the API.

```
Proposed → Downloading → Storing → Active → Settled
                                 ↘    ↓
                                  ↘   ↓
                                  Slashed (failure)
```

## States

### Proposed
On-chain. The client has called `proposeDeal()` on `StorageMarketplace.sol`, escrowed the storage fee, and the prover has been notified.

In this state:
* The client has paid into the escrow.
* The prover has not yet acknowledged.
* If the prover doesn't accept within the timeout, the client gets refunded.

### Downloading
The prover has acknowledged and is fetching the bytes from the URL the client provided.

In this state:
* The prover pulls bytes from `sourceURL` (an HTTPS URL the client published).
* The prover hashes the bytes as it streams; it computes the `piece-cid` independently.
* If the computed cid doesn't match what's on-chain, the deal aborts and the prover is **not** slashed (the client provided bad data).

### Storing
The prover has the bytes. They're computing the on-chain commitment.

In this state:
* The prover writes the bytes to its disk under `pieces/<cid>`.
* The prover prepares the first proof.
* The deal is one block from being active.

### Active
The deal is live. The prover posts a cryptographic proof every 30 seconds.

In this state:
* Each successful proof releases a slice of the escrowed payment to the prover.
* Missed proofs eat into the prover's stake.
* The client can retrieve the file at `https://prova.network/p/<cid>` (or directly from the prover via libp2p / HTTP).
* The deal stays Active for the duration the client paid for (default 30 days, max 5 years).

### Settled
The deal has reached its end-time. The prover has been paid for every successful proof. Any remaining escrow is refunded to the client (if the prover missed proofs and got partially slashed).

The bytes can stay on the prover's disk indefinitely after Settled — the prover just isn't paid for them anymore. Some provers keep historical pieces around for popularity-driven retrieval revenue.

### Slashed (failure mode, not a normal state)
The prover missed too many proofs in a row, and the deal is terminated.

When this happens:
* The remaining escrow is refunded to the client.
* The prover's stake is slashed (a percentage based on how badly they failed).
* The piece-cid is automatically re-listed in the marketplace, so a healthy prover can pick it up.

## Querying state

```bash
# Via API
curl https://prova.network/api/files \
  -H "authorization: Bearer pk_live_..." \
  | jq '.files[] | {cid, status, proofsPosted}'
```

Or on-chain:

```solidity
// On Base
StorageMarketplace.getDeal(uint256 dealId) returns (Deal memory);
```

The `Deal` struct includes the current status, proofs posted, escrow remaining, and last proof timestamp.

## Time scale

| Phase | Typical duration |
| --- | --- |
| Proposed → Downloading | < 30 seconds |
| Downloading | size-dependent (≈ 1 GB/min on a good link) |
| Storing | < 60 seconds |
| Active | the term you paid for |
| Active → Settled | the moment the term ends |

For a 100 MB file with a 30-day term: ~2 minutes from proposal to Active, then 30 days of daily proofs, then Settled.

## What this means for your app

If you're building on Prova, the **observable state** is:

* `cid` — the content-addressed id
* `status` — one of `proposed | downloading | storing | active | settled | slashed`
* `proofsPosted` — count of successful proofs since the deal went Active
* `lastProofAt` — timestamp of the most recent proof
* `endsAt` — when the deal Settles
* `escrowRemaining` — USDC still held in the contract

A piece is **safe to retrieve** as soon as `status === 'active'`. Before that, the prover may not yet have the bytes locally.
