use candid::{CandidType, Nat, Principal};
use serde::Deserialize;

use crate::journal::Account;

pub const MEMO_CREATE: u64 = 0x4145_5243;
pub const SIXTY_DAYS_SECS: u64 = 60 * 86_400;

#[derive(CandidType)]
pub struct CanisterSettingsArgs {
    pub controllers: Option<Vec<Principal>>,
    pub freezing_threshold: Option<Nat>,
}

#[derive(CandidType)]
pub struct NotifyCreateCanisterArg {
    pub block_index: u64,
    pub controller: Principal,
    pub subnet_type: Option<String>,
    pub subnet_selection: Option<()>,
    pub settings: Option<CanisterSettingsArgs>,
}

#[derive(CandidType, Deserialize, Clone, Debug, PartialEq)]
pub enum CmcNotifyError {
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

impl std::fmt::Display for CmcNotifyError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}

fn principal_to_subaccount(p: Principal) -> [u8; 32] {
    let bytes = p.as_slice();
    let mut sub = [0u8; 32];
    sub[0] = bytes.len() as u8;
    sub[1..1 + bytes.len()].copy_from_slice(bytes);
    sub
}

pub fn deposit_account(cmc: Principal, canister: Principal) -> Account {
    Account {
        owner: cmc,
        subaccount: Some(principal_to_subaccount(canister)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn p(n: u8) -> Principal {
        Principal::from_slice(&[n; 29])
    }

    #[test]
    fn t5_3_cmc_notify_error_display_is_debug() {
        assert_eq!(format!("{}", CmcNotifyError::Processing), "Processing");
    }

    #[test]
    fn t5_3_memo_create_matches_spec_bytes() {
        assert_eq!(
            MEMO_CREATE.to_le_bytes(),
            [0x43, 0x52, 0x45, 0x41, 0, 0, 0, 0]
        );
    }

    #[test]
    fn t5_3_deposit_account_derives_length_prefixed_subaccount() {
        let cmc = p(9);
        let canister = p(1);
        let account = deposit_account(cmc, canister);
        assert_eq!(account.owner, cmc);
        let sub = account.subaccount.unwrap();
        assert_eq!(sub[0], canister.as_slice().len() as u8);
        assert_eq!(&sub[1..1 + canister.as_slice().len()], canister.as_slice());
    }

    #[test]
    fn t5_3_deposit_account_differs_by_canister() {
        let cmc = p(9);
        assert_ne!(deposit_account(cmc, p(1)), deposit_account(cmc, p(2)));
    }
}
