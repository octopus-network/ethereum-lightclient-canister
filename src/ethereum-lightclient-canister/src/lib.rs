use std::str::FromStr;
//use alloy::primitives::B256;
use candid::{export_service, Nat};
use ic_cdk::{init, post_upgrade, pre_upgrade, query, update};
use ic_cdk_timers::set_timer;
/*use interface::{Network,
    SetupRequest
};*/
use log::{debug, error};
//use helios_core::execution::rpc::ExecutionRpc;
//use crate::consensus::spec::Ethereum;
//use crate::ic_execution_rpc::IcExecutionRpc;

use crate::state::{mutate_state, read_state};

mod helios;
mod random;
mod stable_memory;
mod utils;
mod state;
mod storable_structures;
mod ic_consensus_rpc;
//mod consensus;
mod ic_execution_rpc;


#[init]
async fn init() {

}



#[pre_upgrade]
async fn pre_upgrade() {
    debug!("Stopping client");

/*    let checkpoint = helios::get_last_checkpoint();
    mutate_state(|s|s.last_checkpoint = checkpoint);

    helios::shutdown().await;*/

    debug!("Client stopped");
}

#[post_upgrade]
fn post_upgrade() {
  /*  let _ = ic_logger::init_with_level(log::Level::Trace);

    // Workaround because cross-canister calls are not allowed in post_upgrade.
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
/*    let rpc: IcExecutionRpc<Ethereum> = IcExecutionRpc::<Ethereum>::new("https://mainnet.infura.io/v3/025779c23a2d44d1b1ecef2bfb4f2b29").unwrap();
    let b256 = B256::from_str(block_hash.as_str()).unwrap();
    let r = rpc.get_block(b256).await.unwrap();*/
    //serde_json::to_string(&r).unwrap()
    "".to_string()
}

ic_cdk::export_candid!();