use crate::rpc_types::lightclient_header::{LightClientHeader, SyncAggregate};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct OptimisticUpdate {
    pub attested_header: LightClientHeader,
    pub sync_aggregate: SyncAggregate,
    #[serde(with = "crate::rpc_types::serde_utils::u64")]
    pub signature_slot: u64,
}
