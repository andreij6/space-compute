use std::cell::RefCell;

use candid::Principal;
use ic_stable_structures::StableBTreeMap;

use crate::memory::{self, Memory};

thread_local! {
    static OWNER_AAA: RefCell<StableBTreeMap<Principal, Principal, Memory>> =
        RefCell::new(StableBTreeMap::init(memory::get(memory::OWNER_AAA)));
}

pub fn record(owner: Principal, aaa: Principal) {
    OWNER_AAA.with_borrow_mut(|m| m.insert(owner, aaa));
}

pub fn get(owner: Principal) -> Option<Principal> {
    OWNER_AAA.with_borrow(|m| m.get(&owner))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn p(n: u8) -> Principal {
        Principal::from_slice(&[n; 29])
    }

    #[test]
    fn t5_3_owner_aaa_records_and_looks_up() {
        assert_eq!(get(p(1)), None);
        record(p(1), p(2));
        assert_eq!(get(p(1)), Some(p(2)));
        record(p(1), p(3));
        assert_eq!(get(p(1)), Some(p(3)));
    }
}
