use std::cell::RefCell;

use candid::{CandidType, Principal};
use ic_stable_structures::StableBTreeMap;
use serde::{Deserialize, Serialize};

use crate::discoveries::{self, Discovery, DiscoveryStatus};
use crate::memory::{self, Memory};

const NS_PER_DAY: u64 = 86_400_000_000_000;

#[derive(CandidType, Serialize, Deserialize, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct ClaimKey {
    pub field: String,
    pub cell_x: i32,
    pub cell_y: i32,
    pub category: String,
}

crate::candid_storable!(ClaimKey);

#[derive(CandidType, Serialize, Deserialize, Clone, Debug, Default)]
struct Seqs(Vec<u64>);

crate::candid_storable!(Seqs);

#[derive(CandidType, Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct Corroboration {
    pub aaa: Principal,
    pub owner: Principal,
    pub classification_id: u64,
    pub at: u64,
}

#[derive(CandidType, Serialize, Deserialize, Clone, Debug, Default)]
struct Corroborations(Vec<Corroboration>);

crate::candid_storable!(Corroborations);

thread_local! {
    static CLAIM_INDEX: RefCell<StableBTreeMap<ClaimKey, Seqs, Memory>> =
        RefCell::new(StableBTreeMap::init(memory::get(memory::CLAIM_INDEX)));
    static CORROBORATIONS: RefCell<StableBTreeMap<u64, Corroborations, Memory>> =
        RefCell::new(StableBTreeMap::init(memory::get(memory::CORROBORATIONS)));
}

pub fn cell(ra_deg: f64, dec_deg: f64, cell_arcsec: f64) -> (i32, i32) {
    let x = (ra_deg * dec_deg.to_radians().cos() * 3600.0 / cell_arcsec).floor();
    let y = (dec_deg * 3600.0 / cell_arcsec).floor();
    (x as i32, y as i32)
}

#[derive(Debug, PartialEq)]
pub enum Resolution {
    New,
    Corroborate(Discovery),
    ClosedRecentlyRejected(Discovery),
    Noop,
}

pub struct ClaimQuery<'a> {
    pub field: &'a str,
    pub cell: (i32, i32),
    pub category: &'a str,
    pub caller: Principal,
    pub reopen_days: u32,
    pub now: u64,
    pub ra_deg: f64,
    pub dec_deg: f64,
    pub unique_radius_arcsec: f64,
}

pub fn separation_arcsec(ra1: f64, dec1: f64, ra2: f64, dec2: f64) -> f64 {
    let dra = (ra1 - ra2) * dec1.to_radians().cos();
    let ddec = dec1 - dec2;
    (dra * dra + ddec * ddec).sqrt() * 3600.0
}

fn neighbours(q: &ClaimQuery) -> Vec<(Discovery, bool)> {
    let mut seqs: Vec<(u64, bool)> = CLAIM_INDEX.with_borrow(|m| {
        (-1..=1)
            .flat_map(|dx| (-1..=1).map(move |dy| (dx, dy)))
            .filter_map(|(dx, dy)| {
                m.get(&ClaimKey {
                    field: q.field.to_string(),
                    cell_x: q.cell.0.saturating_add(dx),
                    cell_y: q.cell.1.saturating_add(dy),
                    category: q.category.to_string(),
                })
                .map(|s| (s, dx == 0 && dy == 0))
            })
            .flat_map(|(s, same)| s.0.into_iter().map(move |seq| (seq, same)))
            .collect()
    });
    seqs.sort_unstable();
    seqs.dedup_by_key(|(seq, _)| *seq);
    seqs.into_iter()
        .filter_map(|(seq, same)| discoveries::get(seq).map(|d| (d, same)))
        .collect()
}

pub fn resolve(q: &ClaimQuery) -> Resolution {
    let found: Vec<Discovery> = neighbours(q)
        .into_iter()
        .filter(|(d, same_cell)| match (d.claim_ra_deg, d.claim_dec_deg) {
            (Some(ra), Some(dec)) => {
                separation_arcsec(q.ra_deg, q.dec_deg, ra, dec) <= q.unique_radius_arcsec
            }
            _ => *same_cell,
        })
        .map(|(d, _)| d)
        .collect();
    if let Some(open) = found.iter().find(|d| d.status != DiscoveryStatus::Rejected) {
        let already = open.discoverer_aaa == q.caller
            || corroborations(open.seq).iter().any(|c| c.aaa == q.caller);
        return if already {
            Resolution::Noop
        } else {
            Resolution::Corroborate(open.clone())
        };
    }
    let window = q.reopen_days as u64 * NS_PER_DAY;
    found
        .into_iter()
        .filter(|d| {
            d.resolved_at
                .is_some_and(|r| q.now.saturating_sub(r) < window)
        })
        .max_by_key(|d| d.resolved_at)
        .map_or(Resolution::New, Resolution::ClosedRecentlyRejected)
}

