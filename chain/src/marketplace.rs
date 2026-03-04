//! CHAIN-027: Model Marketplace — listing, bidding, provider discovery
//!
//! Providers list models they serve with pricing. Clients discover providers
//! by model, compare prices, and place bids for inference slots. The marketplace
//! tracks active listings, handles bid matching, and enforces minimum stake
//! requirements for listing.

use std::collections::{BTreeMap, HashMap};
use crate::types::{Address, Epoch, Hash, ModelId, StakeAmount};

/// Unique listing identifier.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ListingId(pub u64);

/// Price per inference token (in smallest denomination).
pub type TokenPrice = u128;

/// A provider's listing for serving a model.
#[derive(Debug, Clone)]
pub struct Listing {
    pub id: ListingId,
    pub provider: Address,
    pub model_id: ModelId,
    /// Price per 1M input tokens.
    pub price_per_m_input: TokenPrice,
    /// Price per 1M output tokens.
    pub price_per_m_output: TokenPrice,
    /// Maximum concurrent requests this provider will serve.
    pub max_concurrency: u32,
    /// Current active requests against this listing.
    pub active_requests: u32,
    /// Minimum stake the provider has posted (verified against stake ledger).
    pub staked_amount: StakeAmount,
    /// Epoch when listed.
    pub listed_at: Epoch,
    /// Whether the listing is active.
    pub active: bool,
    /// Provider-declared latency SLA in milliseconds (p95).
    pub latency_sla_ms: u64,
    /// Cumulative completed inferences on this listing.
    pub completed_inferences: u64,
    /// Arch group this listing serves.
    pub arch_group: String,
}

/// A client bid for inference on a specific model.
#[derive(Debug, Clone)]
pub struct Bid {
    pub id: u64,
    pub client: Address,
    pub model_id: ModelId,
    /// Maximum price per 1M input tokens the client will pay.
    pub max_price_input: TokenPrice,
    /// Maximum price per 1M output tokens the client will pay.
    pub max_price_output: TokenPrice,
    /// Epoch when bid was placed.
    pub placed_at: Epoch,
    /// Epoch when bid expires (0 = no expiry).
    pub expires_at: Epoch,
    /// Whether this bid has been matched.
    pub matched: bool,
    /// If matched, which listing.
    pub matched_listing: Option<ListingId>,
}

/// Sort criteria for provider discovery.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SortBy {
    PriceAsc,
    PriceDesc,
    LatencyAsc,
    StakeDesc,
    CompletedDesc,
}

/// Discovery filter.
#[derive(Debug, Clone)]
pub struct DiscoveryFilter {
    pub model_id: ModelId,
    pub max_price_input: Option<TokenPrice>,
    pub max_price_output: Option<TokenPrice>,
    pub min_stake: Option<StakeAmount>,
    pub max_latency_ms: Option<u64>,
    pub arch_group: Option<String>,
    pub sort_by: SortBy,
    pub limit: usize,
}

/// The marketplace state.
#[derive(Debug)]
pub struct Marketplace {
    listings: HashMap<ListingId, Listing>,
    /// Model → listing IDs for fast lookup.
    model_index: HashMap<ModelId, Vec<ListingId>>,
    /// Provider → listing IDs.
    provider_index: HashMap<Address, Vec<ListingId>>,
    bids: HashMap<u64, Bid>,
    /// Model → bid IDs.
    bid_index: HashMap<ModelId, Vec<u64>>,
    next_listing_id: u64,
    next_bid_id: u64,
    /// Minimum stake required to create a listing.
    pub min_listing_stake: StakeAmount,
    /// Fee taken from matched bids (basis points, e.g. 50 = 0.5%).
    pub match_fee_bps: u32,
    /// Collected fees.
    pub collected_fees: u128,
    current_epoch: Epoch,
}

#[derive(Debug, PartialEq)]
pub enum MarketError {
    InsufficientStake { required: StakeAmount, actual: StakeAmount },
    ListingNotFound(ListingId),
    ListingNotActive(ListingId),
    NotListingOwner,
    BidNotFound(u64),
    BidExpired(u64),
    BidAlreadyMatched(u64),
    NoCapacity(ListingId),
    PriceTooHigh,
    NoMatchingListings,
}

impl Marketplace {
    pub fn new(min_listing_stake: StakeAmount, match_fee_bps: u32) -> Self {
        Self {
            listings: HashMap::new(),
            model_index: HashMap::new(),
            provider_index: HashMap::new(),
            bids: HashMap::new(),
            bid_index: HashMap::new(),
            next_listing_id: 1,
            next_bid_id: 1,
            min_listing_stake,
            match_fee_bps,
            collected_fees: 0,
            current_epoch: 0,
        }
    }

