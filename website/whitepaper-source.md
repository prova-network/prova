# Prova: A Verifiable Storage, Compute, and Networking Protocol

**Version 0.2 — Draft**
**Author: Prova Team**
**April 2026**

---

## Abstract

We present Prova, a Layer 1 blockchain that unifies verifiable storage, AI compute, and virtual networking into a single protocol. Nodes simultaneously store data with cryptographic proofs of possession, execute AI inference with deterministic verification, and provide routable network connectivity to tenant workloads. The storage layer offers three proof tiers: **Provable Data Possession (PDP)** as the universal baseline (minute-scale onboarding), an optional **TEE-attested fast path** that reduces onboarding to seconds via hardware-managed encryption with unique-replica guarantees, and optional **Proof of Replication (PoRep)** for cold archival storage. The compute layer introduces **Quantized Bisection Proofs (QBP)**, which exploit the determinism of integer-quantized neural network inference to enable efficient fraud proofs with O(log L) verification cost for L-layer models. The networking layer provides IPv6-first connectivity with IPv4 fallback via SNI routing, turning isolated VMs into addressable internet services. Combined with staking-based economic security and random audit sampling, Prova achieves practical proof of compute with defense-in-depth storage verification, mathematics as the security foundation, hardware attestation as an optimization, never a requirement. All data is treated equally: one byte earns one byte of reward, with no privileged data classes or quality multipliers.

---

## 1. Introduction

### 1.1 The Convergence Problem

Artificial intelligence and decentralized infrastructure are on a collision course. AI workloads require massive storage (training datasets, model weights, embeddings, checkpoints), massive compute (training, inference, fine-tuning), and reliable network connectivity (serving predictions, distributing models, federating training). Today, all three are overwhelmingly served by centralized cloud providers. Amazon Web Services, Google Cloud Platform, and Microsoft Azure collectively control over 65% of the cloud infrastructure market.

This centralization creates systemic risks:

- **Vendor lock-in**: Migrating petabytes of training data between providers is prohibitively expensive
- **Censorship risk**: Model weights, datasets, and inference services can be unilaterally restricted
- **Opacity**: No verifiable guarantee that data is stored correctly or that compute was performed honestly
- **Cost**: Cloud margins of 30-60% are passed to AI developers, inflating the cost of intelligence
- **Single points of failure**: Outages at a single provider can take down thousands of dependent services

Decentralized alternatives exist for storage (Filecoin, Arweave) and compute (Akash, Render, io.net) independently, but no protocol unifies storage, compute, and networking with cryptographic verification. This gap is not merely architectural. It reflects a fundamental missing set of primitives: **proof of compute** and **verifiable service delivery**.

### 1.2 The Proof Gap

Proof of storage is a solved problem. Filecoin's Proof of Replication (PoRep) and Proof of Spacetime (PoSt) provide cryptographic guarantees that data is stored uniquely and persistently. Provable Data Possession (PDP) offers lighter-weight proofs for hot data. These protocols have been battle-tested across years of mainnet operation.

Proof of compute remains unsolved in the general case. Verifying that a node performed an arbitrary computation correctly, without re-executing it, is equivalent to succinct verification of computation, the domain of zero-knowledge proofs. While ZK-SNARKs and ZK-STARKs can verify computation succinctly, they are orders of magnitude too expensive for GPU workloads. Generating a ZK proof of a single large language model inference would take longer than running the inference itself hundreds of times.

Alternative approaches rely on trusted hardware (TEE/SGX), which introduces manufacturer trust assumptions and has been repeatedly broken by side-channel attacks, or on simple redundant execution, which multiplies cost linearly.

Meanwhile, no decentralized compute network provides real network connectivity to its workloads. VMs without internet access cannot serve traffic, pull dependencies, or participate in distributed systems, limiting them to batch processing at best.

### 1.3 Our Contribution

Prova makes three contributions:

1. **Quantized Bisection Proofs (QBP)**: A verification protocol that exploits the determinism of integer-quantized neural network inference to enable fraud proofs with logarithmic verification cost.

2. **Three-tier storage proofs**: A defense-in-depth storage layer combining PDP (mathematical, fast), TEE attestation (hardware-accelerated), and optional PoRep (cold archival), where all tiers earn equal rewards and TEE is never a sole trust assumption.

3. **Chain-managed virtual networking**: An IPv6-first networking layer that gives tenant workloads routable internet addresses, HTTP/HTTPS IPv4 fallback via SNI, and on-chain bandwidth accounting, turning Prova nodes from isolated execution environments into full-stack service hosts.

The key insight behind QBP is that we do not need to solve proof of compute in the general case. We need to solve it for a specific, high-value class of computation: neural network inference on quantized models. This constraint makes the problem tractable.

The key insight behind the networking layer is that compute without connectivity is storage with extra steps. Real workloads need to talk to the internet.

---

## 2. Background

### 2.1 Proof of Storage

**Proof of Replication (PoRep)** proves that a storage provider has created a unique, sealed copy of data. The sealing process is computationally expensive (hours per 32 GiB sector), creating a proof-of-work-like cost that prevents providers from generating proofs without actually storing data. PoRep is well-suited for cold archival storage where data is rarely accessed.

