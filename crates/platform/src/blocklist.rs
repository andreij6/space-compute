use sc_types::ApiError;

const BLOCKED: &[&str] = &[
    "admin",
    "administrator",
    "moderator",
    "official",
    "support",
    "staff",
    "spacecompute",
    "platform",
    "system",
    "root",
    "nazi",
    "hitler",
    "rape",
    "fuck",
    "nigger",
    "faggot",
    "cunt",
];

fn normalize(name: &str) -> String {
    name.chars()
        .filter_map(|c| {
            let mapped = match c {
                '0' => 'o',
                '1' | '!' | '|' => 'i',
                '3' => 'e',
                '4' | '@' => 'a',
                '5' | '$' => 's',
                '7' => 't',
                other => other,
            };
            let lower = mapped.to_ascii_lowercase();
            lower.is_ascii_alphanumeric().then_some(lower)
        })
        .collect()
}

pub fn check(name: &str) -> Result<(), ApiError> {
    let normalized = normalize(name);
    if BLOCKED.iter().any(|blocked| normalized.contains(blocked)) {
        return Err(ApiError::invalid("name is not allowed"));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn t4_11_blocklist_rejects_exact_and_leetspeak_and_case_variants() {
        assert!(check("Surveyor-01").is_ok());
        assert!(check("Admin").is_err());
        assert!(check("4dm1n").is_err());
        assert!(check("Sp4ceComput3").is_err());
        assert!(check("ADMINISTRATOR").is_err());
        assert!(check("SysAdmin-42").is_err());
        assert!(check("Rootbeer").is_err());
    }
}
