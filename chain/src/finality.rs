//! Finality gadget — dual-layer finality for Prova.
//!
//! Provides two finality levels:
//! - **Fast finality:** Achieved when a checkpoint is finalized by 2/3+ validator stake
//!   (optimistic, within ~120 epochs / ~1 hour)
//! - **Slow (L1) finality:** Achieved when the checkpoint is anchored on Filecoin L1
//!   and the L1 block reaches sufficient depth (e.g., 900 epochs / ~7.5 hours)
//!
//! Design:
//! - Tracks per-block finality status: Tentative → FastFinal → L1Final
//! - Blocks inherit finality from their covering checkpoint
//! - Provides finality queries for light clients and cross-chain bridges

use std::collections::BTreeMap;

/// Finality level for a Prova block or transaction.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum FinalityLevel {
    /// Block produced but checkpoint not yet finalized.
    Tentative,
    /// Checkpoint finalized by validator quorum (2/3+ stake).
    FastFinal,
    /// Checkpoint anchored on Filecoin L1 with sufficient depth.
    L1Final,
}

impl std::fmt::Display for FinalityLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Tentative => write!(f, "tentative"),
            Self::FastFinal => write!(f, "fast-final"),
            Self::L1Final => write!(f, "L1-final"),
        }
    }
}

/// Minimum L1 depth before considering an anchor as L1-final.
pub const L1_FINALITY_DEPTH: u64 = 900; // ~7.5 hours at 30s Filecoin epochs

/// Record of a finalized checkpoint and its anchoring status.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FinalityRecord {
    /// Checkpoint sequence number.
    pub sequence: u64,
    /// Prova epoch range covered.
    pub epoch_start: u64,
    pub epoch_end: u64,
    /// When the checkpoint was finalized (Prova epoch).
    pub finalized_at: u64,
    /// L1 anchoring info (if anchored).
    pub anchor: Option<AnchorInfo>,
}

/// L1 anchor information.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AnchorInfo {
    /// L1 epoch where the anchor tx was included.
    pub l1_epoch: u64,
    /// L1 tx hash.
    pub tx_hash: [u8; 32],
}

/// The finality gadget tracks finality status for all blocks.
#[derive(Debug)]
pub struct FinalityGadget {
    /// Finality records by checkpoint sequence.
    pub records: BTreeMap<u64, FinalityRecord>,
    /// Current L1 head epoch (updated by L1 watcher).
    pub l1_head: u64,
    /// Latest Prova epoch seen.
    pub prova_head: u64,
    /// Configurable L1 finality depth.
    pub finality_depth: u64,
}

impl FinalityGadget {
    pub fn new() -> Self {
        Self {
            records: BTreeMap::new(),
            l1_head: 0,
            prova_head: 0,
            finality_depth: L1_FINALITY_DEPTH,
        }
    }

    pub fn with_finality_depth(depth: u64) -> Self {
        Self {
            finality_depth: depth,
            ..Self::new()
        }
    }

    /// Register a finalized checkpoint.
    pub fn register_checkpoint(
        &mut self,
        sequence: u64,
        epoch_start: u64,
        epoch_end: u64,
        finalized_at: u64,
    ) -> Result<(), FinalityError> {
        if self.records.contains_key(&sequence) {
            return Err(FinalityError::DuplicateCheckpoint(sequence));
        }
        // Ensure sequential
        if let Some((&last_seq, _)) = self.records.iter().next_back() {
            if sequence != last_seq + 1 {
                return Err(FinalityError::NonSequential {
                    expected: last_seq + 1,
                    got: sequence,
                });
            }
        }
        self.records.insert(
            sequence,
            FinalityRecord {
                sequence,
                epoch_start,
                epoch_end,
                finalized_at,
                anchor: None,
            },
        );
        if epoch_end > self.prova_head {
            self.prova_head = epoch_end;
        }
        Ok(())
    }

    /// Record that a checkpoint has been anchored on L1.
    pub fn record_anchor(
        &mut self,
        sequence: u64,
        l1_epoch: u64,
        tx_hash: [u8; 32],
    ) -> Result<(), FinalityError> {
        let record = self
            .records
            .get_mut(&sequence)
            .ok_or(FinalityError::UnknownCheckpoint(sequence))?;
        if record.anchor.is_some() {
            return Err(FinalityError::AlreadyAnchored(sequence));
        }
        record.anchor = Some(AnchorInfo { l1_epoch, tx_hash });
        Ok(())
    }

    /// Update the known L1 head epoch.
    pub fn update_l1_head(&mut self, l1_epoch: u64) {
        if l1_epoch > self.l1_head {
            self.l1_head = l1_epoch;
        }
    }

