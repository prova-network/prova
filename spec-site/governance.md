<div class="spec-meta">
  <span class="label">State</span> <span class="badge state-wip">Draft / WIP</span>
  <span class="label">Theory audit</span> <span class="badge audit-na">N/A</span>
  <span class="label">Last updated</span> 2026-04-26
</div>

# 5.2 Governance

PROVA-weighted on-chain governance over a bounded set of protocol parameters, with multisig-held emergency pause. This section is **draft** because the on-chain governor contract has not yet been deployed; the parameter space and timelock policy are stable but the voting mechanism is TBD.

## 5.2.1 Scope

Governance MAY change:

| Parameter | Hard cap | Timelock | Where |
| --- | --- | --- | --- |
| `protocolFeeBps` | 300 (3%) | 2 days | StorageMarketplace |
| `slashFraction` | 2500 (25%) | 2 days | StorageMarketplace |
| `slashPerFault` | governance-set | 2 days | StorageMarketplace |
| `MAX_PROOF_GAP` | governance-set | 2 days | StorageMarketplace |
| `minStakePerGiB` | governance-set | 2 days | ProverStaking |
| `unbondingPeriod` | 30 days | 2 days | ProverStaking |
| Prover registry admission rules | n/a | 2 days | ProverRegistry |
| `redundancyCap` | 16 | 2 days | ProverRewards |
| `qualityCutoffBps` | 5000 (50%) | 2 days | ProverRewards |
| `FeeRouter.mode` | n/a | 2 days | FeeRouter |
| `burnShareBps` | 10000 | 2 days | FeeRouter |
| `swapPoolFee` | n/a | 2 days | FeeRouter |
| `ProofVerifier` UUPS upgrade | n/a | 7 days | ProofVerifier |

Governance MAY NOT:

- Mint additional PROVA (no mint authority exists)
- Redirect funds held by `FeeRouter`, `ProverRewards`, or `ProverStaking` outside the protocol's published mechanisms
- Override the slashing math on a per-prover basis
- Reduce vested allocations recorded in `ProvaVesting`

## 5.2.2 Voting (proposed)

A future amendment will deploy an OpenZeppelin Governor with these parameters:

```solidity
// Proposed values, subject to change before mainnet
votingDelay      = 1 day                  // proposal → voting window
votingPeriod     = 5 days                 // voting window length
proposalThreshold = 100_000 ether          // 100,000 PROVA to propose
quorumNumerator  = 4                      // 4% of total supply
timelockDelay    = 2 days                 // for parameter changes
upgradeTimelock  = 7 days                 // for contract upgrades
```

One PROVA is one vote. We are evaluating quadratic-voting alternatives if it materially reduces whale capture; the decision will land before TGE.

## 5.2.3 Emergency pause

A 5-of-9 multisig holds an emergency pause role. Pause halts:

- `proposeDeal` (no new deals)
- `dataSetCreated` (no new acceptances)
- `possessionProven` (proofs are accepted but payment is not released; queue clears on unpause)
- `faultDeal` (no new slashings)

Pause does NOT halt:

- Refunds or claims on already-completed deals
- Token transfers
- Vesting claims

A pause MUST be unpaused within 30 days or it becomes ineffective and a governance vote is required to extend.

## 5.2.4 Proposal etiquette

A proposal SHOULD include:

1. The exact parameter change (function selector + ABI-encoded calldata).
2. A 1-paragraph rationale.
3. A link to a forum thread or PR with at least 7 days of discussion.
4. A risk analysis: what's the worst-case outcome if the change is wrong?

Proposals that change governance parameters themselves (e.g. lowering quorum) MUST go through the 7-day upgrade timelock, not the 2-day parameter timelock.

## 5.2.5 Source of truth

The deployed contracts are authoritative. When this spec and the on-chain values disagree, the on-chain values MUST be treated as correct. We will publish a quarterly governance report in [`prova-network/prova/governance/`](https://github.com/prova-network/prova/tree/main/governance) (when the directory exists) listing every active parameter and its current value.
