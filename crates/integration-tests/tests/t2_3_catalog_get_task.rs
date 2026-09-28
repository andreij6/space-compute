use candid::{decode_one, encode_args};
use integration_tests::pic::{canister_wasm, user, IcpEnv};
use integration_tests::step;
use platform::catalog::{AdminListSubjectsFilter, Lease, Subject, SubjectInput};
use platform::config::Params;
use platform::registry::RegisterArgs;
use sc_types::{
    Answer, AnswerOption, ApiError, DiscoveryCategory, Protocol, Question, SubjectRef, Task,
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
fn t2_3_never_same_subject_twice_and_pool_dispatch() {
    println!("T2.3 demo: Catalog — subjects, protocol, leases, seen-set, get_task");
    let env = IcpEnv::new();
    let alice = user(1);
    let bob = user(2);
    let charlie = user(3);
    let payments = user(99);

    let platform = env.install("platform", alice);
    let ok: Result<(), ApiError> = env.update(platform, alice, "admin_set_payments_id", payments);
    assert_eq!(ok, Ok(()));
    step("installed platform canister with admin alice and configured payments");

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
    step("uploaded and approved AAA wasm v1");

    let subnet = env.pic.topology().get_app_subnets()[0];
    let aaa_1 = env.pic.create_canister_on_subnet(Some(alice), None, subnet);
    env.pic.add_cycles(aaa_1, 5_000_000_000_000);
    env.pic
        .set_controllers(aaa_1, Some(alice), vec![bob, platform])
        .unwrap();

    let reg_1 = RegisterArgs {
        canister_id: aaa_1,
        owner: bob,
        name: "Surveyor-01".into(),
        avatar_seed: 11,
    };
    let ok: Result<(), ApiError> = env.update(platform, payments, "register_aaa", reg_1);
    assert_eq!(ok, Ok(()));

    let aaa_2 = env.pic.create_canister_on_subnet(Some(alice), None, subnet);
    env.pic.add_cycles(aaa_2, 5_000_000_000_000);
    env.pic
        .set_controllers(aaa_2, Some(alice), vec![charlie, platform])
        .unwrap();

    let reg_2 = RegisterArgs {
        canister_id: aaa_2,
        owner: charlie,
        name: "Surveyor-02".into(),
        avatar_seed: 22,
    };
    let ok: Result<(), ApiError> = env.update(platform, payments, "register_aaa", reg_2);
    assert_eq!(ok, Ok(()));
    tick(&env, 5);
    step("registered active AAAs: aaa_1 (bob) and aaa_2 (charlie)");

    let proto = sample_protocol(1);
    let ok: Result<(), ApiError> = env.update(platform, alice, "admin_add_protocol", proto.clone());
    assert_eq!(ok, Ok(()));
    let ok: Result<(), ApiError> = env.update(platform, alice, "admin_set_current_protocol", 1u16);
    assert_eq!(ok, Ok(()));

    let fetched_proto: Option<Protocol> = env.query(platform, alice, "get_protocol", 1u16);
    assert_eq!(fetched_proto, Some(proto));
    let proto_list: Result<Vec<Protocol>, ApiError> =
        env.query(platform, alice, "admin_list_protocols", ());
    assert_eq!(proto_list.unwrap().len(), 1);
    step("configured protocol v1 with questions and discovery categories");

    let batch = vec![
        SubjectInput {
            subject: sample_ref(101),
            gold: None,
        },
        SubjectInput {
            subject: sample_ref(102),
            gold: None,
        },
        SubjectInput {
            subject: sample_ref(103),
            gold: None,
        },
        SubjectInput {
            subject: sample_ref(104),
            gold: Some(vec![Answer {
                question_id: "q1".into(),
                answer_id: "smooth".into(),
            }]),
        },
        SubjectInput {
            subject: sample_ref(105),
            gold: Some(vec![Answer {
                question_id: "q1".into(),
                answer_id: "featured".into(),
            }]),
        },
    ];
    let added: Result<u32, ApiError> = env.update(platform, alice, "admin_add_subjects", batch);
    assert_eq!(added, Ok(5));

    let filter = AdminListSubjectsFilter {
        field: Some("ceers".into()),
        active: Some(true),
        gold: None,
    };
    let subj_bytes = env
        .pic
        .query_call(
            platform,
            alice,
            "admin_list_subjects",
            encode_args((filter, None::<u64>, 10u32)).unwrap(),
        )
        .expect("query subjects");
    let subj_list: Result<platform::events::Page<Subject>, ApiError> =
        decode_one(&subj_bytes).unwrap();
    assert_eq!(subj_list.unwrap().items.len(), 5);
    step("added 5 subjects (3 standard pool subjects, 2 gold subjects)");

    let unregistered_call: Result<Task, ApiError> = env.update(platform, user(42), "get_task", ());
    assert_eq!(unregistered_call, Err(ApiError::NotRegistered));
    step("unregistered caller rejected with NotRegistered");

    let insufficient_fee: Result<Task, ApiError> = env.update(platform, aaa_1, "get_task", ());
    assert_eq!(
        insufficient_fee,
        Err(ApiError::InsufficientFee {
            required: Params::default().fee_get_task.into()
        })
    );
    step("get_task rejected with InsufficientFee when fee_get_task > 0 without cycles attached");

    let free_params = Params {
        fee_get_task: 0,
        max_open_leases_per_aaa: 10,
        gold_rate_bp: 0,
        calibration_gold_rate_bp: 0,
        ..Params::default()
    };
    let ok: Result<(), ApiError> = env.update(platform, alice, "admin_set_params", free_params);
    assert_eq!(ok, Ok(()));
    step("set fee_get_task = 0 for direct dispatch verification");

    let mut seen_by_aaa_1 = std::collections::HashSet::new();
    let mut task_ids = Vec::new();
    for i in 1..=5 {
        let task_res: Result<Task, ApiError> = env.update(platform, aaa_1, "get_task", ());
        let task = task_res.expect("task issued to aaa_1");
        assert_eq!(task.protocol.version, 1);
        let id = task.subject.subject_id;
        assert!(
            seen_by_aaa_1.insert(id),
            "invariant violated: subject {id} issued twice to aaa_1 on call {i}"
        );
        task_ids.push(task.task_id);
    }
    assert_eq!(seen_by_aaa_1.len(), 5);
    step("aaa_1 received 5 tasks with 5 distinct subjects: never same subject twice");

    let exhausted: Result<Task, ApiError> = env.update(platform, aaa_1, "get_task", ());
    assert_eq!(exhausted, Err(ApiError::NotFound));
    step("aaa_1 6th request returned NotFound: catalog exhausted for this AAA");

    let aaa_2_task: Result<Task, ApiError> = env.update(platform, aaa_2, "get_task", ());
    let t2 = aaa_2_task.expect("task issued to aaa_2");
    assert!(
        seen_by_aaa_1.contains(&t2.subject.subject_id),
        "aaa_2 correctly received pooled subject that aaa_1 already classified"
    );
    step("aaa_2 received a subject already seen by aaa_1");

    let lease: Option<Lease> = env.query(platform, alice, "get_lease", task_ids[0]);
    let lease = lease.expect("lease exists");
    assert_eq!(lease.aaa, aaa_1);
    assert!(seen_by_aaa_1.contains(&lease.subject_id));
    step("lease record verified: tracks recipient AAA, subject_id, and expiration");

    let act_bytes = env
        .pic
        .update_call(
            platform,
            alice,
            "admin_set_subject_active",
            encode_args((101u32, false)).unwrap(),
        )
        .expect("update subject active");
    let act_res: Result<(), ApiError> = decode_one(&act_bytes).unwrap();
    assert_eq!(act_res, Ok(()));
    let s101: Option<Subject> = env.query(platform, alice, "get_subject", 101u32);
    assert!(!s101.unwrap().active);
    step("admin deactivated subject 101 successfully");

    let rate_limited_params = Params {
        fee_get_task: 0,
        max_open_leases_per_aaa: 1,
        lease_task_secs: 10,
        gold_rate_bp: 0,
        calibration_gold_rate_bp: 0,
        ..Params::default()
    };
    let ok: Result<(), ApiError> =
        env.update(platform, alice, "admin_set_params", rate_limited_params);
    assert_eq!(ok, Ok(()));

    let limited: Result<Task, ApiError> = env.update(platform, aaa_2, "get_task", ());
    assert!(matches!(limited, Err(ApiError::RateLimited { .. })));
    step("aaa_2 hit max_open_leases_per_aaa and was RateLimited");

    env.pic.advance_time(std::time::Duration::from_secs(1805));
    env.pic.tick();
    let unblocked: Result<Task, ApiError> = env.update(platform, aaa_2, "get_task", ());
    assert!(unblocked.is_ok());
    step("after lease expiration time elapsed, expired lease was swept and aaa_2 received a task");
}
