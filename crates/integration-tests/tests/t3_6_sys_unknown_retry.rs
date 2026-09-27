use candid::{decode_one, encode_args, encode_one};
use integration_tests::pic::{canister_wasm, user, IcpEnv};
use integration_tests::step;
use platform::catalog::SubjectInput;
use platform::registry::RegisterArgs;
use sc_types::{
    Answer, AnswerOption, ApiError, ClassificationReceipt, ClassificationSubmission,
    DiscoveryCategory, Protocol, Question, SubjectRef, Task,
};
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
fn t3_6_submit_retried_after_sys_unknown_produces_one_classification_and_one_record() {
    println!("T3.6 demo: 03 §8.3 — a submit retried after a real SYS_UNKNOWN produces exactly one platform classification and one local record");
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
        .set_controllers(aaa, Some(alice), vec![owner, platform, alice])
        .unwrap();

    let reg = RegisterArgs {
        canister_id: aaa,
        owner,
        name: "SysUnknownSurveyor-01".into(),
        avatar_seed: 42,
    };
    let reg_res: Result<(), ApiError> = env.update(platform, payments, "register_aaa", reg);
    assert_eq!(reg_res, Ok(()));
    tick(&env, 5);
    step("installed and registered AAA canister on platform");

    let add_op: Result<(), ApiError> = call(
        &env,
        aaa,
        owner,
        "add_operator",
        (operator, "bot-1".to_string(), None::<u64>),
    );
    assert_eq!(add_op, Ok(()));
    tick(&env, 5);
    step("owner added operator; synced with platform");

    let task_res: Result<Task, ApiError> = env.update(aaa, operator, "get_task", ());
    assert!(task_res.is_ok(), "operator get_task must succeed");
    let task = task_res.unwrap();
    step("operator fetched a task via the AAA relay");

    let submission = ClassificationSubmission {
        task_id: task.task_id,
        answers: vec![Answer {
            question_id: "q1".into(),
            answer_id: "smooth".into(),
        }],
        observed_image_sha256: vec![1; 32],
        discovery: None,
        agent_label: None,
        submitted_by: operator,
    };

    env.pic
        .stop_canister(platform, Some(alice))
        .expect("stop platform to force a genuine reject on the inflight submit call");
    step("stopped the platform canister to force a real IC reject, not a debug backdoor");

    let msg_id = env
        .pic
        .submit_call(
            aaa,
            operator,
            "submit_classification",
            encode_one(&submission).unwrap(),
        )
        .expect("submit_classification enqueued");

    env.pic.tick();
    step("let the first attempt reach the stopped platform and reject");

    env.pic
        .start_canister(platform, Some(alice))
        .expect("restart platform before the AAA's retry lands");
    step("restarted platform so the AAA's SYS_UNKNOWN-triggered retry can succeed");

    let mut outcome = None;
    for _ in 0..60 {
        env.pic.tick();
        if let Some(r) = env.pic.ingress_status(msg_id.clone()) {
            outcome = Some(r);
            break;
        }
    }
    let bytes = outcome
        .expect("submit_classification did not settle in 60 rounds")
        .unwrap_or_else(|e| {
            panic!("submit_classification must ultimately succeed via retry, got reject: {e:?}")
        });
    let submit_res: Result<ClassificationReceipt, ApiError> = decode_one(&bytes).unwrap();
    let receipt = submit_res.expect("submit_classification must succeed after the retry");
    assert!(
        !receipt.duplicate,
        "the retried submission must not be reported as a duplicate"
    );
    step("submit_classification succeeded after a real reject-then-retry round trip");

    let rec_1: Result<Option<aaa::record::Record>, ApiError> =
        env.query(aaa, owner, "get_record", 1u64);
    assert!(
        rec_1.expect("owner get_record must succeed").is_some(),
        "exactly one local record must exist"
    );
    let rec_2: Result<Option<aaa::record::Record>, ApiError> =
        env.query(aaa, owner, "get_record", 2u64);
    assert!(
        rec_2.expect("owner get_record must succeed").is_none(),
        "the retry must not create a second local record"
    );
    step("verified exactly ONE local record after the SYS_UNKNOWN-triggered retry");

    let plat_class_1: Option<platform::scoring::Classification> =
        env.query(platform, alice, "get_classification", 1u64);
    assert!(
        plat_class_1.is_some(),
        "exactly one platform classification must exist"
    );
    let plat_class_2: Option<platform::scoring::Classification> =
        env.query(platform, alice, "get_classification", 2u64);
    assert!(
        plat_class_2.is_none(),
        "the retry must not produce a second platform classification"
    );
    step("verified exactly ONE platform classification after the SYS_UNKNOWN-triggered retry (acceptance criterion)");
}
