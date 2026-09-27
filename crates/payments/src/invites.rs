use std::cell::RefCell;

use candid::{CandidType, Principal};
use ic_stable_structures::{StableBTreeMap, StableCell};
use sc_types::ApiError;
use serde::Deserialize;
use sha2::{Digest, Sha256};

use crate::memory::{self, Memory};

const CODE_ALPHABET: &[u8] = b"ABCDEFGHJKLMNPQRSTUVWXYZ23456789";
pub const MAX_MINT_COUNT: u32 = 1_000;

#[derive(CandidType, Deserialize, Clone, Debug, PartialEq)]
pub struct Invite {
    pub sponsor_cycles: u128,
    pub expires_at: u64,
    pub used: bool,
    pub used_by: Option<Principal>,
    pub minted_at: u64,
}

crate::candid_storable!(Invite);

#[derive(CandidType, Deserialize, Clone, Debug, Default, PartialEq, Eq)]
pub struct DailyBudget {
    pub day: u64,
    pub spent_e8s: u64,
}

crate::candid_storable!(DailyBudget);

thread_local! {
    static INVITES: RefCell<StableBTreeMap<Vec<u8>, Invite, Memory>> =
        RefCell::new(StableBTreeMap::init(memory::get(memory::INVITES)));
    static SPONSORED_OWNERS: RefCell<StableBTreeMap<Principal, u64, Memory>> =
        RefCell::new(StableBTreeMap::init(memory::get(memory::SPONSORED_OWNERS)));
    static DAILY: RefCell<StableCell<DailyBudget, Memory>> =
        RefCell::new(StableCell::init(memory::get(memory::SPONSOR_DAILY), DailyBudget::default()));
}

pub fn code_hash(code: &str) -> Vec<u8> {
    Sha256::digest(code.as_bytes()).to_vec()
}

fn encode_code(bytes: &[u8; 32]) -> String {
    let chars: String = bytes
        .iter()
        .take(16)
        .map(|b| CODE_ALPHABET[(*b as usize) % CODE_ALPHABET.len()] as char)
        .collect();
    format!(
        "{}-{}-{}-{}",
        &chars[0..4],
        &chars[4..8],
        &chars[8..12],
        &chars[12..16]
    )
}

pub fn generate_codes(seed: &[u8], count: u32) -> Vec<String> {
    (0..count)
        .map(|i| {
            let mut hasher = Sha256::new();
            hasher.update(seed);
            hasher.update(i.to_le_bytes());
            let digest: [u8; 32] = hasher.finalize().into();
            encode_code(&digest)
        })
        .collect()
}

pub fn mint(
    count: u32,
    sponsor_cycles: u128,
    expires_at: u64,
    seed: &[u8],
    now: u64,
) -> Result<Vec<String>, ApiError> {
    if count == 0 || count > MAX_MINT_COUNT {
        return Err(ApiError::invalid(format!(
            "count must be 1..={MAX_MINT_COUNT}"
        )));
    }
    if expires_at <= now {
        return Err(ApiError::invalid("expires_at must be in the future"));
    }
    let codes = generate_codes(seed, count);
    INVITES.with_borrow_mut(|m| {
        for code in &codes {
            m.insert(
                code_hash(code),
                Invite {
                    sponsor_cycles,
                    expires_at,
                    used: false,
                    used_by: None,
                    minted_at: now,
                },
            );
        }
    });
    Ok(codes)
}

pub fn peek(code: &str) -> Option<Invite> {
    INVITES.with_borrow(|m| m.get(&code_hash(code)))
}

pub fn redeem(code: &str, owner: Principal, now: u64) -> Result<Invite, ApiError> {
    let hash = code_hash(code);
    INVITES.with_borrow_mut(|m| {
        let invite = m
            .get(&hash)
            .ok_or_else(|| ApiError::invalid("invite code not found"))?;
        if invite.used {
            return Err(ApiError::Conflict("invite code already used".into()));
        }
        if invite.expires_at <= now {
            return Err(ApiError::invalid("invite code expired"));
        }
        let redeemed = Invite {
            used: true,
            used_by: Some(owner),
            ..invite
        };
        m.insert(hash, redeemed.clone());
        Ok(redeemed)
    })
}

