//! Bisection Dispute Game — on-chain QBP referee.
//!
//! When a challenger disputes an inference commit, the bisection game
//! narrows down to the single disputed layer in O(log L) rounds.
//!
//! Protocol:
//! 1. Challenger opens dispute with their own activation root
//! 2. Chain picks midpoint, asks both parties for activation hash at midpoint
//! 3. If they agree at midpoint → disputed layer is in upper half
//!    If they disagree → disputed layer is in lower half
//! 4. Repeat until interval is [i, i+1]
//! 5. Both parties submit full activation tensors for layer i and i+1
//! 6. Chain (or designated verifier) re-executes single layer to determine winner

use crate::types::*;
use std::collections::HashMap;

/// Configuration for the dispute game.
#[derive(Debug, Clone)]
pub struct DisputeConfig {
    /// Maximum epochs per bisection round before forfeit.
    pub round_timeout: EpochDuration,
    /// Maximum total epochs for entire dispute.
    pub total_timeout: EpochDuration,
}

impl Default for DisputeConfig {
    fn default() -> Self {
        Self {
            round_timeout: 60,  // ~30 minutes
            total_timeout: 480, // ~4 hours
        }
    }
}

/// Current state of the bisection.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DisputePhase {
    /// Waiting for both parties to submit activation hash at midpoint.
    AwaitingMidpoint {
        lo: u32,
        hi: u32,
        mid: u32,
        provider_hash: Option<Hash>,
        challenger_hash: Option<Hash>,
    },
    /// Narrowed to single layer — waiting for full activations.
    AwaitingActivations {
        disputed_layer: u32,
        provider_activation: Option<Hash>,
        challenger_activation: Option<Hash>,
    },
    /// Waiting for verifier to execute and judge.
    AwaitingJudgment { disputed_layer: u32 },
    /// Resolved — provider was correct.
    ResolvedProviderWins,
    /// Resolved — challenger was correct.
    ResolvedChallengerWins,
    /// Timed out — non-responding party loses.
    TimedOut { loser: Address },
}

/// A bisection dispute instance.
#[derive(Debug, Clone)]
pub struct Dispute {
    /// Unique dispute ID.
    pub id: u64,
    /// The commit being disputed.
    pub commit_id: CommitId,
    /// Provider address.
    pub provider: Address,
    /// Challenger address.
    pub challenger: Address,
    /// Model being disputed.
    pub model_id: ModelId,
    /// Architecture group.
    pub arch_group: ArchGroup,
    /// Provider's claimed activation root.
    pub provider_root: Hash,
    /// Challenger's claimed activation root.
    pub challenger_root: Hash,
    /// Total leaf count (input + layers).
    pub leaf_count: u32,
    /// Current bisection state.
    pub phase: DisputePhase,
    /// Epoch when dispute started.
    pub started_at: Epoch,
    /// Epoch of last action.
    pub last_action_at: Epoch,
    /// Number of bisection rounds completed.
    pub rounds: u32,
}

impl Dispute {
    /// Calculate the expected number of bisection rounds for this dispute.
    pub fn expected_rounds(&self) -> u32 {
        // ceil(log2(leaf_count))
        let n = self.leaf_count.next_power_of_two();
        n.trailing_zeros()
    }
}

/// Manages all active disputes.
#[derive(Debug)]
pub struct DisputeArena {
    disputes: HashMap<u64, Dispute>,
    next_id: u64,
    config: DisputeConfig,
}

impl DisputeArena {
    pub fn new(config: DisputeConfig) -> Self {
        Self {
            disputes: HashMap::new(),
            next_id: 1,
            config,
        }
    }

    /// Open a new dispute against an inference commit.
    pub fn open_dispute(
        &mut self,
        commit_id: CommitId,
        provider: Address,
        challenger: Address,
        model_id: ModelId,
        arch_group: ArchGroup,
        provider_root: Hash,
        challenger_root: Hash,
        leaf_count: u32,
        current_epoch: Epoch,
    ) -> Result<u64, DisputeError> {
        if provider == challenger {
            return Err(DisputeError::SelfDispute);
        }

        if provider_root == challenger_root {
            return Err(DisputeError::RootsMatch);
        }

        if leaf_count < 2 {
            return Err(DisputeError::TooFewLayers);
        }

        let id = self.next_id;
        self.next_id += 1;

        let lo = 0u32;
        let hi = leaf_count - 1;
        let mid = (lo + hi) / 2;

        let dispute = Dispute {
            id,
            commit_id,
            provider,
            challenger,
            model_id,
            arch_group,
            provider_root,
            challenger_root,
            leaf_count,
            phase: DisputePhase::AwaitingMidpoint {
                lo,
                hi,
                mid,
                provider_hash: None,
                challenger_hash: None,
            },
            started_at: current_epoch,
            last_action_at: current_epoch,
            rounds: 0,
        };

        self.disputes.insert(id, dispute);
        Ok(id)
    }

