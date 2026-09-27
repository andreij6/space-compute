use std::cell::RefCell;

use candid::{CandidType, Principal};
use ic_stable_structures::StableBTreeMap;
use sc_types::Vote;
use serde::{Deserialize, Serialize};

use crate::memory::{self, Memory};

const NS_PER_DAY: i64 = 86_400_000_000_000;

#[derive(CandidType, Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub enum DiscoveryStatus {
    UnderReview,
    Confirmed,
    Rejected,
}

#[derive(CandidType, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Discovery {
    pub v: u8,
    pub seq: u64,
    pub public_id: String,
    pub subject_id: u32,
    pub classification_id: u64,
    pub discoverer_aaa: Principal,
    pub discoverer_owner: Principal,
    pub discoverer_name_at_time: String,
    pub category: String,
    pub rationale: String,
    pub confidence: u8,
    pub fee: u128,
    pub status: DiscoveryStatus,
    pub needed_reviews: u8,
    pub created_at: u64,
    pub resolved_at: Option<u64>,
    pub is_honeypot: bool,
    pub honeypot_truth: Option<Vote>,
    pub claim_ra_deg: f64,
    pub claim_dec_deg: f64,
}

crate::candid_storable!(Discovery);

thread_local! {
    static DISCOVERIES: RefCell<StableBTreeMap<u64, Discovery, Memory>> =
        RefCell::new(StableBTreeMap::init(memory::get(memory::DISCOVERIES)));
    static PUBLIC_IDS: RefCell<StableBTreeMap<String, u64, Memory>> =
        RefCell::new(StableBTreeMap::init(memory::get(memory::PUBLIC_ID_INDEX)));
    static QUEUE: RefCell<StableBTreeMap<(u8, u64, u64), (), Memory>> =
        RefCell::new(StableBTreeMap::init(memory::get(memory::REVIEW_QUEUE)));
}

pub const QUEUE_HONEYPOT: u8 = 3;

fn queue_status(d: &Discovery) -> u8 {
    if d.is_honeypot {
        return QUEUE_HONEYPOT;
    }
    match d.status {
        DiscoveryStatus::UnderReview => 0,
        DiscoveryStatus::Confirmed => 1,
        DiscoveryStatus::Rejected => 2,
    }
}

fn store(d: &Discovery) {
    if let Some(old) = get(d.seq) {
        QUEUE.with_borrow_mut(|q| q.remove(&(queue_status(&old), old.created_at, old.seq)));
    }
    QUEUE.with_borrow_mut(|q| q.insert((queue_status(d), d.created_at, d.seq), ()));
    DISCOVERIES.with_borrow_mut(|m| m.insert(d.seq, d.clone()));
}

pub fn update(d: &Discovery) {
    store(d);
}

pub fn find_queued(status: u8, mut pred: impl FnMut(&Discovery) -> bool) -> Option<Discovery> {
    QUEUE.with_borrow(|q| {
        q.range((status, 0, 0)..=(status, u64::MAX, u64::MAX))
            .filter_map(|e| get(e.key().2))
            .find(|d| pred(d))
    })
}

pub fn under_review_created_before(cutoff: u64) -> Vec<Discovery> {
    QUEUE.with_borrow(|q| {
        q.range((0, 0, 0)..=(0, cutoff, u64::MAX))
            .filter_map(|e| get(e.key().2))
            .collect()
    })
}

pub fn create_honeypot(
    subject_id: u32,
    category: String,
    rationale: String,
    truth: Vote,
    now: u64,
) -> Discovery {
    let d = Discovery {
        v: 1,
        seq: next_seq(),
        public_id: String::new(),
        subject_id,
        classification_id: 0,
        discoverer_aaa: Principal::anonymous(),
        discoverer_owner: Principal::anonymous(),
        discoverer_name_at_time: String::new(),
        category,
        rationale,
        confidence: 0,
        fee: 0,
        status: DiscoveryStatus::UnderReview,
        needed_reviews: 0,
        created_at: now,
        resolved_at: None,
        is_honeypot: true,
        honeypot_truth: Some(truth),
        claim_ra_deg: 0.0,
        claim_dec_deg: 0.0,
    };
    store(&d);
    d
}

fn civil_from_days(days_since_epoch: i64) -> (i64, i64, i64) {
    let z = days_since_epoch + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    (if m <= 2 { y + 1 } else { y }, m, d)
}

pub fn year_from_ns(ns: u64) -> i64 {
    date_from_ns(ns).0
}

pub fn date_from_ns(ns: u64) -> (i64, i64, i64) {
    civil_from_days((ns / NS_PER_DAY as u64) as i64)
}

