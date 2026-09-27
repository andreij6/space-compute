use candid::Principal;
use sc_types::ApiError;

use crate::config::{self, Config};
use crate::keeper;
use crate::report::{self, Health, Status};
use crate::state::{self, Action, HistoryItem, HistoryKind, Proposal};
use crate::timers::{self, Outcome};

fn require_admin() -> Result<Principal, ApiError> {
    let caller = ic_cdk::api::msg_caller();
    if !config::is_admin(&caller) {
        return Err(ApiError::Unauthorized);
    }
    Ok(caller)
}

fn now() -> u64 {
    ic_cdk::api::time()
}

fn audit(admin: Principal, method: &str, summary: String) {
    state::log(
        now(),
        admin,
        HistoryKind::Admin {
            method: method.into(),
            summary,
        },
    );
}

fn runways() -> Vec<report::CanisterRunway> {
    let watched: Vec<_> = state::watched()
        .into_iter()
        .map(|(c, w)| (c, w.priority, state::metric(&c)))
        .collect();
    report::runways(&watched)
}

fn apply_config(c: Config) -> Result<(), ApiError> {
    let interval_changed = c.check_interval_secs != config::get().check_interval_secs;
    config::set(c)?;
    if interval_changed {
        timers::restart();
    }
    Ok(())
}

#[ic_cdk::query]
fn health() -> Health {
    report::health(&config::get(), &state::snapshot(), &runways())
}

#[ic_cdk::query]
fn status() -> Status {
    report::status(&config::get(), &state::snapshot(), runways())
}

#[ic_cdk::query]
fn history(cursor: Option<u64>, limit: u32) -> (Vec<HistoryItem>, Option<u64>) {
    state::history(cursor, limit)
}

#[ic_cdk::query]
fn admin_get_config() -> Result<Config, ApiError> {
    require_admin()?;
    Ok(config::get())
}

#[ic_cdk::query]
fn admin_proposals() -> Result<Vec<(u64, Proposal)>, ApiError> {
    require_admin()?;
    Ok(state::proposals())
}

#[ic_cdk::update]
fn admin_set_config(c: Config) -> Result<Option<u64>, ApiError> {
    let admin = require_admin()?;
    c.validate()?;
    if config::needs_second_admin(&config::get(), &c) {
        let id = state::propose(Action::SetConfig(c), admin, now());
        state::log(
            now(),
            admin,
            HistoryKind::Proposed {
                id,
                summary: "config change (admins or lower reserve)".into(),
            },
        );
        return Ok(Some(id));
    }
    apply_config(c.clone())?;
    audit(
        admin,
        "admin_set_config",
        format!("{} admins, reserve {} e8s", c.admins.len(), c.reserve_e8s),
    );
    Ok(None)
}

#[ic_cdk::update]
fn admin_watch(canister: Principal, priority: u8, target_days: u32) -> Result<(), ApiError> {
    let admin = require_admin()?;
    state::watch(canister, priority, target_days)?;
    audit(
        admin,
        "admin_watch",
        format!("{canister} priority {priority} target {target_days}d"),
    );
    Ok(())
}

#[ic_cdk::update]
fn admin_unwatch(canister: Principal) -> Result<(), ApiError> {
    let admin = require_admin()?;
    state::unwatch(canister)?;
    audit(admin, "admin_unwatch", canister.to_string());
    Ok(())
}

#[ic_cdk::update]
async fn admin_topup_now(canister: Option<Principal>) -> Result<Health, ApiError> {
    let admin = require_admin()?;
    let target = canister
        .map(|c| c.to_string())
        .unwrap_or_else(|| "all".into());
    audit(
        admin,
        "admin_topup_now",
        format!("keeper run forced for {target}"),
    );
    timers::tick(canister).await;
    Ok(health())
}

#[ic_cdk::update]
fn admin_withdraw(to: Principal, amount_e8s: u64) -> Result<u64, ApiError> {
    let admin = require_admin()?;
    if to == Principal::anonymous() || amount_e8s == 0 {
        return Err(ApiError::invalid(
            "withdraw needs a recipient and a positive amount",
        ));
    }
    let id = state::propose(Action::Withdraw { to, amount_e8s }, admin, now());
    state::log(
        now(),
        admin,
        HistoryKind::Proposed {
            id,
            summary: format!("withdraw {amount_e8s} e8s to {to}"),
        },
    );
    Ok(id)
}

#[ic_cdk::update]
async fn admin_approve(id: u64) -> Result<Option<u64>, ApiError> {
    let admin = require_admin()?;
    let window_ns = config::get().withdraw_approval_secs * 1_000_000_000;
    let p = state::approve(id, admin, now(), window_ns)?;
    match p.action {
        Action::SetConfig(c) => {
            let applied = apply_config(c);
            state::finish(id);
            applied?;
            state::log(now(), admin, HistoryKind::ConfigChanged { id });
            Ok(None)
        }
        Action::Withdraw { to, amount_e8s } => {
            let created_at = p.executing_since.expect("approved proposals are executing");
            match timers::transfer(to, None, amount_e8s, None, created_at).await {
                Outcome::Done(block) => {
                    state::finish(id);
                    let mut s = state::snapshot();
                    s.icp_balance_e8s = s.icp_balance_e8s.saturating_sub(amount_e8s + keeper::LEDGER_FEE_E8S);
                    state::set_snapshot(s);
                    state::log(now(), admin, HistoryKind::Withdrawn { id, to, amount_e8s, block });
                    Ok(Some(block))
                }
                Outcome::Failed(reason) => {
                    state::release(id);
                    state::log(now(), admin, HistoryKind::WithdrawFailed { id, reason: reason.clone() });
                    Err(ApiError::Internal(reason))
                }
                Outcome::Unknown(reason) => Err(ApiError::Internal(format!(
                    "transfer outcome unknown ({reason}); call admin_approve({id}) again to retry safely"
                ))),
            }
        }
    }
}
