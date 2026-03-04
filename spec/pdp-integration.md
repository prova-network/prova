# SPEC-004: PDP Integration Specification

**Status:** Draft  
**Author:** Capri  
**Created:** 2026-03-04

## 1. Overview

Prova integrates Provable Data Possession (PDP) as its storage verification layer. Unlike Filecoin's sealed PoRep, Prova uses lightweight PDP proofs over raw unsealed data, enabling hot/warm storage with lower overhead.

PDP in Prova serves dual purposes:
1. **Storage verification** — prove that model weights and datasets are stored
2. **Model integrity** — prove specific model files match their registered weight hashes

## 2. CommP Compatibility

Prova uses the same CommP (Piece Commitment) format as Filecoin's PDP:
- SHA-256 truncated to 254 bits (Fr-safe)
- Binary Merkle tree over 32-byte leaves
- Piece sizes: powers of 2 from 128 bytes to 64 GiB

### 2.1 Model Storage Format

A registered model is stored as one or more PDP pieces:

```
Model: TinyLlama-1.1B-Q8_0
├── Piece 0: weights_layer_0-7.bin   (CommP: 0x...)
├── Piece 1: weights_layer_8-15.bin  (CommP: 0x...)
├── Piece 2: weights_layer_16-23.bin (CommP: 0x...)
└── Piece 3: weights_layer_24-31.bin (CommP: 0x...)
```

The Model Registry stores both:
- Per-layer weight hashes (for QBP verification)
- Per-piece CommP roots (for PDP storage proofs)

## 3. Proof Set Management

### 3.1 Registration

When a storage provider onboards a model:

```
Provider → Chain: RegisterProofSet(model_id, [CommP_0, ..., CommP_n])
Chain: Verifies CommPs match model registry
Chain: Creates proof set, starts proving schedule
```

### 3.2 Challenge Protocol

Challenges use drand randomness (same as Filecoin PDP):

```
epoch E:
  seed = drand_beacon(E)
  challenges = select_random_roots(proof_set, seed, count=5)
  
Provider → Chain: SubmitPDPProof(proof_set_id, [merkle_proofs])
Chain: Verify all 5 inclusion proofs
```

### 3.3 Proving Schedule

- **Challenge frequency:** Every 2880 epochs (~24 hours at 30s epochs)
- **Response window:** 60 epochs (~30 minutes)
- **Fault tolerance:** 2 consecutive misses before slashing
- **Grace period:** 10 epochs after challenge for proof submission

## 4. Integration with QBP

PDP and QBP operate on different data but share the model registry:

| Aspect | PDP | QBP |
|--------|-----|-----|
| **Proves** | Data is stored | Inference is correct |
| **Data** | Model weight files | Per-layer activations |
| **Hash** | CommP (SHA-256/254) | SHA-256 (full) |
| **Frequency** | Daily | Per-inference |
| **Cost** | ~140M gas/proof | ~1 on-chain tx + O(log L) bisection rounds |

### 4.1 Cross-Verification

When a QBP dispute reaches the single-layer verification stage:
1. Verifier needs the model weights for that layer
2. PDP proof confirms the weight data is available and correct
3. Verifier re-executes the layer with known weights + input activation
4. Comparison with claimed output activation determines winner

```
Dispute Resolution:
  1. Bisection isolates layer K
  2. Fetch weight_hash[K] from Model Registry
  3. PDP proof confirms storage of piece containing layer K weights
  4. Verifier loads weights, re-executes layer K
  5. Compare output → determine winner
```

## 5. Staking Integration

Storage providers must stake for both PDP and QBP:

```
Total Stake = PDP_Stake + QBP_Stake

PDP_Stake = f(data_stored_bytes)     # Proportional to storage committed
QBP_Stake = f(inference_throughput)   # Proportional to compute committed
```

### 5.1 Slashing

| Violation | Slash Amount | Cool-down |
|-----------|-------------|-----------|
| PDP proof miss (1st) | 0% (warning) | — |
| PDP proof miss (2nd consecutive) | 5% of PDP stake | 7 days |
| PDP proof miss (3rd) | 100% of PDP stake | Permanent |
| QBP dispute lost | 10% of QBP stake | 24 hours |
| QBP dispute + data unavailable | 50% of total stake | 30 days |

## 6. Gas Costs (Estimated)

Based on Filecoin PDP empirical data:
- Proof set creation (100 roots): ~140M gas
- Proof set creation (1K roots): ~150M gas
- Proof set creation (10K roots): ~160M gas
- **Scaling: logarithmic** — critical for large model storage

## 7. Future: Dataset PDP

Beyond model weights, PDP can prove storage of:
- Training datasets (for reproducibility)
- Fine-tuning data
- Inference logs (for audit trails)
- User data (for data availability in decentralized apps)

## 8. References

- [Filecoin PDP Spec](https://github.com/filecoin-project/FIPs)
- [SPEC-001: QBP Protocol](./qbp-protocol.md)
- [SPEC-003: Model Registry](./model-registry.md)
- Prova Whitepaper §4 (Storage Layer)
