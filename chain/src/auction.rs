//! CHAIN-028: Dutch Auction for Premium Model Slots
//!
//! A descending-price auction mechanism for allocating scarce premium inference
//! slots on high-demand models. The price starts high and decreases linearly
//! over a configurable duration until a bidder accepts or the reserve price
//! is reached.
//!
//! Design:
//! - Marketplace owner creates an auction for N premium slots on a model
//! - Price decreases from `start_price` to `reserve_price` over `duration_epochs`
//! - Bidders call `accept_current_price` to lock in at the current price
//! - Each acceptance fills one slot; auction ends when all slots are filled or duration expires
//! - Revenue goes to the marketplace fee pool
//! - Anti-sniping: if a bid arrives in the last `snipe_guard_epochs`, deadline extends

use crate::types::{Address, Epoch, ModelId};
use std::collections::HashMap;

/// Unique auction identifier.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct AuctionId(pub u64);

/// A single acceptance/fill in an auction.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AuctionFill {
    pub bidder: Address,
    pub price: u128,
    pub epoch: Epoch,
    pub slot_index: u32,
}

/// Auction state.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AuctionStatus {
    /// Auction is live and accepting bids.
    Active,
    /// All slots filled.
    FilledAll,
    /// Duration expired (some or no slots filled).
    Expired,
    /// Cancelled by creator before any fills.
    Cancelled,
}

/// A Dutch auction for premium model slots.
#[derive(Debug, Clone)]
pub struct Auction {
    pub id: AuctionId,
    pub model_id: ModelId,
    pub creator: Address,
    pub total_slots: u32,
    pub start_price: u128,
    pub reserve_price: u128,
    pub start_epoch: Epoch,
    pub duration_epochs: u64,
    /// Extended deadline (may be pushed by snipe guard).
    pub end_epoch: Epoch,
    pub snipe_guard_epochs: u64,
    pub status: AuctionStatus,
    pub fills: Vec<AuctionFill>,
    pub total_revenue: u128,
}

impl Auction {
    /// Current price at a given epoch (linear decrease).
    pub fn price_at(&self, epoch: Epoch) -> Option<u128> {
        if epoch < self.start_epoch {
            return None;
        }
        let elapsed = epoch - self.start_epoch;
        let total_duration = self.end_epoch - self.start_epoch;
        if elapsed >= total_duration {
            return Some(self.reserve_price);
        }
        let price_range = self.start_price - self.reserve_price;
        let decrease = price_range * elapsed as u128 / total_duration as u128;
        Some(self.start_price - decrease)
    }

    pub fn slots_remaining(&self) -> u32 {
        self.total_slots - self.fills.len() as u32
    }
}

#[derive(Debug, PartialEq)]
pub enum AuctionError {
    AuctionNotFound(AuctionId),
    AuctionNotActive(AuctionId),
    AuctionNotStarted { auction: AuctionId, starts: Epoch },
    NoSlotsRemaining(AuctionId),
    InsufficientFunds { required: u128, provided: u128 },
    InvalidParams(String),
    NotCreator,
    HasFills,
    AlreadyBidder(AuctionId),
}

/// Manages all active and completed auctions.
#[derive(Debug)]
pub struct AuctionHouse {
    auctions: HashMap<AuctionId, Auction>,
    /// Model → auction IDs.
    model_index: HashMap<ModelId, Vec<AuctionId>>,
    next_id: u64,
    current_epoch: Epoch,
    /// Accumulated revenue from all auctions.
    pub total_revenue: u128,
}

impl AuctionHouse {
    pub fn new() -> Self {
        Self {
            auctions: HashMap::new(),
            model_index: HashMap::new(),
            next_id: 1,
            current_epoch: 0,
            total_revenue: 0,
        }
    }

    pub fn set_epoch(&mut self, epoch: Epoch) {
        self.current_epoch = epoch;
    }

