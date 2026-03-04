//! Payment Channels — streaming payments for inference and storage.
//!
//! Inspired by Filecoin Pay but simplified for Prova's use case.
//! Payers lock funds in a channel; providers earn per-inference or
//! per-epoch for storage. Settlement happens on-chain periodically.

use std::collections::HashMap;
use crate::types::*;

/// Payment channel state.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ChannelState {
    /// Active — funds locked, payments streaming.
    Active,
    /// Settling — one party initiated close, in dispute window.
    Settling { close_epoch: Epoch },
    /// Closed — funds distributed, channel archived.
    Closed,
}

/// A payment channel between a payer and a provider.
#[derive(Debug, Clone)]
pub struct PaymentChannel {
    /// Unique channel ID.
    pub id: u64,
    /// Who's paying.
    pub payer: Address,
    /// Who's receiving.
    pub provider: Address,
    /// Total funds locked in the channel.
    pub locked: StakeAmount,
    /// Amount already paid out to provider.
    pub paid: StakeAmount,
    /// Payment rate per inference (or per epoch for storage).
    pub rate: StakeAmount,
    /// Channel state.
    pub state: ChannelState,
    /// Epoch when channel was created.
    pub created_at: Epoch,
    /// Last epoch when payment was processed.
    pub last_payment_epoch: Epoch,
    /// Number of inferences paid for.
    pub inference_count: u64,
}

impl PaymentChannel {
    /// Remaining balance in the channel.
    pub fn balance(&self) -> StakeAmount {
        self.locked.saturating_sub(self.paid)
    }

    /// Number of inferences remaining at current rate.
    pub fn remaining_inferences(&self) -> u64 {
        if self.rate == 0 {
            return u64::MAX;
        }
        (self.balance() / self.rate) as u64
    }
}

/// Settlement dispute window (epochs).
const SETTLE_WINDOW: EpochDuration = 480; // ~4 hours

/// Network fee basis points (0.5% = 50 bps).
const NETWORK_FEE_BPS: u32 = 50;

/// Payment channel manager.
#[derive(Debug)]
pub struct PaymentManager {
    channels: HashMap<u64, PaymentChannel>,
    next_id: u64,
    /// Accumulated network fees (for protocol treasury/burn).
    pub network_fees: StakeAmount,
}

impl PaymentManager {
    pub fn new() -> Self {
        Self {
            channels: HashMap::new(),
            next_id: 1,
            network_fees: 0,
        }
    }

    /// Open a new payment channel.
    pub fn open_channel(
        &mut self,
        payer: Address,
        provider: Address,
        locked: StakeAmount,
        rate: StakeAmount,
        epoch: Epoch,
    ) -> Result<u64, PaymentError> {
        if payer == provider {
            return Err(PaymentError::SelfChannel);
        }
        if locked == 0 {
            return Err(PaymentError::ZeroLock);
        }
        if rate == 0 {
            return Err(PaymentError::ZeroRate);
        }

        let id = self.next_id;
        self.next_id += 1;

        self.channels.insert(
            id,
            PaymentChannel {
                id,
                payer,
                provider,
                locked,
                paid: 0,
                rate,
                state: ChannelState::Active,
                created_at: epoch,
                last_payment_epoch: epoch,
                inference_count: 0,
            },
        );

        Ok(id)
    }

    /// Record a payment for an inference.
    /// Returns the amount paid (rate minus network fee).
    pub fn pay_inference(
        &mut self,
        channel_id: u64,
        epoch: Epoch,
    ) -> Result<StakeAmount, PaymentError> {
        let channel = self
            .channels
            .get_mut(&channel_id)
            .ok_or(PaymentError::NotFound(channel_id))?;

        if channel.state != ChannelState::Active {
            return Err(PaymentError::NotActive(channel_id));
        }

        if channel.balance() < channel.rate {
            return Err(PaymentError::InsufficientFunds {
                balance: channel.balance(),
                required: channel.rate,
            });
        }

        // Deduct network fee
        let fee = (channel.rate as u128 * NETWORK_FEE_BPS as u128 / 10000) as StakeAmount;
        let provider_payment = channel.rate - fee;

        channel.paid += channel.rate;
        channel.last_payment_epoch = epoch;
        channel.inference_count += 1;
        self.network_fees += fee;

        Ok(provider_payment)
    }

