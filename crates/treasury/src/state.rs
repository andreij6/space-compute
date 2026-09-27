use std::cell::RefCell;

use candid::{CandidType, Principal};
use ic_stable_structures::{StableBTreeMap, StableCell, StableLog};
use sc_types::ApiError;
use serde::Deserialize;

use crate::config::Config;
use crate::keeper;
use crate::memory::{self, Memory};

pub const MAX_WATCHED: usize = 50;

#[derive(CandidType, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct Watch {
    pub v: u8,
    pub priority: u8,
    pub target_days: u32,
}

#[derive(CandidType, Deserialize, Clone, Debug, Default, PartialEq, Eq)]
pub struct Metric {
    pub v: u8,
    pub balance: u128,
    pub at: u64,
    pub ema: u128,
    pub burn_per_day: u128,
}

#[derive(CandidType, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct Inflight {
    pub canister: Principal,
    pub e8s: u64,
    pub created_at: u64,
    pub block: Option<u64>,
    pub attempts: u8,
}

#[derive(CandidType, Deserialize, Clone, Debug, Default, PartialEq, Eq)]
pub struct Snapshot {
    pub v: u8,
    pub icp_balance_e8s: u64,
    pub xdr_permyriad_per_icp: u64,
    pub checked_at: u64,
    pub reserve_blocked: bool,
    pub inflight: Option<Inflight>,
}

#[derive(CandidType, Deserialize, Clone, Debug, PartialEq, Eq)]
pub enum Action {
    Withdraw { to: Principal, amount_e8s: u64 },
    SetConfig(Config),
}

#[derive(CandidType, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct Proposal {
    pub v: u8,
    pub action: Action,
    pub proposer: Principal,
    pub proposed_at: u64,
    pub executing_since: Option<u64>,
}

#[derive(CandidType, Deserialize, Clone, Debug, PartialEq, Eq)]
pub enum HistoryKind {
    TopUp {
        canister: Principal,
        e8s: u64,
        cycles: u128,
        block: u64,
    },
    TopUpFailed {
        canister: Principal,
        reason: String,
    },
    SkippedReserve {
        canister: Principal,
        needed_e8s: u64,
    },
    Proposed {
        id: u64,
        summary: String,
    },
    WithdrawFailed {
        id: u64,
        reason: String,
    },
    ConfigChanged {
        id: u64,
    },
    Deposit {
        e8s: u64,
    },
    Withdrawn {
        id: u64,
        to: Principal,
        amount_e8s: u64,
        block: u64,
    },
    Admin {
        method: String,
        summary: String,
    },
}

#[derive(CandidType, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct HistoryItem {
    pub v: u8,
    pub seq: u64,
    pub at: u64,
    pub actor: Principal,
    pub kind: HistoryKind,
}

crate::candid_storable!(Watch);
crate::candid_storable!(Metric);
crate::candid_storable!(Snapshot);
crate::candid_storable!(Proposal);
crate::candid_storable!(HistoryItem);

thread_local! {
    static WATCH: RefCell<StableBTreeMap<Principal, Watch, Memory>> =
        RefCell::new(StableBTreeMap::init(memory::get(memory::WATCH)));
    static METRICS: RefCell<StableBTreeMap<Principal, Metric, Memory>> =
        RefCell::new(StableBTreeMap::init(memory::get(memory::METRICS)));
    static SNAPSHOT: RefCell<StableCell<Snapshot, Memory>> =
        RefCell::new(StableCell::init(memory::get(memory::SNAPSHOT), Snapshot::default()));
    static PROPOSALS: RefCell<StableBTreeMap<u64, Proposal, Memory>> =
        RefCell::new(StableBTreeMap::init(memory::get(memory::PROPOSALS)));
    static HISTORY: RefCell<StableLog<HistoryItem, Memory, Memory>> = RefCell::new(StableLog::init(
        memory::get(memory::HISTORY_INDEX),
        memory::get(memory::HISTORY_DATA),
    ));
}

