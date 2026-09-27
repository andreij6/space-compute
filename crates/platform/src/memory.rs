use std::cell::RefCell;

use ic_stable_structures::memory_manager::{MemoryId, MemoryManager, VirtualMemory};
use ic_stable_structures::DefaultMemoryImpl;

pub type Memory = VirtualMemory<DefaultMemoryImpl>;

pub const CONFIG: u8 = 0;
pub const WASM_STORE: u8 = 1;
pub const WASM_META: u8 = 2;
pub const AAA_REGISTRY: u8 = 5;
pub const AAA_OWNERS: u8 = 6;
pub const AAA_NAMES: u8 = 7;
pub const SUBJECTS: u8 = 10;
pub const PROTOCOLS: u8 = 11;
pub const LEASES: u8 = 12;
pub const SEEN_SET: u8 = 13;
pub const TASK_POOL: u8 = 14;
pub const AAA_OPERATORS: u8 = 49;
pub const AAA_PROVENANCE: u8 = 50;
pub const AUDIT_INDEX: u8 = 52;
pub const AUDIT_DATA: u8 = 53;

thread_local! {
    static MANAGER: RefCell<MemoryManager<DefaultMemoryImpl>> =
        RefCell::new(MemoryManager::init(DefaultMemoryImpl::default()));
}

pub fn get(id: u8) -> Memory {
    MANAGER.with_borrow(|m| m.get(MemoryId::new(id)))
}

#[macro_export]
macro_rules! candid_storable {
    ($t:ty) => {
        impl ic_stable_structures::Storable for $t {
            const BOUND: ic_stable_structures::storable::Bound =
                ic_stable_structures::storable::Bound::Unbounded;

            fn to_bytes(&self) -> std::borrow::Cow<'_, [u8]> {
                std::borrow::Cow::Owned(candid::encode_one(self).expect("candid encode"))
            }

            fn into_bytes(self) -> Vec<u8> {
                candid::encode_one(&self).expect("candid encode")
            }

            fn from_bytes(bytes: std::borrow::Cow<[u8]>) -> Self {
                candid::decode_one(&bytes).expect("candid decode")
            }
        }
    };
}
