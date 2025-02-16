use std::sync::Arc;

use alloy::primitives::U256;
use eyre::Result;

use crate::consensus::Consensus;
use crate::errors::ClientError;
use crate::execution::ExecutionClient;
use crate::execution::rpc::http_rpc::HttpRpc;
use crate::fork_schedule::ForkSchedule;
use crate::network_spec::NetworkSpec;

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
        let execution = Arc::new(
            ExecutionClient::new(execution_rpc)
                .map_err(ClientError::InternalError)?,
        );

        Ok(Node {
            consensus,
            execution,
            history_size: 64,
            fork_schedule,
        })
    }


    // assumes tip of 1 gwei to prevent having to prove out every tx in the block
    pub fn get_priority_fee(&self) -> Result<U256> {
        let tip = U256::from(10_u64.pow(9));
        Ok(tip)
    }

    pub fn chain_id(&self) -> u64 {
        self.consensus.chain_id()
    }

}
