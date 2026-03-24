# Prova: A Verifiable Storage and Compute Network

**Version 0.1 — Draft**
**Author: Prova Team**
**March 2026**

---

## Abstract

We present Prova, a Layer 1 blockchain that unifies verifiable storage and AI compute into a single protocol. Nodes simultaneously store data with cryptographic proofs of possession and execute AI inference with deterministic verification. The storage layer offers three proof tiers: **Provable Data Possession (PDP)** as the universal baseline (minute-scale onboarding), an optional **TEE-attested fast path** that reduces onboarding to seconds via hardware-managed encryption with unique-replica guarantees, and optional **Proof of Replication (PoRep)** for cold archival storage. The compute layer introduces a novel verification protocol — **Quantized Bisection Proofs (QBP)** — which exploits the determinism of integer-quantized neural network inference to enable efficient fraud proofs with O(log L) verification cost for L-layer models. Combined with staking-based economic security and random audit sampling, Prova achieves practical proof of compute with defense-in-depth storage verification — mathematics as the security foundation, hardware attestation as an optimization, never a requirement. All data is treated equally — one byte earns one byte of reward, with no privileged data classes or quality multipliers.

---

## 1. Introduction

### 1.1 The Convergence Problem

Artificial intelligence and decentralized infrastructure are on a collision course. AI workloads require massive storage (training datasets, model weights, embeddings, checkpoints) and massive compute (training, inference, fine-tuning). Today, both are overwhelmingly served by centralized cloud providers — Amazon Web Services, Google Cloud Platform, and Microsoft Azure collectively control over 65% of the cloud infrastructure market.

This centralization creates systemic risks:

- **Vendor lock-in**: Migrating petabytes of training data between providers is prohibitively expensive
- **Censorship risk**: Model weights, datasets, and inference services can be unilaterally restricted
- **Opacity**: No verifiable guarantee that data is stored correctly or that compute was performed honestly
- **Cost**: Cloud margins of 30-60% are passed to AI developers, inflating the cost of intelligence

Decentralized alternatives exist for storage (Filecoin, Arweave) and compute (Akash, Render, io.net) independently, but no protocol unifies both with cryptographic verification. This gap is not merely architectural — it reflects a fundamental missing primitive: **proof of compute**.

### 1.2 The Proof Gap

Proof of storage is a solved problem. Filecoin's Proof of Replication (PoRep) and Proof of Spacetime (PoSt) provide cryptographic guarantees that data is stored uniquely and persistently. Provable Data Possession (PDP) offers lighter-weight proofs for hot data. These protocols have been battle-tested across years of mainnet operation.

Proof of compute remains unsolved in the general case. Verifying that a node performed an arbitrary computation correctly — without re-executing it — is equivalent to succinct verification of computation, which is the domain of zero-knowledge proofs. While ZK-SNARKs and ZK-STARKs can verify computation succinctly, they are orders of magnitude too expensive for GPU workloads. Generating a ZK proof of a single large language model inference would take longer than running the inference itself hundreds of times.

Alternative approaches rely on trusted hardware (TEE/SGX), which introduces manufacturer trust assumptions and has been repeatedly broken by side-channel attacks, or on simple redundant execution, which multiplies cost linearly.

### 1.3 Our Contribution

Prova introduces **Quantized Bisection Proofs (QBP)**, a verification protocol that exploits a specific property of modern AI inference: **quantized models using integer arithmetic produce deterministic outputs across hardware architectures**. This determinism, combined with the natural layer-by-layer structure of neural networks, enables an interactive fraud proof protocol with logarithmic verification cost.

The key insight is that we do not need to solve proof of compute in the general case. We need to solve it for a specific, high-value class of computation: neural network inference on quantized models. This constraint makes the problem tractable.

---

## 2. Background

### 2.1 Proof of Storage

**Proof of Replication (PoRep)** proves that a storage provider has created a unique, sealed copy of data. The sealing process is computationally expensive (hours per 32 GiB sector), creating a proof-of-work-like cost that prevents providers from generating proofs without actually storing data. PoRep is well-suited for cold archival storage where data is rarely accessed.

