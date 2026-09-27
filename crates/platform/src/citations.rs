use std::cell::RefCell;

use candid::{CandidType, Principal};
use ic_stable_structures::StableBTreeMap;
use sc_types::{SubjectRef, Vote};
use serde::{Deserialize, Serialize};

use crate::discoveries::DiscoveryStatus;
use crate::memory::{self, Memory};

#[derive(CandidType, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Credit {
    pub aaa: Principal,
    pub aaa_name_at_time: String,
    pub owner: Principal,
    pub at: u64,
    pub cycles_contributed: u128,
}

#[derive(CandidType, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct ReviewerCredit {
    pub credit: Credit,
    pub vote: Vote,
}

#[derive(CandidType, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Citation {
    pub v: u8,
    pub public_id: String,
    pub discovery_seq: u64,
    pub subject: Option<SubjectRef>,
    pub protocol_version: u16,
    pub category: String,
    pub rationale: String,
    pub outcome: DiscoveryStatus,
    pub created_at: u64,
    pub resolved_at: u64,
    pub discoverer: Credit,
    pub corroborators: Vec<Credit>,
    pub reviewers: Vec<ReviewerCredit>,
    pub total_cycles_contributed: u128,
    pub text: String,
}

crate::candid_storable!(Citation);

thread_local! {
    static CITATIONS: RefCell<StableBTreeMap<u64, Citation, Memory>> =
        RefCell::new(StableBTreeMap::init(memory::get(memory::CITATIONS)));
}

pub fn insert(c: Citation) {
    CITATIONS.with_borrow_mut(|m| {
        if !m.contains_key(&c.discovery_seq) {
            m.insert(c.discovery_seq, c);
        }
    });
}

pub fn get(seq: u64) -> Option<Citation> {
    CITATIONS.with_borrow(|m| m.get(&seq))
}
