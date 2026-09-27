use candid::{encode_one, Nat, Principal};
use integration_tests::pic::{user, IcpEnv, E8S, MEMO_TOP_UP};
use integration_tests::step;
use pocket_ic::CanisterSettings;

fn describe(r: &Result<Vec<u8>, pocket_ic::RejectResponse>) -> String {
    match r {
        Ok(_) => "answered".into(),
        Err(e) => format!(
            "rejected ({:?}: {})",
            e.reject_code,
            e.reject_message.chars().take(70).collect::<String>()
        ),
    }
}

#[test]
fn sp_3_frozen_aaa_rejects_queries_until_topped_up_via_the_cmc() {
    println!("SP-3 demo: what works when an AAA canister is frozen, and how it recovers");
    let env = IcpEnv::new();
    let owner = user(1);
    let aaa = env.install_with_cycles("aaa", owner, 1_000_000_000_000);
    let balance = env.pic.cycle_balance(aaa);
    let burn_per_day: u128 = env
        .pic
        .canister_status(aaa, Some(owner))
        .unwrap()
        .idle_cycles_burned_per_day
        .0
        .try_into()
        .unwrap();
    let threshold_secs = (balance * 2 / burn_per_day.max(1) * 86_400) as u64;
    env.pic
        .update_canister_settings(
            aaa,
            Some(owner),
            CanisterSettings {
                freezing_threshold: Some(Nat::from(threshold_secs)),
                ..Default::default()
            },
        )
        .expect("set freezing threshold");
    step(&format!(
        "aaa holds {:.2}T cycles; freezing threshold set to twice its idle runway → frozen",
        balance as f64 / 1e12
    ));

    let query = env
        .pic
        .query_call(aaa, owner, "version", encode_one(()).unwrap());
    step(&format!("ingress query → {}", describe(&query)));
    let update = env
        .pic
        .update_call(aaa, owner, "version", encode_one(()).unwrap());
    step(&format!("update call → {}", describe(&update)));
    let status = env.pic.canister_status(aaa, Some(owner));
    step(&format!(
        "controller canister_status → {}",
        if status.is_ok() { "works" } else { "rejected" }
    ));
    let stranger = env.pic.canister_status(aaa, Some(Principal::anonymous()));
    step(&format!(
        "non-controller canister_status → {}",
        if stranger.is_ok() {
            "works"
        } else {
            "rejected"
        }
    ));

    let payer = user(9);
    env.mint_icp(payer, 50 * E8S);
    let mut topped = 0u128;
    while env
        .pic
        .query_call(aaa, owner, "version", encode_one(()).unwrap())
        .is_err()
        && topped < 200_000_000_000_000
    {
        let block = env
            .legacy_transfer(payer, IcpEnv::cmc_top_up_account(aaa), 5 * E8S, MEMO_TOP_UP)
            .unwrap();
        let minted: Nat = env
            .notify_top_up(block, aaa)
            .expect("CMC tops up a frozen canister");
        topped += u128::try_from(minted.0).unwrap();
    }
    let after = env
        .pic
        .query_call(aaa, owner, "version", encode_one(()).unwrap());
    step(&format!(
        "topped up {:.2}T cycles through the CMC (works while frozen) → query {}",
        topped as f64 / 1e12,
        describe(&after)
    ));

    assert!(query.is_err(), "frozen canisters reject ingress queries");
    assert!(update.is_err(), "frozen canisters reject updates");
    assert!(stranger.is_err(), "only controllers read canister_status");
    assert!(after.is_ok(), "a CMC top-up unfreezes the canister");
    step("DECISION: the dashboard never depends on a live AAA query — it reads the platform's cached AAA status (heartbeat) and shows 'Frozen — top up' when the AAA does not answer; top-ups go through the CMC, which works on frozen canisters");
}
