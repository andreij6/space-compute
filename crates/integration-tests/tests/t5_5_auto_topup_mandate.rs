use candid::{decode_args, decode_one, encode_args, encode_one, Nat, Principal};
use icrc_ledger_types::icrc1::account::Account as Icrc1Account;
use icrc_ledger_types::icrc2::approve::{ApproveArgs, ApproveError};
use integration_tests::pic::{canister_wasm, user, IcpEnv, E8S, LEDGER};
use integration_tests::step;
use payments::api::{SetMandateArgs, SpawnArgs};
use payments::deposit::Purpose;
use payments::journal::{Account, Op, OpState, PayPath};
use payments::mandate::MandateView;
use payments::quote::Quote;
use sc_types::ApiError;
use sha2::{Digest, Sha256};

fn upload_and_approve_aaa_wasm(env: &IcpEnv, platform: Principal, admin: Principal) {
    let wasm = canister_wasm("aaa");
    let hash = Sha256::digest(&wasm).to_vec();
    let bytes = env
        .pic
        .update_call(
            platform,
            admin,
            "admin_upload_wasm",
            encode_args((1u32, wasm, hash)).unwrap(),
        )
        .expect("admin_upload_wasm");
    let r: Result<(), ApiError> = decode_one(&bytes).unwrap();
    r.expect("upload wasm");
    let ok: Result<(), ApiError> = env.update(platform, admin, "admin_approve_wasm", 1u32);
    ok.expect("approve wasm");
}

fn deposit_account(
    env: &IcpEnv,
    payments: Principal,
    purpose: Purpose,
    beneficiary: Principal,
) -> Icrc1Account {
    let raw = env
        .pic
        .query_call(
            payments,
            beneficiary,
            "get_deposit_account",
            encode_args((purpose, beneficiary)).unwrap(),
        )
        .expect("get_deposit_account");
    let (_, account): (String, Account) = decode_args(&raw).unwrap();
    Icrc1Account {
        owner: account.owner,
        subaccount: account.subaccount,
    }
}

fn spawn_registered_aaa(
    env: &IcpEnv,
    platform: Principal,
    payments: Principal,
    admin: Principal,
    owner: Principal,
) -> Principal {
    let alice = user(2);
    env.mint_icp(alice, 20 * E8S);
    let quote: Result<Quote, ApiError> = env.query(payments, owner, "get_quote_spawn", ());
    let quote = quote.expect("get_quote_spawn");
    let deposit = deposit_account(env, payments, Purpose::Spawn, owner);
    env.icrc1_transfer(alice, deposit, quote.total_e8s, None)
        .expect("fund the spawn deposit account");
    upload_and_approve_aaa_wasm(env, platform, admin);
    let bytes = env
        .pic
        .update_call(
            payments,
            owner,
            "spawn_aaa",
            encode_one(SpawnArgs {
                name: "MandateRover".into(),
                avatar_seed: 1,
                path: PayPath::Deposit,
            })
            .unwrap(),
        )
        .expect("spawn_aaa rejected");
    let op_id: Result<u64, ApiError> = decode_one(&bytes).unwrap();
    let op_id = op_id.expect("spawn_aaa succeeds once the aaa wasm is approved");
    let op: Option<Op> = env.query(payments, owner, "get_op", op_id);
    match op.unwrap().state {
        OpState::Done => {}
        other => panic!("expected spawn op Done, got {other:?}"),
    }
    let record: Option<Principal> = env.query(platform, admin, "aaa_by_owner", owner);
    record.expect("owner has a registered aaa")
}

