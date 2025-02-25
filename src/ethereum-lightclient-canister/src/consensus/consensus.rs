use std::marker::PhantomData;

use eyre::eyre;
use eyre::Result;
use ic_canister_log::log;
use serde::{Deserialize, Serialize};

use tree_hash::fixed_bytes::B256;

use crate::config::Config;
use crate::consensus::consensus_spec::{calc_sync_period, ConsensusSpec, MainnetConsensusSpec};
use crate::consensus::core::{apply_bootstrap, apply_finality_update, apply_update, expected_current_slot, get_bits, verify_bootstrap, verify_finality_update, verify_update};
use crate::consensus::errors::ConsensusError;
use crate::ic_consensus_rpc::{IcpConsensusRpc, MAX_REQUEST_LIGHT_CLIENT_UPDATES};
use crate::ic_log::{INFO, WARNING};
use crate::rpc_types::finality_update::FinalityUpdate;
use crate::rpc_types::lightclient_store::LightClientStore;
use crate::rpc_types::update::Update;
use crate::state::{read_state, StateModifier};
use crate::storable_structures::BlockInfo;

#[derive(Debug, Serialize, Deserialize)]
pub struct Inner<S: ConsensusSpec> {
    pub(crate) store: LightClientStore,
    phantom_data: PhantomData<S>
}



pub async fn start_advance_thread(rpc: &str, config: Config) {

   /* let initial_checkpoint = if let Some(c) = read_state(|s|s.last_checkpoint.clone()) {
        c
    } else { config.default_checkpoint };

    let mut inner = Inner::<MainnetConsensusSpec>::new();
    let res = inner.sync(initial_checkpoint).await;
    match res {
        Ok(_) => {
            inner.store().await;
            //Start task
        }
        Err(_) => {

        }
    }
*/
    /*
       let run = wasm_bindgen_futures::spawn_local;
       run(async move {



           if let Err(err) = res {
               if config.load_external_fallback {
                   let res = sync_all_fallbacks(&mut inner, config.chain.chain_id).await;
                   if let Err(err) = res {
                       error!(target: "helios::consensus", err = %err, "sync failed");
                       process::exit(1);
                   }
               } else if let Some(fallback) = &config.fallback {
                   let res = sync_fallback(&mut inner, fallback).await;
                   if let Err(err) = res {
                       error!(target: "helios::consensus", err = %err, "sync failed");
                       process::exit(1);
                   }
               } else {
                   error!(target: "helios::consensus", err = %err, "sync failed");
                   process::exit(1);
               }
           }

           //TODO
           //_ = inner.send_blocks().await;

         let start = Instant::now() + inner.duration_until_next_update().to_std().unwrap();
           let mut interval = interval_at(start, std::time::Duration::from_secs(12));

           loop {
               tokio::select! {
                       _ = interval.tick() => {
                           let res = inner.advance().await;
                           if let Err(err) = res {
                               warn!(target: "helios::consensus", "advance error: {}", err);
                               continue;
                           }
                           //TODO
                           //let res = inner.send_blocks().await;
                           if let Err(err) = res {
                               warn!(target: "helios::consensus", "send error: {}", err);
                               continue;
                           }
                       }
                   }
           }
       });*/

/*    save_new_checkpoints(
        checkpoint_recv.clone(),
        db.clone(),
        initial_checkpoint,
        shutdown_recv,
    );*/


}

async fn sync_fallback<S: ConsensusSpec>(
    inner: &mut Inner<S>,
    fallback: &str,
) -> Result<()> {
    /*let checkpoint = CheckpointFallback::fetch_checkpoint_from_api(fallback).await?;
    inner.sync(checkpoint).await*/
    //TODO


    Ok(())
}

async fn sync_all_fallbacks<S: ConsensusSpec>(
    inner: &mut Inner<S>,
    chain_id: u64,
) -> Result<()> {
/*    let network = Network::from_chain_id(chain_id)?;
    let checkpoint = CheckpointFallback::new()
        .build()
        .await?
        .fetch_latest_checkpoint(&network)
        .await?;

    inner.sync(checkpoint).await*/
    //TODO
    Ok(())
}

impl<S: ConsensusSpec> Inner<S> {
    pub fn new(
        store: LightClientStore,
    ) -> Inner<S> {
        Inner {
            store,
            phantom_data: Default::default(),
        }
    }

    pub async fn sync(&mut self, checkpoint: B256) -> Result<()> {
        self.bootstrap(checkpoint).await?;
        let current_period = calc_sync_period::<S>(self.store.finalized_header.beacon.slot);
        let updates = IcpConsensusRpc::get_updates(current_period, MAX_REQUEST_LIGHT_CLIENT_UPDATES)
            .await?;

        for update in updates {
            self.verify_update(&update)?;
            self.apply_update(&update);
        }

        let finality_update = IcpConsensusRpc::get_finality_update().await?;
        self.verify_finality_update(&finality_update)?;
        self.apply_finality_update(&finality_update);
        log!( INFO,
            "consensus client in sync with checkpoint: 0x{}",
            hex::encode(checkpoint.0.as_ref())
        );

        Ok(())
    }

