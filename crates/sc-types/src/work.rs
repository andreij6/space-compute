use crate::{Answer, Protocol, SubjectRef};
use candid::{CandidType, Principal};
use serde::{Deserialize, Serialize};

#[derive(CandidType, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Task {
    pub task_id: u64,
    pub subject: SubjectRef,
    pub protocol: Protocol,
    pub lease_expires_at_ns: u64,
}

#[derive(CandidType, Serialize, Deserialize, Clone, Copy, Debug, PartialEq)]
pub struct ClaimPosition {
    pub ra_deg: f64,
    pub dec_deg: f64,
}

#[derive(CandidType, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct DiscoveryFlag {
    pub category: String,
    pub rationale: String,
    pub confidence: u8,
    pub claim_position: Option<ClaimPosition>,
}

#[derive(CandidType, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct ClassificationSubmission {
    pub task_id: u64,
    pub answers: Vec<Answer>,
    #[serde(with = "serde_bytes")]
    pub observed_image_sha256: Vec<u8>,
    pub discovery: Option<DiscoveryFlag>,
    pub agent_label: Option<String>,
    pub submitted_by: Principal,
}

#[derive(CandidType, Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub enum ClaimOutcome {
    New,
    Corroborates(String),
    ClosedRecentlyRejected(String),
}

#[derive(CandidType, Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct ClassificationReceipt {
    pub classification_id: u64,
    pub discovery_id: Option<String>,
    pub xp_awarded: u32,
    pub duplicate: bool,
    pub claim: Option<ClaimOutcome>,
}

#[derive(CandidType, Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub enum Vote {
    Agree,
    Disagree,
}

#[derive(CandidType, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct ReviewAssignment {
    pub assignment_id: u64,
    pub subject: SubjectRef,
    pub protocol_version: u16,
    pub category: String,
    pub rationale: String,
    pub lease_expires_at_ns: u64,
}

#[derive(CandidType, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct ReviewSubmission {
    pub assignment_id: u64,
    pub vote: Vote,
    pub rationale: String,
    #[serde(with = "serde_bytes")]
    pub observed_image_sha256: Vec<u8>,
    pub agent_label: Option<String>,
    pub submitted_by: Principal,
}

#[derive(CandidType, Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct ReviewReceipt {
    pub review_id: u64,
    pub xp_awarded: u32,
    pub duplicate: bool,
}
