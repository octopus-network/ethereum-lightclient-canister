use crate::consensus::consensus_spec::ConsensusSpec;

//pub mod consensus;
pub mod core;
pub mod config;
pub mod verify;
pub mod consensus_spec;

pub fn calc_sync_period<S: ConsensusSpec>(slot: u64) -> u64 {
    let epoch = slot / S::slots_per_epoch();
    epoch / S::epochs_per_sync_commitee_period()
}