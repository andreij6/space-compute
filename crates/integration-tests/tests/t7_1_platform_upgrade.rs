use candid::{decode_one, encode_args, encode_one, Principal};
use integration_tests::pic::{canister_wasm, user, IcpEnv};
use integration_tests::step;
use platform::catalog::SubjectInput;
use platform::citations::{self, CertifiedCitation};
use platform::config::Params;
use platform::discoveries::DiscoveryStatus;
use platform::progression::AaaPublic;
use platform::registry::RegisterArgs;
use platform::scoring::SubjectConsensus;
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
        let platform = env.install_baseline("platform", admin);
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

    fn leaderboard(&self) -> platform::progression::LeaderPage {
        let raw = self
            .env
            .pic
            .query_call(
                self.platform,
                user(200),
                "get_leaderboard",
                encode_args((None::<platform::progression::LeaderCursor>, 10u32)).unwrap(),
            )
            .expect("get_leaderboard");
        decode_one(&raw).unwrap()
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
fn t7_1_platform_v_n_minus_1_state_survives_upgrade_to_v_n() {
    println!("T7.1 demo: platform installed at the pinned vN-1 baseline, populated, upgraded to the current vN wasm — admins/params, subjects, protocol, registered AAAs, classifications, discovery/review/citation, events and progression all survive, and a certified citation still verifies");
    let w = World::new();
    step("installed platform at the pinned vN-1 baseline commit and registered 4 agents, a protocol and 31 subjects");

    let extra_admin = user(9);
    let ok: Result<(), ApiError> =
        w.env
            .update(w.platform, w.admin, "admin_add_admin", extra_admin);
    assert_eq!(ok, Ok(()));

    let public_id = w.flag_and_promote_reviewers(3);
    for &agent in &w.agents[1..] {
        let id = w.assignment(agent).assignment_id;
        w.review(agent, id);
    }
    step(&format!(
        "{public_id} classified, flagged, reviewed by 3 reviewers to Confirmed"
    ));

    let before = w.citation(&public_id);
    let c = &before.citation;
    assert_eq!(c.outcome, DiscoveryStatus::Confirmed);
    assert_eq!(c.subject.subject_id, 1);
    assert_eq!(c.discoverer.aaa_name_at_time, "Discoverer");
    assert_eq!(c.reviewers.len(), 3);
    assert!(!before.certificate.is_empty() && !before.witness.is_empty());
    assert_eq!(citations::verify(&before, w.platform), Ok(()));
    step("get_citation witness verifies against the subnet certificate before the upgrade");

    let admins_before: Vec<Principal> = w
        .env
        .query::<_, Result<Vec<Principal>, ApiError>>(w.platform, w.admin, "admin_list_admins", ())
        .expect("admin_list_admins");
    assert!(admins_before.contains(&extra_admin));
    let params_before: Params = w.env.query(w.platform, w.admin, "get_params", ());
    let protocols_before: Vec<Protocol> = w
        .env
        .query::<_, Result<Vec<Protocol>, ApiError>>(
            w.platform,
            w.admin,
            "admin_list_protocols",
            (),
        )
        .expect("admin_list_protocols");
    let subject1_before: Option<platform::catalog::Subject> =
        w.env.query(w.platform, w.admin, "get_subject", 1u32);
    let subject1_before = subject1_before.expect("subject 1 exists");
    let discovery_before = {
        let d: Option<platform::discoveries::DiscoveryView> =
            w.env
                .query(w.platform, user(200), "get_discovery", public_id.clone());
        d.expect("discovery view exists")
    };
    let consensus_before: Option<SubjectConsensus> =
        w.env
            .query(w.platform, w.admin, "get_subject_consensus", 701u32);
    let leaderboard_before: platform::progression::LeaderPage = w.leaderboard();
    let event0_before: Option<platform::events::Event> =
        w.env.query(w.platform, w.admin, "get_event", 0u64);
    step(&format!(
        "captured pre-upgrade snapshot: {} admins, {} protocols, subject/discovery/consensus/leaderboard/event views",
        admins_before.len(),
        protocols_before.len()
    ));

    w.env
        .pic
        .upgrade_canister(
            w.platform,
            canister_wasm("platform"),
            encode_one(()).unwrap(),
            Some(w.admin),
        )
        .expect("upgrade vN-1 -> vN must succeed");
    for _ in 0..5 {
        w.env.pic.tick();
    }
    step("upgraded platform from the vN-1 baseline to the current vN wasm");

    let after = w.citation(&public_id);
    assert_eq!(after.citation, before.citation);
    assert_eq!(after.witness, before.witness);
    assert_eq!(citations::verify(&after, w.platform), Ok(()));
    step("after upgrade: citation intact, witness still verifies against the new certificate (acceptance criterion)");

    let admins_after: Vec<Principal> = w
        .env
        .query::<_, Result<Vec<Principal>, ApiError>>(w.platform, w.admin, "admin_list_admins", ())
        .expect("admin_list_admins");
    assert_eq!(admins_after, admins_before);
    let params_after: Params = w.env.query(w.platform, w.admin, "get_params", ());
    assert_eq!(params_after, params_before);
    let protocols_after: Vec<Protocol> = w
        .env
        .query::<_, Result<Vec<Protocol>, ApiError>>(
            w.platform,
            w.admin,
            "admin_list_protocols",
            (),
        )
        .expect("admin_list_protocols");
    assert_eq!(protocols_after, protocols_before);
    let subject1_after: Option<platform::catalog::Subject> =
        w.env.query(w.platform, w.admin, "get_subject", 1u32);
    assert_eq!(subject1_after.unwrap(), subject1_before);
    let discovery_after: Option<platform::discoveries::DiscoveryView> =
        w.env
            .query(w.platform, user(200), "get_discovery", public_id.clone());
    assert_eq!(discovery_after.unwrap(), discovery_before);
    let consensus_after: Option<SubjectConsensus> =
        w.env
            .query(w.platform, w.admin, "get_subject_consensus", 701u32);
    assert_eq!(consensus_after, consensus_before);
    let leaderboard_after: platform::progression::LeaderPage = w.leaderboard();
    assert_eq!(leaderboard_after.items, leaderboard_before.items);
    let event0_after: Option<platform::events::Event> =
        w.env.query(w.platform, w.admin, "get_event", 0u64);
    assert_eq!(event0_after, event0_before);
    step("admins, params, protocols, subjects, discovery, consensus, leaderboard and the event log are byte-for-byte intact after the upgrade");

    let discoverer_public: AaaPublic = {
        let p: Option<AaaPublic> =
            w.env
                .query(w.platform, w.admin, "get_aaa_public", w.agents[0].0);
        p.expect("get_aaa_public only needs to decode cleanly once we're on the current wasm")
    };
    assert!(discoverer_public.counters.discoveries >= 1);
    let overview_after: platform::api::Overview = {
        let o: Result<platform::api::Overview, ApiError> =
            w.env.update(w.platform, w.admin, "admin_overview", ());
        o.expect("admin_overview")
    };
    assert!(overview_after.admins.contains(&extra_admin));
    assert_eq!(overview_after.total_aaas, 4);
    step("post-upgrade-only reads (AaaPublic, Overview — types that gained fields since the vN-1 baseline) decode fine against the current wasm");

    let rng_seeded_before = overview_after.rng_seeded_at;
    w.env
        .pic
        .advance_time(std::time::Duration::from_secs(3_600 + 60));
    for _ in 0..20 {
        w.env.pic.tick();
    }
    let overview_ticked: platform::api::Overview = {
        let o: Result<platform::api::Overview, ApiError> =
            w.env.update(w.platform, w.admin, "admin_overview", ());
        o.expect("admin_overview")
    };
    assert!(
        overview_ticked.rng_seeded_at > rng_seeded_before,
        "the hourly reseed timer must resume after post_upgrade re-registers it"
    );
    step(&format!(
        "hourly timers resumed after upgrade: rng_seeded_at {:?} -> {:?}",
        rng_seeded_before, overview_ticked.rng_seeded_at
    ));
}