    /// Create a new Dutch auction.
    pub fn create_auction(
        &mut self,
        creator: Address,
        model_id: ModelId,
        total_slots: u32,
        start_price: u128,
        reserve_price: u128,
        start_epoch: Epoch,
        duration_epochs: u64,
        snipe_guard_epochs: u64,
    ) -> Result<AuctionId, AuctionError> {
        if total_slots == 0 {
            return Err(AuctionError::InvalidParams(
                "total_slots must be > 0".into(),
            ));
        }
        if start_price < reserve_price {
            return Err(AuctionError::InvalidParams(
                "start_price must be >= reserve_price".into(),
            ));
        }
        if duration_epochs == 0 {
            return Err(AuctionError::InvalidParams("duration must be > 0".into()));
        }

        let id = AuctionId(self.next_id);
        self.next_id += 1;

        let auction = Auction {
            id,
            model_id,
            creator,
            total_slots,
            start_price,
            reserve_price,
            start_epoch,
            duration_epochs,
            end_epoch: start_epoch + duration_epochs,
            snipe_guard_epochs,
            status: AuctionStatus::Active,
            fills: Vec::new(),
            total_revenue: 0,
        };

        self.auctions.insert(id, auction);
        self.model_index.entry(model_id).or_default().push(id);
        Ok(id)
    }

    /// Accept the current price in a Dutch auction (i.e., place a bid at the current descending price).
    pub fn accept_current_price(
        &mut self,
        auction_id: AuctionId,
        bidder: Address,
        max_payment: u128,
    ) -> Result<AuctionFill, AuctionError> {
        // Check auction exists and is active
        let auction = self
            .auctions
            .get(&auction_id)
            .ok_or(AuctionError::AuctionNotFound(auction_id))?;

        if auction.status != AuctionStatus::Active {
            return Err(AuctionError::AuctionNotActive(auction_id));
        }
        if self.current_epoch < auction.start_epoch {
            return Err(AuctionError::AuctionNotStarted {
                auction: auction_id,
                starts: auction.start_epoch,
            });
        }
        if auction.slots_remaining() == 0 {
            return Err(AuctionError::NoSlotsRemaining(auction_id));
        }

        // Check if bidder already has a slot
        if auction.fills.iter().any(|f| f.bidder == bidder) {
            return Err(AuctionError::AlreadyBidder(auction_id));
        }

        // Calculate current price
        let price = auction
            .price_at(self.current_epoch)
            .unwrap_or(auction.reserve_price);
        if max_payment < price {
            return Err(AuctionError::InsufficientFunds {
                required: price,
                provided: max_payment,
            });
        }

        let slot_index = auction.fills.len() as u32;

        let fill = AuctionFill {
            bidder,
            price,
            epoch: self.current_epoch,
            slot_index,
        };

        // Now mutate
        let auction = self.auctions.get_mut(&auction_id).unwrap();

        // Anti-snipe: extend if bid in final snipe_guard_epochs
        let snipe_threshold = auction.end_epoch.saturating_sub(auction.snipe_guard_epochs);
        if self.current_epoch >= snipe_threshold && auction.snipe_guard_epochs > 0 {
            auction.end_epoch = self.current_epoch + auction.snipe_guard_epochs;
        }

        auction.fills.push(fill.clone());
        auction.total_revenue += price;
        self.total_revenue += price;

        // Check if all slots filled
        if auction.slots_remaining() == 0 {
            auction.status = AuctionStatus::FilledAll;
        }

        Ok(fill)
    }

    /// Finalize expired auctions. Call periodically (e.g., every epoch tick).
    pub fn finalize_expired(&mut self) -> Vec<AuctionId> {
        let mut finalized = Vec::new();
        for auction in self.auctions.values_mut() {
            if auction.status == AuctionStatus::Active && self.current_epoch >= auction.end_epoch {
                auction.status = AuctionStatus::Expired;
                finalized.push(auction.id);
            }
        }
        finalized
    }