    /// Submit a midpoint activation hash.
    pub fn submit_midpoint(
        &mut self,
        dispute_id: u64,
        submitter: Address,
        hash: Hash,
        current_epoch: Epoch,
    ) -> Result<BisectionStep, DisputeError> {
        let dispute = self
            .disputes
            .get_mut(&dispute_id)
            .ok_or(DisputeError::NotFound(dispute_id))?;

        // Check round timeout
        if current_epoch > dispute.last_action_at + self.config.round_timeout {
            let loser = if submitter == dispute.provider {
                // Provider responding late — but check if challenger also didn't submit
                dispute.challenger
            } else {
                dispute.provider
            };
            dispute.phase = DisputePhase::TimedOut { loser };
            return Ok(BisectionStep::TimedOut { loser });
        }

        match &mut dispute.phase {
            DisputePhase::AwaitingMidpoint {
                lo,
                hi,
                mid,
                provider_hash,
                challenger_hash,
            } => {
                if submitter == dispute.provider {
                    if provider_hash.is_some() {
                        return Err(DisputeError::AlreadySubmitted);
                    }
                    *provider_hash = Some(hash);
                } else if submitter == dispute.challenger {
                    if challenger_hash.is_some() {
                        return Err(DisputeError::AlreadySubmitted);
                    }
                    *challenger_hash = Some(hash);
                } else {
                    return Err(DisputeError::NotParticipant);
                }

                // Check if both have submitted
                if let (Some(p_hash), Some(c_hash)) = (provider_hash, challenger_hash) {
                    let agree = p_hash == c_hash;
                    let (new_lo, new_hi) = if agree {
                        // Agree at midpoint → dispute is in upper half
                        (*mid, *hi)
                    } else {
                        // Disagree at midpoint → dispute is in lower half
                        (*lo, *mid)
                    };

                    dispute.rounds += 1;
                    dispute.last_action_at = current_epoch;

                    if new_hi - new_lo <= 1 {
                        // Narrowed to single layer transition
                        dispute.phase = DisputePhase::AwaitingActivations {
                            disputed_layer: new_hi,
                            provider_activation: None,
                            challenger_activation: None,
                        };
                        Ok(BisectionStep::NarrowedToLayer {
                            layer: new_hi,
                            rounds_taken: dispute.rounds,
                        })
                    } else {
                        let new_mid = (new_lo + new_hi) / 2;
                        dispute.phase = DisputePhase::AwaitingMidpoint {
                            lo: new_lo,
                            hi: new_hi,
                            mid: new_mid,
                            provider_hash: None,
                            challenger_hash: None,
                        };
                        Ok(BisectionStep::Narrowed {
                            lo: new_lo,
                            hi: new_hi,
                            mid: new_mid,
                            agreed: agree,
                        })
                    }
                } else {
                    Ok(BisectionStep::WaitingForOther)
                }
            }
            _ => Err(DisputeError::WrongPhase),
        }
    }

    /// Submit full activation data for final verification.
    pub fn submit_activation(
        &mut self,
        dispute_id: u64,
        submitter: Address,
        activation_hash: Hash,
        current_epoch: Epoch,
    ) -> Result<bool, DisputeError> {
        let dispute = self
            .disputes
            .get_mut(&dispute_id)
            .ok_or(DisputeError::NotFound(dispute_id))?;

        match &mut dispute.phase {
            DisputePhase::AwaitingActivations {
                disputed_layer,
                provider_activation,
                challenger_activation,
            } => {
                if submitter == dispute.provider {
                    *provider_activation = Some(activation_hash);
                } else if submitter == dispute.challenger {
                    *challenger_activation = Some(activation_hash);
                } else {
                    return Err(DisputeError::NotParticipant);
                }

                if provider_activation.is_some() && challenger_activation.is_some() {
                    let layer = *disputed_layer;
                    dispute.phase = DisputePhase::AwaitingJudgment {
                        disputed_layer: layer,
                    };
                    dispute.last_action_at = current_epoch;
                    Ok(true) // Ready for judgment
                } else {
                    Ok(false) // Still waiting
                }
            }
            _ => Err(DisputeError::WrongPhase),
        }
    }

