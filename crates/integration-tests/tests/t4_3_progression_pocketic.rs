use candid::{decode_one, encode_args, Principal};
use integration_tests::pic::{canister_wasm, user, IcpEnv};
use integration_tests::step;
use platform::catalog::SubjectInput;
use platform::config::Params;
use platform::credits::{CreditPage, ListAaaCreditsArgs};
use platform::progression::{AaaPublic, LeaderPage, ReplayStatus};
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

fn correct_answers() -> Vec<Answer> {
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
}

#[test]
fn t4_3_new_aaa_reaches_tier2_in_60_tasks_and_replay_matches_incremental() {
    println!(
        "T4.3 demo: Criterion 8 & 9 — tier 2 within 60 honest tasks, and replay == incremental"
    );
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
        name: "Tier-Climber".into(),
        avatar_seed: 3,
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
    params.gold_rate_bp = 10_000;
    params.calibration_gold_rate_bp = 10_000;
    params.max_open_leases_per_aaa = 5;
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

    let gold_batch: Vec<SubjectInput> = (1..=60u32)
        .map(|id| SubjectInput {
            subject: sample_ref(500 + id),
            gold: Some(correct_answers()),
        })
        .collect();
    let added: Result<u32, ApiError> =
        env.update(platform, alice, "admin_add_subjects", gold_batch);
    assert_eq!(added, Ok(60));
    step("environment initialized: platform, AAA registered, protocol v1, 60 gold subjects seeded");

    let mut tier2_reached_at: Option<u32> = None;
    for i in 1..=60u32 {
        let task_call = env
            .pic
            .update_call(platform, aaa_1, "get_task", encode_args(()).unwrap())
            .expect("get_task");
        let task: Task = decode_one::<Result<Task, ApiError>>(&task_call)
            .unwrap()
            .unwrap_or_else(|e| panic!("get_task {i} failed: {e:?}"));

        let sub_call = env
            .pic
            .update_call(
                platform,
                aaa_1,
                "submit_classification",
                encode_args((ClassificationSubmission {
                    task_id: task.task_id,
                    answers: correct_answers(),
                    observed_image_sha256: task.subject.image_sha256.clone(),
                    discovery: None,
                    agent_label: Some("tier-climber-bot".into()),
                    submitted_by: owner_1,
                },))
                .unwrap(),
            )
            .expect("submit_classification");
        let receipt: ClassificationReceipt =
            decode_one::<Result<ClassificationReceipt, ApiError>>(&sub_call)
                .unwrap()
                .unwrap_or_else(|e| panic!("submit_classification {i} failed: {e:?}"));
        assert!(!receipt.duplicate);

        if tier2_reached_at.is_none() {
            let public: Option<AaaPublic> = env.query(platform, alice, "get_aaa_public", aaa_1);
            if public.expect("aaa public profile").tier >= 2 {
                tier2_reached_at = Some(i);
            }
        }
    }

    let reached =
        tier2_reached_at.expect("tier 2 must be reached within 60 honest all-correct tasks");
    assert!(reached <= 60, "expected tier 2 by task 60, got {reached}");
    step(&format!(
        "criterion 9: a new AAA reached tier 2 after {reached} honest all-correct tasks (<= 60)"
    ));

    let credits_page: CreditPage = env.query(
        platform,
        alice,
        "list_aaa_credits",
        ListAaaCreditsArgs {
            aaa: aaa_1,
            cursor: 0,
        },
    );
    assert!(credits_page.items.is_empty());
    assert!(credits_page.next_cursor.is_none());
    step("list_aaa_credits answers the AAA's daily pull (empty page; no discoveries yet) instead of trapping");

    let before_public: AaaPublic = env
        .query::<Principal, Option<AaaPublic>>(platform, alice, "get_aaa_public", aaa_1)
        .expect("public profile before replay");
    let before_leaderboard = fetch_leaderboard(&env, platform, alice);

    let replay_bytes = env
        .pic
        .update_call(
            platform,
            alice,
            "admin_replay_progression",
            encode_args((0u64, 5_000u32)).unwrap(),
        )
        .expect("admin_replay_progression");
    let replay_res: Result<ReplayStatus, ApiError> = decode_one(&replay_bytes).unwrap();
    let status = replay_res.expect("replay succeeds");
    assert!(status.done);
    tick(&env, 3);

    let after_public: AaaPublic = env
        .query::<Principal, Option<AaaPublic>>(platform, alice, "get_aaa_public", aaa_1)
        .expect("public profile after replay");
    let after_leaderboard = fetch_leaderboard(&env, platform, alice);

    assert_eq!(before_public, after_public);
    assert_eq!(before_leaderboard, after_leaderboard);
    step("criterion 8: replaying the event log from event 0 reproduces an identical Progress for the AAA");
}

fn fetch_leaderboard(env: &IcpEnv, platform: Principal, caller: Principal) -> LeaderPage {
    let bytes = env
        .pic
        .query_call(
            platform,
            caller,
            "get_leaderboard",
            encode_args((None::<platform::progression::LeaderCursor>, 100u32)).unwrap(),
        )
        .expect("get_leaderboard");
    decode_one(&bytes).unwrap()
}