    pub fn set_epoch(&mut self, epoch: Epoch) {
        self.current_epoch = epoch;
    }

    /// Create a new listing. Provider must have sufficient stake.
    pub fn create_listing(
        &mut self,
        provider: Address,
        model_id: ModelId,
        price_per_m_input: TokenPrice,
        price_per_m_output: TokenPrice,
        max_concurrency: u32,
        staked_amount: StakeAmount,
        latency_sla_ms: u64,
        arch_group: &str,
    ) -> Result<ListingId, MarketError> {
        if staked_amount < self.min_listing_stake {
            return Err(MarketError::InsufficientStake {
                required: self.min_listing_stake,
                actual: staked_amount,
            });
        }

        let id = ListingId(self.next_listing_id);
        self.next_listing_id += 1;

        let listing = Listing {
            id,
            provider,
            model_id,
            price_per_m_input,
            price_per_m_output,
            max_concurrency,
            active_requests: 0,
            staked_amount,
            listed_at: self.current_epoch,
            active: true,
            latency_sla_ms,
            completed_inferences: 0,
            arch_group: arch_group.to_string(),
        };

        self.listings.insert(id, listing);
        self.model_index.entry(model_id).or_default().push(id);
        self.provider_index.entry(provider).or_default().push(id);

        Ok(id)
    }

    /// Deactivate a listing.
    pub fn deactivate_listing(
        &mut self,
        listing_id: ListingId,
        caller: Address,
    ) -> Result<(), MarketError> {
        let listing = self.listings.get_mut(&listing_id)
            .ok_or(MarketError::ListingNotFound(listing_id))?;
        if listing.provider != caller {
            return Err(MarketError::NotListingOwner);
        }
        listing.active = false;
        Ok(())
    }

    /// Update pricing on a listing.
    pub fn update_pricing(
        &mut self,
        listing_id: ListingId,
        caller: Address,
        price_per_m_input: TokenPrice,
        price_per_m_output: TokenPrice,
    ) -> Result<(), MarketError> {
        let listing = self.listings.get_mut(&listing_id)
            .ok_or(MarketError::ListingNotFound(listing_id))?;
        if listing.provider != caller {
            return Err(MarketError::NotListingOwner);
        }
        if !listing.active {
            return Err(MarketError::ListingNotActive(listing_id));
        }
        listing.price_per_m_input = price_per_m_input;
        listing.price_per_m_output = price_per_m_output;
        Ok(())
    }

    /// Place a bid for inference on a model.
    pub fn place_bid(
        &mut self,
        client: Address,
        model_id: ModelId,
        max_price_input: TokenPrice,
        max_price_output: TokenPrice,
        expires_at: Epoch,
    ) -> u64 {
        let id = self.next_bid_id;
        self.next_bid_id += 1;

        let bid = Bid {
            id,
            client,
            model_id,
            max_price_input,
            max_price_output,
            placed_at: self.current_epoch,
            expires_at,
            matched: false,
            matched_listing: None,
        };

        self.bids.insert(id, bid);
        self.bid_index.entry(model_id).or_default().push(id);
        id
    }

    /// Match a bid to the cheapest available listing.
    pub fn match_bid(&mut self, bid_id: u64) -> Result<ListingId, MarketError> {
        let bid = self.bids.get(&bid_id).ok_or(MarketError::BidNotFound(bid_id))?;
        if bid.matched {
            return Err(MarketError::BidAlreadyMatched(bid_id));
        }
        if bid.expires_at > 0 && bid.expires_at <= self.current_epoch {
            return Err(MarketError::BidExpired(bid_id));
        }

        let model_id = bid.model_id;
        let max_input = bid.max_price_input;
        let max_output = bid.max_price_output;

        // Find cheapest active listing with capacity
        let listing_ids = self.model_index.get(&model_id)
            .ok_or(MarketError::NoMatchingListings)?;

        let mut best: Option<ListingId> = None;
        let mut best_price = u128::MAX;

        for &lid in listing_ids {
            if let Some(l) = self.listings.get(&lid) {
                if !l.active || l.active_requests >= l.max_concurrency {
                    continue;
                }
                if l.price_per_m_input > max_input || l.price_per_m_output > max_output {
                    continue;
                }
                let combined = l.price_per_m_input + l.price_per_m_output;
                if combined < best_price {
                    best_price = combined;
                    best = Some(lid);
                }
            }
        }

        let listing_id = best.ok_or(MarketError::NoMatchingListings)?;

        // Apply fee
        let fee = best_price * self.match_fee_bps as u128 / 10_000;
        self.collected_fees += fee;

        // Update state
        let listing = self.listings.get_mut(&listing_id).unwrap();
        listing.active_requests += 1;

        let bid = self.bids.get_mut(&bid_id).unwrap();
        bid.matched = true;
        bid.matched_listing = Some(listing_id);

        Ok(listing_id)
    }

