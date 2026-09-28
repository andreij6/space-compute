use candid::{decode_one, encode_args, encode_one, Principal};
use integration_tests::pic::{canister_wasm, IcpEnv};
use integration_tests::{repo_root, seed, step};
use platform::config::Params;
use platform::registry::RegisterArgs;
use pocket_ic::common::rest::{IcpFeatures, IcpFeaturesConfig};
use pocket_ic::PocketIcBuilder;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, HashMap};
use std::time::{Duration, Instant};

use sc_types::{
    Answer, ApiError, ClassificationReceipt, ClassificationSubmission, DiscoveryFlag, Protocol,
    Question, ReviewAssignment, ReviewReceipt, ReviewSubmission, Task, Vote,
};

#[derive(Clone, Copy)]
enum Placement {
    App,
    Fiduciary,
}

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
        .with_fiduciary_subnet()
        .with_max_request_time_ms(Some(1_800_000))
        .build();
    IcpEnv { pic }
}

fn subnet(env: &IcpEnv, placement: Placement) -> Principal {
    let topo = env.pic.topology();
    match placement {
        Placement::App => topo.get_app_subnets()[0],
        Placement::Fiduciary => topo.get_fiduciary().expect("fiduciary subnet"),
    }
}

fn install(env: &IcpEnv, name: &str, controller: Principal, placement: Placement) -> Principal {
    let id = env
        .pic
        .create_canister_on_subnet(Some(controller), None, subnet(env, placement));
    env.pic.add_cycles(id, 20_000_000_000_000);
    env.pic.install_canister(
        id,
        canister_wasm(name),
        encode_one(()).unwrap(),
        Some(controller),
    );
    id
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

type GoldMap = HashMap<u32, HashMap<String, String>>;

fn gold_map() -> GoldMap {
    let raw: HashMap<String, serde_json::Value> = serde_json::from_str(
        &std::fs::read_to_string(repo_root().join("data/curation/v1/gold_v1.json")).unwrap(),
    )
    .unwrap();
    raw.into_iter()
        .map(|(id, row)| {
            let answers = row["answers"]
                .as_object()
                .unwrap()
                .iter()
                .map(|(q, a)| (q.clone(), a.as_str().unwrap().to_string()))
                .collect();
            (id.parse().unwrap(), answers)
        })
        .collect()
}

fn answer_chain(protocol: &Protocol, gold: Option<&HashMap<String, String>>) -> Vec<Answer> {
    let map: HashMap<&str, &Question> = protocol
        .questions
        .iter()
        .map(|q| (q.id.as_str(), q))
        .collect();
    let mut out = Vec::new();
    let mut current = &protocol.questions[0];
    loop {
        let wanted = gold.and_then(|g| g.get(&current.id));
        let opt = current
            .answers
            .iter()
            .find(|a| Some(&a.id) == wanted)
            .unwrap_or(&current.answers[0]);
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

#[derive(Default, Serialize, Deserialize, Clone)]
struct MethodStats {
    calls: u64,
    errors: u64,
    platform_cost_sum: i128,
    platform_cost_max: i128,
    aaa_overhead_sum: i128,
    aaa_overhead_max: i128,
    wall_ns_sum: u128,
    wall_ns_max: u128,
    #[serde(skip)]
    platform_costs: Vec<i128>,
    platform_cost_p50: i128,
    platform_cost_p95: i128,
    platform_cost_p99: i128,
}

impl MethodStats {
    fn record(&mut self, s: Sample) {
        self.calls += 1;
        self.platform_cost_sum += s.platform_cost;
        self.platform_cost_max = self.platform_cost_max.max(s.platform_cost);
        self.aaa_overhead_sum += s.aaa_overhead;
        self.aaa_overhead_max = self.aaa_overhead_max.max(s.aaa_overhead);
        self.wall_ns_sum += s.wall.as_nanos();
        self.wall_ns_max = self.wall_ns_max.max(s.wall.as_nanos());
        self.platform_costs.push(s.platform_cost);
    }

    fn finish(&mut self) {
        self.platform_costs.sort_unstable();
        let at = |pct: usize| {
            self.platform_costs
                .get(self.platform_costs.len().saturating_sub(1) * pct / 100)
                .copied()
                .unwrap_or_default()
        };
        (
            self.platform_cost_p50,
            self.platform_cost_p95,
            self.platform_cost_p99,
        ) = (at(50), at(95), at(99));
    }
}

#[derive(Clone, Copy)]
struct Sample {
    platform_cost: i128,
    aaa_overhead: i128,
    wall: Duration,
}

struct Meter<'a> {
    env: &'a IcpEnv,
    platform: Principal,
    stats: BTreeMap<String, MethodStats>,
}

impl Meter<'_> {
    fn measure<R>(
        &mut self,
        aaa: Principal,
        fee: u128,
        f: impl FnOnce() -> Result<(R, Duration), String>,
    ) -> Result<(R, Sample), String> {
        let aaa_before = self.env.pic.cycle_balance(aaa) as i128;
        let plat_before = self.env.pic.cycle_balance(self.platform) as i128;
        let (r, wall) = f()?;
        let aaa_drop = aaa_before - self.env.pic.cycle_balance(aaa) as i128;
        let plat_gain = self.env.pic.cycle_balance(self.platform) as i128 - plat_before;
        let fee = fee as i128;
        let accepted = if aaa_drop >= fee { fee } else { 0 };
        Ok((
            r,
            Sample {
                platform_cost: accepted - plat_gain,
                aaa_overhead: aaa_drop - accepted,
                wall,
            },
        ))
    }

    fn record(&mut self, label: &str, s: Sample) {
        self.stats.entry(label.into()).or_default().record(s);
    }

    fn error(&mut self, label: &str) {
        self.stats.entry(label.into()).or_default().errors += 1;
    }
}

#[derive(Serialize, Deserialize)]
struct LoadReport {
    placement: String,
    n_aaas: usize,
    n_tasks_per_aaa: usize,
    total_classifications: u64,
    total_discoveries_flagged: u64,
    total_reviews_completed: u64,
    params: BTreeMap<String, u128>,
    stats: BTreeMap<String, MethodStats>,
    platform_memory_bytes: [u128; 3],
    platform_stable_by_round: Vec<(u64, u128, u128)>,
    sample_aaa_memory_bytes: [u128; 3],
    classifications_at_mid: u64,
    wall_seconds_total: f64,
}

fn setup_platform(
    env: &IcpEnv,
    admin: Principal,
    payments: Principal,
    placement: Placement,
    n_subjects: usize,
) -> (Principal, Protocol) {
    let platform = install(env, "platform", admin, placement);
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
        n_subjects,
    );
    for chunk in subjects.chunks(sc_types::limits::ADMIN_BATCH_MAX) {
        let (added, _): (Result<u32, ApiError>, _) = call(
            env,
            platform,
            admin,
            "admin_add_subjects",
            (chunk.to_vec(),),
        )
        .unwrap();
        added.unwrap();
    }
    step(&format!("seeded {} subjects", subjects.len()));
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
    placement: Placement,
) -> Vec<Agent> {
    let subnet = subnet(env, placement);
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

fn stable_and_heap(
    env: &IcpEnv,
    canister: Principal,
    sender: Principal,
    classifications: u64,
) -> (u64, u128, u128) {
    let m = env
        .pic
        .canister_status(canister, Some(sender))
        .expect("canister_status")
        .memory_metrics;
    (
        classifications,
        m.stable_memory_size.0.try_into().unwrap(),
        m.wasm_memory_size.0.try_into().unwrap(),
    )
}

fn run(n_aaas: usize, n_tasks: usize, placement: Placement) -> LoadReport {
    let env = make_env();
    let admin = ident(1);
    let payments = ident(2);
    let n_subjects = (n_aaas * n_tasks).clamp(1_000, 5_000);
    let (platform, protocol) = setup_platform(&env, admin, payments, placement, n_subjects);
    let gold = gold_map();
    let discovery_category = protocol.discovery_categories[0].id.clone();
    let params: Params = env.query(platform, admin, "get_params", ());
    let agents = spawn_aaas(&env, platform, admin, payments, n_aaas, placement);

    let mut meter = Meter {
        env: &env,
        platform,
        stats: BTreeMap::new(),
    };
    let mut platform_mem = [memory_bytes(&env, platform, admin), 0, 0];
    let mut aaa_mem = [memory_bytes(&env, agents[0].aaa, agents[0].owner), 0, 0];
    let mut stable_by_round = Vec::new();
    let mut classifications = 0u64;
    let mut classifications_at_mid = 0u64;
    let mut flagged = 0u64;
    let mut reviews_completed = 0u64;
    let mut counter = 0u64;
    let calibration = params.calibration_tasks as usize;
    let wall_start = Instant::now();

    for t in 0..n_tasks {
        stable_by_round.push(stable_and_heap(&env, platform, admin, classifications));
        if t == n_tasks / 2 {
            platform_mem[1] = memory_bytes(&env, platform, admin);
            aaa_mem[1] = memory_bytes(&env, agents[0].aaa, agents[0].owner);
            classifications_at_mid = classifications;
        }
        for agent in &agents {
            let got = meter.measure(agent.aaa, params.fee_get_task, || {
                call::<_, Result<Task, ApiError>>(&env, agent.aaa, agent.operator, "get_task", ())
            });
            let task = match got {
                Ok((Ok(task), s)) => {
                    meter.record("get_task", s);
                    if t + 10 >= n_tasks {
                        meter.record("get_task_last10", s);
                    }
                    task
                }
                _ => {
                    meter.error("get_task");
                    continue;
                }
            };

            counter += 1;
            let discovery = roughly_one_in(counter, 2).then(|| DiscoveryFlag {
                category: discovery_category.clone(),
                rationale: "load test: possible feature spotted during synthetic pass".into(),
                confidence: 70,
                claim_position: None,
            });
            let label = if discovery.is_some() {
                "submit_classification_flagged"
            } else {
                "submit_classification"
            };
            let submission = ClassificationSubmission {
                task_id: task.task_id,
                answers: answer_chain(&task.protocol, gold.get(&task.subject.subject_id)),
                observed_image_sha256: task.subject.image_sha256.clone(),
                discovery,
                agent_label: None,
                submitted_by: agent.operator,
            };
            match meter.measure(agent.aaa, params.fee_submit_classification, || {
                call::<_, Result<ClassificationReceipt, ApiError>>(
                    &env,
                    agent.aaa,
                    agent.operator,
                    "submit_classification",
                    (submission,),
                )
            }) {
                Ok((Ok(receipt), s)) => {
                    meter.record(label, s);
                    classifications += 1;
                    flagged += receipt.discovery_id.is_some() as u64;
                }
                _ => meter.error(label),
            }

            if t < calibration && (t + 1) % 10 != 0 {
                continue;
            }
            let assigned = meter.measure(agent.aaa, params.fee_get_review, || {
                call::<_, Result<Option<ReviewAssignment>, ApiError>>(
                    &env,
                    agent.aaa,
                    agent.operator,
                    "get_review_assignment",
                    (),
                )
            });
            let assignment = match assigned {
                Ok((Ok(Some(a)), s)) => {
                    meter.record("get_review_assignment_assigned", s);
                    a
                }
                Ok((Ok(None), s)) => {
                    meter.record("get_review_assignment_none", s);
                    continue;
                }
                Ok((Err(_), s)) => {
                    meter.record("get_review_assignment_not_eligible", s);
                    continue;
                }
                Err(_) => {
                    meter.error("get_review_assignment");
                    continue;
                }
            };
            let review = ReviewSubmission {
                assignment_id: assignment.assignment_id,
                vote: Vote::Agree,
                rationale: "load test: reviewed classification for consistency".into(),
                observed_image_sha256: assignment.subject.image_sha256.clone(),
                agent_label: None,
                submitted_by: agent.operator,
            };
            match meter.measure(agent.aaa, params.fee_submit_review, || {
                call::<_, Result<ReviewReceipt, ApiError>>(
                    &env,
                    agent.aaa,
                    agent.operator,
                    "submit_review",
                    (review,),
                )
            }) {
                Ok((Ok(_), s)) => {
                    meter.record("submit_review", s);
                    reviews_completed += 1;
                }
                _ => meter.error("submit_review"),
            }
        }
    }

    stable_by_round.push(stable_and_heap(&env, platform, admin, classifications));
    platform_mem[2] = memory_bytes(&env, platform, admin);
    meter.stats.values_mut().for_each(MethodStats::finish);
    aaa_mem[2] = memory_bytes(&env, agents[0].aaa, agents[0].owner);
    let wall_seconds_total = wall_start.elapsed().as_secs_f64();
    step(&format!(
        "{} {n_aaas}x{n_tasks}: {classifications} classifications, {reviews_completed} reviews in {wall_seconds_total:.1}s",
        placement_name(placement)
    ));

    LoadReport {
        placement: placement_name(placement).into(),
        n_aaas,
        n_tasks_per_aaa: n_tasks,
        total_classifications: classifications,
        total_discoveries_flagged: flagged,
        total_reviews_completed: reviews_completed,
        params: BTreeMap::from([
            ("fee_get_task".into(), params.fee_get_task),
            (
                "fee_submit_classification".into(),
                params.fee_submit_classification,
            ),
            ("fee_get_review".into(), params.fee_get_review),
            ("fee_submit_review".into(), params.fee_submit_review),
            ("calibration_tasks".into(), params.calibration_tasks as u128),
        ]),
        stats: meter.stats,
        platform_memory_bytes: platform_mem,
        platform_stable_by_round: stable_by_round,
        sample_aaa_memory_bytes: aaa_mem,
        classifications_at_mid,
        wall_seconds_total,
    }
}

fn placement_name(p: Placement) -> &'static str {
    match p {
        Placement::App => "app13",
        Placement::Fiduciary => "fiduciary34",
    }
}

