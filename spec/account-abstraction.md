# Account Abstraction Specification (SPEC-023)

## Abstract

Defines the account model for Prova, supporting both externally owned accounts (EOA) and smart accounts (multi-sig wallets, programmatic signers). Smart accounts enable flexible authentication, spending policies, and key rotation without changing on-chain identity.

## Account Types

### Externally Owned Account (EOA)
- Controlled by a single Ed25519 keypair
- Address = SHA-256(pubkey)[0..32]
- Signs transactions directly
- Fields: `nonce: u64`, `balance: u64`, `pubkey: [u8; 32]`

### Smart Account (Multi-sig Wallet)
- Controlled by M-of-N owner set
- Address = SHA-256(owners ++ threshold ++ salt)[0..32]
- Transactions require proposal → approval → execution flow
- Fields: `nonce: u64`, `balance: u64`, `owners: Vec<Address>`, `threshold: u32`, `proposals: Map<ProposalId, Proposal>`

## Transaction Authorization

### EOA Transactions
1. Sender signs `(chain_id, nonce, to, value, calldata, gas_limit, gas_price)` with Ed25519
2. Verifier checks signature against sender's registered pubkey
3. Nonce incremented atomically on execution

### Smart Account Transactions
1. Any owner submits a `Propose` transaction (auto-approves from proposer)
2. Other owners submit `Approve` transactions referencing `(wallet_id, proposal_id)`
3. Once `approvals >= threshold`, any owner submits `Execute`
4. Executed proposal increments wallet nonce, transfers value, invokes calldata

## Proposal Lifecycle

```
Created → Pending → { Approved | Rejected | Expired | Cancelled }
                        ↓
                    Executed
```

- **TTL**: Each wallet sets `proposal_ttl` (epochs). Proposals expire after `created_at + ttl`.
- **Cancellation**: Only the original proposer can cancel.
- **Rejection**: Any owner can reject. Rejections are informational (don't block execution if threshold met).

## Spending Policies

### Daily Limits
- Optional per-wallet `daily_limit: u64`
- Tracked via `daily_spent` counter, resets every `EPOCHS_PER_DAY` (2880 at 30s epochs)
- Proposals exceeding remaining daily budget require `threshold + 1` approvals (elevated threshold)
- Proposals with `value == 0` (governance-only) bypass daily limit

### Future Extensions (not in v1)
- Session keys: temporary authorization for specific actions
- Social recovery: designated guardians can rotate owner set
- Spending curves: rate-limited withdrawals over time windows

## Owner Management

### Add Owner
- Requires existing M-of-N approval as a proposal with calldata `ADD_OWNER(address, new_threshold?)`
- New threshold optional; if omitted, keeps current threshold
- Validation: new owner not already in set, threshold ≤ new owner count

### Remove Owner
- Requires M-of-N approval: calldata `REMOVE_OWNER(address, new_threshold?)`
- Validation: remaining owners ≥ 2, threshold ≤ remaining count
- If removed owner had pending proposals, they remain (can still be approved/executed by others)

### Key Rotation (EOA)
- Owner submits `ROTATE_KEY(old_pubkey, new_pubkey, proof)` where proof = signature from new key over old key
- Atomic: old key invalidated, new key activated in same block

## Gas & Fee Handling

- Smart account transactions consume gas from the **submitter's** EOA balance (the owner who submits propose/approve/execute)
- Executed proposal value comes from the **wallet's** balance
- Gas refunds (unused gas) return to the submitter's EOA

## Security Considerations

1. **Replay protection**: Chain ID + nonce per account. Smart accounts use separate nonce from owner EOAs.
2. **Proposal front-running**: Proposals are content-addressed (hash of target+value+calldata+nonce). Duplicate proposals rejected.
3. **Owner collusion**: Threshold should be set appropriately for the trust model. 2-of-3 minimum recommended for treasury wallets.
4. **Expired proposal cleanup**: Expired proposals can be garbage-collected after `2 × ttl` epochs. No state bloat from abandoned proposals.
5. **Nonce gaps**: Smart account nonce only increments on execution, not on proposal creation. No gaps from cancelled/expired proposals.

## Wire Format

### Proposal (protobuf)
```protobuf
message MultisigProposal {
  uint64 id = 1;
  bytes proposer = 2;      // 32-byte address
  bytes target = 3;         // 32-byte address
  uint64 value = 4;
  bytes calldata = 5;
  repeated bytes approvals = 6;  // set of 32-byte addresses
  repeated bytes rejections = 7;
  uint64 created_at = 8;   // epoch
  bool executed = 9;
  bool cancelled = 10;
}

message MultisigWallet {
  bytes id = 1;             // 32-byte wallet address
  repeated bytes owners = 2;
  uint32 threshold = 3;
  uint64 nonce = 4;
  uint64 daily_limit = 5;
  uint64 proposal_ttl = 6;
  repeated MultisigProposal proposals = 7;
}
```

## RPC Methods

| Method | Params | Returns |
|--------|--------|---------|
| `multisig_createWallet` | `owners, threshold, ttl, daily_limit` | `wallet_id` |
| `multisig_propose` | `wallet_id, target, value, calldata` | `proposal_id` |
| `multisig_approve` | `wallet_id, proposal_id` | `tx_hash` |
| `multisig_reject` | `wallet_id, proposal_id` | `tx_hash` |
| `multisig_execute` | `wallet_id, proposal_id` | `(target, value, calldata)` |
| `multisig_cancel` | `wallet_id, proposal_id` | `tx_hash` |
| `multisig_getWallet` | `wallet_id` | `WalletSummary` |
| `multisig_getProposal` | `wallet_id, proposal_id` | `ProposalSummary` |
| `multisig_listProposals` | `wallet_id, pending_only` | `Vec<ProposalSummary>` |

## Implementation References

- Chain logic: `chain/src/multisig.rs` (15 tests)
- CLI commands: `node/src/multisig_cli.rs` (36 tests)
- Client SDK: `sdk/src/multisig.rs` (16 tests)
