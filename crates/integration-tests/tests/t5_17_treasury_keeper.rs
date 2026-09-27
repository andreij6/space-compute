use candid::{encode_args, encode_one, Principal};
use integration_tests::pic::{canister_wasm, user, IcpEnv, E8S};
use integration_tests::step;
use sc_types::ApiError;
use treasury::config::Config;
use treasury::report::{Health, Status};
use treasury::state::{HistoryItem, HistoryKind};

const T: u128 = 1_000_000_000_000;

fn call<A: candid::utils::ArgumentEncoder, R: candid::CandidType + serde::de::DeserializeOwned>(
    env: &IcpEnv,
    canister: Principal,
    sender: Principal,
    method: &str,
    args: A,
) -> R {
    let raw = env
        .pic
        .update_call(canister, sender, method, encode_args(args).unwrap())
        .unwrap_or_else(|e| panic!("{method} rejected: {e:?}"));
    candid::decode_one(&raw).unwrap()
}

fn history(env: &IcpEnv, treasury: Principal) -> Vec<HistoryItem> {
    let raw = env
        .pic
        .query_call(
            treasury,
            Principal::anonymous(),
            "history",
            encode_args((None::<u64>, 100u32)).unwrap(),
        )
        .expect("history");
    let (items, _): (Vec<HistoryItem>, Option<u64>) = candid::decode_args(&raw).unwrap();
    items
}

fn watched_canister(
    env: &IcpEnv,
    treasury: Principal,
    owner: Principal,
    cycles: u128,
) -> Principal {
    let subnet = env.pic.topology().get_app_subnets()[0];
    let id = env.pic.create_canister_on_subnet(Some(owner), None, subnet);
    env.pic.add_cycles(id, cycles);
    env.pic.install_canister(
        id,
        canister_wasm("spike-probe"),
        encode_one(()).unwrap(),
        Some(owner),
    );
    env.pic
        .set_controllers(id, Some(owner), vec![owner, treasury])
        .unwrap();
    id
}

fn topup_now(env: &IcpEnv, treasury: Principal, admin: Principal) -> Health {
    let h: Result<Health, ApiError> =
        env.update(treasury, admin, "admin_topup_now", None::<Principal>);
    h.unwrap()
}

#[test]
fn t5_17_keeper_tops_up_low_canisters_and_health_flags_the_reserve() {
    println!(
        "T5.17 demo: owner-funded treasury keeps app canisters topped up (real ICP ledger + CMC)"
    );
    let env = IcpEnv::new();
    let (alice, bob) = (user(1), user(2));
    let treasury = env.install("treasury", alice);

    env.mint_icp(treasury, 100 * E8S);
    step("owner funded the treasury with 100 ICP");

    let mut cfg: Config = env
        .query::<_, Result<Config, ApiError>>(treasury, alice, "admin_get_config", ())
        .unwrap();
    cfg.min_balance_cycles = 110 * T;
    let ok: Result<Option<u64>, ApiError> =
        env.update(treasury, alice, "admin_set_config", cfg.clone());
    assert_eq!(ok, Ok(None));

    let low = watched_canister(&env, treasury, alice, 0);
    let healthy = watched_canister(&env, treasury, alice, 100 * T);
    for (c, prio) in [(low, 0u8), (healthy, 1u8)] {
        let ok: Result<(), ApiError> = call(&env, treasury, alice, "admin_watch", (c, prio, 60u32));
        assert_eq!(ok, Ok(()));
    }
    let stranger: Result<(), ApiError> =
        call(&env, treasury, bob, "admin_watch", (low, 0u8, 60u32));
    assert_eq!(stranger, Err(ApiError::Unauthorized));
    step("floor 110T; watching a low canister (~100T) and a healthy one (~200T); non-admins are refused");

    let before = env.pic.cycle_balance(low);
    let health = topup_now(&env, treasury, alice);
    let after = env.pic.cycle_balance(low);
    assert!(
        after >= 110 * T,
        "low canister should be topped up to the 110T floor: {before} → {after}"
    );
    assert!(!health.reserve_breached);
    let log = history(&env, treasury);
    let topped: Vec<_> = log
        .iter()
        .filter_map(|h| match &h.kind {
            HistoryKind::TopUp { canister, .. } => Some(*canister),
            _ => None,
        })
        .collect();
    assert_eq!(topped, vec![low]);
    step(&format!(
        "keeper topped up the low canister {:.2}T → {:.2}T via ledger→CMC; healthy canister skipped",
        before as f64 / 1e12,
        after as f64 / 1e12
    ));

    let status: Status = env.query(treasury, Principal::anonymous(), "status", ());
    assert!(status.icp_balance_e8s < 100 * E8S && status.icp_balance_e8s > 0);
    assert_eq!(status.canisters.len(), 2);
    step(&format!(
        "public status(): {:.4} ICP left, rate {} XDR‱/ICP, {} watched canisters",
        status.icp_balance_e8s as f64 / E8S as f64,
        status.xdr_permyriad_per_icp,
        status.canisters.len()
    ));

    cfg.reserve_e8s = status.icp_balance_e8s - E8S / 100;
    cfg.min_balance_cycles = 900 * T;
    let ok: Result<Option<u64>, ApiError> =
        env.update(treasury, alice, "admin_set_config", cfg.clone());
    assert_eq!(ok, Ok(None));
    let icp_before = env.icp_balance(treasury);
    let health = topup_now(&env, treasury, alice);
    assert_eq!(env.icp_balance(treasury), icp_before);
    assert!(health.reserve_breached);
    assert!(history(&env, treasury).iter().any(
        |h| matches!(h.kind, HistoryKind::SkippedReserve { canister, .. } if canister == low)
    ));
    step(
        "with the reserve raised to the balance, top-ups stop and health().reserve_breached = true",
    );
}
