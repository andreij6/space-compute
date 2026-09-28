use std::cell::RefCell;

use candid::CandidType;
use ic_stable_structures::{StableBTreeMap, StableCell};
use sc_types::ApiError;
use serde::{Deserialize, Serialize};

use crate::memory::{self, Memory};
use crate::record::{
    CreditCopy, ListRecordsFilter, Outcome, PageCreditCopy, PageRecord, Record, RecordKind, Stats,
    StoredSubject,
};

pub const MAX_RECORDS_QUOTA: u64 = 1_000_000;
pub const MAX_PENDING_SUBJECTS: u64 = 256;

#[derive(
    CandidType, Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord,
)]
pub struct IdemKey {
    pub review: bool,
    pub id: u64,
}

crate::candid_storable!(IdemKey);

thread_local! {
    static RECORDS: RefCell<StableBTreeMap<u64, Record, Memory>> =
        RefCell::new(StableBTreeMap::init(memory::get(memory::RECORDS)));
    static IDEMPOTENCY: RefCell<StableBTreeMap<IdemKey, u64, Memory>> =
        RefCell::new(StableBTreeMap::init(memory::get(memory::IDEMPOTENCY)));
    static STATS: RefCell<StableCell<Stats, Memory>> =
        RefCell::new(StableCell::init(memory::get(memory::STATS), Stats::default()));
    static CREDITS: RefCell<StableBTreeMap<String, CreditCopy, Memory>> =
        RefCell::new(StableBTreeMap::init(memory::get(memory::CREDITS)));
    static DISCOVERY_INDEX: RefCell<StableBTreeMap<String, u64, Memory>> =
        RefCell::new(StableBTreeMap::init(memory::get(memory::DISCOVERY_INDEX)));
    static PENDING_SUBJECTS: RefCell<StableBTreeMap<IdemKey, StoredSubject, Memory>> =
        RefCell::new(StableBTreeMap::init(memory::get(memory::PENDING_SUBJECTS)));
}

pub fn insert_record(mut record: Record) -> u64 {
    STATS.with_borrow_mut(|s| {
        let mut stats = s.get().clone();
        stats.total_records = stats.total_records.saturating_add(1);
        let seq = stats.total_records;
        record.seq = seq;

        match record.kind {
            RecordKind::Classification => {
                stats.classifications = stats.classifications.saturating_add(1);
            }
            RecordKind::Discovery => {
                stats.discoveries = stats.discoveries.saturating_add(1);
            }
            RecordKind::Review => {
                stats.reviews = stats.reviews.saturating_add(1);
            }
            RecordKind::TopUp => {
                stats.topups = stats.topups.saturating_add(1);
            }
            _ => {}
        }
        stats.last_activity_at = record.at;
        s.set(stats);

        if let Some(id) = record.task_or_assignment_id {
            let key = IdemKey {
                review: record.kind == RecordKind::Review,
                id,
            };
            IDEMPOTENCY.with_borrow_mut(|idx| {
                idx.insert(key, seq);
            });
        }

        if let Some(public_id) = record.discovery_public_id.clone() {
            DISCOVERY_INDEX.with_borrow_mut(|idx| {
                idx.insert(public_id, seq);
            });
        }

        RECORDS.with_borrow_mut(|r| {
            r.insert(seq, record);
        });

        prune_oldest_classifications_if_needed(MAX_RECORDS_QUOTA);

        seq
    })
}

pub fn get_record(seq: u64) -> Option<Record> {
    RECORDS.with_borrow(|r| r.get(&seq))
}

fn get_seq(review: bool, id: u64) -> Option<u64> {
    IDEMPOTENCY.with_borrow(|idx| idx.get(&IdemKey { review, id }))
}

pub fn get_seq_by_task(task_id: u64) -> Option<u64> {
    get_seq(false, task_id)
}

pub fn get_seq_by_review(assignment_id: u64) -> Option<u64> {
    get_seq(true, assignment_id)
}

pub fn remember_subject(key: IdemKey, subject: sc_types::SubjectRef) {
    PENDING_SUBJECTS.with_borrow_mut(|m| {
        while m.len() >= MAX_PENDING_SUBJECTS && !m.contains_key(&key) {
            m.pop_first();
        }
        m.insert(key, StoredSubject(subject));
    });
}

