use std::cell::RefCell;

use candid::{CandidType, Principal};
use ic_stable_structures::StableBTreeMap;
use sc_types::ApiError;
use serde::Deserialize;

use crate::journal::Account;
use crate::memory::{self, Memory};

pub const ROLLING_WINDOW_SECS: u64 = 30 * 86_400;
const ROLLING_WINDOW_NANOS: u64 = ROLLING_WINDOW_SECS * 1_000_000_000;

#[derive(CandidType, Deserialize, Clone, Debug, PartialEq)]
pub struct Mandate {
    pub aaa: Principal,
    pub payer: Account,
    pub topup_e8s: u64,
    pub cap_30d_e8s: u64,
    pub enabled: bool,
    pub needs_attention: bool,
    pub last_auto_at: u64,
}

crate::candid_storable!(Mandate);

#[derive(CandidType, Deserialize, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct HistoryKey {
    pub aaa: Principal,
    pub op_id: u64,
}

crate::candid_storable!(HistoryKey);

#[derive(CandidType, Deserialize, Clone, Debug, PartialEq)]
pub struct HistoryEntry {
    pub at: u64,
    pub e8s: u64,
}

crate::candid_storable!(HistoryEntry);

#[derive(CandidType, Deserialize, Clone, Debug, PartialEq)]
pub struct MandateView {
    pub aaa: Principal,
    pub payer: Account,
    pub topup_e8s: u64,
    pub cap_30d_e8s: u64,
    pub enabled: bool,
    pub needs_attention: bool,
    pub last_auto_at: u64,
    pub spent_30d_e8s: u64,
    pub remaining_30d_e8s: u64,
}

thread_local! {
    static MANDATES: RefCell<StableBTreeMap<Principal, Mandate, Memory>> =
        RefCell::new(StableBTreeMap::init(memory::get(memory::MANDATE)));
    static HISTORY: RefCell<StableBTreeMap<HistoryKey, HistoryEntry, Memory>> =
        RefCell::new(StableBTreeMap::init(memory::get(memory::AUTO_TOPUP_HISTORY)));
}

pub fn set(
    aaa: Principal,
    payer: Account,
    topup_e8s: u64,
    cap_30d_e8s: u64,
    enabled: bool,
) -> Result<Mandate, ApiError> {
    if topup_e8s == 0 {
        return Err(ApiError::invalid("topup_e8s must be greater than zero"));
    }
    if cap_30d_e8s < topup_e8s {
        return Err(ApiError::invalid("cap_30d_e8s must be at least topup_e8s"));
    }
    let last_auto_at = MANDATES.with_borrow(|m| m.get(&aaa).map(|m| m.last_auto_at).unwrap_or(0));
    let mandate = Mandate {
        aaa,
        payer,
        topup_e8s,
        cap_30d_e8s,
        enabled,
        needs_attention: false,
        last_auto_at,
    };
    MANDATES.with_borrow_mut(|m| m.insert(aaa, mandate.clone()));
    Ok(mandate)
}

pub fn get(aaa: Principal) -> Option<Mandate> {
    MANDATES.with_borrow(|m| m.get(&aaa))
}

pub fn spent_last_30d(aaa: Principal, now: u64) -> u64 {
    let since = now.saturating_sub(ROLLING_WINDOW_NANOS);
    let start = HistoryKey { aaa, op_id: 0 };
    let end = HistoryKey {
        aaa,
        op_id: u64::MAX,
    };
    HISTORY.with_borrow(|m| {
        m.range(start..=end)
            .filter(|e| e.value().at >= since)
            .fold(0u64, |acc, e| acc.saturating_add(e.value().e8s))
    })
}

pub fn record_spend(aaa: Principal, op_id: u64, at: u64, e8s: u64) {
    HISTORY.with_borrow_mut(|m| m.insert(HistoryKey { aaa, op_id }, HistoryEntry { at, e8s }));
}

pub fn view(aaa: Principal, now: u64) -> Option<MandateView> {
    let m = get(aaa)?;
    let spent = spent_last_30d(aaa, now);
    Some(MandateView {
        aaa: m.aaa,
        payer: m.payer,
        topup_e8s: m.topup_e8s,
        cap_30d_e8s: m.cap_30d_e8s,
        enabled: m.enabled,
        needs_attention: m.needs_attention,
        last_auto_at: m.last_auto_at,
        spent_30d_e8s: spent,
        remaining_30d_e8s: m.cap_30d_e8s.saturating_sub(spent),
    })
}

pub fn mark_needs_attention(aaa: Principal, flag: bool) -> Result<(), ApiError> {
    MANDATES.with_borrow_mut(|m| {
        let mut mandate = m.get(&aaa).ok_or(ApiError::NotFound)?;
        mandate.needs_attention = flag;
        m.insert(aaa, mandate);
        Ok(())
    })
}

pub fn set_last_auto_at(aaa: Principal, at: u64) -> Result<(), ApiError> {
    MANDATES.with_borrow_mut(|m| {
        let mut mandate = m.get(&aaa).ok_or(ApiError::NotFound)?;
        mandate.last_auto_at = at;
        m.insert(aaa, mandate);
        Ok(())
    })
}

