// chain/src/lib.rs — Prova chain simulation
//
// Models the on-chain state for QBP dispute resolution:
// - Model registry (registered models with weight hashes)
// - Inference commits (provider publishes activation root)
// - Challenge window (anyone can dispute within window)
// - Bisection game (on-chain referee for QBP protocol)

pub mod audit;
pub mod executor;
pub mod gas;
pub mod governance;
pub mod block;
pub mod commit;
pub mod dispute;
pub mod epoch;
pub mod genesis;
pub mod mempool;
pub mod payment;
pub mod registry;
pub mod rewards;
pub mod scheduler;
pub mod simulation;
pub mod sla;
pub mod stake;
pub mod state;
pub mod types;

#[cfg(test)]
mod integration_test;
