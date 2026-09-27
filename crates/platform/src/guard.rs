use std::cell::RefCell;
use std::collections::HashSet;

use candid::Principal;
use sc_types::ApiError;

thread_local! {
    static IN_FLIGHT: RefCell<HashSet<Principal>> = RefCell::new(HashSet::new());
}

pub struct CallerGuard {
    key: Principal,
}

impl CallerGuard {
    pub fn acquire(key: Principal) -> Result<Self, ApiError> {
        if !IN_FLIGHT.with_borrow_mut(|s| s.insert(key)) {
            return Err(ApiError::Conflict(format!(
                "an operation on {key} is already in flight"
            )));
        }
        Ok(CallerGuard { key })
    }
}

impl Drop for CallerGuard {
    fn drop(&mut self) {
        IN_FLIGHT.with_borrow_mut(|s| s.remove(&self.key));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn t2_9_guard_blocks_same_aaa_until_dropped() {
        let a = Principal::from_slice(&[1; 29]);
        let b = Principal::from_slice(&[2; 29]);
        let g = CallerGuard::acquire(a).unwrap();
        assert!(matches!(
            CallerGuard::acquire(a),
            Err(ApiError::Conflict(_))
        ));
        let other = CallerGuard::acquire(b).unwrap();
        drop(g);
        drop(other);
        drop(CallerGuard::acquire(a).unwrap());
    }
}
