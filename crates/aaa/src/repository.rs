use std::cell::RefCell;

use ic_stable_structures::{StableBTreeMap, StableCell};

use crate::memory::{self, Memory};
use crate::record::{Record, RecordKind, Stats};

thread_local! {
    static RECORDS: RefCell<StableBTreeMap<u64, Record, Memory>> =
        RefCell::new(StableBTreeMap::init(memory::get(memory::RECORDS)));
    static IDEMPOTENCY: RefCell<StableBTreeMap<u64, u64, Memory>> =
        RefCell::new(StableBTreeMap::init(memory::get(memory::IDEMPOTENCY)));
    static STATS: RefCell<StableCell<Stats, Memory>> =
        RefCell::new(StableCell::init(memory::get(memory::STATS), Stats::default()));
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
}
