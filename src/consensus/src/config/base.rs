use std::default::Default;

use alloy::primitives::B256;
use helios_core::fork_schedule::ForkSchedule;
use serde::Serialize;

use helios_consensus_core::types::Forks;

use crate::config::types::ChainConfig;

/// The base configuration for a network.
#[derive(Serialize)]
pub struct BaseConfig {
    pub consensus_rpc: Option<String>,
    pub default_checkpoint: B256,
    pub chain: ChainConfig,
    pub forks: Forks,
    pub execution_forks: ForkSchedule,
    pub max_checkpoint_age: u64,
    pub load_external_fallback: bool,
    pub strict_checkpoint_age: bool,
}

impl Default for BaseConfig {
    fn default() -> Self {
        BaseConfig {
            consensus_rpc: None,
            default_checkpoint: B256::ZERO,
            chain: Default::default(),
            forks: Default::default(),
            max_checkpoint_age: 0,
            execution_forks: ForkSchedule::default(),
            load_external_fallback: false,
            strict_checkpoint_age: false,
        }
    }
}
