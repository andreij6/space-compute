use std::cell::RefCell;

use candid::{CandidType, Principal};
use ic_stable_structures::StableBTreeMap;
use sc_types::ApiError;
use serde::Deserialize;

use crate::memory::{self, Memory};

#[derive(CandidType, Deserialize, Clone, Debug, PartialEq)]
pub struct Account {
    pub owner: Principal,
    pub subaccount: Option<[u8; 32]>,
}

#[derive(CandidType, Deserialize, Clone, Debug, PartialEq)]
pub enum FuelSource {
    Btc { sats: u64 },
    Eth { wei: u128 },
}

#[derive(CandidType, Deserialize, Clone, Debug, PartialEq)]
pub enum OpKind {
    Spawn {
        owner: Principal,
        name: String,
        avatar_seed: u64,
    },
    TopUp {
        aaa: Principal,
    },
    AutoTopUp {
        aaa: Principal,
    },
    FuelPack {
        aaa: Principal,
        source: FuelSource,
        usd_cents: u32,
        packs: u8,
    },
}

#[derive(CandidType, Deserialize, Clone, Debug, PartialEq)]
pub enum PayPath {
    Wallet { payer: Account },
    Deposit,
    Treasury,
    Invite { code: String },
}

#[derive(CandidType, Deserialize, Clone, Debug, PartialEq)]
pub enum NotifiedInfo {
    Canister(Principal),
    Cycles(u128),
}

#[derive(CandidType, Deserialize, Clone, Debug, PartialEq)]
pub enum OpState {
    Pending,
    Pulled { block: u64 },
    Credited { usd_cents: u32 },
    TreasuryPaid { block: u64 },
    Notified { canister_or_cycles: NotifiedInfo },
    Registered,
    Done,
    Failed { reason: String },
    Refunded { block: u64 },
}

impl OpState {
    pub fn is_terminal(&self) -> bool {
        matches!(
            self,
            OpState::Done | OpState::Failed { .. } | OpState::Refunded { .. }
        )
    }

    fn discriminant(&self) -> u8 {
        match self {
            OpState::Pending => 0,
            OpState::Pulled { .. } => 1,
            OpState::Credited { .. } => 2,
            OpState::TreasuryPaid { .. } => 3,
            OpState::Notified { .. } => 4,
            OpState::Registered => 5,
            OpState::Done => 6,
            OpState::Failed { .. } => 7,
            OpState::Refunded { .. } => 8,
        }
    }

    pub fn can_advance_to(&self, next: &OpState) -> bool {
        use OpState::*;
        if self.is_terminal() {
            return false;
        }
        matches!(
            (self, next),
            (_, Failed { .. })
                | (Pulled { .. }, Refunded { .. })
                | (TreasuryPaid { .. }, Refunded { .. })
                | (Pending, Pulled { .. })
                | (Pending, Credited { .. })
                | (Credited { .. }, TreasuryPaid { .. })
                | (Pulled { .. }, Notified { .. })
                | (TreasuryPaid { .. }, Notified { .. })
                | (Notified { .. }, Registered)
                | (Notified { .. }, Done)
                | (Registered, Done)
        )
    }
}

crate::candid_storable!(OpState);

#[derive(CandidType, Deserialize, Clone, Debug, PartialEq)]
pub struct Op {
    pub v: u8,
    pub id: u64,
    pub kind: OpKind,
    pub path: PayPath,
    pub amount_e8s: u64,
    pub created_at: u64,
    pub created_by: Principal,
    pub state: OpState,
    pub updated_at: u64,
    pub attempts: u8,
    pub pull_e8s: Option<u64>,
}

crate::candid_storable!(Op);

impl Op {
    pub fn spawn_fields(&self) -> Result<(Principal, String, u64), ApiError> {
        match &self.kind {
            OpKind::Spawn {
                owner,
                name,
                avatar_seed,
            } => Ok((*owner, name.clone(), *avatar_seed)),
            _ => Err(ApiError::Internal("op is not a Spawn op".into())),
        }
    }

    pub fn topup_fields(&self) -> Result<Principal, ApiError> {
        match &self.kind {
            OpKind::TopUp { aaa } => Ok(*aaa),
            _ => Err(ApiError::Internal("op is not a TopUp op".into())),
        }
    }

