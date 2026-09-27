use std::cell::RefCell;

use candid::{CandidType, Principal};
use ic_cdk::call::Call;
use ic_stable_structures::StableCell;
use sc_types::ApiError;
use serde::Deserialize;

use crate::memory::{self, Memory};

pub fn cmc_id() -> Principal {
    Principal::from_text("rkp4c-7iaaa-aaaaa-aaaca-cai").expect("valid CMC principal")
}

pub fn ledger_id() -> Principal {
    Principal::from_text("ryjl3-tyaaa-aaaaa-aaaba-cai").expect("valid ledger principal")
}

#[derive(CandidType, Deserialize, Clone, Debug, PartialEq)]
pub struct RateCache {
    pub v: u8,
    pub xdr_permyriad_per_icp: u64,
    pub fetched_at_secs: u64,
}

impl Default for RateCache {
    fn default() -> Self {
        RateCache {
            v: 1,
            xdr_permyriad_per_icp: 0,
            fetched_at_secs: 0,
        }
    }
}

impl RateCache {
    pub fn age_secs(&self, now_secs: u64) -> u64 {
        now_secs.saturating_sub(self.fetched_at_secs)
    }

    pub fn is_stale(&self, now_secs: u64, max_age_secs: u64) -> bool {
        self.fetched_at_secs == 0 || self.age_secs(now_secs) > max_age_secs
    }
}

crate::candid_storable!(RateCache);

thread_local! {
    static CELL: RefCell<StableCell<RateCache, Memory>> =
        RefCell::new(StableCell::init(memory::get(memory::XDR_RATE), RateCache::default()));
}

pub fn get() -> RateCache {
    CELL.with_borrow(|c| c.get().clone())
}

fn set(rate: RateCache) {
    CELL.with_borrow_mut(|c| {
        c.set(rate);
    });
}

#[derive(CandidType, Deserialize, Clone, Debug)]
struct IcpXdrConversionRate {
    timestamp_seconds: u64,
    xdr_permyriad_per_icp: u64,
}

#[derive(CandidType, Deserialize, Clone, Debug)]
struct IcpXdrConversionRateResponse {
    data: IcpXdrConversionRate,
    #[allow(dead_code)]
    hash_tree: Vec<u8>,
    #[allow(dead_code)]
    certificate: Vec<u8>,
}

pub async fn refresh() -> Result<RateCache, ApiError> {
    let reply = Call::bounded_wait(cmc_id(), "get_icp_xdr_conversion_rate")
        .with_args(&())
        .await
        .map_err(|e| ApiError::Internal(format!("get_icp_xdr_conversion_rate: {e:?}")))?;
    let resp: IcpXdrConversionRateResponse = reply
        .candid()
        .map_err(|e| ApiError::Internal(format!("xdr rate decode: {e:?}")))?;
    let cache = RateCache {
        v: 1,
        xdr_permyriad_per_icp: resp.data.xdr_permyriad_per_icp,
        fetched_at_secs: resp.data.timestamp_seconds,
    };
    set(cache.clone());
    Ok(cache)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn t5_2_default_rate_is_zero_and_always_stale() {
        let r = RateCache::default();
        assert_eq!(r.xdr_permyriad_per_icp, 0);
        assert!(r.is_stale(1_000, 7_200));
    }

    #[test]
    fn t5_2_rate_staleness_uses_max_age() {
        let r = RateCache {
            v: 1,
            xdr_permyriad_per_icp: 37_300,
            fetched_at_secs: 1_000,
        };
        assert!(!r.is_stale(1_000 + 7_200, 7_200));
        assert!(r.is_stale(1_000 + 7_201, 7_200));
        assert_eq!(r.age_secs(1_500), 500);
    }

    #[test]
    fn t5_2_get_set_round_trips() {
        let r = RateCache {
            v: 1,
            xdr_permyriad_per_icp: 40_000,
            fetched_at_secs: 42,
        };
        set(r.clone());
        assert_eq!(get(), r);
    }

    #[test]
    fn t5_2_known_canister_ids_resolve() {
        assert_eq!(cmc_id().to_text(), "rkp4c-7iaaa-aaaaa-aaaca-cai");
        assert_eq!(ledger_id().to_text(), "ryjl3-tyaaa-aaaaa-aaaba-cai");
    }
}
