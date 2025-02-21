use serde::{Deserialize, Serialize};
use crate::rpc_types::lightclient_header::LightClientHeader;

#[derive(Serialize, Deserialize,Debug)]
pub struct CurrentSyncCommittee {
    pub pubkeys: Vec<String>,
    pub aggregate_pubkey: String,
}


#[derive(Serialize, Deserialize,Debug)]
pub struct Bootstrap {
    pub header: LightClientHeader,
    pub current_sync_committee: CurrentSyncCommittee,
    pub current_sync_committee_branch: Vec<String>,
}

#[derive(Deserialize, Debug)]
pub struct BootstrapResponse {
    pub data: Bootstrap,
}