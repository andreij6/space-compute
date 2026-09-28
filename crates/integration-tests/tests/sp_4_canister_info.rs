use candid::{decode_one, encode_one, CandidType, Principal};
use integration_tests::pic::{user, IcpEnv};
use integration_tests::step;
use serde::Deserialize;

use aaa::config::AaaInit;

fn aaa_init(owner: Principal) -> AaaInit {
    AaaInit {
        owner,
        platform_id: user(90),
        payments_id: user(91),
        name: "Rover One".into(),
        avatar_seed: 1,
    }
}

#[derive(CandidType, Deserialize, Debug)]
struct InfoProbe {
    total_num_changes: u64,
    module_hash_len: u32,
    controllers: u32,
    cycles_spent: u64,
    instructions: u64,
}

fn rounds_until_done(
    env: &IcpEnv,
    canister: Principal,
    method: &str,
    arg: Vec<u8>,
) -> (u32, Vec<u8>) {
    let id = env
        .pic
        .submit_call(canister, user(1), method, arg)
        .expect("submit");
    for round in 1..=100 {
        env.pic.tick();
        if let Some(r) = env.pic.ingress_status(id.clone()) {
            return (round, r.expect("call succeeded"));
        }
    }
    panic!("{method} did not finish in 100 rounds");
}

#[test]
fn sp_4_canister_info_cost_and_latency_same_vs_cross_subnet() {
    println!("SP-4 demo: what does a canister_info provenance check cost per submission?");
    let env = IcpEnv::with_app_subnets(2);
    let owner = user(1);
    let probe = env.install_on("spike-probe", owner, 10_000_000_000_000, 0);
    let local_aaa = env.install_on_with_arg("aaa", owner, 1_000_000_000_000, 0, aaa_init(owner));
    let remote_aaa = env.install_on_with_arg("aaa", owner, 1_000_000_000_000, 1, aaa_init(owner));

    let (baseline, _) = rounds_until_done(&env, local_aaa, "version", encode_one(()).unwrap());
    step(&format!(
        "baseline: a plain update call completes in {baseline} round(s)"
    ));

    for (label, target) in [("same subnet", local_aaa), ("cross subnet", remote_aaa)] {
        let (rounds, bytes) = rounds_until_done(
            &env,
            probe,
            "probe_canister_info",
            encode_one(target).unwrap(),
        );
        let r: Result<InfoProbe, String> = decode_one(&bytes).unwrap();
        let p = r.expect("canister_info");
        assert_eq!(p.module_hash_len, 32);
        step(&format!(
            "{label}: canister_info took {rounds} rounds (+{} vs baseline), cost {} cycles (~${:.6}), {} instructions; total_num_changes={}, controllers={}",
            rounds - baseline,
            p.cycles_spent,
            p.cycles_spent as f64 / 1e12 * 1.35,
            p.instructions,
            p.total_num_changes,
            p.controllers
        ));
        assert!(
            p.cycles_spent < 50_000_000,
            "canister_info should cost well under the 50M submission fee"
        );
    }
    step(
        "fee_submit_classification is 50_000_000 cycles (T7.3); the ~5.9M check is priced into it",
    );
}