**Proof of Spacetime (PoSt)** proves ongoing storage over time. Providers must periodically demonstrate continued possession of sealed data through WindowPoSt proofs submitted on-chain.

**Provable Data Possession (PDP)** is a lighter-weight proof mechanism. Rather than sealing data into a unique encoding, PDP stores raw data and proves possession through Merkle inclusion proofs against random challenges. A set of data roots (CommP hashes) is registered on-chain, and the protocol periodically challenges the provider to prove possession of randomly selected segments. PDP verification costs scale logarithmically (O(log N) for N roots in the proof set), making it efficient even at exabyte scale. PDP trades the anti-outsourcing guarantees of PoRep for dramatically faster onboarding (minutes vs hours) and native support for hot/warm data that needs to be read frequently.

**TEE-attested storage** is a third approach, where a Trusted Execution Environment (Intel SGX, AMD SEV-SNP) manages disk encryption with hardware-sealed keys. The TEE provides unique replica guarantees (same data, different ciphertext per machine) and near-instant onboarding (AES encryption at hardware speed). The trade-off is a shift in trust assumption from mathematics to hardware manufacturers. TEE side-channel attacks (Foreshadow, SGAxe, ÆPIC Leak) have been discovered repeatedly, making TEE unsuitable as a sole proof mechanism but valuable as an optimization alongside mathematical proofs.

### 2.2 Quantized Neural Network Inference

Modern AI inference increasingly uses quantized models, where model weights and/or activations are represented with reduced precision, typically INT8 (8-bit integer) or INT4 (4-bit integer) rather than FP32 or FP16.

Quantization provides 2-4x speedup and proportional memory reduction with minimal accuracy loss. The industry trend is strongly toward quantized deployment: NVIDIA's TensorRT, Apple's Core ML, Google's TensorFlow Lite, and the llama.cpp ecosystem all prioritize quantized inference.

**The determinism property**: Integer arithmetic is fundamentally different from floating point:

- **Integer addition is associative**: `(a + b) + c = a + (b + c)` always. The order of accumulation does not affect the result.
- **Floating point addition is NOT associative**: Due to rounding at each step, `(a + b) + c ≠ a + (b + c)` in general. Parallel reductions (common in GPU computation) produce different results depending on thread scheduling.

In INT8 quantized inference, the core operation is:

```
output[i] = Σ(weight_int8[j] × activation_int8[j])  // INT8 × INT8 → INT32 accumulation
output_scaled[i] = output[i] × scale_factor          // INT32 → dequantize
```

The INT32 accumulation is deterministic regardless of accumulation order. The scale factor multiplication is a single FP32 operation on a known value, also deterministic. Therefore, given identical model weights, identical inputs, and identical quantization parameters, two machines will produce bit-identical outputs.

**Claim**: Quantized neural network inference (INT8 weights, INT32 accumulation) is deterministic across GPU architectures. This claim is being empirically validated (see Section 9).

### 2.3 Interactive Fraud Proofs

Interactive fraud proofs, pioneered in the context of optimistic rollups (Arbitrum, Optimism), enable efficient verification of computation through a challenge-response game. The key technique is **bisection**: when two parties disagree on the result of a computation, they binary-search to find the exact step where their execution histories diverge, then verify only that single step.

For a computation with N steps, bisection requires only O(log N) rounds of interaction, and the final verification requires executing only 1 step. This transforms verification from O(N) to O(1) with O(log N) rounds of communication.

### 2.4 Limitations of Existing Approaches

| Protocol | Storage | Compute | Networking | Core Limitation |
|----------|---------|---------|------------|-----------------|
| Filecoin | PoRep + PoSt + PDP | None | None | No compute or networking; PoRep onboarding too slow for hot data |
| Akash | None | Staking only | Overlay (no native routing) | No proofs of any kind; no real inbound connectivity |
| Bittensor | None | Validation subnet | None | Compute verification requires full re-execution |
| Render | None | Reputation | None | No cryptographic verification |
| io.net | None | Economic (staking) | None | No storage proofs; no compute verification |
| Arweave/AO | Proof of Access | Optimistic (AO) | None | Different proof model; not GPU-optimized |
| TEE-only | TEE attestation | TEE attestation | None | Hardware compromise = total failure |

No existing protocol provides cryptographic storage proofs, efficient compute verification, AND native network connectivity in a unified system.

---

## 3. Protocol Design

### 3.1 Architecture Overview

Prova operates as a three-layer system:

