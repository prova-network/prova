//! SDK-008: Marketplace Client SDK — provider search, bid placement, listing management.
//!
//! High-level async-style client (simulated) that wraps the chain marketplace
//! module. Provides builder patterns for discovery filters, automatic bid
//! management (place + match + poll), and provider comparison utilities.

use prova_chain::marketplace::{
    Bid, DiscoveryFilter, Listing, ListingId, MarketError, Marketplace, SortBy, TokenPrice,
};
use prova_chain::types::{Address, Epoch, ModelId, StakeAmount};
use std::collections::HashMap;

// ── Provider Info (client-side view) ─────────────────────────

/// A snapshot of a provider's listing, enriched with client-side scoring.
#[derive(Debug, Clone)]
pub struct ProviderInfo {
    pub listing_id: ListingId,
    pub provider: Address,
    pub model_id: ModelId,
    pub price_per_m_input: TokenPrice,
    pub price_per_m_output: TokenPrice,
    pub combined_price: TokenPrice,
    pub max_concurrency: u32,
    pub available_slots: u32,
    pub latency_sla_ms: u64,
    pub completed_inferences: u64,
    pub staked_amount: StakeAmount,
    pub arch_group: String,
    /// Client-computed score (higher = better). Weighted by price, latency, reputation.
    pub score: f64,
}

impl ProviderInfo {
    fn from_listing(listing: &Listing, weights: &ScoringWeights) -> Self {
        let combined = listing.price_per_m_input + listing.price_per_m_output;
        let available = listing.max_concurrency.saturating_sub(listing.active_requests);

        // Normalize and score: lower price better, lower latency better, more completions better
        let price_score = if combined > 0 { 1_000_000.0 / combined as f64 } else { 1_000_000.0 };
        let latency_score = if listing.latency_sla_ms > 0 {
            1000.0 / listing.latency_sla_ms as f64
        } else {
            1000.0
        };
        let reputation_score = (listing.completed_inferences as f64).ln().max(0.0) + 1.0;
        let stake_score = (listing.staked_amount as f64).ln().max(0.0);

        let score = weights.price * price_score
            + weights.latency * latency_score
            + weights.reputation * reputation_score
            + weights.stake * stake_score;

        Self {
            listing_id: listing.id,
            provider: listing.provider,
            model_id: listing.model_id,
            price_per_m_input: listing.price_per_m_input,
            price_per_m_output: listing.price_per_m_output,
            combined_price: combined,
            max_concurrency: listing.max_concurrency,
            available_slots: available,
            latency_sla_ms: listing.latency_sla_ms,
            completed_inferences: listing.completed_inferences,
            staked_amount: listing.staked_amount,
            arch_group: listing.arch_group.clone(),
            score,
        }
    }
}

/// Weights for client-side provider scoring.
#[derive(Debug, Clone)]
pub struct ScoringWeights {
    pub price: f64,
    pub latency: f64,
    pub reputation: f64,
    pub stake: f64,
}

impl Default for ScoringWeights {
    fn default() -> Self {
        Self {
            price: 0.4,
            latency: 0.3,
            reputation: 0.2,
            stake: 0.1,
        }
    }
}

// ── Discovery Builder ────────────────────────────────────────

/// Fluent builder for marketplace discovery queries.
#[derive(Debug, Clone)]
pub struct DiscoveryBuilder {
    model_id: ModelId,
    max_price_input: Option<TokenPrice>,
    max_price_output: Option<TokenPrice>,
    min_stake: Option<StakeAmount>,
    max_latency_ms: Option<u64>,
    arch_group: Option<String>,
    sort_by: SortBy,
    limit: usize,
    scoring: ScoringWeights,
}

impl DiscoveryBuilder {
    pub fn new(model_id: ModelId) -> Self {
        Self {
            model_id,
            max_price_input: None,
            max_price_output: None,
            min_stake: None,
            max_latency_ms: None,
            arch_group: None,
            sort_by: SortBy::PriceAsc,
            limit: 20,
            scoring: ScoringWeights::default(),
        }
    }

    pub fn max_price(mut self, input: TokenPrice, output: TokenPrice) -> Self {
        self.max_price_input = Some(input);
        self.max_price_output = Some(output);
        self
    }

    pub fn min_stake(mut self, stake: StakeAmount) -> Self {
        self.min_stake = Some(stake);
        self
    }

    pub fn max_latency_ms(mut self, ms: u64) -> Self {
        self.max_latency_ms = Some(ms);
        self
    }

