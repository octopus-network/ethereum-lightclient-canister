use std::cell::RefCell;

use candid::Nat;
use ic_cdk::{init, post_upgrade, pre_upgrade, query, update};
use ic_cdk_timers::set_timer;
use interface::{
    Address, Erc20BalanceOfRequest, Erc721OwnerOfRequest, EstimateGasRequest, Network,
    SetupRequest, U256,
};
use log::{debug, error};

use crate::state::{mutate_state, read_state};
use crate::utils::IntoCallOpts;

mod erc20;
mod erc721;
mod helios;
mod random;
mod stable_memory;
mod utils;
mod state;
mod storable_structures;


#[init]
async fn init() {
    ic_cdk::setup();
}

/// Setup the helios client with given node urls
///
/// Mainnet:
///   dfx canister call ethereum_canister setup \
///     'record { network = variant { Mainnet }; execution_rpc_url = "https://ethereum.publicnode.com"; consensus_rpc_url = "https://www.lightclientdata.org" }'
///
/// Goerli:
///   dfx canister call ethereum_canister setup \
///     'record { network = variant { Goerli }; execution_rpc_url = "https://ethereum-goerli.publicnode.com"; consensus_rpc_url = "TODO" }'
#[update]
async fn setup(request: SetupRequest) {
    let _ = ic_logger::init_with_level(log::Level::Trace);

    helios::start_client(
        request.network,
        &request.consensus_rpc_url,
        &request.execution_rpc_url,
        request.checkpoint,
    )
    .await
    .expect("starting client failed");

}

#[query]
fn get_block_number() -> Nat {
    let helios = helios::client();

    let head_block_num = helios.get_block_number().expect("get_block_number failed");

    head_block_num.into()
}

#[query]
fn get_gas_price() -> U256 {
    let helios = helios::client();

    let gas_price = helios.get_gas_price().expect("get_gas_price failed");

    gas_price.into()
}

#[update]
async fn estimate_gas(request: EstimateGasRequest) -> U256 {
    let helios = helios::client();

    let gas_cost_estimation = helios
        .estimate_gas(&request.into_call_opts())
        .await
        .expect("estimate_gas failed");

    gas_cost_estimation.into()
}

#[update]
async fn erc20_balance_of(request: Erc20BalanceOfRequest) -> U256 {
    erc20::balance_of(request.contract.into(), request.account.into())
        .await
        .expect("erc20::balance_of failed")
        .into()
}

#[update]
async fn erc721_owner_of(request: Erc721OwnerOfRequest) -> Address {
    erc721::owner_of(request.contract.into(), request.token_id.into())
        .await
        .expect("erc721::owner_of failed")
        .into()
}

#[pre_upgrade]
async fn pre_upgrade() {
    debug!("Stopping client");

    let checkpoint = helios::get_last_checkpoint();
    mutate_state(|s|s.last_checkpoint = checkpoint);

    helios::shutdown().await;

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

            helios::start_client(
                network,
                &consensus_rpc_url,
                &execution_rpc_url,
                checkpoint,
            )
            .await
            .expect("starting client failed");
        });
    });
}
