use std::cmp;
use helios_common::errors::RpcError;
use helios_common::http::get;

use anyhow::anyhow;
use tree_hash::fixed_bytes::B256;
use crate::rpc_types::bootstrap::{Bootstrap, BootstrapResponse};
use crate::rpc_types::finality_update::{FinalityUpdate, FinalityUpdateResponse};
use crate::rpc_types::update::{Update, UpdateData};
pub const MAX_REQUEST_LIGHT_CLIENT_UPDATES: u8 = 128;


#[derive(Debug)]
pub struct IcpConsensusRpc {
    rpc: String,
}

impl IcpConsensusRpc {
    pub(crate) fn new(path: &str) -> Self {
        IcpConsensusRpc {
            rpc: path.trim_end_matches('/').to_string(),
        }
    }

    pub async fn get_bootstrap(&self, checkpoint: B256) -> eyre::Result<Bootstrap> {
        let root_hex = hex::encode(checkpoint.0.as_ref());
        let req = format!(
            "{}/eth/v1/beacon/light_client/bootstrap/0x{}",
            self.rpc, root_hex
        );

        let res: BootstrapResponse = rpc_request("bootstrap", req).await.map_err(|e| RpcError::new("bootstrap", e))?;
        Ok(res.data)

    }

    pub(crate) async fn get_updates(&self, period: u64, count: u8) -> eyre::Result<Vec<Update>> {
        let count = cmp::min(count, MAX_REQUEST_LIGHT_CLIENT_UPDATES);
        let req = format!(
            "{}/eth/v1/beacon/light_client/updates?start_period={}&count={}",
            self.rpc, period, count
        );
        let res: Vec<UpdateData> = rpc_request("updates", &req).await.map_err(|e| RpcError::new("updates", e))?;
        Ok(res.into_iter().map(|d| d.data).collect())
    }

    pub async fn get_finality_update(&self) -> eyre::Result<FinalityUpdate> {
        let req = format!("{}/eth/v1/beacon/light_client/finality_update", self.rpc.as_str());
        let res: FinalityUpdateResponse = rpc_request("finality_update", &req)
            .await
            .map_err(|e| RpcError::new("finality_update", e))?;
        Ok(res.data)
    }
}


pub async fn rpc_request<T>(name: impl AsRef<str>, url: impl AsRef<str>) -> eyre::Result<T>
    where
        T: serde::de::DeserializeOwned,
{
    let name = name.as_ref();
    let url = url.as_ref();
    let resp = get(url).await.map_err(|e| RpcError::new(name, e))?;
    if resp.status != 200 {
        let e = format!("http response with status {}", resp.status);
        Err(RpcError::new(name, e))?;
    }
    let value = serde_json::from_slice(&resp.body).map_err(|e| RpcError::new(name, e))?;
    Ok(value)
}