    /// Initiate channel close (starts settlement window).
    pub fn initiate_close(
        &mut self,
        channel_id: u64,
        requester: Address,
        epoch: Epoch,
    ) -> Result<(), PaymentError> {
        let channel = self
            .channels
            .get_mut(&channel_id)
            .ok_or(PaymentError::NotFound(channel_id))?;

        if channel.state != ChannelState::Active {
            return Err(PaymentError::NotActive(channel_id));
        }

        if requester != channel.payer && requester != channel.provider {
            return Err(PaymentError::NotParticipant);
        }

        channel.state = ChannelState::Settling { close_epoch: epoch };
        Ok(())
    }

    /// Finalize channel close (after settlement window).
    /// Returns (payer_refund, provider_payout).
    pub fn finalize_close(
        &mut self,
        channel_id: u64,
        epoch: Epoch,
    ) -> Result<(StakeAmount, StakeAmount), PaymentError> {
        let channel = self
            .channels
            .get_mut(&channel_id)
            .ok_or(PaymentError::NotFound(channel_id))?;

        match channel.state {
            ChannelState::Settling { close_epoch } => {
                if epoch < close_epoch + SETTLE_WINDOW {
                    return Err(PaymentError::SettleWindowOpen {
                        closes_at: close_epoch + SETTLE_WINDOW,
                    });
                }

                let payer_refund = channel.balance();
                let provider_payout = channel.paid;
                channel.state = ChannelState::Closed;

                Ok((payer_refund, provider_payout))
            }
            _ => Err(PaymentError::NotSettling(channel_id)),
        }
    }

    /// Top up an active channel with additional funds.
    pub fn top_up(
        &mut self,
        channel_id: u64,
        amount: StakeAmount,
    ) -> Result<(), PaymentError> {
        let channel = self
            .channels
            .get_mut(&channel_id)
            .ok_or(PaymentError::NotFound(channel_id))?;

        if channel.state != ChannelState::Active {
            return Err(PaymentError::NotActive(channel_id));
        }

        channel.locked += amount;
        Ok(())
    }

    /// Get a channel by ID.
    pub fn get(&self, id: u64) -> Option<&PaymentChannel> {
        self.channels.get(&id)
    }

    /// Count active channels.
    pub fn active_count(&self) -> usize {
        self.channels
            .values()
            .filter(|c| c.state == ChannelState::Active)
            .count()
    }

    /// Total value locked across all active channels.
    pub fn total_locked(&self) -> StakeAmount {
        self.channels
            .values()
            .filter(|c| matches!(c.state, ChannelState::Active | ChannelState::Settling { .. }))
            .map(|c| c.balance())
            .sum()
    }
}

#[derive(Debug)]
pub enum PaymentError {
    NotFound(u64),
    SelfChannel,
    ZeroLock,
    ZeroRate,
    NotActive(u64),
    NotSettling(u64),
    NotParticipant,
    InsufficientFunds { balance: StakeAmount, required: StakeAmount },
    SettleWindowOpen { closes_at: Epoch },
}

impl std::fmt::Display for PaymentError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NotFound(id) => write!(f, "channel {id} not found"),
            Self::SelfChannel => write!(f, "cannot create channel with yourself"),
            Self::ZeroLock => write!(f, "must lock non-zero funds"),
            Self::ZeroRate => write!(f, "rate must be non-zero"),
            Self::NotActive(id) => write!(f, "channel {id} is not active"),
            Self::NotSettling(id) => write!(f, "channel {id} is not settling"),
            Self::NotParticipant => write!(f, "not a participant in this channel"),
            Self::InsufficientFunds { balance, required } => {
                write!(f, "insufficient funds: {balance} available, {required} required")
            }
            Self::SettleWindowOpen { closes_at } => {
                write!(f, "settlement window open until epoch {closes_at}")
            }
        }
    }
}

impl std::error::Error for PaymentError {}

#[cfg(test)]
mod tests {
    use super::*;

    fn setup() -> PaymentManager {
        PaymentManager::new()
    }

