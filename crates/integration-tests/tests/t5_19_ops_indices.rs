use candid::{decode_one, encode_args, encode_one, CandidType, Principal};
use icrc_ledger_types::icrc1::account::Account as Icrc1Account;
use integration_tests::pic::{canister_wasm, user, IcpEnv, E8S};
use integration_tests::step;
use payments::api::{SpawnArgs, TopUpArgs};
use payments::deposit::Purpose;
use payments::journal::{Account, Op, PayPath};
use sc_types::ApiError;
use serde::de::DeserializeOwned;
use sha2::{Digest, Sha256};

fn upload_and_approve_aaa_wasm(env: &IcpEnv, platform: Principal, admin: Principal) {
    let wasm = canister_wasm("aaa");
    let hash = Sha256::digest(&wasm).to_vec();
    let bytes = env
        .pic
        .update_call(
            platform,
            admin,
            "admin_upload_wasm",
            encode_args((1u32, wasm, hash)).unwrap(),
        )
        .expect("admin_upload_wasm");
    let r: Result<(), ApiError> = decode_one(&bytes).unwrap();
    r.expect("upload wasm");
    let ok: Result<(), ApiError> = env.update(platform, admin, "admin_approve_wasm", 1u32);
    ok.expect("approve wasm");
}

fn spawn_aaa(env: &IcpEnv, payments: Principal, caller: Principal, name: &str) -> Principal {
    let bytes = env
        .pic
        .update_call(
            payments,
            caller,
            "spawn_aaa",
            encode_one(SpawnArgs {
                name: name.into(),
                avatar_seed: 1,
                path: PayPath::Deposit,
            })
            .unwrap(),
        )
        .expect("spawn_aaa rejected");
    let r: Result<u64, ApiError> = decode_one(&bytes).unwrap();
    r.expect("spawn_aaa succeeds");
    caller
}

fn fund(
    env: &IcpEnv,
    payments: Principal,
    purpose: Purpose,
    beneficiary: Principal,
    funder: Principal,
    e8s: u64,
) {
    let raw = env
        .pic
        .query_call(
            payments,
            beneficiary,
            "get_deposit_account",
            encode_args((purpose, beneficiary)).unwrap(),
        )
        .expect("get_deposit_account");
    let (_, account): (String, Account) = candid::decode_args(&raw).unwrap();
    let icrc1 = Icrc1Account {
        owner: account.owner,
        subaccount: account.subaccount,
    };
    env.icrc1_transfer(funder, icrc1, e8s, None)
        .expect("fund deposit account");
}

fn top_up(env: &IcpEnv, payments: Principal, caller: Principal, aaa: Principal) -> u64 {
    let bytes = env
        .pic
        .update_call(
            payments,
            caller,
            "top_up",
            encode_one(TopUpArgs {
                aaa,
                path: PayPath::Deposit,
            })
            .unwrap(),
        )
        .expect("top_up rejected");
    let r: Result<u64, ApiError> = decode_one(&bytes).unwrap();
    r.expect("top_up succeeds")
}

fn query3<A: candid::utils::ArgumentEncoder, R: DeserializeOwned + CandidType>(
    env: &IcpEnv,
    canister: Principal,
    sender: Principal,
    method: &str,
    args: A,
) -> R {
    let bytes = env
        .pic
        .query_call(canister, sender, method, encode_args(args).unwrap())
        .unwrap_or_else(|e| panic!("{method} rejected: {e:?}"));
    decode_one(&bytes).unwrap_or_else(|e| panic!("{method} reply did not decode: {e}"))
}

