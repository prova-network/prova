# Prova: Verifiable Storage for Ethereum

**Version 0.3 — Draft (post-pivot)**
**Author: Prova Team**
**April 2026**

---

## Abstract

We present Prova, an Ethereum-native network for verifiable, retrievable, and proof-backed storage. Provers run off-chain nodes that store content-addressed data, respond to periodic Provable Data Possession (PDP) challenges issued by Ethereum contracts, and optionally serve content over HTTPS. Ethereum handles identity, payment, staking, governance, and proof verification; no separate consensus layer is required. Content is addressed using CommP commitments (Filecoin-compatible, SHA-256 binary Merkle tree), optionally bound to ENS names for human-readable addressing. Provers stake a PROVA ERC-20 token as slashable collateral and earn rewards from storage fees and protocol emissions. Disputes are resolved through interactive fraud-proof games implemented as Ethereum contracts, with optional TEE-attested fast paths and optional ZK-aggregated proof batching reserved for future work.

Prova's central claim is that a dedicated blockchain is no longer required to build a verifiable storage network. The tools Ethereum offers in 2026 (L2 payment scaling, ENS, standardized staking and governance contracts, high-throughput storage-specialized L2s) are sufficient to serve as the trust and settlement root for a proof-backed storage layer, provided the storage and proof execution live off-chain and anchor to L1 periodically. This architecture avoids the complexity and capital cost of running a separate chain, while retaining the cryptographic rigor of PDP-style storage proofs.

---

## 1. Introduction

### 1.1 The Problem

AI and internet-scale infrastructure need storage that is verifiable, retrievable, and economically accountable. Centralized cloud providers offer convenience but no cryptographic guarantee that data is stored correctly, that it will remain retrievable, or that the provider's claim of availability is true. Decentralized alternatives exist (Filecoin, Arweave, Storj, Sia), but each builds its own blockchain, its own economics, its own identity system, and its own developer ecosystem. The user who wants "verifiable storage for my application" must learn a new chain to use them.

### 1.2 The Missing Piece

Ethereum has the payment rails (L1 and L2), the identity layer (ENS, attestations, account abstraction), the development tooling (Hardhat, Foundry, ethers, viem), and the composability (Safe, Uniswap, Sablier, Aragon) to serve as the trust root for a storage network. What it has historically lacked is a way to cryptographically verify that data is actually stored and retrievable.

Provable Data Possession (PDP) solves this problem cryptographically. The Filecoin community has refined PDP over years of production use. The primitive is mature, MIT-licensed, and ready to port.

### 1.3 Our Approach

Prova brings PDP-based storage proofs to Ethereum as a native primitive. Provers stake PROVA, register their endpoints and pricing in a registry contract, and respond to challenges issued by the ProofVerifier contract. Challenges are random; responses are Merkle inclusion proofs verified on-chain in O(log N) gas. Missed proofs trigger disputes; unresolved disputes trigger slashing. Successful proofs release streaming payments to the prover from client-locked funds.

No new consensus. No separate chain. No bridge. No native gas token. Prova lives on Ethereum the way Sablier lives on Ethereum: as a set of contracts, a set of off-chain providers, and a set of clients.

### 1.4 What We Deliberately Omit

This paper does not address:

- **Verifiable AI inference.** The Quantized Bisection Proof (QBP) protocol we designed in v1 remains specified but is deferred to a future version. Storage ships first.
- **Custom consensus.** Ethereum is the consensus.
- **Proof of Replication.** Not required for the current threat model. PDP is sufficient when providers are economically deterred from cheating via slashing.
- **ZK proof aggregation.** Specified as future work. MVP uses direct or batched on-chain verification.
- **Data availability for rollups.** Not our concern at v1; we leave that to EIP-4844, Celestia, EigenDA, and others.

### 1.5 Relationship to Filecoin

Prova is not a Filecoin competitor. It is what Filecoin would be if designed on Ethereum in 2026. We use Filecoin-originated PDP and CommP technology under MIT license, with full attribution. Storage providers currently operating on Filecoin can operate on Prova with the same core technology stack, and the two networks can coexist indefinitely. If the Filecoin ecosystem ships an Ethereum-native story of its own, the two can merge, coexist, or remain complementary — nothing in Prova's design is adversarial to Filecoin.

---

## 2. Paper Structure

- **§3** covers the architecture: Ethereum contracts, off-chain prover nodes, optional aggregators, and client SDKs.
- **§4** specifies the PDP protocol as used by Prova, including challenge generation, response format, and on-chain verification.
- **§5** specifies the staking and slashing economics.
- **§6** specifies the dispute protocol for retrievability and proof validity.
- **§7** specifies content addressing (CommP, CID interop, ENS binding).
- **§8** specifies the TEE-attested fast path (optional, inherited from v1 spec).
- **§9** specifies client-side workflows: storing a file, retrieving a file, hosting a site on `.eth`.
- **§10** specifies the token (PROVA ERC-20) and the points program that precedes any token launch.
- **§11** discusses future work: ZK aggregation, QBP for verifiable inference, cross-L2 expansion.
- **§12** discusses threat model and security considerations.
- **§13** concludes.

*[Rest of paper to be rewritten from whitepaper.md v0.2 source after pivot.]*

---

*This abstract supersedes the v0.2 abstract. See PIVOT.md for context.*
