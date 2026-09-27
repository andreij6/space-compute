use candid::Principal;
use ic_cdk::call::Call;
use sc_types::ApiError;

use crate::config;
use crate::forwarding::{self, is_sys_unknown};
use crate::operators::{self, Operator};
use crate::params;
use crate::record::{
    ListRecordsFilter, PageCreditCopy, PageRecord, PublicStatus, Record, RecordKind, Status,
};
use crate::repository;
use crate::roles::{self, Role};

fn require_owner() -> Result<Principal, ApiError> {
    let caller = ic_cdk::api::msg_caller();
    let owner = config::get().owner;
    roles::require_owner(caller, owner)?;
    Ok(caller)
}

#[derive(candid::CandidType, serde::Serialize)]
struct OperatorSetInput {
    operators: Vec<(Principal, Option<u64>)>,
}

#[ic_cdk::update]
async fn add_operator(
    p: Principal,
    label: String,
    expires_at: Option<u64>,
) -> Result<(), ApiError> {
    let owner = require_owner()?;
    let cfg = config::get();
    let now = ic_cdk::api::time();
    operators::add(owner, p, label, expires_at, now)?;
    let list: Vec<(Principal, Option<u64>)> = operators::list()
        .into_iter()
        .map(|(p, op)| (p, op.expires_at))
        .collect();
    let _ = Call::bounded_wait(cfg.platform_id, "sync_operators")
        .with_arg(&OperatorSetInput { operators: list })
        .change_timeout(10)
        .await;
    Ok(())
}

