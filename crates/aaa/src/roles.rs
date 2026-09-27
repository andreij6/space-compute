use candid::{CandidType, Principal};
use sc_types::ApiError;
use serde::Deserialize;

#[derive(CandidType, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub enum Role {
    Owner,
    Operator,
    Platform,
    None,
}

pub fn classify(
    caller: Principal,
    owner: Principal,
    platform_id: Principal,
    operator_active: bool,
) -> Role {
    if caller == Principal::anonymous() {
        return Role::None;
    }
    if caller == owner {
        return Role::Owner;
    }
    if operator_active {
        return Role::Operator;
    }
    if caller == platform_id {
        return Role::Platform;
    }
    Role::None
}

pub fn require_owner(caller: Principal, owner: Principal) -> Result<(), ApiError> {
    if caller != Principal::anonymous() && caller == owner {
        Ok(())
    } else {
        Err(ApiError::Unauthorized)
    }
}

pub fn can_ingress(caller: Principal, owner: Principal, operator_active: bool) -> bool {
    caller != Principal::anonymous() && (caller == owner || operator_active)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn p(n: u8) -> Principal {
        Principal::from_slice(&[n; 29])
    }

    #[test]
    fn t3_1_classify_matches_the_role_matrix() {
        let (owner, platform_id, operator, stranger) = (p(1), p(2), p(3), p(4));
        assert_eq!(classify(owner, owner, platform_id, false), Role::Owner);
        assert_eq!(classify(operator, owner, platform_id, true), Role::Operator);
        assert_eq!(
            classify(platform_id, owner, platform_id, false),
            Role::Platform
        );
        assert_eq!(classify(stranger, owner, platform_id, false), Role::None);
        assert_eq!(
            classify(Principal::anonymous(), owner, platform_id, false),
            Role::None
        );
        assert_eq!(
            classify(operator, owner, platform_id, false),
            Role::None,
            "an expired operator is treated as no one"
        );
    }

    #[test]
    fn t3_1_require_owner_accepts_only_the_owner() {
        let (owner, stranger) = (p(1), p(2));
        assert_eq!(require_owner(owner, owner), Ok(()));
        assert_eq!(require_owner(stranger, owner), Err(ApiError::Unauthorized));
        assert_eq!(
            require_owner(Principal::anonymous(), owner),
            Err(ApiError::Unauthorized)
        );
    }

    #[test]
    fn t3_1_can_ingress_allows_only_owner_and_active_operators() {
        let (owner, operator, stranger, platform_id) = (p(1), p(2), p(3), p(4));
        assert!(can_ingress(owner, owner, false));
        assert!(can_ingress(operator, owner, true));
        assert!(!can_ingress(operator, owner, false));
        assert!(!can_ingress(stranger, owner, false));
        assert!(!can_ingress(platform_id, owner, false));
        assert!(!can_ingress(Principal::anonymous(), owner, true));
    }
}
