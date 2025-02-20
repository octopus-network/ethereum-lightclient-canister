/*use std::cmp;
use alloy::hex::hex;
use alloy::primitives::B256;
use async_trait::async_trait;
use helios_common::errors::RpcError;
use helios_common::http::get;
use helios_consensus::constants::MAX_REQUEST_LIGHT_CLIENT_UPDATES;
use helios_consensus::rpc::ConsensusRpc;
use helios_consensus_core::consensus_spec::{ConsensusSpec, MainnetConsensusSpec};
use helios_consensus_core::types::{BeaconBlock, Bootstrap, FinalityUpdate, OptimisticUpdate, Update};
use crate::state::read_state;
use helios_consensus::rpc::http_rpc::{BeaconBlockResponse, BootstrapResponse, FinalityUpdateResponse, HttpRpc, OptimisticUpdateResponse, SpecResponse, UpdateData};

#[derive(Debug)]
pub struct IcpConsensusRpc {
    rpc: String,
}

#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
impl<S: ConsensusSpec> ConsensusRpc<S> for IcpConsensusRpc {
    fn new(path: &str) -> Self {
        IcpConsensusRpc {
            rpc: path.trim_end_matches('/').to_string(),
        }
    }

    async fn get_bootstrap(&self, checkpoint: B256) -> eyre::Result<Bootstrap<S>> {
        let root_hex = hex::encode(checkpoint);
        let req = format!(
            "{}/eth/v1/beacon/light_client/bootstrap/0x{}",
            self.rpc, root_hex
        );
        let res: BootstrapResponse<S> = rpc_request("bootstrap", req).await?;
        Ok(res.data)

    }

    async fn get_updates(&self, period: u64, count: u8) -> eyre::Result<Vec<Update<S>>> {
        let count = cmp::min(count, MAX_REQUEST_LIGHT_CLIENT_UPDATES);
        let req = format!(
            "{}/eth/v1/beacon/light_client/updates?start_period={}&count={}",
            self.rpc, period, count
        );

        let res: Vec<UpdateData<S>> = rpc_request("updates", req).await?;

        Ok(res.into_iter().map(|d| d.data).collect())
    }

    async fn get_finality_update(&self) -> eyre::Result<FinalityUpdate<S>> {
        let rpc = read_state(|s|s.consensus_rpc.clone());
        let req = format!("{}/eth/v1/beacon/light_client/finality_update", rpc);
        let res: FinalityUpdateResponse<S> = rpc_request("finality_update", &req)
            .await
            .map_err(|e| RpcError::new("finality_update", e))?;

        Ok(res.data)
    }

    async fn get_optimistic_update(&self) -> eyre::Result<OptimisticUpdate<S>> {
        let req = format!("{}/eth/v1/beacon/light_client/optimistic_update", self.rpc);
        let res: OptimisticUpdateResponse<S> = rpc_request("optimistic_update", req).await?;
        Ok(res.data)
    }

    async fn get_block(&self, slot: u64) -> eyre::Result<BeaconBlock<S>> {
        let req = format!("{}/eth/v2/beacon/blocks/{}", self.rpc, slot);
        let res: BeaconBlockResponse<S> = rpc_request("blocks", req).await?;
        Ok(res.data.message)
    }

    async fn chain_id(&self) -> eyre::Result<u64> {
        let req = format!("{}/eth/v1/config/spec", self.rpc);
        let res: SpecResponse = rpc_request("spec", req).await?;
        Ok(res.data.chain_id)
    }
}


async fn rpc_request<T>(name: impl AsRef<str>, url: impl AsRef<str>) -> eyre::Result<T>
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
*/