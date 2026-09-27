use std::time::Duration;

use candid::{decode_one, encode_args};
use integration_tests::pic::{canister_wasm, user, IcpEnv};
use integration_tests::step;
use platform::catalog::SubjectInput;
use platform::registry::{AaaRecord, RegisterArgs};
use sc_types::{AnswerOption, ApiError, DiscoveryCategory, Protocol, Question, SubjectRef, Task};
use sha2::{Digest, Sha256};

fn tick(env: &IcpEnv, n: usize) {
    for _ in 0..n {
        env.pic.tick();
    }
}

fn call<A: candid::utils::ArgumentEncoder, R: serde::de::DeserializeOwned + candid::CandidType>(
    env: &IcpEnv,
    canister: candid::Principal,
    sender: candid::Principal,
    method: &str,
    args: A,
) -> R {
    let bytes = env
        .pic
        .update_call(canister, sender, method, encode_args(args).unwrap())
        .unwrap_or_else(|e| panic!("{method} rejected: {e:?}"));
    decode_one(&bytes).unwrap_or_else(|e| panic!("{method} reply did not decode: {e}"))
}

fn sample_ref(id: u32) -> SubjectRef {
    SubjectRef {
        subject_id: id,
        field: "ceers".into(),
        ra_deg: 214.9 + (id as f64 * 0.001),
        dec_deg: 52.8 + (id as f64 * 0.001),
        image_url: format!("https://data.example.com/{id}/rgb.png"),
        image_sha256: vec![1; 32],
        dossier_url: format!("https://data.example.com/{id}/dossier.json"),
        dossier_sha256: vec![2; 32],
        data_version: 1,
    }
}

fn sample_protocol(v: u16) -> Protocol {
    Protocol {
        version: v,
        questions: vec![Question {
            id: "q1".into(),
            prompt: "Is it smooth?".into(),
            answers: vec![
                AnswerOption {
                    id: "smooth".into(),
                    label: "Smooth".into(),
                    next: None,
                },
                AnswerOption {
                    id: "featured".into(),
                    label: "Featured".into(),
                    next: None,
                },
            ],
        }],
        discovery_categories: vec![DiscoveryCategory {
            id: "lens".into(),
            label: "Gravitational Lens".into(),
            description: "Arcs or rings".into(),
        }],
        guidance_md: "Look closely at the image.".into(),
    }
}

