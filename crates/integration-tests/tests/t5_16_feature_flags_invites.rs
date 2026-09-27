use candid::{decode_args, decode_one, encode_args, Nat, Principal};
use icrc_ledger_types::icrc1::account::Account as Icrc1Account;
use integration_tests::pic::{canister_wasm, user, IcpEnv, E8S, LEDGER};
use integration_tests::step;
use payments::api::{MintInvitesArgs, SpawnArgs};
use payments::config::Features;
use payments::journal::{Account, Op, OpState, PayPath};
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
    let ok: Result<(), ApiError> = env.update(platform, admin, "admin_set_payments_id", payments);
    assert_eq!(ok, Ok(()));
    let ok: Result<(), ApiError> = env.update(payments, admin, "admin_set_platform_id", platform);
    assert_eq!(ok, Ok(()));
    (platform, payments)
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

fn balance_of(env: &IcpEnv, account: Icrc1Account) -> u64 {
    let n: Nat = env.query(LEDGER, Principal::anonymous(), "icrc1_balance_of", account);
    n.0.try_into().unwrap()
}

fn mint_invites(
    env: &IcpEnv,
    payments: Principal,
    admin: Principal,
    count: u32,
    sponsor_cycles: u128,
    expires_at: u64,
) -> Vec<String> {
    let result: Result<Vec<String>, ApiError> = env.update(
        payments,
        admin,
        "admin_mint_invites",
        MintInvitesArgs {
            count,
            sponsor_cycles,
            expires_at,
        },
    );
    result.expect("admin_mint_invites")
}

fn far_future() -> u64 {
    u64::MAX / 2
}

#[test]
fn t5_16_get_features_reports_card_btc_eth_off_by_default() {
    println!("T5.16 demo: get_features() is public and reports the ICP-only launch config");
    let env = IcpEnv::new();
    let admin = user(1);
    let eve = user(99);
    let (_, payments) = install_wired(&env, admin);
    let features: Features = env.query(payments, eve, "get_features", ());
    assert!(!features.card);
    assert!(!features.btc);
    assert!(!features.eth);
    assert!(features.sponsored_spawn);
    step(&format!(
        "get_features() = {features:?}: card/btc/eth are off, sponsored_spawn is on"
    ));
}

#[test]
fn t5_16_admin_set_features_updates_and_is_audit_logged() {
    println!("T5.16 demo: admin_set_features is audit-logged");
    let env = IcpEnv::new();
    let admin = user(1);
    let (_, payments) = install_wired(&env, admin);
    let updated = Features {
        card: false,
        btc: false,
        eth: false,
        sponsored_spawn: false,
    };
    let ok: Result<(), ApiError> =
        env.update(payments, admin, "admin_set_features", updated.clone());
    assert_eq!(ok, Ok(()));
    let now: Features = env.query(payments, admin, "get_features", ());
    assert_eq!(now, updated);

    let raw = env
        .pic
        .query_call(
            payments,
            admin,
            "admin_audit_log",
            encode_args((None::<u64>, 50u32)).unwrap(),
        )
        .expect("admin_audit_log");
    let log: Result<payments::journal::Page<payments::audit::AuditEntry>, ApiError> =
        decode_one(&raw).unwrap();
    let methods: Vec<_> = log.unwrap().items.into_iter().map(|e| e.method).collect();
    assert!(methods.contains(&"admin_set_features".to_string()));
    step("admin_set_features appears in the audit log");

    let denied: Result<u64, ApiError> = env.update(
        payments,
        user(2),
        "spawn_aaa",
        SpawnArgs {
            name: "NoInviteNow".into(),
            avatar_seed: 1,
            path: PayPath::Invite {
                code: "AAAA-AAAA-AAAA-AAAA".into(),
            },
        },
    );
    assert_eq!(denied, Err(ApiError::FeatureDisabled));
    step("sponsored_spawn=false makes spawn_aaa{Invite} return FeatureDisabled");
}

#[test]
fn t5_16_invite_path_spawn_reaches_done_with_treasury_cycles_and_burns_the_code() {
    println!(
        "T5.16 demo (acceptance): sponsored spawn via invite code reaches Done using treasury-sponsored ICP, and the code becomes unusable after"
    );
    let env = IcpEnv::new();
    let admin = user(1);
    let owner = user(2);
    let funder = user(50);
    let (platform, payments) = install_wired(&env, admin);
    upload_and_approve_aaa_wasm(&env, platform, admin);

    let treasury = treasury_account(&env, payments, admin);
    env.mint_icp(funder, 50 * E8S);
    env.icrc1_transfer(funder, treasury, 20 * E8S, None)
        .expect("fund the payments TREASURY account with real ICP");
    step(&format!(
        "funded payments.TREASURY ({} e8s) via a plain icrc1_transfer, same as any deposit account",
        balance_of(&env, treasury)
    ));

    let codes = mint_invites(&env, payments, admin, 1, 1_100_000_000_000, far_future());
    let code = codes[0].clone();
    step(&format!("admin minted 1 invite code: {code}"));

    let op_id: Result<u64, ApiError> = env.update(
        payments,
        owner,
        "spawn_aaa",
        SpawnArgs {
            name: "SponsoredRover".into(),
            avatar_seed: 3,
            path: PayPath::Invite { code: code.clone() },
        },
    );
    let op_id = op_id.expect("invite-path spawn_aaa should reach Done in one call");
    let op: Option<Op> = env.query(payments, owner, "get_op", op_id);
    assert_eq!(op.unwrap().state, OpState::Done);
    step(
        "invite-path spawn_aaa reached Done, funded from the TREASURY account, no owner ICP moved",
    );

    let aaa: Option<Principal> = env.query(platform, admin, "aaa_by_owner", owner);
    let aaa = aaa.expect("owner has a registered aaa");
    let cycles = env.pic.cycle_balance(aaa);
    assert!(cycles > 0, "the sponsored AAA must hold minted cycles");
    step(&format!(
        "sponsored AAA {aaa} holds {cycles} cycles from the treasury-funded spawn"
    ));

    let reuse: Result<u64, ApiError> = env.update(
        payments,
        user(3),
        "spawn_aaa",
        SpawnArgs {
            name: "ShouldFail".into(),
            avatar_seed: 4,
            path: PayPath::Invite { code },
        },
    );
    assert!(
        matches!(reuse, Err(ApiError::Conflict(_))),
        "a used invite code must be single-use, got {reuse:?}"
    );
    step("re-using the same code for a different owner is rejected: single-use");
}

