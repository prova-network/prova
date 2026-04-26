# Graph Report - .  (2026-04-26)

## Corpus Check
- Large corpus: 298 files · ~558,303 words. Semantic extraction will be expensive (many Claude tokens). Consider running on a subfolder, or use --no-semantic to run AST-only.

## Summary
- 1661 nodes · 3078 edges · 57 communities detected
- Extraction: 73% EXTRACTED · 27% INFERRED · 0% AMBIGUOUS · INFERRED: 834 edges (avg confidence: 0.8)
- Token cost: 0 input · 0 output

## Community Hubs (Navigation)
- [[_COMMUNITY_Community 0|Community 0]]
- [[_COMMUNITY_Community 1|Community 1]]
- [[_COMMUNITY_Community 2|Community 2]]
- [[_COMMUNITY_Community 3|Community 3]]
- [[_COMMUNITY_Community 4|Community 4]]
- [[_COMMUNITY_Community 5|Community 5]]
- [[_COMMUNITY_Community 6|Community 6]]
- [[_COMMUNITY_Community 7|Community 7]]
- [[_COMMUNITY_Community 8|Community 8]]
- [[_COMMUNITY_Community 9|Community 9]]
- [[_COMMUNITY_Community 10|Community 10]]
- [[_COMMUNITY_Community 11|Community 11]]
- [[_COMMUNITY_Community 12|Community 12]]
- [[_COMMUNITY_Community 13|Community 13]]
- [[_COMMUNITY_Community 14|Community 14]]
- [[_COMMUNITY_Community 15|Community 15]]
- [[_COMMUNITY_Community 16|Community 16]]
- [[_COMMUNITY_Community 17|Community 17]]
- [[_COMMUNITY_Community 18|Community 18]]
- [[_COMMUNITY_Community 19|Community 19]]
- [[_COMMUNITY_Community 20|Community 20]]
- [[_COMMUNITY_Community 21|Community 21]]
- [[_COMMUNITY_Community 22|Community 22]]
- [[_COMMUNITY_Community 23|Community 23]]
- [[_COMMUNITY_Community 24|Community 24]]
- [[_COMMUNITY_Community 25|Community 25]]
- [[_COMMUNITY_Community 26|Community 26]]
- [[_COMMUNITY_Community 27|Community 27]]
- [[_COMMUNITY_Community 28|Community 28]]
- [[_COMMUNITY_Community 29|Community 29]]
- [[_COMMUNITY_Community 30|Community 30]]
- [[_COMMUNITY_Community 31|Community 31]]
- [[_COMMUNITY_Community 32|Community 32]]
- [[_COMMUNITY_Community 33|Community 33]]
- [[_COMMUNITY_Community 34|Community 34]]
- [[_COMMUNITY_Community 35|Community 35]]
- [[_COMMUNITY_Community 36|Community 36]]
- [[_COMMUNITY_Community 37|Community 37]]
- [[_COMMUNITY_Community 38|Community 38]]
- [[_COMMUNITY_Community 39|Community 39]]
- [[_COMMUNITY_Community 40|Community 40]]
- [[_COMMUNITY_Community 41|Community 41]]
- [[_COMMUNITY_Community 42|Community 42]]
- [[_COMMUNITY_Community 43|Community 43]]
- [[_COMMUNITY_Community 44|Community 44]]
- [[_COMMUNITY_Community 45|Community 45]]
- [[_COMMUNITY_Community 46|Community 46]]
- [[_COMMUNITY_Community 47|Community 47]]
- [[_COMMUNITY_Community 48|Community 48]]
- [[_COMMUNITY_Community 49|Community 49]]
- [[_COMMUNITY_Community 50|Community 50]]
- [[_COMMUNITY_Community 51|Community 51]]
- [[_COMMUNITY_Community 52|Community 52]]
- [[_COMMUNITY_Community 53|Community 53]]
- [[_COMMUNITY_Community 54|Community 54]]
- [[_COMMUNITY_Community 55|Community 55]]
- [[_COMMUNITY_Community 56|Community 56]]

## God Nodes (most connected - your core abstractions)
1. `New()` - 270 edges
2. `ProofVerifierSession` - 52 edges
3. `ProofVerifierFilterer` - 46 edges
4. `ProofVerifierCaller` - 37 edges
5. `ProofVerifierCallerSession` - 37 edges
6. `StorageMarketplaceSession` - 36 edges
7. `StorageMarketplaceFilterer` - 31 edges
8. `ProverStakingFilterer` - 28 edges
9. `ProverStakingSession` - 25 edges
10. `cmdStart()` - 22 edges

