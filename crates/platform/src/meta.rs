use std::cell::RefCell;

use ic_stable_structures::StableBTreeMap;

use crate::memory::{self, Memory};

pub const REPLAY_NEXT: u8 = 0;
pub const REPLAY_BATCH: u8 = 1;
pub const REPLAY_CLEARING: u8 = 2;
pub const COUNTERS_READY: u8 = 10;
pub const AAA_STATUS_BASE: u8 = 20;
pub const RETIRED_SUBJECTS: u8 = 30;

thread_local! {
    static META: RefCell<StableBTreeMap<u8, u64, Memory>> =
        RefCell::new(StableBTreeMap::init(memory::get(memory::META)));
}

pub fn get(key: u8) -> Option<u64> {
    META.with_borrow(|m| m.get(&key))
}

pub fn set(key: u8, value: u64) {
    META.with_borrow_mut(|m| m.insert(key, value));
}

pub fn remove(key: u8) {
    META.with_borrow_mut(|m| m.remove(&key));
}

pub fn count(key: u8) -> u64 {
    get(key).unwrap_or(0)
}

pub fn bump(key: u8, up: bool) {
    let n = count(key);
    set(
        key,
        if up {
            n.saturating_add(1)
        } else {
            n.saturating_sub(1)
        },
    );
}
