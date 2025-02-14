use std::sync::atomic::{AtomicU64, Ordering};

use async_trait::async_trait;
use ethers_core::types::transaction::eip2718::TypedTransaction;
use ethers_core::types::transaction::eip2930::{AccessList, AccessListWithGasUsed};
use ethers_core::types::{
    Address, BlockId, BlockNumber, Bytes, EIP1186ProofResponse, Eip1559TransactionRequest,
    FeeHistory, Filter, Log, Transaction, TransactionReceipt, H256, U256,
};
use eyre::Result;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};

use crate::types::CallOpts;
use common::errors::{HttpError, JsonRpcError, RpcError};
use common::http;

use super::ExecutionRpc;

pub struct HttpRpc {
    id: AtomicU64,
    url: String,
}

impl Clone for HttpRpc {
    fn clone(&self) -> Self {
        Self::new(&self.url).unwrap()
    }
}

#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
impl ExecutionRpc for HttpRpc {
    fn new(rpc: &str) -> Result<Self> {
        Ok(Self {
            id: AtomicU64::new(0),
            url: rpc.to_owned(),
        })
    }

    async fn get_proof(
        &self,
        address: &Address,
        slots: &[H256],
        block: u64,
    ) -> Result<EIP1186ProofResponse> {
        let address = serde_json::to_value(address)?;
        let slots = slots
            .iter()
            .map(serde_json::to_value)
            .collect::<Result<_, _>>()?;
        let block = serde_json::to_value(BlockId::from(block))?;

        Ok(self
            .request("eth_getProof", [address, slots, block])
            .await
            .map_err(|e| RpcError::new("get_proof", e))?)
    }

    async fn create_access_list(&self, opts: &CallOpts, block: u64) -> Result<AccessList> {
        let mut raw_tx = Eip1559TransactionRequest::new();
        raw_tx.to = Some(opts.to.unwrap_or_default().into());
        raw_tx.from = opts.from;
        raw_tx.value = opts.value;
        raw_tx.gas = Some(opts.gas.unwrap_or(U256::from(100_000_000)));
        raw_tx.max_fee_per_gas = Some(U256::zero());
        raw_tx.max_priority_fee_per_gas = Some(U256::zero());
        raw_tx.data = opts
            .data
            .as_ref()
            .map(|data| Bytes::from(data.as_slice().to_owned()));

        let block = serde_json::to_value(BlockId::from(block))?;
        let tx = serde_json::to_value(TypedTransaction::Eip1559(raw_tx))?;

        Ok(self
            .request::<_, AccessListWithGasUsed>("eth_createAccessList", [tx, block])
            .await
            .map_err(|e| RpcError::new("create_access_list", e))?
            .access_list)
    }

    async fn get_code(&self, address: &Address, block: u64) -> Result<Vec<u8>> {
        let address = serde_json::to_value(address)?;
        let block = serde_json::to_value(BlockId::from(block))?;

        Ok(self
            .request::<_, Bytes>("eth_getCode", [address, block])
            .await
            .map_err(|e| RpcError::new("get_code", e))?
            .to_vec())
    }

    async fn send_raw_transaction(&self, bytes: &[u8]) -> Result<H256> {
        let bytes = Bytes::from(bytes.to_owned());

        Ok(self
            .request("eth_sendRawTransaction", [bytes])
            .await
            .map_err(|e| RpcError::new("send_raw_transaction", e))?)
    }

    async fn get_transaction_receipt(&self, tx_hash: &H256) -> Result<Option<TransactionReceipt>> {
        Ok(self
            .request("eth_getTransactionReceipt", [tx_hash])
            .await
            .map_err(|e| RpcError::new("get_transaction_receipt", e))?)
    }

    async fn get_transaction(&self, tx_hash: &H256) -> Result<Option<Transaction>> {
        Ok(self
            .request("eth_getTransactionByHash", [tx_hash])
            .await
            .map_err(|e| RpcError::new("get_transaction", e))?)
    }

    async fn get_logs(&self, filter: &Filter) -> Result<Vec<Log>> {
        Ok(self
            .request("eth_getLogs", [filter])
            .await
            .map_err(|e| RpcError::new("get_logs", e))?)
    }

    async fn chain_id(&self) -> Result<u64> {
        Ok(self
            .request::<_, U256>("eth_chainId", ())
            .await
            .map_err(|e| RpcError::new("chain_id", e))?
            .as_u64())
    }

    async fn get_fee_history(
        &self,
        block_count: u64,
        last_block: u64,
        reward_percentiles: &[f64],
    ) -> Result<FeeHistory> {
        let block_count_u256 = serde_json::to_value(U256::from(block_count))?;
        let block = serde_json::to_value(BlockNumber::from(last_block))?;
        let reward_percentiles = serde_json::to_value(reward_percentiles)?;

        // The blockCount param is expected to be an unsigned integer up to geth v1.10.6.
        // Geth v1.10.7 onwards, this has been updated to a hex encoded form. Failure to
        // decode the param from client side would fallback to the old API spec.
        let result = match self
            .request(
                "eth_feeHistory",
                [block_count_u256, block.clone(), reward_percentiles.clone()],
            )
            .await
        {
            success @ Ok(_) => success,
            err @ Err(_) => {
                let fallback = self
                    .request(
                        "eth_feeHistory",
                        [
                            serde_json::to_value(block_count)?,
                            block,
                            reward_percentiles,
                        ],
                    )
                    .await;

                if fallback.is_err() {
                    // if the older fallback also resulted in an error, we return the error from the
                    // initial attempt
                    err
                } else {
                    fallback
                }
            }
        };

        Ok(result.map_err(|e| RpcError::new("fee_history", e))?)
    }
}

/// A JSON-RPC response
#[derive(Clone, Debug, Deserialize)]
#[serde(untagged)]
enum Response<T> {
    Success {
        #[serde(rename = "id")]
        _id: u64,
        result: T,
    },
    Error {
        #[serde(rename = "id")]
        _id: u64,
        error: JsonRpcError,
    },
}

impl HttpRpc {
    async fn request<T: Serialize + Send + Sync, R: DeserializeOwned>(
        &self,
        method: &str,
        params: T,
    ) -> Result<R> {
        let next_id = self.id.fetch_add(1, Ordering::SeqCst);

        let payload = serde_json::json!({
            "id": next_id,
            "jsonrpc": "2.0",
            "method": method,
            "params": params,
        });

        let response = http::post(
            &self.url,
            &[("Content-Type", "application/json")],
            serde_json::to_vec(&payload)?,
        )
        .await?;

        if response.status != 200 {
            let body = std::str::from_utf8(&response.body).unwrap_or("couldn't decode error");
            Err(HttpError::Http(response.status, body.to_owned()))?;
        }

        match serde_json::from_slice(&response.body)? {
            Response::Success { result, .. } => Ok(result),
            Response::Error { error, .. } => Err(error)?,
        }
    }
}
