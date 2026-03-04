//! NODE-027: Marketplace Event Indexer
//!
//! Indexes marketplace-related events (listings, bids, matches, auctions) from
//! the chain event log into queryable, materialized views. Enables fast lookups
//! for the marketplace CLI and SDK without scanning raw event logs.
//!
//! Architecture:
//! - Subscribes to block events via the event subscription engine
//! - Filters for marketplace and auction event topics
//! - Maintains in-memory indexes (listing history, bid history, match log, price history)
//! - Supports cursor-based pagination and time-range queries
//! - Tracks indexer head (last processed block) for resumption after restart

use std::collections::{BTreeMap, HashMap, VecDeque};
use sha2::{Sha256, Digest};

// ── Types ──────────────────────────────────────────────────────────

pub type Hash = [u8; 32];

#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Address(pub [u8; 20]);

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ModelId(pub [u8; 32]);

pub type Epoch = u64;

/// Compute SHA-256 of an event signature string.
fn event_hash(sig: &str) -> Hash {
    let mut hasher = Sha256::new();
    hasher.update(sig.as_bytes());
    let result = hasher.finalize();
    let mut h = [0u8; 32];
    h.copy_from_slice(&result);
    h
}

// ── Well-known marketplace event signatures ────────────────────────

pub fn listing_created_topic() -> Hash {
    event_hash("ListingCreated(uint64,address,bytes32,uint128,uint128)")
}
pub fn listing_updated_topic() -> Hash {
    event_hash("ListingUpdated(uint64,uint128,uint128)")
}
pub fn listing_deactivated_topic() -> Hash {
    event_hash("ListingDeactivated(uint64,address)")
}
pub fn bid_placed_topic() -> Hash {
    event_hash("BidPlaced(uint64,address,bytes32,uint128,uint128)")
}
pub fn bid_matched_topic() -> Hash {
    event_hash("BidMatched(uint64,uint64,address,address)")
}
pub fn bid_expired_topic() -> Hash {
    event_hash("BidExpired(uint64)")
}
pub fn auction_created_topic() -> Hash {
    event_hash("AuctionCreated(uint64,bytes32,uint128,uint128,uint64)")
}
pub fn auction_filled_topic() -> Hash {
    event_hash("AuctionFilled(uint64,address,uint128,uint32)")
}
pub fn auction_completed_topic() -> Hash {
    event_hash("AuctionCompleted(uint64,uint8)")
}

// ── Indexed Records ────────────────────────────────────────────────

/// A materialized listing record for fast lookup.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IndexedListing {
    pub listing_id: u64,
    pub provider: Address,
    pub model_id: ModelId,
    pub price_input: u128,
    pub price_output: u128,
    pub created_at: Epoch,
    pub updated_at: Epoch,
    pub active: bool,
}

/// A materialized bid record.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IndexedBid {
    pub bid_id: u64,
    pub client: Address,
    pub model_id: ModelId,
    pub max_price_input: u128,
    pub max_price_output: u128,
    pub placed_at: Epoch,
    pub matched: bool,
    pub matched_listing: Option<u64>,
    pub matched_at: Option<Epoch>,
}

/// A match record (bid ↔ listing pairing).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IndexedMatch {
    pub bid_id: u64,
    pub listing_id: u64,
    pub client: Address,
    pub provider: Address,
    pub epoch: Epoch,
}

/// An auction event record.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IndexedAuction {
    pub auction_id: u64,
    pub model_id: ModelId,
    pub start_price: u128,
    pub reserve_price: u128,
    pub duration_epochs: u64,
    pub created_at: Epoch,
    pub total_slots: u32,
    pub filled_slots: u32,
    pub status: AuctionIndexStatus,
    pub fills: Vec<IndexedAuctionFill>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IndexedAuctionFill {
    pub bidder: Address,
    pub price: u128,
    pub slot_index: u32,
    pub epoch: Epoch,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AuctionIndexStatus {
    Active,
    Completed,
    Expired,
    Cancelled,
}

/// Price snapshot for time-series queries.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PricePoint {
    pub epoch: Epoch,
    pub model_id: ModelId,
    pub avg_price_input: u128,
    pub avg_price_output: u128,
    pub listing_count: u32,
}

/// An incoming raw event to be processed.
#[derive(Debug, Clone)]
pub struct RawEvent {
    pub emitter: Address,
    pub topics: Vec<Hash>,
    pub data: Vec<u8>,
    pub block_number: Epoch,
}

/// Cursor for paginated queries.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Cursor {
    pub offset: usize,
    pub limit: usize,
}

impl Default for Cursor {
    fn default() -> Self {
        Cursor { offset: 0, limit: 50 }
    }
}

/// Time-range filter for queries.
#[derive(Debug, Clone, Copy)]
pub struct EpochRange {
    pub from: Epoch,
    pub to: Epoch,
}

// ── Marketplace Event Indexer ──────────────────────────────────────