pub fn watch(canister: Principal, priority: u8, target_days: u32) -> Result<(), ApiError> {
    if !(1..=3_650).contains(&target_days) {
        return Err(ApiError::invalid("target_days must be in 1..=3650"));
    }
    WATCH.with_borrow_mut(|w| {
        if !w.contains_key(&canister) && w.len() as usize >= MAX_WATCHED {
            return Err(ApiError::invalid("watch list is full"));
        }
        w.insert(
            canister,
            Watch {
                v: 1,
                priority,
                target_days,
            },
        );
        Ok(())
    })
}

pub fn unwatch(canister: Principal) -> Result<(), ApiError> {
    WATCH.with_borrow_mut(|w| w.remove(&canister).map(|_| ()).ok_or(ApiError::NotFound))
}

pub fn watched() -> Vec<(Principal, Watch)> {
    let mut all: Vec<_> = WATCH.with_borrow(|w| w.iter().map(|e| (*e.key(), e.value())).collect());
    all.sort_by_key(|(p, w)| (w.priority, *p));
    all
}

pub fn metric(canister: &Principal) -> Option<Metric> {
    METRICS.with_borrow(|m| m.get(canister))
}

pub fn record_sample(canister: Principal, balance: u128, at: u64, floor_burn: u128) -> Metric {
    let prev = metric(&canister).unwrap_or_default();
    let ema = if prev.at == 0 {
        0
    } else {
        keeper::update_burn(prev.ema, prev.balance, prev.at, balance, at)
    };
    let next = Metric {
        v: 1,
        balance,
        at,
        ema,
        burn_per_day: ema.max(floor_burn),
    };
    METRICS.with_borrow_mut(|m| m.insert(canister, next.clone()));
    next
}

pub fn snapshot() -> Snapshot {
    SNAPSHOT.with_borrow(|s| s.get().clone())
}

pub fn set_snapshot(s: Snapshot) {
    SNAPSHOT.with_borrow_mut(|c| c.set(s));
}

pub fn log(at: u64, actor: Principal, kind: HistoryKind) -> u64 {
    HISTORY.with_borrow(|h| {
        let seq = h.len();
        h.append(&HistoryItem {
            v: 1,
            seq,
            at,
            actor,
            kind,
        })
        .expect("history append")
    })
}

pub fn history(cursor: Option<u64>, limit: u32) -> (Vec<HistoryItem>, Option<u64>) {
    let limit = sc_types::limits::page_limit(limit) as u64;
    HISTORY.with_borrow(|h| {
        let start = cursor.unwrap_or(0);
        let end = h.len().min(start.saturating_add(limit));
        let items = (start..end).filter_map(|i| h.get(i)).collect();
        (items, (end < h.len()).then_some(end))
    })
}

pub fn propose(action: Action, proposer: Principal, now: u64) -> u64 {
    PROPOSALS.with_borrow_mut(|m| {
        let id = m.last_key_value().map(|(k, _)| k + 1).unwrap_or(1);
        m.insert(
            id,
            Proposal {
                v: 1,
                action,
                proposer,
                proposed_at: now,
                executing_since: None,
            },
        );
        id
    })
}

pub fn approve(
    id: u64,
    approver: Principal,
    now: u64,
    window_ns: u64,
) -> Result<Proposal, ApiError> {
    PROPOSALS.with_borrow_mut(|m| {
        let mut p = m.get(&id).ok_or(ApiError::NotFound)?;
        if p.executing_since.is_some() {
            return Ok(p);
        }
        if p.proposer == approver {
            return Err(ApiError::NotEligible(
                "a second, different admin must approve".into(),
            ));
        }
        if now.saturating_sub(p.proposed_at) > window_ns {
            m.remove(&id);
            return Err(ApiError::Conflict("approval window expired".into()));
        }
        p.executing_since = Some(now);
        m.insert(id, p.clone());
        Ok(p)
    })
}

