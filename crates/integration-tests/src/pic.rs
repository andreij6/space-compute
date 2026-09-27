use candid::{decode_one, encode_one, CandidType, Nat, Principal};
use ic_ledger_types::{
    AccountIdentifier, Memo, Subaccount, Tokens, TransferArgs,
    TransferError as LegacyTransferError, DEFAULT_FEE, DEFAULT_SUBACCOUNT,
    MAINNET_CYCLES_MINTING_CANISTER_ID, MAINNET_LEDGER_CANISTER_ID,
};
use icrc_ledger_types::icrc1::account::Account;
use icrc_ledger_types::icrc1::transfer::{TransferArg, TransferError};
use pocket_ic::common::rest::{IcpFeatures, IcpFeaturesConfig};
use pocket_ic::{PocketIc, PocketIcBuilder};
use serde::de::DeserializeOwned;
use std::sync::Once;

pub const MEMO_TOP_UP: u64 = 0x5055_5054;
pub const MEMO_CREATE: u64 = 0x4145_5243;
pub const E8S: u64 = 100_000_000;
pub const LEDGER: Principal = MAINNET_LEDGER_CANISTER_ID;
pub const CMC: Principal = MAINNET_CYCLES_MINTING_CANISTER_ID;

#[derive(CandidType, serde::Deserialize)]
pub struct NotifyTopUpArg {
    pub block_index: u64,
    pub canister_id: Principal,
}

#[derive(CandidType, serde::Deserialize, Debug, Clone, PartialEq)]
pub enum NotifyError {
    Refunded {
        reason: String,
        block_index: Option<u64>,
    },
    Processing,
    TransactionTooOld(u64),
    InvalidTransaction(String),
    Other {
        error_code: u64,
        error_message: String,
    },
}

pub struct IcpEnv {
    pub pic: PocketIc,
}

pub fn canister_wasm(name: &str) -> Vec<u8> {
    static BUILD: Once = Once::new();
    let root = crate::repo_root();
    BUILD.call_once(|| {
        let ok = std::process::Command::new("cargo")
            .args([
                "build",
                "-q",
                "--target",
                "wasm32-unknown-unknown",
                "--release",
            ])
            .args([
                "-p", "platform", "-p", "payments", "-p", "treasury", "-p", "aaa",
            ])
            .current_dir(&root)
            .status()
            .expect("cargo")
            .success();
        assert!(ok, "building canister wasms failed");
    });
    std::fs::read(root.join(format!("target/wasm32-unknown-unknown/release/{name}.wasm")))
        .unwrap_or_else(|e| panic!("missing {name}.wasm: {e}"))
}

pub fn user(n: u8) -> Principal {
    Principal::self_authenticating([n; 32])
}

impl IcpEnv {
    pub fn new() -> Self {
        let features = IcpFeatures {
            icp_token: Some(IcpFeaturesConfig::DefaultConfig),
            cycles_minting: Some(IcpFeaturesConfig::DefaultConfig),
            registry: Some(IcpFeaturesConfig::DefaultConfig),
            ..Default::default()
        };
        let pic = PocketIcBuilder::new()
            .with_nns_subnet()
            .with_application_subnet()
            .with_icp_features(features)
            .build();
        IcpEnv { pic }
    }

    pub fn update<A: CandidType, R: DeserializeOwned + CandidType>(
        &self,
        canister: Principal,
        sender: Principal,
        method: &str,
        arg: A,
    ) -> R {
        let bytes = self
            .pic
            .update_call(canister, sender, method, encode_one(arg).unwrap())
            .unwrap_or_else(|e| panic!("{method} rejected: {e:?}"));
        decode_one(&bytes).unwrap_or_else(|e| panic!("{method} reply did not decode: {e}"))
    }

    pub fn query<A: CandidType, R: DeserializeOwned + CandidType>(
        &self,
        canister: Principal,
        sender: Principal,
        method: &str,
        arg: A,
    ) -> R {
        let bytes = self
            .pic
            .query_call(canister, sender, method, encode_one(arg).unwrap())
            .unwrap_or_else(|e| panic!("{method} rejected: {e:?}"));
        decode_one(&bytes).unwrap_or_else(|e| panic!("{method} reply did not decode: {e}"))
    }

    pub fn install(&self, name: &str, controller: Principal) -> Principal {
        let subnet = self.pic.topology().get_app_subnets()[0];
        let id = self
            .pic
            .create_canister_on_subnet(Some(controller), None, subnet);
        self.pic.add_cycles(id, 10_000_000_000_000);
        self.pic.install_canister(
            id,
            canister_wasm(name),
            encode_one(()).unwrap(),
            Some(controller),
        );
        id
    }

    pub fn icp_balance(&self, owner: Principal) -> u64 {
        let n: Nat = self.query(
            LEDGER,
            Principal::anonymous(),
            "icrc1_balance_of",
            Account {
                owner,
                subaccount: None,
            },
        );
        n.0.try_into().unwrap()
    }

    pub fn mint_icp(&self, to: Principal, e8s: u64) -> u64 {
        self.icrc1_transfer(
            Principal::anonymous(),
            Account {
                owner: to,
                subaccount: None,
            },
            e8s,
            None,
        )
        .expect("funding transfer from the genesis account")
    }

    pub fn icrc1_transfer(
        &self,
        from: Principal,
        to: Account,
        e8s: u64,
        memo: Option<Vec<u8>>,
    ) -> Result<u64, TransferError> {
        let arg = TransferArg {
            from_subaccount: None,
            to,
            fee: None,
            created_at_time: None,
            memo: memo.map(Into::into),
            amount: Nat::from(e8s),
        };
        let r: Result<Nat, TransferError> = self.update(LEDGER, from, "icrc1_transfer", arg);
        r.map(|n| n.0.try_into().unwrap())
    }

    pub fn legacy_transfer(
        &self,
        from: Principal,
        to: AccountIdentifier,
        e8s: u64,
        memo: u64,
    ) -> Result<u64, LegacyTransferError> {
        let arg = TransferArgs {
            memo: Memo(memo),
            amount: Tokens::from_e8s(e8s),
            fee: DEFAULT_FEE,
            from_subaccount: None,
            to,
            created_at_time: None,
        };
        self.update(LEDGER, from, "transfer", arg)
    }

    pub fn cmc_top_up_account(canister: Principal) -> AccountIdentifier {
        AccountIdentifier::new(&CMC, &Subaccount::from(canister))
    }

    pub fn notify_top_up(&self, block_index: u64, canister: Principal) -> Result<Nat, NotifyError> {
        self.update(
            CMC,
            Principal::anonymous(),
            "notify_top_up",
            NotifyTopUpArg {
                block_index,
                canister_id: canister,
            },
        )
    }

    pub fn default_account(owner: Principal) -> AccountIdentifier {
        AccountIdentifier::new(&owner, &DEFAULT_SUBACCOUNT)
    }
}

impl Default for IcpEnv {
    fn default() -> Self {
        Self::new()
    }
}
