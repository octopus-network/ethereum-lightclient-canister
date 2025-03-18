use std::marker::PhantomData;

use eyre::eyre;
use eyre::Result;
use ic_canister_log::log;
use serde::{Deserialize, Serialize};

use tree_hash::fixed_bytes::B256;

use crate::consensus::core::{
    apply_bootstrap, apply_finality_update, apply_update, expected_current_slot, get_bits,
    verify_bootstrap, verify_finality_update, verify_update,
};
use crate::consensus::errors::ConsensusError;
use crate::ic_consensus_rpc::{IcpConsensusRpc, MAX_REQUEST_LIGHT_CLIENT_UPDATES};
use crate::ic_log::{INFO, WARNING};
use crate::state::{read_state, StateModifier};
use crate::storable_structures::BlockInfo;
use helios_common::consensus_spec::{calc_sync_period, ConsensusSpec, MainnetConsensusSpec};
use helios_common::rpc_types::finality_update::FinalityUpdate;
use helios_common::rpc_types::lightclient_store::LightClientStore;
use helios_common::rpc_types::update::Update;

#[derive(Debug, Serialize, Deserialize)]
pub struct Inner<S: ConsensusSpec> {
    pub(crate) store: LightClientStore,
    phantom_data: PhantomData<S>,
}

impl<S: ConsensusSpec> Inner<S> {
    pub fn new(store: LightClientStore) -> Inner<S> {
        Inner {
            store,
            phantom_data: Default::default(),
        }
    }

    pub async fn sync(&mut self, checkpoint: B256) -> Result<()> {
        self.bootstrap(checkpoint).await?;
        let current_period = calc_sync_period::<S>(self.store.finalized_header.beacon.slot);
        let updates =
            IcpConsensusRpc::get_updates(current_period, MAX_REQUEST_LIGHT_CLIENT_UPDATES).await?;

        for update in updates {
            self.verify_update(&update)?;
            self.apply_update(&update);
        }

        let finality_update = IcpConsensusRpc::get_finality_update().await?;
        self.verify_finality_update(&finality_update)?;
        self.apply_finality_update(&finality_update);
        log!(
            INFO,
            "consensus client in sync with checkpoint: 0x{}",
            hex::encode(checkpoint.0.as_ref())
        );

        Ok(())
    }

    pub async fn advance(&mut self) -> Result<()> {
        let finality_update = IcpConsensusRpc::get_finality_update().await?;
        //self.verify_finality_update(&finality_update)?;
        self.apply_finality_update(&finality_update);

/*        if self.store.next_sync_committee.is_none() {
            log!(INFO, "checking for sync committee update");
            let current_period =
                calc_sync_period::<MainnetConsensusSpec>(self.store.finalized_header.beacon.slot);
            let mut updates = IcpConsensusRpc::get_updates(current_period, 1).await?;

            if updates.len() == 1 {
                let update = updates.get_mut(0).unwrap();
                let res = self.verify_update(update);

                if res.is_ok() {
                    log!(INFO, "updating sync committee");
                    self.apply_update(update);
                }
            }
        }*/

        Ok(())
    }

    pub async fn store(&self) {
        let e = self.store.optimistic_header.execution.clone();
        let header_info = BlockInfo {
            receipt_root: e.receipts_root,
            parent_block_hash: e.parent_hash,
            block_number: e.block_number,
            block_hash: e.block_hash,
        };
        StateModifier::push_block(header_info).await;

        let e = self.store.finalized_header.execution.clone();
        let header_info = BlockInfo {
            receipt_root: e.receipts_root,
            parent_block_hash: e.parent_hash,
            block_number: e.block_number,
            block_hash: e.block_hash,
        };
        StateModifier::push_finalized_block(header_info).await;
    }

