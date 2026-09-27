use std::cell::RefCell;

use ic_stable_structures::{StableBTreeMap, StableCell};
use sc_types::ApiError;

use crate::memory::{self, Memory};
use crate::record::{
    CreditCopy, ListRecordsFilter, Outcome, PageCreditCopy, PageRecord, Record, RecordKind, Stats,
};

pub const MAX_RECORDS_QUOTA: u64 = 1_000_000;

thread_local! {
    static RECORDS: RefCell<StableBTreeMap<u64, Record, Memory>> =
        RefCell::new(StableBTreeMap::init(memory::get(memory::RECORDS)));
    static IDEMPOTENCY: RefCell<StableBTreeMap<u64, u64, Memory>> =
        RefCell::new(StableBTreeMap::init(memory::get(memory::IDEMPOTENCY)));
    static STATS: RefCell<StableCell<Stats, Memory>> =
        RefCell::new(StableCell::init(memory::get(memory::STATS), Stats::default()));
    static CREDITS: RefCell<StableBTreeMap<String, CreditCopy, Memory>> =
        RefCell::new(StableBTreeMap::init(memory::get(memory::CREDITS)));
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
            IDEMPOTENCY.with_borrow_mut(|idx| {
                idx.insert(id, seq);
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

pub fn get_seq_by_task(task_or_assignment_id: u64) -> Option<u64> {
    IDEMPOTENCY.with_borrow(|idx| idx.get(&task_or_assignment_id))
}

pub fn count_records() -> u64 {
    RECORDS.with_borrow(|r| r.len())
}

pub fn list_records(filter: ListRecordsFilter) -> PageRecord {
    let limit = (filter.limit as usize).clamp(1, 100);
    RECORDS.with_borrow(|r| {
        let mut items = Vec::new();
        let mut next_cursor = None;

        let entries: Vec<(u64, Record)> = match filter.cursor {
            Some(cur) => r
                .range(..cur)
                .rev()
                .map(|e| (*e.key(), e.value()))
                .collect(),
            None => r.iter().rev().map(|e| (*e.key(), e.value())).collect(),
        };

        for (seq, rec) in entries {
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
    if let Some(seq) = get_seq_by_task(credit.subject_id as u64) {
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
                        IDEMPOTENCY.with_borrow_mut(|idx| idx.remove(&id));
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

    #[test]
    fn t3_2_insert_record_increments_seq_and_populates_idempotency() {
        let r = Record {
            v: 1,
            seq: 0,
            at: 1_000,
            kind: RecordKind::Classification,
            task_or_assignment_id: Some(42),
            subject: None,
            answers: Vec::new(),
            discovery_public_id: None,
            category: None,
            vote: None,
            rationale: None,
            outcome: None,
            xp_awarded: 10,
            agent_label: Some("test-bot".into()),
            fee: 200_000_000,
            by: Principal::anonymous(),
        };
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
        let r_disc = Record {
            v: 1,
            seq: 0,
            at: 2_000,
            kind: RecordKind::Discovery,
            task_or_assignment_id: Some(43),
            subject: None,
            answers: Vec::new(),
            discovery_public_id: Some("DISC-1".into()),
            category: Some("ring".into()),
            vote: None,
            rationale: Some("test rationale".into()),
            outcome: None,
            xp_awarded: 25,
            agent_label: None,
            fee: 200_000_000,
            by: Principal::anonymous(),
        };
        insert_record(r_disc);
        let stats = get_stats();
        assert!(stats.discoveries >= 1);
        assert_eq!(stats.last_activity_at, 2_000);
    }

    #[test]
    fn t3_3_list_records_pagination_and_filter() {
        let r1 = Record {
            v: 1,
            seq: 0,
            at: 3_000,
            kind: RecordKind::Classification,
            task_or_assignment_id: Some(101),
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
        };
        let r2 = Record {
            v: 1,
            seq: 0,
            at: 3_001,
            kind: RecordKind::Review,
            task_or_assignment_id: Some(102),
            subject: None,
            answers: Vec::new(),
            discovery_public_id: None,
            category: None,
            vote: None,
            rationale: None,
            outcome: None,
            xp_awarded: 15,
            agent_label: None,
            fee: 200_000_000,
            by: Principal::anonymous(),
        };
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
        assert_eq!(page.items[0].public_id, "JWST-CEERS-DISC-001");
    }

    #[test]
    fn t3_3_record_outcome_update() {
        let r = Record {
            v: 1,
            seq: 0,
            at: 4_000,
            kind: RecordKind::Discovery,
            task_or_assignment_id: Some(201),
            subject: None,
            answers: Vec::new(),
            discovery_public_id: Some("DISC-201".into()),
            category: None,
            vote: None,
            rationale: None,
            outcome: None,
            xp_awarded: 20,
            agent_label: None,
            fee: 200_000_000,
            by: Principal::anonymous(),
        };
        let seq = insert_record(r);
        assert_eq!(get_record(seq).unwrap().outcome, None);
        update_record_outcome(seq, Outcome::Confirmed).unwrap();
        assert_eq!(get_record(seq).unwrap().outcome, Some(Outcome::Confirmed));
    }

    #[test]
    fn t3_3_prune_oldest_classifications() {
        let base_count = count_records();
        for i in 1..=5 {
            insert_record(Record {
                v: 1,
                seq: 0,
                at: 5_000 + i,
                kind: RecordKind::Classification,
                task_or_assignment_id: Some(1000 + i),
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
            });
        }
        let after_count = count_records();
        assert_eq!(after_count, base_count + 5);

        prune_oldest_classifications_if_needed(base_count + 2);
        assert_eq!(count_records(), base_count + 2);
    }
}
