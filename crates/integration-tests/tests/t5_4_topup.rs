use candid::{decode_args, decode_one, encode_args, encode_one, Nat, Principal};
use icrc_ledger_types::icrc1::account::Account as Icrc1Account;
use integration_tests::pic::{canister_wasm, user, IcpEnv, E8S, LEDGER};
use integration_tests::step;
use payments::api::{SpawnArgs, TopUpArgs};
use payments::deposit::Purpose;
use payments::journal::{Account, Op, OpState, PayPath};
use payments::quote::{Quote, MIN_TOPUP_E8S};
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

fn balance_of(env: &IcpEnv, account: Icrc1Account) -> u64 {
    let n: Nat = env.query(LEDGER, Principal::anonymous(), "icrc1_balance_of", account);
    n.0.try_into().unwrap()
}

fn spawn_registered_aaa(
    env: &IcpEnv,
    platform: Principal,
    payments: Principal,
    admin: Principal,
) -> Principal {
    let alice = user(2);
    env.mint_icp(alice, 20 * E8S);
    let owner = user(3);
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
                name: "GiftedRover".into(),
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

fn top_up(
    env: &IcpEnv,
    payments: Principal,
    caller: Principal,
    aaa: Principal,
) -> Result<u64, ApiError> {
    let bytes = env
        .pic
        .update_call(
            payments,
            caller,
            "top_up",
            encode_one(TopUpArgs {
                aaa,
                path: PayPath::Deposit,
            })
            .unwrap(),
        )
        .expect("top_up rejected");
    decode_one(&bytes).unwrap()
}

#[test]
fn t5_4_a_stranger_gifts_fuel_to_a_registered_aaa_via_deposit_and_below_minimum_is_rejected() {
    println!("T5.4 demo: one-time top-up (Deposit path) — a gift from a stranger reaches Done");
    let env = IcpEnv::new();
    let admin = user(1);

    let platform = env.install("platform", admin);
    let payments = env.install("payments", admin);
    let ok: Result<(), ApiError> = env.update(platform, admin, "admin_set_payments_id", payments);
    assert_eq!(ok, Ok(()));
    let ok: Result<(), ApiError> = env.update(payments, admin, "admin_set_platform_id", platform);
    assert_eq!(ok, Ok(()));
    step("installed platform + payments and wired their peer ids");

    let aaa = spawn_registered_aaa(&env, platform, payments, admin);
    let cycles_before = env.pic.cycle_balance(aaa);
    step(&format!(
        "owner spawned and registered AAA {aaa} holding {cycles_before} cycles"
    ));

    let stranger = user(20);
    env.mint_icp(stranger, 5 * E8S);

    let below_min = MIN_TOPUP_E8S - 1;
    let deposit = deposit_account(&env, payments, Purpose::TopUp, aaa);
    env.icrc1_transfer(stranger, deposit, below_min, None)
        .expect("fund the topup deposit account below the minimum");
    let rejected = top_up(&env, payments, stranger, aaa);
    assert!(
        matches!(rejected, Err(ApiError::InvalidInput(_))),
        "expected a below-minimum top_up to be rejected, got {rejected:?}"
    );
    let deposit_balance = balance_of(&env, deposit);
    assert_eq!(
        deposit_balance, below_min,
        "no funds may move when the gift is below the 0.1 ICP minimum"
    );
    step(&format!(
        "a {below_min}-e8s gift (below the 0.1 ICP minimum) is rejected before any funds move"
    ));

    let top_up_amount = MIN_TOPUP_E8S + 3 * E8S;
    env.icrc1_transfer(stranger, deposit, top_up_amount - below_min, None)
        .expect("top the deposit account up to the required amount");
    step(&format!(
        "the stranger's D(topup, aaa) deposit account now holds {top_up_amount} e8s (>= 0.1 ICP)"
    ));

    let op_id = top_up(&env, payments, stranger, aaa).expect("top_up should reach Done");
    let op: Option<Op> = env.query(payments, stranger, "get_op", op_id);
    let op = op.expect("op exists");
    assert_eq!(op.state, OpState::Done);
    step(&format!(
        "top_up op {op_id} (gifted by an unrelated stranger) reached Done"
    ));

    let deposit_after = balance_of(&env, deposit);
    assert_eq!(
        deposit_after, 0,
        "the deposit account must be swept to zero"
    );

    let cycles_after = env.pic.cycle_balance(aaa);
    assert!(
        cycles_after > cycles_before,
        "the AAA's real cycle balance must increase from the gifted top-up: before={cycles_before} after={cycles_after}"
    );
    step(&format!(
        "the AAA's real cycle balance increased from {cycles_before} to {cycles_after}"
    ));
}
