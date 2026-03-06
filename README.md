# Prova

**A Layer 1 blockchain combining verifiable storage (PDP) and AI compute (QBP) with clean economics from day one.**

*Prova* — Latin: "to prove."

## Status

| Metric | Count |
|--------|-------|
| Rust source files | 108 |
| Lines of Rust | 63,000+ |
| Passing tests | 1,690 |
| Crates | 4 (`prova-chain`, `prova-node`, `prova-sdk`, ops) |
| Specifications | 23 |
| Documentation | 10 guides |
| Commits | 209 |
| External deps | Minimal (sha2, serde, serde_json) |

All tests pass. Clippy clean with `-D warnings`.

## Architecture

```
┌─────────────────────────────────────────────────────┐
│                    Client SDK                        │
│  (rpc_client, event_client, blob_client, cli_wallet) │
├─────────────────────────────────────────────────────┤
│                    Node Layer                        │
│  Network ─ RPC ─ API Gateway ─ Executor ─ P2P       │
│  CLI ─ Wallet ─ Metrics ─ Sync ─ Explorer           │
├─────────────────────────────────────────────────────┤
│                    Chain Layer                        │
│  Blocks ─ Genesis ─ State ─ Mempool ─ Gas            │
│  Commits ─ Disputes ─ Stake ─ Payments ─ Rewards     │
│  Governance ─ Marketplace ─ Delegation ─ DAS         │
│  Checkpoints ─ Bridge ─ Finality ─ Events            │
├─────────────────────────────────────────────────────┤
│                   Proof Layer                        │
│  QBP (Quantized Bisection Proofs) ─ PDP ─ TEE       │
│  Activation Merkle Trees ─ Audits ─ ZK Verifier     │
└─────────────────────────────────────────────────────┘
```

## Core Innovations

### Quantized Bisection Proofs (QBP)

Verifiable AI inference without re-executing the full workload:

1. **Deterministic inference**: Quantized models (INT8) use integer arithmetic — bit-identical across same-architecture GPUs
2. **Activation Merkle trees**: Each layer's intermediate outputs are hashed into a Merkle tree
3. **Interactive bisection**: If challenged, binary search finds the first disagreeing layer in O(log L) steps
4. **Single-layer verification**: Re-execute one layer to determine who's honest — <2% of original compute
5. **Economic security**: Staking + random audit rate makes cheating negative expected value

### Three-Tier Storage Proofs

Providers choose their proof mechanism based on hardware and economics:

**PDP (Baseline)** — Lightweight Merkle proofs for hot/warm data:
- On-chain proof sets with CommP roots + drand randomness → 5 random challenges → Merkle inclusion proofs
- No special hardware required — data onboarded in minutes
- Logarithmic gas scaling (140M gas for 100 roots, 160M for 10K)

**TEE-Attested (Fast Path)** — Hardware-managed encryption with spot-check verification:
- TEE enclave manages all disk encryption — provider never sees plaintext keys
- Per-machine encryption → unique replicas without SNARK sealing
- Onboarding in **seconds** (AES at hardware speed)
- Open source enclave image, chain-versioned, governance-approved updates
- No network access for enclave — disk I/O + bytestream only (minimal TCB)
- Graceful fallback to PDP if TEE vulnerability discovered

**PoRep (Cold Tier)** — Optional SNARK-sealed archival storage:
- Cryptographic unique replica guarantees
- Hours to onboard, but strongest anti-outsourcing properties

All tiers earn equal rewards per byte. TEE is an optimization, never a requirement — the network's security is grounded in math (PDP), with hardware as an accelerator.

### Clean Economics

- **1 byte = 1 byte.** No multipliers, no DataCap, no privileged data classes.
- **Dual reward streams**: Storage (PDP-verified bytes) + Compute (verified inference commits)
- **Streaming payments at L1**: Continuous payment channels with 0.5% network fee
- **Halving emission**: 400M PROVA mined over ~20 years, 50% burned fees
- **No mining reserve**: All tokens accounted for from genesis

## Crate Structure