pub fn has_sponsored(owner: Principal) -> bool {
    SPONSORED_OWNERS.with_borrow(|m| m.contains_key(&owner))
}

pub fn mark_sponsored(owner: Principal, now: u64) {
    SPONSORED_OWNERS.with_borrow_mut(|m| {
        m.insert(owner, now);
    });
}

pub fn unmark_sponsored(owner: Principal) {
    SPONSORED_OWNERS.with_borrow_mut(|m| m.remove(&owner));
}

fn next_daily_budget(
    now_secs: u64,
    amount_e8s: u64,
    cap_e8s: u64,
) -> Result<DailyBudget, ApiError> {
    let day = now_secs / 86_400;
    let current = DAILY.with_borrow(|c| c.get().clone());
    let spent = if current.day == day {
        current.spent_e8s
    } else {
        0
    };
    let next = spent.saturating_add(amount_e8s);
    if next > cap_e8s {
        return Err(ApiError::invalid(
            "daily sponsor budget cap exceeded, try again tomorrow",
        ));
    }
    Ok(DailyBudget {
        day,
        spent_e8s: next,
    })
}

pub fn reserve_daily_budget(now_secs: u64, amount_e8s: u64, cap_e8s: u64) -> Result<(), ApiError> {
    let budget = next_daily_budget(now_secs, amount_e8s, cap_e8s)?;
    DAILY.with_borrow_mut(|c| c.set(budget));
    Ok(())
}

