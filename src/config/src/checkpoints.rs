use std::collections::HashMap;

use common::http;
use serde::{Deserialize, Serialize};

use crate::networks;

/// The location where the list of checkpoint services are stored.
pub const CHECKPOINT_SYNC_SERVICES_LIST: &str = "https://raw.githubusercontent.com/ethpandaops/checkpoint-sync-health-checks/master/_data/endpoints.yaml";

#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RawSlotResponse {
    pub data: RawSlotResponseData,
}

#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RawSlotResponseData {
    pub slots: Vec<Slot>,
}

#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Slot {
    pub slot: u64,
    pub block_root: Option<String>,
    pub state_root: Option<String>,
    pub epoch: u64,
    pub time: StartEndTime,
}

#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StartEndTime {
    /// An ISO 8601 formatted UTC timestamp.
    pub start_time: String,
    /// An ISO 8601 formatted UTC timestamp.
    pub end_time: String,
}

/// A health check for the checkpoint sync service.
#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Health {
    /// If the node is healthy.
    pub result: bool,
    /// An [ISO 8601](https://en.wikipedia.org/wiki/ISO_8601) UTC timestamp.
    pub date: String,
}

/// A checkpoint fallback service.
#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CheckpointFallbackService {
    /// The endpoint for the checkpoint sync service.
    pub endpoint: String,
    /// The checkpoint sync service name.
    pub name: String,
    /// The service state.
    pub state: bool,
    /// If the service is verified.
    pub verification: bool,
    /// Contact information for the service maintainers.
    pub contacts: Option<serde_yaml::Value>,
    /// Service Notes
    pub notes: Option<serde_yaml::Value>,
    /// The service health check.
    pub health: Vec<Health>,
}

/// The CheckpointFallback manages checkpoint fallback services.
#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CheckpointFallback {
    /// Services Map
    pub services: HashMap<networks::Network, Vec<CheckpointFallbackService>>,
    /// A list of supported networks to build.
    /// Default: [mainnet, goerli]
    pub networks: Vec<networks::Network>,
}

impl CheckpointFallback {
    /// Constructs a new checkpoint fallback service.
    pub fn new() -> Self {
        Self {
            services: Default::default(),
            networks: [networks::Network::MAINNET, networks::Network::GOERLI].to_vec(),
        }
    }

    /// Build the checkpoint fallback service from the community-maintained list by [ethPandaOps](https://github.com/ethpandaops).
    ///
    /// The list is defined in [ethPandaOps/checkpoint-fallback-service](https://github.com/ethpandaops/checkpoint-sync-health-checks/blob/master/_data/endpoints.yaml).
    pub async fn build(mut self) -> eyre::Result<Self> {
        // Fetch the services
        let resp = http::get(CHECKPOINT_SYNC_SERVICES_LIST).await?;
        let yaml = String::from_utf8(resp.body)?;

        // Parse the yaml content results.
        let list: serde_yaml::Value = serde_yaml::from_str(&yaml)?;

        // Construct the services mapping from network <> list of services
        let mut services = HashMap::new();
        for network in &self.networks {
            // Try to parse list of checkpoint fallback services
            let service_list = list
                .get(network.to_string().to_lowercase())
                .ok_or_else(|| {
                    eyre::eyre!(format!("missing {network} fallback checkpoint services"))
                })?;
            let parsed: Vec<CheckpointFallbackService> =
                serde_yaml::from_value(service_list.clone())?;
            services.insert(*network, parsed);
        }
        self.services = services;

        Ok(self)
    }


    pub fn construct_url(endpoint: &str) -> String {
        format!("{endpoint}/checkpointz/v1/beacon/slots")
    }

    /// Returns a list of all checkpoint fallback endpoints.
    ///
    /// ### Warning
    ///
    /// These services are not healthchecked **nor** trustworthy and may act with malice by returning invalid checkpoints.
    pub fn get_all_fallback_endpoints(&self, network: &networks::Network) -> Vec<String> {
        self.services[network]
            .iter()
            .map(|service| service.endpoint.clone())
            .collect()
    }

    /// Returns a list of healthchecked checkpoint fallback endpoints.
    ///
    /// ### Warning
    ///
    /// These services are not trustworthy and may act with malice by returning invalid checkpoints.
    pub fn get_healthy_fallback_endpoints(&self, network: &networks::Network) -> Vec<String> {
        self.services[network]
            .iter()
            .filter(|service| service.state)
            .map(|service| service.endpoint.clone())
            .collect()
    }

    /// Returns a list of healthchecked checkpoint fallback services.
    ///
    /// ### Warning
    ///
    /// These services are not trustworthy and may act with malice by returning invalid checkpoints.
    pub fn get_healthy_fallback_services(
        &self,
        network: &networks::Network,
    ) -> Vec<CheckpointFallbackService> {
        self.services[network]
            .iter()
            .filter(|service| service.state)
            .cloned()
            .collect::<Vec<CheckpointFallbackService>>()
    }

    /// Returns the raw checkpoint fallback service objects for a given network.
    pub fn get_fallback_services(
        &self,
        network: &networks::Network,
    ) -> &Vec<CheckpointFallbackService> {
        self.services[network].as_ref()
    }
}
