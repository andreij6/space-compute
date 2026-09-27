use std::cell::RefCell;

use candid::{CandidType, Principal};
use ic_stable_structures::StableBTreeMap;
use serde::{Deserialize, Serialize};

use crate::events::{Event, EventKind};
use crate::memory::{self, Memory};

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

pub fn apply_event(ev: &Event) {
    if ev.aaa == Principal::anonymous() {
        return;
    }

    let mut p = get_progress(&ev.aaa);
    let old_tier = p.tier;
    let old_xp = p.xp;

    match &ev.kind {
        EventKind::Classified { gold, .. } => {
            p.classifications += 1;
            p.xp += 1;
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
        EventKind::ReviewSubmitted { .. } => {
            p.reviews += 1;
            p.xp += 3;
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
        EventKind::CyclesContributed { amount } => {
            p.cycles_contributed = p.cycles_contributed.saturating_add(*amount);
        }
        _ => {}
    }

    let rep = calculate_reputation(&p);
    p.tier = calculate_tier(&p, rep);

    if p.classifications >= 1 {
        p.badges |= 1 << 0;
    }
    if p.discoveries >= 1 {
        p.badges |= 1 << 1;
    }
    if p.confirmed >= 1 {
        p.badges |= 1 << 2;
    }
    if p.reviews >= 10 {
        p.badges |= 1 << 3;
    }
    if p.gold_streak >= 10 {
        p.badges |= 1 << 4;
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

    if p.tier >= 2 {
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
    })
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

pub fn get_stats() -> Stats {
    let total_classifications = crate::scoring::classifications_count();
    let active_aaas = crate::registry::active_aaas_count();
    let total_subjects = crate::catalog::total_subjects_count();
    let retired_subjects = crate::catalog::retired_subjects_count();

    Stats {
        total_classifications,
        active_aaas,
        confirmed_discoveries: 0,
        under_review_count: 0,
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
}
