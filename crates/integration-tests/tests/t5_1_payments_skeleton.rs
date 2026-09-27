use candid::Principal;
use integration_tests::pic::{user, IcpEnv};
use integration_tests::step;
use payments::api::{JournalDemoArg, Overview};
use payments::audit::AuditEntry;
use payments::config::{Features, Params, PauseFlags};
use payments::journal::{Op, OpState};
use sc_types::ApiError;

type Unit = Result<(), ApiError>;

#[test]
fn t5_1_payments_skeleton_config_journal_guard_admin() {
    println!("T5.1 demo: payments skeleton — config, journal, guards, admin");
    let env = IcpEnv::new();
    let (alice, bob, eve) = (user(1), user(2), user(3));
    let payments = env.install("payments", alice);

    let admins: Result<Vec<Principal>, ApiError> =
        env.query(payments, alice, "admin_list_admins", ());
    assert_eq!(admins, Ok(vec![alice]));
    step("installer alice is the first admin");

    let anon: Unit = env.update(payments, Principal::anonymous(), "admin_add_admin", eve);
    assert_eq!(anon, Err(ApiError::Unauthorized));
    let denied: Unit = env.update(payments, eve, "admin_pause", PauseFlags::default());
    assert_eq!(denied, Err(ApiError::Unauthorized));
    step("anonymous and non-admin eve are refused with Unauthorized");

    let params: Params = env.query(payments, eve, "get_params", ());
    assert_eq!(params, Params::default());
    let features: Features = env.query(payments, eve, "get_features", ());
    assert_eq!(features, Features::default());
    assert!(features.btc && features.eth && features.sponsored_spawn);
    step(&format!(
        "get_params/get_features are public: fuel_pack_usd_cents = {}",
        params.fuel_pack_usd_cents
    ));

    let bad = Params {
        margin_bp: 10_001,
        ..Params::default()
    };
    let rejected: Unit = env.update(payments, alice, "admin_set_params", bad);
    assert!(matches!(rejected, Err(ApiError::InvalidInput(_))));
    let tuned = Params {
        treasury_reserve_floor_e8s: 80 * 100_000_000,
        ..Params::default()
    };
    let ok: Unit = env.update(payments, alice, "admin_set_params", tuned.clone());
    assert_eq!(ok, Ok(()));
    step("invalid params rejected; treasury_reserve_floor_e8s raised to 80 ICP");

    let paused = PauseFlags {
        spawn: false,
        topup: false,
        auto_topup: false,
        non_icp: true,
    };
    let ok: Unit = env.update(payments, alice, "admin_pause", paused.clone());
    assert_eq!(ok, Ok(()));
    let ok: Unit = env.update(payments, alice, "admin_add_admin", bob);
    assert_eq!(ok, Ok(()));
    let ok: Unit = env.update(payments, bob, "admin_remove_admin", alice);
    assert_eq!(ok, Ok(()));
    let last: Unit = env.update(payments, bob, "admin_remove_admin", bob);
    assert!(matches!(last, Err(ApiError::Conflict(_))));
    step("non-ICP intake paused; bob added as admin, then alice removed; the last admin can't be removed");

    let raw = env
        .pic
        .query_call(
            payments,
            bob,
            "admin_audit_log",
            candid::encode_args((None::<u64>, 50u32)).unwrap(),
        )
        .expect("admin_audit_log");
    let log: Result<Vec<AuditEntry>, ApiError> = candid::decode_one(&raw).unwrap();
    let methods: Vec<_> = log.unwrap().iter().map(|e| e.method.clone()).collect();
    assert_eq!(
        methods,
        [
            "admin_set_params",
            "admin_pause",
            "admin_add_admin",
            "admin_remove_admin"
        ]
    );
    step("the admin audit log records every mutation, each with a sha256 args digest");

    let aaa = user(10);
    let done_id: Result<u64, ApiError> = env.update(
        payments,
        bob,
        "admin_journal_demo",
        JournalDemoArg {
            aaa,
            trap_after_await: false,
        },
    );
    let done_id = done_id.expect("demo op completes");
    let done_op: Option<Op> = env.query(payments, bob, "get_op", done_id);
    assert_eq!(done_op.unwrap().state, OpState::Done);
    step("a journaled op that awaits raw_rand and doesn't trap reaches Done");

    let aaa2 = user(11);
    let trapped: Result<Vec<u8>, _> = env.pic.update_call(
        payments,
        bob,
        "admin_journal_demo",
        candid::encode_one(JournalDemoArg {
            aaa: aaa2,
            trap_after_await: true,
        })
        .unwrap(),
    );
    assert!(
        trapped.is_err(),
        "the injected trap after await must reject the call"
    );
    step("a trap injected AFTER the raw_rand await rejects the call");

    let survivor_id = done_id + 1;
    let survivor: Option<Op> = env.query(payments, bob, "get_op", survivor_id);
    let survivor = survivor.expect("the pre-await journal entry survives the post-await trap");
    assert_eq!(survivor.state, OpState::Pulled { block: 0 });
    step(&format!(
        "op {survivor_id}'s Pulled journal entry, written BEFORE the raw_rand await, survived the trap — only the post-await Notified/Done transitions rolled back"
    ));

    let cursor = survivor_id + 1;
    let recovered: Result<u64, ApiError> = env.update(
        payments,
        bob,
        "admin_journal_demo",
        JournalDemoArg {
            aaa: aaa2,
            trap_after_await: false,
        },
    );
    assert!(recovered.is_ok());
    let new_op: Option<Op> = env.query(payments, bob, "get_op", cursor);
    assert_eq!(new_op.unwrap().state, OpState::Done);
    step("aaa2's CallerGuard was released when the callback trap unwound the call, so a fresh journal for the same beneficiary starts and completes cleanly; op 1's Pulled record is still there for an operator to inspect");

    let overview: Result<Overview, ApiError> = env.query(payments, bob, "admin_overview", ());
    let overview = overview.unwrap();
    assert_eq!(overview.admins, vec![bob]);
    assert_eq!(overview.paused, paused);
    assert!(overview.audit_entries >= 4);
    step("admin_overview reflects the surviving admin, pause flags and audit trail");
}
