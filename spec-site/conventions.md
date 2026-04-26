<div class="spec-meta">
  <span class="label">State</span> <span class="badge state-reliable">Reliable</span>
  <span class="label">Theory audit</span> <span class="badge audit-na">N/A</span>
  <span class="label">Last updated</span> 2026-04-26
</div>

# 1.3 Conventions

This section defines notational and structural conventions used across the
Prova spec. Every other section assumes these.

## 1.3.1 Numbering

Sections use a hierarchical decimal numbering: `1.`, `1.1`, `1.1.2`, etc.
Numbers reflect the table of contents in the [Status overview](/status).
Cross-references between sections always use the section number, not just
the title.

## 1.3.2 Document metadata

Every spec section carries the same metadata block at the top:

```html
<div class="spec-meta">
  <span class="label">State</span>     <span class="badge state-...">…</span>
  <span class="label">Theory audit</span> <span class="badge audit-...">…</span>
  <span class="label">Last updated</span> YYYY-MM-DD
</div>
```

The five state labels and three audit labels are documented in the
[State legend](/status#state-legend) and [Audit legend](/status#audit-legend).

## 1.3.3 Code examples

- **Solidity** examples are shown in `solidity` fenced blocks. They reflect
  the current `prova-network/contracts` source. When the spec and the source
  diverge, the source is authoritative.
- **TypeScript / JavaScript** examples are shown in `ts` or `js` blocks.
- **Shell** examples use `bash`.
- **Pseudocode** uses unlabeled fenced blocks, written in a Python-ish style.

## 1.3.4 Units

| Concept | Unit | Notes |
|---|---|---|
| Time | seconds (unix epoch where absolute) | `block.timestamp` is canonical |
| Storage size | bytes (binary, GiB / TiB / PiB) | Marketplace uses raw bytes; UIs may format |
| USDC | 6-decimal integer or 18-decimal "ether" in Solidity | Convert at the boundary; we use 18-decimal in tests for simplicity |
| PROVA | 18-decimal "ether" in Solidity | Standard ERC-20 |
| Percentages | basis points (bps), 10 000 = 100 % | Avoid floating-point |

## 1.3.5 Hashes and CIDs

- **piece-CID** (CommP) is the canonical content identifier for every piece
  stored on Prova. See [§2.1 PDP integration](/pdp-integration#piece-cid).
- The printable form for fil-commitment-unsealed CIDs begins with `baga…`.
- All other on-chain identifiers (deal id, dataset id, prover id, etc.)
  are `uint256` unless explicitly noted.

## 1.3.6 Versioning

The spec is versioned as a single document at the same version as the
whitepaper:

- **v1.0** — current.
- **v0.x** — historical drafts; not authoritative.

Breaking changes to a spec section require a major bump and a migration
note. Non-breaking changes go in the [Changelog](/changelog) (when it
exists) and bump the patch version.

## 1.3.7 RFC 2119 keywords

The keywords **MUST**, **MUST NOT**, **REQUIRED**, **SHALL**, **SHALL NOT**,
**SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY**, **OPTIONAL** in this
spec are to be interpreted as described in
[RFC 2119](https://www.rfc-editor.org/rfc/rfc2119).

When these keywords appear they are bolded. Lowercase uses of the same
words are non-normative.

## 1.3.8 Source of truth

When the spec, the public documentation, and the deployed contracts
disagree, the order of authority is:

1. **Deployed contracts** (on Base mainnet, then Base Sepolia)
2. **Source code** in `prova-network/contracts`
3. **This spec**
4. **The whitepaper**
5. **Marketing / docs site copy**

Every spec section that describes contract behavior includes a permalink
to the canonical Solidity source.
