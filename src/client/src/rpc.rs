use arc_swap::ArcSwap;
use ethers_core::{
    abi::AbiEncode,
    types::{Address, Filter, Log, SyncingStatus, Transaction, TransactionReceipt, H256, U256},
};
use eyre::Result;
use log::info;
use std::net::{IpAddr, Ipv4Addr};
use std::{fmt::Display, net::SocketAddr, str::FromStr, sync::Arc};

use jsonrpsee::{
    core::{async_trait, server::rpc_module::Methods, Error},
    http_server::{HttpServerBuilder, HttpServerHandle},
    proc_macros::rpc,
};

use crate::{errors::NodeError, node::Node};

use common::{
    types::BlockTag,
    utils::{hex_str_to_bytes, u64_to_hex_string},
};
use execution::types::{CallOpts, ExecutionBlock};

pub struct Rpc {
    node: Arc<ArcSwap<Node>>,
    handle: Option<HttpServerHandle>,
    address: SocketAddr,
}

impl Rpc {
    pub fn new(node: Arc<ArcSwap<Node>>, ip: Option<IpAddr>, port: Option<u16>) -> Self {
        let address = SocketAddr::new(
            ip.unwrap_or(IpAddr::V4(Ipv4Addr::LOCALHOST)),
            port.unwrap_or(0),
        );
        Rpc {
            node,
            handle: None,
            address,
        }
    }

    pub async fn start(&mut self) -> Result<SocketAddr> {
        let rpc_inner = RpcInner {
            node: self.node.clone(),
            address: self.address,
        };

        let (handle, addr) = start(rpc_inner).await?;
        self.handle = Some(handle);

        info!("rpc server started at {}", addr);

        Ok(addr)
    }
}

#[rpc(server, namespace = "eth")]
trait EthRpc {
    #[method(name = "getBalance")]
    async fn get_balance(&self, address: &str, block: BlockTag) -> Result<String, Error>;
    #[method(name = "getTransactionCount")]
    async fn get_transaction_count(&self, address: &str, block: BlockTag) -> Result<String, Error>;
    #[method(name = "getBlockTransactionCountByHash")]
    async fn get_block_transaction_count_by_hash(&self, hash: &str) -> Result<String, Error>;
    #[method(name = "getBlockTransactionCountByNumber")]
    async fn get_block_transaction_count_by_number(&self, block: BlockTag)
        -> Result<String, Error>;
    #[method(name = "getCode")]
    async fn get_code(&self, address: &str, block: BlockTag) -> Result<String, Error>;
    #[method(name = "call")]
    async fn call(&self, opts: CallOpts, block: BlockTag) -> Result<String, Error>;
    #[method(name = "estimateGas")]
    async fn estimate_gas(&self, opts: CallOpts) -> Result<String, Error>;
    #[method(name = "chainId")]
    async fn chain_id(&self) -> Result<String, Error>;
    #[method(name = "gasPrice")]
    async fn gas_price(&self) -> Result<String, Error>;
    #[method(name = "maxPriorityFeePerGas")]
    async fn max_priority_fee_per_gas(&self) -> Result<String, Error>;
    #[method(name = "blockNumber")]
    async fn block_number(&self) -> Result<String, Error>;
    #[method(name = "getBlockByNumber")]
    async fn get_block_by_number(
        &self,
        block: BlockTag,
        full_tx: bool,
    ) -> Result<Option<ExecutionBlock>, Error>;
    #[method(name = "getBlockByHash")]
    async fn get_block_by_hash(
        &self,
        hash: &str,
        full_tx: bool,
    ) -> Result<Option<ExecutionBlock>, Error>;
    #[method(name = "sendRawTransaction")]
    async fn send_raw_transaction(&self, bytes: &str) -> Result<String, Error>;
    #[method(name = "getTransactionReceipt")]
    async fn get_transaction_receipt(
        &self,
        hash: &str,
    ) -> Result<Option<TransactionReceipt>, Error>;
    #[method(name = "getTransactionByHash")]
    async fn get_transaction_by_hash(&self, hash: &str) -> Result<Option<Transaction>, Error>;
    #[method(name = "getTransactionByBlockHashAndIndex")]
    async fn get_transaction_by_block_hash_and_index(
        &self,
        hash: &str,
        index: usize,
    ) -> Result<Option<Transaction>, Error>;
    #[method(name = "getLogs")]
    async fn get_logs(&self, filter: Filter) -> Result<Vec<Log>, Error>;
    #[method(name = "getStorageAt")]
    async fn get_storage_at(
        &self,
        address: &str,
        slot: H256,
        block: BlockTag,
    ) -> Result<String, Error>;
    #[method(name = "coinbase")]
    async fn coinbase(&self) -> Result<Address, Error>;
    #[method(name = "syncing")]
    async fn syncing(&self) -> Result<SyncingStatus, Error>;
}

