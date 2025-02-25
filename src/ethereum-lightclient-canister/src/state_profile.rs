use candid::CandidType;
use serde::Deserialize;
use serde_derive::Serialize;

#[derive(Serialize,  CandidType, Deserialize)]
struct FinalizedBlock {
    pub receipt_root: String,
    pub parent_block_hash: String,
    pub block_number: i64,
    pub block_hash: String,
}


#[derive(Serialize,  CandidType, Deserialize)]
struct AggregatePubkey {
    pub inner: String,
}


#[derive(Serialize,  CandidType, Deserialize)]
struct Execution {
    pub parent_hash: String,
    pub fee_recipient: String,
    pub state_root: String,
    pub receipts_root: String,
    pub logs_bloom: String,
    pub prev_randao: String,
    pub block_number: String,
    pub gas_limit: String,
    pub gas_used: String,
    pub timestamp: String,
    pub extra_data: String,
    pub base_fee_per_gas: String,
    pub block_hash: String,
    pub transactions_root: String,
    pub withdrawals_root: String,
    pub blob_gas_used: String,
    pub excess_blob_gas: String,
}

#[derive(Serialize,  CandidType, Deserialize)]
struct Beacon {
    pub slot: String,
    pub proposer_index: String,
    pub parent_root: String,
    pub state_root: String,
    pub body_root: String,
}

#[derive(Serialize,  CandidType, Deserialize)]
struct Struct1 {
    pub beacon: Beacon,
    pub execution: Execution,
    pub execution_branch: Vec<String>,
}

#[derive(Serialize,  CandidType, Deserialize)]
struct Store {
    pub finalized_header: Struct1,
    pub optimistic_header: Struct1,
    pub previous_max_active_participants: i64,
    pub current_max_active_participants: i64,
}

#[derive(Serialize,  CandidType, Deserialize)]
struct ExecutionForks {
    pub prague_timestamp: i64,
}

#[derive(Serialize,  CandidType, Deserialize)]
struct Struct {
    pub epoch: i64,
    pub fork_version: String,
}

#[derive(Serialize,  CandidType, Deserialize)]
struct Forks {
    pub genesis: Struct,
    pub altair: Struct,
    pub bellatrix: Struct,
    pub capella: Struct,
    pub deneb: Struct,
    pub electra: Struct,
}

#[derive(Serialize,  CandidType, Deserialize)]
struct Chain {
    pub chain_id: i64,
    pub genesis_time: i64,
    pub genesis_root: String,
}

#[derive(Serialize,  CandidType, Deserialize)]
struct Config {
    pub consensus_rpc: String,
    pub execution_rpc: String,
    pub default_checkpoint: String,
    pub checkpoint: Option<String>,
    pub max_checkpoint_age: i64,
    pub load_external_fallback: bool,
    pub strict_checkpoint_age: bool,
}

#[derive(Serialize,  CandidType, Deserialize)]
pub struct StateProfileView {
    pub config: Config,
    pub last_checkpoint: String,
    pub store: Store,
    pub finalized_block: FinalizedBlock,
    pub history_length: i64,
}