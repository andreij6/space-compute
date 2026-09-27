pub const BURN_SAMPLE_INTERVAL_SECS: u64 = 6 * 3_600;
pub const BURN_EMA_ALPHA: f64 = 0.2;

pub fn tick_burn_sample(prev_balance: Option<u128>, current_balance: u128, prev_ema: u128) -> u128 {
    let Some(prev) = prev_balance else {
        return prev_ema;
    };
    if current_balance >= prev {
        return prev_ema;
    }
    let sample = prev - current_balance;
    let secs_per_day = 86_400u128;
    let daily = sample.saturating_mul(secs_per_day) / BURN_SAMPLE_INTERVAL_SECS as u128;
    let next = BURN_EMA_ALPHA * daily as f64 + (1.0 - BURN_EMA_ALPHA) * prev_ema as f64;
    if next.is_finite() && next >= 0.0 {
        next.round() as u128
    } else {
        prev_ema
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn t3_4_first_sample_has_no_prior_balance() {
        assert_eq!(tick_burn_sample(None, 1_000, 0), 0);
        assert_eq!(tick_burn_sample(None, 1_000, 42), 42);
    }

    #[test]
    fn t3_4_top_up_is_excluded_from_the_average() {
        assert_eq!(tick_burn_sample(Some(1_000), 5_000, 42), 42);
        assert_eq!(tick_burn_sample(Some(1_000), 1_000, 42), 42);
    }

    #[test]
    fn t3_4_a_single_sample_moves_the_ema_toward_the_daily_rate() {
        let ema = tick_burn_sample(Some(1_000_000_000), 940_000_000, 0);
        assert_eq!(ema, 48_000_000);
    }

    #[test]
    fn t3_4_steady_burn_converges_toward_the_daily_rate() {
        let mut ema = 0u128;
        let mut balance = 10_000_000_000u128;
        for _ in 0..40 {
            let next_balance = balance - 60_000_000;
            ema = tick_burn_sample(Some(balance), next_balance, ema);
            balance = next_balance;
        }
        let expected = 240_000_000u128;
        let diff = expected.abs_diff(ema);
        assert!(
            diff < expected / 50,
            "ema {ema} should converge near {expected}"
        );
    }
}
