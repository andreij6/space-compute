use std::cell::RefCell;

use candid::{CandidType, Principal};
use ic_stable_structures::StableBTreeMap;
use serde::{Deserialize, Serialize};

use crate::events::{Event, EventKind};
use crate::memory::{self, Memory};
use crate::meta;
use sc_types::ApiError;

#[derive(CandidType, Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct Progress {
    pub v: u8,
    pub xp: u64,
    pub gold_tasks: u32,
    pub gold_hits: u32,
    pub gold_trials: u32,
    pub cons_hits: u32,
    pub cons_trials: u32,
    pub rev_hits: u32,
    pub rev_trials: u32,
    pub classifications: u64,
    pub discoveries: u64,
    pub confirmed: u64,
    pub reviews: u64,
    pub badges: u64,
    pub tier: u8,
    pub gold_streak: u16,
    pub cycles_contributed: u128,
}

impl Default for Progress {
    fn default() -> Self {
        Self {
            v: 1,
            xp: 0,
            gold_tasks: 0,
            gold_hits: 0,
            gold_trials: 0,
            cons_hits: 0,
            cons_trials: 0,
            rev_hits: 0,
            rev_trials: 0,
            classifications: 0,
            discoveries: 0,
            confirmed: 0,
            reviews: 0,
            badges: 0,
            tier: 1,
            gold_streak: 0,
            cycles_contributed: 0,
        }
    }
}

crate::candid_storable!(Progress);

#[derive(CandidType, Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct AaaCounters {
    pub classifications: u64,
    pub discoveries: u64,
    pub confirmed: u64,
    pub reviews: u64,
}

#[derive(CandidType, Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct AaaPublic {
    pub name: String,
    pub avatar_seed: u64,
    pub status: crate::registry::AaaStatus,
    pub tier: u8,
    pub xp: u64,
    pub next_tier_xp: u64,
    pub reputation_bp: u32,
    pub badges: u64,
    pub counters: AaaCounters,
    pub created_at: u64,
    pub is_house: bool,
}

#[derive(CandidType, Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct Stats {
    pub total_classifications: u64,
    pub active_aaas: u64,
    pub confirmed_discoveries: u64,
    pub under_review_count: u64,
    pub total_subjects: u32,
    pub retired_subjects: u32,
}

#[derive(CandidType, Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct LeaderRow {
    pub rank: u64,
    pub aaa: Principal,
    pub name: String,
    pub tier: u8,
    pub xp: u64,
    pub confirmed_discoveries: u64,
    pub reviews: u64,
}

#[derive(CandidType, Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub struct LeaderCursor {
    pub inverted_xp: u64,
    pub aaa: Principal,
    pub rank: u64,
}

#[derive(CandidType, Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct LeaderPage {
    pub items: Vec<LeaderRow>,
    pub next_cursor: Option<LeaderCursor>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct LeaderboardKey {
    pub inverted_xp: u64,
    pub aaa: Principal,
}

impl ic_stable_structures::Storable for LeaderboardKey {
    const BOUND: ic_stable_structures::storable::Bound =
        ic_stable_structures::storable::Bound::Bounded {
            max_size: 38,
            is_fixed_size: false,
        };

    fn to_bytes(&self) -> std::borrow::Cow<'_, [u8]> {
        let mut bytes = Vec::with_capacity(38);
        bytes.extend_from_slice(&self.inverted_xp.to_be_bytes());
        let p_bytes = self.aaa.as_slice();
        bytes.push(p_bytes.len() as u8);
        bytes.extend_from_slice(p_bytes);
        std::borrow::Cow::Owned(bytes)
    }

    fn into_bytes(self) -> Vec<u8> {
        self.to_bytes().into_owned()
    }

    fn from_bytes(bytes: std::borrow::Cow<[u8]>) -> Self {
        let inverted_xp = u64::from_be_bytes(bytes[0..8].try_into().unwrap());
        let len = bytes[8] as usize;
        let aaa = Principal::from_slice(&bytes[9..9 + len]);
        LeaderboardKey { inverted_xp, aaa }
    }
}

thread_local! {
    static PROGRESS_MAP: RefCell<StableBTreeMap<Principal, Progress, Memory>> =
        RefCell::new(StableBTreeMap::init(memory::get(memory::PROGRESS)));
    static LEADERBOARD_MAP: RefCell<StableBTreeMap<LeaderboardKey, (), Memory>> =
        RefCell::new(StableBTreeMap::init(memory::get(memory::LEADERBOARD)));
}

pub fn calculate_reputation(p: &Progress) -> u32 {
    let hits = p.gold_hits as u64 * 2 + p.cons_hits as u64 + p.rev_hits as u64 * 2;
    let trials = p.gold_trials as u64 * 2 + p.cons_trials as u64 + p.rev_trials as u64 * 2;
    (((hits + 1) * 10_000) / (trials + 2)) as u32
}

