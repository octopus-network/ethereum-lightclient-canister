use std::marker::PhantomData;
use alloy::eips::BlockId;
use alloy::eips::eip2930::AccessList;
use alloy::primitives::{Address, B256, U256};
use alloy::rpc::types::{EIP1186AccountProofResponse, FeeHistory, Filter, FilterChanges, Log};
use async_trait::async_trait;
use ic_cdk::api::management_canister::http_request::HttpMethod::GET;
use helios_common::errors::RpcError;
use helios_common::http::{get, post};
use helios_core::execution::rpc::ExecutionRpc;
use helios_core::network_spec::NetworkSpec;
use helios_core::types::BlockTag;
use crate::consensus::spec::Ethereum;

#[derive(Debug,Clone)]
pub struct IcExecutionRpc<N> {
    rpc: String,
    phantom_data: PhantomData<N>,
}

#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
impl<N: NetworkSpec> ExecutionRpc<N> for IcExecutionRpc<N> {
    fn new(rpcx: &str) -> eyre::Result<Self> where Self: Sized {
        Ok(
            Self {
                rpc: rpcx.to_string(),
                phantom_data: Default::default(),
            }
        )
    }

    async fn get_proof(&self, address: Address, slots: &[B256], block: BlockId) -> eyre::Result<EIP1186AccountProofResponse> {
        todo!()
    }

    async fn create_access_list(&self, tx: &N::TransactionRequest, block: BlockTag) -> eyre::Result<AccessList> {
        todo!()
    }

    async fn get_code(&self, address: Address, block: u64) -> eyre::Result<Vec<u8>> {
        todo!()
    }

    async fn send_raw_transaction(&self, bytes: &[u8]) -> eyre::Result<B256> {
        todo!()
    }

    async fn get_transaction_receipt(&self, tx_hash: B256) -> eyre::Result<Option<N::ReceiptResponse>> {
        todo!()
    }

    async fn get_block_receipts(&self, block: BlockTag) -> eyre::Result<Option<Vec<N::ReceiptResponse>>> {
        todo!()
    }

    async fn get_transaction(&self, tx_hash: B256) -> eyre::Result<Option<N::TransactionResponse>> {
        todo!()
    }

    async fn get_logs(&self, filter: &Filter) -> eyre::Result<Vec<Log>> {
        todo!()
    }

    async fn get_filter_changes(&self, filter_id: U256) -> eyre::Result<FilterChanges> {
        todo!()
    }

    async fn get_filter_logs(&self, filter_id: U256) -> eyre::Result<Vec<Log>> {
        todo!()
    }

    async fn uninstall_filter(&self, filter_id: U256) -> eyre::Result<bool> {
        todo!()
    }

    async fn new_filter(&self, filter: &Filter) -> eyre::Result<U256> {
        todo!()
    }

    async fn new_block_filter(&self) -> eyre::Result<U256> {
        todo!()
    }

    async fn new_pending_transaction_filter(&self) -> eyre::Result<U256> {
        todo!()
    }

    async fn chain_id(&self) -> eyre::Result<u64> {
        todo!()
    }

    async fn get_block(&self, hash: B256) -> eyre::Result<N::BlockResponse> {
        todo!()
    }

    async fn get_fee_history(&self, block_count: u64, last_block: u64, reward_percentiles: &[f64]) -> eyre::Result<FeeHistory> {
        todo!()
    }
}

async fn post_request<T>(name: impl AsRef<str>, body: String, url: impl AsRef<str>) -> eyre::Result<T>
    where
        T: serde::de::DeserializeOwned,
{
    let name = name.as_ref();
    let url = url.as_ref();
    let resp =post(url, &[], body.as_bytes().to_vec()).await.map_err(|e| RpcError::new(name, e))?;
    if resp.status != 200 {
        let e = format!("http response with status {}", resp.status);
        Err(RpcError::new(name, e))?;
    }
    let value = serde_json::from_slice(&resp.body).map_err(|e| RpcError::new(name, e))?;
    Ok(value)
}
