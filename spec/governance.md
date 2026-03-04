# SPEC-011: Governance Specification

**Status:** Draft  
**Authors:** Capri (autonomous)  
**Created:** 2026-03-04

## 1. Overview

Prova uses on-chain governance for protocol parameter changes, treasury spending, and model registry policy. Governance is token-weighted: 1 staked PROVA = 1 vote. Only staked tokens vote (aligns governance power with security commitment).

## 2. Proposal Types

| Type | Description | Quorum | Threshold | Voting Period |
|------|-------------|--------|-----------|---------------|
| `ParameterChange` | Modify chain parameters (block reward, challenge window, etc.) | 10% staked supply | 66.7% | 7 days (20,160 epochs) |
| `TreasurySpend` | Allocate treasury funds to an address | 15% staked supply | 66.7% | 14 days (40,320 epochs) |
| `ModelPolicy` | Change model registry rules (min stake, allowed architectures) | 5% staked supply | 50.1% | 3 days (8,640 epochs) |
| `EmergencyAction` | Fast-track critical fixes (e.g., pause disputes) | 33% staked supply | 75% | 1 day (2,880 epochs) |

## 3. Proposal Lifecycle

```
Created → Active → [Passed | Rejected | Expired] → Executed (if passed + timelock)
```

1. **Created**: Proposer submits proposal with description, type, and payload. Requires deposit of 1,000 PROVA (refunded if quorum met, slashed if <5% participation — anti-spam).
2. **Active**: Voting opens for the type-specific duration. Votes: `Yes`, `No`, `Abstain`. Abstain counts toward quorum but not threshold.
3. **Passed**: Quorum met AND yes votes exceed threshold of (yes + no). Enters timelock.
4. **Rejected**: Threshold not met after voting period.
5. **Expired**: Quorum not met after voting period. Deposit returned.
6. **Executed**: After timelock (2,880 epochs = 1 day), anyone can trigger execution.

## 4. Voting Power

- Voting power = staked amount at proposal creation epoch (snapshot)
- Delegated stake counts toward delegatee's voting power
- A staker who votes directly overrides any delegatee vote
- Vote changes allowed until voting period ends

## 5. Delegation

Stakers may delegate voting power to another address:
- `delegate(delegatee)` — all voting power to delegatee
- `undelegate()` — reclaim voting power (effective next epoch)
- Delegation does NOT transfer stake or rewards
- Delegatees cannot re-delegate received power

## 6. Parameter Change Payloads

ParameterChange proposals specify a key-value pair:

| Key | Type | Range | Description |
|-----|------|-------|-------------|
| `challenge_window` | u64 | 10..1000 | Epochs for dispute window |
| `min_provider_stake` | u128 | 100..1M | Minimum stake to serve inference |
| `block_reward` | u128 | 0..100 | PROVA per epoch |
| `slash_fraction` | u64 | 1..100 | Slash percentage (basis points × 100) |
| `proof_reward` | u128 | 0..50 | Challenger bounty per successful dispute |
| `payment_network_fee_bps` | u64 | 0..500 | Network fee on streaming payments |

## 7. Treasury

- Treasury accrues from: network fees (0.5% of payments), slashed stakes, unclaimed rewards after 1 year
- Treasury balance tracked in state trie
- TreasurySpend proposals specify: recipient address, amount, memo
- Maximum single spend: 10% of treasury balance

## 8. Security Considerations

- **Vote buying**: Mitigated by requiring staked tokens (opportunity cost of staking)
- **Flash governance**: Snapshot at proposal creation prevents stake-borrow-vote-unstake
- **Griefing**: Deposit requirement prevents proposal spam
- **Execution delay**: Timelock gives users time to exit if they disagree with passed proposals
- **Emergency path**: High quorum + threshold for emergency actions prevents capture