    /// Query the finality level of a specific Prova epoch.
    pub fn finality_of(&self, epoch: u64) -> FinalityLevel {
        // Find the checkpoint covering this epoch
        let record = self.records.values().find(|r| {
            epoch >= r.epoch_start && epoch <= r.epoch_end
        });

        match record {
            None => FinalityLevel::Tentative,
            Some(r) => match &r.anchor {
                Some(anchor) => {
                    if self.l1_head >= anchor.l1_epoch + self.finality_depth {
                        FinalityLevel::L1Final
                    } else {
                        FinalityLevel::FastFinal
                    }
                }
                None => FinalityLevel::FastFinal,
            },
        }
    }

    /// Get the highest epoch that has reached a given finality level.
    pub fn highest_at_level(&self, level: FinalityLevel) -> Option<u64> {
        let mut highest = None;
        for record in self.records.values().rev() {
            let finality = match &record.anchor {
                Some(anchor) => {
                    if self.l1_head >= anchor.l1_epoch + self.finality_depth {
                        FinalityLevel::L1Final
                    } else {
                        FinalityLevel::FastFinal
                    }
                }
                None => FinalityLevel::FastFinal,
            };
            if finality >= level {
                highest = Some(record.epoch_end);
                break;
            }
        }
        highest
    }

    /// Get the highest L1-final epoch.
    pub fn l1_final_epoch(&self) -> Option<u64> {
        self.highest_at_level(FinalityLevel::L1Final)
    }

    /// Get the highest fast-final epoch.
    pub fn fast_final_epoch(&self) -> Option<u64> {
        self.highest_at_level(FinalityLevel::FastFinal)
    }

    /// Count checkpoints at each finality level.
    pub fn finality_summary(&self) -> (usize, usize, usize) {
        let mut tentative = 0;
        let mut fast = 0;
        let mut l1 = 0;
        for record in self.records.values() {
            match &record.anchor {
                Some(anchor) => {
                    if self.l1_head >= anchor.l1_epoch + self.finality_depth {
                        l1 += 1;
                    } else {
                        fast += 1;
                    }
                }
                None => fast += 1,
            }
        }
        // Tentative = epochs beyond any checkpoint coverage
        // (not tracked here since we only track checkpointed ranges)
        (tentative, fast, l1)
    }

    /// Total registered checkpoints.
    pub fn checkpoint_count(&self) -> usize {
        self.records.len()
    }

