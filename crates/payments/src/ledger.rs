use candid::{CandidType, Nat};
use serde::Deserialize;

use crate::journal::Account;

#[derive(CandidType)]
pub struct TransferArg {
    pub from_subaccount: Option<[u8; 32]>,
    pub to: Account,
    pub fee: Option<Nat>,
    pub created_at_time: Option<u64>,
    pub memo: Option<Vec<u8>>,
    pub amount: Nat,
}

#[derive(CandidType, Deserialize, Clone, Debug, PartialEq)]
pub enum TransferError {
    BadFee { expected_fee: Nat },
    BadBurn { min_burn_amount: Nat },
    InsufficientFunds { balance: Nat },
    TooOld,
    CreatedInFuture { ledger_time: u64 },
    TemporarilyUnavailable,
    Duplicate { duplicate_of: Nat },
    GenericError { error_code: Nat, message: String },
}

#[derive(CandidType)]
pub struct TransferFromArg {
    pub spender_subaccount: Option<[u8; 32]>,
    pub from: Account,
    pub to: Account,
    pub amount: Nat,
    pub fee: Option<Nat>,
    pub memo: Option<Vec<u8>>,
    pub created_at_time: Option<u64>,
}

#[derive(CandidType)]
pub struct AllowanceArg {
    pub account: Account,
    pub spender: Account,
}

#[derive(CandidType, Deserialize, Clone, Debug, PartialEq)]
pub struct Allowance {
    pub allowance: Nat,
    pub expires_at: Option<u64>,
}

#[derive(CandidType, Deserialize, Clone, Debug, PartialEq)]
pub enum TransferFromError {
    BadFee { expected_fee: Nat },
    BadBurn { min_burn_amount: Nat },
    InsufficientFunds { balance: Nat },
    InsufficientAllowance { allowance: Nat },
    TooOld,
    CreatedInFuture { ledger_time: u64 },
    Duplicate { duplicate_of: Nat },
    TemporarilyUnavailable,
    GenericError { error_code: Nat, message: String },
}

#[derive(Clone, Debug, PartialEq)]
pub enum Outcome {
    Done(u64),
    BadFee(u64),
    Rejected { reason: String, funding: bool },
    Unknown(String),
}

fn block(n: Nat) -> u64 {
    u64::try_from(n.0).unwrap_or(u64::MAX)
}

fn fee_or_reject(expected_fee: Nat) -> Outcome {
    match u64::try_from(expected_fee.0.clone()) {
        Ok(f) => Outcome::BadFee(f),
        Err(_) => Outcome::Rejected {
            reason: format!("BadFee {{ expected_fee: {expected_fee} }}"),
            funding: false,
        },
    }
}

pub fn classify_transfer(r: Result<Nat, TransferError>) -> Outcome {
    match r {
        Ok(n) | Err(TransferError::Duplicate { duplicate_of: n }) => Outcome::Done(block(n)),
        Err(TransferError::BadFee { expected_fee }) => fee_or_reject(expected_fee),
        Err(
            e @ (TransferError::TemporarilyUnavailable | TransferError::CreatedInFuture { .. }),
        ) => Outcome::Unknown(format!("{e:?}")),
        Err(e @ TransferError::InsufficientFunds { .. }) => Outcome::Rejected {
            reason: format!("{e:?}"),
            funding: true,
        },
        Err(e) => Outcome::Rejected {
            reason: format!("{e:?}"),
            funding: false,
        },
    }
}

pub fn classify_transfer_from(r: Result<Nat, TransferFromError>) -> Outcome {
    match r {
        Ok(n) | Err(TransferFromError::Duplicate { duplicate_of: n }) => Outcome::Done(block(n)),
        Err(TransferFromError::BadFee { expected_fee }) => fee_or_reject(expected_fee),
        Err(
            e @ (TransferFromError::TemporarilyUnavailable
            | TransferFromError::CreatedInFuture { .. }),
        ) => Outcome::Unknown(format!("{e:?}")),
        Err(
            e @ (TransferFromError::InsufficientFunds { .. }
            | TransferFromError::InsufficientAllowance { .. }),
        ) => Outcome::Rejected {
            reason: format!("{e:?}"),
            funding: true,
        },
        Err(e) => Outcome::Rejected {
            reason: format!("{e:?}"),
            funding: false,
        },
    }
}