#[rpc(client, server, namespace = "net")]
trait NetRpc {
    #[method(name = "version")]
    async fn version(&self) -> Result<String, Error>;
}

#[derive(Clone)]
struct RpcInner {
    node: Arc<ArcSwap<Node>>,
    address: SocketAddr,
}

#[async_trait]
impl EthRpcServer for RpcInner {
    async fn get_balance(&self, address: &str, block: BlockTag) -> Result<String, Error> {
        let address = convert_err(Address::from_str(address))?;
        let node = self.node.load_full();
        let balance = convert_err(node.get_balance(&address, block).await)?;

        Ok(format_hex(&balance))
    }

    async fn get_transaction_count(&self, address: &str, block: BlockTag) -> Result<String, Error> {
        let address = convert_err(Address::from_str(address))?;
        let node = self.node.load_full();
        let nonce = convert_err(node.get_nonce(&address, block).await)?;

        Ok(format!("0x{nonce:x}"))
    }

    async fn get_block_transaction_count_by_hash(&self, hash: &str) -> Result<String, Error> {
        let hash = convert_err(hex_str_to_bytes(hash))?;
        let node = self.node.load();
        let transaction_count = convert_err(node.get_block_transaction_count_by_hash(&hash))?;

        Ok(u64_to_hex_string(transaction_count))
    }

    async fn get_block_transaction_count_by_number(
        &self,
        block: BlockTag,
    ) -> Result<String, Error> {
        let node = self.node.load();
        let transaction_count = convert_err(node.get_block_transaction_count_by_number(block))?;
        Ok(u64_to_hex_string(transaction_count))
    }

    async fn get_code(&self, address: &str, block: BlockTag) -> Result<String, Error> {
        let address = convert_err(Address::from_str(address))?;
        let node = self.node.load_full();
        let code = convert_err(node.get_code(&address, block).await)?;

        Ok(format!("0x{:}", hex::encode(code)))
    }

    async fn call(&self, opts: CallOpts, block: BlockTag) -> Result<String, Error> {
        let node = self.node.load_full();

        let res = node
            .call(&opts, block)
            .await
            .map_err(NodeError::to_json_rpsee_error)?;

        Ok(format!("0x{}", hex::encode(res)))
    }

    async fn estimate_gas(&self, opts: CallOpts) -> Result<String, Error> {
        let node = self.node.load_full();
        let gas = node
            .estimate_gas(&opts)
            .await
            .map_err(NodeError::to_json_rpsee_error)?;

        Ok(u64_to_hex_string(gas))
    }

    async fn chain_id(&self) -> Result<String, Error> {
        let node = self.node.load();
        let id = node.chain_id();
        Ok(u64_to_hex_string(id))
    }

    async fn gas_price(&self) -> Result<String, Error> {
        let node = self.node.load();
        let gas_price = convert_err(node.get_gas_price())?;
        Ok(format_hex(&gas_price))
    }

    async fn max_priority_fee_per_gas(&self) -> Result<String, Error> {
        let node = self.node.load();
        let tip = convert_err(node.get_priority_fee())?;
        Ok(format_hex(&tip))
    }

    async fn block_number(&self) -> Result<String, Error> {
        let node = self.node.load();
        let num = convert_err(node.get_block_number())?;
        Ok(u64_to_hex_string(num))
    }