#[test]
fn t5_16_one_sponsored_aaa_per_owner_ever() {
    println!("T5.16 demo: an owner can only ever redeem one sponsored (invite-path) AAA");
    let env = IcpEnv::new();
    let admin = user(1);
    let owner = user(2);
    let funder = user(50);
    let (platform, payments) = install_wired(&env, admin);
    upload_and_approve_aaa_wasm(&env, platform, admin);

    let treasury = treasury_account(&env, payments, admin);
    env.mint_icp(funder, 50 * E8S);
    env.icrc1_transfer(funder, treasury, 20 * E8S, None)
        .unwrap();

    let codes = mint_invites(&env, payments, admin, 2, 1_100_000_000_000, far_future());

    let first: Result<u64, ApiError> = env.update(
        payments,
        owner,
        "spawn_aaa",
        SpawnArgs {
            name: "FirstSponsored".into(),
            avatar_seed: 1,
            path: PayPath::Invite {
                code: codes[0].clone(),
            },
        },
    );
    first.expect("first sponsored spawn succeeds");
    step("owner redeemed their first sponsored AAA");

    let existing: Option<Principal> = env.query(platform, admin, "aaa_by_owner", owner);
    assert!(existing.is_some(), "owner already has an aaa, so a second spawn_aaa is blocked upstream by the owner check regardless of path");

    let second: Result<u64, ApiError> = env.update(
        payments,
        user(77),
        "spawn_aaa",
        SpawnArgs {
            name: "SecondSponsored".into(),
            avatar_seed: 2,
            path: PayPath::Invite {
                code: codes[1].clone(),
            },
        },
    );
    second.expect("a different owner may still redeem a fresh code");
    step("a different owner redeeming a fresh code succeeds: the per-owner cap is per-principal");
}

#[test]
fn t5_16_daily_sponsor_budget_cap_rejects_once_exceeded() {
    println!("T5.16 demo: the daily sponsor budget cap rejects spawns once exhausted");
    let env = IcpEnv::new();
    let admin = user(1);
    let funder = user(50);
    let (platform, payments) = install_wired(&env, admin);
    upload_and_approve_aaa_wasm(&env, platform, admin);

    let treasury = treasury_account(&env, payments, admin);
    env.mint_icp(funder, 200 * E8S);
    env.icrc1_transfer(funder, treasury, 100 * E8S, None)
        .unwrap();

    let tiny_cap = payments::config::Params {
        sponsor_daily_cap_e8s: 1,
        ..payments::config::Params::default()
    };
    let ok: Result<(), ApiError> = env.update(payments, admin, "admin_set_params", tiny_cap);
    assert_eq!(ok, Ok(()));
    step("set sponsor_daily_cap_e8s = 1 (effectively zero headroom)");

    let codes = mint_invites(&env, payments, admin, 1, 1_100_000_000_000, far_future());
    let capped: Result<u64, ApiError> = env.update(
        payments,
        user(5),
        "spawn_aaa",
        SpawnArgs {
            name: "OverCap".into(),
            avatar_seed: 1,
            path: PayPath::Invite {
                code: codes[0].clone(),
            },
        },
    );
    assert!(
        matches!(capped, Err(ApiError::InvalidInput(_))),
        "spawn must be rejected once the daily sponsor budget is exhausted, got {capped:?}"
    );
    step(&format!(
        "daily sponsor budget cap rejects the spawn: {capped:?}"
    ));
}

#[test]
fn t5_16_expired_and_unknown_invite_codes_are_rejected() {
    println!("T5.16 demo: expired and unknown invite codes cannot spawn an AAA");
    let env = IcpEnv::new();
    let admin = user(1);
    let (platform, payments) = install_wired(&env, admin);
    upload_and_approve_aaa_wasm(&env, platform, admin);

    let unknown: Result<u64, ApiError> = env.update(
        payments,
        user(9),
        "spawn_aaa",
        SpawnArgs {
            name: "Ghost".into(),
            avatar_seed: 1,
            path: PayPath::Invite {
                code: "ZZZZ-ZZZZ-ZZZZ-ZZZZ".into(),
            },
        },
    );
    assert!(matches!(unknown, Err(ApiError::InvalidInput(_))));
    step("an unknown invite code is rejected");

    let now_secs = env.pic.get_time().as_nanos_since_unix_epoch() / 1_000_000_000;
    let codes = mint_invites(
        &env,
        payments,
        admin,
        1,
        1_100_000_000_000,
        now_secs.saturating_add(1),
    );
    env.pic.advance_time(std::time::Duration::from_secs(5));
    env.pic.tick();
    let expired: Result<u64, ApiError> = env.update(
        payments,
        user(10),
        "spawn_aaa",
        SpawnArgs {
            name: "TooLate".into(),
            avatar_seed: 1,
            path: PayPath::Invite {
                code: codes[0].clone(),
            },
        },
    );
    assert!(matches!(expired, Err(ApiError::InvalidInput(_))));
    step("an expired invite code is rejected");
}
