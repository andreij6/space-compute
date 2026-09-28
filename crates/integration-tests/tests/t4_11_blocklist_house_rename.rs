use candid::{decode_one, encode_args, encode_one, CandidType, Principal};
use integration_tests::pic::{canister_wasm, user, IcpEnv};
use integration_tests::step;
use platform::catalog::SubjectInput;
use platform::citations::CertifiedCitation;
use platform::config::Params;
use platform::progression::AaaPublic;
use platform::registry::{AaaRecord, RegisterArgs};
use sc_types::{
    Answer, AnswerOption, ApiError, ClassificationReceipt, ClassificationSubmission,
    DiscoveryCategory, DiscoveryFlag, Protocol, Question, ReviewAssignment, ReviewReceipt,
    ReviewSubmission, SubjectRef, Task, Vote,
};
use serde::de::DeserializeOwned;
use sha2::{Digest, Sha256};

fn call<A: candid::utils::ArgumentEncoder, R: DeserializeOwned + CandidType>(
    env: &IcpEnv,
    canister: Principal,
    sender: Principal,
    method: &str,
    args: A,
) -> R {
    let bytes = env
        .pic
        .update_call(canister, sender, method, encode_args(args).unwrap())
        .unwrap_or_else(|e| panic!("{method} rejected: {e:?}"));
    decode_one(&bytes).unwrap_or_else(|e| panic!("{method} reply did not decode: {e}"))
}

fn query<A: candid::utils::ArgumentEncoder, R: DeserializeOwned + CandidType>(
    env: &IcpEnv,
    canister: Principal,
    sender: Principal,
    method: &str,
    args: A,
) -> R {
    let bytes = env
        .pic
        .query_call(canister, sender, method, encode_args(args).unwrap())
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
            answers: ["smooth", "featured"]
                .iter()
                .map(|a| AnswerOption {
                    id: (*a).into(),
                    label: (*a).into(),
                    next: None,
                })
                .collect(),
        }],
        discovery_categories: vec![DiscoveryCategory {
            id: "lens".into(),
            label: "Gravitational Lens".into(),
            description: "Arcs or rings".into(),
        }],
        guidance_md: "Look closely.".into(),
    }
}

struct World {
    env: IcpEnv,
    admin: Principal,
    payments: Principal,
    platform: Principal,
    agents: Vec<(Principal, Principal)>,
}

impl World {
    fn new() -> Self {
        let env = IcpEnv::new();
        let admin = user(1);
        let payments = user(2);
        let subnet = env.pic.topology().get_app_subnets()[0];
        let platform = env.install_on("platform", admin, 10_000_000_000_000, 0);
        for _ in 0..2 {
            env.pic.tick();
        }
        let ok: Result<(), ApiError> =
            env.update(platform, admin, "admin_set_payments_id", payments);
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
        let ok: Result<(), ApiError> =
            env.update(platform, admin, "admin_add_protocol", protocol());
        assert_eq!(ok, Ok(()));
        let ok: Result<(), ApiError> =
            env.update(platform, admin, "admin_set_current_protocol", 1u16);
        assert_eq!(ok, Ok(()));
        let agents = ["Discoverer", "Reviewer-A", "Reviewer-B", "Reviewer-C"]
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
        let added: Result<u32, ApiError> =
            env.update(platform, admin, "admin_add_subjects", subjects);
        assert_eq!(added, Ok(31));
        World {
            env,
            admin,
            payments,
            platform,
            agents,
        }
    }

    fn set_params(&self, gold_bp: u16, reviews_min: u16) {
        let mut params: Params = self.env.query(self.platform, self.admin, "get_params", ());
        params.fee_get_task = 0;
        params.fee_submit_classification = 0;
        params.fee_get_review = 0;
        params.fee_submit_review = 0;
        params.gold_rate_bp = gold_bp;
        params.calibration_gold_rate_bp = gold_bp;
        params.honeypot_rate_bp = 0;
        params.max_tasks_per_aaa_per_hour = 1_000;
        params.reviews_min = reviews_min;
        let ok: Result<(), ApiError> =
            self.env
                .update(self.platform, self.admin, "admin_set_params", params);
        assert_eq!(ok, Ok(()));
    }