pub fn index(field: &str, cell: (i32, i32), category: &str, seq: u64) {
    let key = ClaimKey {
        field: field.to_string(),
        cell_x: cell.0,
        cell_y: cell.1,
        category: category.to_string(),
    };
    CLAIM_INDEX.with_borrow_mut(|m| {
        let mut seqs = m.get(&key).unwrap_or_default();
        seqs.0.push(seq);
        m.insert(key, seqs);
    });
}

pub fn corroborate(seq: u64, c: Corroboration) {
    CORROBORATIONS.with_borrow_mut(|m| {
        let mut list = m.get(&seq).unwrap_or_default();
        list.0.push(c);
        m.insert(seq, list);
    });
}

pub fn corroborations(seq: u64) -> Vec<Corroboration> {
    CORROBORATIONS.with_borrow(|m| m.get(&seq).map(|c| c.0).unwrap_or_default())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::discoveries::NewDiscovery;

    const NOW: u64 = 1_790_467_200_000_000_000;

    fn p(n: u8) -> Principal {
        Principal::from_slice(&[n; 29])
    }

    fn new_discovery_at(aaa: Principal, category: &str, ra: f64, dec: f64) -> Discovery {
        discoveries::create(NewDiscovery {
            subject_id: 1,
            classification_id: 1,
            discoverer_aaa: aaa,
            discoverer_owner: aaa,
            discoverer_name_at_time: "A".into(),
            category: category.into(),
            rationale: "arc".into(),
            confidence: 50,
            fee: 0,
            needed_reviews: 3,
            created_at: NOW,
            claim_ra_deg: Some(ra),
            claim_dec_deg: Some(dec),
        })
    }

    fn new_discovery(aaa: Principal, category: &str) -> Discovery {
        new_discovery_at(aaa, category, 0.0, 0.0)
    }

    fn query_at<'a>(
        caller: Principal,
        category: &'a str,
        cell: (i32, i32),
        now: u64,
        ra: f64,
        dec: f64,
    ) -> ClaimQuery<'a> {
        ClaimQuery {
            field: "ceers",
            cell,
            category,
            caller,
            reopen_days: 30,
            now,
            ra_deg: ra,
            dec_deg: dec,
            unique_radius_arcsec: 1.5,
        }
    }

    fn query(caller: Principal, category: &str, cell: (i32, i32), now: u64) -> ClaimQuery<'_> {
        query_at(caller, category, cell, now, 0.0, 0.0)
    }

    #[test]
    fn t4_9_claim_cell_math_matches_spec_including_negative_dec_and_boundaries() {
        assert_eq!(cell(0.0, 0.0, 1.5), (0, 0));
        assert_eq!(cell(0.0, 1.5 / 3600.0, 1.5), (0, 1));
        assert_eq!(cell(0.0, 1.4999 / 3600.0, 1.5), (0, 0));
        assert_eq!(cell(0.0, -0.0001 / 3600.0, 1.5), (0, -1));
        assert_eq!(cell(0.0, -1.5 / 3600.0, 1.5), (0, -1));
        assert_eq!(cell(0.0, -1.5001 / 3600.0, 1.5), (0, -2));
        assert_eq!(cell(1.0, 0.0, 1.5), (2400, 0));
        let (x, y) = cell(214.9, -52.8, 1.5);
        let expected_x = (214.9 * (-52.8f64).to_radians().cos() * 2400.0).floor() as i32;
        assert_eq!((x, y), (expected_x, (-52.8f64 * 2400.0).floor() as i32));
        assert_eq!(cell(214.9, -52.8, 1.5).0, cell(214.9, 52.8, 1.5).0);
        assert_eq!(cell(10.0, 60.0, 3.0), (6000, 72_000));
    }

    #[test]
    fn t4_9_resolve_corroborates_open_claim_in_neighbouring_cells_and_noops_repeat_flaggers() {
        let d = new_discovery(p(1), "lens");
        index("ceers", (100, 200), "lens", d.seq);
        assert_eq!(
            resolve(&query(p(2), "lens", (101, 199), NOW)),
            Resolution::Corroborate(d.clone())
        );
        assert_eq!(
            resolve(&query(p(2), "lens", (102, 200), NOW)),
            Resolution::New
        );
        assert_eq!(
            resolve(&query(p(2), "merger", (100, 200), NOW)),
            Resolution::New
        );
        let other_field = ClaimQuery {
            field: "cosmos",
            ..query(p(2), "lens", (100, 200), NOW)
        };
        assert_eq!(resolve(&other_field), Resolution::New);
        assert_eq!(
            resolve(&query(p(1), "lens", (100, 200), NOW)),
            Resolution::Noop
        );
        corroborate(
            d.seq,
            Corroboration {
                aaa: p(2),
                owner: p(2),
                classification_id: 2,
                at: NOW,
            },
        );
        assert_eq!(
            resolve(&query(p(2), "lens", (100, 200), NOW)),
            Resolution::Noop
        );
        assert_eq!(corroborations(d.seq).len(), 1);
        assert!(corroborations(999).is_empty());
    }

    #[test]
    fn t4_9_rejected_claim_blocks_within_reopen_window_then_reopens() {
        let mut d = new_discovery(p(1), "lens");
        index("ceers", (5, -5), "lens", d.seq);
        d.status = DiscoveryStatus::Rejected;
        d.resolved_at = Some(NOW);
        discoveries::put(&d);
        let day = NS_PER_DAY;
        assert_eq!(
            resolve(&query(p(2), "lens", (5, -5), NOW + 29 * day)),
            Resolution::ClosedRecentlyRejected(d.clone())
        );
        assert_eq!(
            resolve(&query(p(2), "lens", (5, -5), NOW + 30 * day)),
            Resolution::New
        );
        let mut confirmed = new_discovery(p(3), "lens");
        confirmed.status = DiscoveryStatus::Confirmed;
        discoveries::put(&confirmed);
        index("ceers", (5, -4), "lens", confirmed.seq);
        assert_eq!(
            resolve(&query(p(2), "lens", (5, -5), NOW + 29 * day)),
            Resolution::Corroborate(confirmed)
        );
    }

    #[test]
    fn t4_9_distinct_objects_in_the_same_bucket_are_fundamentally_unique() {
        let d = new_discovery_at(p(1), "lens", 214.9, -52.8);
        index("ceers", (100, 200), "lens", d.seq);
        let near = query_at(p(2), "lens", (100, 200), NOW, 214.9, -52.8 + 0.2 / 3600.0);
        assert_eq!(resolve(&near), Resolution::Corroborate(d.clone()));
        let far = query_at(p(2), "lens", (100, 200), NOW, 214.9, -52.8 + 3.0 / 3600.0);
        assert_eq!(resolve(&far), Resolution::New);
    }

    #[test]
    fn t4_9_separation_arcsec_scales_ra_by_cos_dec() {
        assert!((separation_arcsec(0.0, 0.0, 1.0 / 3600.0, 0.0) - 1.0).abs() < 1e-9);
        assert!((separation_arcsec(0.0, 0.0, 0.0, 1.0 / 3600.0) - 1.0).abs() < 1e-9);
        assert_eq!(separation_arcsec(10.0, -50.0, 10.0, -50.0), 0.0);
    }

    #[test]
    fn t4_9_l032_claim_without_exact_position_falls_back_to_same_cell_match() {
        let mut d = new_discovery(p(1), "lens");
        d.claim_ra_deg = None;
        d.claim_dec_deg = None;
        discoveries::put(&d);
        index("ceers", (7, 7), "lens", d.seq);
        let far_same_cell = query_at(p(2), "lens", (7, 7), NOW, 50.0, 50.0);
        assert_eq!(resolve(&far_same_cell), Resolution::Corroborate(d.clone()));
        let neighbour = query_at(p(2), "lens", (8, 7), NOW, 0.0, 0.0);
        assert_eq!(resolve(&neighbour), Resolution::New);
    }

    #[test]
    fn t4_9_l032_legacy_f64_claim_position_decodes_as_some() {
        #[derive(CandidType)]
        struct Legacy {
            claim_ra_deg: f64,
            claim_dec_deg: f64,
        }
        #[derive(CandidType, Deserialize)]
        struct Current {
            claim_ra_deg: Option<f64>,
            claim_dec_deg: Option<f64>,
        }
        let bytes = candid::encode_one(Legacy {
            claim_ra_deg: 214.9,
            claim_dec_deg: -52.8,
        })
        .unwrap();
        let back: Current = candid::decode_one(&bytes).unwrap();
        assert_eq!(back.claim_ra_deg, Some(214.9));
        assert_eq!(back.claim_dec_deg, Some(-52.8));
    }
}
