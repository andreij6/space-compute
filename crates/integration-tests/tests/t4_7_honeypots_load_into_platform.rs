use candid::{decode_one, encode_args};
use integration_tests::pic::{user, IcpEnv};
use integration_tests::step;
use platform::catalog::SubjectInput;
use platform::discoveries::{DiscoveryCard, ListFilter};
use platform::events::Page;
use platform::progression::Stats;
use platform::reviews::HoneypotSpec;
use sc_types::{ApiError, SubjectRef, Vote};

fn sample_ref(id: u32) -> SubjectRef {
    SubjectRef {
        subject_id: id,
        field: "ceers".into(),
        ra_deg: 214.9 + (id as f64 * 0.001),
        dec_deg: 52.8 + (id as f64 * 0.001),
        image_url: format!("https://data.example.com/{id}/rgb.png"),
        image_sha256: vec![1; 32],
        dossier_url: format!("https://data.example.com/{id}/dossier.json"),
        dossier_sha256: vec![2; 32],
        data_version: 1,
    }
}

#[test]
fn t4_7_honeypots_load_into_platform() {
    println!("T4.7 demo: honeypots load onto gold subjects and never surface in public queries");
    let env = IcpEnv::new();
    let admin = user(1);
    let platform = env.install("platform", admin);

    let subjects = vec![
        SubjectInput {
            subject: sample_ref(1),
            gold: Some(vec![]),
        },
        SubjectInput {
            subject: sample_ref(2),
            gold: Some(vec![]),
        },
        SubjectInput {
            subject: sample_ref(3),
            gold: None,
        },
    ];
    let added: Result<u32, ApiError> = env.update(platform, admin, "admin_add_subjects", subjects);
    assert_eq!(added, Ok(3));
    step("2 gold subjects and 1 non-gold subject seeded");

    let specs = vec![
        HoneypotSpec {
            subject_id: 1,
            category: "lensed_arc".into(),
            rationale: "clean smooth elliptical claimed as a gravitational lens".into(),
            truth: Vote::Disagree,
        },
        HoneypotSpec {
            subject_id: 2,
            category: "merger_interaction".into(),
            rationale: "double nucleus and tidal tail; a true major merger".into(),
            truth: Vote::Agree,
        },
    ];
    let ok: Result<u32, ApiError> =
        env.update(platform, admin, "admin_add_honeypots", specs.clone());
    assert_eq!(ok, Ok(2));
    step("2 honeypot specs stored via admin_add_honeypots");

    let rejected: Result<u32, ApiError> = env.update(
        platform,
        admin,
        "admin_add_honeypots",
        vec![HoneypotSpec {
            subject_id: 3,
            category: "lensed_arc".into(),
            rationale: "non-gold subject".into(),
            truth: Vote::Disagree,
        }],
    );
    assert!(matches!(rejected, Err(ApiError::InvalidInput(_))));
    step("a non-gold subject is rejected");

    let raw = env
        .pic
        .query_call(
            platform,
            admin,
            "list_discoveries",
            encode_args((ListFilter::default(), Option::<u64>::None, 50u32)).unwrap(),
        )
        .expect("list_discoveries");
    let page: Page<DiscoveryCard> = decode_one(&raw).unwrap();
    assert!(page.items.is_empty());
    step("honeypots are absent from list_discoveries");

    let stats: Stats = env.query(platform, admin, "get_stats", ());
    assert_eq!(stats.under_review_count, 0);
    step("honeypots do not inflate get_stats().under_review_count");
}
