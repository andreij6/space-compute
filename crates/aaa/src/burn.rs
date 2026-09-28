pub const BURN_SAMPLE_INTERVAL_SECS: u64 = 6 * 3_600;
pub const BURN_EMA_ALPHA: f64 = 0.2;
const NANOS_PER_SEC: u64 = 1_000_000_000;

pub fn tick_burn_sample(
    prev_balance: Option<u128>,
    prev_at: Option<u64>,
    current_balance: u128,
    now: u64,
    prev_ema: u128,
) -> u128 {
    let Some(prev) = prev_balance else {
        return prev_ema;
    };
    if current_balance >= prev {
        return prev_ema;
    }
    let elapsed_secs = match prev_at {
        Some(at) => now.saturating_sub(at) / NANOS_PER_SEC,
        None => BURN_SAMPLE_INTERVAL_SECS,
    };
    if elapsed_secs == 0 {
        return prev_ema;
    }
    let intervals = elapsed_secs as f64 / BURN_SAMPLE_INTERVAL_SECS as f64;
    let alpha = 1.0 - (1.0 - BURN_EMA_ALPHA).powf(intervals);
    let daily = (prev - current_balance) as f64 * 86_400.0 / elapsed_secs as f64;
    let next = alpha * daily + (1.0 - alpha) * prev_ema as f64;
    if next.is_finite() && next >= 0.0 {
        next.round() as u128
    } else {
        prev_ema
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const H: u64 = 3_600 * NANOS_PER_SEC;

    #[test]
    fn t3_4_first_sample_has_no_prior_balance() {
        assert_eq!(tick_burn_sample(None, None, 1_000, 6 * H, 0), 0);
        assert_eq!(tick_burn_sample(None, None, 1_000, 6 * H, 42), 42);
    }

    #[test]
    fn t3_4_top_up_is_excluded_from_the_average() {
        assert_eq!(tick_burn_sample(Some(1_000), Some(0), 5_000, 6 * H, 42), 42);
        assert_eq!(tick_burn_sample(Some(1_000), Some(0), 1_000, 6 * H, 42), 42);
    }

    #[test]
    fn t3_4_a_single_sample_moves_the_ema_toward_the_daily_rate() {
        let ema = tick_burn_sample(Some(1_000_000_000), Some(0), 940_000_000, 6 * H, 0);
        assert_eq!(ema, 48_000_000);
    }

    #[test]
    fn t3_4_legacy_sample_without_timestamp_assumes_one_interval() {
        let ema = tick_burn_sample(Some(1_000_000_000), None, 940_000_000, 6 * H, 0);
        assert_eq!(ema, 48_000_000);
    }

    #[test]
    fn t3_4_ema_scales_by_real_elapsed_time() {
        let late = tick_burn_sample(Some(1_000_000_000), Some(0), 880_000_000, 12 * H, 0);
        assert_eq!(late, 86_400_000);
        let same_instant =
            tick_burn_sample(Some(1_000_000_000), Some(5 * H), 900_000_000, 5 * H, 7);
        assert_eq!(same_instant, 7);
    }

    #[test]
    fn t3_4_steady_burn_converges_toward_the_daily_rate() {
        let mut ema = 0u128;
        let mut balance = 10_000_000_000u128;
        let mut at = 0u64;
        for i in 0..40u64 {
            let step = if i % 2 == 0 { 6 * H } else { 9 * H };
            let next_balance = balance - 10_000_000 * (step / H) as u128;
            ema = tick_burn_sample(Some(balance), Some(at), next_balance, at + step, ema);
            balance = next_balance;
            at += step;
        }
        let expected = 240_000_000u128;
        let diff = expected.abs_diff(ema);
        assert!(
            diff < expected / 50,
            "ema {ema} should converge near {expected}"
        );
    }
}
