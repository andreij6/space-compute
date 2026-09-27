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
    pub fuel_pack_usd_cents: u32,
    pub margin_bp: u16,
    pub treasury_reserve_floor_e8s: u64,
    pub treasury_daily_cap_e8s: u64,
    pub per_aaa_daily_packs: u16,
    pub rate_max_age_secs: u64,
    pub auto_topup_min_interval_secs: u64,
    pub intake_min_runway_days: u32,
}

impl Default for Params {
    fn default() -> Self {
        Params {
            fuel_pack_usd_cents: 500,
            margin_bp: 500,
            treasury_reserve_floor_e8s: 50 * 100_000_000,
            treasury_daily_cap_e8s: 100 * 100_000_000,
            per_aaa_daily_packs: 4,
            rate_max_age_secs: 7_200,
            auto_topup_min_interval_secs: 21_600,
            intake_min_runway_days: 21,
        }
    }
}

impl Params {
    pub fn validate(&self) -> Result<(), ApiError> {
        let ranges = [
            ("margin_bp", self.margin_bp as u64, 0, BP as u64),
            (
                "fuel_pack_usd_cents",
                self.fuel_pack_usd_cents as u64,
                1,
                100_000,
            ),
            (
                "per_aaa_daily_packs",
                self.per_aaa_daily_packs as u64,
                1,
                1_000,
            ),
            ("rate_max_age_secs", self.rate_max_age_secs, 1, 30 * 86_400),
            (
                "auto_topup_min_interval_secs",
                self.auto_topup_min_interval_secs,
                0,
                30 * 86_400,
            ),
            (
                "intake_min_runway_days",
                self.intake_min_runway_days as u64,
                0,
                3_650,
            ),
        ];
        if let Some((name, _, lo, hi)) = ranges.iter().find(|(_, v, lo, hi)| v < lo || v > hi) {
            return Err(ApiError::invalid(format!("{name} must be in {lo}..={hi}")));
        }
        Ok(())
    }
}

#[derive(CandidType, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct Features {
    pub btc: bool,
    pub eth: bool,
    pub sponsored_spawn: bool,
}

impl Default for Features {
    fn default() -> Self {
        Features {
            btc: true,
            eth: true,
            sponsored_spawn: true,
        }
    }
}

#[derive(CandidType, Deserialize, Clone, Debug, Default, PartialEq, Eq)]
pub struct PauseFlags {
    pub spawn: bool,
    pub topup: bool,
    pub auto_topup: bool,
    pub non_icp: bool,
}

#[derive(CandidType, Deserialize, Clone, Debug, PartialEq)]
pub struct Config {
    pub v: u8,
    pub admins: Vec<Principal>,
    pub params: Params,
    pub features: Features,
    pub paused: PauseFlags,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            v: 1,
            admins: vec![],
            params: Params::default(),
            features: Features::default(),
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn t5_1_default_params_and_features_match_spec() {
        let p = Params::default();
        assert_eq!(p.fuel_pack_usd_cents, 500);
        assert_eq!(p.margin_bp, 500);
        assert_eq!(p.treasury_reserve_floor_e8s, 5_000_000_000);
        p.validate().unwrap();
        let f = Features::default();
        assert!(f.btc && f.eth && f.sponsored_spawn);
    }

    #[test]
    fn t5_1_params_validation_rejects_bad_values() {
        let bad = [
            Params {
                margin_bp: 10_001,
                ..Params::default()
            },
            Params {
                fuel_pack_usd_cents: 0,
                ..Params::default()
            },
            Params {
                per_aaa_daily_packs: 0,
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
    fn t5_1_admin_set_rules_and_stored_config_round_trip() {
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
    }
}
