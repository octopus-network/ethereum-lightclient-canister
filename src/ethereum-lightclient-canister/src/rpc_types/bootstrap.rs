use serde::{Deserialize, Serialize};
use ssz_types::FixedVector;
use crate::consensus::consensus_spec::{ConsensusSpec, MainnetConsensusSpec};
use crate::rpc_types::bls::PublicKey;
use crate::rpc_types::lightclient_header::LightClientHeader;

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct SyncCommittee {
    pub pubkeys: FixedVector<PublicKey, <MainnetConsensusSpec as ConsensusSpec>::SyncCommitteeSize>,
    pub aggregate_pubkey: PublicKey,
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