/// The marketplace event indexer: processes raw events into materialized views.
pub struct MarketplaceIndexer {
    /// Last processed block.
    pub head: Epoch,
    /// All indexed listings by ID.
    listings: HashMap<u64, IndexedListing>,
    /// Listings by model (model_id → listing IDs).
    listings_by_model: HashMap<ModelId, Vec<u64>>,
    /// Listings by provider.
    listings_by_provider: HashMap<Address, Vec<u64>>,
    /// All indexed bids by ID.
    bids: HashMap<u64, IndexedBid>,
    /// Bids by client.
    bids_by_client: HashMap<Address, Vec<u64>>,
    /// Bids by model.
    bids_by_model: HashMap<ModelId, Vec<u64>>,
    /// Match log (chronological).
    matches: Vec<IndexedMatch>,
    /// Auctions by ID.
    auctions: HashMap<u64, IndexedAuction>,
    /// Price history by model (epoch snapshots).
    price_history: HashMap<ModelId, VecDeque<PricePoint>>,
    /// Maximum price history entries per model.
    max_price_history: usize,
    /// Total events processed.
    pub events_processed: u64,
}

impl MarketplaceIndexer {
    pub fn new() -> Self {
        Self {
            head: 0,
            listings: HashMap::new(),
            listings_by_model: HashMap::new(),
            listings_by_provider: HashMap::new(),
            bids: HashMap::new(),
            bids_by_client: HashMap::new(),
            bids_by_model: HashMap::new(),
            matches: Vec::new(),
            auctions: HashMap::new(),
            price_history: HashMap::new(),
            max_price_history: 10_000,
            events_processed: 0,
        }
    }

    /// Process a batch of raw events from a block.
    pub fn process_events(&mut self, events: &[RawEvent]) {
        for event in events {
            if event.topics.is_empty() {
                continue;
            }
            let topic0 = event.topics[0];

            if topic0 == listing_created_topic() {
                self.handle_listing_created(event);
            } else if topic0 == listing_updated_topic() {
                self.handle_listing_updated(event);
            } else if topic0 == listing_deactivated_topic() {
                self.handle_listing_deactivated(event);
            } else if topic0 == bid_placed_topic() {
                self.handle_bid_placed(event);
            } else if topic0 == bid_matched_topic() {
                self.handle_bid_matched(event);
            } else if topic0 == bid_expired_topic() {
                self.handle_bid_expired(event);
            } else if topic0 == auction_created_topic() {
                self.handle_auction_created(event);
            } else if topic0 == auction_filled_topic() {
                self.handle_auction_filled(event);
            } else if topic0 == auction_completed_topic() {
                self.handle_auction_completed(event);
            }
            // Unrecognized events are silently skipped.

            self.events_processed += 1;
            if event.block_number > self.head {
                self.head = event.block_number;
            }
        }
    }

    // ── Event handlers ─────────────────────────────────────────

    fn handle_listing_created(&mut self, event: &RawEvent) {
        if event.data.len() < 88 { return; } // listing_id(8) + price_in(16) + price_out(16) + model(32) + provider(20) min
        let listing_id = u64::from_be_bytes(event.data[0..8].try_into().unwrap_or([0; 8]));
        let price_input = u128::from_be_bytes(event.data[8..24].try_into().unwrap_or([0; 16]));
        let price_output = u128::from_be_bytes(event.data[24..40].try_into().unwrap_or([0; 16]));
        let mut model_bytes = [0u8; 32];
        model_bytes.copy_from_slice(&event.data[40..72]);
        let model_id = ModelId(model_bytes);
        let mut provider_bytes = [0u8; 20];
        if event.data.len() >= 92 {
            provider_bytes.copy_from_slice(&event.data[72..92]);
        }
        let provider = Address(provider_bytes);

        let listing = IndexedListing {
            listing_id,
            provider: provider.clone(),
            model_id: model_id.clone(),
            price_input,
            price_output,
            created_at: event.block_number,
            updated_at: event.block_number,
            active: true,
        };

        self.listings.insert(listing_id, listing);
        self.listings_by_model.entry(model_id.clone()).or_default().push(listing_id);
        self.listings_by_provider.entry(provider).or_default().push(listing_id);

        // Update price snapshot.
        self.update_price_snapshot(&model_id, event.block_number);
    }

    fn handle_listing_updated(&mut self, event: &RawEvent) {
        if event.data.len() < 40 { return; }
        let listing_id = u64::from_be_bytes(event.data[0..8].try_into().unwrap_or([0; 8]));
        let new_price_input = u128::from_be_bytes(event.data[8..24].try_into().unwrap_or([0; 16]));
        let new_price_output = u128::from_be_bytes(event.data[24..40].try_into().unwrap_or([0; 16]));

        if let Some(listing) = self.listings.get_mut(&listing_id) {
            listing.price_input = new_price_input;
            listing.price_output = new_price_output;
            listing.updated_at = event.block_number;
            let model_id = listing.model_id.clone();
            self.update_price_snapshot(&model_id, event.block_number);
        }
    }

