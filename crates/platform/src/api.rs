use candid::{CandidType, Principal};
use ic_cdk::call::Call;
use sc_types::ApiError;
use serde::Deserialize;

use crate::audit::{self, AuditEntry};
use crate::catalog::{self, AdminListSubjectsFilter, Lease, Subject, SubjectInput};
use crate::config::{self, Params, PauseFlags};
use crate::registry::{
    self, AaaRecord, AdminListAaasFilter, CheckNameResult, Heartbeat, OperatorSetInput,
    RegisterArgs, UpdateAaaProfileArgs, WasmMeta,
};
use crate::rng;

#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct Overview {
    pub admins: Vec<Principal>,
    pub payments_id: Option<Principal>,
    pub params: Params,
    pub paused: PauseFlags,
    pub current_protocol_version: u16,
    pub cycles: u128,
    pub rng_seeded_at: Option<u64>,
    pub audit_entries: u64,
}

#[derive(CandidType, Deserialize, Clone, Debug)]
pub enum CanisterInstallMode {
    #[serde(rename = "install")]
    Install,
    #[serde(rename = "reinstall")]
    Reinstall,
    #[serde(rename = "upgrade")]
    Upgrade(Option<CanisterUpgradeOptions>),
}

#[derive(CandidType, Deserialize, Clone, Debug, Default)]
pub struct CanisterUpgradeOptions {
    pub skip_pre_upgrade: Option<bool>,
    pub wasm_memory_persistence: Option<WasmMemoryPersistence>,
}

#[derive(CandidType, Deserialize, Clone, Debug)]
pub enum WasmMemoryPersistence {
    #[serde(rename = "keep")]
    Keep,
    #[serde(rename = "replace")]
    Replace,
}

#[derive(CandidType)]
struct InstallCodeArgs {
    mode: CanisterInstallMode,
    canister_id: Principal,
    wasm_module: Vec<u8>,
    arg: Vec<u8>,
    sender_canister_version: Option<u64>,
}

#[derive(CandidType)]
struct CanisterIdRecord {
    canister_id: Principal,
}

#[derive(CandidType)]
struct CanisterInfoArgs {
    canister_id: Principal,
    num_requested_changes: Option<u64>,
}

#[derive(CandidType, Deserialize)]
struct CanisterInfoResult {
    total_num_changes: u64,
    module_hash: Option<Vec<u8>>,
    controllers: Vec<Principal>,
}

