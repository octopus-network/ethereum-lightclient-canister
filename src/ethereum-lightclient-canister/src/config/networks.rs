use std::fmt::Display;
use std::str::FromStr;

use eyre::Result;
use serde::{Deserialize, Serialize};
use strum::EnumIter;
use tree_hash::fixed_bytes::{B256, FixedBytes};


use crate::config::base::BaseConfig;
use crate::config::{Fork, Forks, ForkSchedule};
use crate::config::ChainConfig;

#[derive(
Debug, Clone, Copy, Serialize, Deserialize, EnumIter, Hash, Eq, PartialEq, PartialOrd, Ord,
)]
pub enum Network {
    Mainnet,
    Sepolia,
    Holesky,
    PectraDevnet,
}

impl FromStr for Network {
    type Err = eyre::Report;

    fn from_str(s: &str) -> Result<Self> {
        match s {
            "mainnet" => Ok(Self::Mainnet),
            "sepolia" => Ok(Self::Sepolia),
            "holesky" => Ok(Self::Holesky),
            "pectra-devnet" => Ok(Self::PectraDevnet),
            _ => Err(eyre::eyre!("network not recognized")),
        }
    }
}

impl Display for Network {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let str = match self {
            Self::Mainnet => "mainnet",
            Self::Sepolia => "sepolia",
            Self::Holesky => "holesky",
            Self::PectraDevnet => "pectra-devnet",
        };

        f.write_str(str)
    }
}

impl Network {
    pub fn to_base_config(&self) -> BaseConfig {
        match self {
            Self::Mainnet => mainnet(),
            _ => panic!("unsupport network")
        }
    }

    pub fn from_chain_id(id: u64) -> Result<Self> {
        match id {
            1 => Ok(Network::Mainnet),
            11155111 => Ok(Network::Sepolia),
            17000 => Ok(Network::Holesky),
            _ => Err(eyre::eyre!("chain id not known")),
        }
    }
}

pub fn mainnet() -> BaseConfig {
    BaseConfig {
        default_checkpoint: B256::from_hex(
            "0d5144fae3e0059e1372e5fc8fc28b042f1e2b9e698a007d42856ca6766d6ceb"
        ),
        rpc_port: 8545,
        consensus_rpc: Some("https://ethereum.operationsolarstorm.org".to_string()),
        chain: ChainConfig {
            chain_id: 1,
            genesis_time: 1606824023,
            genesis_root: B256::from_hex("4b363db94e286120d76eb905340fdd4e54bfe9f06bf33ff6cf5ad27f511bfe95"),
        },
        forks: Forks {
            genesis: Fork {
                epoch: 0,
                fork_version: FixedBytes::from_hex("00000000"),
            },
            altair: Fork {
                epoch: 74240,
                fork_version: FixedBytes::from_hex("01000000"),
            },
            bellatrix: Fork {
                epoch: 144896,
                fork_version: FixedBytes::from_hex("02000000"),
            },
            capella: Fork {
                epoch: 194048,
                fork_version: FixedBytes::from_hex("03000000"),
            },
            deneb: Fork {
                epoch: 269568,
                fork_version: FixedBytes::from_hex("04000000"),
            },
            electra: Fork {
                epoch: u64::MAX,
                fork_version: FixedBytes::from_hex("05000000"),
            },
        },
        execution_forks: ForkSchedule {
            prague_timestamp: u64::MAX,
        },
        max_checkpoint_age: 1_209_600, // 14 days
        #[cfg(not(target_arch = "wasm32"))]
        data_dir: Some(data_dir(Network::Mainnet)),
        ..std::default::Default::default()
    }
}
