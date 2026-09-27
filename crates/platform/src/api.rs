use candid::{CandidType, Principal};
use sc_types::ApiError;
use serde::Deserialize;

use crate::audit::{self, AuditEntry};
use crate::config::{self, Params, PauseFlags};
use crate::rng;

#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct Overview {
    pub admins: Vec<Principal>,
    pub params: Params,
    pub paused: PauseFlags,
    pub current_protocol_version: u16,
    pub cycles: u128,
    pub rng_seeded_at: Option<u64>,
    pub audit_entries: u64,
}

fn require_admin() -> Result<Principal, ApiError> {
    let caller = ic_cdk::api::msg_caller();
    if caller == Principal::anonymous() || !config::is_admin(&caller) {
        return Err(ApiError::Unauthorized);
    }
    Ok(caller)
}

fn audit<A: CandidType>(admin: Principal, method: &str, args: &A, summary: String) {
    audit::record(ic_cdk::api::time(), admin, method, args, summary);
}

#[ic_cdk::update]
fn admin_add_admin(p: Principal) -> Result<(), ApiError> {
    let caller = require_admin()?;
    config::update(|c| c.add_admin(p))?;
    audit(caller, "admin_add_admin", &p, format!("added {p}"));
    Ok(())
}

#[ic_cdk::update]
fn admin_remove_admin(p: Principal) -> Result<(), ApiError> {
    let caller = require_admin()?;
    config::update(|c| c.remove_admin(p))?;
    audit(caller, "admin_remove_admin", &p, format!("removed {p}"));
    Ok(())
}

#[ic_cdk::update]
fn admin_set_params(params: Params) -> Result<(), ApiError> {
    let caller = require_admin()?;
    params.validate()?;
    config::update(|c| {
        c.params = params.clone();
        Ok(())
    })?;
    audit(
        caller,
        "admin_set_params",
        &params,
        "params replaced".into(),
    );
    Ok(())
}

#[ic_cdk::update]
fn admin_pause(flags: PauseFlags) -> Result<(), ApiError> {
    let caller = require_admin()?;
    config::update(|c| {
        c.paused = flags.clone();
        Ok(())
    })?;
    let summary = format!(
        "tasks={} reviews={} spawns={}",
        flags.tasks, flags.reviews, flags.spawns
    );
    audit(caller, "admin_pause", &flags, summary);
    Ok(())
}

#[ic_cdk::query]
fn admin_list_admins() -> Result<Vec<Principal>, ApiError> {
    require_admin()?;
    Ok(config::get().admins)
}

#[ic_cdk::query]
fn admin_audit_log(cursor: Option<u64>, limit: u32) -> Result<Vec<AuditEntry>, ApiError> {
    require_admin()?;
    Ok(audit::page(cursor, limit))
}

#[ic_cdk::query]
fn admin_overview() -> Result<Overview, ApiError> {
    require_admin()?;
    let c = config::get();
    Ok(Overview {
        admins: c.admins,
        params: c.params,
        paused: c.paused,
        current_protocol_version: c.current_protocol_version,
        cycles: ic_cdk::api::canister_cycle_balance(),
        rng_seeded_at: rng::seeded_at(),
        audit_entries: audit::len(),
    })
}

#[ic_cdk::query]
fn get_params() -> Params {
    config::get().params
}
