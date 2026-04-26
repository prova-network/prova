# 5.2 Governance

PROVA-weighted on-chain governance over a bounded set of protocol parameters, with multisig-held emergency pause. This section is **draft** because the on-chain governor contract has not yet been deployed; the parameter space and timelock policy are stable but the voting mechanism is TBD.

## 5.2.1 Scope

Governance MAY change:

| Parameter | Hard cap / floor | Timelock | Where |
| --- | --- | --- | --- |
| `protocolFeeBps` | ≤ 300 (3%) | 2 days | StorageMarketplace |
| `slashFraction` | ≤ 2500 (25%) | 2 days | StorageMarketplace |
| `slashPerFault` | governance-set | 2 days | StorageMarketplace |
| `MAX_PROOF_GAP` | ≥ 1 epoch | 2 days | StorageMarketplace |
| `minStakePerTiB` | governance-set | 2 days | ProverStaking |
| `usdFloorPerTiB` | governance-set | 2 days | ProverStaking |
| `unbondingPeriod` | ≤ 30 days | 2 days | ProverStaking |
| Prover registry admission rules | n/a | 2 days | ProverRegistry |
| `redundancyCap` | ≤ 16 | 2 days | ProverRewards |
| `qualityCutoffBps` | ≤ 5000 (50%) | 2 days | ProverRewards |
| `FeeRouter.mode` | n/a | 2 days | FeeRouter |
| `burnShareBps` | ≤ 10000 (100%) | 2 days | FeeRouter |
| `swapPoolFee` | n/a | 2 days | FeeRouter |
| `retrievabilityThreshold` | ≥ 9000 (90%) | 2 days | RetrievabilityRegistry |
| `retrievabilitySlashFraction` | ≤ 1000 (10%) | 2 days | RetrievabilityRegistry |
| `ProofVerifier` UUPS upgrade | n/a | **7 days** | ProofVerifier |
| `StorageMarketplace` UUPS upgrade | n/a | **7 days** | StorageMarketplace |
| `ProverStaking` UUPS upgrade | n/a | **7 days** | ProverStaking |
| Governance contract upgrade | n/a | **14 days** | Governor |

Governance MAY NOT:

- Mint additional PROVA (no mint authority exists; the token contract has no `mint` function after deploy)
- Redirect funds held by `FeeRouter`, `ProverRewards`, or `ProverStaking` outside the protocol's published mechanisms
- Override the slashing math on a per-prover basis (slashing is automatic from on-chain proof results)
- Reduce vested allocations recorded in `ProvaVesting`
- Change the total supply (1,000,000,000 PROVA, fixed at deploy)
- Reverse a successful slash event after the dispute window has closed

## 5.2.2 Voting

Prova uses an OpenZeppelin `Governor` (with `GovernorVotes`, `GovernorVotesQuorumFraction`, `GovernorTimelockControl`, and `GovernorSettings` modules) deployed at `Governor` and parameterized as follows for **mainnet**:

```solidity
votingDelay         = 1 days                  // proposal → voting window opens
votingPeriod        = 5 days                  // voting window length
proposalThreshold   = 100_000 * 10**18         // 100,000 PROVA to propose
quorumNumerator     = 4                        // 4% of total supply
parameterTimelock   = 2 days                   // for entries marked "2 days" above
upgradeTimelock     = 7 days                   // for contract upgrades
governanceTimelock  = 14 days                  // for changes to Governor itself
```

For **testnet (Base Sepolia)** we use shortened timing to enable iterative protocol testing:

```solidity
votingDelay         = 1 hour
votingPeriod        = 6 hours
parameterTimelock   = 1 hour
upgradeTimelock     = 12 hours
governanceTimelock  = 1 day
```

The shortened testnet schedule MUST NOT be used on any mainnet deployment.

### 5.2.2.1 One PROVA, one vote (with caveats)

One PROVA equals one vote, with these adjustments:

- **Bonded stake counts**: PROVA locked as prover stake (`ProverStaking.bondedOf(addr)`) DOES count toward voting power. This is intentional — provers are stakeholders.
- **Unbonding stake does NOT count**: PROVA in the unbonding queue is excluded for the duration of the unbonding period. This prevents a prover from queueing exit immediately after voting.
- **Vesting stake does NOT count**: PROVA in vesting contracts does not count until claimed and held in a self-custodial address. This forces vested holders to actively engage rather than passively dominate votes.
- **Treasury PROVA does NOT count**: the protocol treasury MUST NOT vote with its own holdings, even though it technically holds them.

We are evaluating quadratic-voting alternatives if it materially reduces whale capture; the decision will land before mainnet TGE. The quadratic-voting research is tracked in `governance/research/quadratic-voting.md` (when the file exists).

### 5.2.2.2 Voting mechanism

```solidity
function castVote(uint256 proposalId, uint8 support) external returns (uint256);
function castVoteWithReason(uint256 proposalId, uint8 support, string calldata reason) external returns (uint256);
function castVoteBySig(uint256 proposalId, uint8 support, uint8 v, bytes32 r, bytes32 s) external returns (uint256);
```

Support values: 0 = Against, 1 = For, 2 = Abstain. Abstentions count toward quorum but not toward the For/Against tally.

A proposal succeeds if For > Against AND total votes (For + Abstain) ≥ `quorum`. Otherwise it fails and cannot be re-proposed in the same form for `proposalCooldown` (proposed: 7 days).

## 5.2.3 Emergency pause

A 5-of-9 multisig (the **Guardian Multisig**) holds an emergency pause role. Pause halts:

