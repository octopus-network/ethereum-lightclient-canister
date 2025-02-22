use candid::CandidType;
use serde::{Deserialize, Serialize};
use helios_common::bytes::{ByteList, LogsBloom};
use tree_hash::fixed_bytes::B256;
use crate::rpc_types::address::Address;

#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
pub struct LightClientHeader {
    pub beacon: Beacon,
    pub execution: ExecutionPayloadHeader,
    pub execution_branch: Vec<B256>,
}

#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
pub struct ExecutionPayloadHeader {
    pub parent_hash: B256,
    pub fee_recipient: Address,
    pub state_root: B256,
    pub receipts_root: B256,
    pub logs_bloom: LogsBloom,
    pub prev_randao: B256,
    #[serde(with = "crate::rpc_types::serde_utils::u64")]
    pub block_number: u64,
    #[serde(with = "crate::rpc_types::serde_utils::u64")]
    pub gas_limit: u64,
    #[serde(with = "crate::rpc_types::serde_utils::u64")]
    pub gas_used: u64,
    #[serde(with = "crate::rpc_types::serde_utils::u64")]
    pub timestamp: u64,
    pub extra_data: ByteList<typenum::U32>,
    pub base_fee_per_gas: String,
    pub block_hash: B256,
    pub transactions_root: B256,
    pub withdrawals_root: B256,
    #[serde(with = "crate::rpc_types::serde_utils::u64")]
    pub blob_gas_used: u64,
    #[serde(with = "crate::rpc_types::serde_utils::u64")]
    pub excess_blob_gas: u64,
}

#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
pub struct Beacon {
    #[serde(with = "crate::rpc_types::serde_utils::u64")]
    pub slot: u64,
    #[serde(with = "crate::rpc_types::serde_utils::u64")]
    pub proposer_index: u64,
    pub parent_root: B256,
    pub state_root: B256,
    pub body_root: B256,
}


#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct SyncAggregate {
    pub sync_committee_bits: String,
    pub sync_committee_signature: String,
}