    /// Cancel an auction (only if no fills yet).
    pub fn cancel_auction(
        &mut self,
        auction_id: AuctionId,
        caller: Address,
    ) -> Result<(), AuctionError> {
        let auction = self
            .auctions
            .get(&auction_id)
            .ok_or(AuctionError::AuctionNotFound(auction_id))?;
        if auction.creator != caller {
            return Err(AuctionError::NotCreator);
        }
        if auction.status != AuctionStatus::Active {
            return Err(AuctionError::AuctionNotActive(auction_id));
        }
        if !auction.fills.is_empty() {
            return Err(AuctionError::HasFills);
        }
        let auction = self.auctions.get_mut(&auction_id).unwrap();
        auction.status = AuctionStatus::Cancelled;
        Ok(())
    }

    /// Get auction by ID.
    pub fn get_auction(&self, id: AuctionId) -> Option<&Auction> {
        self.auctions.get(&id)
    }

    /// Get all auctions for a model.
    pub fn model_auctions(&self, model_id: ModelId) -> Vec<&Auction> {
        self.model_index
            .get(&model_id)
            .map(|ids| ids.iter().filter_map(|id| self.auctions.get(id)).collect())
            .unwrap_or_default()
    }

    /// Get active auctions for a model.
    pub fn active_auctions(&self, model_id: ModelId) -> Vec<&Auction> {
        self.model_auctions(model_id)
            .into_iter()
            .filter(|a| a.status == AuctionStatus::Active)
            .collect()
    }

