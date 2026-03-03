# QBP — Quantized Bisection Proofs

**Status:** Draft v0.1
**Author:** Capri (for Prova project)
**Date:** 2026-03-04

## 1. Overview

Quantized Bisection Proofs (QBP) is an interactive fraud proof protocol for verifying neural network inference. It exploits two properties:

1. **Quantized determinism**: INT8-quantized inference with controlled accumulation produces identical outputs on the same GPU architecture
2. **Layer structure**: Neural networks are sequential pipelines of layers, enabling binary search over the execution trace

QBP achieves O(log L) verification cost for L-layer models by narrowing disputes to a single layer via interactive bisection.

## 2. Definitions

### 2.1 Types

```
type Hash = [u8; 32]                    // SHA-256 digest
type Epoch = u64                         // Chain epoch number
type TokenAmount = u128                  // Native token amount (attoProva)
type Address = [u8; 20]                  // Account address
type ModelId = Hash                      // SHA-256 of model manifest
type ComputeCapability = (u8, u8)        // GPU arch version (major, minor)

struct TensorSpec {
    dtype: DType,                        // INT8 | INT32 | FP16 | FP32
    shape: Vec<u64>,                     // Dimension sizes
    layout: Layout,                      // RowMajor | ColMajor
}

enum DType {
    INT8,
    INT32,
    FP16,
    FP32,
}

enum Layout {
    RowMajor,    // C-style, contiguous
    ColMajor,    // Fortran-style
}
```

### 2.2 Model Registration

```
struct ModelManifest {
    model_id: ModelId,                   // SHA-256(serialize(self))
    name: String,                        // Human-readable name
    version: String,                     // Semver
    quantization: QuantSpec,             // Quantization parameters
    layer_count: u32,                    // Number of layers (L)
    layer_weight_hashes: Vec<Hash>,      // SHA-256 per-layer weights [L]
    total_weight_hash: Hash,             // SHA-256 of complete weight file
    input_spec: TensorSpec,              // Expected input format
    output_spec: TensorSpec,             // Expected output format
    arch_group: ComputeCapability,       // Required GPU architecture
}

struct QuantSpec {
    weight_dtype: DType,                 // Weight storage type (INT8, INT4)
    activation_dtype: DType,             // Activation type during compute
    accumulator_dtype: DType,            // Accumulation type (INT32)
    scale_dtype: DType,                  // Scale factor type
    scheme: String,                      // "GGUF_Q8_0" | "TRTINT8" | ...
}
```

### 2.3 Inference Commitment

```
struct InferenceCommitment {
    commitment_id: Hash,                 // Unique identifier
    model_id: ModelId,                   // Which model was used
    input_hash: Hash,                    // SHA-256 of serialized input
    output_hash: Hash,                   // SHA-256 of serialized output
    activation_root: Hash,               // Merkle root of intermediate activations
    node: Address,                       // Compute node that ran inference
    epoch: Epoch,                        // Chain epoch of commitment
    arch_group: ComputeCapability,       // GPU architecture used
}
```

### 2.4 Activation Merkle Tree

```
struct ActivationTree {
    leaves: Vec<Hash>,                   // H(serialize(h_i)) for i = 0..L
    root: Hash,                          // Merkle root
    layer_count: u32,                    // L (= leaves.len() - 1, including input h_0)
}
```

**Construction:**
```
For each layer i in 0..=L:
    leaf_i = SHA-256(serialize_tensor(h_i))
    where h_0 = input, h_i = f_i(h_{i-1}) for i > 0

Root = MerkleRoot(leaf_0, leaf_1, ..., leaf_L)
```

**Tensor serialization** (canonical, deterministic):
```
fn serialize_tensor(tensor: &Tensor) -> Vec<u8> {
    let mut buf = Vec::new();
    // Header: dtype (1 byte) + ndim (1 byte) + shape (8 bytes each)
    buf.push(tensor.dtype as u8);
    buf.push(tensor.ndim as u8);
    for dim in &tensor.shape {
        buf.extend_from_slice(&dim.to_le_bytes());
    }
    // Data: contiguous row-major, little-endian
    for element in tensor.data_row_major() {
        buf.extend_from_slice(&element.to_le_bytes());
    }
    buf
}
```

## 3. Protocol Messages

### 3.1 Message Types