pub fn sponsor_spawn(
    code: &str,
    owner: Principal,
    now: u64,
    creation_fee_cycles: u128,
    rate_xdr_permyriad_per_icp: u64,
    daily_cap_e8s: u64,
) -> Result<u64, ApiError> {
    if has_sponsored(owner) {
        return Err(ApiError::Conflict(
            "this owner already has a sponsored AAA".into(),
        ));
    }
    let now_secs = now / 1_000_000_000;
    let invite = peek(code).ok_or_else(|| ApiError::invalid("invite code not found"))?;
    let cycles = creation_fee_cycles.saturating_add(invite.sponsor_cycles);
    let e8s = crate::quote::cycles_to_e8s(cycles, rate_xdr_permyriad_per_icp)?;
    next_daily_budget(now_secs, e8s, daily_cap_e8s)?;
    redeem(code, owner, now_secs)?;
    reserve_daily_budget(now_secs, e8s, daily_cap_e8s)?;
    mark_sponsored(owner, now);
    Ok(e8s)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn p(n: u8) -> Principal {
        Principal::from_slice(&[n; 29])
    }

    #[test]
    fn t5_16_generate_codes_are_unique_and_formatted() {
        let codes = generate_codes(b"seed-a", 20);
        assert_eq!(codes.len(), 20);
        let unique: std::collections::HashSet<_> = codes.iter().collect();
        assert_eq!(unique.len(), 20);
        for c in &codes {
            assert_eq!(c.len(), 19);
            assert_eq!(c.chars().filter(|&ch| ch == '-').count(), 3);
        }
    }

    #[test]
    fn t5_16_generate_codes_differ_by_seed() {
        let a = generate_codes(b"seed-a", 5);
        let b = generate_codes(b"seed-b", 5);
        assert_ne!(a, b);
    }

    #[test]
    fn t5_16_mint_rejects_bad_count_or_past_expiry() {
        assert!(matches!(
            mint(0, 1_000, 100, b"s", 1),
            Err(ApiError::InvalidInput(_))
        ));
        assert!(matches!(
            mint(MAX_MINT_COUNT + 1, 1_000, 100, b"s", 1),
            Err(ApiError::InvalidInput(_))
        ));
        assert!(matches!(
            mint(1, 1_000, 1, b"s", 1),
            Err(ApiError::InvalidInput(_))
        ));
    }

    #[test]
    fn t5_16_mint_returns_plaintext_codes_stores_only_hash() {
        let codes = mint(3, 500_000_000_000, 1_000, b"seed", 1).unwrap();
        assert_eq!(codes.len(), 3);
        for code in &codes {
            let invite = peek(code).expect("invite stored under its hash");
            assert_eq!(invite.sponsor_cycles, 500_000_000_000);
            assert!(!invite.used);
        }
        INVITES.with_borrow(|m| {
            for code in &codes {
                assert!(m.get(&code.as_bytes().to_vec()).is_none());
                assert_eq!(code_hash(code).len(), 32);
            }
        });
    }

    #[test]
    fn t5_16_redeem_is_single_use() {
        let codes = mint(1, 1_000, 1_000, b"seed", 1).unwrap();
        let code = &codes[0];
        let owner = p(1);
        let invite = redeem(code, owner, 5).unwrap();
        assert!(invite.used);
        assert_eq!(invite.used_by, Some(owner));
        assert!(matches!(redeem(code, owner, 6), Err(ApiError::Conflict(_))));
    }

    #[test]
    fn t5_16_redeem_rejects_unknown_code() {
        assert!(matches!(
            redeem("NOPE-NOPE-NOPE-NOPE", p(1), 1),
            Err(ApiError::InvalidInput(_))
        ));
    }

    #[test]
    fn t5_16_redeem_rejects_expired_code() {
        let codes = mint(1, 1_000, 100, b"seed", 1).unwrap();
        assert!(matches!(
            redeem(&codes[0], p(1), 100),
            Err(ApiError::InvalidInput(_))
        ));
        assert!(matches!(
            redeem(&codes[0], p(1), 101),
            Err(ApiError::InvalidInput(_))
        ));
    }

    #[test]
    fn t5_16_one_sponsored_aaa_per_owner_ever() {
        let owner = p(9);
        assert!(!has_sponsored(owner));
        mark_sponsored(owner, 10);
        assert!(has_sponsored(owner));
    }

    #[test]
    fn t5_16_daily_budget_caps_and_resets_by_day() {
        let cap = 1_000u64;
        reserve_daily_budget(0, 400, cap).unwrap();
        reserve_daily_budget(10, 400, cap).unwrap();
        assert!(reserve_daily_budget(20, 300, cap).is_err());
        reserve_daily_budget(86_400, 900, cap).unwrap();
    }

    #[test]
    fn t5_7_sponsor_spawn_does_not_burn_the_code_when_budget_or_rate_refuses() {
        let owner = Principal::from_slice(&[41; 29]);
        let now = 1_000 * 1_000_000_000u64;
        let codes = mint(2, 1_000, 10_000, b"t5_7-seed", 1_000).unwrap();
        assert!(sponsor_spawn(&codes[0], owner, now, 100, 10_000, 0).is_err());
        assert!(!peek(&codes[0]).unwrap().used);
        assert!(!has_sponsored(owner));
        assert!(sponsor_spawn(&codes[0], owner, now, 100, 0, u64::MAX).is_err());
        assert!(!peek(&codes[0]).unwrap().used);
        assert!(sponsor_spawn("NOPE-NOPE-NOPE-NOPE", owner, now, 100, 10_000, u64::MAX).is_err());
        let e8s = sponsor_spawn(&codes[0], owner, now, 100, 10_000, u64::MAX).unwrap();
        assert_eq!(e8s, 1);
        assert!(peek(&codes[0]).unwrap().used);
        assert!(has_sponsored(owner));
        assert!(matches!(
            sponsor_spawn(&codes[1], owner, now, 100, 10_000, u64::MAX),
            Err(ApiError::Conflict(_))
        ));
        unmark_sponsored(owner);
        assert!(!has_sponsored(owner));
        assert!(matches!(
            sponsor_spawn(&codes[0], owner, now, 100, 10_000, u64::MAX),
            Err(ApiError::Conflict(_))
        ));
    }
}
