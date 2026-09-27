use candid::{CandidType, Nat};
use serde::{Deserialize, Serialize};

#[derive(CandidType, Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub enum ApiError {
    Unauthorized,
    NotRegistered,
    Suspended,
    InsufficientFee { required: Nat },
    RateLimited { retry_after_secs: u32 },
    LeaseExpired,
    LeaseNotFound,
    InvalidInput(String),
    NotFound,
    NotEligible(String),
    Conflict(String),
    Internal(String),
    FeatureDisabled,
}

impl ApiError {
    pub fn invalid(msg: impl Into<String>) -> Self {
        ApiError::InvalidInput(msg.into())
    }
}

impl std::fmt::Display for ApiError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ApiError::Unauthorized => write!(f, "unauthorized"),
            ApiError::NotRegistered => write!(f, "not registered"),
            ApiError::Suspended => write!(f, "suspended"),
            ApiError::InsufficientFee { required } => {
                write!(f, "insufficient fee: {required} cycles required")
            }
            ApiError::RateLimited { retry_after_secs } => {
                write!(f, "rate limited: retry in {retry_after_secs}s")
            }
            ApiError::LeaseExpired => write!(f, "lease expired"),
            ApiError::LeaseNotFound => write!(f, "lease not found"),
            ApiError::InvalidInput(m) => write!(f, "invalid input: {m}"),
            ApiError::NotFound => write!(f, "not found"),
            ApiError::NotEligible(m) => write!(f, "not eligible: {m}"),
            ApiError::Conflict(m) => write!(f, "conflict: {m}"),
            ApiError::Internal(m) => write!(f, "internal error: {m}"),
            ApiError::FeatureDisabled => write!(f, "feature disabled"),
        }
    }
}

impl std::error::Error for ApiError {}
