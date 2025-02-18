use std::cell::RefCell;

use ic_stable_structures::{DefaultMemoryImpl, StableBTreeMap};
use ic_stable_structures::memory_manager::{MemoryId, MemoryManager, VirtualMemory};
use crate::storable_structures::BlockInfo;


pub type InnerMemory = DefaultMemoryImpl;
pub type Memory = VirtualMemory<InnerMemory>;
pub const UPGRADE_STASH_MEMORY_ID: MemoryId = MemoryId::new(0);
pub const BLOCK_HEIGHT_TO_HEADER_MEMORY_ID: MemoryId = MemoryId::new(1);
pub const BLOCK_HASH_TO_HEADER_MEMORY_ID: MemoryId = MemoryId::new(2);

pub const PENDING_TICKET_MAP_MEMORY_ID: MemoryId = MemoryId::new(3);
pub const PENDING_DIRECTIVE_MAP_MEMORY_ID: MemoryId = MemoryId::new(4);
pub const STABLE_LOG_MEMORY_ID: MemoryId = MemoryId::new(5);

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

pub fn get_pending_ticket_map_memory() -> Memory {
    with_memory_manager(|m| m.get(PENDING_TICKET_MAP_MEMORY_ID))
}

pub fn get_pending_directive_map_memory() -> Memory {
    with_memory_manager(|m| m.get(PENDING_DIRECTIVE_MAP_MEMORY_ID))
}

pub fn get_upgrade_stash_memory() -> Memory {
    with_memory_manager(|m| m.get(UPGRADE_STASH_MEMORY_ID))
}

pub fn init_block_height_to_header_map() -> StableBTreeMap<u64, BlockInfo, Memory> {
    StableBTreeMap::init( with_memory_manager(|m| m.get(BLOCK_HEIGHT_TO_HEADER_MEMORY_ID)))
}

pub fn init_block_hash_to_header_map() -> StableBTreeMap<String, BlockInfo, Memory> {
    StableBTreeMap::init( with_memory_manager(|m| m.get(BLOCK_HASH_TO_HEADER_MEMORY_ID)))
}
