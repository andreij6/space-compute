use candid::{CandidType, Nat, Principal};
use ic_cdk::call::Call;
use sc_types::ApiError;
use serde::Deserialize;

use crate::audit::{self, AuditEntry};
use crate::cmc;
use crate::config::{self, Features, Params, PauseFlags};
use crate::deposit::{self, Purpose};
use crate::guard::{self, CallerGuard};
use crate::journal::{self, Account, NotifiedInfo, Op, OpKind, OpState, PayPath};
use crate::ledger;
use crate::owners;
use crate::platform_client;
use crate::quote::{self, Quote};
use crate::rate::{self, RateCache};

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
            "btc={} eth={} sponsored_spawn={}",
            features.btc, features.eth, features.sponsored_spawn
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

#[ic_cdk::query]
fn get_rate() -> RateCache {
    rate::get()
}

#[ic_cdk::update]
async fn admin_refresh_rate() -> Result<RateCache, ApiError> {
    require_admin()?;
    rate::refresh().await
}

#[ic_cdk::query]
fn get_quote_spawn() -> Result<Quote, ApiError> {
    let params = config::get().params;
    let now_secs = ic_cdk::api::time() / 1_000_000_000;
    quote::quote_spawn(&params, &rate::get(), now_secs)
}

#[ic_cdk::query]
fn get_quote_topup(cycles: u128) -> Result<Quote, ApiError> {
    let params = config::get().params;
    let now_secs = ic_cdk::api::time() / 1_000_000_000;
    quote::quote_topup(cycles, &rate::get(), now_secs, params.icp_ledger_fee_e8s)
}