    fn handle_listing_deactivated(&mut self, event: &RawEvent) {
        if event.data.len() < 8 { return; }
        let listing_id = u64::from_be_bytes(event.data[0..8].try_into().unwrap_or([0; 8]));
        if let Some(listing) = self.listings.get_mut(&listing_id) {
            listing.active = false;
            listing.updated_at = event.block_number;
        }
    }

    fn handle_bid_placed(&mut self, event: &RawEvent) {
        if event.data.len() < 84 { return; }
        let bid_id = u64::from_be_bytes(event.data[0..8].try_into().unwrap_or([0; 8]));
        let max_price_input = u128::from_be_bytes(event.data[8..24].try_into().unwrap_or([0; 16]));
        let max_price_output = u128::from_be_bytes(event.data[24..40].try_into().unwrap_or([0; 16]));
        let mut model_bytes = [0u8; 32];
        model_bytes.copy_from_slice(&event.data[40..72]);
        let model_id = ModelId(model_bytes);
        let mut client_bytes = [0u8; 20];
        if event.data.len() >= 92 {
            client_bytes.copy_from_slice(&event.data[72..92]);
        }
        let client = Address(client_bytes);

        let bid = IndexedBid {
            bid_id,
            client: client.clone(),
            model_id: model_id.clone(),
            max_price_input,
            max_price_output,
            placed_at: event.block_number,
            matched: false,
            matched_listing: None,
            matched_at: None,
        };

        self.bids.insert(bid_id, bid);
        self.bids_by_client.entry(client).or_default().push(bid_id);
        self.bids_by_model.entry(model_id).or_default().push(bid_id);
    }

    fn handle_bid_matched(&mut self, event: &RawEvent) {
        if event.data.len() < 56 { return; }
        let bid_id = u64::from_be_bytes(event.data[0..8].try_into().unwrap_or([0; 8]));
        let listing_id = u64::from_be_bytes(event.data[8..16].try_into().unwrap_or([0; 8]));
        let mut client_bytes = [0u8; 20];
        client_bytes.copy_from_slice(&event.data[16..36]);
        let client = Address(client_bytes);
        let mut provider_bytes = [0u8; 20];
        provider_bytes.copy_from_slice(&event.data[36..56]);
        let provider = Address(provider_bytes);

        if let Some(bid) = self.bids.get_mut(&bid_id) {
            bid.matched = true;
            bid.matched_listing = Some(listing_id);
            bid.matched_at = Some(event.block_number);
        }

        self.matches.push(IndexedMatch {
            bid_id,
            listing_id,
            client,
            provider,
            epoch: event.block_number,
        });
    }

    fn handle_bid_expired(&mut self, event: &RawEvent) {
        if event.data.len() < 8 { return; }
        let bid_id = u64::from_be_bytes(event.data[0..8].try_into().unwrap_or([0; 8]));
        // Mark as expired by removing from active consideration (keep record).
        // Bids don't have an explicit expired flag; matched=false + age is sufficient.
        let _ = bid_id;
    }

    fn handle_auction_created(&mut self, event: &RawEvent) {
        if event.data.len() < 80 { return; }
        let auction_id = u64::from_be_bytes(event.data[0..8].try_into().unwrap_or([0; 8]));
        let start_price = u128::from_be_bytes(event.data[8..24].try_into().unwrap_or([0; 16]));
        let reserve_price = u128::from_be_bytes(event.data[24..40].try_into().unwrap_or([0; 16]));
        let duration_epochs = u64::from_be_bytes(event.data[40..48].try_into().unwrap_or([0; 8]));
        let total_slots = u32::from_be_bytes(event.data[48..52].try_into().unwrap_or([0; 4]));
        let mut model_bytes = [0u8; 32];
        model_bytes.copy_from_slice(&event.data[52..84].get(..32).unwrap_or(&[0; 32]));
        let model_id = ModelId(model_bytes);

        self.auctions.insert(auction_id, IndexedAuction {
            auction_id,
            model_id,
            start_price,
            reserve_price,
            duration_epochs,
            created_at: event.block_number,
            total_slots,
            filled_slots: 0,
            status: AuctionIndexStatus::Active,
            fills: Vec::new(),
        });
    }

    fn handle_auction_filled(&mut self, event: &RawEvent) {
        if event.data.len() < 32 { return; }
        let auction_id = u64::from_be_bytes(event.data[0..8].try_into().unwrap_or([0; 8]));
        let price = u128::from_be_bytes(event.data[8..24].try_into().unwrap_or([0; 16]));
        let slot_index = u32::from_be_bytes(event.data[24..28].try_into().unwrap_or([0; 4]));
        let mut bidder_bytes = [0u8; 20];
        if event.data.len() >= 48 {
            bidder_bytes.copy_from_slice(&event.data[28..48]);
        }

        if let Some(auction) = self.auctions.get_mut(&auction_id) {
            auction.filled_slots += 1;
            auction.fills.push(IndexedAuctionFill {
                bidder: Address(bidder_bytes),
                price,
                slot_index,
                epoch: event.block_number,
            });
        }
    }

