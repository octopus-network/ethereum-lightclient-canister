use std::sync::{Arc, Mutex};

use arc_swap::ArcSwap;
use config::networks::Network;
use helios_consensus_core::errors::ConsensusError;
use ethers_core::types::{
    Address, FeeHistory, Filter, Log, SyncingStatus, Transaction, TransactionReceipt, H256, U256,
};
use eyre::{eyre, Result};

use common::types::BlockTag;
use config::{CheckpointFallback, Config};
use consensus::{ ConsensusClient};
use log::{debug, error, info, warn};

#[cfg(target_arch = "wasm32")]
use ic_cdk::spawn;
#[cfg(target_arch = "wasm32")]
use ic_cdk_timers::{clear_timer, set_timer_interval, TimerId};
#[cfg(target_arch = "wasm32")]
use std::time::Duration;
use serde::{Deserialize, Serialize};

use crate::database::Database;
use crate::errors::NodeError;
use crate::node::Node;

#[derive(Default)]
pub struct ClientBuilder {
    network: Option<Network>,
    consensus_rpc: Option<String>,
    execution_rpc: Option<String>,
    checkpoint: Option<Vec<u8>>,
    config: Option<Config>,
    fallback: Option<String>,
    load_external_fallback: bool,
    strict_checkpoint_age: bool,
}

impl ClientBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn network(mut self, network: Network) -> Self {
        self.network = Some(network);
        self
    }

    pub fn consensus_rpc(mut self, consensus_rpc: &str) -> Self {
        self.consensus_rpc = Some(consensus_rpc.to_string());
        self
    }

    pub fn execution_rpc(mut self, execution_rpc: &str) -> Self {
        self.execution_rpc = Some(execution_rpc.to_string());
        self
    }

    pub fn checkpoint(mut self, checkpoint: &str) -> Self {
        let checkpoint = hex::decode(checkpoint.strip_prefix("0x").unwrap_or(checkpoint))
            .expect("cannot parse checkpoint");
        self.checkpoint = Some(checkpoint);
        self
    }

    pub fn config(mut self, config: Config) -> Self {
        self.config = Some(config);
        self
    }

    pub fn fallback(mut self, fallback: &str) -> Self {
        self.fallback = Some(fallback.to_string());
        self
    }

    pub fn load_external_fallback(mut self) -> Self {
        self.load_external_fallback = true;
        self
    }

    pub fn strict_checkpoint_age(mut self) -> Self {
        self.strict_checkpoint_age = true;
        self
    }

    pub fn build<DB: Database>(self) -> Result<Client<DB>> {
        let base_config = if let Some(network) = self.network {
            network.to_base_config()
        } else {
            let config = self
                .config
                .as_ref()
                .ok_or(eyre!("missing network config"))?;
            config.to_base_config()
        };

        let consensus_rpc = self.consensus_rpc.unwrap_or_else(|| {
            self.config
                .as_ref()
                .expect("missing consensus rpc")
                .consensus_rpc
                .clone()
        });

        let execution_rpc = self.execution_rpc.unwrap_or_else(|| {
            self.config
                .as_ref()
                .expect("missing execution rpc")
                .execution_rpc
                .clone()
        });

        let checkpoint = if let Some(checkpoint) = self.checkpoint {
            Some(checkpoint)
        } else if let Some(config) = &self.config {
            config.checkpoint.clone()
        } else {
            None
        };

        let default_checkpoint = if let Some(config) = &self.config {
            config.default_checkpoint.clone()
        } else {
            base_config.default_checkpoint.clone()
        };

        let fallback = if self.fallback.is_some() {
            self.fallback
        } else if let Some(config) = &self.config {
            config.fallback.clone()
        } else {
            None
        };

        let load_external_fallback = if let Some(config) = &self.config {
            self.load_external_fallback || config.load_external_fallback
        } else {
            self.load_external_fallback
        };

        let strict_checkpoint_age = if let Some(config) = &self.config {
            self.strict_checkpoint_age || config.strict_checkpoint_age
        } else {
            self.strict_checkpoint_age
        };

        let config = Config {
            consensus_rpc,
            execution_rpc,
            checkpoint,
            default_checkpoint,

            #[cfg(target_arch = "wasm32")]
            rpc_bind_ip: None,
            #[cfg(target_arch = "wasm32")]
            rpc_port: None,
            #[cfg(target_arch = "wasm32")]
            data_dir: None,
            chain: base_config.chain,
            forks: base_config.forks,
            max_checkpoint_age: base_config.max_checkpoint_age,
            fallback,
            load_external_fallback,
            strict_checkpoint_age,
        };

        Client::new(config)
    }
}


pub struct Client<DB: Database> {
    node: Arc<ArcSwap<Node>>,
    db: DB,
    fallback: Option<String>,
    load_external_fallback: bool,
}