    pub fn auto_topup_fields(&self) -> Result<Principal, ApiError> {
        match &self.kind {
            OpKind::AutoTopUp { aaa } => Ok(*aaa),
            _ => Err(ApiError::Internal("op is not an AutoTopUp op".into())),
        }
    }
}

thread_local! {
    static OPS: RefCell<StableBTreeMap<u64, Op, Memory>> =
        RefCell::new(StableBTreeMap::init(memory::get(memory::OPS)));
    static AAA_INDEX: RefCell<StableBTreeMap<(Principal, u64), (), Memory>> =
        RefCell::new(StableBTreeMap::init(memory::get(memory::AAA_OPS_INDEX)));
    static OWNER_INDEX: RefCell<StableBTreeMap<(Principal, u64), (), Memory>> =
        RefCell::new(StableBTreeMap::init(memory::get(memory::OWNER_OPS_INDEX)));
}

pub fn index_aaa(aaa: Principal, op_id: u64) {
    AAA_INDEX.with_borrow_mut(|m| m.insert((aaa, op_id), ()));
}

pub fn index_owner(owner: Principal, op_id: u64) {
    OWNER_INDEX.with_borrow_mut(|m| m.insert((owner, op_id), ()));
}

fn op_principals(op: &Op) -> (Option<Principal>, Option<Principal>) {
    match &op.kind {
        OpKind::Spawn { owner, .. } => (None, Some(*owner)),
        OpKind::TopUp { aaa } => (Some(*aaa), Some(op.created_by)),
        OpKind::AutoTopUp { aaa } => (Some(*aaa), None),
        OpKind::FuelPack { aaa, .. } => (Some(*aaa), None),
    }
}

pub fn backfill_indexes() {
    OPS.with_borrow(|m| {
        for entry in m.iter() {
            let op = entry.value();
            let (aaa, owner) = op_principals(&op);
            if let Some(aaa) = aaa {
                index_aaa(aaa, op.id);
            }
            if let Some(owner) = owner {
                index_owner(owner, op.id);
            }
        }
    });
}

pub fn backfill_indexes_once() {
    if crate::config::get().indexes_backfilled == Some(true) {
        return;
    }
    backfill_indexes();
    crate::config::update(|c| {
        c.indexes_backfilled = Some(true);
        Ok(())
    })
    .expect("mark indexes backfilled");
}

pub fn create(kind: OpKind, path: PayPath, amount_e8s: u64, created_by: Principal, now: u64) -> Op {
    let id = OPS.with_borrow(|m| m.last_key_value().map(|(k, _)| k + 1).unwrap_or(0));
    let op = Op {
        v: 1,
        id,
        kind,
        path,
        amount_e8s,
        created_at: now,
        created_by,
        state: OpState::Pending,
        updated_at: now,
        attempts: 0,
        pull_e8s: None,
    };
    OPS.with_borrow_mut(|m| m.insert(id, op.clone()));
    op
}

pub fn get(id: u64) -> Option<Op> {
    OPS.with_borrow(|m| m.get(&id))
}

pub fn advance(id: u64, next: OpState, now: u64) -> Result<Op, ApiError> {
    OPS.with_borrow_mut(|m| {
        let mut op = m.get(&id).ok_or(ApiError::NotFound)?;
        if !op.state.can_advance_to(&next) {
            return Err(ApiError::Conflict(format!(
                "op {id}: cannot advance {:?} -> {:?}",
                op.state.discriminant(),
                next.discriminant()
            )));
        }
        op.state = next;
        op.updated_at = now;
        op.attempts = op.attempts.saturating_add(1);
        m.insert(id, op.clone());
        Ok(op)
    })
}

pub fn fix_pull(id: u64, amount_e8s: u64) -> Result<u64, ApiError> {
    OPS.with_borrow_mut(|m| {
        let mut op = m.get(&id).ok_or(ApiError::NotFound)?;
        if let Some(fixed) = op.pull_e8s {
            return Ok(fixed);
        }
        if op.state != OpState::Pending {
            return Err(ApiError::Conflict(format!("op {id}: pull already settled")));
        }
        op.pull_e8s = Some(amount_e8s);
        m.insert(id, op);
        Ok(amount_e8s)
    })
}

