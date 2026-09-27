use candid::{CandidType, Principal};
use serde::Deserialize;

use crate::config::Config;
use crate::keeper;
use crate::state::{Metric, Snapshot};

#[derive(CandidType, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct Health {
    pub reserve_breached: bool,
    pub min_runway_days: u32,
    pub worst_canister: Option<Principal>,
}

#[derive(CandidType, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct CanisterRunway {
    pub canister: Principal,
    pub priority: u8,
    pub cycles: u128,
    pub burn_per_day: u128,
    pub runway_days: u32,
}

#[derive(CandidType, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct Status {
    pub icp_balance_e8s: u64,
    pub reserve_e8s: u64,
    pub xdr_permyriad_per_icp: u64,
    pub daily_burn_cycles: u128,
    pub canisters: Vec<CanisterRunway>,
    pub projected_runway_months: u32,
    pub checked_at: u64,
}

pub fn runways(watched: &[(Principal, u8, Option<Metric>)]) -> Vec<CanisterRunway> {
    watched
        .iter()
        .map(|(canister, priority, m)| {
            let m = m.clone().unwrap_or_default();
            CanisterRunway {
                canister: *canister,
                priority: *priority,
                cycles: m.balance,
                burn_per_day: m.burn_per_day,
                runway_days: keeper::runway_days(m.balance, m.burn_per_day),
            }
        })
        .collect()
}

pub fn health(config: &Config, snap: &Snapshot, runways: &[CanisterRunway]) -> Health {
    let worst = runways
        .iter()
        .filter(|r| r.burn_per_day > 0)
        .min_by_key(|r| r.runway_days);
    let min_runway_days = worst.map(|r| r.runway_days).unwrap_or(u32::MAX);
    let reserve_breached = snap.checked_at > 0
        && (snap.icp_balance_e8s <= config.reserve_e8s
            || snap.reserve_blocked
            || min_runway_days < config.runway_alert_days);
    Health {
        reserve_breached,
        min_runway_days,
        worst_canister: worst.map(|r| r.canister),
    }
}

pub fn status(config: &Config, snap: &Snapshot, canisters: Vec<CanisterRunway>) -> Status {
    let daily_burn_cycles: u128 = canisters.iter().map(|r| r.burn_per_day).sum();
    let reserve_cycles = keeper::cycles_for_e8s(snap.icp_balance_e8s, snap.xdr_permyriad_per_icp);
    let projected_runway_months = reserve_cycles
        .checked_div(daily_burn_cycles)
        .map(|days| u32::try_from(days / 30).unwrap_or(u32::MAX))
        .unwrap_or(u32::MAX);
    Status {
        icp_balance_e8s: snap.icp_balance_e8s,
        reserve_e8s: config.reserve_e8s,
        xdr_permyriad_per_icp: snap.xdr_permyriad_per_icp,
        daily_burn_cycles,
        canisters,
        projected_runway_months,
        checked_at: snap.checked_at,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const T: u128 = 1_000_000_000_000;

    fn p(n: u8) -> Principal {
        Principal::from_slice(&[n; 29])
    }

    fn snap(icp: u64) -> Snapshot {
        Snapshot {
            v: 1,
            icp_balance_e8s: icp,
            xdr_permyriad_per_icp: 30_000,
            checked_at: 1,
            reserve_blocked: false,
            inflight: None,
        }
    }

    fn metric(balance: u128, burn: u128) -> Option<Metric> {
        Some(Metric {
            v: 1,
            balance,
            at: 1,
            ema: burn,
            burn_per_day: burn,
        })
    }

    #[test]
    fn t5_17_health_flags_reserve_and_low_runway() {
        let config = Config::default();
        let rw = runways(&[
            (p(1), 0, metric(60 * T, T)),
            (p(2), 1, metric(10 * T, T)),
            (p(3), 2, None),
        ]);
        let h = health(&config, &snap(100 * 100_000_000), &rw);
        assert_eq!(
            h,
            Health {
                reserve_breached: true,
                min_runway_days: 10,
                worst_canister: Some(p(2))
            }
        );
        let healthy = runways(&[(p(1), 0, metric(60 * T, T))]);
        assert!(!health(&config, &snap(100 * 100_000_000), &healthy).reserve_breached);
        assert!(health(&config, &snap(config.reserve_e8s), &healthy).reserve_breached);
        let blocked = Snapshot {
            reserve_blocked: true,
            ..snap(100 * 100_000_000)
        };
        assert!(health(&config, &blocked, &healthy).reserve_breached);
        assert!(!health(&config, &Snapshot::default(), &[]).reserve_breached);
    }

    #[test]
    fn t5_17_status_projects_total_runway() {
        let config = Config::default();
        let s = status(
            &config,
            &snap(10 * 100_000_000),
            runways(&[(p(1), 0, metric(60 * T, T))]),
        );
        assert_eq!(s.daily_burn_cycles, T);
        assert_eq!(s.projected_runway_months, 1);
        assert_eq!(s.canisters[0].runway_days, 60);
        let idle = status(&config, &snap(1), vec![]);
        assert_eq!(idle.projected_runway_months, u32::MAX);
    }
}