fn next_seq() -> u64 {
    DISCOVERIES.with_borrow(|m| m.last_key_value().map(|(k, _)| k + 1).unwrap_or(1))
}

pub struct NewDiscovery {
    pub subject_id: u32,
    pub classification_id: u64,
    pub discoverer_aaa: Principal,
    pub discoverer_owner: Principal,
    pub discoverer_name_at_time: String,
    pub category: String,
    pub rationale: String,
    pub confidence: u8,
    pub fee: u128,
    pub needed_reviews: u8,
    pub created_at: u64,
    pub claim_ra_deg: f64,
    pub claim_dec_deg: f64,
}

pub fn create(input: NewDiscovery) -> Discovery {
    let seq = next_seq();
    let year = year_from_ns(input.created_at);
    let public_id = format!("SC-{year:04}-{seq:06}");
    let discovery = Discovery {
        v: 1,
        seq,
        public_id: public_id.clone(),
        subject_id: input.subject_id,
        classification_id: input.classification_id,
        discoverer_aaa: input.discoverer_aaa,
        discoverer_owner: input.discoverer_owner,
        discoverer_name_at_time: input.discoverer_name_at_time,
        category: input.category,
        rationale: input.rationale,
        confidence: input.confidence,
        fee: input.fee,
        status: DiscoveryStatus::UnderReview,
        needed_reviews: input.needed_reviews,
        created_at: input.created_at,
        resolved_at: None,
        is_honeypot: false,
        honeypot_truth: None,
        claim_ra_deg: input.claim_ra_deg,
        claim_dec_deg: input.claim_dec_deg,
    };
    store(&discovery);
    PUBLIC_IDS.with_borrow_mut(|m| m.insert(public_id, seq));
    discovery
}

#[cfg(test)]
pub fn put(d: &Discovery) {
    store(d);
}

pub fn get(seq: u64) -> Option<Discovery> {
    DISCOVERIES.with_borrow(|m| m.get(&seq))
}

pub fn get_by_public_id(public_id: &str) -> Option<Discovery> {
    let seq = PUBLIC_IDS.with_borrow(|m| m.get(&public_id.to_string()))?;
    get(seq)
}

pub fn count() -> u64 {
    DISCOVERIES.with_borrow(|m| m.len())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn t4_1_year_from_ns_matches_known_dates() {
        assert_eq!(year_from_ns(0), 1970);
        assert_eq!(year_from_ns(1_735_689_600_000_000_000), 2025);
        assert_eq!(year_from_ns(1_798_761_600_000_000_000), 2027);
        assert_eq!(year_from_ns(1_767_225_599_000_000_000), 2025);
        assert_eq!(year_from_ns(1_767_225_600_000_000_000), 2026);
    }

    fn p(n: u8) -> Principal {
        Principal::from_slice(&[n; 29])
    }

    #[test]
    fn t4_1_create_assigns_monotonic_seq_and_well_formed_public_id() {
        let d1 = create(NewDiscovery {
            subject_id: 1,
            classification_id: 10,
            discoverer_aaa: p(1),
            discoverer_owner: p(2),
            discoverer_name_at_time: "Alice-AAA".into(),
            category: "lens".into(),
            rationale: "arc visible".into(),
            confidence: 80,
            fee: 200_000_000,
            needed_reviews: 3,
            created_at: 1_790_467_200_000_000_000,
            claim_ra_deg: 214.9,
            claim_dec_deg: -52.8,
        });
        assert_eq!(d1.seq, 1);
        assert_eq!(d1.public_id, "SC-2026-000001");
        assert_eq!(d1.status, DiscoveryStatus::UnderReview);
        assert_eq!(d1.needed_reviews, 3);
        assert!(!d1.is_honeypot);
        assert_eq!(d1.honeypot_truth, None);

        let d2 = create(NewDiscovery {
            subject_id: 2,
            classification_id: 11,
            discoverer_aaa: p(1),
            discoverer_owner: p(2),
            discoverer_name_at_time: "Alice-AAA".into(),
            category: "lens".into(),
            rationale: "another arc".into(),
            confidence: 60,
            fee: 200_000_000,
            needed_reviews: 3,
            created_at: 1_790_467_200_000_000_000,
            claim_ra_deg: 215.0,
            claim_dec_deg: -52.9,
        });
        assert_eq!(d2.seq, 2);
        assert_eq!(d2.public_id, "SC-2026-000002");

        assert_eq!(get(1), Some(d1.clone()));
        assert_eq!(get_by_public_id("SC-2026-000001"), Some(d1));
        assert_eq!(get_by_public_id("SC-2026-000002"), Some(d2));
        assert_eq!(get_by_public_id("nope"), None);
        assert_eq!(count(), 2);
    }
}