pub fn resumable_from(watermark: u64, limit: usize) -> (Vec<Op>, u64) {
    OPS.with_borrow(|m| {
        let mut next_watermark = None;
        let mut out = Vec::new();
        let mut last = None;
        for entry in m.range(watermark..) {
            let op = entry.value();
            last = Some(op.id);
            if op.state.is_terminal() {
                continue;
            }
            next_watermark.get_or_insert(op.id);
            if out.len() == limit {
                break;
            }
            out.push(op);
        }
        let fallback = last.map(|l| l + 1).unwrap_or(watermark);
        (out, next_watermark.unwrap_or(fallback))
    })
}

pub fn list_by_created(cursor: Option<u64>, limit: u32) -> Vec<Op> {
    let limit = sc_types::limits::page_limit(limit) as usize;
    let start = cursor.unwrap_or(0);
    OPS.with_borrow(|m| m.range(start..).take(limit).map(|e| e.value()).collect())
}

#[derive(CandidType, Deserialize, Clone, Debug, PartialEq)]
pub struct Page<T> {
    pub items: Vec<T>,
    pub next_cursor: Option<u64>,
}

#[derive(CandidType, Deserialize, Clone, Debug, Default)]
pub struct OpFilter {
    pub state: Option<OpState>,
    pub kind: Option<OpKind>,
    pub since: Option<u64>,
}

impl OpFilter {
    pub fn matches(&self, op: &Op) -> bool {
        if let Some(k) = &self.kind {
            if std::mem::discriminant(k) != std::mem::discriminant(&op.kind) {
                return false;
            }
        }
        if let Some(s) = &self.state {
            if std::mem::discriminant(s) != std::mem::discriminant(&op.state) {
                return false;
            }
        }
        if let Some(since) = self.since {
            if op.created_at < since {
                return false;
            }
        }
        true
    }
}

pub fn list_filtered(filter: &OpFilter, cursor: Option<u64>, limit: u32) -> Page<Op> {
    let limit = sc_types::limits::page_limit(limit) as usize;
    let start = cursor.unwrap_or(0);
    OPS.with_borrow(|m| {
        let mut items = Vec::new();
        let mut next_cursor = None;
        for entry in m.range(start..) {
            let op = entry.value();
            if !filter.matches(&op) {
                continue;
            }
            if items.len() == limit {
                next_cursor = Some(op.id);
                break;
            }
            items.push(op);
        }
        Page { items, next_cursor }
    })
}

fn page_from_ids(mut ids: Vec<u64>, limit: usize) -> Page<Op> {
    let has_more = ids.len() > limit;
    ids.truncate(limit);
    let next_cursor = if has_more { ids.last().copied() } else { None };
    let items = ids.into_iter().filter_map(get).collect();
    Page { items, next_cursor }
}

pub fn list_for_aaa(aaa: Principal, cursor: Option<u64>, limit: u32) -> Page<Op> {
    let limit = sc_types::limits::page_limit(limit) as usize;
    let upper = cursor.unwrap_or(u64::MAX);
    let ids = AAA_INDEX.with_borrow(|m| {
        m.range((aaa, 0)..(aaa, upper))
            .rev()
            .take(limit + 1)
            .map(|e| e.key().1)
            .collect()
    });
    page_from_ids(ids, limit)
}

pub fn list_for_owner(owner: Principal, cursor: Option<u64>, limit: u32) -> Page<Op> {
    let limit = sc_types::limits::page_limit(limit) as usize;
    let upper = cursor.unwrap_or(u64::MAX);
    let ids = OWNER_INDEX.with_borrow(|m| {
        m.range((owner, 0)..(owner, upper))
            .rev()
            .take(limit + 1)
            .map(|e| e.key().1)
            .collect()
    });
    page_from_ids(ids, limit)
}

fn kind_label(kind: &OpKind) -> &'static str {
    match kind {
        OpKind::Spawn { .. } => "spawn",
        OpKind::TopUp { .. } => "topup",
        OpKind::AutoTopUp { .. } => "auto_topup",
        OpKind::FuelPack { .. } => "fuel_pack",
    }
}