```
┌─────────────────────────────────────────────────────────────────┐
│                       SETTLEMENT LAYER                           │
│                    (Blockchain Consensus)                         │
│                                                                   │
│  ┌──────────┐  ┌──────────┐  ┌──────────┐  ┌──────────────────┐ │
│  │  Model    │  │  Proof   │  │  Stake   │  │  Payment         │ │
│  │ Registry  │  │  Sets    │  │  Ledger  │  │  Channels        │ │
│  └──────────┘  └──────────┘  └──────────┘  └──────────────────┘ │
│  ┌──────────────────┐  ┌──────────────────────────────────────┐  │
│  │  Address Registry │  │  Bandwidth Accounting               │  │
│  └──────────────────┘  └──────────────────────────────────────┘  │
├─────────────────────────────────────────────────────────────────┤
│                       EXECUTION LAYER                            │
│                 (Off-chain, Low-latency)                          │
│                                                                   │
│  ┌────────────────┐  ┌────────────────────┐  ┌────────────────┐ │
│  │ Storage Engine  │  │  Compute Engine    │  │ Network Engine │ │
│  │ ├─ PDP proofs   │  │  ├─ Quantized inf. │  │ ├─ IPv6 alloc. │ │
│  │ ├─ Data serving │  │  ├─ Activation     │  │ ├─ SNI routing │ │
│  │ ├─ TEE attest.  │  │  │   Merkle trees  │  │ ├─ Bandwidth  │ │
│  │ └─ Optional     │  │  ├─ Result delivery│  │ │   metering   │ │
│  │    PoRep        │  │  └─ Bisection game │  │ └─ Firewall    │ │
│  └────────────────┘  └────────────────────┘  └────────────────┘ │
└─────────────────────────────────────────────────────────────────┘
```

The settlement layer handles consensus, token transfers, proof verification, staking, address allocation, bandwidth accounting, and payment settlement. The execution layer handles actual data storage, retrieval, AI inference, and network traffic routing.

### 3.2 Storage Layer

#### 3.2.1 PDP-First Design

Unlike Filecoin's PoRep-first approach (which requires hours of GPU-intensive sealing before data can be stored), Prova uses PDP as its primary storage proof:

1. **Onboarding**: Provider receives raw data, computes CommP (SHA-256 Merkle root), registers the root on-chain. Time: minutes.
2. **Proving**: The protocol periodically selects random challenges via drand randomness. Provider responds with Merkle inclusion proofs. Verified on-chain in O(log N) gas.
3. **Retrieval**: Data is stored unsealed. Providers can serve it directly without unsealing. Ideal for hot/warm storage patterns.

#### 3.2.2 TEE-Attested Storage (Fast Path)

For providers with Trusted Execution Environment hardware (Intel SGX/TDX, AMD SEV-SNP), Prova offers a hardware-attested storage path that reduces onboarding from minutes to **seconds**.

A TEE enclave manages all disk encryption. The enclave holds the only keys that can read or write sector data. The host node, and the storage provider operating it, never sees plaintext keys. Each machine's enclave derives a unique master key from hardware-sealed storage, so the same data produces different ciphertext on different machines. This provides **unique replica guarantees without SNARK-based sealing**. Deduplication across machines is cryptographically impossible.

**Verification** uses random spot checks:

1. The chain selects random 4 KiB chunk indices within a sector
2. The enclave decrypts those chunks and verifies they contain the expected content
3. The enclave signs an attestation that the chunks are valid
4. The chain verifies the attestation against the registered enclave's public key and measurement hash

**Security model:** The enclave image is open source, auditable, and chain-versioned. New versions require governance approval. The enclave has no network access, only disk I/O and a sealed bytestream to the host node, minimizing attack surface.

**Critical design choice:** TEE is an optimization, never a requirement. If a TEE vulnerability is discovered, the chain can deprecate affected TCB levels via governance, and nodes fall back to PDP proofs during a grace period. The network's security is grounded in mathematics (PDP), with TEE as an acceleration layer.

TEE storage also synergizes with confidential inference (Section 3.3): model weights stored under TEE encryption can be loaded directly into TEE-attested inference without leaving the enclave boundary, protecting model intellectual property by hardware.

#### 3.2.3 Optional PoRep Tier

For cold archival storage where anti-outsourcing guarantees are desired, providers can optionally seal data using PoRep. Sealed sectors earn a modest bonus reward (e.g., 1.2x) to compensate for the sealing cost, but this is a cost-recovery mechanism, not a quality multiplier. All data is fundamentally equal.

#### 3.2.4 Three-Tier Summary

| Tier | Trust Basis | Onboarding | Unique Replica | Hardware Required | Fallback |
|------|------------|------------|----------------|-------------------|----------|
| **PDP** (baseline) | Mathematics | Minutes | No | None | — |
| **TEE** (fast path) | Hardware attestation | Seconds | Yes (per-machine encryption) | SGX/TDX/SEV | → PDP |
| **PoRep** (cold tier) | Mathematics (SNARKs) | Hours | Yes (sealed) | GPU | → PDP |

All three tiers earn equal rewards per byte stored. The choice is the provider's.

#### 3.2.5 Variable Sector Sizes

Prova does not mandate fixed sector sizes. Providers register proof sets with roots of any size from 1 MiB to 64 GiB+. This eliminates padding waste and allows providers to store data in its natural size.

### 3.3 Compute Layer

#### 3.3.1 Model Registry

Models available for verified inference are registered on-chain:

```
ModelRegistration {
    model_id:       Hash,
    name:           String,           // e.g., "llama-3-70b-int8"
    quantization:   QuantSpec,        // INT8/INT32, INT4/INT32, etc.
    layer_count:    u32,
    weight_hashes:  Vec<Hash>,        // SHA-256 per layer weights
    total_hash:     Hash,             // SHA-256 of complete weights
    input_spec:     TensorSpec,       // Expected input format
    output_spec:    TensorSpec,       // Expected output format
    arch_group:     ArchGroup,        // GPU architecture compatibility group
    registered_by:  Address,
    stake:          TokenAmount,      // Registrant stakes correctness
}
```

