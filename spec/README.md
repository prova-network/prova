# Prova Protocol Specifications

Formal specifications for the Prova Network v2 protocol.

## Active specs

- [`pdp-integration.md`](./pdp-integration.md) — Provable Data Possession, the single storage proof mechanism used by Prova.
- [`marketplace.md`](./marketplace.md) — Storage deal lifecycle, escrow, and settlement.
- [`checkpoint-anchoring.md`](./checkpoint-anchoring.md) — Proof batch anchoring to Base.
- [`data-availability.md`](./data-availability.md) — DA sampling for retrievability challenges.
- [`api-gateway.md`](./api-gateway.md) — Client-facing REST surface.
- [`network-protocol.md`](./network-protocol.md) — Prover-to-prover HTTPS conventions.
- [`event-schema.md`](./event-schema.md) — Canonical on-chain event shapes.
- [`governance.md`](./governance.md) — Protocol parameter change process.
- [`token-economics.md`](./token-economics.md) — PROVA token utility summary (see also [TOKENOMICS-v2.md](../TOKENOMICS-v2.md) for the authoritative tokenomics).
- [`security-audit-checklist.md`](./security-audit-checklist.md) — Pre-deployment review surface.
- [`security-threat-model.md`](./security-threat-model.md) — Attack vectors + mitigations.

## Archived (pre-pivot)

The following v1 specs describe the old Prova Layer 1 design (standalone
chain + AI compute proofs + sealed storage tiers). They are preserved for
historical reference but do not describe the current project. See
[`../archive/specs-v1/`](../archive/specs-v1/):

- `qbp-protocol.md` — Quantized Bisection Proofs (AI inference verification)
- `model-registry.md` — On-chain model registry for QBP
- `activation-merkle-tree.md` — Per-layer activation hashing for QBP
- `confidential-inference.md` — TEE-attested inference
- `tee-storage-proofs.md` — TEE-attested storage fast path
- `audit-protocol.md` — Audit protocol for QBP disputes
- `validator-set.md`, `delegation-staking.md`, `bridge-security.md`,
  `light-client.md`, `account-abstraction.md`, `streaming-payments.md`
  (archived in an earlier pass; see `spec/archive/`)

## Scope of Prova v2

Prova v2 is deliberately narrow: a verifiable-storage network backed by
**PDP alone**. No sealing, no PoRep, no TEE fast-path, no AI inference
proofs. PDP is the lightweight choice; anything heavier is out of scope.
