# Token Economics — Initial Sketch

## Design Principles
1. No privileged data classes (1 byte = 1 byte)
2. Dual reward streams must self-balance
3. Low barrier to entry for small operators
4. Simple pledge model
5. No mining reserve (no hidden supply)

## Reward Streams

### Storage Rewards
- Proportional to raw bytes stored and proven (PDP/PoSt)
- No multipliers
- Reward per byte decreases as total network storage grows (similar to Filecoin baseline)

### Compute Rewards
- Proportional to verified inference jobs completed
- Priced in compute units (benchmarked, not raw GPU model)
- Revenue = client payments + protocol subsidy (early network)

### Rebalancing Mechanism
If one stream becomes disproportionately profitable:
- Market forces: operators add hardware for the profitable activity → competition drives profit down
- Protocol nudge: epoch-by-epoch adjustment of subsidy split based on supply/demand ratio
- Target: roughly equal ROI for storage-only, compute-only, and hybrid nodes

## Pledge Model
```
pledge = base_pledge + (expected_reward × lock_multiplier)
```
- base_pledge: minimum to participate (prevents spam)
- expected_reward: projected earnings over sector/commitment lifetime
- lock_multiplier: simple constant (e.g., 0.5× — lock half of expected earnings)
- NO quality-adjusted anything

## Token Distribution (Conceptual)
| Allocation | % | Notes |
|---|---|---|
| Mining rewards (storage + compute) | 60% | Released over time via minting function |
| Team + development | 15% | 4-year vesting, 1-year cliff |
| Ecosystem / grants | 10% | DAO-governed after year 1 |
| Early operators (genesis) | 10% | Incentive for bootstrapping |
| Public sale | 5% | Fair launch mechanism TBD |

No mining reserve. No foundation slush fund. All allocations transparent and on-chain.

## Comparison to Filecoin
| Mechanism | Filecoin | This chain |
|---|---|---|
| Quality multiplier | 10× for verified | None (1× always) |
| Minting function | Simple + baseline (dual) | TBD (simpler) |
| Pledge | Complex (consensus + storage + deal) | Simple (base + expected_reward) |
| Mining reserve | 15% of supply (300M FIL) | None |
| Compute rewards | None | First-class |
| Data class privilege | Fil+ / DataCap | None |