pub fn check_eligible(m: &Mandate, now: u64, min_interval_secs: u64) -> Result<(), ApiError> {
    if !m.enabled {
        return Err(ApiError::invalid("mandate is disabled"));
    }
    let min_interval_nanos = min_interval_secs.saturating_mul(1_000_000_000);
    if m.last_auto_at != 0 && now.saturating_sub(m.last_auto_at) < min_interval_nanos {
        return Err(ApiError::invalid(
            "auto_topup_min_interval_secs has not elapsed",
        ));
    }
    let spent = spent_last_30d(m.aaa, now);
    if spent.saturating_add(m.topup_e8s) > m.cap_30d_e8s {
        return Err(ApiError::invalid(
            "auto top-up would exceed the rolling 30-day cap",
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn p(n: u8) -> Principal {
        Principal::from_slice(&[n; 29])
    }

    fn account(n: u8) -> Account {
        Account {
            owner: p(n),
            subaccount: None,
        }
    }

    const DAY_NANOS: u64 = 86_400 * 1_000_000_000;

    #[test]
    fn t5_5_set_mandate_rejects_zero_topup_and_cap_below_topup() {
        assert!(matches!(
            set(p(1), account(2), 0, 1_000, true),
            Err(ApiError::InvalidInput(_))
        ));
        assert!(matches!(
            set(p(1), account(2), 1_000, 500, true),
            Err(ApiError::InvalidInput(_))
        ));
    }

    #[test]
    fn t5_5_set_mandate_preserves_last_auto_at_across_updates() {
        let aaa = p(3);
        set(aaa, account(4), 1_000, 10_000, true).unwrap();
        set_last_auto_at(aaa, 555).unwrap();
        let m = set(aaa, account(4), 2_000, 20_000, true).unwrap();
        assert_eq!(m.last_auto_at, 555);
    }

    #[test]
    fn t5_5_rolling_window_sums_only_entries_within_30_days() {
        let aaa = p(5);
        let now = 40 * DAY_NANOS;
        record_spend(aaa, 1, now - 40 * DAY_NANOS, 1_000_000);
        record_spend(aaa, 2, now - 29 * DAY_NANOS, 2_000_000);
        record_spend(aaa, 3, now - DAY_NANOS, 3_000_000);
        assert_eq!(spent_last_30d(aaa, now), 5_000_000);
    }

    #[test]
    fn t5_5_rolling_window_is_per_aaa() {
        let a = p(6);
        let b = p(7);
        let now = 10 * DAY_NANOS;
        record_spend(a, 1, now, 1_000);
        record_spend(b, 1, now, 9_000);
        assert_eq!(spent_last_30d(a, now), 1_000);
        assert_eq!(spent_last_30d(b, now), 9_000);
    }

    #[test]
    fn t5_5_check_eligible_rejects_disabled_mandate() {
        let m = Mandate {
            aaa: p(8),
            payer: account(9),
            topup_e8s: 1_000,
            cap_30d_e8s: 10_000,
            enabled: false,
            needs_attention: false,
            last_auto_at: 0,
        };
        assert!(matches!(
            check_eligible(&m, 100, 60),
            Err(ApiError::InvalidInput(_))
        ));
    }

    #[test]
    fn t5_5_check_eligible_enforces_min_interval() {
        let aaa = p(10);
        let m = Mandate {
            aaa,
            payer: account(11),
            topup_e8s: 1_000,
            cap_30d_e8s: 10_000,
            enabled: true,
            needs_attention: false,
            last_auto_at: 1_000_000_000,
        };
        let min_interval_secs = 3_600;
        let too_soon = m.last_auto_at + 1_000_000_000;
        assert!(check_eligible(&m, too_soon, min_interval_secs).is_err());
        let late_enough = m.last_auto_at + min_interval_secs * 1_000_000_000;
        assert!(check_eligible(&m, late_enough, min_interval_secs).is_ok());
    }

    #[test]
    fn t5_5_check_eligible_allows_first_ever_auto_topup_regardless_of_interval() {
        let m = Mandate {
            aaa: p(12),
            payer: account(13),
            topup_e8s: 1_000,
            cap_30d_e8s: 10_000,
            enabled: true,
            needs_attention: false,
            last_auto_at: 0,
        };
        assert!(check_eligible(&m, 1, 1_000_000).is_ok());
    }

    #[test]
    fn t5_5_check_eligible_enforces_rolling_cap() {
        let aaa = p(14);
        let now = 5 * DAY_NANOS;
        record_spend(aaa, 1, now, 9_500);
        let m = Mandate {
            aaa,
            payer: account(15),
            topup_e8s: 1_000,
            cap_30d_e8s: 10_000,
            enabled: true,
            needs_attention: false,
            last_auto_at: 0,
        };
        assert!(matches!(
            check_eligible(&m, now, 0),
            Err(ApiError::InvalidInput(_))
        ));
        let m2 = Mandate {
            topup_e8s: 500,
            ..m
        };
        assert!(check_eligible(&m2, now, 0).is_ok());
    }

    #[test]
    fn t5_5_view_reports_spent_and_remaining_allowance() {
        let aaa = p(16);
        set(aaa, account(17), 1_000, 10_000, true).unwrap();
        let now = DAY_NANOS;
        record_spend(aaa, 1, now, 4_000);
        let v = view(aaa, now).unwrap();
        assert_eq!(v.spent_30d_e8s, 4_000);
        assert_eq!(v.remaining_30d_e8s, 6_000);
    }

    #[test]
    fn t5_5_view_is_none_for_unknown_aaa() {
        assert_eq!(view(p(200), 0), None);
    }

    #[test]
    fn t5_5_mark_needs_attention_round_trips() {
        let aaa = p(18);
        set(aaa, account(19), 1_000, 10_000, true).unwrap();
        mark_needs_attention(aaa, true).unwrap();
        assert!(get(aaa).unwrap().needs_attention);
        mark_needs_attention(aaa, false).unwrap();
        assert!(!get(aaa).unwrap().needs_attention);
    }

    #[test]
    fn t5_5_mark_needs_attention_unknown_aaa_not_found() {
        assert_eq!(mark_needs_attention(p(201), true), Err(ApiError::NotFound));
    }
}
