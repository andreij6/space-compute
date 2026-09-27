use std::cell::RefCell;
use std::collections::HashSet;

use candid::Principal;
use sc_types::ApiError;

thread_local! {
    static IN_FLIGHT: RefCell<HashSet<Vec<u8>>> = RefCell::new(HashSet::new());
}

pub fn key(purpose: &str, beneficiary: Principal) -> Vec<u8> {
    let mut k = purpose.as_bytes().to_vec();
    k.push(0);
    k.extend_from_slice(beneficiary.as_slice());
    k
}

pub struct CallerGuard {
    key: Vec<u8>,
}

impl CallerGuard {
    pub fn acquire(key: Vec<u8>) -> Result<Self, ApiError> {
        let acquired = IN_FLIGHT.with_borrow_mut(|s| s.insert(key.clone()));
        if !acquired {
            return Err(ApiError::Conflict(
                "an operation for this purpose/beneficiary is already in flight".into(),
            ));
        }
        Ok(CallerGuard { key })
    }

    #[cfg(test)]
    pub fn held_count() -> usize {
        IN_FLIGHT.with_borrow(|s| s.len())
    }
}

impl Drop for CallerGuard {
    fn drop(&mut self) {
        IN_FLIGHT.with_borrow_mut(|s| {
            s.remove(&self.key);
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn p(n: u8) -> Principal {
        Principal::from_slice(&[n; 29])
    }

    #[test]
    fn t5_1_guard_rejects_same_key_while_held_and_releases_on_drop() {
        let k1 = key("topup", p(1));
        let g1 = CallerGuard::acquire(k1.clone()).unwrap();
        assert!(matches!(
            CallerGuard::acquire(k1.clone()),
            Err(ApiError::Conflict(_))
        ));
        drop(g1);
        let g2 = CallerGuard::acquire(k1).unwrap();
        drop(g2);
        assert_eq!(CallerGuard::held_count(), 0);
    }

    #[test]
    fn t5_1_guard_allows_distinct_keys_concurrently() {
        let a = CallerGuard::acquire(key("topup", p(1))).unwrap();
        let b = CallerGuard::acquire(key("topup", p(2))).unwrap();
        let c = CallerGuard::acquire(key("spawn", p(1))).unwrap();
        assert_eq!(CallerGuard::held_count(), 3);
        drop(a);
        drop(b);
        drop(c);
    }
}
