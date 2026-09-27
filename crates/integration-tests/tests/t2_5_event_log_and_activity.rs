use candid::{decode_one, encode_args};
use integration_tests::pic::{canister_wasm, user, IcpEnv};
use integration_tests::step;
use platform::catalog::SubjectInput;
use platform::config::Params;
use platform::events::{ActivityItem, Event, EventKind, Page};
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
fn t2_5_events_append_and_paged_activity() {
    println!("T2.5 demo: Event log, per-AAA index, and paged activity query");
    let env = IcpEnv::new();
    let alice = user(1);
    let payments = user(2);
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
    let ok: Result<(), ApiError> = env.update(platform, alice, "admin_approve_wasm", 1u32);
    assert_eq!(ok, Ok(()));

    let zero_fee_params = Params {
        fee_get_task: 0,
        fee_submit_classification: 0,
        max_open_leases_per_aaa: 10,
        retire_after_k: 5,
        ..Default::default()
    };
    let ok: Result<(), ApiError> = env.update(platform, alice, "admin_set_params", zero_fee_params);
    assert_eq!(ok, Ok(()));

    let proto = sample_protocol(1);
    let ok: Result<(), ApiError> = env.update(platform, alice, "admin_add_protocol", proto);
    assert_eq!(ok, Ok(()));
    let ok: Result<(), ApiError> = env.update(platform, alice, "admin_set_current_protocol", 1u16);
    assert_eq!(ok, Ok(()));
    step("installed platform and approved wasm v1");

    let subnet = env.pic.topology().get_app_subnets()[0];

    let owner1 = user(101);
    let aaa_1 = env.pic.create_canister_on_subnet(Some(alice), None, subnet);
    env.pic.add_cycles(aaa_1, 5_000_000_000_000);
    env.pic
        .set_controllers(aaa_1, Some(alice), vec![owner1, platform])
        .unwrap();
    let reg1 = RegisterArgs {
        canister_id: aaa_1,
        owner: owner1,
        name: "Surveyor-01".into(),
        avatar_seed: 1,
    };
    let ok: Result<(), ApiError> = env.update(platform, payments, "register_aaa", reg1);
    assert_eq!(ok, Ok(()));

    let owner2 = user(102);
    let aaa_2 = env.pic.create_canister_on_subnet(Some(alice), None, subnet);
    env.pic.add_cycles(aaa_2, 5_000_000_000_000);
    env.pic
        .set_controllers(aaa_2, Some(alice), vec![owner2, platform])
        .unwrap();
    let reg2 = RegisterArgs {
        canister_id: aaa_2,
        owner: owner2,
        name: "Surveyor-02".into(),
        avatar_seed: 2,
    };
    let ok: Result<(), ApiError> = env.update(platform, payments, "register_aaa", reg2);
    assert_eq!(ok, Ok(()));
    tick(&env, 5);

    step("spawned 2 AAAs; verified AaaSpawned events appended");
    let act1: Page<ActivityItem> = decode_one(
        &env.pic
            .query_call(
                platform,
                owner1,
                "list_aaa_activity",
                encode_args((aaa_1, Option::<u64>::None, 10u32)).unwrap(),
            )
            .unwrap(),
    )
    .unwrap();
    assert_eq!(act1.items.len(), 1);
    assert_eq!(act1.items[0].aaa, aaa_1);
    assert_eq!(act1.items[0].owner, owner1);
    assert!(matches!(act1.items[0].kind, EventKind::AaaSpawned { .. }));

    let spawned_event_id = act1.items[0].id;
    let single_event: Option<Event> = decode_one(
        &env.pic
            .query_call(
                platform,
                alice,
                "get_event",
                encode_args((spawned_event_id,)).unwrap(),
            )
            .unwrap(),
    )
    .unwrap();
    assert!(single_event.is_some());
    assert_eq!(single_event.unwrap().id, spawned_event_id);

    let batch = vec![
        SubjectInput {
            subject: sample_ref(601),
            gold: None,
        },
        SubjectInput {
            subject: sample_ref(602),
            gold: None,
        },
    ];
    let added: Result<u32, ApiError> = env.update(platform, alice, "admin_add_subjects", batch);
    assert_eq!(added, Ok(2));

    let task1_res: Result<Task, ApiError> = decode_one(
        &env.pic
            .update_call(platform, aaa_1, "get_task", encode_args(()).unwrap())
            .unwrap(),
    )
    .unwrap();
    let task1 = task1_res.expect("task 1 for aaa 1");

    let sub1 = ClassificationSubmission {
        task_id: task1.task_id,
        answers: vec![Answer {
            question_id: "q1".into(),
            answer_id: "smooth".into(),
        }],
        observed_image_sha256: vec![1; 32],
        discovery: None,
        agent_label: Some("agent-v1".into()),
        submitted_by: owner1,
    };
    let rec1: Result<ClassificationReceipt, ApiError> = decode_one(
        &env.pic
            .update_call(
                platform,
                aaa_1,
                "submit_classification",
                encode_args((sub1,)).unwrap(),
            )
            .unwrap(),
    )
    .unwrap();
    assert!(rec1.is_ok());

    let task2_res: Result<Task, ApiError> = decode_one(
        &env.pic
            .update_call(platform, aaa_1, "get_task", encode_args(()).unwrap())
            .unwrap(),
    )
    .unwrap();
    let task2 = task2_res.expect("task 2 for aaa 1");

    let sub2 = ClassificationSubmission {
        task_id: task2.task_id,
        answers: vec![Answer {
            question_id: "q1".into(),
            answer_id: "featured".into(),
        }],
        observed_image_sha256: vec![1; 32],
        discovery: None,
        agent_label: Some("agent-v1".into()),
        submitted_by: owner1,
    };
    let rec2: Result<ClassificationReceipt, ApiError> = decode_one(
        &env.pic
            .update_call(
                platform,
                aaa_1,
                "submit_classification",
                encode_args((sub2,)).unwrap(),
            )
            .unwrap(),
    )
    .unwrap();
    assert!(rec2.is_ok());
    step("submitted 2 classifications for aaa_1");

    let act_all: Page<ActivityItem> = decode_one(
        &env.pic
            .query_call(
                platform,
                owner1,
                "list_aaa_activity",
                encode_args((aaa_1, Option::<u64>::None, 10u32)).unwrap(),
            )
            .unwrap(),
    )
    .unwrap();
    assert_eq!(act_all.items.len(), 3);
    assert!(act_all.items[0].id > act_all.items[1].id);
    assert!(act_all.items[1].id > act_all.items[2].id);
    assert!(matches!(
        act_all.items[0].kind,
        EventKind::Classified { .. }
    ));
    assert!(matches!(
        act_all.items[1].kind,
        EventKind::Classified { .. }
    ));
    assert!(matches!(
        act_all.items[2].kind,
        EventKind::AaaSpawned { .. }
    ));
    for item in &act_all.items {
        assert_eq!(item.aaa, aaa_1);
    }
    step("all 3 events returned in newest-first order with strict per-AAA isolation");

    let p1: Page<ActivityItem> = decode_one(
        &env.pic
            .query_call(
                platform,
                owner1,
                "list_aaa_activity",
                encode_args((aaa_1, Option::<u64>::None, 1u32)).unwrap(),
            )
            .unwrap(),
    )
    .unwrap();
    assert_eq!(p1.items.len(), 1);
    assert_eq!(p1.items[0].id, act_all.items[0].id);
    assert!(p1.next_cursor.is_some());

    let p2: Page<ActivityItem> = decode_one(
        &env.pic
            .query_call(
                platform,
                owner1,
                "list_aaa_activity",
                encode_args((aaa_1, p1.next_cursor, 1u32)).unwrap(),
            )
            .unwrap(),
    )
    .unwrap();
    assert_eq!(p2.items.len(), 1);
    assert_eq!(p2.items[0].id, act_all.items[1].id);
    assert!(p2.next_cursor.is_some());

    let p3: Page<ActivityItem> = decode_one(
        &env.pic
            .query_call(
                platform,
                owner1,
                "list_aaa_activity",
                encode_args((aaa_1, p2.next_cursor, 1u32)).unwrap(),
            )
            .unwrap(),
    )
    .unwrap();
    assert_eq!(p3.items.len(), 1);
    assert_eq!(p3.items[0].id, act_all.items[2].id);
    assert!(p3.next_cursor.is_none());
    step("paged activity with cursor visits all records exactly once in order");

    let suspend_bytes = env
        .pic
        .update_call(
            platform,
            alice,
            "admin_suspend_aaa",
            encode_args((aaa_1, "testing suspension event".to_string())).unwrap(),
        )
        .expect("admin_suspend_aaa");
    let ok: Result<(), ApiError> = decode_one(&suspend_bytes).unwrap();
    assert_eq!(ok, Ok(()));

    let act_after_suspend: Page<ActivityItem> = decode_one(
        &env.pic
            .query_call(
                platform,
                owner1,
                "list_aaa_activity",
                encode_args((aaa_1, Option::<u64>::None, 1u32)).unwrap(),
            )
            .unwrap(),
    )
    .unwrap();
    assert_eq!(act_after_suspend.items.len(), 1);
    assert!(matches!(
        act_after_suspend.items[0].kind,
        EventKind::AaaSuspended { .. }
    ));
    step("AaaSuspended event appended and retrieved as newest activity");
}
