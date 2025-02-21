use std::str::FromStr;
//use alloy::primitives::B256;
use candid::{export_service, Nat};
use ic_cdk::{init, post_upgrade, pre_upgrade, query, update};
use ic_cdk_timers::set_timer;
/*use interface::{Network,
    SetupRequest
};*/
use log::{debug, error};
use crate::ic_consensus_rpc::IcpConsensusRpc;
use crate::ic_execution_rpc::IcExecutionRpc;
use crate::rpc_types::block::ExecutionBlock;
use crate::rpc_types::finality_update::FinalityUpdate;
//use helios_core::execution::rpc::ExecutionRpc;
//use crate::consensus::spec::Ethereum;
//use crate::ic_execution_rpc::IcExecutionRpc;

use crate::state::{mutate_state, read_state};

mod stable_memory;
mod state;
mod storable_structures;
mod ic_consensus_rpc;
mod consensus;
mod ic_execution_rpc;
mod rpc_types;


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
pub async fn query_block(block_hash: String) -> ExecutionBlock {
    let rpc: IcExecutionRpc = IcExecutionRpc::new("https://mainnet.infura.io/v3/025779c23a2d44d1b1ecef2bfb4f2b29").unwrap();
    rpc.get_block(block_hash).await.unwrap()
}

#[update]
pub async fn get_finality() -> FinalityUpdate {
    let rpc: IcpConsensusRpc = IcpConsensusRpc::new("https://ethereum.operationsolarstorm.org");
    rpc.get_finality_update().await.unwrap()
}

ic_cdk::export_candid!();