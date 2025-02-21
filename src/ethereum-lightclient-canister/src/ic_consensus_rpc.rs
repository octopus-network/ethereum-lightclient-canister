

use helios_common::errors::RpcError;
use helios_common::http::get;

use anyhow::anyhow;
use crate::rpc_types::bootstrap::{Bootstrap, BootstrapResponse};
use crate::rpc_types::finality_update::{FinalityUpdate, FinalityUpdateResponse};
use crate::state::read_state;

#[derive(Debug)]
pub struct IcpConsensusRpc {
    rpc: String,
}

impl IcpConsensusRpc {
    fn new(path: &str) -> Self {
        IcpConsensusRpc {
            rpc: path.trim_end_matches('/').to_string(),
        }
    }

    async fn get_bootstrap(&self, checkpoint: String) -> anyhow::Result<Bootstrap> {
        let root_hex = hex::encode(checkpoint);
        let req = format!(
            "{}/eth/v1/beacon/light_client/bootstrap/{}",
            self.rpc, root_hex
        );

        let res: BootstrapResponse = rpc_request("bootstrap", req).await?;
        Ok(res.data)

    }

    async fn get_finality_update(&self) -> anyhow::Result<FinalityUpdate> {
        let rpc = read_state(|s|s.consensus_rpc.clone());
        let req = format!("{}/eth/v1/beacon/light_client/finality_update", rpc);
        let res: FinalityUpdateResponse = rpc_request("finality_update", &req)
            .await
            .map_err(|e| anyhow!(e.to_string()))?;

        Ok(res.data)
    }
}


async fn rpc_request<T>(name: impl AsRef<str>, url: impl AsRef<str>) -> anyhow::Result<T>
    where
        T: serde::de::DeserializeOwned,
{
    let name = name.as_ref();
    let url = url.as_ref();
/*    let resp = get(url).await.map_err(|e| RpcError::new(name, e))?;
    if resp.status != 200 {
        let e = format!("http response with status {}", resp.status);
        Err(RpcError::new(name, e))?;
    }
    let value = serde_json::from_slice(&resp.body).map_err(|e| RpcError::new(name, e))?;
    Ok(value)*/
    Err(anyhow!("".to_string()))
}