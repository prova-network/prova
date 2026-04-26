# Prova Protocol Specifications

Formal specifications for the Prova protocol.

The **canonical rendered version** is at [spec.prova.network](https://spec.prova.network).
This directory holds the markdown source. Each file here corresponds to a
section of the published spec; see [`status`](https://spec.prova.network/status)
for state and audit metadata per section.

## Sections

| § | Spec | File |
| --- | --- | --- |
| 2.1 | PDP integration | [`pdp-integration.md`](./pdp-integration.md) |
| 2.2 | Checkpoint anchoring | [`checkpoint-anchoring.md`](./checkpoint-anchoring.md) |
| 2.3 | Data availability | [`data-availability.md`](./data-availability.md) |
| 3.1 | Marketplace | [`marketplace.md`](./marketplace.md) |
| 3.2 | Event schema | [`event-schema.md`](./event-schema.md) |
| 4.1 | Network protocol | [`network-protocol.md`](./network-protocol.md) |
| 4.2 | API gateway | [`api-gateway.md`](./api-gateway.md) |
| 5.1 | Token economics | [`token-economics.md`](./token-economics.md) |
| 5.2 | Governance | [`governance.md`](./governance.md) |
| 6.1 | Security threat model | [`security-threat-model.md`](./security-threat-model.md) |
| 6.2 | Audit checklist | [`security-audit-checklist.md`](./security-audit-checklist.md) |

## Source-of-truth order

When this spec, the public documentation, and the deployed contracts
disagree, the order of authority is:

1. **Deployed contracts** (Base mainnet, then Base Sepolia)
2. **Source code** in `prova-network/contracts`
3. **This spec** (rendered at spec.prova.network)
4. **The whitepaper** at prova.network/whitepaper
5. **Marketing / docs site copy**

## Contributing

To propose changes, edit a file in this directory and open a PR. The
spec.prova.network site rebuilds automatically from this directory on
merge to `main`.

If you're touching a section that's marked **Draft / WIP** at
[spec.prova.network/status](https://spec.prova.network/status) — go
ahead, those are explicitly open. For sections marked **Reliable** or
**Stable**, please open an issue first.