    fn classify(&self, (aaa, owner): (Principal, Principal), flag: bool) -> ClassificationReceipt {
        let task: Task = decode_one::<Result<Task, ApiError>>(
            &self
                .env
                .pic
                .update_call(self.platform, aaa, "get_task", encode_args(()).unwrap())
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
            self.env
                .update(self.platform, aaa, "submit_classification", sub);
        r.expect("submit_classification")
    }

    fn flag_and_promote_reviewers(&self, reviews_min: u16) -> String {
        self.set_params(0, reviews_min);
        let public_id = self
            .classify(self.agents[0], true)
            .discovery_id
            .expect("discovery created");
        self.set_params(10_000, reviews_min);
        for &agent in &self.agents[1..] {
            for _ in 0..25 {
                self.classify(agent, false);
            }
            assert!(self.public(agent.0).tier >= 2);
        }
        public_id
    }

    fn assignment(&self, (aaa, owner): (Principal, Principal)) -> ReviewAssignment {
        let r: Result<Option<ReviewAssignment>, ApiError> =
            self.env
                .update(self.platform, aaa, "get_review_assignment", Some(owner));
        r.unwrap().expect("an eligible discovery is assigned")
    }

    fn review(&self, (aaa, owner): (Principal, Principal), assignment_id: u64) -> ReviewReceipt {
        let sub = ReviewSubmission {
            assignment_id,
            vote: Vote::Agree,
            rationale: "a clear tangential arc around the lens galaxy".into(),
            observed_image_sha256: vec![1; 32],
            agent_label: None,
            submitted_by: owner,
        };
        decode_one::<Result<ReviewReceipt, ApiError>>(
            &self
                .env
                .pic
                .update_call(
                    self.platform,
                    aaa,
                    "submit_review",
                    encode_one(sub).unwrap(),
                )
                .unwrap(),
        )
        .unwrap()
        .expect("submit_review")
    }

    fn public(&self, aaa: Principal) -> AaaPublic {
        let p: Option<AaaPublic> = self
            .env
            .query(self.platform, self.admin, "get_aaa_public", aaa);
        p.unwrap()
    }

    fn citation(&self, public_id: &str) -> CertifiedCitation {
        let c: Option<CertifiedCitation> = self.env.query(
            self.platform,
            user(200),
            "get_citation",
            public_id.to_string(),
        );
        c.expect("citation exists")
    }
}

#[test]
fn t4_11_renamed_aaa_keeps_historical_citation_name() {
    println!("T4.11 demo: admin_rename_aaa changes the live profile, but a confirmed citation keeps the discoverer's name at the time of discovery");
    let w = World::new();
    let public_id = w.flag_and_promote_reviewers(3);
    for &agent in &w.agents[1..] {
        let id = w.assignment(agent).assignment_id;
        w.review(agent, id);
    }

    let before = w.citation(&public_id);
    assert_eq!(before.citation.discoverer.aaa_name_at_time, "Discoverer");
    let discoverer_aaa = w.agents[0].0;
    assert_eq!(w.public(discoverer_aaa).name, "Discoverer");
    step("discovery confirmed; citation and live profile both show the original name 'Discoverer'");

    let renamed: Result<(), ApiError> = call(
        &w.env,
        w.platform,
        w.admin,
        "admin_rename_aaa",
        (
            discoverer_aaa,
            "Renamed-Surveyor".to_string(),
            "brand cleanup".to_string(),
        ),
    );
    assert_eq!(renamed, Ok(()));

    let after = w.citation(&public_id);
    assert_eq!(after.citation.discoverer.aaa_name_at_time, "Discoverer");
    assert_eq!(after.citation, before.citation);
    step("after admin_rename_aaa, get_citation is unchanged: discoverer_name_at_time is still 'Discoverer'");

    let live = w.public(discoverer_aaa);
    assert_eq!(live.name, "Renamed-Surveyor");
    let record: Option<AaaRecord> = w.env.query(w.platform, w.admin, "get_aaa", discoverer_aaa);
    assert_eq!(record.unwrap().name, "Renamed-Surveyor");
    step("get_aaa_public and get_aaa now show the new name 'Renamed-Surveyor' — history and identity diverge as designed");
}

#[test]
fn t4_11_name_blocklist_blocks_registration_and_profile_update_but_not_admin_rename() {
    println!("T4.11 demo: the name blocklist gates registration and self-service renames, but admin_rename_aaa can override it for moderation");
    let w = World::new();
    let subnet = w.env.pic.topology().get_app_subnets()[0];
    let owner = user(150);
    let canister = w
        .env
        .pic
        .create_canister_on_subnet(Some(w.admin), None, subnet);
    w.env.pic.add_cycles(canister, 5_000_000_000_000);
    w.env
        .pic
        .set_controllers(canister, Some(w.admin), vec![owner, w.platform])
        .unwrap();
    let blocked_reg = RegisterArgs {
        canister_id: canister,
        owner,
        name: "Pl4tf0rm-Admin".into(),
        avatar_seed: 9,
    };
    let rejected: Result<(), ApiError> =
        w.env
            .update(w.platform, w.payments, "register_aaa", blocked_reg);
    assert!(matches!(rejected, Err(ApiError::InvalidInput(_))));
    step("register_aaa with a leetspeak blocklisted name ('Pl4tf0rm-Admin') is rejected");

    let clean_reg = RegisterArgs {
        canister_id: canister,
        owner,
        name: "Clean-Surveyor".into(),
        avatar_seed: 9,
    };
    let ok: Result<(), ApiError> = w
        .env
        .update(w.platform, w.payments, "register_aaa", clean_reg);
    assert_eq!(ok, Ok(()));

    let blocked_update: Result<(), ApiError> = w.env.update(
        w.platform,
        canister,
        "update_aaa_profile",
        platform::registry::UpdateAaaProfileArgs {
            name: Some("Official-Support".into()),
            avatar_seed: None,
        },
    );
    assert!(matches!(blocked_update, Err(ApiError::InvalidInput(_))));
    step("update_aaa_profile to a blocklisted name ('Official-Support') is rejected");

    let admin_override: Result<(), ApiError> = call(
        &w.env,
        w.platform,
        w.admin,
        "admin_rename_aaa",
        (
            canister,
            "Official-Support-Team".to_string(),
            "designated house account".to_string(),
        ),
    );
    assert_eq!(admin_override, Ok(()));
    let record: Option<AaaRecord> = w.env.query(w.platform, w.admin, "get_aaa", canister);
    assert_eq!(record.unwrap().name, "Official-Support-Team");
    step("admin_rename_aaa bypasses the blocklist for legitimate moderation/house-account naming");
}

#[test]
fn t4_11_house_aaas_are_labeled_and_excluded_from_the_leaderboard() {
    println!("T4.11 demo: admin_set_house marks a team AAA as house; it keeps progressing but is excluded from the public leaderboard");
    let w = World::new();
    w.flag_and_promote_reviewers(3);
    let house_aaa = w.agents[1].0;
    assert!(w.public(house_aaa).tier >= 2);

    let on_leaderboard =
        |env: &IcpEnv, platform: Principal, admin: Principal, target: Principal| {
            let page: platform::progression::LeaderPage = query(
                env,
                platform,
                admin,
                "get_leaderboard",
                (None::<platform::progression::LeaderCursor>, 100u32),
            );
            page.items.iter().any(|row| row.aaa == target)
        };
    assert!(on_leaderboard(&w.env, w.platform, w.admin, house_aaa));

    let ok: Result<(), ApiError> = call(
        &w.env,
        w.platform,
        w.admin,
        "admin_set_house",
        (house_aaa, true),
    );
    assert_eq!(ok, Ok(()));
    assert!(!on_leaderboard(&w.env, w.platform, w.admin, house_aaa));
    let pub_house = w.public(house_aaa);
    assert!(pub_house.is_house);
    assert!(pub_house.tier >= 2);
    step("house AAA disappears from the leaderboard immediately but keeps its tier/progress (is_house=true)");

    let ok: Result<(), ApiError> = call(
        &w.env,
        w.platform,
        w.admin,
        "admin_set_house",
        (house_aaa, false),
    );
    assert_eq!(ok, Ok(()));
    assert!(on_leaderboard(&w.env, w.platform, w.admin, house_aaa));
    step("un-housing an AAA restores its leaderboard visibility");
}
