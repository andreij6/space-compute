use std::cell::RefCell;

use candid::{CandidType, Principal};
use ic_stable_structures::{StableBTreeMap, StableLog};
use serde::{Deserialize, Serialize};

use crate::memory::{self, Memory};

#[derive(CandidType, Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub enum EventKind {
    AaaSpawned {
        name: String,
    },
    Classified {
        classification_id: u64,
        subject_id: u32,
        gold: Option<(u8, u8)>,
        fee: u64,
    },
    DiscoveryFlagged {
        seq: u64,
    },
    ReviewSubmitted {
        review_id: u64,
        seq: u64,
        honeypot: bool,
        fee: u64,
    },
    DiscoveryResolved {
        seq: u64,
        outcome: String,
    },
    ReviewScored {
        review_id: u64,
        matched: bool,
    },
    ConsensusScored {
        subject_id: u32,
        agree: bool,
        trials: u8,
    },
    BadgeAwarded {
        badge: String,
    },
    TierChanged {
        from: u8,
        to: u8,
    },
    AaaSuspended {
        reason: String,
    },
    AaaUnsuspended,
    ImageMismatch {
        subject_id: u32,
    },
    CyclesContributed {
        amount: u128,
    },
    Admin {
        method: String,
    },
}

#[derive(CandidType, Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct Event {
    pub v: u8,
    pub id: u64,
    pub at: u64,
    pub aaa: Principal,
    pub owner: Principal,
    pub kind: EventKind,
}

crate::candid_storable!(Event);

#[derive(CandidType, Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct ActivityItem {
    pub id: u64,
    pub at: u64,
    pub aaa: Principal,
    pub owner: Principal,
    pub kind: EventKind,
}

#[derive(CandidType, Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct Page<T> {
    pub items: Vec<T>,
    pub next_cursor: Option<u64>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct AaaActivityKey {
    pub aaa: Principal,
    pub event_id: u64,
}

impl ic_stable_structures::Storable for AaaActivityKey {
    const BOUND: ic_stable_structures::storable::Bound =
        ic_stable_structures::storable::Bound::Bounded {
            max_size: 38,
            is_fixed_size: false,
        };

    fn to_bytes(&self) -> std::borrow::Cow<'_, [u8]> {
        let mut bytes = Vec::with_capacity(38);
        let p_bytes = self.aaa.as_slice();
        bytes.push(p_bytes.len() as u8);
        bytes.extend_from_slice(p_bytes);
        bytes.extend_from_slice(&self.event_id.to_be_bytes());
        std::borrow::Cow::Owned(bytes)
    }

    fn into_bytes(self) -> Vec<u8> {
        self.to_bytes().into_owned()
    }

    fn from_bytes(bytes: std::borrow::Cow<[u8]>) -> Self {
        let len = bytes[0] as usize;
        let aaa = Principal::from_slice(&bytes[1..1 + len]);
        let event_id = u64::from_be_bytes(bytes[1 + len..9 + len].try_into().unwrap());
        AaaActivityKey { aaa, event_id }
    }
}

thread_local! {
    static LOG: RefCell<StableLog<Event, Memory, Memory>> = RefCell::new(StableLog::init(
        memory::get(memory::EVENTS_INDEX),
        memory::get(memory::EVENTS_DATA),
    ));
    static AAA_ACTIVITY_MAP: RefCell<StableBTreeMap<AaaActivityKey, (), Memory>> =
        RefCell::new(StableBTreeMap::init(memory::get(memory::AAA_ACTIVITY)));
}

pub fn record_event(at: u64, aaa: Principal, owner: Principal, kind: EventKind) -> u64 {
    let id = LOG.with_borrow(|log| log.len());
    let entry = Event {
        v: 1,
        id,
        at,
        aaa,
        owner,
        kind: kind.clone(),
    };
    LOG.with_borrow(|log| log.append(&entry))
        .expect("event log append");

    if !matches!(kind, EventKind::Admin { .. }) && aaa != Principal::anonymous() {
        AAA_ACTIVITY_MAP.with_borrow_mut(|m| {
            m.insert(AaaActivityKey { aaa, event_id: id }, ());
        });
        crate::progression::apply_event(&entry);
    }

    id
}

pub fn len() -> u64 {
    LOG.with_borrow(|log| log.len())
}

pub fn get_event(id: u64) -> Option<Event> {
    LOG.with_borrow(|log| log.get(id))
}

const RECENT_ACTIVITY_SCAN_MAX: u64 = 5_000;

#[derive(Debug, PartialEq, Eq, Default)]
pub struct RecentActivity {
    pub events: u64,
    pub image_mismatches: u64,
}

pub fn recent_activity(now: u64, window_ns: u64) -> RecentActivity {
    let cutoff = now.saturating_sub(window_ns);
    let total = LOG.with_borrow(|log| log.len());
    let mut out = RecentActivity::default();
    LOG.with_borrow(|log| {
        let mut scanned = 0u64;
        let mut i = total;
        while i > 0 && scanned < RECENT_ACTIVITY_SCAN_MAX {
            i -= 1;
            scanned += 1;
            let Some(ev) = log.get(i) else { break };
            if ev.at < cutoff {
                break;
            }
            out.events += 1;
            if matches!(ev.kind, EventKind::ImageMismatch { .. }) {
                out.image_mismatches += 1;
            }
        }
    });
    out
}

