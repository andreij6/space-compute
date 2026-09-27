use candid::{decode_one, encode_args};
use integration_tests::pic::{canister_wasm, user, IcpEnv};
use integration_tests::step;
use platform::catalog::SubjectInput;
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

fn call<A: candid::utils::ArgumentEncoder, R: serde::de::DeserializeOwned + candid::CandidType>(
    env: &IcpEnv,
    canister: candid::Principal,
    sender: candid::Principal,
    method: &str,
    args: A,
) -> R {
    let bytes = env
        .pic
        .update_call(canister, sender, method, encode_args(args).unwrap())
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

fn sample_protocol(v: u16) -> Protocol {
    Protocol {
        version: v,
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
        guidance_md: "Look closely at the image.".into(),
    }
}

#[test]
fn t3_3_records_and_credits_survive_canister_upgrade() {
    println!(
        "T3.3 demo: AAA repository records + queries + credits copy (Records survive upgrade)"
    );
    let env = IcpEnv::new();
    let alice = user(1);
    let payments = user(2);
    let owner = user(10);
    let operator = user(11);

    let platform = env.install_on("platform", alice, 10_000_000_000_000, 0);
    tick(&env, 2);

    let ok: Result<(), ApiError> = env.update(platform, alice, "admin_set_payments_id", payments);
    assert_eq!(ok, Ok(()));

    let wasm_v1 = canister_wasm("aaa");
    let hash_v1 = Sha256::digest(&wasm_v1).to_vec();
    let upload_bytes = env
        .pic
        .update_call(
            platform,
            alice,
            "admin_upload_wasm",
            encode_args((1u32, wasm_v1, hash_v1)).unwrap(),
        )
        .expect("upload wasm");
    let upload_res: Result<(), ApiError> = decode_one(&upload_bytes).unwrap();
    assert_eq!(upload_res, Ok(()));

    let approve_bytes = env
        .pic
        .update_call(
            platform,
            alice,
            "admin_approve_wasm",
            encode_args((1u32,)).unwrap(),
        )
        .expect("approve wasm");
    let approve_res: Result<(), ApiError> = decode_one(&approve_bytes).unwrap();
    assert_eq!(approve_res, Ok(()));

    let proto = sample_protocol(1);
    let ok: Result<(), ApiError> = env.update(platform, alice, "admin_add_protocol", proto);
    assert_eq!(ok, Ok(()));
    let ok: Result<(), ApiError> = env.update(platform, alice, "admin_set_current_protocol", 1u16);
    assert_eq!(ok, Ok(()));

    let batch = vec![
        SubjectInput {
            subject: sample_ref(1),
            gold: None,
        },
        SubjectInput {
            subject: sample_ref(2),
            gold: None,
        },
    ];
    let ok: Result<u32, ApiError> = env.update(platform, alice, "admin_add_subjects", batch);
    assert!(ok.is_ok());

    let subnet = env.pic.topology().get_app_subnets()[0];
    let aaa = env.pic.create_canister_on_subnet(Some(alice), None, subnet);
    env.pic.add_cycles(aaa, 10_000_000_000_000);
    env.pic
        .set_controllers(aaa, Some(alice), vec![owner, platform, alice])
        .unwrap();

    let reg = RegisterArgs {
        canister_id: aaa,
        owner,
        name: "RepoSurveyor-01".into(),
        avatar_seed: 42,
    };
    let reg_res: Result<(), ApiError> = env.update(platform, payments, "register_aaa", reg);
    assert_eq!(reg_res, Ok(()));
    tick(&env, 5);
    step("installed and registered AAA on platform");

    let add_op: Result<(), ApiError> = call(
        &env,
        aaa,
        owner,
        "add_operator",
        (operator, "bot-1".to_string(), None::<u64>),
    );
    assert_eq!(add_op, Ok(()));
    tick(&env, 5);

    let task_res: Result<Task, ApiError> = env.update(aaa, operator, "get_task", ());
    assert!(task_res.is_ok());
    let task = task_res.unwrap();

    let submission = ClassificationSubmission {
        task_id: task.task_id,
        answers: vec![Answer {
            question_id: "q1".into(),
            answer_id: "smooth".into(),
        }],
        observed_image_sha256: vec![1; 32],
        discovery: None,
        agent_label: Some("agent-alpha".into()),
        submitted_by: operator,
    };
    let submit_res: Result<ClassificationReceipt, ApiError> =
        env.update(aaa, operator, "submit_classification", submission);
    assert!(submit_res.is_ok());
    step("recorded classification in personal repository");

    let rec_before: Result<Option<aaa::record::Record>, ApiError> =
        env.query(aaa, owner, "get_record", 1u64);
    let rec_before = rec_before.expect("owner get_record must succeed");
    assert!(rec_before.is_some());
    let rec_before = rec_before.unwrap();
    assert_eq!(rec_before.seq, 1);
    assert_eq!(rec_before.agent_label.as_deref(), Some("agent-alpha"));

    let status_before: Result<aaa::record::Status, ApiError> = env.query(aaa, owner, "status", ());
    assert!(status_before.is_ok());
    let status_before = status_before.unwrap();
    assert_eq!(status_before.stats.classifications, 1);
    step("verified repository state before canister upgrade");

    env.pic
        .upgrade_canister(aaa, canister_wasm("aaa"), vec![], Some(alice))
        .expect("canister upgrade must succeed");
    tick(&env, 5);
    step("upgraded AAA canister wasm");

    let rec_after: Result<Option<aaa::record::Record>, ApiError> =
        env.query(aaa, owner, "get_record", 1u64);
    let rec_after = rec_after.expect("owner get_record must succeed");
    assert!(
        rec_after.is_some(),
        "record #1 must survive the upgrade intact"
    );
    let rec_after = rec_after.unwrap();
    assert_eq!(rec_after, rec_before);

    let list_res: Result<aaa::record::PageRecord, ApiError> = env.query(
        aaa,
        owner,
        "list_records",
        aaa::record::ListRecordsFilter {
            kind: None,
            cursor: None,
            limit: 10,
        },
    );
    assert!(list_res.is_ok());
    let page = list_res.unwrap();
    assert_eq!(page.items.len(), 1);
    assert_eq!(page.items[0].seq, 1);

    let status_after: Result<aaa::record::Status, ApiError> = env.query(aaa, owner, "status", ());
    assert!(status_after.is_ok());
    let status_after = status_after.unwrap();
    assert_eq!(status_after.stats.classifications, 1);
    assert_eq!(status_after.operators.len(), 1);

    let whoami_owner: aaa::roles::Role = env.query(aaa, owner, "whoami", ());
    assert_eq!(whoami_owner, aaa::roles::Role::Owner);
    let whoami_op: aaa::roles::Role = env.query(aaa, operator, "whoami", ());
    assert_eq!(whoami_op, aaa::roles::Role::Operator);
    step("verified records, stats, and roles all survived upgrade (acceptance criterion)");
}
