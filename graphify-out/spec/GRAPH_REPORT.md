# Graph Report - spec  (2026-04-26)

## Corpus Check
- Corpus is ~7,124 words - fits in a single context window. You may not need a graph.

## Summary
- 82 nodes · 101 edges · 8 communities detected
- Extraction: 92% EXTRACTED · 8% INFERRED · 0% AMBIGUOUS · INFERRED: 8 edges (avg confidence: 0.76)
- Token cost: 0 input · 0 output

## Community Hubs (Navigation)
- [[_COMMUNITY_security-threat-model|security-threat-model]]
- [[_COMMUNITY_pdp-integration|pdp-integration]]
- [[_COMMUNITY_api-gateway|api-gateway]]
- [[_COMMUNITY_token-economics|token-economics]]
- [[_COMMUNITY_token-economics|token-economics]]
- [[_COMMUNITY_api-gateway|api-gateway]]
- [[_COMMUNITY_marketplace|marketplace]]
- [[_COMMUNITY_security-threat-model|security-threat-model]]

## God Nodes (most connected - your core abstractions)
1. `Adversary threat model` - 11 edges
2. `API gateway at prova.network/api` - 7 edges
3. `PROVA governance (1-token-1-vote)` - 7 edges
4. `Piece-CID (CommP)` - 6 edges
5. `50M PROVA prover emission (8y curve)` - 6 edges
6. `Provable Data Possession (PDP)` - 5 edges
7. `Retrievability sampling` - 5 edges
8. `Storage deal` - 5 edges
9. `Per-piece redundancy cap (default 4)` - 5 edges
10. `PDP challenge` - 4 edges

## Surprising Connections (you probably didn't know these)
- `Pre-deployment audit checklist` --semantically_similar_to--> `Adversary threat model`  [INFERRED] [semantically similar]
  spec/security-audit-checklist.md → spec/security-threat-model.md
- `Retrievability sampling` --conceptually_related_to--> `ZK aggregation (future)`  [INFERRED]
  spec/data-availability.md → spec/checkpoint-anchoring.md
- `Retrieval response headers` --semantically_similar_to--> `Global CSP + security headers`  [INFERRED] [semantically similar]
  spec/network-protocol.md → spec/api-gateway.md
- `GET /p/{cid}` --semantically_similar_to--> `Retrieval response headers`  [INFERRED] [semantically similar]
  spec/api-gateway.md → spec/network-protocol.md
- `A1: prover fails to store` --rationale_for--> `Provable Data Possession (PDP)`  [EXTRACTED]
  spec/security-threat-model.md → spec/pdp-integration.md

## Hyperedges (group relationships)
- **PDP protocol stack** — concept_pdp, concept_piece_cid, concept_challenge, concept_merkle_inclusion_proof, concept_proof_verifier_uups [EXTRACTED 1.00]
- **Deal lifecycle on-chain** — concept_deal, concept_marketplace_contract, concept_usdc_escrow, concept_streaming_release, concept_fault_deal, concept_deal_state_machine [EXTRACTED 1.00]
- **Prover-emission anti-gaming protections** — concept_prover_emission, concept_anti_self_dealing, concept_redundancy_cap, concept_30d_vesting_buffer, concept_quality_multiplier [EXTRACTED 1.00]
- **USDC fee → PROVA burn loop** — concept_protocol_fee, concept_fee_router, concept_fee_router_modes, concept_uniswap_v3_pool, concept_prova_token [EXTRACTED 1.00]
- **Governance timelock structure** — concept_governance, concept_param_timelock, concept_upgrade_timelock, concept_emergency_pause [EXTRACTED 1.00]
- **Adversary attack catalog (A1-A10)** — concept_threat_model, concept_a1_failure_to_store, concept_a2_self_dealing, concept_a3_replication_doubleclaim, concept_a4_sybil, concept_a5_wash_uploads, concept_a6_free_tier_exploit, concept_a7_fast_churn, concept_a8_cid_poisoning, concept_a9_token_secret_theft, concept_a10_smart_contract_bug [EXTRACTED 1.00]
- **Identity attestation tier ladder** — concept_identity_attestation, concept_hobby_tier, concept_prosumer_tier, concept_enterprise_tier [EXTRACTED 1.00]

