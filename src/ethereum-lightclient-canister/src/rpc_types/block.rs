use candid::CandidType;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, CandidType)]
pub struct ExecutionBlock {
    #[serde(rename = "baseFeePerGas")]
    pub base_fee_per_gas: String,
    #[serde(rename = "blobGasUsed")]
    pub blob_gas_used: String,
    pub difficulty: String,
    #[serde(rename = "excessBlobGas")]
    pub excess_blob_gas: String,
    #[serde(rename = "extraData")]
    pub extra_data: String,
    #[serde(rename = "gasLimit")]
    pub gas_limit: String,
    #[serde(rename = "gasUsed")]
    pub gas_used: String,
    pub hash: String,
    #[serde(rename = "logsBloom")]
    pub logs_bloom: String,
    pub miner: String,
    #[serde(rename = "mixHash")]
    pub mix_hash: String,
    pub nonce: String,
    pub number: String,
    #[serde(rename = "parentBeaconBlockRoot")]
    pub parent_beacon_block_root: String,
    #[serde(rename = "parentHash")]
    pub parent_hash: String,
    #[serde(rename = "receiptsRoot")]
    pub receipts_root: String,
    #[serde(rename = "sha3Uncles")]
    pub sha3uncles: String,
    pub size: String,
    #[serde(rename = "stateRoot")]
    pub state_root: String,
    pub timestamp: String,
    pub transactions: Vec<String>,
    #[serde(rename = "transactionsRoot")]
    pub transactions_root: String,
    #[serde(rename = "withdrawalsRoot")]
    pub withdrawals_root: String,
}