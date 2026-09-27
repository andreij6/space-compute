use candid::{decode_one, encode_args, encode_one, Principal};
use integration_tests::pic::{canister_wasm, user, IcpEnv};
use integration_tests::step;
use platform::catalog::SubjectInput;
use platform::citations::{self, CertifiedCitation};
use platform::config::Params;
use platform::discoveries::DiscoveryStatus;
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
fn t4_5_witness_verifies_after_upgrade() {
    println!("T4.5 demo: a Confirmed discovery gets a frozen citation whose witness verifies against the subnet certificate, before and after a platform upgrade");
    let w = World::new();
    let public_id = w.flag_and_promote_reviewers(3);
    for &agent in &w.agents[1..] {
        let id = w.assignment(agent).assignment_id;
        w.review(agent, id);
    }
    let missing: Option<CertifiedCitation> = w.env.query(
        w.platform,
        user(200),
        "get_citation",
        "SC-2099-000001".to_string(),
    );
    assert_eq!(missing, None);

    let before = w.citation(&public_id);
    let c = &before.citation;
    assert_eq!(c.outcome, DiscoveryStatus::Confirmed);
    assert_eq!(c.subject.subject_id, 1);
    assert_eq!(c.discoverer.aaa_name_at_time, "Discoverer");
    assert_eq!(c.reviewers.len(), 3);
    assert!(c.reviewers.iter().all(|r| r.vote == Vote::Agree));
    assert!(c.text.starts_with(&format!(
        "{public_id} — Gravitational Lens. Discovered by Discoverer; reviewed by "
    )));
    assert!(!before.certificate.is_empty() && !before.witness.is_empty());
    assert_eq!(citations::verify(&before, w.platform), Ok(()));
    step(&format!(
        "{public_id} Confirmed; get_citation witness ({} B) proves sha256(candid(citation)) under the certified_data in a {} B certificate",
        before.witness.len(),
        before.certificate.len()
    ));
    let mut forged = before.clone();
    forged.citation.discoverer.aaa_name_at_time = "Mallory".into();
    assert!(citations::verify(&forged, w.platform).is_err());
    step("a forged discoverer name fails verification");

    w.env
        .pic
        .upgrade_canister(
            w.platform,
            canister_wasm("platform"),
            encode_one(()).unwrap(),
            Some(w.admin),
        )
        .expect("upgrade");
    for _ in 0..3 {
        w.env.pic.tick();
    }
    let after = w.citation(&public_id);
    assert_eq!(after.citation, before.citation);
    assert_eq!(after.witness, before.witness);
    assert_eq!(citations::verify(&after, w.platform), Ok(()));
    step("after upgrade: citation intact, tree rebuilt from mem 42, witness still verifies against the new certificate");
}