    pub fn arch_group(mut self, group: &str) -> Self {
        self.arch_group = Some(group.to_string());
        self
    }

    pub fn sort_by(mut self, sort: SortBy) -> Self {
        self.sort_by = sort;
        self
    }

    pub fn limit(mut self, n: usize) -> Self {
        self.limit = n;
        self
    }

    pub fn scoring_weights(mut self, weights: ScoringWeights) -> Self {
        self.scoring = weights;
        self
    }

    fn to_filter(&self) -> DiscoveryFilter {
        DiscoveryFilter {
            model_id: self.model_id,
            max_price_input: self.max_price_input,
            max_price_output: self.max_price_output,
            min_stake: self.min_stake,
            max_latency_ms: self.max_latency_ms,
            arch_group: self.arch_group.clone(),
            sort_by: self.sort_by,
            limit: self.limit,
        }
    }
}

// ── Bid Tracker ──────────────────────────────────────────────

/// Status of a client's bid.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BidStatus {
    Pending,
    Matched(ListingId),
    Expired,
    Cancelled,
}

/// Tracks client-side bid state.
#[derive(Debug, Clone)]
pub struct BidTracker {
    pub bid_id: u64,
    pub model_id: ModelId,
    pub max_price_input: TokenPrice,
    pub max_price_output: TokenPrice,
    pub expires_at: Epoch,
    pub status: BidStatus,
}

// ── Marketplace Client ───────────────────────────────────────

/// High-level marketplace client for inference consumers.
///
/// Wraps the on-chain marketplace with convenience methods:
/// - Provider search with scoring
/// - Bid lifecycle management
/// - Cheapest-provider auto-selection
/// - Bid portfolio tracking
#[derive(Debug)]
pub struct MarketplaceClient {
    address: Address,
    active_bids: HashMap<u64, BidTracker>,
    scoring: ScoringWeights,
}

#[derive(Debug, PartialEq)]
pub enum ClientError {
    Chain(MarketError),
    NoBidsActive,
    BidNotTracked(u64),
    NoProvidersFound,
}

impl From<MarketError> for ClientError {
    fn from(e: MarketError) -> Self {
        ClientError::Chain(e)
    }
}

impl MarketplaceClient {
    /// Create a new marketplace client for the given address.
    pub fn new(address: Address) -> Self {
        Self {
            address,
            active_bids: HashMap::new(),
            scoring: ScoringWeights::default(),
        }
    }

    /// Set custom scoring weights for provider ranking.
    pub fn set_scoring(&mut self, weights: ScoringWeights) {
        self.scoring = weights;
    }

