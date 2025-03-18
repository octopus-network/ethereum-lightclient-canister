use candid::CandidType;
use ic_canisters_http_types::{HttpRequest, HttpResponse};
use ic_cdk::{init, post_upgrade, pre_upgrade, query, update};
use ic_cdk_timers::set_timer_interval;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::time::Duration;
use ic_canister_log::log;
use ic_cdk::api::management_canister::http_request;
use ic_cdk::api::management_canister::http_request::TransformArgs;

use tree_hash::fixed_bytes::B256;

use crate::config::networks::mainnet;
use crate::config::Config;
use crate::consensus::consensus::Inner;
use crate::ic_execution_rpc::IcExecutionRpc;
use crate::state::{mutate_state, read_state, replace_state, LightClientState, StateProfile};
use crate::state_profile::StateProfileView;
use crate::storable_structures::BlockInfo;
use crate::tasks::lightclient_task;
use helios_common::consensus_spec::MainnetConsensusSpec;
use crate::ic_log::INFO;

mod config;
mod consensus;
mod guard;
mod ic_consensus_rpc;
mod ic_execution_rpc;
pub mod ic_log;
mod stable_memory;
mod state;
mod state_profile;
mod storable_structures;
mod tasks;

#[init]
async fn init(args: InitArgs) {
    let mut config = Config::from(mainnet());
    config.execution_rpc = args.execution_rpc;
    let state = LightClientState::init(config);
    replace_state(state);
}

#[update]
pub async fn set_up(check_point: String) -> Result<(), String>{
    if read_state(|s|s.started) {
        return Err("set up can be called only once".to_string());
    }
    let store = read_state(|s| s.store.clone());
    let mut inner = Inner::<MainnetConsensusSpec>::new(store);
    let c =  B256::from_hex(check_point.as_str());
    let _ = inner.sync(c).await.unwrap();
    inner.store().await;
    mutate_state(|s| {
        s.store = inner.store;
        s.started = true;
    });
    set_timer_interval(Duration::from_secs(12), lightclient_task);
    Ok(())
}

#[query]
fn transform(raw: TransformArgs) -> http_request::HttpResponse {
    http_request::HttpResponse {
        status: raw.response.status.clone(),
        body: raw.response.body.clone(),
        headers: vec![],
    }
}

#[update]
fn set_execution_url(url: String) {
    mutate_state(|s|s.config.execution_rpc = url);
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
    let r = read_state(|s| StateProfile::from(s));
    let s = serde_json::to_string(&r).unwrap();
    serde_json::from_str(s.as_str()).unwrap()
}

#[pre_upgrade]
async fn pre_upgrade() {
    read_state(|s| s.pre_upgrade());
}

#[post_upgrade]
fn post_upgrade() {
    LightClientState::post_upgrade();
    set_timer_interval(Duration::from_secs(12), lightclient_task);
}

#[query]
pub  fn query_block(height: u64) -> BlockInfo {
    read_state(|s|s.blocks.get(&height).clone()).expect("none")
}

#[query]
pub fn hashes() -> BTreeMap<B256, u64> {
    let r = read_state(|s|s.hashes.clone());
    r
}

#[query]
pub fn query_finality() -> Option<BlockInfo>{
    log!(INFO, "query user");
    read_state(|s|s.finalized_block.clone())
}

#[query]
pub fn query_latest_block() -> Option<(u64, BlockInfo)>{
    read_state(|s|s.blocks.last_key_value())
}

#[query]
pub fn query_first_block() -> Option<(u64, BlockInfo)>{
    read_state(|s|s.blocks.first_key_value())
}

#[query]
pub fn verify(block_number: u64, except_root: String) -> Result<(), String> {
    let Some(f) = read_state(|s|s.finalized_block.clone()) else {
        return Err("Block not found".to_string());
    };
    let first = read_state(|s|s.blocks.first_key_value().unwrap_or_default().0);
    if f.block_number >= block_number{
        let Some(bl) = read_state(|s|s.blocks.get(&block_number)) else {
            return Err(format!("block is expired, the blocks from {} to {} are avaliable now", first, f.block_number));
        };

        let ex = B256::from_hex(except_root.as_str());
        if bl.receipt_root != ex {
            Err("check failed: lightclient check receipt root failed".to_string())
        }else {
            Ok(())
        }
    }else {
        Err(format!("block not finalized , the blocks from {} to {} are avaliable now, please try later!", first, f.block_number))
    }
}
ic_cdk::export_candid!();
