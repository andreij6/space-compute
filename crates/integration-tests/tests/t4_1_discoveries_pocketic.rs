use candid::{decode_one, encode_args, Principal};
use integration_tests::pic::{canister_wasm, user, IcpEnv};
use integration_tests::step;
use platform::catalog::SubjectInput;
use platform::config::Params;
use platform::registry::RegisterArgs;
use sc_types::{
    Answer, AnswerOption, ApiError, ClassificationReceipt, ClassificationSubmission,
    DiscoveryCategory, DiscoveryFlag, Protocol, Question, SubjectRef, Task,
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

fn discovery_flag() -> DiscoveryFlag {
    DiscoveryFlag {
        category: "lens".into(),
        rationale: "possible arc near the galaxy core".into(),
        confidence: 75,
        claim_position: None,
    }
}

fn submit(
    env: &IcpEnv,
    platform: Principal,
    aaa: Principal,
    owner: Principal,
    task: &Task,
    flag: bool,
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
                discovery: flag.then(discovery_flag),
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
fn t4_1_discovery_flag_public_id_and_rolling_rate_limit() {
    println!("T4.1 demo: a discovery flag creates a Discovery with a well-formed public_id, and the 101st flag in a rolling 100-classification window is rejected while the classification itself still succeeds");
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
        name: "Flag-Runner".into(),
        avatar_seed: 7,
    };
    let ok: Result<(), ApiError> = env.update(platform, payments, "register_aaa", reg);
    assert_eq!(ok, Ok(()));

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
    params.max_open_leases_per_aaa = 5;
    params.max_tasks_per_aaa_per_hour = 200;
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

    let subjects: Vec<SubjectInput> = (1..=101u32)
        .map(|id| SubjectInput {
            subject: sample_ref(600 + id),
            gold: None,
        })
        .collect();
    let added: Result<u32, ApiError> = env.update(platform, alice, "admin_add_subjects", subjects);
    assert_eq!(added, Ok(101));
    step("environment initialized: platform, AAA registered, protocol v1 with the 'lens' discovery category, 101 subjects seeded");

    let task1 = get_task(&env, platform, aaa_1);
    let receipt1 = submit(&env, platform, aaa_1, owner_1, &task1, true)
        .unwrap_or_else(|e| panic!("first flagged submission failed: {e:?}"));
    let public_id = receipt1
        .discovery_id
        .expect("a discovery flag must create a Discovery with a public_id");
    let parts: Vec<&str> = public_id.split('-').collect();
    assert_eq!(parts.len(), 3, "public_id must be SC-{{year}}-{{seq:06}}");
    assert_eq!(parts[0], "SC");
    assert_eq!(parts[1].len(), 4);
    assert!(parts[1].chars().all(|c| c.is_ascii_digit()));
    assert_eq!(parts[2].len(), 6);
    assert!(parts[2].chars().all(|c| c.is_ascii_digit()));
    step(&format!(
        "criterion: discovery flag created Discovery with well-formed public_id {public_id}"
    ));

    for _ in 2..=10u32 {
        let task = get_task(&env, platform, aaa_1);
        let r = submit(&env, platform, aaa_1, owner_1, &task, true)
            .expect("flag within the 10% cap must succeed");
        assert!(
            r.discovery_id.is_some(),
            "flags 2-10 of the first 100 classifications stay within max_flag_rate_bp"
        );
    }

    for _ in 11..=100u32 {
        let task = get_task(&env, platform, aaa_1);
        let r = submit(&env, platform, aaa_1, owner_1, &task, false)
            .expect("unflagged classification always succeeds");
        assert!(r.discovery_id.is_none());
        assert!(!r.duplicate);
    }

    let task_101 = get_task(&env, platform, aaa_1);
    let r101 = submit(&env, platform, aaa_1, owner_1, &task_101, true)
        .unwrap_or_else(|e| panic!("the 101st classification itself must still succeed: {e:?}"));
    assert!(
        r101.discovery_id.is_none(),
        "the 101st flag exceeds max_flag_rate_bp over the AAA's last 100 classifications, so the flag is dropped"
    );
    assert!(
        !r101.duplicate,
        "rate-limiting the flag never fails or duplicates the underlying classification"
    );
    step("criterion: the 101st flag in the rolling 100-classification window is silently dropped (max_flag_rate_bp enforced) while submit_classification itself still returns Ok");
}
