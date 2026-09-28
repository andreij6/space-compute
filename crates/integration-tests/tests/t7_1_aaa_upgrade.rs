use candid::{decode_one, encode_args, Principal};
use integration_tests::pic::{canister_wasm, user, IcpEnv};
use integration_tests::step;
use platform::catalog::SubjectInput;
use platform::registry::{AaaRecord, RegisterArgs};
use sc_types::{
    Answer, AnswerOption, ApiError, ClassificationReceipt, ClassificationSubmission,
    DiscoveryCategory, DiscoveryFlag, Protocol, Question, SubjectRef, Task,
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
fn t7_1_aaa_v_n_minus_1_state_survives_upgrade_to_v_n() {
    println!("T7.1 demo: aaa installed at the pinned vN-1 baseline, populated, upgraded to the current vN wasm — operators, records and auto-topup config survive, and the burn/heartbeat timers (added after the baseline) start on post_upgrade");
    let env = IcpEnv::new();
    let alice = user(1);
    let fake_payments = user(2);
    let owner = user(10);
    let operator = user(11);

    let platform = env.install_on("platform", alice, 10_000_000_000_000, 0);
    tick(&env, 2);
    let ok: Result<(), ApiError> =
        env.update(platform, alice, "admin_set_payments_id", fake_payments);
    assert_eq!(ok, Ok(()));

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
    step("installed a current-wasm platform with a protocol and 2 subjects");

    let wasm_v1 = integration_tests::pic::baseline_wasm("aaa");
    let hash_v1 = Sha256::digest(&wasm_v1).to_vec();
    let upload: Result<(), ApiError> = call(
        &env,
        platform,
        alice,
        "admin_upload_wasm",
        (1u32, wasm_v1, hash_v1),
    );
    assert_eq!(upload, Ok(()));
    let approve: Result<(), ApiError> = env.update(platform, alice, "admin_approve_wasm", 1u32);
    assert_eq!(approve, Ok(()));
    step("approved the pinned vN-1 aaa wasm as platform's spawn template");

    let subnet = env.pic.topology().get_app_subnets()[0];
    let aaa = env.pic.create_canister_on_subnet(Some(alice), None, subnet);
    env.pic.add_cycles(aaa, 10_000_000_000_000);
    env.pic
        .set_controllers(aaa, Some(alice), vec![owner, platform, alice])
        .unwrap();

    let reg = RegisterArgs {
        canister_id: aaa,
        owner,
        name: "BaselineRover-01".into(),
        avatar_seed: 42,
    };
    let reg_res: Result<(), ApiError> = env.update(platform, fake_payments, "register_aaa", reg);
    assert_eq!(reg_res, Ok(()));
    tick(&env, 5);
    step("platform installed the vN-1 baseline wasm onto the newly registered aaa during registration");

    let add_op: Result<(), ApiError> = call(
        &env,
        aaa,
        owner,
        "add_operator",
        (operator, "bot-1".to_string(), None::<u64>),
    );
    assert_eq!(add_op, Ok(()));
    let add_op2: Result<(), ApiError> = call(
        &env,
        aaa,
        owner,
        "add_operator",
        (user(12), "bot-2".to_string(), Some(u64::MAX / 2)),
    );
    assert_eq!(add_op2, Ok(()));
    step("registered on platform and added 2 operators");

    let huge_threshold = 500_000_000_000_000u128;
    let auto: Result<(), ApiError> =
        call(&env, aaa, owner, "set_auto_topup", (Some(huge_threshold),));
    assert_eq!(auto, Ok(()));

    let task_res: Result<Task, ApiError> = env.update(aaa, operator, "get_task", ());
    let task = task_res.expect("get_task");
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

    let task2_res: Result<Task, ApiError> = env.update(aaa, operator, "get_task", ());
    let task2 = task2_res.expect("get_task 2");
    let submission2 = ClassificationSubmission {
        task_id: task2.task_id,
        answers: vec![Answer {
            question_id: "q1".into(),
            answer_id: "smooth".into(),
        }],
        observed_image_sha256: vec![1; 32],
        discovery: Some(DiscoveryFlag {
            category: "lens".into(),
            rationale: "possible arc near the galaxy core".into(),
            confidence: 80,
            claim_position: None,
        }),
        agent_label: Some("agent-alpha".into()),
        submitted_by: operator,
    };
    let submit_res2: Result<ClassificationReceipt, ApiError> =
        env.update(aaa, operator, "submit_classification", submission2);
    assert!(submit_res2.is_ok());
    step("recorded 2 classifications (one flagged as a discovery) in the personal repository");

    tick(&env, 5);
    env.pic
        .advance_time(std::time::Duration::from_secs(6 * 3_600 + 60));
    tick(&env, 30);
    let status_before: aaa::record::Status = env
        .query::<_, Result<aaa::record::Status, ApiError>>(aaa, owner, "status", ())
        .expect("status");
    assert_eq!(
        status_before.stats.auto_topup_failures, 0,
        "the vN-1 baseline predates the T3.4 burn timer, so nothing may have fired yet"
    );
    step("advanced 6h on the vN-1 baseline: no burn timer exists yet, auto_topup_failures=0");

    let rec_before: Option<aaa::record::Record> = env.query(aaa, owner, "get_record", 1u64);
    let rec_before = rec_before.expect("the vN-1 get_record returns opt Record");
    assert_eq!(rec_before.seq, 1);
    assert_eq!(rec_before.agent_label.as_deref(), Some("agent-alpha"));

    let list_before: aaa::record::PageRecord = env
        .query::<_, Result<aaa::record::PageRecord, ApiError>>(
            aaa,
            owner,
            "list_records",
            aaa::record::ListRecordsFilter {
                kind: None,
                cursor: None,
                limit: 10,
            },
        )
        .expect("list_records");
    assert_eq!(list_before.items.len(), 2);

    let operators_before: Vec<(Principal, aaa::operators::Operator)> = env
        .query::<_, Result<Vec<(Principal, aaa::operators::Operator)>, ApiError>>(
            aaa,
            owner,
            "list_operators",
            (),
        )
        .expect("list_operators");
    assert_eq!(operators_before.len(), 2);
    let last_seen_before = {
        let rec: Result<AaaRecord, ApiError> = env.query(platform, alice, "admin_get_aaa", aaa);
        rec.expect("platform sees the aaa").last_seen_at
    };
    step("captured pre-upgrade snapshot: 2 records, 2 operators, 0 auto-topup failures");

    env.pic
        .upgrade_canister(aaa, canister_wasm("aaa"), vec![], Some(alice))
        .expect("upgrade vN-1 -> vN must succeed");
    tick(&env, 5);
    step("upgraded aaa from the vN-1 baseline to the current vN wasm");

    let rec_after: Result<Option<aaa::record::Record>, ApiError> =
        env.query(aaa, owner, "get_record", 1u64);
    let rec_after = rec_after.expect("owner get_record must succeed").unwrap();
    assert_eq!(rec_after, rec_before);

    let list_after: aaa::record::PageRecord = env
        .query::<_, Result<aaa::record::PageRecord, ApiError>>(
            aaa,
            owner,
            "list_records",
            aaa::record::ListRecordsFilter {
                kind: None,
                cursor: None,
                limit: 10,
            },
        )
        .expect("list_records");
    assert_eq!(list_after.items, list_before.items);

    let operators_after: Vec<(Principal, aaa::operators::Operator)> = env
        .query::<_, Result<Vec<(Principal, aaa::operators::Operator)>, ApiError>>(
            aaa,
            owner,
            "list_operators",
            (),
        )
        .expect("list_operators");
    assert_eq!(operators_after, operators_before);

    let whoami_owner: aaa::roles::Role = env.query(aaa, owner, "whoami", ());
    assert_eq!(whoami_owner, aaa::roles::Role::Owner);
    let whoami_op: aaa::roles::Role = env.query(aaa, operator, "whoami", ());
    assert_eq!(whoami_op, aaa::roles::Role::Operator);
    step("records, operators and roles are byte-for-byte intact after the upgrade");

    env.pic
        .advance_time(std::time::Duration::from_secs(6 * 3_600 + 60));
    tick(&env, 30);
    let status_after: aaa::record::Status = env
        .query::<_, Result<aaa::record::Status, ApiError>>(aaa, owner, "status", ())
        .expect("status");
    assert!(
        status_after.stats.auto_topup_failures > status_before.stats.auto_topup_failures,
        "the 6h burn timer must start after post_upgrade registers it: the auto_topup threshold ({huge_threshold}) set on the baseline must still be in effect"
    );
    step(&format!(
        "burn_tick timer started after upgrade: auto_topup_failures {} -> {} (the auto_topup config value survived the upgrade too)",
        status_before.stats.auto_topup_failures, status_after.stats.auto_topup_failures
    ));

    env.pic
        .advance_time(std::time::Duration::from_secs(18 * 3_600 + 60));
    tick(&env, 60);
    let last_seen_after = {
        let rec: Result<AaaRecord, ApiError> = env.query(platform, alice, "admin_get_aaa", aaa);
        rec.expect("platform sees the aaa").last_seen_at
    };
    assert!(
        last_seen_after > last_seen_before,
        "the 24h daily_tick timer must start after post_upgrade and call platform.heartbeat"
    );
    step(&format!(
        "daily_tick timer started after upgrade: platform.last_seen_at {last_seen_before} -> {last_seen_after}"
    ));
}