**Proof of Spacetime (PoSt)** proves ongoing storage over time. Providers must periodically demonstrate continued possession of sealed data through WindowPoSt proofs submitted on-chain.

**Provable Data Possession (PDP)** is a lighter-weight proof mechanism. Rather than sealing data into a unique encoding, PDP stores raw data and proves possession through Merkle inclusion proofs against random challenges. A set of data roots (CommP hashes) is registered on-chain, and the protocol periodically challenges the provider to prove possession of randomly selected segments. PDP verification costs scale logarithmically — O(log N) for N roots in the proof set — making it efficient even at exabyte scale.

PDP trades the anti-outsourcing guarantees of PoRep for dramatically faster onboarding (minutes vs hours) and native support for hot/warm data that needs to be read frequently.

**TEE-attested storage** is a third approach, where a Trusted Execution Environment (Intel SGX, AMD SEV-SNP) manages disk encryption with hardware-sealed keys. The TEE provides unique replica guarantees (same data, different ciphertext per machine) and near-instant onboarding (AES encryption at hardware speed). The trade-off is a shift in trust assumption from mathematics to hardware manufacturers — TEE side-channel attacks (Foreshadow, SGAxe, ÆPIC Leak) have been discovered repeatedly, making TEE unsuitable as a sole proof mechanism but valuable as an optimization alongside mathematical proofs.

### 2.2 Quantized Neural Network Inference

Modern AI inference increasingly uses quantized models, where model weights and/or activations are represented with reduced precision — typically INT8 (8-bit integer) or INT4 (4-bit integer) rather than FP32 (32-bit floating point) or FP16 (16-bit floating point).

Quantization provides 2-4× speedup and proportional memory reduction with minimal accuracy loss. The industry trend is strongly toward quantized deployment: NVIDIA's TensorRT, Apple's Core ML, Google's TensorFlow Lite, and the llama.cpp ecosystem all prioritize quantized inference.

**The determinism property**: Integer arithmetic is fundamentally different from floating point:

- **Integer addition is associative**: `(a + b) + c = a + (b + c)` always. The order of accumulation does not affect the result.
- **Floating point addition is NOT associative**: Due to rounding at each step, `(a + b) + c ≠ a + (b + c)` in general. This means parallel reductions (common in GPU computation) produce different results depending on thread scheduling.

In INT8 quantized inference, the core operation is:
```
output[i] = Σ(weight_int8[j] × activation_int8[j])  // INT8 × INT8 → INT32 accumulation
output_scaled[i] = output[i] × scale_factor          // INT32 → dequantize
```

The INT32 accumulation is deterministic regardless of accumulation order. The scale factor multiplication is a single FP32 operation on a known value — also deterministic. Therefore, given identical model weights, identical inputs, and identical quantization parameters, two machines will produce bit-identical outputs.

**Claim**: Quantized neural network inference (INT8 weights, INT32 accumulation) is deterministic across GPU architectures.

This claim is being empirically validated (see Section 8).

### 2.3 Interactive Fraud Proofs

Interactive fraud proofs, pioneered in the context of optimistic rollups (Arbitrum, Optimism), enable efficient verification of computation through a challenge-response game. The key technique is **bisection**: when two parties disagree on the result of a computation, they binary-search to find the exact step where their execution histories diverge, then verify only that single step.

For a computation with N steps, bisection requires only O(log N) rounds of interaction, and the final verification requires executing only 1 step. This transforms verification from O(N) to O(1) with O(log N) rounds of communication.

### 2.4 Limitations of Existing Approaches

| Protocol | Storage Proofs | Compute Verification | Limitation |
|----------|---------------|---------------------|------------|
| Filecoin | PoRep + PoSt + PDP | None | No compute layer; PoRep onboarding too slow for hot data |
| Akash | None | Staking only | No proof that computation was performed correctly |
| Bittensor | None | Validation subnet (replication) | No storage proofs; compute verification requires full re-execution |
| Render | None | Reputation | No cryptographic verification of either storage or compute |
| io.net | None | Economic (staking) | No storage proofs; no compute verification |
| Arweave/AO | Proof of Access | Optimistic (AO) | Different proof model; not GPU-optimized |
| TEE-only proposals | TEE attestation | TEE attestation | Single trust assumption; hardware compromise = total failure |

