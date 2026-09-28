use std::time::Duration;

use candid::Principal;
use ic_cdk::call::Call;

use crate::rng;

const HOURLY: Duration = Duration::from_secs(3_600);
const RETRY: Duration = Duration::from_secs(30);

pub fn start() {
    ic_cdk_timers::set_timer(Duration::ZERO, reseed());
    ic_cdk_timers::set_timer(Duration::ZERO, sample_cycles());
    ic_cdk_timers::set_timer_interval(HOURLY, reseed);
    ic_cdk_timers::set_timer_interval(HOURLY, starvation_sweep);
    ic_cdk_timers::set_timer_interval(HOURLY, sample_cycles);
    ic_cdk_timers::set_timer_interval(HOURLY, prune_leases);
    if crate::progression::replay_active() {
        schedule_replay_continue();
    }
}

async fn sample_cycles() {
    crate::metrics::sample(ic_cdk::api::time(), ic_cdk::api::canister_cycle_balance());
}

const LEASE_PRUNE_BATCH: usize = 5_000;

async fn prune_leases() {
    crate::catalog::prune_leases(ic_cdk::api::time(), LEASE_PRUNE_BATCH);
}

async fn starvation_sweep() {
    let cfg = crate::config::get();
    crate::reviews::apply_starvation(
        &crate::registry::reviewer_candidates(),
        &cfg.params,
        cfg.current_protocol_version,
        ic_cdk::api::time(),
    );
}

async fn raw_rand() -> Option<[u8; 32]> {
    let reply = Call::bounded_wait(Principal::management_canister(), "raw_rand")
        .with_args(&())
        .await
        .ok()?;
    let bytes: Vec<u8> = reply.candid().ok()?;
    bytes.as_slice().try_into().ok()
}

async fn reseed() {
    match raw_rand().await {
        Some(seed) => rng::seed(seed, ic_cdk::api::time()),
        None => {
            ic_cdk_timers::set_timer(RETRY, reseed());
        }
    }
}

pub fn schedule_replay_continue() {
    ic_cdk_timers::set_timer(Duration::ZERO, continue_replay());
}

async fn continue_replay() {
    if !crate::progression::replay_step().done {
        schedule_replay_continue();
    }
}