    /// Check if a specific checkpoint is L1-final.
    pub fn is_l1_final(&self, sequence: u64) -> bool {
        self.records.get(&sequence).map_or(false, |r| {
            r.anchor.as_ref().map_or(false, |a| {
                self.l1_head >= a.l1_epoch + self.finality_depth
            })
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FinalityError {
    DuplicateCheckpoint(u64),
    NonSequential { expected: u64, got: u64 },
    UnknownCheckpoint(u64),
    AlreadyAnchored(u64),
}

impl std::fmt::Display for FinalityError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::DuplicateCheckpoint(s) => write!(f, "duplicate checkpoint {}", s),
            Self::NonSequential { expected, got } => {
                write!(f, "expected sequence {}, got {}", expected, got)
            }
            Self::UnknownCheckpoint(s) => write!(f, "unknown checkpoint {}", s),
            Self::AlreadyAnchored(s) => write!(f, "checkpoint {} already anchored", s),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn setup() -> FinalityGadget {
        let mut fg = FinalityGadget::with_finality_depth(10); // Short depth for testing
        fg.register_checkpoint(0, 1, 120, 120).unwrap();
        fg.register_checkpoint(1, 121, 240, 240).unwrap();
        fg.register_checkpoint(2, 241, 360, 360).unwrap();
        fg
    }

    #[test]
    fn test_register_checkpoint() {
        let fg = setup();
        assert_eq!(fg.checkpoint_count(), 3);
        assert_eq!(fg.prova_head, 360);
    }

    #[test]
    fn test_duplicate_rejected() {
        let mut fg = setup();
        let err = fg.register_checkpoint(0, 1, 120, 120).unwrap_err();
        assert_eq!(err, FinalityError::DuplicateCheckpoint(0));
    }

    #[test]
    fn test_non_sequential_rejected() {
        let mut fg = setup();
        let err = fg.register_checkpoint(5, 361, 480, 480).unwrap_err();
        assert_eq!(err, FinalityError::NonSequential { expected: 3, got: 5 });
    }

    #[test]
    fn test_tentative_before_checkpoint() {
        let fg = setup();
        // Epoch 500 is beyond any checkpoint
        assert_eq!(fg.finality_of(500), FinalityLevel::Tentative);
    }

    #[test]
    fn test_fast_final_after_checkpoint() {
        let fg = setup();
        assert_eq!(fg.finality_of(100), FinalityLevel::FastFinal);
        assert_eq!(fg.finality_of(200), FinalityLevel::FastFinal);
    }

    #[test]
    fn test_l1_final_after_anchor_depth() {
        let mut fg = setup();
        fg.record_anchor(0, 1000, [1u8; 32]).unwrap();
        fg.update_l1_head(1005);
        // Not deep enough yet
        assert_eq!(fg.finality_of(100), FinalityLevel::FastFinal);

        fg.update_l1_head(1010);
        // Now deep enough (1010 >= 1000 + 10)
        assert_eq!(fg.finality_of(100), FinalityLevel::L1Final);
    }

    #[test]
    fn test_anchor_unknown_checkpoint() {
        let mut fg = setup();
        let err = fg.record_anchor(99, 1000, [1u8; 32]).unwrap_err();
        assert_eq!(err, FinalityError::UnknownCheckpoint(99));
    }

    #[test]
    fn test_double_anchor_rejected() {
        let mut fg = setup();
        fg.record_anchor(0, 1000, [1u8; 32]).unwrap();
        let err = fg.record_anchor(0, 1001, [2u8; 32]).unwrap_err();
        assert_eq!(err, FinalityError::AlreadyAnchored(0));
    }

    #[test]
    fn test_is_l1_final() {
        let mut fg = setup();
        assert!(!fg.is_l1_final(0));
        fg.record_anchor(0, 1000, [1u8; 32]).unwrap();
        assert!(!fg.is_l1_final(0));
        fg.update_l1_head(1010);
        assert!(fg.is_l1_final(0));
        assert!(!fg.is_l1_final(1)); // Not anchored
    }

    #[test]
    fn test_highest_fast_final() {
        let fg = setup();
        assert_eq!(fg.fast_final_epoch(), Some(360));
    }

    #[test]
    fn test_highest_l1_final() {
        let mut fg = setup();
        assert_eq!(fg.l1_final_epoch(), None);
        fg.record_anchor(0, 1000, [1u8; 32]).unwrap();
        fg.record_anchor(1, 1005, [2u8; 32]).unwrap();
        fg.update_l1_head(1015);
        // Checkpoint 0 (anchor 1000): 1015 >= 1010 ✓
        // Checkpoint 1 (anchor 1005): 1015 >= 1015 ✓
        // Checkpoint 2: not anchored
        assert_eq!(fg.l1_final_epoch(), Some(240));
    }

    #[test]
    fn test_finality_summary() {
        let mut fg = setup();
        let (t, f, l) = fg.finality_summary();
        assert_eq!((t, f, l), (0, 3, 0));

        fg.record_anchor(0, 1000, [1u8; 32]).unwrap();
        fg.update_l1_head(1010);
        let (t, f, l) = fg.finality_summary();
        assert_eq!((t, f, l), (0, 2, 1));
    }

    #[test]
    fn test_finality_level_ordering() {
        assert!(FinalityLevel::Tentative < FinalityLevel::FastFinal);
        assert!(FinalityLevel::FastFinal < FinalityLevel::L1Final);
    }

    #[test]
    fn test_finality_display() {
        assert_eq!(FinalityLevel::Tentative.to_string(), "tentative");
        assert_eq!(FinalityLevel::FastFinal.to_string(), "fast-final");
        assert_eq!(FinalityLevel::L1Final.to_string(), "L1-final");
    }

    #[test]
    fn test_error_display() {
        assert_eq!(
            FinalityError::DuplicateCheckpoint(5).to_string(),
            "duplicate checkpoint 5"
        );
        assert_eq!(
            FinalityError::NonSequential { expected: 3, got: 5 }.to_string(),
            "expected sequence 3, got 5"
        );
    }

    #[test]
    fn test_update_l1_head_monotonic() {
        let mut fg = setup();
        fg.update_l1_head(100);
        assert_eq!(fg.l1_head, 100);
        fg.update_l1_head(50); // Should not decrease
        assert_eq!(fg.l1_head, 100);
        fg.update_l1_head(200);
        assert_eq!(fg.l1_head, 200);
    }

    #[test]
    fn test_empty_gadget() {
        let fg = FinalityGadget::new();
        assert_eq!(fg.checkpoint_count(), 0);
        assert_eq!(fg.finality_of(100), FinalityLevel::Tentative);
        assert_eq!(fg.fast_final_epoch(), None);
        assert_eq!(fg.l1_final_epoch(), None);
        assert_eq!(fg.finality_depth, L1_FINALITY_DEPTH);
    }
}
