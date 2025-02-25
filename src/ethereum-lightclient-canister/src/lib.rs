use candid::CandidType;
use ic_cdk::{init, post_upgrade, pre_upgrade, update};

use log::{debug};
use serde::{Deserialize, Serialize};
use tree_hash::fixed_bytes::B256;
use crate::config::Config;
use crate::config::networks::mainnet;
use crate::consensus::consensus::Inner;
use crate::consensus::consensus_spec::MainnetConsensusSpec;
use crate::ic_consensus_rpc::IcpConsensusRpc;
use crate::ic_execution_rpc::IcExecutionRpc;
use crate::state::{LightClientState, read_state, replace_state};

mod stable_memory;
mod state;
mod storable_structures;
mod ic_consensus_rpc;
mod consensus;
mod ic_execution_rpc;
mod rpc_types;
mod config;
mod ic_log;


#[init]
async fn init(args: InitArgs) {
    let mut  config = Config::from(mainnet());
    config.execution_rpc = args.execution_rpc;
    let state = LightClientState::init(config);
    replace_state(state);
}

#[derive(CandidType, Deserialize, Serialize)]
pub struct InitArgs {
    pub execution_rpc: String,
}


#[pre_upgrade]
async fn pre_upgrade() {
    read_state(|s|s.pre_upgrade());
}

#[post_upgrade]
fn post_upgrade() {

   /* // Workaround because cross-canister calls are not allowed in post_upgrade.
    // Client will be started from a timer in a second.
    set_timer(std::time::Duration::from_secs(1), || {
        ic_cdk::spawn(async move {
            let (consensus_rpc_url, execution_rpc_url) = read_state(|s|(s.consensus_rpc.clone(), s.execution_rpc.clone()));
            let network = Network::Mainnet;
            let checkpoint = read_state(|s|s.last_checkpoint.clone());

            debug!(
                "Resuming client with: network = {}, execution_rpc_url = {}, consensus_rpc_url = {}, checkpoint: {:?}",
                network,
                &execution_rpc_url,
                &consensus_rpc_url,
                &checkpoint,
            );
        });
    });*/
}


#[update]
pub async fn query_block(block_hash: String) -> String {
    let rpc: IcExecutionRpc = IcExecutionRpc::new("https://mainnet.infura.io/v3/025779c23a2d44d1b1ecef2bfb4f2b29").unwrap();
    let b = B256::from_hex(block_hash.as_str());
    serde_json::to_string( &rpc.get_block(b).await.unwrap()).unwrap()

}

#[update]
pub async fn get_finality() -> String {
    serde_json::to_string(&IcpConsensusRpc::get_finality_update().await.unwrap()).unwrap()
}

ic_cdk::export_candid!();