impl<DB: Database> Client<DB> {
    fn new(mut config: Config) -> Result<Self> {
        let db = DB::new(&config)?;
        if config.checkpoint.is_none() {
            let checkpoint = db.load_checkpoint()?;
            config.checkpoint = Some(checkpoint);
        }

        let config = Arc::new(config);
        let node = Node::new(config.clone())?;
        let node = Arc::new(ArcSwap::from(Arc::new(node)));


        Ok(Client {
            node,
            db,
            fallback: config.fallback.clone(),
            load_external_fallback: config.load_external_fallback,
        })
    }

    pub async fn start(&mut self) -> Result<()> {
        let mut node = Node::clone(&*self.node.load());

        let sync_res = node.sync().await;

        if let Err(err) = sync_res {
            match err {
                NodeError::ConsensusSyncError(err) => match err.downcast_ref() {
                    Some(ConsensusError::CheckpointTooOld) => {
                        warn!(
                            "failed to sync consensus node with checkpoint: 0x{}",
                            hex::encode(node.config.checkpoint.clone().unwrap_or_default()),
                        );

                        let fallback = self.boot_from_fallback(&mut node).await;
                        if fallback.is_err() && self.load_external_fallback {
                            self.boot_from_external_fallbacks(&mut node).await?
                        } else if fallback.is_err() {
                            error!("Invalid checkpoint. Please update your checkpoint too a more recent block. Alternatively, set an explicit checkpoint fallback service url with the `-f` flag or use the configured external fallback services with `-l` (NOT RECOMMENDED). See https://github.com/a16z/helios#additional-options for more information.");
                            return Err(err);
                        }
                    }
                    _ => return Err(err),
                },
                _ => return Err(err.into()),
            }
        }

        self.node.store(Arc::new(node));
        self.start_advance_thread();

        Ok(())
    }

    #[cfg(target_arch = "wasm32")]
    fn start_advance_thread(&self) {
        let arc_swap_node = self.node.clone();
        let in_progress_guard = Arc::new(Mutex::new(()));

        let timer_id = set_timer_interval(Duration::from_secs(12), move || {
            let arc_swap_node = arc_swap_node.clone();
            let in_progress_guard = in_progress_guard.clone();

            spawn(async move {
                // This is std mutex in async context, so it should be used only with try_lock!
                let Ok(_guard) = in_progress_guard.try_lock() else {
                    debug!("Advancing already in progress");
                    return;
                };

                debug!("Advancing the client");

                let mut node = Node::clone(&*arc_swap_node.load());

                match node.advance().await {
                    Ok(_) => arc_swap_node.store(Arc::new(node)),
                    Err(e) => warn!("advancing node error: {e}"),
                }

                debug!("Advancing finished");
            });
        });
    }

    async fn boot_from_fallback(&self, node: &mut Node) -> eyre::Result<()> {
        if let Some(fallback) = &self.fallback {
            info!(
                "attempting to load checkpoint from fallback \"{}\"",
                fallback
            );

            let checkpoint = CheckpointFallback::fetch_checkpoint_from_api(fallback)
                .await
                .map_err(|_| {
                    eyre::eyre!("Failed to fetch checkpoint from fallback \"{}\"", fallback)
                })?;

            info!(
                "external fallbacks responded with checkpoint 0x{:?}",
                checkpoint
            );

            // Try to sync again with the new checkpoint by reconstructing the consensus client
            // We fail fast here since the node is unrecoverable at this point
            let config = node.config.clone();
            let consensus =
                ConsensusClient::new(&config.consensus_rpc, checkpoint.as_bytes(), config.clone())?;
            node.consensus = consensus;
            node.sync().await?;

            Ok(())
        } else {
            Err(eyre::eyre!("no explicit fallback specified"))
        }
    }

    async fn boot_from_external_fallbacks(&self, node: &mut Node) -> eyre::Result<()> {
        info!("attempting to fetch checkpoint from external fallbacks...");
        // Build the list of external checkpoint fallback services
        let list = CheckpointFallback::new()
            .build()
            .await
            .map_err(|_| eyre::eyre!("Failed to construct external checkpoint sync fallbacks"))?;

        let checkpoint = if node.config.chain.chain_id == 5 {
            list.fetch_latest_checkpoint(&Network::GOERLI)
                .await
                .map_err(|_| {
                    eyre::eyre!("Failed to fetch latest goerli checkpoint from external fallbacks")
                })?
        } else {
            list.fetch_latest_checkpoint(&Network::MAINNET)
                .await
                .map_err(|_| {
                    eyre::eyre!("Failed to fetch latest mainnet checkpoint from external fallbacks")
                })?
        };

        info!(
            "external fallbacks responded with checkpoint {:?}",
            checkpoint
        );

        // Try to sync again with the new checkpoint by reconstructing the consensus client
        // We fail fast here since the node is unrecoverable at this point
        let config = node.config.clone();
        let consensus =
            ConsensusClient::new(&config.consensus_rpc, checkpoint.as_bytes(), config.clone())?;
        node.consensus = consensus;
        node.sync().await?;

        Ok(())
    }


    pub fn get_last_checkpoint(&self) -> Option<String> {
        self.node
            .load()
            .get_last_checkpoint()
            .map(|checkpoint| format!("0x{}", hex::encode(checkpoint)))
    }

}
