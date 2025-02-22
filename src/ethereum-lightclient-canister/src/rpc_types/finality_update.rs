use candid::{CandidType, Deserialize};
use serde::Serialize;
use crate::rpc_types::lightclient_header::{LightClientHeader, SyncAggregate};

#[derive(Serialize, Deserialize,Debug, CandidType)]
pub struct FinalityUpdate {
    pub attested_header: LightClientHeader,
    pub finalized_header: LightClientHeader,
    pub finality_branch: Vec<String>,
    pub sync_aggregate: SyncAggregate,
    #[serde(with = "crate::rpc_types::serde_utils::u64")]
    pub signature_slot: u64,
}

#[derive(Deserialize, Debug)]
pub struct FinalityUpdateResponse {
    pub data: FinalityUpdate,
}