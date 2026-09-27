use std::time::Duration;

use candid::Principal;
use ic_cdk::call::Call;

use crate::rng;

const HOURLY: Duration = Duration::from_secs(3_600);
const RETRY: Duration = Duration::from_secs(30);

pub fn start() {
    ic_cdk_timers::set_timer(Duration::ZERO, reseed());
    ic_cdk_timers::set_timer_interval(HOURLY, reseed);
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

pub fn schedule_replay_continue(from_event_id: u64, batch: u32) {
    ic_cdk_timers::set_timer(Duration::ZERO, continue_replay(from_event_id, batch));
}

async fn continue_replay(from_event_id: u64, batch: u32) {
    let status = crate::progression::replay(from_event_id, batch);
    if !status.done {
        schedule_replay_continue(status.next_event_id, batch);
    }
}
