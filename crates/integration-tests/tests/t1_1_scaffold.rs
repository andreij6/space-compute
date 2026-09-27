use integration_tests::{repo_root, step};
use std::process::Command;

const ENV_CANISTERS: [&str; 3] = ["platform", "payments", "treasury"];

#[test]
fn t1_1_icp_build_produces_every_canister_wasm() {
    let root = repo_root();
    println!("T1.1 demo: `icp build` compiles every environment canister");
    let out = Command::new("icp")
        .arg("build")
        .current_dir(&root)
        .output()
        .expect("icp CLI must be installed");
    assert!(
        out.status.success(),
        "icp build failed: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    step("icp build succeeded");
    for name in ENV_CANISTERS {
        let wasm = std::fs::read(root.join(".icp/cache/artifacts").join(name))
            .unwrap_or_else(|e| panic!("missing artifact for {name}: {e}"));
        assert_eq!(&wasm[..4], b"\0asm", "{name} artifact is not a wasm module");
        let has_candid = wasm.windows(14).any(|w| w == b"candid:service");
        assert!(has_candid, "{name} wasm lacks candid:service metadata");
        step(&format!(
            "{name}: {} KiB wasm with candid metadata",
            wasm.len() / 1024
        ));
    }
}

#[test]
fn t1_1_every_canister_commits_its_candid_interface() {
    let root = repo_root();
    for name in ["platform", "payments", "treasury", "aaa"] {
        let did =
            std::fs::read_to_string(root.join("crates").join(name).join(format!("{name}.did")))
                .unwrap_or_else(|e| panic!("missing {name}.did: {e}"));
        assert!(did.contains("service"), "{name}.did has no service");
        step(&format!("{name}.did committed"));
    }
}

#[test]
fn t1_1_toolchain_is_pinned_with_wasm_target() {
    let toolchain = std::fs::read_to_string(repo_root().join("rust-toolchain.toml")).unwrap();
    assert!(toolchain.contains("wasm32-unknown-unknown"));
    step("rust-toolchain.toml pins the wasm32 target");
}
