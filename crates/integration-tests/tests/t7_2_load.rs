use candid::{decode_one, encode_args, Principal};
use integration_tests::pic::{canister_wasm, IcpEnv};
use integration_tests::{repo_root, seed, step};
use platform::config::Params;
use platform::registry::RegisterArgs;
use pocket_ic::common::rest::{IcpFeatures, IcpFeaturesConfig};
use pocket_ic::PocketIcBuilder;
use serde::Serialize;
use sha2::{Digest, Sha256};
use std::time::{Duration, Instant};

use sc_types::{
    Answer, ApiError, ClassificationReceipt, ClassificationSubmission, DiscoveryFlag, Protocol,
    Question, ReviewAssignment, ReviewReceipt, ReviewSubmission, Task, Vote,
};

fn ident(n: u32) -> Principal {
    let mut bytes = [0u8; 32];
    bytes[..4].copy_from_slice(&n.to_be_bytes());
    Principal::self_authenticating(bytes)
}

fn make_env() -> IcpEnv {
    let features = IcpFeatures {
        icp_token: Some(IcpFeaturesConfig::DefaultConfig),
        cycles_minting: Some(IcpFeaturesConfig::DefaultConfig),
        registry: Some(IcpFeaturesConfig::DefaultConfig),
        ..Default::default()
    };
    let pic = PocketIcBuilder::new()
        .with_nns_subnet()
        .with_icp_features(features)
        .with_application_subnet()
        .with_max_request_time_ms(Some(1_800_000))
        .build();
    IcpEnv { pic }
}

fn call<A: candid::utils::ArgumentEncoder, R: serde::de::DeserializeOwned + candid::CandidType>(
    env: &IcpEnv,
    canister: Principal,
    sender: Principal,
    method: &str,
    args: A,
) -> Result<(R, Duration), String> {
    let start = Instant::now();
    let bytes = env
        .pic
        .update_call(canister, sender, method, encode_args(args).unwrap())
        .map_err(|e| format!("{method} rejected: {e:?}"))?;
    let elapsed = start.elapsed();
    let r: R = decode_one(&bytes).map_err(|e| format!("{method} reply did not decode: {e}"))?;
    Ok((r, elapsed))
}

fn answer_chain(protocol: &Protocol) -> Vec<Answer> {
    let map: std::collections::HashMap<&str, &Question> = protocol
        .questions
        .iter()
        .map(|q| (q.id.as_str(), q))
        .collect();
    let mut out = Vec::new();
    let mut current = &protocol.questions[0];
    loop {
        let opt = &current.answers[0];
        out.push(Answer {
            question_id: current.id.clone(),
            answer_id: opt.id.clone(),
        });
        match &opt.next {
            Some(next_id) => current = map[next_id.as_str()],
            None => break,
        }
    }
    out
}

fn roughly_one_in(counter: u64, pct: u64) -> bool {
    (counter.wrapping_mul(2_654_435_761) % 100) < pct
}

#[derive(Default, Serialize, Clone)]
struct MethodStats {
    calls: u64,
    errors: u64,
    aaa_cycles_sum: u128,
    aaa_cycles_min: u128,
    aaa_cycles_max: u128,
    platform_gain_sum: i128,
    platform_gain_min: i128,
    platform_gain_max: i128,
    wall_ns_sum: u128,
    wall_ns_max: u128,
}

impl MethodStats {
    fn record(&mut self, aaa_cycles: u128, platform_gain: i128, wall: Duration) {
        if self.calls == 0 {
            self.aaa_cycles_min = aaa_cycles;
            self.platform_gain_min = platform_gain;
        }
        self.calls += 1;
        self.aaa_cycles_sum += aaa_cycles;
        self.aaa_cycles_min = self.aaa_cycles_min.min(aaa_cycles);
        self.aaa_cycles_max = self.aaa_cycles_max.max(aaa_cycles);
        self.platform_gain_sum += platform_gain;
        self.platform_gain_min = self.platform_gain_min.min(platform_gain);
        self.platform_gain_max = self.platform_gain_max.max(platform_gain);
        self.wall_ns_sum += wall.as_nanos();
        self.wall_ns_max = self.wall_ns_max.max(wall.as_nanos());
    }
}

