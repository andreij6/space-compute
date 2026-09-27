use candid::{decode_one, encode_args, Principal};
use integration_tests::pic::{canister_wasm, user, IcpEnv};
use integration_tests::step;
use platform::catalog::SubjectInput;
use platform::config::Params;
use platform::registry::RegisterArgs;
use sc_types::{
    Answer, AnswerOption, ApiError, ClaimOutcome, ClaimPosition, ClassificationReceipt,
    ClassificationSubmission, DiscoveryCategory, DiscoveryFlag, Protocol, Question, SubjectRef,
    Task,
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
        discovery_categories: vec![
            DiscoveryCategory {
                id: "lens".into(),
                label: "Gravitational Lens".into(),
                description: "Arcs or rings".into(),
            },
            DiscoveryCategory {
                id: "merger".into(),
                label: "Merger".into(),
                description: "Interacting galaxies".into(),
            },
        ],
        guidance_md: "Look closely at the image.".into(),
    }
}

fn discovery_flag(category: &str) -> DiscoveryFlag {
    DiscoveryFlag {
        category: category.into(),
        rationale: "possible arc near the galaxy core".into(),
        confidence: 75,
        claim_position: Some(ClaimPosition {
            ra_deg: 214.95,
            dec_deg: 52.85,
        }),
    }
}

fn submit(
    env: &IcpEnv,
    platform: Principal,
    aaa: Principal,
    owner: Principal,
    task: &Task,
    category: &str,
) -> Result<ClassificationReceipt, ApiError> {
    let bytes = env
        .pic
        .update_call(
            platform,
            aaa,
            "submit_classification",
            encode_args((ClassificationSubmission {
                task_id: task.task_id,
                answers: vec![Answer {
                    question_id: "q1".into(),
                    answer_id: "smooth".into(),
                }],
                observed_image_sha256: task.subject.image_sha256.clone(),
                discovery: Some(discovery_flag(category)),
                agent_label: Some("flag-bot".into()),
                submitted_by: owner,
            },))
            .unwrap(),
        )
        .expect("submit_classification");
    decode_one(&bytes).unwrap()
}

fn get_task(env: &IcpEnv, platform: Principal, aaa: Principal) -> Task {
    let bytes = env
        .pic
        .update_call(platform, aaa, "get_task", encode_args(()).unwrap())
        .expect("get_task");
    decode_one::<Result<Task, ApiError>>(&bytes)
        .unwrap()
        .expect("task available")
}

