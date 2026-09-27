use std::cell::RefCell;

use candid::{CandidType, Principal};
use ic_stable_structures::StableBTreeMap;
use sc_types::ApiError;
use serde::Deserialize;

use crate::memory::{self, Memory};

pub const MAX_OPERATORS: usize = 5;

#[derive(CandidType, Deserialize, Clone, Debug, PartialEq)]
pub struct Operator {
    pub v: u8,
    pub label: String,
    pub added_at: u64,
    pub expires_at: Option<u64>,
    pub last_used_at: Option<u64>,
}

crate::candid_storable!(Operator);

thread_local! {
    static MAP: RefCell<StableBTreeMap<Principal, Operator, Memory>> =
        RefCell::new(StableBTreeMap::init(memory::get(memory::OPERATORS)));
}

fn active(op: &Operator, now: u64) -> bool {
    match op.expires_at {
        Some(exp) => exp > now,
        None => true,
    }
}

pub fn is_active(p: &Principal, now: u64) -> bool {
    MAP.with_borrow(|m| m.get(p).is_some_and(|op| active(&op, now)))
}

pub fn get(p: &Principal) -> Option<Operator> {
    MAP.with_borrow(|m| m.get(p))
}

pub fn list() -> Vec<(Principal, Operator)> {
    MAP.with_borrow(|m| m.iter().map(|e| (*e.key(), e.value())).collect())
}

pub fn add(
    owner: Principal,
    p: Principal,
    label: String,
    expires_at: Option<u64>,
    now: u64,
) -> Result<(), ApiError> {
    if p == Principal::anonymous() {
        return Err(ApiError::invalid(
            "operator can't be the anonymous principal",
        ));
    }
    if p == owner {
        return Err(ApiError::invalid("the owner can't also be an operator"));
    }
    sc_types::limits::operator_label(&label)?;
    if let Some(exp) = expires_at {
        if exp <= now {
            return Err(ApiError::invalid("expires_at must be in the future"));
        }
    }
    MAP.with_borrow_mut(|m| {
        if m.get(&p).is_none() && m.len() as usize >= MAX_OPERATORS {
            return Err(ApiError::invalid(format!(
                "at most {MAX_OPERATORS} operators"
            )));
        }
        m.insert(
            p,
            Operator {
                v: 1,
                label,
                added_at: now,
                expires_at,
                last_used_at: None,
            },
        );
        Ok(())
    })
}

pub fn remove(p: Principal) -> Result<(), ApiError> {
    MAP.with_borrow_mut(|m| {
        if m.remove(&p).is_some() {
            Ok(())
        } else {
            Err(ApiError::NotFound)
        }
    })
}

pub fn touch(p: &Principal, now: u64) {
    MAP.with_borrow_mut(|m| {
        if let Some(mut op) = m.get(p) {
            op.last_used_at = Some(now);
            m.insert(*p, op);
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    fn p(n: u8) -> Principal {
        Principal::from_slice(&[n; 29])
    }

    fn reset() {
        for (k, _) in list() {
            remove(k).ok();
        }
    }

    #[test]
    fn t3_1_add_rejects_anonymous_owner_and_bad_labels() {
        reset();
        let owner = p(1);
        assert!(matches!(
            add(owner, Principal::anonymous(), "x".into(), None, 0),
            Err(ApiError::InvalidInput(_))
        ));
        assert!(matches!(
            add(owner, owner, "x".into(), None, 0),
            Err(ApiError::InvalidInput(_))
        ));
        let too_long = "x".repeat(33);
        assert!(matches!(
            add(owner, p(2), too_long, None, 0),
            Err(ApiError::InvalidInput(_))
        ));
        assert!(matches!(
            add(owner, p(2), "ok".into(), Some(5), 10),
            Err(ApiError::InvalidInput(_)),
        ));
    }

    #[test]
    fn t3_1_add_enforces_the_five_operator_cap() {
        reset();
        let owner = p(1);
        for i in 10..15u8 {
            add(owner, p(i), "op".into(), None, 0).unwrap();
        }
        assert!(matches!(
            add(owner, p(99), "op".into(), None, 0),
            Err(ApiError::InvalidInput(_))
        ));
        add(owner, p(10), "renamed".into(), None, 0).unwrap();
        remove(p(10)).unwrap();
        add(owner, p(20), "op".into(), None, 0).unwrap();
        assert_eq!(remove(p(99)), Err(ApiError::NotFound));
    }

    #[test]
    fn t3_1_expiry_makes_an_operator_inactive() {
        reset();
        let owner = p(1);
        let who = p(30);
        add(owner, who, "op".into(), Some(100), 0).unwrap();
        assert!(is_active(&who, 0));
        assert!(is_active(&who, 99));
        assert!(!is_active(&who, 100));
        assert!(!is_active(&who, 200));
        assert!(!is_active(&p(31), 0), "unknown principal is never active");
    }

    #[test]
    fn t3_1_touch_updates_last_used_at() {
        reset();
        let owner = p(1);
        let who = p(40);
        add(owner, who, "op".into(), None, 0).unwrap();
        touch(&who, 55);
        assert_eq!(get(&who).unwrap().last_used_at, Some(55));
        touch(&p(41), 55);
    }
}
