use serde::{Deserialize, Serialize};
use crate::rpc_types::bootstrap::SyncCommittee;
use crate::rpc_types::lightclient_header::{LightClientHeader, SyncAggregate};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Update {
    pub attested_header: LightClientHeader,
    pub next_sync_committee: SyncCommittee,
    pub next_sync_committee_branch: Vec<String>,
    pub finalized_header: LightClientHeader,
    pub finality_branch: Vec<String>,
    pub sync_aggregate: SyncAggregate,
    pub signature_slot: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct UpdateData {
    pub data: Update,
}