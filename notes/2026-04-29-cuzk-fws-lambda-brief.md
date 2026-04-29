# cuZK + fws-lambda: how this fits into Prova

_Written 2026-04-29. Source material: ZenGround0/fws-lambda (hackathon prototype),
Tao-Lu-123/cuZK (paper repo), filecoin-project/curio/lib/proofsvc (production
SNARK marketplace), ZenGround0's "fun ideas" thread shared by Nicklas._

## TL;DR

fws-lambda is the missing piece that turns Prova from "storage marketplace"
into "verifiable compute platform." cuZK is the GPU library that makes the
economics work. Together they're a credible v3 thesis. v2 (storage) still
ships first; this is the next chapter, not a pivot.

---

## What each thing actually is

### fws-lambda (ZenGround0)

A 4-crate Rust workspace + Solidity contracts. Hackathon-grade, but the
shape is right.

- **`JobRegistry.sol`** (FEVM contract). Two functions: `postJob(inputCommp,
  wasmCommp, witnesses, bounty=msg.value)` and `submitProof(jobId, seal,
  journal, outputWitness)`. Job has Open / Completed states. Bounty held
  in escrow, paid to the worker on valid proof.
- **PDP coupling.** Every input/output/wasm blob must be a live PDP piece.
  The contract calls `IPDPVerifier.pieceLive()` and
  `getPieceCid()` and compares the on-chain digest to the CommP the
  client claimed. Data availability is enforced at the contract level,
  not assumed.
- **risc0 zkVM** runs WASM via wasmi inside a guest binary. The journal
  is exactly `inputCommp || wasmCommp || outputCommp` (96 bytes). The
  final receipt is Groth16-over-BN254 so EVM precompiles can verify it.
- **Performance ceiling per the README:** ~10 minutes to prove ~10KB of
  WASM + input on a 32-core CPU. "GPU integration would 10x this."

This is not production code. The contract has zero protocol fee, no slashing,
no expiry, no proverregistry, no quality scoring. But the integration shape
between PDP + SNARK verifier + bounty escrow is correct and matches Prova's
existing surface almost line-for-line.

### cuZK (Lu et al., TCHES 2023)

A research-grade CUDA library implementing the two hot loops in Groth16
proving:

- **MSM** (multi-scalar multiplication): claimed 2.08–2.94× faster than
  bellperson's GPU MSM on BLS12-381.
- **NTT** (parallel polynomial transforms).
- **End-to-end Groth16 proving:** 2.18× speedup on Filecoin's exact
  workload (their primary benchmark target).

Despite the name, cuZK is *not* about zero-knowledge — it's a faster
prover for the same Groth16 SNARK that risc0 already compresses to. So the
naming is misleading: it's "CUDA-accelerated SNARK," not "CUDA ZK."

Two GitHub mirrors exist (Tao-Lu-123 and speakspeak); content is identical.
Apache-2.0 OR MIT licensed. Targets BLS12-381 (Filecoin/Prova) and BN254
(Ethereum verifier curve). Both curves matter for fws-lambda.

### Curio's `lib/proofsvc` (the production prior art)

Already running, already paying out FIL on calibnet/mainnet. Worth knowing
about because it proves the *market* works, not just the cryptography.

- Provider API: register an actor ID, set a min price, get matched to jobs
  cheapest-first.
- Vouchers signed off-chain, redeemed on-chain at a hub contract.
- `Provider TOS v0.1 (Curio Storage Inc., 23 Jun 2025)` — Delaware-governed,
  explicit warning that operation may transfer to a DAO or successor
  operator.
- Live tasks: PoRep, WdPost, WinPost, TreeRC, UpdateProve. None of them are
  general-purpose compute; they're all Filecoin-internal SNARKs.
- The `proofshare` task pool already implements priority/conflict resolution
  so that PSProve doesn't starve the SP's own deadline-bound proofs.

This is the model fws-lambda generalizes from. Curio sells the SP's spare
GPU cycles to other Filecoin SPs; fws-lambda would sell them to anyone with
an arbitrary WASM program.