- `proposeDeal` (no new deals)
- `dataSetCreated` (no new acceptances)
- `possessionProven` (proofs are accepted but payment is not released; queue clears on unpause)
- `faultDeal` (no new slashings)
- `markRetrievabilityFault` (no new retrievability slashes)

Pause does NOT halt:

- Refunds or claims on already-completed deals
- Token transfers (PROVA, USDC)
- Vesting claims from `ProvaVesting`
- Dispute closures already in flight
- `ProverRegistry.deregister` (a prover can always exit)

A pause MUST be unpaused within 30 days OR it becomes ineffective and a governance vote is required to extend (via the 7-day upgrade timelock). This prevents the multisig from indefinitely holding the network hostage.

### 5.2.3.1 Guardian Multisig composition

The 9 signers MUST include:

- 2 representatives from the founding team
- 3 independent technical signers (current/former Filecoin, Ethereum, or comparable protocol contributors)
- 2 representatives from active provers holding ≥ 1% of bonded stake (rotated annually)
- 2 community signers elected by token-weighted vote with a 6-month term

Signer addresses are published on-chain at deploy. A signer rotation requires a 5-of-9 multisig vote AND a governance proposal passing the parameter timelock.

### 5.2.3.2 Pause triggers (recommended, non-binding)

The Guardian Multisig SHOULD pause if any of:

- A provable on-chain accounting bug is discovered (e.g., slashing math returning incorrect values).
- A critical CVE is published in `provad` or in the Solidity contracts.
- A successful exploit drains > 5% of `FeeRouter` or `ProverStaking` balances.
- A coordinated retrievability outage affecting > 25% of bonded stake is detected and traced to a single root cause.

The Guardian Multisig SHOULD NOT pause for:

- Price volatility in PROVA, USDC, or ETH.
- Disputes about specific deals that have a defined dispute path.
- Disagreement about a governance vote outcome.
- Routine prover operational issues (single-prover slashing).

## 5.2.4 Proposal etiquette

A proposal SHOULD include:

1. The exact parameter change (function selector + ABI-encoded calldata) — the on-chain form is canonical.
2. A 1-paragraph rationale.
3. A link to a forum thread or PR with at least 7 days of discussion.
4. A risk analysis: what's the worst-case outcome if the change is wrong?
5. A rollback plan: how do we reverse this change if it goes badly?

Proposals that change governance parameters themselves (e.g. lowering quorum, shortening timelocks) MUST go through the `governanceTimelock` (14 days), not the parameter timelock (2 days).

A proposer MAY include a `simulation hash` — the keccak256 of a foundry `forge script --json` simulation output — as evidence that the proposal does what it claims. Voters SHOULD verify the simulation independently.

### 5.2.4.1 Proposal lifecycle

```
PENDING (votingDelay)
    ↓
ACTIVE (votingPeriod)
    ↓
SUCCEEDED ──→ QUEUED (timelock) ──→ EXECUTED
    ↓                                    
DEFEATED                               EXPIRED (if not executed within executionGracePeriod)
    ↓                                    
EXECUTED       
```

A `SUCCEEDED` proposal that is not queued within `queueGracePeriod` (proposed: 30 days) reverts to `EXPIRED` and cannot be executed. This prevents zombie proposals from being executed long after the political context has changed.

### 5.2.4.2 Cancellation

A proposer MAY cancel their own proposal at any time before it enters `EXECUTED` state. The Guardian Multisig MAY cancel any proposal up until it enters the timelock queue, but only by burning a non-trivial amount of PROVA (proposed: 1% of `proposalThreshold`, sent to `0x000…dead`) — this makes anti-vote censorship costly.

## 5.2.5 Source of truth

The deployed contracts are authoritative. When this spec and the on-chain values disagree, the on-chain values MUST be treated as correct. A quarterly governance report at `prova-network/prova/governance/` (when the directory exists) lists every active parameter and its current value.

Each parameter change emits an event:

```solidity
event ParameterChanged(
    address indexed contract_,
    bytes32 indexed param,
    bytes oldValue,
    bytes newValue,
    uint256 proposalId
);
```

These events are indexed by `proposalId` so any historical state can be reconstructed from the chain alone.

## 5.2.6 Open questions

- **Quadratic voting vs one-token-one-vote**: research ongoing. The decision will be made before mainnet TGE and documented in `governance/research/quadratic-voting.md`.
- **Delegation**: the OpenZeppelin Governor supports vote delegation (`delegate(address)`). We have NOT decided whether to expose this in the official UI; allowing delegation might centralize voting power around a small set of "delegate" addresses, which is the opposite of what governance is supposed to look like. Current leaning: support it on-chain (the contract supports it natively) but do not encourage it in official UIs.
- **Off-chain signaling**: a proposal MAY reference an off-chain signaling vote (Snapshot or similar) for sentiment-checking before the on-chain proposal. We do not bind these signaling votes to on-chain execution; they're informational.
- **Prover voting power**: bonded stake counts as voting power. This means a single large prover could dominate votes that affect their own profitability. The 4% quorum is the structural mitigation, but at low protocol-token-distribution stages this is fragile. A future amendment may add a "prover-stake-only" voting class for parameters that affect provers, separate from "all-stake" voting on protocol-wide changes.

This section will be promoted from Draft to Reliable when:

1. The Governor contract is deployed and verified on Base Sepolia.
2. At least 3 distinct proposal-vote-execute cycles have completed end-to-end on testnet.
3. The Guardian Multisig is composed and the signer addresses are published on-chain.
4. The voting-power exclusions for unbonding/vesting/treasury stake are validated by an external audit.
