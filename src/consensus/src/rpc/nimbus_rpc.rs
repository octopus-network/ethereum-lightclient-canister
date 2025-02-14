use async_trait::async_trait;
use eyre::Result;
use std::cmp;

use super::ConsensusRpc;
use crate::constants::MAX_REQUEST_LIGHT_CLIENT_UPDATES;
use crate::types::*;
use common::errors::RpcError;
use common::http;

#[derive(Debug)]
pub struct NimbusRpc {
    rpc: String,
}

#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
impl ConsensusRpc for NimbusRpc {
    fn new(rpc: &str) -> Self {
        NimbusRpc {
            rpc: rpc.to_string(),
        }
    }

    async fn get_bootstrap(&self, block_root: &'_ [u8]) -> Result<Bootstrap> {
        let root_hex = hex::encode(block_root);
        let req = format!(
            "{}/eth/v1/beacon/light_client/bootstrap/0x{}",
            self.rpc, root_hex
        );

        let res: BootstrapResponse = rpc_request("bootstrap", req).await?;

        Ok(res.data)
    }

    async fn get_updates(&self, period: u64, count: u8) -> Result<Vec<Update>> {
        let count = cmp::min(count, MAX_REQUEST_LIGHT_CLIENT_UPDATES);
        let req = format!(
            "{}/eth/v1/beacon/light_client/updates?start_period={}&count={}",
            self.rpc, period, count
        );

        let res: UpdateResponse = rpc_request("updates", req).await?;

        Ok(res.into_iter().map(|d| d.data).collect())
    }

    async fn get_finality_update(&self) -> Result<FinalityUpdate> {
        let req = format!("{}/eth/v1/beacon/light_client/finality_update", self.rpc);
        let res: FinalityUpdateResponse = rpc_request("finality_update", req).await?;
        Ok(res.data)
    }

    async fn get_optimistic_update(&self) -> Result<OptimisticUpdate> {
        let req = format!("{}/eth/v1/beacon/light_client/optimistic_update", self.rpc);
        let res: OptimisticUpdateResponse = rpc_request("optimistic_update", req).await?;
        Ok(res.data)
    }

    async fn get_block(&self, slot: u64) -> Result<BeaconBlock> {
        let req = format!("{}/eth/v2/beacon/blocks/{}", self.rpc, slot);
        let res: BeaconBlockResponse = rpc_request("blocks", req).await?;
        Ok(res.data.message)
    }

    async fn chain_id(&self) -> Result<u64> {
        let req = format!("{}/eth/v1/config/spec", self.rpc);
        let res: SpecResponse = rpc_request("spec", req).await?;
        Ok(res.data.chain_id)
    }
}

#[derive(serde::Deserialize, Debug)]
struct BeaconBlockResponse {
    data: BeaconBlockData,
}

#[derive(serde::Deserialize, Debug)]
struct BeaconBlockData {
    message: BeaconBlock,
}

type UpdateResponse = Vec<UpdateData>;

#[derive(serde::Deserialize, Debug)]
struct UpdateData {
    data: Update,
}

#[derive(serde::Deserialize, Debug)]
struct FinalityUpdateResponse {
    data: FinalityUpdate,
}

#[derive(serde::Deserialize, Debug)]
struct OptimisticUpdateResponse {
    data: OptimisticUpdate,
}

#[derive(serde::Deserialize, Debug)]
struct BootstrapResponse {
    data: Bootstrap,
}

#[derive(serde::Deserialize, Debug)]
struct SpecResponse {
    data: Spec,
}

#[derive(serde::Deserialize, Debug)]
struct Spec {
    #[serde(rename = "DEPOSIT_NETWORK_ID", deserialize_with = "u64_deserialize")]
    chain_id: u64,
}

async fn rpc_request<T>(name: impl AsRef<str>, url: impl AsRef<str>) -> Result<T>
where
    T: serde::de::DeserializeOwned,
{
    let name = name.as_ref();
    let url = url.as_ref();
    let resp = http::get(url).await.map_err(|e| RpcError::new(name, e))?;

    if resp.status != 200 {
        let e = format!("http response with status {}", resp.status);
        Err(RpcError::new(name, e))?;
    }

    let value = serde_json::from_slice(&resp.body).map_err(|e| RpcError::new(name, e))?;

    Ok(value)
}
