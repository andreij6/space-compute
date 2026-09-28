use std::cell::RefCell;

use candid::{CandidType, Principal};
use ic_stable_structures::StableCell;
use sc_types::ApiError;
use serde::Deserialize;

use crate::memory::{self, Memory};

pub const MAX_ADMINS: usize = 20;
const BP: u16 = 10_000;

#[derive(CandidType, Deserialize, Clone, Debug, PartialEq)]
pub struct Params {
    pub fee_get_task: u128,
    pub fee_submit_classification: u128,
    pub fee_get_review: u128,
    pub fee_submit_review: u128,
    pub retire_after_k: u16,
    pub gold_rate_bp: u16,
    pub calibration_tasks: u32,
    pub calibration_gold_rate_bp: u16,
    pub review_starvation_days: u32,
    pub heartbeat_min_interval_secs: u64,
    pub honeypot_rate_bp: u16,
    pub lease_task_secs: u64,
    pub lease_review_secs: u64,
    pub max_open_leases_per_aaa: u16,
    pub max_tasks_per_aaa_per_hour: u32,
    pub reviews_min: u16,
    pub reviews_max: u16,
    pub max_flag_rate_bp: u16,
    pub aaa_initial_cycles: u128,
    pub data_refresh_interval_days: u32,
    pub claim_cell_arcsec: f64,
    pub claim_reopen_days: u32,
}

impl Default for Params {
    fn default() -> Self {
        Params {
            fee_get_task: 60_000_000,
            fee_submit_classification: 50_000_000,
            fee_get_review: 20_000_000,
            fee_submit_review: 50_000_000,
            retire_after_k: 5,
            gold_rate_bp: 1_000,
            calibration_tasks: 50,
            calibration_gold_rate_bp: 4_000,
            review_starvation_days: 7,
            heartbeat_min_interval_secs: 3_600,
            honeypot_rate_bp: 1_000,
            lease_task_secs: 1_800,
            lease_review_secs: 86_400,
            max_open_leases_per_aaa: 3,
            max_tasks_per_aaa_per_hour: 120,
            reviews_min: 3,
            reviews_max: 7,
            max_flag_rate_bp: 1_000,
            aaa_initial_cycles: 1_000_000_000_000,
            data_refresh_interval_days: 15,
            claim_cell_arcsec: 1.5,
            claim_reopen_days: 30,
        }
    }
}

const DAY_SECS: u64 = 86_400;

impl Params {
    pub fn validate(&self) -> Result<(), ApiError> {
        let bp = BP as u64;
        let ranges = [
            ("gold_rate_bp", self.gold_rate_bp as u64, 0, bp),
            (
                "calibration_gold_rate_bp",
                self.calibration_gold_rate_bp as u64,
                0,
                bp,
            ),
            ("honeypot_rate_bp", self.honeypot_rate_bp as u64, 0, bp),
            ("max_flag_rate_bp", self.max_flag_rate_bp as u64, 0, bp),
            ("retire_after_k", self.retire_after_k as u64, 1, 1_000),
            (
                "calibration_tasks",
                self.calibration_tasks as u64,
                0,
                10_000,
            ),
            ("lease_task_secs", self.lease_task_secs, 1, 30 * DAY_SECS),
            (
                "lease_review_secs",
                self.lease_review_secs,
                1,
                30 * DAY_SECS,
            ),
            (
                "heartbeat_min_interval_secs",
                self.heartbeat_min_interval_secs,
                0,
                30 * DAY_SECS,
            ),
            (
                "max_open_leases_per_aaa",
                self.max_open_leases_per_aaa as u64,
                1,
                100,
            ),
            (
                "max_tasks_per_aaa_per_hour",
                self.max_tasks_per_aaa_per_hour as u64,
                1,
                100_000,
            ),
            ("reviews_min", self.reviews_min as u64, 1, 100),
            ("reviews_max", self.reviews_max as u64, 1, 100),
            (
                "review_starvation_days",
                self.review_starvation_days as u64,
                1,
                3_650,
            ),
            (
                "data_refresh_interval_days",
                self.data_refresh_interval_days as u64,
                1,
                3_650,
            ),
            ("claim_reopen_days", self.claim_reopen_days as u64, 0, 3_650),
        ];
        if let Some((name, _, lo, hi)) = ranges.iter().find(|(_, v, lo, hi)| v < lo || v > hi) {
            return Err(ApiError::invalid(format!("{name} must be in {lo}..={hi}")));
        }
        if self.reviews_min > self.reviews_max {
            return Err(ApiError::invalid("reviews_min must be ≤ reviews_max"));
        }
        if !(self.claim_cell_arcsec > 0.0 && self.claim_cell_arcsec <= 3_600.0) {
            return Err(ApiError::invalid("claim_cell_arcsec must be in (0, 3600]"));
        }
        Ok(())
    }
}