```
prova/
├── chain/          prova-chain — on-chain state, consensus, economics
│   └── src/
│       ├── block.rs          Block production, weighted round-robin
│       ├── genesis.rs        Genesis state, devnet/testnet configs
│       ├── commit.rs         Inference commit store + challenge window
│       ├── dispute.rs        Bisection game + dispute arena
│       ├── stake.rs          Stake ledger + slashing
│       ├── payment.rs        Streaming payment channels
│       ├── rewards.rs        Block + inference + storage rewards
│       ├── gas.rs            EIP-1559 style gas metering
│       ├── mempool.rs        Transaction priority queue
│       ├── state.rs          Account state trie
│       ├── governance.rs     On-chain proposal system
│       ├── marketplace.rs    Model marketplace + bidding
│       ├── delegation.rs     Delegated staking + liquid staking
│       ├── das.rs            Data availability sampling
│       ├── checkpoint.rs     Filecoin L1 checkpoint anchoring
│       ├── bridge.rs         Cross-chain state proofs
│       ├── finality.rs       Fast + slow finality gadget
│       ├── events.rs         Structured event log
│       ├── snapshot.rs       State snapshots for fast sync
│       ├── migration.rs      State migration system
│       ├── network_sim.rs    Multi-node network simulator
│       ├── chaos.rs          Chaos testing scenarios
│       └── ...               (54 modules total)
│
├── node/           prova-node — node implementation
│   └── src/
│       ├── merkle.rs         Activation Merkle tree builder
│       ├── participant.rs    QBP bisection participant
│       ├── runner.rs         Inference runner (mock + faulty)
│       ├── llamacpp.rs       Real llama.cpp integration
│       ├── determinism.rs    Cross-arch determinism testing
│       ├── canonical_cpu.rs  CPU canonical verification path
│       ├── network.rs        P2P gossip networking
│       ├── pdp.rs            PDP proof engine
│       ├── cli.rs            CLI interface (run, status, tx)
│       ├── wallet.rs         Ed25519 wallet + keystore
│       ├── rpc.rs            JSON-RPC 2.0 API
│       ├── api_gateway.rs    HTTP API with auth + rate limiting
│       ├── devnet.rs         In-memory devnet simulation
│       └── ...               (42 modules total)
│
├── sdk/            prova-sdk — client SDK
│   └── src/
│       ├── lib.rs            Request builder, signing, discovery
│       ├── rpc_client.rs     JSON-RPC client
│       ├── cli_wallet.rs     CLI wallet integration
│       ├── event_client.rs   Event subscription
│       ├── marketplace.rs    Marketplace client
│       ├── delegation.rs     Staking client
│       ├── blob_client.rs    Blob upload (erasure coded)
│       └── ...               (12 modules total)
│
├── spec/           Protocol specifications (22 specs)
├── proto/          Protobuf wire format definitions
├── docs/           Developer documentation (10 guides)
├── ops/            Operational configs (testnet genesis, bootnodes)
├── research/       Experiment results + design exploration
└── whitepaper.md   Whitepaper v0.1
```

## Quick Start

```bash
# Clone (private repo — requires access)
git clone git@github.com:Reiers/prova.git
cd prova

# Build all crates
cargo build --workspace

# Run all 1,690 tests
cargo test --workspace

# Lint (must pass with zero warnings)
cargo clippy --workspace -- -D warnings

# Run the node demo
cargo run -p prova-node
```

## Specifications

| Spec | Title |
|------|-------|
| SPEC-001 | [QBP Protocol](spec/qbp-protocol.md) — Quantized Bisection Proofs |
| SPEC-002 | [Activation Merkle Tree](spec/activation-merkle-tree.md) — Hash format |
| SPEC-003 | [Model Registry](spec/model-registry.md) — On-chain model registration |
| SPEC-004 | [PDP Integration](spec/pdp-integration.md) — Storage proofs |
| SPEC-005 | [Audit Protocol](spec/audit-protocol.md) — Random audit system |
| SPEC-006 | [Streaming Payments](spec/streaming-payments.md) — L1 payment channels |
| SPEC-007 | [Network Protocol](spec/network-protocol.md) — P2P gossip |
| SPEC-008 | [Token Economics](spec/token-economics.md) — Supply, emission, fees |
| SPEC-010 | [Token Economics (expanded)](spec/token-economics.md) |
| SPEC-011 | [Governance](spec/governance.md) — On-chain proposals |
| SPEC-012 | [Light Client](spec/light-client.md) — Minimal verification |
| SPEC-013 | [Security Threat Model](spec/security-threat-model.md) — 21 threats |
| SPEC-014 | [Checkpoint Anchoring](spec/checkpoint-anchoring.md) — Filecoin L1 |
| SPEC-015 | [Bridge Security](spec/bridge-security.md) — Cross-chain proofs |
| SPEC-016 | [Event Schema](spec/event-schema.md) — Structured events |
| SPEC-017 | [Security Audit Checklist](spec/security-audit-checklist.md) — 73 checks |
| SPEC-018 | [Marketplace](spec/marketplace.md) — Model marketplace |
| SPEC-019 | [Data Availability](spec/data-availability.md) — DAS protocol |
| SPEC-020 | [Delegation & Staking](spec/delegation-staking.md) |
| SPEC-021 | [Validator Set](spec/validator-set.md) — Validator management |
| SPEC-022 | [Confidential Inference](spec/confidential-inference.md) — Privacy |
| SPEC-023 | [Account Abstraction](spec/account-abstraction.md) — Multi-sig |
| SPEC-024 | [API Gateway](spec/api-gateway.md) — External integration |
| SPEC-025 | State Migration — *planned* |
| SPEC-026 | [TEE-Attested Storage Proofs](spec/tee-storage-proofs.md) — Hardware fast path |