pub fn pending_subject(key: IdemKey) -> Option<sc_types::SubjectRef> {
    PENDING_SUBJECTS.with_borrow(|m| m.get(&key).map(|s| s.0))
}

pub fn take_subject(key: IdemKey) -> Option<sc_types::SubjectRef> {
    PENDING_SUBJECTS.with_borrow_mut(|m| m.remove(&key).map(|s| s.0))
}

pub fn count_pending_subjects() -> u64 {
    PENDING_SUBJECTS.with_borrow(|m| m.len())
}

pub fn count_records() -> u64 {
    RECORDS.with_borrow(|r| r.len())
}

pub fn list_records(filter: ListRecordsFilter) -> PageRecord {
    let limit = (filter.limit as usize).clamp(1, 100);
    RECORDS.with_borrow(|r| {
        let mut items = Vec::with_capacity(limit);
        let mut next_cursor = None;

        macro_rules! collect {
            ($iter:expr) => {
                for entry in $iter {
                    let seq = *entry.key();
                    let rec = entry.value();
                    if let Some(kind) = filter.kind {
                        if rec.kind != kind {
                            continue;
                        }
                    }
                    if items.len() < limit {
                        items.push(rec);
                    } else {
                        next_cursor = Some(seq);
                        break;
                    }
                }
            };
        }

        match filter.cursor {
            Some(cur) => collect!(r.range(..cur).rev()),
            None => collect!(r.iter().rev()),
        }

        PageRecord { items, next_cursor }
    })
}

pub fn update_record_outcome(seq: u64, outcome: Outcome) -> Result<(), ApiError> {
    RECORDS.with_borrow_mut(|r| {
        let mut rec = r.get(&seq).ok_or(ApiError::NotFound)?;
        rec.outcome = Some(outcome);
        r.insert(seq, rec);
        Ok(())
    })
}

pub fn upsert_credit(credit: CreditCopy) {
    if let Some(seq) = DISCOVERY_INDEX.with_borrow(|idx| idx.get(&credit.public_id)) {
        let _ = update_record_outcome(seq, credit.outcome);
    }
    CREDITS.with_borrow_mut(|c| {
        c.insert(credit.public_id.clone(), credit);
    });
}

pub fn get_credit(public_id: &str) -> Option<CreditCopy> {
    CREDITS.with_borrow(|c| c.get(&public_id.to_string()))
}

pub fn count_credits() -> u64 {
    CREDITS.with_borrow(|c| c.len())
}

pub fn list_credits(cursor: Option<String>, limit: u16) -> PageCreditCopy {
    let limit = (limit as usize).clamp(1, 100);
    CREDITS.with_borrow(|c| {
        let mut items = Vec::new();
        let mut next_cursor = None;

        let mut start_collecting = cursor.is_none();
        for entry in c.iter() {
            let key = entry.key();
            if !start_collecting {
                if Some(key) == cursor.as_ref() {
                    start_collecting = true;
                }
                continue;
            }
            if items.len() < limit {
                items.push(entry.value());
            } else {
                next_cursor = Some(key.clone());
                break;
            }
        }

        PageCreditCopy { items, next_cursor }
    })
}

pub fn prune_oldest_classifications_if_needed(target_quota: u64) {
    RECORDS.with_borrow_mut(|r| {
        if r.len() > target_quota {
            let mut to_remove = Vec::new();
            for entry in r.iter() {
                if entry.value().kind == RecordKind::Classification {
                    to_remove.push(*entry.key());
                    if r.len() - (to_remove.len() as u64) <= target_quota {
                        break;
                    }
                }
            }
            for seq in to_remove {
                if let Some(rec) = r.remove(&seq) {
                    if let Some(id) = rec.task_or_assignment_id {
                        let key = IdemKey {
                            review: rec.kind == RecordKind::Review,
                            id,
                        };
                        IDEMPOTENCY.with_borrow_mut(|idx| {
                            idx.remove(&key);
                        });
                    }
                    if let Some(public_id) = rec.discovery_public_id {
                        DISCOVERY_INDEX.with_borrow_mut(|idx| {
                            idx.remove(&public_id);
                        });
                    }
                }
            }
        }
    });
}

