use candid::{decode_one, encode_args};
use integration_tests::pic::{canister_wasm, user, IcpEnv};
use integration_tests::step;
use platform::catalog::{Subject, SubjectInput};
use platform::config::Params;
use platform::registry::RegisterArgs;
use platform::scoring::SubjectConsensus;
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
                prompt: "Is it smooth or featured?".into(),
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
                prompt: "Does it have a spiral arm?".into(),
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
fn t2_7_registered_aaa_task_receipt_and_idempotent_duplicate() {
    println!("T2.7 demo part 1: Criteria 1 & 2 — Registration, InsufficientFee, task receipt & idempotency");
    let env = IcpEnv::new();
    let alice = user(1);
    let payments = user(2);
    let stranger = user(42);
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

    let subnet = env.pic.topology().get_app_subnets()[0];
    let owner_1 = user(101);
    let aaa_1 = env.pic.create_canister_on_subnet(Some(alice), None, subnet);
    env.pic.add_cycles(aaa_1, 5_000_000_000_000);
    env.pic
        .set_controllers(aaa_1, Some(alice), vec![owner_1, platform])
        .unwrap();

    let reg = RegisterArgs {
        canister_id: aaa_1,
        owner: owner_1,
        name: "Rover-Alpha".into(),
        avatar_seed: 7,
    };
    let ok: Result<(), ApiError> = env.update(platform, payments, "register_aaa", reg);
    assert_eq!(ok, Ok(()));

    let proto = sample_protocol(1);
    let ok: Result<(), ApiError> = env.update(platform, alice, "admin_add_protocol", proto);
    assert_eq!(ok, Ok(()));
    let ok: Result<(), ApiError> = env.update(platform, alice, "admin_set_current_protocol", 1u16);
    assert_eq!(ok, Ok(()));

    let batch = vec![SubjectInput {
        subject: sample_ref(301),
        gold: None,
    }];
    let added: Result<u32, ApiError> = env.update(platform, alice, "admin_add_subjects", batch);
    assert_eq!(added, Ok(1));
    step("environment initialized: platform, AAA registered, protocol v1, and subject 301 added");

    let stranger_task: Result<Task, ApiError> = env.update(platform, stranger, "get_task", ());
    assert_eq!(stranger_task, Err(ApiError::NotRegistered));

    let dummy_sub = ClassificationSubmission {
        task_id: 1,
        answers: vec![Answer {
            question_id: "q1".into(),
            answer_id: "smooth".into(),
        }],
        observed_image_sha256: vec![1; 32],
        discovery: None,
        agent_label: None,
        submitted_by: stranger,
    };
    let stranger_sub: Result<ClassificationReceipt, ApiError> =
        env.update(platform, stranger, "submit_classification", dummy_sub);
    assert_eq!(stranger_sub, Err(ApiError::NotRegistered));
    step("criterion 2 (part 1): non-AAA caller rejected with NotRegistered on get_task and submit_classification");

    let fee_task_res: Result<Task, ApiError> = env.update(platform, aaa_1, "get_task", ());
    assert_eq!(
        fee_task_res,
        Err(ApiError::InsufficientFee {
            required: 50_000_000u64.into(),
        })
    );

    let p_bytes = env
        .pic
        .query_call(platform, alice, "get_params", encode_args(()).unwrap())
        .expect("get_params");
    let mut params: Params = decode_one(&p_bytes).unwrap();
    params.fee_get_task = 0;
    let set_bytes = env
        .pic
        .update_call(
            platform,
            alice,
            "admin_set_params",
            encode_args((params.clone(),)).unwrap(),
        )
        .expect("set params");
    let set_res: Result<(), ApiError> = decode_one(&set_bytes).unwrap();
    assert_eq!(set_res, Ok(()));

    let task_call = env
        .pic
        .update_call(platform, aaa_1, "get_task", encode_args(()).unwrap())
        .expect("get_task");
    let task_res: Result<Task, ApiError> = decode_one(&task_call).unwrap();
    let task = task_res.expect("task 1 issued to aaa_1");
    assert_eq!(task.subject.subject_id, 301);

    let fee_sub_call = env
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
                agent_label: Some("agent-alpha".into()),
                submitted_by: owner_1,
            },))
            .unwrap(),
        )
        .expect("submit classification with fee required");
    let fee_sub_res: Result<ClassificationReceipt, ApiError> = decode_one(&fee_sub_call).unwrap();
    assert_eq!(
        fee_sub_res,
        Err(ApiError::InsufficientFee {
            required: 200_000_000u64.into(),
        })
    );
    step("criterion 2 (part 2): InsufficientFee returned when cycles are missing on get_task and submit_classification");

    params.fee_submit_classification = 0;
    let set_bytes2 = env
        .pic
        .update_call(
            platform,
            alice,
            "admin_set_params",
            encode_args((params,)).unwrap(),
        )
        .expect("set params free");
    let set_res2: Result<(), ApiError> = decode_one(&set_bytes2).unwrap();
    assert_eq!(set_res2, Ok(()));

    let sub_call = env
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
                agent_label: Some("agent-alpha".into()),
                submitted_by: owner_1,
            },))
            .unwrap(),
        )
        .expect("submit classification free");
    let sub_res: Result<ClassificationReceipt, ApiError> = decode_one(&sub_call).unwrap();
    let receipt = sub_res.expect("receipt returned");
    assert_eq!(receipt.classification_id, 1);
    assert!(!receipt.duplicate);
    assert_eq!(receipt.xp_awarded, 1);

    let subj_before: Option<Subject> = env.query(platform, alice, "get_subject", 301u32);
    let tally_before = subj_before.unwrap().tally_count;
    assert_eq!(tally_before, 1);

    let dup_call = env
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
                agent_label: Some("agent-alpha".into()),
                submitted_by: owner_1,
            },))
            .unwrap(),
        )
        .expect("resubmit duplicate classification");
    let dup_res: Result<ClassificationReceipt, ApiError> = decode_one(&dup_call).unwrap();
    let dup_receipt = dup_res.expect("duplicate receipt returned");
    assert_eq!(dup_receipt.classification_id, 1);
    assert!(dup_receipt.duplicate);
    assert_eq!(dup_receipt.xp_awarded, receipt.xp_awarded);

    let subj_after: Option<Subject> = env.query(platform, alice, "get_subject", 301u32);
    let tally_after = subj_after.unwrap().tally_count;
    assert_eq!(tally_after, 1);
    step("criterion 1: registered AAA gets task, submits, receives receipt; resubmit returns duplicate=true and state unchanged");
}

