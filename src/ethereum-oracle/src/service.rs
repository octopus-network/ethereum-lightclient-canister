use crate::ic_execution_rpc::IcExecutionRpc;
use crate::rpc_types::receipt::encode_receipt;
use ic_cdk::{export_candid, query, update};
use tree_hash::fixed_bytes::B256;
use triehash_ethereum::ordered_trie_root;

#[query]
pub fn set_resup() -> String {
    "".to_string()
}

#[update]
pub async fn query_block_receipts(hash: String) -> String {
    let rpc = IcExecutionRpc::new("https://mainnet.infura.io/v3/025779c23a2d44d1b1ecef2bfb4f2b29")
        .unwrap();
    let hash = B256::from_hex(hash.as_str());
    let r = rpc.get_block_receipts(hash).await.unwrap();
    let v: Vec<Vec<u8>> = r.iter().map(encode_receipt).collect();
    let root = ordered_trie_root(v);
    let r = hex::encode(root.0.as_slice());
    r
}

export_candid!();
