---
description: Three roles, one cryptographic contract.
---

# How Prova works

Prova is built around **three roles** and a single on-chain contract that mediates between them.

```
   ┌────────┐         ┌────────┐         ┌──────────┐
   │ Client │ ◄─────► │ Prover │ ◄─────► │ Ethereum │
   └────────┘         └────────┘         └──────────┘
   stores · pays      stakes · stores    verifies · slashes
   verifies           proves             pays
```

## The roles

### Client
You. Or your app. Anyone who wants a file stored with cryptographic guarantees.

A client:
* Computes a `piece-cid` from their file (a content-addressed identifier)
* Pays the storage fee in USDC
* Verifies the file is still retrievable, whenever they want

### Prover
A node operator with disk and bandwidth. Provers earn USDC by storing client files and proving they still hold them.

A prover:
* Stakes PROVA against the durability of what they hold (slashing burns PROVA)
* Stores files addressed by `piece-cid`
* Posts a cryptographic proof every 30 seconds (continuous PDP)
* Gets slashed if they miss a proof

### Ethereum (Base L2)
The judge. Holds the storage marketplace contract, the prover registry, and the staking contract.

Ethereum:
* Records every deal proposal
* Releases payment to the prover as proofs land
* Slashes the prover's stake if a proof is missed
* Is the only source of truth for who owes whom what

## The deal

A **deal** is the contract between a client and a prover for a single piece, for a fixed duration. Deals move through five states:

`Proposed → Downloading → Storing → Active → Settled`

You can read more in [Deal lifecycle](deal-lifecycle.md).

## What's intentionally not in Prova

* **No new chain.** Prova lives on Base. Settlement is Ethereum-grade.
* **No bridges.** Client payments are native USDC. No wrapped anything.
* **No sealed PoRep.** No TEEs. No fancy crypto theater. Just PDP — a 25-year-old, well-understood proof scheme.
* **No custody.** Provers can't see your data unless you give them retrieval rights. They prove they're storing the bytes; they don't necessarily know what the bytes mean.
* **No PROVA in the client UX.** Clients pay USDC. PROVA only matters to provers (stake) and PROVA holders (governance + fee burn).

## Why this works

A prover's incentive structure is simple:

| Action | Outcome |
| --- | --- |
| Post a proof on time | Earn 99% of the deal's per-block payment |
| Miss a proof | Get slashed; deal is re-pinned to a healthy prover |
| Try to delete the file | Next proof fails; get slashed |
| Try to lie in the proof | Cryptographically impossible |

The math is in [Continuous proof of storage](continuous-proof.md).

## Pieces, not files

Prova stores **pieces**, not files. A piece is a content-addressed blob. If your file is bigger than the per-piece limit, you split it client-side and store each piece independently. (The CLI and SDK handle this for you.)

Two clients who upload the same file get the same `piece-cid`. The prover only stores one copy. The two clients each get their own deal.

## Where the bytes live

For each piece-cid, multiple provers may hold a copy. This is **redundancy** — the [resilience model](resilience.md). When you retrieve, the network routes you to the fastest healthy prover. If they're down, retrieval falls through to the next one.
