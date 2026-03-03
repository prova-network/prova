# Project Codename: TBD

## Vision
A new Layer 1 blockchain combining verifiable storage and AI compute, built on the best ideas from Filecoin's proof-of-storage protocol with clean economics from day one.

## Core Innovation: Proof of Compute via Quantized Determinism
Quantized AI inference (INT8/INT4) uses integer arithmetic, which is deterministic across GPU architectures. Combined with interactive bisection fraud proofs (inspired by Arbitrum), this enables efficient verification of GPU computation without re-executing the full workload.

### How it works
1. **Deterministic inference**: Mandated quantized models produce identical outputs on any hardware
2. **Activation Merkle trees**: Each layer's intermediate outputs are hashed into a Merkle tree
3. **Interactive bisection**: If challenged, binary search finds the first disagreeing layer in O(log L) steps
4. **Single-layer verification**: Re-execute one layer to determine who's honest (<2% of original work)
5. **Economic security**: Staking + random 5% audit rate makes cheating -EV

### Protocol Stack
```
ON-CHAIN (slow, secure)
├─ Model registry (weight hashes per layer)
├─ Quantization spec (INT8/INT32 accumulate)
├─ Stake ledger (node deposits)
├─ Challenge/response adjudication
└─ Slashing execution

OFF-CHAIN (fast, practical)
├─ Inference execution (quantized, deterministic)
├─ Activation Merkle tree construction
├─ Result delivery to client
└─ Bisection game (if challenged)
```

## Storage Layer
- PDP-first (Provable Data Possession) — onboard storage in minutes, not hours
- No sealing pipeline required for hot/warm data
- PoRep available as optional cold archival tier
- Variable sector sizes (not fixed 32/64 GiB)

## Economics
- 1 byte = 1 byte. No multipliers. No DataCap. No privileged data classes.
- Dual reward streams: storage + compute
- Dynamic rebalancing between storage and compute rewards
- Simple pledge: proportional to reward and sector lifetime
- Streaming payments at L1

## What we take from Filecoin (Apache 2.0 / MIT)
- Proof of Replication (PoRep) — optional cold tier
- Proof of Spacetime (PoSt) — ongoing storage verification
- PDP — lightweight hot data proofs
- FVM / actor model — smart contracts
- Economic insights from 4+ years of mainnet operation

## What we fix
- No Fil+ / DataCap (eliminated, not deprecated)
- No mining reserve
- PDP-first instead of PoRep-first
- Compute verification built into consensus
- On-chain governance from day one
- Clean token distribution

## Research Status
- [x] Protocol design (conceptual)
- [x] Economic analysis framework (CUDA simulation infrastructure)
- [ ] **IN PROGRESS**: Quantized determinism experiment (RTX 5080 vs RTX 6000)
- [ ] Formal bisection protocol specification
- [ ] Activation Merkle tree benchmarking
- [ ] Economic modeling (optimal challenge rate, stake requirements)
- [ ] Whitepaper draft

## Experimental Infrastructure
- **Blackwell**: RTX 5080 (compute 12.0, Blackwell arch) — 192.168.50.203
- **hexa-2**: 4× Quadro RTX 6000 (compute 7.5, Turing arch) — datacenter
- **hexa-4**: 4× Quadro RTX 6000 (compute 7.5, Turing arch) — datacenter
- Testing quantized inference determinism across architectures using llama.cpp + TinyLlama 1.1B Q8_0

## Prior Art
| Project | Storage | Compute | Proofs | Gap |
|---------|---------|---------|--------|-----|
| Filecoin | ✅ PoRep/PoSt/PDP | ❌ | Cryptographic | No compute layer |
| Akash | ❌ | ✅ Containers | Economic (staking) | No storage proofs |
| Bittensor | ❌ | ✅ AI inference | Validation subnet | No real storage |
| Render | ❌ | ✅ GPU rendering | Reputation | Specialized, no storage |
| io.net | ❌ | ✅ GPU cluster | Economic | No storage proofs |
| Arweave/AO | ✅ Permanent | ✅ (AO) | Proof of Access | Different proof model |
| **This** | ✅ PDP+PoRep | ✅ AI inference | Quantized bisection | — |

## License
Private / Proprietary — not yet open source.
