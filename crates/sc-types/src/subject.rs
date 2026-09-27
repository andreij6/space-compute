use candid::CandidType;
use serde::{Deserialize, Serialize};

pub const FIELDS: [&str; 6] = [
    "ceers",
    "jades-gds",
    "jades-gdn",
    "primer-uds",
    "primer-cosmos",
    "abell2744",
];

#[derive(CandidType, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct SubjectRef {
    pub subject_id: u32,
    pub field: String,
    pub ra_deg: f64,
    pub dec_deg: f64,
    pub image_url: String,
    #[serde(with = "serde_bytes")]
    pub image_sha256: Vec<u8>,
    pub dossier_url: String,
    #[serde(with = "serde_bytes")]
    pub dossier_sha256: Vec<u8>,
    pub data_version: u16,
}