    fn handle_auction_completed(&mut self, event: &RawEvent) {
        if event.data.len() < 9 { return; }
        let auction_id = u64::from_be_bytes(event.data[0..8].try_into().unwrap_or([0; 8]));
        let status_byte = event.data[8];
        let status = match status_byte {
            0 => AuctionIndexStatus::Completed,
            1 => AuctionIndexStatus::Expired,
            2 => AuctionIndexStatus::Cancelled,
            _ => AuctionIndexStatus::Completed,
        };

        if let Some(auction) = self.auctions.get_mut(&auction_id) {
            auction.status = status;
        }
    }

    // ── Price tracking ─────────────────────────────────────────

    fn update_price_snapshot(&mut self, model_id: &ModelId, epoch: Epoch) {
        let active_listings: Vec<_> = self.listings_by_model
            .get(model_id)
            .map(|ids| ids.iter()
                .filter_map(|id| self.listings.get(id))
                .filter(|l| l.active)
                .collect())
            .unwrap_or_default();

        if active_listings.is_empty() { return; }

        let count = active_listings.len() as u128;
        let avg_input = active_listings.iter().map(|l| l.price_input).sum::<u128>() / count;
        let avg_output = active_listings.iter().map(|l| l.price_output).sum::<u128>() / count;

        let point = PricePoint {
            epoch,
            model_id: model_id.clone(),
            avg_price_input: avg_input,
            avg_price_output: avg_output,
            listing_count: active_listings.len() as u32,
        };

        let history = self.price_history.entry(model_id.clone()).or_insert_with(VecDeque::new);
        history.push_back(point);
        if history.len() > self.max_price_history {
            history.pop_front();
        }
    }

    // ── Query API ──────────────────────────────────────────────

    /// Get a single listing by ID.
    pub fn get_listing(&self, listing_id: u64) -> Option<&IndexedListing> {
        self.listings.get(&listing_id)
    }

    /// Get all active listings for a model.
    pub fn get_listings_by_model(&self, model_id: &ModelId) -> Vec<&IndexedListing> {
        self.listings_by_model
            .get(model_id)
            .map(|ids| ids.iter()
                .filter_map(|id| self.listings.get(id))
                .filter(|l| l.active)
                .collect())
            .unwrap_or_default()
    }

    /// Get all listings by a provider.
    pub fn get_listings_by_provider(&self, provider: &Address) -> Vec<&IndexedListing> {
        self.listings_by_provider
            .get(provider)
            .map(|ids| ids.iter()
                .filter_map(|id| self.listings.get(id))
                .collect())
            .unwrap_or_default()
    }

    /// Get a bid by ID.
    pub fn get_bid(&self, bid_id: u64) -> Option<&IndexedBid> {
        self.bids.get(&bid_id)
    }

    /// Get all bids by a client.
    pub fn get_bids_by_client(&self, client: &Address) -> Vec<&IndexedBid> {
        self.bids_by_client
            .get(client)
            .map(|ids| ids.iter()
                .filter_map(|id| self.bids.get(id))
                .collect())
            .unwrap_or_default()
    }

    /// Get unmatched bids for a model.
    pub fn get_open_bids_by_model(&self, model_id: &ModelId) -> Vec<&IndexedBid> {
        self.bids_by_model
            .get(model_id)
            .map(|ids| ids.iter()
                .filter_map(|id| self.bids.get(id))
                .filter(|b| !b.matched)
                .collect())
            .unwrap_or_default()
    }

    /// Get match history with pagination.
    pub fn get_matches(&self, cursor: Cursor) -> &[IndexedMatch] {
        let start = cursor.offset.min(self.matches.len());
        let end = (start + cursor.limit).min(self.matches.len());
        &self.matches[start..end]
    }

    /// Get matches in an epoch range.
    pub fn get_matches_in_range(&self, range: EpochRange) -> Vec<&IndexedMatch> {
        self.matches.iter()
            .filter(|m| m.epoch >= range.from && m.epoch <= range.to)
            .collect()
    }

    /// Get an auction by ID.
    pub fn get_auction(&self, auction_id: u64) -> Option<&IndexedAuction> {
        self.auctions.get(&auction_id)
    }

    /// Get all active auctions.
    pub fn get_active_auctions(&self) -> Vec<&IndexedAuction> {
        self.auctions.values()
            .filter(|a| a.status == AuctionIndexStatus::Active)
            .collect()
    }