    /// Search for providers using a discovery builder.
    /// Returns providers enriched with client-side scoring, sorted by score descending.
    pub fn search_providers(
        &self,
        marketplace: &Marketplace,
        builder: &DiscoveryBuilder,
    ) -> Vec<ProviderInfo> {
        let filter = builder.to_filter();
        let listings = marketplace.discover(&filter);
        let weights = &builder.scoring;

        let mut providers: Vec<ProviderInfo> = listings
            .iter()
            .map(|l| ProviderInfo::from_listing(l, weights))
            .collect();

        // Re-sort by composite score (descending)
        providers.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));
        providers
    }

    /// Find the single best provider for a model (highest score).
    pub fn best_provider(
        &self,
        marketplace: &Marketplace,
        model_id: ModelId,
    ) -> Result<ProviderInfo, ClientError> {
        let builder = DiscoveryBuilder::new(model_id)
            .scoring_weights(self.scoring.clone())
            .limit(50);
        let results = self.search_providers(marketplace, &builder);
        results.into_iter().next().ok_or(ClientError::NoProvidersFound)
    }

    /// Place a bid and track it client-side.
    pub fn place_bid(
        &mut self,
        marketplace: &mut Marketplace,
        model_id: ModelId,
        max_price_input: TokenPrice,
        max_price_output: TokenPrice,
        expires_at: Epoch,
    ) -> u64 {
        let bid_id = marketplace.place_bid(
            self.address,
            model_id,
            max_price_input,
            max_price_output,
            expires_at,
        );

        self.active_bids.insert(
            bid_id,
            BidTracker {
                bid_id,
                model_id,
                max_price_input,
                max_price_output,
                expires_at,
                status: BidStatus::Pending,
            },
        );

        bid_id
    }

    /// Attempt to match a tracked bid. Updates local tracker on success.
    pub fn match_bid(
        &mut self,
        marketplace: &mut Marketplace,
        bid_id: u64,
    ) -> Result<ListingId, ClientError> {
        let tracker = self
            .active_bids
            .get_mut(&bid_id)
            .ok_or(ClientError::BidNotTracked(bid_id))?;

        match marketplace.match_bid(bid_id) {
            Ok(listing_id) => {
                tracker.status = BidStatus::Matched(listing_id);
                Ok(listing_id)
            }
            Err(MarketError::BidExpired(_)) => {
                tracker.status = BidStatus::Expired;
                Err(ClientError::Chain(MarketError::BidExpired(bid_id)))
            }
            Err(e) => Err(ClientError::Chain(e)),
        }
    }

    /// Place a bid and immediately try to match it (convenience).
    pub fn bid_and_match(
        &mut self,
        marketplace: &mut Marketplace,
        model_id: ModelId,
        max_price_input: TokenPrice,
        max_price_output: TokenPrice,
    ) -> Result<(u64, ListingId), ClientError> {
        let bid_id = self.place_bid(marketplace, model_id, max_price_input, max_price_output, 0);
        let listing_id = self.match_bid(marketplace, bid_id)?;
        Ok((bid_id, listing_id))
    }

    /// Get status of a tracked bid.
    pub fn bid_status(&self, bid_id: u64) -> Option<&BidStatus> {
        self.active_bids.get(&bid_id).map(|t| &t.status)
    }

    /// List all active (pending) bids.
    pub fn pending_bids(&self) -> Vec<&BidTracker> {
        self.active_bids
            .values()
            .filter(|t| t.status == BidStatus::Pending)
            .collect()
    }

    /// List all matched bids.
    pub fn matched_bids(&self) -> Vec<&BidTracker> {
        self.active_bids
            .values()
            .filter(|t| matches!(t.status, BidStatus::Matched(_)))
            .collect()
    }

    /// Cancel a pending bid (client-side only; chain doesn't support cancel yet).
    pub fn cancel_bid(&mut self, bid_id: u64) -> Result<(), ClientError> {
        let tracker = self
            .active_bids
            .get_mut(&bid_id)
            .ok_or(ClientError::BidNotTracked(bid_id))?;
        tracker.status = BidStatus::Cancelled;
        Ok(())
    }

    /// Compare two providers side by side.
    pub fn compare_providers(a: &ProviderInfo, b: &ProviderInfo) -> ProviderComparison {
        ProviderComparison {
            price_diff_pct: if a.combined_price > 0 {
                ((b.combined_price as f64 - a.combined_price as f64) / a.combined_price as f64) * 100.0
            } else {
                0.0
            },
            latency_diff_ms: b.latency_sla_ms as i64 - a.latency_sla_ms as i64,
            reputation_diff: b.completed_inferences as i64 - a.completed_inferences as i64,
            stake_diff: b.staked_amount as i64 - a.staked_amount as i64,
            score_diff: b.score - a.score,
            recommendation: if a.score >= b.score {
                Recommendation::PreferA
            } else {
                Recommendation::PreferB
            },
        }
    }

    /// Total number of tracked bids.
    pub fn total_bids(&self) -> usize {
        self.active_bids.len()
    }

    /// Client address.
    pub fn address(&self) -> Address {
        self.address
    }
}

