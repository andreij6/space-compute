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
    CorroborationConfirmed {
        seq: u64,
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
pub enum ActivityKind {
    AaaSpawned {
        name: String,
    },
    Classified {
        classification_id: u64,
        subject_id: u32,
        fee: u64,
    },
    DiscoveryFlagged {
        public_id: String,
    },
    ReviewSubmitted {
        review_id: u64,
        fee: u64,
    },
    DiscoveryResolved {
        public_id: String,
        outcome: String,
    },
    CorroborationConfirmed {
        public_id: String,
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
}

#[derive(CandidType, Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct ActivityItem {
    pub id: u64,
    pub at: u64,
    pub aaa: Principal,
    pub owner: Principal,
    pub kind: ActivityKind,
}

fn public_id(seq: u64, privileged: bool) -> Option<String> {
    let d = crate::discoveries::get(seq)?;
    let open = d.is_honeypot || d.status == crate::discoveries::DiscoveryStatus::UnderReview;
    (privileged || !open).then_some(d.public_id)
}

fn public_kind(kind: EventKind, privileged: bool) -> Option<ActivityKind> {
    use ActivityKind as A;
    Some(match kind {
        EventKind::AaaSpawned { name } => A::AaaSpawned { name },
        EventKind::Classified {
            classification_id,
            subject_id,
            fee,
            ..
        } => A::Classified {
            classification_id,
            subject_id,
            fee,
        },
        EventKind::DiscoveryFlagged { seq } => A::DiscoveryFlagged {
            public_id: public_id(seq, privileged)?,
        },
        EventKind::ReviewSubmitted {
            review_id,
            seq,
            honeypot,
            fee,
        } => {
            if !privileged && (honeypot || public_id(seq, false).is_none()) {
                return None;
            }
            A::ReviewSubmitted { review_id, fee }
        }
        EventKind::DiscoveryResolved { seq, outcome } => A::DiscoveryResolved {
            public_id: public_id(seq, false)?,
            outcome,
        },
        EventKind::CorroborationConfirmed { seq } => A::CorroborationConfirmed {
            public_id: public_id(seq, false)?,
        },
        EventKind::ReviewScored { review_id, matched } => {
            let seq = crate::reviews::get_review(review_id)?.discovery_seq;
            public_id(seq, false)?;
            A::ReviewScored { review_id, matched }
        }
        EventKind::ConsensusScored {
            subject_id,
            agree,
            trials,
        } => A::ConsensusScored {
            subject_id,
            agree,
            trials,
        },
        EventKind::BadgeAwarded { badge } => A::BadgeAwarded { badge },
        EventKind::TierChanged { from, to } => A::TierChanged { from, to },
        EventKind::AaaSuspended { reason } => A::AaaSuspended { reason },
        EventKind::AaaUnsuspended => A::AaaUnsuspended,
        EventKind::ImageMismatch { subject_id } => A::ImageMismatch { subject_id },
        EventKind::CyclesContributed { amount } => A::CyclesContributed { amount },
        EventKind::Admin { .. } => return None,
    })
}

#[derive(CandidType, Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct Page<T> {
    pub items: Vec<T>,
    pub next_cursor: Option<u64>,
}

pub const SCAN_MAX: usize = 2_000;