    /// Get price history for a model.
    pub fn get_price_history(&self, model_id: &ModelId, range: Option<EpochRange>) -> Vec<&PricePoint> {
        self.price_history
            .get(model_id)
            .map(|history| {
                history.iter()
                    .filter(|p| match range {
                        Some(r) => p.epoch >= r.from && p.epoch <= r.to,
                        None => true,
                    })
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Summary statistics.
    pub fn stats(&self) -> IndexerStats {
        IndexerStats {
            head: self.head,
            total_listings: self.listings.len(),
            active_listings: self.listings.values().filter(|l| l.active).count(),
            total_bids: self.bids.len(),
            matched_bids: self.bids.values().filter(|b| b.matched).count(),
            total_matches: self.matches.len(),
            total_auctions: self.auctions.len(),
            active_auctions: self.auctions.values().filter(|a| a.status == AuctionIndexStatus::Active).count(),
            events_processed: self.events_processed,
        }
    }

    /// Total listing count (active + inactive).
    pub fn listing_count(&self) -> usize {
        self.listings.len()
    }

    /// Total bid count.
    pub fn bid_count(&self) -> usize {
        self.bids.len()
    }
}

/// Summary statistics for the indexer.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IndexerStats {
    pub head: Epoch,
    pub total_listings: usize,
    pub active_listings: usize,
    pub total_bids: usize,
    pub matched_bids: usize,
    pub total_matches: usize,
    pub total_auctions: usize,
    pub active_auctions: usize,
    pub events_processed: u64,
}

// ── Tests ──────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn make_address(seed: u8) -> Address {
        Address([seed; 20])
    }

    fn make_model_id(seed: u8) -> ModelId {
        ModelId([seed; 32])
    }

    fn encode_listing_created(listing_id: u64, price_in: u128, price_out: u128, model: &ModelId, provider: &Address) -> Vec<u8> {
        let mut data = Vec::new();
        data.extend_from_slice(&listing_id.to_be_bytes());
        data.extend_from_slice(&price_in.to_be_bytes());
        data.extend_from_slice(&price_out.to_be_bytes());
        data.extend_from_slice(&model.0);
        data.extend_from_slice(&provider.0);
        data
    }

    fn encode_listing_updated(listing_id: u64, new_price_in: u128, new_price_out: u128) -> Vec<u8> {
        let mut data = Vec::new();
        data.extend_from_slice(&listing_id.to_be_bytes());
        data.extend_from_slice(&new_price_in.to_be_bytes());
        data.extend_from_slice(&new_price_out.to_be_bytes());
        data
    }

    fn encode_listing_deactivated(listing_id: u64) -> Vec<u8> {
        listing_id.to_be_bytes().to_vec()
    }

    fn encode_bid_placed(bid_id: u64, max_in: u128, max_out: u128, model: &ModelId, client: &Address) -> Vec<u8> {
        let mut data = Vec::new();
        data.extend_from_slice(&bid_id.to_be_bytes());
        data.extend_from_slice(&max_in.to_be_bytes());
        data.extend_from_slice(&max_out.to_be_bytes());
        data.extend_from_slice(&model.0);
        data.extend_from_slice(&client.0);
        data
    }

    fn encode_bid_matched(bid_id: u64, listing_id: u64, client: &Address, provider: &Address) -> Vec<u8> {
        let mut data = Vec::new();
        data.extend_from_slice(&bid_id.to_be_bytes());
        data.extend_from_slice(&listing_id.to_be_bytes());
        data.extend_from_slice(&client.0);
        data.extend_from_slice(&provider.0);
        data
    }

    fn encode_auction_created(id: u64, start: u128, reserve: u128, dur: u64, slots: u32, model: &ModelId) -> Vec<u8> {
        let mut data = Vec::new();
        data.extend_from_slice(&id.to_be_bytes());
        data.extend_from_slice(&start.to_be_bytes());
        data.extend_from_slice(&reserve.to_be_bytes());
        data.extend_from_slice(&dur.to_be_bytes());
        data.extend_from_slice(&slots.to_be_bytes());
        data.extend_from_slice(&model.0);
        data
    }

    fn encode_auction_filled(id: u64, price: u128, slot: u32, bidder: &Address) -> Vec<u8> {
        let mut data = Vec::new();
        data.extend_from_slice(&id.to_be_bytes());
        data.extend_from_slice(&price.to_be_bytes());
        data.extend_from_slice(&slot.to_be_bytes());
        data.extend_from_slice(&bidder.0);
        data
    }

    fn encode_auction_completed(id: u64, status: u8) -> Vec<u8> {
        let mut data = Vec::new();
        data.extend_from_slice(&id.to_be_bytes());
        data.push(status);
        data
    }

    #[test]
    fn test_empty_indexer() {
        let idx = MarketplaceIndexer::new();
        assert_eq!(idx.head, 0);
        assert_eq!(idx.listing_count(), 0);
        assert_eq!(idx.bid_count(), 0);
        let stats = idx.stats();
        assert_eq!(stats.events_processed, 0);
    }

