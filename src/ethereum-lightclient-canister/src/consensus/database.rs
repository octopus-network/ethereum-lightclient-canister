#[cfg(not(target_arch = "wasm32"))]
use std::{
    fs,
    io::{Read, Write},
    path::PathBuf,
};
use std::str::FromStr;
use alloy::hex;

use alloy::primitives::B256;
use eyre::{Report, Result};

use helios_consensus::config::Config;
use crate::state::{mutate_state, read_state};

pub trait Database: Clone + Sync + Send + 'static {
    fn new(config: &Config) -> Result<Self>
    where
        Self: Sized;

    fn save_checkpoint(&self, checkpoint: B256) -> Result<()>;
    fn load_checkpoint(&self) -> Result<B256>;
}


#[derive(Clone)]
pub struct ConfigDB;

impl Database for ConfigDB {
    fn new(config: &Config) -> Result<Self> {
        Ok(Self)
    }

    fn save_checkpoint(&self, checkpoint: B256) -> Result<()> {
        let s = hex::encode(checkpoint.0.as_slice());
        mutate_state(|state|state.last_checkpoint = Some(s));
        Ok(())
    }

    fn load_checkpoint(&self) -> Result<B256> {
        match read_state(|s|s.last_checkpoint.clone()) {
            None => {
                Err(Report::msg("not initial".to_string()))
            }
            Some(hash) => {
                let b  = B256::from_str(hash.as_str()).unwrap();
                Ok(b)
            }
        }
    }
}
