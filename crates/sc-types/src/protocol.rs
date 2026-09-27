use candid::CandidType;
use serde::{Deserialize, Serialize};

#[derive(CandidType, Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct Protocol {
    pub version: u16,
    pub questions: Vec<Question>,
    pub discovery_categories: Vec<DiscoveryCategory>,
    pub guidance_md: String,
}

#[derive(CandidType, Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct Question {
    pub id: String,
    pub prompt: String,
    pub answers: Vec<AnswerOption>,
}

#[derive(CandidType, Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct AnswerOption {
    pub id: String,
    pub label: String,
    pub next: Option<String>,
}

#[derive(CandidType, Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct DiscoveryCategory {
    pub id: String,
    pub label: String,
    pub description: String,
}

#[derive(CandidType, Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct Answer {
    pub question_id: String,
    pub answer_id: String,
}
