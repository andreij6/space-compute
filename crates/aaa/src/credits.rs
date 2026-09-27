use candid::{CandidType, Principal};
use serde::Deserialize;

use crate::record::CreditCopy;
use crate::repository;

#[derive(CandidType, Deserialize, Clone, Debug, PartialEq)]
pub struct CreditPage {
    pub items: Vec<CreditCopy>,
    pub next_cursor: Option<u64>,
}

#[derive(CandidType, Clone, Debug)]
pub struct ListAaaCreditsArgs {
    pub aaa: Principal,
    pub cursor: u64,
}

pub fn apply_credit_page(page: CreditPage, cursor_before: u64) -> u64 {
    for credit in page.items {
        repository::upsert_credit(credit);
    }
    page.next_cursor.unwrap_or(cursor_before)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::record::{CreditRole, Outcome};

    fn credit(id: &str) -> CreditCopy {
        CreditCopy {
            v: 1,
            public_id: id.into(),
            category: "ring".into(),
            role: CreditRole::Discoverer,
            outcome: Outcome::Confirmed,
            at: 1,
            subject_id: 1,
            citation_url: None,
        }
    }

    #[test]
    fn t3_4_apply_credit_page_upserts_and_advances_cursor() {
        let page = CreditPage {
            items: vec![credit("SC-2026-000001")],
            next_cursor: Some(7),
        };
        let next = apply_credit_page(page, 0);
        assert_eq!(next, 7);
        assert_eq!(
            repository::get_credit("SC-2026-000001").map(|c| c.public_id),
            Some("SC-2026-000001".into())
        );
    }

    #[test]
    fn t3_4_apply_credit_page_keeps_cursor_when_no_next_page() {
        let page = CreditPage {
            items: vec![],
            next_cursor: None,
        };
        let next = apply_credit_page(page, 5);
        assert_eq!(next, 5);
    }
}
