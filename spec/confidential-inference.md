# Confidential Inference Specification (SPEC-022)

**Status:** Draft  
**Author:** Capri  
**Created:** 2026-03-04  

## 1. Overview

Confidential inference allows providers to submit inference results with encrypted activation roots, preserving privacy of model inputs and outputs during normal (unchallenged) operation. Plaintext is only revealed when a dispute is opened, at which point the standard QBP bisection game takes over.

## 2. Motivation

Inference inputs and outputs may contain sensitive data (user prompts, proprietary model outputs). Without confidentiality, all activation trees are publicly visible on-chain, creating privacy risks for both users and providers. This spec defines a commit-reveal scheme that achieves:

1. **Privacy by default** — activations encrypted unless disputed
2. **Verifiability on demand** — disputes force revelation, enabling bisection
3. **Accountability** — failure to reveal results in automatic slashing

## 3. Definitions

| Term | Definition |
|------|-----------|
| **Encrypted Root** | Ciphertext of the activation Merkle root, opaque to observers |
| **Blinding Factor** | Random bytes chosen by the provider, used to construct the blinding hash |
| **Blinding Hash** | `H(plaintext_root ‖ blinding_factor)` — committed on-chain, verified at reveal |
| **Challenge Window** | 10 epochs after commit during which disputes may be opened |
| **Reveal Window** | 5 epochs after dispute during which provider must reveal |
| **Default** | Automatic slash applied when provider fails to reveal within the window |

## 4. Protocol Flow

### 4.1 Normal Path (No Dispute)

```
Provider                              Chain
   │                                    │
   │── commit(encrypted_root,           │
   │   blinding_hash, model_id) ───────►│ Store commitment
   │                                    │
   │         ... 10 epochs pass ...     │
   │                                    │
   │                                    │── finalize() → status = Finalized
   │                                    │   (plaintext never revealed)
```

### 4.2 Dispute Path

```
Provider              Challenger              Chain
   │                      │                     │
   │── commit(...) ──────────────────────────►│
   │                      │                     │
   │                      │── dispute(id) ────►│ status = Disputed
   │                      │                     │
   │── reveal(plaintext,  │                     │
   │   blinding_factor) ─────────────────────►│ Verify H(pt‖bf) == blinding_hash
   │                      │                     │ status = Revealed
   │                      │                     │ → enter QBP bisection game
```

### 4.3 Default Path (Provider Fails to Reveal)

```
Provider              Challenger              Chain
   │                      │                     │
   │── commit(...) ──────────────────────────►│
   │                      │── dispute(id) ────►│ status = Disputed
   │                      │                     │
   │     ... 5 epochs pass without reveal ...   │
   │                      │                     │
   │                      │                     │── enforce_defaults()
   │                      │                     │   status = Defaulted
   │                      │                     │   → provider slashed
```

## 5. Data Structures

### 5.1 ConfidentialCommit

```
struct ConfidentialCommit {
    id:             CommitId,       // Monotonic identifier
    provider:       Address,        // Submitting provider
    model_id:       ModelId,        // Model being inferred
    encrypted_root: Hash,           // Encrypted activation root
    blinding_hash:  Hash,           // H(plaintext_root ‖ blinding_factor)
    epoch:          u64,            // Submission epoch
    status:         Status,         // {Committed, Disputed, Revealed, Finalized, Defaulted}
}
```

### 5.2 Status Enum

| Status | Transitions From | Transitions To |
|--------|-----------------|---------------|
| Committed | (initial) | Disputed, Finalized |
| Disputed | Committed | Revealed, Defaulted |
| Revealed | Disputed | (terminal — enters bisection) |
| Finalized | Committed | (terminal) |
| Defaulted | Disputed | (terminal — slashed) |

## 6. Cryptographic Binding

The blinding hash ensures reveal integrity:

```
blinding_hash = SHA256(plaintext_root ‖ blinding_factor)
```

At reveal time, the chain recomputes `SHA256(provided_plaintext ‖ provided_factor)` and verifies equality with the stored `blinding_hash`. This prevents:

- **Substitution attacks** — provider cannot swap in a different plaintext
- **Replay attacks** — blinding factor is unique per commitment
- **Pre-image attacks** — SHA-256 collision resistance

## 7. Timing Parameters

| Parameter | Value | Rationale |
|-----------|-------|-----------|
| CHALLENGE_WINDOW | 10 epochs | Sufficient time for challengers to verify and dispute |
| REVEAL_WINDOW | 5 epochs | Enough for provider to respond, short enough to limit griefing |

These values are governance-adjustable via the parameter update mechanism.

## 8. Security Properties

### 8.1 Privacy

- Activation data is never on-chain unless disputed
- Blinding hash reveals nothing about the plaintext (preimage resistance)
- Encrypted root format is opaque (provider chooses encryption scheme)

### 8.2 Accountability

- Every commit is cryptographically bound to a specific plaintext via blinding hash
- Failure to reveal triggers automatic slashing (no escape from disputes)
- Self-dispute is prohibited (provider cannot dispute their own commits)

### 8.3 Liveness

- Finalization is automatic after challenge window
- Default enforcement is automatic after reveal window
- No action required from honest providers whose work is unchallenged

## 9. Integration with QBP

When a commit reaches `Revealed` status, the plaintext activation root is fed into the standard QBP bisection game. The dispute resolution proceeds identically to non-confidential inference, using the revealed root as the activation tree root.

## 10. CLI Interface

The `prova confidential` subcommand provides operator access:

```
prova confidential commit <model-id> <encrypted-root> --blinding-factor <hex>
prova confidential reveal <commit-id> <plaintext-root> --blinding-factor <hex>
prova confidential dispute <commit-id> [--challenger <addr>]
prova confidential status <commit-id> [--json]
prova confidential list [--provider <addr>] [--status <filter>] [--limit <n>]
prova confidential finalize [--epoch <n>]
prova confidential enforce-defaults [--epoch <n>]
```

## 11. Gas Considerations

- Commit: ~50K gas (two hash stores + metadata)
- Dispute: ~30K gas (status update + event)
- Reveal: ~80K gas (hash verification + status update + bisection trigger)
- Finalize/Default: ~20K gas per commit (batch operation)

## 12. Future Work

- **ZK-SNARK verification** (CHAIN-036): Replace reveal with zero-knowledge proof that the plaintext satisfies model constraints without revealing it
- **Ephemeral key management** (NODE-032): Per-inference encryption keys for encrypted_root
- **Client SDK** (SDK-011): High-level API for confidential inference submission