#[derive(Serialize)]
struct LoadReport {
    variant: String,
    n_aaas: usize,
    n_tasks_per_aaa: usize,
    total_classifications: u64,
    total_discoveries_flagged: u64,
    total_reviews_attempted: u64,
    total_reviews_completed: u64,
    fee_get_task: u128,
    fee_submit_classification: u128,
    fee_get_review: u128,
    fee_submit_review: u128,
    get_task: MethodStats,
    submit_classification: MethodStats,
    get_review_assignment: MethodStats,
    submit_review: MethodStats,
    platform_memory_bytes_before: u128,
    platform_memory_bytes_after: u128,
    sample_aaa_memory_bytes_before: u128,
    sample_aaa_memory_bytes_after: u128,
    wall_seconds_total: f64,
}

fn setup_platform(env: &IcpEnv, admin: Principal, payments: Principal) -> (Principal, Protocol) {
    let platform = env.install_on("platform", admin, 20_000_000_000_000, 0);
    let ok: Result<(), ApiError> = env.update(platform, admin, "admin_set_payments_id", payments);
    assert_eq!(ok, Ok(()));

    let wasm = canister_wasm("aaa");
    let hash = Sha256::digest(&wasm).to_vec();
    let (up, _): (Result<(), ApiError>, _) = call(
        env,
        platform,
        admin,
        "admin_upload_wasm",
        (1u32, wasm, hash),
    )
    .unwrap();
    up.unwrap();
    let (ok, _): (Result<(), ApiError>, _) =
        call(env, platform, admin, "admin_approve_wasm", (1u32,)).unwrap();
    ok.unwrap();
    step("uploaded and approved AAA wasm v1 on platform");

    let root = repo_root();
    let protocol = seed::protocol(&root.join("data/protocol/protocol_v1.json"));
    let ok: Result<(), ApiError> =
        env.update(platform, admin, "admin_add_protocol", protocol.clone());
    assert_eq!(ok, Ok(()));
    let ok: Result<(), ApiError> = env.update(
        platform,
        admin,
        "admin_set_current_protocol",
        protocol.version,
    );
    assert_eq!(ok, Ok(()));

    let subjects = seed::subjects(
        &root.join("data/curation/v1/manifest_v1.jsonl"),
        &root.join("data/curation/v1/gold_v1.json"),
        "http://127.0.0.1:8765",
        5_000,
    );
    assert!(subjects.len() >= 5_000, "need >=5000 seeded subjects");
    let mut total_added = 0u32;
    for chunk in subjects.chunks(sc_types::limits::ADMIN_BATCH_MAX) {
        let (added, _): (Result<u32, ApiError>, _) = call(
            env,
            platform,
            admin,
            "admin_add_subjects",
            (chunk.to_vec(),),
        )
        .unwrap();
        total_added += added.unwrap();
    }
    step(&format!("seeded {total_added} subjects"));

    (platform, protocol)
}

struct Agent {
    aaa: Principal,
    owner: Principal,
    operator: Principal,
}

fn spawn_aaas(
    env: &IcpEnv,
    platform: Principal,
    admin: Principal,
    payments: Principal,
    n: usize,
) -> Vec<Agent> {
    let subnet = env.pic.topology().get_app_subnets()[0];
    let mut agents = Vec::with_capacity(n);
    for i in 0..n {
        let owner = ident(1_000 + i as u32);
        let operator = ident(2_000 + i as u32);
        let aaa = env.pic.create_canister_on_subnet(Some(admin), None, subnet);
        env.pic.add_cycles(aaa, 5_000_000_000_000);
        env.pic
            .set_controllers(aaa, Some(admin), vec![owner, platform])
            .unwrap();
        let reg = RegisterArgs {
            canister_id: aaa,
            owner,
            name: format!("LoadTest-{i:04}"),
            avatar_seed: i as u64,
        };
        let ok: Result<(), ApiError> = env.update(platform, payments, "register_aaa", reg);
        assert_eq!(ok, Ok(()), "register_aaa for agent {i}");
        let (ok, _): (Result<(), ApiError>, _) = call(
            env,
            aaa,
            owner,
            "add_operator",
            (operator, format!("bot-{i:04}"), None::<u64>),
        )
        .unwrap();
        ok.unwrap();
        agents.push(Agent {
            aaa,
            owner,
            operator,
        });
    }
    for _ in 0..5 {
        env.pic.tick();
    }
    step(&format!("registered and staffed {n} AAA canisters"));
    agents
}