    #[test]
    fn test_open_channel() {
        let mut mgr = setup();
        let id = mgr
            .open_channel(Address::test(1), Address::test(2), 1_000_000, 1000, 100)
            .unwrap();

        let ch = mgr.get(id).unwrap();
        assert_eq!(ch.balance(), 1_000_000);
        assert_eq!(ch.remaining_inferences(), 1000);
        assert_eq!(ch.state, ChannelState::Active);
    }

    #[test]
    fn test_self_channel_rejected() {
        let mut mgr = setup();
        assert!(mgr
            .open_channel(Address::test(1), Address::test(1), 1000, 100, 100)
            .is_err());
    }

    #[test]
    fn test_pay_inference() {
        let mut mgr = setup();
        let id = mgr
            .open_channel(Address::test(1), Address::test(2), 100_000, 1000, 100)
            .unwrap();

        let payment = mgr.pay_inference(id, 101).unwrap();

        // Rate 1000, fee 0.5% = 5, provider gets 995
        assert_eq!(payment, 995);
        assert_eq!(mgr.network_fees, 5);

        let ch = mgr.get(id).unwrap();
        assert_eq!(ch.balance(), 99_000);
        assert_eq!(ch.inference_count, 1);
    }

    #[test]
    fn test_pay_until_empty() {
        let mut mgr = setup();
        let id = mgr
            .open_channel(Address::test(1), Address::test(2), 3000, 1000, 100)
            .unwrap();

        mgr.pay_inference(id, 101).unwrap();
        mgr.pay_inference(id, 102).unwrap();
        mgr.pay_inference(id, 103).unwrap();

        // Fourth payment should fail
        assert!(mgr.pay_inference(id, 104).is_err());

        let ch = mgr.get(id).unwrap();
        assert_eq!(ch.inference_count, 3);
        assert_eq!(ch.balance(), 0);
    }

    #[test]
    fn test_close_lifecycle() {
        let mut mgr = setup();
        let id = mgr
            .open_channel(Address::test(1), Address::test(2), 10_000, 1000, 100)
            .unwrap();

        // Pay for 3 inferences
        for e in 101..104 {
            mgr.pay_inference(id, e).unwrap();
        }

        // Initiate close
        mgr.initiate_close(id, Address::test(1), 200).unwrap();

        // Too early to finalize
        assert!(mgr.finalize_close(id, 201).is_err());

        // After settlement window
        let (refund, payout) = mgr.finalize_close(id, 200 + SETTLE_WINDOW).unwrap();
        assert_eq!(payout, 3000); // 3 × 1000 rate
        assert_eq!(refund, 7000); // 10000 - 3000
    }

    #[test]
    fn test_top_up() {
        let mut mgr = setup();
        let id = mgr
            .open_channel(Address::test(1), Address::test(2), 5000, 1000, 100)
            .unwrap();

        assert_eq!(mgr.get(id).unwrap().remaining_inferences(), 5);

        mgr.top_up(id, 5000).unwrap();
        assert_eq!(mgr.get(id).unwrap().remaining_inferences(), 10);
    }

    #[test]
    fn test_network_fee_accumulation() {
        let mut mgr = setup();
        let id = mgr
            .open_channel(Address::test(1), Address::test(2), 100_000, 10_000, 100)
            .unwrap();

        for i in 0..5 {
            mgr.pay_inference(id, 100 + i).unwrap();
        }

        // 5 payments × 10,000 rate × 0.5% fee = 250
        assert_eq!(mgr.network_fees, 250);
    }

    #[test]
    fn test_total_locked() {
        let mut mgr = setup();
        mgr.open_channel(Address::test(1), Address::test(2), 10_000, 100, 100)
            .unwrap();
        mgr.open_channel(Address::test(3), Address::test(4), 20_000, 200, 100)
            .unwrap();

        assert_eq!(mgr.total_locked(), 30_000);
        assert_eq!(mgr.active_count(), 2);
    }

    #[test]
    fn test_cannot_pay_settling_channel() {
        let mut mgr = setup();
        let id = mgr
            .open_channel(Address::test(1), Address::test(2), 10_000, 1000, 100)
            .unwrap();

        mgr.initiate_close(id, Address::test(1), 200).unwrap();
        assert!(mgr.pay_inference(id, 201).is_err());
    }
}
