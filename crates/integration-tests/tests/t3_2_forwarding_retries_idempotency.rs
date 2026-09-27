use candid::{decode_one, encode_args};
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
fn t3_2_one_record_per_task_under_retry() {
    println!("T3.2 demo: AAA forwarding w/ fees, retries, idempotency, low-cycles guard");
    let env = IcpEnv::new();
    let alice = user(1);
    let payments = user(2);
    let owner = user(10);
    let operator = user(11);
    let stranger = user(99);

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

    let batch = vec![
        SubjectInput {
            subject: sample_ref(1),
            gold: None,
        },
        SubjectInput {
            subject: sample_ref(2),
            gold: None,
        },
    ];
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
        name: "Surveyor-01".into(),
        avatar_seed: 42,
    };
    let reg_res: Result<(), ApiError> = env.update(platform, payments, "register_aaa", reg);
    assert_eq!(reg_res, Ok(()));
    tick(&env, 5);
    step("installed and registered AAA canister on platform");

    let stranger_call = env
        .pic
        .update_call(aaa, stranger, "get_task", encode_args(()).unwrap());
    assert!(
        stranger_call.is_err(),
        "stranger ingress must be rejected by inspect_message"
    );

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

    let initial_cycles = env.pic.cycle_balance(aaa);
    let task_res: Result<Task, ApiError> = env.update(aaa, operator, "get_task", ());
    assert!(task_res.is_ok(), "operator get_task must succeed");
    let task = task_res.unwrap();
    let cycles_after_task = env.pic.cycle_balance(aaa);
    assert!(
        cycles_after_task < initial_cycles,
        "fee_get_task must be deducted from AAA cycle balance"
    );
    step("operator fetched task via AAA relay with fee deducted");

    let submission = ClassificationSubmission {
        task_id: task.task_id,
        answers: vec![Answer {
            question_id: "q1".into(),
            answer_id: "smooth".into(),
        }],
        observed_image_sha256: vec![1; 32],
        discovery: None,
        agent_label: None,
        submitted_by: stranger,
    };

    let submit_res: Result<ClassificationReceipt, ApiError> =
        env.update(aaa, operator, "submit_classification", submission.clone());
    assert!(
        submit_res.is_ok(),
        "first submission must succeed: {submit_res:?}"
    );
    let receipt = submit_res.unwrap();
    assert!(
        !receipt.duplicate,
        "the first submission must not be reported as a duplicate"
    );
    step("first submission accepted by the AAA relay");

    let rec_1: Result<Option<aaa::record::Record>, ApiError> =
        env.query(aaa, owner, "get_record", 1u64);
    let rec_1 = rec_1.expect("owner get_record must succeed");
    assert!(rec_1.is_some(), "local repository must contain record #1");
    let rec_1 = rec_1.unwrap();
    assert_eq!(rec_1.seq, 1);
    assert_eq!(rec_1.task_or_assignment_id, Some(task.task_id));
    assert_eq!(
        rec_1.by, operator,
        "submitted_by must be stamped with operator caller"
    );

    let rec_2: Result<Option<aaa::record::Record>, ApiError> =
        env.query(aaa, owner, "get_record", 2u64);
    assert!(
        rec_2.expect("owner get_record must succeed").is_none(),
        "local repository must contain exactly ONE record, not two"
    );
    step("verified exactly ONE local record in repository (acceptance criterion)");

    let plat_class_1: Option<platform::scoring::Classification> =
        env.query(platform, alice, "get_classification", 1u64);
    assert!(plat_class_1.is_some());
    let plat_class_2: Option<platform::scoring::Classification> =
        env.query(platform, alice, "get_classification", 2u64);
    assert!(plat_class_2.is_none());
    step("verified exactly ONE classification recorded on platform");

    let stranger_get_record =
        env.pic
            .query_call(aaa, stranger, "get_record", encode_args((1u64,)).unwrap());
    assert!(
        stranger_get_record.is_err()
            || decode_one::<Result<Option<aaa::record::Record>, ApiError>>(
                &stranger_get_record.unwrap()
            )
            .unwrap()
            .is_err(),
        "get_record must reject callers who are neither owner nor operator"
    );
    step("verified get_record requires owner/operator authorization");

    let submit_res2: Result<ClassificationReceipt, ApiError> =
        env.update(aaa, operator, "submit_classification", submission);
    assert!(submit_res2.is_ok());
    assert!(
        submit_res2.unwrap().duplicate,
        "a caller-level retry of the same task must be reported as a duplicate by the platform"
    );
    let rec_2_again: Result<Option<aaa::record::Record>, ApiError> =
        env.query(aaa, owner, "get_record", 2u64);
    assert!(
        rec_2_again
            .expect("owner get_record must succeed")
            .is_none(),
        "duplicate submit must not create a second record"
    );
    step("operator-side retry of the same task produced exactly one record (acceptance criterion)");

    let remove_op: Result<(), ApiError> = env.update(aaa, owner, "remove_operator", operator);
    assert_eq!(remove_op, Ok(()));
    tick(&env, 5);

    let get_task_rejected =
        env.pic
            .update_call(aaa, operator, "get_task", encode_args(()).unwrap());
    assert!(
        get_task_rejected.is_err(),
        "removed operator must be rejected immediately"
    );
    step("removed operator rejected immediately (acceptance criterion)");
}
