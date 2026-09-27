use std::time::Duration;

use integration_tests::pic::{user, IcpEnv};
use integration_tests::step;
use sc_types::ApiError;

#[test]
fn t3_6_below_threshold_the_6h_timer_calls_request_auto_topup_exactly_once_per_interval() {
    println!("T3.6 demo: 03 §8.5 — below threshold, the 6h timer calls payments.request_auto_topup exactly once per interval, driven purely by IC time, not by tick volume");
    let env = IcpEnv::new();
    let owner = user(10);

    let init = aaa::config::AaaInit {
        owner,
        platform_id: user(90),
        payments_id: user(91),
        name: "TopupOnce-01".into(),
        avatar_seed: 1,
    };
    let aaa_id = env.install_with_arg("aaa", owner, init);
    step("installed a real AAA canister; payments_id points at a plain principal so every top-up attempt genuinely fails and is counted");

    let status0: Result<aaa::record::Status, ApiError> = env.query(aaa_id, owner, "status", ());
    assert_eq!(status0.unwrap().stats.auto_topup_failures, 0);

    let set_topup: Result<(), ApiError> = env.update(
        aaa_id,
        owner,
        "set_auto_topup",
        Some(1_000_000_000_000_000u128),
    );
    assert_eq!(set_topup, Ok(()));
    step("owner enabled auto top-up with a threshold above the canister's cycle balance");

    for _ in 0..30 {
        env.pic.tick();
    }
    let status_idle: Result<aaa::record::Status, ApiError> = env.query(aaa_id, owner, "status", ());
    assert_eq!(
        status_idle.unwrap().stats.auto_topup_failures,
        0,
        "ticking without advancing IC time must not fire the 6h timer early"
    );
    step("verified 30 ticks with no elapsed time triggered zero top-up attempts");

    for interval in 1..=3u64 {
        env.pic.advance_time(Duration::from_secs(6 * 3_600));
        for _ in 0..40 {
            env.pic.tick();
        }
        let status: Result<aaa::record::Status, ApiError> = env.query(aaa_id, owner, "status", ());
        let failures = status.unwrap().stats.auto_topup_failures;
        assert_eq!(
            failures, interval,
            "after {interval} elapsed 6h interval(s), request_auto_topup must have been attempted exactly {interval} time(s), got {failures}"
        );
        step(&format!(
            "interval {interval}: exactly {failures} auto-topup attempt(s) so far (acceptance criterion)"
        ));
    }

    for _ in 0..40 {
        env.pic.tick();
    }
    let status_final: Result<aaa::record::Status, ApiError> =
        env.query(aaa_id, owner, "status", ());
    assert_eq!(
        status_final.unwrap().stats.auto_topup_failures,
        3,
        "extra ticks without advancing time must not re-fire the timer within the same interval"
    );
    step("verified extra ticks inside the same interval did not double-count the attempt");
}
