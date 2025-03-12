use crate::rpc_types::bootstrap::SyncCommittee;
use crate::rpc_types::lightclient_header::{LightClientHeader, SyncAggregate};
use serde::{Deserialize, Serialize};
use tree_hash::fixed_bytes::B256;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Update {
    pub attested_header: LightClientHeader,
    pub next_sync_committee: SyncCommittee,
    pub next_sync_committee_branch: Vec<B256>,
    pub finalized_header: LightClientHeader,
    pub finality_branch: Vec<B256>,
    pub sync_aggregate: SyncAggregate,
    #[serde(with = "crate::rpc_types::serde_utils::u64")]
    pub signature_slot: u64,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct UpdateData {
    pub data: Update,
}
