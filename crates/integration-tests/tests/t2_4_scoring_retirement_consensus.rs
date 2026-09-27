use candid::{decode_one, encode_args};
use integration_tests::pic::{canister_wasm, user, IcpEnv};
use integration_tests::step;
use platform::catalog::{Subject, SubjectInput};
use platform::config::Params;
use platform::registry::RegisterArgs;
use platform::scoring::{Classification, SubjectConsensus};
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
        questions: vec![
            Question {
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
                        next: Some("q2".into()),
                    },
                ],
            },
            Question {
                id: "q2".into(),
                prompt: "Has spiral arms?".into(),
                answers: vec![
                    AnswerOption {
                        id: "yes".into(),
                        label: "Yes".into(),
                        next: None,
                    },
                    AnswerOption {
                        id: "no".into(),
                        label: "No".into(),
                        next: None,
                    },
                ],
            },
        ],
        discovery_categories: vec![DiscoveryCategory {
            id: "lens".into(),
            label: "Gravitational Lens".into(),
            description: "Arcs or rings".into(),
        }],
        guidance_md: "Look closely at the image.".into(),
    }
}

#[test]
fn t2_4_retire_at_k5_and_idempotent_submit() {
    println!(
        "T2.4 demo: Scoring — validation, gold, tallies, retirement at K=5, consensus, idempotency"
    );
    let env = IcpEnv::new();
    let alice = user(1);
    let payments = user(99);

    let platform = env.install("platform", alice);
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
    let ok: Result<(), ApiError> = env.update(platform, alice, "admin_approve_wasm", 1u32);
    assert_eq!(ok, Ok(()));
    step("installed platform, approved AAA wasm v1");

    let subnet = env.pic.topology().get_app_subnets()[0];
    let mut aaas = Vec::new();

    for i in 1..=6 {
        let owner = user(100 + i);
        let aaa = env.pic.create_canister_on_subnet(Some(alice), None, subnet);
        env.pic.add_cycles(aaa, 5_000_000_000_000);
        env.pic
            .set_controllers(aaa, Some(alice), vec![owner, platform])
            .unwrap();

        let reg = RegisterArgs {
            canister_id: aaa,
            owner,
            name: format!("Surveyor-{i:02}"),
            avatar_seed: i as u64,
        };
        let ok: Result<(), ApiError> = env.update(platform, payments, "register_aaa", reg);
        assert_eq!(ok, Ok(()));
        aaas.push((aaa, owner));
    }
    tick(&env, 10);
    step("registered 6 distinct active AAAs with their respective owners");

    let proto = sample_protocol(1);
    let ok: Result<(), ApiError> = env.update(platform, alice, "admin_add_protocol", proto);
    assert_eq!(ok, Ok(()));
    let ok: Result<(), ApiError> = env.update(platform, alice, "admin_set_current_protocol", 1u16);
    assert_eq!(ok, Ok(()));

    let batch = vec![
        SubjectInput {
            subject: sample_ref(501),
            gold: None,
        },
        SubjectInput {
            subject: sample_ref(502),
            gold: Some(vec![
                Answer {
                    question_id: "q1".into(),
                    answer_id: "featured".into(),
                },
                Answer {
                    question_id: "q2".into(),
                    answer_id: "yes".into(),
                },
            ]),
        },
    ];
    let added: Result<u32, ApiError> = env.update(platform, alice, "admin_add_subjects", batch);
    assert_eq!(added, Ok(2));

    let zero_fee_params = Params {
        fee_get_task: 0,
        fee_submit_classification: 0,
        max_open_leases_per_aaa: 10,
        retire_after_k: 5,
        gold_rate_bp: 0,
        calibration_gold_rate_bp: 0,
        ..Params::default()
    };
    let ok: Result<(), ApiError> = env.update(platform, alice, "admin_set_params", zero_fee_params);
    assert_eq!(ok, Ok(()));
    step("added subject 501 (retire_after_k=5) and gold subject 502; set fees to 0 for testing");

    let unregistered_sub: Result<ClassificationReceipt, ApiError> = env.update(
        platform,
        user(42),
        "submit_classification",
        ClassificationSubmission {
            task_id: 1,
            answers: vec![Answer {
                question_id: "q1".into(),
                answer_id: "smooth".into(),
            }],
            observed_image_sha256: vec![1; 32],
            discovery: None,
            agent_label: None,
            submitted_by: user(42),
        },
    );
    assert_eq!(unregistered_sub, Err(ApiError::NotRegistered));
    step("unregistered caller rejected with NotRegistered");

    let (aaa_1, owner_1) = aaas[0];
    let task_1: Result<Task, ApiError> = env.update(platform, aaa_1, "get_task", ());
    let task_1 = task_1.expect("task 1 issued");
    assert_eq!(task_1.subject.subject_id, 501);

    let foreign_submitter: Result<ClassificationReceipt, ApiError> = env.update(
        platform,
        aaa_1,
        "submit_classification",
        ClassificationSubmission {
            task_id: task_1.task_id,
            answers: vec![Answer {
                question_id: "q1".into(),
                answer_id: "smooth".into(),
            }],
            observed_image_sha256: vec![1; 32],
            discovery: None,
            agent_label: None,
            submitted_by: user(250),
        },
    );
    assert_eq!(foreign_submitter, Err(ApiError::Unauthorized));
    step("unauthorized submitted_by principal rejected with Unauthorized");

    let invalid_path: Result<ClassificationReceipt, ApiError> = env.update(
        platform,
        aaa_1,
        "submit_classification",
        ClassificationSubmission {
            task_id: task_1.task_id,
            answers: vec![Answer {
                question_id: "q1".into(),
                answer_id: "featured".into(),
            }],
            observed_image_sha256: vec![1; 32],
            discovery: None,
            agent_label: None,
            submitted_by: owner_1,
        },
    );
    assert!(matches!(invalid_path, Err(ApiError::InvalidInput(_))));
    step("incomplete answer path rejected with InvalidInput");

    let receipt_1: Result<ClassificationReceipt, ApiError> = env.update(
        platform,
        aaa_1,
        "submit_classification",
        ClassificationSubmission {
            task_id: task_1.task_id,
            answers: vec![Answer {
                question_id: "q1".into(),
                answer_id: "smooth".into(),
            }],
            observed_image_sha256: vec![1; 32],
            discovery: None,
            agent_label: Some("bot-v1".into()),
            submitted_by: owner_1,
        },
    );
    let r1 = receipt_1.expect("receipt 1");
    assert!(!r1.duplicate);
    assert_eq!(r1.xp_awarded, 1);
    let s_tally1: Option<Subject> = env.query(platform, alice, "get_subject", 501u32);
    assert_eq!(s_tally1.unwrap().tally_count, 1);
    step("aaa_1 submitted valid classification; tally incremented to 1");

    let dup_1: Result<ClassificationReceipt, ApiError> = env.update(
        platform,
        aaa_1,
        "submit_classification",
        ClassificationSubmission {
            task_id: task_1.task_id,
            answers: vec![Answer {
                question_id: "q1".into(),
                answer_id: "smooth".into(),
            }],
            observed_image_sha256: vec![1; 32],
            discovery: None,
            agent_label: Some("bot-v1".into()),
            submitted_by: owner_1,
        },
    );
    let d1 = dup_1.expect("duplicate receipt");
    assert!(d1.duplicate);
    assert_eq!(d1.classification_id, r1.classification_id);
    assert_eq!(d1.xp_awarded, 0);
    let s_tally1_again: Option<Subject> = env.query(platform, alice, "get_subject", 501u32);
    assert_eq!(s_tally1_again.unwrap().tally_count, 1);
    step("idempotent resubmit: returned duplicate=true, same id, 0 xp, tally unchanged");

    let mut preregistered_tasks = Vec::new();
    for &(aaa, _) in &aaas[1..] {
        let t: Result<Task, ApiError> = env.update(platform, aaa, "get_task", ());
        preregistered_tasks.push(t.expect("task issued"));
    }

    for (i, &(aaa, owner)) in aaas[1..4].iter().enumerate() {
        let tid = preregistered_tasks[i].task_id;
        let ans = if i < 2 { "smooth" } else { "featured" };
        let answers = if ans == "smooth" {
            vec![Answer {
                question_id: "q1".into(),
                answer_id: "smooth".into(),
            }]
        } else {
            vec![
                Answer {
                    question_id: "q1".into(),
                    answer_id: "featured".into(),
                },
                Answer {
                    question_id: "q2".into(),
                    answer_id: "yes".into(),
                },
            ]
        };

        let res: Result<ClassificationReceipt, ApiError> = env.update(
            platform,
            aaa,
            "submit_classification",
            ClassificationSubmission {
                task_id: tid,
                answers,
                observed_image_sha256: vec![1; 32],
                discovery: None,
                agent_label: None,
                submitted_by: owner,
            },
        );
        assert!(!res.unwrap().duplicate);
        let s: Option<Subject> = env.query(platform, alice, "get_subject", 501u32);
        assert_eq!(s.unwrap().tally_count, i as u16 + 2);
    }
    step("submissions 2, 3, 4 accepted and tallied; subject stays active");

    let (aaa_5, owner_5) = aaas[4];
    let tid_5 = preregistered_tasks[3].task_id;
    let res_5: Result<ClassificationReceipt, ApiError> = env.update(
        platform,
        aaa_5,
        "submit_classification",
        ClassificationSubmission {
            task_id: tid_5,
            answers: vec![Answer {
                question_id: "q1".into(),
                answer_id: "smooth".into(),
            }],
            observed_image_sha256: vec![1; 32],
            discovery: None,
            agent_label: None,
            submitted_by: owner_5,
        },
    );
    assert!(!res_5.unwrap().duplicate);

    let s_retired: Option<Subject> = env.query(platform, alice, "get_subject", 501u32);
    let s_retired = s_retired.unwrap();
    assert_eq!(s_retired.tally_count, 5);
    assert!(!s_retired.active);
    step("5th submission reached K=5: subject 501 retired and removed from pool");

    let consensus: Option<SubjectConsensus> =
        env.query(platform, alice, "get_subject_consensus", 501u32);
    let cons = consensus.expect("consensus resolved");
    assert_eq!(cons.consensus.len(), 1);
    assert_eq!(cons.consensus[0], ("q1".into(), "smooth".into()));
    step("consensus scoring evaluated: majority answer smooth chosen (4 votes to 1)");

    let c1: Option<Classification> = env.query(platform, alice, "get_classification", 1u64);
    assert_eq!(c1.unwrap().consensus_score, Some((1, 1)));
    step("past classifications updated with consensus scores");

    let (aaa_6, owner_6) = aaas[5];
    let tid_6 = preregistered_tasks[4].task_id;
    let res_6: Result<ClassificationReceipt, ApiError> = env.update(
        platform,
        aaa_6,
        "submit_classification",
        ClassificationSubmission {
            task_id: tid_6,
            answers: vec![Answer {
                question_id: "q1".into(),
                answer_id: "smooth".into(),
            }],
            observed_image_sha256: vec![1; 32],
            discovery: None,
            agent_label: None,
            submitted_by: owner_6,
        },
    );
    let r6 = res_6.expect("receipt 6");
    assert!(!r6.duplicate);
    let s_still_5: Option<Subject> = env.query(platform, alice, "get_subject", 501u32);
    assert_eq!(s_still_5.unwrap().tally_count, 5);
    step("late submission for retired subject accepted and scored against consensus without modifying tally");
}
