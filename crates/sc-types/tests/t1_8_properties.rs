use candid::{decode_one, encode_one};
use proptest::prelude::*;
use sc_types::{limits, SubjectRef};

proptest! {
    #![proptest_config(ProptestConfig { failure_persistence: None, ..ProptestConfig::default() })]
    #[test]
    fn t1_8_prop_name_validation_never_panics_and_is_idempotent(raw in ".{0,64}") {
        if let Ok(name) = limits::aaa_name(&raw) {
            prop_assert_eq!(limits::aaa_name(&name).unwrap(), name.clone());
            prop_assert_eq!(limits::name_key(&limits::name_key(&name)), limits::name_key(&name));
        }
    }

    #[test]
    fn t1_8_prop_rationale_accepts_exactly_the_spec_range(len in 0usize..1200) {
        let text = "a".repeat(len);
        prop_assert_eq!(limits::rationale(&text).is_ok(), (20..=1000).contains(&len));
    }

    #[test]
    fn t1_8_prop_subject_ref_round_trips(
        id in any::<u32>(), ra in 0.0f64..360.0, dec in -90.0f64..90.0,
        img in proptest::collection::vec(any::<u8>(), 32), ver in any::<u16>()
    ) {
        let s = SubjectRef {
            subject_id: id, field: "ceers".into(), ra_deg: ra, dec_deg: dec,
            image_url: "u".into(), image_sha256: img.clone(), dossier_url: "d".into(),
            dossier_sha256: img, data_version: ver,
        };
        let back: SubjectRef = decode_one(&encode_one(&s).unwrap()).unwrap();
        prop_assert_eq!(back, s);
    }
}