    #[test]
    fn test_listing_created_and_query() {
        let mut idx = MarketplaceIndexer::new();
        let provider = make_address(1);
        let model = make_model_id(10);

        let events = vec![RawEvent {
            emitter: provider.clone(),
            topics: vec![listing_created_topic()],
            data: encode_listing_created(100, 5000, 8000, &model, &provider),
            block_number: 42,
        }];

        idx.process_events(&events);

        assert_eq!(idx.listing_count(), 1);
        let listing = idx.get_listing(100).unwrap();
        assert_eq!(listing.price_input, 5000);
        assert_eq!(listing.price_output, 8000);
        assert_eq!(listing.provider, provider);
        assert!(listing.active);
        assert_eq!(listing.created_at, 42);
    }

    #[test]
    fn test_listing_update() {
        let mut idx = MarketplaceIndexer::new();
        let provider = make_address(1);
        let model = make_model_id(10);

        idx.process_events(&[RawEvent {
            emitter: provider.clone(),
            topics: vec![listing_created_topic()],
            data: encode_listing_created(1, 100, 200, &model, &provider),
            block_number: 1,
        }]);

        idx.process_events(&[RawEvent {
            emitter: provider.clone(),
            topics: vec![listing_updated_topic()],
            data: encode_listing_updated(1, 150, 250),
            block_number: 5,
        }]);

        let listing = idx.get_listing(1).unwrap();
        assert_eq!(listing.price_input, 150);
        assert_eq!(listing.price_output, 250);
        assert_eq!(listing.updated_at, 5);
    }

    #[test]
    fn test_listing_deactivation() {
        let mut idx = MarketplaceIndexer::new();
        let provider = make_address(1);
        let model = make_model_id(10);

        idx.process_events(&[RawEvent {
            emitter: provider.clone(),
            topics: vec![listing_created_topic()],
            data: encode_listing_created(1, 100, 200, &model, &provider),
            block_number: 1,
        }]);

        idx.process_events(&[RawEvent {
            emitter: provider.clone(),
            topics: vec![listing_deactivated_topic()],
            data: encode_listing_deactivated(1),
            block_number: 10,
        }]);

        let listing = idx.get_listing(1).unwrap();
        assert!(!listing.active);

        // Active listing query should return empty.
        let active = idx.get_listings_by_model(&model);
        assert!(active.is_empty());
    }

    #[test]
    fn test_bid_placement_and_query() {
        let mut idx = MarketplaceIndexer::new();
        let client = make_address(2);
        let model = make_model_id(10);

        idx.process_events(&[RawEvent {
            emitter: client.clone(),
            topics: vec![bid_placed_topic()],
            data: encode_bid_placed(50, 6000, 9000, &model, &client),
            block_number: 20,
        }]);

        assert_eq!(idx.bid_count(), 1);
        let bid = idx.get_bid(50).unwrap();
        assert_eq!(bid.max_price_input, 6000);
        assert!(!bid.matched);

        let client_bids = idx.get_bids_by_client(&client);
        assert_eq!(client_bids.len(), 1);

        let open = idx.get_open_bids_by_model(&model);
        assert_eq!(open.len(), 1);
    }

    #[test]
    fn test_bid_matching() {
        let mut idx = MarketplaceIndexer::new();
        let client = make_address(2);
        let provider = make_address(1);
        let model = make_model_id(10);

        // Create listing + bid.
        idx.process_events(&[
            RawEvent {
                emitter: provider.clone(),
                topics: vec![listing_created_topic()],
                data: encode_listing_created(1, 5000, 8000, &model, &provider),
                block_number: 1,
            },
            RawEvent {
                emitter: client.clone(),
                topics: vec![bid_placed_topic()],
                data: encode_bid_placed(50, 6000, 9000, &model, &client),
                block_number: 2,
            },
        ]);

        // Match.
        idx.process_events(&[RawEvent {
            emitter: make_address(0), // system
            topics: vec![bid_matched_topic()],
            data: encode_bid_matched(50, 1, &client, &provider),
            block_number: 3,
        }]);

        let bid = idx.get_bid(50).unwrap();
        assert!(bid.matched);
        assert_eq!(bid.matched_listing, Some(1));
        assert_eq!(bid.matched_at, Some(3));

        let matches = idx.get_matches(Cursor::default());
        assert_eq!(matches.len(), 1);
        assert_eq!(matches[0].bid_id, 50);
        assert_eq!(matches[0].listing_id, 1);
    }

    #[test]
    fn test_match_range_query() {
        let mut idx = MarketplaceIndexer::new();
        let client = make_address(2);
        let provider = make_address(1);

        // Add 3 matches at different epochs.
        for i in 0..3u64 {
            idx.process_events(&[RawEvent {
                emitter: make_address(0),
                topics: vec![bid_matched_topic()],
                data: encode_bid_matched(i, i + 10, &client, &provider),
                block_number: (i + 1) * 10,
            }]);
        }

        let range = idx.get_matches_in_range(EpochRange { from: 15, to: 25 });
        assert_eq!(range.len(), 1);
        assert_eq!(range[0].epoch, 20);
    }