pub fn list_aaa_activity(aaa: Principal, cursor: Option<u64>, limit: u32) -> Page<ActivityItem> {
    let limit = sc_types::limits::page_limit(limit) as usize;
    let max_id = cursor.unwrap_or(u64::MAX);
    let start_key = AaaActivityKey { aaa, event_id: 0 };
    let end_key = AaaActivityKey {
        aaa,
        event_id: max_id,
    };

    let event_ids: Vec<u64> = AAA_ACTIVITY_MAP.with_borrow(|m| {
        m.range(start_key..=end_key)
            .rev()
            .take(limit + 1)
            .map(|e| e.key().event_id)
            .collect()
    });

    let (batch, next_cursor) = if event_ids.len() > limit {
        (&event_ids[..limit], Some(event_ids[limit]))
    } else {
        (&event_ids[..], None)
    };

    let items: Vec<ActivityItem> = LOG.with_borrow(|log| {
        batch
            .iter()
            .filter_map(|&id| log.get(id))
            .map(|ev| ActivityItem {
                id: ev.id,
                at: ev.at,
                aaa: ev.aaa,
                owner: ev.owner,
                kind: ev.kind,
            })
            .collect()
    });

    Page { items, next_cursor }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn t2_5_events_append_and_retrieve() {
        let aaa = Principal::from_slice(&[1, 2, 3, 4]);
        let owner = Principal::from_slice(&[5, 6, 7, 8]);
        let id1 = record_event(
            1_000,
            aaa,
            owner,
            EventKind::AaaSpawned {
                name: "agent-1".into(),
            },
        );
        let id2 = record_event(
            2_000,
            aaa,
            owner,
            EventKind::Classified {
                classification_id: 42,
                subject_id: 101,
                gold: Some((2, 2)),
                fee: 100,
            },
        );

        assert!(id2 > id1);
        assert!(len() >= 2);

        let ev1 = get_event(id1).expect("event 1");
        assert_eq!(ev1.id, id1);
        assert_eq!(ev1.at, 1_000);
        assert_eq!(ev1.aaa, aaa);
        assert_eq!(
            ev1.kind,
            EventKind::AaaSpawned {
                name: "agent-1".into()
            }
        );

        let ev2 = get_event(id2).expect("event 2");
        assert_eq!(ev2.id, id2);
        assert_eq!(ev2.at, 2_000);
        assert_eq!(
            ev2.kind,
            EventKind::Classified {
                classification_id: 42,
                subject_id: 101,
                gold: Some((2, 2)),
                fee: 100
            }
        );
    }

    #[test]
    fn t2_5_paged_activity_newest_first_and_cursor() {
        let aaa_a = Principal::from_slice(&[10, 11, 12, 13]);
        let aaa_b = Principal::from_slice(&[20, 21, 22, 23]);
        let owner = Principal::from_slice(&[30, 31, 32, 33]);

        for i in 1..=5 {
            record_event(
                i * 10,
                aaa_a,
                owner,
                EventKind::Classified {
                    classification_id: i,
                    subject_id: i as u32,
                    gold: None,
                    fee: 0,
                },
            );
            record_event(
                i * 10 + 1,
                aaa_b,
                owner,
                EventKind::Classified {
                    classification_id: i + 100,
                    subject_id: (i + 100) as u32,
                    gold: None,
                    fee: 0,
                },
            );
        }

        let page1 = list_aaa_activity(aaa_a, None, 2);
        assert_eq!(page1.items.len(), 2);
        assert!(page1.items[0].id > page1.items[1].id);
        assert_eq!(page1.items[0].aaa, aaa_a);
        assert_eq!(page1.items[1].aaa, aaa_a);
        assert!(page1.next_cursor.is_some());

        let page2 = list_aaa_activity(aaa_a, page1.next_cursor, 2);
        assert_eq!(page2.items.len(), 2);
        assert!(page2.items[0].id > page2.items[1].id);
        assert!(page1.items[1].id > page2.items[0].id);
        assert!(page2.next_cursor.is_some());

        let page3 = list_aaa_activity(aaa_a, page2.next_cursor, 2);
        assert_eq!(page3.items.len(), 1);
        assert!(page2.items[1].id > page3.items[0].id);
        assert!(page3.next_cursor.is_none());

        for item in page1.items.iter().chain(&page2.items).chain(&page3.items) {
            assert_eq!(item.aaa, aaa_a);
        }
    }

    #[test]
    fn t2_5_admin_events_excluded_from_activity() {
        let admin = Principal::from_slice(&[99, 98, 97]);
        let aaa = Principal::from_slice(&[55, 56, 57]);
        let owner = Principal::from_slice(&[66, 67, 68]);

        record_event(
            500,
            admin,
            Principal::anonymous(),
            EventKind::Admin {
                method: "admin_set_params".into(),
            },
        );

        let initial_activity = list_aaa_activity(aaa, None, 10);
        let initial_len = initial_activity.items.len();

        record_event(
            600,
            aaa,
            owner,
            EventKind::Classified {
                classification_id: 999,
                subject_id: 1,
                gold: None,
                fee: 0,
            },
        );

        let after_activity = list_aaa_activity(aaa, None, 10);
        assert_eq!(after_activity.items.len(), initial_len + 1);
        assert_eq!(
            after_activity.items[0].kind,
            EventKind::Classified {
                classification_id: 999,
                subject_id: 1,
                gold: None,
                fee: 0
            }
        );

        let admin_activity = list_aaa_activity(admin, None, 10);
        assert_eq!(admin_activity.items.len(), 0);
    }
}
