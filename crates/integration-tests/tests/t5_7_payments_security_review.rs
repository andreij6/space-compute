use candid::{decode_args, encode_args, encode_one, CandidType, Nat, Principal};
use icrc_ledger_types::icrc1::account::Account as Icrc1Account;
use icrc_ledger_types::icrc2::approve::{ApproveArgs, ApproveError};
use integration_tests::pic::{user, IcpEnv, E8S, LEDGER};
use integration_tests::step;
use payments::api::{SpawnArgs, TopUpArgs};
use payments::config::PauseFlags;
use payments::deposit::{self, Purpose};
use payments::journal::{Account, Op, OpState, PayPath};
use payments::quote::Quote;
use sc_types::ApiError;
use serde::Deserialize;

#[derive(CandidType, Deserialize)]
struct WithdrawArgs {
    to: Account,
    amount: Nat,
    created_at_time: Option<u64>,
}

fn install_wired(env: &IcpEnv, admin: Principal) -> (Principal, Principal) {
    let platform = env.install("platform", admin);
    let payments = env.install("payments", admin);
    for _ in 0..5 {
        env.pic.tick();
    }
    let ok: Result<(), ApiError> = env.update(platform, admin, "admin_set_payments_id", payments);
    assert_eq!(ok, Ok(()));
    let ok: Result<(), ApiError> = env.update(payments, admin, "admin_set_platform_id", platform);
    assert_eq!(ok, Ok(()));
    (platform, payments)
}

fn balance_of(env: &IcpEnv, account: Icrc1Account) -> u64 {
    let n: Nat = env.query(LEDGER, Principal::anonymous(), "icrc1_balance_of", account);
    n.0.try_into().unwrap()
}

fn approve(env: &IcpEnv, owner: Principal, spender: Icrc1Account, amount_e8s: u64) {
    let r: Result<Nat, ApproveError> = env.update(
        LEDGER,
        owner,
        "icrc2_approve",
        ApproveArgs {
            from_subaccount: None,
            spender,
            amount: Nat::from(amount_e8s),
            expected_allowance: None,
            expires_at: None,
            fee: None,
            memo: None,
            created_at_time: None,
        },
    );
    r.expect("icrc2_approve");
}

fn spawn(
    env: &IcpEnv,
    payments: Principal,
    caller: Principal,
    args: SpawnArgs,
) -> Result<u64, ApiError> {
    let bytes = env
        .pic
        .update_call(payments, caller, "spawn_aaa", encode_one(args).unwrap())
        .expect("spawn_aaa");
    candid::decode_one(&bytes).unwrap()
}

#[test]
fn t5_7_a_rejected_wallet_pull_fails_the_op_and_is_never_retried() {
    println!("T5.7 demo: a definitively rejected ledger pull ends the op; later allowances are never pulled");
    let env = IcpEnv::new();
    let admin = user(1);
    let owner = user(20);
    let (_platform, payments) = install_wired(&env, admin);
    env.mint_icp(owner, 20 * E8S);

    let quote: Result<Quote, ApiError> = env.query(payments, owner, "get_quote_spawn", ());
    let quote = quote.expect("quote");
    let spender = Icrc1Account {
        owner: payments,
        subaccount: Some(deposit::spender_subaccount(Purpose::Spawn, owner)),
    };
    approve(&env, owner, spender, quote.deposit_e8s / 2);
    step("owner approved only half the spawn quote to S(spawn, owner)");

    let payer = Account {
        owner,
        subaccount: None,
    };
    let r = spawn(
        &env,
        payments,
        owner,
        SpawnArgs {
            name: "Rejected-01".into(),
            avatar_seed: 1,
            path: PayPath::Wallet { payer },
        },
    );
    assert!(
        r.is_err(),
        "an insufficient allowance must reject the spawn"
    );
    let op: Option<Op> = env.query(payments, owner, "get_op", 0u64);
    assert!(
        matches!(op.unwrap().state, OpState::Failed { .. }),
        "a definitive InsufficientAllowance must end the op as Failed, not leave it resumable"
    );
    step("the InsufficientAllowance rejection marks op 0 Failed");

    let owner_account = Icrc1Account {
        owner,
        subaccount: None,
    };
    approve(&env, owner, spender, 10 * E8S);
    let before = balance_of(&env, owner_account);
    let _: Result<(), ApiError> = env.update(payments, user(9), "resume", 0u64);
    env.pic.advance_time(std::time::Duration::from_secs(301));
    for _ in 0..5 {
        env.pic.tick();
    }
    assert_eq!(
        balance_of(&env, owner_account),
        before,
        "a later, larger allowance must never be pulled by the dead op"
    );
    step("after a bigger approval, resume + the 5-min sweep pull nothing for the failed op");
}

