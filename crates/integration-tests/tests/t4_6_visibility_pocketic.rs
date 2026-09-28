use candid::{decode_one, encode_args, Principal};
use integration_tests::pic::{canister_wasm, user, IcpEnv};
use integration_tests::step;
use platform::catalog::SubjectInput;
use platform::config::Params;
use platform::discoveries::{DiscoveryCard, DiscoveryStatus, DiscoveryView, ListFilter};
use platform::events::{ActivityItem, ActivityKind, Event, Page};
use platform::progression::{LeaderPage, Stats};
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
        let subnet = env.pic.topology().get_app_subnets()[0];
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
        let r: Result<ReviewReceipt, ApiError> =
            self.env.update(self.platform, aaa, "submit_review", sub);
        r.expect("submit_review")
    }

    fn list_discoveries(&self, caller: Principal, filter: ListFilter) -> Page<DiscoveryCard> {
        let bytes = self
            .env
            .pic
            .query_call(
                self.platform,
                caller,
                "list_discoveries",
                encode_args((filter, None::<u64>, 100u32)).unwrap(),
            )
            .unwrap_or_else(|e| panic!("list_discoveries rejected: {e:?}"));
        decode_one(&bytes).unwrap_or_else(|e| panic!("list_discoveries reply did not decode: {e}"))
    }

    fn get_discovery(&self, caller: Principal, public_id: &str) -> Option<DiscoveryView> {
        self.env.query(
            self.platform,
            caller,
            "get_discovery",
            public_id.to_string(),
        )
    }

    fn stranger(&self) -> Principal {
        user(200)
    }

    fn activity(&self, caller: Principal, aaa: Principal) -> Vec<ActivityKind> {
        let bytes = self
            .env
            .pic
            .query_call(
                self.platform,
                caller,
                "list_aaa_activity",
                encode_args((aaa, None::<u64>, 100u32)).unwrap(),
            )
            .unwrap();
        decode_one::<Page<ActivityItem>>(&bytes)
            .unwrap()
            .items
            .into_iter()
            .map(|i| i.kind)
            .collect()
    }

    fn stranger_sees_no_raw_events(&self) {
        let total = (0..10_000u64)
            .take_while(|&id| {
                self.env
                    .query::<_, Option<Event>>(self.platform, self.admin, "get_event", id)
                    .is_some()
            })
            .count() as u64;
        assert!(total > 0);
        for id in 0..total {
            let e: Option<Event> = self
                .env
                .query(self.platform, self.stranger(), "get_event", id);
            assert_eq!(e, None);
        }
    }
}

fn has(kinds: &[ActivityKind], pred: impl Fn(&ActivityKind) -> bool) -> bool {
    kinds.iter().any(pred)
}

#[test]
fn t4_6_under_review_hidden_from_strangers_visible_to_owner_and_discoverer() {
    println!("T4.6 demo: an UnderReview discovery is invisible to a stranger, but visible to its discoverer AAA and its owner");
    let w = World::new();
    let public_id = w.flag_and_promote_reviewers(3);
    let (discoverer_aaa, discoverer_owner) = w.agents[0];
    let stranger = w.stranger();

    assert_eq!(w.get_discovery(stranger, &public_id), None);
    step(&format!(
        "{public_id} is UnderReview: a stranger's get_discovery returns None"
    ));

    let owner_view = w
        .get_discovery(discoverer_owner, &public_id)
        .expect("owner can see their own UnderReview discovery");
    assert_eq!(owner_view.status, DiscoveryStatus::UnderReview);
    assert!(owner_view.reviews.is_empty());
    let aaa_view = w
        .get_discovery(discoverer_aaa, &public_id)
        .expect("the discoverer AAA can see its own UnderReview discovery");
    assert_eq!(aaa_view.status, DiscoveryStatus::UnderReview);
    step("the owner and the discoverer AAA both see it UnderReview, with no reviews exposed pre-resolution");

    let stranger_list = w.list_discoveries(stranger, ListFilter::default());
    assert!(!stranger_list.items.iter().any(|c| c.public_id == public_id));
    let owner_list = w.list_discoveries(discoverer_owner, ListFilter::default());
    assert!(owner_list.items.iter().any(|c| c.public_id == public_id));
    step("list_discoveries excludes it for the stranger but includes it for the owner");

    let stats: Stats = w.env.query(w.platform, w.admin, "get_stats", ());
    assert_eq!(stats.under_review_count, 1);
    assert_eq!(stats.confirmed_discoveries, 0);

    let flagged = |k: &ActivityKind| matches!(k, ActivityKind::DiscoveryFlagged { .. });
    assert!(!has(&w.activity(stranger, discoverer_aaa), flagged));
    assert!(w.activity(discoverer_owner, discoverer_aaa).contains(
        &ActivityKind::DiscoveryFlagged {
            public_id: public_id.clone()
        }
    ));
    step("R-12: a stranger's list_aaa_activity hides the UnderReview DiscoveryFlagged; the owner still sees it");

    let ids: Vec<u64> = w.agents[1..]
        .iter()
        .map(|&a| w.assignment(a).assignment_id)
        .collect();
    let (reviewer_a, reviewer_a_owner) = w.agents[1];
    w.review(w.agents[1], ids[0]);
    let submitted = |k: &ActivityKind| matches!(k, ActivityKind::ReviewSubmitted { .. });
    assert!(!has(&w.activity(stranger, reviewer_a), submitted));
    assert!(has(&w.activity(reviewer_a_owner, reviewer_a), submitted));
    w.stranger_sees_no_raw_events();
    step("R-12: a live review stays hidden from strangers (owner sees it); get_event returns None to strangers for every event");

    for (&agent, &id) in w.agents[2..].iter().zip(&ids[1..]) {
        w.review(agent, id);
    }
    assert!(w
        .activity(stranger, discoverer_aaa)
        .contains(&ActivityKind::DiscoveryFlagged {
            public_id: public_id.clone()
        }));
    assert!(has(&w.activity(stranger, reviewer_a), submitted));
    step(&format!(
        "three reviewers agree: {public_id} resolves to Confirmed in this message"
    ));

    let public_view = w
        .get_discovery(stranger, &public_id)
        .expect("resolved discoveries are public");
    assert_eq!(public_view.status, DiscoveryStatus::Confirmed);
    assert_eq!(public_view.reviews.len(), 3);
    let public_list = w.list_discoveries(stranger, ListFilter::default());
    assert!(public_list.items.iter().any(|c| c.public_id == public_id));
    step("once Confirmed, the stranger sees the full discovery, including all 3 reviews, and it appears in the public feed");
}

