use candid::{decode_one, encode_args, encode_one, Principal};
use integration_tests::pic::{canister_wasm, user, IcpEnv};
use integration_tests::step;
use platform::api::Overview;
use platform::audit::AuditEntry;
use platform::config::{Params, PauseFlags};
use sc_types::ApiError;

type Unit = Result<(), ApiError>;

fn tick(env: &IcpEnv, n: usize) {
    for _ in 0..n {
        env.pic.tick();
    }
}

#[test]
fn t2_1_upgrade_keeps_config_admins_audit_log_and_reseeds_rng() {
    println!("T2.1 demo: platform skeleton — config/admin, memory map, timers, RNG");
    let env = IcpEnv::new();
    let (alice, bob, eve) = (user(1), user(2), user(3));
    let platform = env.install("platform", alice);

    let admins: Result<Vec<Principal>, ApiError> =
        env.query(platform, alice, "admin_list_admins", ());
    assert_eq!(admins, Ok(vec![alice]));
    step("installer alice is the first admin");

    let denied: Unit = env.update(platform, eve, "admin_pause", PauseFlags::default());
    assert_eq!(denied, Err(ApiError::Unauthorized));
    let anon: Unit = env.update(platform, Principal::anonymous(), "admin_add_admin", eve);
    assert_eq!(anon, Err(ApiError::Unauthorized));
    step("eve and anonymous are refused with Unauthorized");

    let params: Params = env.query(platform, eve, "get_params", ());
    assert_eq!(params, Params::default());
    step(&format!(
        "get_params is public: fee_get_task = {} cycles, retire_after_k = {}",
        params.fee_get_task, params.retire_after_k
    ));

    tick(&env, 5);
    let before: Result<Overview, ApiError> = env.query(platform, alice, "admin_overview", ());
    let seeded_first = before
        .unwrap()
        .rng_seeded_at
        .expect("rng seeded by the init timer");
    step("the init timer seeded the ChaCha20 RNG from raw_rand");

    let bad = Params {
        reviews_min: 9,
        ..Params::default()
    };
    let rejected: Unit = env.update(platform, alice, "admin_set_params", bad);
    assert!(matches!(rejected, Err(ApiError::InvalidInput(_))));
    let tuned = Params {
        fee_get_task: 70_000_000,
        retire_after_k: 6,
        ..Params::default()
    };
    let ok: Unit = env.update(platform, alice, "admin_set_params", tuned.clone());
    assert_eq!(ok, Ok(()));
    step("invalid params rejected; fee_get_task → 70M and retire_after_k → 6 accepted");

    let ok: Unit = env.update(platform, alice, "admin_add_admin", bob);
    assert_eq!(ok, Ok(()));
    let paused = PauseFlags {
        tasks: true,
        reviews: false,
        spawns: true,
    };
    let ok: Unit = env.update(platform, bob, "admin_pause", paused.clone());
    assert_eq!(ok, Ok(()));
    let ok: Unit = env.update(platform, bob, "admin_remove_admin", alice);
    assert_eq!(ok, Ok(()));
    let last: Unit = env.update(platform, bob, "admin_remove_admin", bob);
    assert!(matches!(last, Err(ApiError::Conflict(_))));
    step("bob added, pauses tasks+spawns, removes alice; removing the last admin is refused");

    let raw = env
        .pic
        .query_call(
            platform,
            bob,
            "admin_audit_log",
            encode_args((None::<u64>, 50u32)).unwrap(),
        )
        .expect("admin_audit_log");
    let log: Result<Vec<AuditEntry>, ApiError> = decode_one(&raw).unwrap();
    let log = log.unwrap();
    let methods: Vec<_> = log.iter().map(|e| e.method.as_str()).collect();
    assert_eq!(
        methods,
        [
            "admin_set_params",
            "admin_add_admin",
            "admin_pause",
            "admin_remove_admin"
        ]
    );
    assert!(log.iter().all(|e| e.args_digest.len() == 32));
    step(&format!(
        "audit log holds {} entries, each with a sha256 args digest",
        log.len()
    ));

    env.pic.advance_time(std::time::Duration::from_secs(60));
    env.pic
        .upgrade_canister(
            platform,
            canister_wasm("platform"),
            encode_one(()).unwrap(),
            Some(alice),
        )
        .expect("upgrade");
    tick(&env, 5);
    step("upgraded the platform canister");

    let after: Overview = env
        .query::<_, Result<Overview, ApiError>>(platform, bob, "admin_overview", ())
        .unwrap();
    assert_eq!(after.params, tuned);
    assert_eq!(after.admins, vec![bob]);
    assert_eq!(after.paused, paused);
    assert_eq!(after.audit_entries, 4);
    assert!(after.rng_seeded_at.unwrap() > seeded_first);
    step("after upgrade: params, admins, pause flags and the 4 audit entries are intact; RNG reseeded");
}
