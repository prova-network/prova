# SPEC-020: Delegation & Staking Specification

**Status:** Draft  
**Authors:** Capri (autonomous)  
**Created:** 2026-03-04  
**Implements:** SPEC-020  
**Dependencies:** CHAIN-031 (Delegation), CHAIN-032 (Liquid Staking), SPEC-011 (Governance)

## 1. Overview

This specification formalizes Prova's delegation and staking system — the economic layer that lets token holders participate in network security and governance without operating infrastructure. Delegators stake PROVA to inference providers, earning proportional rewards minus commission, while bearing proportional slashing risk.

The system comprises three interlocking mechanisms:
1. **Direct delegation** — stake PROVA to a provider, earn rewards, bear slashing
2. **Liquid staking derivatives** — receive transferable stPROVA representing the staked position
3. **Governance voting** — delegated stake carries governance weight (with delegator override)

## 2. Staking Parameters

| Parameter | Value | Rationale |
|-----------|-------|-----------|
| `UNBONDING_PERIOD` | 14,400 epochs (~10 days) | Long enough to slash misbehaviour discovered post-fact |
| `MIN_DELEGATION` | 1,000,000 units (1 PROVA) | Prevents dust delegations that bloat state |
| `MAX_COMMISSION_BPS` | 5,000 (50%) | Caps provider extraction; market competition drives lower |
| `COMMISSION_CHANGE_COOLDOWN` | 7,200 epochs (~5 days) | Prevents bait-and-switch commission spikes |
| `MAX_COMMISSION_INCREASE_BPS` | 500 (5% per change) | Gradual increases only; unlimited decreases |
| `MIN_SELF_DELEGATION` | 10,000,000 units (10 PROVA) | Providers must have skin in the game |
| `MAX_DELEGATORS_PER_PROVIDER` | 10,000 | Bounds state size and reward iteration cost |
| `REDELEGATION_COOLDOWN` | 7,200 epochs (~5 days) | Prevents delegation hopping to avoid slashing |

## 3. Delegation Lifecycle

### 3.1 Delegate

```
Delegator → DelegationSystem.delegate(provider, amount)
  ├─ Validate: provider registered, amount ≥ MIN_DELEGATION
  ├─ Transfer: amount locked from delegator balance
  ├─ Record: DelegationEntry { delegator, provider, amount, start_epoch }
  ├─ Mint: stPROVA at current exchange_rate (amount / rate)
  └─ Emit: DelegationCreated event
```

Delegation is effective immediately for reward accrual. For governance snapshots, the delegation counts from the next epoch boundary.

### 3.2 Undelegate

```
Delegator → DelegationSystem.undelegate(provider, amount)
  ├─ Validate: delegation exists, amount ≤ delegated
  ├─ Burn: corresponding stPROVA
  ├─ Create: UnbondingEntry { amount, completion_epoch: now + UNBONDING_PERIOD }
  ├─ Reduce: active delegation immediately (no longer earning rewards)
  └─ Emit: UndelegationStarted event
```

After `UNBONDING_PERIOD`, delegator calls `claim_unbonded()` to receive liquid PROVA. During unbonding, tokens are still subject to slashing.

### 3.3 Redelegate

```
Delegator → DelegationSystem.redelegate(from_provider, to_provider, amount)
  ├─ Validate: different providers, cooldown elapsed, amount valid
  ├─ Atomic: reduce from_provider delegation, increase to_provider
  ├─ Burn old stPROVA, mint new stPROVA-<to_provider> at respective rates
  ├─ Record: redelegation timestamp (cooldown starts)
  └─ No unbonding period (instant switch)
```

Redelegation skips the unbonding queue but enforces `REDELEGATION_COOLDOWN` to prevent slash evasion via rapid hopping.

## 4. Reward Distribution

### 4.1 Accumulation

Providers accumulate rewards from:
- **Block production rewards** — proportional to total stake
- **Inference fees** — from jobs executed
- **Challenger bounties** — from successful fraud proofs

Rewards accrue to the provider's reward pool, not to individual delegators.

### 4.2 Distribution Formula

Per distribution epoch (every 2,880 epochs = ~2 days):

```
provider_commission = pool_rewards × commission_rate
delegator_pool = pool_rewards - provider_commission

For each delegator:
  share = delegator.amount / total_delegated_to_provider
  reward = delegator_pool × share
```

### 4.3 Auto-Compound

Delegators may enable auto-compound, which automatically re-delegates rewards rather than crediting liquid balance. This increases their stPROVA position at zero gas cost (batched by the protocol).

## 5. Slashing Propagation

When a provider is slashed (fraud proof, liveness fault):

```
slash_amount = provider_total_stake × slash_rate
provider_self_slash = slash_amount × (self_stake / total_stake)
delegator_slash = slash_amount × (delegator_amount / total_stake)  // per delegator
```

Slashing reduces:
1. Active delegation amounts proportionally
2. stPROVA exchange rate (all derivative holders share the loss)
3. Unbonding entries (tokens in unbonding are still slashable)

### 5.1 Slash Caps

