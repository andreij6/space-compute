use integration_tests::{repo_root, step};
use std::process::Command;

fn run(script: &str, envs: &[(&str, &str)]) -> (bool, String) {
    let out = Command::new("bash")
        .arg(script)
        .envs(envs.iter().copied())
        .current_dir(repo_root())
        .output()
        .expect("bash");
    let text = format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    (out.status.success(), text)
}

#[test]
fn t1_3_candid_drift_check_passes_then_catches_a_changed_interface() {
    println!("T1.3 demo: the .did drift gate");
    let (clean, text) = run("scripts/check-candid.sh", &[]);
    assert!(clean, "committed .did files should match the code:\n{text}");
    step("committed .did files match the compiled canisters");

    let tmp = std::env::temp_dir().join(format!("sc-did-{}", std::process::id()));
    for c in ["platform", "payments", "treasury", "aaa"] {
        let dir = tmp.join(c);
        std::fs::create_dir_all(&dir).unwrap();
        let did = std::fs::read_to_string(repo_root().join(format!("crates/{c}/{c}.did"))).unwrap();
        let did = if c == "platform" {
            did.replace("version", "renamed_version")
        } else {
            did
        };
        std::fs::write(dir.join(format!("{c}.did")), did).unwrap();
    }
    let (clean, text) = run(
        "scripts/check-candid.sh",
        &[("DID_ROOT", tmp.to_str().unwrap())],
    );
    std::fs::remove_dir_all(&tmp).ok();
    assert!(!clean, "a tampered platform.did must be reported as drift");
    assert!(text.contains("candid drift"), "{text}");
    step("a tampered platform.did is reported as drift");
}

#[test]
fn t1_3_deploy_local_is_idempotent_and_every_canister_answers() {
    println!("T1.3 demo: one-shot local deploy, run twice");
    for attempt in 1..=2 {
        let (ok, text) = run("scripts/deploy-local.sh", &[]);
        assert!(ok, "deploy-local.sh run {attempt} failed:\n{text}");
        for c in ["platform", "payments", "treasury"] {
            let expect = format!("{c} 0.1.0");
            assert!(
                text.contains(&expect),
                "run {attempt}: {c} did not answer version:\n{text}"
            );
        }
        step(&format!(
            "run {attempt}: platform, payments, treasury deployed and answering"
        ));
    }
}