fn memory_bytes(env: &IcpEnv, canister: Principal, sender: Principal) -> u128 {
    env.pic
        .canister_status(canister, Some(sender))
        .expect("canister_status")
        .memory_size
        .0
        .try_into()
        .unwrap()
}

fn run(n_aaas: usize, n_tasks: usize, variant: &str) -> LoadReport {
    let env = make_env();
    let admin = ident(1);
    let payments = ident(2);
    let (platform, protocol) = setup_platform(&env, admin, payments);
    let chain = answer_chain(&protocol);
    let discovery_category = protocol.discovery_categories[0].id.clone();

    let params: Params = env.query(platform, admin, "get_params", ());

    let agents = spawn_aaas(&env, platform, admin, payments, n_aaas);
    let platform_memory_bytes_before = memory_bytes(&env, platform, admin);
    let sample_aaa_memory_bytes_before = memory_bytes(&env, agents[0].aaa, agents[0].owner);

    let mut get_task_stats = MethodStats::default();
    let mut submit_classification_stats = MethodStats::default();
    let mut get_review_assignment_stats = MethodStats::default();
    let mut submit_review_stats = MethodStats::default();
    let mut total_discoveries_flagged = 0u64;
    let mut total_reviews_attempted = 0u64;
    let mut total_reviews_completed = 0u64;
    let mut discovery_counter = 0u64;

    let wall_start = Instant::now();

    for agent in &agents {
        for t in 0..n_tasks {
            let aaa_before = env.pic.cycle_balance(agent.aaa);
            let plat_before = env.pic.cycle_balance(platform);
            let task_call: Result<(Result<Task, ApiError>, Duration), String> =
                call(&env, agent.aaa, agent.operator, "get_task", ());
            let (task_res, wall) = match task_call {
                Ok(v) => v,
                Err(_) => {
                    get_task_stats.errors += 1;
                    continue;
                }
            };
            let aaa_after = env.pic.cycle_balance(agent.aaa);
            let plat_after = env.pic.cycle_balance(platform);
            get_task_stats.record(
                aaa_before.saturating_sub(aaa_after),
                plat_after as i128 - plat_before as i128,
                wall,
            );
            let task = match task_res {
                Ok(task) => task,
                Err(_) => {
                    get_task_stats.errors += 1;
                    continue;
                }
            };

            discovery_counter += 1;
            let flag = roughly_one_in(discovery_counter, 2);
            let discovery = flag.then(|| {
                total_discoveries_flagged += 1;
                DiscoveryFlag {
                    category: discovery_category.clone(),
                    rationale: "load test: possible feature spotted during synthetic pass".into(),
                    confidence: 70,
                    claim_position: None,
                }
            });
            let submission = ClassificationSubmission {
                task_id: task.task_id,
                answers: chain.clone(),
                observed_image_sha256: task.subject.image_sha256.clone(),
                discovery,
                agent_label: None,
                submitted_by: agent.operator,
            };

            let aaa_before = env.pic.cycle_balance(agent.aaa);
            let plat_before = env.pic.cycle_balance(platform);
            let submit_call: Result<(Result<ClassificationReceipt, ApiError>, Duration), String> =
                call(
                    &env,
                    agent.aaa,
                    agent.operator,
                    "submit_classification",
                    (submission,),
                );
            let (_submit_res, wall) = match submit_call {
                Ok(v) => v,
                Err(_) => {
                    submit_classification_stats.errors += 1;
                    continue;
                }
            };
            let aaa_after = env.pic.cycle_balance(agent.aaa);
            let plat_after = env.pic.cycle_balance(platform);
            submit_classification_stats.record(
                aaa_before.saturating_sub(aaa_after),
                plat_after as i128 - plat_before as i128,
                wall,
            );

            if (t + 1) % 10 == 0 {
                total_reviews_attempted += 1;
                let aaa_before = env.pic.cycle_balance(agent.aaa);
                let plat_before = env.pic.cycle_balance(platform);
                let assign_call: Result<
                    (Result<Option<ReviewAssignment>, ApiError>, Duration),
                    String,
                > = call(&env, agent.aaa, agent.operator, "get_review_assignment", ());
                let Ok((assign_res, wall)) = assign_call else {
                    get_review_assignment_stats.errors += 1;
                    continue;
                };
                let aaa_after = env.pic.cycle_balance(agent.aaa);
                let plat_after = env.pic.cycle_balance(platform);
                get_review_assignment_stats.record(
                    aaa_before.saturating_sub(aaa_after),
                    plat_after as i128 - plat_before as i128,
                    wall,
                );
                if let Ok(Some(assignment)) = assign_res {
                    let review_sub = ReviewSubmission {
                        assignment_id: assignment.assignment_id,
                        vote: Vote::Agree,
                        rationale: "load test: reviewed classification for consistency".into(),
                        observed_image_sha256: assignment.subject.image_sha256.clone(),
                        agent_label: None,
                        submitted_by: agent.operator,
                    };
                    let aaa_before = env.pic.cycle_balance(agent.aaa);
                    let plat_before = env.pic.cycle_balance(platform);
                    let review_call: Result<(Result<ReviewReceipt, ApiError>, Duration), String> =
                        call(
                            &env,
                            agent.aaa,
                            agent.operator,
                            "submit_review",
                            (review_sub,),
                        );
                    if let Ok((res, wall)) = review_call {
                        let aaa_after = env.pic.cycle_balance(agent.aaa);
                        let plat_after = env.pic.cycle_balance(platform);
                        submit_review_stats.record(
                            aaa_before.saturating_sub(aaa_after),
                            plat_after as i128 - plat_before as i128,
                            wall,
                        );
                        if res.is_ok() {
                            total_reviews_completed += 1;
                        }
                    } else {
                        submit_review_stats.errors += 1;
                    }
                }
            }
        }
    }

    let wall_seconds_total = wall_start.elapsed().as_secs_f64();
    let platform_memory_bytes_after = memory_bytes(&env, platform, admin);
    let sample_aaa_memory_bytes_after = memory_bytes(&env, agents[0].aaa, agents[0].owner);

    step(&format!(
        "{variant}: {} classifications across {n_aaas} AAAs in {wall_seconds_total:.1}s",
        get_task_stats.calls
    ));

    LoadReport {
        variant: variant.to_string(),
        n_aaas,
        n_tasks_per_aaa: n_tasks,
        total_classifications: submit_classification_stats.calls,
        total_discoveries_flagged,
        total_reviews_attempted,
        total_reviews_completed,
        fee_get_task: params.fee_get_task,
        fee_submit_classification: params.fee_submit_classification,
        fee_get_review: params.fee_get_review,
        fee_submit_review: params.fee_submit_review,
        get_task: get_task_stats,
        submit_classification: submit_classification_stats,
        get_review_assignment: get_review_assignment_stats,
        submit_review: submit_review_stats,
        platform_memory_bytes_before,
        platform_memory_bytes_after,
        sample_aaa_memory_bytes_before,
        sample_aaa_memory_bytes_after,
        wall_seconds_total,
    }
}