## Communities

### Community 0 - "security-threat-model"
Cohesion: 0.17
Nodes (16): 30-day emission vesting buffer, A10: smart-contract bug, A2: self-dealing, A3: replication double-claim, A5: wash uploads, A6: free-tier exploitation, A7: fast-churn, Self-dealing prohibition (+8 more)

### Community 1 - "pdp-integration"
Cohesion: 0.17
Nodes (13): 32-epoch anchor cadence (maxAnchorGap), PDP challenge, 30-second challenge cadence, Proof checkpoint, Retrievability sampling, 24-hour dispute window, Merkle inclusion proof, Provable Data Possession (PDP) (+5 more)

### Community 2 - "api-gateway"
Cohesion: 0.18
Nodes (13): A9: token-secret theft, POST /api/abuse/report, API gateway at prova.network/api, Bearer-only auth (no ?token= query), Global CSP + security headers, HTTPS-only network transport, Magic-link sign-in (start + verify), Origin/CSRF guard (+5 more)

### Community 3 - "token-economics"
Cohesion: 0.2
Nodes (12): Allocation 45/50/5 split, FeeRouter (USDC→PROVA→burn), FeeRouter modes (HOLD/BURN/SPLIT), PROVA governance (1-token-1-vote), Public LBP at TGE (6% / 6M), 2-day parameter timelock, ProofVerifier UUPS upgradability, 1% protocol fee (cap 3%) (+4 more)

### Community 4 - "token-economics"
Cohesion: 0.29
Nodes (8): A1: prover fails to store, faultDeal() permissionless slashing, 7-day stake-floor grace window, MAX_PROOF_GAP (6h), minStakePerGiB (100 PROVA), PROVA prover stake, Slashing (PROVA burned), Chainlink USDC-equivalent stake floor

### Community 5 - "api-gateway"
Cohesion: 0.32
Nodes (8): A8: CID poisoning, baga... CID prefix required at upload, CID mismatch check at intake (HTTP 422), Fr32 padding, Piece-CID (CommP), Per-IP 60-uploads/min rate limit, trunc254 multihash, POST /api/upload

### Community 6 - "marketplace"
Cohesion: 0.29
Nodes (7): Storage deal, Deal state machine (Proposed→Active→Completed/Cancelled/Slashed), Canonical event schema, Indexer guidance, StorageMarketplace contract, Linear time-streaming USDC payout, USDC client escrow

### Community 7 - "security-threat-model"
Cohesion: 0.4
Nodes (5): A4: Sybil identity, Enterprise tier (>5 PB, KYB), Hobby tier (≤100 TB, pseudonymous), Tier-gated identity attestation, Prosumer tier (100 TB-5 PB, ENS/EAS)

## Knowledge Gaps
- **31 isolated node(s):** `Fr32 padding`, `trunc254 multihash`, `Merkle inclusion proof`, `30-second challenge cadence`, `Challenge response window` (+26 more)
  These have ≤1 connection - possible missing edges or undocumented components.

## Suggested Questions
_Questions this graph is uniquely positioned to answer:_

- **Why does `Adversary threat model` connect `security-threat-model` to `api-gateway`, `token-economics`, `api-gateway`, `security-threat-model`?**
  _High betweenness centrality (0.566) - this node is a cross-community bridge._
- **Why does `A1: prover fails to store` connect `token-economics` to `security-threat-model`, `pdp-integration`?**
  _High betweenness centrality (0.327) - this node is a cross-community bridge._
- **Why does `Provable Data Possession (PDP)` connect `pdp-integration` to `token-economics`, `api-gateway`?**
  _High betweenness centrality (0.289) - this node is a cross-community bridge._
- **What connects `Fr32 padding`, `trunc254 multihash`, `Merkle inclusion proof` to the rest of the system?**
  _31 weakly-connected nodes found - possible documentation gaps or missing edges._