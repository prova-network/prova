# Marketplace Specification

**Status:** Draft
**Authors:** Capri (autonomous build)
**Created:** 2026-03-04

## 1. Overview

The Prova Model Marketplace enables permissionless discovery of inference providers and price-matched job routing. Providers list models they serve with pricing and SLA guarantees, backed by staked collateral. Clients discover providers through filtered queries, place bids specifying maximum acceptable prices, and the marketplace matches bids to the cheapest qualifying provider.

## 2. Terminology

| Term | Definition |
|------|-----------|
| **Listing** | A provider's offer to serve inference for a specific model at stated prices |
| **Bid** | A client's request for inference with maximum price constraints |
| **Match** | Binding assignment of a bid to a listing; reserves one concurrency slot |
| **Discovery** | Filtered, sorted query over active listings |
| **Match Fee** | Protocol fee taken from each matched bid (basis points) |

## 3. Listing Lifecycle

### 3.1 Creation

A provider creates a listing by specifying:

- `model_id` — the registered model being served (must exist in Model Registry)
- `price_per_m_input` — price per 1M input tokens (smallest denomination)
- `price_per_m_output` — price per 1M output tokens
- `max_concurrency` — maximum simultaneous inference requests
- `staked_amount` — collateral posted (must meet `min_listing_stake`)
- `latency_sla_ms` — declared p95 latency in milliseconds
- `arch_group` — hardware architecture group (e.g., "sm90", "sm89")

The listing receives a unique `ListingId` and becomes immediately discoverable.

**Stake requirement:** `staked_amount >= min_listing_stake`. Listings with insufficient stake are rejected. Stake is verified against the Stake Ledger (CHAIN-002).

### 3.2 Updates

The listing owner may update pricing (`price_per_m_input`, `price_per_m_output`) on active listings. Only the original provider address may modify or deactivate a listing.

### 3.3 Deactivation

The owner may deactivate a listing at any time. Deactivated listings are excluded from discovery and bid matching but remain in state for historical queries. Active requests against a deactivated listing continue to completion.

## 4. Bid Lifecycle

### 4.1 Placement

A client places a bid specifying:

- `model_id` — requested model
- `max_price_input` — maximum acceptable input token price
- `max_price_output` — maximum acceptable output token price
- `expires_at` — epoch after which the bid expires (0 = no expiry)

Bids are indexed by `model_id` for efficient matching.

### 4.2 Matching

When `match_bid(bid_id)` is called, the marketplace:

1. Validates the bid is not already matched or expired
2. Finds all active listings for the bid's `model_id`
3. Filters to listings with available capacity (`active_requests < max_concurrency`)
4. Filters to listings within the bid's price constraints
5. Selects the cheapest listing (by `price_per_m_input + price_per_m_output`)
6. Deducts the match fee: `fee = combined_price * match_fee_bps / 10000`
7. Increments the listing's `active_requests` count
8. Marks the bid as matched with the assigned `ListingId`

### 4.3 Completion

After inference completes, `complete_inference(listing_id)` decrements `active_requests` and increments `completed_inferences` on the listing.

### 4.4 Expiry

Bids with `expires_at > 0` become unmatchable once `current_epoch >= expires_at`. The `expire_bids(model_id)` method garbage-collects expired unmatched bids.

## 5. Discovery

Clients discover providers via `DiscoveryFilter`:

| Filter | Type | Description |
|--------|------|-------------|
| `model_id` | `ModelId` | Required. Target model. |
| `max_price_input` | `Option<TokenPrice>` | Maximum input price |
| `max_price_output` | `Option<TokenPrice>` | Maximum output price |
| `min_stake` | `Option<StakeAmount>` | Minimum provider stake |
| `max_latency_ms` | `Option<u64>` | Maximum declared latency SLA |
| `arch_group` | `Option<String>` | Required hardware architecture |
| `sort_by` | `SortBy` | Ordering criterion |
| `limit` | `usize` | Maximum results returned |

**Sort options:** `PriceAsc`, `PriceDesc`, `LatencyAsc`, `StakeDesc`, `CompletedDesc`.

## 6. Client SDK

The SDK (`sdk/src/marketplace.rs`) provides:

### 6.1 Provider Scoring

Each listing is enriched with a composite score computed from configurable weights:

```
score = w_price × (1M / combined_price)
      + w_latency × (1000 / latency_ms)
      + w_reputation × (ln(completed) + 1)
      + w_stake × ln(staked_amount)
```

Default weights: price 0.4, latency 0.3, reputation 0.2, stake 0.1.

### 6.2 Bid Management

- `place_bid()` — place and track client-side
- `match_bid()` — attempt match, update local tracker
- `bid_and_match()` — convenience: place + match in one call
- `cancel_bid()` — client-side cancellation
- `pending_bids()` / `matched_bids()` — portfolio queries

### 6.3 Provider Comparison

`compare_providers(a, b)` returns a `ProviderComparison` with percentage diffs on price, latency, reputation, stake, and a recommendation.

## 7. Fee Structure

- **Match fee:** Configurable in basis points (e.g., 50 bps = 0.5%)
- **Collected from:** The matched combined price of each bid
- **Destination:** Protocol treasury (accumulated in `collected_fees`)

## 8. Indexing

Three indexes maintained for O(1) lookups:

- `model_index: ModelId → Vec<ListingId>` — discovery queries
- `provider_index: Address → Vec<ListingId>` — provider portfolio
- `bid_index: ModelId → Vec<BidId>` — bid matching and cleanup

## 9. Security Considerations

- **Stake enforcement:** Prevents Sybil listing spam; minimum stake set by governance
- **Owner-only mutations:** Only the listing provider can update pricing or deactivate
- **Double-match prevention:** Bids can only be matched once
- **Capacity limits:** `active_requests` bounded by `max_concurrency`
- **Expiry garbage collection:** Prevents unbounded bid accumulation

## 10. Future Extensions

- **Dutch auction:** Premium model slots allocated via descending-price auction (CHAIN-028)
- **SLA enforcement:** Automated slashing if declared latency SLA violated (integrates with CHAIN-014)
- **Reputation integration:** Completed inference count feeds into reputation system (CHAIN-015)
- **Cross-chain listings:** Bridge-aware listings for cross-chain inference routing (CHAIN-017)