    pub async fn advance(&mut self) -> Result<()> {
        let finality_update = IcpConsensusRpc::get_finality_update().await?;
        self.verify_finality_update(&finality_update)?;
        self.apply_finality_update(&finality_update);

        if self.store.next_sync_committee.is_none() {
            log!(INFO, "checking for sync committee update");
            let current_period = calc_sync_period::<MainnetConsensusSpec>(self.store.finalized_header.beacon.slot);
            let mut updates = IcpConsensusRpc::get_updates(current_period, 1).await?;

            if updates.len() == 1 {
                let update = updates.get_mut(0).unwrap();
                let res = self.verify_update(update);

                if res.is_ok() {
                    log!(INFO,"updating sync committee");
                    self.apply_update(update);
                }
            }
        }

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
        let bootstrap = IcpConsensusRpc
            ::get_bootstrap(checkpoint)
            .await
            .map_err(|err| eyre!("could not fetch bootstrap: {}", err))?;

        let is_valid = self.is_valid_checkpoint(bootstrap.header.beacon.slot);
        let strict_checkpoint_age = read_state(|s|s.config.strict_checkpoint_age);
        if !is_valid {
            if strict_checkpoint_age {
                return Err(ConsensusError::CheckpointTooOld.into());
            } else {
                log!(WARNING,"checkpoint too old, consider using a more recent block");
            }
        }
        let forks = read_state(|s|s.config.forks);
        verify_bootstrap::<S>(&bootstrap, checkpoint, &forks)?;
        apply_bootstrap::<S>(&mut self.store, &bootstrap);
        Ok(())
    }

    pub fn verify_update(&self, update: &Update) -> Result<()> {
        let config = read_state(|s|s.config.clone());
        verify_update::<S>(
            update,
            self.expected_current_slot(),
            &self.store,
            config.chain.genesis_root.clone(),
            &config.forks,
        )
    }

    fn verify_finality_update(&self, update: &FinalityUpdate) -> Result<()> {
        let config = read_state(|s|s.config.clone());
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

        log!(INFO,
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

        log!(INFO,
            "updated head               slot={}  confidence={:.decimals$}%",
            self.store.optimistic_header.beacon.slot,
            participation
        );
    }

    pub fn expected_current_slot(&self) -> u64 {
        let now = ic_cdk::api::time();
        let genesis_time = read_state(|s|s.config.chain.genesis_time);
        expected_current_slot(now, genesis_time)
    }

    fn slot_timestamp(&self, slot: u64) -> u64 {
        let genesis_time  = read_state(|s|s.config.chain.genesis_time);
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
        let max_checkpoint_age = read_state(|s|s.config.max_checkpoint_age);
        slot_age < max_checkpoint_age
    }
}


/*
fn payload_to_block(value: ExecutionPayload) -> Block<Transaction> {
    let empty_nonce = fixed_bytes!("0000000000000000");
    let empty_uncle_hash =
        b256!("1dcc4de8dec75d7aab85b567b6ccd41ad312451b948a7413f0a142fd40d49347");

    let txs = value
        .transactions()
        .iter()
        .enumerate()
        .map(|(i, tx_bytes)| {
            let tx_bytes = tx_bytes.inner.to_vec();
            let mut tx_bytes_slice = tx_bytes.as_slice();
            let tx_envelope = TxEnvelope::decode(&mut tx_bytes_slice).unwrap();

            let base_fee = Some(value.base_fee_per_gas().to());

            Transaction {
                block_hash: Some(*value.block_hash()),
                block_number: Some(*value.block_number()),
                transaction_index: Some(i as u64),
                from: tx_envelope.recover_signer().unwrap().clone(),
                effective_gas_price: Some(tx_envelope.effective_gas_price(base_fee)),
                inner: tx_envelope,
            }
        })
        .collect::<Vec<_>>();
    let tx_envelopes = txs.iter().map(|tx| tx.inner.clone()).collect::<Vec<_>>();
    let txs_root = calculate_transaction_root(&tx_envelopes);

    let withdrawals: Vec<Withdrawal> = value
        .withdrawals()
        .unwrap()
        .into_iter()
        .map(|w| w.clone().into())
        .collect();
    let withdrawals_root = calculate_withdrawals_root(&withdrawals);

    let logs_bloom: Bloom =
        Bloom::from(BloomInput::Raw(&value.logs_bloom().clone().inner.to_vec()));

    let consensus_header = ConsensusHeader {
        parent_hash: *value.parent_hash(),
        ommers_hash: empty_uncle_hash,
        beneficiary: *value.fee_recipient(),
        state_root: *value.state_root(),
        transactions_root: txs_root,
        receipts_root: *value.receipts_root(),
        withdrawals_root: Some(withdrawals_root),
        logs_bloom: logs_bloom,
        difficulty: U256::ZERO,
        number: *value.block_number(),
        gas_limit: *value.gas_limit(),
        gas_used: *value.gas_used(),
        timestamp: *value.timestamp(),
        mix_hash: *value.prev_randao(),
        nonce: empty_nonce,
        base_fee_per_gas: Some(value.base_fee_per_gas().to::<u64>()),
        blob_gas_used: value.blob_gas_used().cloned().ok(),
        excess_blob_gas: value.excess_blob_gas().cloned().ok(),
        parent_beacon_block_root: None,
        extra_data: value.extra_data().inner.to_vec().into(),
        requests_hash: None,
    };

    let header = Header {
        hash: *value.block_hash(),
        inner: consensus_header,
        total_difficulty: Some(U256::ZERO),
        size: Some(U256::ZERO),
    };

    Block::new(header, BlockTransactions::Full(txs))
        .with_withdrawals(Some(Withdrawals::new(withdrawals)))
}*/