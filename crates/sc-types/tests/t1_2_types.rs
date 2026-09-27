use candid::{decode_one, encode_one, CandidType, Nat, Principal};
use sc_types::*;
use serde::de::DeserializeOwned;
use std::fmt::Debug;

fn roundtrip<T: CandidType + DeserializeOwned + PartialEq + Debug>(label: &str, value: T) {
    let bytes = encode_one(&value).expect("encode");
    let back: T = decode_one(&bytes).expect("decode");
    assert_eq!(back, value, "{label} changed in a candid round trip");
    println!("  ✓ {label}: {} bytes round-trip", bytes.len());
}

fn subject() -> SubjectRef {
    SubjectRef {
        subject_id: 104233,
        field: "ceers".into(),
        ra_deg: 214.9152,
        dec_deg: 52.8741,
        image_url: "https://data.example/v1/subjects/104233/rgb.png".into(),
        image_sha256: vec![7; 32],
        dossier_url: "https://data.example/v1/subjects/104233/dossier.json".into(),
        dossier_sha256: vec![9; 32],
        data_version: 1,
    }
}

fn protocol() -> Protocol {
    Protocol {
        version: 1,
        questions: vec![Question {
            id: "shape".into(),
            prompt: "Smooth, featured, compact or artifact?".into(),
            answers: vec![AnswerOption {
                id: "smooth".into(),
                label: "Smooth".into(),
                next: Some("clumps".into()),
            }],
        }],
        discovery_categories: vec![DiscoveryCategory {
            id: "lensed_arc".into(),
            label: "Lensed arc".into(),
            description: "Gravitational arc or multiple images".into(),
        }],
        guidance_md: "Judge the image, not the text.".into(),
    }
}

fn flag() -> DiscoveryFlag {
    DiscoveryFlag {
        category: "lensed_arc".into(),
        rationale: "A thin tangential arc 2 arcsec north-east of the cluster core.".into(),
        confidence: 80,
        claim_position: Some(ClaimPosition {
            ra_deg: 3.58,
            dec_deg: -30.4,
        }),
    }
}

#[test]
fn t1_2_every_shared_type_survives_a_candid_round_trip() {
    println!("T1.2 demo: every sc-types record encodes and decodes unchanged");
    let agent = Principal::from_slice(&[1, 2, 3]);
    roundtrip("SubjectRef", subject());
    roundtrip("Protocol", protocol());
    roundtrip(
        "Task",
        Task {
            task_id: 1,
            subject: subject(),
            protocol: protocol(),
            lease_expires_at_ns: 42,
        },
    );
    roundtrip(
        "ClassificationSubmission",
        ClassificationSubmission {
            task_id: 1,
            answers: vec![Answer {
                question_id: "shape".into(),
                answer_id: "smooth".into(),
            }],
            observed_image_sha256: vec![7; 32],
            discovery: Some(flag()),
            agent_label: Some("claude-code".into()),
            submitted_by: agent,
        },
    );
    roundtrip(
        "ClassificationReceipt",
        ClassificationReceipt {
            classification_id: 9,
            discovery_id: Some("SC-2026-000001".into()),
            xp_awarded: 3,
            duplicate: false,
            claim: Some(ClaimOutcome::Corroborates("SC-2026-000000".into())),
        },
    );
    roundtrip(
        "ReviewAssignment",
        ReviewAssignment {
            assignment_id: 5,
            subject: subject(),
            protocol_version: 1,
            category: "ring".into(),
            rationale: "untrusted text".into(),
            lease_expires_at_ns: 99,
        },
    );
    roundtrip(
        "ReviewSubmission",
        ReviewSubmission {
            assignment_id: 5,
            vote: Vote::Disagree,
            rationale: "The ring is a diffraction spike artifact.".into(),
            observed_image_sha256: vec![7; 32],
            agent_label: None,
            submitted_by: agent,
        },
    );
    roundtrip(
        "ReviewReceipt",
        ReviewReceipt {
            review_id: 2,
            xp_awarded: 1,
            duplicate: true,
        },
    );
}

