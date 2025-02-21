use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize,Debug)]
pub struct LightClientHeader {
    pub beacon: Beacon,
    pub execution: Execution,
    pub execution_branch: Vec<String>,
}

#[derive(Serialize, Deserialize,Debug)]
pub struct Execution {
    pub parent_hash: String,
    pub fee_recipient: String,
    pub state_root: String,
    pub receipts_root: String,
    pub logs_bloom: String,
    pub prev_randao: String,
    pub block_number: String,
    pub gas_limit: String,
    pub gas_used: String,
    pub timestamp: String,
    pub extra_data: String,
    pub base_fee_per_gas: String,
    pub block_hash: String,
    pub transactions_root: String,
    pub withdrawals_root: String,
    pub blob_gas_used: String,
    pub excess_blob_gas: String,
}

#[derive(Serialize, Deserialize,Debug)]
pub struct Beacon {
    pub slot: String,
    pub proposer_index: String,
    pub parent_root: String,
    pub state_root: String,
    pub body_root: String,
}


#[derive(Serialize, Deserialize,Debug)]
pub struct SyncAggregate {
    pub sync_committee_bits: String,
    pub sync_committee_signature: String,
}