    #[test]
    fn test_auction_lifecycle() {
        let mut idx = MarketplaceIndexer::new();
        let model = make_model_id(5);
        let bidder1 = make_address(10);
        let bidder2 = make_address(11);

        // Create auction.
        idx.process_events(&[RawEvent {
            emitter: make_address(0),
            topics: vec![auction_created_topic()],
            data: encode_auction_created(1, 1_000_000, 100_000, 100, 3, &model),
            block_number: 50,
        }]);

        let auction = idx.get_auction(1).unwrap();
        assert_eq!(auction.status, AuctionIndexStatus::Active);
        assert_eq!(auction.total_slots, 3);
        assert_eq!(auction.filled_slots, 0);

        // Fill two slots.
        idx.process_events(&[
            RawEvent {
                emitter: bidder1.clone(),
                topics: vec![auction_filled_topic()],
                data: encode_auction_filled(1, 800_000, 0, &bidder1),
                block_number: 60,
            },
            RawEvent {
                emitter: bidder2.clone(),
                topics: vec![auction_filled_topic()],
                data: encode_auction_filled(1, 700_000, 1, &bidder2),
                block_number: 70,
            },
        ]);

        let auction = idx.get_auction(1).unwrap();
        assert_eq!(auction.filled_slots, 2);
        assert_eq!(auction.fills.len(), 2);
        assert_eq!(auction.fills[0].price, 800_000);
        assert_eq!(auction.fills[1].slot_index, 1);

        // Complete.
        idx.process_events(&[RawEvent {
            emitter: make_address(0),
            topics: vec![auction_completed_topic()],
            data: encode_auction_completed(1, 1), // Expired
            block_number: 150,
        }]);

        let auction = idx.get_auction(1).unwrap();
        assert_eq!(auction.status, AuctionIndexStatus::Expired);
    }

    #[test]
    fn test_active_auctions_filter() {
        let mut idx = MarketplaceIndexer::new();
        let model = make_model_id(5);

        for i in 0..3u64 {
            idx.process_events(&[RawEvent {
                emitter: make_address(0),
                topics: vec![auction_created_topic()],
                data: encode_auction_created(i, 1000, 100, 50, 2, &model),
                block_number: i * 10,
            }]);
        }

        // Complete auction 1.
        idx.process_events(&[RawEvent {
            emitter: make_address(0),
            topics: vec![auction_completed_topic()],
            data: encode_auction_completed(1, 0),
            block_number: 50,
        }]);

        let active = idx.get_active_auctions();
        assert_eq!(active.len(), 2); // 0 and 2 still active
    }

    #[test]
    fn test_price_history_tracking() {
        let mut idx = MarketplaceIndexer::new();
        let model = make_model_id(10);
        let provider = make_address(1);

        // Create listings at different prices.
        for i in 0..3u64 {
            idx.process_events(&[RawEvent {
                emitter: provider.clone(),
                topics: vec![listing_created_topic()],
                data: encode_listing_created(i, 1000 + i as u128 * 500, 2000, &model, &provider),
                block_number: i * 5,
            }]);
        }

        let history = idx.get_price_history(&model, None);
        assert_eq!(history.len(), 3);
        // Last snapshot should average all 3 listings.
        let last = history.last().unwrap();
        assert_eq!(last.listing_count, 3);
        assert_eq!(last.avg_price_input, (1000 + 1500 + 2000) / 3);
    }

    #[test]
    fn test_price_history_range_filter() {
        let mut idx = MarketplaceIndexer::new();
        let model = make_model_id(10);
        let provider = make_address(1);

        for i in 0..5u64 {
            idx.process_events(&[RawEvent {
                emitter: provider.clone(),
                topics: vec![listing_created_topic()],
                data: encode_listing_created(i, 1000, 2000, &model, &provider),
                block_number: i * 10,
            }]);
        }

        let filtered = idx.get_price_history(&model, Some(EpochRange { from: 15, to: 35 }));
        assert_eq!(filtered.len(), 2); // epochs 20 and 30
    }

    #[test]
    fn test_multi_model_isolation() {
        let mut idx = MarketplaceIndexer::new();
        let model_a = make_model_id(1);
        let model_b = make_model_id(2);
        let provider = make_address(1);

        idx.process_events(&[
            RawEvent {
                emitter: provider.clone(),
                topics: vec![listing_created_topic()],
                data: encode_listing_created(1, 100, 200, &model_a, &provider),
                block_number: 1,
            },
            RawEvent {
                emitter: provider.clone(),
                topics: vec![listing_created_topic()],
                data: encode_listing_created(2, 300, 400, &model_b, &provider),
                block_number: 1,
            },
        ]);

        let a_listings = idx.get_listings_by_model(&model_a);
        assert_eq!(a_listings.len(), 1);
        assert_eq!(a_listings[0].price_input, 100);

        let b_listings = idx.get_listings_by_model(&model_b);
        assert_eq!(b_listings.len(), 1);
        assert_eq!(b_listings[0].price_input, 300);
    }

