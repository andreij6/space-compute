use candid::Principal;
use sc_types::{ApiError, ClassificationSubmission, ReviewSubmission};

pub fn verify_caller(
    caller: Principal,
    owner: Principal,
    operator_active: bool,
) -> Result<(), ApiError> {
    if !crate::roles::can_ingress(caller, owner, operator_active) {
        Err(ApiError::Unauthorized)
    } else {
        Ok(())
    }
}

pub fn check_low_cycles(balance: u128, threshold: u128) -> Result<(), ApiError> {
    if balance < threshold {
        Err(ApiError::Internal("low cycles: top up".into()))
    } else {
        Ok(())
    }
}

pub fn prepare_classification_submission(
    mut submission: ClassificationSubmission,
    caller: Principal,
    default_agent_label: Option<String>,
) -> ClassificationSubmission {
    submission.submitted_by = caller;
    if submission.agent_label.is_none() {
        submission.agent_label = default_agent_label;
    }
    submission
}

pub fn prepare_review_submission(
    mut submission: ReviewSubmission,
    caller: Principal,
    default_agent_label: Option<String>,
) -> ReviewSubmission {
    submission.submitted_by = caller;
    if submission.agent_label.is_none() {
        submission.agent_label = default_agent_label;
    }
    submission
}

#[cfg(test)]
mod tests {
    use super::*;

    fn p(n: u8) -> Principal {
        Principal::from_slice(&[n; 29])
    }

    #[test]
    fn t3_2_verify_caller_allows_owner_and_active_operator() {
        let (owner, operator, stranger) = (p(1), p(2), p(3));
        assert!(verify_caller(owner, owner, false).is_ok());
        assert!(verify_caller(operator, owner, true).is_ok());
        assert!(verify_caller(operator, owner, false).is_err());
        assert!(verify_caller(stranger, owner, false).is_err());
        assert!(verify_caller(Principal::anonymous(), owner, true).is_err());
    }

    #[test]
    fn t3_2_check_low_cycles_refuses_when_under_threshold() {
        assert!(check_low_cycles(5_000, 10_000).is_err());
        assert!(check_low_cycles(10_000, 10_000).is_ok());
        assert!(check_low_cycles(15_000, 10_000).is_ok());
    }

    #[test]
    fn t3_2_prepare_submission_stamps_caller_and_defaults_label() {
        let sub = ClassificationSubmission {
            task_id: 1,
            answers: Vec::new(),
            observed_image_sha256: vec![1, 2, 3],
            discovery: None,
            agent_label: None,
            submitted_by: p(99),
        };
        let stamped = prepare_classification_submission(sub, p(4), Some("default-bot".into()));
        assert_eq!(stamped.submitted_by, p(4));
        assert_eq!(stamped.agent_label.as_deref(), Some("default-bot"));

        let rev = ReviewSubmission {
            assignment_id: 2,
            vote: sc_types::Vote::Agree,
            rationale: "valid".into(),
            observed_image_sha256: vec![1, 2, 3],
            agent_label: Some("custom-bot".into()),
            submitted_by: p(99),
        };
        let stamped_rev = prepare_review_submission(rev, p(4), Some("default-bot".into()));
        assert_eq!(stamped_rev.submitted_by, p(4));
        assert_eq!(stamped_rev.agent_label.as_deref(), Some("custom-bot"));
    }
}
