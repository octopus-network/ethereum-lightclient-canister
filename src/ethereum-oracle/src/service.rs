use ic_cdk::{export_candid, query, update};
use tree_hash::fixed_bytes::B256;
use crate::ic_execution_rpc::IcExecutionRpc;

#[query]
pub fn set_resup() -> String {
    "".to_string()
}

/*#[update]
pub async fn query_block_receipts(hash: String) -> String {
    let rpc = IcExecutionRpc::new("https://mainnet.infura.io/v3/025779c23a2d44d1b1ecef2bfb4f2b29").unwrap();
    let hash = B256::from_hex(hash.as_str());
    let r = rpc.get_block_receipts(hash).await.unwrap();
    return serde_json::to_string(&r).unwrap();
}*/

export_candid!();