#[test]
fn t4_6_honeypots_never_appear_in_discoveries_leaderboard_or_citations() {
    println!(
        "T4.6 demo: honeypots never surface in list_discoveries, get_leaderboard or get_citation"
    );
    let w = World::new();
    let spec = sc_types_honeypot_spec();
    let ok: Result<u32, ApiError> =
        w.env
            .update(w.platform, w.admin, "admin_add_honeypots", vec![spec]);
    assert_eq!(ok, Ok(1));
    step("one honeypot seeded on a gold subject");

    w.set_params(10_000, 3);
    for _ in 0..25 {
        w.classify(w.agents[1], false);
    }
    let mut params: Params = w.env.query(w.platform, w.admin, "get_params", ());
    params.honeypot_rate_bp = 10_000;
    let ok: Result<(), ApiError> = w
        .env
        .update(w.platform, w.admin, "admin_set_params", params);
    assert_eq!(ok, Ok(()));
    let assignment: ReviewAssignment = {
        let r: Result<Option<ReviewAssignment>, ApiError> = w.env.update(
            w.platform,
            w.agents[1].0,
            "get_review_assignment",
            Some(w.agents[1].1),
        );
        r.unwrap().expect("honeypot assignment")
    };
    let sub = ReviewSubmission {
        assignment_id: assignment.assignment_id,
        vote: Vote::Disagree,
        rationale: "no lensing features on this elliptical".into(),
        observed_image_sha256: vec![1; 32],
        agent_label: None,
        submitted_by: w.agents[1].1,
    };
    let receipt: Result<ReviewReceipt, ApiError> =
        w.env
            .update(w.platform, w.agents[1].0, "submit_review", sub);
    assert!(receipt.is_ok());
    step("the honeypot was assigned and scored immediately");

    let (aaa, owner) = w.agents[1];
    let review_kinds = |k: &ActivityKind| {
        matches!(
            k,
            ActivityKind::ReviewSubmitted { .. } | ActivityKind::ReviewScored { .. }
        )
    };
    assert!(!has(&w.activity(w.stranger(), aaa), review_kinds));
    let own = w.activity(owner, aaa);
    assert!(has(&own, |k| matches!(
        k,
        ActivityKind::ReviewSubmitted { .. }
    )));
    assert!(!has(&own, |k| matches!(
        k,
        ActivityKind::ReviewScored { .. }
    )));
    w.stranger_sees_no_raw_events();
    step("R-12: the honeypot review and its immediate score never reach a stranger; the owner sees the submission but no tell-tale score");

    let admin_list = w.list_discoveries(w.admin, ListFilter::default());
    assert!(
        admin_list.items.is_empty(),
        "no real discovery was flagged; the honeypot must not surface here"
    );
    step("list_discoveries (even for the admin) never returns the honeypot");

    let leaderboard: LeaderPage = {
        let bytes = w
            .env
            .pic
            .query_call(
                w.platform,
                w.admin,
                "get_leaderboard",
                encode_args((None::<platform::progression::LeaderCursor>, 100u32)).unwrap(),
            )
            .unwrap_or_else(|e| panic!("get_leaderboard rejected: {e:?}"));
        decode_one(&bytes).unwrap_or_else(|e| panic!("get_leaderboard reply did not decode: {e}"))
    };
    for row in &leaderboard.items {
        assert_ne!(row.name, "");
    }
    step("get_leaderboard lists only registered AAAs; the honeypot's anonymous discoverer never appears");
}

fn sc_types_honeypot_spec() -> platform::reviews::HoneypotSpec {
    platform::reviews::HoneypotSpec {
        subject_id: 701,
        category: "lens".into(),
        rationale: "a lens on a clean elliptical".into(),
        truth: Vote::Disagree,
    }
}