    async fn get_block_by_number(
        &self,
        block: BlockTag,
        full_tx: bool,
    ) -> Result<Option<ExecutionBlock>, Error> {
        let node = self.node.load_full();
        let block = convert_err(node.get_block_by_number(block, full_tx).await)?;
        Ok(block)
    }

    async fn get_block_by_hash(
        &self,
        hash: &str,
        full_tx: bool,
    ) -> Result<Option<ExecutionBlock>, Error> {
        let hash = convert_err(hex_str_to_bytes(hash))?;
        let node = self.node.load_full();
        let block = convert_err(node.get_block_by_hash(&hash, full_tx).await)?;
        Ok(block)
    }

    async fn send_raw_transaction(&self, bytes: &str) -> Result<String, Error> {
        let node = self.node.load_full();
        let bytes = convert_err(hex_str_to_bytes(bytes))?;
        let tx_hash = convert_err(node.send_raw_transaction(&bytes).await)?;
        Ok(hex::encode(tx_hash))
    }

    async fn get_transaction_receipt(
        &self,
        hash: &str,
    ) -> Result<Option<TransactionReceipt>, Error> {
        let node = self.node.load_full();
        let hash = H256::from_slice(&convert_err(hex_str_to_bytes(hash))?);
        let receipt = convert_err(node.get_transaction_receipt(&hash).await)?;
        Ok(receipt)
    }

    async fn get_transaction_by_hash(&self, hash: &str) -> Result<Option<Transaction>, Error> {
        let node = self.node.load_full();
        let hash = H256::from_slice(&convert_err(hex_str_to_bytes(hash))?);
        convert_err(node.get_transaction_by_hash(&hash).await)
    }

    async fn get_transaction_by_block_hash_and_index(
        &self,
        hash: &str,
        index: usize,
    ) -> Result<Option<Transaction>, Error> {
        let hash = convert_err(hex_str_to_bytes(hash))?;
        let node = self.node.load_full();
        convert_err(
            node.get_transaction_by_block_hash_and_index(&hash, index)
                .await,
        )
    }

    async fn coinbase(&self) -> Result<Address, Error> {
        let node = self.node.load();
        Ok(node.get_coinbase().unwrap())
    }

    async fn syncing(&self) -> Result<SyncingStatus, Error> {
        let node = self.node.load();
        convert_err(node.syncing())
    }

    async fn get_logs(&self, filter: Filter) -> Result<Vec<Log>, Error> {
        let node = self.node.load_full();
        convert_err(node.get_logs(&filter).await)
    }

    async fn get_storage_at(
        &self,
        address: &str,
        slot: H256,
        block: BlockTag,
    ) -> Result<String, Error> {
        let address = convert_err(Address::from_str(address))?;
        let node = self.node.load_full();
        let storage = convert_err(node.get_storage_at(&address, slot, block).await)?;

        Ok(format_hex(&storage))
    }
}

#[async_trait]
impl NetRpcServer for RpcInner {
    async fn version(&self) -> Result<String, Error> {
        let node = self.node.load();
        Ok(node.chain_id().to_string())
    }
}

async fn start(rpc: RpcInner) -> Result<(HttpServerHandle, SocketAddr)> {
    let server = HttpServerBuilder::default().build(rpc.address).await?;
    let addr = server.local_addr()?;

    let mut methods = Methods::new();
    let eth_methods: Methods = EthRpcServer::into_rpc(rpc.clone()).into();
    let net_methods: Methods = NetRpcServer::into_rpc(rpc).into();

    methods.merge(eth_methods)?;
    methods.merge(net_methods)?;

    let handle = server.start(methods)?;

    Ok((handle, addr))
}

fn convert_err<T, E: Display>(res: Result<T, E>) -> Result<T, Error> {
    res.map_err(|err| Error::Custom(err.to_string()))
}

fn format_hex(num: &U256) -> String {
    let stripped = num
        .encode_hex()
        .strip_prefix("0x")
        .unwrap()
        .trim_start_matches('0')
        .to_string();

    let stripped = if stripped.is_empty() {
        "0".to_string()
    } else {
        stripped
    };

    format!("0x{stripped}")
}
