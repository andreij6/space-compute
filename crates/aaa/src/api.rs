use candid::Principal;
use sc_types::ApiError;

use crate::config;
use crate::operators::{self, Operator};
use crate::roles::{self, Role};

fn require_owner() -> Result<Principal, ApiError> {
    let caller = ic_cdk::api::msg_caller();
    let owner = config::get().owner;
    roles::require_owner(caller, owner)?;
    Ok(caller)
}

#[ic_cdk::update]
fn add_operator(p: Principal, label: String, expires_at: Option<u64>) -> Result<(), ApiError> {
    let owner = require_owner()?;
    let now = ic_cdk::api::time();
    operators::add(owner, p, label, expires_at, now)
}

#[ic_cdk::update]
fn remove_operator(p: Principal) -> Result<(), ApiError> {
    require_owner()?;
    operators::remove(p)
}

#[ic_cdk::update]
fn set_agent_label(label: Option<String>) -> Result<(), ApiError> {
    require_owner()?;
    config::update(|c| c.set_agent_label(label))
}

#[ic_cdk::update]
fn set_auto_topup(threshold: Option<u128>) -> Result<(), ApiError> {
    require_owner()?;
    config::update(|c| c.set_auto_topup(threshold))
}

#[ic_cdk::query]
fn whoami() -> Role {
    let caller = ic_cdk::api::msg_caller();
    let cfg = config::get();
    let now = ic_cdk::api::time();
    roles::classify(
        caller,
        cfg.owner,
        cfg.platform_id,
        operators::is_active(&caller, now),
    )
}

#[ic_cdk::query]
fn list_operators() -> Result<Vec<(Principal, Operator)>, ApiError> {
    require_owner()?;
    Ok(operators::list())
}
