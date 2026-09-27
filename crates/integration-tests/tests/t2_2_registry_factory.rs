use candid::{decode_one, encode_args, Principal};
use integration_tests::pic::{canister_wasm, user, IcpEnv};
use integration_tests::step;
use platform::registry::{
    AaaRecord, AaaStatus, CheckNameResult, Heartbeat, OperatorSetInput, RegisterArgs,
    UpdateAaaProfileArgs,
};
use sc_types::ApiError;
use sha2::{Digest, Sha256};

fn tick(env: &IcpEnv, n: usize) {
    for _ in 0..n {
        env.pic.tick();
    }
}

fn upload_wasm(
    env: &IcpEnv,
    platform: Principal,
    admin: Principal,
    version: u32,
    blob: Vec<u8>,
    sha256: Vec<u8>,
) -> Result<(), ApiError> {
    let bytes = env
        .pic
        .update_call(
            platform,
            admin,
            "admin_upload_wasm",
            encode_args((version, blob, sha256)).unwrap(),
        )
        .expect("upload wasm");
    decode_one(&bytes).unwrap()
}

#[test]
fn t2_2_spawned_aaa_verified_and_upgrade_works() {
    println!("T2.2 demo: Registry & factory — register/install/verify/upgrade/profile");
    let env = IcpEnv::new();
    let alice = user(1);
    let bob = user(2);
    let charlie = user(3);
    let payments = user(99);

    let platform = env.install("platform", alice);
    let ok: Result<(), ApiError> = env.update(platform, alice, "admin_set_payments_id", payments);
    assert_eq!(ok, Ok(()));
    step("installed platform canister with admin alice and configured payments principal");

    let wasm_v1 = canister_wasm("aaa");
    let hash_v1 = Sha256::digest(&wasm_v1).to_vec();
    upload_wasm(&env, platform, alice, 1, wasm_v1.clone(), hash_v1.clone()).unwrap();
    let ok: Result<(), ApiError> = env.update(platform, alice, "admin_approve_wasm", 1u32);
    assert_eq!(ok, Ok(()));
    step("admin alice uploaded and approved AAA wasm v1");

    let check_ok: CheckNameResult =
        env.query(platform, bob, "check_name", "OrionSurveyor-01".to_string());
    assert_eq!(check_ok, CheckNameResult::Ok);
    let check_bad: CheckNameResult = env.query(platform, bob, "check_name", "bad!".to_string());
    assert_eq!(check_bad, CheckNameResult::Invalid);
    step("check_name query: OrionSurveyor-01 is Ok, invalid charset is Invalid");

    let subnet = env.pic.topology().get_app_subnets()[0];
    let aaa_bob = env.pic.create_canister_on_subnet(Some(alice), None, subnet);
    env.pic.add_cycles(aaa_bob, 5_000_000_000_000);
    env.pic
        .set_controllers(aaa_bob, Some(alice), vec![bob, platform])
        .unwrap();

    let reg_args = RegisterArgs {
        canister_id: aaa_bob,
        owner: bob,
        name: "OrionSurveyor-01".into(),
        avatar_seed: 42,
    };
    let ok: Result<(), ApiError> = env.update(platform, payments, "register_aaa", reg_args.clone());
    assert_eq!(ok, Ok(()));
    tick(&env, 5);

    let record: Option<AaaRecord> = env.query(platform, alice, "get_aaa", aaa_bob);
    let record = record.expect("registered record");
    assert_eq!(record.status, AaaStatus::Active);
    assert_eq!(record.name, "OrionSurveyor-01");
    assert_eq!(record.wasm_version, 1);
    assert!(record.platform_is_controller);
    assert_eq!(record.owner, bob);

    let owner_lookup: Option<Principal> = env.query(platform, alice, "aaa_by_owner", bob);
    assert_eq!(owner_lookup, Some(aaa_bob));
    let canister_owner: Option<Principal> = env.query(platform, alice, "aaa_owner", aaa_bob);
    assert_eq!(canister_owner, Some(bob));

    let check_taken: CheckNameResult = env.query(
        platform,
        alice,
        "check_name",
        "OrionSurveyor-01".to_string(),
    );
    assert_eq!(check_taken, CheckNameResult::Taken);
    step("spawned AAA for bob: status is Active, controllers verified, name is Taken");

    let idemp: Result<(), ApiError> =
        env.update(platform, payments, "register_aaa", reg_args.clone());
    assert_eq!(idemp, Ok(()));

    let duplicate_canister = env.pic.create_canister_on_subnet(Some(alice), None, subnet);
    env.pic
        .set_controllers(duplicate_canister, Some(alice), vec![bob, platform])
        .unwrap();
    let conflict_args = RegisterArgs {
        canister_id: duplicate_canister,
        owner: bob,
        name: "OrionSurveyor-Another".into(),
        avatar_seed: 43,
    };
    let conflict: Result<(), ApiError> =
        env.update(platform, payments, "register_aaa", conflict_args);
    assert!(matches!(conflict, Err(ApiError::Conflict(_))));
    step("register_aaa is idempotent on canister_id; second live AAA for same owner rejected");

    let aaa_charlie = env.pic.create_canister_on_subnet(Some(alice), None, subnet);
    env.pic.add_cycles(aaa_charlie, 5_000_000_000_000);
    env.pic
        .set_controllers(aaa_charlie, Some(alice), vec![charlie, platform])
        .unwrap();
    let collision_args = RegisterArgs {
        canister_id: aaa_charlie,
        owner: charlie,
        name: "OrionSurveyor-01".into(),
        avatar_seed: 100,
    };
    let ok: Result<(), ApiError> = env.update(platform, payments, "register_aaa", collision_args);
    assert_eq!(ok, Ok(()));
    tick(&env, 5);

    let charlie_record: Option<AaaRecord> = env.query(platform, alice, "get_aaa", aaa_charlie);
    assert_eq!(charlie_record.unwrap().name, "OrionSurveyor-01-2");
    step("competing registration with same name automatically assigned suffix -2");

    let hb: Result<(), ApiError> = env.update(
        platform,
        aaa_bob,
        "heartbeat",
        Heartbeat {
            cycles: 4_500_000_000_000,
            wasm_version: 1,
        },
    );
    assert_eq!(hb, Ok(()));
    let after_hb = env
        .query::<_, Option<AaaRecord>>(platform, alice, "get_aaa", aaa_bob)
        .unwrap();
    assert_eq!(after_hb.last_cycles, 4_500_000_000_000);

    let sync_res: Result<(), ApiError> = env.update(
        platform,
        aaa_bob,
        "sync_operators",
        OperatorSetInput {
            operators: vec![(user(5), None)],
        },
    );
    assert_eq!(sync_res, Ok(()));

    let profile_res: Result<(), ApiError> = env.update(
        platform,
        aaa_bob,
        "update_aaa_profile",
        UpdateAaaProfileArgs {
            name: Some("OrionSurveyor-Prime".into()),
            avatar_seed: Some(777),
        },
    );
    assert_eq!(profile_res, Ok(()));
    let renamed = env
        .query::<_, Option<AaaRecord>>(platform, alice, "get_aaa", aaa_bob)
        .unwrap();
    assert_eq!(renamed.name, "OrionSurveyor-Prime");
    assert_eq!(renamed.avatar_seed, 777);

    assert_eq!(
        env.query::<_, CheckNameResult>(
            platform,
            alice,
            "check_name",
            "OrionSurveyor-01".to_string()
        ),
        CheckNameResult::Ok
    );
    step("heartbeat, sync_operators and profile rename succeed; rename frees the old name");

    let verify_res =
        env.pic
            .update_call(platform, alice, "verify", encode_args((aaa_bob,)).unwrap());
    assert!(verify_res.is_err());
    step("verify is internal (register/upgrade/AAA calls), not a public endpoint");

    let wasm_v2 = canister_wasm("spike-probe");
    let hash_v2 = Sha256::digest(&wasm_v2).to_vec();
    upload_wasm(&env, platform, alice, 2, wasm_v2, hash_v2).unwrap();
    env.update::<_, Result<(), ApiError>>(platform, alice, "admin_approve_wasm", 2u32)
        .unwrap();

    let unauth: Result<(), ApiError> = env.update(platform, charlie, "upgrade_aaa", aaa_bob);
    assert_eq!(unauth, Err(ApiError::Unauthorized));

    let upgraded: Result<(), ApiError> = env.update(platform, bob, "upgrade_aaa", aaa_bob);
    assert_eq!(upgraded, Ok(()));
    tick(&env, 5);

    let upgraded_record = env
        .query::<_, Option<AaaRecord>>(platform, alice, "get_aaa", aaa_bob)
        .unwrap();
    assert_eq!(upgraded_record.wasm_version, 2);
    assert_eq!(upgraded_record.status, AaaStatus::Active);
    step("wasm v2 approved, bob upgrades AAA successfully, provenance confirms v2");
}
