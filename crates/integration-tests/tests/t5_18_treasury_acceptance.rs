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
fn t5_18_withdraw_needs_two_admins_and_state_survives_upgrade() {
    println!("T5.18 demo: treasury acceptance 12 §5 #5-#6 (two-admin withdraw, upgrade)");
    let env = IcpEnv::new();
    let (alice, bob) = (user(1), user(2));
    let treasury = env.install("treasury", alice);
    env.mint_icp(treasury, 100 * E8S);
    let mut cfg: Config = env
        .query::<_, Result<Config, ApiError>>(treasury, alice, "admin_get_config", ())
        .unwrap();
    let low = watched_canister(&env, treasury, alice, 0);
    let ok: Result<(), ApiError> = call(&env, treasury, alice, "admin_watch", (low, 0u8, 60u32));
    assert_eq!(ok, Ok(()));
    topup_now(&env, treasury, alice);
    step("treasury funded with 100 ICP, one canister watched, keeper ran once");

    let id: Result<u64, ApiError> = call(&env, treasury, alice, "admin_withdraw", (bob, E8S));
    let id = id.unwrap();
    let solo: Result<Option<u64>, ApiError> = env.update(treasury, alice, "admin_approve", id);
    assert!(matches!(solo, Err(ApiError::NotEligible(_))));
    assert_eq!(env.icp_balance(bob), 0);
    step("admin_withdraw proposed by alice does not execute on her own approval");

    cfg.admins = vec![alice, bob];
    let ok: Result<Option<u64>, ApiError> =
        env.update(treasury, alice, "admin_set_config", cfg.clone());
    assert_eq!(ok, Ok(None));
    let block: Result<Option<u64>, ApiError> = env.update(treasury, bob, "admin_approve", id);
    assert!(matches!(block, Ok(Some(_))));
    assert_eq!(env.icp_balance(bob), E8S);
    step("a second admin (bob) approved within 24 h → 1 ICP withdrawn");

    let reserve = cfg.reserve_e8s;
    cfg.reserve_e8s = 0;
    let proposed: Result<Option<u64>, ApiError> =
        env.update(treasury, alice, "admin_set_config", cfg);
    let pid = proposed
        .unwrap()
        .expect("lowering the reserve needs a proposal");
    let current: Config = env
        .query::<_, Result<Config, ApiError>>(treasury, alice, "admin_get_config", ())
        .unwrap();
    assert_eq!(current.reserve_e8s, reserve);
    let applied: Result<Option<u64>, ApiError> = env.update(treasury, bob, "admin_approve", pid);
    assert_eq!(applied, Ok(None));
    let current: Config = env
        .query::<_, Result<Config, ApiError>>(treasury, alice, "admin_get_config", ())
        .unwrap();
    assert_eq!(current.reserve_e8s, 0);
    step(
        "lowering the reserve by one admin only creates a proposal; it applies after bob approves",
    );

    let entries = history(&env, treasury).len();
    env.pic
        .upgrade_canister(
            treasury,
            canister_wasm("treasury"),
            encode_one(()).unwrap(),
            Some(alice),
        )
        .expect("upgrade");
    let after_upgrade = history(&env, treasury);
    assert_eq!(after_upgrade.len(), entries);
    let status: Status = env.query(treasury, Principal::anonymous(), "status", ());
    assert_eq!(status.canisters.len(), 1);
    let checked = status.checked_at;
    env.pic
        .advance_time(std::time::Duration::from_secs(6 * 3_600 + 60));
    for _ in 0..20 {
        env.pic.tick();
    }
    let status: Status = env.query(treasury, Principal::anonymous(), "status", ());
    assert!(status.checked_at > checked);
    step(&format!(
        "after upgrade: {} history items and the watch list intact; the 6 h timer resumed",
        entries
    ));
}