pub fn calculate_tier(p: &Progress, rep_bp: u32) -> u8 {
    if p.xp >= 10_000 && rep_bp >= 8500 && p.gold_tasks >= 300 {
        5
    } else if p.xp >= 2500 && rep_bp >= 8000 && p.gold_tasks >= 150 {
        4
    } else if p.xp >= 500 && rep_bp >= 7000 && p.gold_tasks >= 60 {
        3
    } else if p.xp >= 50 && rep_bp >= 6000 && p.gold_tasks >= 20 {
        2
    } else {
        1
    }
}

pub fn next_tier_xp(tier: u8) -> u64 {
    match tier {
        1 => 50,
        2 => 500,
        3 => 2500,
        4 => 10000,
        _ => 10000,
    }
}

pub fn get_progress(aaa: &Principal) -> Progress {
    PROGRESS_MAP.with_borrow(|m| m.get(aaa).unwrap_or_default())
}

type BadgeRule = fn(&Progress) -> bool;

pub const BADGES: &[(u8, &str, BadgeRule)] = &[
    (0, "first_light", |p| p.classifications >= 1),
    (1, "first_find", |p| p.discoveries >= 1),
    (2, "confirmed_discoverer", |p| p.confirmed >= 1),
    (3, "peer_reviewer", |p| p.reviews >= 10),
    (4, "sharp_eye", |p| p.gold_streak >= 10),
];

const XP_CORROBORATOR: u64 = 5;

pub fn apply_event(ev: &Event) {
    apply(ev, true);
}

fn apply(ev: &Event, emit: bool) {
    if ev.aaa == Principal::anonymous() {
        return;
    }

    let mut p = get_progress(&ev.aaa);
    let old_tier = p.tier;
    let old_xp = p.xp;
    let old_badges = p.badges;

    match &ev.kind {
        EventKind::Classified { gold, fee, .. } => {
            p.classifications += 1;
            p.xp += 1;
            p.cycles_contributed = p.cycles_contributed.saturating_add(u128::from(*fee));
            if let Some((hits, trials)) = gold {
                p.gold_tasks += 1;
                p.gold_hits += *hits as u32;
                p.gold_trials += *trials as u32;
                if *trials > 0 && hits == trials {
                    p.gold_streak += 1;
                    p.xp += 1;
                } else {
                    p.gold_streak = 0;
                }
            }
        }
        EventKind::ConsensusScored { agree, trials, .. } => {
            p.cons_trials += *trials as u32;
            if *agree {
                p.cons_hits += *trials as u32;
            }
        }
        EventKind::ReviewSubmitted { fee, .. } => {
            p.reviews += 1;
            p.xp += 3;
            p.cycles_contributed = p.cycles_contributed.saturating_add(u128::from(*fee));
        }
        EventKind::ReviewScored { matched, .. } => {
            p.rev_trials += 1;
            if *matched {
                p.rev_hits += 1;
                p.xp += 2;
            }
        }
        EventKind::DiscoveryFlagged { .. } => {
            p.discoveries += 1;
        }
        EventKind::DiscoveryResolved { outcome, .. } if outcome == "Confirmed" => {
            p.confirmed += 1;
            p.xp += 50;
        }
        EventKind::CorroborationConfirmed { .. } => {
            p.xp += XP_CORROBORATOR;
        }
        EventKind::CyclesContributed { amount } => {
            p.cycles_contributed = p.cycles_contributed.saturating_add(*amount);
        }
        _ => {}
    }

    let rep = calculate_reputation(&p);
    p.tier = calculate_tier(&p, rep);
    for (bit, _, rule) in BADGES {
        if rule(&p) {
            p.badges |= 1 << bit;
        }
    }

    PROGRESS_MAP.with_borrow_mut(|m| m.insert(ev.aaa, p.clone()));

    if old_tier >= 2 {
        LEADERBOARD_MAP.with_borrow_mut(|m| {
            m.remove(&LeaderboardKey {
                inverted_xp: u64::MAX - old_xp,
                aaa: ev.aaa,
            });
        });
    }

    let is_house = crate::registry::get_aaa(&ev.aaa)
        .and_then(|r| r.is_house)
        .unwrap_or(false);
    if p.tier >= 2 && !is_house {
        LEADERBOARD_MAP.with_borrow_mut(|m| {
            m.insert(
                LeaderboardKey {
                    inverted_xp: u64::MAX - p.xp,
                    aaa: ev.aaa,
                },
                (),
            );
        });
    }

    if !emit {
        return;
    }
    let record = |kind| crate::events::record_event(ev.at, ev.aaa, ev.owner, kind);
    if p.tier != old_tier {
        record(EventKind::TierChanged {
            from: old_tier,
            to: p.tier,
        });
    }
    for (bit, id, _) in BADGES {
        if p.badges & !old_badges & (1 << bit) != 0 {
            record(EventKind::BadgeAwarded {
                badge: (*id).into(),
            });
        }
    }
}

