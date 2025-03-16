use std::cell::RefCell;
use std::collections::BTreeMap;

use eyre::eyre;
use ic_canister_log::log;
use ic_stable_structures::writer::Writer;
use ic_stable_structures::StableBTreeMap;
use serde::{Deserialize, Serialize};

use tree_hash::fixed_bytes::B256;

use crate::config::Config;
use crate::ic_execution_rpc::IcExecutionRpc;
use crate::ic_log::INFO;
use crate::stable_memory;
use crate::stable_memory::{init_block_height_to_header_map, Memory};
use crate::storable_structures::BlockInfo;
use helios_common::rpc_types::convert::hex_to_u64;
use helios_common::rpc_types::lightclient_store::LightClientStore;

thread_local! {
    static STATE: RefCell<Option<LightClientState >> = RefCell::new(None);
}

impl LightClientState {
    pub fn init(config: Config) -> Self {
        let ret = LightClientState {
            config,
            last_checkpoint: None,
            blocks: init_block_height_to_header_map(),
            store: Default::default(),
            hashes: Default::default(),
            finalized_block: None,
            history_length: 72000,
            is_timer_running: false,
        };
        ret
    }

    pub fn pre_upgrade(&self) {
        let mut state_bytes = vec![];
        let _ = ciborium::ser::into_writer(self, &mut state_bytes);
        let len = state_bytes.len() as u32;
        let mut memory = stable_memory::get_upgrade_stash_memory();
        let mut writer = Writer::new(&mut memory, 0);
        writer
            .write(&len.to_le_bytes())
            .expect("failed to save lightclient state len");
        writer
            .write(&state_bytes)
            .expect("failed to save lightclient state");
    }

    pub fn post_upgrade() {
        use ic_stable_structures::Memory;
        let memory = stable_memory::get_upgrade_stash_memory();
        // Read the length of the state bytes.
        let mut state_len_bytes = [0; 4];
        memory.read(0, &mut state_len_bytes);
        let state_len = u32::from_le_bytes(state_len_bytes) as usize;
        let mut state_bytes = vec![0; state_len];
        memory.read(4, &mut state_bytes);
        let state: LightClientState =
            ciborium::de::from_reader(&*state_bytes).expect("failed to decode state");
        replace_state(state);
    }
}

#[derive(Deserialize, Serialize)]
pub struct LightClientState {
    pub config: Config,
    pub last_checkpoint: Option<B256>,
    #[serde(
        skip,
        default = "crate::stable_memory::init_block_height_to_header_map"
    )]
    pub blocks: StableBTreeMap<u64, BlockInfo, Memory>,
    pub store: LightClientStore,
    pub hashes: BTreeMap<B256, u64>,
    pub finalized_block: Option<BlockInfo>,
    pub history_length: u64,
    #[serde(skip)]
    pub is_timer_running: bool,
    pub started: bool,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct StateProfile {
    pub config: Config,
    pub last_checkpoint: Option<B256>,
    pub store: LightClientStore,
    pub hashes: BTreeMap<B256, u64>,
    pub finalized_block: Option<BlockInfo>,
    pub history_length: u64,
}

impl From<&LightClientState> for StateProfile {
    fn from(value: &LightClientState) -> Self {
        Self {
            config: value.config.clone(),
            last_checkpoint: value.last_checkpoint.clone(),
            store: value.store.clone(),
            hashes: value.hashes.clone(),
            finalized_block: value.finalized_block.clone(),
            history_length: value.history_length.clone(),
        }
    }
}

pub fn mutate_state<F, R>(f: F) -> R
where
    F: FnOnce(&mut LightClientState) -> R,
{
    STATE.with(|s| f(s.borrow_mut().as_mut().expect("State not initialized!")))
}

pub fn read_state<F, R>(f: F) -> R
where
    F: FnOnce(&LightClientState) -> R,
{
    STATE.with(|s| f(s.borrow().as_ref().expect("State not initialized!")))
}

/// Replaces the current state.
pub fn replace_state(state: LightClientState) {
    STATE.with(|s| {
        *s.borrow_mut() = Some(state);
    });
}

pub fn take_state<F, R>(f: F) -> R
where
    F: FnOnce(LightClientState) -> R,
{
    STATE.with(|s| f(s.take().expect("State not initialized!")))
}

