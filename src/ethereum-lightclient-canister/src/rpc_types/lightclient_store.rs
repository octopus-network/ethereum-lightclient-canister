use serde::{Deserialize, Serialize};
use crate::rpc_types::bootstrap::SyncCommittee;
use crate::rpc_types::convert::{default_branch_to_none, default_header_to_none};
use crate::rpc_types::finality_update::FinalityUpdate;
use crate::rpc_types::lightclient_header::{LightClientHeader, SyncAggregate};

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct LightClientStore {
    pub finalized_header: LightClientHeader,
    pub current_sync_committee: SyncCommittee,
    pub next_sync_committee: Option<SyncCommittee>,
    pub optimistic_header: LightClientHeader,
    pub previous_max_active_participants: u64,
    pub current_max_active_participants: u64,
    pub best_valid_update: Option<GenericUpdate>,
}



#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct GenericUpdate {
    pub attested_header: LightClientHeader,
    pub sync_aggregate: SyncAggregate,
    pub signature_slot: String,
    pub next_sync_committee: Option<SyncCommittee>,
    pub next_sync_committee_branch: Option<Vec<String>>,
    pub finalized_header: Option<LightClientHeader>,
    pub finality_branch: Option<Vec<String>>,
}


impl From<&FinalityUpdate> for GenericUpdate {
    fn from(update: &FinalityUpdate) -> Self {
        Self {
            attested_header: update.attested_header.clone(),
            sync_aggregate: update.sync_aggregate.clone(),
            signature_slot: update.signature_slot.clone(),
            next_sync_committee: None,
            next_sync_committee_branch: None,
            finalized_header: default_header_to_none(update.finalized_header.clone()),
            finality_branch: default_branch_to_none(&update.finality_branch),
        }
    }
}