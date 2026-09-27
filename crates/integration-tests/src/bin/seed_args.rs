use integration_tests::{repo_root, seed};

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let base_url = args
        .get(1)
        .cloned()
        .unwrap_or_else(|| "http://127.0.0.1:8765".into());
    let limit: usize = args.get(2).and_then(|s| s.parse().ok()).unwrap_or(500);
    let root = repo_root();
    let protocol = seed::protocol(&root.join("data/protocol/protocol_v1.json"));
    let subjects = seed::subjects(
        &root.join("data/curation/v1/manifest_v1.jsonl"),
        &root.join("data/curation/v1/gold_v1.json"),
        &base_url,
        limit,
    );
    let wasm = std::fs::read(root.join("target/wasm32-unknown-unknown/release/aaa.wasm"))
        .expect("build aaa first");
    let batches = seed::write_args(&root.join("target/seed"), &protocol, &subjects, &wasm);
    let gold = subjects.iter().filter(|s| s.gold.is_some()).count();
    println!(
        "{} subjects ({gold} gold) in {batches} batches, protocol v{}, aaa wasm {} B",
        subjects.len(),
        protocol.version,
        wasm.len()
    );
}
