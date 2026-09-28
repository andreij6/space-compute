use candid::{decode_one, encode_args, CandidType, Principal};
use integration_tests::pic::{canister_wasm, user, IcpEnv};
use integration_tests::{repo_root, step};
use platform::api::{AdminDiscoveryCard, Overview};
use platform::audit::AuditEntry;
use platform::catalog::SubjectInput;
use platform::config::{Params, PauseFlags};
use platform::discoveries::AdminListDiscoveriesFilter;
use platform::events::Page;
use platform::registry::{AaaStatus, AdminListAaasFilter, RegisterArgs};
use platform::reviews::HoneypotSpec;
use sc_types::{
    Answer, AnswerOption, ApiError, DiscoveryCategory, Protocol, Question, SubjectRef, Vote,
};
use serde::de::DeserializeOwned;
use sha2::{Digest, Sha256};
use std::collections::HashSet;

fn tick(env: &IcpEnv, n: usize) {
    for _ in 0..n {
        env.pic.tick();
    }
}

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

fn admin_update_methods() -> Vec<String> {
    let did = std::fs::read_to_string(repo_root().join("crates/platform/platform.did"))
        .expect("read platform.did");
    let start = did.find("service").expect("service block present");
    let body = &did[start..];
    let open = body.find('{').expect("service block open brace");
    let close = body.rfind('}').expect("service block close brace");
    body[open + 1..close]
        .split(';')
        .filter_map(|raw| {
            let entry: String = raw.split_whitespace().collect::<Vec<_>>().join(" ");
            if entry.is_empty() {
                return None;
            }
            let name = entry.split(':').next()?.trim().to_string();
            if !name.starts_with("admin_") || entry.trim_end().ends_with("query") {
                return None;
            }
            Some(name)
        })
        .collect()
}

fn audited_methods(env: &IcpEnv, platform: Principal, admin: Principal) -> HashSet<String> {
    let mut methods = HashSet::new();
    let mut cursor: Option<u64> = None;
    loop {
        let page: Result<Vec<AuditEntry>, ApiError> =
            query(env, platform, admin, "admin_audit_log", (cursor, 100u32));
        let page = page.expect("admin_audit_log");
        let n = page.len();
        for e in page {
            methods.insert(e.method);
        }
        if n < 100 {
            break;
        }
        cursor = Some(cursor.unwrap_or(0) + n as u64);
    }
    methods
}

