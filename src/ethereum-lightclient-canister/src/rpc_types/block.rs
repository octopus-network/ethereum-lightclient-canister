use serde::{Deserialize, Serialize};
use tree_hash::fixed_bytes::B256;
use crate::rpc_types::address::Address;

#[derive(Serialize, Deserialize)]
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
    pub hash: B256,
    #[serde(rename = "logsBloom")]
    pub logs_bloom: String,
    pub miner: Address,
    #[serde(rename = "mixHash")]
    pub mix_hash: B256,
    pub nonce: String,
    pub number: String,
    #[serde(rename = "parentBeaconBlockRoot")]
    pub parent_beacon_block_root: B256,
    #[serde(rename = "parentHash")]
    pub parent_hash: B256,
    #[serde(rename = "receiptsRoot")]
    pub receipts_root: B256,
    #[serde(rename = "sha3Uncles")]
    pub sha3uncles: String,
    pub size: String,
    #[serde(rename = "stateRoot")]
    pub state_root: B256,
    pub timestamp: String,
    pub transactions: Vec<B256>,
    #[serde(rename = "transactionsRoot")]
    pub transactions_root: B256,
    #[serde(rename = "withdrawalsRoot")]
    pub withdrawals_root: B256,
}