#[test]
fn t5_19_list_ops_for_aaa_and_owner_page_and_isolate_between_two_aaas() {
    println!("T5.19 demo: list_ops_for_aaa / list_ops_for_owner page newest-first and isolate between AAAs");
    let env = IcpEnv::new();
    let admin = user(1);
    let alice = user(2);

    let platform = env.install("platform", admin);
    let payments = env.install("payments", admin);
    for _ in 0..5 {
        env.pic.tick();
    }
    let ok: Result<(), ApiError> = env.update(platform, admin, "admin_set_payments_id", payments);
    assert_eq!(ok, Ok(()));
    let ok: Result<(), ApiError> = env.update(payments, admin, "admin_set_platform_id", platform);
    assert_eq!(ok, Ok(()));
    upload_and_approve_aaa_wasm(&env, platform, admin);
    step("installed platform + payments, wired peer ids, approved the AAA wasm");

    env.mint_icp(alice, 20 * E8S);

    let owner1 = user(10);
    fund(&env, payments, Purpose::Spawn, owner1, alice, E8S);
    spawn_aaa(&env, payments, owner1, "OrionSurveyor-01");
    let aaa1: Option<Principal> = env.query(platform, admin, "aaa_by_owner", owner1);
    let aaa1 = aaa1.expect("owner1's AAA registered");

    let owner2 = user(11);
    fund(&env, payments, Purpose::Spawn, owner2, alice, E8S);
    spawn_aaa(&env, payments, owner2, "OrionSurveyor-02");
    let aaa2: Option<Principal> = env.query(platform, admin, "aaa_by_owner", owner2);
    let aaa2 = aaa2.expect("owner2's AAA registered");
    step(&format!("spawned two real AAAs: aaa1={aaa1}, aaa2={aaa2}"));

    let mut aaa1_topup_ids = Vec::new();
    for _ in 0..3 {
        fund(&env, payments, Purpose::TopUp, aaa1, alice, 20_000_000);
        aaa1_topup_ids.push(top_up(&env, payments, owner1, aaa1));
    }
    fund(&env, payments, Purpose::TopUp, aaa2, alice, 20_000_000);
    let aaa2_topup_id = top_up(&env, payments, owner2, aaa2);
    step("aaa1 topped up 3 times, aaa2 topped up once, all reaching Done");

    let page1: payments::journal::Page<Op> = query3(
        &env,
        payments,
        owner1,
        "list_ops_for_aaa",
        (aaa1, None::<u64>, 2u16),
    );
    assert_eq!(page1.items.len(), 2);
    assert_eq!(
        page1.items.iter().map(|o| o.id).collect::<Vec<_>>(),
        vec![aaa1_topup_ids[2], aaa1_topup_ids[1]],
        "newest first"
    );
    assert!(page1.next_cursor.is_some());

    let page2: payments::journal::Page<Op> = query3(
        &env,
        payments,
        owner1,
        "list_ops_for_aaa",
        (aaa1, page1.next_cursor, 2u16),
    );
    assert_eq!(page2.items.len(), 1);
    assert_eq!(page2.items[0].id, aaa1_topup_ids[0]);
    assert_eq!(page2.next_cursor, None);
    step("list_ops_for_aaa(aaa1) pages newest-first across two pages");

    let aaa2_page: payments::journal::Page<Op> = query3(
        &env,
        payments,
        owner2,
        "list_ops_for_aaa",
        (aaa2, None::<u64>, 100u16),
    );
    assert_eq!(aaa2_page.items.len(), 1);
    assert_eq!(aaa2_page.items[0].id, aaa2_topup_id);
    assert!(aaa2_page
        .items
        .iter()
        .all(|o| !aaa1_topup_ids.contains(&o.id)));
    step("list_ops_for_aaa(aaa2) sees only its own op, isolated from aaa1's ops");

    let owner1_page: payments::journal::Page<Op> = query3(
        &env,
        payments,
        owner1,
        "list_ops_for_owner",
        (owner1, None::<u64>, 100u16),
    );
    assert_eq!(owner1_page.items.len(), 4, "1 spawn + 3 top-ups for owner1");

    let owner2_page: payments::journal::Page<Op> = query3(
        &env,
        payments,
        owner2,
        "list_ops_for_owner",
        (owner2, None::<u64>, 100u16),
    );
    assert_eq!(owner2_page.items.len(), 2, "1 spawn + 1 top-up for owner2");
    let owner1_ids: Vec<u64> = owner1_page.items.iter().map(|o| o.id).collect();
    assert!(owner2_page
        .items
        .iter()
        .all(|o| !owner1_ids.contains(&o.id)));
    step("list_ops_for_owner isolates owner1's ops from owner2's ops");
}