pub fn expected_fee(e: &TransferError) -> Option<u64> {
    match e {
        TransferError::BadFee { expected_fee } => u64::try_from(expected_fee.0.clone()).ok(),
        _ => None,
    }
}

pub fn expected_fee_from(e: &TransferFromError) -> Option<u64> {
    match e {
        TransferFromError::BadFee { expected_fee } => u64::try_from(expected_fee.0.clone()).ok(),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn t5_3_expected_fee_extracts_bad_fee_only() {
        assert_eq!(
            expected_fee(&TransferError::BadFee {
                expected_fee: Nat::from(20_000u64)
            }),
            Some(20_000)
        );
        assert_eq!(expected_fee(&TransferError::TooOld), None);
        assert_eq!(
            expected_fee(&TransferError::Duplicate {
                duplicate_of: Nat::from(1u64)
            }),
            None
        );
    }

    #[test]
    fn t5_3_expected_fee_from_extracts_bad_fee_only() {
        assert_eq!(
            expected_fee_from(&TransferFromError::BadFee {
                expected_fee: Nat::from(20_000u64)
            }),
            Some(20_000)
        );
        assert_eq!(expected_fee_from(&TransferFromError::TooOld), None);
        assert_eq!(
            expected_fee_from(&TransferFromError::InsufficientAllowance {
                allowance: Nat::from(0u64)
            }),
            None
        );
    }

    #[test]
    fn t5_7_classify_transfer_never_treats_ambiguous_outcomes_as_failed() {
        assert_eq!(classify_transfer(Ok(Nat::from(7u64))), Outcome::Done(7));
        assert_eq!(
            classify_transfer(Err(TransferError::Duplicate {
                duplicate_of: Nat::from(3u64)
            })),
            Outcome::Done(3)
        );
        assert_eq!(
            classify_transfer(Err(TransferError::BadFee {
                expected_fee: Nat::from(20_000u64)
            })),
            Outcome::BadFee(20_000)
        );
        assert!(matches!(
            classify_transfer(Err(TransferError::TemporarilyUnavailable)),
            Outcome::Unknown(_)
        ));
        assert!(matches!(
            classify_transfer(Err(TransferError::CreatedInFuture { ledger_time: 1 })),
            Outcome::Unknown(_)
        ));
        assert!(matches!(
            classify_transfer(Err(TransferError::InsufficientFunds {
                balance: Nat::from(0u64)
            })),
            Outcome::Rejected { funding: true, .. }
        ));
        assert!(matches!(
            classify_transfer(Err(TransferError::TooOld)),
            Outcome::Rejected { funding: false, .. }
        ));
        assert!(matches!(
            classify_transfer(Err(TransferError::BadFee {
                expected_fee: Nat::from(u128::MAX)
            })),
            Outcome::Rejected { funding: false, .. }
        ));
    }

    #[test]
    fn t5_7_classify_transfer_from_flags_allowance_and_funds_as_funding_rejections() {
        assert_eq!(
            classify_transfer_from(Err(TransferFromError::Duplicate {
                duplicate_of: Nat::from(9u64)
            })),
            Outcome::Done(9)
        );
        assert!(matches!(
            classify_transfer_from(Err(TransferFromError::InsufficientAllowance {
                allowance: Nat::from(0u64)
            })),
            Outcome::Rejected { funding: true, .. }
        ));
        assert!(matches!(
            classify_transfer_from(Err(TransferFromError::InsufficientFunds {
                balance: Nat::from(0u64)
            })),
            Outcome::Rejected { funding: true, .. }
        ));
        assert!(matches!(
            classify_transfer_from(Err(TransferFromError::TemporarilyUnavailable)),
            Outcome::Unknown(_)
        ));
        assert!(matches!(
            classify_transfer_from(Err(TransferFromError::GenericError {
                error_code: Nat::from(1u64),
                message: "x".into()
            })),
            Outcome::Rejected { funding: false, .. }
        ));
        assert_eq!(
            classify_transfer_from(Err(TransferFromError::BadFee {
                expected_fee: Nat::from(10_000u64)
            })),
            Outcome::BadFee(10_000)
        );
    }
}
