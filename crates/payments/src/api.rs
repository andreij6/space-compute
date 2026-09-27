use candid::{CandidType, Principal};
use ic_cdk::call::Call;
use sc_types::ApiError;
use serde::Deserialize;

use crate::audit::{self, AuditEntry};
use crate::config::{self, Features, Params, PauseFlags};
use crate::guard::{self, CallerGuard};
use crate::journal::{self, NotifiedInfo, Op, OpState, PayPath};

#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct Overview {
    pub admins: Vec<Principal>,
    pub params: Params,
    pub features: Features,
    pub paused: PauseFlags,
    pub cycles: u128,
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

async fn raw_rand() -> Result<Vec<u8>, ApiError> {
    let reply = Call::bounded_wait(Principal::management_canister(), "raw_rand")
        .with_args(&())
        .await
        .map_err(|e| ApiError::Internal(format!("raw_rand: {e:?}")))?;
    reply
        .candid::<Vec<u8>>()
        .map_err(|e| ApiError::Internal(format!("raw_rand decode: {e:?}")))
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
fn admin_set_features(features: Features) -> Result<(), ApiError> {
    let caller = require_admin()?;
    config::update(|c| {
        c.features = features.clone();
        Ok(())
    })?;
    audit(
        caller,
        "admin_set_features",
        &features,
        format!(
            "card={} btc={} eth={} sponsored_spawn={}",
            features.card, features.btc, features.eth, features.sponsored_spawn
        ),
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
        "spawn={} topup={} auto_topup={} non_icp={}",
        flags.spawn, flags.topup, flags.auto_topup, flags.non_icp
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
fn admin_list_ops(cursor: Option<u64>, limit: u32) -> Result<Vec<Op>, ApiError> {
    require_admin()?;
    Ok(journal::list_by_created(cursor, limit))
}

#[ic_cdk::query]
fn admin_overview() -> Result<Overview, ApiError> {
    require_admin()?;
    let c = config::get();
    Ok(Overview {
        admins: c.admins,
        params: c.params,
        features: c.features,
        paused: c.paused,
        cycles: ic_cdk::api::canister_cycle_balance(),
        audit_entries: audit::len(),
    })
}

#[ic_cdk::query]
fn get_params() -> Params {
    config::get().params
}

#[ic_cdk::query]
fn get_features() -> Features {
    config::get().features
}

#[ic_cdk::query]
fn get_op(id: u64) -> Option<Op> {
    journal::get(id)
}

#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct JournalDemoArg {
    pub aaa: Principal,
    pub trap_after_await: bool,
}

#[ic_cdk::update]
async fn admin_journal_demo(arg: JournalDemoArg) -> Result<u64, ApiError> {
    let caller = require_admin()?;
    let _guard = CallerGuard::acquire(guard::key("demo", arg.aaa))?;
    let op = journal::create(
        journal::OpKind::TopUp { aaa: arg.aaa },
        PayPath::Deposit,
        0,
        caller,
        ic_cdk::api::time(),
    );
    let id = op.id;
    journal::advance(id, OpState::Pulled { block: 0 }, ic_cdk::api::time())?;
    raw_rand().await?;
    if arg.trap_after_await {
        ic_cdk::trap("t5_1 injected trap after await");
    }
    journal::advance(
        id,
        OpState::Notified {
            canister_or_cycles: NotifiedInfo::Cycles(0),
        },
        ic_cdk::api::time(),
    )?;
    journal::advance(id, OpState::Done, ic_cdk::api::time())?;
    Ok(id)
}
