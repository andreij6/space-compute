use candid::{decode_args, decode_one, encode_args, encode_one, Nat, Principal};
use icrc_ledger_types::icrc1::account::{Account as Icrc1Account, Subaccount};
use icrc_ledger_types::icrc2::approve::{ApproveArgs, ApproveError};
use integration_tests::pic::{canister_wasm, user, IcpEnv, NotifyError, CMC, E8S, LEDGER};
use integration_tests::step;
use payments::api::{SetMandateArgs, SpawnArgs, TopUpArgs};
use payments::deposit::{self, Purpose};
use payments::journal::{Account, Op, OpState, PayPath};
use payments::mandate::MandateView;
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

fn spawn_via_deposit(
    env: &IcpEnv,
    platform: Principal,
    payments: Principal,
    admin: Principal,
    funder: Principal,
    owner: Principal,
    name: &str,
) -> Principal {
    env.mint_icp(funder, 20 * E8S);
    let quote: Result<Quote, ApiError> = env.query(payments, owner, "get_quote_spawn", ());
    let quote = quote.expect("get_quote_spawn");
    let deposit = deposit_account(env, payments, Purpose::Spawn, owner);
    env.icrc1_transfer(funder, deposit, quote.total_e8s, None)
        .expect("fund the spawn deposit account");
    upload_and_approve_aaa_wasm(env, platform, admin);
    let bytes = env
        .pic
        .update_call(
            payments,
            owner,
            "spawn_aaa",
            encode_one(SpawnArgs {
                name: name.into(),
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

fn top_up_call(
    env: &IcpEnv,
    payments: Principal,
    caller: Principal,
    aaa: Principal,
    path: PayPath,
) -> Result<u64, ApiError> {
    let bytes = env
        .pic
        .update_call(
            payments,
            caller,
            "top_up",
            encode_one(TopUpArgs { aaa, path }).unwrap(),
        )
        .expect("top_up rejected at the ic level");
    decode_one(&bytes).unwrap()
}

#[test]
fn t5_6_wallet_path_spawn_reaches_done_with_controllers_and_cycles() {
    println!("T5.6 demo (acceptance #1): Wallet-path spawn — approve then spawn_aaa");
    let env = IcpEnv::new();
    let admin = user(1);
    let owner = user(2);
    let (platform, payments) = install_wired(&env, admin);
    upload_and_approve_aaa_wasm(&env, platform, admin);
    step("installed platform + payments, wired peer ids, approved the AAA wasm");

    env.mint_icp(owner, 20 * E8S);
    let quote: Result<Quote, ApiError> = env.query(payments, owner, "get_quote_spawn", ());
    let quote = quote.expect("get_quote_spawn");
    let spender = Icrc1Account {
        owner: payments,
        subaccount: Some(deposit::spender_subaccount(Purpose::Spawn, owner)),
    };
    approve(&env, owner, spender, quote.total_e8s);
    step(&format!(
        "owner approved {} e8s to S(spawn, owner) instead of funding a deposit account",
        quote.total_e8s
    ));

    let op_id: Result<u64, ApiError> = env.update(
        payments,
        owner,
        "spawn_aaa",
        SpawnArgs {
            name: "WalletRover".into(),
            avatar_seed: 7,
            path: PayPath::Wallet {
                payer: Account {
                    owner,
                    subaccount: None,
                },
            },
        },
    );
    let op_id = op_id.expect("wallet-path spawn_aaa should reach Done in one call");
    let op: Option<Op> = env.query(payments, owner, "get_op", op_id);
    assert_eq!(op.unwrap().state, OpState::Done);
    step("wallet-path spawn_aaa reached Done via icrc2_transfer_from, no deposit account used");

    let canister_id: Option<Principal> = env.query(platform, admin, "aaa_by_owner", owner);
    let canister_id = canister_id.expect("owner has a registered aaa");
    let controllers = env.pic.get_controllers(canister_id);
    assert!(controllers.contains(&owner) && controllers.contains(&platform));
    assert_eq!(controllers.len(), 2);
    let cycles = env.pic.cycle_balance(canister_id);
    assert!(cycles > 0, "the new AAA must hold cycles minted by the CMC");
    step(&format!(
        "AAA {canister_id} exists with controllers [owner, platform] and {cycles} cycles"
    ));
}

#[test]
fn t5_6_topup_wallet_allowance_is_scoped_to_the_correct_aaa() {
    println!(
        "T5.6 demo (acceptance #3): a Wallet-path top-up allowance for AAA X is unusable for AAA Y"
    );
    let env = IcpEnv::new();
    let admin = user(1);
    let victim = user(2);
    let (platform, payments) = install_wired(&env, admin);

    let aaa_x = spawn_via_deposit(&env, platform, payments, admin, user(90), user(10), "AaaX");
    let aaa_y = spawn_via_deposit(&env, platform, payments, admin, user(91), user(11), "AaaY");
    step(&format!("registered two AAAs: X={aaa_x} Y={aaa_y}"));

    env.mint_icp(victim, 20 * E8S);
    let spender_x = Icrc1Account {
        owner: payments,
        subaccount: Some(deposit::spender_subaccount(Purpose::TopUp, aaa_x)),
    };
    approve(&env, victim, spender_x, 5 * E8S);
    step("victim approved an allowance to S(topup, X) only");

    let stranger = user(20);
    let victim_account = Account {
        owner: victim,
        subaccount: None,
    };
    let cycles_y_before = env.pic.cycle_balance(aaa_y);
    let stolen = top_up_call(
        &env,
        payments,
        stranger,
        aaa_y,
        PayPath::Wallet {
            payer: victim_account.clone(),
        },
    );
    assert!(
        stolen.is_err(),
        "an allowance granted to S(topup, X) must not be usable for Y, got {stolen:?}"
    );
    let cycles_y_after = env.pic.cycle_balance(aaa_y);
    assert_eq!(
        cycles_y_before, cycles_y_after,
        "no cycles may move for Y from an allowance scoped to X"
    );
    step(&format!(
        "a stranger calling top_up{{aaa: Y, Wallet{{payer: victim}}}} is rejected: {stolen:?}"
    ));

    let cycles_x_before = env.pic.cycle_balance(aaa_x);
    let op_id = top_up_call(
        &env,
        payments,
        stranger,
        aaa_x,
        PayPath::Wallet {
            payer: victim_account,
        },
    )
    .expect("the same allowance IS usable for the AAA it was scoped to");
    let op: Option<Op> = env.query(payments, stranger, "get_op", op_id);
    assert_eq!(op.unwrap().state, OpState::Done);
    let cycles_x_after = env.pic.cycle_balance(aaa_x);
    assert!(
        cycles_x_after > cycles_x_before,
        "top_up for X using victim's S(topup, X) allowance must succeed"
    );
    step(&format!(
        "the correctly-scoped call for X succeeds (op {op_id}), cycles {cycles_x_before} -> {cycles_x_after}"
    ));
}

#[test]
fn t5_6_spawn_survives_platform_stop_and_resumes_without_double_charge() {
    println!("T5.6 demo (acceptance #4): stopping platform mid-spawn strands the op at Notified; resume after restart completes it once");
    let env = IcpEnv::new();
    let admin = user(1);
    let alice = user(2);
    let owner = user(3);
    let (platform, payments) = install_wired(&env, admin);
    step("installed platform + payments (the AAA wasm is NOT approved yet, so register_aaa will stall)");

    env.mint_icp(alice, 20 * E8S);
    let quote: Result<Quote, ApiError> = env.query(payments, owner, "get_quote_spawn", ());
    let quote = quote.expect("get_quote_spawn");
    let deposit = deposit_account(&env, payments, Purpose::Spawn, owner);
    env.icrc1_transfer(alice, deposit, quote.total_e8s, None)
        .expect("fund the spawn deposit account");
    step(&format!(
        "funded owner's D(spawn, owner) deposit account with the quoted {} e8s",
        quote.total_e8s
    ));

    let first: Result<u64, ApiError> = env.update(
        payments,
        owner,
        "spawn_aaa",
        SpawnArgs {
            name: "ResilientRover".into(),
            avatar_seed: 3,
            path: PayPath::Deposit,
        },
    );
    assert!(
        first.is_err(),
        "register_aaa must fail: no AAA wasm is approved on platform yet"
    );
    step(&format!(
        "spawn_aaa fails mid-saga because no AAA wasm is approved yet: {first:?}"
    ));

    let op0: Option<Op> = env.query(payments, owner, "get_op", 0u64);
    let op0 = op0.expect("op 0 exists");
    let canister_id = match op0.state {
        OpState::Notified {
            canister_or_cycles: payments::journal::NotifiedInfo::Canister(id),
        } => id,
        other => panic!("expected op 0 stuck at Notified, got {other:?}"),
    };
    let cycles_after_cmc = env.pic.cycle_balance(canister_id);
    assert!(
        cycles_after_cmc > 0,
        "the CMC already minted the new AAA's cycles"
    );
    step(&format!(
        "op 0 is stuck at Notified{{{canister_id}}}: the CMC created and funded the AAA already"
    ));

    let deposit_balance_stuck = balance_of(&env, deposit);
    assert_eq!(
        deposit_balance_stuck, 0,
        "the deposit was already swept exactly once"
    );

    upload_and_approve_aaa_wasm(&env, platform, admin);
    step("the AAA wasm is now approved (the operator fixed the underlying outage)");

    env.pic
        .stop_canister(platform, Some(admin))
        .expect("stop platform");
    step("but before anyone resumes, an operator kills platform (stopped canister), simulating T5.6 acceptance #4");

    let resume_while_stopped: Result<(), ApiError> = env.update(payments, alice, "resume", 0u64);
    assert!(
        resume_while_stopped.is_err(),
        "resume(0) must fail while platform is stopped, got {resume_while_stopped:?}"
    );
    let op0_still_notified: Option<Op> = env.query(payments, owner, "get_op", 0u64);
    assert_eq!(
        op0_still_notified.unwrap().state,
        OpState::Notified {
            canister_or_cycles: payments::journal::NotifiedInfo::Canister(canister_id),
        }
    );
    let cycles_while_stopped = env.pic.cycle_balance(canister_id);
    assert_eq!(
        cycles_after_cmc, cycles_while_stopped,
        "no double charge: cycles must not change while resume fails against the stopped platform"
    );
    step(&format!(
        "resume(0), called by an unrelated principal, fails while platform is stopped: {resume_while_stopped:?}; op 0 stays at Notified"
    ));

    env.pic
        .start_canister(platform, Some(admin))
        .expect("restart platform");
    step("platform is restarted");

    let resumed: Result<(), ApiError> = env.update(payments, alice, "resume", 0u64);
    assert_eq!(resumed, Ok(()));
    let op0_done: Option<Op> = env.query(payments, owner, "get_op", 0u64);
    assert_eq!(op0_done.unwrap().state, OpState::Done);
    step("resume(0) completes the saga once platform is healthy again");

    let owner_lookup: Option<Principal> = env.query(platform, admin, "aaa_by_owner", owner);
    assert_eq!(owner_lookup, Some(canister_id));
    let cycles_final = env.pic.cycle_balance(canister_id);
    assert!(
        cycles_final <= cycles_after_cmc,
        "no double charge: the CMC minted cycles exactly once; resume only installs the wasm and never mints more (before={cycles_after_cmc} after={cycles_final})"
    );
    let deposit_balance_final = balance_of(&env, deposit);
    assert_eq!(deposit_balance_final, 0);
    step(&format!(
        "no double charge: AAA cycles only ever decreased from install costs ({cycles_after_cmc} -> {cycles_final}), never re-minted; deposit account still empty"
    ));

    let dup: Result<(), ApiError> = env.update(payments, alice, "resume", 0u64);
    assert_eq!(dup, Ok(()));
    step("a further duplicate resume(0) on a Done op is a no-op");
}

#[test]
fn t5_6_auto_topup_flags_a_revoked_allowance_without_moving_funds() {
    println!(
        "T5.6 demo (acceptance #5b): revoking the mandate's ledger allowance flags needs_attention"
    );
    let env = IcpEnv::new();
    let admin = user(1);
    let owner = user(3);
    let (platform, payments) = install_wired(&env, admin);
    let aaa = spawn_via_deposit(
        &env,
        platform,
        payments,
        admin,
        user(80),
        owner,
        "MandateRover",
    );

    let topup_e8s = E8S;
    let cap_30d_e8s = 10 * E8S;
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
    step("owner set an auto top-up mandate: 1 ICP per top-up, 10 ICP rolling 30-day cap");

    env.mint_icp(owner, 20 * E8S);
    let spender = Icrc1Account {
        owner: payments,
        subaccount: Some(deposit::spender_subaccount(Purpose::Auto, aaa)),
    };
    approve(&env, owner, spender, 5 * topup_e8s);
    step("owner approved an allowance to the mandate's spender subaccount");

    let cycles_before_ok = env.pic.cycle_balance(aaa);
    let op_id: Result<u64, ApiError> = env.update(payments, aaa, "request_auto_topup", ());
    let op_id = op_id.expect("first auto top-up should succeed");
    let op: Option<Op> = env.query(payments, owner, "get_op", op_id);
    assert_eq!(op.unwrap().state, OpState::Done);
    let cycles_after_ok = env.pic.cycle_balance(aaa);
    assert!(cycles_after_ok > cycles_before_ok);
    step("first auto top-up succeeds normally");

    env.pic
        .advance_time(std::time::Duration::from_secs(6 * 3_600 + 60));
    env.pic.tick();

    approve(&env, owner, spender, 0);
    step("owner revokes the ledger allowance (icrc2_approve amount = 0)");

    let cycles_before = env.pic.cycle_balance(aaa);
    let rejected: Result<u64, ApiError> = env.update(payments, aaa, "request_auto_topup", ());
    assert!(
        matches!(rejected, Err(ApiError::InvalidInput(_))),
        "a revoked allowance must reject request_auto_topup, got {rejected:?}"
    );
    let cycles_after = env.pic.cycle_balance(aaa);
    assert_eq!(
        cycles_before, cycles_after,
        "no cycles may move when the allowance was revoked"
    );
    step(&format!(
        "request_auto_topup with a revoked allowance is rejected: {rejected:?}"
    ));

    let view: Option<MandateView> = env.query(payments, owner, "get_mandate", aaa);
    let view = view.expect("mandate view exists");
    assert!(
        view.needs_attention,
        "the mandate must be flagged needs_attention after the revoked-allowance failure"
    );
    step("get_mandate reports needs_attention = true after the revoked allowance was hit");
}

fn cmc_account(canister: Principal) -> Icrc1Account {
    let mut sub = [0u8; 32];
    let bytes = canister.as_slice();
    sub[0] = bytes.len() as u8;
    sub[1..1 + bytes.len()].copy_from_slice(bytes);
    Icrc1Account {
        owner: CMC,
        subaccount: Some(sub as Subaccount),
    }
}

#[test]
fn t5_6_cmc_refunds_a_topup_notify_for_a_canister_that_does_not_exist() {
    println!("T5.6 demo (acceptance #6): the real CMC refunds notify_top_up for an invalid/nonexistent canister");
    let env = IcpEnv::new();
    let admin = user(1);
    let owner = user(3);
    let (platform, payments) = install_wired(&env, admin);
    let aaa = spawn_via_deposit(
        &env,
        platform,
        payments,
        admin,
        user(70),
        owner,
        "RefundRover",
    );
    step(&format!("registered a real AAA {aaa} via the deposit path"));

    let fake_aaa = user(250);
    let stranger = user(60);
    env.mint_icp(stranger, 5 * E8S);
    let top_up_amount = MIN_TOPUP_E8S + 3 * E8S;
    step(&format!(
        "fake_aaa {fake_aaa} was never registered by platform and is not a real canister on this subnet"
    ));

    let owned: Option<Principal> = env.query(platform, admin, "aaa_owner", fake_aaa);
    assert_eq!(owned, None, "fake_aaa is not a registered aaa on platform");

    let cmc_deposit = cmc_account(fake_aaa);
    let cycles_before = if env.pic.canister_exists(fake_aaa) {
        env.pic.cycle_balance(fake_aaa)
    } else {
        0
    };
    let block = env
        .icrc1_transfer(
            stranger,
            cmc_deposit,
            top_up_amount,
            Some(integration_tests::pic::MEMO_TOP_UP.to_le_bytes().to_vec()),
        )
        .expect("transfer the memoed TPUP amount straight to the CMC deposit account");
    let notified: Result<Nat, NotifyError> = env.notify_top_up(block, fake_aaa);
    step(&format!(
        "notify_top_up(block={block}, canister_id={fake_aaa}) against the real CMC -> {notified:?}"
    ));
    assert!(
        matches!(notified, Err(NotifyError::Refunded { .. })),
        "the CMC must refund a notify_top_up whose target is not a real canister, got {notified:?}"
    );
    if env.pic.canister_exists(fake_aaa) {
        let cycles_after = env.pic.cycle_balance(fake_aaa);
        assert_eq!(cycles_before, cycles_after);
    }
    step("DECISION: notify_top_up against a nonexistent canister id is refunded by the real CMC, matching payments::advance_topup_saga's Refunded handling");
}

#[test]
fn t5_6_concurrent_topups_for_the_same_aaa_cannot_both_pull_the_allowance() {
    println!("T5.6 demo (acceptance #7): two interleaved top_up calls for the same AAA cannot both pull the same allowance");
    let env = IcpEnv::new();
    let admin = user(1);
    let owner = user(3);
    let (platform, payments) = install_wired(&env, admin);
    let aaa = spawn_via_deposit(
        &env,
        platform,
        payments,
        admin,
        user(65),
        owner,
        "GuardedRover",
    );

    let payer = user(66);
    env.mint_icp(payer, 20 * E8S);
    let spender = Icrc1Account {
        owner: payments,
        subaccount: Some(deposit::spender_subaccount(Purpose::TopUp, aaa)),
    };
    let allowance = MIN_TOPUP_E8S + 3 * E8S;
    approve(&env, payer, spender, allowance);
    step(&format!(
        "payer approved a single allowance of {allowance} e8s to S(topup, aaa), enough for exactly one top_up"
    ));

    let payer_account = Account {
        owner: payer,
        subaccount: None,
    };
    let args = encode_one(TopUpArgs {
        aaa,
        path: PayPath::Wallet {
            payer: payer_account,
        },
    })
    .unwrap();

    let first_msg = env
        .pic
        .submit_call(payments, user(1), "top_up", args.clone())
        .expect("submit first top_up");
    let second_msg = env
        .pic
        .submit_call(payments, user(2), "top_up", args)
        .expect("submit second top_up before the first has finished");
    step("two top_up calls for the same aaa are submitted back-to-back before either completes");

    let first_bytes = env.pic.await_call(first_msg).expect("first top_up call");
    let second_bytes = env.pic.await_call(second_msg).expect("second top_up call");
    let first: Result<u64, ApiError> = decode_one(&first_bytes).unwrap();
    let second: Result<u64, ApiError> = decode_one(&second_bytes).unwrap();
    step(&format!("first -> {first:?}, second -> {second:?}"));

    let (winner, loser) = if first.is_ok() {
        (first, second)
    } else {
        (second, first)
    };
    let winner_id = winner.expect("exactly one of the two concurrent top_up calls must win");
    assert!(
        matches!(loser, Err(ApiError::Conflict(_))),
        "the losing concurrent call must be rejected by the CallerGuard, got {loser:?}"
    );
    let op: Option<Op> = env.query(payments, owner, "get_op", winner_id);
    assert_eq!(op.unwrap().state, OpState::Done);
    step("exactly one call wins and reaches Done; the other is rejected with Conflict before touching the ledger, so the allowance can never be pulled twice");
}