#[test]
fn t4_9_concurrent_same_cell_flags_resolve_to_one_discovery() {
    println!("T4.9 demo: three AAAs flag the same sky position + category in the same round; the first claim wins one Discovery and the other two are recorded as corroborations, while a different category at the same position opens its own Discovery");
    let env = IcpEnv::new();
    let alice = user(1);
    let payments = user(2);
    let platform = env.install_on("platform", alice, 10_000_000_000_000, 0);
    tick(&env, 2);
    let ok: Result<(), ApiError> = env.update(platform, alice, "admin_set_payments_id", payments);
    assert_eq!(ok, Ok(()));

    let wasm = canister_wasm("aaa");
    let hash = Sha256::digest(&wasm).to_vec();
    let _ = env.pic.update_call(
        platform,
        alice,
        "admin_upload_wasm",
        encode_args((1u32, wasm, hash)).unwrap(),
    );
    let _ = env.pic.update_call(
        platform,
        alice,
        "admin_approve_wasm",
        encode_args((1u32,)).unwrap(),
    );

    let proto = sample_protocol(1);
    let ok: Result<(), ApiError> = env.update(platform, alice, "admin_add_protocol", proto);
    assert_eq!(ok, Ok(()));
    let ok: Result<(), ApiError> = env.update(platform, alice, "admin_set_current_protocol", 1u16);
    assert_eq!(ok, Ok(()));

    let p_bytes = env
        .pic
        .query_call(platform, alice, "get_params", encode_args(()).unwrap())
        .expect("get_params");
    let mut params: Params = decode_one(&p_bytes).unwrap();
    params.fee_get_task = 0;
    params.fee_submit_classification = 0;
    params.gold_rate_bp = 0;
    params.calibration_gold_rate_bp = 0;
    params.calibration_tasks = 0;
    params.max_flag_rate_bp = 1_000;
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

    let subjects: Vec<SubjectInput> = (1..=8u32)
        .map(|id| SubjectInput {
            subject: sample_ref(900 + id),
            gold: None,
        })
        .collect();
    let added: Result<u32, ApiError> = env.update(platform, alice, "admin_add_subjects", subjects);
    assert_eq!(added, Ok(8));

    let subnet = env.pic.topology().get_app_subnets()[0];
    let aaas: Vec<(Principal, Principal)> = (0..3u8)
        .map(|i| {
            let owner = user(201 + i);
            let aaa = env.pic.create_canister_on_subnet(Some(alice), None, subnet);
            env.pic.add_cycles(aaa, 5_000_000_000_000);
            env.pic
                .set_controllers(aaa, Some(alice), vec![owner, platform])
                .unwrap();
            let reg = RegisterArgs {
                canister_id: aaa,
                owner,
                name: format!("Claimer-{i}"),
                avatar_seed: i as u64,
            };
            let ok: Result<(), ApiError> = env.update(platform, payments, "register_aaa", reg);
            assert_eq!(ok, Ok(()));
            (aaa, owner)
        })
        .collect();
    let tasks: Vec<Task> = aaas
        .iter()
        .map(|(aaa, _)| get_task(&env, platform, *aaa))
        .collect();
    step("three AAAs registered, each leased a different subject whose cutout covers the same claim position");

    let msgs: Vec<_> = aaas
        .iter()
        .zip(&tasks)
        .map(|((aaa, owner), task)| {
            env.pic
                .submit_call(
                    platform,
                    *aaa,
                    "submit_classification",
                    encode_args((ClassificationSubmission {
                        task_id: task.task_id,
                        answers: vec![Answer {
                            question_id: "q1".into(),
                            answer_id: "smooth".into(),
                        }],
                        observed_image_sha256: task.subject.image_sha256.clone(),
                        discovery: Some(discovery_flag("lens")),
                        agent_label: None,
                        submitted_by: *owner,
                    },))
                    .unwrap(),
                )
                .expect("submit")
        })
        .collect();
    tick(&env, 3);
    let receipts: Vec<ClassificationReceipt> = msgs
        .into_iter()
        .map(|m| {
            let bytes = env.pic.await_call(m).expect("await");
            decode_one::<Result<ClassificationReceipt, ApiError>>(&bytes)
                .unwrap()
                .expect("receipt")
        })
        .collect();

    let created: Vec<&String> = receipts
        .iter()
        .filter_map(|r| r.discovery_id.as_ref())
        .collect();
    assert_eq!(
        created.len(),
        1,
        "exactly one Discovery for the same cell + category"
    );
    let public_id = created[0].clone();
    let new_count = receipts
        .iter()
        .filter(|r| r.claim == Some(ClaimOutcome::New))
        .count();
    let corroborations = receipts
        .iter()
        .filter(|r| r.claim == Some(ClaimOutcome::Corroborates(public_id.clone())))
        .count();
    assert_eq!((new_count, corroborations), (1, 2));
    step(&format!(
        "criterion: concurrent flags -> one discovery ({public_id}) + 2 corroborations"
    ));

    let (aaa, owner) = aaas[1];
    let task = get_task(&env, platform, aaa);
    let r = submit(&env, platform, aaa, owner, &task, "merger").expect("merger flag");
    assert_eq!(r.claim, Some(ClaimOutcome::New));
    let merger_id = r
        .discovery_id
        .expect("a different category opens a separate Discovery");
    assert_ne!(merger_id, public_id);
    step(&format!(
        "criterion: a different category at the same position opens its own Discovery ({merger_id})"
    ));
}
