use candid::{decode_one, encode_args, CandidType, Nat, Principal};
use icrc_ledger_types::icrc1::account::Account as Icrc1Account;
use integration_tests::pic::{user, IcpEnv, E8S};
use integration_tests::step;
use payments::api::{PaymentsOverview, TreasuryWithdrawArgs};
use payments::journal::{Account, OpFilter, OpKind, OpState, Page};
use sc_types::ApiError;
use serde::de::DeserializeOwned;

fn balance_of(env: &IcpEnv, account: Icrc1Account) -> u64 {
    let n: Nat = env.query(
        integration_tests::pic::LEDGER,
        Principal::anonymous(),
        "icrc1_balance_of",
        account,
    );
    n.0.try_into().unwrap()
}

fn query_multi<A: candid::utils::ArgumentEncoder, R: DeserializeOwned + CandidType>(
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

fn get_treasury_account(env: &IcpEnv, payments: Principal, admin: Principal) -> Account {
    let bytes = env
        .pic
        .query_call(
            payments,
            admin,
            "get_treasury_account",
            encode_args(()).unwrap(),
        )
        .expect("get_treasury_account");
    let (_, account): (String, Account) = candid::decode_args(&bytes).unwrap();
    account
}

#[test]
fn t5_15_admin_overview_icp_balance_matches_a_real_icrc1_balance_query() {
    println!("T5.15 demo: admin_overview's ICP balance matches icrc1_balance_of after real ops");
    let env = IcpEnv::new();
    let admin = user(1);
    let payments = env.install("payments", admin);
    step("installed payments, admin is the installer");

    let funder = user(2);
    env.mint_icp(funder, 20 * E8S);
    let treasury_account = get_treasury_account(&env, payments, admin);
    let treasury_icrc1 = Icrc1Account {
        owner: treasury_account.owner,
        subaccount: treasury_account.subaccount,
    };
    env.icrc1_transfer(funder, treasury_icrc1, 5 * E8S, None)
        .expect("fund the treasury subaccount directly, a real ICP ledger op");
    step("sent 5 ICP straight into the payments TREASURY subaccount on the real ledger");

    let overview: Result<PaymentsOverview, ApiError> =
        env.update(payments, admin, "admin_overview", ());
    let overview = overview.expect("admin_overview");

    let real_treasury_balance = balance_of(&env, treasury_icrc1);
    assert_eq!(overview.treasury_icp_balance_e8s, real_treasury_balance);
    assert_eq!(overview.treasury_icp_balance_e8s, 5 * E8S);

    let main_icrc1 = Icrc1Account {
        owner: payments,
        subaccount: None,
    };
    let real_main_balance = balance_of(&env, main_icrc1);
    assert_eq!(overview.main_icp_balance_e8s, real_main_balance);
    step(&format!(
        "admin_overview reports treasury={} main={} e8s, both matching direct icrc1_balance_of queries",
        overview.treasury_icp_balance_e8s, overview.main_icp_balance_e8s
    ));
}

#[test]
fn t5_15_admin_treasury_withdraw_moves_icp_and_is_admin_gated() {
    println!("T5.15 demo: admin_treasury_withdraw moves real ICP and is team-only");
    let env = IcpEnv::new();
    let admin = user(1);
    let stranger = user(9);
    let payments = env.install("payments", admin);

    let funder = user(2);
    env.mint_icp(funder, 20 * E8S);
    let treasury_account = get_treasury_account(&env, payments, admin);
    let treasury_icrc1 = Icrc1Account {
        owner: treasury_account.owner,
        subaccount: treasury_account.subaccount,
    };
    env.icrc1_transfer(funder, treasury_icrc1, 5 * E8S, None)
        .expect("fund the treasury");
    step("funded the TREASURY subaccount with 5 ICP");

    let dest = user(3);
    let dest_account = Account {
        owner: dest,
        subaccount: None,
    };
    let dest_icrc1 = Icrc1Account {
        owner: dest,
        subaccount: None,
    };

    let denied: Result<u64, ApiError> = env.update(
        payments,
        stranger,
        "admin_treasury_withdraw",
        TreasuryWithdrawArgs {
            to: dest_account.clone(),
            amount: Nat::from(E8S),
            created_at_time: None,
        },
    );
    assert_eq!(denied, Err(ApiError::Unauthorized));
    step("a non-admin's admin_treasury_withdraw is rejected with Unauthorized");
    assert_eq!(balance_of(&env, dest_icrc1), 0);

    let before_treasury = balance_of(&env, treasury_icrc1);
    let ok: Result<u64, ApiError> = env.update(
        payments,
        admin,
        "admin_treasury_withdraw",
        TreasuryWithdrawArgs {
            to: dest_account,
            amount: Nat::from(2 * E8S),
            created_at_time: None,
        },
    );
    ok.expect("admin_treasury_withdraw by the admin succeeds");
    step("the admin withdrew 2 ICP from the treasury to a destination account");

    let after_treasury = balance_of(&env, treasury_icrc1);
    assert_eq!(before_treasury - after_treasury, 2 * E8S + 10_000);
    assert_eq!(balance_of(&env, dest_icrc1), 2 * E8S);
    step(&format!(
        "treasury dropped by {} e8s (amount + ledger fee), destination received {} e8s — a real icrc1_transfer",
        before_treasury - after_treasury,
        2 * E8S
    ));

    let raw = env
        .pic
        .query_call(
            payments,
            admin,
            "admin_audit_log",
            candid::encode_args((None::<u64>, 50u32)).unwrap(),
        )
        .expect("admin_audit_log");
    let log: Result<Page<payments::audit::AuditEntry>, ApiError> =
        candid::decode_one(&raw).unwrap();
    let methods: Vec<_> = log
        .unwrap()
        .items
        .iter()
        .map(|e| e.method.clone())
        .collect();
    assert!(methods.contains(&"admin_treasury_withdraw".to_string()));
    step("admin_treasury_withdraw is recorded in the audit log");
}

#[test]
fn t5_15_admin_list_ops_filters_by_kind_and_paginates() {
    println!("T5.15 demo: admin_list_ops filters by kind/state/since and paginates");
    let env = IcpEnv::new();
    let admin = user(1);
    let payments = env.install("payments", admin);

    for i in 0..3u8 {
        let aaa = user(10 + i);
        let _: Result<u64, ApiError> = env.update(
            payments,
            admin,
            "admin_journal_demo",
            payments::api::JournalDemoArg {
                aaa,
                trap_after_await: false,
            },
        );
    }
    step("ran three admin_journal_demo ops, all reaching Done");

    let page: Result<Page<payments::journal::Op>, ApiError> = query_multi(
        &env,
        payments,
        admin,
        "admin_list_ops",
        (
            OpFilter {
                kind: Some(OpKind::TopUp {
                    aaa: Principal::anonymous(),
                }),
                state: None,
                since: None,
            },
            None::<u64>,
            1u32,
        ),
    );
    let page = page.expect("admin_list_ops");
    assert_eq!(page.items.len(), 1);
    assert!(matches!(page.items[0].kind, OpKind::TopUp { .. }));
    assert_eq!(page.next_cursor, Some(page.items[0].id + 1));
    step(&format!(
        "first page of TopUp ops returns 1 item with next_cursor={:?}",
        page.next_cursor
    ));

    let page2: Result<Page<payments::journal::Op>, ApiError> = query_multi(
        &env,
        payments,
        admin,
        "admin_list_ops",
        (
            OpFilter {
                kind: Some(OpKind::TopUp {
                    aaa: Principal::anonymous(),
                }),
                state: Some(OpState::Done),
                since: None,
            },
            page.next_cursor,
            10u32,
        ),
    );
    let page2 = page2.expect("admin_list_ops page 2");
    assert_eq!(page2.items.len(), 2);
    assert!(page2.items.iter().all(|op| op.state == OpState::Done));
    assert_eq!(page2.next_cursor, None);
    step("second page (state=Done) returns the remaining 2 ops and no further cursor");

    let denied: Result<Page<payments::journal::Op>, ApiError> = query_multi(
        &env,
        payments,
        user(99),
        "admin_list_ops",
        (OpFilter::default(), None::<u64>, 10u32),
    );
    assert_eq!(denied, Err(ApiError::Unauthorized));
    step("a non-admin's admin_list_ops is rejected with Unauthorized");
}