    /// Judge the dispute (called by verifier after re-executing the single layer).
    pub fn judge(
        &mut self,
        dispute_id: u64,
        provider_correct: bool,
    ) -> Result<Address, DisputeError> {
        let dispute = self
            .disputes
            .get_mut(&dispute_id)
            .ok_or(DisputeError::NotFound(dispute_id))?;

        match dispute.phase {
            DisputePhase::AwaitingJudgment { .. } => {
                if provider_correct {
                    dispute.phase = DisputePhase::ResolvedProviderWins;
                    Ok(dispute.provider) // Winner
                } else {
                    dispute.phase = DisputePhase::ResolvedChallengerWins;
                    Ok(dispute.challenger) // Winner
                }
            }
            _ => Err(DisputeError::WrongPhase),
        }
    }

    /// Get a dispute by ID.
    pub fn get(&self, id: u64) -> Option<&Dispute> {
        self.disputes.get(&id)
    }

    /// Count active (unresolved) disputes.
    pub fn active_count(&self) -> usize {
        self.disputes
            .values()
            .filter(|d| {
                matches!(
                    d.phase,
                    DisputePhase::AwaitingMidpoint { .. }
                        | DisputePhase::AwaitingActivations { .. }
                        | DisputePhase::AwaitingJudgment { .. }
                )
            })
            .count()
    }
}

/// Result of a bisection step.
#[derive(Debug)]
pub enum BisectionStep {
    /// Waiting for the other party to submit.
    WaitingForOther,
    /// Interval narrowed, new midpoint to query.
    Narrowed {
        lo: u32,
        hi: u32,
        mid: u32,
        agreed: bool,
    },
    /// Bisection complete — isolated single layer.
    NarrowedToLayer { layer: u32, rounds_taken: u32 },
    /// Round timed out — party forfeits.
    TimedOut { loser: Address },
}

#[derive(Debug)]
pub enum DisputeError {
    NotFound(u64),
    SelfDispute,
    RootsMatch,
    TooFewLayers,
    AlreadySubmitted,
    NotParticipant,
    WrongPhase,
}

impl std::fmt::Display for DisputeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NotFound(id) => write!(f, "dispute {id} not found"),
            Self::SelfDispute => write!(f, "cannot dispute your own commit"),
            Self::RootsMatch => write!(f, "roots match — no dispute needed"),
            Self::TooFewLayers => write!(f, "need at least 2 layers for bisection"),
            Self::AlreadySubmitted => write!(f, "already submitted for this round"),
            Self::NotParticipant => write!(f, "not a participant in this dispute"),
            Self::WrongPhase => write!(f, "dispute is not in the correct phase"),
        }
    }
}

impl std::error::Error for DisputeError {}

#[cfg(test)]
mod tests {
    use super::*;

    fn setup() -> DisputeArena {
        DisputeArena::new(DisputeConfig {
            round_timeout: 60,
            total_timeout: 480,
        })
    }

    #[test]
    fn test_open_dispute() {
        let mut arena = setup();
        let id = arena
            .open_dispute(
                CommitId(1),
                Address::test(1),
                Address::test(2),
                ModelId([0xAA; 32]),
                ArchGroup::new("test"),
                [0x11; 32],
                [0x22; 32],
                33, // 32 layers + input
                1000,
            )
            .unwrap();

        let dispute = arena.get(id).unwrap();
        assert_eq!(dispute.rounds, 0);
        assert_eq!(dispute.leaf_count, 33);
        // log2(64) = 6 expected rounds (33 padded to 64)
        assert_eq!(dispute.expected_rounds(), 6);
    }

    #[test]
    fn test_self_dispute_rejected() {
        let mut arena = setup();
        let result = arena.open_dispute(
            CommitId(1),
            Address::test(1),
            Address::test(1),
            ModelId([0; 32]),
            ArchGroup::new("test"),
            [0x11; 32],
            [0x22; 32],
            33,
            1000,
        );
        assert!(result.is_err());
    }

