# Prova Development Roadmap

Last updated: April 1, 2026

## Status: Foundation Complete, Testnet Next

63K+ LOC Rust, 1,690 tests, 4 crates, 23 specs. Core abstractions built.
Next milestone: multi-node testnet with real consensus and proof verification.

---

## Phase 1: Foundation ✅ (Q1 2026, Complete)

- [x] Chain crate: block production, staking, rewards, governance, disputes
- [x] Node crate: P2P skeleton, RPC, storage, PDP proofs, devnet, CLI
- [x] SDK crate: RPC client, events, marketplace, delegation, wallet
- [x] QBP protocol design and specification
- [x] Three-tier storage proof design (PDP/TEE/PoRep)
- [x] GPU determinism experiments (RTX 5080 + RTX 6000)
- [x] ERC-20 smart contracts (token, vesting, crowdsale)
- [x] Whitepaper v0.2 (storage + compute + networking)
- [x] Website with SAFT tier model

## Phase 2: Consensus & Verification (Q2 2026, Active)

### P0: Consensus (blocks 3-node testnet)
- [ ] **CONS-001**: BFT finality gadget (2/3 voting rounds, timeout handling)
- [ ] **CONS-002**: VRF-based block producer election (weighted by storage+compute+network)
- [ ] **CONS-003**: Epoch transitions (reward distribution, power recalculation)
- [ ] **CONS-004**: Slashing logic (double-sign, offline penalties, compute fraud)

### P0: P2P Network (blocks multi-node)
- [ ] **NET-001**: libp2p integration (gossipsub for blocks/txs, kademlia for peer discovery)
- [ ] **NET-002**: Block propagation and validation pipeline
- [ ] **NET-003**: Transaction gossip with deduplication and priority
- [ ] **NET-004**: Peer scoring and ban list
- [ ] **NET-005**: NAT traversal (relay, hole-punching)

### P0: State Machine
- [ ] **STATE-001**: Deterministic state transition function (account model)
- [ ] **STATE-002**: Transaction types (transfer, stake, register-provider, submit-proof, challenge)
- [ ] **STATE-003**: State root (Merkle Patricia trie or similar)
- [ ] **STATE-004**: Gas metering for on-chain operations

### P0: QBP End-to-End
- [ ] **QBP-001**: Activation Merkle tree builder (per-layer SHA-256 hashing)
- [ ] **QBP-002**: On-chain bisection game (challenge, reveal, binary search rounds)
- [ ] **QBP-003**: Single-layer verifier (re-execute one layer, compare output)
- [ ] **QBP-004**: Architecture group registry (nodes grouped by GPU compute capability)
- [ ] **QBP-005**: Random audit selection (drand-based, configurable audit rate)
- [ ] **QBP-006**: Integration test: two nodes, same model, verify honest result accepted
- [ ] **QBP-007**: Integration test: dishonest node detected and slashed via bisection

### P1: Storage Proofs
- [ ] **STOR-001**: PDP proof generation and on-chain verification (end-to-end)
- [ ] **STOR-002**: Proof set registration and challenge scheduling (drand)
- [ ] **STOR-003**: Data onboarding pipeline (CommP computation, root registration)
- [ ] **STOR-004**: Retrieval protocol (serve raw data on request)

### P1: Genesis & Tokenomics
- [ ] **GEN-001**: Genesis block generator (initial allocations per tokenomics v2)
- [ ] **GEN-002**: Vesting schedule enforcement on-chain
- [ ] **GEN-003**: Mining reward minting function (halving schedule: 5 halvings, ~20yr)
- [ ] **GEN-004**: Three-way reward split oracle (storage/compute/network demand weighting)
- [ ] **GEN-005**: Fee burn mechanism (50% of transaction fees)

## Phase 3: Testnet (Q3 2026)

### P0: Multi-Node Testnet
- [ ] **TEST-001**: 3-5 node testnet on real machines (not simulated)
- [ ] **TEST-002**: Block production at target cadence (30s epochs)
- [ ] **TEST-003**: Storage providers onboard data and pass PDP challenges
- [ ] **TEST-004**: Compute providers serve inference and pass random audits
- [ ] **TEST-005**: Staking, delegation, and reward distribution working end-to-end
- [ ] **TEST-006**: Faucet and block explorer for testnet

### P0: Networking Layer (Andy's design)
- [ ] **IPv6-001**: LIR registration, acquire /48 IPv6 block (~$200/yr)
- [ ] **IPv6-002**: Per-VM IPv6 allocation and DNS (vm{id}.{tenant}.prova.network)
- [ ] **IPv6-003**: BGP announcement from Tier 1 SP nodes
- [ ] **SNI-001**: HTTP/HTTPS IPv4 fallback via SNI routing on SP edge
- [ ] **BW-001**: Bandwidth metering and per-epoch reporting
- [ ] **BW-002**: Egress pricing (per-GB, SP-configurable with ceiling)
- [ ] **DDoS-001**: Per-VM ingress caps and blackhole routing

### P1: Developer Experience
- [ ] **DX-001**: TypeScript SDK (npm package)
- [ ] **DX-002**: Python SDK (pip package)
- [ ] **DX-003**: CLI wallet with HD key derivation and mnemonic backup
- [ ] **DX-004**: Docker-based local devnet (single command: `prova devnet start`)
- [ ] **DX-005**: API documentation (OpenAPI spec, auto-generated)

### P1: TEE Fast Path
- [ ] **TEE-001**: SGX/TDX enclave design (disk encryption, key sealing)
- [ ] **TEE-002**: Enclave build pipeline (reproducible, chain-versioned)
- [ ] **TEE-003**: Spot-check verification protocol
- [ ] **TEE-004**: Fallback-to-PDP grace period on TCB deprecation

## Phase 4: Security & Hardening (Q4 2026)

- [ ] **SEC-001**: External security audit (consensus, state machine, smart contracts)
- [ ] **SEC-002**: Formal verification of QBP bisection protocol
- [ ] **SEC-003**: Fuzzing campaign (chain state transitions, P2P message handling)
- [ ] **SEC-004**: Economic simulation (attack cost analysis, slashing parameter tuning)
- [ ] **SEC-005**: Bug bounty program launch
- [ ] **SEC-006**: Stress test (100+ nodes, sustained load, adversarial conditions)

## Phase 5: Mainnet (Q1 2027)

- [ ] **MAIN-001**: Genesis ceremony (multisig key generation, initial state)
- [ ] **MAIN-002**: ERC-20 to native bridge (1:1 token swap)
- [ ] **MAIN-003**: BGP peering with Tier 1 SPs
- [ ] **MAIN-004**: Exchange listing preparation (custody, API integration)
- [ ] **MAIN-005**: Ecosystem grant program
- [ ] **MAIN-006**: Governance activation (parameter adjustment voting)

---

## Priority Legend

- **P0**: Blocks the next milestone. Must complete before advancing.
- **P1**: Important but can ship testnet/mainnet without it initially.
- **P2**: Nice to have, improves quality of life.

## Open Research Questions

1. **Cross-architecture determinism**: Can TensorRT strict INT8 mode produce identical results across Turing/Ampere/Ada/Blackwell? If yes, architecture groups become unnecessary.
2. **Canonical CPU verifier**: Is x86-64 IEEE 754 strict mode sufficient for single-layer verification? What about ARM nodes?
3. **Networking economics**: Optimal bandwidth pricing model (flat vs tiered vs auction)?
4. **TEE trust lifecycle**: How quickly can governance deprecate a compromised TCB level?
