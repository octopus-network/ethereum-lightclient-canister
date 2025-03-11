use std::collections::BTreeMap;
use std::time::Duration;
use candid::CandidType;
use ic_canisters_http_types::{HttpRequest, HttpResponse};
use ic_cdk::{init, post_upgrade, pre_upgrade, query, update};
use ic_cdk_timers::set_timer_interval;
use serde::{Deserialize, Serialize};

use tree_hash::fixed_bytes::B256;

use crate::config::Config;
use crate::config::networks::mainnet;
use crate::consensus::consensus::Inner;
use crate::consensus::consensus_spec::MainnetConsensusSpec;
use crate::ic_execution_rpc::IcExecutionRpc;
use crate::state::{LightClientState, mutate_state, read_state, replace_state, StateProfile};
use crate::state_profile::StateProfileView;
use crate::storable_structures::BlockInfo;
use crate::tasks::lightclient_task;

mod stable_memory;
mod state;
mod storable_structures;
mod ic_consensus_rpc;
mod consensus;
mod ic_execution_rpc;
mod rpc_types;
mod config;
pub mod ic_log;
mod state_profile;
mod guard;
mod tasks;


#[init]
async fn init(args: InitArgs) {
    let mut  config = Config::from(mainnet());
    config.execution_rpc = args.execution_rpc;
    let state = LightClientState::init(config);
    replace_state(state);
}

#[update]
pub async fn set_up() {
    let store = read_state(|s|s.store.clone());
    let mut inner = Inner::<MainnetConsensusSpec>::new(store);
    let c = read_state(|s|s.config.default_checkpoint);
    let _ = inner.sync(c).await.unwrap();
    inner.store().await;
    mutate_state(|s|s.store = inner.store);
    set_timer_interval(Duration::from_secs(12), lightclient_task);
}

#[query(hidden = true)]
fn http_request(req: HttpRequest) -> HttpResponse {
    if ic_cdk::api::data_certificate().is_none() {
        ic_cdk::trap("update call rejected");
    }
    ic_log::http_request(req)
}

#[derive(CandidType, Deserialize, Serialize)]
pub struct InitArgs {
    pub execution_rpc: String,
}

#[query]
pub fn state() -> StateProfileView {
    let r = read_state(|s|StateProfile::from(s));
    let s = serde_json::to_string(&r).unwrap();
    serde_json::from_str(s.as_str()).unwrap()
}

#[pre_upgrade]
async fn pre_upgrade() {
    read_state(|s|s.pre_upgrade());
}

#[post_upgrade]
fn post_upgrade() {
    LightClientState::post_upgrade();
    set_timer_interval(Duration::from_secs(12), lightclient_task);
}


#[update]
pub async fn query_block(block_hash: String) -> String {
    let rpc: IcExecutionRpc = IcExecutionRpc::new("https://mainnet.infura.io/v3/025779c23a2d44d1b1ecef2bfb4f2b29").unwrap();
    let b = B256::from_hex(block_hash.as_str());
    serde_json::to_string( &rpc.get_block(b).await.unwrap()).unwrap()
}

#[query]
pub async fn get_finality(height: u64) -> Option<BlockInfo> {
    read_state(|s|s.blocks.get(&height))
}

#[query]
pub async fn hashes() -> BTreeMap<B256, u64> {
    read_state(|s|s.hashes.clone())
}

ic_cdk::export_candid!();