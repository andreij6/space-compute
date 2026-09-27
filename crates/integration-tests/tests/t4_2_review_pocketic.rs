use candid::{decode_one, encode_args, CandidType, Deserialize, Principal};
use integration_tests::pic::{canister_wasm, user, IcpEnv};
use integration_tests::step;
use platform::catalog::SubjectInput;
use platform::config::Params;
use platform::credits::{CreditOutcome, CreditPage, CreditRole, ListAaaCreditsArgs};
use platform::progression::AaaPublic;
use platform::registry::RegisterArgs;
use sc_types::{
    Answer, AnswerOption, ApiError, ClassificationReceipt, ClassificationSubmission,
    DiscoveryCategory, DiscoveryFlag, Protocol, Question, ReviewAssignment, ReviewReceipt,
    ReviewSubmission, SubjectRef, Task, Vote,
};
use sha2::{Digest, Sha256};

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

fn answers() -> Vec<Answer> {
    vec![Answer {
        question_id: "q1".into(),
        answer_id: "smooth".into(),
    }]
}

fn protocol() -> Protocol {
    Protocol {
        version: 1,
        questions: vec![Question {
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
                    next: None,
                },
            ],
        }],
        discovery_categories: vec![DiscoveryCategory {
            id: "lens".into(),
            label: "Gravitational Lens".into(),
            description: "Arcs or rings".into(),
        }],
        guidance_md: "Look closely.".into(),
    }
}

fn set_params(env: &IcpEnv, platform: Principal, admin: Principal, gold_bp: u16) {
    let mut params: Params = env.query(platform, admin, "get_params", ());
    params.fee_get_task = 0;
    params.fee_submit_classification = 0;
    params.fee_get_review = 0;
    params.fee_submit_review = 0;
    params.gold_rate_bp = gold_bp;
    params.calibration_gold_rate_bp = gold_bp;
    params.honeypot_rate_bp = 0;
    params.max_tasks_per_aaa_per_hour = 1_000;
    let ok: Result<(), ApiError> = env.update(platform, admin, "admin_set_params", params);
    assert_eq!(ok, Ok(()));
}

fn classify(
    env: &IcpEnv,
    platform: Principal,
    aaa: Principal,
    owner: Principal,
    flag: bool,
) -> ClassificationReceipt {
    let task: Task = decode_one::<Result<Task, ApiError>>(
        &env.pic
            .update_call(platform, aaa, "get_task", encode_args(()).unwrap())
            .unwrap(),
    )
    .unwrap()
    .expect("task");
    let sub = ClassificationSubmission {
        task_id: task.task_id,
        answers: answers(),
        observed_image_sha256: task.subject.image_sha256.clone(),
        discovery: flag.then(|| DiscoveryFlag {
            category: "lens".into(),
            rationale: "possible arc near the galaxy core".into(),
            confidence: 75,
            claim_position: None,
        }),
        agent_label: None,
        submitted_by: owner,
    };
    let r: Result<ClassificationReceipt, ApiError> =
        env.update(platform, aaa, "submit_classification", sub);
    r.expect("submit_classification")
}

#[derive(CandidType, Deserialize, Debug, PartialEq)]
struct Peek {
    assignment_id: u64,
    discoverer_aaa: Option<Principal>,
    discoverer_owner: Option<Principal>,
    discoverer_name_at_time: Option<String>,
    public_id: Option<String>,
    reviews: Option<Vec<Vote>>,
    needed_reviews: Option<u8>,
}

fn request_raw(env: &IcpEnv, platform: Principal, aaa: Principal, owner: Principal) -> Vec<u8> {
    env.pic
        .update_call(
            platform,
            aaa,
            "get_review_assignment",
            encode_args((Some(owner),)).unwrap(),
        )
        .expect("get_review_assignment")
}