The weight hashes per layer are critical. They enable layer-level verification during bisection without downloading the entire model. The `arch_group` field binds the registration to a specific GPU architecture group (see Section 9.4).

#### 3.3.2 Inference Flow

```
Client                          Node                           Chain
  │                               │                               │
  ├── InferenceRequest ──────────>│                               │
  │   (model_id, input, fee)      │                               │
  │                               ├── Run quantized inference     │
  │                               ├── Build activation Merkle tree│
  │                               │                               │
  │<── InferenceResult ───────────┤                               │
  │   (output, merkle_root,       │                               │
  │    signature)                 │                               │
  │                               ├── Commit(result_hash, ───────>│
  │                               │         merkle_root)          │
  │                               │                               │
  │            Challenge window (T epochs)                         │
  │                               │                               │
  │                  [If no challenge: finalize, pay node]         │
  │                  [If challenged: bisection game]               │
```

#### 3.3.3 Quantized Bisection Proofs (QBP)

The core verification protocol.

**Definitions:**
- Model M with L layers: M = (f_1, f_2, ..., f_L)
- Input: x
- Intermediate activations: h_0 = x, h_i = f_i(h_{i-1}) for i = 1..L
- Output: y = h_L
- Activation Merkle tree: T = MerkleTree(H(h_0), H(h_1), ..., H(h_L)), where H is SHA-256 over the serialized tensor

**Protocol:**

1. **Commit phase**: Prover P submits (y, root(T)). Deposit stake S.

2. **Challenge phase**: If challenger C disagrees, C submits (y', root(T')) with stake S'. Both parties have committed to their full execution trace via the Merkle root.

3. **Bisection phase**: The protocol binary-searches for the first layer where P and C disagree:
   ```
   lo = 0, hi = L
   while hi - lo > 1:
       mid = (lo + hi) / 2
       P reveals H(h_mid) with Merkle proof against root(T)
       C reveals H(h'_mid) with Merkle proof against root(T')
       if H(h_mid) == H(h'_mid):
           lo = mid    // Agree up to mid, disagreement is in upper half
       else:
           hi = mid    // Disagreement is in lower half
   ```
   After log_2(L) rounds, `lo` is the last agreed layer and `hi = lo + 1` is the first disagreed layer.

4. **Verification phase**: Both parties reveal the agreed input activation h_lo (with Merkle proof) and their claimed outputs h_hi and h'_hi (with Merkle proofs). The on-chain verifier re-executes layer f_hi on input h_lo using the registered model weights for layer hi.

   - If verifier_output == h_hi: P is honest, C is slashed.
   - If verifier_output == h'_hi: C is honest, P is slashed.
   - If neither matches: both slashed.

**Complexity:**
- Communication: O(log L) rounds, each with two Merkle proofs
- On-chain verification: 1 layer forward pass
- For an 80-layer model: 7 bisection rounds + 1 layer verification
- Verifier performs less than 2% of the original computation

#### 3.3.4 Random Audit Protocol

Not every inference is challenged. Prova uses probabilistic auditing:

1. Each epoch, the protocol selects a random subset of recent inference commitments for audit (target: 5% audit rate, configurable by governance).
2. For selected inferences, a second randomly-chosen node within the same architecture group re-executes the inference.
3. If results match: both nodes receive a small audit reward.
4. If results differ: bisection game determines fault; the dishonest node is slashed.

The audit rate, combined with slashing penalties, establishes the economic security of the system (see Section 5).

### 3.4 Networking Layer

Storage and compute are necessary but not sufficient. A VM that cannot talk to the internet cannot serve an API, pull a package, join a peer-to-peer network, or do anything a real server does. Prova's networking layer gives tenant workloads routable addresses and metered connectivity.

#### 3.4.1 Design Principles

- **Good for SPs**: No liability exposure from tenant traffic. Traffic filtering, rate controls, and common-carrier protections.
- **Good for tenants**: Off-the-shelf server software should just work. No custom SDKs, no tunnel clients, no overlay hacks.
- **Minimal trust**: Bandwidth metering is on-chain. Address allocation is deterministic. Abuse response is protocol-level.

#### 3.4.2 Outbound Connectivity

All tenant VMs may open arbitrary outbound TCP connections. This is the baseline capability that makes standard server software functional: package managers, API calls, blockchain RPC, webhook delivery, federated protocols.

**Abuse mitigation (SP-level):**

| Risk | Mitigation |
|------|------------|
| DDoS origination | Per-VM egress rate limits (configurable by SP, default 100 Mbps burst / 10 Mbps sustained). Protocol-level circuit breaker: if a VM exceeds 10x its sustained cap for >60s, traffic is throttled automatically. |
| DMCA/illicit downloads | Protocol-level BitTorrent fingerprint filter on outbound connections. SPs act in the ISP/common-carrier role; DMCA takedown endpoint baked into every SP's node software for automated response. |
| Network abuse to SP infrastructure | Tenant traffic is isolated in a network namespace. SP's own management traffic is never routable from tenant VMs. |

Common carrier principles apply: the SP provides the pipe; the tenant is responsible for what flows through it. The protocol's built-in DMCA handler satisfies the safe harbor requirements of 17 U.S.C. § 512, and comparable frameworks in other jurisdictions, without requiring SPs to police content proactively.