#[ic_cdk::query]
fn get_deposit_account(purpose: Purpose, beneficiary: Principal) -> (String, Account) {
    deposit::deposit_account(ic_cdk::api::canister_self(), purpose, beneficiary)
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

#[ic_cdk::update]
fn admin_set_platform_id(p: Principal) -> Result<(), ApiError> {
    let caller = require_admin()?;
    config::update(|c| c.set_platform_id(p))?;
    audit(
        caller,
        "admin_set_platform_id",
        &p,
        format!("platform_id set to {p}"),
    );
    Ok(())
}

fn nat_u64(n: Nat) -> u64 {
    u64::try_from(n.0).unwrap_or(u64::MAX)
}

async fn platform_aaa_by_owner(
    platform: Principal,
    owner: Principal,
) -> Result<Option<Principal>, ApiError> {
    let reply = Call::bounded_wait(platform, "aaa_by_owner")
        .with_arg(owner)
        .await
        .map_err(|e| ApiError::Internal(format!("platform.aaa_by_owner: {e:?}")))?;
    reply
        .candid()
        .map_err(|e| ApiError::Internal(format!("aaa_by_owner decode: {e:?}")))
}

async fn platform_check_name(
    platform: Principal,
    name: String,
) -> Result<platform_client::CheckNameResult, ApiError> {
    let reply = Call::bounded_wait(platform, "check_name")
        .with_arg(name)
        .await
        .map_err(|e| ApiError::Internal(format!("platform.check_name: {e:?}")))?;
    reply
        .candid()
        .map_err(|e| ApiError::Internal(format!("check_name decode: {e:?}")))
}

async fn platform_register_aaa(
    platform: Principal,
    args: platform_client::RegisterArgs,
) -> Result<(), ApiError> {
    let reply = Call::bounded_wait(platform, "register_aaa")
        .with_arg(args)
        .await
        .map_err(|e| ApiError::Internal(format!("platform.register_aaa: {e:?}")))?;
    reply
        .candid::<Result<(), ApiError>>()
        .map_err(|e| ApiError::Internal(format!("register_aaa decode: {e:?}")))?
}

async fn ledger_balance(account: Account) -> Result<u64, ApiError> {
    let reply = Call::bounded_wait(rate::ledger_id(), "icrc1_balance_of")
        .with_arg(account)
        .await
        .map_err(|e| ApiError::Internal(format!("icrc1_balance_of: {e:?}")))?;
    let n: Nat = reply
        .candid()
        .map_err(|e| ApiError::Internal(format!("balance decode: {e:?}")))?;
    Ok(nat_u64(n))
}

async fn ledger_transfer_once(
    from_subaccount: Option<[u8; 32]>,
    to: Account,
    amount: u64,
    fee: u64,
    memo: Vec<u8>,
    created_at: u64,
) -> Result<Result<u64, ledger::TransferError>, ApiError> {
    let arg = ledger::TransferArg {
        from_subaccount,
        to,
        fee: Some(Nat::from(fee)),
        created_at_time: Some(created_at),
        memo: Some(memo),
        amount: Nat::from(amount),
    };
    let reply = Call::bounded_wait(rate::ledger_id(), "icrc1_transfer")
        .with_arg(arg)
        .await
        .map_err(|e| ApiError::Internal(format!("icrc1_transfer: {e:?}")))?;
    let decoded: Result<Nat, ledger::TransferError> = reply
        .candid()
        .map_err(|e| ApiError::Internal(format!("icrc1_transfer decode: {e:?}")))?;
    Ok(decoded.map(nat_u64))
}

async fn ledger_transfer(
    from_subaccount: Option<[u8; 32]>,
    to: Account,
    amount: u64,
    fee: u64,
    memo: Vec<u8>,
    created_at: u64,
) -> Result<u64, ApiError> {
    match ledger_transfer_once(
        from_subaccount,
        to.clone(),
        amount,
        fee,
        memo.clone(),
        created_at,
    )
    .await?
    {
        Ok(block) => Ok(block),
        Err(ledger::TransferError::Duplicate { duplicate_of }) => Ok(nat_u64(duplicate_of)),
        Err(ref e) if ledger::expected_fee(e).is_some() => {
            let corrected = ledger::expected_fee(e).unwrap();
            match ledger_transfer_once(from_subaccount, to, amount, corrected, memo, created_at)
                .await?
            {
                Ok(block) => Ok(block),
                Err(e2) => Err(ApiError::Internal(format!(
                    "icrc1_transfer rejected: {e2:?}"
                ))),
            }
        }
        Err(e) => Err(ApiError::Internal(format!(
            "icrc1_transfer rejected: {e:?}"
        ))),
    }
}

async fn ledger_transfer_from_once(
    spender_subaccount: Option<[u8; 32]>,
    from: Account,
    to: Account,
    amount: u64,
    fee: u64,
    memo: Vec<u8>,
    created_at: u64,
) -> Result<Result<u64, ledger::TransferFromError>, ApiError> {
    let arg = ledger::TransferFromArg {
        spender_subaccount,
        from,
        to,
        amount: Nat::from(amount),
        fee: Some(Nat::from(fee)),
        memo: Some(memo),
        created_at_time: Some(created_at),
    };
    let reply = Call::bounded_wait(rate::ledger_id(), "icrc2_transfer_from")
        .with_arg(arg)
        .await
        .map_err(|e| ApiError::Internal(format!("icrc2_transfer_from: {e:?}")))?;
    let decoded: Result<Nat, ledger::TransferFromError> = reply
        .candid()
        .map_err(|e| ApiError::Internal(format!("icrc2_transfer_from decode: {e:?}")))?;
    Ok(decoded.map(nat_u64))
}

async fn ledger_transfer_from(
    spender_subaccount: Option<[u8; 32]>,
    from: Account,
    to: Account,
    amount: u64,
    fee: u64,
    memo: Vec<u8>,
    created_at: u64,
) -> Result<u64, ApiError> {
    match ledger_transfer_from_once(
        spender_subaccount,
        from.clone(),
        to.clone(),
        amount,
        fee,
        memo.clone(),
        created_at,
    )
    .await?
    {
        Ok(block) => Ok(block),
        Err(ledger::TransferFromError::Duplicate { duplicate_of }) => Ok(nat_u64(duplicate_of)),
        Err(ref e) if ledger::expected_fee_from(e).is_some() => {
            let corrected = ledger::expected_fee_from(e).unwrap();
            match ledger_transfer_from_once(
                spender_subaccount,
                from,
                to,
                amount,
                corrected,
                memo,
                created_at,
            )
            .await?
            {
                Ok(block) => Ok(block),
                Err(e2) => Err(ApiError::Internal(format!(
                    "icrc2_transfer_from rejected: {e2:?}"
                ))),
            }
        }
        Err(e) => Err(ApiError::Internal(format!(
            "icrc2_transfer_from rejected: {e:?}"
        ))),
    }
}

async fn cmc_notify_create(
    block_index: u64,
    controller: Principal,
    controllers: Vec<Principal>,
) -> Result<Result<Principal, cmc::CmcNotifyError>, ApiError> {
    let reply = Call::bounded_wait(rate::cmc_id(), "notify_create_canister")
        .with_arg(cmc::NotifyCreateCanisterArg {
            block_index,
            controller,
            subnet_type: None,
            subnet_selection: None,
            settings: Some(cmc::CanisterSettingsArgs {
                controllers: Some(controllers),
                freezing_threshold: Some(Nat::from(cmc::SIXTY_DAYS_SECS)),
            }),
        })
        .await
        .map_err(|e| ApiError::Internal(format!("notify_create_canister: {e:?}")))?;
    reply
        .candid::<Result<Principal, cmc::CmcNotifyError>>()
        .map_err(|e| ApiError::Internal(format!("notify_create_canister decode: {e:?}")))
}

async fn pull_or_sweep_spawn(op: &Op) -> Result<u64, ApiError> {
    let (owner, _, _) = op.spawn_fields()?;
    let params = config::get().params;
    let self_id = ic_cdk::api::canister_self();
    let to = cmc::deposit_account(rate::cmc_id(), self_id);
    let now = ic_cdk::api::time();
    let memo = cmc::MEMO_CREATE.to_le_bytes().to_vec();
    match &op.path {
        PayPath::Deposit => {
            let sub = deposit::deposit_subaccount(Purpose::Spawn, owner);
            let from_account = Account {
                owner: self_id,
                subaccount: Some(sub),
            };
            let balance = ledger_balance(from_account).await?;
            let min_required = op.amount_e8s.saturating_add(params.icp_ledger_fee_e8s);
            let amount = deposit::sweep_amount(balance, min_required, params.icp_ledger_fee_e8s)?;
            ledger_transfer(Some(sub), to, amount, params.icp_ledger_fee_e8s, memo, now).await
        }
        PayPath::Wallet { payer } => {
            let sub = deposit::spender_subaccount(Purpose::Spawn, owner);
            ledger_transfer_from(
                Some(sub),
                payer.clone(),
                to,
                op.amount_e8s,
                params.icp_ledger_fee_e8s,
                memo,
                now,
            )
            .await
        }
        PayPath::Treasury | PayPath::Invite { .. } => Err(ApiError::Internal(
            "spawn does not support this path here".into(),
        )),
    }
}

async fn advance_spawn_saga(op_id: u64, platform: Principal) -> Result<(), ApiError> {
    loop {
        let op = journal::get(op_id).ok_or(ApiError::NotFound)?;
        if op.state.is_terminal() {
            return Ok(());
        }
        match op.state.clone() {
            OpState::Pending => {
                let block = pull_or_sweep_spawn(&op).await?;
                journal::advance(op_id, OpState::Pulled { block }, ic_cdk::api::time())?;
            }
            OpState::Pulled { block } => {
                let (owner, _, _) = op.spawn_fields()?;
                let self_id = ic_cdk::api::canister_self();
                match cmc_notify_create(block, self_id, vec![owner, platform]).await? {
                    Ok(canister_id) => {
                        owners::record(owner, canister_id);
                        journal::advance(
                            op_id,
                            OpState::Notified {
                                canister_or_cycles: NotifiedInfo::Canister(canister_id),
                            },
                            ic_cdk::api::time(),
                        )?;
                    }
                    Err(cmc::CmcNotifyError::Refunded { block_index, .. }) => {
                        journal::advance(
                            op_id,
                            OpState::Refunded {
                                block: block_index.unwrap_or(block),
                            },
                            ic_cdk::api::time(),
                        )?;
                        return Err(ApiError::Internal(
                            "the CMC refunded the create-canister transfer".into(),
                        ));
                    }
                    Err(e) => {
                        return Err(ApiError::Internal(format!("notify_create_canister: {e}")))
                    }
                }
            }
            OpState::Notified {
                canister_or_cycles: NotifiedInfo::Canister(canister_id),
            } => {
                let (owner, name, avatar_seed) = op.spawn_fields()?;
                platform_register_aaa(
                    platform,
                    platform_client::RegisterArgs {
                        canister_id,
                        owner,
                        name,
                        avatar_seed,
                    },
                )
                .await?;
                journal::advance(op_id, OpState::Registered, ic_cdk::api::time())?;
            }
            OpState::Notified { .. } => {
                return Err(ApiError::Internal(
                    "spawn op notified with cycles instead of a canister".into(),
                ));
            }
            OpState::Registered => {
                journal::advance(op_id, OpState::Done, ic_cdk::api::time())?;
            }
            _ => return Err(ApiError::Internal("unexpected spawn op state".into())),
        }
    }
}

#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct SpawnArgs {
    pub name: String,
    pub avatar_seed: u64,
    pub path: PayPath,
}

