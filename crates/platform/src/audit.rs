use std::cell::RefCell;

use candid::{CandidType, Principal};
use ic_stable_structures::StableLog;
use serde::Deserialize;
use sha2::{Digest, Sha256};

use crate::memory::{self, Memory};

#[derive(CandidType, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct AuditEntry {
    pub v: u8,
    pub at: u64,
    pub admin: Principal,
    pub method: String,
    pub args_digest: Vec<u8>,
    pub summary: String,
}

crate::candid_storable!(AuditEntry);

thread_local! {
    static LOG: RefCell<StableLog<AuditEntry, Memory, Memory>> = RefCell::new(StableLog::init(
        memory::get(memory::AUDIT_INDEX),
        memory::get(memory::AUDIT_DATA),
    ));
}

pub fn record<A: CandidType>(at: u64, admin: Principal, method: &str, args: &A, summary: String) {
    let entry = AuditEntry {
        v: 1,
        at,
        admin,
        method: method.into(),
        args_digest: Sha256::digest(candid::encode_one(args).unwrap_or_default()).to_vec(),
        summary,
    };
    LOG.with_borrow(|log| log.append(&entry))
        .expect("audit log append");
}

pub fn len() -> u64 {
    LOG.with_borrow(|log| log.len())
}

pub fn page(cursor: Option<u64>, limit: u32) -> Vec<AuditEntry> {
    let limit = sc_types::limits::page_limit(limit) as u64;
    let start = cursor.unwrap_or(0);
    LOG.with_borrow(|log| {
        (start..log.len().min(start.saturating_add(limit)))
            .filter_map(|i| log.get(i))
            .collect()
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn t2_1_audit_log_appends_digests_and_pages() {
        let admin = Principal::from_slice(&[1; 29]);
        for i in 0..5u64 {
            record(i, admin, "admin_pause", &i, format!("entry {i}"));
        }
        assert_eq!(len(), 5);
        let all = page(None, 100);
        assert_eq!(all.len(), 5);
        assert!(all.iter().all(|e| e.args_digest.len() == 32));
        assert_ne!(all[0].args_digest, all[1].args_digest);
        let tail = page(Some(3), 100);
        assert_eq!(tail.iter().map(|e| e.at).collect::<Vec<_>>(), [3, 4]);
        assert_eq!(page(Some(1), 2).len(), 2);
        assert!(page(Some(99), 10).is_empty());
    }
}
