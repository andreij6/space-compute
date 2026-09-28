use std::cell::RefCell;

use candid::{CandidType, Principal};
use ic_stable_structures::StableBTreeMap;
use serde::{Deserialize, Serialize};

use crate::memory::{self, Memory};

#[derive(CandidType, Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub enum CreditRole {
    Discoverer,
    Reviewer,
    Corroborator,
}

#[derive(CandidType, Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub enum CreditOutcome {
    Confirmed,
    Rejected,
    NeedsMoreReview,
}

#[derive(CandidType, Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct CreditCopy {
    pub v: u8,
    pub public_id: String,
    pub category: String,
    pub role: CreditRole,
    pub outcome: CreditOutcome,
    pub at: u64,
    pub subject_id: u32,
    pub citation_url: Option<String>,
}

crate::candid_storable!(CreditCopy);

#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct ListAaaCreditsArgs {
    pub aaa: Principal,
    pub cursor: u64,
}

#[derive(CandidType, Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct CreditPage {
    pub items: Vec<CreditCopy>,
    pub next_cursor: Option<u64>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct CreditKey {
    pub aaa: Principal,
    pub discovery_seq: u64,
}

impl ic_stable_structures::Storable for CreditKey {
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
        bytes.extend_from_slice(&self.discovery_seq.to_be_bytes());
        std::borrow::Cow::Owned(bytes)
    }

    fn into_bytes(self) -> Vec<u8> {
        self.to_bytes().into_owned()
    }

    fn from_bytes(bytes: std::borrow::Cow<[u8]>) -> Self {
        let len = bytes[0] as usize;
        let aaa = Principal::from_slice(&bytes[1..1 + len]);
        let discovery_seq = u64::from_be_bytes(bytes[1 + len..9 + len].try_into().unwrap());
        CreditKey { aaa, discovery_seq }
    }
}

thread_local! {
    static CREDIT_INDEX: RefCell<StableBTreeMap<CreditKey, CreditCopy, Memory>> =
        RefCell::new(StableBTreeMap::init(memory::get(memory::CREDIT_INDEX)));
}

pub fn record_credit(aaa: Principal, discovery_seq: u64, credit: CreditCopy) {
    CREDIT_INDEX.with_borrow_mut(|m| {
        m.insert(CreditKey { aaa, discovery_seq }, credit);
    });
}

pub fn list_aaa_credits(args: ListAaaCreditsArgs) -> CreditPage {
    let limit = sc_types::limits::PAGE_LIMIT_MAX as usize;
    let start = CreditKey {
        aaa: args.aaa,
        discovery_seq: args.cursor,
    };
    let end = CreditKey {
        aaa: args.aaa,
        discovery_seq: u64::MAX,
    };

    let entries: Vec<(CreditKey, CreditCopy)> = CREDIT_INDEX.with_borrow(|m| {
        m.range(start..=end)
            .take(limit + 1)
            .map(|e| (*e.key(), e.value()))
            .collect()
    });

    let (batch, next_cursor) = if entries.len() > limit {
        (&entries[..limit], Some(entries[limit].0.discovery_seq))
    } else {
        (&entries[..], None)
    };

    CreditPage {
        items: batch.iter().map(|(_, c)| c.clone()).collect(),
        next_cursor,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn credit(id: &str) -> CreditCopy {
        CreditCopy {
            v: 1,
            public_id: id.into(),
            category: "lens".into(),
            role: CreditRole::Discoverer,
            outcome: CreditOutcome::Confirmed,
            at: 1,
            subject_id: 1,
            citation_url: None,
        }
    }

    #[test]
    fn t4_3_list_aaa_credits_empty_when_nothing_recorded() {
        let aaa = Principal::from_slice(&[7, 7, 7]);
        let page = list_aaa_credits(ListAaaCreditsArgs { aaa, cursor: 0 });
        assert!(page.items.is_empty());
        assert!(page.next_cursor.is_none());
    }

    #[test]
    fn t4_3_list_aaa_credits_pages_by_discovery_seq_per_aaa() {
        let aaa = Principal::from_slice(&[8, 8, 8]);
        let other = Principal::from_slice(&[9, 9, 9]);
        for seq in 1..=3u64 {
            record_credit(aaa, seq, credit(&format!("SC-2026-{seq:06}")));
        }
        record_credit(other, 1, credit("SC-2026-999999"));

        let page = list_aaa_credits(ListAaaCreditsArgs { aaa, cursor: 0 });
        assert_eq!(page.items.len(), 3);
        assert!(page
            .items
            .iter()
            .all(|c| c.public_id.starts_with("SC-2026-0")));
        assert!(page.next_cursor.is_none());

        let page_other = list_aaa_credits(ListAaaCreditsArgs {
            aaa: other,
            cursor: 0,
        });
        assert_eq!(page_other.items.len(), 1);
    }
}
