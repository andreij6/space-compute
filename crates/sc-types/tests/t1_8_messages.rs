use candid::{Nat, Principal};
use sc_types::*;

#[test]
fn t1_8_every_api_error_has_a_readable_message() {
    let cases = [
        (ApiError::Unauthorized, "unauthorized"),
        (ApiError::NotRegistered, "not registered"),
        (ApiError::Suspended, "suspended"),
        (
            ApiError::InsufficientFee {
                required: Nat::from(5u8),
            },
            "insufficient fee",
        ),
        (
            ApiError::RateLimited {
                retry_after_secs: 3,
            },
            "retry in 3s",
        ),
        (ApiError::LeaseExpired, "lease expired"),
        (ApiError::LeaseNotFound, "lease not found"),
        (ApiError::invalid("x"), "invalid input: x"),
        (ApiError::NotFound, "not found"),
        (ApiError::NotEligible("tier".into()), "not eligible: tier"),
        (ApiError::Conflict("dup".into()), "conflict: dup"),
        (ApiError::Internal("bug".into()), "internal error: bug"),
    ];
    for (e, text) in cases {
        assert!(e.to_string().contains(text), "{e}");
        let _: &dyn std::error::Error = &e;
    }
}

#[test]
fn t1_8_validators_reject_every_malformed_input() {
    assert!(limits::agent_label(&Some("bad\u{7}".into())).is_err());
    assert!(limits::agent_label(&Some("claude-code".into())).is_ok());
    assert!(limits::operator_label("laptop\n").is_err());
    assert!(limits::operator_label("laptop").is_ok());
    assert!(limits::sha256(&[0; 32]).is_ok());
    let flag = |category: &str, confidence: u8| DiscoveryFlag {
        category: category.into(),
        rationale: "A clear tidal tail extends to the south-west.".into(),
        confidence,
        claim_position: None,
    };
    assert!(limits::discovery_flag(&flag("", 50)).is_err());
    assert!(limits::discovery_flag(&flag(&"c".repeat(65), 50)).is_err());
    assert!(limits::discovery_flag(&flag("tidal_feature", 50)).is_ok());
    assert_eq!(limits::page_limit(0), 1);
    assert_eq!(build_version("platform"), "platform 0.1.0");
    let _ = Principal::anonymous();
}
