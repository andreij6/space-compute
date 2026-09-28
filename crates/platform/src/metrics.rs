use std::cell::Cell;

thread_local! {
    static LAST_SAMPLE: Cell<Option<(u64, u128)>> = const { Cell::new(None) };
}

pub fn sample(now: u64, cycles: u128) {
    LAST_SAMPLE.set(Some((now, cycles)));
}

pub fn burn_per_hour(now: u64, current_cycles: u128) -> i128 {
    match LAST_SAMPLE.get() {
        Some((at, cycles)) if now > at => {
            let elapsed_hours = (now - at) as f64 / 3_600_000_000_000.0;
            (((cycles as i128) - (current_cycles as i128)) as f64 / elapsed_hours) as i128
        }
        _ => 0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn t4_10_burn_per_hour_uses_the_last_sample() {
        assert_eq!(burn_per_hour(1_000, 500), 0);
        sample(0, 1_000_000_000_000);
        assert_eq!(
            burn_per_hour(3_600_000_000_000, 999_000_000_000),
            1_000_000_000
        );
        assert_eq!(burn_per_hour(0, 999_000_000_000), 0);
    }
}