fn approve(env: &IcpEnv, owner: Principal, spender: Icrc1Account, amount_e8s: u64) {
    let approve: Result<Nat, ApproveError> = env.update(
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
    approve.expect("icrc2_approve");
}

fn spender_account(payments: Principal, aaa: Principal) -> Icrc1Account {
    let sub = payments::deposit::spender_subaccount(Purpose::Auto, aaa);
    Icrc1Account {
        owner: payments,
        subaccount: Some(sub),
    }
}

fn set_mandate(
    env: &IcpEnv,
    payments: Principal,
    owner: Principal,
    aaa: Principal,
    topup_e8s: u64,
    cap_30d_e8s: u64,
) {
    let ok: Result<(), ApiError> = env.update(
        payments,
        owner,
        "set_mandate",
        SetMandateArgs {
            aaa,
            payer: Account {
                owner,
                subaccount: None,
            },
            topup_e8s,
            cap_30d_e8s,
            enabled: true,
        },
    );
    ok.expect("set_mandate");
}

fn request_auto_topup(env: &IcpEnv, payments: Principal, aaa: Principal) -> Result<u64, ApiError> {
    let bytes = env
        .pic
        .update_call(payments, aaa, "request_auto_topup", encode_one(()).unwrap())
        .expect("request_auto_topup rejected at the ic level");
    decode_one(&bytes).unwrap()
}

fn setup() -> (IcpEnv, Principal, Principal, Principal, Principal) {
    let env = IcpEnv::new();
    let admin = user(1);
    let owner = user(3);

    let platform = env.install("platform", admin);
    let payments = env.install("payments", admin);
    for _ in 0..5 {
        env.pic.tick();
    }
    let ok: Result<(), ApiError> = env.update(platform, admin, "admin_set_payments_id", payments);
    assert_eq!(ok, Ok(()));
    let ok: Result<(), ApiError> = env.update(payments, admin, "admin_set_platform_id", platform);
    assert_eq!(ok, Ok(()));

    let aaa = spawn_registered_aaa(&env, platform, payments, admin, owner);
    (env, admin, owner, payments, aaa)
}

#[test]
fn t5_5_request_auto_topup_succeeds_within_cap_and_interval_then_second_call_rejected_by_interval()
{
    println!("T5.5 demo: auto top-up mandate — succeeds within cap/interval, blocked again by the min interval");
    let (env, _admin, owner, payments, aaa) = setup();

    let topup_e8s = E8S;
    let cap_30d_e8s = 10 * E8S;
    set_mandate(&env, payments, owner, aaa, topup_e8s, cap_30d_e8s);
    step("owner set an auto top-up mandate: 1 ICP per top-up, 10 ICP rolling 30-day cap");

    env.mint_icp(owner, 20 * E8S);
    approve(&env, owner, spender_account(payments, aaa), 5 * topup_e8s);
    step("owner approved an allowance to the mandate's spender subaccount");

    let cycles_before = env.pic.cycle_balance(aaa);
    let op_id = request_auto_topup(&env, payments, aaa).expect("first auto top-up should succeed");
    let op: Option<Op> = env.query(payments, owner, "get_op", op_id);
    assert_eq!(op.unwrap().state, OpState::Done);
    let cycles_after = env.pic.cycle_balance(aaa);
    assert!(
        cycles_after > cycles_before,
        "the AAA's cycle balance must increase from the auto top-up: before={cycles_before} after={cycles_after}"
    );
    step(&format!(
        "request_auto_topup succeeded (op {op_id}), AAA cycles {cycles_before} -> {cycles_after}"
    ));

    let view: Option<MandateView> = env.query(payments, owner, "get_mandate", aaa);
    let view = view.expect("mandate view exists");
    assert_eq!(view.spent_30d_e8s, topup_e8s);
    assert!(!view.needs_attention);
    step(&format!(
        "get_mandate reports spent_30d_e8s={} remaining_30d_e8s={}",
        view.spent_30d_e8s, view.remaining_30d_e8s
    ));

    let cycles_before_second = env.pic.cycle_balance(aaa);
    let rejected = request_auto_topup(&env, payments, aaa);
    assert!(
        matches!(rejected, Err(ApiError::InvalidInput(_))),
        "a second call inside auto_topup_min_interval_secs must be rejected, got {rejected:?}"
    );
    let cycles_after_second = env.pic.cycle_balance(aaa);
    assert_eq!(
        cycles_before_second, cycles_after_second,
        "no cycles may move when the interval check rejects the call"
    );
    step("a second immediate call is rejected by auto_topup_min_interval_secs; no funds moved");
}

#[test]
fn t5_5_request_auto_topup_rejects_when_it_would_exceed_the_rolling_30d_cap() {
    println!("T5.5 demo: auto top-up mandate — exceeding the rolling 30-day cap is rejected without moving funds");
    let (env, _admin, owner, payments, aaa) = setup();

    let topup_e8s = 6 * E8S;
    let cap_30d_e8s = 10 * E8S;
    set_mandate(&env, payments, owner, aaa, topup_e8s, cap_30d_e8s);

    env.mint_icp(owner, 40 * E8S);
    approve(&env, owner, spender_account(payments, aaa), 10 * topup_e8s);

    let op_id = request_auto_topup(&env, payments, aaa).expect("first auto top-up should succeed");
    let op: Option<Op> = env.query(payments, owner, "get_op", op_id);
    assert_eq!(op.unwrap().state, OpState::Done);
    step(&format!(
        "first auto top-up of {topup_e8s} e8s succeeded (spent_30d now {topup_e8s} of {cap_30d_e8s} cap)"
    ));

    env.pic
        .advance_time(std::time::Duration::from_secs(6 * 3_600 + 60));
    env.pic.tick();

    let allowance_before = spender_allowance(&env, owner, payments, aaa);
    let cycles_before = env.pic.cycle_balance(aaa);
    let rejected = request_auto_topup(&env, payments, aaa);
    assert!(
        matches!(rejected, Err(ApiError::InvalidInput(_))),
        "a second top-up of {topup_e8s} would bring spent_30d to {} > cap {cap_30d_e8s}, expected rejection, got {rejected:?}",
        2 * topup_e8s
    );
    let allowance_after = spender_allowance(&env, owner, payments, aaa);
    let cycles_after = env.pic.cycle_balance(aaa);
    assert_eq!(
        allowance_before, allowance_after,
        "the ledger allowance must not move when the 30-day cap check rejects the call"
    );
    assert_eq!(
        cycles_before, cycles_after,
        "no cycles may be minted when the 30-day cap check rejects the call"
    );
    step("second auto top-up exceeding the rolling 30-day cap is rejected; allowance and cycles unchanged");
}

fn spender_allowance(env: &IcpEnv, owner: Principal, payments: Principal, aaa: Principal) -> u64 {
    #[derive(candid::CandidType)]
    struct AllowanceArg {
        account: Icrc1Account,
        spender: Icrc1Account,
    }
    #[derive(candid::CandidType, serde::Deserialize)]
    struct Allowance {
        allowance: Nat,
        expires_at: Option<u64>,
    }
    let a: Allowance = env.query(
        LEDGER,
        Principal::anonymous(),
        "icrc2_allowance",
        AllowanceArg {
            account: Icrc1Account {
                owner,
                subaccount: None,
            },
            spender: spender_account(payments, aaa),
        },
    );
    a.allowance.0.try_into().unwrap()
}
