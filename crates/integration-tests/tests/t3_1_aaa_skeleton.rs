use candid::utils::ArgumentEncoder;
use candid::{decode_one, encode_args, encode_one, CandidType, Principal};
use serde::de::DeserializeOwned;

use integration_tests::pic::{canister_wasm, user, IcpEnv};
use integration_tests::step;

use aaa::config::AaaInit;
use aaa::operators::Operator;
use aaa::roles::Role;
use sc_types::ApiError;

type Unit = Result<(), ApiError>;

fn init_arg(owner: Principal, platform_id: Principal, payments_id: Principal) -> AaaInit {
    AaaInit {
        owner,
        platform_id,
        payments_id,
        name: "Rover One".into(),
        avatar_seed: 7,
    }
}

fn call<A: ArgumentEncoder, R: DeserializeOwned + CandidType>(
    env: &IcpEnv,
    aaa: Principal,
    sender: Principal,
    method: &str,
    args: A,
) -> R {
    let bytes = env
        .pic
        .update_call(aaa, sender, method, encode_args(args).unwrap())
        .unwrap_or_else(|e| panic!("{method} rejected: {e:?}"));
    decode_one(&bytes).unwrap_or_else(|e| panic!("{method} reply did not decode: {e}"))
}

fn query<A: ArgumentEncoder, R: DeserializeOwned + CandidType>(
    env: &IcpEnv,
    aaa: Principal,
    sender: Principal,
    method: &str,
    args: A,
) -> R {
    let bytes = env
        .pic
        .query_call(aaa, sender, method, encode_args(args).unwrap())
        .unwrap_or_else(|e| panic!("{method} rejected: {e:?}"));
    decode_one(&bytes).unwrap_or_else(|e| panic!("{method} reply did not decode: {e}"))
}

fn ingress_rejected<A: ArgumentEncoder>(
    env: &IcpEnv,
    aaa: Principal,
    sender: Principal,
    method: &str,
    args: A,
) {
    let result = env
        .pic
        .update_call(aaa, sender, method, encode_args(args).unwrap());
    assert!(
        result.is_err(),
        "inspect_message should have rejected ingress to {method} from {sender}"
    );
}

fn whoami(env: &IcpEnv, aaa: Principal, sender: Principal) -> Role {
    query(env, aaa, sender, "whoami", ())
}

#[test]
fn t3_1_aaa_role_matrix_owner_operator_platform_anon_stranger() {
    println!("T3.1 demo: AAA skeleton — roles, operators, config, inspect_message");
    let env = IcpEnv::new();
    let (owner, platform_id, payments_id, operator, expiring, stranger) =
        (user(1), user(2), user(3), user(4), user(5), user(6));

    let aaa = env.install_with_arg("aaa", owner, init_arg(owner, platform_id, payments_id));
    step("installed the aaa canister with owner, platform_id and payments_id");

    assert_eq!(whoami(&env, aaa, owner), Role::Owner);
    assert_eq!(whoami(&env, aaa, platform_id), Role::Platform);
    assert_eq!(whoami(&env, aaa, stranger), Role::None);
    assert_eq!(whoami(&env, aaa, Principal::anonymous()), Role::None);
    step("whoami reports Owner / Platform / None correctly (queries bypass inspect_message)");

    let ok: Unit = call(
        &env,
        aaa,
        owner,
        "add_operator",
        (operator, "bot-1", None::<u64>),
    );
    assert_eq!(ok, Ok(()));
    assert_eq!(whoami(&env, aaa, operator), Role::Operator);
    step("owner adds an operator; whoami now reports Operator for it");

    ingress_rejected(
        &env,
        aaa,
        stranger,
        "add_operator",
        (stranger, "x", None::<u64>),
    );
    ingress_rejected(
        &env,
        aaa,
        Principal::anonymous(),
        "add_operator",
        (stranger, "x", None::<u64>),
    );
    ingress_rejected(
        &env,
        aaa,
        platform_id,
        "add_operator",
        (stranger, "x", None::<u64>),
    );
    step("inspect_message rejects ingress from a stranger, anonymous, and the platform principal");

    let denied: Unit = call(
        &env,
        aaa,
        operator,
        "add_operator",
        (stranger, "x", None::<u64>),
    );
    assert_eq!(
        denied,
        Err(ApiError::Unauthorized),
        "an operator passes inspect_message but is refused by the explicit owner-only check"
    );
    step("an operator reaches add_operator (passes inspect_message) but is refused: owner-only");

    let now_ns = env.pic.get_time().as_nanos_since_unix_epoch();
    let expires_at = now_ns + 60_000_000_000;
    let ok: Unit = call(
        &env,
        aaa,
        owner,
        "add_operator",
        (expiring, "bot-2", Some(expires_at)),
    );
    assert_eq!(ok, Ok(()));
    assert_eq!(whoami(&env, aaa, expiring), Role::Operator);
    step("owner adds a second operator with a 60s expiry; it starts out active");

    env.pic.advance_time(std::time::Duration::from_secs(120));
    env.pic.tick();
    assert_eq!(whoami(&env, aaa, expiring), Role::None);
    ingress_rejected(
        &env,
        aaa,
        expiring,
        "add_operator",
        (stranger, "x", None::<u64>),
    );
    step("after the expiry passes, the operator is treated as no one and ingress is rejected");

    let list: Result<Vec<(Principal, Operator)>, ApiError> =
        query(&env, aaa, owner, "list_operators", ());
    let list = list.unwrap();
    assert_eq!(list.len(), 2);
    step(&format!(
        "list_operators (owner-only) shows {} operators",
        list.len()
    ));

    let denied: Result<Vec<(Principal, Operator)>, ApiError> =
        query(&env, aaa, stranger, "list_operators", ());
    assert_eq!(denied, Err(ApiError::Unauthorized));
    step("list_operators refuses a stranger query with Unauthorized");

    let ok: Unit = call(&env, aaa, owner, "remove_operator", (operator,));
    assert_eq!(ok, Ok(()));
    assert_eq!(whoami(&env, aaa, operator), Role::None);
    ingress_rejected(
        &env,
        aaa,
        operator,
        "add_operator",
        (stranger, "x", None::<u64>),
    );
    step("owner removes an operator; it is rejected immediately, both by role and by ingress");

    let ok: Unit = call(&env, aaa, owner, "set_agent_label", (Some("claude-code"),));
    assert_eq!(ok, Ok(()));
    let bad: Unit = call(&env, aaa, owner, "set_auto_topup", (Some(0u128),));
    assert!(matches!(bad, Err(ApiError::InvalidInput(_))));
    let ok: Unit = call(
        &env,
        aaa,
        owner,
        "set_auto_topup",
        (Some(5_000_000_000u128),),
    );
    assert_eq!(ok, Ok(()));
    step("owner sets agent_label and a valid auto_topup threshold; a zero threshold is rejected");

    env.pic
        .upgrade_canister(
            aaa,
            canister_wasm("aaa"),
            encode_one(()).unwrap(),
            Some(owner),
        )
        .expect("upgrade");
    step("upgraded the aaa canister");

    let list: Result<Vec<(Principal, Operator)>, ApiError> =
        query(&env, aaa, owner, "list_operators", ());
    assert_eq!(list.unwrap().len(), 1);
    assert_eq!(whoami(&env, aaa, owner), Role::Owner);
    let ok: Unit = call(&env, aaa, owner, "set_agent_label", (None::<String>,));
    assert_eq!(ok, Ok(()));
    step("after upgrade: config and the remaining operator survive intact");
}