    pub async fn bootstrap(&mut self, checkpoint: B256) -> Result<()> {
        let bootstrap = IcpConsensusRpc::get_bootstrap(checkpoint)
            .await
            .map_err(|err| eyre!("could not fetch bootstrap: {}", err))?;

        let is_valid = self.is_valid_checkpoint(bootstrap.header.beacon.slot);
        let strict_checkpoint_age = read_state(|s| s.config.strict_checkpoint_age);
        if !is_valid {
            if strict_checkpoint_age {
                return Err(ConsensusError::CheckpointTooOld.into());
            } else {
                log!(
                    WARNING,
                    "checkpoint too old, consider using a more recent block"
                );
            }
        }
        let forks = read_state(|s| s.config.forks);
        verify_bootstrap::<S>(&bootstrap, checkpoint, &forks)?;
        apply_bootstrap::<S>(&mut self.store, &bootstrap);
        Ok(())
    }

    pub fn verify_update(&self, update: &Update) -> Result<()> {
        let config = read_state(|s| s.config.clone());
        verify_update::<S>(
            update,
            self.expected_current_slot(),
            &self.store,
            config.chain.genesis_root.clone(),
            &config.forks,
        )
    }

    fn verify_finality_update(&self, update: &FinalityUpdate) -> Result<()> {
        let config = read_state(|s| s.config.clone());
        verify_finality_update::<S>(
            update,
            self.expected_current_slot(),
            &self.store,
            config.chain.genesis_root,
            &config.forks,
        )
    }

    pub fn apply_update(&mut self, update: &Update) {
        let new_checkpoint = apply_update::<S>(&mut self.store, update);
        StateModifier::update_last_checkpoint(new_checkpoint);
    }

    fn apply_finality_update(&mut self, update: &FinalityUpdate) {
        let prev_finalized_slot = self.store.finalized_header.beacon.slot;
        let prev_optimistic_slot = self.store.optimistic_header.beacon.slot;
        let new_checkpoint = apply_finality_update::<S>(&mut self.store, update);
        let new_finalized_slot = self.store.finalized_header.beacon.slot;
        let new_optimistic_slot = self.store.optimistic_header.beacon.slot;
        StateModifier::update_last_checkpoint(new_checkpoint);
        if new_finalized_slot != prev_finalized_slot {
            self.log_finality_update(update);
        }
        if new_optimistic_slot != prev_optimistic_slot {
            self.log_optimistic_update(update)
        }
    }

    fn log_finality_update(&self, update: &FinalityUpdate) {
        let size = S::sync_commitee_size() as f32;
        let participation =
            get_bits::<S>(&update.sync_aggregate.sync_committee_bits) as f32 / size * 100f32;
        let decimals = if participation == 100.0 { 1 } else { 2 };

        log!(
            INFO,
            "finalized slot             slot={}  confidence={:.decimals$}%",
            self.store.finalized_header.beacon.slot,
            participation,
        );
    }

    fn log_optimistic_update(&self, update: &FinalityUpdate) {
        let size = S::sync_commitee_size() as f32;
        let participation =
            get_bits::<S>(&update.sync_aggregate.sync_committee_bits) as f32 / size * 100f32;
        let decimals = if participation == 100.0 { 1 } else { 2 };

        log!(
            INFO,
            "updated head               slot={}  confidence={:.decimals$}%",
            self.store.optimistic_header.beacon.slot,
            participation
        );
    }

    pub fn expected_current_slot(&self) -> u64 {
        let now = ic_cdk::api::time();
        let genesis_time = read_state(|s| s.config.chain.genesis_time);
        expected_current_slot(now, genesis_time)
    }

    fn slot_timestamp(&self, slot: u64) -> u64 {
        let genesis_time = read_state(|s| s.config.chain.genesis_time);
        slot * 12 + genesis_time
    }

    // Determines blockhash_slot age and returns true if it is less than 14 days old
    fn is_valid_checkpoint(&self, blockhash_slot: u64) -> bool {
        let current_slot = self.expected_current_slot();
        let current_slot_timestamp = self.slot_timestamp(current_slot);
        let blockhash_slot_timestamp = self.slot_timestamp(blockhash_slot);

        let slot_age = current_slot_timestamp
            .checked_sub(blockhash_slot_timestamp)
            .unwrap_or_default();
        let max_checkpoint_age = read_state(|s| s.config.max_checkpoint_age);
        slot_age < max_checkpoint_age
    }
}
