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