pub fn scan_page<K, V>(
    entries: impl Iterator<Item = (K, V)>,
    cap: usize,
    keep: impl Fn(&V) -> bool,
) -> (Vec<V>, Option<K>) {
    let mut items = Vec::new();
    for (i, (k, v)) in entries.enumerate() {
        if items.len() == cap || i == SCAN_MAX {
            return (items, Some(k));
        }
        if keep(&v) {
            items.push(v);
        }
    }
    (items, None)
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
        if !crate::progression::replay_active() {
            crate::progression::apply_event(&entry);
        }
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

pub fn list_aaa_activity(
    aaa: Principal,
    privileged: bool,
    cursor: Option<u64>,
    limit: u32,
) -> Page<ActivityItem> {
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
            .filter_map(|ev| {
                Some(ActivityItem {
                    id: ev.id,
                    at: ev.at,
                    aaa: ev.aaa,
                    owner: ev.owner,
                    kind: public_kind(ev.kind, privileged)?,
                })
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

        let page1 = list_aaa_activity(aaa_a, false, None, 2);
        assert_eq!(page1.items.len(), 2);
        assert!(page1.items[0].id > page1.items[1].id);
        assert_eq!(page1.items[0].aaa, aaa_a);
        assert_eq!(page1.items[1].aaa, aaa_a);
        assert!(page1.next_cursor.is_some());

        let page2 = list_aaa_activity(aaa_a, false, page1.next_cursor, 2);
        assert_eq!(page2.items.len(), 2);
        assert!(page2.items[0].id > page2.items[1].id);
        assert!(page1.items[1].id > page2.items[0].id);
        assert!(page2.next_cursor.is_some());

        let page3 = list_aaa_activity(aaa_a, false, page2.next_cursor, 2);
        assert_eq!(page3.items.len(), 2);
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

        let initial_activity = list_aaa_activity(aaa, false, None, 10);
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

        let after_activity = list_aaa_activity(aaa, false, None, 10);
        assert_eq!(after_activity.items.len(), initial_len + 2);
        assert_eq!(
            after_activity.items[1].kind,
            ActivityKind::Classified {
                classification_id: 999,
                subject_id: 1,
                fee: 0
            }
        );
        assert_eq!(
            after_activity.items[0].kind,
            ActivityKind::BadgeAwarded {
                badge: "first_light".into()
            }
        );

        let admin_activity = list_aaa_activity(admin, true, None, 10);
        assert_eq!(admin_activity.items.len(), 0);
    }

    #[test]
    fn t4_6_r12_public_activity_hides_live_discoveries_honeypots_and_gold() {
        use crate::discoveries::{self, DiscoveryStatus, NewDiscovery};
        use crate::reviews;
        use sc_types::Vote;

        let aaa = Principal::from_slice(&[150, 1]);
        let owner = Principal::from_slice(&[150, 2]);
        let d = discoveries::create(NewDiscovery {
            subject_id: 1,
            classification_id: 1,
            discoverer_aaa: aaa,
            discoverer_owner: owner,
            discoverer_name_at_time: "A".into(),
            category: "lens".into(),
            rationale: "arc".into(),
            confidence: 80,
            fee: 0,
            needed_reviews: 3,
            created_at: 1,
            claim_ra_deg: Some(1.0),
            claim_dec_deg: Some(1.0),
        });
        let honeypot = discoveries::create_honeypot(1, "lens".into(), "r".into(), Vote::Agree, 1);
        record_event(
            1,
            aaa,
            owner,
            EventKind::Classified {
                classification_id: 1,
                subject_id: 1,
                gold: Some((1, 2)),
                fee: 5,
            },
        );
        record_event(2, aaa, owner, EventKind::DiscoveryFlagged { seq: d.seq });
        record_event(
            3,
            aaa,
            owner,
            EventKind::ReviewSubmitted {
                review_id: 7,
                seq: d.seq,
                honeypot: false,
                fee: 0,
            },
        );
        record_event(
            4,
            aaa,
            owner,
            EventKind::ReviewSubmitted {
                review_id: 8,
                seq: honeypot.seq,
                honeypot: true,
                fee: 0,
            },
        );
        reviews::put_review_for_test(8, honeypot.seq);
        record_event(
            5,
            aaa,
            owner,
            EventKind::ReviewScored {
                review_id: 8,
                matched: true,
            },
        );

        let kinds = |privileged| -> Vec<ActivityKind> {
            list_aaa_activity(aaa, privileged, None, 100)
                .items
                .into_iter()
                .map(|i| i.kind)
                .collect()
        };
        let stranger = kinds(false);
        assert!(stranger.contains(&ActivityKind::Classified {
            classification_id: 1,
            subject_id: 1,
            fee: 5
        }));
        assert!(!stranger.iter().any(|k| matches!(
            k,
            ActivityKind::DiscoveryFlagged { .. }
                | ActivityKind::ReviewSubmitted { .. }
                | ActivityKind::ReviewScored { .. }
        )));

        let own = kinds(true);
        assert!(own.contains(&ActivityKind::DiscoveryFlagged {
            public_id: d.public_id.clone()
        }));
        assert!(own.contains(&ActivityKind::ReviewSubmitted {
            review_id: 7,
            fee: 0
        }));
        assert!(own.contains(&ActivityKind::ReviewSubmitted {
            review_id: 8,
            fee: 0
        }));
        assert!(!own
            .iter()
            .any(|k| matches!(k, ActivityKind::ReviewScored { .. })));

        let mut resolved = discoveries::get(d.seq).unwrap();
        resolved.status = DiscoveryStatus::Confirmed;
        discoveries::update(&resolved);
        let after = kinds(false);
        assert!(after.contains(&ActivityKind::DiscoveryFlagged {
            public_id: d.public_id
        }));
        assert!(after.contains(&ActivityKind::ReviewSubmitted {
            review_id: 7,
            fee: 0
        }));
        assert!(!after
            .iter()
            .any(|k| matches!(k, ActivityKind::ReviewSubmitted { review_id: 8, .. })));
    }
}
