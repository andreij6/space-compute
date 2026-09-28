use candid::{encode_args, encode_one, Principal};
use integration_tests::pic::{canister_wasm, user, IcpEnv, E8S};
use integration_tests::step;
use sc_types::ApiError;
use treasury::config::Config;
use treasury::report::{Health, Status};
use treasury::state::HistoryItem;

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
            encode_args((None::<u64>, 200u32)).unwrap(),
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
fn t7_1_treasury_v_n_minus_1_state_survives_upgrade_to_v_n() {
    println!("T7.1 demo: treasury installed at the pinned vN-1 baseline, populated, upgraded to the current vN wasm — watch list, history, proposals and the keeper timer all survive");
    let env = IcpEnv::new();
    let (alice, bob) = (user(1), user(2));
    let treasury = env.install_baseline("treasury", alice);
    step("installed treasury at the pinned vN-1 baseline commit");

    env.mint_icp(treasury, 100 * E8S);
    let mut cfg: Config = env
        .query::<_, Result<Config, ApiError>>(treasury, alice, "admin_get_config", ())
        .unwrap();
    cfg.min_balance_cycles = 110_000_000_000_000;
    let ok: Result<Option<u64>, ApiError> =
        env.update(treasury, alice, "admin_set_config", cfg.clone());
    assert_eq!(ok, Ok(None));

    let low = watched_canister(&env, treasury, alice, 0);
    let healthy = watched_canister(&env, treasury, alice, 200_000_000_000_000);
    for (c, prio) in [(low, 0u8), (healthy, 1u8)] {
        let ok: Result<(), ApiError> = call(&env, treasury, alice, "admin_watch", (c, prio, 60u32));
        assert_eq!(ok, Ok(()));
    }
    topup_now(&env, treasury, alice);
    step("funded 100 ICP, watching a low and a healthy canister, keeper ran once (topped up the low one)");

    let id: Result<u64, ApiError> = call(&env, treasury, alice, "admin_withdraw", (bob, E8S));
    let withdraw_id = id.unwrap();
    let solo: Result<Option<u64>, ApiError> =
        env.update(treasury, alice, "admin_approve", withdraw_id);
    assert!(matches!(solo, Err(ApiError::NotEligible(_))));

    cfg.admins = vec![alice, bob];
    let ok: Result<Option<u64>, ApiError> =
        env.update(treasury, alice, "admin_set_config", cfg.clone());
    assert_eq!(ok, Ok(None));
    let approved: Result<Option<u64>, ApiError> =
        env.update(treasury, bob, "admin_approve", withdraw_id);
    assert!(matches!(approved, Ok(Some(_))));
    assert_eq!(env.icp_balance(bob), E8S);

    let reserve_before = cfg.reserve_e8s;
    cfg.reserve_e8s = 0;
    let proposed: Result<Option<u64>, ApiError> =
        env.update(treasury, alice, "admin_set_config", cfg.clone());
    let config_proposal_id = proposed.unwrap().expect("lowering the reserve proposes");
    step("a pending config-change proposal is on the books, and one withdraw has already executed");

    let admins_before = cfg.admins.clone();
    let history_before = history(&env, treasury);
    let watch_before: Status = env.query(treasury, Principal::anonymous(), "status", ());
    assert_eq!(watch_before.canisters.len(), 2);
    let checked_before = watch_before.checked_at;
    let bob_balance_before = env.icp_balance(bob);

    env.pic
        .upgrade_canister(
            treasury,
            canister_wasm("treasury"),
            encode_one(()).unwrap(),
            Some(alice),
        )
        .expect("upgrade vN-1 -> vN must succeed");
    for _ in 0..5 {
        env.pic.tick();
    }
    step("upgraded treasury from the vN-1 baseline to the current vN wasm");

    let after_upgrade = history(&env, treasury);
    assert_eq!(after_upgrade.len(), history_before.len());
    assert_eq!(after_upgrade, history_before);
    let status_after: Status = env.query(treasury, Principal::anonymous(), "status", ());
    assert_eq!(status_after.canisters.len(), 2);
    let current: Config = env
        .query::<_, Result<Config, ApiError>>(treasury, alice, "admin_get_config", ())
        .unwrap();
    assert_eq!(current.admins, admins_before);
    assert_eq!(
        current.reserve_e8s, reserve_before,
        "the reserve-lowering proposal must not have applied itself"
    );
    step("history, watch list and admin set are intact after the upgrade");

    let applied: Result<Option<u64>, ApiError> =
        env.update(treasury, bob, "admin_approve", config_proposal_id);
    assert_eq!(applied, Ok(None));
    let current: Config = env
        .query::<_, Result<Config, ApiError>>(treasury, alice, "admin_get_config", ())
        .unwrap();
    assert_eq!(current.reserve_e8s, 0);
    step("the pre-upgrade proposal (id survived the upgrade) still approves and applies");

    let stale = env.icp_balance(bob);
    let replay: Result<Option<u64>, ApiError> =
        env.update(treasury, bob, "admin_approve", withdraw_id);
    assert_eq!(replay, Err(ApiError::NotFound));
    assert_eq!(env.icp_balance(bob), stale);
    assert_eq!(bob_balance_before, stale);
    step("the already-executed withdraw cannot be re-approved after the upgrade");

    env.pic
        .advance_time(std::time::Duration::from_secs(6 * 3_600 + 60));
    for _ in 0..20 {
        env.pic.tick();
    }
    let status_ticked: Status = env.query(treasury, Principal::anonymous(), "status", ());
    assert!(
        status_ticked.checked_at > checked_before,
        "the 6h keeper timer must resume after post_upgrade re-registers it"
    );
    step(&format!(
        "keeper timer resumed after upgrade: checked_at {} -> {}",
        checked_before, status_ticked.checked_at
    ));
}
