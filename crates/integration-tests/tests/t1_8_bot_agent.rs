use integration_tests::{repo_root, step};
use std::process::Command;

#[test]
fn t1_8_bot_agent_talks_to_every_local_canister() {
    println!("T1.8 demo: the deterministic (non-LLM) bot agent reaches the local canisters");
    let root = repo_root();
    let deploy = Command::new("bash")
        .arg("scripts/deploy-local.sh")
        .current_dir(&root)
        .output()
        .unwrap();
    assert!(
        deploy.status.success(),
        "{}",
        String::from_utf8_lossy(&deploy.stderr)
    );
    step("local network up and canisters deployed");
    let bot = root.join("agent-kit/tests/bot-agent");
    if !bot.join("node_modules").exists() {
        let npm = Command::new("npm")
            .args(["ci", "--silent"])
            .current_dir(&bot)
            .status()
            .unwrap();
        assert!(npm.success(), "npm ci failed");
    }
    let out = Command::new("node")
        .arg("smoke.mjs")
        .current_dir(&bot)
        .output()
        .unwrap();
    let text = String::from_utf8_lossy(&out.stdout).to_string();
    assert!(
        out.status.success(),
        "{text}{}",
        String::from_utf8_lossy(&out.stderr)
    );
    for line in text
        .lines()
        .filter(|l| l.contains('✓') || l.starts_with("bot agent"))
    {
        step(line.trim().trim_start_matches('✓').trim());
    }
    assert!(text.contains("BOT_OK 3"), "{text}");
}
