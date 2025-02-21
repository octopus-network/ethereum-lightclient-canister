use std::cell::RefCell;
use std::str::FromStr;

use candid::CandidType;
use ic_stable_structures::{StableBTreeMap};
use ic_stable_structures::writer::Writer;
use serde::{Deserialize, Serialize};
use crate::stable_memory;
use std::collections::BTreeMap;
use eyre::eyre;
use crate::ic_execution_rpc::IcExecutionRpc;
use crate::rpc_types::convert::hex_to_u64;

use crate::stable_memory::{init_block_hash_to_header_map, init_block_height_to_header_map, Memory};
use crate::storable_structures::BlockInfo;

thread_local! {
    static STATE: RefCell<Option<LightClientState >> = RefCell::new(None);
}

impl LightClientState {
    pub fn init(args: InitArgs) -> anyhow::Result<Self> {
        let ret = LightClientState {
            consensus_rpc: "".to_string(),
            execution_rpc: "".to_string(),
            last_checkpoint: None,
            blocks: init_block_height_to_header_map(),
            hashes: Default::default(),
            finalized_block: None,
            history_length: 72000,
        };
        Ok(ret)
    }

    pub fn pre_upgrade(&self) {
        let mut state_bytes = vec![];
        let _ = ciborium::ser::into_writer(self, &mut state_bytes);
        let len = state_bytes.len() as u32;
        let mut memory = stable_memory::get_upgrade_stash_memory();
        let mut writer = Writer::new(&mut memory, 0);
        writer
            .write(&len.to_le_bytes())
            .expect("failed to save hub state len");
        writer
            .write(&state_bytes)
            .expect("failed to save hub state");
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
    pub consensus_rpc: String,
    pub execution_rpc: String,
    pub last_checkpoint: Option<String>,
    #[serde(skip, default = "crate::stable_memory::init_block_height_to_header_map")]
    pub blocks: StableBTreeMap<u64, BlockInfo, Memory>,
    pub hashes: BTreeMap<String, u64>,
    pub finalized_block: Option<BlockInfo>,
    pub history_length: u64,
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

#[derive(CandidType, Deserialize)]
pub struct InitArgs {

}


pub struct StateModifier;

impl StateModifier {
    pub async fn push_block(block: BlockInfo) {
        let block_number = block.block_number;
        if Self::try_insert_tip(block) {
            let mut n = block_number;

            loop {
                if let Ok(backfilled) = Self::backfill_behind(n).await {
                    if !backfilled {
                        break;
                    }
                    n -= 1;
                } else {
                    Self::prune_before(n);
                    break;
                }
            }
            let (link_child, link_parent) = read_state(|s|(s.blocks.get(&n), s.blocks.get(&(n-1))));
            if let (Some(parent), Some(child)) = (link_parent, link_child) {
                if child.block_hash != parent.block_hash {
                   // warn!("detected block reorganization");
                    Self::prune_before(n);
                }
            }
            Self::prune();
        }
    }

    fn try_insert_tip(block: BlockInfo) -> bool {
        if let Some((num, _)) = read_state(|s|s.blocks.last_key_value().clone()) {
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
        mutate_state(|s|{
            while s.blocks.len() as u64 > s.history_length {
                if let Some((number, _)) = s.blocks.first_key_value() {
                    if let Some(block) = s.blocks.remove(&number) {
                        s.hashes.remove(&block.block_hash);
                    }
                }
            }
        });
    }

    fn prune_before( n: u64) {
        mutate_state(|s|{
            loop {
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
            }
        });
    }

    async fn backfill_behind( n: u64) -> eyre::Result<bool> {
        if read_state(|s|s.blocks.len() < 2) {
            return Ok(false);
        }

        match read_state(|s|s.blocks.get(&n)) {
            None => {
                Ok(false)
            }
            Some(block) => {
                let prev = n - 1;
                match read_state(|s|s.blocks.get(&prev).clone()) {
                    None => {
                        let execution_rpc = IcExecutionRpc::new(read_state(|s|s.execution_rpc.clone()).as_str()).unwrap();
                        let parent_hash = block.parent_block_hash.clone();
                        let backfilled = execution_rpc.get_block(parent_hash).await?;
                        if block.parent_block_hash == backfilled.hash
                        {
                            let rroot = backfilled.receipts_root;
                            let pphash = backfilled.parent_hash;
                            let block_info = BlockInfo {
                                receipt_root: rroot,
                                parent_block_hash: pphash,
                                block_number: prev,
                                block_hash: block.parent_block_hash.clone(),
                            };
                            mutate_state(|s|{
                                s.blocks.insert(hex_to_u64(backfilled.number.as_str()), block_info);
                            });
                            Ok(true)
                        } else {
                            Err(eyre!("bad backfill"))
                        }
                    }
                    Some(_) => {
                        Ok(false)
                    }
                }
            }
        }
    }

    pub async fn push_finalized_block(block: BlockInfo) {
        mutate_state(|s|{
            if let Some(old_block) = s.blocks.get(&block.block_number) {
                if old_block.block_hash != block.block_hash {
                    s.blocks.clear_new();
                }
            }
            s.finalized_block = Some(block.clone());
        });
        Self::push_block(block).await;
    }

    pub fn update_last_checkpoint(new_checkpoint: String) {
        mutate_state(|s|s.last_checkpoint = Some(new_checkpoint));
    }
}

