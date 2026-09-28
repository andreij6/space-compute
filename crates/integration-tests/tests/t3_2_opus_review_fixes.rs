use std::time::Duration;

use candid::encode_one;
use integration_tests::step;
use integration_tests::world::World;
use sc_types::{ApiError, ClassificationReceipt, Task};

#[test]
fn t3_2_duplicate_submit_after_success_is_not_charged_again() {
    println!("T3.2 review fix: a retried submit that the platform answers as a duplicate gets its fee refunded");
    let w = World::new("DupFee-01", 1);
    let params: aaa::params::PlatformParams = w.env.query(w.platform, w.admin, "get_params", ());
    let fee = params.fee_submit_classification as i128;
    let task = w.get_task();
    let sub = w.submission(&task);
    let balance = |c| w.env.pic.cycle_balance(c) as i128;

    let (p0, a0) = (balance(w.platform), balance(w.aaa));
    let first: Result<ClassificationReceipt, ApiError> =
        w.env
            .update(w.aaa, w.operator, "submit_classification", sub.clone());
    assert!(!first.unwrap().duplicate);
    let (p1, a1) = (balance(w.platform), balance(w.aaa));
    assert!(p1 - p0 > fee / 2, "first submit pays the fee");
    assert!(a0 - a1 > fee / 2);
    step("first submit charged the fee");

    let second: Result<ClassificationReceipt, ApiError> =
        w.env
            .update(w.aaa, w.operator, "submit_classification", sub);
    assert!(second.unwrap().duplicate);
    let (p2, a2) = (balance(w.platform), balance(w.aaa));
    assert!(
        p2 - p1 < fee / 2,
        "platform must not keep a duplicate's fee: gained {}",
        p2 - p1
    );
    assert!(
        a1 - a2 < fee / 2,
        "AAA must get the duplicate's fee back: spent {}",
        a1 - a2
    );
    step("duplicate submit returned the original receipt and its cycles were refunded");
}

#[test]
fn t3_2_concurrent_submits_for_one_task_forward_once() {
    println!(
        "T3.2 review fix: two concurrent submits of the same task_id forward to the platform once"
    );
    let w = World::new("Concurrent-01", 1);
    let task = w.get_task();
    let sub = encode_one(w.submission(&task)).unwrap();
    let m1 = w
        .env
        .pic
        .submit_call(w.aaa, w.operator, "submit_classification", sub.clone())
        .unwrap();
    let m2 = w
        .env
        .pic
        .submit_call(w.aaa, w.operator, "submit_classification", sub)
        .unwrap();
    let r1: Result<ClassificationReceipt, ApiError> = w.await_ingress(m1);
    let r2: Result<ClassificationReceipt, ApiError> = w.await_ingress(m2);
    let (won, lost) = if r1.is_ok() { (r1, r2) } else { (r2, r1) };
    assert!(!won.unwrap().duplicate);
    assert!(
        matches!(lost, Err(ApiError::Conflict(_))),
        "the other concurrent submit must be refused while the first is in flight, got {lost:?}"
    );
    let second: Option<aaa::record::Record> = w
        .env
        .query::<_, Result<Option<aaa::record::Record>, ApiError>>(
            w.aaa,
            w.owner,
            "get_record",
            2u64,
        )
        .unwrap();
    assert!(second.is_none());
    step("one forwarded submit, one Conflict, one local record");
}

#[test]
fn t3_2_low_cycles_requests_auto_topup_only_when_enabled() {
    println!("T3.2 review fix: the low-cycles guard calls payments.request_auto_topup only when auto top-up is enabled");
    let w = World::new("LowFuel-01", 1);
    let mut params: platform::config::Params = w.env.query(w.platform, w.admin, "get_params", ());
    params.fee_get_task = 4_000_000_000_000;
    let ok: Result<(), ApiError> = w
        .env
        .update(w.platform, w.admin, "admin_set_params", params);
    assert_eq!(ok, Ok(()));
    w.env.pic.advance_time(Duration::from_secs(24 * 3_600));
    w.tick(40);
    step("platform raised fee_get_task; the AAA's daily refresh put its balance under the low-cycles threshold");
    let (env, aaa_id, owner) = (&w.env, w.aaa, w.owner);
    let failures = || {
        let s: Result<aaa::record::Status, ApiError> = env.query(aaa_id, owner, "status", ());
        s.unwrap().stats.auto_topup_failures
    };
    assert_eq!(failures(), 0);

    let r: Result<Task, ApiError> = env.update(aaa_id, owner, "get_task", ());
    assert!(
        matches!(r, Err(ApiError::Internal(ref m)) if m.contains("low cycles")),
        "{r:?}"
    );
    assert_eq!(failures(), 0, "auto top-up disabled: no payments call");
    step("auto top-up off: low-cycles refusal without calling payments");

    let ok: Result<(), ApiError> = env.update(aaa_id, owner, "set_auto_topup", Some(1u128));
    assert_eq!(ok, Ok(()));
    let r: Result<Task, ApiError> = env.update(aaa_id, owner, "get_task", ());
    assert!(r.is_err());
    assert_eq!(failures(), 1, "auto top-up enabled: payments was called");
    step("auto top-up on: the guard requested a top-up");
}

#[test]
fn t3_1_set_profile_is_owner_only_and_pushes_to_platform() {
    println!("T3.1 review fix: set_profile (03 §4.2) is owner-only and forwards to platform.update_aaa_profile");
    let w = World::new("Profile-01", 1);
    let args = aaa::api::ProfileUpdate {
        name: Some("Renamed-02".into()),
        avatar_seed: Some(7),
    };
    let denied: Result<(), ApiError> = w.env.update(w.aaa, w.operator, "set_profile", args.clone());
    assert_eq!(denied, Err(ApiError::Unauthorized));
    let bad: Result<(), ApiError> = w.env.update(
        w.aaa,
        w.owner,
        "set_profile",
        aaa::api::ProfileUpdate {
            name: Some("x".into()),
            avatar_seed: None,
        },
    );
    assert!(matches!(bad, Err(ApiError::InvalidInput(_))));
    step("operator refused; invalid name refused locally");

    let ok: Result<(), ApiError> = w.env.update(w.aaa, w.owner, "set_profile", args);
    assert_eq!(ok, Ok(()));
    let rec: Option<platform::registry::AaaRecord> =
        w.env.query(w.platform, w.admin, "get_aaa", w.aaa);
    let rec = rec.unwrap();
    assert_eq!((rec.name.as_str(), rec.avatar_seed), ("Renamed-02", 7));
    let public: aaa::record::PublicStatus = w.env.query(w.aaa, w.owner, "status_public", ());
    assert_eq!(public.name, "Renamed-02");
    step("owner renamed the AAA on both the platform and locally");
}
