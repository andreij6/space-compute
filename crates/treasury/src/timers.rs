use std::cell::{Cell, RefCell};
use std::time::Duration;

use candid::{CandidType, Nat, Principal};
use ic_cdk::call::Call;
use ic_cdk_timers::TimerId;
use serde::Deserialize;

use crate::config;
use crate::keeper;
use crate::state::{self, HistoryKind, Inflight};

pub const LEDGER: Principal = Principal::from_slice(&[0, 0, 0, 0, 0, 0, 0, 2, 1, 1]);
pub const CMC: Principal = Principal::from_slice(&[0, 0, 0, 0, 0, 0, 0, 4, 1, 1]);
const MEMO_TOP_UP: u64 = 0x5055_5054;
const MAX_ATTEMPTS: u8 = 5;

thread_local! {
    static RUNNING: Cell<bool> = const { Cell::new(false) };
    static INTERVAL: RefCell<Option<TimerId>> = const { RefCell::new(None) };
}

struct RunLock;

impl RunLock {
    fn acquire() -> Option<Self> {
        (!RUNNING.replace(true)).then_some(RunLock)
    }
}

impl Drop for RunLock {
    fn drop(&mut self) {
        RUNNING.set(false);
    }
}

#[derive(CandidType, Deserialize)]
struct Account {
    owner: Principal,
    subaccount: Option<Vec<u8>>,
}

#[derive(CandidType)]
struct TransferArg {
    from_subaccount: Option<Vec<u8>>,
    to: Account,
    amount: Nat,
    fee: Option<Nat>,
    memo: Option<Vec<u8>>,
    created_at_time: Option<u64>,
}

#[derive(CandidType, Deserialize, Debug)]
enum TransferError {
    BadFee { expected_fee: Nat },
    BadBurn { min_burn_amount: Nat },
    InsufficientFunds { balance: Nat },
    TooOld,
    CreatedInFuture { ledger_time: u64 },
    TemporarilyUnavailable,
    Duplicate { duplicate_of: Nat },
    GenericError { error_code: Nat, message: String },
}

pub enum Outcome {
    Done(u64),
    Failed(String),
    Unknown(String),
}

#[derive(CandidType, Deserialize)]
struct RateData {
    xdr_permyriad_per_icp: u64,
}

#[derive(CandidType, Deserialize)]
struct RateResponse {
    data: RateData,
}

#[derive(CandidType)]
struct StatusArgs {
    canister_id: Principal,
}

#[derive(CandidType, Deserialize)]
struct CanisterStatus {
    cycles: Nat,
    idle_cycles_burned_per_day: Nat,
}

#[derive(CandidType)]
struct NotifyTopUp {
    block_index: u64,
    canister_id: Principal,
}

fn nat_u128(n: &Nat) -> u128 {
    u128::try_from(n.0.clone()).unwrap_or(u128::MAX)
}

fn nat_u64(n: &Nat) -> u64 {
    u64::try_from(n.0.clone()).unwrap_or(u64::MAX)
}

fn me() -> Principal {
    ic_cdk::api::canister_self()
}

fn now() -> u64 {
    ic_cdk::api::time()
}

pub fn start() {
    ic_cdk_timers::set_timer(Duration::ZERO, tick(None));
    restart();
}

pub fn restart() {
    let every = Duration::from_secs(config::get().check_interval_secs);
    if let Some(old) = INTERVAL.take() {
        ic_cdk_timers::clear_timer(old);
    }
    INTERVAL.set(Some(ic_cdk_timers::set_timer_interval(every, || {
        tick(None)
    })));
}

async fn icp_balance() -> Option<u64> {
    let account = Account {
        owner: me(),
        subaccount: None,
    };
    let reply = Call::bounded_wait(LEDGER, "icrc1_balance_of")
        .with_arg(account)
        .await
        .ok()?;
    reply.candid::<Nat>().ok().map(|n| nat_u64(&n))
}

async fn xdr_rate() -> Option<u64> {
    let reply = Call::bounded_wait(CMC, "get_icp_xdr_conversion_rate")
        .with_args(&())
        .await
        .ok()?;
    reply
        .candid::<RateResponse>()
        .ok()
        .map(|r| r.data.xdr_permyriad_per_icp)
}

async fn canister_status(canister: Principal) -> Option<CanisterStatus> {
    let reply = Call::bounded_wait(Principal::management_canister(), "canister_status")
        .with_arg(StatusArgs {
            canister_id: canister,
        })
        .await
        .ok()?;
    reply.candid::<CanisterStatus>().ok()
}

pub async fn transfer(
    to: Principal,
    subaccount: Option<Vec<u8>>,
    e8s: u64,
    memo: Option<Vec<u8>>,
    created_at: u64,
) -> Outcome {
    let arg = TransferArg {
        from_subaccount: None,
        to: Account {
            owner: to,
            subaccount,
        },
        amount: Nat::from(e8s),
        fee: Some(Nat::from(keeper::LEDGER_FEE_E8S)),
        memo,
        created_at_time: Some(created_at),
    };
    let reply = match Call::bounded_wait(LEDGER, "icrc1_transfer")
        .with_arg(arg)
        .await
    {
        Ok(r) => r,
        Err(e) => return Outcome::Unknown(format!("{e:?}")),
    };
    match reply.candid::<Result<Nat, TransferError>>() {
        Ok(Ok(block)) => Outcome::Done(nat_u64(&block)),
        Ok(Err(TransferError::Duplicate { duplicate_of })) => Outcome::Done(nat_u64(&duplicate_of)),
        Ok(Err(TransferError::TemporarilyUnavailable)) => {
            Outcome::Unknown("ledger temporarily unavailable".into())
        }
        Ok(Err(e)) => Outcome::Failed(format!("{e:?}")),
        Err(e) => Outcome::Unknown(format!("{e:?}")),
    }
}