#[ic_cdk::update]
async fn spawn_aaa(args: SpawnArgs) -> Result<u64, ApiError> {
    let caller = ic_cdk::api::msg_caller();
    if caller == Principal::anonymous() {
        return Err(ApiError::Unauthorized);
    }
    if let PayPath::Invite { .. } = args.path {
        return Err(ApiError::Internal(
            "the invite spawn path is not implemented yet (T5.16)".into(),
        ));
    }
    let _guard = CallerGuard::acquire(guard::key("spawn", caller))?;
    let platform = config::platform_id()?;

    let existing = platform_aaa_by_owner(platform, caller).await?;
    platform_client::evaluate_owner_lookup(existing)?;
    let check = platform_check_name(platform, args.name.clone()).await?;
    platform_client::evaluate_check_name(check)?;

    let params = config::get().params;
    let now = ic_cdk::api::time();
    let quote = quote::quote_spawn(&params, &rate::get(), now / 1_000_000_000)?;

    let op = journal::create(
        OpKind::Spawn {
            owner: caller,
            name: args.name,
            avatar_seed: args.avatar_seed,
        },
        args.path,
        quote.deposit_e8s,
        caller,
        now,
    );
    advance_spawn_saga(op.id, platform).await?;
    Ok(op.id)
}

#[ic_cdk::update]
pub(crate) async fn resume(op_id: u64) -> Result<(), ApiError> {
    let op = journal::get(op_id).ok_or(ApiError::NotFound)?;
    if op.state.is_terminal() {
        return Ok(());
    }
    let owner = match &op.kind {
        OpKind::Spawn { owner, .. } => *owner,
        _ => {
            return Err(ApiError::Internal(
                "resume: only Spawn ops are implemented in T5.3".into(),
            ))
        }
    };
    let _guard = CallerGuard::acquire(guard::key("spawn", owner))?;
    let platform = config::platform_id()?;
    advance_spawn_saga(op_id, platform).await
}
