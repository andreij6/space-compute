use std::cell::RefCell;

use candid::{CandidType, Principal};
use ic_stable_structures::StableCell;
use sc_types::ApiError;
use serde::Deserialize;

use crate::memory::{self, Memory};

pub const MAX_ADMINS: usize = 20;
const DAY_SECS: u64 = 86_400;
const T: u128 = 1_000_000_000_000;

#[derive(CandidType, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct Config {
    pub v: u8,
    pub admins: Vec<Principal>,
    pub reserve_e8s: u64,
    pub default_target_days: u32,
    pub min_topup_cycles: u128,
    pub min_balance_cycles: u128,
    pub runway_alert_days: u32,
    pub check_interval_secs: u64,
    pub withdraw_approval_secs: u64,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            v: 1,
            admins: vec![],
            reserve_e8s: 5 * 100_000_000,
            default_target_days: 60,
            min_topup_cycles: T / 2,
            min_balance_cycles: 2 * T,
            runway_alert_days: 21,
            check_interval_secs: 6 * 3_600,
            withdraw_approval_secs: DAY_SECS,
        }
    }
}

impl Config {
    pub fn validate(&self) -> Result<(), ApiError> {
        if self.admins.is_empty() || self.admins.len() > MAX_ADMINS {
            return Err(ApiError::invalid(format!(
                "admins must be 1..={MAX_ADMINS}"
            )));
        }
        if self.admins.contains(&Principal::anonymous()) {
            return Err(ApiError::invalid("anonymous can't be an admin"));
        }
        if !(1..=3_650).contains(&self.default_target_days) {
            return Err(ApiError::invalid("default_target_days must be in 1..=3650"));
        }
        if !(1..=3_650).contains(&self.runway_alert_days) {
            return Err(ApiError::invalid("runway_alert_days must be in 1..=3650"));
        }
        if !(600..=7 * DAY_SECS).contains(&self.check_interval_secs) {
            return Err(ApiError::invalid(
                "check_interval_secs must be in 600..=604800",
            ));
        }
        if !(3_600..=DAY_SECS).contains(&self.withdraw_approval_secs) {
            return Err(ApiError::invalid(
                "withdraw_approval_secs must be in 3600..=86400",
            ));
        }
        if self.min_topup_cycles == 0 || self.min_topup_cycles > 1_000 * T {
            return Err(ApiError::invalid("min_topup_cycles must be in 1..=1000T"));
        }
        if self.min_balance_cycles > 1_000 * T {
            return Err(ApiError::invalid("min_balance_cycles must be ≤ 1000T"));
        }
        Ok(())
    }
}

pub fn needs_second_admin(old: &Config, new: &Config) -> bool {
    let lowers_reserve = new.reserve_e8s < old.reserve_e8s;
    let removes_admin = old.admins.iter().any(|a| !new.admins.contains(a));
    let adds_admin = new.admins.iter().any(|a| !old.admins.contains(a));
    lowers_reserve || removes_admin || (adds_admin && old.admins.len() > 1)
}

crate::candid_storable!(Config);

thread_local! {
    static CELL: RefCell<StableCell<Config, Memory>> =
        RefCell::new(StableCell::init(memory::get(memory::CONFIG), Config::default()));
}

pub fn get() -> Config {
    CELL.with_borrow(|c| c.get().clone())
}

pub fn set(config: Config) -> Result<(), ApiError> {
    config.validate()?;
    CELL.with_borrow_mut(|c| c.set(config));
    Ok(())
}

pub fn is_admin(p: &Principal) -> bool {
    *p != Principal::anonymous() && CELL.with_borrow(|c| c.get().admins.contains(p))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn admin() -> Principal {
        Principal::from_slice(&[1; 29])
    }

    #[test]
    fn t5_17_config_defaults_validate_and_round_trip() {
        let c = Config {
            admins: vec![admin()],
            ..Config::default()
        };
        c.validate().unwrap();
        assert_eq!(c.default_target_days, 60);
        assert_eq!(c.min_topup_cycles, T / 2);
        set(c.clone()).unwrap();
        assert_eq!(get(), c);
        assert!(is_admin(&admin()));
        assert!(!is_admin(&Principal::anonymous()));
    }

    #[test]
    fn t5_17_config_rejects_bad_values() {
        let ok = Config {
            admins: vec![admin()],
            ..Config::default()
        };
        let bad = [
            Config {
                admins: vec![],
                ..ok.clone()
            },
            Config {
                admins: vec![Principal::anonymous()],
                ..ok.clone()
            },
            Config {
                default_target_days: 0,
                ..ok.clone()
            },
            Config {
                runway_alert_days: 0,
                ..ok.clone()
            },
            Config {
                check_interval_secs: 1,
                ..ok.clone()
            },
            Config {
                withdraw_approval_secs: 1,
                ..ok.clone()
            },
            Config {
                min_topup_cycles: 0,
                ..ok.clone()
            },
            Config {
                min_balance_cycles: u128::MAX,
                ..ok.clone()
            },
        ];
        for c in bad {
            assert!(
                matches!(set(c.clone()), Err(ApiError::InvalidInput(_))),
                "{c:?}"
            );
        }
    }

    #[test]
    fn t5_17_sensitive_config_changes_need_a_second_admin() {
        let b = Principal::from_slice(&[2; 29]);
        let solo = Config {
            admins: vec![admin()],
            ..Config::default()
        };
        let pair = Config {
            admins: vec![admin(), b],
            ..solo.clone()
        };
        assert!(!needs_second_admin(&solo, &pair));
        assert!(needs_second_admin(&pair, &solo));
        let three = Config {
            admins: vec![admin(), b, Principal::from_slice(&[3; 29])],
            ..solo.clone()
        };
        assert!(needs_second_admin(&pair, &three));
        let lower = Config {
            reserve_e8s: 1,
            ..pair.clone()
        };
        assert!(needs_second_admin(&pair, &lower));
        let higher = Config {
            reserve_e8s: pair.reserve_e8s + 1,
            check_interval_secs: 3_600,
            ..pair.clone()
        };
        assert!(!needs_second_admin(&pair, &higher));
    }
}
