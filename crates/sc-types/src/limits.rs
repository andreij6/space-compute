use crate::{Answer, ApiError, DiscoveryFlag, FIELDS};

pub const NAME_MIN: usize = 3;
pub const NAME_MAX: usize = 32;
pub const RATIONALE_MIN: usize = 20;
pub const RATIONALE_MAX: usize = 1000;
pub const AGENT_LABEL_MAX: usize = 64;
pub const OPERATOR_LABEL_MAX: usize = 32;
pub const ANSWERS_MAX: usize = 16;
pub const CONFIDENCE_MAX: u8 = 100;
pub const ADMIN_BATCH_MAX: usize = 500;
pub const ADMIN_BATCH_BYTES_MAX: usize = 1_500_000;
pub const ARG_BYTES_MAX: usize = 256 * 1024;
pub const WASM_CHUNK_BYTES_MAX: usize = 1024 * 1024;
pub const SHA256_LEN: usize = 32;
pub const PAGE_LIMIT_MAX: u32 = 100;

type Check = Result<(), ApiError>;

pub fn aaa_name(raw: &str) -> Result<String, ApiError> {
    let name = raw.trim();
    let len = name.chars().count();
    if !(NAME_MIN..=NAME_MAX).contains(&len) {
        return Err(ApiError::invalid(format!(
            "name must be {NAME_MIN}-{NAME_MAX} characters"
        )));
    }
    if !name
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || matches!(c, ' ' | '_' | '.' | '-'))
    {
        return Err(ApiError::invalid(
            "name may only contain letters, digits, space, _ . -",
        ));
    }
    Ok(name.to_string())
}

pub fn name_key(name: &str) -> String {
    name.trim().to_ascii_lowercase()
}

pub fn rationale(text: &str) -> Check {
    let len = text.chars().count();
    if !(RATIONALE_MIN..=RATIONALE_MAX).contains(&len) {
        return Err(ApiError::invalid(format!(
            "rationale must be {RATIONALE_MIN}-{RATIONALE_MAX} characters"
        )));
    }
    if text.chars().any(|c| c.is_control() && c != '\n') {
        return Err(ApiError::invalid("rationale contains control characters"));
    }
    Ok(())
}

pub fn agent_label(label: &Option<String>) -> Check {
    match label {
        None => Ok(()),
        Some(l) if l.chars().count() > AGENT_LABEL_MAX => Err(ApiError::invalid(format!(
            "agent_label must be at most {AGENT_LABEL_MAX} characters"
        ))),
        Some(l) if l.chars().any(char::is_control) => {
            Err(ApiError::invalid("agent_label must be printable"))
        }
        Some(_) => Ok(()),
    }
}

pub fn operator_label(label: &str) -> Check {
    if label.chars().count() > OPERATOR_LABEL_MAX || label.chars().any(char::is_control) {
        return Err(ApiError::invalid(format!(
            "operator label must be at most {OPERATOR_LABEL_MAX} printable characters"
        )));
    }
    Ok(())
}

pub fn answers(answers: &[Answer]) -> Check {
    if answers.is_empty() || answers.len() > ANSWERS_MAX {
        return Err(ApiError::invalid(format!(
            "answers must contain 1-{ANSWERS_MAX} entries"
        )));
    }
    Ok(())
}

pub fn sha256(bytes: &[u8]) -> Check {
    if bytes.len() != SHA256_LEN {
        return Err(ApiError::invalid("expected a 32-byte sha256"));
    }
    Ok(())
}

pub fn field(field: &str) -> Check {
    if !FIELDS.contains(&field) {
        return Err(ApiError::invalid(format!("unknown field {field}")));
    }
    Ok(())
}

pub fn discovery_flag(flag: &DiscoveryFlag) -> Check {
    if flag.confidence > CONFIDENCE_MAX {
        return Err(ApiError::invalid("confidence must be 0-100"));
    }
    if flag.category.is_empty() || flag.category.len() > 64 {
        return Err(ApiError::invalid("category must be 1-64 characters"));
    }
    rationale(&flag.rationale)
}

pub fn page_limit(limit: u32) -> u32 {
    limit.clamp(1, PAGE_LIMIT_MAX)
}