fn write_report(report: &LoadReport) {
    let dir = repo_root().join("docs/perf");
    std::fs::create_dir_all(&dir).unwrap();
    let json_path = dir.join("load-test-T7.2.json");

    let mut all: serde_json::Map<String, serde_json::Value> = std::fs::read_to_string(&json_path)
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_default();
    all.insert(
        report.variant.clone(),
        serde_json::to_value(report).unwrap(),
    );
    std::fs::write(
        &json_path,
        serde_json::to_string_pretty(&serde_json::Value::Object(all.clone())).unwrap(),
    )
    .unwrap();

    let md_path = dir.join("load-test-T7.2.md");
    let mut md = String::new();
    md.push_str("# T7.2 Load Test — 200 AAAs x 100 Tasks\n\n");
    md.push_str("## Methodology\n\n");
    md.push_str(
        "PocketIC harness (`crates/integration-tests/tests/t7_2_load.rs`, `#[ignore]`, run via `just load-test`). \
         Installs `platform`, uploads and approves the AAA wasm, seeds >=5000 real subjects from \
         `data/curation/v1` (enough that `retire_after_k=5` never exhausts the pool at 200x100 = 20,000 \
         classifications), then registers N AAA canisters via the same lightest real path as `t2_2`/`t3_2` \
         (create canister -> fund cycles -> `platform.register_aaa` as the payments principal -> `add_operator`), \
         so each AAA is a real installed canister with a real operator. Each agent then runs M rounds of \
         `aaa.get_task` -> `aaa.submit_classification` through the real AAA relay with fees attached, a ~2% \
         discovery-flag rate (deterministic pseudo-random via a multiplicative hash, not `rand`), and every \
         10th round an `aaa.get_review_assignment` -> `aaa.submit_review` attempt (tier-1 AAAs get \
         `NotEligible(\"tier\")` until `calibration_tasks=50` promotes them to tier 2, so later rounds exercise \
         real tier-2 reviews).\n\n\
         Instructions/compute cost is approximated via cycle-balance deltas (PocketIC's `cycle_balance` is a \
         free in-memory read, not a consensus round, so it can be sampled on every call): the AAA's balance \
         drop per call is `fee attached + AAA's own relay execution cost`; the platform's balance gain per call \
         is `fee accepted - platform's own execution cost`, so a per-call gain close to the fee means the fee \
         is cheap to serve, and a small or negative gain flags an under-priced fee. A rough instruction \
         estimate can be derived from cycles via the IC's published ~0.4 cycles/instruction compute cost on a \
         13-node application subnet; this is an approximation, not `ic0.performance_counter` (PocketIC does not \
         expose that to test code without a benchmarking-subnet build).\n\n\
         Memory growth is `canister_status(platform).memory_size` and one sample AAA's `memory_size`, before \
         seeding/registration and after the full run.\n\n",
    );

    for key in ["200x100", "20x20"] {
        if let Some(v) = all.get(key) {
            md.push_str(&format!("## Variant: {key}\n\n"));
            render_variant(&mut md, v);
        }
    }
    md.push_str("## Fee adequacy (T7.3 input)\n\n");
    md.push_str(
        "`avg platform gain` is `fee - platform's own execution cost` per call: positive and close to the \
         fee means the fee comfortably covers compute, so T7.3 has headroom to lower it if desired; near \
         zero or negative would mean the fee needs raising. The \"compute-only estimate\" lines isolate the \
         actual instruction-driven cost (platform's own execution, and the AAA relay's own execution) from \
         the fee pass-through itself. Those compute-only estimates were in the tens of millions of cycles \
         per call in this run, roughly two to three orders of magnitude below the several-billion-cycle \
         compute budget implied by the IC's ~20B instruction per-message limit at the published ~0.4 \
         cycles/instruction application-subnet rate — no call came close to that limit. Flag any future run \
         where a compute-only estimate approaches 1% of that budget (~80M cycles).\n",
    );

    std::fs::write(&md_path, md).unwrap();
    step(&format!("wrote {}", md_path.display()));
}

