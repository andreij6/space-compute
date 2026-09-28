use candid::{decode_args, decode_one, encode_args, encode_one, Nat, Principal};
use icrc_ledger_types::icrc1::account::Account as Icrc1Account;
use icrc_ledger_types::icrc2::approve::{ApproveArgs, ApproveError};
use integration_tests::pic::{canister_wasm, user, IcpEnv, E8S, LEDGER};
use integration_tests::step;
use payments::api::{MintInvitesArgs, PaymentsOverview, SetMandateArgs, SpawnArgs};
use payments::config::{Features, Params, PauseFlags};
use payments::deposit::Purpose;
use payments::journal::{Account, NotifiedInfo, Op, OpState, PayPath};
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

fn treasury_account(env: &IcpEnv, payments: Principal, caller: Principal) -> Icrc1Account {
    let raw = env
        .pic
        .query_call(
            payments,
            caller,
            "get_treasury_account",
            encode_args(()).unwrap(),
        )
        .expect("get_treasury_account");
    let (_, account): (String, Account) = decode_args(&raw).unwrap();
    Icrc1Account {
        owner: account.owner,
        subaccount: account.subaccount,
    }
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
    let (_, account): (String, Account) = decode_args(&raw).unwrap();
    Icrc1Account {
        owner: account.owner,
        subaccount: account.subaccount,
    }
}

fn spawn_via_deposit(
    env: &IcpEnv,
    platform: Principal,
    payments: Principal,
    funder: Principal,
    owner: Principal,
    name: &str,
) -> (Principal, u64) {
    let quote: Result<Quote, ApiError> = env.query(payments, owner, "get_quote_spawn", ());
    let quote = quote.expect("get_quote_spawn");
    let deposit = deposit_account(env, payments, owner);
    env.icrc1_transfer(funder, deposit, quote.total_e8s, None)
        .expect("fund the spawn deposit account");
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
    let op_id = op_id.expect("spawn_aaa via deposit");
    let op: Option<Op> = env.query(payments, owner, "get_op", op_id);
    let op = op.expect("op exists");
    assert!(
        matches!(op.state, OpState::Done),
        "spawn op must settle Done: {op:?}"
    );
    (aaa_from_registration(env, platform, owner), op_id)
}

fn aaa_from_registration(env: &IcpEnv, platform: Principal, owner: Principal) -> Principal {
    let aaa: Option<Principal> = env.query(platform, owner, "aaa_by_owner", owner);
    aaa.expect("platform must have registered the spawned aaa")
}

