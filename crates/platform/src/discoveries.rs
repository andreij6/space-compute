use std::cell::RefCell;

use candid::{CandidType, Principal};
use ic_stable_structures::StableBTreeMap;
use sc_types::{SubjectRef, Vote};
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

pub fn count_by_queue_status(status: u8) -> u64 {
    QUEUE.with_borrow(|q| {
        q.range((status, 0, 0)..=(status, u64::MAX, u64::MAX))
            .count() as u64
    })
}

pub fn is_visible(d: &Discovery, caller: Principal) -> bool {
    if d.is_honeypot {
        return false;
    }
    if d.status == DiscoveryStatus::UnderReview {
        return caller == d.discoverer_owner || caller == d.discoverer_aaa;
    }
    true
}

#[derive(CandidType, Deserialize, Clone, Debug, Default)]
pub struct ListFilter {
    pub category: Option<String>,
    pub status: Option<DiscoveryStatus>,
}

#[derive(CandidType, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct DiscoveryCard {
    pub public_id: String,
    pub subject_id: u32,
    pub category: String,
    pub rationale: String,
    pub confidence: u8,
    pub status: DiscoveryStatus,
    pub discoverer_aaa: Principal,
    pub discoverer_name: String,
    pub needed_reviews: u8,
    pub created_at: u64,
    pub resolved_at: Option<u64>,
}

impl From<&Discovery> for DiscoveryCard {
    fn from(d: &Discovery) -> Self {
        DiscoveryCard {
            public_id: d.public_id.clone(),
            subject_id: d.subject_id,
            category: d.category.clone(),
            rationale: d.rationale.clone(),
            confidence: d.confidence,
            status: d.status,
            discoverer_aaa: d.discoverer_aaa,
            discoverer_name: d.discoverer_name_at_time.clone(),
            needed_reviews: d.needed_reviews,
            created_at: d.created_at,
            resolved_at: d.resolved_at,
        }
    }
}

#[derive(CandidType, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct ReviewView {
    pub reviewer_aaa: Principal,
    pub reviewer_name: String,
    pub vote: Vote,
    pub rationale: String,
}

#[derive(CandidType, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct DiscoveryView {
    pub public_id: String,
    pub subject: SubjectRef,
    pub category: String,
    pub rationale: String,
    pub confidence: u8,
    pub status: DiscoveryStatus,
    pub reviews_done: u8,
    pub needed_reviews: u8,
    pub discoverer_aaa: Principal,
    pub discoverer_name: String,
    pub discoverer_tier: u8,
    pub created_at: u64,
    pub resolved_at: Option<u64>,
    pub reviews: Vec<ReviewView>,
}

const LIST_SCAN_MAX: usize = 2_000;

