use candid::{CandidType, Principal};
use ic_ledger_types::{AccountIdentifier, Subaccount};
use sc_types::ApiError;
use serde::Deserialize;
use sha2::{Digest, Sha256};

use crate::journal::Account;

#[derive(CandidType, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub enum Purpose {
    Spawn,
    TopUp,
    Auto,
}

impl Purpose {
    fn tag(self) -> &'static [u8] {
        match self {
            Purpose::Spawn => b"spawn",
            Purpose::TopUp => b"topup",
            Purpose::Auto => b"auto",
        }
    }
}

fn derive_subaccount(prefix: &[u8], purpose: Purpose, beneficiary: Principal) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(prefix);
    hasher.update(purpose.tag());
    hasher.update(beneficiary.as_slice());
    hasher.finalize().into()
}

pub fn deposit_subaccount(purpose: Purpose, beneficiary: Principal) -> [u8; 32] {
    derive_subaccount(b"sc-deposit", purpose, beneficiary)
}

pub fn spender_subaccount(purpose: Purpose, beneficiary: Principal) -> [u8; 32] {
    derive_subaccount(b"sc-spender", purpose, beneficiary)
}

pub fn deposit_account(
    self_id: Principal,
    purpose: Purpose,
    beneficiary: Principal,
) -> (String, Account) {
    let sub = deposit_subaccount(purpose, beneficiary);
    let text = AccountIdentifier::new(&self_id, &Subaccount(sub)).to_string();
    (
        text,
        Account {
            owner: self_id,
            subaccount: Some(sub),
        },
    )
}

pub fn sweep_amount(balance: u64, min_required_e8s: u64, fee_e8s: u64) -> Result<u64, ApiError> {
    if balance < min_required_e8s {
        return Err(ApiError::invalid(
            "deposit balance is below the required amount",
        ));
    }
    Ok(balance.saturating_sub(fee_e8s))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn p(n: u8) -> Principal {
        Principal::from_slice(&[n; 29])
    }

    #[test]
    fn t5_3_sweep_amount_leaves_zero_dust_when_balance_covers_fee() {
        assert_eq!(
            sweep_amount(1_000_010_000, 1_000_000_000, 10_000),
            Ok(1_000_000_000)
        );
    }

    #[test]
    fn t5_3_sweep_amount_rejects_balance_below_requirement() {
        assert!(matches!(
            sweep_amount(999, 1_000, 10),
            Err(ApiError::InvalidInput(_))
        ));
    }

    #[test]
    fn t5_2_deposit_subaccount_matches_the_spec_formula() {
        let mut hasher = Sha256::new();
        hasher.update(b"sc-deposit");
        hasher.update(b"spawn");
        hasher.update(p(1).as_slice());
        let expected: [u8; 32] = hasher.finalize().into();
        assert_eq!(deposit_subaccount(Purpose::Spawn, p(1)), expected);
    }

    #[test]
    fn t5_2_spender_subaccount_matches_the_spec_formula() {
        let mut hasher = Sha256::new();
        hasher.update(b"sc-spender");
        hasher.update(b"topup");
        hasher.update(p(2).as_slice());
        let expected: [u8; 32] = hasher.finalize().into();
        assert_eq!(spender_subaccount(Purpose::TopUp, p(2)), expected);
    }

    #[test]
    fn t5_2_subaccounts_differ_by_purpose_and_beneficiary() {
        let a = deposit_subaccount(Purpose::Spawn, p(1));
        let b = deposit_subaccount(Purpose::TopUp, p(1));
        let c = deposit_subaccount(Purpose::Spawn, p(2));
        assert_ne!(a, b);
        assert_ne!(a, c);
        assert_ne!(
            deposit_subaccount(Purpose::Auto, p(1)),
            spender_subaccount(Purpose::Auto, p(1))
        );
    }

    #[test]
    fn t5_2_deposit_account_returns_hex_text_and_icrc1_account() {
        let payments = p(9);
        let beneficiary = p(1);
        let (text, account) = deposit_account(payments, Purpose::Spawn, beneficiary);
        assert_eq!(text.len(), 64);
        assert!(text.chars().all(|c| c.is_ascii_hexdigit()));
        assert_eq!(account.owner, payments);
        assert_eq!(
            account.subaccount,
            Some(deposit_subaccount(Purpose::Spawn, beneficiary))
        );
    }
}