No existing protocol provides cryptographic storage proofs AND efficient compute verification in a unified system. Proposals that rely solely on TEE for both storage and compute verification inherit a single point of failure — when a TEE generation is compromised (a recurring event), the entire proof system breaks. Prova's defense-in-depth approach uses mathematical proofs as the foundation with TEE as an optional accelerator.

---

## 3. Protocol Design

### 3.1 Architecture Overview

Prova operates as a two-layer system:

```
┌─────────────────────────────────────────────────────────────┐
│                     SETTLEMENT LAYER                         │
│                  (Blockchain Consensus)                       │
│                                                              │
│  ┌──────────┐  ┌──────────┐  ┌──────────┐  ┌─────────────┐ │
│  │  Model    │  │  Proof   │  │  Stake   │  │  Payment    │ │
│  │ Registry  │  │  Sets    │  │  Ledger  │  │  Channels   │ │
│  └──────────┘  └──────────┘  └──────────┘  └─────────────┘ │
├─────────────────────────────────────────────────────────────┤
│                     EXECUTION LAYER                          │
│               (Off-chain, Low-latency)                       │
│                                                              │
│  ┌──────────────────┐  ┌──────────────────────────────────┐ │
│  │  Storage Engine   │  │  Compute Engine                  │ │
│  │  ├─ PDP proofs    │  │  ├─ Quantized inference          │ │
│  │  ├─ Data serving  │  │  ├─ Activation Merkle trees      │ │
│  │  └─ Optional PoRep│  │  ├─ Result delivery              │ │
│  │                    │  │  └─ Bisection game (if needed)   │ │
│  └──────────────────┘  └──────────────────────────────────┘ │
└─────────────────────────────────────────────────────────────┘
```

The settlement layer handles consensus, token transfers, proof verification, staking, and payment settlement. The execution layer handles actual data storage, retrieval, and AI inference at low latency.

### 3.2 Storage Layer

#### 3.2.1 PDP-First Design

Unlike Filecoin's PoRep-first approach (which requires hours of GPU-intensive sealing before data can be stored), Prova uses PDP as its primary storage proof:

1. **Onboarding**: Provider receives raw data, computes CommP (SHA-256 Merkle root), registers the root on-chain. Time: minutes.
2. **Proving**: The protocol periodically selects random challenges via drand randomness. Provider responds with Merkle inclusion proofs. Verified on-chain in O(log N) gas.
3. **Retrieval**: Data is stored unsealed — providers can serve it directly without unsealing. Ideal for hot/warm storage patterns.

#### 3.2.2 TEE-Attested Storage (Fast Path)

For providers with Trusted Execution Environment hardware (Intel SGX/TDX, AMD SEV-SNP), Prova offers a hardware-attested storage path that reduces onboarding from minutes to **seconds**.

The design is simple: a TEE enclave manages all disk encryption. The enclave holds the only keys that can read or write sector data. The host node — and the storage provider operating it — never sees plaintext keys. Each machine's enclave derives a unique master key from hardware-sealed storage, so the same data produces different ciphertext on different machines. This provides **unique replica guarantees without SNARK-based sealing** — deduplication across machines is cryptographically impossible.

**Verification** uses random spot checks rather than Merkle proofs:

1. The chain selects random 4 KiB chunk indices within a sector
2. The enclave decrypts those chunks and verifies they contain the expected content (provider ID, sector ID, chunk ID for empty space; inline hashes for stored data)
3. The enclave signs an attestation that the chunks are valid
4. The chain verifies the attestation against the registered enclave's public key and measurement hash

**Security model:** The enclave image is open source, auditable, and chain-versioned. New versions require governance approval. The enclave has no network access — only disk I/O and a sealed bytestream to the host node, minimizing attack surface.

**Critical design choice:** TEE is an optimization, never a requirement. If a TEE vulnerability is discovered (as has happened repeatedly with SGX side-channel attacks), the chain can deprecate affected TCB levels via governance, and nodes fall back to PDP proofs during a grace period. The network's security is grounded in mathematics (PDP), with TEE as an acceleration layer.

