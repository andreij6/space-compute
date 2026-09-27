use candid::{CandidType, Principal};
use serde::{Deserialize, Serialize};

#[derive(CandidType, Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub enum RecordKind {
    Classification,
    Discovery,
    Review,
    TopUp,
    Operator,
    Profile,
}

#[derive(CandidType, Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub enum Outcome {
    Confirmed,
    Rejected,
    NeedsMoreReview,
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
}

crate::candid_storable!(Record);
crate::candid_storable!(Stats);