pub fn list(
    filter: &ListFilter,
    cursor: Option<u64>,
    limit: u32,
    caller: Principal,
) -> (Vec<Discovery>, Option<u64>) {
    let limit = sc_types::limits::page_limit(limit) as usize;
    let upper = cursor.unwrap_or(u64::MAX);
    let mut matches = Vec::new();
    let mut examined = 0usize;
    let mut last_seq = 0u64;
    let mut exhausted = true;
    DISCOVERIES.with_borrow(|m| {
        for entry in m.range(..upper).rev() {
            if examined >= LIST_SCAN_MAX || matches.len() >= limit {
                exhausted = false;
                break;
            }
            last_seq = *entry.key();
            examined += 1;
            let d = entry.value();
            if !is_visible(&d, caller) {
                continue;
            }
            if let Some(cat) = &filter.category {
                if &d.category != cat {
                    continue;
                }
            }
            if let Some(status) = filter.status {
                if d.status != status {
                    continue;
                }
            }
            matches.push(d);
        }
    });
    let next_cursor = if exhausted { None } else { Some(last_seq) };
    (matches, next_cursor)
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

    fn make(aaa: Principal, owner: Principal, status: DiscoveryStatus, at: u64) -> Discovery {
        let mut d = create(NewDiscovery {
            subject_id: 1,
            classification_id: 1,
            discoverer_aaa: aaa,
            discoverer_owner: owner,
            discoverer_name_at_time: "Secret".into(),
            category: "lens".into(),
            rationale: "an arc around the core".into(),
            confidence: 80,
            fee: 1,
            needed_reviews: 3,
            created_at: at,
            claim_ra_deg: 0.0,
            claim_dec_deg: 0.0,
        });
        d.status = status;
        if status != DiscoveryStatus::UnderReview {
            d.resolved_at = Some(at);
        }
        put(&d);
        d
    }

    #[test]
    fn t4_6_is_visible_hides_under_review_from_non_owners_and_honeypots_from_everyone() {
        let owner = p(1);
        let aaa = p(2);
        let stranger = p(3);
        let under_review = make(
            aaa,
            owner,
            DiscoveryStatus::UnderReview,
            1_790_467_200_000_000_000,
        );
        assert!(is_visible(&under_review, owner));
        assert!(is_visible(&under_review, aaa));
        assert!(!is_visible(&under_review, stranger));

        let confirmed = make(
            p(4),
            p(5),
            DiscoveryStatus::Confirmed,
            1_790_467_200_000_000_000,
        );
        assert!(is_visible(&confirmed, stranger));

        let honeypot = create_honeypot(1, "lens".into(), "r".into(), Vote::Disagree, 1);
        assert!(!is_visible(&honeypot, stranger));
        assert!(!is_visible(&honeypot, Principal::anonymous()));
    }

    #[test]
    fn t4_6_list_hides_under_review_and_honeypots_from_non_owners_shows_owner_their_own() {
        const NOW: u64 = 1_790_467_200_000_000_000;
        let owner = p(10);
        let aaa = p(11);
        let hidden = make(aaa, owner, DiscoveryStatus::UnderReview, NOW);
        let confirmed = make(p(20), p(21), DiscoveryStatus::Confirmed, NOW + 1);
        let rejected = make(p(22), p(23), DiscoveryStatus::Rejected, NOW + 2);
        create_honeypot(1, "lens".into(), "r".into(), Vote::Disagree, NOW + 3);

        let filter = ListFilter::default();
        let (stranger_view, cursor) = list(&filter, None, 100, Principal::anonymous());
        assert!(cursor.is_none());
        let ids: Vec<String> = stranger_view.iter().map(|d| d.public_id.clone()).collect();
        assert!(!ids.contains(&hidden.public_id));
        assert!(ids.contains(&confirmed.public_id));
        assert!(ids.contains(&rejected.public_id));
        assert_eq!(ids.len(), 2);

        let (owner_view, _) = list(&filter, None, 100, owner);
        let owner_ids: Vec<String> = owner_view.iter().map(|d| d.public_id.clone()).collect();
        assert!(owner_ids.contains(&hidden.public_id));
        assert_eq!(owner_ids.len(), 3);
    }

    #[test]
    fn t4_6_list_filters_by_category_and_status_newest_first_with_cursor() {
        const NOW: u64 = 1_790_467_200_000_000_000;
        let a = make(p(30), p(31), DiscoveryStatus::Confirmed, NOW);
        let mut b = create(NewDiscovery {
            subject_id: 2,
            classification_id: 2,
            discoverer_aaa: p(32),
            discoverer_owner: p(33),
            discoverer_name_at_time: "B".into(),
            category: "merger".into(),
            rationale: "an interacting pair".into(),
            confidence: 70,
            fee: 1,
            needed_reviews: 3,
            created_at: NOW + 1,
            claim_ra_deg: 1.0,
            claim_dec_deg: 1.0,
        });
        b.status = DiscoveryStatus::Confirmed;
        b.resolved_at = Some(NOW + 1);
        put(&b);
        let c = make(p(34), p(35), DiscoveryStatus::Confirmed, NOW + 2);

        let filter = ListFilter {
            category: Some("lens".into()),
            status: None,
        };
        let (items, _) = list(&filter, None, 100, Principal::anonymous());
        let ids: Vec<String> = items.iter().map(|d| d.public_id.clone()).collect();
        assert_eq!(ids, vec![c.public_id.clone(), a.public_id.clone()]);

        let by_status = ListFilter {
            category: None,
            status: Some(DiscoveryStatus::Rejected),
        };
        let (none, _) = list(&by_status, None, 100, Principal::anonymous());
        assert!(none.is_empty());

        let all = ListFilter::default();
        let (page1, cursor1) = list(&all, None, 1, Principal::anonymous());
        assert_eq!(page1.len(), 1);
        assert_eq!(page1[0].public_id, c.public_id);
        assert!(cursor1.is_some());
        let (page2, _) = list(&all, cursor1, 1, Principal::anonymous());
        assert_eq!(page2[0].public_id, b.public_id);
    }

    #[test]
    fn t4_6_count_by_queue_status_excludes_honeypots() {
        const NOW: u64 = 1_790_467_200_000_000_000;
        assert_eq!(count_by_queue_status(0), 0);
        assert_eq!(count_by_queue_status(1), 0);
        make(p(40), p(41), DiscoveryStatus::UnderReview, NOW);
        make(p(42), p(43), DiscoveryStatus::Confirmed, NOW);
        create_honeypot(1, "lens".into(), "r".into(), Vote::Disagree, NOW);
        assert_eq!(count_by_queue_status(0), 1);
        assert_eq!(count_by_queue_status(1), 1);
        assert_eq!(count_by_queue_status(QUEUE_HONEYPOT), 1);
    }

    #[test]
    fn t4_6_discovery_card_from_discovery_carries_frozen_discoverer_name() {
        let d = make(
            p(50),
            p(51),
            DiscoveryStatus::Confirmed,
            1_790_467_200_000_000_000,
        );
        let card = DiscoveryCard::from(&d);
        assert_eq!(card.public_id, d.public_id);
        assert_eq!(card.discoverer_name, "Secret");
        assert_eq!(card.status, DiscoveryStatus::Confirmed);
    }
}
