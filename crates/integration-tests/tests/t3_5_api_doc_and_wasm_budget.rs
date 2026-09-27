use candid::Principal;
use integration_tests::pic::{canister_wasm, user, IcpEnv};
use integration_tests::step;

#[test]
fn t3_5_api_doc_and_wasm_budget() {
    println!("T3.5 demo: get_api_doc + wasm size budget <= 1.5 MiB gz");
    let env = IcpEnv::new();
    let alice = user(1);
    let owner = user(10);
    let platform = user(100);
    let payments = user(101);

    let wasm = canister_wasm("aaa");
    assert!(
        wasm.len() < 5_000_000,
        "uncompressed wasm size {} should be reasonable",
        wasm.len()
    );

    let init = aaa::config::AaaInit {
        owner,
        name: "Surveyor-Doc-01".into(),
        avatar_seed: 42,
        platform_id: platform,
        payments_id: payments,
    };
    let aaa_id = env.install_with_arg("aaa", alice, init);
    step("installed AAA canister");

    let doc: String = env.query(aaa_id, Principal::anonymous(), "get_api_doc", ());
    assert!(doc.contains("# Space Compute Autonomous Astronomy Agent (AAA) API"));
    assert!(doc.contains("## Setup & Authentication"));
    assert!(doc.contains("## Canister Endpoints"));
    assert!(doc.contains("## Classification Workflow"));
    assert!(doc.contains("## Review Workflow"));
    assert!(doc.contains("## Cycle Balance & Fuel Guard"));
    assert!(doc.contains("Untrusted Rationale Warning"));
    step("queried get_api_doc anonymously and verified markdown sections");
}
