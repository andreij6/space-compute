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
pub const OPEN_LEASES: u8 = 15;
pub const GOLD_SUBJECTS: u8 = 16;
pub const CLASSIFICATIONS: u8 = 20;
pub const SUBJECT_CLASSIFICATIONS: u8 = 21;
pub const CONSENSUS: u8 = 22;
pub const DISCOVERIES: u8 = 30;
pub const ASSIGNMENTS: u8 = 31;
pub const REVIEWS: u8 = 32;
pub const REVIEW_QUEUE: u8 = 33;
pub const ASSIGNED_SET: u8 = 34;
pub const AAA_CLASSIFICATIONS: u8 = 35;
pub const DISCOVERY_ASSIGNMENTS: u8 = 36;
pub const EVENTS_INDEX: u8 = 40;
pub const EVENTS_DATA: u8 = 41;
pub const CITATIONS: u8 = 42;
pub const PROGRESS: u8 = 43;
pub const LEADERBOARD: u8 = 44;
pub const CREDIT_INDEX: u8 = 45;
pub const PUBLIC_ID_INDEX: u8 = 46;
pub const AAA_ACTIVITY: u8 = 47;
pub const CLAIM_INDEX: u8 = 48;
pub const AAA_OPERATORS: u8 = 49;
pub const AAA_PROVENANCE: u8 = 50;
pub const CORROBORATIONS: u8 = 51;
pub const AUDIT_INDEX: u8 = 52;
pub const AUDIT_DATA: u8 = 53;
pub const AWAITING_REVIEWERS: u8 = 54;
pub const META: u8 = 55;

pub const BUCKET_PAGES: u16 = 16;

fn manager<M: ic_stable_structures::Memory>(memory: M) -> MemoryManager<M> {
    MemoryManager::init_with_bucket_size(memory, BUCKET_PAGES)
}

thread_local! {
    static MANAGER: RefCell<MemoryManager<DefaultMemoryImpl>> =
        RefCell::new(manager(DefaultMemoryImpl::default()));
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

#[cfg(test)]
mod tests {
    use super::*;
    use ic_stable_structures::{Memory as _, StableBTreeMap, VectorMemory};

    const WASM_PAGE: u64 = 65_536;

    #[test]
    fn t7_12_fresh_install_first_touch_costs_one_mib_bucket_per_memory() {
        let raw = VectorMemory::default();
        let mm = manager(raw.clone());
        let base = raw.size();
        for id in [
            LEASES,
            SEEN_SET,
            CLASSIFICATIONS,
            EVENTS_DATA,
            GOLD_SUBJECTS,
        ] {
            let mut m: StableBTreeMap<u32, u32, _> =
                StableBTreeMap::init(mm.get(MemoryId::new(id)));
            m.insert(1, 1);
        }
        let per_memory = (raw.size() - base) * WASM_PAGE / 5;
        assert_eq!(per_memory, BUCKET_PAGES as u64 * WASM_PAGE);
        assert!(per_memory <= 1 << 20);
    }

    #[test]
    fn t7_12_existing_8mib_bucket_layout_still_loads_after_bucket_change() {
        let raw = VectorMemory::default();
        {
            let old = MemoryManager::init(raw.clone());
            let mut m: StableBTreeMap<u32, u32, _> =
                StableBTreeMap::init(old.get(MemoryId::new(SUBJECTS)));
            m.insert(7, 42);
        }
        let reopened = manager(raw.clone());
        let mut m: StableBTreeMap<u32, u32, _> =
            StableBTreeMap::init(reopened.get(MemoryId::new(SUBJECTS)));
        assert_eq!(m.get(&7), Some(42));
        let before = raw.size();
        let mut other: StableBTreeMap<u32, u32, _> =
            StableBTreeMap::init(reopened.get(MemoryId::new(LEASES)));
        other.insert(1, 1);
        m.insert(8, 43);
        assert_eq!(raw.size() - before, 128);
    }
}