#### 3.4.3 Inbound Connectivity: IPv6-First

Prova registers and manages a global IPv6 address block through the Regional Internet Registry (RIR) system:

**Bootstrap (65K subnets):**
- Acquire a /48 via a sponsoring Local Internet Registry (LIR) for approximately $200/year
- 65,536 /64 subnets, each assignable to a tenant VM
- Sub-prefixed to SPs based on their registered capacity
- SPs announce their assigned prefixes via BGP

**Growth path (millions of subnets):**
- When utilization exceeds 80% of allocated space, upgrade to direct ARIN/RIPE membership (~$1,000/year)
- Governance proposal triggers the upgrade; fee paid from protocol treasury

**Per-VM addressing:**
```
AddressAllocation {
    vm_id:        VmId,
    ipv6_prefix:  /64,              // Unique per VM
    dns:          "vm{id}.{tenant}.prova.network",
    sp_id:        ProviderId,
    allocated_at: Epoch,
}
```

Each VM receives a /64 prefix and a DNS record under `prova.network`. Standard AAAA resolution. No client-side configuration needed.

**SP requirements for inbound IPv6:**
- **Tier 1 (datacenter SP):** Full BGP peering. Announces assigned prefix to upstream transit providers. Required for inbound IPv6 routing.
- **Tier 2 (home/small SP):** No BGP capability. Can provide outbound connectivity and storage/compute, but cannot host inbound-routable VMs. Earns storage and compute rewards only. May upgrade to Tier 1 by establishing BGP peering.

This tiering is explicit: not every SP can route inbound traffic, and the protocol does not pretend otherwise.

#### 3.4.4 IPv4 Fallback via SNI Routing

Approximately 40% of global internet traffic still cannot reach IPv6 destinations. For HTTP workloads, Prova provides IPv4 reachability through SNI-based routing:

