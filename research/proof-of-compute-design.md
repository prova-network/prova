# Proof of Compute — Design Document

## Problem Statement
Verifying that a GPU node performed a computation correctly, without re-executing the full computation, without trusting hardware manufacturers (TEE), and cheaply enough to be practical at scale.

## Core Insight: Quantized Determinism
Floating point operations are non-deterministic across GPU architectures due to:
- Parallel reduction ordering
- Different hardware FP implementations
- cuDNN algorithm selection

**However**, quantized inference (INT8/INT4) uses integer arithmetic:
- INT8 × INT8 → INT32 accumulation: fully deterministic
- Integer addition is associative (unlike FP addition)
- Same model + same input + same quantization = same output, any hardware

This is the foundational claim. Experiment in progress to verify empirically.

## Verification Protocol: Interactive Bisection

### Neural Network Structure
A neural network is a sequential pipeline of layers:
```
Input → Layer₀ → h₀ → Layer₁ → h₁ → ... → Layer_L → Output
```

Each layer is a deterministic function (given quantized weights). This structure enables efficient verification via binary search.

### Protocol Steps

**Normal flow (no challenge):**
1. Client submits inference request (input + model ID)
2. Node runs quantized inference, produces output
3. Node commits: `Commit(output, MerkleRoot(h₀, h₁, ..., h_L))`
4. Client receives output immediately
5. Challenge window opens (e.g., 10 minutes)
6. If no challenge → finalized

**Challenge flow:**
1. Challenger stakes claim that output is wrong
2. Challenger runs same inference, produces different output
3. Both parties reveal Merkle roots of intermediate activations
4. If roots differ → bisection game begins:
   - Verifier picks midpoint layer
   - Both parties reveal activation hash at midpoint
   - Verifier identifies which half has the disagreement
   - Repeat until a single layer is isolated
5. Re-execute the disputed layer with known inputs
6. Compare output to both parties' claimed activation
7. Honest party wins, liar's stake is slashed

**Complexity:**
- L layers → log₂(L) bisection steps
- 80-layer model → 7 steps
- Final verification: 1 layer forward pass (<2% of total work)

### Merkle Tree Construction
For each layer i, compute:
```
leaf_i = SHA-256(serialize(h_i))
```
where h_i is the intermediate activation tensor.

Build standard Merkle tree over all leaves. Commit the root on-chain.

**Overhead estimate (70B parameter model, 80 layers):**
- ~64 MB per activation tensor (batch=1, seq=2048, dim=8192)
- SHA-256 hashing: ~150ms per tensor
- Total: ~12 seconds hashing overhead
- For smaller models (1-7B): <1 second

### Serialization
Activation tensors must be serialized deterministically:
- Fixed byte order (little-endian)
- Fixed tensor layout (row-major, contiguous)
- Include shape metadata in hash

## Economic Security Layer

### Random Auditing
Not every inference is challenged. Instead:
- ~5% of jobs are randomly selected for audit
- A second node re-runs the selected job
- If results match → both rewarded
- If results differ → bisection game determines fault

### Stake Requirements
- Nodes must stake tokens proportional to claimed compute capacity
- Minimum stake = f(GPU VRAM, claimed TFLOPS)
- Slashing: 100% of stake forfeited on proven fraud

### Expected Value Analysis
For cheating to be rational:
```
EV(cheat) = P(not_audited) × reward - P(audited) × stake
           = 0.95 × reward - 0.05 × stake
```
For EV(cheat) < 0: stake > 19 × reward per job
With reasonable staking requirements, cheating is always -EV.

## Known Limitations

1. **Latency**: Challenge window adds finalization delay (~10 min). Acceptable for batch, not for interactive chat.
2. **Liveness**: Interactive bisection requires both parties online. Timeout defaults to challenger winning.
3. **Model size scaling**: Merkle overhead grows with model size. 400B+ models may have 30+ second overhead.
4. **Not ZK**: This is an optimistic protocol. Security is economic + statistical, not cryptographic.
5. **Quantization assumption**: If quantized inference is NOT deterministic across architectures, the protocol needs modification.

## Experimental Validation (In Progress)

### Setup
- **Machine A**: NVIDIA RTX 5080 (Blackwell architecture, compute 12.0)
- **Machine B**: NVIDIA Quadro RTX 6000 (Turing architecture, compute 7.5)
- **Model**: TinyLlama 1.1B Q8_0 GGUF
- **Framework**: llama.cpp (same version on both machines)

### Tests
1. Single-GPU self-consistency (20 runs each)
2. Cross-architecture output comparison
3. Perplexity-level logit comparison
4. Multiple quantization levels (Q8_0, Q5_K_M, Q4_0)

### Success Criteria
- Bit-identical output from same model/prompt across different GPU architectures
- If partial determinism: identify exactly which operations introduce variance

## Future Work
- ZK proof integration (when feasible for GPU workloads, ~3-5 years)
- Formal security proof for the bisection protocol
- Benchmark gas costs for on-chain adjudication
- Multi-model pipeline verification (agentic workflows)