#[derive(CandidType, Deserialize, Clone, Debug, Default, PartialEq, Eq)]
pub struct PauseFlags {
    pub tasks: bool,
    pub reviews: bool,
    pub spawns: bool,
}

#[derive(CandidType, Deserialize, Clone, Debug, PartialEq)]
pub struct Config {
    pub v: u8,
    pub admins: Vec<Principal>,
    pub payments_id: Option<Principal>,
    pub params: Params,
    pub current_protocol_version: u16,
    pub paused: PauseFlags,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            v: 1,
            admins: vec![],
            payments_id: None,
            params: Params::default(),
            current_protocol_version: 1,
            paused: PauseFlags::default(),
        }
    }
}

impl Config {
    pub fn add_admin(&mut self, p: Principal) -> Result<(), ApiError> {
        if p == Principal::anonymous() {
            return Err(ApiError::invalid("anonymous can't be an admin"));
        }
        if self.admins.contains(&p) {
            return Err(ApiError::Conflict("already an admin".into()));
        }
        if self.admins.len() >= MAX_ADMINS {
            return Err(ApiError::invalid("too many admins"));
        }
        self.admins.push(p);
        Ok(())
    }

    pub fn remove_admin(&mut self, p: Principal) -> Result<(), ApiError> {
        if !self.admins.contains(&p) {
            return Err(ApiError::NotFound);
        }
        if self.admins.len() == 1 {
            return Err(ApiError::Conflict("the last admin can't be removed".into()));
        }
        self.admins.retain(|a| a != &p);
        Ok(())
    }

    pub fn set_payments_id(&mut self, p: Principal) -> Result<(), ApiError> {
        if p == Principal::anonymous() {
            return Err(ApiError::invalid("payments_id can't be anonymous"));
        }
        self.payments_id = Some(p);
        Ok(())
    }

    pub fn set_current_protocol_version(&mut self, version: u16) -> Result<(), ApiError> {
        if version == 0 {
            return Err(ApiError::invalid("protocol version must be > 0"));
        }
        self.current_protocol_version = version;
        Ok(())
    }
}

crate::candid_storable!(Config);

thread_local! {
    static CELL: RefCell<StableCell<Config, Memory>> =
        RefCell::new(StableCell::init(memory::get(memory::CONFIG), Config::default()));
}

pub fn get() -> Config {
    CELL.with_borrow(|c| c.get().clone())
}

pub fn update<R>(f: impl FnOnce(&mut Config) -> Result<R, ApiError>) -> Result<R, ApiError> {
    CELL.with_borrow_mut(|c| {
        let mut next = c.get().clone();
        let out = f(&mut next)?;
        c.set(next);
        Ok(out)
    })
}

pub fn is_admin(p: &Principal) -> bool {
    CELL.with_borrow(|c| c.get().admins.contains(p))
}

pub fn payments_id() -> Option<Principal> {
    CELL.with_borrow(|c| c.get().payments_id)
}