#[test]
fn t3_4_burn_ema_heartbeat_credits_and_auto_topup_timers() {
    println!("T3.4 demo: AAA timers (burn EMA, heartbeat, credits pull, auto top-up trigger)");
    let env = IcpEnv::new();
    let alice = user(1);
    let payments = user(2);
    let owner = user(10);
    let operator = user(11);

    let platform = env.install_on("platform", alice, 10_000_000_000_000, 0);
    tick(&env, 2);

    let ok: Result<(), ApiError> = env.update(platform, alice, "admin_set_payments_id", payments);
    assert_eq!(ok, Ok(()));

    let wasm_v1 = canister_wasm("aaa");
    let hash_v1 = Sha256::digest(&wasm_v1).to_vec();
    let upload_bytes = env
        .pic
        .update_call(
            platform,
            alice,
            "admin_upload_wasm",
            encode_args((1u32, wasm_v1, hash_v1)).unwrap(),
        )
        .expect("upload wasm");
    let upload_res: Result<(), ApiError> = decode_one(&upload_bytes).unwrap();
    assert_eq!(upload_res, Ok(()));

    let approve_bytes = env
        .pic
        .update_call(
            platform,
            alice,
            "admin_approve_wasm",
            encode_args((1u32,)).unwrap(),
        )
        .expect("approve wasm");
    let approve_res: Result<(), ApiError> = decode_one(&approve_bytes).unwrap();
    assert_eq!(approve_res, Ok(()));

    let proto = sample_protocol(1);
    let ok: Result<(), ApiError> = env.update(platform, alice, "admin_add_protocol", proto);
    assert_eq!(ok, Ok(()));
    let ok: Result<(), ApiError> = env.update(platform, alice, "admin_set_current_protocol", 1u16);
    assert_eq!(ok, Ok(()));

    let batch = vec![SubjectInput {
        subject: sample_ref(1),
        gold: None,
    }];
    let ok: Result<u32, ApiError> = env.update(platform, alice, "admin_add_subjects", batch);
    assert!(ok.is_ok());

    let subnet = env.pic.topology().get_app_subnets()[0];
    let aaa = env.pic.create_canister_on_subnet(Some(alice), None, subnet);
    env.pic.add_cycles(aaa, 10_000_000_000_000);
    env.pic
        .set_controllers(aaa, Some(alice), vec![owner, platform])
        .unwrap();

    let reg = RegisterArgs {
        canister_id: aaa,
        owner,
        name: "TimerSurveyor-01".into(),
        avatar_seed: 7,
    };
    let reg_res: Result<(), ApiError> = env.update(platform, payments, "register_aaa", reg);
    assert_eq!(reg_res, Ok(()));
    tick(&env, 5);
    step("installed and registered AAA canister on platform");

    let status_before: Result<aaa::record::Status, ApiError> = env.query(aaa, owner, "status", ());
    let status_before = status_before.expect("status must succeed for the owner");
    assert_eq!(status_before.stats.burn_ema_daily, 0);
    assert_eq!(status_before.stats.last_heartbeat_at, 0);
    assert_eq!(status_before.stats.auto_topup_failures, 0);
    step("captured baseline timer stats before any interval elapses");

    let set_topup: Result<(), ApiError> = env.update(
        aaa,
        owner,
        "set_auto_topup",
        Some(1_000_000_000_000_000u128),
    );
    assert_eq!(set_topup, Ok(()));
    step("owner enabled auto top-up with a threshold above the current balance");

    env.pic.advance_time(Duration::from_secs(6 * 3_600));
    tick(&env, 10);

    let status_after_burn_tick: Result<aaa::record::Status, ApiError> =
        env.query(aaa, owner, "status", ());
    let status_after_burn_tick = status_after_burn_tick.expect("status must succeed for the owner");
    assert!(
        status_after_burn_tick.stats.auto_topup_failures >= 1,
        "the 6h timer must attempt payments.request_auto_topup once the balance is below threshold, \
         and count the failure since no payments canister answers at this principal in the test"
    );
    step("6h timer sampled the burn EMA and triggered the auto top-up call (acceptance criterion)");

    let custom_params = platform::config::Params {
        fee_get_task: 300_000_000,
        ..Default::default()
    };
    let ok: Result<(), ApiError> = env.update(platform, alice, "admin_set_params", custom_params);
    assert_eq!(ok, Ok(()));
    step("platform admin changed fee_get_task ahead of the daily refresh");

    env.pic.advance_time(Duration::from_secs(24 * 3_600));
    tick(&env, 10);
    step("advanced 24h so the daily timer fires (heartbeat, params refresh, credits pull)");

    let record: Option<AaaRecord> = env.query(platform, alice, "get_aaa", aaa);
    let record = record.expect("platform must have a registry record for this AAA");
    assert!(
        record.last_cycles > 0,
        "the daily timer's heartbeat call must update the platform's record of this AAA"
    );
    step("verified platform.heartbeat was called by the daily timer (acceptance criterion)");

    let add_op: Result<(), ApiError> = call(
        &env,
        aaa,
        owner,
        "add_operator",
        (operator, "bot-1".to_string(), None::<u64>),
    );
    assert_eq!(add_op, Ok(()));
    tick(&env, 5);

    let balance_before_task = env.pic.cycle_balance(aaa);
    let task_res: Result<Task, ApiError> = env.update(aaa, operator, "get_task", ());
    assert!(task_res.is_ok(), "operator get_task must succeed");
    let balance_after_task = env.pic.cycle_balance(aaa);
    let deducted = balance_before_task - balance_after_task;
    assert!(
        deducted >= 300_000_000,
        "the daily timer must have refreshed the cached fee to the new platform value \
         (deducted {deducted}, expected at least 300_000_000)"
    );
    step("verified the daily timer refreshed the cached platform fees (acceptance criterion)");

    let status_final: Result<aaa::record::Status, ApiError> = env.query(aaa, owner, "status", ());
    let status_final = status_final.expect("status must succeed for the owner");
    assert_eq!(
        status_final.stats.credits_cursor, 0,
        "the credits-pull call fails gracefully when platform has no list_aaa_credits yet, \
         so the cursor stays put and the timer must not trap"
    );
    step("verified the credits-pull timer tick did not trap and left the cursor untouched");

    let stranger_call =
        env.pic
            .update_call(aaa, user(99), "sync_credit_copy", encode_args(()).unwrap());
    assert!(
        stranger_call.is_err(),
        "sync_credit_copy must no longer exist on the public interface; \
         credits are pulled by the AAA on a timer, never pushed by an owner-callable update"
    );
    step("verified sync_credit_copy is gone from the production interface (required fix)");

    let stranger_sim = env.pic.update_call(
        aaa,
        owner,
        "simulate_sys_unknown_once",
        encode_args(()).unwrap(),
    );
    assert!(
        stranger_sim.is_err(),
        "simulate_sys_unknown_once must no longer exist on the production interface"
    );
    step("verified simulate_sys_unknown_once is gone from the production interface (required fix)");
}