**Supported (via shared IPv4 on SP's edge):**
- HTTP/1.1 and HTTP/2 (Host header routing)
- HTTPS/TLS 1.2+ (SNI extension carries hostname before encryption)
- HTTP/3 (QUIC contains TLS inside UDP, SNI available)

**Not supported over IPv4:**
- Raw TCP (databases, SSH, custom protocols): Use IPv6 directly. Operators with IPv6 connectivity (the majority of datacenter and developer traffic) are unaffected.
- SMTP: Technically possible but intentionally excluded. Email deliverability depends on IP reputation, SPF, DKIM, and years of sender history that a shared IPv4 cannot provide.

This is not a limitation, it is a design choice. IPv4 is a compatibility shim for web traffic. Everything else uses the VM's native IPv6 address. The protocol does not try to solve IPv4 exhaustion; it routes around it.

#### 3.4.5 Bandwidth Metering and Pricing

Network traffic is a metered resource, like storage and compute:

```
BandwidthReport {
    vm_id:        VmId,
    epoch:        Epoch,
    egress_bytes: u64,
    ingress_bytes: u64,
    sp_signature: Signature,
}
```

SPs submit bandwidth reports per epoch. Tenants pay per-GB egress (ingress is free, matching industry convention). Pricing is set per-SP with a protocol-enforced ceiling to prevent gouging.

**Verification:** The chain cannot directly observe network traffic. Instead, Prova uses economic verification:
- SPs report bandwidth. Tenants can dispute reports within a window.
- Random cross-checks: the protocol occasionally routes a synthetic request through a tenant's VM and verifies the SP's bandwidth accounting matches.
- Persistent over-reporting (compared to tenant-side counters or cross-checks) triggers investigation and potential slashing.

This is weaker than the cryptographic guarantees of the storage and compute layers, and we are explicit about that. Bandwidth verification is economic, not mathematical. The slashing risk makes systematic fraud unprofitable, but individual epoch reports are trusted-then-verified.

#### 3.4.6 DDoS Protection

Inbound DDoS is the primary risk to SP infrastructure from hosting routable VMs:

- **Per-VM ingress cap:** Protocol-enforced maximum (default 1 Gbps, configurable by SP). Traffic exceeding the cap is dropped at the SP's edge.
- **Blackhole routing:** SPs can blackhole a specific VM's prefix via a single on-chain transaction, taking effect within one block (~10 seconds). The VM loses inbound connectivity but the SP's network is protected.
- **Third-party integration:** SPs may optionally place Cloudflare, AWS Shield, or similar DDoS mitigation in front of their edge. The protocol does not mandate this but does not interfere with it.

The protocol's position: DDoS protection is an SP operational concern, not a consensus problem. The protocol provides the tools (per-VM caps, blackhole routing) and stays out of the way.

### 3.5 Node Architecture

A Prova node combines storage, compute, and networking capabilities:

```
┌───────────────────────────────────────────────────────────────┐
│                         PROVA NODE                             │
│                                                                 │
│  ┌──────────────┐  ┌──────────────────┐  ┌─────────────────┐  │
│  │   Storage     │  │    Compute       │  │   Networking    │  │
│  │   ├─ HDD/SSD  │  │   ├─ GPU(s)     │  │   ├─ IPv6 BGP   │  │
│  │   ├─ PDP      │  │   ├─ VRAM       │  │   ├─ SNI router │  │
│  │   └─ (PoRep)  │  │   └─ Quantized  │  │   ├─ Firewall   │  │
│  │               │  │     runtime      │  │   └─ BW meter   │  │
│  └───────┬───────┘  └────────┬─────────┘  └────────┬────────┘  │
│          │    DATA LOCALITY   │     VM CONNECTIVITY  │          │
│          └────────────────────┴──────────────────────┘          │
│    Model weights stored locally = instant inference             │
│    VMs have real IPs = real internet services                   │
│                                                                 │
│  ┌──────────────┐  ┌──────────────────────────────────────┐    │
│  │   TEE        │  │           Proof Engine                │    │
│  │   Enclave    │  │  ├─ PDP proof generation (baseline)   │    │
│  │   (optional) │  │  ├─ TEE attestation proofs            │    │
│  │   ├─ Disk    │  │  ├─ Activation Merkle tree builder    │    │
│  │   │  keys    │  │  ├─ Bisection game participant        │    │
│  │   ├─ Fast    │  │  ├─ Bandwidth report signing          │    │
│  │   │  onboard │  │  └─ (Optional) PoRep sealing          │    │
│  │   └─ Conf.   │  └──────────────────────────────────────┘    │
│  │     infer.   │                                               │
│  └──────────────┘                                               │
└───────────────────────────────────────────────────────────────┘
```

**Data locality advantage**: Because nodes store data AND run inference, model weights are always local. There is no need to transfer multi-gigabyte model files between storage and compute providers.

**Connectivity advantage**: Because nodes route traffic AND run VMs, tenant workloads have native IP addresses. There is no overlay network, no tunnel, no NAT traversal. A web server running on Prova is as reachable as one on AWS.

Nodes may specialize:
- **Storage-only**: Large disk, modest GPU, no BGP. Earns storage rewards.
- **Compute-only**: Powerful GPU, limited disk. Stores only the models it serves.
- **Full-stack**: Storage + compute + networking. Earns from all three reward streams.
- **Home nodes**: No BGP, no inbound routing. Can provide storage, compute, and outbound connectivity. Lower barrier to entry.

---

## 4. Consensus

### 4.1 Block Production

Prova uses a proof-of-stake consensus mechanism weighted by storage, compute, and networking contributions:

```
VotingPower(node) = α × StoragePower + β × ComputePower + γ × NetworkPower
```

Where:
- StoragePower = total bytes stored and proven (PDP + TEE + PoRep)
- ComputePower = demonstrated compute capacity (benchmark-based, periodically re-measured)
- NetworkPower = available inbound bandwidth (verified by periodic probes)
- α, β, γ = weighting parameters (governance-adjustable, initially α = 0.4, β = 0.4, γ = 0.2)

Block producers are selected proportionally to their voting power using a verifiable random function (VRF).

### 4.2 Finality

Prova uses a BFT-style finality gadget on top of block production. With a 10-second block time and 2/3 voting threshold, blocks achieve single-slot finality: once produced and attested, a block is final with no possibility of reorgs. This is a fundamental improvement over Filecoin's 900-epoch (~7.5 hour) probabilistic finality.

An epoch consists of 60 blocks (10 minutes). Epochs are the reward distribution boundary: at each epoch transition, mining rewards are calculated and split across storage, compute, and network contributors.

Inference results have a separate finality timeline. The challenge window (6 epochs, ~60 minutes) adds delay before inference results are considered settled. This dual-finality approach keeps block production fast (10s) while allowing time for compute verification via QBP.

---

## 5. Economic Security

### 5.1 Token Supply and Distribution

| Allocation | Percentage | Vesting |
|---|---|---|
| Mining rewards (storage + compute + network) | 55% | Released via minting function (~20 years, 5 halvings) |
| Team and founders | 15% | 18-month cliff, 3-year linear vest |
| Public sale (SAFT) | 10% | Three tiers: T1 $0.033 (35% off, 3yr vest), T2 $0.040 (20% off, 2yr vest), T3 $0.050 (base, 10% TGE + 12mo vest) |
| Ecosystem and grants | 10% | DAO-governed after year 1 |
| Community and airdrop | 5% | Testnet operators, bug bounty, early contributors |
| Seed round | 3% | 12-month cliff, 2-year linear vest |
| Liquidity provision | 2% | Unlocked at TGE for DEX pools |

Total supply is 1,000,000,000 PROVA, fixed at genesis. No reserve fund, no hidden allocations. Every token has a defined purpose.

### 5.2 Three Reward Streams

Each epoch, newly minted tokens are split between storage, compute, and networking rewards:

```
StorageReward(epoch)  = MintedTokens(epoch) × R_s(epoch)
ComputeReward(epoch)  = MintedTokens(epoch) × R_c(epoch)
NetworkReward(epoch)  = MintedTokens(epoch) × R_n(epoch)
```

Where R_s + R_c + R_n = 1 always. The split is determined by a **demand-weighted oracle**:

- If storage utilization is high relative to compute and network, R_s increases
- If compute utilization is high, R_c increases
- If bandwidth utilization is high, R_n increases
- Equilibrium: all resource types earn comparable ROI

This prevents the "everyone does the profitable thing" death spiral that plagues single-resource networks.

### 5.3 Pledge and Staking

```
Pledge(node) = BasePledge + ExpectedReward × LockMultiplier
```

- **BasePledge**: Minimum stake to participate (anti-spam, scales with claimed capacity)
- **ExpectedReward**: Projected earnings over commitment period
- **LockMultiplier**: Simple constant (e.g., 0.5x)

No quality adjustments, no deal-weighted calculations, no sector-type multipliers.

### 5.4 Slashing Economics

For the audit mechanism to provide security, the expected value of cheating must be negative:

```
EV(cheat) = P(not_audited) × savings - P(audited) × slash_amount
          = (1 - r) × s - r × S
```

Where:
- r = audit rate (0.05 = 5%)
- s = savings from not running inference (the reward for one job)
- S = slash amount (stake at risk)

For EV(cheat) < 0: S > ((1 - r) / r) x s = 19 x s

With a stake of 20x a single job reward and a 5% audit rate, cheating is always irrational. The more a node cheats, the more likely it is to be caught at least once, and a single catch loses the entire stake.

| Jobs cheated | P(detection) |
|---|---|
| 1 | 5% |
| 10 | 40% |
| 20 | 64% |
| 50 | 92% |
| 100 | 99.4% |

### 5.5 Streaming Payments

Client-to-node payments use streaming payment channels at the protocol level:

- Client locks `rate × period + fixed_deposit` into a payment channel
- Funds stream from client to node per epoch
- Settlement on-chain at configurable intervals
- No intermediary takes a cut (protocol fee only, set by governance)

---

## 6. Comparison to Prior Art

### 6.1 vs. Filecoin

Prova inherits Filecoin's proven storage proof mechanisms (PDP, PoRep, PoSt) but diverges in three areas:

1. **PDP-first vs PoRep-first**: Prova optimizes for hot/warm data with fast onboarding.
2. **Compute + networking**: Prova natively verifies AI inference and provides tenant connectivity. Filecoin has neither.
3. **Equal data**: All bytes are equal. Filecoin's Fil+ program creates a 10x multiplier for "verified" data, distorting economics.

### 6.2 vs. Akash Network

Akash provides a decentralized compute marketplace but has no storage proofs, no compute verification (nodes are trusted via staking alone), and no native inbound connectivity (relies on overlay networking with manual port forwarding). Prova adds cryptographic storage proofs, QBP for compute verification, and real IPv6 addresses for every VM.

### 6.3 vs. Bittensor

Bittensor incentivizes AI model quality through a validation subnet where validators re-run inference to check miner responses. This is O(N) verification (full re-execution). Prova's bisection protocol achieves O(1) verification with O(log L) communication, and only for audited jobs.

### 6.4 vs. Arweave/AO

Arweave provides permanent storage with Proof of Access. AO adds a compute layer with optimistic verification. Prova's approach is similar in spirit to AO but is GPU-native, specifically optimized for neural network inference, and includes a networking layer that AO lacks.

---

## 7. Roadmap

### Phase 1: Foundation (Q2 2026)
- Empirical validation of quantized determinism across GPU architectures
- Formal specification of QBP protocol
- Core chain implementation (PDP-first, clean economics)
- Private testnet with 3-5 nodes

### Phase 2: Testnet (Q3 2026)
- Public testnet launch
- Compute layer integration (model registry, inference flow, audit mechanism)
- IPv6 address block acquisition via LIR
- Networking layer prototype (outbound + IPv6 inbound on testnet nodes)
- Economic parameter tuning
- Developer SDK and CLI tooling

### Phase 3: Security (Q4 2026)
- External security audit
- Bisection protocol formal verification
- SNI routing and bandwidth metering hardening
- Stress testing at scale
- Bug bounty program

### Phase 4: Mainnet (Q1 2027)
- Token generation event
- Genesis block with all three layers active
- BGP peering with Tier 1 SPs
- Exchange listings
- Ecosystem grant program launch

---

## 8. Governance

### 8.1 Protocol Parameters

The following parameters are governance-adjustable via on-chain voting:

- Reward split weights (α, β, γ)
- Audit rate (default 5%)
- Slash multiplier (default 20x)
- TEE TCB deprecation (emergency: expedited vote)
- IPv6 block expansion trigger (utilization threshold)
- Per-VM bandwidth caps (default and ceiling)
- Outbound traffic filter rules (BitTorrent fingerprints, etc.)

### 8.2 Upgrade Path

Protocol upgrades follow a propose-vote-activate cycle with a mandatory delay between vote passage and activation. This delay allows node operators to upgrade software and SPs to adjust networking configuration before new rules take effect.

### 8.3 Implementation Architecture

Prova uses a dual-language architecture: **Go for the node application, Rust for the proof core.** This follows the same pattern proven by Filecoin/Lotus (Go application calling Rust cryptographic libraries via FFI).

- **Rust:** Consensus state machine, proof verification (PDP, QBP), Merkle tree construction, cryptographic primitives, deterministic computation. Performance-critical and safety-critical code where Rust's guarantees matter.
- **Go:** Node orchestration, P2P networking (go-libp2p), RPC server, CLI tooling, networking layer (IPv6/SNI/bandwidth), operator-facing infrastructure. Leverages the team's deep Go experience from building Filecoin storage infrastructure.
- **Solidity:** ERC-20 token contracts (pre-mainnet phase, Ethereum).
- **TypeScript + Python:** Developer SDKs for adoption.

The Go node wraps the Rust core via C FFI, identical to how Lotus wraps filecoin-ffi. This boundary is well-understood by the team and allows each language to play to its strengths.

---

## 9. Experimental Validation

The foundational claim of Prova's compute verification, that quantized inference is deterministic across GPU architectures, is currently being empirically tested.

### 9.1 Experimental Setup

- **Machine A**: NVIDIA RTX 5080 (Blackwell architecture, compute capability 12.0, 16 GB VRAM)
- **Machine B**: NVIDIA Quadro RTX 6000 (Turing architecture, compute capability 7.5, 23 GB VRAM)
- **Model**: TinyLlama 1.1B, GGUF Q8_0 quantization
- **Framework**: llama.cpp (identical version on both machines)
- **Test**: Identical prompts, identical parameters, bit-level output comparison across 1,000+ inference runs

### 9.2 Results

**Single-GPU determinism: CONFIRMED**
- RTX 5080 (Blackwell, compute 12.0): 20/20 runs bit-identical
- RTX 6000 (Turing, compute 7.5): 20/20 runs bit-identical
- Both architectures produce perfectly reproducible output on the same hardware

**Cross-architecture determinism: FAILS**
- Outputs diverge at the very first generated token
- Not a subtle bit-flip: entirely different generation paths

### 9.3 Analysis

The failure is expected upon reflection. GGUF Q8_0 stores weights as INT8 but computes in FP16/FP32. Different GPU architectures implement FMA (fused multiply-add) units with different intermediate precision and rounding behavior, producing different floating-point results. With temperature 0 (greedy sampling), even a tiny logit difference changes which token is selected, causing cascading divergence.

**Critical distinction**: This experiment tested llama.cpp's GGUF Q8_0, which is NOT pure integer inference. True INT8-to-INT32 accumulation (available in TensorRT and custom CUDA kernels) avoids floating-point entirely and may be cross-architecture deterministic. This remains to be tested.

### 9.4 Protocol Adaptation

The QBP protocol remains viable with architectural awareness:

**Approach 1: Architecture-Locked Verification Groups (Immediate)**

Nodes are grouped by GPU compute capability. Verification occurs only within the same architecture group. Single-GPU determinism guarantees correctness within each group. This reduces the verifier pool per architecture but is functional immediately.

**Approach 2: True Integer Inference (Research)**

TensorRT's strict INT8 mode performs the entire pipeline in integers: INT8 weights x INT8 activations to INT32 accumulation to requantize to INT8. No floating-point operations involved. If empirically validated as cross-architecture deterministic, this eliminates the need for architecture grouping.

**Approach 3: Canonical CPU Verification (Hybrid)**

Use GPU for execution speed. Define a canonical CPU computation path (x86-64, IEEE 754 strict mode, deterministic thread scheduling) for the single-layer verification step in bisection. Since the verifier only re-executes one layer, CPU speed is sufficient. This provides cross-architecture verification at the cost of requiring a reference CPU implementation.

The protocol specification requires architecture-aware verification by default, with cross-architecture support as a future enhancement pending experimental validation.

---

## 10. Conclusion

Prova addresses a fundamental gap in decentralized infrastructure: the absence of a unified, verifiable storage, compute, and networking protocol. By combining proven storage proof mechanisms with a novel compute verification protocol and a practical networking layer, Prova enables a new class of decentralized applications where data integrity, computation correctness, and service reachability are guaranteed by the protocol rather than by trust in a provider.

The three technical contributions, Quantized Bisection Proofs, three-tier defense-in-depth storage, and chain-managed virtual networking, are each practical rather than theoretical. QBP exploits domain-specific determinism rather than solving general verifiable computation. The storage tiers use existing, battle-tested proof systems. The networking layer builds on standard internet infrastructure (IPv6, BGP, SNI) rather than inventing overlay protocols.

Prova is not an incremental improvement on existing networks. It is the infrastructure for a world where "deploy a verified AI service" is as simple as "deploy a container."

---

## References

1. Filecoin: A Decentralized Storage Network. Protocol Labs, 2017.
2. PoRep: Proofs of Replication. Fisch, B., 2018.
3. Provable Data Possession at Untrusted Stores. Ateniese et al., 2007.
4. Arbitrum: Scalable, private smart contracts. Kalodner et al., 2018.
5. TrueBit: A scalable verification solution for blockchains. Teutsch & Reitwiesner, 2017.
6. A Survey of Quantization Methods for Efficient Neural Network Inference. Gholami et al., 2021.
7. LLM.int8(): 8-bit Matrix Multiplication for Transformers at Scale. Dettmers et al., 2022.
8. GGML: A Tensor Library for Machine Learning. Gerganov, 2023.
9. RFC 6177: IPv6 Address Assignment to End Sites. Narten et al., 2011.
10. RFC 8446: The Transport Layer Security (TLS) Protocol Version 1.3. Rescorla, 2018.

---

*Version 0.2 incorporates the virtual networking layer (credit: Andy) and structural improvements. This is a living document. Subsequent versions will incorporate empirical results from true integer inference testing and formal specifications.*
