use std::cell::RefCell;
use std::str::FromStr;

use candid::CandidType;
use ic_stable_structures::StableBTreeMap;
use ic_stable_structures::writer::Writer;
use serde::{Deserialize, Serialize};
use crate::stable_memory;

use crate::stable_memory::{init_block_hash_to_header_map, init_block_height_to_header_map, Memory};
use crate::storable_structures::HeaderInfo;

thread_local! {
    static STATE: RefCell<Option<LightClientState >> = RefCell::new(None);
}

impl LightClientState {
    pub fn init(args: InitArgs) -> anyhow::Result<Self> {
        let ret = LightClientState {
            consensus_rpc: "".to_string(),
            execution_rpc: "".to_string(),
            last_checkpoint: None,
            hash_to_headers: init_block_hash_to_header_map(),
            height_to_headers: init_block_height_to_header_map(),
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
    #[serde(skip, default = "crate::stable_memory::init_block_hash_to_header_map")]
    pub hash_to_headers: StableBTreeMap<String, HeaderInfo, Memory>,
    #[serde(skip, default = "crate::stable_memory::init_block_height_to_header_map")]
    pub height_to_headers: StableBTreeMap<u64, HeaderInfo, Memory>,
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
