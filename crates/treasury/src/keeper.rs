pub const E8S_PER_ICP: u128 = 100_000_000;
pub const LEDGER_FEE_E8S: u64 = 10_000;
const EMA_WEIGHT_PCT: u128 = 30;
const NANOS_PER_DAY: u128 = 86_400 * 1_000_000_000;

pub fn target_cycles(burn_per_day: u128, target_days: u32, min_balance: u128) -> u128 {
    burn_per_day
        .saturating_mul(target_days as u128)
        .max(min_balance)
}

pub fn cycles_needed(
    balance: u128,
    burn_per_day: u128,
    target_days: u32,
    min_balance: u128,
    min_topup: u128,
) -> Option<u128> {
    let target = target_cycles(burn_per_day, target_days, min_balance);
    (balance < target).then(|| (target - balance).max(min_topup))
}

pub fn e8s_for_cycles(cycles: u128, xdr_permyriad_per_icp: u64) -> Option<u64> {
    let cycles_per_e8 = xdr_permyriad_per_icp as u128;
    if cycles_per_e8 == 0 {
        return None;
    }
    u64::try_from(cycles.div_ceil(cycles_per_e8)).ok()
}

pub fn cycles_for_e8s(e8s: u64, xdr_permyriad_per_icp: u64) -> u128 {
    (e8s as u128).saturating_mul(xdr_permyriad_per_icp as u128)
}

pub fn affordable(icp_balance_e8s: u64, cost_e8s: u64, reserve_e8s: u64) -> bool {
    icp_balance_e8s
        .checked_sub(cost_e8s.saturating_add(LEDGER_FEE_E8S))
        .is_some_and(|left| left >= reserve_e8s)
}

pub fn update_burn(
    prev_ema: u128,
    prev_balance: u128,
    prev_at: u64,
    balance: u128,
    at: u64,
) -> u128 {
    let elapsed = at.saturating_sub(prev_at) as u128;
    if elapsed == 0 || balance > prev_balance {
        return prev_ema;
    }
    let observed = (prev_balance - balance).saturating_mul(NANOS_PER_DAY) / elapsed;
    if prev_ema == 0 {
        return observed;
    }
    (observed * EMA_WEIGHT_PCT + prev_ema * (100 - EMA_WEIGHT_PCT)) / 100
}

pub fn runway_days(balance: u128, burn_per_day: u128) -> u32 {
    if burn_per_day == 0 {
        return u32::MAX;
    }
    u32::try_from(balance / burn_per_day).unwrap_or(u32::MAX)
}

pub fn topup_subaccount(canister: &candid::Principal) -> [u8; 32] {
    let bytes = canister.as_slice();
    let mut sub = [0u8; 32];
    sub[0] = bytes.len() as u8;
    sub[1..=bytes.len()].copy_from_slice(bytes);
    sub
}

#[cfg(test)]
mod tests {
    use super::*;

    const T: u128 = 1_000_000_000_000;

    #[test]
    fn t5_17_top_up_restores_target_days_with_minimum() {
        assert_eq!(cycles_needed(10 * T, T / 10, 60, 2 * T, T / 2), None);
        assert_eq!(cycles_needed(T, T / 10, 60, 2 * T, T / 2), Some(5 * T));
        assert_eq!(cycles_needed(T, 0, 60, 2 * T, T / 2), Some(T));
        assert_eq!(cycles_needed(19 * T / 10, 0, 60, 2 * T, T / 2), Some(T / 2));
        assert_eq!(target_cycles(u128::MAX, 60, 0), u128::MAX);
    }

    #[test]
    fn t5_17_icp_cycles_conversion_rounds_up_and_rejects_zero_rate() {
        assert_eq!(e8s_for_cycles(T, 30_000), Some(33_333_334));
        assert_eq!(cycles_for_e8s(E8S_PER_ICP as u64, 30_000), 3 * T);
        assert_eq!(e8s_for_cycles(T, 0), None);
        assert_eq!(e8s_for_cycles(u128::MAX, 1), None);
    }

    #[test]
    fn t5_17_reserve_floor_is_never_crossed() {
        assert!(affordable(
            10 * E8S_PER_ICP as u64,
            E8S_PER_ICP as u64,
            5 * E8S_PER_ICP as u64
        ));
        assert!(!affordable(
            6 * E8S_PER_ICP as u64,
            E8S_PER_ICP as u64,
            5 * E8S_PER_ICP as u64
        ));
        assert!(!affordable(0, 1, 0));
    }

    #[test]
    fn t5_17_burn_ema_ignores_top_ups_and_smooths() {
        let day = 86_400_000_000_000u64;
        assert_eq!(update_burn(0, 10 * T, 0, 9 * T, day), T);
        assert_eq!(update_burn(T, 9 * T, day, 20 * T, 2 * day), T);
        assert_eq!(
            update_burn(T, 9 * T, day, 7 * T, 2 * day),
            (2 * T * 30 + T * 70) / 100
        );
        assert_eq!(update_burn(T, 9 * T, day, 8 * T, day), T);
    }

    #[test]
    fn t5_17_runway_and_cmc_subaccount() {
        assert_eq!(runway_days(60 * T, T), 60);
        assert_eq!(runway_days(T, 0), u32::MAX);
        let p = candid::Principal::from_slice(&[7; 10]);
        let sub = topup_subaccount(&p);
        assert_eq!(sub[0], 10);
        assert_eq!(&sub[1..11], &[7; 10]);
        assert!(sub[11..].iter().all(|b| *b == 0));
    }
}