#[test]
fn t4_2_review_assignment_tier_gate_blind_record_and_three_agrees_confirm() {
    println!("T4.2 demo: tier-1 AAAs get NotEligible, assignments are blind, and three agreeing tier-2 reviewers confirm a discovery");
    let env = IcpEnv::new();
    let admin = user(1);
    let payments = user(2);
    let platform = env.install_on("platform", admin, 10_000_000_000_000, 0);
    for _ in 0..2 {
        env.pic.tick();
    }
    let ok: Result<(), ApiError> = env.update(platform, admin, "admin_set_payments_id", payments);
    assert_eq!(ok, Ok(()));
    let wasm = canister_wasm("aaa");
    let hash = Sha256::digest(&wasm).to_vec();
    env.pic
        .update_call(
            platform,
            admin,
            "admin_upload_wasm",
            encode_args((1u32, wasm, hash)).unwrap(),
        )
        .unwrap();
    env.pic
        .update_call(
            platform,
            admin,
            "admin_approve_wasm",
            encode_args((1u32,)).unwrap(),
        )
        .unwrap();
    let ok: Result<(), ApiError> = env.update(platform, admin, "admin_add_protocol", protocol());
    assert_eq!(ok, Ok(()));
    let ok: Result<(), ApiError> = env.update(platform, admin, "admin_set_current_protocol", 1u16);
    assert_eq!(ok, Ok(()));

    let subnet = env.pic.topology().get_app_subnets()[0];
    let names = ["Discoverer", "Reviewer-A", "Reviewer-B", "Reviewer-C"];
    let agents: Vec<(Principal, Principal)> = names
        .iter()
        .enumerate()
        .map(|(i, name)| {
            let owner = user(101 + i as u8);
            let aaa = env.pic.create_canister_on_subnet(Some(admin), None, subnet);
            env.pic.add_cycles(aaa, 5_000_000_000_000);
            env.pic
                .set_controllers(aaa, Some(admin), vec![owner, platform])
                .unwrap();
            let reg = RegisterArgs {
                canister_id: aaa,
                owner,
                name: (*name).into(),
                avatar_seed: i as u64,
            };
            let ok: Result<(), ApiError> = env.update(platform, payments, "register_aaa", reg);
            assert_eq!(ok, Ok(()));
            (aaa, owner)
        })
        .collect();
    let (discoverer, discoverer_owner) = agents[0];

    let mut subjects: Vec<SubjectInput> = (1..=30u32)
        .map(|id| SubjectInput {
            subject: sample_ref(700 + id),
            gold: Some(answers()),
        })
        .collect();
    subjects.push(SubjectInput {
        subject: sample_ref(1),
        gold: None,
    });
    let added: Result<u32, ApiError> = env.update(platform, admin, "admin_add_subjects", subjects);
    assert_eq!(added, Ok(31));

    set_params(&env, platform, admin, 0);
    let flagged = classify(&env, platform, discoverer, discoverer_owner, true);
    let public_id = flagged.discovery_id.expect("discovery created");
    step(&format!("discoverer flagged {public_id}"));

    let res: Result<Option<ReviewAssignment>, ApiError> =
        decode_one(&request_raw(&env, platform, agents[1].0, agents[1].1)).unwrap();
    assert_eq!(res, Err(ApiError::NotEligible("tier".into())));
    step("a tier-1 AAA asking for a review gets NotEligible(\"tier\")");

    set_params(&env, platform, admin, 10_000);
    for &(aaa, owner) in &agents[1..] {
        for _ in 0..25 {
            classify(&env, platform, aaa, owner, false);
        }
        let public: Option<AaaPublic> = env.query(platform, admin, "get_aaa_public", aaa);
        assert!(public.unwrap().tier >= 2);
    }
    step("three reviewers reached tier 2 through 25 honest gold tasks each");

    let mut receipts = vec![];
    for &(aaa, owner) in &agents[1..] {
        let raw = request_raw(&env, platform, aaa, owner);
        for secret in [
            public_id.as_bytes(),
            b"Discoverer".as_slice(),
            discoverer.as_slice(),
            discoverer_owner.as_slice(),
        ] {
            assert!(
                !raw.windows(secret.len()).any(|w| w == secret),
                "assignment leaked {secret:?}"
            );
        }
        let peek = decode_one::<Result<Option<Peek>, ApiError>>(&raw)
            .unwrap()
            .unwrap()
            .unwrap();
        assert_eq!(
            (
                peek.discoverer_aaa,
                peek.discoverer_owner,
                peek.discoverer_name_at_time,
                peek.public_id,
                peek.reviews,
                peek.needed_reviews
            ),
            (None, None, None, None, None, None)
        );
        let a: ReviewAssignment = decode_one::<Result<Option<ReviewAssignment>, ApiError>>(&raw)
            .unwrap()
            .unwrap()
            .expect("an eligible discovery is assigned");
        assert_eq!(a.subject.subject_id, 1);
        assert_eq!(a.category, "lens");
        let r: Result<ReviewReceipt, ApiError> = env.update(
            platform,
            aaa,
            "submit_review",
            ReviewSubmission {
                assignment_id: a.assignment_id,
                vote: Vote::Agree,
                rationale: "a clear tangential arc around the lens galaxy".into(),
                observed_image_sha256: vec![1; 32],
                agent_label: None,
                submitted_by: owner,
            },
        );
        receipts.push(r.expect("submit_review"));
    }
    step("each assignment carried only subject, protocol, category, rationale and lease: no discoverer, public id, votes or tallies");
    assert_eq!(receipts[2].xp_awarded, 5);

    let credits: CreditPage = env.query(
        platform,
        admin,
        "list_aaa_credits",
        ListAaaCreditsArgs {
            aaa: discoverer,
            cursor: 0,
        },
    );
    let credit = credits
        .items
        .iter()
        .find(|c| c.public_id == public_id)
        .expect("discoverer credited");
    assert_eq!(credit.role, CreditRole::Discoverer);
    assert_eq!(credit.outcome, CreditOutcome::Confirmed);
    let public: Option<AaaPublic> = env.query(platform, admin, "get_aaa_public", discoverer);
    assert!(public.unwrap().xp >= 50);
    step(&format!(
        "three agreeing reviewers resolved {public_id} to Confirmed; discoverer +50 XP and credited"
    ));

    let again: Result<Option<ReviewAssignment>, ApiError> =
        decode_one(&request_raw(&env, platform, agents[1].0, agents[1].1)).unwrap();
    assert_eq!(again, Ok(None));
    step("nothing left to review: get_review_assignment returns None");
}
