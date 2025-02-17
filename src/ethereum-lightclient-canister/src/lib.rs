use std::cell::RefCell;

use candid::Nat;
use ic_cdk::{init, post_upgrade, pre_upgrade, query, update};
use ic_cdk_timers::set_timer;
use interface::{
    Address, Erc20BalanceOfRequest, Erc721OwnerOfRequest, Network,
    SetupRequest, U256,
};
use log::{debug, error};

use crate::state::{mutate_state, read_state};

mod helios;
mod random;
mod stable_memory;
mod utils;
mod state;
mod storable_structures;
mod ic_consensus_rpc;
mod ic_consensus_rpc_types;
mod consensus;


#[init]
async fn init() {
    ic_cdk::setup();
}

#[update]
async fn setup(request: SetupRequest) {
    let _ = ic_logger::init_with_level(log::Level::Trace);

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
    let _ = ic_logger::init_with_level(log::Level::Trace);

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
    });
}
