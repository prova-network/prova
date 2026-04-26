---
description: Why a Prova-stored file is more durable than one in S3.
---

# Continuous proof of storage

S3 has 11 nines of durability. Prova has **continuous, verifiable** durability. That's a stronger property.

## The difference

| Property | S3 / Backblaze / IPFS pin | Prova |
| --- | --- | --- |
| **Durability claim** | Vendor-asserted | Cryptographically verified |
| **Verification** | None — you trust the vendor | Every 30 seconds, public proof |
| **Loss detection** | When you try to read | Within seconds, on-chain |
| **Economic backing** | Vendor SLA credits | Slashed prover stake |
| **Audit trail** | Internal logs | Public Ethereum L2 |

S3 says _"we won't lose your file"_ and you take their word for it. Prova says _"here is a fresh proof, posted to the public ledger 8 seconds ago, that the bytes you stored are still on the prover's disk."_

## How it works

Prova uses **PDP** (Provable Data Possession), specifically the variant pioneered by Filecoin and adapted for Base in `ProofVerifier.sol`.

Every 30 seconds:

1. **Challenge.** The on-chain contract emits a pseudo-random challenge — a small set of byte offsets within the piece.
2. **Prove.** The prover reads those bytes off disk, computes a Merkle proof against the piece-cid, and submits the proof to the contract.
3. **Verify.** The contract checks the proof matches the on-chain cid. Pass → release payment. Fail → don't release; if it fails too many times in a row, slash.

The key property: **the prover cannot fake a proof without actually having the bytes**. The Merkle structure is committed at deal creation, so any byte the prover claims to read has to hash up to the original cid.

## What "every 30 seconds" buys you

* **Loss detection in seconds.** If a prover loses your file, the next missed proof tells you within 30 seconds. Compare to S3, where you might not know until you try to read.
* **Non-repudiation.** The prover cannot say "we had it, but our logs got corrupted." There is a public chain of signed proofs.
* **Slashing economics.** A prover who deletes your file to save disk loses _more_ in slashed stake than they save. The math is in [Earnings](../provers/earnings.md).
* **Re-pinning.** When a prover starts failing, the marketplace automatically re-lists the cid. Healthy provers pick it up. Your file is never one disk failure away from being lost.

## Why 30 seconds and not, say, 1 second

Compute and gas cost. Each proof is a Base L2 transaction. At 30 second cadence:

* ~2,880 proofs per day per deal.
* ~$0.003 gas per proof on Base L2.
* ~$8.64/day in gas per deal — bounded by the deal's payment, not the protocol's.

Faster cadence is technically possible but reduces the prover's net margin. We picked 30 seconds as a balance: fast enough to detect failures within a single human attention span, slow enough that the gas cost is a small fraction of the storage fee.

## What this is _not_

* It's **not** Proof of Replication. We don't prove the prover sealed the data into a unique encoded form. PoRep is heavier and the security gain is marginal for non-Sybil-resistant networks.
* It's **not** zero-knowledge. The prover learns nothing about the challenge ahead of time, but the proof itself is public (and trivially small). If you need privacy on top, encrypt before uploading.
* It's **not** time-locked. The prover proves the file is _currently_ there, not that it was _continuously_ there since deal start. The two are equivalent in practice (you'd notice the gap), but a paranoid client should index the on-chain proofs themselves.

## Audit trail

You can verify any deal's history:

```bash
# Get all proofs for a deal
curl https://prova.network/api/files/<cid>/proofs \
  -H "authorization: Bearer pk_live_..."
```

Returns a stream of `{ blockNumber, txHash, proofData, valid }` records, one per attempted proof. You can independently re-verify any of them by calling `ProofVerifier.verifyProof(...)` on Base.