pub fn get_stats() -> Stats {
    STATS.with_borrow(|s| s.get().clone())
}

pub fn update_stats<R>(f: impl FnOnce(&mut Stats) -> R) -> R {
    STATS.with_borrow_mut(|s| {
        let mut stats = s.get().clone();
        let out = f(&mut stats);
        s.set(stats);
        out
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::record::CreditRole;
    use candid::Principal;

    fn blank_record(kind: RecordKind, task_or_assignment_id: Option<u64>, at: u64) -> Record {
        Record {
            v: 1,
            seq: 0,
            at,
            kind,
            task_or_assignment_id,
            subject: None,
            answers: Vec::new(),
            discovery_public_id: None,
            category: None,
            vote: None,
            rationale: None,
            outcome: None,
            xp_awarded: 10,
            agent_label: None,
            fee: 200_000_000,
            by: Principal::anonymous(),
        }
    }

    #[test]
    fn t3_2_insert_record_increments_seq_and_populates_idempotency() {
        let r = blank_record(RecordKind::Classification, Some(42), 1_000);
        let seq = insert_record(r);
        assert!(seq > 0);
        let fetched = get_record(seq).expect("record exists");
        assert_eq!(fetched.seq, seq);
        assert_eq!(fetched.task_or_assignment_id, Some(42));
        assert_eq!(get_seq_by_task(42), Some(seq));
        assert_eq!(get_seq_by_task(99), None);
    }

    #[test]
    fn t3_2_stats_counters_bump_correctly() {
        let mut r_disc = blank_record(RecordKind::Discovery, Some(43), 2_000);
        r_disc.discovery_public_id = Some("DISC-1".into());
        r_disc.category = Some("ring".into());
        r_disc.rationale = Some("test rationale".into());
        r_disc.xp_awarded = 25;
        insert_record(r_disc);
        let stats = get_stats();
        assert!(stats.discoveries >= 1);
        assert_eq!(stats.last_activity_at, 2_000);
    }

    #[test]
    fn t3_3_list_records_pagination_and_filter() {
        let r1 = blank_record(RecordKind::Classification, Some(101), 3_000);
        let mut r2 = blank_record(RecordKind::Review, Some(102), 3_001);
        r2.xp_awarded = 15;
        let s1 = insert_record(r1);
        let s2 = insert_record(r2);
        assert!(s2 > s1);

        let page = list_records(ListRecordsFilter {
            kind: None,
            cursor: None,
            limit: 1,
        });
        assert_eq!(page.items.len(), 1);
        assert_eq!(page.items[0].seq, s2);
        assert!(page.next_cursor.is_some());

        let page_reviews = list_records(ListRecordsFilter {
            kind: Some(RecordKind::Review),
            cursor: None,
            limit: 10,
        });
        assert!(page_reviews
            .items
            .iter()
            .all(|r| r.kind == RecordKind::Review));
    }

    #[test]
    fn t3_3_upsert_credit_and_list_credits() {
        let mut disc = blank_record(RecordKind::Discovery, Some(9_500), 4_900);
        disc.discovery_public_id = Some("JWST-CEERS-DISC-001".into());
        insert_record(disc);

        let credit = CreditCopy {
            v: 1,
            public_id: "JWST-CEERS-DISC-001".into(),
            category: "ring".into(),
            role: CreditRole::Discoverer,
            outcome: Outcome::Confirmed,
            at: 5_000,
            subject_id: 1,
            citation_url: Some("https://data.example.com/c/001".into()),
        };
        upsert_credit(credit.clone());
        assert_eq!(get_credit("JWST-CEERS-DISC-001"), Some(credit));

        let page = list_credits(None, 10);
        assert!(!page.items.is_empty());
        assert!(page
            .items
            .iter()
            .any(|c| c.public_id == "JWST-CEERS-DISC-001"));
    }

    #[test]
    fn t3_3_record_outcome_update() {
        let mut r = blank_record(RecordKind::Discovery, Some(201), 4_000);
        r.discovery_public_id = Some("DISC-201".into());
        r.xp_awarded = 20;
        let seq = insert_record(r);
        assert_eq!(get_record(seq).unwrap().outcome, None);
        update_record_outcome(seq, Outcome::Confirmed).unwrap();
        assert_eq!(get_record(seq).unwrap().outcome, Some(Outcome::Confirmed));
    }

    #[test]
    fn t3_3_prune_oldest_classifications() {
        let base_count = count_records();
        for i in 1..=5 {
            insert_record(blank_record(
                RecordKind::Classification,
                Some(1000 + i),
                5_000 + i,
            ));
        }
        let after_count = count_records();
        assert_eq!(after_count, base_count + 5);

        prune_oldest_classifications_if_needed(base_count + 2);
        assert_eq!(count_records(), base_count + 2);
    }

    #[test]
    fn t3_4_idempotency_is_independent_for_tasks_and_reviews() {
        let seq_task = insert_record(blank_record(RecordKind::Classification, Some(500), 6_000));
        let seq_review = insert_record(blank_record(RecordKind::Review, Some(500), 6_001));
        assert_ne!(seq_task, seq_review);
        assert_eq!(get_seq_by_task(500), Some(seq_task));
        assert_eq!(get_seq_by_review(500), Some(seq_review));
        assert_eq!(get_seq_by_task(500), get_seq_by_task(500));
    }

    #[test]
    fn t3_4_upsert_credit_updates_the_correct_record_by_public_id_not_subject_id() {
        let mut discovery = blank_record(RecordKind::Discovery, Some(9_001), 7_000);
        discovery.discovery_public_id = Some("SC-2026-000777".into());
        let disc_seq = insert_record(discovery);

        let unrelated = blank_record(RecordKind::Classification, Some(777), 7_001);
        let unrelated_seq = insert_record(unrelated);

        let credit = CreditCopy {
            v: 1,
            public_id: "SC-2026-000777".into(),
            category: "ring".into(),
            role: CreditRole::Discoverer,
            outcome: Outcome::Confirmed,
            at: 1,
            subject_id: 777,
            citation_url: None,
        };
        upsert_credit(credit);

        assert_eq!(
            get_record(disc_seq).unwrap().outcome,
            Some(Outcome::Confirmed)
        );
        assert_eq!(
            get_record(unrelated_seq).unwrap().outcome,
            None,
            "must not corrupt the unrelated record whose task_id coincidentally equals credit.subject_id"
        );
    }

    fn subject(id: u32) -> sc_types::SubjectRef {
        sc_types::SubjectRef {
            subject_id: id,
            field: "ceers".into(),
            ra_deg: 1.0,
            dec_deg: 2.0,
            image_url: "https://x/img.png".into(),
            image_sha256: vec![1; 32],
            dossier_url: "https://x/dossier.json".into(),
            dossier_sha256: vec![2; 32],
            data_version: 1,
        }
    }

    #[test]
    fn t3_4_remember_and_take_subject_round_trips_once() {
        let key = IdemKey {
            review: false,
            id: 4242,
        };
        remember_subject(key, subject(1));
        assert_eq!(pending_subject(key), Some(subject(1)));
        assert_eq!(take_subject(key), Some(subject(1)));
        assert_eq!(take_subject(key), None);
    }

    #[test]
    fn t3_4_pending_subjects_are_keyed_by_kind_and_bounded() {
        let task = IdemKey {
            review: false,
            id: 7_000_000,
        };
        let review = IdemKey {
            review: true,
            id: 7_000_000,
        };
        remember_subject(task, subject(1));
        remember_subject(review, subject(2));
        assert_eq!(take_subject(task), Some(subject(1)));
        assert_eq!(take_subject(review), Some(subject(2)));

        for id in 0..MAX_PENDING_SUBJECTS + 10 {
            remember_subject(IdemKey { review: false, id }, subject(3));
        }
        assert_eq!(count_pending_subjects(), MAX_PENDING_SUBJECTS);
        let newest = IdemKey {
            review: false,
            id: MAX_PENDING_SUBJECTS + 9,
        };
        assert_eq!(pending_subject(newest), Some(subject(3)));
    }
}
