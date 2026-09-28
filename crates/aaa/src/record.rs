use candid::{CandidType, Principal};
use serde::{Deserialize, Serialize};

use crate::operators::Operator;

#[derive(CandidType, Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub enum RecordKind {
    Classification,
    Discovery,
    Review,
    TopUp,
    Operator,
    Profile,
}

#[derive(CandidType, Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub enum Outcome {
    Confirmed,
    Rejected,
    NeedsMoreReview,
}

#[derive(CandidType, Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub enum CreditRole {
    Discoverer,
    Reviewer,
    Corroborator,
}

#[derive(CandidType, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Record {
    pub v: u8,
    pub seq: u64,
    pub at: u64,
    pub kind: RecordKind,
    pub task_or_assignment_id: Option<u64>,
    pub subject: Option<sc_types::SubjectRef>,
    pub answers: Vec<sc_types::Answer>,
    pub discovery_public_id: Option<String>,
    pub category: Option<String>,
    pub vote: Option<sc_types::Vote>,
    pub rationale: Option<String>,
    pub outcome: Option<Outcome>,
    pub xp_awarded: u32,
    pub agent_label: Option<String>,
    pub fee: u128,
    pub by: Principal,
}

#[derive(CandidType, Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct CreditCopy {
    pub v: u8,
    pub public_id: String,
    pub category: String,
    pub role: CreditRole,
    pub outcome: Outcome,
    pub at: u64,
    pub subject_id: u32,
    pub citation_url: Option<String>,
}

#[derive(CandidType, Serialize, Deserialize, Clone, Debug, PartialEq, Eq, Default)]
pub struct Stats {
    pub total_records: u64,
    pub classifications: u64,
    pub discoveries: u64,
    pub reviews: u64,
    pub topups: u64,
    pub last_activity_at: u64,
    pub last_heartbeat_at: u64,
    pub credits_cursor: u64,
    pub auto_topup_failures: u64,
    pub burn_ema_daily: u128,
    pub last_balance_sample: Option<u128>,
    pub last_sample_at: Option<u64>,
}

#[derive(CandidType, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct StoredSubject(pub sc_types::SubjectRef);

crate::candid_storable!(StoredSubject);

#[derive(CandidType, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct ListRecordsFilter {
    pub kind: Option<RecordKind>,
    pub cursor: Option<u64>,
    pub limit: u16,
}

#[derive(CandidType, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct PageRecord {
    pub items: Vec<Record>,
    pub next_cursor: Option<u64>,
}

#[derive(CandidType, Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct PageCreditCopy {
    pub items: Vec<CreditCopy>,
    pub next_cursor: Option<String>,
}

#[derive(CandidType, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Status {
    pub cycles: u128,
    pub days_of_fuel_estimate: f64,
    pub operators: Vec<(Principal, Operator)>,
    pub stats: Stats,
    pub version: String,
    pub wasm_version: String,
}

#[derive(CandidType, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct PublicStatus {
    pub name: String,
    pub version: String,
    pub owner: Principal,
    pub last_activity_at: u64,
}

crate::candid_storable!(Record);
crate::candid_storable!(CreditCopy);
crate::candid_storable!(Stats);