## Surprising Connections (you probably didn't know these)
- `run()` --calls--> `setup()`  [INFERRED]
  cli/src/index.mjs → desktop/main/provad.js
- `TestServer_ListenAndServe_GracefulShutdown()` --calls--> `sleep()`  [INFERRED]
  prover/pkg/metrics/metrics_test.go → website/upload/upload.js
- `TestListenAndServe_GracefulShutdown()` --calls--> `sleep()`  [INFERRED]
  prover/pkg/httpserver/server_test.go → website/upload/upload.js
- `putCmd()` --calls--> `Stat()`  [INFERRED]
  cli/src/cmd/put.mjs → prover/webui/src/components/Stat.tsx
- `run()` --calls--> `setupIpcMain()`  [INFERRED]
  cli/src/index.mjs → desktop/main/ipc.js

## Communities

### Community 0 - "Community 0"
Cohesion: 0.02
Nodes (14): ContentRegistryFilterer, New(), TestNew_RequiresEngine(), MockProofVerifierFilterer, ProofVerifierCaller, ProofVerifierFilterer, ProverRegistryCaller, ProverRegistryFilterer (+6 more)

### Community 1 - "Community 1"
Cohesion: 0.05
Nodes (58): commpCID(), NewStoreBackedBuilder(), TestStoreBackedBuilder_FullFlow(), Accepter, Deal, DealID, Engine, EngineOptions (+50 more)

### Community 2 - "Community 2"
Cohesion: 0.03
Nodes (6): ProverRegistryCallerSession, ProverRegistrySession, ProverRegistryTransactorSession, ProverStakingCallerSession, ProverStakingSession, ProverStakingTransactorSession

### Community 3 - "Community 3"
Cohesion: 0.03
Nodes (3): ProofVerifierCallerSession, ProofVerifierSession, ProofVerifierTransactorSession

### Community 4 - "Community 4"
Cohesion: 0.05
Nodes (55): ChainReader, ChainSnapshot, dealView, fakeChain, MetricsCollectorAdapter, MetricsReader, MetricsSummary, Options (+47 more)

### Community 5 - "Community 5"
Cohesion: 0.03
Nodes (42): bindProofVerifier(), CidsCid, DeployProofVerifier(), IPDPTypesPieceIdAndOffset, IPDPTypesProof, NewProofVerifier(), NewProofVerifierCaller(), NewProofVerifierFilterer() (+34 more)

### Community 6 - "Community 6"
Cohesion: 0.03
Nodes (26): bindContentRegistry(), ContentRegistry, ContentRegistryCaller, ContentRegistryCallerRaw, ContentRegistryCallerSession, ContentRegistryContent, ContentRegistryContentDealUpdated, ContentRegistryContentDealUpdatedIterator (+18 more)

### Community 7 - "Community 7"
Cohesion: 0.03
Nodes (31): bindStorageMarketplace(), CidsCid, DeployStorageMarketplace(), NewStorageMarketplaceCaller(), NewStorageMarketplaceFilterer(), NewStorageMarketplaceTransactor(), StorageMarketplace, StorageMarketplaceDeal (+23 more)

### Community 8 - "Community 8"
Cohesion: 0.04
Nodes (52): NewOnChainAccepter(), TestNewOnChainAccepter_Validation(), NewDealSink(), NewHTTPSink(), ChainName(), Dial(), EventPollerOptions, mockWaiter (+44 more)

### Community 9 - "Community 9"
Cohesion: 0.04
Nodes (29): bindProverStaking(), DeployProverStaking(), NewProverStaking(), NewProverStakingCaller(), NewProverStakingFilterer(), NewProverStakingTransactor(), ProverStaking, ProverStakingAuthorizedControllerSet (+21 more)

### Community 10 - "Community 10"
Cohesion: 0.04
Nodes (22): bindMockProofVerifier(), DeployMockProofVerifier(), MockProofVerifier, MockProofVerifierCaller, MockProofVerifierCallerRaw, MockProofVerifierCallerSession, MockProofVerifierDataSetCreated, MockProofVerifierDataSetCreatedIterator (+14 more)

### Community 11 - "Community 11"
Cohesion: 0.04
Nodes (3): StorageMarketplaceCallerSession, StorageMarketplaceSession, StorageMarketplaceTransactorSession

### Community 12 - "Community 12"
Cohesion: 0.05
Nodes (40): Activities, frame(), greatCircle(), latLonToVec(), pingLoop(), spawnPing(), spawnProofRing(), j() (+32 more)

