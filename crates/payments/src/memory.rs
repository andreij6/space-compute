use std::cell::RefCell;

use ic_stable_structures::memory_manager::{MemoryId, MemoryManager, VirtualMemory};
use ic_stable_structures::DefaultMemoryImpl;

pub type Memory = VirtualMemory<DefaultMemoryImpl>;

pub const CONFIG: u8 = 0;
pub const OPS: u8 = 1;
#[allow(dead_code)]
pub const OWNER_AAA: u8 = 2;
#[allow(dead_code)]
pub const MANDATE: u8 = 3;
#[allow(dead_code)]
pub const AUTO_TOPUP_HISTORY: u8 = 4;
#[allow(dead_code)]
pub const STRIPE_DEDUPE: u8 = 6;
#[allow(dead_code)]
pub const BTC_ADDR_CACHE: u8 = 7;
pub const AUDIT_INDEX: u8 = 60;
pub const AUDIT_DATA: u8 = 61;

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