    /// Mark an inference as complete, freeing capacity.
    pub fn complete_inference(&mut self, listing_id: ListingId) -> Result<(), MarketError> {
        let listing = self.listings.get_mut(&listing_id)
            .ok_or(MarketError::ListingNotFound(listing_id))?;
        if listing.active_requests == 0 {
            return Ok(());
        }
        listing.active_requests -= 1;
        listing.completed_inferences += 1;
        Ok(())
    }

    /// Discover providers for a model with filtering and sorting.
    pub fn discover(&self, filter: &DiscoveryFilter) -> Vec<&Listing> {
        let listing_ids = match self.model_index.get(&filter.model_id) {
            Some(ids) => ids,
            None => return vec![],
        };

        let mut results: Vec<&Listing> = listing_ids.iter()
            .filter_map(|id| self.listings.get(id))
            .filter(|l| {
                if !l.active { return false; }
                if let Some(max) = filter.max_price_input {
                    if l.price_per_m_input > max { return false; }
                }
                if let Some(max) = filter.max_price_output {
                    if l.price_per_m_output > max { return false; }
                }
                if let Some(min) = filter.min_stake {
                    if l.staked_amount < min { return false; }
                }
                if let Some(max_lat) = filter.max_latency_ms {
                    if l.latency_sla_ms > max_lat { return false; }
                }
                if let Some(ref ag) = filter.arch_group {
                    if &l.arch_group != ag { return false; }
                }
                true
            })
            .collect();

        match filter.sort_by {
            SortBy::PriceAsc => results.sort_by_key(|l| l.price_per_m_input + l.price_per_m_output),
            SortBy::PriceDesc => results.sort_by_key(|l| std::cmp::Reverse(l.price_per_m_input + l.price_per_m_output)),
            SortBy::LatencyAsc => results.sort_by_key(|l| l.latency_sla_ms),
            SortBy::StakeDesc => results.sort_by_key(|l| std::cmp::Reverse(l.staked_amount)),
            SortBy::CompletedDesc => results.sort_by_key(|l| std::cmp::Reverse(l.completed_inferences)),
        }

        results.truncate(filter.limit);
        results
    }

    /// Get a listing by ID.
    pub fn get_listing(&self, id: ListingId) -> Option<&Listing> {
        self.listings.get(&id)
    }

    /// Get all listings for a provider.
    pub fn provider_listings(&self, provider: Address) -> Vec<&Listing> {
        self.provider_index.get(&provider)
            .map(|ids| ids.iter().filter_map(|id| self.listings.get(id)).collect())
            .unwrap_or_default()
    }

    /// Get all active listings count.
    pub fn active_listing_count(&self) -> usize {
        self.listings.values().filter(|l| l.active).count()
    }

    /// Get bid by ID.
    pub fn get_bid(&self, id: u64) -> Option<&Bid> {
        self.bids.get(&id)
    }

