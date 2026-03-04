// chain/src/lib.rs — Prova chain simulation
//
// Models the on-chain state for QBP dispute resolution:
// - Model registry (registered models with weight hashes)
// - Inference commits (provider publishes activation root)
// - Challenge window (anyone can dispute within window)
// - Bisection game (on-chain referee for QBP protocol)

pub mod audit;
pub mod commit;
pub mod dispute;
pub mod epoch;
pub mod payment;
pub mod registry;
pub mod simulation;
pub mod stake;
pub mod types;

#[cfg(test)]
mod integration_test;
