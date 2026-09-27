use candid::{Nat, Principal};
use icrc_ledger_types::icrc1::account::{Account, Subaccount};
use icrc_ledger_types::icrc2::approve::{ApproveArgs, ApproveError};
use icrc_ledger_types::icrc2::transfer_from::{TransferFromArgs, TransferFromError};
use integration_tests::pic::{
    user, IcpEnv, NotifyError, CMC, E8S, LEDGER, MEMO_CREATE, MEMO_TOP_UP,
};

#[derive(candid::CandidType)]
struct NotifyCreateCanisterArg {
    block_index: u64,
    controller: Principal,
}
use integration_tests::step;

fn cmc_account(canister: Principal) -> Account {
    let mut sub = [0u8; 32];
    let bytes = canister.as_slice();
    sub[0] = bytes.len() as u8;
    sub[1..1 + bytes.len()].copy_from_slice(bytes);
    Account {
        owner: CMC,
        subaccount: Some(sub as Subaccount),
    }
}

fn outcome(r: &Result<Nat, NotifyError>) -> String {
    match r {
        Ok(c) => format!("accepted, minted {c} cycles"),
        Err(e) => format!("rejected: {e:?}"),
    }
}

#[test]
fn sp_1_cmc_notify_top_up_accepts_which_transfer_kinds() {
    println!("SP-1 demo: which ledger transfers can notify_top_up consume?");
    let env = IcpEnv::new();
    let alice = user(1);
    let payments = user(2);
    env.mint_icp(alice, 20 * E8S);
    let target = env.install("platform", alice);

    let legacy = env
        .legacy_transfer(alice, IcpEnv::cmc_top_up_account(target), E8S, MEMO_TOP_UP)
        .unwrap();
    let r_legacy = env.notify_top_up(legacy, target);
    step(&format!(
        "legacy transfer, u64 memo TPUP → {}",
        outcome(&r_legacy)
    ));

    let le = MEMO_TOP_UP.to_le_bytes().to_vec();
    let icrc1_le = env
        .icrc1_transfer(alice, cmc_account(target), E8S, Some(le.clone()))
        .unwrap();
    let r_icrc1_le = env.notify_top_up(icrc1_le, target);
    step(&format!(
        "icrc1_transfer, icrc1 memo = TPUP (8 bytes LE) → {}",
        outcome(&r_icrc1_le)
    ));

    let icrc1_none = env
        .icrc1_transfer(alice, cmc_account(target), E8S, None)
        .unwrap();
    let r_icrc1_none = env.notify_top_up(icrc1_none, target);
    step(&format!(
        "icrc1_transfer, no memo → {}",
        outcome(&r_icrc1_none)
    ));

    let approve: Result<Nat, ApproveError> = env.update(
        LEDGER,
        alice,
        "icrc2_approve",
        ApproveArgs {
            from_subaccount: None,
            spender: Account {
                owner: payments,
                subaccount: None,
            },
            amount: Nat::from(5 * E8S),
            expected_allowance: None,
            expires_at: None,
            fee: None,
            memo: None,
            created_at_time: None,
        },
    );
    approve.expect("approve");
    let pulled: Result<Nat, TransferFromError> = env.update(
        LEDGER,
        payments,
        "icrc2_transfer_from",
        TransferFromArgs {
            spender_subaccount: None,
            from: Account {
                owner: alice,
                subaccount: None,
            },
            to: cmc_account(target),
            amount: Nat::from(E8S),
            fee: None,
            memo: Some(le.into()),
            created_at_time: None,
        },
    );
    let block: u64 = pulled.expect("transfer_from").0.try_into().unwrap();
    let r_icrc2 = env.notify_top_up(block, target);
    step(&format!(
        "icrc2_transfer_from by a spender, icrc1 memo TPUP → {}",
        outcome(&r_icrc2)
    ));

    let approve_create: Result<Nat, ApproveError> = env.update(
        LEDGER,
        alice,
        "icrc2_approve",
        ApproveArgs {
            from_subaccount: None,
            spender: Account {
                owner: payments,
                subaccount: None,
            },
            amount: Nat::from(5 * E8S),
            expected_allowance: None,
            expires_at: None,
            fee: None,
            memo: None,
            created_at_time: None,
        },
    );
    approve_create.expect("approve");
    let crea: Result<Nat, TransferFromError> = env.update(
        LEDGER,
        payments,
        "icrc2_transfer_from",
        TransferFromArgs {
            spender_subaccount: None,
            from: Account {
                owner: alice,
                subaccount: None,
            },
            to: cmc_account(payments),
            amount: Nat::from(2 * E8S),
            fee: None,
            memo: Some(MEMO_CREATE.to_le_bytes().to_vec().into()),
            created_at_time: None,
        },
    );
    let crea_block: u64 = crea.expect("transfer_from CREA").0.try_into().unwrap();
    let created: Result<Principal, NotifyError> = env.update(
        CMC,
        payments,
        "notify_create_canister",
        NotifyCreateCanisterArg {
            block_index: crea_block,
            controller: payments,
        },
    );
    step(&format!(
        "icrc2_transfer_from with memo CREA → notify_create_canister → {}",
        match &created {
            Ok(id) => format!("created canister {id}"),
            Err(e) => format!("rejected: {e:?}"),
        }
    ));

    assert!(r_legacy.is_ok(), "legacy top-up path");
    assert!(r_icrc1_le.is_ok(), "icrc1 transfer with TPUP memo");
    assert!(
        matches!(r_icrc1_none, Err(NotifyError::Refunded { .. })),
        "memo-less transfers are refunded"
    );
    assert!(r_icrc2.is_ok(), "icrc2 transfer_from with TPUP memo");
    let created = created.expect("icrc2 transfer_from with CREA memo creates a canister");
    assert_eq!(env.pic.get_controllers(created), vec![payments]);
    step("DECISION: payments pulls with icrc2_transfer_from straight into the CMC deposit account (memo = 8-byte LE u64); the two-step legacy fallback is not needed");
}