#[derive(candid::CandidType, Deserialize, Serialize)]
struct SubnetCosts {
    create_canister: u128,
    call_empty: u128,
    call_per_kib: u128,
}

#[derive(Serialize)]
struct PricingProbe {
    placement: String,
    costs: SubnetCosts,
    ingress_update_base_cycles: f64,
    cycles_per_instruction: f64,
}

fn pricing_probe(placement: Placement) -> PricingProbe {
    let env = make_env();
    let admin = ident(1);
    let probe = install(&env, "spike-probe", admin, placement);
    let costs: SubnetCosts = env.update(probe, admin, "subnet_costs", ());
    let mut points = Vec::new();
    for rounds in [1_000u64, 10_000_000] {
        let before = env.pic.cycle_balance(probe);
        let instructions: u64 = env.update(probe, admin, "burn", rounds);
        let spent = before - env.pic.cycle_balance(probe);
        points.push((instructions as f64, spent as f64));
    }
    let slope = (points[1].1 - points[0].1) / (points[1].0 - points[0].0);
    PricingProbe {
        placement: placement_name(placement).into(),
        costs,
        ingress_update_base_cycles: points[0].1 - slope * points[0].0,
        cycles_per_instruction: slope,
    }
}

fn write_json(key: &str, value: serde_json::Value) {
    let path = repo_root().join("docs/perf/load-test-T7.3.json");
    let mut all: serde_json::Map<String, serde_json::Value> = std::fs::read_to_string(&path)
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_default();
    all.insert(key.into(), value);
    std::fs::write(&path, serde_json::to_string_pretty(&all).unwrap()).unwrap();
    step(&format!("wrote {key} to {}", path.display()));
}

fn run_and_write(n_aaas: usize, n_tasks: usize, placement: Placement) {
    let report = run(n_aaas, n_tasks, placement);
    let key = format!("{}_{n_aaas}x{n_tasks}", report.placement);
    write_json(&key, serde_json::to_value(&report).unwrap());
}

#[test]

fn t7_3_pricing_probe() {
    for placement in [Placement::App, Placement::Fiduciary] {
        let probe = pricing_probe(placement);
        write_json(
            &format!("pricing_{}", probe.placement),
            serde_json::to_value(&probe).unwrap(),
        );
    }
}

#[test]

fn t7_3_load_app_40x70() {
    run_and_write(40, 70, Placement::App);
}

#[test]

fn t7_3_load_multiplier_10x20() {
    run_and_write(10, 20, Placement::App);
    run_and_write(10, 20, Placement::Fiduciary);
}

#[test]

fn t7_2_load_200x100() {
    run_and_write(200, 100, Placement::App);
}

#[test]

fn t7_2_load_20x20() {
    run_and_write(20, 20, Placement::App);
}
