use candid::{decode_args, encode_args, Nat, Principal};
use icrc_ledger_types::icrc1::account::{Account as Icrc1Account, Subaccount};
use integration_tests::pic::{user, IcpEnv, NotifyError, E8S, MEMO_TOP_UP};
use integration_tests::step;
use payments::deposit::Purpose;
use payments::journal::Account;
use payments::quote::Quote;
use payments::rate::RateCache;
use sc_types::ApiError;

fn cmc_account(canister: Principal) -> Icrc1Account {
    let mut sub = [0u8; 32];
    let bytes = canister.as_slice();
    sub[0] = bytes.len() as u8;
    sub[1..1 + bytes.len()].copy_from_slice(bytes);
    Icrc1Account {
        owner: integration_tests::pic::CMC,
        subaccount: Some(sub as Subaccount),
    }
}

#[test]
fn t5_2_quotes_are_within_2_percent_of_what_the_cmc_actually_mints() {
    println!("T5.2 demo: XDR cache, quotes, deposit accounts — proven against the real CMC");
    let env = IcpEnv::new();
    let alice = user(1);
    let payments = env.install("payments", alice);
    env.mint_icp(alice, 20 * E8S);

    for _ in 0..5 {
        env.pic.tick();
    }
    let rate: RateCache = env.query(payments, alice, "get_rate", ());
    step(&format!(
        "the startup timer primed the XDR cache before any query: {} XDR-permyriad/ICP, fetched at {}s",
        rate.xdr_permyriad_per_icp, rate.fetched_at_secs
    ));
    assert!(
        rate.xdr_permyriad_per_icp > 0,
        "rate must be populated on install"
    );

    let target_cycles: u128 = 1_000_000_000_000;
    let quote: Result<Quote, ApiError> =
        env.query(payments, alice, "get_quote_topup", target_cycles);
    let quote = quote.expect("get_quote_topup");
    step(&format!(
        "get_quote_topup({target_cycles}) = deposit {} e8s + fee {} e8s (total {} e8s) at rate {}",
        quote.deposit_e8s, quote.fee_e8s, quote.total_e8s, quote.rate_xdr_permyriad_per_icp
    ));
    assert_eq!(quote.cycles, target_cycles);

    let beneficiary = env.install("platform", alice);
    let cycles_before = env.pic.cycle_balance(beneficiary);

    let block = env
        .icrc1_transfer(
            alice,
            cmc_account(beneficiary),
            quote.deposit_e8s,
            Some(MEMO_TOP_UP.to_le_bytes().to_vec()),
        )
        .expect("transfer the quoted e8s straight into the CMC deposit account");
    let minted: Result<Nat, NotifyError> = env.notify_top_up(block, beneficiary);
    let minted_cycles: u128 = minted
        .expect("notify_top_up should accept the memoed transfer")
        .0
        .try_into()
        .unwrap();
    let cycles_after = env.pic.cycle_balance(beneficiary);
    step(&format!(
        "CMC minted {minted_cycles} cycles; the beneficiary's balance rose by {}",
        cycles_after - cycles_before
    ));

    let diff = minted_cycles.abs_diff(target_cycles);
    let error_pct = diff as f64 / target_cycles as f64 * 100.0;
    step(&format!(
        "quote error vs the real mint: {error_pct:.4}% (quoted {target_cycles}, minted {minted_cycles})"
    ));
    assert!(
        error_pct <= 2.0,
        "quote must be within 2% of what the CMC actually mints, was {error_pct}%"
    );

    let spawn_quote: Result<Quote, ApiError> = env.query(payments, alice, "get_quote_spawn", ());
    let spawn_quote = spawn_quote.expect("get_quote_spawn");
    step(&format!(
        "get_quote_spawn() = {} cycles for {} e8s (2% buffer + creation fee baked in)",
        spawn_quote.cycles, spawn_quote.deposit_e8s
    ));
    assert!(spawn_quote.cycles > 1_000_000_000_000);

    let raw = env
        .pic
        .query_call(
            payments,
            alice,
            "get_deposit_account",
            encode_args((Purpose::Spawn, alice)).unwrap(),
        )
        .expect("get_deposit_account");
    let (text, account): (String, Account) = decode_args(&raw).unwrap();
    step(&format!(
        "get_deposit_account(Spawn, alice) = {text} (owner {}, subaccount set)",
        account.owner
    ));
    assert_eq!(account.owner, payments);
    assert_eq!(text.len(), 64);
    assert!(text.chars().all(|c| c.is_ascii_hexdigit()));

    let raw2 = env
        .pic
        .query_call(
            payments,
            alice,
            "get_deposit_account",
            encode_args((Purpose::TopUp, alice)).unwrap(),
        )
        .expect("get_deposit_account for a different purpose");
    let (text2, _): (String, Account) = decode_args(&raw2).unwrap();
    assert_ne!(
        text, text2,
        "distinct purposes derive distinct deposit accounts"
    );

    let refreshed: Result<RateCache, ApiError> =
        env.update(payments, alice, "admin_refresh_rate", ());
    let refreshed = refreshed.expect("admin can refresh the rate on demand");
    step(&format!(
        "admin_refresh_rate() re-fetched the CMC rate on demand: {} XDR-permyriad/ICP",
        refreshed.xdr_permyriad_per_icp
    ));

    let denied: Result<RateCache, ApiError> =
        env.update(payments, user(2), "admin_refresh_rate", ());
    assert_eq!(denied, Err(ApiError::Unauthorized));
    step("a non-admin cannot force an on-demand rate refresh");
}