#[test]
fn t5_7_pause_flags_block_new_spawn_topup_and_auto_topup() {
    println!("T5.7 demo: admin_pause is a real kill switch");
    let env = IcpEnv::new();
    let admin = user(1);
    let owner = user(21);
    let (_platform, payments) = install_wired(&env, admin);
    let ok: Result<(), ApiError> = env.update(
        payments,
        admin,
        "admin_pause",
        PauseFlags {
            spawn: true,
            topup: true,
            auto_topup: true,
            non_icp: false,
        },
    );
    assert_eq!(ok, Ok(()));

    let s = spawn(
        &env,
        payments,
        owner,
        SpawnArgs {
            name: "Paused-01".into(),
            avatar_seed: 1,
            path: PayPath::Deposit,
        },
    );
    assert!(matches!(s, Err(ApiError::NotEligible(_))), "spawn: {s:?}");
    let t: Result<u64, ApiError> = env.update(
        payments,
        owner,
        "top_up",
        TopUpArgs {
            aaa: user(22),
            path: PayPath::Deposit,
        },
    );
    assert!(matches!(t, Err(ApiError::NotEligible(_))), "top_up: {t:?}");
    let a: Result<u64, ApiError> = env.update(payments, user(22), "request_auto_topup", ());
    assert!(matches!(a, Err(ApiError::NotEligible(_))), "auto: {a:?}");
    step("with spawn/topup/auto_topup paused, all three entry points refuse before any await");
}

#[test]
fn t5_7_treasury_withdraw_retry_with_the_same_created_at_time_pays_once() {
    println!("T5.7 demo: admin_treasury_withdraw is ledger-deduplicated on retry");
    let env = IcpEnv::new();
    let admin = user(1);
    let (_platform, payments) = install_wired(&env, admin);
    let raw = env
        .pic
        .query_call(
            payments,
            admin,
            "get_treasury_account",
            encode_args(()).unwrap(),
        )
        .expect("get_treasury_account");
    let (_, treasury): (String, Account) = decode_args(&raw).unwrap();
    let treasury = Icrc1Account {
        owner: treasury.owner,
        subaccount: treasury.subaccount,
    };
    env.mint_icp(user(2), 20 * E8S);
    env.icrc1_transfer(user(2), treasury, 5 * E8S, None)
        .expect("fund treasury");

    let dest = Icrc1Account {
        owner: user(3),
        subaccount: None,
    };
    let created_at = env.pic.get_time().as_nanos_since_unix_epoch();
    let args = || WithdrawArgs {
        to: Account {
            owner: user(3),
            subaccount: None,
        },
        amount: Nat::from(E8S),
        created_at_time: Some(created_at),
    };
    let first: Result<u64, ApiError> =
        env.update(payments, admin, "admin_treasury_withdraw", args());
    env.pic.advance_time(std::time::Duration::from_secs(2));
    env.pic.tick();
    let second: Result<u64, ApiError> =
        env.update(payments, admin, "admin_treasury_withdraw", args());
    assert_eq!(
        first, second,
        "the retry must resolve to the same ledger block"
    );
    assert_eq!(
        balance_of(&env, dest),
        E8S,
        "the destination is paid exactly once"
    );
    step(
        "a retried withdraw with the same created_at_time returns the original block and pays once",
    );
}
