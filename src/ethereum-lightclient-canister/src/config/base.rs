use std::default::Default;
use std::net::{IpAddr, Ipv4Addr};
use std::path::PathBuf;

use crate::config::{ChainConfig, Config, ForkSchedule, Forks};
use serde::Serialize;
use tree_hash::fixed_bytes::B256;

/// The base configuration for a network.
#[derive(Serialize)]
pub struct BaseConfig {
    pub rpc_bind_ip: IpAddr,
    pub rpc_port: u16,
    pub consensus_rpc: Option<String>,
    pub default_checkpoint: B256,
    pub chain: ChainConfig,
    pub forks: Forks,
    pub execution_forks: ForkSchedule,
    pub max_checkpoint_age: u64,
    pub data_dir: Option<PathBuf>,
    pub load_external_fallback: bool,
    pub strict_checkpoint_age: bool,
}

impl Default for BaseConfig {
    fn default() -> Self {
        BaseConfig {
            rpc_bind_ip: IpAddr::V4(Ipv4Addr::LOCALHOST),
            rpc_port: 0,
            consensus_rpc: None,
            default_checkpoint: B256::ZERO,
            chain: Default::default(),
            forks: Default::default(),
            max_checkpoint_age: 0,
            data_dir: None,
            execution_forks: ForkSchedule::default(),
            load_external_fallback: false,
            strict_checkpoint_age: false,
        }
    }
}

impl From<BaseConfig> for Config {
    fn from(base: BaseConfig) -> Self {
        Config {
            consensus_rpc: base.consensus_rpc.unwrap_or_default(),
            execution_rpc: String::new(),
            checkpoint: None,
            default_checkpoint: base.default_checkpoint,
            chain: base.chain,
            forks: base.forks,
            execution_forks: base.execution_forks,
            max_checkpoint_age: base.max_checkpoint_age,
            fallback: None,
            load_external_fallback: base.load_external_fallback,
            strict_checkpoint_age: base.strict_checkpoint_age,
        }
    }
}
