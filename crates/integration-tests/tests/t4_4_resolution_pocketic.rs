use std::time::Duration;

use candid::{decode_one, encode_args, encode_one, Principal};
use integration_tests::pic::{canister_wasm, user, IcpEnv};
use integration_tests::{repo_root, step};
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

fn fault_injection_platform_wasm() -> Vec<u8> {
    let root = repo_root();
    let target = root.join("target/fault-injection");
    let ok = std::process::Command::new("cargo")
        .args([
            "build",
            "-q",
            "--target",
            "wasm32-unknown-unknown",
            "--release",
        ])
        .args(["-p", "platform", "--features", "fault-injection"])
        .arg("--target-dir")
        .arg(&target)
        .current_dir(&root)
        .status()
        .expect("cargo")
        .success();
    assert!(ok, "building the fault-injection platform wasm failed");
    std::fs::read(target.join("wasm32-unknown-unknown/release/platform.wasm")).unwrap()
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
    platform: Principal,
    agents: Vec<(Principal, Principal)>,
}

impl World {
    fn new(platform_wasm: Option<Vec<u8>>) -> Self {
        let env = IcpEnv::new();
        let admin = user(1);
        let payments = user(2);
        let subnet = env.pic.topology().get_app_subnets()[0];
        let platform = match platform_wasm {
            Some(wasm) => {
                let id = env.pic.create_canister_on_subnet(Some(admin), None, subnet);
                env.pic.add_cycles(id, 10_000_000_000_000);
                env.pic
                    .install_canister(id, wasm, encode_one(()).unwrap(), Some(admin));
                id
            }
            None => env.install_on("platform", admin, 10_000_000_000_000, 0),
        };
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

    fn review_raw(
        &self,
        (aaa, owner): (Principal, Principal),
        assignment_id: u64,
    ) -> Result<Vec<u8>, pocket_ic::RejectResponse> {
        let sub = ReviewSubmission {
            assignment_id,
            vote: Vote::Agree,
            rationale: "a clear tangential arc around the lens galaxy".into(),
            observed_image_sha256: vec![1; 32],
            agent_label: None,
            submitted_by: owner,
        };
        self.env.pic.update_call(
            self.platform,
            aaa,
            "submit_review",
            encode_one(sub).unwrap(),
        )
    }

    fn review(&self, agent: (Principal, Principal), assignment_id: u64) -> ReviewReceipt {
        decode_one::<Result<ReviewReceipt, ApiError>>(
            &self.review_raw(agent, assignment_id).unwrap(),
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

    fn credits(&self, aaa: Principal) -> CreditPage {
        self.env.query(
            self.platform,
            self.admin,
            "list_aaa_credits",
            ListAaaCreditsArgs { aaa, cursor: 0 },
        )
    }

    fn assert_resolved_confirmed(&self, public_id: &str) {
        let d = self.credits(self.agents[0].0);
        let c = d.items.iter().find(|c| c.public_id == public_id).unwrap();
        assert_eq!(
            (c.role, c.outcome),
            (CreditRole::Discoverer, CreditOutcome::Confirmed)
        );
        for &(aaa, _) in &self.agents[1..] {
            let page = self.credits(aaa);
            let c = page
                .items
                .iter()
                .find(|c| c.public_id == public_id)
                .unwrap();
            assert_eq!(
                (c.role, c.outcome),
                (CreditRole::Reviewer, CreditOutcome::Confirmed)
            );
        }
    }
}

#[test]
fn t4_4_injected_fault_after_resolution_rolls_back_the_whole_message() {
    println!("T4.4 demo: 3 equal-weight agreeing reviews confirm a discovery; a fault injected after resolution rolls back review, citation, credits and XP as one unit");
    let w = World::new(Some(fault_injection_platform_wasm()));
    let public_id = w.flag_and_promote_reviewers(3);
    let reps: Vec<u32> = w.agents[1..]
        .iter()
        .map(|a| w.public(a.0).reputation_bp)
        .collect();
    assert!(reps.iter().all(|r| *r == reps[0]));
    step(&format!(
        "{public_id} flagged; three reviewers at tier 2 with equal weight {} bp",
        reps[0]
    ));
    let ids: Vec<u64> = w.agents[1..]
        .iter()
        .map(|&a| w.assignment(a).assignment_id)
        .collect();
    for (&agent, &id) in w.agents[1..3].iter().zip(&ids) {
        w.review(agent, id);
    }
    let xp_before = w.public(w.agents[0].0).xp;
    let last_before = w.public(w.agents[3].0);

    let ok: Result<(), ApiError> =
        w.env
            .update(w.platform, w.admin, "debug_set_review_fault", true);
    assert_eq!(ok, Ok(()));
    let trapped = w.review_raw(w.agents[3], ids[2]).unwrap_err();
    assert!(format!("{trapped:?}").contains("injected fault after review resolution"));
    assert!(w.credits(w.agents[0].0).items.is_empty());
    assert!(w.agents[1..]
        .iter()
        .all(|a| w.credits(a.0).items.is_empty()));
    assert_eq!(w.public(w.agents[0].0).xp, xp_before);
    assert_eq!(w.public(w.agents[3].0), last_before);
    step("fault trapped after the resolution code: no credits, no discoverer XP, reviewer C's progress untouched");

    let ok: Result<(), ApiError> =
        w.env
            .update(w.platform, w.admin, "debug_set_review_fault", false);
    assert_eq!(ok, Ok(()));
    let receipt = w.review(w.agents[3], ids[2]);
    assert!(!receipt.duplicate);
    assert_eq!(receipt.xp_awarded, 5);
    step("retrying the same assignment is a fresh (non-duplicate) review: the review row and assignment consumption were rolled back too");
    w.assert_resolved_confirmed(&public_id);
    assert_eq!(w.public(w.agents[0].0).xp, xp_before + 50);
    step(&format!(
        "{public_id} Confirmed atomically: discoverer +50 XP, all 3 reviewers credited on the citation"
    ));
}

#[test]
fn t4_4_starved_discovery_resolves_by_weighted_majority_on_hourly_timer() {
    println!("T4.4 demo: a discovery needing 5 reviews gets 3, runs out of eligible reviewers, and the hourly timer resolves it by weighted majority after review_starvation_days");
    let w = World::new(None);
    let public_id = w.flag_and_promote_reviewers(5);
    let ids: Vec<u64> = w.agents[1..]
        .iter()
        .map(|&a| w.assignment(a).assignment_id)
        .collect();
    for (i, id) in ids.iter().enumerate() {
        w.review(w.agents[1 + i], *id);
    }
    let xp_before = w.public(w.agents[0].0).xp;
    assert!(w.credits(w.agents[0].0).items.is_empty());
    step(&format!(
        "{public_id} has 3 of 5 reviews and every other AAA is excluded: still UnderReview"
    ));

    advance(&w.env, Duration::from_secs(6 * 86_400));
    assert!(w.credits(w.agents[0].0).items.is_empty());
    step("after 6 days the hourly sweep leaves it open");

    advance(&w.env, Duration::from_secs(86_400 + 3_600));
    w.assert_resolved_confirmed(&public_id);
    assert_eq!(w.public(w.agents[0].0).xp, xp_before + 50);
    step(&format!(
        "past review_starvation_days the hourly timer resolved {public_id} Confirmed by weighted majority"
    ));
}

fn advance(env: &IcpEnv, by: Duration) {
    env.pic.advance_time(by);
    for _ in 0..5 {
        env.pic.tick();
    }
}