    /// Current price for an auction, or None if not active.
    pub fn current_price(&self, auction_id: AuctionId) -> Option<u128> {
        let auction = self.auctions.get(&auction_id)?;
        if auction.status != AuctionStatus::Active {
            return None;
        }
        auction.price_at(self.current_epoch)
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
    fn test_create_auction() {
        let mut house = AuctionHouse::new();
        let aid = house
            .create_auction(addr(1), model(1), 3, 1000, 100, 10, 100, 5)
            .unwrap();
        let a = house.get_auction(aid).unwrap();
        assert_eq!(a.total_slots, 3);
        assert_eq!(a.start_price, 1000);
        assert_eq!(a.reserve_price, 100);
        assert_eq!(a.status, AuctionStatus::Active);
    }

    #[test]
    fn test_invalid_params() {
        let mut house = AuctionHouse::new();
        // zero slots
        assert!(house
            .create_auction(addr(1), model(1), 0, 1000, 100, 10, 100, 5)
            .is_err());
        // start < reserve
        assert!(house
            .create_auction(addr(1), model(1), 3, 50, 100, 10, 100, 5)
            .is_err());
        // zero duration
        assert!(house
            .create_auction(addr(1), model(1), 3, 1000, 100, 10, 0, 5)
            .is_err());
    }

    #[test]
    fn test_price_decreases_linearly() {
        let mut house = AuctionHouse::new();
        let aid = house
            .create_auction(addr(1), model(1), 5, 1000, 0, 0, 100, 0)
            .unwrap();
        let a = house.get_auction(aid).unwrap();
        // At epoch 0: 1000, epoch 50: 500, epoch 100: 0
        assert_eq!(a.price_at(0), Some(1000));
        assert_eq!(a.price_at(50), Some(500));
        assert_eq!(a.price_at(100), Some(0));
        // Past end → reserve
        assert_eq!(a.price_at(200), Some(0));
    }

    #[test]
    fn test_price_with_reserve() {
        let mut house = AuctionHouse::new();
        let aid = house
            .create_auction(addr(1), model(1), 5, 1000, 200, 0, 100, 0)
            .unwrap();
        let a = house.get_auction(aid).unwrap();
        // Range is 800 over 100 epochs. At 50: 1000 - 400 = 600
        assert_eq!(a.price_at(0), Some(1000));
        assert_eq!(a.price_at(50), Some(600));
        assert_eq!(a.price_at(100), Some(200));
    }

    #[test]
    fn test_accept_current_price() {
        let mut house = AuctionHouse::new();
        let aid = house
            .create_auction(addr(1), model(1), 3, 1000, 100, 0, 100, 0)
            .unwrap();
        house.set_epoch(50);
        // Price at 50: 1000 - (900 * 50/100) = 550
        let fill = house.accept_current_price(aid, addr(2), 600).unwrap();
        assert_eq!(fill.price, 550);
        assert_eq!(fill.slot_index, 0);
        assert_eq!(house.get_auction(aid).unwrap().slots_remaining(), 2);
        assert_eq!(house.total_revenue, 550);
    }

    #[test]
    fn test_insufficient_funds() {
        let mut house = AuctionHouse::new();
        let aid = house
            .create_auction(addr(1), model(1), 3, 1000, 100, 0, 100, 0)
            .unwrap();
        house.set_epoch(0); // price = 1000
        let err = house.accept_current_price(aid, addr(2), 500).unwrap_err();
        assert_eq!(
            err,
            AuctionError::InsufficientFunds {
                required: 1000,
                provided: 500
            }
        );
    }

    #[test]
    fn test_all_slots_filled() {
        let mut house = AuctionHouse::new();
        let aid = house
            .create_auction(addr(1), model(1), 2, 1000, 100, 0, 100, 0)
            .unwrap();
        house.set_epoch(50);
        house.accept_current_price(aid, addr(2), 1000).unwrap();
        house.accept_current_price(aid, addr(3), 1000).unwrap();
        assert_eq!(
            house.get_auction(aid).unwrap().status,
            AuctionStatus::FilledAll
        );
        // Third bid fails
        let err = house.accept_current_price(aid, addr(4), 1000).unwrap_err();
        assert_eq!(err, AuctionError::AuctionNotActive(aid));
    }

    #[test]
    fn test_no_duplicate_bidder() {
        let mut house = AuctionHouse::new();
        let aid = house
            .create_auction(addr(1), model(1), 5, 1000, 100, 0, 100, 0)
            .unwrap();
        house.set_epoch(50);
        house.accept_current_price(aid, addr(2), 1000).unwrap();
        let err = house.accept_current_price(aid, addr(2), 1000).unwrap_err();
        assert_eq!(err, AuctionError::AlreadyBidder(aid));
    }

    #[test]
    fn test_auction_not_started() {
        let mut house = AuctionHouse::new();
        let aid = house
            .create_auction(addr(1), model(1), 3, 1000, 100, 50, 100, 0)
            .unwrap();
        house.set_epoch(10);
        let err = house.accept_current_price(aid, addr(2), 1000).unwrap_err();
        assert_eq!(
            err,
            AuctionError::AuctionNotStarted {
                auction: aid,
                starts: 50
            }
        );
    }

    #[test]
    fn test_snipe_guard_extends_deadline() {
        let mut house = AuctionHouse::new();
        // Auction: epochs 0-100, snipe guard = 10 epochs
        let aid = house
            .create_auction(addr(1), model(1), 5, 1000, 100, 0, 100, 10)
            .unwrap();
        assert_eq!(house.get_auction(aid).unwrap().end_epoch, 100);

        // Bid at epoch 95 (within snipe guard zone: 100-10=90)
        house.set_epoch(95);
        house.accept_current_price(aid, addr(2), 1000).unwrap();
        // Deadline should be extended to 95 + 10 = 105
        assert_eq!(house.get_auction(aid).unwrap().end_epoch, 105);
    }

    #[test]
    fn test_finalize_expired() {
        let mut house = AuctionHouse::new();
        let a1 = house
            .create_auction(addr(1), model(1), 3, 1000, 100, 0, 50, 0)
            .unwrap();
        let a2 = house
            .create_auction(addr(1), model(2), 3, 1000, 100, 0, 100, 0)
            .unwrap();
        house.set_epoch(60);
        let expired = house.finalize_expired();
        assert_eq!(expired, vec![a1]);
        assert_eq!(
            house.get_auction(a1).unwrap().status,
            AuctionStatus::Expired
        );
        assert_eq!(house.get_auction(a2).unwrap().status, AuctionStatus::Active);
    }

    #[test]
    fn test_cancel_auction() {
        let mut house = AuctionHouse::new();
        let aid = house
            .create_auction(addr(1), model(1), 3, 1000, 100, 0, 100, 0)
            .unwrap();
        house.cancel_auction(aid, addr(1)).unwrap();
        assert_eq!(
            house.get_auction(aid).unwrap().status,
            AuctionStatus::Cancelled
        );
    }

    #[test]
    fn test_cancel_wrong_creator() {
        let mut house = AuctionHouse::new();
        let aid = house
            .create_auction(addr(1), model(1), 3, 1000, 100, 0, 100, 0)
            .unwrap();
        assert_eq!(
            house.cancel_auction(aid, addr(2)).unwrap_err(),
            AuctionError::NotCreator
        );
    }

    #[test]
    fn test_cancel_with_fills_rejected() {
        let mut house = AuctionHouse::new();
        let aid = house
            .create_auction(addr(1), model(1), 3, 1000, 100, 0, 100, 0)
            .unwrap();
        house.set_epoch(50);
        house.accept_current_price(aid, addr(2), 1000).unwrap();
        assert_eq!(
            house.cancel_auction(aid, addr(1)).unwrap_err(),
            AuctionError::HasFills
        );
    }

    #[test]
    fn test_active_auctions_query() {
        let mut house = AuctionHouse::new();
        house
            .create_auction(addr(1), model(1), 3, 1000, 100, 0, 50, 0)
            .unwrap();
        house
            .create_auction(addr(1), model(1), 3, 500, 50, 0, 100, 0)
            .unwrap();
        house.set_epoch(60);
        house.finalize_expired();
        assert_eq!(house.active_auctions(model(1)).len(), 1);
        assert_eq!(house.model_auctions(model(1)).len(), 2);
    }

    #[test]
    fn test_revenue_accumulates() {
        let mut house = AuctionHouse::new();
        let aid = house
            .create_auction(addr(1), model(1), 3, 1000, 0, 0, 100, 0)
            .unwrap();
        house.set_epoch(0); // price = 1000
        house.accept_current_price(aid, addr(2), 1000).unwrap();
        house.set_epoch(50); // price = 500
        house.accept_current_price(aid, addr(3), 500).unwrap();
        assert_eq!(house.get_auction(aid).unwrap().total_revenue, 1500);
        assert_eq!(house.total_revenue, 1500);
    }

    #[test]
    fn test_current_price_helper() {
        let mut house = AuctionHouse::new();
        let aid = house
            .create_auction(addr(1), model(1), 3, 1000, 100, 0, 100, 0)
            .unwrap();
        house.set_epoch(25);
        assert_eq!(house.current_price(aid), Some(775)); // 1000 - 900*25/100
    }

    #[test]
    fn test_later_bidders_pay_less() {
        let mut house = AuctionHouse::new();
        let aid = house
            .create_auction(addr(1), model(1), 3, 900, 0, 0, 90, 0)
            .unwrap();
        // Each epoch = 10 price decrease
        house.set_epoch(10);
        let f1 = house.accept_current_price(aid, addr(2), 900).unwrap();
        house.set_epoch(30);
        let f2 = house.accept_current_price(aid, addr(3), 900).unwrap();
        house.set_epoch(60);
        let f3 = house.accept_current_price(aid, addr(4), 900).unwrap();
        assert_eq!(f1.price, 800);
        assert_eq!(f2.price, 600);
        assert_eq!(f3.price, 300);
        assert_eq!(
            house.get_auction(aid).unwrap().status,
            AuctionStatus::FilledAll
        );
    }
}