async fn notify(block: u64, canister: Principal) -> Result<u128, String> {
    let reply = Call::bounded_wait(CMC, "notify_top_up")
        .with_arg(NotifyTopUp {
            block_index: block,
            canister_id: canister,
        })
        .await
        .map_err(|e| format!("{e:?}"))?;
    match reply
        .candid::<Result<Nat, candid::Reserved>>()
        .map_err(|e| format!("{e:?}"))?
    {
        Ok(cycles) => Ok(nat_u128(&cycles)),
        Err(_) => Err("notify_top_up rejected".into()),
    }
}

fn set_inflight(i: Option<Inflight>) {
    let mut s = state::snapshot();
    s.inflight = i;
    state::set_snapshot(s);
}

fn retry_or_fail(mut i: Inflight, reason: String) {
    i.attempts = i.attempts.saturating_add(1);
    if i.attempts >= MAX_ATTEMPTS {
        set_inflight(None);
        state::log(
            now(),
            me(),
            HistoryKind::TopUpFailed {
                canister: i.canister,
                reason,
            },
        );
    } else {
        set_inflight(Some(i));
    }
}

async fn settle(mut i: Inflight) -> bool {
    let block = match i.block {
        Some(b) => b,
        None => {
            let sub = Some(keeper::topup_subaccount(&i.canister).to_vec());
            let memo = Some(MEMO_TOP_UP.to_le_bytes().to_vec());
            match transfer(CMC, sub, i.e8s, memo, i.created_at).await {
                Outcome::Done(b) => {
                    i.block = Some(b);
                    set_inflight(Some(i.clone()));
                    b
                }
                Outcome::Failed(reason) => {
                    set_inflight(None);
                    state::log(
                        now(),
                        me(),
                        HistoryKind::TopUpFailed {
                            canister: i.canister,
                            reason,
                        },
                    );
                    return false;
                }
                Outcome::Unknown(reason) => {
                    retry_or_fail(i, reason);
                    return false;
                }
            }
        }
    };
    match notify(block, i.canister).await {
        Ok(cycles) => {
            set_inflight(None);
            state::log(
                now(),
                me(),
                HistoryKind::TopUp {
                    canister: i.canister,
                    e8s: i.e8s,
                    cycles,
                    block,
                },
            );
            true
        }
        Err(reason) => {
            retry_or_fail(i, reason);
            false
        }
    }
}

pub async fn tick(only: Option<Principal>) {
    let Some(_lock) = RunLock::acquire() else {
        return;
    };
    run(only).await;
}

async fn run(only: Option<Principal>) {
    if let Some(i) = state::snapshot().inflight {
        if !settle(i).await {
            return;
        }
    }
    let (Some(mut icp), Some(rate)) = (icp_balance().await, xdr_rate().await) else {
        return;
    };
    let prev = state::snapshot();
    if prev.checked_at > 0 && icp > prev.icp_balance_e8s {
        state::log(
            now(),
            me(),
            HistoryKind::Deposit {
                e8s: icp - prev.icp_balance_e8s,
            },
        );
    }
    let cfg = config::get();
    let mut blocked = false;
    for (canister, watch) in state::watched() {
        if only.is_some_and(|o| o != canister) {
            continue;
        }
        let Some(st) = canister_status(canister).await else {
            state::log(
                now(),
                me(),
                HistoryKind::TopUpFailed {
                    canister,
                    reason: "canister_status failed (is treasury a controller?)".into(),
                },
            );
            continue;
        };
        let balance = nat_u128(&st.cycles);
        let m = state::record_sample(
            canister,
            balance,
            now(),
            nat_u128(&st.idle_cycles_burned_per_day),
        );
        let Some(cycles) = keeper::cycles_needed(
            balance,
            m.burn_per_day,
            watch.target_days,
            cfg.min_balance_cycles,
            cfg.min_topup_cycles,
        ) else {
            continue;
        };
        let Some(e8s) = keeper::e8s_for_cycles(cycles, rate) else {
            continue;
        };
        if blocked || !keeper::affordable(icp, e8s, cfg.reserve_e8s) {
            blocked = true;
            state::log(
                now(),
                me(),
                HistoryKind::SkippedReserve {
                    canister,
                    needed_e8s: e8s,
                },
            );
            continue;
        }
        let i = Inflight {
            canister,
            e8s,
            created_at: now(),
            block: None,
            attempts: 0,
        };
        set_inflight(Some(i));
        let done = settle(state::snapshot().inflight.expect("inflight set")).await;
        icp = match icp_balance().await {
            Some(b) => b,
            None => icp.saturating_sub(e8s + keeper::LEDGER_FEE_E8S),
        };
        if !done && state::snapshot().inflight.is_some() {
            break;
        }
    }
    let mut s = state::snapshot();
    s.icp_balance_e8s = icp;
    s.xdr_permyriad_per_icp = rate;
    s.checked_at = now();
    s.reserve_blocked = blocked;
    state::set_snapshot(s);
}