#[test]
fn t4_10_every_admin_mutation_writes_an_audit_entry() {
    println!("T4.10 demo: every admin_* update method in platform.did leaves an audit entry");
    let env = IcpEnv::new();
    let alice = user(1);
    let bob = user(2);
    let payments = user(3);
    let platform = env.install_on("platform", alice, 10_000_000_000_000, 0);
    tick(&env, 2);

    let ok: Result<(), ApiError> =
        call(&env, platform, alice, "admin_set_payments_id", (payments,));
    assert_eq!(ok, Ok(()));

    let wasm = canister_wasm("aaa");
    let hash = Sha256::digest(&wasm).to_vec();
    let ok: Result<(), ApiError> = call(
        &env,
        platform,
        alice,
        "admin_upload_wasm",
        (1u32, wasm, hash),
    );
    assert_eq!(ok, Ok(()));
    let ok: Result<(), ApiError> = call(&env, platform, alice, "admin_approve_wasm", (1u32,));
    assert_eq!(ok, Ok(()));

    let ok: Result<(), ApiError> = call(
        &env,
        platform,
        alice,
        "admin_add_protocol",
        (sample_protocol(1),),
    );
    assert_eq!(ok, Ok(()));
    let ok: Result<(), ApiError> =
        call(&env, platform, alice, "admin_set_current_protocol", (1u16,));
    assert_eq!(ok, Ok(()));

    let batch = vec![
        SubjectInput {
            subject: sample_ref(1),
            gold: None,
        },
        SubjectInput {
            subject: sample_ref(2),
            gold: Some(vec![Answer {
                question_id: "q1".into(),
                answer_id: "smooth".into(),
            }]),
        },
    ];
    let added: Result<u32, ApiError> = call(&env, platform, alice, "admin_add_subjects", (batch,));
    assert_eq!(added, Ok(2));
    step("bootstrapped platform: payments id, aaa wasm v1, protocol v1, subjects 1 & 2");

    let subnet = env.pic.topology().get_app_subnets()[0];
    let owner_1 = user(101);
    let aaa_1 = env.pic.create_canister_on_subnet(Some(alice), None, subnet);
    env.pic.add_cycles(aaa_1, 5_000_000_000_000);
    env.pic
        .set_controllers(aaa_1, Some(alice), vec![owner_1, platform])
        .unwrap();
    let ok: Result<(), ApiError> = call(
        &env,
        platform,
        payments,
        "register_aaa",
        (RegisterArgs {
            canister_id: aaa_1,
            owner: owner_1,
            name: "Audit-One".into(),
            avatar_seed: 1,
        },),
    );
    assert_eq!(ok, Ok(()));

    let owner_2 = user(102);
    let aaa_2 = env.pic.create_canister_on_subnet(Some(alice), None, subnet);
    env.pic.add_cycles(aaa_2, 5_000_000_000_000);
    env.pic
        .set_controllers(aaa_2, Some(alice), vec![owner_2, platform])
        .unwrap();
    let ok: Result<(), ApiError> = call(
        &env,
        platform,
        payments,
        "register_aaa",
        (RegisterArgs {
            canister_id: aaa_2,
            owner: owner_2,
            name: "Audit-Two".into(),
            avatar_seed: 2,
        },),
    );
    assert_eq!(ok, Ok(()));
    step("registered two real AAAs (Audit-One, Audit-Two) as targets for admin actions");

    let ok: Result<(), ApiError> = call(&env, platform, alice, "admin_add_admin", (bob,));
    assert_eq!(ok, Ok(()));
    let ok: Result<(), ApiError> = call(&env, platform, alice, "admin_set_house", (aaa_1, true));
    assert_eq!(ok, Ok(()));
    let ok: Result<(), ApiError> = call(
        &env,
        platform,
        alice,
        "admin_rename_aaa",
        (
            aaa_1,
            "Audit-One-Renamed".to_string(),
            "moderation".to_string(),
        ),
    );
    assert_eq!(ok, Ok(()));
    let ok: Result<(), ApiError> = call(
        &env,
        platform,
        alice,
        "admin_suspend_aaa",
        (aaa_2, "policy review".to_string()),
    );
    assert_eq!(ok, Ok(()));
    let ok: Result<(), ApiError> = call(&env, platform, alice, "admin_unsuspend_aaa", (aaa_2,));
    assert_eq!(ok, Ok(()));
    let ok: Result<(), ApiError> = call(
        &env,
        platform,
        alice,
        "admin_set_subject_active",
        (1u32, false),
    );
    assert_eq!(ok, Ok(()));

    let honeypot_added: Result<u32, ApiError> = call(
        &env,
        platform,
        alice,
        "admin_add_honeypots",
        (vec![HoneypotSpec {
            subject_id: 2,
            category: "lens".into(),
            rationale: "a lens on a clean elliptical".into(),
            truth: Vote::Disagree,
        }],),
    );
    assert_eq!(honeypot_added, Ok(1));

    let params_bytes = env
        .pic
        .query_call(platform, alice, "get_params", encode_args(()).unwrap())
        .expect("get_params");
    let mut params: Params = decode_one(&params_bytes).unwrap();
    params.fee_get_task = 0;
    let ok: Result<(), ApiError> = call(&env, platform, alice, "admin_set_params", (params,));
    assert_eq!(ok, Ok(()));

    let ok: Result<(), ApiError> = call(
        &env,
        platform,
        alice,
        "admin_pause",
        (PauseFlags {
            tasks: false,
            reviews: false,
            spawns: false,
        },),
    );
    assert_eq!(ok, Ok(()));

    let replay: Result<platform::progression::ReplayStatus, ApiError> = call(
        &env,
        platform,
        alice,
        "admin_replay_progression",
        (0u64, 1000u32),
    );
    assert!(replay.is_ok());

    let ok: Result<(), ApiError> = call(&env, platform, alice, "admin_remove_admin", (bob,));
    assert_eq!(ok, Ok(()));
    step("exercised admin_add_admin, admin_set_house, admin_rename_aaa, admin_suspend/unsuspend_aaa, admin_set_subject_active, admin_add_honeypots, admin_set_params, admin_pause, admin_replay_progression, admin_remove_admin");

    let owner_3 = user(103);
    let aaa_3 = env.pic.create_canister_on_subnet(Some(alice), None, subnet);
    env.pic.add_cycles(aaa_3, 5_000_000_000_000);
    env.pic
        .set_controllers(aaa_3, Some(alice), vec![alice, owner_3])
        .unwrap();
    let failed: Result<(), ApiError> = call(
        &env,
        platform,
        payments,
        "register_aaa",
        (RegisterArgs {
            canister_id: aaa_3,
            owner: owner_3,
            name: "Audit-Three".into(),
            avatar_seed: 3,
        },),
    );
    assert!(
        failed.is_err(),
        "install must fail without platform as a controller"
    );
    let stuck: Option<platform::registry::AaaRecord> =
        query(&env, platform, alice, "get_aaa", (aaa_3,));
    assert_eq!(stuck.unwrap().status, AaaStatus::Installing);

    env.pic
        .set_controllers(aaa_3, Some(alice), vec![alice, owner_3, platform])
        .unwrap();
    let retried: Result<(), ApiError> =
        call(&env, platform, alice, "admin_retry_install", (aaa_3,));
    assert_eq!(retried, Ok(()));
    step("admin_retry_install recovered a stuck Installing AAA after fixing its controllers");

    let methods = audited_methods(&env, platform, alice);
    let expected = admin_update_methods();
    assert!(
        !expected.is_empty(),
        "expected admin_* update methods parsed from platform.did"
    );
    let missing: Vec<&String> = expected.iter().filter(|m| !methods.contains(*m)).collect();
    assert!(
        missing.is_empty(),
        "admin mutation methods missing from the audit log: {missing:?} (present: {methods:?})"
    );
    step(&format!(
        "every admin_* update method in platform.did ({} methods) produced an AuditEntry",
        expected.len()
    ));
}