The TEE storage path also synergizes with confidential inference (Section 3.3): model weights stored under TEE encryption can be loaded directly into TEE-attested inference without ever leaving the enclave boundary — protecting model intellectual property by hardware.

See SPEC-026 for the full protocol specification including sector format, encryption hierarchy, migration, and deprecation protocol.

#### 3.2.3 Optional PoRep Tier

For cold archival storage where anti-outsourcing guarantees are desired, providers can optionally seal data using PoRep. Sealed sectors earn a modest bonus reward (e.g., 1.2×) to compensate for the sealing cost, but this is a cost-recovery mechanism, not a quality multiplier — all data is fundamentally equal.

#### 3.2.4 Three-Tier Storage Proof Summary

| Tier | Trust Basis | Onboarding | Unique Replica | Hardware Required | Fallback |
|------|------------|------------|----------------|-------------------|----------|
| **PDP** (baseline) | Mathematics | Minutes | No | None | — |
| **TEE** (fast path) | Hardware attestation | Seconds | Yes (per-machine encryption) | SGX/TDX/SEV | → PDP |
| **PoRep** (cold tier) | Mathematics (SNARKs) | Hours | Yes (sealed) | GPU | → PDP |

All three tiers earn equal rewards per byte stored. The choice is the provider's — Prova does not privilege one proof mechanism over another.

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
    registered_by:  Address,
    stake:          TokenAmount,      // Registrant stakes correctness
}
```

The weight hashes per layer are critical — they enable layer-level verification during bisection without downloading the entire model.

#### 3.3.2 Inference Flow

```
Client                          Node                           Chain
  │                               │                               │
  ├─── InferenceRequest ─────────>│                               │
  │    (model_id, input, fee)     │                               │
  │                               ├── Run quantized inference     │
  │                               ├── Build activation Merkle tree│
  │                               │                               │
  │<── InferenceResult ───────────┤                               │
  │    (output, merkle_root,      │                               │
  │     signature)                │                               │
  │                               ├── Commit(result_hash, ───────>│
  │                               │         merkle_root)          │
  │                               │                               │
  │              Challenge window (T epochs)                       │
  │                               │                               │
  │                    [If no challenge: finalize, pay node]       │
  │                    [If challenged: bisection game]             │