fn render_variant(md: &mut String, v: &serde_json::Value) {
    let n_aaas = v["n_aaas"].as_u64().unwrap_or(0);
    let n_tasks = v["n_tasks_per_aaa"].as_u64().unwrap_or(0);
    let wall = v["wall_seconds_total"].as_f64().unwrap_or(0.0);
    md.push_str(&format!(
        "{n_aaas} AAAs x {n_tasks} tasks, {} classifications, {} discoveries flagged, \
         {} review attempts ({} completed), wall time {wall:.1}s.\n\n",
        v["total_classifications"],
        v["total_discoveries_flagged"],
        v["total_reviews_attempted"],
        v["total_reviews_completed"],
    ));
    md.push_str("| Method | Calls | Errors | Avg AAA cycles (incl. fee) | Max AAA cycles | Avg platform gain (fee kept) | Min platform gain | Avg ms | Max ms |\n");
    md.push_str("|---|---|---|---|---|---|---|---|---|\n");
    let mut compute_only_max = 0f64;
    for (method, fee_key) in [
        ("get_task", "fee_get_task"),
        ("submit_classification", "fee_submit_classification"),
        ("get_review_assignment", "fee_get_review"),
        ("submit_review", "fee_submit_review"),
    ] {
        let m = &v[method];
        let calls = m["calls"].as_u64().unwrap_or(0);
        let errors = m["errors"].as_u64().unwrap_or(0);
        let avg_aaa = m["aaa_cycles_sum"].as_f64().unwrap_or(0.0) / calls.max(1) as f64;
        let max_aaa = m["aaa_cycles_max"].as_f64().unwrap_or(0.0);
        let avg_gain = m["platform_gain_sum"].as_f64().unwrap_or(0.0) / calls.max(1) as f64;
        let min_gain = m["platform_gain_min"].as_f64().unwrap_or(0.0);
        let wall_sum = m["wall_ns_sum"].as_f64().unwrap_or(0.0);
        let avg_ms = wall_sum / calls.max(1) as f64 / 1_000_000.0;
        let max_ms = m["wall_ns_max"].as_f64().unwrap_or(0.0) / 1_000_000.0;
        md.push_str(&format!(
            "| {method} | {calls} | {errors} | {avg_aaa:.0} | {max_aaa:.0} | {avg_gain:.0} | {min_gain:.0} | {avg_ms:.2} | {max_ms:.2} |\n"
        ));
        let fee = v[fee_key].as_f64().unwrap_or(0.0);
        if calls > 0 {
            let platform_exec_est = (fee - avg_gain).max(0.0);
            let aaa_relay_est = (avg_aaa - fee).max(0.0);
            compute_only_max = compute_only_max.max(platform_exec_est).max(aaa_relay_est);
        }
    }
    md.push('\n');
    md.push_str(&format!(
        "Compute-only estimate (excludes the fee itself, which is a cycle transfer, not a burn): the \
         largest per-call figure across `fee - avg platform gain` (platform's own execution) and \
         `avg AAA cycles - fee` (the AAA relay's own execution) in this variant was ~{compute_only_max:.0} \
         cycles.\n\n"
    ));
    let fee_get_task = v["fee_get_task"].as_f64().unwrap_or(0.0);
    let fee_submit = v["fee_submit_classification"].as_f64().unwrap_or(0.0);
    md.push_str(&format!(
        "Configured fees: `fee_get_task`={fee_get_task:.0}, `fee_submit_classification`={fee_submit:.0} cycles.\n\n"
    ));
    let plat_before = v["platform_memory_bytes_before"].as_f64().unwrap_or(0.0);
    let plat_after = v["platform_memory_bytes_after"].as_f64().unwrap_or(0.0);
    let aaa_before = v["sample_aaa_memory_bytes_before"].as_f64().unwrap_or(0.0);
    let aaa_after = v["sample_aaa_memory_bytes_after"].as_f64().unwrap_or(0.0);
    md.push_str(&format!(
        "Platform stable memory: {plat_before:.0} -> {plat_after:.0} bytes (+{:.0}).\n\n",
        plat_after - plat_before
    ));
    md.push_str(&format!(
        "Sample AAA (agent 0) memory: {aaa_before:.0} -> {aaa_after:.0} bytes (+{:.0}), over {n_tasks} of its own records.\n\n",
        aaa_after - aaa_before
    ));
}

#[test]
#[ignore]
fn t7_2_load_200x100() {
    println!("T7.2 load test: 200 AAAs x 100 tasks");
    let report = run(200, 100, "200x100");
    write_report(&report);
}

#[test]
#[ignore]
fn t7_2_load_20x20() {
    println!("T7.2 load test (repeatable variant): 20 AAAs x 20 tasks");
    let report = run(20, 20, "20x20");
    write_report(&report);
}