### Community 13 - "Community 13"
Cohesion: 0.05
Nodes (23): bindProverRegistry(), DeployProverRegistry(), NewProverRegistryCaller(), NewProverRegistryFilterer(), NewProverRegistryTransactor(), ProverRegistry, ProverRegistryCallerRaw, ProverRegistryENSBound (+15 more)

### Community 14 - "Community 14"
Cohesion: 0.1
Nodes (31): $(), api(), boot(), clearToken(), escapeHtml(), formatSize(), getToken(), refreshAll() (+23 more)

### Community 15 - "Community 15"
Cohesion: 0.1
Nodes (26): api(), setupAppMenu(), setupCheckForUpdatesMenuItem(), setupIpcEventListeners(), authCmd(), clearConfig(), loadConfig(), requireToken() (+18 more)

### Community 16 - "Community 16"
Cohesion: 0.09
Nodes (29): fetchJSON(), onRequest(), serveR2(), Fetcher, FetcherOptions, sendMail(), sendViaPostmark(), sendViaResend() (+21 more)

### Community 17 - "Community 17"
Cohesion: 0.11
Nodes (24): ChallengeIndex(), ChallengeIndices(), GenerateProofs(), pad32Left(), TestChallengeIndex_BoundedByTotalLeaves(), TestChallengeIndex_Deterministic(), TestChallengeIndex_DifferentDataSetsDiffer(), TestChallengeIndex_DifferentProofIndicesDiffer() (+16 more)

### Community 18 - "Community 18"
Cohesion: 0.13
Nodes (12): ChainClient, OnChainClient, Runner, RunnerOptions, stubChainClient, NewRunner(), silentLog(), TestRunner_ProveSet_Happy() (+4 more)

### Community 19 - "Community 19"
Cohesion: 0.18
Nodes (18): base32LowerNoPad(), bytesToHex(), computePieceCid(), encodeFilCommP(), fr32Expand127(), fr32ExpandBits(), nextPow2(), selfTest() (+10 more)

### Community 20 - "Community 20"
Cohesion: 0.2
Nodes (18): fr32Pad(), BuildMemtree(), BuildMemtreeFromSnapshot(), computeTotalNodes(), leadingZeros64(), MemtreeProof(), nextPow2(), paddedSize() (+10 more)

### Community 21 - "Community 21"
Cohesion: 0.21
Nodes (13): SourceURLResolver, commpCIDString(), NewSourceURLResolver(), TestSourceURLResolver_ClientRawTemplate(), TestSourceURLResolver_CommpCidTemplate(), TestSourceURLResolver_Disabled(), TestSourceURLResolver_NilReceiver(), TestSourceURLResolver_TemplateSubstitution() (+5 more)

### Community 22 - "Community 22"
Cohesion: 0.15
Nodes (13): ChainConfig, Config, Contracts, DashboardConfig, HTTPConfig, IdentityConfig, Load(), MetricsConfig (+5 more)

### Community 23 - "Community 23"
Cohesion: 0.2
Nodes (6): showDialogSync(), beforeQuitCleanup(), onUpdateDownloaded(), onUpdateNotAvailable(), onUpdaterError(), quitAndInstall()

### Community 24 - "Community 24"
Cohesion: 0.44
Nodes (10): $(), accent(), bg(), ink(), inkSoft(), renderAll(), renderArchitecture(), renderClientFlows() (+2 more)

### Community 25 - "Community 25"
Cohesion: 0.22
Nodes (0): 

### Community 26 - "Community 26"
Cohesion: 0.48
Nodes (5): formatBytes(), formatDuration(), relativeTime(), shortAddr(), shortHash()

### Community 27 - "Community 27"
Cohesion: 0.4
Nodes (0): 

### Community 28 - "Community 28"
Cohesion: 0.4
Nodes (1): Collector

### Community 29 - "Community 29"
Cohesion: 0.5
Nodes (0): 

### Community 30 - "Community 30"
Cohesion: 0.67
Nodes (1): StatusBadge()

### Community 31 - "Community 31"
Cohesion: 0.67
Nodes (1): Logo()

### Community 32 - "Community 32"
Cohesion: 0.67
Nodes (0): 

### Community 33 - "Community 33"
Cohesion: 0.67
Nodes (0): 

### Community 34 - "Community 34"
Cohesion: 0.67
Nodes (0): 

### Community 35 - "Community 35"
Cohesion: 1.0
Nodes (0): 

### Community 36 - "Community 36"
Cohesion: 1.0
Nodes (0): 

### Community 37 - "Community 37"
Cohesion: 1.0
Nodes (0): 

### Community 38 - "Community 38"
Cohesion: 1.0
Nodes (0): 

### Community 39 - "Community 39"
Cohesion: 1.0
Nodes (0): 

