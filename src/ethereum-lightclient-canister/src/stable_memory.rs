use std::cell::RefCell;

use crate::storable_structures::BlockInfo;
use ic_stable_structures::memory_manager::{MemoryId, MemoryManager, VirtualMemory};
use ic_stable_structures::{DefaultMemoryImpl, StableBTreeMap};

pub type InnerMemory = DefaultMemoryImpl;
pub type Memory = VirtualMemory<InnerMemory>;
pub const UPGRADE_STASH_MEMORY_ID: MemoryId = MemoryId::new(0);
pub const BLOCK_HEIGHT_TO_HEADER_MEMORY_ID: MemoryId = MemoryId::new(1);
pub const BLOCK_HASH_TO_HEADER_MEMORY_ID: MemoryId = MemoryId::new(2);
pub const STABLE_LOG_MEMORY_ID: MemoryId = MemoryId::new(3);

thread_local! {
    static MEMORY: RefCell<Option<InnerMemory>> = RefCell::new(Some(InnerMemory::default()));

    static MEMORY_MANAGER: RefCell<Option<MemoryManager<InnerMemory>>> =
        RefCell::new(Some(MemoryManager::init(MEMORY.with(|m| m.borrow().clone().unwrap()))));

}

fn with_memory_manager<R>(f: impl FnOnce(&MemoryManager<InnerMemory>) -> R) -> R {
    MEMORY_MANAGER.with(|cell| {
        f(cell
            .borrow()
            .as_ref()
            .expect("memory manager not initialized"))
    })
}

pub fn get_upgrade_stash_memory() -> Memory {
    with_memory_manager(|m| m.get(UPGRADE_STASH_MEMORY_ID))
}

pub fn init_block_height_to_header_map() -> StableBTreeMap<u64, BlockInfo, Memory> {
    StableBTreeMap::init(with_memory_manager(|m| {
        m.get(BLOCK_HEIGHT_TO_HEADER_MEMORY_ID)
    }))
}

pub fn init_block_hash_to_header_map() -> StableBTreeMap<String, BlockInfo, Memory> {
    StableBTreeMap::init(with_memory_manager(|m| {
        m.get(BLOCK_HASH_TO_HEADER_MEMORY_ID)
    }))
}

pub fn init_stable_log() -> StableBTreeMap<Vec<u8>, Vec<u8>, Memory> {
    StableBTreeMap::init(with_memory_manager(|m| m.get(STABLE_LOG_MEMORY_ID)))
}
