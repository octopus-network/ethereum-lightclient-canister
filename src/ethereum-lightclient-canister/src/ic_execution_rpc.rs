use ic_canister_log::log;
use serde::{Deserialize, Serialize};

use helios_common::errors::RpcError;
use helios_common::http::post;
use helios_common::rpc_types::block::ExecutionBlock;
use tree_hash::fixed_bytes::B256;
use crate::ic_log::INFO;

#[derive(Debug, Clone)]
pub struct IcExecutionRpc {
    rpc: String,
}

impl IcExecutionRpc {
    pub(crate) fn new(rpcx: &str) -> eyre::Result<Self>
    where
        Self: Sized,
    {
        Ok(Self {
            rpc: rpcx.to_string(),
        })
    }

    pub(crate) async fn get_block(&self, hash: B256) -> eyre::Result<ExecutionBlock> {
        let real_hex = format!("0x{}", hex::encode(hash.0.as_slice()));
        log!(INFO, "get_block {}",&real_hex);
        let params = r#"{"jsonrpc":"2.0", "method":"eth_getBlockByHash","params":["block_hash",false],"id":1}"#;
        let params = params.replace("block_hash", &real_hex);
        post_request("eth_getBlockByHash", params, self.rpc.clone(), 40*1000, 500000000).await
    }
}

async fn post_request<T>(
    name: impl AsRef<str>,
    body: String,
    url: impl AsRef<str>,
    max_response_size: u64,
    max_cost_cycles: u128
) -> eyre::Result<T>
where
    T: serde::de::DeserializeOwned,
{
    let name = name.as_ref();
    let url = url.as_ref();
    let resp = post(url, body.as_bytes().to_vec(), max_response_size, max_cost_cycles)
        .await
        .map_err(|e| RpcError::new(name, e))?;
    if resp.status != 200 {
        let e = format!("http response with status {}", resp.status);
        Err(RpcError::new(name, e))?;
    }
    let res = resp.body;
    let value: EvmRpcResponse<T> =
        serde_json::from_slice(&res).map_err(|e| RpcError::new(name, e))?;
    if value.result.is_some() {
        Ok(value.result.unwrap())
    } else {
        log!(INFO, "query block result: {:?}", String::from_utf8(res));
        Err(RpcError::new(name, "result is null".to_string()))?
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct EvmJsonRpcRequest {
    pub method: String,
    pub params: Vec<String>,
    pub id: u64,
    pub jsonrpc: String,
}

#[derive(Deserialize, Clone, Debug, Default)]
pub struct EvmRpcResponse<T> {
    pub result: Option<T>,
}
