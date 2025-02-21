use serde::{Deserialize, Serialize};
use crate::rpc_types::lightclient_header::LightClientHeader;

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct SyncCommittee {
    pub pubkeys: Vec<String>,
    pub aggregate_pubkey: String,
}


#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct Bootstrap {
    pub header: LightClientHeader,
    pub current_sync_committee: SyncCommittee,
    pub current_sync_committee_branch: Vec<String>,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct BootstrapResponse {
    pub data: Bootstrap,
}