#[derive(CandidType)]
struct AaaInitArg {
    owner: Principal,
    platform_id: Principal,
    payments_id: Principal,
    name: String,
    avatar_seed: u64,
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
fn admin_set_payments_id(p: Principal) -> Result<(), ApiError> {
    let caller = require_admin()?;
    config::update(|c| c.set_payments_id(p))?;
    audit(
        caller,
        "admin_set_payments_id",
        &p,
        format!("payments_id set to {p}"),
    );
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

#[ic_cdk::update]
fn admin_upload_wasm(version: u32, blob: Vec<u8>, sha256: Vec<u8>) -> Result<(), ApiError> {
    let caller = require_admin()?;
    registry::upload_wasm(version, blob.clone(), sha256.clone())?;
    audit(
        caller,
        "admin_upload_wasm",
        &(version, sha256),
        format!("uploaded wasm v{version} size={}", blob.len()),
    );
    Ok(())
}

#[ic_cdk::update]
fn admin_approve_wasm(version: u32) -> Result<(), ApiError> {
    let caller = require_admin()?;
    registry::approve_wasm(version, ic_cdk::api::time())?;
    audit(
        caller,
        "admin_approve_wasm",
        &version,
        format!("approved wasm v{version}"),
    );
    Ok(())
}

#[ic_cdk::query]
fn admin_list_wasms() -> Result<Vec<(u32, WasmMeta)>, ApiError> {
    require_admin()?;
    Ok(registry::list_wasms())
}

#[ic_cdk::update]
fn admin_suspend_aaa(aaa: Principal, reason: String) -> Result<(), ApiError> {
    let caller = require_admin()?;
    registry::suspend_aaa(aaa)?;
    audit(
        caller,
        "admin_suspend_aaa",
        &(aaa, reason.clone()),
        format!("suspended {aaa}: {reason}"),
    );
    Ok(())
}

#[ic_cdk::update]
fn admin_unsuspend_aaa(aaa: Principal) -> Result<(), ApiError> {
    let caller = require_admin()?;
    registry::unsuspend_aaa(aaa)?;
    audit(
        caller,
        "admin_unsuspend_aaa",
        &aaa,
        format!("unsuspended {aaa}"),
    );
    Ok(())
}

#[ic_cdk::update]
fn admin_rename_aaa(aaa: Principal, new_name: String, reason: String) -> Result<(), ApiError> {
    let caller = require_admin()?;
    registry::rename_aaa(aaa, new_name.clone())?;
    audit(
        caller,
        "admin_rename_aaa",
        &(aaa, new_name.clone(), reason.clone()),
        format!("renamed {aaa} to {new_name}: {reason}"),
    );
    Ok(())
}

#[ic_cdk::query]
fn admin_get_aaa(aaa: Principal) -> Result<AaaRecord, ApiError> {
    require_admin()?;
    registry::get_aaa(&aaa).ok_or(ApiError::NotFound)
}

#[ic_cdk::query]
fn admin_list_aaas(
    filter: AdminListAaasFilter,
    cursor: Option<u64>,
    limit: u32,
) -> Result<Vec<AaaRecord>, ApiError> {
    require_admin()?;
    Ok(registry::list_aaas(&filter, cursor, limit))
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
        payments_id: c.payments_id,
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

async fn execute_install_and_verify(
    canister_id: Principal,
    owner: Principal,
    name: String,
    avatar_seed: u64,
    wasm_blob: Vec<u8>,
    wasm_version: u32,
    caller: Principal,
) -> Result<(), ApiError> {
    let init_arg = AaaInitArg {
        owner,
        platform_id: ic_cdk::api::canister_self(),
        payments_id: caller,
        name,
        avatar_seed,
    };
    let install_arg =
        candid::encode_one(init_arg).map_err(|e| ApiError::Internal(e.to_string()))?;
    let install_call = Call::bounded_wait(Principal::management_canister(), "install_code")
        .with_arg(InstallCodeArgs {
            mode: CanisterInstallMode::Install,
            canister_id,
            wasm_module: wasm_blob,
            arg: install_arg,
            sender_canister_version: None,
        })
        .await;
    install_call.map_err(|e| ApiError::Internal(format!("install_code failed: {e:?}")))?;

    let info_call = Call::bounded_wait(Principal::management_canister(), "canister_info")
        .with_arg(CanisterInfoArgs {
            canister_id,
            num_requested_changes: Some(1),
        })
        .await
        .map_err(|e| ApiError::Internal(format!("canister_info failed: {e:?}")))?;
    let info: CanisterInfoResult = info_call
        .candid()
        .map_err(|e| ApiError::Internal(format!("canister_info decode: {e:?}")))?;
    let module_hash = info
        .module_hash
        .ok_or_else(|| ApiError::Internal("canister has no module".into()))?;

    registry::complete_register_aaa(
        canister_id,
        wasm_version,
        module_hash,
        &info.controllers,
        ic_cdk::api::canister_self(),
        ic_cdk::api::time(),
    )?;
    Ok(())
}

#[ic_cdk::update]
async fn register_aaa(args: RegisterArgs) -> Result<(), ApiError> {
    let caller = ic_cdk::api::msg_caller();
    if !config::is_payments(&caller) && !config::is_admin(&caller) {
        return Err(ApiError::Unauthorized);
    }
    if config::get().paused.spawns {
        return Err(ApiError::invalid("spawns are paused"));
    }
    let (resolved_name, version, wasm_blob, is_active) =
        registry::pre_register_aaa(&args, ic_cdk::api::time())?;
    if is_active {
        return Ok(());
    }

    execute_install_and_verify(
        args.canister_id,
        args.owner,
        resolved_name,
        args.avatar_seed,
        wasm_blob,
        version,
        caller,
    )
    .await
}

#[ic_cdk::update]
async fn admin_retry_install(canister_id: Principal) -> Result<(), ApiError> {
    let caller = require_admin()?;
    let attempts = registry::increment_install_attempts(&canister_id)?;
    if attempts > 6 {
        return Err(ApiError::invalid("install attempts exceeded limit of 6"));
    }
    let record = registry::get_aaa(&canister_id).ok_or(ApiError::NotFound)?;
    let (version, _, wasm_blob) = registry::latest_approved_wasm()
        .ok_or_else(|| ApiError::Internal("no approved wasm available".into()))?;

    let res = execute_install_and_verify(
        canister_id,
        record.owner,
        record.name,
        record.avatar_seed,
        wasm_blob,
        version,
        caller,
    )
    .await;
    audit(
        caller,
        "admin_retry_install",
        &canister_id,
        format!("retry install for {canister_id} attempt {attempts}"),
    );
    res
}

#[ic_cdk::update]
async fn upgrade_aaa(aaa: Principal) -> Result<(), ApiError> {
    let caller = ic_cdk::api::msg_caller();
    let (record, version, wasm_blob) = registry::pre_upgrade_aaa(aaa, caller)?;

    let _ = Call::bounded_wait(Principal::management_canister(), "stop_canister")
        .with_arg(CanisterIdRecord { canister_id: aaa })
        .await;

    let install_call = Call::bounded_wait(Principal::management_canister(), "install_code")
        .with_arg(InstallCodeArgs {
            mode: CanisterInstallMode::Upgrade(None),
            canister_id: aaa,
            wasm_module: wasm_blob,
            arg: candid::encode_one(()).unwrap(),
            sender_canister_version: None,
        })
        .await;

    let _ = Call::bounded_wait(Principal::management_canister(), "start_canister")
        .with_arg(CanisterIdRecord { canister_id: aaa })
        .await;

    install_call.map_err(|e| ApiError::Internal(format!("install_code failed: {e:?}")))?;

    let info_call = Call::bounded_wait(Principal::management_canister(), "canister_info")
        .with_arg(CanisterInfoArgs {
            canister_id: aaa,
            num_requested_changes: Some(1),
        })
        .await
        .map_err(|e| ApiError::Internal(format!("canister_info failed: {e:?}")))?;
    let info: CanisterInfoResult = info_call
        .candid()
        .map_err(|e| ApiError::Internal(format!("canister_info decode: {e:?}")))?;
    let module_hash = info
        .module_hash
        .ok_or_else(|| ApiError::Internal("canister has no module".into()))?;

    registry::complete_upgrade_aaa(
        aaa,
        version,
        module_hash,
        info.total_num_changes,
        ic_cdk::api::time(),
    )?;
    let _ = record;
    Ok(())
}

#[ic_cdk::update]
async fn verify(aaa: Principal) -> Result<(), ApiError> {
    let info_call = Call::bounded_wait(Principal::management_canister(), "canister_info")
        .with_arg(CanisterInfoArgs {
            canister_id: aaa,
            num_requested_changes: Some(1),
        })
        .await
        .map_err(|e| ApiError::Internal(format!("canister_info failed: {e:?}")))?;
    let info: CanisterInfoResult = info_call
        .candid()
        .map_err(|e| ApiError::Internal(format!("canister_info decode: {e:?}")))?;
    registry::verify_provenance(
        aaa,
        info.module_hash,
        info.total_num_changes,
        &info.controllers,
        ic_cdk::api::canister_self(),
        ic_cdk::api::time(),
    )
}

#[ic_cdk::update]
fn heartbeat(args: Heartbeat) -> Result<(), ApiError> {
    let caller = ic_cdk::api::msg_caller();
    let min_interval = config::get().params.heartbeat_min_interval_secs;
    registry::record_heartbeat(caller, args, min_interval, ic_cdk::api::time())
}

#[ic_cdk::update]
fn sync_operators(args: OperatorSetInput) -> Result<(), ApiError> {
    let caller = ic_cdk::api::msg_caller();
    registry::record_sync_operators(caller, args, ic_cdk::api::time())
}

#[ic_cdk::update]
fn update_aaa_profile(args: UpdateAaaProfileArgs) -> Result<(), ApiError> {
    let caller = ic_cdk::api::msg_caller();
    registry::record_update_profile(caller, args)
}

#[ic_cdk::query]
fn check_name(name: String) -> CheckNameResult {
    registry::check_name(&name)
}

#[ic_cdk::query]
fn get_aaa(aaa: Principal) -> Option<AaaRecord> {
    registry::get_aaa(&aaa)
}

#[ic_cdk::query]
fn aaa_by_owner(owner: Principal) -> Option<Principal> {
    registry::get_aaa_by_owner(&owner)
}

#[ic_cdk::query]
fn aaa_owner(aaa: Principal) -> Option<Principal> {
    registry::get_aaa_owner(&aaa)
}

#[ic_cdk::query]
fn get_aaa_by_name(name: String) -> Option<AaaRecord> {
    registry::get_aaa_by_name(&name)
}

#[ic_cdk::update]
fn admin_add_subjects(batch: Vec<SubjectInput>) -> Result<u32, ApiError> {
    let caller = require_admin()?;
    let count = catalog::add_subjects(batch)?;
    audit(
        caller,
        "admin_add_subjects",
        &count,
        format!("added {count} subjects"),
    );
    Ok(count)
}

#[ic_cdk::update]
fn admin_set_subject_active(subject_id: u32, active: bool) -> Result<(), ApiError> {
    let caller = require_admin()?;
    let retire_after_k = config::get().params.retire_after_k;
    catalog::set_subject_active(subject_id, active, retire_after_k)?;
    audit(
        caller,
        "admin_set_subject_active",
        &(subject_id, active),
        format!("subject {subject_id} active={active}"),
    );
    Ok(())
}

#[ic_cdk::update]
fn admin_add_protocol(protocol: sc_types::Protocol) -> Result<(), ApiError> {
    let caller = require_admin()?;
    let version = protocol.version;
    catalog::add_protocol(protocol)?;
    audit(
        caller,
        "admin_add_protocol",
        &version,
        format!("added protocol v{version}"),
    );
    Ok(())
}

#[ic_cdk::update]
fn admin_set_current_protocol(version: u16) -> Result<(), ApiError> {
    let caller = require_admin()?;
    if catalog::get_protocol(version).is_none() {
        return Err(ApiError::NotFound);
    }
    config::update(|c| c.set_current_protocol_version(version))?;
    audit(
        caller,
        "admin_set_current_protocol",
        &version,
        format!("current protocol set to v{version}"),
    );
    Ok(())
}

#[ic_cdk::query]
fn admin_list_subjects(
    filter: AdminListSubjectsFilter,
    cursor: Option<u64>,
    limit: u32,
) -> Result<Vec<Subject>, ApiError> {
    require_admin()?;
    Ok(catalog::list_subjects(&filter, cursor, limit))
}

#[ic_cdk::query]
fn admin_list_protocols() -> Result<Vec<sc_types::Protocol>, ApiError> {
    require_admin()?;
    Ok(catalog::list_protocols())
}

#[ic_cdk::query]
fn get_protocol(version: u16) -> Option<sc_types::Protocol> {
    catalog::get_protocol(version)
}

#[ic_cdk::query]
fn get_subject(subject_id: u32) -> Option<Subject> {
    catalog::get_subject(subject_id)
}

#[ic_cdk::query]
fn get_lease(task_id: u64) -> Option<Lease> {
    catalog::get_lease(task_id)
}

#[ic_cdk::update]
async fn get_task() -> Result<sc_types::Task, ApiError> {
    let cfg = config::get();
    if cfg.paused.tasks {
        return Err(ApiError::Unauthorized);
    }
    let caller = ic_cdk::api::msg_caller();
    let record = registry::get_aaa(&caller).ok_or(ApiError::NotRegistered)?;
    if record.status == registry::AaaStatus::Suspended {
        return Err(ApiError::Suspended);
    }
    if record.status != registry::AaaStatus::Active
        && record.status != registry::AaaStatus::SelfManaged
    {
        return Err(ApiError::NotRegistered);
    }
    let fee = cfg.params.fee_get_task;
    if fee > 0 {
        let available = ic_cdk::api::msg_cycles_available();
        if available < fee {
            return Err(ApiError::InsufficientFee {
                required: fee.into(),
            });
        }
        ic_cdk::api::msg_cycles_accept(fee);
    }
    let now = ic_cdk::api::time();
    if record.verified_at == 0 || now.saturating_sub(record.verified_at) >= 3_600 * 1_000_000_000 {
        verify(caller).await?;
    }
    let classifications_count = 0;
    let roll = rng::next_u32();
    catalog::issue_task(
        caller,
        classifications_count,
        &cfg.params,
        cfg.current_protocol_version,
        now,
        roll,
    )
}
