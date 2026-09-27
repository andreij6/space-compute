use candid::CandidType;
use sc_types::ApiError;
use serde::Deserialize;

use crate::config::Params;
use crate::rate::RateCache;

#[derive(CandidType, Deserialize, Clone, Debug, PartialEq)]
pub struct Quote {
    pub v: u8,
    pub cycles: u128,
    pub deposit_e8s: u64,
    pub fee_e8s: u64,
    pub total_e8s: u64,
    pub rate_xdr_permyriad_per_icp: u64,
    pub rate_age_secs: u64,
}

pub fn cycles_to_e8s(cycles: u128, rate_xdr_permyriad_per_icp: u64) -> Result<u64, ApiError> {
    if rate_xdr_permyriad_per_icp == 0 {
        return Err(ApiError::Internal("xdr rate unavailable".into()));
    }
    let e8s = cycles.div_ceil(rate_xdr_permyriad_per_icp as u128);
    u64::try_from(e8s).map_err(|_| ApiError::invalid("quote overflows e8s"))
}

pub fn e8s_to_cycles(e8s: u64, rate_xdr_permyriad_per_icp: u64) -> u128 {
    e8s as u128 * rate_xdr_permyriad_per_icp as u128
}

pub fn quote_topup(
    cycles: u128,
    rate: &RateCache,
    now_secs: u64,
    fee_e8s: u64,
) -> Result<Quote, ApiError> {
    let deposit_e8s = cycles_to_e8s(cycles, rate.xdr_permyriad_per_icp)?;
    Ok(Quote {
        v: 1,
        cycles,
        deposit_e8s,
        fee_e8s,
        total_e8s: deposit_e8s.saturating_add(fee_e8s),
        rate_xdr_permyriad_per_icp: rate.xdr_permyriad_per_icp,
        rate_age_secs: rate.age_secs(now_secs),
    })
}

pub fn quote_spawn(params: &Params, rate: &RateCache, now_secs: u64) -> Result<Quote, ApiError> {
    let base_cycles = params.aaa_initial_cycles + params.spawn_creation_fee_cycles;
    let buffered = base_cycles * (10_000 + params.spawn_quote_buffer_bp as u128) / 10_000;
    quote_topup(buffered, rate, now_secs, params.icp_ledger_fee_e8s)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rate(xdr_permyriad_per_icp: u64, fetched_at_secs: u64) -> RateCache {
        RateCache {
            v: 1,
            xdr_permyriad_per_icp,
            fetched_at_secs,
        }
    }

    #[test]
    fn t5_2_cycles_to_e8s_round_trips_exactly_on_multiples() {
        let r = 37_300u64;
        let e8s = 1_000_000u64;
        let cycles = e8s_to_cycles(e8s, r);
        assert_eq!(cycles_to_e8s(cycles, r).unwrap(), e8s);
    }

    #[test]
    fn t5_2_cycles_to_e8s_rounds_up_never_under_quotes() {
        let r = 37_300u64;
        let cycles = e8s_to_cycles(1, r) + 1;
        let e8s = cycles_to_e8s(cycles, r).unwrap();
        assert!(e8s_to_cycles(e8s, r) >= cycles);
        assert_eq!(e8s, 2);
    }

    #[test]
    fn t5_2_cycles_to_e8s_errors_on_zero_rate() {
        assert!(matches!(
            cycles_to_e8s(1_000, 0),
            Err(ApiError::Internal(_))
        ));
    }

    #[test]
    fn t5_2_cycles_to_e8s_errors_on_overflow() {
        assert!(matches!(
            cycles_to_e8s(u128::MAX, 1),
            Err(ApiError::InvalidInput(_))
        ));
    }

    #[test]
    fn t5_2_quote_topup_computes_total_and_age() {
        let r = rate(37_300, 1_000);
        let q = quote_topup(1_000_000_000_000, &r, 1_500, 10_000).unwrap();
        assert_eq!(q.cycles, 1_000_000_000_000);
        assert_eq!(
            q.deposit_e8s,
            cycles_to_e8s(1_000_000_000_000, 37_300).unwrap()
        );
        assert_eq!(q.total_e8s, q.deposit_e8s + 10_000);
        assert_eq!(q.rate_age_secs, 500);
    }

    #[test]
    fn t5_2_quote_spawn_applies_buffer_and_creation_fee() {
        let params = Params::default();
        let r = rate(37_300, 0);
        let q = quote_spawn(&params, &r, 0).unwrap();
        let base = params.aaa_initial_cycles + params.spawn_creation_fee_cycles;
        let buffered = base * (10_000 + params.spawn_quote_buffer_bp as u128) / 10_000;
        assert_eq!(q.cycles, buffered);
        assert_eq!(q.fee_e8s, params.icp_ledger_fee_e8s);
        assert!(e8s_to_cycles(q.deposit_e8s, 37_300) >= buffered);
    }

    #[test]
    fn t5_2_quote_spawn_propagates_stale_or_missing_rate_error() {
        let params = Params::default();
        let r = rate(0, 0);
        assert!(quote_spawn(&params, &r, 0).is_err());
    }
}