pub struct StateModifier;

impl StateModifier {
    pub async fn push_block(block: BlockInfo) {
        let block_number = block.block_number;
        if Self::try_insert_tip(block) {
            let mut n = block_number;

            loop {

                match  Self::backfill_behind(n).await {
                    Ok(backfilled) => {
                        if !backfilled {
                            break;
                        }
                        n -= 1;
                    }
                    Err(e) => {
                        log!(INFO, "fallback_error: {}, {}",n, e.to_string());
                        if e.to_string().contains("retry") {
                            continue;
                        }
                        Self::prune_before(n);
                        break;

                    }
                }
            }
            let (link_child, link_parent) =
                read_state(|s| (s.blocks.get(&n), s.blocks.get(&(n - 1))));
            if let (Some(parent), Some(child)) = (link_parent, link_child) {
                if child.parent_block_hash != parent.block_hash {
                    // warn!("detected block reorganization");
                    Self::prune_before(n);
                }
            }
            Self::prune();
        }
    }

    fn try_insert_tip(block: BlockInfo) -> bool {
        if let Some((num, _)) = read_state(|s| s.blocks.last_key_value().clone()) {
            if num > block.block_number {
                return false;
            }
        }
        mutate_state(|s| {
            s.hashes
                .insert(block.block_hash.clone(), block.block_number);
            s.blocks.insert(block.block_number, block);
        });
        true
    }

    fn prune() {
        mutate_state(|s| {
            while s.blocks.len() as u64 > s.history_length {
                if let Some((number, _)) = s.blocks.first_key_value() {
                    if let Some(block) = s.blocks.remove(&number) {
                        s.hashes.remove(&block.block_hash);
                    }
                }
            }
        });
    }

    fn prune_before(n: u64) {
        mutate_state(|s| loop {
            if let Some((oldest, _)) = s.blocks.first_key_value() {
                if oldest < n {
                    if let Some(block) = s.blocks.remove(&oldest) {
                        s.hashes.remove(&block.block_hash);
                    }
                } else {
                    break;
                }
            } else {
                break;
            }
        });
    }

    async fn backfill_behind(n: u64) -> eyre::Result<bool> {
        if read_state(|s| s.blocks.len() < 2) {
            return Ok(false);
        }
        match read_state(|s| s.blocks.get(&n)) {
            None => Ok(false),
            Some(block) => {
                let prev = n - 1;
                match read_state(|s| s.blocks.get(&prev).clone()) {
                    None => {
                        let execution_rpc = IcExecutionRpc::new(
                            read_state(|s| s.config.execution_rpc.clone()).as_str(),
                        )
                        .unwrap();
                        let parent_hash = block.parent_block_hash.clone();
                        let Ok(backfilled) = execution_rpc.get_block(parent_hash).await else {
                            return Err(eyre!("retry"));
                        };
                        if block.parent_block_hash == backfilled.hash {
                            let rroot = backfilled.receipts_root;
                            let pphash = backfilled.parent_hash;
                            let block_info = BlockInfo {
                                receipt_root: rroot,
                                parent_block_hash: pphash,
                                block_number: prev,
                                block_hash: block.parent_block_hash.clone(),
                            };
                            mutate_state(|s| {
                                s.blocks
                                    .insert(hex_to_u64(backfilled.number.as_str()), block_info);
                                log!(INFO, "backfill block {}", u64::from_str_radix(&backfilled.number[2..], 16).unwrap_or_default());
                            });
                            Ok(true)
                        } else {
                            Err(eyre!("bad backfill"))
                        }
                    }
                    Some(_) => Ok(false),
                }
            }
        }
    }

    pub async fn push_finalized_block(block: BlockInfo) {
        mutate_state(|s| {
            if let Some(old_block) = s.blocks.get(&block.block_number) {
                if old_block.block_hash != block.block_hash {
                    log!(INFO, "forked: {:?} {:?}", old_block.block_hash.clone(), old_block.block_hash.clone());
                    s.blocks.clear_new();
                }
            }
            s.finalized_block = Some(block.clone());
        });
    }

    pub fn update_last_checkpoint(new_checkpoint: Option<B256>) {
        if new_checkpoint.is_some() {
            mutate_state(|s| s.last_checkpoint = new_checkpoint);
        }
    }
}