pub fn get_aaa_public(aaa: &Principal) -> Option<AaaPublic> {
    let rec = crate::registry::get_aaa(aaa)?;
    let p = get_progress(aaa);
    let rep = calculate_reputation(&p);
    let next_xp = next_tier_xp(p.tier);

    Some(AaaPublic {
        name: rec.name,
        avatar_seed: rec.avatar_seed,
        status: rec.status,
        tier: p.tier,
        xp: p.xp,
        next_tier_xp: next_xp,
        reputation_bp: rep,
        badges: p.badges,
        counters: AaaCounters {
            classifications: p.classifications,
            discoveries: p.discoveries,
            confirmed: p.confirmed,
            reviews: p.reviews,
        },
        created_at: rec.created_at,
        is_house: rec.is_house.unwrap_or(false),
    })
}

pub fn sync_leaderboard_house(aaa: Principal, is_house: bool) {
    let p = get_progress(&aaa);
    if p.tier < 2 {
        return;
    }
    let key = LeaderboardKey {
        inverted_xp: u64::MAX - p.xp,
        aaa,
    };
    LEADERBOARD_MAP.with_borrow_mut(|m| {
        if is_house {
            m.remove(&key);
        } else {
            m.insert(key, ());
        }
    });
}

pub fn get_leaderboard(cursor: Option<LeaderCursor>, limit: u32) -> LeaderPage {
    let limit = sc_types::limits::page_limit(limit) as usize;
    let (start_key, first_rank) = match cursor {
        Some(c) => (
            LeaderboardKey {
                inverted_xp: c.inverted_xp,
                aaa: c.aaa,
            },
            c.rank,
        ),
        None => (
            LeaderboardKey {
                inverted_xp: 0,
                aaa: Principal::from_slice(&[]),
            },
            1,
        ),
    };

    let entries: Vec<LeaderboardKey> = LEADERBOARD_MAP.with_borrow(|m| {
        m.range(start_key..)
            .take(limit + 1)
            .map(|e| *e.key())
            .collect()
    });

    let mut rows = Vec::new();
    let mut rank = first_rank;
    for k in entries.iter().take(limit) {
        if let Some(rec) = crate::registry::get_aaa(&k.aaa) {
            let p = get_progress(&k.aaa);
            rows.push(LeaderRow {
                rank,
                aaa: k.aaa,
                name: rec.name,
                tier: p.tier,
                xp: p.xp,
                confirmed_discoveries: p.confirmed,
                reviews: p.reviews,
            });
            rank += 1;
        }
    }

    LeaderPage {
        items: rows,
        next_cursor: entries.get(limit).map(|k| LeaderCursor {
            inverted_xp: k.inverted_xp,
            aaa: k.aaa,
            rank,
        }),
    }
}

#[derive(CandidType, Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub struct ReplayStatus {
    pub next_event_id: u64,
    pub processed: u64,
    pub done: bool,
}

pub const REPLAY_BATCH_MAX: u32 = 5_000;

pub fn replay_active() -> bool {
    meta::get(meta::REPLAY_NEXT).is_some()
}

pub fn start_replay(from_event_id: u64, batch: u32) -> Result<ReplayStatus, ApiError> {
    if from_event_id != 0 {
        return Err(ApiError::invalid("a replay always starts at event 0"));
    }
    if replay_active() {
        return Err(ApiError::Conflict("a replay is already running".into()));
    }
    meta::set(
        meta::REPLAY_BATCH,
        u64::from(batch.clamp(1, REPLAY_BATCH_MAX)),
    );
    meta::set(meta::REPLAY_CLEARING, 1);
    meta::set(meta::REPLAY_NEXT, 0);
    Ok(replay_step())
}

fn clear_some<
    K: ic_stable_structures::Storable + Ord + Clone,
    V: ic_stable_structures::Storable,
>(
    m: &mut StableBTreeMap<K, V, Memory>,
    budget: u64,
) -> u64 {
    let mut n = 0;
    while n < budget && m.pop_first().is_some() {
        n += 1;
    }
    n
}

pub fn replay_step() -> ReplayStatus {
    let Some(next) = meta::get(meta::REPLAY_NEXT) else {
        return ReplayStatus {
            next_event_id: crate::events::len(),
            processed: 0,
            done: true,
        };
    };
    let mut budget = meta::get(meta::REPLAY_BATCH).unwrap_or(u64::from(REPLAY_BATCH_MAX));
    if meta::get(meta::REPLAY_CLEARING).is_some() {
        budget -= PROGRESS_MAP.with_borrow_mut(|m| clear_some(m, budget));
        budget -= LEADERBOARD_MAP.with_borrow_mut(|m| clear_some(m, budget));
        if budget == 0 {
            return ReplayStatus {
                next_event_id: next,
                processed: 0,
                done: false,
            };
        }
        meta::remove(meta::REPLAY_CLEARING);
    }

    let total = crate::events::len();
    let end = next.saturating_add(budget).min(total);
    for id in next..end {
        if let Some(ev) = crate::events::get_event(id) {
            if !matches!(ev.kind, EventKind::Admin { .. }) {
                apply(&ev, false);
            }
        }
    }
    let done = end >= total;
    if done {
        meta::remove(meta::REPLAY_NEXT);
        meta::remove(meta::REPLAY_BATCH);
    } else {
        meta::set(meta::REPLAY_NEXT, end);
    }
    ReplayStatus {
        next_event_id: end,
        processed: end - next,
        done,
    }
}

