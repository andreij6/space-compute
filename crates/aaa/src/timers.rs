use std::time::Duration;

use candid::CandidType;
use ic_cdk::call::Call;
use serde::Serialize;

use crate::burn;
use crate::config;
use crate::credits::{CreditPage, ListAaaCreditsArgs};
use crate::params::{self, PlatformParams};
use crate::repository;

const BURN_TICK: Duration = Duration::from_secs(6 * 3_600);
const DAILY_TICK: Duration = Duration::from_secs(24 * 3_600);

#[derive(CandidType, Serialize)]
struct Heartbeat {
    cycles: u128,
    wasm_version: u32,
}

pub fn start() {
    ic_cdk_timers::set_timer_interval(BURN_TICK, burn_tick);
    ic_cdk_timers::set_timer_interval(DAILY_TICK, daily_tick);
}

async fn burn_tick() {
    let balance = ic_cdk::api::canister_cycle_balance();
    let now = ic_cdk::api::time();
    repository::update_stats(|s| {
        s.burn_ema_daily = burn::tick_burn_sample(
            s.last_balance_sample,
            s.last_sample_at,
            balance,
            now,
            s.burn_ema_daily,
        );
        s.last_balance_sample = Some(balance);
        s.last_sample_at = Some(now);
    });

    let cfg = config::get();
    if let Some(threshold) = cfg.auto_topup {
        if balance < threshold {
            let result = Call::bounded_wait(cfg.payments_id, "request_auto_topup")
                .with_args(&())
                .change_timeout(10)
                .await;
            if result.is_err() {
                repository::update_stats(|s| {
                    s.auto_topup_failures = s.auto_topup_failures.saturating_add(1)
                });
            }
        }
    }
}

async fn daily_tick() {
    let cfg = config::get();

    let balance = ic_cdk::api::canister_cycle_balance();
    let wasm_version = cfg.wasm_version.parse::<u32>().unwrap_or(0);
    let heartbeat = Call::bounded_wait(cfg.platform_id, "heartbeat")
        .with_arg(&Heartbeat {
            cycles: balance,
            wasm_version,
        })
        .change_timeout(10)
        .await;
    if heartbeat.is_ok() {
        repository::update_stats(|s| s.last_heartbeat_at = ic_cdk::api::time());
    }

    if let Ok(reply) = Call::bounded_wait(cfg.platform_id, "get_params")
        .with_args(&())
        .change_timeout(10)
        .await
    {
        if let Ok(new_params) = reply.candid::<PlatformParams>() {
            params::apply_platform_params(new_params, ic_cdk::api::time());
        }
    }

    sync_credits(cfg.platform_id).await;
}

async fn sync_credits(platform_id: candid::Principal) {
    for _ in 0..crate::credits::MAX_PAGES_PER_TICK {
        let cursor = repository::get_stats().credits_cursor;
        let Ok(reply) = Call::bounded_wait(platform_id, "list_aaa_credits")
            .with_arg(&ListAaaCreditsArgs {
                aaa: ic_cdk::api::canister_self(),
                cursor,
            })
            .change_timeout(10)
            .await
        else {
            return;
        };
        let Ok(page) = reply.candid::<CreditPage>() else {
            return;
        };
        let (next_cursor, more) = crate::credits::apply_credit_page(page, cursor);
        repository::update_stats(|s| s.credits_cursor = next_cursor);
        if !more {
            return;
        }
    }
}