### Community 40 - "Community 40"
Cohesion: 1.0
Nodes (0): 

### Community 41 - "Community 41"
Cohesion: 1.0
Nodes (1): Store

### Community 42 - "Community 42"
Cohesion: 1.0
Nodes (0): 

### Community 43 - "Community 43"
Cohesion: 1.0
Nodes (0): 

### Community 44 - "Community 44"
Cohesion: 1.0
Nodes (0): 

### Community 45 - "Community 45"
Cohesion: 1.0
Nodes (0): 

### Community 46 - "Community 46"
Cohesion: 1.0
Nodes (0): 

### Community 47 - "Community 47"
Cohesion: 1.0
Nodes (0): 

### Community 48 - "Community 48"
Cohesion: 1.0
Nodes (0): 

### Community 49 - "Community 49"
Cohesion: 1.0
Nodes (0): 

### Community 50 - "Community 50"
Cohesion: 1.0
Nodes (0): 

### Community 51 - "Community 51"
Cohesion: 1.0
Nodes (0): 

### Community 52 - "Community 52"
Cohesion: 1.0
Nodes (0): 

### Community 53 - "Community 53"
Cohesion: 1.0
Nodes (0): 

### Community 54 - "Community 54"
Cohesion: 1.0
Nodes (0): 

### Community 55 - "Community 55"
Cohesion: 1.0
Nodes (0): 

### Community 56 - "Community 56"
Cohesion: 1.0
Nodes (0): 

## Knowledge Gaps
- **104 isolated node(s):** `Options`, `Store`, `FetcherOptions`, `EventPollerOptions`, `MetricsSink` (+99 more)
  These have ≤1 connection - possible missing edges or undocumented components.
- **Thin community `Community 35`** (2 nodes): `onRequest()`, `signup.ts`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Community 36`** (2 nodes): `loadInitial()`, `App.tsx`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Community 37`** (2 nodes): `bridgeAvailable()`, `api.ts`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Community 38`** (2 nodes): `ui.js`, `setupIpcEventForwarding()`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Community 39`** (2 nodes): `getBuildVersion()`, `build-version.js`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Community 40`** (2 nodes): `settings.js`, `setup()`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Community 41`** (2 nodes): `store.go`, `Store`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Community 42`** (1 nodes): `tailwind.config.js`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Community 43`** (1 nodes): `playwright.config.js`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Community 44`** (1 nodes): `vite.config.js`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Community 45`** (1 nodes): `postcss.config.js`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Community 46`** (1 nodes): `main.tsx`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Community 47`** (1 nodes): `smoke.test.js`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Community 48`** (1 nodes): `app-launch.e2e.test.js`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Community 49`** (1 nodes): `typings.ts`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Community 50`** (1 nodes): `typings.ts`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Community 51`** (1 nodes): `utils.test.js`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Community 52`** (1 nodes): `tailwind.config.js`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Community 53`** (1 nodes): `vite.config.ts`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Community 54`** (1 nodes): `postcss.config.js`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Community 55`** (1 nodes): `main.tsx`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Community 56`** (1 nodes): `embed.go`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.

## Suggested Questions
_Questions this graph is uniquely positioned to answer:_

- **Why does `New()` connect `Community 0` to `Community 1`, `Community 4`, `Community 5`, `Community 6`, `Community 7`, `Community 8`, `Community 9`, `Community 10`, `Community 13`, `Community 14`, `Community 17`, `Community 18`, `Community 20`, `Community 21`?**
  _High betweenness centrality (0.503) - this node is a cross-community bridge._
- **Why does `cmdStart()` connect `Community 8` to `Community 0`, `Community 1`, `Community 4`, `Community 5`, `Community 21`?**
  _High betweenness centrality (0.106) - this node is a cross-community bridge._
- **Why does `DeployProofVerifier()` connect `Community 5` to `Community 0`?**
  _High betweenness centrality (0.060) - this node is a cross-community bridge._
- **Are the 269 inferred relationships involving `New()` (e.g. with `cmdStart()` and `weiToETH()`) actually correct?**
  _`New()` has 269 INFERRED edges - model-reasoned connections that need verification._
- **What connects `Options`, `Store`, `FetcherOptions` to the rest of the system?**
  _104 weakly-connected nodes found - possible documentation gaps or missing edges._
- **Should `Community 0` be split into smaller, more focused modules?**
  _Cohesion score 0.02 - nodes in this community are weakly interconnected._
- **Should `Community 1` be split into smaller, more focused modules?**
  _Cohesion score 0.05 - nodes in this community are weakly interconnected._