use std::sync::Arc;

use alloy::consensus::BlockHeader;
use alloy::network::BlockResponse;
use alloy::primitives::{Address, Bytes, B256, U256};
use alloy::rpc::types::{Filter, FilterChanges, Log, SyncInfo, SyncStatus};
use eyre::{eyre, Result};

use crate::consensus::Consensus;
use crate::errors::ClientError;
use crate::execution::evm::Evm;
use crate::execution::rpc::http_rpc::HttpRpc;
use crate::execution::state::State;
use crate::execution::ExecutionClient;
use crate::fork_schedule::ForkSchedule;
use crate::network_spec::NetworkSpec;
use crate::time::{SystemTime, UNIX_EPOCH};
use crate::types::BlockTag;

pub struct Node<N: NetworkSpec, C: Consensus<N::BlockResponse>> {
    pub consensus: C,
    pub execution: Arc<ExecutionClient<N, HttpRpc<N>>>,
    pub history_size: usize,
    fork_schedule: ForkSchedule,
}

impl<N: NetworkSpec, C: Consensus<N::BlockResponse>> Node<N, C> {
    pub fn new(
        execution_rpc: &str,
        mut consensus: C,
        fork_schedule: ForkSchedule,
    ) -> Result<Self, ClientError> {
        let block_recv = consensus.block_recv().take().unwrap();
        let finalized_block_recv = consensus.finalized_block_recv().take().unwrap();

        let state = State::new(block_recv, finalized_block_recv, 256, execution_rpc);
        let execution = Arc::new(
            ExecutionClient::new(execution_rpc, state, fork_schedule)
                .map_err(ClientError::InternalError)?,
        );

        Ok(Node {
            consensus,
            execution,
            history_size: 64,
            fork_schedule,
        })
    }

    pub async fn call(
        &self,
        tx: &N::TransactionRequest,
        block: BlockTag,
    ) -> Result<Bytes, ClientError> {
        self.check_blocktag_age(&block).await?;

        let mut evm = Evm::new(
            self.execution.clone(),
            self.chain_id(),
            self.fork_schedule,
            block,
        );
        evm.call(tx).await.map_err(ClientError::EvmError)
    }

    pub async fn estimate_gas(&self, tx: &N::TransactionRequest) -> Result<u64, ClientError> {
        self.check_head_age().await?;

        let mut evm = Evm::new(
            self.execution.clone(),
            self.chain_id(),
            self.fork_schedule,
            BlockTag::Latest,
        );

        evm.estimate_gas(tx).await.map_err(ClientError::EvmError)
    }

    pub async fn get_balance(&self, address: Address, tag: BlockTag) -> Result<U256> {
        self.check_blocktag_age(&tag).await?;

        let account = self.execution.get_account(address, None, tag).await?;
        Ok(account.balance)
    }


    // assumes tip of 1 gwei to prevent having to prove out every tx in the block
    pub fn get_priority_fee(&self) -> Result<U256> {
        let tip = U256::from(10_u64.pow(9));
        Ok(tip)
    }

    pub fn chain_id(&self) -> u64 {
        self.consensus.chain_id()
    }

    pub async fn syncing(&self) -> Result<SyncStatus> {
        if self.check_head_age().await.is_ok() {
            Ok(SyncStatus::None)
        } else {
            let latest_synced_block = self.get_block_number().await.unwrap_or(U256::ZERO);
            let highest_block = self.consensus.expected_highest_block();

            Ok(SyncStatus::Info(Box::new(SyncInfo {
                current_block: latest_synced_block,
                highest_block: U256::from(highest_block),
                starting_block: U256::ZERO,
                ..Default::default()
            })))
        }
    }

    pub async fn get_coinbase(&self) -> Result<Address> {
        self.check_head_age().await?;

        let block = self
            .execution
            .get_block(BlockTag::Latest, false)
            .await
            .ok_or(eyre!(ClientError::BlockNotFound(BlockTag::Latest)))?;

        Ok(block.header().beneficiary())
    }

    async fn check_head_age(&self) -> Result<(), ClientError> {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_else(|_| panic!("unreachable"))
            .as_secs();

        let block_timestamp = self
            .execution
            .get_block(BlockTag::Latest, false)
            .await
            .ok_or_else(|| ClientError::OutOfSync(timestamp))?
            .header()
            .timestamp();

        let delay = timestamp.checked_sub(block_timestamp).unwrap_or_default();
        if delay > 60 {
            return Err(ClientError::OutOfSync(delay));
        }

        Ok(())
    }

    async fn check_blocktag_age(&self, block: &BlockTag) -> Result<(), ClientError> {
        match block {
            BlockTag::Latest => self.check_head_age().await,
            BlockTag::Finalized => Ok(()),
            BlockTag::Number(_) => Ok(()),
        }
    }
}
