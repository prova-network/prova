# Model Registry — Specification

**Status:** Draft v0.1
**Date:** 2026-03-04

## 1. Purpose

The Model Registry is an on-chain directory of AI models available for verified inference on Prova. It stores model metadata, per-layer weight hashes, and quantization specifications — everything needed to verify that a node is running the correct model.

## 2. Schema

### 2.1 Model Manifest

```
struct ModelManifest {
    // Identity
    model_id:           Hash,           // SHA-256 of canonical manifest (excluding this field)
    name:               String,         // e.g., "llama-3-8b"
    version:            String,         // Semver, e.g., "1.0.0"
    description:        String,         // Human-readable description

    // Architecture
    architecture:       String,         // "transformer" | "mamba" | "rwkv" | ...
    layer_count:        u32,            // L
    parameter_count:    u64,            // Total parameters
    context_length:     u32,            // Max sequence length

    // Quantization
    quant_spec:         QuantSpec,

    // Weight Integrity
    weight_format:      String,         // "GGUF" | "SafeTensors" | "ONNX" | "TensorRT"
    total_weight_hash:  Hash,           // SHA-256 of complete weight file
    total_weight_size:  u64,            // Bytes
    layer_weight_hashes: Vec<Hash>,     // SHA-256 per layer [L entries]

    // I/O Specification
    input_spec:         TensorSpec,
    output_spec:        TensorSpec,

    // Architecture Group
    arch_groups:        Vec<ComputeCapability>, // Which GPU archs this is validated for

    // Registration
    registrant:         Address,
    registration_epoch: Epoch,
    stake:              TokenAmount,    // Registrant's integrity stake
}

struct QuantSpec {
    scheme:             String,         // "Q8_0" | "Q4_0" | "INT8_TRT" | ...
    weight_bits:        u8,             // 4, 8, 16, 32
    activation_bits:    u8,
    accumulator_bits:   u8,             // Typically 32
    group_size:         u32,            // Quantization group size (e.g., 32 for Q8_0)
    symmetric:          bool,           // Symmetric vs asymmetric quantization
}
```

### 2.2 Registration Flow

```
1. Registrant prepares model weights in specified format
2. Registrant computes:
   - total_weight_hash = SHA-256(weight_file)
   - layer_weight_hashes[i] = SHA-256(layer_i_weights) for each layer
3. Registrant submits ModelManifest + stake to chain
4. Chain verifies:
   - Stake >= MIN_MODEL_STAKE
   - No duplicate model_id
   - layer_weight_hashes.len() == layer_count
5. Model enters PENDING state for verification period (7 days)
6. Any node can challenge by:
   - Downloading weights
   - Recomputing hashes
   - Submitting proof of mismatch
7. After verification period with no successful challenges: ACTIVE

State transitions:
  PENDING --[verification period, no challenge]--> ACTIVE
  PENDING --[valid challenge]--> REJECTED (registrant slashed)
  ACTIVE  --[registrant deregisters]--> DEPRECATED
  ACTIVE  --[governance vote]--> DEPRECATED
```

### 2.3 Model Lookup

```
// By ID
fn get_model(model_id: Hash) -> Option<ModelManifest>

// By name + version
fn find_model(name: &str, version: &str) -> Option<ModelManifest>

// By architecture group
fn models_for_arch(arch: ComputeCapability) -> Vec<ModelManifest>

// Active models only
fn active_models() -> Vec<ModelManifest>
```

## 3. Per-Layer Weight Hashes

### 3.1 Why Per-Layer

The bisection protocol's verification step re-executes a single layer. The verifier needs to:
1. Confirm the correct weights were used for that specific layer
2. Without downloading the entire model

Per-layer hashes enable:
- Layer-level weight verification during disputes
- Incremental model download (fetch only the disputed layer's weights)
- Storage optimization (nodes can cache popular layers)

### 3.2 Layer Boundary Definition

For transformer models, one "layer" = one transformer block:
```
TransformerBlock {
    self_attention: {q_proj, k_proj, v_proj, o_proj}
    feed_forward: {gate_proj, up_proj, down_proj}  // or {fc1, fc2} for dense
    layer_norm: {weight, bias} × 2
}
```

The hash covers ALL weights in the block, serialized in a canonical order:
```
layer_hash = SHA-256(
    serialize(q_proj) ||
    serialize(k_proj) ||
    serialize(v_proj) ||
    serialize(o_proj) ||
    serialize(gate_proj) ||
    serialize(up_proj) ||
    serialize(down_proj) ||
    serialize(ln1_weight) ||
    serialize(ln1_bias) ||
    serialize(ln2_weight) ||
    serialize(ln2_bias)
)
```

Additional layers (not transformer blocks):
- Layer 0: Embedding layer (token_embedding + position_embedding)
- Layer L: Output head (lm_head + final_layer_norm)

## 4. Governance

### 4.1 Model Deprecation
Models can be deprecated by:
- Registrant voluntary deregistration (stake returned after cooldown)
- Governance vote (e.g., security vulnerability found)
- Automatic deprecation after N epochs of zero usage

### 4.2 Model Updates
New versions are registered as separate models. The old version remains active until deprecated. No in-place updates.

### 4.3 Disputed Models
If a model's weights are found to produce non-deterministic results within an architecture group, any node can submit evidence → governance review → potential deprecation + registrant slashing.

## 5. Storage Requirements

On-chain per model:
- Manifest: ~500 bytes (fixed fields)
- Layer hashes: L × 32 bytes
- For 80-layer model: 500 + 2,560 = ~3 KB

Off-chain (CDN/IPFS):
- Complete weight files (multi-GB)
- Referenced by total_weight_hash for integrity
