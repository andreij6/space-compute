use std::io::Write;
use std::process::Command;

use flate2::write::GzEncoder;
use flate2::Compression;
use integration_tests::pic::{canister_wasm, user, IcpEnv};
use integration_tests::{repo_root, step};

const MAX_GZ_BYTES: usize = 1_572_864;

#[test]
fn t3_6_aaa_wasm_shrunk_and_gzipped_is_at_most_1_5_mib() {
    println!("T3.6 demo: 03 §8.6 — the aaa wasm is <= 1.5 MiB after a real ic-wasm shrink + gzip, verified against a real installed canister");
    let raw = canister_wasm("aaa");

    let env = IcpEnv::new();
    let owner = user(10);
    let init = aaa::config::AaaInit {
        owner,
        platform_id: user(90),
        payments_id: user(91),
        name: "GzBudget-01".into(),
        avatar_seed: 1,
    };
    let aaa_id = env.install_with_arg("aaa", owner, init);
    let version: String = env.query(aaa_id, owner, "version", ());
    assert!(
        !version.is_empty(),
        "the exact wasm bytes being measured must also boot as a real canister"
    );
    step("installed the exact aaa wasm bytes under test as a real PocketIC canister");

    let root = repo_root();
    let raw_path = root.join("target/sc-t3-6-aaa-raw.wasm");
    let shrunk_path = root.join("target/sc-t3-6-aaa-shrunk.wasm");
    std::fs::write(&raw_path, &raw).expect("write raw wasm for shrinking");

    let status = Command::new("ic-wasm")
        .arg(&raw_path)
        .arg("-o")
        .arg(&shrunk_path)
        .arg("shrink")
        .status()
        .expect("run ic-wasm shrink (required toolchain, matches scripts/verify-local.sh)");
    assert!(status.success(), "ic-wasm shrink must succeed");
    let shrunk = std::fs::read(&shrunk_path).expect("read shrunk wasm");

    let mut gz = GzEncoder::new(Vec::new(), Compression::best());
    gz.write_all(&shrunk).expect("gzip shrunk wasm");
    let gz_bytes = gz.finish().expect("finish gzip stream");

    step(&format!(
        "aaa wasm: raw {} B, ic-wasm shrunk {} B, shrunk+gzip {} B (budget {} B / 1.5 MiB)",
        raw.len(),
        shrunk.len(),
        gz_bytes.len(),
        MAX_GZ_BYTES
    ));
    assert!(
        gz_bytes.len() <= MAX_GZ_BYTES,
        "aaa wasm gzipped after ic-wasm shrink is {} B, must be <= {} B (1.5 MiB) so the platform can install it in one call",
        gz_bytes.len(),
        MAX_GZ_BYTES
    );
}
