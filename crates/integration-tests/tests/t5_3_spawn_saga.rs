use candid::{decode_one, encode_args, encode_one, Nat, Principal};
use icrc_ledger_types::icrc1::account::Account as Icrc1Account;
use integration_tests::pic::{canister_wasm, user, IcpEnv, E8S, LEDGER};
use integration_tests::step;
use payments::api::SpawnArgs;
use payments::deposit::Purpose;
use payments::journal::{Account, NotifiedInfo, Op, OpState, PayPath};
use payments::quote::Quote;
use platform::registry::AaaRecord;
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

fn spawn_aaa(
    env: &IcpEnv,
    payments: Principal,
    caller: Principal,
    args: SpawnArgs,
) -> Result<u64, ApiError> {
    let bytes = env
        .pic
        .update_call(payments, caller, "spawn_aaa", encode_one(args).unwrap())
        .expect("spawn_aaa rejected");
    decode_one(&bytes).unwrap()
}

fn deposit_account(env: &IcpEnv, payments: Principal, beneficiary: Principal) -> Icrc1Account {
    let raw = env
        .pic
        .query_call(
            payments,
            beneficiary,
            "get_deposit_account",
            encode_args((Purpose::Spawn, beneficiary)).unwrap(),
        )
        .expect("get_deposit_account");
    let (_, account): (String, Account) = candid::decode_args(&raw).unwrap();
    Icrc1Account {
        owner: account.owner,
        subaccount: account.subaccount,
    }
}

fn balance_of(env: &IcpEnv, account: Icrc1Account) -> u64 {
    let n: Nat = env.query(LEDGER, Principal::anonymous(), "icrc1_balance_of", account);
    n.0.try_into().unwrap()
}

fn fund_deposit_account(
    env: &IcpEnv,
    payments: Principal,
    alice: Principal,
    owner: Principal,
    e8s: u64,
) -> Icrc1Account {
    let account = deposit_account(env, payments, owner);
    env.icrc1_transfer(alice, account, e8s, None)
        .expect("fund the spawn deposit account");
    account
}

#[test]
fn t5_3_spawn_saga_deposit_path_reaches_done_and_resumes_after_platform_failure() {
    println!("T5.3 demo: spawn saga (Deposit path) + resume timer");
    let env = IcpEnv::new();
    let admin = user(1);
    let alice = user(2);

    let platform = env.install("platform", admin);
    let payments = env.install("payments", admin);
    for _ in 0..5 {
        env.pic.tick();
    }
    let ok: Result<(), ApiError> = env.update(platform, admin, "admin_set_payments_id", payments);
    assert_eq!(ok, Ok(()));
    let ok: Result<(), ApiError> = env.update(payments, admin, "admin_set_platform_id", platform);
    assert_eq!(ok, Ok(()));
    step("installed platform + payments and wired their peer ids");

    env.mint_icp(alice, 20 * E8S);
    let owner1 = user(10);

    let quote: Result<Quote, ApiError> = env.query(payments, owner1, "get_quote_spawn", ());
    let quote = quote.expect("get_quote_spawn");
    let deposit1 = fund_deposit_account(&env, payments, alice, owner1, quote.total_e8s);
    step(&format!(
        "funded owner1's D(spawn, owner1) deposit account with the quoted {} e8s",
        quote.total_e8s
    ));

    let spawn_args = SpawnArgs {
        name: "OrionSurveyor-01".into(),
        avatar_seed: 1,
        path: PayPath::Deposit,
    };
    let first = spawn_aaa(&env, payments, owner1, spawn_args);
    assert!(
        first.is_err(),
        "no AAA wasm is approved on platform yet, so register_aaa must fail mid-saga"
    );
    step(&format!(
        "spawn_aaa without an approved AAA wasm on platform fails mid-saga: {first:?}"
    ));

    let op0: Option<Op> = env.query(payments, owner1, "get_op", 0u64);
    let op0 = op0.expect("op 0 exists");
    let canister_id = match op0.state {
        OpState::Notified {
            canister_or_cycles: NotifiedInfo::Canister(id),
        } => id,
        other => panic!("expected op 0 stuck at Notified, got {other:?}"),
    };
    step(&format!(
        "op 0 is a resumable record stuck at Notified{{{canister_id}}}: the CMC created the AAA canister but platform.register_aaa could not install it yet"
    ));

    upload_and_approve_aaa_wasm(&env, platform, admin);
    step("admin uploaded and approved the AAA wasm on platform (the platform-side outage is over)");

    let resumed: Result<(), ApiError> = env.update(payments, alice, "resume", 0u64);
    assert_eq!(resumed, Ok(()));
    step("resume(0), called by an unrelated principal, completes the saga");

    let op0_done: Option<Op> = env.query(payments, owner1, "get_op", 0u64);
    assert_eq!(op0_done.unwrap().state, OpState::Done);

    let record: Option<AaaRecord> = env.query(platform, admin, "get_aaa", canister_id);
    let record = record.expect("platform shows the new AAA registered");
    assert_eq!(record.owner, owner1);
    assert_eq!(record.name, "OrionSurveyor-01");
    assert!(record.platform_is_controller);
    let owner_lookup: Option<Principal> = env.query(platform, admin, "aaa_by_owner", owner1);
    assert_eq!(owner_lookup, Some(canister_id));
    let controllers = env.pic.get_controllers(canister_id);
    assert!(controllers.contains(&owner1) && controllers.contains(&platform));
    step(&format!(
        "platform shows {canister_id} Active, owned by owner1, controllers [owner, platform]"
    ));

    let cycles = env.pic.cycle_balance(canister_id);
    assert!(
        cycles > 0,
        "the new AAA must have received cycles from the CMC"
    );
    step(&format!("the new AAA holds {cycles} cycles from the CMC"));

    let deposit_balance = balance_of(&env, deposit1);
    assert_eq!(
        deposit_balance, 0,
        "the deposit account must be swept to zero"
    );

    let dup: Result<(), ApiError> = env.update(payments, alice, "resume", 0u64);
    assert_eq!(dup, Ok(()));
    let op0_still_done: Option<Op> = env.query(payments, owner1, "get_op", 0u64);
    assert_eq!(op0_still_done.unwrap().state, OpState::Done);
    step("a duplicate resume(0) on a Done op is a no-op");

    let owner2 = user(11);
    let quote2: Result<Quote, ApiError> = env.query(payments, owner2, "get_quote_spawn", ());
    let quote2 = quote2.expect("get_quote_spawn");
    fund_deposit_account(&env, payments, alice, owner2, quote2.total_e8s);
    let op2 = spawn_aaa(
        &env,
        payments,
        owner2,
        SpawnArgs {
            name: "OrionSurveyor-02".into(),
            avatar_seed: 2,
            path: PayPath::Deposit,
        },
    )
    .expect("with the wasm approved, a single spawn_aaa call reaches Done directly");
    let op2_view: Option<Op> = env.query(payments, owner2, "get_op", op2);
    assert_eq!(op2_view.unwrap().state, OpState::Done);
    step("a second owner spawns end-to-end in one spawn_aaa call once platform is healthy");
}
