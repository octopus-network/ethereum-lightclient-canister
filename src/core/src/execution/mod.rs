use std::collections::{HashMap, HashSet};

use alloy::consensus::BlockHeader;
use alloy::network::primitives::HeaderResponse;
use alloy::network::{BlockResponse, ReceiptResponse};
use alloy::primitives::{keccak256, Address, B256, U256};
use alloy::rlp;
use alloy::rpc::types::{BlockTransactions, Filter, FilterChanges, Log};
use alloy_trie::root::ordered_trie_root_with_encoder;
use eyre::Result;
use futures::future::try_join_all;
use revm::primitives::{BlobExcessGasAndPrice, KECCAK_EMPTY};
use tracing::warn;

use crate::fork_schedule::ForkSchedule;
use crate::network_spec::NetworkSpec;
use crate::types::BlockTag;

use self::constants::MAX_SUPPORTED_LOGS_NUMBER;
use self::errors::ExecutionError;
use self::proof::{verify_account_proof, verify_storage_proof};
use self::rpc::ExecutionRpc;
use self::state::{FilterType, State};
use self::types::Account;

pub mod constants;
pub mod errors;
pub mod evm;
pub mod proof;
pub mod rpc;
pub mod state;
pub mod types;

#[derive(Clone)]
pub struct ExecutionClient<R: ExecutionRpc<N>> {
    pub rpc: R,
}

impl<N: NetworkSpec, R: ExecutionRpc<N>> ExecutionClient< R> {
    pub fn new(rpc: &str, state: State<N, R>, fork_schedule: ForkSchedule) -> Result<Self> {
        let rpc: R = ExecutionRpc::new(rpc)?;
        Ok(ExecutionClient::<R> {
            rpc,
        })
    }

    pub async fn check_rpc(&self, chain_id: u64) -> Result<()> {
        if self.rpc.chain_id().await? != chain_id {
            Err(ExecutionError::IncorrectRpcNetwork().into())
        } else {
            Ok(())
        }
    }

    pub async fn send_raw_transaction(&self, bytes: &[u8]) -> Result<B256> {
        self.rpc.send_raw_transaction(bytes).await
    }

    /// Ensure that each log entry in the given array of logs match the given filter.
    async fn ensure_logs_match_filter(&self, logs: &[Log], filter: &Filter) -> Result<()> {
        fn log_matches_filter(log: &Log, filter: &Filter) -> bool {
            if let Some(block_hash) = filter.get_block_hash() {
                if log.block_hash.unwrap() != block_hash {
                    return false;
                }
            }
            if let Some(from_block) = filter.get_from_block() {
                if log.block_number.unwrap() < from_block {
                    return false;
                }
            }
            if let Some(to_block) = filter.get_to_block() {
                if log.block_number.unwrap() > to_block {
                    return false;
                }
            }
            if !filter.address.matches(&log.address()) {
                return false;
            }
            for (i, topic) in filter.topics.iter().enumerate() {
                if let Some(log_topic) = log.topics().get(i) {
                    if !topic.matches(log_topic) {
                        return false;
                    }
                } else {
                    // if filter topic is not present in log, it's a mismatch
                    return false;
                }
            }
            true
        }
        for log in logs {
            if !log_matches_filter(log, filter) {
                return Err(ExecutionError::LogFilterMismatch().into());
            }
        }
        Ok(())
    }
}

/// Compute a trie root of a collection of encoded items.
/// Ref: https://github.com/alloy-rs/trie/blob/main/src/root.rs.
fn ordered_trie_root(items: &[Vec<u8>]) -> B256 {
    fn noop_encoder(item: &Vec<u8>, buffer: &mut Vec<u8>) {
        buffer.extend_from_slice(item);
    }

    ordered_trie_root_with_encoder(items, noop_encoder)
}