---

## How this maps onto Prova v2 contracts

The fws-lambda contract surface is small and slots into Prova almost
unchanged:

| fws-lambda concept       | Prova v2 equivalent (existing or +new) |
| ------------------------ | -------------------------------------- |
| `JobRegistry`            | **+new** `ComputeMarketplace.sol`, parallel to `StorageMarketplace.sol` |
| Input/output PDP pieces  | `ContentRegistry` + `ProofVerifier` (no change) |
| Worker = SP              | `ProverRegistry` (add a feature bit `COMPUTE_GPU = 4`) |
| Bounty escrow in FIL     | USDC escrow via the same payment plumbing as deals |
| risc0 verifier contract  | **+new** thin verifier wrapper (or import risc0's; both work) |
| Worker stake             | `ProverStaking` (no change, one stake covers storage + compute) |
| Slashing on bad proof    | `ProverStaking.slash()` already exists; add a controller for the compute marketplace |
| Quality / reputation     | `ProverRewards` already tracks success/failure ratios per prover |
| 1% protocol fee → burn   | `FeeRouter` (no change) |

That's roughly **one new contract + one new feature bit + a new authorized
controller**. Nothing else needs to move.

Schema impact on the indexer is small: add `prova_compute_job_*` event
tables paralleling the seven `prova_deal_*` tables. Maybe ~6 new tables.

---

## Why cuZK matters specifically

The fws-lambda README is honest: 10 minutes to prove a 10KB transformation
on a CPU. At that speed, the only economically viable jobs are batch
analytics where latency doesn't matter. With cuZK on a Blackwell-class GPU,
the proving time drops to single-digit seconds for the same workload (rough
estimate: 10× from GPU + cuZK's claimed 2× over baseline GPU = 20× cumulative,
so 10 min → 30 s). At that speed, **request-response APIs become viable**.

Once request-response is viable, the AWS Lambda compatibility tier of
ZenGround0's "fun ideas" (TS/JS/Go via wasmtime + a thin shim) becomes
worth doing because real workloads can migrate.

Without cuZK (or an equivalent), the whole compute-marketplace pitch caps
out at "verifiable batch jobs," which is a much smaller addressable market.

---

## Honest assessment of the four-tier ladder

ZenGround0's note proposes 4 tiers. Let me grade each.

### Tier 1: verifiable lambda for smart-contract devs ✅ realistic

This is what fws-lambda already is. Adoption is gated on (a) proving cost
dropping, (b) somebody building a half-decent SDK, (c) a couple of
flagship integrations (rollup off-chain compute, merkle-tree translation,
ZK-bridge attestations). The market exists. RISC0/SP1/Jolt are chasing
the same niche.

### Tier 2: AWS Lambda API surface compatibility 🟡 ambitious

The Lambda *invoke* API is one HTTP call. The Lambda *platform* is years
of work: IAM, VPCs, SDK auth, environment variables, layers, dead-letter
queues, CloudWatch, X-Ray, regional placement, cold-start tier, provisioned
concurrency, container images. Replicating just enough of it to be useful
is a 10-engineer-quarter effort. Replicating enough that AWS workloads
**move** to it is a 50-engineer-quarter effort.

That said: the *runtime contract* (zip of TS/JS/Go that exposes a
`handler(event, context)` function) is small and could be shimmed in 3-6
months if someone scopes ruthlessly. Skip everything except handler-ABI
+ HTTP trigger + S3-compat object reads.

### Tier 3: distributed webserver / static-hosting API layer 🟡 thesis-dependent

Static hosting is a commodity (Vercel, Netlify, Cloudflare Pages, ~$0
cost). A distributed-PDP-backed alternative only wins if it competes on
**a** trust-minimization (verifiable origin = "this response was generated
by code X over data Y, both signed and stored") or **b** censorship
resistance. Neither is a mass-market driver today; both are niche.

The lack-of-distributed-DB problem ZenGround0 mentions is real and unsolved.
PDP gives you blob storage, not queryable state. You'd need a verifiable KV
or a CRDT layer on top (Iroh, Earthstar, Hypercore-class), and none of those
are production-grade for paid workloads.

### Tier 4: "anti-Filecoin chain" 🔴 separate startup

WebRTC P2P + Avalanche consensus + snark-protected smart contracts +
provers-only state retention is a coherent thesis but it's a different
company. It would:

- Not reuse Prova's contracts (different L1).
- Not reuse Base's tooling.
- Need a new fundraise, new team, new tokenomics.

If Andy/the Curio crew are seriously thinking about it, the alignment with
Prova is "Prova provides the storage/compute primitives that this new chain
consumes," not "Prova becomes that chain."

---

## What this means for Prova's roadmap

The way I'd sequence it, given the Filecoin sunset signals already in
play:

1. **Ship Prova v2 storage** as planned. Don't add compute to v2.
2. **Forward-compat the schema:** reserve feature bit 4 for `COMPUTE_GPU`
   in `ProverRegistry`, add a comment in the spec that it's pre-allocated.
   Costs nothing now, prevents a migration later.
3. **Track cuZK as a prover-side tool**, not a Prova-maintained library.
   Document it in the prover ops guide as the recommended GPU stack
   alongside supranational/sppark.
4. **Watch fws-lambda as a v3 design source**, not a fork target. The
   contract shape is right; the implementation is hackathon-grade and
   would need a full rewrite with proper protocol fees, slashing,
   expirations, and a quality oracle before it's marketplace-ready.
5. **Don't commit to AWS Lambda compatibility publicly** until somebody
   has scoped the runtime-contract subset and proven proving costs at
   single-digit seconds. Otherwise it's a credibility tax.

## What's worth doing this week / this month

- Reach out to Andy directly about whether Prova should aim to be the
  compute-market settlement layer, or whether Curio's `proofsvc` is
  where that lives. Right now those two systems would compete for the
  same prover GPU time. We need clarity on division of labor before we
  both invest in the same primitive.
- Write a 2-page Prova spec amendment: `feat: reserve compute feature
  bit and define ComputeMarketplace contract surface (forward-only, not
  yet implemented)`. Lands in prova-network/prova as a tracked issue
  that can be picked up post-v2.
- Read the cuZK paper end-to-end (eprint 2022/1321) before any public
  commentary, so we don't repeat the README's marketing numbers
  uncritically.

## Risks and competitors to keep on the radar

- **RISC0 / SP1 / Jolt / Powdr.** All chasing the same general-purpose
  zkVM market. RISC0 already has Bonsai (their hosted prover network)
  and they're commercial. Prova competing with them on proving is wrong;
  Prova partnering (using their zkVM, providing storage + settlement) is
  right.
- **Akash.** Compute markets without proofs. Cheaper, more workloads.
  Their thesis: trust hardware attestation + reputation, not SNARKs. The
  honest truth is most compute buyers don't actually want ZK; they want
  cheap GPU. Prova's compute pitch only wins on the "verifiable" axis,
  which is a real but narrow market.
- **Aztec / Aleo / Ten / Fhenix.** Privacy-preserving compute chains.
  Different threat model (hide inputs) but they overlap on the
  "verifiable execution" pitch. None of them have a storage primitive,
  so PDP could plug in there too.
- **The Anti-Filecoin chain (tier 4)** is a separate fundraise. Don't let
  it crowd out v2 attention.

## Personal note (for me, future-Capri)

This is the kind of thesis Nicklas thrives on: a real engineering problem
(SNARK proving cost), a credible existing system (Curio proofsvc), and a
forward-compatible architecture (Prova v2 already has 90% of the contracts
needed). The temptation will be to start writing the ComputeMarketplace
contract immediately. Resist. v2 storage isn't shipped yet. The discipline
is: scope the spec amendment, file the feature bit, document the
roadmap, **then** go back to the M2 Curio invoice and the v2 ABIs.