pub fn stats_since(now: u64, since_secs_ago: u64) -> Vec<(String, u64, u64)> {
    let since = now.saturating_sub(since_secs_ago.saturating_mul(1_000_000_000));
    let labels = ["spawn", "topup", "auto_topup", "fuel_pack"];
    let mut counts = [0u64; 4];
    let mut spends = [0u64; 4];
    OPS.with_borrow(|m| {
        for entry in m.iter() {
            let op = entry.value();
            if op.created_at < since {
                continue;
            }
            let idx = labels
                .iter()
                .position(|l| *l == kind_label(&op.kind))
                .unwrap();
            counts[idx] += 1;
            spends[idx] = spends[idx].saturating_add(op.amount_e8s);
        }
    });
    labels
        .iter()
        .enumerate()
        .map(|(i, l)| (l.to_string(), counts[i], spends[i]))
        .collect()
}

pub fn failed_count() -> u64 {
    OPS.with_borrow(|m| {
        m.iter()
            .filter(|e| matches!(e.value().state, OpState::Failed { .. }))
            .count() as u64
    })
}

pub const STUCK_AFTER_SECS: u64 = 3_600;

pub fn stuck_count(now: u64, stuck_after_secs: u64) -> u64 {
    let threshold_ns = stuck_after_secs.saturating_mul(1_000_000_000);
    OPS.with_borrow(|m| {
        m.iter()
            .filter(|e| {
                let op = e.value();
                !op.state.is_terminal() && now.saturating_sub(op.updated_at) > threshold_ns
            })
            .count() as u64
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn p(n: u8) -> Principal {
        Principal::from_slice(&[n; 29])
    }

    #[test]
    fn t5_1_create_assigns_monotonic_ids_starting_pending() {
        let a = create(OpKind::TopUp { aaa: p(1) }, PayPath::Deposit, 0, p(9), 1);
        let b = create(OpKind::TopUp { aaa: p(2) }, PayPath::Deposit, 0, p(9), 2);
        assert_eq!(a.id, 0);
        assert_eq!(b.id, 1);
        assert_eq!(a.state, OpState::Pending);
        assert_eq!(get(0), Some(a));
        assert_eq!(get(1), Some(b));
        assert_eq!(get(99), None);
    }

    #[test]
    fn t5_1_advance_follows_the_spawn_state_machine_and_rejects_bad_edges() {
        let op = create(
            OpKind::Spawn {
                owner: p(1),
                name: "Rover".into(),
                avatar_seed: 1,
            },
            PayPath::Wallet {
                payer: Account {
                    owner: p(1),
                    subaccount: None,
                },
            },
            100,
            p(1),
            10,
        );
        assert!(matches!(
            advance(op.id, OpState::Registered, 11),
            Err(ApiError::Conflict(_))
        ));
        let op = advance(op.id, OpState::Pulled { block: 5 }, 11).unwrap();
        assert_eq!(op.attempts, 1);
        let op = advance(
            op.id,
            OpState::Notified {
                canister_or_cycles: NotifiedInfo::Canister(p(2)),
            },
            12,
        )
        .unwrap();
        let op = advance(op.id, OpState::Registered, 13).unwrap();
        let op = advance(op.id, OpState::Done, 14).unwrap();
        assert!(op.state.is_terminal());
        assert!(matches!(
            advance(op.id, OpState::Pending, 15),
            Err(ApiError::Conflict(_))
        ));
    }

    #[test]
    fn t5_1_advance_treasury_path_and_failed_from_any_non_terminal_state() {
        let op = create(
            OpKind::FuelPack {
                aaa: p(3),
                source: FuelSource::Btc { sats: 50_000 },
                usd_cents: 1_200,
                packs: 0,
            },
            PayPath::Treasury,
            0,
            p(3),
            1,
        );
        let op = advance(op.id, OpState::Credited { usd_cents: 1_200 }, 2).unwrap();
        let op = advance(op.id, OpState::TreasuryPaid { block: 7 }, 3).unwrap();
        let failed = advance(
            op.id,
            OpState::Failed {
                reason: "boom".into(),
            },
            4,
        )
        .unwrap();
        assert!(matches!(failed.state, OpState::Failed { .. }));
        assert!(matches!(
            advance(op.id, OpState::Done, 5),
            Err(ApiError::Conflict(_))
        ));
    }

    #[test]
    fn t5_1_unknown_op_is_not_found() {
        assert_eq!(advance(12345, OpState::Done, 1), Err(ApiError::NotFound));
    }

    #[test]
    fn t5_3_spawn_fields_extracts_from_spawn_kind_only() {
        let spawn = create(
            OpKind::Spawn {
                owner: p(5),
                name: "Rover".into(),
                avatar_seed: 7,
            },
            PayPath::Deposit,
            0,
            p(5),
            1,
        );
        assert_eq!(spawn.spawn_fields(), Ok((p(5), "Rover".to_string(), 7)));
        let topup = create(OpKind::TopUp { aaa: p(6) }, PayPath::Deposit, 0, p(6), 2);
        assert!(matches!(topup.spawn_fields(), Err(ApiError::Internal(_))));
    }

    #[test]
    fn t5_4_topup_fields_extracts_from_topup_kind_only() {
        let topup = create(OpKind::TopUp { aaa: p(7) }, PayPath::Deposit, 0, p(7), 1);
        assert_eq!(topup.topup_fields(), Ok(p(7)));
        let spawn = create(
            OpKind::Spawn {
                owner: p(8),
                name: "Rover".into(),
                avatar_seed: 1,
            },
            PayPath::Deposit,
            0,
            p(8),
            2,
        );
        assert!(matches!(spawn.topup_fields(), Err(ApiError::Internal(_))));
    }

    #[test]
    fn t5_5_auto_topup_fields_extracts_from_auto_topup_kind_only() {
        let auto = create(
            OpKind::AutoTopUp { aaa: p(11) },
            PayPath::Deposit,
            0,
            p(11),
            1,
        );
        assert_eq!(auto.auto_topup_fields(), Ok(p(11)));
        let topup = create(OpKind::TopUp { aaa: p(12) }, PayPath::Deposit, 0, p(12), 2);
        assert!(matches!(
            topup.auto_topup_fields(),
            Err(ApiError::Internal(_))
        ));
    }

    #[test]
    fn t5_15_list_filtered_by_kind_state_and_since_pages_correctly() {
        create(OpKind::TopUp { aaa: p(1) }, PayPath::Deposit, 100, p(9), 10);
        let spawn = create(
            OpKind::Spawn {
                owner: p(2),
                name: "Rover".into(),
                avatar_seed: 1,
            },
            PayPath::Deposit,
            200,
            p(9),
            20,
        );
        create(OpKind::TopUp { aaa: p(3) }, PayPath::Deposit, 300, p(9), 30);
        advance(spawn.id, OpState::Pulled { block: 1 }, 21).unwrap();

        let by_kind = list_filtered(
            &OpFilter {
                kind: Some(OpKind::TopUp { aaa: p(0) }),
                ..Default::default()
            },
            None,
            100,
        );
        assert_eq!(by_kind.items.len(), 2);
        assert!(by_kind
            .items
            .iter()
            .all(|op| matches!(op.kind, OpKind::TopUp { .. })));
        assert_eq!(by_kind.next_cursor, None);

        let by_state = list_filtered(
            &OpFilter {
                state: Some(OpState::Pulled { block: 0 }),
                ..Default::default()
            },
            None,
            100,
        );
        assert_eq!(by_state.items, vec![get(spawn.id).unwrap()]);

        let by_since = list_filtered(
            &OpFilter {
                since: Some(20),
                ..Default::default()
            },
            None,
            100,
        );
        assert_eq!(by_since.items.len(), 2);

        let page1 = list_filtered(&OpFilter::default(), None, 2);
        assert_eq!(page1.items.len(), 2);
        assert_eq!(page1.next_cursor, Some(2));
        let page2 = list_filtered(&OpFilter::default(), page1.next_cursor, 2);
        assert_eq!(page2.items.len(), 1);
        assert_eq!(page2.next_cursor, None);
    }

    #[test]
    fn t5_15_stats_since_buckets_count_and_spend_by_kind_within_window() {
        const WINDOW_NS: u64 = 24 * 3_600 * 1_000_000_000;
        let now: u64 = WINDOW_NS * 10;
        create(
            OpKind::TopUp { aaa: p(1) },
            PayPath::Deposit,
            1_000,
            p(9),
            now - 100,
        );
        create(
            OpKind::TopUp { aaa: p(2) },
            PayPath::Deposit,
            2_000,
            p(9),
            now - 200,
        );
        create(
            OpKind::Spawn {
                owner: p(3),
                name: "Rover".into(),
                avatar_seed: 1,
            },
            PayPath::Deposit,
            5_000,
            p(9),
            now - WINDOW_NS - 1,
        );
        let stats = stats_since(now, 24 * 3_600);
        let topup = stats.iter().find(|(k, ..)| k == "topup").unwrap();
        assert_eq!(topup.1, 2);
        assert_eq!(topup.2, 3_000);
        let spawn = stats.iter().find(|(k, ..)| k == "spawn").unwrap();
        assert_eq!(spawn.1, 0);
        assert_eq!(spawn.2, 0);
    }

    #[test]
    fn t5_15_failed_and_stuck_counts() {
        let a = create(OpKind::TopUp { aaa: p(1) }, PayPath::Deposit, 0, p(9), 0);
        advance(
            a.id,
            OpState::Failed {
                reason: "boom".into(),
            },
            1,
        )
        .unwrap();
        let b = create(OpKind::TopUp { aaa: p(2) }, PayPath::Deposit, 0, p(9), 0);
        advance(b.id, OpState::Pulled { block: 1 }, 1_000).unwrap();

        assert_eq!(failed_count(), 1);
        assert_eq!(stuck_count(1_000 + 2 * 3_600 * 1_000_000_000, 3_600), 1);
        assert_eq!(stuck_count(1_000, 3_600), 0);
    }

    #[test]
    fn t5_1_list_by_created_pages() {
        for i in 0..3u8 {
            create(
                OpKind::TopUp { aaa: p(i) },
                PayPath::Deposit,
                0,
                p(9),
                i as u64,
            );
        }
        assert_eq!(list_by_created(None, 100).len(), 3);
        assert_eq!(list_by_created(Some(1), 100).len(), 2);
        assert_eq!(list_by_created(Some(0), 1).len(), 1);
    }

    #[test]
    fn t5_7_fix_pull_pins_the_first_amount_and_refuses_after_pending() {
        let op = create(OpKind::TopUp { aaa: p(1) }, PayPath::Deposit, 0, p(9), 1);
        assert_eq!(fix_pull(op.id, 500), Ok(500));
        assert_eq!(fix_pull(op.id, 900), Ok(500));
        assert_eq!(get(op.id).unwrap().pull_e8s, Some(500));
        let other = create(OpKind::TopUp { aaa: p(2) }, PayPath::Deposit, 0, p(9), 1);
        advance(other.id, OpState::Pulled { block: 1 }, 2).unwrap();
        assert!(matches!(fix_pull(other.id, 1), Err(ApiError::Conflict(_))));
        assert_eq!(fix_pull(777, 1), Err(ApiError::NotFound));
    }

    #[test]
    fn t5_7_resumable_from_reaches_ops_beyond_the_first_page_and_advances_the_watermark() {
        for i in 0..60u8 {
            let op = create(OpKind::TopUp { aaa: p(i) }, PayPath::Deposit, 0, p(9), 1);
            if i < 55 {
                advance(op.id, OpState::Failed { reason: "x".into() }, 2).unwrap();
            }
        }
        let (ops, wm) = resumable_from(0, 50);
        assert_eq!(
            ops.iter().map(|o| o.id).collect::<Vec<_>>(),
            vec![55, 56, 57, 58, 59]
        );
        assert_eq!(wm, 55);
        let (ops, wm) = resumable_from(wm, 2);
        assert_eq!(ops.len(), 2);
        assert_eq!(wm, 55);
        for id in 55..60 {
            advance(id, OpState::Failed { reason: "x".into() }, 3).unwrap();
        }
        let (ops, wm) = resumable_from(55, 50);
        assert!(ops.is_empty());
        assert_eq!(wm, 60);
        assert_eq!(resumable_from(60, 50), (vec![], 60));
    }

    #[test]
    fn t5_7_ops_stored_before_pull_e8s_existed_still_decode() {
        #[derive(CandidType)]
        struct OpV1 {
            v: u8,
            id: u64,
            kind: OpKind,
            path: PayPath,
            amount_e8s: u64,
            created_at: u64,
            created_by: Principal,
            state: OpState,
            updated_at: u64,
            attempts: u8,
        }
        let bytes = candid::encode_one(OpV1 {
            v: 1,
            id: 4,
            kind: OpKind::TopUp { aaa: p(1) },
            path: PayPath::Deposit,
            amount_e8s: 5,
            created_at: 6,
            created_by: p(2),
            state: OpState::Pending,
            updated_at: 6,
            attempts: 0,
        })
        .unwrap();
        let op: Op = candid::decode_one(&bytes).unwrap();
        assert_eq!(op.id, 4);
        assert_eq!(op.pull_e8s, None);
    }

    #[test]
    fn t5_19_list_for_aaa_pages_newest_first_and_isolates_other_aaas() {
        let aaa = p(1);
        let other = p(2);
        for i in 0..3u64 {
            let op = create(OpKind::TopUp { aaa }, PayPath::Deposit, 0, p(9), i);
            index_aaa(aaa, op.id);
        }
        let op = create(OpKind::TopUp { aaa: other }, PayPath::Deposit, 0, p(9), 10);
        index_aaa(other, op.id);

        let page1 = list_for_aaa(aaa, None, 2);
        assert_eq!(page1.items.iter().map(|o| o.id).collect::<Vec<_>>(), [2, 1]);
        assert!(page1.next_cursor.is_some());
        let page2 = list_for_aaa(aaa, page1.next_cursor, 2);
        assert_eq!(page2.items.iter().map(|o| o.id).collect::<Vec<_>>(), [0]);
        assert_eq!(page2.next_cursor, None);

        let other_page = list_for_aaa(other, None, 100);
        assert_eq!(other_page.items.len(), 1);
        assert!(other_page
            .items
            .iter()
            .all(|o| o.kind == OpKind::TopUp { aaa: other }));
    }

    #[test]
    fn t5_19_list_for_owner_pages_newest_first_and_isolates_other_owners() {
        let owner = p(3);
        let other = p(4);
        for i in 0..2u64 {
            let op = create(
                OpKind::Spawn {
                    owner,
                    name: "Rover".into(),
                    avatar_seed: i,
                },
                PayPath::Deposit,
                0,
                owner,
                i,
            );
            index_owner(owner, op.id);
        }
        let op = create(
            OpKind::Spawn {
                owner: other,
                name: "Other".into(),
                avatar_seed: 0,
            },
            PayPath::Deposit,
            0,
            other,
            10,
        );
        index_owner(other, op.id);

        let page = list_for_owner(owner, None, 100);
        assert_eq!(page.items.len(), 2);
        assert!(page
            .items
            .iter()
            .all(|o| matches!(&o.kind, OpKind::Spawn { owner: w, .. } if *w == owner)));

        let other_page = list_for_owner(other, None, 100);
        assert_eq!(other_page.items.len(), 1);
    }

    #[test]
    fn t5_19_backfill_indexes_populates_from_existing_journal() {
        let aaa = p(5);
        let owner = p(6);
        let topup = create(OpKind::TopUp { aaa }, PayPath::Deposit, 0, owner, 1);
        let spawn = create(
            OpKind::Spawn {
                owner,
                name: "Rover".into(),
                avatar_seed: 1,
            },
            PayPath::Deposit,
            0,
            owner,
            2,
        );

        backfill_indexes();

        let aaa_page = list_for_aaa(aaa, None, 100);
        assert_eq!(
            aaa_page.items.iter().map(|o| o.id).collect::<Vec<_>>(),
            [topup.id]
        );
        let owner_page = list_for_owner(owner, None, 100);
        let mut ids: Vec<u64> = owner_page.items.iter().map(|o| o.id).collect();
        ids.sort_unstable();
        assert_eq!(ids, [topup.id, spawn.id]);
    }

    #[test]
    fn t5_19_backfill_indexes_runs_only_once() {
        let aaa = p(7);
        let first = create(OpKind::TopUp { aaa }, PayPath::Deposit, 0, p(8), 1);
        backfill_indexes_once();
        assert_eq!(crate::config::get().indexes_backfilled, Some(true));
        create(OpKind::TopUp { aaa }, PayPath::Deposit, 0, p(8), 2);
        backfill_indexes_once();
        let ids: Vec<u64> = list_for_aaa(aaa, None, 100)
            .items
            .iter()
            .map(|o| o.id)
            .collect();
        assert_eq!(ids, [first.id]);
    }
}
