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
}

crate::candid_storable!(Op);

thread_local! {
    static OPS: RefCell<StableBTreeMap<u64, Op, Memory>> =
        RefCell::new(StableBTreeMap::init(memory::get(memory::OPS)));
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

pub fn list_by_created(cursor: Option<u64>, limit: u32) -> Vec<Op> {
    let limit = sc_types::limits::page_limit(limit) as usize;
    let start = cursor.unwrap_or(0);
    OPS.with_borrow(|m| m.range(start..).take(limit).map(|e| e.value()).collect())
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
}
