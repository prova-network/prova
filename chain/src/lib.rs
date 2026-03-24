#![allow(non_snake_case, dead_code, unused_imports, unused_variables, unused_assignments, unused_mut, clippy::all)]
// chain/src/lib.rs — Prova chain simulation
//
// Models the on-chain state for QBP dispute resolution:
// - Model registry (registered models with weight hashes)
// - Inference commits (provider publishes activation root)
// - Challenge window (anyone can dispute within window)
// - Bisection game (on-chain referee for QBP protocol)

pub mod access;
pub mod auction;
pub mod bridge;
pub mod events;
pub mod checkpoint;
pub mod audit;
pub mod executor;
pub mod finality;
pub mod gas;
pub mod invariants;
pub mod governance;
pub mod block;
pub mod commit;
pub mod dispute;
pub mod epoch;
pub mod genesis;
pub mod mempool;
pub mod payment;
pub mod pruning;
pub mod rate_limiter;
pub mod receipts;
pub mod registry;
pub mod reputation;
pub mod rewards;
pub mod scheduler;
pub mod simulation;
pub mod snapshot;
pub mod sla;
pub mod stake;
pub mod state;
pub mod types;
pub mod das;
pub mod delegation;
pub mod delegation_gov;
pub mod liquid_staking;
pub mod blob_tx;
pub mod marketplace;
pub mod upgrade;
pub mod validator_set;
pub mod confidential;
pub mod zk_verifier;
pub mod multisig;
pub mod network_sim;
pub mod chaos;
pub mod load_test;
pub mod viz;
pub mod adversarial_net;
pub mod benchmark;
pub mod bench_history;
pub mod migration;

#[cfg(test)]
mod integration_test;
#[cfg(test)]
mod adversarial_test;
#[cfg(test)]
mod das_adversarial_test;
