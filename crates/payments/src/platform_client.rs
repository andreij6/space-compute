use candid::{CandidType, Principal};
use sc_types::ApiError;
use serde::Deserialize;

#[derive(CandidType, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub enum CheckNameResult {
    Ok,
    Taken,
    Invalid,
}

#[derive(CandidType, Clone, Debug)]
pub struct RegisterArgs {
    pub canister_id: Principal,
    pub owner: Principal,
    pub name: String,
    pub avatar_seed: u64,
}

pub fn evaluate_owner_lookup(existing: Option<Principal>) -> Result<(), ApiError> {
    match existing {
        None => Ok(()),
        Some(aaa) => Err(ApiError::Conflict(format!(
            "owner already has an AAA: {aaa}"
        ))),
    }
}

pub fn evaluate_check_name(result: CheckNameResult) -> Result<(), ApiError> {
    match result {
        CheckNameResult::Ok => Ok(()),
        CheckNameResult::Taken => Err(ApiError::Conflict("name already taken".into())),
        CheckNameResult::Invalid => Err(ApiError::invalid("invalid name")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn p(n: u8) -> Principal {
        Principal::from_slice(&[n; 29])
    }

    #[test]
    fn t5_3_evaluate_owner_lookup_rejects_only_when_present() {
        assert_eq!(evaluate_owner_lookup(None), Ok(()));
        assert!(matches!(
            evaluate_owner_lookup(Some(p(1))),
            Err(ApiError::Conflict(_))
        ));
    }

    #[test]
    fn t5_3_evaluate_check_name_maps_each_variant() {
        assert_eq!(evaluate_check_name(CheckNameResult::Ok), Ok(()));
        assert!(matches!(
            evaluate_check_name(CheckNameResult::Taken),
            Err(ApiError::Conflict(_))
        ));
        assert!(matches!(
            evaluate_check_name(CheckNameResult::Invalid),
            Err(ApiError::InvalidInput(_))
        ));
    }
}
