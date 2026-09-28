use candid::{CandidType, Nat, Principal};
use ic_cdk::call::Call;
use sc_types::ApiError;
use serde::Deserialize;

use crate::audit::{self, AuditEntry};
use crate::cmc;
use crate::config::{self, Features, Params, PauseFlags};
use crate::deposit::{self, Purpose};
use crate::guard::{self, CallerGuard};
use crate::invites;
use crate::journal::{self, Account, NotifiedInfo, Op, OpFilter, OpKind, OpState, Page, PayPath};
use crate::ledger;
use crate::mandate::{self, MandateView};
use crate::owners;
use crate::platform_client;
use crate::quote::{self, Quote};
use crate::rate::{self, RateCache};

#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct PaymentsOverview {
    pub admins: Vec<Principal>,
    pub params: Params,
    pub features: Features,
    pub paused: PauseFlags,
    pub cycles: u128,
    pub audit_entries: u64,
    pub main_icp_balance_e8s: u64,
    pub treasury_icp_balance_e8s: u64,
    pub ops_24h_by_kind: Vec<(String, u64, u64)>,
    pub failed_ops: u64,
    pub stuck_ops: u64,
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

#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct MintInvitesArgs {
    pub count: u32,
    pub sponsor_cycles: u128,
    pub expires_at: u64,
}

#[ic_cdk::update]
async fn admin_mint_invites(args: MintInvitesArgs) -> Result<Vec<String>, ApiError> {
    let caller = require_admin()?;
    let seed = raw_rand().await?;
    let now_secs = ic_cdk::api::time() / 1_000_000_000;
    let codes = invites::mint(
        args.count,
        args.sponsor_cycles,
        args.expires_at,
        &seed,
        now_secs,
    )?;
    audit(
        caller,
        "admin_mint_invites",
        &(args.count, args.sponsor_cycles, args.expires_at),
        format!(
            "minted {} invite(s), sponsor_cycles={} expires_at={}",
            args.count, args.sponsor_cycles, args.expires_at
        ),
    );
    Ok(codes)
}

