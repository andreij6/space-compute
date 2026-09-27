use integration_tests::{repo_root, seed};

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let base_url = args
        .get(1)
        .cloned()
        .unwrap_or_else(|| "http://127.0.0.1:8765".into());
    let limit: usize = args.get(2).and_then(|s| s.parse().ok()).unwrap_or(500);
    let root = repo_root();
    let manifest = root.join("data/curation/v1/manifest_v1.jsonl");
    let gold_path = root.join("data/curation/v1/gold_v1.json");
    let protocol = seed::protocol(&root.join("data/protocol/protocol_v1.json"));
    let mut subjects = seed::subjects(&manifest, &gold_path, &base_url, limit);
    let honeypots = seed::honeypots(&root.join("data/curation/v1/honeypots_v1.json"));
    let honeypot_ids: Vec<u32> = honeypots.iter().map(|h| h.subject_id).collect();
    seed::ensure_gold_subjects(
        &manifest,
        &gold_path,
        &base_url,
        &mut subjects,
        &honeypot_ids,
    );
    let wasm = std::fs::read(root.join("target/wasm32-unknown-unknown/release/aaa.wasm"))
        .expect("build aaa first");
    let batches = seed::write_args(&root.join("target/seed"), &protocol, &subjects, &wasm);
    seed::write_honeypots(&root.join("target/seed"), &honeypots);
    let gold = subjects.iter().filter(|s| s.gold.is_some()).count();
    println!(
        "{} subjects ({gold} gold) in {batches} batches, protocol v{}, aaa wasm {} B, {} honeypots",
        subjects.len(),
        protocol.version,
        wasm.len(),
        honeypots.len()
    );
}
