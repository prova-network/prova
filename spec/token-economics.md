# SPEC-008: Token Economics

**Status:** Draft  
**Author:** Capri (with Koda input)  
**Created:** 2026-03-04

## 1. Token Supply

**Total supply:** 1,000,000,000 PROVA (1 billion)  
**Smallest unit:** 1 nPROVA (10⁻⁹ PROVA)  
**Deflationary mechanism:** Network fee burn (see §4)

## 2. Distribution

| Allocation | Amount | Percentage | Vesting |
|-----------|--------|-----------|---------|
| Mining rewards (storage + compute) | 400M | 40% | Emitted over ~20 years |
| Team & contributors | 200M | 20% | 4-year linear, 1-year cliff |
| Ecosystem fund | 150M | 15% | DAO-governed, no time lock |
| Investors | 150M | 15% | 2-year linear, 6-month cliff |
| Community & airdrops | 100M | 10% | Various schedules |

## 3. Emission Schedule

Mining rewards follow a halving schedule inspired by Bitcoin:

| Year | Block Reward (PROVA/epoch) | Annual Emission | Cumulative |
|------|---------------------------|-----------------|------------|
| 1-4 | ~38.05 | 100M/year | 400M max → 200M |
| 5-8 | ~19.03 | 50M/year | 200M → 300M |
| 9-12 | ~9.51 | 25M/year | 300M → 350M |
| 13-16 | ~4.76 | 12.5M/year | 350M → 375M |
| ... | halving continues | ... | asymptotic to 400M |

**Epoch duration:** 30 seconds (~1,051,200 epochs/year)

### 3.1 Dual-Weighted Distribution

Each epoch's block reward is split between storage and compute:

```
storage_share = α × total_reward
compute_share = (1 - α) × total_reward

α = governance parameter (initial: 0.5)
```

**Storage reward** is proportional to PDP-verified bytes stored.  
**Compute reward** is proportional to verified inference throughput (successful commits).

## 4. Fee Model

### 4.1 Network Fee
- **Rate:** 0.5% (50 basis points) on all payment channel transactions
- **Collection:** Automatic at payment time

### 4.2 Fee Distribution
```
network_fee → 50% burned (deflationary pressure)
            → 50% to active stakers (proportional to power)
```

### 4.3 Other Fees
| Fee Type | Amount | Destination |
|----------|--------|-------------|
| Model registration | 100 PROVA | Burned |
| Proof set creation | Gas only | Block producers |
| Dispute initiation | 50 PROVA | Escrowed (returned to winner) |

## 5. Staking Rewards

Stakers earn from two sources:
1. **Block rewards:** Share of epoch emission proportional to power
2. **Fee redistribution:** 50% of network fees

### 5.1 Power Calculation
```
total_power(node) = storage_power + compute_power

storage_power = pdp_verified_bytes / 1 GiB
compute_power = successful_inferences_last_30d × model_complexity_weight
```

### 5.2 Annual Staking Yield (Estimated)

| Scenario | Storage APY | Compute APY |
|----------|------------|-------------|
| Low utilization (10% capacity) | ~15% | ~20% |
| Medium utilization (50%) | ~8% | ~12% |
| High utilization (90%) | ~5% | ~7% |

Yields decrease as more stake enters the system (dilution).

## 6. Slashing Flows

| Event | Slash % | Distribution |
|-------|---------|-------------|
| QBP dispute lost (provider) | 10% | 50% burned, 50% to challenger |
| False challenge (challenger) | 5% | 50% burned, 50% to provider |
| PDP miss (2nd consecutive) | 5% | 100% burned |
| PDP miss (3rd) | 100% | 100% burned |
| Dispute + data unavailable | 50% | 50% burned, 25% challenger, 25% verifier |

**Annual expected slash rate:** <0.5% of total staked (at 5% audit rate with rational actors)

## 7. Economic Security Analysis

### 7.1 Minimum Viable Attack Cost

For a 51% consensus attack:
```
attack_cost = 0.51 × total_staked_value
            = 0.51 × (total_PROVA_staked × PROVA_price)
```

At $0.10/PROVA with 30% staked (300M):
```
attack_cost = 0.51 × 300M × $0.10 = $15.3M
```

### 7.2 Inference Cheating Economics

Expected value of cheating per inference:
```
EV_cheat = (1 - audit_rate) × inference_fee - audit_rate × slash_amount
         = 0.95 × fee - 0.05 × (0.10 × stake)
```

For cheating to be profitable: `fee > 0.00526 × stake`

With minimum stake of 1M PROVA ($100K): fee must exceed $526/inference — far above market rates.

### 7.3 Data Withholding Economics

If a provider stores data but doesn't actually run inference:
- PDP proofs still required (storage verified)
- QBP commits would produce random/incorrect roots
- 5% audit rate means detection within ~20 fake commits
- Slash: 10% per dispute → 50%+ loss after 5 disputes

**Conclusion:** Honest behavior is the dominant strategy for any rational actor.

## 8. Governance

The following parameters are adjustable via on-chain governance:

| Parameter | Initial Value | Range |
|-----------|--------------|-------|
| Network fee rate | 0.5% | 0.1% - 2% |
| Fee burn ratio | 50% | 25% - 100% |
| Storage/compute split (α) | 0.5 | 0.2 - 0.8 |
| Minimum provider stake | 1M PROVA | 100K - 10M |
| Minimum challenger stake | 500K PROVA | 50K - 5M |
| Audit rate | 5% | 1% - 20% |
| Slash percentages | See §6 | ±50% of initial |

## 9. References

- Bitcoin halving schedule
- Filecoin token economics (for dual-mining inspiration)
- [SPEC-004: PDP Integration](./pdp-integration.md)
- [SPEC-005: Audit Protocol](./audit-protocol.md)
- [SPEC-006: Streaming Payments](./streaming-payments.md)
