use candid::{decode_one, encode_args};
use integration_tests::pic::{canister_wasm, user, IcpEnv};
use integration_tests::step;
use platform::catalog::SubjectInput;
use platform::config::Params;
use platform::registry::{AaaRecord, AaaStatus, OperatorSetInput, RegisterArgs};
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
fn t2_8_foreign_expired_unsynced_submitter_rejected() {
    println!("T2.8 demo: Submitter security — foreign, expired, unsynced submitter rejection & strict provenance");
    let env = IcpEnv::new();
    let alice = user(1);
    let payments = user(2);
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

    let p_bytes = env
        .pic
        .query_call(platform, alice, "get_params", encode_args(()).unwrap())
        .expect("get_params");
    let mut params: Params = decode_one(&p_bytes).unwrap();
    params.fee_submit_classification = 0;
    params.fee_get_task = 0;
    let set_bytes = env
        .pic
        .update_call(
            platform,
            alice,
            "admin_set_params",
            encode_args((params,)).unwrap(),
        )
        .expect("set params");
    let set_res: Result<(), ApiError> = decode_one(&set_bytes).unwrap();
    assert_eq!(set_res, Ok(()));

    let subnet = env.pic.topology().get_app_subnets()[0];
    let owner_1 = user(101);
    let aaa_1 = env.pic.create_canister_on_subnet(Some(alice), None, subnet);
    env.pic.add_cycles(aaa_1, 5_000_000_000_000);
    env.pic
        .set_controllers(aaa_1, Some(alice), vec![owner_1, platform])
        .unwrap();

    let reg_1 = RegisterArgs {
        canister_id: aaa_1,
        owner: owner_1,
        name: "SecuritySurveyor-01".into(),
        avatar_seed: 101,
    };
    let reg_res: Result<(), ApiError> = env.update(platform, payments, "register_aaa", reg_1);
    assert_eq!(reg_res, Ok(()));

    let owner_2 = user(102);
    let aaa_2 = env.pic.create_canister_on_subnet(Some(alice), None, subnet);
    env.pic.add_cycles(aaa_2, 5_000_000_000_000);
    env.pic
        .set_controllers(aaa_2, Some(alice), vec![owner_2, platform])
        .unwrap();

    let reg_2 = RegisterArgs {
        canister_id: aaa_2,
        owner: owner_2,
        name: "SecuritySurveyor-02".into(),
        avatar_seed: 102,
    };
    let reg_res2: Result<(), ApiError> = env.update(platform, payments, "register_aaa", reg_2);
    assert_eq!(reg_res2, Ok(()));
    tick(&env, 5);
    step("registered two active AAAs (aaa_1 and aaa_2) with approved wasm");

    let proto = sample_protocol(1);
    let ok: Result<(), ApiError> = env.update(platform, alice, "admin_add_protocol", proto);
    assert_eq!(ok, Ok(()));
    let ok: Result<(), ApiError> = env.update(platform, alice, "admin_set_current_protocol", 1u16);
    assert_eq!(ok, Ok(()));

    let batch = vec![
        SubjectInput {
            subject: sample_ref(901),
            gold: None,
        },
        SubjectInput {
            subject: sample_ref(902),
            gold: None,
        },
        SubjectInput {
            subject: sample_ref(903),
            gold: None,
        },
    ];
    let added: Result<u32, ApiError> = env.update(platform, alice, "admin_add_subjects", batch);
    assert_eq!(added, Ok(3));

    let operator_valid = user(201);
    let operator_future = user(202);
    let operator_expired = user(203);
    let operator_unsynced = user(204);
    let foreign_stranger = user(250);

    let sync_args = OperatorSetInput {
        operators: vec![
            (operator_valid, None),
            (operator_future, Some(500_000_000_000_000)),
            (operator_expired, Some(10_000_000)),
        ],
    };
    let sync_res: Result<(), ApiError> = env.update(platform, aaa_1, "sync_operators", sync_args);
    assert_eq!(sync_res, Ok(()));
    step("synced operators on aaa_1: operator_valid (no expiry), operator_future (future), operator_expired (near-term)");

    let task_call = env
        .pic
        .update_call(platform, aaa_1, "get_task", encode_args(()).unwrap())
        .expect("get_task");
    let task_res: Result<Task, ApiError> = decode_one(&task_call).unwrap();
    let task = task_res.expect("task issued to aaa_1");

    let foreign_call = env
        .pic
        .update_call(
            platform,
            aaa_1,
            "submit_classification",
            encode_args((ClassificationSubmission {
                task_id: task.task_id,
                answers: vec![Answer {
                    question_id: "q1".into(),
                    answer_id: "smooth".into(),
                }],
                observed_image_sha256: vec![1; 32],
                discovery: None,
                agent_label: Some("foreign-agent".into()),
                submitted_by: foreign_stranger,
            },))
            .unwrap(),
        )
        .expect("submit foreign");
    let foreign_res: Result<ClassificationReceipt, ApiError> = decode_one(&foreign_call).unwrap();
    assert_eq!(foreign_res, Err(ApiError::Unauthorized));
    step("foreign submitter (stranger) rejected with Unauthorized");

    let other_owner_call = env
        .pic
        .update_call(
            platform,
            aaa_1,
            "submit_classification",
            encode_args((ClassificationSubmission {
                task_id: task.task_id,
                answers: vec![Answer {
                    question_id: "q1".into(),
                    answer_id: "smooth".into(),
                }],
                observed_image_sha256: vec![1; 32],
                discovery: None,
                agent_label: Some("cross-owner-agent".into()),
                submitted_by: owner_2,
            },))
            .unwrap(),
        )
        .expect("submit cross owner");
    let other_owner_res: Result<ClassificationReceipt, ApiError> =
        decode_one(&other_owner_call).unwrap();
    assert_eq!(other_owner_res, Err(ApiError::Unauthorized));
    step("foreign submitter (owner of another AAA) rejected with Unauthorized");

    let unsynced_call = env
        .pic
        .update_call(
            platform,
            aaa_1,
            "submit_classification",
            encode_args((ClassificationSubmission {
                task_id: task.task_id,
                answers: vec![Answer {
                    question_id: "q1".into(),
                    answer_id: "smooth".into(),
                }],
                observed_image_sha256: vec![1; 32],
                discovery: None,
                agent_label: Some("unsynced-agent".into()),
                submitted_by: operator_unsynced,
            },))
            .unwrap(),
        )
        .expect("submit unsynced");
    let unsynced_res: Result<ClassificationReceipt, ApiError> = decode_one(&unsynced_call).unwrap();
    assert_eq!(unsynced_res, Err(ApiError::Unauthorized));
    step("unsynced operator (not in synced set) rejected with Unauthorized");

    let expired_call = env
        .pic
        .update_call(
            platform,
            aaa_1,
            "submit_classification",
            encode_args((ClassificationSubmission {
                task_id: task.task_id,
                answers: vec![Answer {
                    question_id: "q1".into(),
                    answer_id: "smooth".into(),
                }],
                observed_image_sha256: vec![1; 32],
                discovery: None,
                agent_label: Some("expired-agent".into()),
                submitted_by: operator_expired,
            },))
            .unwrap(),
        )
        .expect("submit expired");
    let expired_res: Result<ClassificationReceipt, ApiError> = decode_one(&expired_call).unwrap();
    assert_eq!(expired_res, Err(ApiError::Unauthorized));
    step("expired operator rejected with Unauthorized");

    let cross_aaa_call = env
        .pic
        .update_call(
            platform,
            aaa_2,
            "submit_classification",
            encode_args((ClassificationSubmission {
                task_id: task.task_id,
                answers: vec![Answer {
                    question_id: "q1".into(),
                    answer_id: "smooth".into(),
                }],
                observed_image_sha256: vec![1; 32],
                discovery: None,
                agent_label: Some("cross-operator-agent".into()),
                submitted_by: operator_valid,
            },))
            .unwrap(),
        )
        .expect("submit cross aaa");
    let cross_aaa_res: Result<ClassificationReceipt, ApiError> =
        decode_one(&cross_aaa_call).unwrap();
    assert_eq!(cross_aaa_res, Err(ApiError::Unauthorized));
    step("operator of aaa_1 attempting to submit via aaa_2 rejected with Unauthorized");

    let valid_op_call = env
        .pic
        .update_call(
            platform,
            aaa_1,
            "submit_classification",
            encode_args((ClassificationSubmission {
                task_id: task.task_id,
                answers: vec![Answer {
                    question_id: "q1".into(),
                    answer_id: "smooth".into(),
                }],
                observed_image_sha256: vec![1; 32],
                discovery: None,
                agent_label: Some("valid-agent".into()),
                submitted_by: operator_valid,
            },))
            .unwrap(),
        )
        .expect("submit valid op");
    let valid_op_res: Result<ClassificationReceipt, ApiError> = decode_one(&valid_op_call).unwrap();
    assert!(valid_op_res.is_ok());
    assert_eq!(valid_op_res.unwrap().classification_id, 1);
    step("valid synced operator accepted and produced classification #1");

    let task2_call = env
        .pic
        .update_call(platform, aaa_1, "get_task", encode_args(()).unwrap())
        .expect("get_task 2");
    let task2_res: Result<Task, ApiError> = decode_one(&task2_call).unwrap();
    let task2 = task2_res.expect("task 2 issued to aaa_1");

    let valid_owner_call = env
        .pic
        .update_call(
            platform,
            aaa_1,
            "submit_classification",
            encode_args((ClassificationSubmission {
                task_id: task2.task_id,
                answers: vec![Answer {
                    question_id: "q1".into(),
                    answer_id: "featured".into(),
                }],
                observed_image_sha256: vec![1; 32],
                discovery: None,
                agent_label: Some("owner-agent".into()),
                submitted_by: owner_1,
            },))
            .unwrap(),
        )
        .expect("submit valid owner");
    let valid_owner_res: Result<ClassificationReceipt, ApiError> =
        decode_one(&valid_owner_call).unwrap();
    assert!(valid_owner_res.is_ok());
    assert_eq!(valid_owner_res.unwrap().classification_id, 2);
    step("valid owner accepted and produced classification #2");

    let probe_wasm = canister_wasm("spike-probe");
    let _ = env
        .pic
        .reinstall_canister(aaa_2, probe_wasm, vec![], Some(owner_2));

    let tampered_submit: Result<ClassificationReceipt, ApiError> = env.update(
        platform,
        aaa_2,
        "submit_classification",
        ClassificationSubmission {
            task_id: task2.task_id,
            answers: vec![Answer {
                question_id: "q1".into(),
                answer_id: "smooth".into(),
            }],
            observed_image_sha256: vec![1; 32],
            discovery: None,
            agent_label: None,
            submitted_by: owner_2,
        },
    );
    assert_eq!(tampered_submit, Err(ApiError::Suspended));

    let tampered_record: Option<AaaRecord> = env.query(platform, alice, "get_aaa", aaa_2);
    assert_eq!(tampered_record.unwrap().status, AaaStatus::Suspended);
    step("submit_classification re-verifies provenance: unapproved module hash suspends the AAA");

    let tampered_get_task = env
        .pic
        .update_call(platform, aaa_2, "get_task", encode_args(()).unwrap())
        .expect("get_task on suspended");
    let tampered_res: Result<Task, ApiError> = decode_one(&tampered_get_task).unwrap();
    assert_eq!(tampered_res, Err(ApiError::Suspended));
    step("suspended AAA rejected on get_task with Suspended");
}