pub fn finish(id: u64) {
    PROPOSALS.with_borrow_mut(|m| m.remove(&id));
}

pub fn release(id: u64) {
    PROPOSALS.with_borrow_mut(|m| {
        if let Some(mut p) = m.get(&id) {
            p.executing_since = None;
            m.insert(id, p);
        }
    });
}

pub fn proposals() -> Vec<(u64, Proposal)> {
    PROPOSALS.with_borrow(|m| m.iter().map(|e| (*e.key(), e.value())).collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn p(n: u8) -> Principal {
        Principal::from_slice(&[n; 29])
    }

    #[test]
    fn t5_17_watch_list_orders_by_priority_and_validates() {
        watch(p(3), 2, 60).unwrap();
        watch(p(1), 0, 30).unwrap();
        watch(p(2), 1, 60).unwrap();
        assert_eq!(
            watched().iter().map(|(c, _)| *c).collect::<Vec<_>>(),
            [p(1), p(2), p(3)]
        );
        assert!(matches!(watch(p(4), 0, 0), Err(ApiError::InvalidInput(_))));
        unwatch(p(2)).unwrap();
        assert_eq!(unwatch(p(2)), Err(ApiError::NotFound));
        for i in 10..(10 + MAX_WATCHED as u8 - 2) {
            watch(p(i), 5, 60).unwrap();
        }
        assert!(matches!(
            watch(p(200), 5, 60),
            Err(ApiError::InvalidInput(_))
        ));
        watch(p(1), 0, 90).unwrap();
    }

    #[test]
    fn t5_17_samples_track_burn_with_floor() {
        let day = 86_400_000_000_000u64;
        let first = record_sample(p(1), 10, 1, 5);
        assert_eq!(first.burn_per_day, 5);
        let second = record_sample(p(1), 1_000_000_000_000, day + 1, 0);
        assert_eq!(second.burn_per_day, 0);
        let third = record_sample(p(1), 900_000_000_000, 2 * day + 1, 0);
        assert_eq!(third.burn_per_day, 100_000_000_000);
        assert_eq!(metric(&p(1)), Some(third));
    }

    #[test]
    fn t5_17_history_pages_and_snapshot_round_trips() {
        for i in 0..5 {
            log(
                i,
                p(1),
                HistoryKind::Admin {
                    method: "m".into(),
                    summary: i.to_string(),
                },
            );
        }
        let (page, next) = history(None, 2);
        assert_eq!(page.iter().map(|h| h.seq).collect::<Vec<_>>(), [0, 1]);
        assert_eq!(next, Some(2));
        assert_eq!(
            history(Some(4), 10),
            (vec![history(Some(4), 1).0[0].clone()], None)
        );
        let s = Snapshot {
            v: 1,
            icp_balance_e8s: 7,
            xdr_permyriad_per_icp: 3,
            checked_at: 9,
            reserve_blocked: true,
            inflight: None,
        };
        set_snapshot(s.clone());
        assert_eq!(snapshot(), s);
    }

    #[test]
    fn t5_17_proposals_need_a_second_admin_and_keep_executing_ones() {
        let w = Action::Withdraw {
            to: p(9),
            amount_e8s: 100,
        };
        let id = propose(w.clone(), p(1), 10);
        assert!(matches!(
            approve(id, p(1), 11, 100),
            Err(ApiError::NotEligible(_))
        ));
        let approved = approve(id, p(2), 50, 100).unwrap();
        assert_eq!(approved.executing_since, Some(50));
        assert_eq!(
            approve(id, p(1), 60, 100).unwrap().executing_since,
            Some(50)
        );
        release(id);
        assert_eq!(proposals()[0].1.executing_since, None);
        finish(id);
        assert_eq!(approve(id, p(2), 50, 100), Err(ApiError::NotFound));
        let late = propose(w, p(1), 10);
        assert!(matches!(
            approve(late, p(2), 500, 100),
            Err(ApiError::Conflict(_))
        ));
        assert!(proposals().is_empty());
        release(999);
    }
}