fn spawn_via_invite(
    env: &IcpEnv,
    platform: Principal,
    payments: Principal,
    owner: Principal,
    code: &str,
    name: &str,
) -> (Principal, u64) {
    let bytes = env
        .pic
        .update_call(
            payments,
            owner,
            "spawn_aaa",
            encode_one(SpawnArgs {
                name: name.into(),
                avatar_seed: 2,
                path: PayPath::Invite { code: code.into() },
            })
            .unwrap(),
        )
        .expect("spawn_aaa rejected");
    let op_id: Result<u64, ApiError> = decode_one(&bytes).unwrap();
    let op_id = op_id.expect("spawn_aaa via invite");
    let aaa = aaa_from_registration(env, platform, owner);
    (aaa, op_id)
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

fn set_mandate(env: &IcpEnv, payments: Principal, owner: Principal, aaa: Principal) {
    let ok: Result<(), ApiError> = env.update(
        payments,
        owner,
        "set_mandate",
        SetMandateArgs {
            aaa,
            topup_e8s: 5_000_000,
            enabled: true,
            cap_30d_e8s: 50_000_000,
            payer: Account {
                owner,
                subaccount: None,
            },
        },
    );
    ok.expect("set_mandate");
}

fn request_auto_topup(env: &IcpEnv, payments: Principal, aaa: Principal) -> Result<u64, ApiError> {
    let raw = env
        .pic
        .update_call(payments, aaa, "request_auto_topup", encode_one(()).unwrap())
        .expect("request_auto_topup rejected at the ic level");
    decode_one(&raw).unwrap()
}

#[test]
fn t7_1_payments_v_n_minus_1_state_survives_upgrade_to_v_n() {
    println!("T7.1 demo: payments installed at the pinned vN-1 baseline, populated, upgraded to the current vN wasm — config, journal ops, mandates and invites all survive");
    let env = IcpEnv::new();
    let admin = user(1);
    let platform = env.install("platform", admin);
    let payments = env.install_baseline("payments", admin);
    for _ in 0..5 {
        env.pic.tick();
    }
    step("installed payments at the pinned vN-1 baseline commit, wired to a current-wasm platform");

    let ok: Result<(), ApiError> = env.update(platform, admin, "admin_set_payments_id", payments);
    assert_eq!(ok, Ok(()));
    let ok: Result<(), ApiError> = env.update(payments, admin, "admin_set_platform_id", platform);
    assert_eq!(ok, Ok(()));

    let stuck_alice = user(50);
    env.mint_icp(stuck_alice, 20 * E8S);
    let owner_stuck = user(100);
    let stuck_quote: Result<Quote, ApiError> =
        env.query(payments, owner_stuck, "get_quote_spawn", ());
    let stuck_deposit = deposit_account(&env, payments, owner_stuck);
    env.icrc1_transfer(
        stuck_alice,
        stuck_deposit,
        stuck_quote.unwrap().total_e8s,
        None,
    )
    .expect("fund the stuck owner's spawn deposit account");
    let stuck_result = env.pic.update_call(
        payments,
        owner_stuck,
        "spawn_aaa",
        encode_one(SpawnArgs {
            name: "Stuck-Saga".into(),
            avatar_seed: 9,
            path: PayPath::Deposit,
        })
        .unwrap(),
    );
    let stuck_result: Result<u64, ApiError> = decode_one(&stuck_result.unwrap()).unwrap();
    assert!(
        stuck_result.is_err(),
        "no AAA wasm is approved yet, so this spawn must stall mid-saga"
    );
    let stuck_id = 0u64;
    let stuck_before: Option<Op> = env.query(payments, admin, "get_op", stuck_id);
    let stuck_before = stuck_before.expect("the stalled saga's journal entry exists");
    let stuck_canister = match stuck_before.state {
        OpState::Notified {
            canister_or_cycles: NotifiedInfo::Canister(id),
        } => id,
        other => panic!("expected op {stuck_id} stuck at Notified{{canister}}, got {other:?}"),
    };
    step(&format!(
        "journal op #{stuck_id} is a real stalled saga: the CMC created AAA canister {stuck_canister} but platform.register_aaa couldn't install it (no approved wasm yet) — left for the resume-sweep timer"
    ));

    upload_and_approve_aaa_wasm(&env, platform, admin);
    step("admin uploaded and approved the AAA wasm on platform (the outage that stalled op 0 is over)");

    let mut features: Features = env.query(payments, admin, "get_features", ());
    features.btc = true;
    let ok: Result<(), ApiError> =
        env.update(payments, admin, "admin_set_features", features.clone());
    assert_eq!(ok, Ok(()));

    let mut params: Params = env.query(payments, admin, "get_params", ());
    params.margin_bp = 777;
    let ok: Result<(), ApiError> = env.update(payments, admin, "admin_set_params", params.clone());
    assert_eq!(ok, Ok(()));

    let extra_admin = user(9);
    let ok: Result<(), ApiError> = env.update(payments, admin, "admin_add_admin", extra_admin);
    assert_eq!(ok, Ok(()));

    let pause = PauseFlags {
        topup: false,
        auto_topup: false,
        spawn: false,
        non_icp: true,
    };
    let ok: Result<(), ApiError> = env.update(payments, admin, "admin_pause", pause.clone());
    assert_eq!(ok, Ok(()));
    step("config populated: features, params, an extra admin, and a pause flag");

    let alice = user(2);
    env.mint_icp(alice, 40 * E8S);
    let owner1 = user(101);
    let (aaa1, deposit_op) =
        spawn_via_deposit(&env, platform, payments, alice, owner1, "Baseline-Deposit");
    step(&format!(
        "journal op #{deposit_op}: spawn_aaa via Deposit settled Done, aaa registered at {aaa1}"
    ));

    let treasury = treasury_account(&env, payments, admin);
    env.icrc1_transfer(alice, treasury, 20 * E8S, None)
        .expect("fund the payments TREASURY account for sponsored invite spawns");

    let codes = {
        let r: Result<Vec<String>, ApiError> = env.update(
            payments,
            admin,
            "admin_mint_invites",
            MintInvitesArgs {
                count: 1,
                sponsor_cycles: 2_000_000_000_000,
                expires_at: u64::MAX / 2,
            },
        );
        r.expect("admin_mint_invites")
    };
    let owner2 = user(102);
    let (aaa2, invite_op) = spawn_via_invite(
        &env,
        platform,
        payments,
        owner2,
        &codes[0],
        "Baseline-Invite",
    );
    step(&format!(
        "journal op #{invite_op}: spawn_aaa via a minted invite settled, aaa registered at {aaa2}; the code is now spent"
    ));

    let reused = env
        .pic
        .update_call(
            payments,
            user(103),
            "spawn_aaa",
            encode_one(SpawnArgs {
                name: "Reuse-Attempt".into(),
                avatar_seed: 3,
                path: PayPath::Invite {
                    code: codes[0].clone(),
                },
            })
            .unwrap(),
        )
        .expect("spawn_aaa rejected at ic level");
    let reused: Result<u64, ApiError> = decode_one(&reused).unwrap();
    assert!(reused.is_err(), "a spent invite code must not spawn twice");

    set_mandate(&env, payments, owner1, aaa1);
    env.mint_icp(owner1, 5 * E8S);
    approve(&env, owner1, spender_account(payments, aaa1), 100_000_000);
    let auto_op =
        request_auto_topup(&env, payments, aaa1).expect("first auto top-up should succeed");
    let immediate_retry = request_auto_topup(&env, payments, aaa1);
    assert!(
        immediate_retry.is_err(),
        "the min-interval guard should reject an immediate second call"
    );
    step(&format!(
        "journal op #{auto_op}: mandate-backed auto top-up settled; the min-interval guard is armed"
    ));

    let stuck_still_before_upgrade: Option<Op> = env.query(payments, admin, "get_op", stuck_id);
    assert_eq!(
        stuck_still_before_upgrade.unwrap().state,
        OpState::Notified {
            canister_or_cycles: NotifiedInfo::Canister(stuck_canister)
        },
        "op {stuck_id} must still be stalled going into the upgrade (nothing resumed it yet)"
    );

    let mandate_before: MandateView = env
        .query::<_, Option<MandateView>>(payments, owner1, "get_mandate", aaa1)
        .expect("mandate exists");
    let overview_before: PaymentsOverview = {
        let r: Result<PaymentsOverview, ApiError> =
            env.update(payments, admin, "admin_overview", ());
        r.expect("admin_overview")
    };
    let ops_before: Vec<Op> = [deposit_op, invite_op, auto_op]
        .iter()
        .map(|id| {
            let op: Option<Op> = env.query(payments, admin, "get_op", *id);
            op.expect("op must exist before upgrade")
        })
        .collect();
    step(&format!(
        "captured pre-upgrade snapshot: {} settled ops plus 1 stuck op, mandate spent_30d={} e8s",
        ops_before.len(),
        mandate_before.spent_30d_e8s,
    ));

    env.pic
        .upgrade_canister(
            payments,
            canister_wasm("payments"),
            encode_one(()).unwrap(),
            Some(admin),
        )
        .expect("upgrade vN-1 -> vN must succeed");
    for _ in 0..5 {
        env.pic.tick();
    }
    step("upgraded payments from the vN-1 baseline to the current vN wasm");

    let features_after: Features = env.query(payments, admin, "get_features", ());
    assert_eq!(features_after, features);
    let params_after: Params = env.query(payments, admin, "get_params", ());
    assert_eq!(params_after.margin_bp, 777);
    let overview_after: PaymentsOverview = {
        let r: Result<PaymentsOverview, ApiError> =
            env.update(payments, admin, "admin_overview", ());
        r.expect("admin_overview")
    };
    assert_eq!(overview_after.admins, overview_before.admins);
    assert!(overview_after.admins.contains(&extra_admin));
    assert_eq!(overview_after.paused, pause);
    step("config (features, params, admins, pause flags) is intact after the upgrade");

    let mandate_after: MandateView = env
        .query::<_, Option<MandateView>>(payments, owner1, "get_mandate", aaa1)
        .expect("mandate must survive the upgrade");
    assert_eq!(mandate_after, mandate_before);

    for (before, id) in ops_before.iter().zip([deposit_op, invite_op, auto_op]) {
        let after: Option<Op> = env.query(payments, admin, "get_op", id);
        let after = after.expect("op must survive the upgrade");
        assert_eq!(&after, before, "op {id} must be byte-for-byte intact");
    }
    step("mandate and all three journal ops are byte-for-byte intact after the upgrade");

    let by_owner: payments::journal::Page<Op> = decode_one(
        &env.pic
            .query_call(
                payments,
                owner1,
                "list_ops_for_owner",
                encode_args((owner1, None::<u64>, 50u16)).unwrap(),
            )
            .expect("list_ops_for_owner"),
    )
    .unwrap();
    assert!(
        by_owner.items.iter().any(|op| op.id == deposit_op),
        "ops journaled by the vN-1 baseline (which had no owner index) must be backfilled into list_ops_for_owner"
    );
    let by_aaa: payments::journal::Page<Op> = decode_one(
        &env.pic
            .query_call(
                payments,
                owner1,
                "list_ops_for_aaa",
                encode_args((aaa1, None::<u64>, 50u16)).unwrap(),
            )
            .expect("list_ops_for_aaa"),
    )
    .unwrap();
    assert!(
        by_aaa.items.iter().any(|op| op.id == auto_op),
        "ops journaled by the vN-1 baseline (which had no aaa index) must be backfilled into list_ops_for_aaa"
    );
    step("the owner/aaa op indexes that did not exist in the baseline were backfilled on post_upgrade");

    let reused_after = env
        .pic
        .update_call(
            payments,
            user(104),
            "spawn_aaa",
            encode_one(SpawnArgs {
                name: "Reuse-Attempt-2".into(),
                avatar_seed: 4,
                path: PayPath::Invite {
                    code: codes[0].clone(),
                },
            })
            .unwrap(),
        )
        .expect("spawn_aaa rejected at ic level");
    let reused_after: Result<u64, ApiError> = decode_one(&reused_after).unwrap();
    assert!(
        reused_after.is_err(),
        "the invite must still be spent after the upgrade"
    );
    let retry_after_upgrade = request_auto_topup(&env, payments, aaa1);
    assert!(
        retry_after_upgrade.is_err(),
        "the mandate's min-interval / cap state must still be enforced after the upgrade"
    );
    step("invite-spent and mandate-guard state both survived: replays are still refused");

    let stuck_right_after_upgrade: Option<Op> = env.query(payments, admin, "get_op", stuck_id);
    assert_eq!(
        stuck_right_after_upgrade.unwrap().state,
        OpState::Notified {
            canister_or_cycles: NotifiedInfo::Canister(stuck_canister)
        },
        "the stuck op must not resolve on its own before the resume-sweep timer fires"
    );

    env.pic
        .advance_time(std::time::Duration::from_secs(300 + 30));
    for _ in 0..30 {
        env.pic.tick();
    }
    let stuck_after: Option<Op> = env.query(payments, admin, "get_op", stuck_id);
    let stuck_after = stuck_after.expect("stuck op still exists");
    assert_eq!(
        stuck_after.state,
        OpState::Done,
        "the 5-minute resume-sweep timer must resume after post_upgrade re-registers it and finish the stalled saga"
    );
    let stuck_record: Option<platform::registry::AaaRecord> =
        env.query(platform, admin, "get_aaa", stuck_canister);
    assert!(
        stuck_record.is_some(),
        "the stalled AAA must end up registered on platform once the timer resumes the saga"
    );
    step("resume-sweep timer resumed after upgrade: the stalled spawn saga reached Done and the AAA got registered, without a manual resume() call");
}
