// chain/src/lib.rs — Prova chain simulation
//
// Models the on-chain state for QBP dispute resolution:
// - Model registry (registered models with weight hashes)
// - Inference commits (provider publishes activation root)
// - Challenge window (anyone can dispute within window)
// - Bisection game (on-chain referee for QBP protocol)

pub mod types;
pub mod registry;
pub mod commit;
pub mod dispute;
pub mod stake;
pub mod payment;
pub mod simulation;