```
enum QBPMessage {
    // --- Commitment Phase ---
    Commit {
        commitment: InferenceCommitment,
        signature: Signature,
    },

    // --- Challenge Phase ---
    Challenge {
        commitment_id: Hash,
        challenger: Address,
        challenger_output_hash: Hash,
        challenger_activation_root: Hash,
        stake: TokenAmount,
        signature: Signature,
    },

    // --- Bisection Phase ---
    BisectionReveal {
        commitment_id: Hash,
        round: u32,
        layer_index: u32,
        activation_hash: Hash,
        merkle_proof: Vec<Hash>,         // Proof against committed root
        sender: Address,
        signature: Signature,
    },

    BisectionPick {
        commitment_id: Hash,
        round: u32,
        direction: BisectionDirection,    // Upper | Lower
    },

    // --- Verification Phase ---
    VerificationRequest {
        commitment_id: Hash,
        disputed_layer: u32,             // The layer to re-execute
        input_activation: Vec<u8>,       // Serialized h_{layer-1}
        input_merkle_proof: Vec<Hash>,
        prover_output_hash: Hash,
        challenger_output_hash: Hash,
    },

    VerificationResult {
        commitment_id: Hash,
        disputed_layer: u32,
        computed_output_hash: Hash,      // What the verifier computed
        honest_party: Address,
        dishonest_party: Address,
    },

    // --- Resolution ---
    Slash {
        commitment_id: Hash,
        slashed: Address,
        amount: TokenAmount,
        reason: SlashReason,
    },

    Timeout {
        commitment_id: Hash,
        defaulted: Address,              // Party that failed to respond
        reason: TimeoutReason,
    },
}

enum BisectionDirection {
    Upper,  // Disagreement is in upper half (hi..=end)
    Lower,  // Disagreement is in lower half (start..=lo)
}

enum SlashReason {
    BisectionFraud,          // Proved dishonest via bisection
    TimeoutDefault,          // Failed to respond within deadline
    InvalidMerkleProof,      // Submitted invalid Merkle proof
}

enum TimeoutReason {
    BisectionTimeout,        // Didn't submit bisection reveal in time
    ChallengeExpired,        // Challenge window closed, no challenge
}
```

## 4. State Machine

### 4.1 Dispute States

```
enum DisputeState {
    // Commitment submitted, challenge window open
    Open {
        commitment: InferenceCommitment,
        deadline: Epoch,                 // Challenge window closes
    },

    // Challenge received, bisection in progress
    Bisecting {
        commitment: InferenceCommitment,
        challenger: Address,
        challenger_root: Hash,
        round: u32,                      // Current bisection round
        lo: u32,                         // Last agreed layer
        hi: u32,                         // First disagreed layer (or upper bound)
        turn: Address,                   // Whose turn to reveal
        deadline: Epoch,                 // Current round deadline
    },

    // Bisection complete, single layer isolated for verification
    Verifying {
        commitment: InferenceCommitment,
        challenger: Address,
        disputed_layer: u32,
        prover_hash: Hash,              // Prover's claimed h[disputed_layer]
        challenger_hash: Hash,          // Challenger's claimed h[disputed_layer]
        input_hash: Hash,              // Agreed h[disputed_layer - 1]
        deadline: Epoch,
    },

    // Resolved
    Resolved {
        commitment: InferenceCommitment,
        outcome: DisputeOutcome,
    },
}

enum DisputeOutcome {
    NoChallenge,                         // Challenge window expired, commitment valid
    ProverHonest,                        // Bisection proved prover correct
    ChallengerHonest,                    // Bisection proved challenger correct
    ProverDefaulted,                     // Prover timed out
    ChallengerDefaulted,                 // Challenger timed out
    BothDishonest,                       // Neither output matches verification
}
```

### 4.2 State Transitions

```
Open --[Challenge received]--> Bisecting { round: 0, lo: 0, hi: L }
Open --[Deadline passed, no challenge]--> Resolved { NoChallenge }

Bisecting --[Both reveal at midpoint, agree]--> Bisecting { lo: mid, round++ }
Bisecting --[Both reveal at midpoint, disagree]--> Bisecting { hi: mid, round++ }
Bisecting --[hi - lo == 1]--> Verifying { disputed_layer: hi }
Bisecting --[Party timeout]--> Resolved { [Defaulter]Defaulted }

Verifying --[Verifier computed, matches prover]--> Resolved { ProverHonest }
Verifying --[Verifier computed, matches challenger]--> Resolved { ChallengerHonest }
Verifying --[Verifier computed, matches neither]--> Resolved { BothDishonest }
Verifying --[Deadline passed]--> Resolved { [Defaulter]Defaulted }
```

### 4.3 Timing Parameters

```
const CHALLENGE_WINDOW: u32 = 120;       // Epochs (~60 min at 30s/epoch)
const BISECTION_ROUND_TIMEOUT: u32 = 10; // Epochs (~5 min)
const VERIFICATION_TIMEOUT: u32 = 20;    // Epochs (~10 min)
const MIN_STAKE: TokenAmount = 1000;     // Minimum challenge stake
```