pub fn is_payments(p: &Principal) -> bool {
    CELL.with_borrow(|c| c.get().payments_id.as_ref() == Some(p))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn t2_1_default_params_match_spec_01_and_validate() {
        let p = Params::default();
        assert_eq!(p.fee_get_task, 60_000_000);
        assert_eq!(p.fee_submit_classification, 50_000_000);
        assert_eq!(p.fee_get_review, 20_000_000);
        assert_eq!(p.fee_submit_review, 50_000_000);
        assert_eq!(p.aaa_initial_cycles, 1_000_000_000_000);
        assert_eq!(p.retire_after_k, 5);
        assert_eq!(p.data_refresh_interval_days, 15);
        p.validate().unwrap();
    }

    struct MeasuredCost {
        avg: u128,
        p99: u128,
    }

    const SAFETY_FACTOR: u128 = 2;
    const GET_TASK: MeasuredCost = MeasuredCost {
        avg: 15_080_000,
        p99: 57_040_000,
    };
    const SUBMIT_CLASSIFICATION: MeasuredCost = MeasuredCost {
        avg: 21_110_000,
        p99: 46_970_000,
    };
    const GET_REVIEW: MeasuredCost = MeasuredCost {
        avg: 8_450_000,
        p99: 9_040_000,
    };
    const SUBMIT_REVIEW: MeasuredCost = MeasuredCost {
        avg: 24_540_000,
        p99: 36_290_000,
    };

    fn required_fee(c: &MeasuredCost) -> u128 {
        (c.avg * SAFETY_FACTOR).max(c.p99)
    }

    #[test]
    fn t7_3_default_fees_cover_measured_platform_cost_times_safety_factor() {
        let p = Params::default();
        for (name, fee, cost) in [
            ("fee_get_task", p.fee_get_task, GET_TASK),
            (
                "fee_submit_classification",
                p.fee_submit_classification,
                SUBMIT_CLASSIFICATION,
            ),
            ("fee_get_review", p.fee_get_review, GET_REVIEW),
            ("fee_submit_review", p.fee_submit_review, SUBMIT_REVIEW),
        ] {
            let need = required_fee(&cost);
            assert!(fee >= need, "{name} = {fee} < required {need}");
            assert!(
                fee <= need * 3 / 2,
                "{name} = {fee} overcharges owners (> 1.5 × required {need})"
            );
        }
    }

    #[test]
    fn t2_1_params_validation_rejects_bad_values() {
        let bad = [
            Params {
                gold_rate_bp: 10_001,
                ..Params::default()
            },
            Params {
                reviews_min: 8,
                ..Params::default()
            },
            Params {
                retire_after_k: 0,
                ..Params::default()
            },
            Params {
                claim_cell_arcsec: f64::NAN,
                ..Params::default()
            },
        ];
        for p in bad {
            assert!(
                matches!(p.validate(), Err(ApiError::InvalidInput(_))),
                "{p:?}"
            );
        }
    }

    #[test]
    fn t2_1_admin_set_rules_and_stored_config_round_trip() {
        let (a, b) = (
            Principal::from_slice(&[1; 29]),
            Principal::from_slice(&[2; 29]),
        );
        let mut c = Config::default();
        c.add_admin(a).unwrap();
        assert!(matches!(c.add_admin(a), Err(ApiError::Conflict(_))));
        assert!(matches!(
            c.add_admin(Principal::anonymous()),
            Err(ApiError::InvalidInput(_))
        ));
        assert!(matches!(c.remove_admin(a), Err(ApiError::Conflict(_))));
        assert_eq!(c.remove_admin(b), Err(ApiError::NotFound));
        c.add_admin(b).unwrap();
        c.remove_admin(a).unwrap();
        assert_eq!(c.admins, vec![b]);
        for i in 0..(MAX_ADMINS - 1) {
            c.add_admin(Principal::from_slice(&[10 + i as u8; 29]))
                .unwrap();
        }
        assert!(matches!(c.add_admin(a), Err(ApiError::InvalidInput(_))));

        update(|cfg| cfg.add_admin(a)).unwrap();
        assert!(update(|cfg| cfg.add_admin(a)).is_err());
        assert!(is_admin(&a) && !is_admin(&b));
        assert_eq!(get().admins, vec![a]);

        assert!(matches!(
            c.set_payments_id(Principal::anonymous()),
            Err(ApiError::InvalidInput(_))
        ));
        c.set_payments_id(b).unwrap();
        assert_eq!(c.payments_id, Some(b));

        update(|cfg| cfg.set_payments_id(b)).unwrap();
        assert_eq!(payments_id(), Some(b));
        assert!(is_payments(&b));
        assert!(!is_payments(&a));

        assert!(matches!(
            c.set_current_protocol_version(0),
            Err(ApiError::InvalidInput(_))
        ));
        c.set_current_protocol_version(3).unwrap();
        assert_eq!(c.current_protocol_version, 3);
    }
}