#[test]
fn t4_10_admin_read_apis_are_paged_admin_only_and_reflect_state() {
    println!(
        "T4.10 demo: admin_overview/admin_list_* are paged, admin-gated, and reflect live state"
    );
    let env = IcpEnv::new();
    let alice = user(1);
    let stranger = user(9);
    let payments = user(3);
    let platform = env.install_on("platform", alice, 10_000_000_000_000, 0);
    tick(&env, 2);

    let _: Result<(), ApiError> = call(&env, platform, alice, "admin_set_payments_id", (payments,));
    let wasm = canister_wasm("aaa");
    let hash = Sha256::digest(&wasm).to_vec();
    let _: Result<(), ApiError> = call(
        &env,
        platform,
        alice,
        "admin_upload_wasm",
        (1u32, wasm, hash),
    );
    let _: Result<(), ApiError> = call(&env, platform, alice, "admin_approve_wasm", (1u32,));

    let subnet = env.pic.topology().get_app_subnets()[0];
    let mut aaas = Vec::new();
    for i in 1..=3u8 {
        let owner = user(200 + i);
        let canister = env.pic.create_canister_on_subnet(Some(alice), None, subnet);
        env.pic.add_cycles(canister, 5_000_000_000_000);
        env.pic
            .set_controllers(canister, Some(alice), vec![owner, platform])
            .unwrap();
        let ok: Result<(), ApiError> = call(
            &env,
            platform,
            payments,
            "register_aaa",
            (RegisterArgs {
                canister_id: canister,
                owner,
                name: format!("Paged-{i}"),
                avatar_seed: i as u64,
            },),
        );
        assert_eq!(ok, Ok(()));
        aaas.push(canister);
    }
    step("registered 3 AAAs to exercise admin_list_aaas paging");

    let unauthorized: Result<Overview, ApiError> =
        query(&env, platform, stranger, "admin_overview", ());
    assert_eq!(unauthorized, Err(ApiError::Unauthorized));
    step("a non-admin's admin_overview call is rejected with Unauthorized");

    let overview: Result<Overview, ApiError> = query(&env, platform, alice, "admin_overview", ());
    let overview = overview.expect("admin_overview");
    assert_eq!(overview.total_aaas, 3);
    assert_eq!(overview.current_wasm_version, Some(1));
    assert_eq!(overview.alerts.suspended_aaas, 0);

    let ok: Result<(), ApiError> = call(
        &env,
        platform,
        alice,
        "admin_suspend_aaa",
        (aaas[0], "test".to_string()),
    );
    assert_eq!(ok, Ok(()));
    let overview_after: Overview =
        query::<_, Result<Overview, ApiError>>(&env, platform, alice, "admin_overview", ())
            .unwrap();
    assert_eq!(overview_after.alerts.suspended_aaas, 1);
    step("admin_overview alerts.suspended_aaas reflects a real admin_suspend_aaa");

    let page1: Result<platform::registry::AaaPage, ApiError> = query(
        &env,
        platform,
        alice,
        "admin_list_aaas",
        (AdminListAaasFilter::default(), None::<Principal>, 2u32),
    );
    let page1 = page1.expect("admin_list_aaas page 1");
    assert_eq!(page1.items.len(), 2);
    let page2: Result<platform::registry::AaaPage, ApiError> = query(
        &env,
        platform,
        alice,
        "admin_list_aaas",
        (AdminListAaasFilter::default(), page1.next_cursor, 2u32),
    );
    let page2 = page2.expect("admin_list_aaas page 2");
    assert_eq!(page2.items.len(), 1);
    assert_eq!(page2.next_cursor, None);
    let mut names: Vec<String> = page1
        .items
        .iter()
        .chain(page2.items.iter())
        .map(|r| r.name.clone())
        .collect();
    names.sort();
    assert_eq!(names, vec!["Paged-1", "Paged-2", "Paged-3"]);
    step("admin_list_aaas pages through all 3 AAAs with a cursor, 2 per page");

    let suspended_only: Result<platform::registry::AaaPage, ApiError> = query(
        &env,
        platform,
        alice,
        "admin_list_aaas",
        (
            AdminListAaasFilter {
                status: Some(AaaStatus::Suspended),
                name_prefix: None,
                owner: None,
            },
            None::<Principal>,
            100u32,
        ),
    );
    assert_eq!(suspended_only.unwrap().items.len(), 1);

    let subject_batch = vec![SubjectInput {
        subject: sample_ref(1),
        gold: Some(vec![Answer {
            question_id: "q1".into(),
            answer_id: "smooth".into(),
        }]),
    }];
    let _: Result<u32, ApiError> = call(
        &env,
        platform,
        alice,
        "admin_add_subjects",
        (subject_batch,),
    );
    let honeypots: Result<u32, ApiError> = call(
        &env,
        platform,
        alice,
        "admin_add_honeypots",
        (vec![HoneypotSpec {
            subject_id: 1,
            category: "lens".into(),
            rationale: "a lens on a clean elliptical".into(),
            truth: Vote::Disagree,
        }],),
    );
    assert_eq!(honeypots, Ok(1));

    let honeypot_cards: Result<Page<AdminDiscoveryCard>, ApiError> = query(
        &env,
        platform,
        alice,
        "admin_list_discoveries",
        (
            AdminListDiscoveriesFilter {
                honeypot: Some(true),
                ..Default::default()
            },
            None::<u64>,
            100u32,
        ),
    );
    let honeypot_cards = honeypot_cards.expect("admin_list_discoveries");
    assert_eq!(honeypot_cards.items.len(), 1);
    assert!(honeypot_cards.items[0].is_honeypot);
    step("admin_list_discoveries surfaces the seeded honeypot under filter.honeypot=true");

    let stats: Result<Vec<platform::reviews::HoneypotStat>, ApiError> =
        query(&env, platform, alice, "admin_honeypot_stats", ());
    assert_eq!(stats, Ok(vec![]));
    step("admin_honeypot_stats is reachable and empty before any honeypot has been reviewed");
}