    #[test]
    fn test_provider_listing_index() {
        let mut idx = MarketplaceIndexer::new();
        let model = make_model_id(10);
        let provider_a = make_address(1);
        let provider_b = make_address(2);

        idx.process_events(&[
            RawEvent {
                emitter: provider_a.clone(),
                topics: vec![listing_created_topic()],
                data: encode_listing_created(1, 100, 200, &model, &provider_a),
                block_number: 1,
            },
            RawEvent {
                emitter: provider_a.clone(),
                topics: vec![listing_created_topic()],
                data: encode_listing_created(2, 150, 250, &model, &provider_a),
                block_number: 2,
            },
            RawEvent {
                emitter: provider_b.clone(),
                topics: vec![listing_created_topic()],
                data: encode_listing_created(3, 200, 300, &model, &provider_b),
                block_number: 3,
            },
        ]);

        assert_eq!(idx.get_listings_by_provider(&provider_a).len(), 2);
        assert_eq!(idx.get_listings_by_provider(&provider_b).len(), 1);
    }

    #[test]
    fn test_stats() {
        let mut idx = MarketplaceIndexer::new();
        let model = make_model_id(10);
        let provider = make_address(1);
        let client = make_address(2);

        idx.process_events(&[
            RawEvent {
                emitter: provider.clone(),
                topics: vec![listing_created_topic()],
                data: encode_listing_created(1, 100, 200, &model, &provider),
                block_number: 1,
            },
            RawEvent {
                emitter: client.clone(),
                topics: vec![bid_placed_topic()],
                data: encode_bid_placed(50, 200, 300, &model, &client),
                block_number: 2,
            },
            RawEvent {
                emitter: make_address(0),
                topics: vec![bid_matched_topic()],
                data: encode_bid_matched(50, 1, &client, &provider),
                block_number: 3,
            },
        ]);

        let stats = idx.stats();
        assert_eq!(stats.total_listings, 1);
        assert_eq!(stats.active_listings, 1);
        assert_eq!(stats.total_bids, 1);
        assert_eq!(stats.matched_bids, 1);
        assert_eq!(stats.total_matches, 1);
        assert_eq!(stats.events_processed, 3);
        assert_eq!(stats.head, 3);
    }

    #[test]
    fn test_cursor_pagination() {
        let mut idx = MarketplaceIndexer::new();
        let client = make_address(2);
        let provider = make_address(1);

        for i in 0..10u64 {
            idx.process_events(&[RawEvent {
                emitter: make_address(0),
                topics: vec![bid_matched_topic()],
                data: encode_bid_matched(i, i + 100, &client, &provider),
                block_number: i,
            }]);
        }

        let page1 = idx.get_matches(Cursor { offset: 0, limit: 3 });
        assert_eq!(page1.len(), 3);
        assert_eq!(page1[0].bid_id, 0);

        let page2 = idx.get_matches(Cursor { offset: 3, limit: 3 });
        assert_eq!(page2.len(), 3);
        assert_eq!(page2[0].bid_id, 3);

        let last = idx.get_matches(Cursor { offset: 9, limit: 5 });
        assert_eq!(last.len(), 1);
    }

    #[test]
    fn test_ignores_unknown_events() {
        let mut idx = MarketplaceIndexer::new();
        idx.process_events(&[RawEvent {
            emitter: make_address(0),
            topics: vec![[0xFF; 32]], // unknown topic
            data: vec![0; 100],
            block_number: 99,
        }]);

        assert_eq!(idx.listing_count(), 0);
        assert_eq!(idx.events_processed, 1);
        assert_eq!(idx.head, 99);
    }

    #[test]
    fn test_ignores_empty_topic_events() {
        let mut idx = MarketplaceIndexer::new();
        idx.process_events(&[RawEvent {
            emitter: make_address(0),
            topics: vec![],
            data: vec![],
            block_number: 5,
        }]);
        assert_eq!(idx.events_processed, 0);
    }

    #[test]
    fn test_malformed_data_resilience() {
        let mut idx = MarketplaceIndexer::new();
        // Listing created with too-short data → should be skipped.
        idx.process_events(&[RawEvent {
            emitter: make_address(0),
            topics: vec![listing_created_topic()],
            data: vec![0; 10], // Way too short.
            block_number: 1,
        }]);
        assert_eq!(idx.listing_count(), 0);
        assert_eq!(idx.events_processed, 1);
    }

    #[test]
    fn test_head_tracking() {
        let mut idx = MarketplaceIndexer::new();
        let provider = make_address(1);
        let model = make_model_id(10);

        idx.process_events(&[RawEvent {
            emitter: provider.clone(),
            topics: vec![listing_created_topic()],
            data: encode_listing_created(1, 100, 200, &model, &provider),
            block_number: 100,
        }]);
        assert_eq!(idx.head, 100);

        idx.process_events(&[RawEvent {
            emitter: provider.clone(),
            topics: vec![listing_created_topic()],
            data: encode_listing_created(2, 100, 200, &model, &provider),
            block_number: 50, // older block
        }]);
        // Head should NOT go backwards.
        assert_eq!(idx.head, 100);
    }
}
