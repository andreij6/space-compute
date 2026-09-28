use std::cell::RefCell;

use candid::CandidType;
use ic_stable_structures::StableCell;
use serde::{Deserialize, Serialize};

use crate::memory::{self, Memory};

#[derive(CandidType, Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct CachedParams {
    pub fee_get_task: u128,
    pub fee_submit_classification: u128,
    pub fee_get_review: u128,
    pub fee_submit_review: u128,
    pub freezing_reserve: u128,
    pub last_refreshed_at: u64,
}

impl Default for CachedParams {
    fn default() -> Self {
        Self {
            fee_get_task: 60_000_000,
            fee_submit_classification: 50_000_000,
            fee_get_review: 20_000_000,
            fee_submit_review: 50_000_000,
            freezing_reserve: 10_000_000_000,
            last_refreshed_at: 0,
        }
    }
}

impl CachedParams {
    pub fn max_fee(&self) -> u128 {
        self.fee_get_task
            .max(self.fee_submit_classification)
            .max(self.fee_get_review)
            .max(self.fee_submit_review)
    }

    pub fn low_cycles_threshold(&self) -> u128 {
        self.freezing_reserve.saturating_add(50 * self.max_fee())
    }
}

crate::candid_storable!(CachedParams);

thread_local! {
    static CELL: RefCell<StableCell<CachedParams, Memory>> =
        RefCell::new(StableCell::init(memory::get(memory::PARAMS), CachedParams::default()));
}

pub fn get() -> CachedParams {
    CELL.with_borrow(|c| c.get().clone())
}

pub fn set(params: CachedParams) {
    CELL.with_borrow_mut(|c| {
        c.set(params);
    });
}

pub fn update_fee_get_task(fee: u128) {
    CELL.with_borrow_mut(|c| {
        let mut p = c.get().clone();
        p.fee_get_task = fee;
        c.set(p);
    });
}

pub fn update_fee_submit_classification(fee: u128) {
    CELL.with_borrow_mut(|c| {
        let mut p = c.get().clone();
        p.fee_submit_classification = fee;
        c.set(p);
    });
}

pub fn update_fee_get_review(fee: u128) {
    CELL.with_borrow_mut(|c| {
        let mut p = c.get().clone();
        p.fee_get_review = fee;
        c.set(p);
    });
}

pub fn update_fee_submit_review(fee: u128) {
    CELL.with_borrow_mut(|c| {
        let mut p = c.get().clone();
        p.fee_submit_review = fee;
        c.set(p);
    });
}

#[derive(CandidType, Deserialize, Clone, Debug, PartialEq)]
pub struct PlatformParams {
    pub fee_get_task: u128,
    pub fee_submit_classification: u128,
    pub fee_get_review: u128,
    pub fee_submit_review: u128,
}

pub fn apply_platform_params(new: PlatformParams, now: u64) {
    CELL.with_borrow_mut(|c| {
        let mut p = c.get().clone();
        p.fee_get_task = new.fee_get_task;
        p.fee_submit_classification = new.fee_submit_classification;
        p.fee_get_review = new.fee_get_review;
        p.fee_submit_review = new.fee_submit_review;
        p.last_refreshed_at = now;
        c.set(p);
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn t3_2_params_defaults_and_threshold() {
        let p = CachedParams::default();
        assert_eq!(p.fee_get_task, 60_000_000);
        assert_eq!(p.fee_submit_classification, 50_000_000);
        assert_eq!(p.fee_get_review, 20_000_000);
        assert_eq!(p.fee_submit_review, 50_000_000);
        assert_eq!(p.max_fee(), 60_000_000);
        assert_eq!(p.low_cycles_threshold(), 10_000_000_000 + 50 * 60_000_000);
    }

    #[test]
    fn t3_2_params_updates() {
        update_fee_submit_classification(250_000_000);
        assert_eq!(get().fee_submit_classification, 250_000_000);
        update_fee_get_task(60_000_000);
        assert_eq!(get().fee_get_task, 60_000_000);
        update_fee_get_review(70_000_000);
        assert_eq!(get().fee_get_review, 70_000_000);
        update_fee_submit_review(220_000_000);
        assert_eq!(get().fee_submit_review, 220_000_000);
    }

    #[test]
    fn t3_4_apply_platform_params_updates_fees_and_refresh_time() {
        apply_platform_params(
            PlatformParams {
                fee_get_task: 111,
                fee_submit_classification: 222,
                fee_get_review: 333,
                fee_submit_review: 444,
            },
            999,
        );
        let p = get();
        assert_eq!(p.fee_get_task, 111);
        assert_eq!(p.fee_submit_classification, 222);
        assert_eq!(p.fee_get_review, 333);
        assert_eq!(p.fee_submit_review, 444);
        assert_eq!(p.last_refreshed_at, 999);
    }
}
