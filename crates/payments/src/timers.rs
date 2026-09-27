use std::time::Duration;

use crate::journal;
use crate::rate;

const RESUME_INTERVAL: Duration = Duration::from_secs(300);
const RATE_REFRESH_INTERVAL: Duration = Duration::from_secs(3_600);
const RESUME_SWEEP_LIMIT: u32 = 50;

pub fn start() {
    ic_cdk_timers::set_timer_interval(RESUME_INTERVAL, resume_sweep);
    ic_cdk_timers::set_timer_interval(RATE_REFRESH_INTERVAL, rate_refresh_sweep);
    ic_cdk_timers::set_timer(Duration::ZERO, rate_refresh_sweep());
}

async fn resume_sweep() {
    for op in journal::list_by_created(None, RESUME_SWEEP_LIMIT) {
        if !op.state.is_terminal() {
            let _ = crate::api::resume(op.id).await;
        }
    }
}

async fn rate_refresh_sweep() {
    let _ = rate::refresh().await;
}