    /// Expire stale bids for a model. Returns count of expired bids.
    pub fn expire_bids(&mut self, model_id: ModelId) -> usize {
        let bid_ids: Vec<u64> = self.bid_index.get(&model_id)
            .map(|ids| ids.clone())
            .unwrap_or_default();

        let mut expired = 0;
        for bid_id in bid_ids {
            if let Some(bid) = self.bids.get(&bid_id) {
                if !bid.matched && bid.expires_at > 0 && bid.expires_at <= self.current_epoch {
                    self.bids.remove(&bid_id);
                    expired += 1;
                }
            }
        }
        expired
    }
}

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

    #[test]
    fn test_create_listing() {
        let mut mp = Marketplace::new(1000, 50);
        let lid = mp.create_listing(addr(1), model(1), 100, 200, 10, 2000, 50, "sm90").unwrap();
        assert_eq!(mp.active_listing_count(), 1);
        let l = mp.get_listing(lid).unwrap();
        assert_eq!(l.provider, addr(1));
        assert_eq!(l.price_per_m_input, 100);
    }

    #[test]
    fn test_insufficient_stake() {
        let mut mp = Marketplace::new(1000, 50);
        let err = mp.create_listing(addr(1), model(1), 100, 200, 10, 500, 50, "sm90").unwrap_err();
        assert_eq!(err, MarketError::InsufficientStake { required: 1000, actual: 500 });
    }

    #[test]
    fn test_deactivate_listing() {
        let mut mp = Marketplace::new(100, 50);
        let lid = mp.create_listing(addr(1), model(1), 100, 200, 10, 200, 50, "sm90").unwrap();
        mp.deactivate_listing(lid, addr(1)).unwrap();
        assert!(!mp.get_listing(lid).unwrap().active);
        assert_eq!(mp.active_listing_count(), 0);
    }

    #[test]
    fn test_deactivate_wrong_owner() {
        let mut mp = Marketplace::new(100, 50);
        let lid = mp.create_listing(addr(1), model(1), 100, 200, 10, 200, 50, "sm90").unwrap();
        assert_eq!(mp.deactivate_listing(lid, addr(2)).unwrap_err(), MarketError::NotListingOwner);
    }

    #[test]
    fn test_update_pricing() {
        let mut mp = Marketplace::new(100, 50);
        let lid = mp.create_listing(addr(1), model(1), 100, 200, 10, 200, 50, "sm90").unwrap();
        mp.update_pricing(lid, addr(1), 50, 75).unwrap();
        let l = mp.get_listing(lid).unwrap();
        assert_eq!(l.price_per_m_input, 50);
        assert_eq!(l.price_per_m_output, 75);
    }

    #[test]
    fn test_place_and_match_bid() {
        let mut mp = Marketplace::new(100, 50); // 0.5% fee
        let lid = mp.create_listing(addr(1), model(1), 100, 200, 5, 200, 50, "sm90").unwrap();
        let bid_id = mp.place_bid(addr(2), model(1), 150, 250, 0);
        let matched = mp.match_bid(bid_id).unwrap();
        assert_eq!(matched, lid);
        assert!(mp.get_bid(bid_id).unwrap().matched);
        assert_eq!(mp.get_listing(lid).unwrap().active_requests, 1);
        // Fee: (100+200) * 50/10000 = 1
        assert_eq!(mp.collected_fees, 1);
    }

    #[test]
    fn test_bid_price_filter() {
        let mut mp = Marketplace::new(100, 50);
        mp.create_listing(addr(1), model(1), 500, 500, 5, 200, 50, "sm90").unwrap();
        let bid_id = mp.place_bid(addr(2), model(1), 100, 100, 0);
        assert_eq!(mp.match_bid(bid_id).unwrap_err(), MarketError::NoMatchingListings);
    }

    #[test]
    fn test_bid_capacity_exhaustion() {
        let mut mp = Marketplace::new(100, 50);
        let lid = mp.create_listing(addr(1), model(1), 100, 100, 1, 200, 50, "sm90").unwrap();
        let b1 = mp.place_bid(addr(2), model(1), 200, 200, 0);
        mp.match_bid(b1).unwrap();
        let b2 = mp.place_bid(addr(3), model(1), 200, 200, 0);
        assert_eq!(mp.match_bid(b2).unwrap_err(), MarketError::NoMatchingListings);
        // Complete first, then second should match
        mp.complete_inference(lid).unwrap();
        let b3 = mp.place_bid(addr(3), model(1), 200, 200, 0);
        assert_eq!(mp.match_bid(b3).unwrap(), lid);
    }

    #[test]
    fn test_bid_expiry() {
        let mut mp = Marketplace::new(100, 50);
        mp.create_listing(addr(1), model(1), 100, 100, 5, 200, 50, "sm90").unwrap();
        let bid_id = mp.place_bid(addr(2), model(1), 200, 200, 10);
        mp.set_epoch(10);
        assert_eq!(mp.match_bid(bid_id).unwrap_err(), MarketError::BidExpired(bid_id));
    }

    #[test]
    fn test_expire_bids_cleanup() {
        let mut mp = Marketplace::new(100, 50);
        mp.place_bid(addr(1), model(1), 100, 100, 5);
        mp.place_bid(addr(2), model(1), 100, 100, 8);
        mp.place_bid(addr(3), model(1), 100, 100, 0); // no expiry
        mp.set_epoch(6);
        let expired = mp.expire_bids(model(1));
        assert_eq!(expired, 1); // only first bid expired
    }

    #[test]
    fn test_discovery_price_sort() {
        let mut mp = Marketplace::new(100, 50);
        mp.create_listing(addr(1), model(1), 300, 300, 5, 200, 50, "sm90").unwrap();
        mp.create_listing(addr(2), model(1), 100, 100, 5, 200, 30, "sm90").unwrap();
        mp.create_listing(addr(3), model(1), 200, 200, 5, 200, 40, "sm90").unwrap();

        let filter = DiscoveryFilter {
            model_id: model(1),
            max_price_input: None,
            max_price_output: None,
            min_stake: None,
            max_latency_ms: None,
            arch_group: None,
            sort_by: SortBy::PriceAsc,
            limit: 10,
        };
        let results = mp.discover(&filter);
        assert_eq!(results.len(), 3);
        assert_eq!(results[0].provider, addr(2)); // cheapest
        assert_eq!(results[2].provider, addr(1)); // most expensive
    }

    #[test]
    fn test_discovery_filters() {
        let mut mp = Marketplace::new(100, 50);
        mp.create_listing(addr(1), model(1), 100, 100, 5, 5000, 20, "sm90").unwrap();
        mp.create_listing(addr(2), model(1), 500, 500, 5, 200, 100, "sm89").unwrap();
        mp.create_listing(addr(3), model(1), 200, 200, 5, 3000, 40, "sm90").unwrap();

        let filter = DiscoveryFilter {
            model_id: model(1),
            max_price_input: Some(300),
            max_price_output: None,
            min_stake: Some(1000),
            max_latency_ms: Some(50),
            arch_group: Some("sm90".to_string()),
            sort_by: SortBy::StakeDesc,
            limit: 10,
        };
        let results = mp.discover(&filter);
        assert_eq!(results.len(), 2); // addr(1) and addr(3) pass all filters
        assert_eq!(results[0].provider, addr(1)); // highest stake first
        assert_eq!(results[1].provider, addr(3));
    }

    #[test]
    fn test_discovery_limit() {
        let mut mp = Marketplace::new(100, 50);
        for i in 0..10 {
            mp.create_listing(addr(i), model(1), 100 + i as u128, 100, 5, 200, 50, "sm90").unwrap();
        }
        let filter = DiscoveryFilter {
            model_id: model(1),
            max_price_input: None,
            max_price_output: None,
            min_stake: None,
            max_latency_ms: None,
            arch_group: None,
            sort_by: SortBy::PriceAsc,
            limit: 3,
        };
        assert_eq!(mp.discover(&filter).len(), 3);
    }

    #[test]
    fn test_provider_listings() {
        let mut mp = Marketplace::new(100, 50);
        mp.create_listing(addr(1), model(1), 100, 100, 5, 200, 50, "sm90").unwrap();
        mp.create_listing(addr(1), model(2), 200, 200, 5, 200, 50, "sm90").unwrap();
        mp.create_listing(addr(2), model(1), 100, 100, 5, 200, 50, "sm90").unwrap();
        assert_eq!(mp.provider_listings(addr(1)).len(), 2);
        assert_eq!(mp.provider_listings(addr(2)).len(), 1);
        assert_eq!(mp.provider_listings(addr(3)).len(), 0);
    }

    #[test]
    fn test_complete_inference_tracking() {
        let mut mp = Marketplace::new(100, 50);
        let lid = mp.create_listing(addr(1), model(1), 100, 100, 5, 200, 50, "sm90").unwrap();
        let b1 = mp.place_bid(addr(2), model(1), 200, 200, 0);
        mp.match_bid(b1).unwrap();
        mp.complete_inference(lid).unwrap();
        let l = mp.get_listing(lid).unwrap();
        assert_eq!(l.active_requests, 0);
        assert_eq!(l.completed_inferences, 1);
    }

    #[test]
    fn test_cheapest_listing_wins_match() {
        let mut mp = Marketplace::new(100, 50);
        mp.create_listing(addr(1), model(1), 500, 500, 5, 200, 50, "sm90").unwrap();
        let cheap = mp.create_listing(addr(2), model(1), 50, 50, 5, 200, 50, "sm90").unwrap();
        mp.create_listing(addr(3), model(1), 300, 300, 5, 200, 50, "sm90").unwrap();
        let bid = mp.place_bid(addr(4), model(1), 600, 600, 0);
        assert_eq!(mp.match_bid(bid).unwrap(), cheap);
    }

    #[test]
    fn test_double_match_rejected() {
        let mut mp = Marketplace::new(100, 50);
        mp.create_listing(addr(1), model(1), 100, 100, 5, 200, 50, "sm90").unwrap();
        let bid = mp.place_bid(addr(2), model(1), 200, 200, 0);
        mp.match_bid(bid).unwrap();
        assert_eq!(mp.match_bid(bid).unwrap_err(), MarketError::BidAlreadyMatched(bid));
    }
}
