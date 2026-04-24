# Prova: Notes on Verifiable, Retrievable Storage

**Working draft — 2026**

---

## Abstract

These notes describe an early-stage research project exploring a narrow question: what would it take to make storage verifiable and retrievable by default, for the kinds of data that actually need to outlive a single provider? By "verifiable" we mean cryptographically, continuously, and cheaply enough to do in practice. By "retrievable" we mean demonstrably reachable, not merely held. The notes are deliberately short and avoid committing to specifics that are still being worked out. An updated draft will follow when the design is closer to implementation.

---

## 1. Motivation

The storage we rely on is mostly unverifiable. A provider tells us our bytes are safe; we trust them, or we don't use them. When the provider disappears, so does our confidence in the data. For short-lived content this is tolerable. For long-lived content — research datasets, public archives, legal and regulatory records, AI training corpora, cultural artifacts, evidence — it is not.

Decentralized storage networks have addressed parts of this problem, each with meaningful tradeoffs. This project revisits the problem from a narrower angle: what if the proof machinery could be separated from the rest of the network economics, packaged as a minimal primitive, and used from whichever existing settlement layer is most convenient? Fewer moving parts, more confidence in each one.

## 2. Primitives

The design centers on well-studied building blocks and avoids inventing new cryptography where existing work is sufficient.

### 2.1 Content commitments

Each object is identified by a commitment computed over the bytes of its padded form. The commitment is a binary Merkle root with a standard leaf and inner-node hash, producing a short identifier computable by anyone who holds the bytes. The commitment serves as both identity and integrity: the same bytes always produce the same identifier, and anyone holding the bytes can recompute and verify the identifier.

### 2.2 Provable data possession

A provider that holds an object can respond to a random challenge by returning a short Merkle inclusion proof of one challenged leaf. Verification is logarithmic in the object's size, and the probability of a dishonest provider producing a valid proof for a random challenge without actually holding the bytes is negligible. Repeated challenges over time compound this into a continuous proof of retention.

### 2.3 Settlement

Proofs are verified on an existing programmable settlement layer. This layer handles identity, payment, escrow, and dispute resolution. The project does not propose a new settlement layer. The storage network is off-chain; only the proof verification, staking, and payment accounting live on-chain.

### 2.4 Optional optimizations

Two optimizations are compatible with the baseline and may be introduced where their tradeoffs are appropriate.

- **Hardware-attested storage.** Trusted execution environments can reduce onboarding cost and enable unique-replica guarantees without increasing trust assumptions on the baseline math. These are optimizations, not replacements.
- **Proof aggregation.** Multiple proofs from one or many providers can be compressed into a single verifiable object. Aggregation reduces per-proof settlement cost, at the price of added off-chain coordination.

Both are deferred until the baseline is stable.

## 3. What this is not

- This is not a new blockchain.
- This is not a token launch.
- This is not a replacement for any existing decentralized storage network.
- This is not a research claim that the underlying primitives are novel; they are not. The contribution, if there is one, is in packaging, scope reduction, and interoperability.

## 4. Provenance and credit

The cryptographic primitives described here have been developed, published, and refined by several research communities over the past decade. Where this work benefits from that prior work, credit is given at the source-file level in the eventual reference implementation. This project is not in competition with those communities; it is a distillation of a subset of their work, applied to a narrower product question.

## 5. Status

The project is private, pre-implementation, and pre-announcement. These notes exist to make the thesis legible to collaborators and to pre-empt any misunderstanding about fundraising, token offerings, or public commitments, of which there are none.

## 6. Correspondence

Research correspondence and serious inquiries may be directed to `hello@prova.network`. Please do not write about investment.

---

*End of notes.*