    #[test]
    fn test_full_bisection_game() {
        let mut arena = setup();
        let dispute_id = arena
            .open_dispute(
                CommitId(1),
                Address::test(1), // provider
                Address::test(2), // challenger
                ModelId([0xAA; 32]),
                ArchGroup::new("test"),
                [0x11; 32], // provider root
                [0x22; 32], // challenger root
                8,          // 8 leaves (7 layers + input)
                1000,
            )
            .unwrap();

        // 8 leaves → need log2(8) = 3 rounds
        let epoch = 1001;

        // Simulate bisection where dispute is at layer 5
        // Initial: lo=0, hi=7, mid=3
        // Round 1: both agree at 3 → lo=3, hi=7, mid=5
        let agree_hash = [0xAA; 32];
        let step = arena
            .submit_midpoint(dispute_id, Address::test(1), agree_hash, epoch)
            .unwrap();
        assert!(matches!(step, BisectionStep::WaitingForOther));

        let step = arena
            .submit_midpoint(dispute_id, Address::test(2), agree_hash, epoch)
            .unwrap();
        match step {
            BisectionStep::Narrowed {
                lo,
                hi,
                mid,
                agreed,
            } => {
                assert_eq!(lo, 3);
                assert_eq!(hi, 7);
                assert_eq!(mid, 5);
                assert!(agreed);
            }
            _ => panic!("expected Narrowed"),
        }

        // Round 2: disagree at 5 → lo=3, hi=5, mid=4
        let step = arena
            .submit_midpoint(dispute_id, Address::test(1), [0xBB; 32], epoch)
            .unwrap();
        assert!(matches!(step, BisectionStep::WaitingForOther));

        let step = arena
            .submit_midpoint(dispute_id, Address::test(2), [0xCC; 32], epoch)
            .unwrap();
        match step {
            BisectionStep::Narrowed {
                lo,
                hi,
                mid,
                agreed,
            } => {
                assert_eq!(lo, 3);
                assert_eq!(hi, 5);
                assert_eq!(mid, 4);
                assert!(!agreed);
            }
            _ => panic!("expected Narrowed"),
        }

        // Round 3: agree at 4 → lo=4, hi=5 → narrowed to single layer!
        let step = arena
            .submit_midpoint(dispute_id, Address::test(1), agree_hash, epoch)
            .unwrap();
        let step = arena
            .submit_midpoint(dispute_id, Address::test(2), agree_hash, epoch)
            .unwrap();
        match step {
            BisectionStep::NarrowedToLayer {
                layer,
                rounds_taken,
            } => {
                assert_eq!(layer, 5);
                assert_eq!(rounds_taken, 3);
            }
            _ => panic!("expected NarrowedToLayer"),
        }

        // Submit activations
        let ready = arena
            .submit_activation(dispute_id, Address::test(1), [0xDD; 32], epoch)
            .unwrap();
        assert!(!ready);
        let ready = arena
            .submit_activation(dispute_id, Address::test(2), [0xEE; 32], epoch)
            .unwrap();
        assert!(ready);

        // Judge: provider was correct
        let winner = arena.judge(dispute_id, true).unwrap();
        assert_eq!(winner, Address::test(1));

        let dispute = arena.get(dispute_id).unwrap();
        assert!(matches!(dispute.phase, DisputePhase::ResolvedProviderWins));
    }

    #[test]
    fn test_challenger_wins() {
        let mut arena = setup();
        let dispute_id = arena
            .open_dispute(
                CommitId(1),
                Address::test(1),
                Address::test(2),
                ModelId([0; 32]),
                ArchGroup::new("test"),
                [0x11; 32],
                [0x22; 32],
                2, // Minimal: just input + 1 layer
                1000,
            )
            .unwrap();

        // 2 leaves → lo=0, hi=1, mid=0
        // Disagree at 0 → narrowed to layer 0..1 immediately?
        // Actually lo=0, hi=1 means hi-lo=1, so after first disagreement it narrows
        let step = arena
            .submit_midpoint(dispute_id, Address::test(1), [0xAA; 32], 1001)
            .unwrap();
        let step = arena
            .submit_midpoint(dispute_id, Address::test(2), [0xBB; 32], 1001)
            .unwrap();
        match step {
            BisectionStep::NarrowedToLayer {
                layer,
                rounds_taken,
            } => {
                assert_eq!(rounds_taken, 1);
            }
            _ => panic!("expected NarrowedToLayer"),
        }

        // Submit activations and judge: challenger wins
        arena
            .submit_activation(dispute_id, Address::test(1), [0xDD; 32], 1002)
            .unwrap();
        arena
            .submit_activation(dispute_id, Address::test(2), [0xEE; 32], 1002)
            .unwrap();

        let winner = arena.judge(dispute_id, false).unwrap();
        assert_eq!(winner, Address::test(2));
    }

    #[test]
    fn test_32_layer_model_rounds() {
        // 33 leaves (32 layers + input) → padded to 64 → 6 rounds
        let mut arena = setup();
        let dispute_id = arena
            .open_dispute(
                CommitId(1),
                Address::test(1),
                Address::test(2),
                ModelId([0; 32]),
                ArchGroup::new("test"),
                [0x11; 32],
                [0x22; 32],
                33,
                1000,
            )
            .unwrap();

        let dispute = arena.get(dispute_id).unwrap();
        assert_eq!(dispute.expected_rounds(), 6);
    }
}