## 5. Bisection Algorithm (Detailed)

```
fn bisection_game(
    prover_root: Hash,
    challenger_root: Hash,
    layer_count: u32,  // L
) -> DisputedLayer {
    let mut lo = 0u32;
    let mut hi = layer_count;  // L

    while hi - lo > 1 {
        let mid = (lo + hi) / 2;

        // Both parties reveal H(h_mid) with Merkle proofs
        let prover_h_mid = prover.reveal(mid, prover_root);
        let challenger_h_mid = challenger.reveal(mid, challenger_root);

        // Verify Merkle proofs on-chain
        assert!(verify_merkle_proof(prover_h_mid, mid, prover_root));
        assert!(verify_merkle_proof(challenger_h_mid, mid, challenger_root));

        if prover_h_mid == challenger_h_mid {
            // Agree up to mid → disagreement is in upper half
            lo = mid;
        } else {
            // Disagree at mid → disagreement is in lower half
            hi = mid;
        }
    }

    // lo = last agreed layer, hi = first disagreed layer
    // hi == lo + 1, so we verify layer hi
    DisputedLayer {
        layer_index: hi,
        agreed_input: lo,     // Both agree on h[lo]
    }
}
```

**Example for 80-layer model:**
```
Round 0: lo=0, hi=80, mid=40  → compare h[40]
Round 1: lo=40, hi=80, mid=60 → compare h[60]  (agreed at 40)
Round 2: lo=60, hi=80, mid=70 → compare h[70]  (agreed at 60)
Round 3: lo=60, hi=70, mid=65 → compare h[65]  (disagreed at 70)
Round 4: lo=60, hi=65, mid=62 → compare h[62]  (disagreed at 65)
Round 5: lo=62, hi=65, mid=63 → compare h[63]  (agreed at 62)
Round 6: lo=63, hi=65, mid=64 → compare h[64]  (agreed at 63)
→ hi - lo == 1: disputed layer = 64
→ Verify: re-execute f_64(h[63]) and compare to h[64]
```

7 rounds for 80 layers. The verifier executes 1 layer out of 80 (1.25% of total work).

## 6. Architecture Groups

Based on experimental findings (Section 8 of whitepaper), cross-architecture determinism fails. QBP requires **architecture-locked verification**:

```
struct ArchGroup {
    compute_capability: ComputeCapability,  // e.g., (7, 5) for Turing
    name: String,                           // "Turing" | "Ampere" | "Blackwell"
}
```

**Rules:**
1. Model registrations specify an `arch_group`
2. Inference commitments include the `arch_group` used
3. Challenges may only come from nodes in the same `arch_group`
4. Random audits select verifier nodes from the same `arch_group`

**Implications:**
- Each architecture group is independently secure
- Minimum viable group: 2 nodes (prover + potential verifier)
- Cross-group verification is a future enhancement (pending true INT8 or CPU canonical path)

## 7. Security Considerations

### 7.1 Honest Majority Assumption (per arch group)
Within each architecture group, at least one honest node must exist to challenge fraudulent commitments. With random auditing at rate r, the probability of catching a cheater over N jobs is 1 - (1-r)^N.

### 7.2 Merkle Proof Binding
Once a party commits to an activation Merkle root, they cannot change their claimed execution trace. Any deviation in a bisection reveal will fail Merkle proof verification → immediate slashing.

### 7.3 Verifier Trust
The single-layer verification step requires a trusted verifier (or multiple verifiers with majority agreement). In production, this is performed by the chain's validator set.

### 7.4 Timing Attacks
Deadlines prevent griefing via indefinite delays. A party that cannot respond in time forfeits their stake. Deadlines must be generous enough to account for network latency and chain congestion.

## 8. Gas Cost Estimates

Per bisection round:
- 2 Merkle proof verifications: ~50K gas each
- 1 state update: ~20K gas
- **Per round: ~120K gas**

Total for 80-layer model (7 rounds):
- Bisection: 7 × 120K = ~840K gas
- Final verification: depends on layer complexity (~200K-2M gas)
- **Total dispute: ~1-3M gas**

Non-disputed commitment (happy path):
- Commit: ~60K gas
- Finalize (no challenge): ~30K gas
- **Total happy path: ~90K gas**

## 9. Open Questions

1. **Layer granularity**: Should bisection operate at the layer level, or at finer granularity (e.g., individual operations within a layer)?
2. **Multi-head attention**: Transformer attention layers contain multiple parallel heads — can these be bisected independently?
3. **KV cache**: For autoregressive generation, the KV cache grows per token. How does this affect the activation tree?
4. **Batch inference**: Can multiple inference requests be batched and committed together?
5. **Model updates**: How to handle model version upgrades while maintaining in-flight commitments?