/// Side-by-side provider comparison result.
#[derive(Debug, Clone)]
pub struct ProviderComparison {
    /// Positive = B more expensive, negative = B cheaper.
    pub price_diff_pct: f64,
    /// Positive = B higher latency, negative = B lower.
    pub latency_diff_ms: i64,
    /// Positive = B more completions.
    pub reputation_diff: i64,
    /// Positive = B more stake.
    pub stake_diff: i64,
    /// Positive = B higher score.
    pub score_diff: f64,
    pub recommendation: Recommendation,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Recommendation {
    PreferA,
    PreferB,
}

// ── Tests ────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn model(id: u8) -> ModelId {
        let mut h = [0u8; 32];
        h[0] = id;
        ModelId(h)
    }

    fn addr(id: u8) -> Address {
        Address::test(id)
    }

    fn setup_marketplace() -> Marketplace {
        let mut mp = Marketplace::new(100, 50);
        // Provider 1: cheap, high latency, low reputation
        mp.create_listing(addr(1), model(1), 50, 50, 10, 5000, 100, "sm90").unwrap();
        // Provider 2: mid-price, low latency, high reputation
        let lid2 = mp.create_listing(addr(2), model(1), 150, 150, 10, 3000, 20, "sm90").unwrap();
        // Simulate completed inferences for reputation
        for _ in 0..100 {
            let b = mp.place_bid(addr(99), model(1), 200, 200, 0);
            mp.match_bid(b).unwrap();
            mp.complete_inference(lid2).unwrap();
        }
        // Provider 3: expensive, lowest latency, medium reputation
        mp.create_listing(addr(3), model(1), 400, 400, 5, 8000, 10, "sm90").unwrap();
        mp
    }

    #[test]
    fn test_search_providers_returns_scored_results() {
        let mp = setup_marketplace();
        let client = MarketplaceClient::new(addr(10));
        let builder = DiscoveryBuilder::new(model(1));
        let results = client.search_providers(&mp, &builder);
        assert_eq!(results.len(), 3);
        // All should have positive scores
        for p in &results {
            assert!(p.score > 0.0);
        }
        // Results sorted by score descending
        assert!(results[0].score >= results[1].score);
        assert!(results[1].score >= results[2].score);
    }

    #[test]
    fn test_best_provider() {
        let mp = setup_marketplace();
        let client = MarketplaceClient::new(addr(10));
        let best = client.best_provider(&mp, model(1)).unwrap();
        assert!(best.score > 0.0);
    }

    #[test]
    fn test_best_provider_no_listings() {
        let mp = Marketplace::new(100, 50);
        let client = MarketplaceClient::new(addr(10));
        assert_eq!(
            client.best_provider(&mp, model(99)).unwrap_err(),
            ClientError::NoProvidersFound
        );
    }

    #[test]
    fn test_place_and_track_bid() {
        let mut mp = Marketplace::new(100, 50);
        mp.create_listing(addr(1), model(1), 100, 100, 5, 200, 50, "sm90").unwrap();

        let mut client = MarketplaceClient::new(addr(10));
        let bid_id = client.place_bid(&mut mp, model(1), 200, 200, 0);

        assert_eq!(client.total_bids(), 1);
        assert_eq!(*client.bid_status(bid_id).unwrap(), BidStatus::Pending);
        assert_eq!(client.pending_bids().len(), 1);
    }

    #[test]
    fn test_match_tracked_bid() {
        let mut mp = Marketplace::new(100, 50);
        let lid = mp.create_listing(addr(1), model(1), 100, 100, 5, 200, 50, "sm90").unwrap();

        let mut client = MarketplaceClient::new(addr(10));
        let bid_id = client.place_bid(&mut mp, model(1), 200, 200, 0);
        let matched = client.match_bid(&mut mp, bid_id).unwrap();

        assert_eq!(matched, lid);
        assert_eq!(*client.bid_status(bid_id).unwrap(), BidStatus::Matched(lid));
        assert_eq!(client.matched_bids().len(), 1);
        assert_eq!(client.pending_bids().len(), 0);
    }

    #[test]
    fn test_bid_and_match_convenience() {
        let mut mp = Marketplace::new(100, 50);
        let lid = mp.create_listing(addr(1), model(1), 100, 100, 5, 200, 50, "sm90").unwrap();

        let mut client = MarketplaceClient::new(addr(10));
        let (bid_id, listing_id) = client.bid_and_match(&mut mp, model(1), 200, 200).unwrap();

        assert_eq!(listing_id, lid);
        assert_eq!(*client.bid_status(bid_id).unwrap(), BidStatus::Matched(lid));
    }

    #[test]
    fn test_bid_and_match_no_providers() {
        let mut mp = Marketplace::new(100, 50);
        let mut client = MarketplaceClient::new(addr(10));
        let result = client.bid_and_match(&mut mp, model(1), 200, 200);
        assert!(matches!(result, Err(ClientError::Chain(MarketError::NoMatchingListings))));
    }

    #[test]
    fn test_cancel_bid() {
        let mut mp = Marketplace::new(100, 50);
        let mut client = MarketplaceClient::new(addr(10));
        let bid_id = client.place_bid(&mut mp, model(1), 200, 200, 0);
        client.cancel_bid(bid_id).unwrap();
        assert_eq!(*client.bid_status(bid_id).unwrap(), BidStatus::Cancelled);
        assert_eq!(client.pending_bids().len(), 0);
    }

    #[test]
    fn test_cancel_nonexistent_bid() {
        let mut client = MarketplaceClient::new(addr(10));
        assert_eq!(client.cancel_bid(999).unwrap_err(), ClientError::BidNotTracked(999));
    }

    #[test]
    fn test_match_expired_bid_updates_tracker() {
        let mut mp = Marketplace::new(100, 50);
        mp.create_listing(addr(1), model(1), 100, 100, 5, 200, 50, "sm90").unwrap();

        let mut client = MarketplaceClient::new(addr(10));
        let bid_id = client.place_bid(&mut mp, model(1), 200, 200, 5);
        mp.set_epoch(10);
        let result = client.match_bid(&mut mp, bid_id);
        assert!(result.is_err());
        assert_eq!(*client.bid_status(bid_id).unwrap(), BidStatus::Expired);
    }

    #[test]
    fn test_discovery_builder_filters() {
        let mp = setup_marketplace();
        let client = MarketplaceClient::new(addr(10));

        let builder = DiscoveryBuilder::new(model(1))
            .max_price(200, 200)
            .max_latency_ms(50)
            .min_stake(1000);

        let results = client.search_providers(&mp, &builder);
        // Only provider 2 passes: price 150/150 ≤ 200, latency 20 ≤ 50, stake 3000 ≥ 1000
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].provider, addr(2));
    }

    #[test]
    fn test_discovery_builder_arch_filter() {
        let mut mp = Marketplace::new(100, 50);
        mp.create_listing(addr(1), model(1), 100, 100, 5, 200, 50, "sm90").unwrap();
        mp.create_listing(addr(2), model(1), 100, 100, 5, 200, 50, "sm89").unwrap();

        let client = MarketplaceClient::new(addr(10));
        let builder = DiscoveryBuilder::new(model(1)).arch_group("sm90");
        let results = client.search_providers(&mp, &builder);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].provider, addr(1));
    }

    #[test]
    fn test_compare_providers() {
        let mp = setup_marketplace();
        let client = MarketplaceClient::new(addr(10));
        let builder = DiscoveryBuilder::new(model(1));
        let results = client.search_providers(&mp, &builder);

        let cmp = MarketplaceClient::compare_providers(&results[0], &results[1]);
        assert!(cmp.score_diff <= 0.0); // B score ≤ A score (A is best)
        assert_eq!(cmp.recommendation, Recommendation::PreferA);
    }

    #[test]
    fn test_custom_scoring_weights() {
        let mp = setup_marketplace();
        let mut client = MarketplaceClient::new(addr(10));

        // Weight heavily toward latency
        client.set_scoring(ScoringWeights {
            price: 0.0,
            latency: 1.0,
            reputation: 0.0,
            stake: 0.0,
        });

        let best = client.best_provider(&mp, model(1)).unwrap();
        // Provider 3 has lowest latency (10ms)
        assert_eq!(best.provider, addr(3));
    }

    #[test]
    fn test_provider_info_fields() {
        let mut mp = Marketplace::new(100, 50);
        mp.create_listing(addr(1), model(1), 100, 200, 8, 5000, 50, "sm90").unwrap();

        let client = MarketplaceClient::new(addr(10));
        let builder = DiscoveryBuilder::new(model(1));
        let results = client.search_providers(&mp, &builder);

        assert_eq!(results.len(), 1);
        let p = &results[0];
        assert_eq!(p.price_per_m_input, 100);
        assert_eq!(p.price_per_m_output, 200);
        assert_eq!(p.combined_price, 300);
        assert_eq!(p.max_concurrency, 8);
        assert_eq!(p.available_slots, 8);
        assert_eq!(p.latency_sla_ms, 50);
        assert_eq!(p.arch_group, "sm90");
    }

    #[test]
    fn test_multiple_bids_tracking() {
        let mut mp = Marketplace::new(100, 50);
        mp.create_listing(addr(1), model(1), 100, 100, 10, 200, 50, "sm90").unwrap();
        mp.create_listing(addr(2), model(2), 200, 200, 10, 200, 50, "sm90").unwrap();

        let mut client = MarketplaceClient::new(addr(10));
        let b1 = client.place_bid(&mut mp, model(1), 200, 200, 0);
        let b2 = client.place_bid(&mut mp, model(2), 300, 300, 0);
        let b3 = client.place_bid(&mut mp, model(1), 150, 150, 0);

        assert_eq!(client.total_bids(), 3);
        assert_eq!(client.pending_bids().len(), 3);

        client.match_bid(&mut mp, b1).unwrap();
        assert_eq!(client.pending_bids().len(), 2);
        assert_eq!(client.matched_bids().len(), 1);

        client.cancel_bid(b3).unwrap();
        assert_eq!(client.pending_bids().len(), 1);
    }

    #[test]
    fn test_client_address() {
        let client = MarketplaceClient::new(addr(42));
        assert_eq!(client.address(), addr(42));
    }
}
