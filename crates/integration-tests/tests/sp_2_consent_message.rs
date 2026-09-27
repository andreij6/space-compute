use candid::{encode_one, Nat};
use icrc_ledger_types::icrc1::account::Account;
use icrc_ledger_types::icrc2::approve::ApproveArgs;
use icrc_ledger_types::icrc21::errors::Icrc21Error;
use icrc_ledger_types::icrc21::requests::{
    ConsentMessageMetadata, ConsentMessageRequest, ConsentMessageSpec, DisplayMessageType,
};
use icrc_ledger_types::icrc21::responses::{ConsentInfo, ConsentMessage};
use integration_tests::pic::{user, IcpEnv, E8S, LEDGER};
use integration_tests::step;

fn consent(env: &IcpEnv, args: &ApproveArgs, display: DisplayMessageType) -> String {
    let req = ConsentMessageRequest {
        method: "icrc2_approve".into(),
        arg: encode_one(args).unwrap(),
        user_preferences: ConsentMessageSpec {
            metadata: ConsentMessageMetadata {
                language: "en".into(),
                utc_offset_minutes: None,
            },
            device_spec: Some(display),
        },
    };
    let r: Result<ConsentInfo, Icrc21Error> =
        env.update(LEDGER, user(1), "icrc21_canister_call_consent_message", req);
    match r
        .expect("the ICP ledger implements ICRC-21")
        .consent_message
    {
        ConsentMessage::GenericDisplayMessage(t) => t,
        ConsentMessage::FieldsDisplayMessage(f) => format!("{f:?}"),
    }
}

#[test]
fn sp_2_ledger_consent_message_shows_the_spender_subaccount() {
    println!("SP-2 demo: what does a wallet (OISY) show for an approve to a spender subaccount?");
    let env = IcpEnv::new();
    let payments = user(2);
    let mut sub = [0u8; 32];
    sub[..8].copy_from_slice(b"sc-spend");
    sub[31] = 7;
    let spender = Account {
        owner: payments,
        subaccount: Some(sub),
    };
    let args = ApproveArgs {
        from_subaccount: None,
        spender,
        amount: Nat::from(3 * E8S),
        expected_allowance: None,
        expires_at: None,
        fee: None,
        memo: None,
        created_at_time: None,
    };
    let generic = consent(&env, &args, DisplayMessageType::GenericDisplay);
    let fields = consent(&env, &args, DisplayMessageType::FieldsDisplay);
    println!("----- generic consent message -----\n{generic}\n-----------------------------------");
    let textual = spender.to_string();
    let sub_hex = hex_tail(&sub);
    step(&format!("spender in ICRC-1 textual form: {textual}"));
    let shows_generic = generic.contains(&textual) || generic.contains(&sub_hex);
    let shows_fields = fields.contains(&textual) || fields.contains(&sub_hex);
    step(&format!(
        "generic display shows the spender subaccount → {shows_generic}"
    ));
    step(&format!(
        "fields display shows the spender subaccount → {shows_fields}"
    ));
    assert!(
        shows_generic && shows_fields,
        "the ledger's consent message must identify the full spender account"
    );
    step("DECISION: the ICP ledger's ICRC-21 message names the full spender account (owner + subaccount), and OISY renders the ledger's message, so the beneficiary-bound spender subaccount is visible to the user; keep the wallet path. Deposit addresses stay as the fallback for wallets without ICRC-21.");
}

fn hex_tail(sub: &[u8; 32]) -> String {
    let h: String = sub.iter().map(|b| format!("{b:02x}")).collect();
    h.trim_start_matches('0').to_string()
}
