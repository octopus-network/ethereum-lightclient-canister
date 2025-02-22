use candid::CandidType;
use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize, CandidType)]
pub struct LightClientHeader {
    pub beacon: Beacon,
    pub execution: Execution,
    pub execution_branch: Vec<String>,
}

#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize, CandidType)]
pub struct Execution {
    pub parent_hash: String,
    pub fee_recipient: String,
    pub state_root: String,
    pub receipts_root: String,
    pub logs_bloom: String,
    pub prev_randao: String,
    #[serde(with = "crate::rpc_types::serde_utils::u64")]
    pub block_number: u64,
    #[serde(with = "crate::rpc_types::serde_utils::u64")]
    pub gas_limit: u64,
    #[serde(with = "crate::rpc_types::serde_utils::u64")]
    pub gas_used: u64,
    #[serde(with = "crate::rpc_types::serde_utils::u64")]
    pub timestamp: u64,
    pub extra_data: String,
    pub base_fee_per_gas: String,
    pub block_hash: String,
    pub transactions_root: String,
    pub withdrawals_root: String,
    #[serde(with = "crate::rpc_types::serde_utils::u64")]
    pub blob_gas_used: u64,
    #[serde(with = "crate::rpc_types::serde_utils::u64")]
    pub excess_blob_gas: u64,
}

#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize, CandidType)]
pub struct Beacon {
    #[serde(with = "crate::rpc_types::serde_utils::u64")]
    pub slot: u64,
    #[serde(with = "crate::rpc_types::serde_utils::u64")]
    pub proposer_index: u64,
    pub parent_root: String,
    pub state_root: String,
    pub body_root: String,
}


#[derive(Debug, Default, Clone, Serialize, Deserialize, CandidType)]
pub struct SyncAggregate {
    pub sync_committee_bits: String,
    pub sync_committee_signature: String,
}