```

#### 3.3.3 Quantized Bisection Proofs (QBP)

The core verification protocol:

**Definitions:**
- Model M with L layers: M = (f₁, f₂, ..., f_L)
- Input: x
- Intermediate activations: h₀ = x, h_i = f_i(h_{i-1}) for i = 1..L
- Output: y = h_L
- Activation Merkle tree: T = MerkleTree(H(h₀), H(h₁), ..., H(h_L))
  where H is SHA-256 over the serialized tensor

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
   After log₂(L) rounds, `lo` is the last agreed layer and `hi = lo + 1` is the first disagreed layer.

4. **Verification phase**: Both parties reveal:
   - The agreed input activation h_lo (with Merkle proof)
   - Their claimed output h_hi and h'_hi (with Merkle proofs)
   
   The on-chain verifier re-executes layer f_hi on input h_lo using the registered model weights for layer hi.
   
   If verifier_output == h_hi: P is honest, C is slashed.
   If verifier_output == h'_hi: C is honest, P is slashed.
   If neither matches: both slashed (both dishonest or protocol error).

**Complexity:**
- Communication: O(log L) rounds, each with two Merkle proofs
- On-chain verification: 1 layer forward pass
- For an 80-layer model: 7 bisection rounds + 1 layer verification
- Verifier performs <2% of the original computation

#### 3.3.4 Random Audit Protocol

Not every inference is challenged. Instead, Prova uses probabilistic auditing:

1. Each epoch, the protocol selects a random subset of recent inference commitments for audit (target: 5% audit rate, configurable by governance).
2. For selected inferences, a second randomly-chosen node re-executes the inference.
3. If results match: both nodes receive a small audit reward.
4. If results differ: bisection game determines fault; dishonest node is slashed.

The audit rate, combined with slashing penalties, establishes the economic security of the system (see Section 5).

### 3.4 Node Architecture

A Prova node combines storage and compute capabilities:

```
┌──────────────────────────────────────────────────┐
│                    PROVA NODE                      │
│                                                    │
│  ┌────────────┐  ┌────────────────┐  ┌─────────┐ │
│  │   Storage   │  │    Compute     │  │   TEE   │ │
│  │   ├─ HDD/SSD│  │   ├─ GPU(s)   │  │ Enclave │ │
│  │   ├─ PDP    │  │   ├─ VRAM     │  │ (opt.)  │ │
│  │   └─ (PoRep)│  │   └─ Quantized│  │ ├─ Disk │ │
│  │             │  │     runtime    │  │ │  keys  │ │
│  └──────┬──────┘  └───────┬────────┘  │ ├─ Fast │ │
│         │  DATA LOCALITY  │           │ │ onboard│ │
│         └─────────────────┘           │ └─ Conf. │ │
│    Model weights stored locally =     │   infer. │ │
│    instant inference, no transfer     └─────────┘ │
│                                                    │
│  ┌──────────────────────────────────────────────┐ │
│  │              Proof Engine                     │ │
│  │  ├─ PDP proof generation (baseline)           │ │
│  │  ├─ TEE attestation proofs (fast path)        │ │
│  │  ├─ Activation Merkle tree builder            │ │
│  │  ├─ Bisection game participant                │ │
│  │  └─ (Optional) PoRep sealing                  │ │
│  └──────────────────────────────────────────────┘ │
└──────────────────────────────────────────────────┘
```

**Data locality advantage**: Because nodes store data AND run inference, model weights are always local. There is no need to transfer multi-gigabyte model files between storage and compute providers — the same node does both. This is a fundamental architectural advantage over systems where storage and compute are separate networks.

Nodes may specialize:
- **Storage-only**: Nodes with large disk but modest GPU. Earn storage rewards only.
- **Compute-only**: Nodes with powerful GPU but limited disk. Store only the models they serve.
- **Hybrid**: Nodes with both. Earn from both reward streams with the data locality bonus.

---

## 4. Consensus

### 4.1 Block Production

Prova uses a proof-of-stake consensus mechanism weighted by both storage and compute contributions:

```
VotingPower(node) = α × StoragePower(node) + β × ComputePower(node)
```

Where:
- StoragePower = total bytes stored and proven (PDP + PoRep)
- ComputePower = demonstrated compute capacity (benchmark-based, periodically re-measured)
- α, β = weighting parameters (governance-adjustable, initially α = β = 0.5)

Block producers are selected proportionally to their voting power using a verifiable random function (VRF).

### 4.2 Finality

Prova uses a BFT-style finality gadget on top of the block production mechanism. Blocks achieve finality after 2/3 of voting power attests, typically within 2-3 epochs (~60-90 seconds).

Inference results have a separate finality timeline — the challenge window (Section 3.3.2) adds an additional delay before inference results are considered settled. This dual-finality approach keeps block production fast while allowing time for compute verification.

---

## 5. Economic Security

### 5.1 Token Supply and Distribution

| Allocation | Percentage | Vesting |
|---|---|---|
| Mining rewards (storage + compute) | 60% | Released via minting function |
| Team and development | 15% | 4-year vest, 1-year cliff |
| Ecosystem and grants | 10% | DAO-governed after year 1 |
| Genesis operators | 10% | 1-year vest |
| Public distribution | 5% | TBD |

Total supply is fixed at genesis. No mining reserve, no hidden allocations.

### 5.2 Dual Reward Streams

Each epoch, newly minted tokens are split between storage and compute rewards:

```
StorageReward(epoch) = MintedTokens(epoch) × R_s(epoch)
ComputeReward(epoch) = MintedTokens(epoch) × R_c(epoch)
```

Where R_s + R_c = 1 always. The split is determined by a **demand-weighted oracle**:

- If storage utilization is high relative to compute → R_s increases (attract more storage)
- If compute utilization is high relative to storage → R_c increases (attract more compute)
- Equilibrium: both resource types earn comparable ROI

This prevents the "everyone does the profitable thing" death spiral that plagues single-resource networks.

### 5.3 Pledge and Staking

```
Pledge(node) = BasePledge + ExpectedReward × LockMultiplier
```

- **BasePledge**: Minimum stake to participate (anti-spam, scales with claimed capacity)
- **ExpectedReward**: Projected earnings over commitment period
- **LockMultiplier**: Simple constant (e.g., 0.5×)

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

For EV(cheat) < 0:
```
S > ((1 - r) / r) × s
S > (0.95 / 0.05) × s
S > 19 × s
```

With a stake of 20× a single job reward and a 5% audit rate, cheating is always irrational regardless of the number of jobs attempted. The more a node cheats, the more likely it is to be caught at least once, and a single catch loses the entire stake.

**Probability of detection over N cheated jobs:**
```
P(caught) = 1 - (1 - r)^N = 1 - 0.95^N
```

| Jobs cheated | P(detection) |
|---|---|
| 1 | 5% |
| 10 | 40% |
| 20 | 64% |
| 50 | 92% |
| 100 | 99.4% |

A node that cheats consistently will be caught with near certainty. The stake loss from a single detection exceeds the cumulative savings from all previous undetected cheating.

### 5.5 Streaming Payments

Client-to-node payments use streaming payment channels at the protocol level:

- Client locks `rate × period + fixed_deposit` into a payment channel
- Funds stream from client to node per epoch
- Settlement on-chain at configurable intervals
- No intermediary takes a cut (protocol fee only, set by governance)

This eliminates the need for upfront lump-sum payments and allows real-time adjustment of storage and compute consumption.

---

## 6. Comparison to Prior Art

### 6.1 vs. Filecoin

Prova inherits Filecoin's proven storage proof mechanisms (PDP, PoRep, PoSt) but diverges fundamentally in three areas:

1. **PDP-first vs PoRep-first**: Prova optimizes for hot/warm data with fast onboarding. Filecoin's sealing pipeline (hours per sector) is available but optional.
2. **Compute layer**: Prova natively verifies AI inference. Filecoin has no compute verification.
3. **Equal data**: All bytes are equal. Filecoin's Fil+ program creates a privileged 10× multiplier for "verified" data, distorting economics and concentrating rewards.

### 6.2 vs. Akash Network

Akash provides a decentralized compute marketplace but has no storage proofs and no compute verification — nodes are trusted based on staking alone. Prova adds cryptographic storage proofs and the QBP protocol for compute verification.

### 6.3 vs. Bittensor

Bittensor incentivizes AI model quality through a validation subnet where validators re-run inference to check miner responses. This is O(N) verification (full re-execution). Prova's bisection protocol achieves O(1) verification with O(log L) communication, and only for audited jobs.

### 6.4 vs. Arweave/AO

Arweave provides permanent storage with Proof of Access. AO adds a compute layer with optimistic verification. Prova's approach is similar in spirit to AO but is GPU-native and specifically optimized for neural network inference rather than general WASM computation.

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
- Economic parameter tuning
- Developer SDK and CLI tooling

### Phase 3: Security (Q4 2026)
- External security audit
- Bisection protocol formal verification
- Stress testing at scale
- Bug bounty program

### Phase 4: Mainnet (Q1 2027)
- Token generation event
- Genesis block
- Exchange listings
- Ecosystem grant program launch

---

## 8. Experimental Validation

The foundational claim of Prova's compute verification — that quantized inference is deterministic across GPU architectures — is currently being empirically tested.

### 8.1 Experimental Setup

- **Machine A**: NVIDIA RTX 5080 (Blackwell architecture, compute capability 12.0, 16 GB VRAM)
- **Machine B**: NVIDIA Quadro RTX 6000 (Turing architecture, compute capability 7.5, 23 GB VRAM)
- **Model**: TinyLlama 1.1B, GGUF Q8_0 quantization
- **Framework**: llama.cpp (identical version on both machines)
- **Test**: Identical prompts, identical parameters, bit-level output comparison across 1,000+ inference runs

### 8.2 Results

**Single-GPU determinism: CONFIRMED ✅**
- RTX 5080 (Blackwell, compute 12.0): 20/20 runs bit-identical (MD5: `bfaae656...`)
- RTX 6000 (Turing, compute 7.5): 20/20 runs bit-identical (MD5: `7be2ff34...`)
- Both architectures produce perfectly reproducible output on the same hardware

**Cross-architecture determinism: FAILS ❌**
- Outputs diverge at the very first generated token
- Blackwell: "indistinguishability" vs Turing: "non-interactive verification"
- Not a subtle bit-flip — entirely different generation paths

### 8.3 Analysis

The failure is expected upon reflection. GGUF Q8_0 stores weights as INT8 but computes in FP16/FP32. Different GPU architectures implement FMA (fused multiply-add) units with different intermediate precision and rounding behavior, producing different floating-point results. With temperature 0 (greedy sampling), even a tiny logit difference changes which token is selected, causing cascading divergence.

**Critical distinction**: This experiment tested llama.cpp's GGUF Q8_0, which is NOT pure integer inference. True INT8→INT32 accumulation (available in TensorRT and custom CUDA kernels) avoids floating-point entirely and may be cross-architecture deterministic. This remains to be tested.

### 8.4 Protocol Adaptation

The QBP protocol remains viable with architectural awareness:

**Approach 1: Architecture-Locked Verification Groups (Immediate)**
Nodes are grouped by GPU compute capability. Verification (bisection challenges and audits) occurs only within the same architecture group. Single-GPU determinism guarantees correctness within each group. This reduces the verifier pool per architecture but is functional immediately.

**Approach 2: True Integer Inference (Research)**
TensorRT's strict INT8 mode performs the entire compute pipeline in integers: INT8 weights × INT8 activations → INT32 accumulation → requantize to INT8. No floating-point operations are involved. If empirically validated as cross-architecture deterministic, this eliminates the need for architecture grouping.

**Approach 3: Canonical CPU Verification (Hybrid)**
Use GPU for execution speed. Define a canonical CPU computation path (x86-64, IEEE 754 strict mode, deterministic thread scheduling) for the single-layer verification step in bisection. Since the verifier only re-executes one layer, CPU speed is sufficient (milliseconds). This provides cross-architecture verification at the cost of requiring a reference CPU implementation.

The whitepaper's QBP specification is updated to require architecture-aware verification by default, with cross-architecture support as a future enhancement pending experimental validation of true integer inference.

---

## 9. Conclusion

Prova addresses a fundamental gap in decentralized infrastructure: the absence of a unified, verifiable storage and compute network. By combining proven storage proof mechanisms with a novel compute verification protocol tailored to the specific structure of neural network inference, Prova enables a new class of decentralized AI applications where both data integrity and computation correctness are cryptographically guaranteed.

The key technical contribution — Quantized Bisection Proofs — is neither a general solution to verifiable computation nor a perfect cryptographic proof. It is a practical protocol that exploits domain-specific properties (integer determinism, layered architecture) to achieve efficient verification for the most economically important class of computation in the current era.

Prova is not an incremental improvement on existing networks. It is a new foundation for the convergence of storage and intelligence.

---

## References

1. Filecoin: A Decentralized Storage Network. Protocol Labs, 2017.
2. PoRep: Proofs of Replication. Fisch, B., 2018.
3. Provable Data Possession at Untrusted Stores. Ateniese et al., 2007.
4. Arbitrum: Scalable, private smart contracts. Kalodner et al., 2018.
5. TrueBit: A scalable verification solution for blockchains. Teutsch & Reitwießner, 2017.
6. A Survey of Quantization Methods for Efficient Neural Network Inference. Gholami et al., 2021.
7. LLM.int8(): 8-bit Matrix Multiplication for Transformers at Scale. Dettmers et al., 2022.
8. GGML: A Tensor Library for Machine Learning. Gerganov, 2023.

---

*This is a living document. Version 0.1 reflects the initial protocol design prior to experimental validation. Subsequent versions will incorporate empirical results and formal specifications.*