#[test]
fn t1_2_api_error_has_every_spec_variant() {
    let all = vec![
        ApiError::Unauthorized,
        ApiError::NotRegistered,
        ApiError::Suspended,
        ApiError::InsufficientFee {
            required: Nat::from(1_000_000u64),
        },
        ApiError::RateLimited {
            retry_after_secs: 30,
        },
        ApiError::LeaseExpired,
        ApiError::LeaseNotFound,
        ApiError::InvalidInput("x".into()),
        ApiError::NotFound,
        ApiError::NotEligible("tier".into()),
        ApiError::Conflict("dup".into()),
        ApiError::Internal("bug".into()),
    ];
    for e in all {
        roundtrip(&format!("ApiError::{e}"), e);
    }
}

#[test]
fn t1_2_input_limits_match_the_security_spec() {
    println!("T1.2 demo: 08 §3 input limits");
    assert_eq!(limits::aaa_name("  Hubble-9  ").unwrap(), "Hubble-9");
    assert!(limits::aaa_name("ab").is_err());
    assert!(limits::aaa_name(&"x".repeat(33)).is_err());
    assert!(limits::aaa_name("bad/name").is_err());
    assert_eq!(limits::name_key(" Hubble-9 "), "hubble-9");
    println!("  ✓ AAA name: 3-32 chars, [A-Za-z0-9 _.-], case-insensitive key");

    assert!(limits::rationale("too short").is_err());
    assert!(limits::rationale(&"a".repeat(1001)).is_err());
    assert!(limits::rationale("line one is long enough\nline two").is_ok());
    assert!(limits::rationale("contains a tab\tcharacter here!!").is_err());
    println!("  ✓ rationale: 20-1000 chars, no control chars except newline");

    assert!(limits::agent_label(&Some("x".repeat(65))).is_err());
    assert!(limits::agent_label(&None).is_ok());
    assert!(limits::operator_label(&"x".repeat(33)).is_err());
    println!("  ✓ agent_label ≤ 64, operator label ≤ 32");

    let a = Answer {
        question_id: "q".into(),
        answer_id: "a".into(),
    };
    assert!(limits::answers(&vec![a.clone(); 16]).is_ok());
    assert!(limits::answers(&vec![a; 17]).is_err());
    assert!(limits::answers(&[]).is_err());
    println!("  ✓ answers: 1-16 entries");

    assert!(limits::sha256(&[0; 31]).is_err());
    assert!(limits::field("ceers").is_ok() && limits::field("sdss").is_err());
    let mut bad = flag();
    bad.confidence = 101;
    assert!(limits::discovery_flag(&bad).is_err());
    assert_eq!(limits::page_limit(1000), 100);
    println!("  ✓ sha256 length, known fields, confidence 0-100, page limit ≤ 100");
}

#[test]
fn t1_2_candid_shapes_match_the_platform_spec() {
    let shape = |t: candid::types::Type| candid::pretty::candid::pp_ty(&t).pretty(200).to_string();
    let subject = shape(SubjectRef::ty());
    for field in [
        "subject_id : nat32",
        "field : text",
        "ra_deg : float64",
        "image_sha256 : blob",
        "dossier_sha256 : blob",
        "data_version : nat16",
    ] {
        assert!(
            subject.contains(field),
            "SubjectRef missing `{field}`: {subject}"
        );
    }
    let sub = shape(ClassificationSubmission::ty());
    assert!(
        sub.contains("submitted_by : principal") && sub.contains("observed_image_sha256 : blob")
    );
    let err = shape(ApiError::ty());
    assert!(
        err.contains("InsufficientFee : record { required : nat }"),
        "{err}"
    );
    println!("  ✓ candid shapes of SubjectRef, ClassificationSubmission and ApiError match 02 §2");
}
