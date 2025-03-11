use serde_derive::Deserialize;

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
pub struct TransactionReceipt {
    #[serde(rename = "blockHash")]
    pub block_hash: String,
    #[serde(rename = "blockNumber")]
    pub block_number: String,
    #[serde(rename = "gasUsed")]
    pub gas_used: String,
    pub status: String,
    #[serde(rename = "transactionHash")]
    pub transaction_hash: String,
    #[serde(rename = "contractAddress")]
    pub contract_address: Option<String>,
    pub from: String,
    pub logs: Vec<cketh_common::eth_rpc::LogEntry>,
    #[serde(rename = "logsBloom")]
    pub logs_bloom: String,
    pub to: String,
    #[serde(rename = "transactionIndex")]
    pub transaction_index: String,
    pub r#type: String,
}
