use candid::{decode_one, encode_one};
use integration_tests::pic::{canister_wasm, user, IcpEnv};
use integration_tests::{repo_root, seed, step};
use platform::progression::Stats;
use sc_types::ApiError;

fn raw(
    env: &IcpEnv,
    canister: candid::Principal,
    sender: candid::Principal,
    method: &str,
    arg: Vec<u8>,
) -> Vec<u8> {
    env.pic
        .update_call(canister, sender, method, arg)
        .unwrap_or_else(|e| panic!("{method} rejected: {e:?}"))
}

#[test]
fn t3_8_seed_args_load_into_platform_and_reseeding_is_idempotent() {
    println!("T3.8 demo: the local seed (protocol v1, subjects spread across fields, AAA wasm) loads cleanly and twice");
    let env = IcpEnv::new();
    let admin = user(1);
    let platform = env.install("platform", admin);
    let root = repo_root();
    let protocol = seed::protocol(&root.join("data/protocol/protocol_v1.json"));
    let subjects = seed::subjects(
        &root.join("data/curation/v1/manifest_v1.jsonl"),
        &root.join("data/curation/v1/gold_v1.json"),
        "http://127.0.0.1:8765",
        500,
    );
    let fields: std::collections::BTreeSet<_> =
        subjects.iter().map(|s| s.subject.field.clone()).collect();
    assert_eq!(subjects.len(), 500);
    assert_eq!(
        fields.len(),
        6,
        "seed should span all six fields: {fields:?}"
    );
    step(&format!(
        "built 500 subjects across {} fields from the committed manifest",
        fields.len()
    ));

    for round in 1..=2 {
        let r: Result<(), ApiError> = decode_one(&raw(
            &env,
            platform,
            admin,
            "admin_add_protocol",
            encode_one(&protocol).unwrap(),
        ))
        .unwrap();
        assert_eq!(
            r,
            Ok(()),
            "round {round}: protocol v1 must be accepted (identical re-add is a no-op)"
        );
        let added: Result<u32, ApiError> = decode_one(&raw(
            &env,
            platform,
            admin,
            "admin_add_subjects",
            encode_one(&subjects).unwrap(),
        ))
        .unwrap();
        let expected = if round == 1 { 500 } else { 0 };
        assert_eq!(added, Ok(expected), "round {round}");
        let (_, wasm_arg) = seed::wasm_upload(1, &canister_wasm("aaa"));
        let up: Result<(), ApiError> =
            decode_one(&raw(&env, platform, admin, "admin_upload_wasm", wasm_arg)).unwrap();
        assert!(
            up.is_ok() || matches!(up, Err(ApiError::Conflict(_))),
            "round {round}: {up:?}"
        );
        step(&format!(
            "round {round}: protocol v1 ok, {expected} new subjects, AAA wasm v1 uploaded"
        ));
    }
    let stats: Stats = env.query(platform, admin, "get_stats", ());
    assert_eq!(stats.total_subjects, 500);
    step("platform reports 500 subjects after two seed runs — nothing duplicated or reset");
}
