# Contributing to Prova

Thanks for contributing to Prova. This document covers the basics to get you started.

## Getting Started

```bash
# Clone (private repo — requires access)
git clone github.com/prova-network/prova
cd prova

# Build
cargo build --workspace

# Test
cargo test --workspace
```

No external dependencies required beyond a standard Rust toolchain.

## Code Style

We use standard Rust formatting and linting:

```bash
# Format code
cargo fmt

# Run clippy (must pass with zero warnings)
cargo clippy --workspace -D warnings
```

All PRs must pass clippy without warnings. If you believe a lint is incorrect, suppress it with a comment explaining why.

## Commit Messages

Use [Conventional Commits](https://www.conventionalcommits.org/):

| Type | Use for |
|------|---------|
| `feat` | New functionality |
| `fix` | Bug fixes |
| `docs` | Documentation changes |
| `test` | Adding or fixing tests |
| `chore` | Maintenance, deps, tooling |

Examples:
```
feat(qbp): add early termination on matching roots
fix(tree): handle odd-numbered layer counts
docs(spec): clarify PDP challenge flow
test(chain): add dispute arena integration tests
```

## Testing Requirements

All PRs must pass the full test suite:

```bash
cargo test --workspace
```

- Include tests for new functionality
- Ensure existing tests continue to pass
- Do not disable tests without explicit justification in the PR description

## Spec Contributions

Protocol specifications live in `spec/` as Markdown files. Every spec must include:

1. **Overview** — What problem does this solve?
2. **Security Considerations** — Threat model, attack vectors, mitigations

Specs are documentation, not code — prioritize clarity and precision over cleverness.

## Architecture Groups

Prova relies on **architecture groups** for determinism: nodes with identical GPU hardware produce bit-identical quantized outputs. If you're adding support for a new GPU architecture:

1. Run the determinism validation suite on the target hardware
2. Document the architecture group identifier (e.g., `nvidia-h100-sm90`)
3. Include benchmark results showing identical outputs across multiple runs
4. Add the new group to the supported architectures list

Cross-architecture verification is intentionally out of scope — do not attempt to bridge different GPU families.

## Where to Find Things

| Directory | Contents |
|-----------|----------|
| `spec/` | Protocol specifications (QBP, PDP, Model Registry, etc.) |
| `proto/` | Protobuf wire format definitions |
| `node/` | Rust node implementation (Merkle trees, inference runner, QBP participant) |
| `chain/` | Chain simulation (commit store, dispute arena, stake ledger, model registry) |
| `research/` | Experiment results and design exploration |

When in doubt:
- Protocol questions → check `spec/`
- Chain behavior → check `chain/`
- Node implementation → check `node/`

## Questions?

Open an issue in the private repo for architectural questions or clarification on requirements.
