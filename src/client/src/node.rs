use std::collections::BTreeMap;
use std::sync::Arc;
use std::time::Duration;

use ethers_core::types::{
    Address, FeeHistory, Filter, Log, SyncProgress, SyncingStatus, Transaction, TransactionReceipt,
    H256, U256,
};
use eyre::{eyre, Result};

use common::errors::BlockNotFoundError;
use common::types::BlockTag;
use config::Config;

use helios_consensus_core::types::{ExecutionPayload};
use helios_core::execution::ExecutionClient;
use consensus::consensus::{ConsensusClient};
use consensus::database::Database;
use consensus::rpc::ConsensusRpc;
use helios_consensus_core::consensus_spec::ConsensusSpec;
use crate::errors::NodeError;

#[derive(Clone)]
pub struct Node<S: ConsensusSpec, R: ConsensusRpc<S>, DB: Database> {
    pub consensus: ConsensusClient<S, R, DB>,
    pub config: Arc<Config>,
    payloads: BTreeMap<u64, ExecutionPayload<S>>,
    finalized_payloads: BTreeMap<u64, ExecutionPayload<S>>,
    current_slot: Option<u64>,
    pub history_size: usize,
}

impl<S: ConsensusSpec, R: ConsensusRpc<S>, DB: Database> Node<S,R,DB> {
    pub fn new(config: Arc<Config>) -> Result<Self, NodeError> {
        let consensus_rpc = &config.consensus_rpc;
        let checkpoint_hash = &config.checkpoint.as_ref().unwrap();
        let execution_rpc = &config.execution_rpc;

        let consensus = ConsensusClient::new(consensus_rpc, config.clone())
            .map_err(NodeError::ConsensusClientCreationError)?;
        let execution = Arc::new(
            ExecutionClient::new(execution_rpc).map_err(NodeError::ExecutionClientCreationError)?,
        );

        let payloads = BTreeMap::new();
        let finalized_payloads = BTreeMap::new();

        Ok(Node {
            consensus,
            config,
            payloads,
            finalized_payloads,
            current_slot: None,
            history_size: 36000,
        })
    }

    pub async fn sync(&mut self) -> Result<(), NodeError> {
        let chain_id = self.config.chain.chain_id;

        self.consensus
            .sync()
            .await
            .map_err(NodeError::ConsensusSyncError)?;

        self.update_payloads().await
    }

    pub async fn advance(&mut self) -> Result<(), NodeError> {
        self.consensus
            .advance()
            .await
            .map_err(NodeError::ConsensusAdvanceError)?;
        self.update_payloads().await
    }

    async fn update_payloads(&mut self) -> Result<(), NodeError> {
        let latest_header = self.consensus.get_header();
        let latest_payload = self
            .consensus
            .get_execution_payload(&Some(latest_header.slot))
            .await
            .map_err(NodeError::ConsensusPayloadError)?;

        let finalized_header = self.consensus.get_finalized_header();
        let finalized_payload = self
            .consensus
            .get_execution_payload(&Some(finalized_header.slot))
            .await
            .map_err(NodeError::ConsensusPayloadError)?;

        self.payloads
            .insert(*latest_payload.block_number(), latest_payload);
        self.payloads
            .insert(*finalized_payload.block_number(), finalized_payload.clone());
        self.finalized_payloads
            .insert(*finalized_payload.block_number(), finalized_payload);

        let start_slot = self
            .current_slot
            .unwrap_or(latest_header.slot - self.history_size as u64);
        let backfill_payloads = self
            .consensus
            .get_payloads(start_slot, latest_header.slot)
            .await
            .map_err(NodeError::ConsensusPayloadError)?;
        for payload in backfill_payloads {
            self.payloads.insert(*payload.block_number(), payload);
        }

        self.current_slot = Some(latest_header.slot);

        while self.payloads.len() > self.history_size {
            self.payloads.pop_first();
        }

        // only save one finalized block per epoch
        // finality updates only occur on epoch boundaries
        while self.finalized_payloads.len() > usize::max(self.history_size / 32, 1) {
            self.finalized_payloads.pop_first();
        }

        Ok(())
    }

    fn get_payload(&self, block: BlockTag) -> Result<&ExecutionPayload<S>, BlockNotFoundError> {
        match block {
            BlockTag::Latest => {
                let payload = self.payloads.last_key_value();
                Ok(payload.ok_or(BlockNotFoundError::new(BlockTag::Latest))?.1)
            }
            BlockTag::Finalized => {
                let payload = self.finalized_payloads.last_key_value();
                Ok(payload
                    .ok_or(BlockNotFoundError::new(BlockTag::Finalized))?
                    .1)
            }
            BlockTag::Number(num) => {
                let payload = self.payloads.get(&num);
                payload.ok_or(BlockNotFoundError::new(BlockTag::Number(num)))
            }
        }
    }

    fn get_payload_by_hash(&self, hash: &Vec<u8>) -> Result<(&u64, &ExecutionPayload<S>)> {
        let payloads = self
            .payloads
            .iter()
            .filter(|entry| &entry.1.block_hash().to_vec() == hash)
            .collect::<Vec<(&u64, &ExecutionPayload<S>)>>();

        payloads
            .get(0)
            .cloned()
            .ok_or(eyre!("Block not found by hash"))
    }

    fn check_head_age(&self) -> Result<(), NodeError> {
        let synced_slot = self.consensus.get_header().slot;
        let expected_slot = self.consensus.expected_current_slot();
        let slot_delay = expected_slot - synced_slot;

        if slot_delay > 10 {
            return Err(NodeError::OutOfSync(slot_delay));
        }

        Ok(())
    }

    fn check_blocktag_age(&self, block: &BlockTag) -> Result<(), NodeError> {
        match block {
            BlockTag::Latest => self.check_head_age(),
            BlockTag::Finalized => Ok(()),
            BlockTag::Number(_) => Ok(()),
        }
    }
}
