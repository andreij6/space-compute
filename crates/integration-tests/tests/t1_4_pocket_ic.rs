use integration_tests::pic::{user, IcpEnv, E8S, MEMO_TOP_UP};
use integration_tests::step;

#[test]
fn t1_4_harness_mints_icp_and_tops_up_a_canister_through_the_cmc() {
    println!("T1.4 demo: PocketIC with the real ICP ledger + cycles minting canister");
    let env = IcpEnv::new();
    let alice = user(1);

    env.mint_icp(alice, 10 * E8S);
    assert_eq!(env.icp_balance(alice), 10 * E8S);
    step("minted 10 ICP to alice from the genesis account");

    let platform = env.install("platform", alice);
    let version: String = env.query(platform, alice, "version", ());
    assert_eq!(version, "platform 0.1.0");
    step(&format!(
        "installed the platform canister on an application subnet → {version}"
    ));

    let before = env.pic.cycle_balance(platform);
    let block = env
        .legacy_transfer(
            alice,
            IcpEnv::cmc_top_up_account(platform),
            E8S,
            MEMO_TOP_UP,
        )
        .expect("transfer to the CMC top-up account");
    step(&format!(
        "sent 1 ICP to the CMC deposit account for platform (block {block}, memo TPUP)"
    ));

    let minted = env.notify_top_up(block, platform).expect("notify_top_up");
    let after = env.pic.cycle_balance(platform);
    assert!(after > before, "cycles must increase: {before} → {after}");
    step(&format!(
        "notify_top_up minted {minted} cycles; platform balance {:.2}T → {:.2}T",
        before as f64 / 1e12,
        after as f64 / 1e12
    ));
    assert_eq!(env.icp_balance(alice), 9 * E8S - 10_000);
    step("alice paid 1 ICP + 0.0001 ICP fee");
}