#[ic_cdk::update]
async fn remove_operator(p: Principal) -> Result<(), ApiError> {
    require_owner()?;
    let cfg = config::get();
    operators::remove(p)?;
    let list: Vec<(Principal, Option<u64>)> = operators::list()
        .into_iter()
        .map(|(p, op)| (p, op.expires_at))
        .collect();
    let _ = Call::bounded_wait(cfg.platform_id, "sync_operators")
        .with_arg(&OperatorSetInput { operators: list })
        .change_timeout(10)
        .await;
    Ok(())
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

#[ic_cdk::update]
async fn get_task() -> Result<sc_types::Task, ApiError> {
    let caller = ic_cdk::api::msg_caller();
    let cfg = config::get();
    let now = ic_cdk::api::time();
    let op_active = operators::is_active(&caller, now);
    forwarding::verify_caller(caller, cfg.owner, op_active)?;

    let cached = params::get();
    let balance = ic_cdk::api::canister_cycle_balance();
    if let Err(e) = forwarding::check_low_cycles(balance, cached.low_cycles_threshold()) {
        let _ = Call::bounded_wait(cfg.payments_id, "request_auto_topup")
            .with_args(&())
            .change_timeout(10)
            .await;
        return Err(e);
    }

    let mut fee = cached.fee_get_task;
    let reply = Call::bounded_wait(cfg.platform_id, "get_task")
        .with_cycles(fee)
        .change_timeout(60)
        .await;

    match reply {
        Ok(resp) => {
            let res: Result<sc_types::Task, ApiError> = resp
                .candid()
                .map_err(|e| ApiError::Internal(format!("decode get_task: {e}")))?;
            match res {
                Ok(task) => {
                    if op_active {
                        operators::touch(&caller, now);
                    }
                    repository::update_stats(|s| s.last_activity_at = now);
                    repository::remember_subject(task.task_id, task.subject.clone());
                    Ok(task)
                }
                Err(ApiError::InsufficientFee { required }) => {
                    fee = required.0.try_into().unwrap_or(fee);
                    params::update_fee_get_task(fee);
                    let retry = Call::bounded_wait(cfg.platform_id, "get_task")
                        .with_cycles(fee)
                        .change_timeout(60)
                        .await
                        .map_err(|e| ApiError::Internal(format!("get_task retry: {e}")))?;
                    let res: Result<sc_types::Task, ApiError> = retry
                        .candid()
                        .map_err(|e| ApiError::Internal(format!("decode get_task retry: {e}")))?;
                    if let Ok(ref task) = res {
                        if op_active {
                            operators::touch(&caller, now);
                        }
                        repository::update_stats(|s| s.last_activity_at = now);
                        repository::remember_subject(task.task_id, task.subject.clone());
                    }
                    res
                }
                Err(e) => Err(e),
            }
        }
        Err(e) => Err(ApiError::Internal(format!("get_task call failed: {e}"))),
    }
}

#[ic_cdk::update]
async fn submit_classification(
    submission: sc_types::ClassificationSubmission,
) -> Result<sc_types::ClassificationReceipt, ApiError> {
    let caller = ic_cdk::api::msg_caller();
    let cfg = config::get();
    let now = ic_cdk::api::time();
    let op_active = operators::is_active(&caller, now);
    forwarding::verify_caller(caller, cfg.owner, op_active)?;

    let cached = params::get();
    let balance = ic_cdk::api::canister_cycle_balance();
    if let Err(e) = forwarding::check_low_cycles(balance, cached.low_cycles_threshold()) {
        let _ = Call::bounded_wait(cfg.payments_id, "request_auto_topup")
            .with_args(&())
            .change_timeout(10)
            .await;
        return Err(e);
    }

    let sub = forwarding::prepare_classification_submission(submission, caller, cfg.agent_label);
    let mut fee = cached.fee_submit_classification;
    let subject = repository::take_subject(sub.task_id);

    let call_res = Call::bounded_wait(cfg.platform_id, "submit_classification")
        .with_arg(&sub)
        .with_cycles(fee)
        .change_timeout(60)
        .await;

    let resp = match call_res {
        Ok(r) => r,
        Err(e) => {
            if is_sys_unknown(&e) {
                Call::bounded_wait(cfg.platform_id, "submit_classification")
                    .with_arg(&sub)
                    .with_cycles(fee)
                    .change_timeout(60)
                    .await
                    .map_err(|e| {
                        ApiError::Internal(format!("submit_classification retry failed: {e}"))
                    })?
            } else {
                return Err(ApiError::Internal(format!(
                    "submit_classification failed: {e}"
                )));
            }
        }
    };

    let res: Result<sc_types::ClassificationReceipt, ApiError> = resp
        .candid()
        .map_err(|e| ApiError::Internal(format!("decode submit_classification: {e}")))?;

    match res {
        Ok(receipt) => {
            if repository::get_seq_by_task(sub.task_id).is_none() {
                let record = Record {
                    v: 1,
                    seq: 0,
                    at: now,
                    kind: if sub.discovery.is_some() {
                        RecordKind::Discovery
                    } else {
                        RecordKind::Classification
                    },
                    task_or_assignment_id: Some(sub.task_id),
                    subject: subject.clone(),
                    answers: sub.answers,
                    discovery_public_id: receipt.discovery_id.clone(),
                    category: sub.discovery.as_ref().map(|d| d.category.clone()),
                    vote: None,
                    rationale: sub.discovery.as_ref().map(|d| d.rationale.clone()),
                    outcome: None,
                    xp_awarded: receipt.xp_awarded,
                    agent_label: sub.agent_label,
                    fee,
                    by: caller,
                };
                repository::insert_record(record);
            }
            if op_active {
                operators::touch(&caller, now);
            }
            Ok(receipt)
        }
        Err(ApiError::InsufficientFee { required }) => {
            fee = required.0.try_into().unwrap_or(fee);
            params::update_fee_submit_classification(fee);
            let retry_resp = Call::bounded_wait(cfg.platform_id, "submit_classification")
                .with_arg(&sub)
                .with_cycles(fee)
                .change_timeout(60)
                .await
                .map_err(|e| ApiError::Internal(format!("retry submit_classification: {e}")))?;
            let retry_res: Result<sc_types::ClassificationReceipt, ApiError> = retry_resp
                .candid()
                .map_err(|e| ApiError::Internal(format!("decode retry: {e}")))?;
            if let Ok(ref receipt) = retry_res {
                if repository::get_seq_by_task(sub.task_id).is_none() {
                    let record = Record {
                        v: 1,
                        seq: 0,
                        at: now,
                        kind: if sub.discovery.is_some() {
                            RecordKind::Discovery
                        } else {
                            RecordKind::Classification
                        },
                        task_or_assignment_id: Some(sub.task_id),
                        subject,
                        answers: sub.answers,
                        discovery_public_id: receipt.discovery_id.clone(),
                        category: sub.discovery.as_ref().map(|d| d.category.clone()),
                        vote: None,
                        rationale: sub.discovery.as_ref().map(|d| d.rationale.clone()),
                        outcome: None,
                        xp_awarded: receipt.xp_awarded,
                        agent_label: sub.agent_label,
                        fee,
                        by: caller,
                    };
                    repository::insert_record(record);
                }
                if op_active {
                    operators::touch(&caller, now);
                }
            }
            retry_res
        }
        Err(e) => Err(e),
    }
}

#[ic_cdk::update]
async fn get_review_assignment() -> Result<Option<sc_types::ReviewAssignment>, ApiError> {
    let caller = ic_cdk::api::msg_caller();
    let cfg = config::get();
    let now = ic_cdk::api::time();
    let op_active = operators::is_active(&caller, now);
    forwarding::verify_caller(caller, cfg.owner, op_active)?;

    let cached = params::get();
    let balance = ic_cdk::api::canister_cycle_balance();
    if let Err(e) = forwarding::check_low_cycles(balance, cached.low_cycles_threshold()) {
        let _ = Call::bounded_wait(cfg.payments_id, "request_auto_topup")
            .with_args(&())
            .change_timeout(10)
            .await;
        return Err(e);
    }

    let mut fee = cached.fee_get_review;
    let reply = Call::bounded_wait(cfg.platform_id, "get_review_assignment")
        .with_cycles(fee)
        .change_timeout(60)
        .await;

    match reply {
        Ok(resp) => {
            let res: Result<Option<sc_types::ReviewAssignment>, ApiError> = resp
                .candid()
                .map_err(|e| ApiError::Internal(format!("decode get_review_assignment: {e}")))?;
            match res {
                Ok(assignment) => {
                    if op_active {
                        operators::touch(&caller, now);
                    }
                    repository::update_stats(|s| s.last_activity_at = now);
                    if let Some(ref a) = assignment {
                        repository::remember_subject(a.assignment_id, a.subject.clone());
                    }
                    Ok(assignment)
                }
                Err(ApiError::InsufficientFee { required }) => {
                    fee = required.0.try_into().unwrap_or(fee);
                    params::update_fee_get_review(fee);
                    let retry = Call::bounded_wait(cfg.platform_id, "get_review_assignment")
                        .with_cycles(fee)
                        .change_timeout(60)
                        .await
                        .map_err(|e| {
                            ApiError::Internal(format!("get_review_assignment retry: {e}"))
                        })?;
                    let res: Result<Option<sc_types::ReviewAssignment>, ApiError> =
                        retry.candid().map_err(|e| {
                            ApiError::Internal(format!("decode get_review_assignment retry: {e}"))
                        })?;
                    if let Ok(ref assignment) = res {
                        if op_active {
                            operators::touch(&caller, now);
                        }
                        repository::update_stats(|s| s.last_activity_at = now);
                        if let Some(a) = assignment {
                            repository::remember_subject(a.assignment_id, a.subject.clone());
                        }
                    }
                    res
                }
                Err(e) => Err(e),
            }
        }
        Err(e) => Err(ApiError::Internal(format!(
            "get_review_assignment call failed: {e}"
        ))),
    }
}

#[ic_cdk::update]
async fn submit_review(
    submission: sc_types::ReviewSubmission,
) -> Result<sc_types::ReviewReceipt, ApiError> {
    let caller = ic_cdk::api::msg_caller();
    let cfg = config::get();
    let now = ic_cdk::api::time();
    let op_active = operators::is_active(&caller, now);
    forwarding::verify_caller(caller, cfg.owner, op_active)?;

    let cached = params::get();
    let balance = ic_cdk::api::canister_cycle_balance();
    if let Err(e) = forwarding::check_low_cycles(balance, cached.low_cycles_threshold()) {
        let _ = Call::bounded_wait(cfg.payments_id, "request_auto_topup")
            .with_args(&())
            .change_timeout(10)
            .await;
        return Err(e);
    }

    let sub = forwarding::prepare_review_submission(submission, caller, cfg.agent_label);
    let mut fee = cached.fee_submit_review;
    let subject = repository::take_subject(sub.assignment_id);

    let call_res = Call::bounded_wait(cfg.platform_id, "submit_review")
        .with_arg(&sub)
        .with_cycles(fee)
        .change_timeout(60)
        .await;

    let resp = match call_res {
        Ok(r) => r,
        Err(e) => {
            if is_sys_unknown(&e) {
                Call::bounded_wait(cfg.platform_id, "submit_review")
                    .with_arg(&sub)
                    .with_cycles(fee)
                    .change_timeout(60)
                    .await
                    .map_err(|e| ApiError::Internal(format!("submit_review retry failed: {e}")))?
            } else {
                return Err(ApiError::Internal(format!("submit_review failed: {e}")));
            }
        }
    };

    let res: Result<sc_types::ReviewReceipt, ApiError> = resp
        .candid()
        .map_err(|e| ApiError::Internal(format!("decode submit_review: {e}")))?;

    match res {
        Ok(receipt) => {
            if repository::get_seq_by_review(sub.assignment_id).is_none() {
                let record = Record {
                    v: 1,
                    seq: 0,
                    at: now,
                    kind: RecordKind::Review,
                    task_or_assignment_id: Some(sub.assignment_id),
                    subject: subject.clone(),
                    answers: Vec::new(),
                    discovery_public_id: None,
                    category: None,
                    vote: Some(sub.vote),
                    rationale: Some(sub.rationale),
                    outcome: None,
                    xp_awarded: receipt.xp_awarded,
                    agent_label: sub.agent_label,
                    fee,
                    by: caller,
                };
                repository::insert_record(record);
            }
            if op_active {
                operators::touch(&caller, now);
            }
            Ok(receipt)
        }
        Err(ApiError::InsufficientFee { required }) => {
            fee = required.0.try_into().unwrap_or(fee);
            params::update_fee_submit_review(fee);
            let retry_resp = Call::bounded_wait(cfg.platform_id, "submit_review")
                .with_arg(&sub)
                .with_cycles(fee)
                .change_timeout(60)
                .await
                .map_err(|e| ApiError::Internal(format!("retry submit_review: {e}")))?;
            let retry_res: Result<sc_types::ReviewReceipt, ApiError> = retry_resp
                .candid()
                .map_err(|e| ApiError::Internal(format!("decode retry: {e}")))?;
            if let Ok(ref receipt) = retry_res {
                if repository::get_seq_by_review(sub.assignment_id).is_none() {
                    let record = Record {
                        v: 1,
                        seq: 0,
                        at: now,
                        kind: RecordKind::Review,
                        task_or_assignment_id: Some(sub.assignment_id),
                        subject,
                        answers: Vec::new(),
                        discovery_public_id: None,
                        category: None,
                        vote: Some(sub.vote),
                        rationale: Some(sub.rationale),
                        outcome: None,
                        xp_awarded: receipt.xp_awarded,
                        agent_label: sub.agent_label,
                        fee,
                        by: caller,
                    };
                    repository::insert_record(record);
                }
                if op_active {
                    operators::touch(&caller, now);
                }
            }
            retry_res
        }
        Err(e) => Err(e),
    }
}

#[ic_cdk::query]
fn get_record(seq: u64) -> Result<Option<Record>, ApiError> {
    let caller = ic_cdk::api::msg_caller();
    let cfg = config::get();
    let now = ic_cdk::api::time();
    let op_active = operators::is_active(&caller, now);
    forwarding::verify_caller(caller, cfg.owner, op_active)?;
    Ok(repository::get_record(seq))
}

#[ic_cdk::query]
fn list_records(filter: ListRecordsFilter) -> Result<PageRecord, ApiError> {
    let caller = ic_cdk::api::msg_caller();
    let cfg = config::get();
    let now = ic_cdk::api::time();
    let op_active = operators::is_active(&caller, now);
    forwarding::verify_caller(caller, cfg.owner, op_active)?;
    Ok(repository::list_records(filter))
}

#[ic_cdk::query]
fn list_credits(cursor: Option<String>, limit: u16) -> Result<PageCreditCopy, ApiError> {
    let caller = ic_cdk::api::msg_caller();
    let cfg = config::get();
    let now = ic_cdk::api::time();
    let op_active = operators::is_active(&caller, now);
    forwarding::verify_caller(caller, cfg.owner, op_active)?;
    Ok(repository::list_credits(cursor, limit))
}

#[ic_cdk::query]
fn status() -> Result<Status, ApiError> {
    let caller = ic_cdk::api::msg_caller();
    let cfg = config::get();
    let now = ic_cdk::api::time();
    let op_active = operators::is_active(&caller, now);
    forwarding::verify_caller(caller, cfg.owner, op_active)?;

    let cached = params::get();
    let balance = ic_cdk::api::canister_cycle_balance();
    let stats = repository::get_stats();
    let fuel_days =
        forwarding::calculate_fuel_days(balance, cached.freezing_reserve, stats.burn_ema_daily);

    Ok(Status {
        cycles: balance,
        days_of_fuel_estimate: fuel_days,
        operators: operators::list(),
        stats,
        version: sc_types::build_version("aaa"),
        wasm_version: cfg.wasm_version,
    })
}

#[ic_cdk::query]
fn status_public() -> PublicStatus {
    let cfg = config::get();
    let stats = repository::get_stats();
    PublicStatus {
        name: cfg.name,
        version: sc_types::build_version("aaa"),
        owner: cfg.owner,
        last_activity_at: stats.last_activity_at,
    }
}

#[ic_cdk::query]
fn get_api_doc() -> String {
    crate::doc::get_api_doc().to_string()
}