pub fn get_stats() -> Stats {
    let total_classifications = crate::scoring::classifications_count();
    let active_aaas = crate::registry::active_aaas_count();
    let total_subjects = crate::catalog::total_subjects_count();
    let retired_subjects = crate::catalog::retired_subjects_count();
    let confirmed_discoveries = crate::discoveries::count_by_queue_status(1);
    let under_review_count = crate::discoveries::count_by_queue_status(0);

    Stats {
        total_classifications,
        active_aaas,
        confirmed_discoveries,
        under_review_count,
        total_subjects,
        retired_subjects,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ic_stable_structures::Storable;

    #[test]
    fn t2_6_progression_and_public_profile() {
        let aaa = Principal::from_slice(&[41, 42, 43, 44]);
        let owner = Principal::from_slice(&[51, 52, 53, 54]);

        let p_init = get_progress(&aaa);
        assert_eq!(p_init.tier, 1);
        assert_eq!(p_init.xp, 0);
        assert_eq!(calculate_reputation(&p_init), 5000);

        let ev1 = Event {
            v: 1,
            id: 1,
            at: 100,
            aaa,
            owner,
            kind: EventKind::Classified {
                classification_id: 1,
                subject_id: 10,
                gold: Some((2, 2)),
                fee: 0,
            },
        };
        apply_event(&ev1);

        let p1 = get_progress(&aaa);
        assert_eq!(p1.classifications, 1);
        assert_eq!(p1.xp, 2);
        assert_eq!(p1.gold_streak, 1);
        assert_eq!(p1.badges & 1, 1);

        let ev_fail = Event {
            v: 1,
            id: 2,
            at: 101,
            aaa,
            owner,
            kind: EventKind::Classified {
                classification_id: 2,
                subject_id: 11,
                gold: Some((1, 2)),
                fee: 0,
            },
        };
        apply_event(&ev_fail);
        let p_fail = get_progress(&aaa);
        assert_eq!(p_fail.gold_streak, 0);

        let ev_rev = Event {
            v: 1,
            id: 3,
            at: 102,
            aaa,
            owner,
            kind: EventKind::ReviewSubmitted {
                review_id: 1,
                seq: 1,
                honeypot: false,
                fee: 0,
            },
        };
        apply_event(&ev_rev);

        let ev_rev_score = Event {
            v: 1,
            id: 4,
            at: 103,
            aaa,
            owner,
            kind: EventKind::ReviewScored {
                review_id: 1,
                matched: true,
            },
        };
        apply_event(&ev_rev_score);

        let ev_flag = Event {
            v: 1,
            id: 5,
            at: 104,
            aaa,
            owner,
            kind: EventKind::DiscoveryFlagged { seq: 1 },
        };
        apply_event(&ev_flag);

        let ev_resolve = Event {
            v: 1,
            id: 6,
            at: 105,
            aaa,
            owner,
            kind: EventKind::DiscoveryResolved {
                seq: 1,
                outcome: "Confirmed".into(),
            },
        };
        apply_event(&ev_resolve);

        let ev_cycles = Event {
            v: 1,
            id: 7,
            at: 106,
            aaa,
            owner,
            kind: EventKind::CyclesContributed { amount: 1_000_000 },
        };
        apply_event(&ev_cycles);

        let p_final = get_progress(&aaa);
        assert_eq!(p_final.discoveries, 1);
        assert_eq!(p_final.confirmed, 1);
        assert_eq!(p_final.reviews, 1);
        assert_eq!(p_final.cycles_contributed, 1_000_000);
        assert!(p_final.xp >= 56);
    }

    #[test]
    fn t2_6_leaderboard_pages_bounded_at_100() {
        let lb = get_leaderboard(None, 200);
        assert!(lb.items.is_empty());

        let key = LeaderboardKey {
            inverted_xp: 12345,
            aaa: Principal::from_slice(&[1, 2, 3]),
        };
        let b = key.to_bytes();
        let key2 = LeaderboardKey::from_bytes(b);
        assert_eq!(key, key2);

        let stats = get_stats();
        assert_eq!(stats.confirmed_discoveries, 0);
        assert_eq!(stats.under_review_count, 0);

        let none_pub = get_aaa_public(&Principal::from_slice(&[99, 99]));
        assert!(none_pub.is_none());
    }

    #[test]
    fn t2_6_tier_transitions_and_leaderboard_ranking() {
        use sha2::{Digest, Sha256};
        let blob = vec![1, 2, 3, 4];
        let hash = Sha256::digest(&blob).to_vec();
        let _ = crate::registry::upload_wasm(100, blob, hash);
        let _ = crate::registry::approve_wasm(100, 50);

        let aaa_tier2 = Principal::from_slice(&[81, 82, 83]);
        let owner2 = Principal::from_slice(&[84, 85, 86]);
        let aaa_tier3 = Principal::from_slice(&[91, 92, 93]);
        let owner3 = Principal::from_slice(&[94, 95, 96]);

        crate::registry::pre_register_aaa(
            &crate::registry::RegisterArgs {
                canister_id: aaa_tier2,
                owner: owner2,
                name: "Leader-Two".into(),
                avatar_seed: 2,
            },
            1_000,
        )
        .unwrap();

        crate::registry::pre_register_aaa(
            &crate::registry::RegisterArgs {
                canister_id: aaa_tier3,
                owner: owner3,
                name: "Leader-Three".into(),
                avatar_seed: 3,
            },
            1_000,
        )
        .unwrap();

        for i in 1..=25 {
            apply_event(&Event {
                v: 1,
                id: i,
                at: 1_000 + i,
                aaa: aaa_tier2,
                owner: owner2,
                kind: EventKind::Classified {
                    classification_id: i,
                    subject_id: i as u32,
                    gold: Some((2, 2)),
                    fee: 0,
                },
            });
        }
        let p2 = get_progress(&aaa_tier2);
        assert!(p2.tier >= 2);

        for i in 1..=70 {
            apply_event(&Event {
                v: 1,
                id: 100 + i,
                at: 2_000 + i,
                aaa: aaa_tier3,
                owner: owner3,
                kind: EventKind::Classified {
                    classification_id: 100 + i,
                    subject_id: 100 + i as u32,
                    gold: Some((4, 4)),
                    fee: 0,
                },
            });
        }
        apply_event(&Event {
            v: 1,
            id: 200,
            at: 3_000,
            aaa: aaa_tier3,
            owner: owner3,
            kind: EventKind::DiscoveryResolved {
                seq: 1,
                outcome: "Confirmed".into(),
            },
        });
        for _ in 0..8 {
            apply_event(&Event {
                v: 1,
                id: 201,
                at: 3_001,
                aaa: aaa_tier3,
                owner: owner3,
                kind: EventKind::DiscoveryResolved {
                    seq: 2,
                    outcome: "Confirmed".into(),
                },
            });
        }
        let p3 = get_progress(&aaa_tier3);
        assert!(p3.tier >= 3);

        let lb = get_leaderboard(None, 10);
        assert_eq!(lb.items.len(), 2);
        assert_eq!(lb.items[0].aaa, aaa_tier3);
        assert_eq!(lb.items[0].rank, 1);
        assert_eq!(lb.items[1].aaa, aaa_tier2);
        assert_eq!(lb.items[1].rank, 2);

        let p_pub2 = get_aaa_public(&aaa_tier2).expect("public profile for tier 2");
        assert_eq!(p_pub2.name, "Leader-Two");
        assert!(p_pub2.tier >= 2);
        assert!(p_pub2.badges & (1 << 4) != 0);

        let p_pub3 = get_aaa_public(&aaa_tier3).expect("public profile for tier 3");
        assert_eq!(p_pub3.name, "Leader-Three");
        assert!(p_pub3.tier >= 3);

        let p_dummy = Progress {
            xp: 12000,
            gold_tasks: 350,
            gold_hits: 700,
            gold_trials: 700,
            ..Default::default()
        };
        assert_eq!(calculate_tier(&p_dummy, 9000), 5);
        assert_eq!(next_tier_xp(4), 10000);
        assert_eq!(next_tier_xp(5), 10000);

        let p_t4 = Progress {
            xp: 3000,
            gold_tasks: 160,
            gold_hits: 320,
            gold_trials: 320,
            ..Default::default()
        };
        assert_eq!(calculate_tier(&p_t4, 8500), 4);
    }

    #[test]
    fn t2_9_leaderboard_cursor_pages_through_ties_with_continuous_rank() {
        use sha2::Digest;
        let aaas: Vec<Principal> = (1..=5u8)
            .map(|i| Principal::from_slice(&[120, i]))
            .collect();
        let blob = vec![7, 7];
        let _ = crate::registry::upload_wasm(1, blob.clone(), sha2::Sha256::digest(&blob).to_vec());
        let _ = crate::registry::approve_wasm(1, 1);
        for (i, aaa) in aaas.iter().enumerate() {
            crate::registry::pre_register_aaa(
                &crate::registry::RegisterArgs {
                    canister_id: *aaa,
                    owner: Principal::from_slice(&[121, i as u8]),
                    name: format!("Tied-{i}"),
                    avatar_seed: 0,
                },
                1,
            )
            .unwrap();
            let xp = if i == 0 { 900 } else { 500 };
            LEADERBOARD_MAP.with_borrow_mut(|m| {
                m.insert(
                    LeaderboardKey {
                        inverted_xp: u64::MAX - xp,
                        aaa: *aaa,
                    },
                    (),
                )
            });
        }
        let mut seen = Vec::new();
        let mut ranks = Vec::new();
        let mut cursor = None;
        loop {
            let page = get_leaderboard(cursor, 2);
            for row in &page.items {
                seen.push(row.aaa);
                ranks.push(row.rank);
            }
            match page.next_cursor {
                Some(c) => cursor = Some(c),
                None => break,
            }
        }
        assert_eq!(ranks, vec![1, 2, 3, 4, 5]);
        assert_eq!(seen[0], aaas[0]);
        let mut unique = seen.clone();
        unique.sort();
        unique.dedup();
        assert_eq!(unique.len(), 5);
        assert_eq!(get_progress(&aaas[0]).v, 1);
    }

    #[test]
    fn t4_6_stats_counts_real_discoveries_and_excludes_honeypots() {
        use crate::discoveries::{self, DiscoveryStatus, NewDiscovery};
        use sc_types::Vote;

        let mut under_review = discoveries::create(NewDiscovery {
            subject_id: 1,
            classification_id: 1,
            discoverer_aaa: Principal::from_slice(&[220, 1]),
            discoverer_owner: Principal::from_slice(&[220, 2]),
            discoverer_name_at_time: "A".into(),
            category: "lens".into(),
            rationale: "an arc around the core".into(),
            confidence: 80,
            fee: 1,
            needed_reviews: 3,
            created_at: 1,
            claim_ra_deg: Some(0.0),
            claim_dec_deg: Some(0.0),
        });
        under_review.status = DiscoveryStatus::UnderReview;
        discoveries::put(&under_review);

        let mut confirmed = discoveries::create(NewDiscovery {
            subject_id: 2,
            classification_id: 2,
            discoverer_aaa: Principal::from_slice(&[220, 3]),
            discoverer_owner: Principal::from_slice(&[220, 4]),
            discoverer_name_at_time: "B".into(),
            category: "lens".into(),
            rationale: "another arc".into(),
            confidence: 80,
            fee: 1,
            needed_reviews: 3,
            created_at: 2,
            claim_ra_deg: Some(0.0),
            claim_dec_deg: Some(0.0),
        });
        confirmed.status = DiscoveryStatus::Confirmed;
        confirmed.resolved_at = Some(2);
        discoveries::put(&confirmed);

        discoveries::create_honeypot(1, "lens".into(), "r".into(), Vote::Disagree, 3);

        let stats = get_stats();
        assert_eq!(stats.under_review_count, 1);
        assert_eq!(stats.confirmed_discoveries, 1);
    }

    #[test]
    fn t4_11_house_aaas_follow_progress_but_are_excluded_from_leaderboard() {
        use sha2::Digest;
        let blob = vec![9, 8, 7];
        let _ =
            crate::registry::upload_wasm(300, blob.clone(), sha2::Sha256::digest(&blob).to_vec());
        let _ = crate::registry::approve_wasm(300, 1);

        let aaa = Principal::from_slice(&[230, 1]);
        let owner = Principal::from_slice(&[230, 2]);
        crate::registry::pre_register_aaa(
            &crate::registry::RegisterArgs {
                canister_id: aaa,
                owner,
                name: "House-One".into(),
                avatar_seed: 1,
            },
            1_000,
        )
        .unwrap();
        crate::registry::set_house(aaa, true).unwrap();

        for i in 1..=25 {
            apply_event(&Event {
                v: 1,
                id: i,
                at: 1_000 + i,
                aaa,
                owner,
                kind: EventKind::Classified {
                    classification_id: i,
                    subject_id: i as u32,
                    gold: Some((2, 2)),
                    fee: 0,
                },
            });
        }
        let p = get_progress(&aaa);
        assert!(p.tier >= 2);
        assert!(get_leaderboard(None, 100)
            .items
            .iter()
            .all(|row| row.aaa != aaa));
        assert!(get_aaa_public(&aaa).unwrap().is_house);

        crate::registry::set_house(aaa, false).unwrap();
        sync_leaderboard_house(aaa, false);
        assert!(get_leaderboard(None, 100)
            .items
            .iter()
            .any(|row| row.aaa == aaa));
        assert!(!get_aaa_public(&aaa).unwrap().is_house);
    }

    #[test]
    fn t4_3_new_aaa_reaches_tier2_within_60_honest_tasks() {
        let aaa = Principal::from_slice(&[201, 202, 203]);
        let owner = Principal::from_slice(&[204, 205, 206]);

        let mut tier2_at = None;
        for i in 1..=60u64 {
            apply_event(&Event {
                v: 1,
                id: i,
                at: i,
                aaa,
                owner,
                kind: EventKind::Classified {
                    classification_id: i,
                    subject_id: i as u32,
                    gold: Some((2, 2)),
                    fee: 0,
                },
            });
            if tier2_at.is_none() && get_progress(&aaa).tier >= 2 {
                tier2_at = Some(i);
            }
        }

        let reached = tier2_at.expect("tier 2 reached within 60 honest all-correct tasks");
        assert!(
            reached <= 60,
            "tier 2 must be reached in <= 60 tasks, got {reached}"
        );
        assert!(get_progress(&aaa).tier >= 2);
    }

    #[test]
    fn t4_3_replay_from_event_0_reproduces_identical_progress() {
        use crate::events;

        let aaa_1 = Principal::from_slice(&[210, 1]);
        let owner_1 = Principal::from_slice(&[210, 2]);
        let aaa_2 = Principal::from_slice(&[210, 3]);
        let owner_2 = Principal::from_slice(&[210, 4]);
        let admin = Principal::from_slice(&[210, 5]);

        for i in 1..=30u64 {
            events::record_event(
                i,
                aaa_1,
                owner_1,
                EventKind::Classified {
                    classification_id: i,
                    subject_id: i as u32,
                    gold: Some((2, 2)),
                    fee: 0,
                },
            );
        }
        events::record_event(
            31,
            aaa_1,
            owner_1,
            EventKind::ReviewSubmitted {
                review_id: 1,
                seq: 1,
                honeypot: false,
                fee: 0,
            },
        );
        events::record_event(
            32,
            aaa_1,
            owner_1,
            EventKind::ReviewScored {
                review_id: 1,
                matched: true,
            },
        );
        events::record_event(
            33,
            aaa_1,
            owner_1,
            EventKind::DiscoveryResolved {
                seq: 1,
                outcome: "Confirmed".into(),
            },
        );
        events::record_event(
            34,
            aaa_2,
            owner_2,
            EventKind::Classified {
                classification_id: 1_000,
                subject_id: 1,
                gold: None,
                fee: 0,
            },
        );
        events::record_event(
            35,
            admin,
            Principal::anonymous(),
            EventKind::Admin {
                method: "admin_pause".into(),
            },
        );

        let expected_1 = get_progress(&aaa_1);
        let expected_2 = get_progress(&aaa_2);
        let expected_lb = get_leaderboard(None, 100);

        PROGRESS_MAP.with_borrow_mut(|m| {
            m.insert(
                aaa_1,
                Progress {
                    xp: 999_999,
                    ..Default::default()
                },
            );
        });
        LEADERBOARD_MAP.with_borrow_mut(|m| {
            let keys: Vec<LeaderboardKey> = m.iter().map(|e| *e.key()).collect();
            for k in keys {
                m.remove(&k);
            }
        });
        assert_ne!(get_progress(&aaa_1), expected_1);

        let status = run_replay(3);
        assert_eq!(status.next_event_id, events::len());

        assert_eq!(get_progress(&aaa_1), expected_1);
        assert_eq!(get_progress(&aaa_2), expected_2);
        assert_eq!(get_progress(&admin), Progress::default());
        assert_eq!(get_leaderboard(None, 100), expected_lb);
    }

    fn run_replay(batch: u32) -> ReplayStatus {
        let mut status = start_replay(0, batch).unwrap();
        while !status.done {
            status = replay_step();
        }
        status
    }

    fn classify(i: u64, gold: Option<(u8, u8)>, fee: u64) -> EventKind {
        EventKind::Classified {
            classification_id: i,
            subject_id: i as u32,
            gold,
            fee,
        }
    }

    #[test]
    fn t4_3_replay_applies_events_recorded_mid_replay_exactly_once() {
        use crate::events;
        let aaa = Principal::from_slice(&[240, 1]);
        let owner = Principal::from_slice(&[240, 2]);
        for i in 0..4 {
            events::record_event(i, aaa, owner, classify(i, None, 0));
        }
        start_replay(0, 2).unwrap();
        assert!(replay_active());
        let mut status = replay_step();
        assert!(!status.done);
        events::record_event(10, aaa, owner, classify(10, None, 0));
        assert!(get_progress(&aaa).classifications < 4);
        while !status.done {
            status = replay_step();
        }
        assert!(!replay_active());
        assert_eq!(get_progress(&aaa).classifications, 5);
        events::record_event(11, aaa, owner, classify(11, None, 0));
        assert_eq!(get_progress(&aaa).classifications, 6);
    }

    #[test]
    fn t4_3_replay_rejects_second_and_non_zero_start_and_batches_clear() {
        let owner = Principal::from_slice(&[241, 0]);
        for i in 0..7u8 {
            apply_event(&Event {
                v: 1,
                id: 0,
                at: 1,
                aaa: Principal::from_slice(&[241, 1, i]),
                owner,
                kind: classify(1, None, 0),
            });
        }
        assert!(matches!(start_replay(5, 2), Err(ApiError::InvalidInput(_))));
        let first = start_replay(0, 3).unwrap();
        assert!(!first.done);
        assert_eq!(first.processed, 0);
        assert_eq!(PROGRESS_MAP.with_borrow(|m| m.len()), 4);
        assert!(matches!(start_replay(0, 3), Err(ApiError::Conflict(_))));
        let mut status = replay_step();
        while !status.done {
            status = replay_step();
        }
        for i in 0..7u8 {
            assert_eq!(
                get_progress(&Principal::from_slice(&[241, 1, i])).classifications,
                0
            );
        }
        assert!(start_replay(0, 3).is_ok());
    }

    #[test]
    fn t4_3_tier_and_badge_changes_emit_events_live_but_not_on_replay() {
        use crate::events;
        let aaa = Principal::from_slice(&[242, 1]);
        let owner = Principal::from_slice(&[242, 2]);
        let kinds = |from: u64| -> Vec<EventKind> {
            (from..events::len())
                .filter_map(events::get_event)
                .filter(|e| e.aaa == aaa)
                .map(|e| e.kind)
                .collect()
        };
        let start = events::len();
        for i in 1..=25u64 {
            events::record_event(i, aaa, owner, classify(i, Some((2, 2)), 0));
        }
        let live = kinds(start);
        assert!(live.contains(&EventKind::BadgeAwarded {
            badge: "first_light".into()
        }));
        assert!(live.contains(&EventKind::TierChanged { from: 1, to: 2 }));
        assert_eq!(
            live.iter()
                .filter(|k| matches!(k, EventKind::BadgeAwarded { .. }))
                .count(),
            2
        );
        let before = get_progress(&aaa);
        let len = events::len();
        run_replay(REPLAY_BATCH_MAX);
        assert_eq!(events::len(), len);
        assert_eq!(get_progress(&aaa), before);
    }

    #[test]
    fn t4_3_fees_count_as_cycles_contributed_and_corroborators_earn_5_xp() {
        let aaa = Principal::from_slice(&[243, 1]);
        let owner = Principal::from_slice(&[243, 2]);
        let ev = |kind| Event {
            v: 1,
            id: 0,
            at: 1,
            aaa,
            owner,
            kind,
        };
        apply_event(&ev(classify(1, None, 200)));
        apply_event(&ev(EventKind::ReviewSubmitted {
            review_id: 1,
            seq: 1,
            honeypot: false,
            fee: 50,
        }));
        apply_event(&ev(EventKind::CorroborationConfirmed { seq: 1 }));
        let p = get_progress(&aaa);
        assert_eq!(p.cycles_contributed, 250);
        assert_eq!(p.xp, 1 + 3 + 5);
    }

    fn t4_8_event_kind_strategy() -> impl proptest::strategy::Strategy<Value = EventKind> {
        use proptest::prelude::*;
        prop_oneof![
            (0u8..=4, 0u8..=4).prop_map(|(a, b)| {
                let (hits, trials) = (a.min(b), a.max(b));
                EventKind::Classified {
                    classification_id: 0,
                    subject_id: 0,
                    gold: Some((hits, trials)),
                    fee: 0,
                }
            }),
            Just(EventKind::Classified {
                classification_id: 0,
                subject_id: 0,
                gold: None,
                fee: 0,
            }),
            Just(EventKind::ReviewSubmitted {
                review_id: 0,
                seq: 0,
                honeypot: false,
                fee: 0,
            }),
            any::<bool>().prop_map(|matched| EventKind::ReviewScored {
                review_id: 0,
                matched,
            }),
            Just(EventKind::DiscoveryFlagged { seq: 0 }),
            any::<bool>().prop_map(|confirmed| EventKind::DiscoveryResolved {
                seq: 0,
                outcome: if confirmed { "Confirmed" } else { "Rejected" }.into(),
            }),
            (0u64..1_000_000_000u64).prop_map(|amount| EventKind::CyclesContributed {
                amount: amount as u128
            }),
        ]
    }

    static T4_8_NEXT_CASE: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);

    fn t4_8_unique_principal(marker: u8) -> Principal {
        let n = T4_8_NEXT_CASE.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        Principal::from_slice(&[
            marker,
            (n >> 24) as u8,
            (n >> 16) as u8,
            (n >> 8) as u8,
            n as u8,
        ])
    }

    proptest::proptest! {
        #![proptest_config(proptest::prelude::ProptestConfig { failure_persistence: None, cases: 64, ..proptest::prelude::ProptestConfig::default() })]

        #[test]
        fn t4_8_prop_replay_from_zero_reproduces_incremental_progress_for_every_aaa(
            events_a in proptest::collection::vec(t4_8_event_kind_strategy(), 0..12),
            events_b in proptest::collection::vec(t4_8_event_kind_strategy(), 0..12),
        ) {
            let aaa_a = t4_8_unique_principal(210);
            let owner_a = t4_8_unique_principal(211);
            let aaa_b = t4_8_unique_principal(212);
            let owner_b = t4_8_unique_principal(213);

            let mut at = 1u64;
            for kind in &events_a {
                crate::events::record_event(at, aaa_a, owner_a, kind.clone());
                at += 1;
            }
            for kind in &events_b {
                crate::events::record_event(at, aaa_b, owner_b, kind.clone());
                at += 1;
            }
            crate::events::record_event(
                at,
                t4_8_unique_principal(214),
                Principal::anonymous(),
                EventKind::Admin { method: "noop".into() },
            );

            let incremental_a = get_progress(&aaa_a);
            let incremental_b = get_progress(&aaa_b);

            run_replay(REPLAY_BATCH_MAX);

            proptest::prop_assert_eq!(get_progress(&aaa_a), incremental_a);
            proptest::prop_assert_eq!(get_progress(&aaa_b), incremental_b);
        }
    }
}