| Fault Type | Max Slash Rate |
|-----------|---------------|
| Single inference fraud | 5% of provider stake |
| Repeated fraud (3+ in 7 days) | 25% of provider stake |
| Liveness fault (missed 50%+ windows) | 1% per missed window, max 10% |
| Coordinated attack (>33% stake) | 100% (jailable) |

## 6. Liquid Staking Derivatives

### 6.1 stPROVA Token

Each provider has a unique derivative: `stPROVA-<provider_address_prefix>`. The exchange rate is:

```
exchange_rate = total_staked_to_provider / total_stPROVA_supply_for_provider
```

As rewards accrue (increasing total_staked), exchange_rate increases → stPROVA appreciates.

### 6.2 Transferability

stPROVA tokens are freely transferable. This enables:
- Secondary market trading of staked positions
- Use as collateral in DeFi protocols (future)
- Exit without waiting for unbonding (sell stPROVA instead)

The recipient inherits the staking position including slashing risk.

### 6.3 Exchange Rate Updates

Exchange rate is recalculated on every:
- Delegation (new stake → mint stPROVA)
- Undelegation (burn stPROVA → reduce stake)
- Reward distribution (stake increases, supply unchanged → rate increases)
- Slash event (stake decreases, supply unchanged → rate decreases)

## 7. Governance Integration

### 7.1 Vote Weight

Governance voting power derives from staked PROVA:
- Direct stakers: voting power = staked amount at snapshot epoch
- Delegators: voting power flows to provider by default
- **Delegator override**: a delegator may vote directly, overriding their provider's vote for their share

### 7.2 Delegation Governance Voting (CHAIN-033)

Providers inherit governance voting power from all delegations. The flow:

```
Provider votes Yes with 1M delegated + 100K self-stake = 1.1M votes Yes
Delegator A (200K) votes No → overrides 200K from provider
Result: Provider's effective vote = 900K Yes, Delegator A = 200K No
```

Override priority: delegator direct vote > provider vote > abstain (default).

### 7.3 Vote Delegation Chain

Prova does NOT support transitive vote delegation (A→B→C). Only single-hop: delegator→provider. This prevents:
- Circular delegation
- Unbounded chain traversal at vote-counting time
- Governance capture through delegation chains

## 8. Security Considerations

### 8.1 Stake Centralization

Risk: A single provider accumulates majority stake through attractive commission rates.

Mitigations:
- `MAX_DELEGATORS_PER_PROVIDER` caps delegation capacity
- Quadratic slashing: coordinated faults slash proportionally more
- Governance dashboard shows stake distribution
- No protocol-enforced cap on per-provider stake (market-driven)

### 8.2 Slash Evasion

Risk: Delegators redelegate away before slash executes.

Mitigations:
- `REDELEGATION_COOLDOWN` prevents rapid hopping
- Unbonding tokens remain slashable
- Slashing is applied atomically in the same block as the fraud proof

### 8.3 Commission Manipulation

Risk: Provider sets 0% commission, attracts delegators, then raises to 50%.

Mitigations:
- `COMMISSION_CHANGE_COOLDOWN` (5 days between changes)
- `MAX_COMMISSION_INCREASE_BPS` (5% max increase per change)
- All commission changes emit events (delegators can monitor and exit)

## 9. State Requirements

### 9.1 Per-Provider State

```
ProviderStakeInfo {
    self_delegation: Amount,
    total_delegated: Amount,
    commission_bps: u16,
    last_commission_change: Epoch,
    reward_pool: Amount,
    delegator_count: u32,
    st_supply: Amount,          // stPROVA supply for this provider
    exchange_rate_num: u128,    // numerator (for precision)
    exchange_rate_den: u128,    // denominator
}
```

### 9.2 Per-Delegation State

```
DelegationEntry {
    delegator: Address,
    provider: Address,
    amount: Amount,
    st_balance: Amount,         // stPROVA held
    auto_compound: bool,
    last_redelegation: Epoch,
    start_epoch: Epoch,
}
```

### 9.3 Unbonding Queue

```
UnbondingEntry {
    delegator: Address,
    provider: Address,
    amount: Amount,
    completion_epoch: Epoch,
}
```

## 10. Gas Costs

| Operation | Estimated Gas |
|-----------|--------------|
| Delegate | ~200K (state write + mint) |
| Undelegate | ~150K (state update + queue entry) |
| Redelegate | ~300K (two provider updates + mint/burn) |
| Claim unbonded | ~100K (queue scan + transfer) |
| Vote (direct) | ~80K (snapshot lookup + record) |
| Reward distribution (batch) | ~50K per delegator in batch |

## 11. Implementation Status

- `chain/src/delegation.rs` — Core delegation (18 tests) ✅
- `chain/src/liquid_staking.rs` — stPROVA derivatives (15 tests) ✅
- `node/src/delegation_cli.rs` — CLI commands (31 tests) ✅
- `sdk/src/delegation.rs` — Client SDK (16 tests) ✅
- `chain/src/governance.rs` — Base governance (17 tests) ✅
- `chain/src/delegation_gov.rs` — Delegation governance voting (CHAIN-033) ✅
