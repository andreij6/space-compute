use aaa::config::AaaInit;
use candid::{decode_one, encode_args, encode_one, Principal};
use integration_tests::pic::{canister_wasm, user, IcpEnv};
use integration_tests::step;
use platform::audit::AuditEntry;
use platform::catalog::{Subject, SubjectInput};
use platform::config::Params;
use platform::registry::{AaaRecord, AaaStatus, RegisterArgs};
use platform::scoring::Classification;
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
        ra_deg: 214.9,
        dec_deg: 52.8,
        image_url: format!("https://data.example.com/{id}/rgb.png"),
        image_sha256: vec![1; 32],
        dossier_url: format!("https://data.example.com/{id}/dossier.json"),
        dossier_sha256: vec![2; 32],
        data_version: 1,
    }
}

fn smooth() -> Vec<Answer> {
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
            prompt: "Is it smooth?".into(),
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

struct Setup {
    env: IcpEnv,
    platform: Principal,
    admin: Principal,
    payments: Principal,
    wasm: Vec<u8>,
}

fn setup(tune: impl FnOnce(&mut Params), subjects: Vec<SubjectInput>) -> Setup {
    let env = IcpEnv::new();
    let admin = user(1);
    let payments = user(2);
    let platform = env.install("platform", admin);
    let ok: Result<(), ApiError> = env.update(platform, admin, "admin_set_payments_id", payments);
    assert_eq!(ok, Ok(()));
    let wasm = canister_wasm("aaa");
    let hash = Sha256::digest(&wasm).to_vec();
    let bytes = env
        .pic
        .update_call(
            platform,
            admin,
            "admin_upload_wasm",
            encode_args((1u32, wasm.clone(), hash)).unwrap(),
        )
        .unwrap();
    assert_eq!(decode_one::<Result<(), ApiError>>(&bytes).unwrap(), Ok(()));
    let ok: Result<(), ApiError> = env.update(platform, admin, "admin_approve_wasm", 1u32);
    assert_eq!(ok, Ok(()));
    let mut params: Params = env.query(platform, admin, "get_params", ());
    params.fee_get_task = 0;
    params.fee_submit_classification = 0;
    tune(&mut params);
    let ok: Result<(), ApiError> = env.update(platform, admin, "admin_set_params", params);
    assert_eq!(ok, Ok(()));
    let ok: Result<(), ApiError> = env.update(platform, admin, "admin_add_protocol", protocol());
    assert_eq!(ok, Ok(()));
    let n = subjects.len() as u32;
    let added: Result<u32, ApiError> = env.update(platform, admin, "admin_add_subjects", subjects);
    assert_eq!(added, Ok(n));
    Setup {
        env,
        platform,
        admin,
        payments,
        wasm,
    }
}

fn subject(id: u32, gold: bool) -> SubjectInput {
    SubjectInput {
        subject: sample_ref(id),
        gold: gold.then(smooth),
    }
}

fn new_canister(s: &Setup, controllers: Vec<Principal>) -> Principal {
    let subnet = s.env.pic.topology().get_app_subnets()[0];
    let c = s
        .env
        .pic
        .create_canister_on_subnet(Some(s.admin), None, subnet);
    s.env.pic.add_cycles(c, 5_000_000_000_000);
    s.env
        .pic
        .set_controllers(c, Some(s.admin), controllers)
        .unwrap();
    c
}

fn get_task(s: &Setup, aaa: Principal, submitter: Option<Principal>) -> Result<Task, ApiError> {
    s.env.update(s.platform, aaa, "get_task", submitter)
}

fn submit(
    s: &Setup,
    aaa: Principal,
    task_id: u64,
    owner: Principal,
) -> Result<ClassificationReceipt, ApiError> {
    s.env.update(
        s.platform,
        aaa,
        "submit_classification",
        ClassificationSubmission {
            task_id,
            answers: smooth(),
            observed_image_sha256: vec![1; 32],
            discovery: None,
            agent_label: None,
            submitted_by: owner,
        },
    )
}

fn status(s: &Setup, aaa: Principal) -> AaaStatus {
    s.env
        .query::<_, Option<AaaRecord>>(s.platform, s.admin, "get_aaa", aaa)
        .unwrap()
        .status
}

#[test]
fn t2_9_gold_hidden_submitter_checked_calibration_ends_verify_internal() {
    println!("T2.9 demo A: gold/answers hidden, get_task submitter check, calibration ends, verify internal, admin suspension sticks");
    let s = setup(
        |p| {
            p.calibration_tasks = 1;
            p.calibration_gold_rate_bp = 10_000;
            p.gold_rate_bp = 0;
        },
        vec![
            subject(1001, true),
            subject(1002, true),
            subject(1003, false),
        ],
    );
    let owner = user(10);
    let stranger = user(66);
    let aaa = new_canister(&s, vec![owner, s.platform]);
    let reg = RegisterArgs {
        canister_id: aaa,
        owner,
        name: "LeadReview-A".into(),
        avatar_seed: 1,
    };
    let by_admin: Result<(), ApiError> =
        s.env
            .update(s.platform, s.admin, "register_aaa", reg.clone());
    assert_eq!(by_admin, Err(ApiError::Unauthorized));
    let ok: Result<(), ApiError> = s.env.update(s.platform, s.payments, "register_aaa", reg);
    assert_eq!(ok, Ok(()));
    tick(&s.env, 3);
    assert_eq!(status(&s, aaa), AaaStatus::Active);
    step("register_aaa: admin rejected (Unauthorized), payments registers the AAA (Active)");

    assert_eq!(
        get_task(&s, aaa, Some(stranger)),
        Err(ApiError::Unauthorized)
    );
    let t1 = get_task(&s, aaa, Some(owner)).expect("task for owner");
    step("get_task applies the submitter check: stranger Unauthorized, owner gets a task");

    let public: Option<Subject> =
        s.env
            .query(s.platform, stranger, "get_subject", t1.subject.subject_id);
    assert_eq!(public, None);
    let admin_view: Option<Subject> =
        s.env
            .query(s.platform, s.admin, "get_subject", t1.subject.subject_id);
    assert!(admin_view.expect("admin sees subject").gold.is_some());
    step("calibration task is gold; get_subject hides it (and its gold answers) from non-admins");

    let r1 = submit(&s, aaa, t1.task_id, owner).expect("receipt");
    let leaked: Vec<Classification> = s.env.query(
        s.platform,
        stranger,
        "list_subject_classifications",
        t1.subject.subject_id,
    );
    assert!(leaked.is_empty());
    let one: Option<Classification> = s.env.query(
        s.platform,
        stranger,
        "get_classification",
        r1.classification_id,
    );
    assert_eq!(one, None);
    let admin_list: Vec<Classification> = s.env.query(
        s.platform,
        s.admin,
        "list_subject_classifications",
        t1.subject.subject_id,
    );
    assert_eq!(admin_list.len(), 1);
    step("other agents' answers are admin-only: list_subject_classifications/get_classification empty for the public");

    let t2 = get_task(&s, aaa, Some(owner)).expect("second task");
    assert_eq!(t2.subject.subject_id, 1003);
    step("after calibration_tasks=1 classification the gold rate drops to gold_rate_bp=0: next task is non-gold 1003 (gold 1002 still available)");

    let verify = s
        .env
        .pic
        .update_call(s.platform, stranger, "verify", encode_one(aaa).unwrap());
    assert!(verify.is_err());
    step("verify is not a public method (no cycle burn, no status flip by strangers)");

    let bytes = s
        .env
        .pic
        .update_call(
            s.platform,
            s.admin,
            "admin_suspend_aaa",
            encode_args((aaa, "moderation".to_string())).unwrap(),
        )
        .unwrap();
    assert_eq!(decode_one::<Result<(), ApiError>>(&bytes).unwrap(), Ok(()));
    assert_eq!(submit(&s, aaa, t2.task_id, owner), Err(ApiError::Suspended));
    assert_eq!(get_task(&s, aaa, Some(owner)), Err(ApiError::Suspended));
    assert_eq!(status(&s, aaa), AaaStatus::Suspended);
    step("admin suspension sticks: the AAA stays Suspended");
}

#[test]
fn t2_9_retry_skips_existing_module_and_unrecorded_change_suspends() {
    println!(
        "T2.9 demo B: install idempotency after lost reply, admin retry, strict total_num_changes"
    );
    let s = setup(|_| {}, vec![subject(2001, false), subject(2002, false)]);
    let owner = user(20);
    let aaa = new_canister(&s, vec![s.admin, owner]);
    let reg = RegisterArgs {
        canister_id: aaa,
        owner,
        name: "LeadReview-B".into(),
        avatar_seed: 2,
    };
    let first: Result<(), ApiError> = s.env.update(s.platform, s.payments, "register_aaa", reg);
    assert!(matches!(first, Err(ApiError::Internal(_))));
    assert_eq!(status(&s, aaa), AaaStatus::Installing);
    step("first register_aaa fails (platform not a controller yet): record stays Installing");

    let init = AaaInit {
        owner,
        platform_id: s.platform,
        payments_id: s.payments,
        name: "LeadReview-B".into(),
        avatar_seed: 2,
    };
    s.env.pic.install_canister(
        aaa,
        s.wasm.clone(),
        encode_one(init).unwrap(),
        Some(s.admin),
    );
    s.env
        .pic
        .set_controllers(aaa, Some(s.admin), vec![owner, s.platform])
        .unwrap();
    step("simulate SYS_UNKNOWN: the approved module is already installed and controllers are {owner, platform}");

    let retry: Result<(), ApiError> = s
        .env
        .update(s.platform, s.admin, "admin_retry_install", aaa);
    assert_eq!(retry, Ok(()));
    assert_eq!(status(&s, aaa), AaaStatus::Active);
    step("admin_retry_install sees the approved module via canister_info, skips install_code, and activates the AAA");

    let task = get_task(&s, aaa, None).expect("task");
    s.env
        .pic
        .set_controllers(aaa, Some(owner), vec![owner, s.platform])
        .unwrap();
    assert_eq!(
        submit(&s, aaa, task.task_id, owner),
        Err(ApiError::Suspended)
    );
    assert_eq!(status(&s, aaa), AaaStatus::Suspended);
    let bytes = s
        .env
        .pic
        .query_call(
            s.platform,
            s.admin,
            "admin_audit_log",
            encode_args((None::<u64>, 100u32)).unwrap(),
        )
        .unwrap();
    let log: Vec<AuditEntry> = decode_one::<Result<Vec<AuditEntry>, ApiError>>(&bytes)
        .unwrap()
        .unwrap();
    assert!(log
        .iter()
        .any(|e| e.method == "verify" && e.summary.contains("total_num_changes")));
    step("an unrecorded canister change (total_num_changes != recorded) suspends the AAA and writes an audit entry");

    let ok: Result<(), ApiError> = s
        .env
        .update(s.platform, s.admin, "admin_unsuspend_aaa", aaa);
    assert_eq!(ok, Ok(()));
    let receipt = submit(&s, aaa, task.task_id, owner).expect("receipt after unsuspend");
    assert!(!receipt.duplicate);
    let dup = submit(&s, aaa, task.task_id, owner).expect("duplicate");
    assert!(dup.duplicate);
    assert_eq!(dup.xp_awarded, receipt.xp_awarded);
    step("admin unsuspend re-baselines provenance; submission accepted; duplicate returns the original receipt");
}
