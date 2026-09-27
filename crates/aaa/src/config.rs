use std::cell::RefCell;

use candid::{CandidType, Principal};
use ic_stable_structures::StableCell;
use sc_types::ApiError;
use serde::Deserialize;

use crate::memory::{self, Memory};

#[derive(CandidType, Deserialize, Clone, Debug, PartialEq)]
pub struct AaaInit {
    pub owner: Principal,
    pub platform_id: Principal,
    pub payments_id: Principal,
    pub name: String,
    pub avatar_seed: u64,
}

#[derive(CandidType, Deserialize, Clone, Debug, PartialEq)]
pub struct Config {
    pub v: u8,
    pub owner: Principal,
    pub platform_id: Principal,
    pub payments_id: Principal,
    pub name: String,
    pub avatar_seed: u64,
    pub agent_label: Option<String>,
    pub auto_topup: Option<u128>,
    pub wasm_version: String,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            v: 1,
            owner: Principal::anonymous(),
            platform_id: Principal::anonymous(),
            payments_id: Principal::anonymous(),
            name: String::new(),
            avatar_seed: 0,
            agent_label: None,
            auto_topup: None,
            wasm_version: String::new(),
        }
    }
}

impl Config {
    pub fn from_init(init: AaaInit) -> Result<Self, ApiError> {
        if init.owner == Principal::anonymous() {
            return Err(ApiError::invalid("owner can't be the anonymous principal"));
        }
        if init.platform_id == Principal::anonymous() || init.payments_id == Principal::anonymous()
        {
            return Err(ApiError::invalid(
                "platform_id and payments_id can't be the anonymous principal",
            ));
        }
        let name = sc_types::limits::aaa_name(&init.name)?;
        Ok(Config {
            v: 1,
            owner: init.owner,
            platform_id: init.platform_id,
            payments_id: init.payments_id,
            name,
            avatar_seed: init.avatar_seed,
            agent_label: None,
            auto_topup: None,
            wasm_version: String::new(),
        })
    }

    pub fn set_agent_label(&mut self, label: Option<String>) -> Result<(), ApiError> {
        sc_types::limits::agent_label(&label)?;
        self.agent_label = label;
        Ok(())
    }

    pub fn set_auto_topup(&mut self, threshold: Option<u128>) -> Result<(), ApiError> {
        if threshold == Some(0) {
            return Err(ApiError::invalid("auto_topup threshold must be > 0"));
        }
        self.auto_topup = threshold;
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

pub fn set(cfg: Config) {
    CELL.with_borrow_mut(|c| {
        c.set(cfg);
    });
}

pub fn update<R>(f: impl FnOnce(&mut Config) -> Result<R, ApiError>) -> Result<R, ApiError> {
    CELL.with_borrow_mut(|c| {
        let mut next = c.get().clone();
        let out = f(&mut next)?;
        c.set(next);
        Ok(out)
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn p(n: u8) -> Principal {
        Principal::from_slice(&[n; 29])
    }

    fn init(owner: Principal) -> AaaInit {
        AaaInit {
            owner,
            platform_id: p(200),
            payments_id: p(201),
            name: "Rover One".into(),
            avatar_seed: 7,
        }
    }

    #[test]
    fn t3_1_from_init_rejects_anonymous_principals_and_bad_names() {
        assert!(matches!(
            Config::from_init(init(Principal::anonymous())),
            Err(ApiError::InvalidInput(_))
        ));
        let mut bad = init(p(1));
        bad.platform_id = Principal::anonymous();
        assert!(matches!(
            Config::from_init(bad),
            Err(ApiError::InvalidInput(_))
        ));
        let mut short_name = init(p(1));
        short_name.name = "x".into();
        assert!(matches!(
            Config::from_init(short_name),
            Err(ApiError::InvalidInput(_))
        ));
    }

    #[test]
    fn t3_1_from_init_stores_the_normalized_config() {
        let cfg = Config::from_init(init(p(1))).unwrap();
        assert_eq!(cfg.owner, p(1));
        assert_eq!(cfg.platform_id, p(200));
        assert_eq!(cfg.payments_id, p(201));
        assert_eq!(cfg.name, "Rover One");
        assert_eq!(cfg.avatar_seed, 7);
        assert_eq!(cfg.agent_label, None);
        assert_eq!(cfg.auto_topup, None);
    }

    #[test]
    fn t3_1_set_agent_label_and_auto_topup_validate() {
        let mut cfg = Config::from_init(init(p(1))).unwrap();
        cfg.set_agent_label(Some("claude-code".into())).unwrap();
        assert_eq!(cfg.agent_label.as_deref(), Some("claude-code"));
        assert!(matches!(
            cfg.set_agent_label(Some("x".repeat(65))),
            Err(ApiError::InvalidInput(_))
        ));
        cfg.set_auto_topup(Some(1_000_000)).unwrap();
        assert_eq!(cfg.auto_topup, Some(1_000_000));
        assert!(matches!(
            cfg.set_auto_topup(Some(0)),
            Err(ApiError::InvalidInput(_))
        ));
        cfg.set_auto_topup(None).unwrap();
        assert_eq!(cfg.auto_topup, None);
    }

    #[test]
    fn t3_1_stored_config_round_trips_through_the_stable_cell() {
        let cfg = Config::from_init(init(p(5))).unwrap();
        set(cfg.clone());
        assert_eq!(get(), cfg);
        update(|c| c.set_agent_label(Some("bot".into()))).unwrap();
        assert_eq!(get().agent_label.as_deref(), Some("bot"));
        assert!(update(|c| c.set_agent_label(Some("x".repeat(65)))).is_err());
    }
}