## Experiments

| ID | Title | Result |
|----|-------|--------|
| EXP-001 | Single-GPU determinism (TinyLlama Q8_0) | ✅ **PASS** — 5/5 runs bit-identical on both RTX 5080 and RTX 6000 |
| EXP-002 | Cross-architecture determinism | ❌ **FAIL** — Diverges at token 1 between Blackwell and Turing |
| EXP-003 | CPU canonical verification path | ✅ **PASS** — Fixed-point INT8 GEMM with no floating point |

**Key finding**: Same-architecture determinism is perfect. Cross-architecture fails due to different floating-point rounding in dequantization. This is why Prova uses **Architecture Groups** — nodes with identical GPU hardware are grouped together for verification.

## What's Built vs What's Left

### ✅ Complete (Phase 1–19)
- On-chain state: blocks, genesis, state trie, gas, mempool, rewards, governance
- Proof layer: QBP bisection, PDP, audits, activation Merkle trees
- Economics: staking, slashing, payments, delegation, liquid staking, marketplace
- Networking: P2P gossip, sync, checkpoints, bridge, finality
- Node: CLI, wallet, RPC, API gateway, metrics, fast sync
- SDK: RPC client, event subscriptions, blob upload, marketplace, multi-sig
- Security: access control, rate limiting, TLS, invariant checker, fuzz harness
- Testing: 1,690 tests, chaos scenarios, network simulation, load testing, benchmarks
- Specs: 22 protocol specifications, security audit checklist (73 checks)
- Docs: quickstart, architecture, FAQ, SDK guide, testnet operator guide

### 🔜 Next Milestones
- [ ] Real networking (replace mock transport with TCP/QUIC)
- [ ] Persistent state (replace in-memory with on-disk)
- [ ] Real GPU inference integration (llama.cpp production pipeline)
- [ ] Testnet launch (genesis ceremony, boot nodes, monitoring)
- [ ] TEE enclave prototype (SGX sector encryption + spot-check verification)
- [ ] TensorRT strict INT8 experiment (potential universal cross-arch solution)
- [ ] Formal security audit
- [ ] Whitepaper v1.0

## Prior Art

| Project | Storage | Compute | Proofs | Gap |
|---------|---------|---------|--------|-----|
| Filecoin | ✅ PoRep/PoSt/PDP | ❌ | Cryptographic | No compute layer |
| Akash | ❌ | ✅ Containers | Economic | No storage proofs |
| Bittensor | ❌ | ✅ AI inference | Validation subnet | No real storage |
| Render | ❌ | ✅ GPU rendering | Reputation | Specialized, no storage |
| io.net | ❌ | ✅ GPU cluster | Economic | No storage proofs |
| Arweave/AO | ✅ Permanent | ✅ (AO) | Proof of Access | Different proof model |
| TEE-only | TEE attestation | TEE attestation | Hardware only | Single point of failure |
| **Prova** | ✅ PDP+TEE+PoRep | ✅ AI inference | QBP + PDP + TEE | Defense in depth |

## Language Strategy

**Core protocol (Rust)** — Chain, node, and proof layer stay in Rust. No GC pauses for block production, memory safety for consensus code, zero-cost abstractions. Every major L1 (Solana, Polkadot, Near, Sui, Reth) made the same choice.

**GPU proof layer (Rust + C++ FFI)** — Real inference runs through llama.cpp (C++) or CUDA kernels. Rust calls in via FFI. The `llamacpp.rs` module is already designed for this.

**Client SDKs (polyglot)** — The Rust `sdk/` crate is the reference implementation. Production adoption needs:
- **TypeScript SDK** — `npm install @prova/sdk` for web/Node.js developers
- **Python SDK** — `pip install prova` for ML researchers registering models
- WASM bindings for browser-native usage

**Smart contracts / VM** — If we add a programmable layer, WASM or Move for developer ergonomics rather than raw Rust.

*Rust for the engine, polyglot for the interfaces.*

## Team

Built by Nicklas and Capri.

## License

Private / Proprietary — not yet open source.