#[ic_cdk::query]
fn get_treasury_account() -> (String, Account) {
    deposit::treasury_account(ic_cdk::api::canister_self())
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
fn admin_audit_log(cursor: Option<u64>, limit: u32) -> Result<Page<AuditEntry>, ApiError> {
    require_admin()?;
    Ok(audit::page(cursor, limit))
}

#[ic_cdk::query]
fn admin_list_ops(filter: OpFilter, cursor: Option<u64>, limit: u32) -> Result<Page<Op>, ApiError> {
    require_admin()?;
    Ok(journal::list_filtered(&filter, cursor, limit))
}

#[ic_cdk::query]
fn admin_list_invites(
    cursor: Option<Vec<u8>>,
    limit: u32,
) -> Result<invites::InvitePage, ApiError> {
    require_admin()?;
    Ok(invites::list(cursor, limit))
}

const OVERVIEW_WINDOW_SECS: u64 = 24 * 3_600;

#[ic_cdk::update]
async fn admin_overview() -> Result<PaymentsOverview, ApiError> {
    require_admin()?;
    let c = config::get();
    let self_id = ic_cdk::api::canister_self();
    let now = ic_cdk::api::time();
    let main_account = Account {
        owner: self_id,
        subaccount: None,
    };
    let treasury_account = Account {
        owner: self_id,
        subaccount: Some(deposit::treasury_subaccount()),
    };
    let main_icp_balance_e8s = ledger_balance(main_account).await?;
    let treasury_icp_balance_e8s = ledger_balance(treasury_account).await?;
    Ok(PaymentsOverview {
        admins: c.admins,
        params: c.params,
        features: c.features,
        paused: c.paused,
        cycles: ic_cdk::api::canister_cycle_balance(),
        audit_entries: audit::len(),
        main_icp_balance_e8s,
        treasury_icp_balance_e8s,
        ops_24h_by_kind: journal::stats_since(now, OVERVIEW_WINDOW_SECS),
        failed_ops: journal::failed_count(),
        stuck_ops: journal::stuck_count(now, journal::STUCK_AFTER_SECS),
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
fn list_ops_for_aaa(aaa: Principal, cursor: Option<u64>, limit: u16) -> Page<Op> {
    journal::list_for_aaa(aaa, cursor, u32::from(limit))
}

#[ic_cdk::query]
fn list_ops_for_owner(owner: Principal, cursor: Option<u64>, limit: u16) -> Page<Op> {
    journal::list_for_owner(owner, cursor, u32::from(limit))
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

async fn platform_aaa_owner(
    platform: Principal,
    aaa: Principal,
) -> Result<Option<Principal>, ApiError> {
    let reply = Call::bounded_wait(platform, "aaa_owner")
        .with_arg(aaa)
        .await
        .map_err(|e| ApiError::Internal(format!("platform.aaa_owner: {e:?}")))?;
    reply
        .candid()
        .map_err(|e| ApiError::Internal(format!("aaa_owner decode: {e:?}")))
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

async fn icrc1_transfer_once(
    from_subaccount: Option<[u8; 32]>,
    to: Account,
    amount: u64,
    fee: u64,
    memo: Vec<u8>,
    created_at: u64,
) -> ledger::Outcome {
    let arg = ledger::TransferArg {
        from_subaccount,
        to,
        fee: Some(Nat::from(fee)),
        created_at_time: Some(created_at),
        memo: Some(memo),
        amount: Nat::from(amount),
    };
    match Call::bounded_wait(rate::ledger_id(), "icrc1_transfer")
        .with_arg(arg)
        .await
    {
        Ok(reply) => match reply.candid::<Result<Nat, ledger::TransferError>>() {
            Ok(r) => ledger::classify_transfer(r),
            Err(e) => ledger::Outcome::Unknown(format!("icrc1_transfer decode: {e:?}")),
        },
        Err(e) => ledger::Outcome::Unknown(format!("icrc1_transfer: {e:?}")),
    }
}

async fn icrc1_transfer(
    from_subaccount: Option<[u8; 32]>,
    to: Account,
    amount: u64,
    memo: Vec<u8>,
    created_at: u64,
) -> ledger::Outcome {
    let fee = config::get().params.icp_ledger_fee_e8s;
    match icrc1_transfer_once(
        from_subaccount,
        to.clone(),
        amount,
        fee,
        memo.clone(),
        created_at,
    )
    .await
    {
        ledger::Outcome::BadFee(expected) => {
            icrc1_transfer_once(from_subaccount, to, amount, expected, memo, created_at).await
        }
        outcome => outcome,
    }
}

async fn icrc2_transfer_from_once(
    spender_subaccount: [u8; 32],
    from: Account,
    to: Account,
    amount: u64,
    fee: u64,
    memo: Vec<u8>,
    created_at: u64,
) -> ledger::Outcome {
    let arg = ledger::TransferFromArg {
        spender_subaccount: Some(spender_subaccount),
        from,
        to,
        amount: Nat::from(amount),
        fee: Some(Nat::from(fee)),
        memo: Some(memo),
        created_at_time: Some(created_at),
    };
    match Call::bounded_wait(rate::ledger_id(), "icrc2_transfer_from")
        .with_arg(arg)
        .await
    {
        Ok(reply) => match reply.candid::<Result<Nat, ledger::TransferFromError>>() {
            Ok(r) => ledger::classify_transfer_from(r),
            Err(e) => ledger::Outcome::Unknown(format!("icrc2_transfer_from decode: {e:?}")),
        },
        Err(e) => ledger::Outcome::Unknown(format!("icrc2_transfer_from: {e:?}")),
    }
}

async fn icrc2_transfer_from(
    spender_subaccount: [u8; 32],
    from: Account,
    to: Account,
    amount: u64,
    memo: Vec<u8>,
    created_at: u64,
) -> ledger::Outcome {
    let fee = config::get().params.icp_ledger_fee_e8s;
    match icrc2_transfer_from_once(
        spender_subaccount,
        from.clone(),
        to.clone(),
        amount,
        fee,
        memo.clone(),
        created_at,
    )
    .await
    {
        ledger::Outcome::BadFee(expected) => {
            icrc2_transfer_from_once(
                spender_subaccount,
                from,
                to,
                amount,
                expected,
                memo,
                created_at,
            )
            .await
        }
        outcome => outcome,
    }
}

enum Pull {
    Block(u64),
    Rejected { reason: String, funding: bool },
}

fn settle(outcome: ledger::Outcome, created_at: u64) -> Result<Pull, ApiError> {
    match outcome {
        ledger::Outcome::Done(block) => Ok(Pull::Block(block)),
        ledger::Outcome::Rejected { reason, funding } => Ok(Pull::Rejected { reason, funding }),
        ledger::Outcome::BadFee(expected) => Ok(Pull::Rejected {
            reason: format!("BadFee after retry, expected {expected}"),
            funding: false,
        }),
        ledger::Outcome::Unknown(reason) => Err(ApiError::Internal(format!(
            "ledger outcome unknown ({reason}); retry reuses created_at_time {created_at}"
        ))),
    }
}

fn fail_op(op_id: u64, reason: &str) -> Result<(), ApiError> {
    journal::advance(
        op_id,
        OpState::Failed {
            reason: reason.to_string(),
        },
        ic_cdk::api::time(),
    )?;
    Ok(())
}

async fn ledger_allowance(account: Account, spender: Account) -> Result<u64, ApiError> {
    let reply = Call::bounded_wait(rate::ledger_id(), "icrc2_allowance")
        .with_arg(ledger::AllowanceArg { account, spender })
        .await
        .map_err(|e| ApiError::Internal(format!("icrc2_allowance: {e:?}")))?;
    let allowance: ledger::Allowance = reply
        .candid()
        .map_err(|e| ApiError::Internal(format!("icrc2_allowance decode: {e:?}")))?;
    Ok(nat_u64(allowance.allowance))
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

async fn cmc_notify_top_up(
    block_index: u64,
    canister_id: Principal,
) -> Result<Result<u128, cmc::CmcNotifyError>, ApiError> {
    let reply = Call::bounded_wait(rate::cmc_id(), "notify_top_up")
        .with_arg(cmc::NotifyTopUpArg {
            block_index,
            canister_id,
        })
        .await
        .map_err(|e| ApiError::Internal(format!("notify_top_up: {e:?}")))?;
    let decoded: Result<Nat, cmc::CmcNotifyError> = reply
        .candid()
        .map_err(|e| ApiError::Internal(format!("notify_top_up decode: {e:?}")))?;
    Ok(decoded.map(|n| n.0.try_into().unwrap_or(u128::MAX)))
}

fn ensure_not_paused(paused: bool, what: &str) -> Result<(), ApiError> {
    if paused {
        return Err(ApiError::NotEligible(format!("{what} is paused")));
    }
    Ok(())
}

async fn topup_pull_amount(
    aaa: Principal,
    path: &PayPath,
) -> Result<Result<u64, String>, ApiError> {
    let fee = config::get().params.icp_ledger_fee_e8s;
    let self_id = ic_cdk::api::canister_self();
    let available = match path {
        PayPath::Deposit => {
            ledger_balance(Account {
                owner: self_id,
                subaccount: Some(deposit::deposit_subaccount(Purpose::TopUp, aaa)),
            })
            .await?
        }
        PayPath::Wallet { payer } => {
            let spender = Account {
                owner: self_id,
                subaccount: Some(deposit::spender_subaccount(Purpose::TopUp, aaa)),
            };
            ledger_allowance(payer.clone(), spender).await?
        }
        PayPath::Treasury | PayPath::Invite { .. } => {
            return Ok(Err("top_up does not support this path".into()))
        }
    };
    Ok(deposit::sweep_amount(available, quote::MIN_TOPUP_E8S, fee).map_err(|e| e.to_string()))
}

async fn topup_transfer(aaa: Principal, op: &Op, amount: u64) -> ledger::Outcome {
    let to = cmc::deposit_account(rate::cmc_id(), aaa);
    let memo = cmc::MEMO_TOP_UP.to_le_bytes().to_vec();
    match &op.path {
        PayPath::Deposit => {
            let sub = deposit::deposit_subaccount(Purpose::TopUp, aaa);
            icrc1_transfer(Some(sub), to, amount, memo, op.created_at).await
        }
        PayPath::Wallet { payer } => {
            let sub = deposit::spender_subaccount(Purpose::TopUp, aaa);
            icrc2_transfer_from(sub, payer.clone(), to, amount, memo, op.created_at).await
        }
        PayPath::Treasury | PayPath::Invite { .. } => ledger::Outcome::Rejected {
            reason: "top_up does not support this path".into(),
            funding: false,
        },
    }
}

async fn advance_topup_saga(op_id: u64) -> Result<(), ApiError> {
    loop {
        let op = journal::get(op_id).ok_or(ApiError::NotFound)?;
        if op.state.is_terminal() {
            return Ok(());
        }
        let aaa = op.topup_fields()?;
        match op.state.clone() {
            OpState::Pending => {
                let amount = match op.pull_e8s {
                    Some(a) => a,
                    None => match topup_pull_amount(aaa, &op.path).await? {
                        Ok(a) => journal::fix_pull(op_id, a)?,
                        Err(reason) => {
                            fail_op(op_id, &reason)?;
                            return Err(ApiError::invalid(reason));
                        }
                    },
                };
                match settle(topup_transfer(aaa, &op, amount).await, op.created_at)? {
                    Pull::Block(block) => {
                        journal::advance(op_id, OpState::Pulled { block }, ic_cdk::api::time())?;
                    }
                    Pull::Rejected { reason, .. } => {
                        fail_op(op_id, &reason)?;
                        return Err(ApiError::Internal(format!(
                            "top-up pull rejected: {reason}"
                        )));
                    }
                }
            }
            OpState::Pulled { block } => match cmc_notify_top_up(block, aaa).await? {
                Ok(cycles) => {
                    journal::advance(
                        op_id,
                        OpState::Notified {
                            canister_or_cycles: NotifiedInfo::Cycles(cycles),
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
                        "the CMC refunded the top-up transfer".into(),
                    ));
                }
                Err(e) => return Err(ApiError::Internal(format!("notify_top_up: {e}"))),
            },
            OpState::Notified {
                canister_or_cycles: NotifiedInfo::Cycles(_),
            } => {
                journal::advance(op_id, OpState::Done, ic_cdk::api::time())?;
            }
            OpState::Notified { .. } => {
                return Err(ApiError::Internal(
                    "top-up op notified with a canister instead of cycles".into(),
                ));
            }
            _ => return Err(ApiError::Internal("unexpected top-up op state".into())),
        }
    }
}

#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct TopUpArgs {
    pub aaa: Principal,
    pub path: PayPath,
}

#[ic_cdk::update]
async fn top_up(args: TopUpArgs) -> Result<u64, ApiError> {
    let caller = ic_cdk::api::msg_caller();
    if caller == Principal::anonymous() {
        return Err(ApiError::Unauthorized);
    }
    ensure_not_paused(config::get().paused.topup, "top_up")?;
    if matches!(args.path, PayPath::Treasury | PayPath::Invite { .. }) {
        return Err(ApiError::invalid(
            "top_up only supports the Wallet or Deposit paths",
        ));
    }
    let platform = config::platform_id()?;
    let owner = platform_aaa_owner(platform, args.aaa).await?;
    if owner.is_none() {
        return Err(ApiError::NotFound);
    }
    let _guard = CallerGuard::acquire(guard::key("topup", args.aaa))?;
    let now = ic_cdk::api::time();
    let op = journal::create(OpKind::TopUp { aaa: args.aaa }, args.path, 0, caller, now);
    journal::index_aaa(args.aaa, op.id);
    journal::index_owner(caller, op.id);
    advance_topup_saga(op.id).await?;
    Ok(op.id)
}

#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct SetMandateArgs {
    pub aaa: Principal,
    pub payer: Account,
    pub topup_e8s: u64,
    pub cap_30d_e8s: u64,
    pub enabled: bool,
}

#[ic_cdk::update]
async fn set_mandate(args: SetMandateArgs) -> Result<(), ApiError> {
    let caller = ic_cdk::api::msg_caller();
    if caller == Principal::anonymous() {
        return Err(ApiError::Unauthorized);
    }
    let platform = config::platform_id()?;
    let owner = platform_aaa_owner(platform, args.aaa).await?;
    if owner != Some(caller) {
        return Err(ApiError::Unauthorized);
    }
    mandate::set(
        args.aaa,
        args.payer,
        args.topup_e8s,
        args.cap_30d_e8s,
        args.enabled,
    )?;
    Ok(())
}

#[ic_cdk::query]
fn get_mandate(aaa: Principal) -> Option<MandateView> {
    mandate::view(aaa, ic_cdk::api::time())
}

async fn advance_auto_topup_saga(op_id: u64) -> Result<(), ApiError> {
    loop {
        let op = journal::get(op_id).ok_or(ApiError::NotFound)?;
        if op.state.is_terminal() {
            return Ok(());
        }
        let aaa = op.auto_topup_fields()?;
        match op.state.clone() {
            OpState::Pending => {
                let PayPath::Wallet { payer } = op.path.clone() else {
                    return Err(ApiError::Internal("auto top-up op without a payer".into()));
                };
                let outcome = icrc2_transfer_from(
                    deposit::spender_subaccount(Purpose::Auto, aaa),
                    payer,
                    cmc::deposit_account(rate::cmc_id(), aaa),
                    op.amount_e8s,
                    cmc::MEMO_TOP_UP.to_le_bytes().to_vec(),
                    op.created_at,
                )
                .await;
                match settle(outcome, op.created_at)? {
                    Pull::Block(block) => {
                        journal::advance(op_id, OpState::Pulled { block }, ic_cdk::api::time())?;
                    }
                    Pull::Rejected { reason, funding } => {
                        fail_op(op_id, &reason)?;
                        mandate::release(aaa, op_id);
                        if funding {
                            mandate::mark_needs_attention(aaa, true)?;
                            return Err(ApiError::invalid(format!(
                                "auto top-up needs attention: {reason}"
                            )));
                        }
                        return Err(ApiError::Internal(format!(
                            "auto top-up pull rejected: {reason}"
                        )));
                    }
                }
            }
            OpState::Pulled { block } => match cmc_notify_top_up(block, aaa).await? {
                Ok(cycles) => {
                    journal::advance(
                        op_id,
                        OpState::Notified {
                            canister_or_cycles: NotifiedInfo::Cycles(cycles),
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
                    mandate::release(aaa, op_id);
                    return Err(ApiError::Internal(
                        "the CMC refunded the auto top-up transfer".into(),
                    ));
                }
                Err(e) => return Err(ApiError::Internal(format!("notify_top_up: {e}"))),
            },
            OpState::Notified {
                canister_or_cycles: NotifiedInfo::Cycles(_),
            } => {
                journal::advance(op_id, OpState::Done, ic_cdk::api::time())?;
                mandate::mark_needs_attention(aaa, false)?;
            }
            OpState::Notified { .. } => {
                return Err(ApiError::Internal(
                    "auto top-up op notified with a canister instead of cycles".into(),
                ));
            }
            _ => return Err(ApiError::Internal("unexpected auto top-up op state".into())),
        }
    }
}

#[ic_cdk::update]
async fn request_auto_topup() -> Result<u64, ApiError> {
    let aaa = ic_cdk::api::msg_caller();
    if aaa == Principal::anonymous() {
        return Err(ApiError::Unauthorized);
    }
    ensure_not_paused(config::get().paused.auto_topup, "auto top-up")?;
    let _guard = CallerGuard::acquire(guard::key("auto", aaa))?;
    let mandate = mandate::get(aaa).ok_or(ApiError::NotFound)?;
    let now = ic_cdk::api::time();
    let params = config::get().params;
    mandate::check_eligible(&mandate, now, params.auto_topup_min_interval_secs)?;

    let op = journal::create(
        OpKind::AutoTopUp { aaa },
        PayPath::Wallet {
            payer: mandate.payer.clone(),
        },
        mandate.topup_e8s,
        aaa,
        now,
    );
    journal::index_aaa(aaa, op.id);
    mandate::reserve(aaa, op.id, now, mandate.topup_e8s)?;
    advance_auto_topup_saga(op.id).await?;
    Ok(op.id)
}

async fn spawn_pull_amount(op: &Op, owner: Principal) -> Result<Result<u64, String>, ApiError> {
    match &op.path {
        PayPath::Deposit => {
            let fee = config::get().params.icp_ledger_fee_e8s;
            let balance = ledger_balance(Account {
                owner: ic_cdk::api::canister_self(),
                subaccount: Some(deposit::deposit_subaccount(Purpose::Spawn, owner)),
            })
            .await?;
            let min_required = op.amount_e8s.saturating_add(fee);
            Ok(deposit::sweep_amount(balance, min_required, fee).map_err(|e| e.to_string()))
        }
        PayPath::Wallet { .. } | PayPath::Invite { .. } => Ok(Ok(op.amount_e8s)),
        PayPath::Treasury => Ok(Err("spawn does not support the Treasury path".into())),
    }
}

async fn spawn_transfer(op: &Op, owner: Principal, amount: u64) -> ledger::Outcome {
    let to = cmc::deposit_account(rate::cmc_id(), ic_cdk::api::canister_self());
    let memo = cmc::MEMO_CREATE.to_le_bytes().to_vec();
    match &op.path {
        PayPath::Deposit => {
            let sub = deposit::deposit_subaccount(Purpose::Spawn, owner);
            icrc1_transfer(Some(sub), to, amount, memo, op.created_at).await
        }
        PayPath::Wallet { payer } => {
            let sub = deposit::spender_subaccount(Purpose::Spawn, owner);
            icrc2_transfer_from(sub, payer.clone(), to, amount, memo, op.created_at).await
        }
        PayPath::Invite { .. } => {
            let sub = deposit::treasury_subaccount();
            icrc1_transfer(Some(sub), to, amount, memo, op.created_at).await
        }
        PayPath::Treasury => ledger::Outcome::Rejected {
            reason: "spawn does not support the Treasury path".into(),
            funding: false,
        },
    }
}

fn fail_spawn(op: &Op, owner: Principal, reason: &str) -> Result<(), ApiError> {
    fail_op(op.id, reason)?;
    if matches!(op.path, PayPath::Invite { .. }) {
        invites::unmark_sponsored(owner);
    }
    Ok(())
}

async fn advance_spawn_saga(op_id: u64, platform: Principal) -> Result<(), ApiError> {
    loop {
        let op = journal::get(op_id).ok_or(ApiError::NotFound)?;
        if op.state.is_terminal() {
            return Ok(());
        }
        match op.state.clone() {
            OpState::Pending => {
                let (owner, _, _) = op.spawn_fields()?;
                let amount = match op.pull_e8s {
                    Some(a) => a,
                    None => match spawn_pull_amount(&op, owner).await? {
                        Ok(a) => journal::fix_pull(op_id, a)?,
                        Err(reason) => {
                            fail_spawn(&op, owner, &reason)?;
                            return Err(ApiError::invalid(reason));
                        }
                    },
                };
                match settle(spawn_transfer(&op, owner, amount).await, op.created_at)? {
                    Pull::Block(block) => {
                        journal::advance(op_id, OpState::Pulled { block }, ic_cdk::api::time())?;
                    }
                    Pull::Rejected { reason, .. } => {
                        fail_spawn(&op, owner, &reason)?;
                        return Err(ApiError::Internal(format!("spawn pull rejected: {reason}")));
                    }
                }
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
    ensure_not_paused(config::get().paused.spawn, "spawn")?;
    if matches!(args.path, PayPath::Treasury) {
        return Err(ApiError::invalid(
            "spawn does not support the Treasury path",
        ));
    }
    if matches!(args.path, PayPath::Invite { .. }) && !config::get().features.sponsored_spawn {
        return Err(ApiError::FeatureDisabled);
    }
    let _guard = CallerGuard::acquire(guard::key("spawn", caller))?;
    let platform = config::platform_id()?;

    let existing = platform_aaa_by_owner(platform, caller).await?;
    platform_client::evaluate_owner_lookup(existing)?;
    let check = platform_check_name(platform, args.name.clone()).await?;
    platform_client::evaluate_check_name(check)?;

    let params = config::get().params;
    let now = ic_cdk::api::time();
    let rate = rate::get();

    let deposit_e8s = match &args.path {
        PayPath::Invite { code } => invites::sponsor_spawn(
            code,
            caller,
            now,
            params.spawn_creation_fee_cycles,
            rate.xdr_permyriad_per_icp,
            params.sponsor_daily_cap_e8s,
        )?,
        _ => quote::quote_spawn(&params, &rate, now / 1_000_000_000)?.deposit_e8s,
    };

    let op = journal::create(
        OpKind::Spawn {
            owner: caller,
            name: args.name,
            avatar_seed: args.avatar_seed,
        },
        args.path,
        deposit_e8s,
        caller,
        now,
    );
    journal::index_owner(caller, op.id);
    advance_spawn_saga(op.id, platform).await?;
    Ok(op.id)
}

#[ic_cdk::update]
pub(crate) async fn resume(op_id: u64) -> Result<(), ApiError> {
    let op = journal::get(op_id).ok_or(ApiError::NotFound)?;
    if op.state.is_terminal() {
        return Ok(());
    }
    match &op.kind {
        OpKind::Spawn { owner, .. } => {
            let owner = *owner;
            let _guard = CallerGuard::acquire(guard::key("spawn", owner))?;
            let platform = config::platform_id()?;
            advance_spawn_saga(op_id, platform).await
        }
        OpKind::TopUp { aaa } => {
            let aaa = *aaa;
            let _guard = CallerGuard::acquire(guard::key("topup", aaa))?;
            advance_topup_saga(op_id).await
        }
        OpKind::AutoTopUp { aaa } => {
            let aaa = *aaa;
            let _guard = CallerGuard::acquire(guard::key("auto", aaa))?;
            advance_auto_topup_saga(op_id).await
        }
        _ => Err(ApiError::Internal(
            "resume: this op kind is not implemented yet".into(),
        )),
    }
}

#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct TreasuryWithdrawArgs {
    pub to: Account,
    pub amount: Nat,
    pub created_at_time: Option<u64>,
}

#[ic_cdk::update]
async fn admin_treasury_withdraw(args: TreasuryWithdrawArgs) -> Result<u64, ApiError> {
    let caller = require_admin()?;
    let amount = u64::try_from(args.amount.0.clone())
        .map_err(|_| ApiError::invalid("amount exceeds u64 e8s"))?;
    if amount == 0 {
        return Err(ApiError::invalid("amount must be greater than zero"));
    }
    let created_at = args.created_at_time.unwrap_or_else(ic_cdk::api::time);
    audit(
        caller,
        "admin_treasury_withdraw",
        &args,
        format!(
            "intent: {amount} e8s to {:?}, created_at_time {created_at}",
            args.to
        ),
    );
    let outcome = icrc1_transfer(
        Some(deposit::treasury_subaccount()),
        args.to.clone(),
        amount,
        cmc::MEMO_WITHDRAW.to_le_bytes().to_vec(),
        created_at,
    )
    .await;
    audit(
        caller,
        "admin_treasury_withdraw",
        &args,
        format!("outcome for created_at_time {created_at}: {outcome:?}"),
    );
    match settle(outcome, created_at)? {
        Pull::Block(block) => Ok(block),
        Pull::Rejected { reason, .. } => Err(ApiError::Internal(format!(
            "icrc1_transfer rejected: {reason}"
        ))),
    }
}
