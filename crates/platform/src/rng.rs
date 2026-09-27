use std::cell::{Cell, RefCell};

use rand_chacha::rand_core::{RngCore, SeedableRng};
use rand_chacha::ChaCha20Rng;

thread_local! {
    static RNG: RefCell<Option<ChaCha20Rng>> = const { RefCell::new(None) };
    static SEEDED_AT: Cell<Option<u64>> = const { Cell::new(None) };
}

pub fn seed(bytes: [u8; 32], now: u64) {
    RNG.set(Some(ChaCha20Rng::from_seed(bytes)));
    SEEDED_AT.set(Some(now));
}

pub fn seeded_at() -> Option<u64> {
    SEEDED_AT.get()
}

#[allow(dead_code)]
pub fn next_u64() -> Option<u64> {
    RNG.with_borrow_mut(|r| r.as_mut().map(|r| r.next_u64()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn t2_1_rng_is_deterministic_per_seed_and_empty_until_seeded() {
        assert_eq!(next_u64(), None);
        seed([7; 32], 1);
        let a: Vec<_> = (0..4).map(|_| next_u64().unwrap()).collect();
        seed([7; 32], 2);
        let b: Vec<_> = (0..4).map(|_| next_u64().unwrap()).collect();
        assert_eq!(a, b);
        seed([8; 32], 3);
        assert_ne!(next_u64().unwrap(), a[0]);
        assert_eq!(seeded_at(), Some(3));
    }
}