#[test]
fn t2_7_fifth_classification_retires_and_seen_set_never_reissues() {
    println!("T2.7 demo part 2: Criterion 3 — 5th classification retires subject (never reissued); AAA never sees same subject twice");
    let env = IcpEnv::new();
    let alice = user(1);
    let payments = user(2);
    let platform = env.install_on("platform", alice, 10_000_000_000_000, 0);
    tick(&env, 2);

    let ok: Result<(), ApiError> = env.update(platform, alice, "admin_set_payments_id", payments);
    assert_eq!(ok, Ok(()));

    let wasm_v1 = canister_wasm("aaa");
    let hash_v1 = Sha256::digest(&wasm_v1).to_vec();
    let _ = env.pic.update_call(
        platform,
        alice,
        "admin_upload_wasm",
        encode_args((1u32, wasm_v1, hash_v1)).unwrap(),
    );
    let _ = env.pic.update_call(
        platform,
        alice,
        "admin_approve_wasm",
        encode_args((1u32,)).unwrap(),
    );

    let p_bytes = env
        .pic
        .query_call(platform, alice, "get_params", encode_args(()).unwrap())
        .expect("get_params");
    let mut params: Params = decode_one(&p_bytes).unwrap();
    params.fee_submit_classification = 0;
    params.fee_get_task = 0;
    params.gold_rate_bp = 0;
    params.calibration_gold_rate_bp = 0;
    params.max_open_leases_per_aaa = 20;
    let _ = env.pic.update_call(
        platform,
        alice,
        "admin_set_params",
        encode_args((params,)).unwrap(),
    );

    let subnet = env.pic.topology().get_app_subnets()[0];
    let mut aaas = Vec::new();
    for i in 1..=6 {
        let owner = user(110 + i as u8);
        let canister = env.pic.create_canister_on_subnet(Some(alice), None, subnet);
        env.pic.add_cycles(canister, 5_000_000_000_000);
        env.pic
            .set_controllers(canister, Some(alice), vec![owner, platform])
            .unwrap();

        let reg = RegisterArgs {
            canister_id: canister,
            owner,
            name: format!("Surveyor-Cluster-{i}"),
            avatar_seed: 50 + i as u64,
        };
        let ok: Result<(), ApiError> = env.update(platform, payments, "register_aaa", reg);
        assert_eq!(ok, Ok(()));
        aaas.push((canister, owner));
    }
    tick(&env, 5);

    let proto = sample_protocol(1);
    let ok: Result<(), ApiError> = env.update(platform, alice, "admin_add_protocol", proto);
    assert_eq!(ok, Ok(()));
    let ok: Result<(), ApiError> = env.update(platform, alice, "admin_set_current_protocol", 1u16);
    assert_eq!(ok, Ok(()));

    let batch = vec![
        SubjectInput {
            subject: sample_ref(401),
            gold: None,
        },
        SubjectInput {
            subject: sample_ref(402),
            gold: None,
        },
    ];
    let added: Result<u32, ApiError> = env.update(platform, alice, "admin_add_subjects", batch);
    assert_eq!(added, Ok(2));
    step("registered 6 distinct AAAs, added protocol v1, added subjects 401 and 402");

    let (aaa_1, _) = aaas[0];
    let t1_call = env
        .pic
        .update_call(platform, aaa_1, "get_task", encode_args(()).unwrap())
        .expect("get_task 1");
    let t1: Task = decode_one::<Result<Task, ApiError>>(&t1_call)
        .unwrap()
        .unwrap();
    let t2_call = env
        .pic
        .update_call(platform, aaa_1, "get_task", encode_args(()).unwrap())
        .expect("get_task 2");
    let t2: Task = decode_one::<Result<Task, ApiError>>(&t2_call)
        .unwrap()
        .unwrap();
    assert_ne!(t1.subject.subject_id, t2.subject.subject_id);

    let t3_call = env
        .pic
        .update_call(platform, aaa_1, "get_task", encode_args(()).unwrap())
        .expect("get_task 3");
    let t3_res = decode_one::<Result<Task, ApiError>>(&t3_call).unwrap();
    assert_eq!(t3_res, Err(ApiError::NotFound));
    step("criterion 3 (part 1): aaa_1 received distinct subjects 401 and 402, then NotFound; never sees same subject twice");

    let target_subject_id = 401u32;
    for (i, &(aaa, owner)) in aaas.iter().take(4).enumerate() {
        let task = if i == 0 {
            if t1.subject.subject_id == target_subject_id {
                t1.clone()
            } else {
                t2.clone()
            }
        } else {
            let mut found = None;
            for _ in 0..2 {
                let tc = env
                    .pic
                    .update_call(platform, aaa, "get_task", encode_args(()).unwrap())
                    .expect("get_task");
                let tr: Task = decode_one::<Result<Task, ApiError>>(&tc).unwrap().unwrap();
                if tr.subject.subject_id == target_subject_id {
                    found = Some(tr);
                    break;
                }
            }
            found.expect("found target subject")
        };

        let sub_call = env
            .pic
            .update_call(
                platform,
                aaa,
                "submit_classification",
                encode_args((ClassificationSubmission {
                    task_id: task.task_id,
                    answers: vec![
                        Answer {
                            question_id: "q1".into(),
                            answer_id: "featured".into(),
                        },
                        Answer {
                            question_id: "q2".into(),
                            answer_id: "yes".into(),
                        },
                    ],
                    observed_image_sha256: vec![1; 32],
                    discovery: None,
                    agent_label: Some("cluster-bot".into()),
                    submitted_by: owner,
                },))
                .unwrap(),
            )
            .expect("submit");
        let receipt: ClassificationReceipt =
            decode_one::<Result<ClassificationReceipt, ApiError>>(&sub_call)
                .unwrap()
                .unwrap();
        assert!(!receipt.duplicate);
    }

    let subj_tally4: Option<Subject> = env.query(platform, alice, "get_subject", target_subject_id);
    assert_eq!(subj_tally4.unwrap().tally_count, 4);
    let consensus_before: Option<SubjectConsensus> =
        env.query(platform, alice, "get_subject_consensus", target_subject_id);
    assert!(consensus_before.is_none());
    step("4 distinct AAAs classified subject 401; tally count is 4, consensus not yet reached");

    let (aaa_5, owner_5) = aaas[4];
    let mut task_5 = None;
    for _ in 0..2 {
        let tc = env
            .pic
            .update_call(platform, aaa_5, "get_task", encode_args(()).unwrap())
            .expect("get_task");
        let tr: Task = decode_one::<Result<Task, ApiError>>(&tc).unwrap().unwrap();
        if tr.subject.subject_id == target_subject_id {
            task_5 = Some(tr);
            break;
        }
    }
    let task_5 = task_5.expect("aaa_5 received subject 401");

    let sub_5_call = env
        .pic
        .update_call(
            platform,
            aaa_5,
            "submit_classification",
            encode_args((ClassificationSubmission {
                task_id: task_5.task_id,
                answers: vec![
                    Answer {
                        question_id: "q1".into(),
                        answer_id: "featured".into(),
                    },
                    Answer {
                        question_id: "q2".into(),
                        answer_id: "yes".into(),
                    },
                ],
                observed_image_sha256: vec![1; 32],
                discovery: None,
                agent_label: Some("cluster-bot".into()),
                submitted_by: owner_5,
            },))
            .unwrap(),
        )
        .expect("submit 5");
    let receipt_5: ClassificationReceipt =
        decode_one::<Result<ClassificationReceipt, ApiError>>(&sub_5_call)
            .unwrap()
            .unwrap();
    assert!(!receipt_5.duplicate);

    let subj_retired: Option<Subject> =
        env.query(platform, alice, "get_subject", target_subject_id);
    assert_eq!(subj_retired.unwrap().tally_count, 5);
    let consensus_after: Option<SubjectConsensus> =
        env.query(platform, alice, "get_subject_consensus", target_subject_id);
    let cons = consensus_after.expect("consensus reached");
    assert_eq!(cons.subject_id, target_subject_id);
    assert_eq!(cons.consensus.len(), 2);
    step("criterion 3 (part 2): 5th classification evaluated consensus and retired subject 401");

    let (aaa_6, _) = aaas[5];
    let mut reissued = false;
    for _ in 0..3 {
        let tc = env
            .pic
            .update_call(platform, aaa_6, "get_task", encode_args(()).unwrap())
            .expect("get_task 6");
        let tr_res = decode_one::<Result<Task, ApiError>>(&tc).unwrap();
        match tr_res {
            Ok(t) => {
                if t.subject.subject_id == target_subject_id {
                    reissued = true;
                }
            }
            Err(ApiError::NotFound) => break,
            Err(e) => panic!("unexpected error: {e:?}"),
        }
    }
    assert!(!reissued, "retired subject 401 must NEVER be reissued");
    step("criterion 3 (part 3): brand-new AAA-6 with empty seen-set NEVER receives retired subject 401");
}
