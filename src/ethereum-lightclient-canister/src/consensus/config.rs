use serde::{Deserialize, Serialize};
use tree_hash::fixed_bytes::FixedBytes;

#[derive(Deserialize, Debug, Default)]
pub struct Config {
    pub consensus_rpc: String,
    pub execution_rpc: String,
    pub default_checkpoint: String,
    pub checkpoint: Option<String>,
    pub chain: ChainConfig,
    pub forks: Forks,
    pub execution_forks: ForkSchedule,
    pub max_checkpoint_age: u64,
    pub fallback: Option<String>,
    pub load_external_fallback: bool,
    pub strict_checkpoint_age: bool,
    pub database_type: Option<String>,
}


#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct ChainConfig {
    pub chain_id: u64,
    pub genesis_time: u64,
    pub genesis_root: String,
}


#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct Forks {
    pub genesis: Fork,
    pub altair: Fork,
    pub bellatrix: Fork,
    pub capella: Fork,
    pub deneb: Fork,
    pub electra: Fork,
}

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct Fork {
    pub epoch: u64,
    pub fork_version: FixedBytes<4>,
}



#[derive(Clone, Copy, Serialize, Deserialize, Default, Debug)]
pub struct ForkSchedule {
    pub prague_timestamp: u64,
}

